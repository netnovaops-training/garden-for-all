//! JSON Pipeline → "🌟 Wishlist (Future Plants)" (`_render_wishlist_pipeline`
//! in tab_json_pipeline.py): add a wishlist plant, export up to 20 with
//! blanks, and import the filled-in file. Everything here writes ONLY to
//! future_plants; the garden table is never touched.
//!
//! The three inner tabs are `&wsub=1..3`; the uploaded file's token is `wup`.
//!
//! URL: /streamlit/?tab=11&sub=5 (tab 11 is "JSON Pipeline", inner tab 5).
//!
//! HOW TO READ THIS FILE. `render` draws the page and picks one of three
//! inner tabs: `add_form` (1), `export_tab` (2), `import_tab` (3). `export`
//! builds the download. `action` handles the POST buttons. The helpers it
//! uses (`diff`, `commit`, `review_body`, `post_form`...) live in pipeline.rs.
//!
//! Rust ideas used often in this file (explained once here):
//!   - `async fn` waits for the database; `.await` pauses until it answers.
//!   - `&str` is a borrowed look at text; `String` is text the code owns.
//!   - `Option<T>` is "a value or nothing" (`Some(x)` / `None`).
//!   - `Result<T, E>` is "worked" (`Ok(x)`) or "failed" (`Err(e)`).
//!   - `let Some(x) = .. else { .. }` continues only when the value is there.
//!   - `?` after a call means: if it is empty or failed, return that now.
//!   - `Map<String, Value>` is one JSON object (a row: field -> value).
//!   - `h.push_str(..)` adds text to the end of the page text `h`.

use std::collections::HashMap;

use serde_json::{Map, Value};

use super::form::Flash;
use super::opts;
use super::pipeline::{self, diff, dumps, flag, hidden, ids_of, load_upload, pick_box, post_form, review_body, s, set_flag, stamp, uploader};
use super::ui;
use super::{all_plant_names, rows, Req};
use crate::php::{he, php_int};
use crate::AppState;

/// Names of the three inner tabs, in order (`&wsub=1` is the first).
const WSUBS: [&str; 3] = ["➕ Add a Plant", "📦 Bulk Export 20", "📥 Backfill Import & Review"];

/// `get_future_partials`: wishlist rows missing one of these, by name.
/// Fields a wishlist plant should have. A plant missing any of them is listed
/// for export. (`[&str; 20]` is a fixed-size list of exactly 20 texts.)
const REQUIRED: [&str; 20] = [
    "botanical_name", "family", "genus", "species", "plant_type",
    "zones", "sun_exposure", "water_needs", "bloom_color",
    "foliage_color", "plant_length", "plant_width", "soil_type",
    "deer_resistance", "toxicity", "care_notes", "grow_months",
    "bloom_months", "native_status", "growth_rate",
];

/// Reads every wishlist plant and keeps those with at least one empty field from
/// `REQUIRED`. Each kept row gets `_missing_fields` (the names that are empty).
/// Takes the app state; gives back a list of rows.
async fn future_partials(state: &AppState) -> Vec<Map<String, Value>> {
    // All wishlist rows, sorted by name.
    let all = rows(state, "SELECT * FROM future_plants ORDER BY plant_name").await;
    // `filter_map` keeps a row only when the closure gives `Some(row)`.
    all.into_iter().filter_map(|mut r| {
        // A field is missing when the column exists and its value is empty.
        let missing: Vec<Value> = REQUIRED.iter().filter(|f| r.contains_key(**f) && pipeline::is_empty(r.get(**f))).map(|f| Value::String(f.to_string())).collect();
        // Nothing missing: drop this row.
        if missing.is_empty() {
            return None;
        }
        r.insert("_missing_fields".into(), Value::Array(missing));
        Some(r)
    }).collect()
}

