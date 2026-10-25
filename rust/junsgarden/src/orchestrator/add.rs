//! "🌱 Add Plant" (tab_add_plant.py).
//!
//! HOW TO READ THIS FILE (for Jun). Adding a plant is a few steps, and each
//! step's answers ride along in the address (`name`, `bot`, `checked`, `ok`,
//! `path`, ...), the way Streamlit kept them in its session:
//!   1. type the common and botanical names;
//!   2. search both the garden and the wishlist for a match (a wishlist
//!      match can be promoted instead of adding a new record);
//!   3. Quick Create (a stub record plus a JSON schema to fill in later) or
//!      AI Enrich (free AI fills what it can; its answer is kept in
//!      data/orchestrator-ai/add-<token>.json until you create the record);
//!   4. the record is created and you can open the full editor.
//!
//! DIFFERENCES FROM STREAMLIT (DEVIATIONS.md): the garden login already says
//! who you are, so the "Account email" box is gone (S-11); when the free AI
//! is busy the reason is shown (S-12).
//! URL: /streamlit/?tab=1. `render` draws the page for the current step.
//! `action` handles the POST buttons (create a stub, ask the AI, create the
//! record, promote a wishlist plant). `edit_phase` shows the new record
//! with the full editor. `schema_json` and `missing_json` build downloads.
//!
//! Rust ideas used often in this file (explained once here):
//!   - `async fn` waits for the database; `.await` pauses until it answers.
//!   - `&str` is a borrowed look at text; `String` is text the code owns.
//!   - `Option<T>` is "a value or nothing" (`Some(x)` / `None`).
//!   - `Result<T, E>` is "worked" (`Ok(x)`) or "failed" (`Err(e)`).
//!   - `?` after a call means: if it failed or is empty, return that from
//!     this function now; otherwise carry on with the good value.
//!   - `let Some(x) = .. else { .. }` continues only when the value is there.
//!   - `|x| ..` is a small unnamed function (a closure).
//!   - `Map<String, Value>` is one JSON object (field -> value).
//!   - `h.push_str(..)` adds text to the end of the page text `h`.
//! The page is one long function that returns early (`return h + ..`) when
//! a step is not finished yet, so later steps only appear after earlier ones.

use serde_json::{Map, Value};

use super::ai;
use super::export;
use super::form::{self, Flash, FormCtx};
use super::opts::{self, text};
use super::ui;
use super::{all_plant_names, rows, Req};
use crate::php::{he, php_int, urlencode};
use crate::AppState;

/// The text of the field reference guide (plant_schema_reference.txt), built into
/// the program at compile time by `include_str!`. Served as a download.
pub const REFERENCE: &str = include_str!("plant_schema_reference.txt");

/// The two ways to add a plant, shown as radio buttons in Step 3.
/// `[&str; 2]` is a fixed-size list holding exactly two pieces of text.
const PATHS: [&str; 2] = [
    "⚡ Quick Create — enter basics now, complete data later via JSON",
    "✨ AI Enrich — auto-fill fields with Claude + Perenual + Wikipedia",
];

/// Reads field `k` of a row as plain text ("" when missing or null).
fn s(m: &Map<String, Value>, k: &str) -> String {
    text(m.get(k))
}

/// Hidden inputs carrying these address settings into a form.
/// Takes the request and a list of setting names. Gives back hidden form inputs
/// for each setting that has a value, so a form keeps them when it is sent.
fn keep(req: &Req, keys: &[&str]) -> String {
    keys.iter()
        .filter(|k| !req.get(k).is_empty())
        .map(|k| {
            format!(
                "<input type=\"hidden\" name=\"{k}\" value=\"{}\">",
                he(req.get(k))
            )
        })
        .collect()
}

/// This tab's address with these settings (and any extra `&k=v`).
/// Builds the address of this tab with the named settings kept, plus `extra`
/// (for example "&promote=5"). Values are URL-encoded so they are safe in an address.
fn link(req: &Req, keys: &[&str], extra: &str) -> String {
    let mut u = String::from("/streamlit/?tab=1");
    for k in keys {
        if !req.get(k).is_empty() {
            u.push_str(&format!("&{k}={}", urlencode(req.get(k))));
        }
    }
    u + extra
}

/// Where an AI answer is kept while the user reviews it: add-<token>.json.
/// Gives `None` unless the token is only letters and digits, so a token such
/// as "../x" cannot point at some other file.
fn token_path(state: &AppState, token: &str) -> Option<std::path::PathBuf> {
    if token.is_empty() || !token.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(
        state
            .data
            .join("orchestrator-ai")
            .join(format!("add-{token}.json")),
    )
}

