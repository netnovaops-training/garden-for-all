//! The plant admin's AI: `ask_claude_ai`, `fetch_location_context`,
//! `fetch_perenual_data` and the companion-plant picker from backend.py and
//! tab_ai_safe_fill.py — answered by FREE cloud models only.
//!
//! HOW TO READ THIS FILE (for Jun). `ask` sends the Streamlit app's own
//! prompt (the 28 field rules, Clifton Park's zone, the current values, the
//! exact JSON template) to a free model and applies the app's own safety
//! rules to the answer: values a person already entered are kept unless the
//! tab allows proposals. `chat` is the free-model chain, the same one
//! garden-ingest uses (freeai.py): Google Gemini's free tier first, then
//! OpenRouter's free models; a model that just said "busy" is rested for a
//! few minutes. When every one is busy the tab says so; nothing is queued
//! here (the app's tag queue is in garden-ingest).
//!
//! Settings: JUNSGARDEN_GEMINI_API_KEY (the FREE key), JUNSGARDEN_OPENROUTER_API_KEY,
//! JUNSGARDEN_PERENUAL_KEY. Never the PC's GPU, never a paid key (Jason,
//! 2026-10-07). The Streamlit app used Ollama on the PC, or Anthropic.
//!
//! RUST WORDS USED HERE. `async fn` can pause while it waits for the network; callers write
//! `.await`. `Result<T, E>` is `Ok(value)` or `Err(error)`. `Option<T>` is `Some(value)` or `None`.
//! `let Some(x) = ... else { ... }` stops early when the pattern does not match. `?` after a call
//! means "if this failed, return the failure from this function now". `static` is one value that
//! lives for the whole run. A `Mutex` lets only one piece of code at a time use a value.

// `HashMap` is a lookup table from a key to a value.
use std::collections::HashMap;
// A `Mutex` guards a value that several requests may touch at once; code must `lock()` it first.
use std::sync::Mutex;
// `Duration` is a length of time and `Instant` is a moment on the server's clock.
use std::time::{Duration, Instant};

// `Map` is a JSON object and `Value` is any JSON value.
use serde_json::{Map, Value};

// `text` turns a JSON value into plain text.
use super::opts::{self, text};
use crate::AppState;

/// Google Gemini's chat address. It speaks the same message format as OpenAI, so one code path
/// serves both providers.
const GEMINI_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions";
/// OpenRouter's chat address (it offers some models for free).
const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
/// The standing instruction sent with every plant question: answer with JSON only.
const SYSTEM: &str = "You are a strict botanical data-API. Your ONLY output must be a valid JSON object. Do not include markdown formatting or explanations.";

/// Remembers which models said "busy" and until when. The outer `Option` starts as `None` and the
/// table is created on first use. A `Mutex` is needed because many requests can run at the same
/// time.
static RESTING: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);

/// Reads a setting (environment variable) as trimmed text; empty if it is not set. Keys are never
/// written in the code, only read here.
fn env(k: &str) -> String {
    std::env::var(k).unwrap_or_default().trim().to_string()
}

/// The list of free models to try, in order. Each item is (chat address, key, model name). A
/// provider is included only if its key is set. Model names can be replaced by
/// JUNSGARDEN_GEMINI_MODELS (comma separated); otherwise two default Gemini models are used.
/// `&'static str` is fixed text built into the program.
fn providers() -> Vec<(&'static str, String, String)> {
    // The list being built.
    let mut v = Vec::new();
    let g = env("JUNSGARDEN_GEMINI_API_KEY");
    if !g.is_empty() {
        // The model names: the ones from the setting, or (through `chain`) the two defaults when
        // the setting is empty.
        for m in env("JUNSGARDEN_GEMINI_MODELS").split(',').map(str::trim).filter(|m| !m.is_empty()).chain(
            if env("JUNSGARDEN_GEMINI_MODELS").is_empty() { vec!["gemini-flash-latest", "gemini-flash-lite-latest"] } else { vec![] }) {
            v.push((GEMINI_URL, g.clone(), m.to_string()));
        }
    }
    // OpenRouter's free models are tried after Gemini.
    let o = env("JUNSGARDEN_OPENROUTER_API_KEY");
    if !o.is_empty() {
        for m in ["google/gemma-4-31b-it:free", "google/gemma-4-26b-a4b-it:free"] {
            v.push((OPENROUTER_URL, o.clone(), m.to_string()));
        }
    }
    v
}