/// Draws the Wishlist tab. `req` holds the address settings (`wsub` = inner
/// tab); `st` is the saved pipeline progress. Gives back HTML text.
pub async fn render(state: &AppState, req: &Req, st: &Map<String, Value>) -> String {
    let mut h = ui::subheader("🌟 Wishlist — Future Plants");
    // The note that wishlist work never touches the main table.
    h.push_str(&format!("<div class=\"st-alert st-info\"><p>🌱</p>{}</div>", ui::md("Backfill placeholder data for plants you **don't** own yet.  Everything here writes **only** to the `future_plants` table — your main `juns_garden_db` / `data_source` is never touched until you *promote* a plant from the Wishlist tab.")));
    // Inner tab number forced into 1..3.
    let w = php_int(req.get("wsub")).clamp(1, 3) as usize;
    // The row of inner-tab links; the current one gets class "on".
    h.push_str("<nav class=\"st-tabs\">");
    for (i, l) in WSUBS.iter().enumerate() {
        h.push_str(&format!("<a href=\"/streamlit/?tab=11&amp;sub=5&amp;wsub={}\"{}>{}</a>", i + 1, if i + 1 == w { " class=\"on\"" } else { "" }, he(l)));
    }
    h.push_str("</nav>");
    // Page so far + the chosen inner tab.
    h + &match w {
        1 => add_form(),
        2 => export_tab(state, req, st).await,
        _ => import_tab(state, req).await,
    }
}

/// Inner tab 1: a form with the basics for a new wishlist plant. Gives back HTML.
fn add_form() -> String {
    // Drop-down choices as owned text, since the form helpers take lists of `String`.
    let types: Vec<String> = opts::PLANT_TYPES.iter().map(|x| x.to_string()).collect();
    let sun: Vec<String> = opts::SUN_OPTIONS.iter().map(|x| x.to_string()).collect();
    // Two columns of inputs, then the submit button.
    let body = ui::columns(&[1, 1], vec![
        ui::text_input("name", "Plant Name *", "", "") + &ui::text_input("company", "Company / Brand", "", "") + &ui::text_input("botanical", "Botanical Name", "", "")
            + &ui::text_input("family", "Family", "", "") + &ui::select_plain("ptype", "Plant Type", &types, &types[0]),
        ui::text_input("zones", "Zones", "", "") + &ui::select_plain("sun", "Sun Exposure", &sun, &sun[0]) + &ui::text_input("bloom", "Bloom Color", "", "")
            + &ui::text_input("cost", "Approx. Cost ($)", "", "") + &ui::text_area("notes", "Notes", "", 80),
    ]) + &ui::button("go", "1", "➕ Add to Wishlist", true, false);
    ui::markdown("Add a plant that isn't in your garden yet.  Only the name is required — fill in whatever you know now and backfill the rest later.")
        // The form posts `act=jp_wl_add`, handled by `action` below.
        + &format!("<div class=\"st-form\">{}</div>", post_form("jp_wl_add", 5, &body, &hidden("wsub", "1")))
}

