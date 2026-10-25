//! `brain.php`, `fuse.php` and `discover.php` — the site's local AI.
//!
//! All three talk to an Ollama model (`OLLAMA_URL`, `OLLAMA_MODEL`; the
//! defaults point at the desktop PC as the PHP did). Every call fails soft:
//! if the model is unreachable the page shows the arithmetic it already
//! computed and says the AI half is unavailable. Nothing a model produces is
//! a number on a chart; the numbers come from the database.
//!
//! Readings are cached as files in the AI cache folder, and a page load
//! never waits for the model: only the warmer (`data.php?warm_ai=1`, called
//! by the weather-collector loop) is allowed to spend the time regenerating.
//!
//! HOW TO READ THIS FILE. It has three parts, in this order. First the shared helpers
//! (`generate` talks to the model, `available` checks it is up). Then `predict` and
//! `insights` (water-needs guess and the written garden reading). Then `fuse_ask`
//! (the AI reading of a cross-tab) and `disc_scan` / `disc_run` (find links between
//! plant columns). `disc_scan` is plain arithmetic; only the wording of the final
//! answer comes from the model.
//!
//! Rust ideas used: `Option` is "a value, or nothing". The `?` after a call means: if the
//! result is nothing or an error, stop and return nothing from this function.
//! `async fn` can pause while waiting (for the network); `.await` is where it waits.
//! `&str` is borrowed text, `String` is owned text.

// `use` brings names into scope so they can be written short.
use std::time::Duration;

// `serde_json` handles JSON. `Value` is any JSON value; `Map` is a JSON object.
use serde_json::{json, Map, Value};

// `crate::` means "from this same program".
use crate::store::SafePath;
use crate::AppState;

/// The five water-need levels the model must choose from, from driest to wettest.
/// `[&str; 5]` is a fixed list of five pieces of text.
pub const WATER_CLASSES: [&str; 5] = [
    "Low",
    "Low to Average",
    "Average",
    "Average to High",
    "High",
];
/// How long a cached insights reading counts as fresh: 3600 seconds, one hour.
const BRAIN_TTL_SECS: u64 = 3600;

/// Turns the model address (ending in `/api/generate`) into its base address by cutting
/// everything from `/api/` onward. Used to reach other Ollama endpoints such as `/api/tags`.
fn host(url: &str) -> String {
    // `match` chooses between "found at position i" and "not found".
    match url.find("/api/") {
        Some(i) => url[..i].to_string(),
        None => url.to_string(),
    }
}

/// How many seconds ago a file was last changed. `None` if the file is missing or its
/// time cannot be read. Used to decide whether a cached answer is still fresh.
fn file_age_secs(p: &SafePath) -> Option<u64> {
    std::fs::metadata(p.as_path())
        .ok()?
        .modified()
        .ok()?
        .elapsed()
        .ok()
        .map(|d| d.as_secs())
}