/// One answer from the first free model with room, or why there was none.
/// Sends one question to the first free model that is not resting and returns its answer text.
/// `system` is the standing instruction, `prompt` the question, `max_tokens` the longest answer
/// allowed. Returns `Err(text)` with the reasons if no model could answer.
pub async fn chat(state: &AppState, system: &str, prompt: &str, max_tokens: u32) -> Result<String, String> {
    // Get the list of models; with no keys at all, say so.
    let ps = providers();
    if ps.is_empty() {
        return Err("No free AI key is set (JUNSGARDEN_GEMINI_API_KEY / JUNSGARDEN_OPENROUTER_API_KEY).".into());
    }
    // Why each model failed, for the final error message.
    let mut reasons = Vec::new();
    // Try the models in order.
    for (url, key, model) in ps {
        // Look in the shared table: is this model resting until a time that has not come yet?
        // `lock()` can fail, so it is handled as "not resting".
        let resting = RESTING.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&model).copied())).map(|t| t > Instant::now()).unwrap_or(false);
        // Skip a model that is resting.
        if resting {
            reasons.push(format!("{model}: resting"));
            continue;
        }
        // The request in the OpenAI chat format. Temperature 0.2 keeps answers steady and not
        // creative.
        let body = serde_json::json!({
            "model": model, "temperature": 0.2, "max_tokens": max_tokens,
            "messages": [{"role": "system", "content": system}, {"role": "user", "content": prompt}],
        });
        // Build the HTTP request. The key is sent as a bearer token. The 90 second limit stops a
        // stuck model from hanging the page.
        let mut rq = state.http.post(url).bearer_auth(&key).json(&body).timeout(Duration::from_secs(90));
        // OpenRouter asks callers to say which site they are.
        if url == OPENROUTER_URL {
            rq = rq.header("HTTP-Referer", "https://junsgarden.zapto.org").header("X-Title", "Jun's Garden");
        }
        // A closure that puts a model to rest for 300 seconds (5 minutes).
        let rest = |m: &str| {
            if let Ok(mut g) = RESTING.lock() {
                g.get_or_insert_with(HashMap::new).insert(m.to_string(), Instant::now() + Duration::from_secs(300));
            }
        };
        // Send the request and look at the outcome.
        match rq.send().await {
            // Success (a 200-range status): read the JSON and take the answer text from
            // `choices[0].message.content`.
            Ok(r) if r.status().is_success() => {
                let v: Value = r.json().await.unwrap_or(Value::Null);
                let t = v.pointer("/choices/0/message/content").and_then(Value::as_str).unwrap_or("").to_string();
                if !t.trim().is_empty() {
                    tracing::info!("orchestrator AI: {model} answered");
                    return Ok(t);
                }
                reasons.push(format!("{model}: empty answer"));
            }
            // The server answered with an error. Rate limits (429) and server errors (500-504) put
            // the model to rest so it is not tried again for a while.
            Ok(r) => {
                let code = r.status().as_u16();
                reasons.push(format!("{model}: {code}"));
                if matches!(code, 429 | 500 | 502 | 503 | 504) {
                    rest(&model);
                }
            }
            // No answer at all (timeout or network problem). Rest the model too.
            Err(e) => {
                reasons.push(format!("{model}: {}", if e.is_timeout() { "timeout".to_string() } else { "unreachable".to_string() }));
                rest(&model);
            }
        }
    }
    // Every model failed.
    Err(format!("The free AI readers are all busy or at today's free limit ({}). Try again later.", reasons.join("; ")))
}