/// Inner tab 2: pick up to 20 wishlist plants that still have blanks and download
/// them as one JSON file. Takes the app state, the request and the saved
/// progress. Gives back HTML.
async fn export_tab(state: &AppState, req: &Req, st: &Map<String, Value>) -> String {
    let mut h = ui::markdown("Select up to **20** wishlist plants that still have blank fields, then download them as **one JSON file** (an array of up to 20 plant objects).  Fill in the blanks offline and re-upload under **Backfill Import**.");
    // "Reload" button; the list is only read after "Load" has been pressed.
    let reload = post_form("jp_wl_reload", 5, &ui::button("go", "1", "🔄 Reload", false, true), &hidden("wsub", "2"));
    // Not loaded yet: show only the Load and Reload buttons.
    if !flag(st, "wl_loaded") {
        return h + &ui::columns(&[3, 1], vec![post_form("jp_wl_load", 5, &ui::button("go", "1", "🔍 Load Wishlist Plants with Missing Fields", true, true), &hidden("wsub", "2")), reload]);
    }
    h.push_str(&ui::columns(&[3, 1], vec![String::new(), reload]));
    // Plants that have missing fields.
    let partials = future_partials(state).await;
    // None: everything is filled in.
    if partials.is_empty() {
        return h + &ui::success("🎉 Every wishlist plant has its key fields filled in!");
    }
    h.push_str(&ui::caption(&format!("**{}** wishlist plants have missing fields.", partials.len())));
    // Filter text (`wf`), lower case.
    let t = req.get("wf").trim().to_lowercase();
    // Keep plants whose name, company or botanical name contains it.
    let shown: Vec<&Map<String, Value>> = partials.iter().filter(|p| t.is_empty() || ["plant_name", "company", "botanical_name"].iter().any(|k| s(p, k).to_lowercase().contains(&t))).collect();
    // Picker choices: (id, label). The label shows id, name, company, and missing count.
    let opts_: Vec<(String, String)> = shown.iter().map(|p| {
        let co = s(p, "company");
        let n = p.get("_missing_fields").and_then(Value::as_array).map(Vec::len).unwrap_or(0);
        (s(p, "id"), format!("[ID {}] {} ({}) — {n} missing", s(p, "id"), { let x = s(p, "plant_name"); if p.contains_key("plant_name") { x } else { "Unknown".into() } }, if p.get("company").map(|v| v.is_null()).unwrap_or(true) { "None".to_string() } else { co }))
    }).collect();
    // The ids already picked (in the address), at most 20.
    let picked: Vec<String> = ids_of(req.get("wpick[]")).into_iter().take(20).collect();
    // A GET form that reloads the page on any change, keeping the tab and inner tab.
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"5\"><input type=\"hidden\" name=\"wsub\" value=\"2\">{}{}</form>",
        ui::text_input("wf", "🔍 Filter", req.get("wf"), "Name, company, botanical…"),
        pick_box("wpick[]", &format!("Select plants to export (max 20 — {} shown):", shown.len()), &opts_, &picked, 20)));
    // The picked plants, looked up in the shown list.
    let chosen: Vec<&&Map<String, Value>> = picked.iter().filter_map(|id| shown.iter().find(|r| s(r, "id") == *id)).collect();
    // None picked yet.
    if chosen.is_empty() {
        return h + &ui::info("Select up to 20 plants above to generate the export.");
    }
    // File name and ids for the download link, answered by `export` below.
    let fname = format!("wishlist_export_{}plants_{}.json", chosen.len(), stamp());
    let ids = chosen.iter().map(|r| s(r, "id")).collect::<Vec<_>>().join(",");
    h.push_str(&format!("<a class=\"st-btn st-primary st-full\" href=\"/streamlit/?export=jp_wl&amp;ids={ids}&amp;fn={}\">{}</a>", crate::php::urlencode(&fname),
        he(&format!("⬇️ Download {fname}  ({} plant{})", chosen.len(), if chosen.len() != 1 { "s" } else { "" }))));
    h + &ui::caption("Fill in the blanks, then re-upload under **Backfill Import & Review**.")
}

/// Builds the wishlist export. `ids` is a comma list of ids and `fname` the
/// wanted file name. Gives (file text, file name). Called from `export` in
/// pipeline.rs when the address says `export=jp_wl`.
pub async fn export(state: &AppState, ids: &str, fname: &str) -> Option<(String, String)> {
    // Only digit-only ids are kept.
    let ids = ids_of(ids);
    let partials = future_partials(state).await;
    // Up to 20 chosen rows, without the large QR/barcode columns.
    let chosen: Vec<Value> = ids.iter().take(20).filter_map(|id| partials.iter().find(|r| s(r, "id") == *id)).map(|r| {
        let mut c = r.clone();
        c.remove("qr_code");
        c.remove("barcode_image");
        Value::Object(c)
    }).collect();
    // Use the wanted file name only if it has safe characters; else make one.
    let name = if !fname.is_empty() && fname.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')) { fname.to_string() } else { format!("wishlist_export_{}plants_{}.json", chosen.len(), stamp()) };
    Some((dumps(&Value::Array(chosen)), name))
}