/// Reads the saved AI answer for `token`. Gives `None` if missing or not a JSON object.
fn load_ai(state: &AppState, token: &str) -> Option<Map<String, Value>> {
    let p = token_path(state, token)?;
    match serde_json::from_slice::<Value>(&std::fs::read(p).ok()?).ok()? {
        Value::Object(m) => Some(m),
        _ => None,
    }
}

/// A new random token: 8 random bytes written as 16 hex characters.
fn new_token() -> String {
    let b: [u8; 8] = rand::random();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The plant-tag guide as markdown text: a table of tag words and their meaning.
/// Used when the photo plant_tag_guide.jpg is not on the server.
const TAG_TABLE: &str = "| Tag field | What it means |\n|---|---|\n| **Common name** | Friendly name (e.g. *Cranesbill*) — usually largest text on front |\n| **Botanical name** | Scientific name in italics (e.g. *Geranium cinereum 'Ballerina'*) |\n| **PERENNIAL / ANNUAL / BIENNIAL** | Life cycle |\n| **☀️ ☀️🌤 🌥** | Sun icons = light requirement (full sun → full shade) |\n| **Bloom time** | Season or months when flowers appear |\n| **Exposure** | Sun / Part Shade / Shade |\n| **Water** | Moisture needs |\n| **Height / Width** | Mature size — use for *plant_length* and *plant_width* |\n| **Hardiness Zones** | USDA zone range (e.g. *USDA Zone 3-8*) |\n| **Deer resistant** | Look for a deer icon on the tag |";

/// The tag guide's markdown table as an HTML table.
fn tag_table() -> String {
    // The first line is the header row; the second is the |---| divider, which is skipped.
    let mut lines = TAG_TABLE.lines();
    let head: Vec<String> = lines
        .next()
        .unwrap_or("")
        .trim_matches('|')
        .split('|')
        .map(|c| format!("<th>{}</th>", ui::md(c.trim())))
        .collect();
    let body: String = lines
        .skip(1)
        .map(|l| {
            format!(
                "<tr>{}</tr>",
                l.trim_matches('|')
                    .split('|')
                    .map(|c| format!("<td>{}</td>", ui::md(c.trim())))
                    .collect::<String>()
            )
        })
        .collect();
    format!(
        "<table class=\"st-md-table\"><thead><tr>{}</tr></thead><tbody>{body}</tbody></table>",
        head.concat()
    )
}

/// Draws the Add Plant tab. `req` holds the address settings; each step stores
/// its answer there (name, bot, checked, ok, path, ai, new). Gives back HTML.
/// Steps: 1 names, 2 duplicate check, 3 choose a path, 4 create the record.
pub async fn render(state: &AppState, req: &Req) -> String {
    let mut h = ui::header("🌱 Add a New Plant");
    // Show the photo of the tag guide if it was saved on the server; else a text table.
    let guide = if state.data.join("plant_tag_guide.jpg").is_file() {
        "<img src=\"/streamlit/?tagguide=1\" alt=\"How to read a plant tag\" style=\"width:100%\">"
            .to_string()
    } else {
        tag_table() + &ui::caption("💡 To show the visual photo here, save the image as `plant_tag_guide.jpg` in the site's data folder on the server.")
    };
    h.push_str(&ui::expander(
        "📋 How to read a plant tag — click to expand",
        false,
        &guide,
    ));

    // `new` in the address means a record was just created: show its editor instead.
    if !req.get("new").is_empty() {
        return h + &edit_phase(state, req).await;
    }

    // The two names typed in Step 1, trimmed.
    let (name, bot) = (
        req.get("name").trim().to_string(),
        req.get("bot").trim().to_string(),
    );
    h.push_str(&ui::subheader("Step 1 · Name the Plant"));
    // Submits by itself once both boxes have text (onchange).
    h.push_str("<form method=\"get\" action=\"/streamlit/\" onchange=\"if(this.name.value.trim()&&this.bot.value.trim())this.submit()\"><input type=\"hidden\" name=\"tab\" value=\"1\">");
    h.push_str(&ui::columns(
        &[1, 1],
        vec![
            ui::text_input(
                "name",
                "Common / Display Name *",
                req.get("name"),
                "e.g.  Dwarf Hummingbird Mint",
            ),
            ui::text_input(
                "bot",
                "Botanical Name *",
                req.get("bot"),
                "e.g.  Agastache cana 'Sinning'",
            ),
        ],
    ));
    h.push_str("<button type=\"submit\" hidden></button></form>");
    // Both names are required before Step 2 appears.
    if name.is_empty() || bot.is_empty() {
        return h + &ui::info("Enter both a common name and a botanical name to continue.");
    }

    // Step 2: search the garden and the wishlist for a similar plant.
    h.push_str(&ui::subheader("Step 2 · Check for Duplicates"));
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"1\">{}<input type=\"hidden\" name=\"checked\" value=\"1\">{}</form>",
        keep(req, &["name", "bot"]), ui::button("go", "1", "🔍 Search Database for Existing Match", false, true)));
    // Wait for the user to press the search button (it sets `checked=1`).
    if req.get("checked") != "1" {
        return h + &ui::caption("Click **Search Database** above before continuing.");
    }
    // `like`: wrap text in % signs; in SQL LIKE, % means "anything".
    let like = |v: &str| format!("%{v}%");
    // Garden matches: the botanical name OR the common name contains the typed text.
    // `?` marks are filled by `.bind(..)`; the typed text is never put into the SQL itself.
    // One match: id, plant name, botanical name, and beds (garden) or company (wish list).
    type Match = (i64, Option<String>, Option<String>, Option<String>);
    let ds: Vec<Match> = sqlx::query_as("SELECT id, plant_name, botanical_name, garden_beds FROM data_source WHERE botanical_name LIKE ? OR plant_name LIKE ?")
        .bind(like(&bot)).bind(like(&name)).fetch_all(&state.db).await.unwrap_or_default();
    // Wishlist matches use the same search on the future_plants table.
    let fp: Vec<Match> = sqlx::query_as("SELECT id, plant_name, botanical_name, company FROM future_plants WHERE botanical_name LIKE ? OR plant_name LIKE ?")
        .bind(like(&bot)).bind(like(&name)).fetch_all(&state.db).await.unwrap_or_default();
    // Shows a dash for a missing value.
    let or_dash = |v: &Option<String>| {
        v.clone()
            .filter(|x| !x.is_empty())
            .unwrap_or_else(|| "—".into())
    };
    // Garden matches: warn so a duplicate is not added by accident.
    if !ds.is_empty() {
        h.push_str(&ui::warning(&format!("⚠️  **{} match(es) already in your garden database.** Review them carefully before adding a duplicate.", ds.len())));
        for (id, n, b, beds) in &ds {
            h.push_str(&ui::markdown(&format!(
                "- **[ID {id}]  {}**  ·  _{}_  ·  Beds: {}",
                n.clone().unwrap_or_default(),
                or_dash(b),
                beds.clone()
                    .filter(|x| !x.is_empty())
                    .unwrap_or_else(|| "To Be Sorted".into())
            )));
        }
    }
    // Wishlist matches: a tick box can promote one into the garden instead.
    if !fp.is_empty() {
        h.push_str(&ui::info(&format!("🌟 **{} match(es) found in your Wishlist (future_plants).** You can promote one directly instead of creating a new record.", fp.len())));
        for (id, n, b, co) in &fp {
            let n = n.clone().unwrap_or_default();
            // `ready` is true after the user ticked the promote box for this row (`promote=<id>`).
            let ready = req.get("promote") == id.to_string();
            // The tick box is a link that toggles `promote` in the address.
            let toggle_to = if ready {
                link(req, &["name", "bot", "checked"], "")
            } else {
                link(req, &["name", "bot", "checked"], &format!("&promote={id}"))
            };
            h.push_str(&ui::columns(&[3, 1], vec![
                ui::markdown(&format!("- **Wish List** · **{n}**  ·  _{}_  ·  Company: {}", or_dash(b), or_dash(co))),
                format!("<a class=\"st-check\" href=\"{}\"><input type=\"checkbox\" onclick=\"return false\"{}> Promote to garden</a>", he(&toggle_to), if ready { " checked" } else { "" }),
            ]));
            // Show the confirm button and stop here, so nothing below can run at the same time.
            if ready {
                h.push_str(&format!("<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"add_promote\"><input type=\"hidden\" name=\"tab\" value=\"1\"><input type=\"hidden\" name=\"fp\" value=\"{id}\">{}{}</form>",
                    keep(req, &["name", "bot"]), ui::button("go", "1", &format!("🌱 Confirm: Move \"{n}\" to garden database"), true, false)));
                return h;
            }
        }
    }
    // No matches: carry on. Matches in the garden: ask the user to confirm first.
    if ds.is_empty() && fp.is_empty() {
        h.push_str(&ui::success("✅  No matches found — this plant is new!"));
    } else if !ds.is_empty() {
        let label = if fp.is_empty() {
            "✅  I've reviewed the matches — this is a genuinely different plant, continue."
        } else {
            "✅  I've reviewed all matches — continue adding as a new plant."
        };
        h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"tab\" value=\"1\">{}{}</form>", keep(req, &["name", "bot", "checked"]), ui::checkbox("ok", label, req.get("ok") == "1", None)));
        if req.get("ok") != "1" {
            return h;
        }
    }

    h.push_str(&ui::subheader(
        "Step 3 · How would you like to add this plant?",
    ));
    // `path=ai` chooses AI Enrich; anything else is Quick Create.
    let path = if req.get("path") == "ai" {
        PATHS[1]
    } else {
        PATHS[0]
    };
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"tab\" value=\"1\">{}", keep(req, &["name", "bot", "checked", "ok"])));
    // The radio buttons post back as `path=quick` or `path=ai`: the long label
    // text is swapped for those short values.
    h.push_str(
        &ui::radio("pathl", "Choose a path:", &PATHS, path, true, false)
            .replace(&format!("value=\"{}\"", he(PATHS[0])), "value=\"quick\"")
            .replace(&format!("value=\"{}\"", he(PATHS[1])), "value=\"ai\"")
            .replace("name=\"pathl\"", "name=\"path\""),
    );
    h.push_str("</form>");
    // Hidden fields that keep all earlier answers in the next form.
    let carry = keep(req, &["name", "bot", "checked", "ok", "path"]);

    // Quick Create: ask for year, beds and quantity, then make a stub record.
    if path == PATHS[0] {
        h.push_str(&ui::markdown("Enter the essentials below. A full **JSON schema** and **reference guide** will be generated so you can fill in the remaining fields offline and import later."));
        // Year choices: a blank first, then the known years.
        let years: Vec<String> = std::iter::once(String::new())
            .chain(opts::YEARS_ADDED.iter().map(|y| y.to_string()))
            .collect();
        let beds: Vec<String> = opts::GARDEN_BEDS.iter().map(|b| b.to_string()).collect();
        // multipart, so the bed chips arrive joined with ", " (formdata.rs)
        h.push_str(&format!("<form method=\"post\" action=\"/streamlit/\" enctype=\"multipart/form-data\"><input type=\"hidden\" name=\"act\" value=\"add_quick\"><input type=\"hidden\" name=\"tab\" value=\"1\">{carry}"));
        h.push_str(&ui::columns(
            &[1, 1, 1],
            vec![
                ui::select_plain("year", "Year Added", &years, ""),
                ui::multiselect("beds[]", "Garden Bed(s)", &beds, &[]),
                ui::number_input("qty", "Quantity", "1", Some(0), "1"),
            ],
        ));
        h.push_str(&ui::button(
            "go",
            "1",
            "📋 Create Stub & Download Schema",
            true,
            true,
        ));
        h.push_str("</form>");
        return h;
    }

    // AI Enrich path.
    h.push_str(&ui::markdown("**Claude + Perenual + Wikipedia** will research this plant and pre-fill as many fields as possible.  You will review and edit every value before anything is saved permanently."));
    // No saved AI answer yet (`ai` token missing): show the Fetch and Skip buttons.
    let Some(job) = load_ai(state, req.get("ai")) else {
        // Text shown while the page waits (data-busy is used by st.js).
        let busy = format!(
            "Researching {name} via Claude + Perenual + Wikipedia…  This may take 20–40 seconds."
        );
        // `post`: builds one small POST form for a button.
        let post = |act: &str, label: &str, primary: bool, busy: &str| {
            format!(
            "<form method=\"post\" action=\"/streamlit/\"{}><input type=\"hidden\" name=\"act\" value=\"{act}\"><input type=\"hidden\" name=\"tab\" value=\"1\">{carry}{}</form>",
            if busy.is_empty() { String::new() } else { format!(" data-busy=\"{}\"", he(busy)) }, ui::button("go", "1", label, primary, true))
        };
        h.push_str(&ui::columns(
            &[3, 1],
            vec![
                post("add_ai", "✨ Fetch Plant Data with AI", true, &busy),
                post("add_skip", "⏭ Skip AI", false, ""),
            ],
        ));
        return h;
    };
    // There is an answer: show what it contains. `data` = the values, `fields` = which were filled.
    let data = job
        .get("data")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let fields: Vec<String> = job
        .get("fields")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|x| text(Some(x))).collect())
        .unwrap_or_default();
    // If the AI failed, the reason is shown (S-12 in DEVIATIONS.md).
    if !s(&job, "error").is_empty() {
        h.push_str(&ui::warning(&format!(
            "The AI could not answer: {}",
            s(&job, "error")
        )));
    }
    if fields.is_empty() {
        h.push_str(&ui::info(
            "AI returned no suggestions — you'll fill everything in manually.",
        ));
    } else {
        h.push_str(&ui::success(&format!(
            "✨  AI pre-filled **{}** field(s) — review them in the form below.",
            fields.len()
        )));
        let t: Vec<Vec<String>> = fields
            .iter()
            .map(|f| {
                vec![
                    ui::cell(f),
                    ui::cell(&s(&data, f).chars().take(120).collect::<String>()),
                ]
            })
            .collect();
        h.push_str(&ui::expander(
            "Fields suggested by AI",
            false,
            &ui::dataframe(&["Field", "AI Suggestion"], &t),
        ));
    }
    // Step 4: create the record from the answer.
    h.push_str(&ui::divider());
    h.push_str(&ui::subheader("Step 4 · Create the Record"));
    h.push_str(&ui::markdown("Click below to create the plant record.  All AI-suggested values are written to the database immediately; you'll get a quick confirmation and can move on to the next plant.  Use **Open full editor** afterward only if you need to review."));
    h.push_str(&format!("<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"add_create\"><input type=\"hidden\" name=\"tab\" value=\"1\">{}{}</form>",
        keep(req, &["name", "bot", "ai"]), ui::button("go", "1", "📋 Create Record →", true, true)));
    h
}