/// Reads a file and parses it as JSON. Returns `None` if it is missing or not valid JSON.
fn read_json_file(p: &SafePath) -> Option<Value> {
    std::fs::read(p.as_path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
}

/// Writes JSON to a file. Errors are ignored on purpose (`let _ =`): a cache that cannot be
/// saved should not break the page.
fn write_json_file(p: &SafePath, v: &Value) {
    let _ = std::fs::write(p.as_path(), serde_json::to_string(v).unwrap_or_default());
}

/// One of the AI cache files, `name` inside `ai_cache_dir` (made first if needed), checked
/// by `store::file_in`. `None` if the folder cannot be used: then nothing is cached.
fn cache_file(state: &AppState, name: &str) -> Option<SafePath> {
    let _ = std::fs::create_dir_all(&state.ai_cache_dir);
    crate::store::file_in(&state.ai_cache_dir, name).ok()
}

/// One JSON round trip to the model. None on any failure.
// Inputs: the shared state, the prompt text, how many seconds to wait, how random the
// answer may be (0 = least), and the most pieces of output text to allow (`num_predict`).
pub async fn generate(
    state: &AppState,
    prompt: &str,
    timeout_secs: u64,
    temperature: f64,
    num_predict: u32,
) -> Option<Value> {
    // The request body for Ollama. `stream: false` asks for one complete answer.
    let body = json!({
        "model": state.ollama_model,
        "prompt": prompt,
        "stream": false,
        // `format: json` asks the model to reply in valid JSON.
        "format": "json",
        "options": { "temperature": temperature, "num_predict": num_predict },
    });
    // Send the request. Any network failure gives `None` through the `?`.
    let resp = state
        .http
        .post(&state.ollama_url)
        .timeout(Duration::from_secs(timeout_secs))
        .json(&body)
        .send()
        .await
        .ok()?;
    // A non-success status (for example 500) means no answer.
    if !resp.status().is_success() {
        tracing::warn!("brain: ollama answered {}", resp.status());
        return None;
    }
    // Ollama wraps its answer: the outer JSON has a `response` field that holds the answer as text.
    let outer: Value = resp.json().await.ok()?;
    // Take that text...
    let inner = outer.get("response")?.as_str()?;
    // ...and parse it as JSON again to get the model's own answer.
    let v: Value = serde_json::from_str(inner).ok()?;
    // Only accept it if it is a JSON object. `then_some` gives `Some(v)` when the check is true.
    v.is_object().then_some(v)
}

/// Is the model reachable AND installed? The answer is cached on disk for
/// a minute so a page load does not pay a connect timeout every time.
pub async fn available(state: &AppState) -> bool {
    // A small file remembers the last answer so the model is not asked on every page load.
    let cache = cache_file(state, "health.json");
    // If that answer is less than 60 seconds old, use it.
    if cache
        .as_ref()
        .and_then(file_age_secs)
        .is_some_and(|a| a < 60)
    {
        if let Some(v) = cache.as_ref().and_then(read_json_file) {
            return v.get("ok").and_then(Value::as_bool).unwrap_or(false);
        }
    }
    // Ask Ollama which models it has installed (`/api/tags`). Wait at most 4 seconds.
    let ok = match state
        .http
        .get(format!("{}/api/tags", host(&state.ollama_url)))
        .timeout(Duration::from_secs(4))
        .send()
        .await
    {
        // A good reply: read the list of model names and see whether ours is there.
        Ok(r) if r.status().is_success() => {
            let tags: Value = r.json().await.unwrap_or(Value::Null);
            let names: Vec<&str> = tags
                .get("models")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|m| m.get("name").and_then(Value::as_str))
                        .collect()
                })
                .unwrap_or_default();
            let installed = names.contains(&state.ollama_model.as_str());
            if !installed {
                tracing::warn!(
                    "brain: ollama is up but {} is not installed",
                    state.ollama_model
                );
            }
            installed
        }
        // No reply or an error: not available.
        _ => false,
    };
    // Save the answer, with the time, for the next minute.
    if let Some(cache) = &cache {
        write_json_file(
            cache,
            &json!({ "ok": ok, "at": common::now_local().unix_timestamp() }),
        );
    }
    ok
}

/// Predict a plant's water needs: the shape the Plant Weather Detective draws.
// `plant` is one plant's database row as a JSON object.
pub async fn predict(state: &AppState, plant: &Map<String, Value>) -> Option<Value> {
    // Build a list of short fact lines, such as "Sun: Full Sun", from these columns.
    let mut facts: Vec<String> = Vec::new();
    // Each pair is (database column, label to show the model).
    for (col, label) in [
        ("plant_name", "Name"),
        ("family", "Family"),
        ("plant_type", "Type"),
        ("sun_exposure", "Sun"),
        ("zones", "Hardiness zones"),
        ("native_status", "Origin"),
        ("is_evergreen", "Foliage"),
        ("growth_rate", "Growth rate"),
        ("root_system", "Roots"),
        ("care_notes", "Care notes"),
    ] {
        // Read the value as text. Numbers are turned into text. A missing column gives "".
        let v = plant
            .get(col)
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_default();
        let v = v.trim();
        // Skip empty values and the word "null".
        if !v.is_empty() && v != "null" {
            facts.push(format!("{label}: {v}"));
        }
    }
    // With no facts there is nothing to ask the model.
    if facts.is_empty() {
        return None;
    }
    // The five level names joined as: Low", "Low to Average", ... for use inside the prompt.
    let classes = WATER_CLASSES.join("\", \"");
    // The prompt tells the model the plant facts and asks for JSON only. Double braces are
    // literal braces inside `format!`. The scores are asked for so a confidence can be drawn.
    let prompt = format!(
        "You are a horticulturist assessing one plant in a home garden in USDA zone 6a,\nwestern New York.\n\n{}\n\nHow much water does this plant need through a normal growing season? Choose\nfrom exactly these five levels: \"{classes}\".\n\nReply with JSON only:\n{{\n  \"prediction\": \"one of the five levels\",\n  \"scores\": {{\"Low\": 0.0, \"Low to Average\": 0.0, \"Average\": 0.0, \"Average to High\": 0.0, \"High\": 0.0}},\n  \"why\": \"one sentence, under 25 words, naming the trait that decided it\"\n}}\nThe five scores must be your confidence in each level and must sum to 1.0.",
        facts.join("\n")
    );
    // Ask for a reply. 90 seconds, low randomness (0.1), up to 600 output pieces.
    let out = generate(state, &prompt, 90, 0.1, 600).await?;
    // The model must name one of the five known levels; anything else is rejected.
    let prediction = out.get("prediction")?.as_str()?.to_string();
    if !WATER_CLASSES.contains(&prediction.as_str()) {
        tracing::warn!("brain: model returned an unknown class: {prediction}");
        return None;
    }
    // Read the model's score for each level (missing or negative becomes 0), in a fixed order.
    let mut scores: Vec<(String, f64)> = WATER_CLASSES
        .iter()
        .map(|c| {
            (
                c.to_string(),
                out.get("scores")
                    .and_then(|s| s.get(*c))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
                    .max(0.0),
            )
        })
        .collect();
    // Add the scores so they can be scaled to sum to 1.
    let mut total: f64 = scores.iter().map(|(_, v)| v).sum();
    // If the model gave no usable scores, give its chosen level 1.0 and the others 0.
    if total <= 0.0 {
        for (c, v) in scores.iter_mut() {
            *v = if *c == prediction { 1.0 } else { 0.0 };
        }
        total = 1.0;
    }
    // Scale each score by the total and round to 3 places, as PHP's `round` would.
    let mut all = Map::new();
    for (c, v) in &scores {
        all.insert(c.clone(), json!(crate::php::php_round(v / total, 3)));
    }
    // The confidence is the chosen level's scaled score.
    let confidence = all.get(&prediction).cloned().unwrap_or(json!(0));
    // The reply shape the Plant Weather Detective widget expects.
    Some(json!({
        "prediction": prediction,
        "all_scores": all,
        "confidence": confidence,
        "target": "water_needs",
        "why": out.get("why").and_then(Value::as_str).unwrap_or(""),
        "model": state.ollama_model,
    }))
}