/// `fetch_location_context`: two DuckDuckGo result snippets about growing
/// the plant in Clifton Park.
/// Searches DuckDuckGo's plain HTML page and returns about two short result snippets about growing
/// this plant near Clifton Park, New York. Used to give the AI local facts. Returns empty text on
/// any problem.
pub async fn location_context(state: &AppState, plant: &str) -> String {
    // The search words, built from the plant name.
    let q = format!("{plant} growing season bloom time Clifton Park New York zone 5b 6a annual perennial evergreen invasive scent fruit lifespan reproduction root system");
    // Post the search. If it fails, return empty text.
    let Ok(r) = state.http.post("https://html.duckduckgo.com/html/").header("User-Agent", "Mozilla/5.0")
        .form(&[("q", q.as_str())]).timeout(Duration::from_secs(15)).send().await else { return String::new() };
    let html = r.text().await.unwrap_or_default();
    let mut out = Vec::new();
    // Each result's text sits after the marker `class="result__snippet"`. Skip the first piece
    // (before any result) and take two.
    for part in html.split("class=\"result__snippet\"").skip(1).take(2) {
        // Cut off the rest of the tag; what follows is the snippet.
        let body = part.split_once('>').map(|(_, b)| b).unwrap_or("");
        // The snippet ends at the closing `</a>`.
        let snippet = body.split("</a>").next().unwrap_or("");
        out.push(strip_tags(snippet));
    }
    out.join(" ")
}

/// Removes HTML tags from `s`: text between `<` and `>` is dropped. Then HTML codes such as &amp;
/// are turned back into characters.
fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    // True while we are inside a tag.
    let mut tag = false;
    // Look at each character: `<` starts a tag, `>` ends it, other characters are kept only outside
    // a tag.
    for c in s.chars() {
        match c {
            '<' => tag = true,
            '>' => tag = false,
            _ if !tag => out.push(c),
            _ => {}
        }
    }
    // Turn codes like &amp; back into characters; &#x27; and &nbsp; are handled by hand.
    crate::php::htmlspecialchars_decode(&out).replace("&#x27;", "'").replace("&nbsp;", " ").trim().to_string()
}

/// `fetch_perenual_data`: the Perenual species record as JSON text, or "".
/// Asks Perenual (a plant database) first for the species matching `plant`, then for that species'
/// details. Returns the details as JSON text, or empty text if no key is set or anything fails.
pub async fn perenual(state: &AppState, plant: &str) -> String {
    // No key means no lookup.
    let key = env("JUNSGARDEN_PERENUAL_KEY");
    if key.is_empty() {
        return String::new();
    }
    // Step 1: search the species list.
    let Ok(r) = state.http.get("https://perenual.com/api/species-list").query(&[("key", key.as_str()), ("q", plant)]).timeout(Duration::from_secs(10)).send().await else { return String::new() };
    let Ok(v) = r.json::<Value>().await else { return String::new() };
    // The first result's id, read with a JSON pointer (`/data/0/id`).
    let Some(id) = v.pointer("/data/0/id").and_then(Value::as_i64) else { return String::new() };
    // Step 2: fetch the details for that id.
    let Ok(r) = state.http.get(format!("https://perenual.com/api/species/details/{id}")).query(&[("key", key.as_str())]).timeout(Duration::from_secs(10)).send().await else { return String::new() };
    r.text().await.unwrap_or_default()
}

/// The 57 plant fields sent to the AI as current data, and checked in the answer. Months
/// (grow_months, bloom_months) are handled apart.
const FIELDS: [&str; 57] = [
    "company", "family", "genus", "botanical_name", "species", "plant_type", "sun_exposure", "cleanup_notes",
    "care_notes", "cuttings", "fertilizer", "maintenance_notes", "attracts", "zones", "water_needs", "deer_resistance",
    "soil_ph", "soil_type", "toxicity", "bloom_color", "foliage_color", "stem_color", "disease_susceptibility", "propagation_method",
    "plant_length", "plant_width", "spacing", "average_cost", "extra_details", "image_search_term", "leaf_shape", "leaf_type", "leaf_margin",
    "flower_symmetry", "flower_shape", "flower_size", "fruit_type", "fruit_shape", "fruit_color", "fruit_size", "fruit_edibility",
    "invasive_local", "native_status", "growth_rate", "planting_depth", "chilling_hours", "companion_plants", "garden_beds", "aromatics",
    "is_evergreen", "quantity", "year_added", "life_expectancy", "reproduction_type", "root_system", "root_morphology", "root_function",
];