/// Shows the page for a freshly created plant (`new=<id>` in the address):
/// a success message, an Add Another button, optional downloads, and the full
/// editor when the "Open full editor" switch is on. Gives back HTML.
async fn edit_phase(state: &AppState, req: &Req) -> String {
    // The new plant's id.
    let id = php_int(req.get("new"));
    let row = rows(state, &format!("SELECT * FROM data_source WHERE id = {id}")).await;
    // Load the record; if it is gone, say so and offer to start over.
    let Some(plant) = row.into_iter().next() else {
        return ui::error(&format!(
            "Could not load plant ID {id} from the database.  The record may have been deleted."
        )) + &ui::link_button("/streamlit/?tab=1", "Start Over");
    };
    let name = s(&plant, "plant_name");
    // `full=1` means the user switched on the full editor.
    let full = req.get("full") == "1";
    let mut h = ui::success(&format!(
        "✅ ok — **{name}** added (ID {id}). All fields saved."
    ));
    h.push_str(&ui::columns(&[1, 1], vec![
        format!("<a class=\"st-btn st-primary st-full\" href=\"/streamlit/?tab=1\">➕ Add Another Plant</a>"),
        format!("<form method=\"get\" action=\"/streamlit/\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"tab\" value=\"1\">{}{}</form>",
            keep(req, &["new", "ai", "qc"]), ui::toggle("full", "✏️ Open full editor for this plant", full, Some("Only if you need to review or fix something before moving on."))),
    ]));
    if !full {
        return h;
    }
    h.push_str(&ui::divider());
    h.push_str(&ui::caption("Fields pre-filled by AI are marked ✨.  Garden beds, company, quantity, year and cost are yours to set manually."));
    // A wide button made from a link.
    let full_btn = |href: String, label: &str| {
        ui::link_button(&href, label).replace("class=\"st-btn\"", "class=\"st-btn st-full\"")
    };
    // `qc=1` marks a Quick Create record: offer the schema and guide downloads.
    if req.get("qc") == "1" {
        h.push_str(&ui::columns(
            &[1, 1],
            vec![
                full_btn(
                    format!("/streamlit/?export=add_schema&id={id}"),
                    "⬇️ Download JSON Schema",
                ),
                full_btn(
                    "/streamlit/?export=schema_ref".into(),
                    "📖 Download Field Reference Guide",
                ),
            ],
        ));
        h.push_str(&ui::info("💡 Fill in the blank fields in the JSON, then head to **🔄 JSON Pipeline → 📥 Phase 3 — Import & Review** to bring the data in."));
    }
    // Offer a download of the fields that are still empty.
    let missing = opts::missing_fields(&plant);
    if missing.is_empty() {
        h.push_str(&ui::success(
            "🎉 All required fields are filled — no missing data!",
        ));
    } else {
        let body = ui::link_button(&format!("/streamlit/?export=add_missing&id={id}"), "⬇️ Download missing-fields JSON")
            + &ui::info("💡 After downloading and filling in the JSON, go to **🔄 JSON Pipeline → 📥 Phase 3 — Import & Review** to bring it back in.");
        h.push_str(&ui::expander(
            &format!(
                "📋 Export missing-fields JSON  ({} field(s) still empty)",
                missing.len()
            ),
            false,
            &body,
        ));
    }
    h.push_str(&ui::divider());
    // Which fields the AI filled, so the editor can mark them with a star.
    let ai_fields: Vec<String> = load_ai(state, req.get("ai"))
        .and_then(|j| j.get("fields").and_then(Value::as_array).cloned())
        .map(|a| a.iter().map(|x| text(Some(x))).collect())
        .unwrap_or_default();
    let names = all_plant_names(state).await;
    let empty = Map::new();
    // Hidden fields that bring Save back to this same view.
    let mut back = vec![
        ("tab", "1".to_string()),
        ("new", id.to_string()),
        ("full", "1".to_string()),
    ];
    for k in ["ai", "qc"] {
        if !req.get(k).is_empty() {
            back.push((k, req.get(k).to_string()));
        }
    }
    // Context for the editor. `&ai_fields` borrows the list; `images_dir` is where photos live.
    let ctx = FormCtx {
        back: back.clone(),
        is_ai: true,
        new_fields: &ai_fields,
        changed_fields: &empty,
        all_names: &names,
        images_dir: state.www.join("images/Database"),
    };
    // Draw the editor, then the quantity logger under it.
    h.push_str(&form::render(&plant, &plant, &ctx));
    h.push_str(&form::qty_logger(id, &back));
    h.push_str(&ui::divider());
    h.push_str(&ui::caption("This plant is now in the database.  You can also find and edit it at any time via the **✍️ Manual Editor** tab.  Use the **➕ Add Another Plant** button above when you're ready for the next one."));
    h
}