/// Reads a JSON value that should be a list of text lines. A single text becomes a list of
/// one; an array keeps its text items. Empty items are dropped, and at most `cap` are kept.
/// Used to tidy lists the model returns, because models are not always consistent.
fn str_list(v: Option<&Value>, cap: usize) -> Vec<String> {
    // Collect the candidate items as references to JSON values.
    let items: Vec<&Value> = match v {
        Some(Value::String(_)) => vec![v.unwrap()],
        Some(Value::Array(a)) => a.iter().collect(),
        _ => Vec::new(),
    };
    // Keep only text items, trim them, drop empty ones, copy them, and stop at `cap` items.
    items
        .into_iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .take(cap)
        .collect()
}

/// A written reading of the garden for the Garden Intelligence widget.
/// Served from the cache whether or not it has expired; only the warmer
/// (`allow_slow`) may regenerate it.
pub async fn insights(
    state: &AppState,
    context: &[(String, Vec<String>)],
    allow_slow: bool,
) -> Option<Value> {
    // The cache file name is an MD5 hash of the context, so the same context finds the same file.
    let mut hasher = md5::Context::new();
    hasher.consume(serde_json::to_string(context).unwrap_or_default());
    let key = cache_file(state, &format!("insights_{:x}.json", hasher.finalize()));

    // The cached reading, if it exists and is a JSON object.
    let cached = key
        .as_ref()
        .and_then(read_json_file)
        .filter(Value::is_object);
    // Fresh means younger than BRAIN_TTL_SECS.
    let fresh = cached.is_some()
        && key
            .as_ref()
            .and_then(file_age_secs)
            .is_some_and(|a| a < BRAIN_TTL_SECS);
    // Use the cache if it is fresh. Page loads (`allow_slow` false) also accept an old one,
    // marked stale, because they must never wait for the model.
    if fresh || (!allow_slow && cached.is_some()) {
        let mut c = cached.unwrap();
        c["stale"] = json!(!fresh);
        return Some(c);
    }
    // No cache and not allowed to wait: nothing to show.
    if !allow_slow {
        return None;
    }

    // Turn the context (heading plus lines) into a block of text for the prompt.
    let mut body = String::new();
    for (heading, lines) in context {
        body.push_str(&format!("\n{heading}\n"));
        for l in lines {
            body.push_str(&format!("  - {l}\n"));
        }
    }
    // Nothing to describe: do not ask.
    if body.trim().is_empty() {
        return None;
    }
    // The prompt gives the model the real numbers and tells it not to invent or recompute any.
    let prompt = format!(
        "You are advising the owner of a home garden in USDA zone 6a, western New York.\nBelow are facts already computed from their database and weather history.\nEvery number is real. Do not invent or recompute any of them.\n{body}\nGive the gardener a reading of this. Refer to the specific years, beds and\nplant groups named above. Prefer one concrete, actionable observation over\nthree vague ones. If the data does not support a conclusion, say so.\n\nReply with JSON only:\n{{\n  \"headline\": \"one sentence, under 20 words\",\n  \"findings\": [\"three to five observations, each under 30 words\"],\n  \"advice\": [\"two or three specific things to do differently next season\"]\n}}"
    );
    // 120 seconds allowed, low randomness (0.1).
    let out = generate(state, &prompt, 120, 0.1, 600).await?;
    // Keep only the parts we want and tidy them. `str_list` limits findings to 5 and advice to 3.
    let result = json!({
        "headline": out.get("headline").and_then(Value::as_str).map(str::trim).unwrap_or(""),
        "findings": str_list(out.get("findings"), 5),
        "advice": str_list(out.get("advice"), 3),
        "model": state.ollama_model,
    });
    // Save for next time and return it.
    if let Some(key) = &key {
        write_json_file(key, &result);
    }
    Some(result)
}