/// The 28 field rules from the Streamlit app, sent word for word to the model. They fix the allowed
/// choices for each drop-down. The text ends with "CURRENT DATA:", after which the current values
/// are added. Do not change the wording without a reason: it matches the original app.
const RULES: &str = r#"
    AI INFERENCING INSTRUCTIONS:
    You MUST make highly educated assumptions using your vast botanical knowledge. If a field is blank, infer the correct value based on the plant species. DO NOT leave fields blank if you can deduce a logical answer!

    1. 'plant_type': "Perennial", "Annual", "Biennial", "Shrub", "Tree", "Succulent", "Vine". IF UNSURE, DEFAULT TO "Perennial".
    2. 'grow_months' & 'bloom_months': Array of integers (1-12).
    3. 'sun_exposure': "Full Sun", "Partial", "Shade", "Full Sun - Partial", "Full Shade", "Full Sun - Full Shade", "Partial Shade - Full Shade", "Full Shade - Partial Shade".
    4. 'deer_resistance': Interpret existing data and map to ONLY: "Deer Resistant", "Rabbit Resistant", "Both", "Not Resistant".
    5. 'fertilizer': Output an ARRAY of matching products (e.g., ["Rose-tone", "Bloom Booster"]). Map free-form text to the nearest logical -tone product.
    6. 'leaf_shape': "Linear (Long, narrow)", "Lanceolate (Long tapering tip)", "Ovate (Egg-shaped)", "Elliptic (Oval tapering ends)", "Cordate (Heart-shaped)", "Palmate (Hand-like lobes)", "Deltoid (Triangular)".
    7. 'leaf_type': "Simple (Single undivided blade)", "Pinnately Compound (Feather-like leaflets)", "Palmately Compound (Radiating leaflets)".
    8. 'leaf_margin': "Entire (Smooth, no teeth)", "Serrated (Saw-toothed edges)", "Shallowly Serrate", "Lobed (Deep indentations)".
    9. 'flower_symmetry': "Actinomorphic (Radial symmetry)", "Zygomorphic (Bilateral symmetry)".
    10. 'flower_shape': "Rotate (Flat wheel-shaped)", "Stellate (Star-shaped)", "Cruciate (Cross-shaped)", "Campanulate (Bell-shaped)", "Infundibuliform (Funnel-shaped)", "Tubular (Cylindrical tube)", "Salverform (Tube opening flat)", "Bilabiate (Two-lipped)", "Urceolate (Urn-shaped)", "High-centered", "Double Blossoms".
    11. 'flower_size': "Tiny (<1 in)", "Small (1-2 in)", "Medium (2-4 in)", "Large (4-6 in)", "Massive (>6 in)".
    12. 'plant_length': Target mature length or height (e.g., '3-4 feet').
    13. 'plant_width': Target mature width or spread (e.g., '2-3 feet').
    14. 'growth_rate': Make an assumption: "Slow", "Moderate", "Fast".
    15. 'invasive_local': Make an assumption for NY: "Yes", "No", "Watchlist".
    16. 'native_status': Make an assumption for NY: "Native", "Non-native", "Introduced", "Cultivar", or "Unknown".
    17. 'aromatics': Use 1-3 words to deduce scent (e.g., 'Sweet', 'Lemony', 'Minty', 'Foul', or 'None').
    18. FRUIT/BERRY TRAITS (Strictly 1-2 words each!):
        - IF NO FRUIT/BERRIES: explicitly set all fruit fields below to "None".
        - 'fruit_type': (e.g., 'Berry', 'Capsule', 'Pod', 'None')
        - 'fruit_shape': (e.g., 'Round', 'Oval', 'Oblong', 'None')
        - 'fruit_color': (e.g., 'Red', 'Dark Purple', 'None')
        - 'fruit_size': (e.g., 'Small', 'Large', '1 inch', 'None')
        - 'fruit_edibility': MUST be "Edible", "Inedible", "Toxic", "Wildlife Only", or "None".
    19. 'planting_depth': Lookup instructions (e.g., 'Crown at soil level', '1/4 inch deep').
    20. 'chilling_hours': If fruit/tree, estimate required chill hours. Otherwise leave blank.
    21. 'companion_plants': Deduce 2-3 compatible plants for Clifton Park. Output as a comma-separated string list.
    22. 'toxicity': Make an assumption. MUST be one of: "Non-toxic", "Toxic", "Toxic to Dogs/Cats", "Toxic to Humans", "Toxic to Humans & Pets", or "Unknown".
    23. 'is_evergreen': Deduce foliage retention. Output an ARRAY of tags (e.g., ["Evergreen"], ["Semi-evergreen"], ["Deciduous"]).
    24. 'life_expectancy': Deduce the general lifespan or longevity of the plant (e.g., '1-2 years', '10-15 years', '50+ years').
    25. 'reproduction_type': Deduce reproduction. MUST be one of: "Hermaphrodite", "Monoecious", "Dioecious", "Polygamous", or "Unknown".
    26. 'root_system': MUST be one of: "Taproot Systems", "Fibrous/Adventitious Systems", "Specialized Systems", or "Unknown".
    27. 'root_morphology': MUST be one of: "Fine Roots (< 2 mm)", "Coarse Roots (> 2 mm)", "Mixed Fine & Coarse", or "Unknown".
    28. 'root_function': Describe functional root order (e.g., 'Absorptive Roots (Lower Orders)', 'Transport Roots (Higher Orders)', 'Mixed Orders').

    CRITICAL RULE: DO NOT OVERWRITE EXISTING DATA UNLESS YOU ARE FIXING FORMATTING TO MATCH STRICT LISTS.
    CURRENT DATA:
    "#;