/// `_build_schema_json`, from what the stub record holds.
/// Builds the JSON file for filling in a Quick Create plant. It lists every
/// field with a blank value (the name, years, beds and quantity are filled in).
/// Takes the plant id; gives (file text, file name) or `None` if not found.
pub async fn schema_json(state: &AppState, id: i64) -> Option<(String, String)> {
    let row = rows(state, &format!("SELECT * FROM data_source WHERE id = {id}"))
        .await
        .into_iter()
        .next()?;
    // Every field that goes in the file, in order. `const` inside a function is local to it.
    const ALL: [&str; 60] = [
        "plant_name",
        "botanical_name",
        "family",
        "genus",
        "species",
        "plant_type",
        "plant_length",
        "plant_width",
        "spacing",
        "zones",
        "sun_exposure",
        "water_needs",
        "soil_ph",
        "soil_type",
        "bloom_color",
        "foliage_color",
        "stem_color",
        "attracts",
        "deer_resistance",
        "toxicity",
        "care_notes",
        "maintenance_notes",
        "cleanup_notes",
        "disease_susceptibility",
        "propagation_method",
        "cuttings",
        "fertilizer",
        "extra_details",
        "companion_plants",
        "aromatics",
        "fruit_type",
        "fruit_shape",
        "fruit_color",
        "fruit_size",
        "fruit_edibility",
        "leaf_shape",
        "leaf_type",
        "leaf_margin",
        "flower_shape",
        "flower_size",
        "flower_symmetry",
        "growth_rate",
        "native_status",
        "invasive_local",
        "is_evergreen",
        "life_expectancy",
        "chilling_hours",
        "planting_depth",
        "reproduction_type",
        "root_system",
        "root_morphology",
        "root_function",
        "grow_months",
        "bloom_months",
        "company",
        "local_store",
        "average_cost",
        "year_planted",
        "garden_beds",
        "qty",
    ];
    // Keys starting with `_` are notes for the person; the import skips them.
    let name = s(&row, "plant_name");
    let mut m = Map::new();
    m.insert("_id".into(), serde_json::json!(id));
    m.insert("_plant_name".into(), Value::String(name.clone()));
    m.insert("_note".into(), Value::String("Fill in every empty field below, then import via JSON Pipeline → Phase 3.  See plant_schema_reference.txt for field descriptions.".into()));
    // `_missing_fields` lists the blanks that still need filling.
    m.insert(
        "_missing_fields".into(),
        Value::Array(
            ALL.iter()
                .filter(|f| {
                    !matches!(
                        **f,
                        "plant_name" | "botanical_name" | "year_planted" | "garden_beds" | "qty"
                    )
                })
                .map(|f| Value::String(f.to_string()))
                .collect(),
        ),
    );
    // Fill each field: some from the row, `qty` as a number, everything else blank text.
    for f in ALL {
        let v = match f {
            "plant_name" | "botanical_name" | "year_planted" | "garden_beds" => {
                Value::String(s(&row, f))
            }
            "qty" => serde_json::json!(php_int(&s(&row, "qty"))),
            _ => Value::String(String::new()),
        };
        m.insert(f.into(), v);
    }
    // ensure_ascii=False here, unlike the other exports.
    Some((
        serde_json::to_string_pretty(&Value::Object(m)).unwrap_or_default(),
        export::download_name(&name, "_schema.json"),
    ))
}