/// Inner tab 3: upload a filled-in file, show what would change in each wishlist
/// row, and offer commit buttons. Takes the app state and the request. Gives HTML.
async fn import_tab(state: &AppState, req: &Req) -> String {
    // `wup` is the uploaded file's token, kept in the address.
    let wup = req.get("wup");
    let mut h = ui::markdown("Upload a wishlist JSON (from the export above or hand-edited).  Changes are compared against the current `future_plants` row.  **Nothing is written to the main database** — commits go to `future_plants` only.")
        + &uploader("jp_wl_upload", 5, "Upload wishlist JSON", &hidden("wsub", "3"));
    // No file yet: show a hint.
    let Some(records) = load_upload(state, wup) else {
        return h + &ui::info("Upload a JSON file above to begin the review.");
    };
    h.push_str(&ui::success(&format!("Loaded **{}** wishlist records.", records.len())));
    // All wishlist rows by id, for comparing.
    let db: HashMap<String, Map<String, Value>> = rows(state, "SELECT * FROM future_plants").await.into_iter().map(|r| (s(&r, "id"), r)).collect();
    // Records that would change something: (position, diff, name, id).
    let committable: Vec<(usize, pipeline::Diff, String, String)> = records.iter().enumerate().filter_map(|(i, r)| {
        // `?` here skips records whose id is not in the wishlist.
        let d = db.get(&s(r, "id"))?;
        let df = diff(r, d);
        let name = if r.contains_key("plant_name") { s(r, "plant_name") } else { format!("ID {}", s(r, "id")) };
        (!df.0.is_empty() || !df.1.is_empty()).then(|| (i, df, name, s(r, "id")))
    }).collect();
    if committable.is_empty() {
        return h + &ui::info("No differences between the uploaded JSON and the current wishlist rows.");
    }
    // Totals: new fills (tn) and overwrites (to) across all records.
    let (tn, to) = committable.iter().fold((0, 0), |(a, b), (_, (n, o), _, _)| (a + n.len(), b + o.len()));
    // `waow=1` means the box "Also accept ALL overwrites" is ticked.
    let aow = req.get("waow") == "1";
    let note = if aow { format!(" + {to} overwrites") } else { String::new() };
    let extra = hidden("wsub", "3") + &hidden("wup", wup) + &hidden("waow", req.get("waow"));
    // "Approve ALL" button (future_plants only).
    let all_btn = post_form("jp_wl_all", 5, &ui::button("go", "1", &format!("✅ Approve ALL — commit {tn} new fills{note} across {} plants (future_plants only)", committable.len()), true, true), &extra);
    // The tick box reloads the page with `waow` set.
    let aow_box = format!("<form method=\"get\" action=\"/streamlit/\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"5\"><input type=\"hidden\" name=\"wsub\" value=\"3\">{}{}</form>",
        hidden("wup", wup), ui::checkbox("waow", "Also accept ALL overwrites", aow, None));
    h.push_str(&ui::columns(&[3, 2], vec![all_btn, aow_box]));
    h.push_str(&ui::divider());
    // One expander per record, with its review inside.
    for (i, df, name, id) in &committable {
        // Same review as Phase 3, with the wishlist's own words on the button.
        // The review HTML comes from pipeline.rs; the text below is replaced so the
        // forms use the wishlist's field names (`wup`, `wsub`) and wording.
        let body = review_body(df, *i, wup, name, 5, "jp_wl_commit")
            .replace("<input type=\"hidden\" name=\"up\"", "<input type=\"hidden\" name=\"wup\"")
            .replace("— fields that were empty in the DB:", "(was empty):")
            .replace("<strong>⚠️ Proposed overwrites</strong> — fields that already have a DB value:", "<strong>⚠️ Proposed overwrites:</strong>")
            .replace(&format!("✅ Approve &amp; Commit  ({} new fill{})", df.0.len(), if df.0.len() != 1 { "s" } else { "" }), &format!("💾 Commit {} field(s)", df.0.len()))
            .replace("<input type=\"hidden\" name=\"sub\" value=\"5\">", "<input type=\"hidden\" name=\"sub\" value=\"5\"><input type=\"hidden\" name=\"wsub\" value=\"3\">");
        h.push_str(&ui::expander(&format!("🌿 {name}  (future_plants ID {id})"), false, &body));
    }
    h
}