// --- fuse.php: the AI reading of a cross-tab --------------------------------

/// Fixed numbers for a named temperature scenario: (average F, lowest F, highest F,
/// months with frost). `None` for an unknown name. The values are the scenario's
/// assumptions given to the model, not measurements.
pub fn temp_scenario(name: &str) -> Option<(f64, f64, f64, u32)> {
    match name {
        "cold" => Some((42.0, 18.0, 65.0, 5)),
        "normal" => Some((50.0, 25.0, 78.0, 3)),
        "hot" => Some((58.0, 32.0, 92.0, 1)),
        _ => None,
    }
}

/// Fixed numbers for a named rainfall scenario: (inches for the year, average humidity
/// percent). `None` for an unknown name.
pub fn rain_scenario(name: &str) -> Option<(f64, f64)> {
    match name {
        "drought" => Some((20.0, 45.0)),
        "normal" => Some((35.0, 60.0)),
        "wet" => Some((52.0, 78.0)),
        _ => None,
    }
}

/// The badge text shown next to "AI Predictions" on the heatmap.
pub fn scenario_label(temp: Option<&str>, rain: Option<&str>) -> Option<String> {
    // Pieces of text for the badge, e.g. "Cold year" and "wet rainfall".
    let mut bits = Vec::new();
    // Only a known scenario name is used.
    if let Some(t) = temp.filter(|t| temp_scenario(t).is_some()) {
        // Make the first letter a capital: take the first character, upper-case it, add the rest.
        let mut c = t.chars();
        let cap = c
            .next()
            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
            .unwrap_or_default();
        bits.push(format!("{cap} year"));
    }
    if let Some(r) = rain.filter(|r| rain_scenario(r).is_some()) {
        bits.push(format!("{r} rainfall"));
    }
    // Join the pieces with " + ", or give `None` if there are none.
    if bits.is_empty() {
        None
    } else {
        Some(bits.join(" + "))
    }
}

/// The cross-tab as the compact text the model reads.
fn table_text(ct: &Value, max_rows: usize, max_cells: usize) -> String {
    // Row and column labels from the cross-tab JSON; a missing list becomes empty.
    let rows = ct["rowLabels"].as_array().cloned().unwrap_or_default();
    let cols = ct["colLabels"].as_array().cloned().unwrap_or_default();
    let mut lines = Vec::new();
    // Only the first `max_rows` rows are written, to keep the prompt short.
    for (ri, r) in rows.iter().enumerate().take(max_rows) {
        // For one row: list (column label, count) for columns with a count above zero.
        let mut cells: Vec<(String, i64)> = cols
            .iter()
            .enumerate()
            .filter_map(|(ci, c)| {
                let v = ct["matrix"][ri][ci].as_i64().unwrap_or(0);
                (v > 0).then(|| {
                    (
                        c.as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| c.to_string()),
                        v,
                    )
                })
            })
            .collect();
        // Largest counts first, then keep at most `max_cells` of them.
        cells.sort_by_key(|e| std::cmp::Reverse(e.1));
        cells.truncate(max_cells);
        // One line per row: label, its total, and the biggest cells.
        let parts: Vec<String> = cells.iter().map(|(c, v)| format!("{c}: {v}")).collect();
        let label = r
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| r.to_string());
        let total = ct["rowTotals"][ri].as_i64().unwrap_or(0);
        lines.push(format!("- {label} (total {total}) -> {}", parts.join(", ")));
    }
    lines.join("\n")
}