/// The exact JSON shape the AI must answer with. It is shown to the model at the end of the prompt.
const TEMPLATE: &str = r#"
    Format EXACTLY like this template:
    {
        "company": "", "botanical_name": "", "family": "", "genus": "", "species": "", "plant_type": "", "sun_exposure": "",
        "cleanup_notes": "", "care_notes": "", "cuttings": "", "fertilizer": [], "maintenance_notes": "",
        "attracts": "", "zones": "", "water_needs": "", "deer_resistance": "", "soil_ph": "", "soil_type": "",
        "toxicity": "", "bloom_color": "", "foliage_color": "", "stem_color": "", "disease_susceptibility": "", "propagation_method": "",
        "plant_length": "", "plant_width": "", "spacing": "", "average_cost": 0.00, "extra_details": "", "image_search_term": "", "grow_months": [], "bloom_months": [],
        "leaf_shape": "", "leaf_type": "", "leaf_margin": "", "flower_symmetry": "", "flower_shape": "", "flower_size": "",
        "fruit_type": "", "fruit_shape": "", "fruit_color": "", "fruit_size": "", "fruit_edibility": "",
        "invasive_local": "", "native_status": "", "growth_rate": "", "planting_depth": "", "chilling_hours": "", "companion_plants": "",
        "garden_beds": [], "aromatics": "", "is_evergreen": [], "quantity": 1, "year_added": [], "life_expectancy": "",
        "reproduction_type": "", "root_system": "", "root_morphology": "", "root_function": ""
    }
    "#;

/// True when a value counts as "nothing yet": empty, 0, unknown, none or "[]". Values like these
/// may be replaced by the AI; others are kept.
fn is_empty(v: Option<&Value>) -> bool {
    matches!(text(v).trim().to_lowercase().as_str(), "" | "0.00" | "0.0" | "0" | "unknown" | "none" | "[]" | "unknown only")
}

/// A list the AI sent (["a","b"]) as the comma text the database stores.
/// Fields such as fertilizer arrive as a list. This joins them with ", ", skipping empty items.
/// Other values are returned unchanged.
fn flat(v: &Value) -> Value {
    match v {
        Value::Array(a) => Value::String(a.iter().map(|x| text(Some(x))).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ")),
        other => other.clone(),
    }
}

/// Reads months from either an AI list (numbers or text numbers) or a 12-letter code. Only 1 to 12
/// are kept. Anything else gives no months.
fn months(v: Option<&Value>) -> Vec<u32> {
    match v {
        // A list: accept numbers and number-like text; drop anything else.
        Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.trim().parse().ok()))).map(|m| m as u32).filter(|m| (1..=12).contains(m)).collect(),
        // Text: read it as a month code.
        Some(Value::String(s)) => opts::parse_m(s),
        _ => Vec::new(),
    }
}

/// What goes wrong when asking: a message, and the raw answer text (if any) so the person can see
/// it.
pub struct AiError {
    /// A short message for the screen.
    pub error: String,
    /// What the AI actually wrote, or "None".
    pub raw: String,
}