/// Handles every wishlist button (POST). `act` names the button; `f` is the
/// submitted form. Gives back the messages to show. Called from `action` in
/// pipeline.rs for any button name that starts with `jp_wl_`.
pub async fn action(state: &AppState, act: &str, f: &crate::formdata::FormData) -> Vec<Flash> {
    match act {
        // Remember whether the export list has been loaded.
        "jp_wl_load" => { set_flag(state, "wl_loaded", Value::Bool(true)); vec![] }
        "jp_wl_reload" => { set_flag(state, "wl_loaded", Value::Bool(false)); vec![] }
        // Add a plant to the wishlist.
        "jp_wl_add" => {
            // Only the name is required.
            let name = f.str("name").trim().to_string();
            if name.is_empty() {
                return vec![Flash::Error("Plant name is required.".into())];
            }
            // `out` collects the messages to show.
            let mut out = Vec::new();
            // Warn (but do not block) if the garden already has this name.
            if all_plant_names(state).await.contains(&name) {
                out.push(Flash::Warning(format!("⚠️ '{name}' already exists in your main database (data_source).  Adding it to the wishlist anyway — it won't be duplicated into the main DB.")));
            }
            // `blank`: trimmed text, or None when empty (so the database stores NULL).
            let blank = |k: &str| { let v = f.str(k).trim(); if v.is_empty() { None } else { Some(v.to_string()) } };
            // Cost as a number; text that is not a number becomes NULL.
            let cost: Option<f64> = f.str("cost").trim().parse().ok();
            // `?` marks are filled by `.bind(..)`, so the typed text can never change the SQL.
            let r = sqlx::query("INSERT INTO future_plants (plant_name, company, botanical_name, family, plant_type, zones, sun_exposure, bloom_color, extra_details, average_cost) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&name).bind(blank("company")).bind(blank("botanical")).bind(blank("family")).bind(blank("ptype")).bind(blank("zones"))
                .bind(blank("sun")).bind(blank("bloom")).bind(blank("notes")).bind(cost)
                .execute(&state.db).await;
            // Show success with the new id, a friendly duplicate message, or the raw error.
            out.push(match r {
                Ok(r) => Flash::Success(format!("✅ Added '{name}' to the wishlist (future_plants ID {}).", r.last_insert_id())),
                Err(e) if e.to_string().contains("Duplicate") => Flash::Error("Could not add: A wishlist plant with that name + company already exists".into()),
                Err(e) => Flash::Error(format!("Could not add: {e}")),
            });
            out
        }
        // Commit every uploaded record's new fills (and all overwrites if ticked).
        "jp_wl_all" => {
            let recs = load_upload(state, f.str("wup")).unwrap_or_default();
            let db: HashMap<String, Map<String, Value>> = rows(state, "SELECT * FROM future_plants").await.into_iter().map(|r| (s(&r, "id"), r)).collect();
            // `ok` = plants saved, `n` = fields saved, `errs` = error messages.
            let (mut ok, mut n, mut errs) = (0, 0, Vec::new());
            for r in &recs {
                // Skip records not in the wishlist.
                let Some(d) = db.get(&s(r, "id")) else { continue };
                let (mut fields, ow) = diff(r, d);
                // The tick box adds every overwrite too.
                if f.str("waow") == "1" {
                    for (k, _, jv) in ow { fields.insert(k, jv); }
                }
                if fields.is_empty() { continue; }
                // `true` = write empty strings as NULL (the wishlist rule).
                match pipeline::commit(state, "future_plants", php_int(&s(r, "id")), &fields, true).await {
                    Ok(_) => { ok += 1; n += fields.len(); }
                    Err(e) => errs.push(format!("{}: {e}", s(r, "plant_name"))),
                }
            }
            let mut v = vec![Flash::Success(format!("✅ Committed {n} field(s) across {ok} wishlist plant(s).{}", if errs.is_empty() { String::new() } else { format!("  ⚠️ {} failed.", errs.len()) }))];
            v.extend(errs.into_iter().take(10).map(Flash::Error));
            v
        }
        // Commit or skip one record; only ticked overwrites (`acc[]`) are applied.
        "jp_wl_commit" => {
            if f.str("do") == "skip" {
                return vec![Flash::Info(format!("Skipped **{}**.", f.str("name")))];
            }
            let recs = load_upload(state, f.str("wup")).unwrap_or_default();
            // The position (`idx`) of the record in the uploaded file.
            let Some(r) = recs.get(php_int(f.str("idx")).max(0) as usize) else { return vec![Flash::Error("That record is no longer loaded.".into())] };
            let id = php_int(&s(r, "id"));
            // Re-read the row so the comparison is with the current data.
            let Some(d) = rows(state, &format!("SELECT * FROM future_plants WHERE id = {id}")).await.into_iter().next() else { return vec![Flash::Error("Failed: not found".into())] };
            let (mut fields, ow) = diff(r, &d);
            // Names of the overwrites the user ticked.
            let acc: Vec<&str> = f.str("acc[]").split(", ").collect();
            for (k, _, jv) in ow {
                if acc.contains(&k.as_str()) { fields.insert(k, jv); }
            }
            let n = fields.len();
            match pipeline::commit(state, "future_plants", id, &fields, true).await {
                Ok(_) => vec![Flash::Success(format!("✅ Saved {n} field(s) to wishlist."))],
                Err(e) => vec![Flash::Error(format!("Failed: {e}"))],
            }
        }
        _ => vec![Flash::Error("Unknown action.".into())],
    }
}