/// Asks the model for a short reading of a cross-tab, under a chosen weather scenario.
/// Takes the cross-tab, the temperature and rain scenario names (either may be missing)
/// and a text describing any filter. Returns the reading as JSON, or `None` on failure.
pub async fn fuse_ask(
    state: &AppState,
    ct: &Value,
    temp: Option<&str>,
    rain: Option<&str>,
    filter_label: Option<&str>,
) -> Option<Value> {
    // Describe each chosen weather scenario in words for the prompt.
    let mut scenario_lines = Vec::new();
    if let Some((avg, min, max, frost)) = temp.and_then(temp_scenario) {
        scenario_lines.push(format!(
            "Temperature: a {} year — average {avg}F, lows to {min}F, highs to {max}F, {frost} months with frost.",
            temp.unwrap()
        ));
    }
    if let Some((total, hum)) = rain.and_then(rain_scenario) {
        scenario_lines.push(format!(
            "Rainfall: {} — {total} inches for the year, average humidity {hum}%.",
            rain.unwrap()
        ));
    }
    // Join the descriptions, or say no scenario was chosen.
    let scenario_text = if scenario_lines.is_empty() {
        "No weather scenario chosen — describe the garden as it stands.".to_string()
    } else {
        scenario_lines.join("\n")
    };
    // An optional line telling the model the table is filtered.
    let filter_text = filter_label
        .map(|f| format!("Only plants where {f}.\n"))
        .unwrap_or_default();
    // The prompt gives the real counts and forbids inventing numbers; it asks for JSON only.
    let prompt = format!(
        "You are reading one cross-tabulation from a home garden database in USDA zone 6a\nin western New York. Every number below is a real count of plants, already\ncounted from the database. Do not invent, adjust or recompute any number.\n\nRows are {}. Columns are {}.\n{filter_text}\n{}\n\nWeather scenario to reason about:\n{scenario_text}\n\nWrite a short, practical reading of this table for the gardener. Be concrete and\nrefer to the actual row and column names above. Say what the shape of the table\nimplies for watering and plant care under that scenario. If the table shows\nnothing interesting, say so plainly rather than inventing significance.\n\nReply with JSON only, in exactly this shape:\n{{\n  \"headline\": \"one sentence, under 20 words\",\n  \"findings\": [\"three to five short observations, each under 25 words\"],\n  \"scenario_effect\": \"two or three sentences on what the weather scenario means for these plants\",\n  \"watch\": [\"one to three specific row or column names most at risk, with a few words why\"]\n}}",
        ct["dim_a_label"].as_str().unwrap_or(""),
        ct["dim_b_label"].as_str().unwrap_or(""),
        table_text(ct, 10, 6)
    );
    // 120 seconds allowed, a little more randomness (0.2), up to 700 output pieces.
    let inner = generate(state, &prompt, 120, 0.2, 700).await?;
    // Keep only the expected fields, tidied; `str_list` limits the lists.
    Some(json!({
        "headline": inner.get("headline").and_then(Value::as_str).map(str::trim).unwrap_or(""),
        "findings": str_list(inner.get("findings"), 5),
        "scenario_effect": inner.get("scenario_effect").and_then(Value::as_str).map(str::trim).unwrap_or(""),
        "watch": str_list(inner.get("watch"), 3),
        "model": state.ollama_model,
    }))
}

// --- discover.php: the connections nobody thought to look for ---------------

/// A column must be filled in for at least this share (55%) of plants to be compared.
const DISC_MIN_FILL: f64 = 0.55;
/// A column with more different values than this (14) is skipped as too spread out.
const DISC_MAX_LEVELS: usize = 14;
/// A pair of columns needs at least this many plants that have both filled in.
const DISC_MIN_ROWS: usize = 60;
/// A combination must occur at least this many times (8) to be reported.
const DISC_MIN_CELL: i64 = 8;
/// A combination must occur at least this many times more often than chance (1.6x).
const DISC_MIN_LIFT: f64 = 1.6;

/// Values that mean "we do not know", not a category.
fn disc_clean(v: Option<&str>) -> Option<String> {
    let v = v?.trim();
    if v.is_empty() {
        return None;
    }
    let low = v.to_lowercase();
    if ["null", "none", "n/a", "unspecified", "not specified"].contains(&low.as_str()) {
        None
    } else {
        Some(v.to_string())
    }
}