/// The "missing-fields JSON" for a new plant.
/// Takes the plant id; gives (file text, file name) or `None`. Includes every column of the row.
pub async fn missing_json(state: &AppState, id: i64) -> Option<(String, String)> {
    let row = rows(state, &format!("SELECT * FROM data_source WHERE id = {id}"))
        .await
        .into_iter()
        .next()?;
    // The helper fields go first, then every column of the row.
    let mut m = Map::new();
    m.insert(
        "_missing_fields".into(),
        Value::Array(
            opts::missing_fields(&row)
                .into_iter()
                .map(|f| Value::String(f.into()))
                .collect(),
        ),
    );
    m.insert("_note".into(), Value::String("Fields listed in _missing_fields are still blank.  Fill them and import via JSON Pipeline → Phase 3.".into()));
    for (k, v) in &row {
        m.insert(k.clone(), v.clone());
    }
    Some((
        export::python_json(&Value::Object(m)),
        export::download_name(&s(&row, "plant_name"), "_new_plant_missing.json"),
    ))
}

/// The real column names of data_source, so an AI answer is only saved to columns that exist.
async fn columns_of(state: &AppState) -> Vec<String> {
    sqlx::query_scalar("SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'data_source'")
        .fetch_all(&state.db).await.unwrap_or_default()
}