/// `ask_claude_ai`. The answer is a field map in the database's own shapes
/// (lists joined with ", ", months as the 12-letter code), ready for the
/// editor.
/// `existing` is the plant's current values, `loc` the web snippets, `peren` the Perenual text, and
/// `allow_proposals` says whether the AI may suggest changes to values already filled in. On
/// success the map holds the AI's values; on failure an `AiError`.
pub async fn ask(state: &AppState, existing: &Map<String, Value>, loc: &str, peren: &str, allow_proposals: bool) -> Result<Map<String, Value>, AiError> {
    // The plant name for the prompt, or "Unknown Plant".
    let name = { let n = text(existing.get("plant_name")); if n.is_empty() { "Unknown Plant".into() } else { n } };
    // Copy the current value of each of the 57 fields. Missing ones are empty text.
    let mut current = Map::new();
    for f in FIELDS {
        current.insert(f.into(), existing.get(f).cloned().unwrap_or(Value::String(String::new())));
    }
    // Build the question: name, location, Perenual facts, the rules, the current data and the
    // answer template.
    let prompt = format!(
        "\n    Provide data for '{name}'.\n    \n    LOCATION CONTEXT: Clifton Park, New York (USDA Hardiness Zone 5b/6a). \n    {loc}\n    \n    PERENUAL API FACTUAL DATA:\n    {}\n    {RULES}{}\n    {TEMPLATE}",
        if peren.is_empty() { "No Perenual API data found. Rely on internal knowledge." } else { peren },
        serde_json::to_string_pretty(&Value::Object(current)).unwrap_or_default(),
    );
    // Ask the free models (2000 tokens at most). `map_err` turns the plain text error into an
    // `AiError`; `?` returns it at once on failure.
    let raw = chat(state, SYSTEM, &prompt, 2000).await.map_err(|e| AiError { error: e, raw: "None".into() })?;
    // The answer may have extra words around the JSON. Find the first `{` and the last `}`. If
    // either is missing, report that the answer was not JSON.
    let (Some(a), Some(b)) = (raw.find('{'), raw.rfind('}')) else {
        return Err(AiError { error: "AI did not output a recognizable JSON object.".into(), raw });
    };
    if a > b {
        return Err(AiError { error: "AI did not output a recognizable JSON object.".into(), raw });
    }
    // Models often write `,` before a closing bracket, which JSON forbids. Copy the text, leaving
    // out any comma whose next non-space character is `}` or `]`.
    // Trailing commas, which models love and JSON forbids.
    let mut cleaned = String::new();
    let body: Vec<char> = raw[a..=b].chars().collect();
    let mut i = 0;
    while i < body.len() {
        if body[i] == ',' {
            let mut j = i + 1;
            while j < body.len() && body[j].is_whitespace() {
                j += 1;
            }
            if j < body.len() && (body[j] == '}' || body[j] == ']') {
                i += 1;
                continue;
            }
        }
        cleaned.push(body[i]);
        i += 1;
    }
    // Parse the cleaned text as a JSON object.
    let Ok(Value::Object(mut data)) = serde_json::from_str::<Value>(&cleaned) else {
        return Err(AiError { error: "Execution Error: the answer was not valid JSON".into(), raw });
    };
    // Force the cost to be a number (0 if it is not).
    let cost = text(data.get("average_cost")).trim().parse::<f64>().unwrap_or(0.0);
    data.insert("average_cost".into(), serde_json::json!(cost));
    // Force the quantity to be a whole number (1 if it is not).
    let qty = text(data.get("quantity")).trim().parse::<f64>().map(|q| q as i64).unwrap_or(1);
    data.insert("quantity".into(), serde_json::json!(qty));
    // Safety rule: if the plant already has a real value, keep it. Only when proposals are allowed
    // and the AI gave a real value may it replace it. List fields are handled below.
    for f in FIELDS {
        if matches!(f, "fertilizer" | "garden_beds" | "companion_plants" | "year_added" | "is_evergreen") {
            continue;
        }
        if !is_empty(existing.get(f)) && (!allow_proposals || is_empty(data.get(f))) {
            data.insert(f.into(), existing.get(f).cloned().unwrap_or(Value::Null));
        }
    }
    // Months: keep the existing months unless proposals are allowed and the AI gave some. Then
    // store them as a 12-letter code.
    for (k, ex) in [("grow_months", months(existing.get("grow_months"))), ("bloom_months", months(existing.get("bloom_months")))] {
        let ai = months(data.get(k));
        let pick = if !ex.is_empty() { if allow_proposals && !ai.is_empty() { ai } else { ex } } else { ai };
        data.insert(k.into(), Value::String(opts::format_m(&pick)));
    }
    // Turn list answers into comma text for the database.
    for k in ["fertilizer", "garden_beds", "companion_plants", "year_added", "is_evergreen"] {
        if let Some(v) = data.get(k).cloned() {
            data.insert(k.into(), flat(&v));
        }
    }
    Ok(data)
}