// Input: one JSON object per plant. Output: (findings, number of pairs tested, number of
// plants). A finding records how strongly two columns are linked.
//
// The method: for every pair of columns, count how many plants have each combination of
// values (a cross-tab). Compare each count (observed) with the count expected if the two
// columns were unrelated (row total x column total / all). `lift` is observed / expected.
// Cramer's V (called `strength` in the output) turns the chi-squared sum into a number
// from 0 (no link) to 1 (one column decides the other).
/// The whole scan: Cramer's V per pair, lift per combination. Pure arithmetic.
pub fn disc_scan(rows: &[Map<String, Value>]) -> (Vec<Value>, usize, usize) {
    let n = rows.len();
    // No plants: nothing to do.
    if n == 0 {
        return (Vec::new(), 0, 0);
    }
    // A closure that turns a JSON value into clean text, or `None` for blanks and "unknown" words.
    let as_text = |v: Option<&Value>| -> Option<String> {
        match v {
            Some(Value::Null) | None => None,
            Some(Value::String(s)) => disc_clean(Some(s)),
            Some(other) => disc_clean(Some(&other.to_string())),
        }
    };
    // Keep only columns that are filled in enough and have 2 to 14 different values.
    let mut usable: Vec<(String, Vec<Option<String>>)> = Vec::new();
    // The column names come from the first row.
    for col in rows[0].keys() {
        let vals: Vec<Option<String>> = rows.iter().map(|r| as_text(r.get(col))).collect();
        let filled = vals.iter().filter(|v| v.is_some()).count();
        if (filled as f64) / (n as f64) < DISC_MIN_FILL {
            continue;
        }
        // The distinct values of the column: sort and remove repeats.
        let mut levels: Vec<&String> = vals.iter().flatten().collect();
        levels.sort();
        levels.dedup();
        if levels.len() < 2 || levels.len() > DISC_MAX_LEVELS {
            continue;
        }
        usable.push((col.clone(), vals));
    }
    // Sort the usable columns by name so results are in a fixed order.
    usable.sort_by(|a, b| a.0.cmp(&b.0));

    let mut findings: Vec<Value> = Vec::new();
    let mut pairs = 0usize;
    // Compare every column with every later column, once per pair.
    for i in 0..usable.len() {
        for j in (i + 1)..usable.len() {
            let (a, va) = &usable[i];
            let (b, vb) = &usable[j];
            // `ca` and `cb` count each value of the two columns; `joint` counts each combination.
            let mut ca: Vec<(String, i64)> = Vec::new();
            let mut cb: Vec<(String, i64)> = Vec::new();
            let mut joint: std::collections::HashMap<(String, String), i64> =
                std::collections::HashMap::new();
            let mut m = 0i64;
            // Go through the plants that have both columns filled in.
            for k in 0..n {
                // `let (..) = .. else { continue }` skips plants where either value is missing.
                let (Some(x), Some(y)) = (&va[k], &vb[k]) else {
                    continue;
                };
                m += 1;
                bump(&mut ca, x);
                bump(&mut cb, y);
                *joint.entry((x.clone(), y.clone())).or_insert(0) += 1;
            }
            // Too few plants with both values: skip this pair.
            if (m as usize) < DISC_MIN_ROWS {
                continue;
            }
            pairs += 1;
            // `chi2` adds up (observed - expected)^2 / expected over all combinations.
            let mut chi2 = 0.0f64;
            let mut cells: Vec<Value> = Vec::new();
            for (x, cx) in &ca {
                for (y, cy) in &cb {
                    // Expected count if the columns were unrelated.
                    let exp = (*cx as f64) * (*cy as f64) / (m as f64);
                    if exp <= 0.0 {
                        continue;
                    }
                    let obs = *joint.get(&(x.clone(), y.clone())).unwrap_or(&0);
                    chi2 += ((obs as f64) - exp).powi(2) / exp;
                    // Report combinations that occur often enough and well above chance.
                    if obs >= DISC_MIN_CELL && (obs as f64) / exp >= DISC_MIN_LIFT {
                        cells.push(json!({
                            "lift": crate::php::php_round((obs as f64) / exp, 1),
                            "plants": obs,
                            "expected": crate::php::php_round(exp, 1),
                            "a_value": x,
                            "b_value": y,
                        }));
                    }
                }
            }
            if cells.is_empty() {
                continue;
            }
            // `k` is the smaller number of values of the two columns.
            let k = ca.len().min(cb.len());
            // Cramer's V = square root of chi2 / (plants x (k - 1)).
            let v = if k > 1 {
                (chi2 / ((m as f64) * ((k - 1) as f64))).sqrt()
            } else {
                0.0
            };
            // Keep the two strongest combinations (largest lift first).
            cells.sort_by(|p, q| {
                q["lift"]
                    .as_f64()
                    .partial_cmp(&p["lift"].as_f64())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            cells.truncate(2);
            findings.push(json!({ "a": a, "b": b, "strength": crate::php::php_round(v, 2), "plants": m, "cells": cells }));
        }
    }
    // Strongest column pairs first.
    findings.sort_by(|p, q| {
        q["strength"]
            .as_f64()
            .partial_cmp(&p["strength"].as_f64())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    (findings, pairs, n)
}

/// Adds 1 to the count for `key` in a list of (value, count) pairs, adding it if it is new.
fn bump(counts: &mut Vec<(String, i64)>, key: &str) {
    if let Some(e) = counts.iter_mut().find(|(k, _)| k == key) {
        e.1 += 1;
    } else {
        counts.push((key.to_string(), 1));
    }
}

/// Ask the model which three findings are worth a gardener's attention.
async fn disc_judge(
    state: &AppState,
    findings: &[Value],
    tested: usize,
    plants: usize,
) -> Option<Value> {
    // One line per notable combination for the model to read, using the real numbers.
    let mut lines = Vec::new();
    // At most 22 findings are shown to the model.
    for f in findings.iter().take(22) {
        for c in f["cells"].as_array().cloned().unwrap_or_default() {
            lines.push(format!(
                "{} \"{}\" together with {} \"{}\": {} plants, chance predicts {} ({:.1} times as often). Overall link strength {:.2}.",
                f["a"].as_str().unwrap_or(""),
                c["a_value"].as_str().unwrap_or(""),
                f["b"].as_str().unwrap_or(""),
                c["b_value"].as_str().unwrap_or(""),
                c["plants"].as_i64().unwrap_or(0),
                c["expected"],
                c["lift"].as_f64().unwrap_or(0.0),
                f["strength"].as_f64().unwrap_or(0.0)
            ));
        }
    }
    // Nothing to judge.
    if lines.is_empty() {
        return None;
    }
    let body = lines.join("\n");
    // The prompt explains what some columns really mean (so the model does not call a
    // dormant plant dead), lists the findings, and asks it to pick three. Reply is JSON only.
    let prompt = format!(
        "A home garden database in USDA zone 6a, western New York, holds {plants} plants.\nEvery pair of descriptive columns was cross-tabulated -- {tested} pairs in all --\nand these combinations occur more often than chance predicts. All numbers below\nare already computed. Do not recalculate or invent any.\n\nWHAT SOME OF THESE COLUMNS ACTUALLY MEAN. Without this you will misread them:\n  outcome \"Gone\"      = the plant's quantity is currently zero. It does NOT mean\n                        the plant died. An annual that finished its year, or a\n                        perennial that is dormant underground, both read \"Gone\".\n                        The database's own \"died\" flag is set on no plant at all.\n  is_evergreen        = whether foliage persists. \"Summer Dormant\" and \"Spring\n                        Ephemeral\" are plants that VANISH ABOVE GROUND on purpose\n                        and come back. They are not failures.\n  plant_type \"Annual\" = lives one season by definition.\n  deer_resistance     = a claim made when the plant was catalogued, not an\n                        observation of what the deer actually did.\n  price_band          = what it cost, bucketed.\n\nSo a combination of \"Gone\" with anything that is dormant or annual is most\nlikely telling you THE RECORDS ARE MISLEADING, not that plants are dying. That\nis a genuinely useful finding -- say so plainly when you see it.\n\n{body}\n\nPick the THREE most genuinely useful to the gardener, and say why each matters.\n\nReject, and do not pick:\n  - restatements of the same fact (\"toxic fruit\" with \"toxic\" is one fact twice);\n  - botanical necessities (a leaf shape implying a leaf type);\n  - things anyone already knows (trees cost more than annuals).\n\nPrefer findings that would change what the gardener DOES: something about what\nsurvives, what gets eaten, what a bed or a supplier is quietly doing, or a\npattern that suggests the records themselves are misleading.\n\nRemember these are correlations from one person's garden. Most reflect what was\nbought and where it was planted, not what the plants did. Say so where it\napplies rather than implying cause.\n\nReply with JSON only:\n{{\n  \"findings\": [\n    {{\"title\": \"under 10 words\",\n     \"what\": \"the pattern in one plain sentence, using the real numbers above\",\n     \"why\": \"why it is worth the gardener's attention, under 30 words\",\n     \"caution\": \"the most likely innocent explanation, under 20 words\"}}\n  ],\n  \"overall\": \"one sentence on what these three together suggest\"\n}}"
    );
    // 150 seconds allowed, low randomness (0.1).
    let out = generate(state, &prompt, 150, 0.1, 600).await?;
    let found = out.get("findings")?.as_array()?;
    // A small helper closure: read field `k` of a JSON object as trimmed text.
    let s = |f: &Value, k: &str| {
        f.get(k)
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    // Keep at most 3 findings, each of which must be an object.
    let clean: Vec<Value> = found
        .iter()
        .take(3)
        .filter(|f| f.is_object())
        .map(|f| json!({ "title": s(f, "title"), "what": s(f, "what"), "why": s(f, "why"), "caution": s(f, "caution") }))
        .collect();
    // No usable finding came back.
    if clean.is_empty() {
        return None;
    }
    Some(json!({ "findings": clean, "overall": s(&out, "overall"), "model": state.ollama_model }))
}

/// The `?discover` endpoint: cached, and only the warmer may pay for a
/// fresh judgement.
pub async fn disc_run(state: &AppState, allow_slow: bool) -> Value {
    // Cached answer in a file.
    let cache = cache_file(state, "discover.json");
    let cached = cache
        .as_ref()
        .and_then(read_json_file)
        .filter(Value::is_object);
    // Fresh means less than 3600 seconds (one hour) old.
    let fresh = cached.is_some()
        && cache
            .as_ref()
            .and_then(file_age_secs)
            .is_some_and(|a| a < 3600);
    // Same rule as insights: page loads accept a stale cache and never wait for the model.
    if fresh || (!allow_slow && cached.is_some()) {
        let mut c = cached.unwrap();
        c["stale"] = json!(!fresh);
        return c;
    }
    // Load one row per plant. The SQL picks about 25 descriptive columns, plus three made-up
    // columns: `yr` (the first four characters of year_planted), `outcome` ("Gone" when the
    // quantity is 0, else "Growing"), and `price_band` (cost grouped into three bands).
    let rows: Vec<Map<String, Value>> = sqlx::query(
        "SELECT family, company, local_store, sun_exposure, fertilizer, flower_bed,
                zones, water_needs, deer_resistance, season, soil_ph, soil_type,
                toxicity, bloom_color, foliage_color, propagation_method, plant_type,
                leaf_shape, leaf_type, flower_shape, fruit_type, native_status,
                is_evergreen, growth_rate, root_system, fruit_edibility, invasive_local,
                grow_season, LEFT(year_planted,4) AS yr,
                CASE WHEN qty = 0 THEN 'Gone' ELSE 'Growing' END AS outcome,
                CASE WHEN average_cost IS NULL THEN NULL
                     WHEN average_cost < 15 THEN 'under $15'
                     WHEN average_cost < 35 THEN '$15-35'
                     ELSE 'over $35' END AS price_band
         FROM data_source",
    )
    // Run it and turn each database row into a JSON object (see rowjson.rs).
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .iter()
    .filter_map(|r| {
        if let Value::Object(m) = crate::rowjson::row_to_json(r) {
            Some(m)
        } else {
            None
        }
    })
    .collect();
    // The numeric scan; no AI is involved here.
    let (findings, pairs, plants) = disc_scan(&rows);
    // The result before any AI wording is added.
    let mut result = json!({
        "pairs_tested": pairs,
        "plants": plants,
        "top_raw": findings.iter().take(8).cloned().collect::<Vec<_>>(),
        "ai": Value::Null,
        "stale": false,
    });
    // Page loads stop here with the arithmetic only.
    if !allow_slow {
        return result;
    }
    // Only the warmer gets here. If the model is up, ask it to pick the best findings.
    if available(state).await {
        result["ai"] = disc_judge(state, &findings, pairs, plants)
            .await
            .unwrap_or(Value::Null);
    }
    // Save for the next page loads.
    if let Some(cache) = &cache {
        write_json_file(cache, &result);
    }
    result
}

// Tests: only compiled with `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_labels() {
        assert_eq!(
            scenario_label(Some("cold"), Some("wet")).as_deref(),
            Some("Cold year + wet rainfall")
        );
        assert_eq!(scenario_label(None, None), None);
        assert_eq!(scenario_label(Some("bogus"), None), None);
    }

    #[test]
    fn scan_finds_a_perfect_link() {
        // 80 rows where column a decides column b exactly.
        let rows: Vec<Map<String, Value>> = (0..80)
            .map(|i| {
                let mut m = Map::new();
                m.insert("a".into(), json!(if i % 2 == 0 { "x" } else { "y" }));
                m.insert("b".into(), json!(if i % 2 == 0 { "p" } else { "q" }));
                m
            })
            .collect();
        let (findings, pairs, plants) = disc_scan(&rows);
        assert_eq!((pairs, plants), (1, 80));
        assert_eq!(findings[0]["strength"].as_f64(), Some(1.0));
        assert_eq!(findings[0]["cells"][0]["lift"].as_f64(), Some(2.0));
    }
}