/// Makes a new row with just a name (and the "needs review" flag on).
/// Gives back the new id, or the database error.
async fn insert_stub(state: &AppState, name: &str) -> Result<i64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO data_source (plant_name, needs_review) VALUES (?, 1)")
        .bind(name)
        .execute(&state.db)
        .await?;
    state.invalidate_widget_cache();
    Ok(r.last_insert_id() as i64)
}

/// The tab's POST buttons. Returns the messages and where to go next.
/// Takes the button name `act` and the submitted form `f`. Gives back
/// (messages to show, settings for the next address). Called by `action` in mod.rs.
pub async fn action(
    state: &AppState,
    act: &str,
    f: &crate::formdata::FormData,
) -> (Vec<Flash>, Vec<(&'static str, String)>) {
    // The names typed in Step 1 are carried in every form.
    let name = f.str("name").trim().to_string();
    let bot = f.str("bot").trim().to_string();
    // `step` = settings to carry to the next page: names plus any earlier answers.
    let mut step: Vec<(&'static str, String)> = vec![("name", name.clone()), ("bot", bot.clone())];
    for k in ["checked", "ok", "path"] {
        if !f.str(k).is_empty() {
            step.push((k, f.str(k).to_string()));
        }
    }
    match act {
        // Promote a wishlist plant into the garden (the work is in wishlist.rs).
        "add_promote" => match super::wishlist::promote(state, php_int(f.str("fp")), false).await {
            Ok(new_id) => {
                let n: String =
                    sqlx::query_scalar("SELECT plant_name FROM data_source WHERE id = ?")
                        .bind(new_id)
                        .fetch_optional(&state.db)
                        .await
                        .ok()
                        .flatten()
                        .flatten()
                        .unwrap_or_default();
                (
                    vec![Flash::Success(format!(
                        "✅ **{n}** promoted to garden! New ID: {new_id}"
                    ))],
                    vec![("name", name), ("bot", bot)],
                )
            }
            Err(e) => (vec![Flash::Error(format!("Failed: {e}"))], step),
        },
        // Quick Create: make the stub row, then fill the basics.
        "add_quick" => {
            if name.is_empty() {
                return (vec![Flash::Error("Enter a name first.".into())], step);
            }
            // `match` on the result: `Ok(i)` gives the id; an error returns a message at once.
            let id = match insert_stub(state, &name).await {
                Ok(i) => i,
                Err(e) => return (vec![Flash::Error(format!("Database error: {e}"))], step),
            };
            let beds = f.str("beds[]");
            let year = f.str("year");
            // Fill in botanical name, year, beds and quantity. Empty text is stored as NULL.
            let _ = sqlx::query("UPDATE data_source SET botanical_name = ?, year_planted = ?, garden_beds = ?, qty = ? WHERE id = ?")
                .bind(&bot).bind(if year.is_empty() { None } else { Some(year) })
                .bind(if beds.is_empty() { None } else { Some(beds) })
                .bind(php_int(f.str("qty")).max(0)).bind(id).execute(&state.db).await;
            // Go to the new record's page with the download buttons shown (`qc=1`).
            (
                vec![],
                vec![
                    ("new", id.to_string()),
                    ("qc", "1".into()),
                    ("full", "1".into()),
                ],
            )
        }
        // "add_ai" asks the free AI; "add_skip" saves an empty answer so the
        // user fills everything in by hand.
        "add_ai" | "add_skip" => {
            // `job` is the saved answer: `data` (values), `fields` (names filled), maybe `error`.
            let mut job = Map::new();
            let mut data = Map::new();
            let mut fields: Vec<Value> = Vec::new();
            // Start with only the two names, and ask the AI about the rest.
            if act == "add_ai" {
                let mut skel = Map::new();
                skel.insert("plant_name".into(), Value::String(name.clone()));
                skel.insert("botanical_name".into(), Value::String(bot.clone()));
                // Weather and local context for the AI, and Perenual plant data.
                let loc = ai::location_context(state, &name).await;
                let peren = ai::perenual(state, &name).await;
                match ai::ask(state, &skel, &loc, &peren, true).await {
                    Ok(llm) => {
                        // Keep only answers that have a real value.
                        for (k, v) in llm {
                            let t = text(Some(&v));
                            if !v.is_null() && !matches!(t.trim(), "" | "null") {
                                fields.push(Value::String(k.clone()));
                                data.insert(k, v);
                            }
                        }
                    }
                    // The AI could not answer; remember why so the page can show it.
                    Err(e) => {
                        job.insert("error".into(), Value::String(e.error));
                    }
                }
            }
            // The two typed names always win.
            data.insert("plant_name".into(), Value::String(name.clone()));
            data.insert("botanical_name".into(), Value::String(bot.clone()));
            job.insert("data".into(), Value::Object(data));
            job.insert("fields".into(), Value::Array(fields));
            // Save the answer to a file and carry its token to the next page.
            let token = new_token();
            if let Some(p) = token_path(state, &token) {
                let _ = std::fs::create_dir_all(p.parent().unwrap_or(&state.data));
                let _ = std::fs::write(p, Value::Object(job).to_string());
            }
            step.push(("ai", token));
            (vec![], step)
        }
        // Create the record, then write every AI value whose column exists.
        "add_create" => {
            let job = load_ai(state, f.str("ai")).unwrap_or_default();
            let data = job
                .get("data")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            let id = match insert_stub(state, &name).await {
                Ok(i) => i,
                Err(e) => return (vec![Flash::Error(format!("Database error: {e}"))], step),
            };
            // `_apply_ai_to_record`: every AI value whose column exists.
            let cols = columns_of(state).await;
            // Helper fields and the id are never written.
            let skip = [
                "id",
                "_missing_fields",
                "_enriched_fields",
                "_proposed_changes",
                "_ai_filled_fields",
            ];
            // Keep real columns only. Month codes must be exactly 12 characters.
            let sets: Vec<(String, Value)> = data
                .into_iter()
                .filter(|(k, _)| !skip.contains(&k.as_str()) && cols.iter().any(|c| c == k))
                .filter(|(k, v)| {
                    !matches!(k.as_str(), "grow_months" | "bloom_months")
                        || text(Some(v)).chars().count() == 12
                })
                .collect();
            if !sets.is_empty() {
                // Build: UPDATE data_source SET `a` = ?, `b` = ? WHERE id = ?
                let sql = format!(
                    "UPDATE data_source SET {} WHERE id = ?",
                    sets.iter()
                        .map(|(k, _)| format!("`{k}` = ?"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let mut q = sqlx::query(&sql);
                // Bind each value in order, using the type that matches the JSON value.
                for (_, v) in &sets {
                    q = match v {
                        Value::Null => q.bind(None::<String>),
                        Value::Number(n) if n.is_i64() => q.bind(n.as_i64()),
                        Value::Number(n) => q.bind(n.as_f64()),
                        Value::Bool(b) => q.bind(*b as i64),
                        other => q.bind(text(Some(other))),
                    };
                }
                // Last bind is the id; if saving fails, the stub still exists, so say so.
                if let Err(e) = q.bind(id).execute(&state.db).await {
                    return (vec![Flash::Error(format!("The record was created (ID {id}) but the AI values could not be saved: {e}"))], vec![("new", id.to_string())]);
                }
            }
            (
                vec![],
                vec![("new", id.to_string()), ("ai", f.str("ai").to_string())],
            )
        }
        _ => (vec![Flash::Error("Unknown action.".into())], step),
    }
}