/// `_suggest_companions_from_db`: up to 8 names, only from this garden's
/// own plant list, checked case-insensitively. Free models (S-8).
/// `row` is the plant, `all` every plant name in the garden. Asks a free model for up to 8
/// companion plants and returns only names that are really in the garden's list.
pub async fn companions(state: &AppState, row: &Map<String, Value>, all: &[String]) -> Vec<String> {
    // Nothing to choose from.
    if all.is_empty() {
        return Vec::new();
    }
    // A closure that reads a field as text.
    let g = |k: &str| text(row.get(k));
    let name = g("plant_name");
    // The garden's plant names, one per line.
    let list = all.iter().map(|n| format!("- {n}")).collect::<Vec<_>>().join("\n");
    // The prompt tells the model to pick only exact names from the list, one per line, or write
    // NONE.
    let prompt = format!("You are a horticulture expert helping to populate a garden database.\n\nPLANT TO EVALUATE:\n  Name:      {name}\n  Botanical: {}\n  Family:    {}\n  Type:      {}\n  Zones:     {}\n  Sun:       {}\n  Water:     {}\n\nAVAILABLE DATABASE PLANTS (you may ONLY choose from this exact list):\n{list}\n\nTASK: Select up to 8 plants from the list above that are genuinely good companion\nplants for \"{name}\". Good companions share similar growing conditions OR\nprovide a known beneficial relationship (e.g. pest deterrence, pollinator support,\nnitrogen fixation, shade tolerance pairing).\n\nSTRICT RULES:\n- Return ONLY names that appear EXACTLY in the database list above.\n- Do NOT invent, abbreviate, or modify any name.\n- Output one plant name per line, no bullets, no numbering, no explanations.\n- If no plants from the list are suitable, output exactly: NONE",
        g("botanical_name"), g("family"), g("plant_type"), g("zones"), g("sun_exposure"), g("water_needs"));
    // Ask. A failure just means no suggestions.
    let Ok(raw) = chat(state, "You are a horticulture expert.", &prompt, 512).await else { return Vec::new() };
    // The model said NONE.
    if raw.trim().eq_ignore_ascii_case("NONE") {
        return Vec::new();
    }
    // A lookup from lower-case name to the real name, so matching ignores capitals.
    let lower: HashMap<String, &String> = all.iter().map(|n| (n.trim().to_lowercase(), n)).collect();
    // The accepted names.
    let mut out: Vec<String> = Vec::new();
    // Clean each line (bullets and spaces removed), look it up, and accept it only if it exists in
    // the garden, is not the plant itself and is not already chosen.
    for line in raw.lines() {
        let c = line.trim().trim_start_matches(['-', '•', '*', ' ']).trim().to_lowercase();
        if let Some(canon) = lower.get(&c) {
            if **canon != name && !out.contains(canon) {
                out.push((*canon).clone());
            }
        }
    }
    // Keep at most 8.
    out.truncate(8);
    out
}

/// Tests, built only for `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    /// Checks that months from an AI list and from a code work, and that month 13 is dropped.
    #[test]
    fn months_from_ai_and_codes() {
        assert_eq!(months(Some(&serde_json::json!([3, "4", 13]))), vec![3, 4]);
        assert_eq!(months(Some(&serde_json::json!("..MA........"))), vec![3, 4]);
    }

    /// Checks that HTML tags are removed and codes decoded.
    #[test]
    fn tags_are_stripped() {
        assert_eq!(strip_tags("Grows <b>well</b> &amp; fast"), "Grows well & fast");
    }
}
