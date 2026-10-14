//! "🔄 JSON Pipeline" (tab_json_pipeline.py): six inner tabs, `&sub=1..6`.
//!
//! HOW TO READ THIS FILE (for Jun). The pipeline moves plant data out to a
//! JSON file and back in:
//!   1 Phase 1   export one incomplete plant at a time, "Mark as Done";
//!   2 Phase 2   export up to 5 incomplete plants in one file;
//!   3 Enrich    let the free AI fill blanks for many plants at once
//!               (runs in the background; the page reloads until it is done);
//!   4 Phase 3   upload a JSON file, see what it would change, commit;
//!   5 Wishlist  the same three steps for future_plants (pipeline_wl.rs);
//!   6 Reference every field, its type and allowed values (pipeline_ref.rs).
//! The LOCK RULE everywhere: a field that already has a value is never
//! overwritten unless you tick that overwrite yourself.
//! What Streamlit kept "for the session" (the Phase 1 done list, the Enrich
//! results) is kept in data/orchestrator-ai/pipeline.json; uploaded files in
//! data/orchestrator-ai/upload-<token>.json.
//!
//! Rust ideas used often in this file (explained once here):
//!   - `async fn` / `.await`: a function that may wait (for the database or
//!     the disk). `.await` pauses this function until the wait is over.
//!   - `String` is text the code owns; `&str` is a borrowed look at text.
//!   - `Option<T>` is "a value or nothing" (`Some(x)` / `None`).
//!   - `Result<T, E>` is "worked" (`Ok(x)`) or "failed" (`Err(e)`).
//!   - `?` after a call means: if it failed or is empty, return that from
//!     this function right now; otherwise carry on with the good value.
//!   - `match` picks one branch by the shape of a value.
//!   - `Map<String, Value>` is one JSON object (a plant row: field -> value).
//!     `Value` is any JSON value (text, number, true/false, list, object).
//!   - `h.push_str(..)` adds text to the end of the page text `h`.
//! HTML is built as plain text with `format!`. `{}` in the text is filled
//! with the next argument, and `{name}` with the variable `name`.
//!
//! DIFFERENCES FROM STREAMLIT (DEVIATIONS.md): imports only write columns
//! the table really has (S-13); an AI failure during Enrich is listed as an
//! error instead of silently filling nothing (S-14).

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Map, Value};

use super::ai;
use super::form::Flash;
use super::opts::{self, text};
use super::ui;
use super::{rows, Req};
use crate::php::{he, php_int};
use crate::AppState;

/// The names of the six inner tabs, in order (`&sub=1` is the first).
/// `[&str; 6]` is a fixed-size array holding exactly 6 text values.
pub const SUBS: [&str; 6] = ["📤 Phase 1 — Export", "📦 Phase 2 — Bulk Export 5", "🤖 Phase 2 — Batch Enrich", "📥 Phase 3 — Import & Review", "🌟 Wishlist (Future Plants)", "📋 Field Reference"];

// ---- shared helpers (also used by pipeline_wl.rs) -------------------------

/// Reads field `k` of a plant row as plain text ("" when missing or null).
/// Short name because it is used everywhere in these files.
pub fn s(m: &Map<String, Value>, k: &str) -> String {
    text(m.get(k))
}

/// `_is_empty`: only blank or placeholder-unknown values count as empty.
/// Takes an optional JSON value; gives back true when it counts as empty.
/// Empty means: missing, null, an empty list, or placeholder text such as
/// "unknown" or a row of dots (an unset month code).
pub fn is_empty(v: Option<&Value>) -> bool {
    match v {
        // Nothing there at all, or an explicit null.
        None | Some(Value::Null) => true,
        // A list is empty when it has no items.
        Some(Value::Array(a)) => a.is_empty(),
        // Anything else: compare its text (trimmed, lower case) with the placeholders.
        Some(x) => matches!(text(Some(x)).trim().to_lowercase().as_str(), "" | "null" | "[]" | "............" | "unknown" | "unknown only"),
    }
}

/// Python's `str(value)` for the review screens.
/// Takes a JSON value; gives back text shaped the way Python would print it
/// (None, True, False, ['a', 'b']) so the screens look like the Streamlit app.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::Null => "None".into(),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::Array(a) => format!("[{}]", a.iter().map(|x| match x { Value::String(s) => format!("'{s}'"), o => py_str(o) }).collect::<Vec<_>>().join(", ")),
        other => text(Some(other)),
    }
}

/// Where the pipeline's saved progress lives: data/orchestrator-ai/pipeline.json.
/// `PathBuf` is an owned file path. `state.data` is the app's data folder.
fn state_path(state: &AppState) -> std::path::PathBuf {
    state.data.join("orchestrator-ai").join("pipeline.json")
}

/// Reads the saved pipeline progress. Gives back an empty map when the file
/// is missing or broken, so the pages still draw.
pub fn load_state(state: &AppState) -> Map<String, Value> {
    // read file -> bytes -> JSON. `.ok()` turns a failure into "nothing",
    // and `.and_then(..)` runs the next step only if the earlier one worked.
    match std::fs::read(state_path(state)).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()) {
        Some(Value::Object(m)) => m,
        _ => Map::new(),
    }
}

/// Saves the pipeline progress. It writes to a temporary file and then
/// renames it, so a crash half way never leaves a broken pipeline.json.
/// Failures are ignored (`let _ =`): losing a progress note is not serious.
pub fn save_state(state: &AppState, m: &Map<String, Value>) {
    let p = state_path(state);
    // Make sure the folder exists.
    let _ = std::fs::create_dir_all(p.parent().unwrap_or(&state.data));
    // Same name with the extension changed to .tmp.
    let tmp = p.with_extension("tmp");
    if std::fs::write(&tmp, Value::Object(m.clone()).to_string()).is_ok() {
        let _ = std::fs::rename(tmp, p);
    }
}

/// Reads a true/false setting `k` from the saved progress. Missing counts as false.
pub fn flag(st: &Map<String, Value>, k: &str) -> bool {
    st.get(k).and_then(Value::as_bool).unwrap_or(false)
}

/// Loads the progress file, sets one value, and saves it again.
/// Used by the "Load" and "Reload" buttons to remember what was pressed.
pub fn set_flag(state: &AppState, k: &str, v: Value) {
    let mut st = load_state(state);
    st.insert(k.into(), v);
    save_state(state, &st);
}

/// Where an uploaded JSON file is kept: upload-<token>.json.
/// Gives back `None` unless the token is only letters and digits. This
/// matters for safety: it stops a token such as "../x" from reaching other
/// folders. `.then(|| ..)` builds the path only when the test is true.
fn upload_path(state: &AppState, token: &str) -> Option<std::path::PathBuf> {
    (!token.is_empty() && token.chars().all(|c| c.is_ascii_alphanumeric()))
        .then(|| state.data.join("orchestrator-ai").join(format!("upload-{token}.json")))
}

/// An uploaded JSON file, kept until the review is done.
/// Takes the token from the page's hidden field. Gives back `None` when the
/// token is bad, the file is gone, or the file is not a JSON list.
pub fn load_upload(state: &AppState, token: &str) -> Option<Vec<Map<String, Value>>> {
    // Each `?` here gives up with `None` if that step fails.
    let v: Value = serde_json::from_slice(&std::fs::read(upload_path(state, token)?).ok()?).ok()?;
    // Keep only the list items that are objects; copy them (`cloned`).
    Some(v.as_array()?.iter().filter_map(|r| r.as_object().cloned()).collect())
}

/// `_load_json_upload`: store the file and hand back its token.
/// Takes the file picked in the upload form. Gives back the new token, or an
/// error message to show the user.
fn store_upload(state: &AppState, f: &crate::formdata::FormData) -> Result<String, String> {
    // The form field is named `json`. `let .. else` leaves early when no file came.
    let Some(u) = f.files_for("json").into_iter().next() else { return Err("Choose a JSON file first.".into()) };
    // Parse the bytes as JSON; `map_err` changes the error into a friendly message.
    let v: Value = serde_json::from_slice(&u.bytes).map_err(|e| format!("Could not parse JSON: {e}"))?;
    // The Streamlit rule: the file must be a list of plant objects.
    if !v.is_array() {
        return Err("JSON must be a top-level array of plant objects.".into());
    }
    // 8 random bytes written as 16 hex characters make a hard-to-guess token.
    let b: [u8; 8] = rand::random();
    let token: String = b.iter().map(|x| format!("{x:02x}")).collect();
    // Build the path (this also re-checks the token is safe).
    let p = upload_path(state, &token).ok_or("bad token")?;
    // Create the folder, then write the file; any error becomes a text message.
    std::fs::create_dir_all(p.parent().unwrap_or(&state.data)).map_err(|e| e.to_string())?;
    std::fs::write(p, v.to_string()).map_err(|e| e.to_string())?;
    Ok(token)
}

/// Fields an import never writes: ids, helper fields added by the pipeline,
/// and fields owned by other tools (pictures, quantity, approvals).
/// Any field starting with `_` is also skipped (see `diff`).
const IMPORT_SKIP: [&str; 15] = ["id", "_missing_fields", "_enriched_fields", "_proposed_changes", "plant_name", "picture", "quantity", "year_added", "garden_beds", "approved_jason", "approved_jun", "needs_review", "image_verified", "image_search_term", "date_added"];

/// The result of comparing an uploaded record with the database row:
/// (new fills as field -> value, overwrites as (field, db value, json value)).
pub type Diff = (Map<String, Value>, Vec<(String, Value, Value)>);

/// `_compute_record_diff`: (new fills, overwrites as (field, db, json)).
/// This is the LOCK RULE: a value already in the database is never replaced
/// unless the user approves it.
pub fn diff(rec: &Map<String, Value>, db: &Map<String, Value>) -> Diff {
    // `new` = fields empty in the DB but filled in the file; `ow` = changed values.
    let (mut new, mut ow) = (Map::new(), Vec::new());
    // Look at every field of the uploaded record.
    for (k, jv) in rec {
        // Skip fields the import must not touch.
        if IMPORT_SKIP.contains(&k.as_str()) || k.starts_with('_') {
            continue;
        }
        // The database value for this field (null when the column is missing).
        let dv = db.get(k).cloned().unwrap_or(Value::Null);
        // DB empty and file has a value: a safe new fill.
        if is_empty(Some(&dv)) && !is_empty(Some(jv)) {
            new.insert(k.clone(), jv.clone());
        // Both have values: it is an overwrite only if they differ.
        } else if !is_empty(Some(&dv)) && !is_empty(Some(jv)) {
            // If both look like numbers, compare them as numbers so "8.990" equals 8.99.
            // Otherwise compare the trimmed text.
            let equal = match (text(Some(&dv)).trim().parse::<f64>(), text(Some(jv)).trim().parse::<f64>()) {
                (Ok(a), Ok(b)) => a == b,
                _ => text(Some(&dv)).trim() == text(Some(jv)).trim(),
            };
            if !equal {
                ow.push((k.clone(), dv, jv.clone()));
            }
        }
    }
    // Give back both lists.
    (new, ow)
}

/// grow/bloom months from a list or a "[...]" string to the 12-letter code.
/// A list such as [5, 6] means May and June; the code is "....MJ......".
/// Gives `None` when the value is not a month list.
fn month_code(v: &Value) -> Option<String> {
    // Get a list of month numbers from either a real list or text that looks like one.
    let list = match v {
        Value::Array(a) => a.clone(),
        Value::String(s) if s.starts_with('[') => serde_json::from_str::<Vec<Value>>(s).ok()?,
        _ => return None,
    };
    // Month numbers may be numbers or text; `opts::format_m` makes the 12 letters.
    Some(opts::format_m(&list.iter().filter_map(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.trim().parse().ok()))).map(|m| m as u32).collect::<Vec<_>>()))
}

/// `_format_month_display`: a month bar for grow/bloom, else the text.
/// Draws a field's value for the review screens. Month fields become a row of
/// coloured letters (green for grow, pink for bloom); all other fields are
/// shown as escaped text cut to 200 characters.
pub fn month_display(field: &str, v: &Value) -> String {
    // A small helper (closure) that makes safe, shortened text.
    // `he` turns characters such as < and & into HTML-safe forms.
    let short = |v: &Value| he(&py_str(v).chars().take(200).collect::<String>());
    if field != "grow_months" && field != "bloom_months" {
        return short(v);
    }
    // Turn the value into a 12-letter code; else accept a ready 12-character string.
    let bitmap = match month_code(v) {
        Some(b) => b,
        None => match v { Value::String(s) if s.chars().count() == 12 => s.clone(), _ => return short(v) },
    };
    // Green for growing months, pink for blooming months.
    let color = if field == "grow_months" { "#2e7d32" } else { "#d81b60" };
    // First letter of each month.
    let letters = ['J', 'F', 'M', 'A', 'M', 'J', 'J', 'A', 'S', 'O', 'N', 'D'];
    // One small box per month; a dot in the code means "not active" (grey dot).
    bitmap.chars().enumerate().map(|(i, c)| if c != '.' {
        format!("<span style='display:inline-block;width:22px;text-align:center;background:{color};color:#fff;border-radius:3px;margin:0 1px;font-size:11px;font-weight:700;padding:2px 0;'>{}</span>", letters.get(i).unwrap_or(&'?'))
    } else {
        "<span style='display:inline-block;width:22px;text-align:center;color:#ccc;font-size:11px;padding:2px 0;margin:0 1px;'>·</span>".to_string()
    }).collect()
}

/// The green "New fills" table.
pub fn fills_table(new: &Map<String, Value>) -> String {
    // One table row per new field, with a green background.
    let rows: String = new.iter().map(|(f, v)| format!("<tr style='background:#d4edda'><td style='padding:5px 10px;font-weight:600;color:#155724;width:30%'>{}</td><td style='padding:5px 10px;color:#155724'>{}</td></tr>", he(f), month_display(f, v))).collect();
    format!("<table style='width:100%;border-collapse:collapse;font-size:13px'>{rows}</table>")
}

/// Write {field: value} to one row: only real columns, lists joined, months
/// as the 12-letter code. `blank_null` is future_plants' '' -> NULL rule.
/// Returns how many columns were written, or an error message.
/// `table` is data_source or future_plants. The SQL is built from column
/// names that were checked against the real table, and the values are sent
/// separately with `.bind` so they can never be run as SQL.
pub async fn commit(state: &AppState, table: &str, id: i64, fields: &Map<String, Value>, blank_null: bool) -> Result<usize, String> {
    // Ask the database which columns this table really has (S-13 in DEVIATIONS.md).
    // DATABASE() means the database this connection uses.
    let cols: Vec<String> = sqlx::query_scalar("SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?")
        .bind(table).fetch_all(&state.db).await.map_err(|e| e.to_string())?;
    // Keep only real columns, never id or date_added; fix up each value.
    let clean: Vec<(&String, Value)> = fields.iter().filter(|(k, _)| cols.contains(k) && !matches!(k.as_str(), "id" | "date_added")).map(|(k, v)| {
        // Month fields are stored as the 12-letter code.
        let v = if matches!(k.as_str(), "grow_months" | "bloom_months") { month_code(v).map(Value::String).unwrap_or_else(|| v.clone()) } else { v.clone() };
        // Lists become "a, b, c" text, objects become JSON text, and for the
        // wishlist an empty string becomes NULL (the `blank_null` rule).
        let v = match v {
            Value::Array(a) => Value::String(a.iter().filter(|x| !x.is_null() && text(Some(x)) != "").map(|x| text(Some(x))).collect::<Vec<_>>().join(", ")),
            Value::Object(_) => Value::String(v.to_string()),
            Value::String(s) if s.is_empty() && blank_null => Value::Null,
            o => o,
        };
        (k, v)
    }).collect();
    // Nothing valid to write.
    if clean.is_empty() {
        return Err("No valid columns to update".into());
    }
    // Build: UPDATE `table` SET `a` = ?, `b` = ? WHERE id = ?  (the ? are filled in below).
    let sql = format!("UPDATE `{table}` SET {} WHERE id = ?", clean.iter().map(|(k, _)| format!("`{k}` = ?")).collect::<Vec<_>>().join(", "));
    let mut q = sqlx::query(&sql);
    // Bind each value in order, using the type that matches the JSON value.
    for (_, v) in &clean {
        q = match v {
            Value::Null => q.bind(None::<String>),
            Value::Number(n) if n.is_i64() => q.bind(n.as_i64()),
            Value::Number(n) => q.bind(n.as_f64()),
            Value::Bool(b) => q.bind(*b as i64),
            o => q.bind(text(Some(o))),
        };
    }
    // Last bind is the id for WHERE; then run it.
    q.bind(id).execute(&state.db).await.map_err(|e| e.to_string())?;
    // The home-page widgets cache plant data, so tell them it is out of date.
    state.invalidate_widget_cache();
    Ok(clean.len())
}

/// Python's `json.dumps(..., indent=2, ensure_ascii=False)`.
pub fn dumps(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default()
}

/// A date-time stamp like 20261007_142530, used in download file names.
pub fn stamp() -> String {
    let n = common::now_local();
    format!("{}{:02}{:02}_{:02}{:02}{:02}", n.year(), u8::from(n.month()), n.day(), n.hour(), n.minute(), n.second())
}

/// A row as Phase 1-3 export it: every column plus `_missing_fields`.
// Adds `_missing_fields` to the row: the list of fields that need filling.
fn with_missing(mut r: Map<String, Value>) -> Map<String, Value> {
    let missing: Vec<Value> = opts::missing_fields(&r).into_iter().map(|f| Value::String(f.into())).collect();
    r.insert("_missing_fields".into(), Value::Array(missing));
    r
}

/// `_fetch_partials`: every plant with a missing field.
// `order` is extra SQL text such as " ORDER BY plant_name ASC" (or empty).
// It is only ever a fixed string written in this file, never user input.
async fn partials(state: &AppState, order: &str) -> Vec<Map<String, Value>> {
    rows(state, &format!("SELECT * FROM data_source{order}")).await.into_iter().filter(|r| !opts::missing_fields(r).is_empty()).map(with_missing).collect()
}

/// How many fields a row says are missing (0 if the list is absent).
fn missing_n(r: &Map<String, Value>) -> usize {
    r.get("_missing_fields").and_then(Value::as_array).map(Vec::len).unwrap_or(0)
}

/// `st.multiselect` whose values are ids, with Streamlit's max_selections.
/// Draws a multi-pick box whose values are plant ids. `max` limits how many can
/// be chosen (0 means no limit). `st.js` adds the chip display and the limit.
pub fn pick_box(name: &str, label: &str, opts_: &[(String, String)], selected: &[String], max: usize) -> String {
    // `data-ms` tells st.js to turn this <select> into the chip box.
    let mut s = format!("<label class=\"st-field\"><span class=\"st-label\">{}</span><select class=\"st-multi\" multiple data-ms data-max=\"{max}\" name=\"{}\">", he(label), he(name));
    for (v, t) in opts_ {
        s.push_str(&format!("<option value=\"{}\"{}>{}</option>", he(v), if selected.contains(v) { " selected" } else { "" }, he(t)));
    }
    s + "</select></label>"
}

/// Wraps `body` in a POST form that sends `act` (which button), `tab=11` and `sub`.
/// `extra` is more hidden inputs. `enctype=multipart` allows file uploads.
pub fn post_form(act: &str, sub: usize, body: &str, extra: &str) -> String {
    format!("<form method=\"post\" action=\"/streamlit/\" enctype=\"multipart/form-data\"><input type=\"hidden\" name=\"act\" value=\"{act}\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"{sub}\">{extra}{body}</form>")
}

/// A hidden form input, or nothing when the value is empty.
pub fn hidden(k: &str, v: &str) -> String {
    if v.is_empty() { String::new() } else { format!("<input type=\"hidden\" name=\"{k}\" value=\"{}\">", he(v)) }
}

/// The file drop zone, sent as soon as a file is chosen (Streamlit reads an
/// upload the moment it lands).
pub fn uploader(act: &str, sub: usize, label: &str, extra: &str) -> String {
    format!("<form method=\"post\" action=\"/streamlit/\" enctype=\"multipart/form-data\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"act\" value=\"{act}\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"{sub}\">{extra}{}</form>",
        ui::file_uploader("json", label, &["json"], false))
}

/// Turns "3, 5,x" into ["3", "5"]: comma-separated, trimmed, digits only.
pub fn ids_of(v: &str) -> Vec<String> {
    v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit())).collect()
}

// ---- the tab ----------------------------------------------------------------

/// Draws the whole JSON Pipeline tab. `req` holds `sub` (which inner tab).
/// Gives back HTML text. It loads the saved progress once, then asks the
/// right inner-tab function to draw itself.
pub async fn render(state: &AppState, req: &Req) -> String {
    // Page heading and intro box.
    let mut h = ui::header("🔄 JSON Export / Enrich / Import Pipeline");
    h.push_str(&format!("<div class=\"st-alert st-info\"><p>🔄</p>{}</div>", ui::md("**Four-phase workflow.**  **Phase 1** exports one plant at a time.  **Phase 2** bulk-exports up to 5 plants as one JSON for manual editing.  **Phase 2 Enrich** runs Claude + Perenual AI in-app.  **Phase 3** uploads any JSON and lets you review every change before committing.")));
    // Inner tab number, forced into the range 1..6 (default 1).
    let sub = php_int(req.get("sub")).clamp(1, 6) as usize;
    // The row of inner-tab links; the current one gets class "on".
    h.push_str("<nav class=\"st-tabs\">");
    for (i, l) in SUBS.iter().enumerate() {
        h.push_str(&format!("<a href=\"/streamlit/?tab=11&amp;sub={}\"{}>{}</a>", i + 1, if i + 1 == sub { " class=\"on\"" } else { "" }, he(l)));
    }
    h.push_str("</nav>");
    // Saved progress (loaded flags, done list, enrich results).
    let st = load_state(state);
    // Page so far + the chosen inner tab. `match sub` picks one branch.
    h + &match sub {
        1 => phase1(state, req, &st).await,
        2 => phase2_export(state, req, &st).await,
        3 => enrich(state, &st).await,
        4 => phase3(state, req).await,
        5 => super::pipeline_wl::render(state, req, &st).await,
        _ => super::pipeline_ref::render(req),
    }
}

/// Phase 1: export ONE incomplete plant at a time. Shows a progress bar,
/// a plant picker, the missing fields, a download link and a "Mark as Done"
/// tick box. `st` is the saved progress. Gives back HTML.
async fn phase1(state: &AppState, req: &Req, st: &Map<String, Value>) -> String {
    let mut h = ui::subheader("📤 Export Plants with Missing Fields — One at a Time")
        + &ui::markdown("Works through every incomplete plant one by one.  Export the plant's JSON, then check **Mark as Done** to remove it from the queue and move to the next one.  Your progress is remembered for the session.");
    // "Reload" clears the done list; "Load" turns the tab on. Nothing is queried until Load.
    let reload = post_form("jp_p1_reset", 1, &ui::button("go", "1", "🔄 Reload List", false, true), "");
    if !flag(st, "p1_loaded") {
        return h + &ui::columns(&[3, 1], vec![post_form("jp_p1_load", 1, &ui::button("go", "1", "🔍 Load Plants with Missing Fields", true, true), ""), reload]);
    }
    h.push_str(&ui::columns(&[3, 1], vec![String::new(), reload]));
    // Ids already marked done this session.
    let done: Vec<i64> = st.get("p1_done").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    // The queue: incomplete plants that are not done yet.
    let queue: Vec<Map<String, Value>> = partials(state, "").await.into_iter().filter(|p| !done.contains(&php_int(&s(p, "id")))).collect();
    // Percent done for the progress bar (whole-number maths).
    let total = queue.len() + done.len();
    if total > 0 {
        let pct = done.len() * 100 / total;
        h.push_str(&format!("<p class=\"st-text\">✅ {} done · {} remaining  ({total} total)</p><div class=\"st-progress\"><div style=\"width:{pct}%\"></div></div>", done.len(), queue.len()));
    }
    if queue.is_empty() {
        return h + &ui::success("🎉 All plants have been marked as done for this session!") + &ui::caption("Click **Reload List** to start over or re-check for new partials.");
    }
    // Choices for the drop-down: text shown, and the id as its value.
    let opts_: Vec<(String, String)> = queue.iter().map(|p| (s(p, "id"), format!("{}  ({} missing)  [ID {}]", s(p, "plant_name"), missing_n(p), s(p, "id")))).collect();
    // The chosen plant (`p` in the address), or the first in the queue.
    let plant = queue.iter().find(|p| s(p, "id") == req.get("p")).unwrap_or(&queue[0]);
    let (id, name) = (s(plant, "id"), s(plant, "plant_name"));
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"1\">{}</form>", ui::selectbox("p", "🌿 Choose a plant to export:", &opts_, &id, true)));
    h.push_str(&ui::divider());
    // Names of this plant's missing fields.
    let missing: Vec<String> = plant.get("_missing_fields").and_then(Value::as_array).map(|a| a.iter().map(|x| text(Some(x))).collect()).unwrap_or_default();
    h.push_str(&ui::columns(&[4, 1], vec![ui::subheader(&format!("🌿 {name}")), ui::metric("Missing fields", &missing.len().to_string())]));
    if !missing.is_empty() {
        h.push_str(&ui::markdown(&format!("**Fields that need filling:** {}", missing.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join("  "))));
    }
    h.push_str(&ui::divider());
    // The download link is answered by `export` below (kind jp_p1).
    let dl = format!("<a class=\"st-btn st-primary st-full\" href=\"/streamlit/?export=jp_p1&amp;id={}\">{}</a>", he(&id), ui::md(&format!("⬇️  Download JSON for {name}")));
    // Ticking the box submits at once (onchange) and adds the id to the done list.
    let mark = format!("<form method=\"post\" action=\"/streamlit/\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"act\" value=\"jp_p1_done\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"1\"><input type=\"hidden\" name=\"id\" value=\"{}\">{}</form>",
        he(&id), ui::checkbox("done", "✅ Mark as Done", false, Some("Remove this plant from the queue.")));
    h.push_str(&ui::columns(&[2, 1], vec![dl, mark]));
    // A preview table of all current values; internal helper fields are hidden.
    let t: Vec<Vec<String>> = plant.iter().filter(|(k, _)| !matches!(k.as_str(), "_missing_fields" | "_enriched_fields" | "_proposed_changes"))
        .map(|(k, v)| vec![ui::cell(k), ui::cell(&if v.is_null() { "—".to_string() } else { py_str(v) }), ui::cell(if missing.contains(k) { "⚠️" } else { "✓" })]).collect();
    h.push_str(&ui::expander("🔍 Preview all current field values", false, &ui::dataframe(&["Field", "Current Value", "Needs Fill"], &t)));
    h.push_str(&ui::caption("💡 After downloading, edit the JSON to fill in the missing fields, then upload it in **Phase 3** to review and commit the changes."));
    h
}

/// Phase 2 (export): pick up to 5 incomplete plants, see exactly which fields
/// are missing, and download them as one JSON file. Gives back HTML.
async fn phase2_export(state: &AppState, req: &Req, st: &Map<String, Value>) -> String {
    let mut h = ui::subheader("📦 Bulk Export — up to 5 plants with missing fields")
        + &ui::markdown("Shows only plants that have **incomplete data**.  Select up to **5**, review exactly which fields are missing, then download as a single JSON file.  Edit it offline and upload in **Phase 3** to commit changes.");
    let reload = post_form("jp_p2e_reset", 2, &ui::button("go", "1", "🔄 Reload", false, true), "");
    if !flag(st, "p2e_loaded") {
        return h + &ui::columns(&[3, 1], vec![post_form("jp_p2e_load", 2, &ui::button("go", "1", "🔍 Load Plants with Missing Fields", true, true), ""), reload]);
    }
    h.push_str(&ui::columns(&[3, 1], vec![String::new(), reload]));
    let all = partials(state, " ORDER BY plant_name ASC").await;
    if all.is_empty() {
        return h + &ui::success("🎉 Every plant is complete — nothing to export!");
    }
    h.push_str(&ui::caption(&format!("**{}** plants have missing fields (complete plants hidden).", all.len())));
    // Filter box text, lower case.
    let term = req.get("f2").trim().to_lowercase();
    // Plants whose name, botanical name or family contains the filter text.
    let shown: Vec<&Map<String, Value>> = all.iter().filter(|r| term.is_empty() || ["plant_name", "botanical_name", "family"].iter().any(|k| s(r, k).to_lowercase().contains(&term))).collect();
    let opts_: Vec<(String, String)> = shown.iter().map(|r| (s(r, "id"), format!("[ID {}]  {}  ({} missing)", s(r, "id"), s(r, "plant_name"), missing_n(r)))).collect();
    // The ids ticked in the picker, at most 5 (`take(5)`).
    let picked: Vec<String> = ids_of(req.get("pick[]")).into_iter().take(5).collect();
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"2\">{}{}</form>",
        ui::text_input("f2", "🔍 Filter:", req.get("f2"), "Name, family, botanical…"),
        pick_box("pick[]", &format!("Select plants to export (max 5 — {} shown):", shown.len()), &opts_, &picked, 5)));
    // Look the picked ids up in the shown list.
    let chosen: Vec<&Map<String, Value>> = picked.iter().filter_map(|id| shown.iter().find(|r| s(r, "id") == *id).copied()).collect();
    if chosen.is_empty() {
        return h + &ui::info("Select up to 5 plants above to see their missing fields and generate the export.");
    }
    h.push_str(&ui::divider());
    h.push_str(&ui::h4("Missing fields for selected plants"));
    // One expander per chosen plant, listing its missing fields.
    for r in &chosen {
        let missing: Vec<String> = r.get("_missing_fields").and_then(Value::as_array).map(|a| a.iter().map(|x| text(Some(x))).collect()).unwrap_or_default();
        let body = if missing.is_empty() { ui::success("No missing fields — already complete.") } else {
            let rows_: String = missing.iter().map(|f| { let v = s(r, f); format!("<tr><td style='padding:4px 10px;color:#f0ad4e;font-weight:600'>{}</td><td style='padding:4px 10px;color:#888;font-style:italic'>{}</td></tr>", he(f), he(if v.trim().is_empty() { "— empty —" } else { v.trim() })) }).collect();
            format!("<table style='width:100%;border-collapse:collapse;font-size:13px'><tr><th style='padding:4px 10px;text-align:left;color:#aaa'>Field</th><th style='padding:4px 10px;text-align:left;color:#aaa'>Current value</th></tr>{rows_}</table>")
        };
        h.push_str(&ui::expander(&format!("**[ID {}] {}** — {} missing field{}", s(r, "id"), s(r, "plant_name"), missing.len(), if missing.len() != 1 { "s" } else { "" }), true, &body));
    }
    h.push_str(&ui::divider());
    // File name for the download, with a time stamp.
    let fname = format!("bulk_export_{}plants_{}.json", chosen.len(), stamp());
    let ids = chosen.iter().map(|r| s(r, "id")).collect::<Vec<_>>().join(",");
    // The link is answered by `export` below (kind jp_p2e).
    h.push_str(&format!("<a class=\"st-btn st-primary st-full\" href=\"/streamlit/?export=jp_p2e&amp;ids={ids}&amp;fn={}\">{}</a>", crate::php::urlencode(&fname),
        he(&format!("⬇️ Download {fname}  ({} plant{})", chosen.len(), if chosen.len() != 1 { "s" } else { "" }))));
    h.push_str(&ui::info("💡 Fill in the missing fields in the downloaded JSON, then upload it in **📥 Phase 3 — Import & Review** to review and commit the changes."));
    h
}

/// Phase 2 (enrich): choose plants, press Start, and the free AI fills blanks in
/// the background. The page shows a progress bar and reloads every 4 seconds
/// while `running` is true. When `done` it lists results and a download link.
/// Gives back HTML.
async fn enrich(state: &AppState, st: &Map<String, Value>) -> String {
    let mut h = ui::subheader("🤖 Batch AI Enrichment")
        + &ui::markdown("Processes plants with missing fields through **Claude + Perenual (+ optional web search)**, applying the same strict lock rule as the **🔒 AI — Missing Only** tab: existing values are never touched.  Plants are enriched **in parallel**, so a batch finishes in roughly *(per-plant time ÷ workers)*.");
    // `p2` is the saved Enrich job: loaded / running / done / results / errors.
    let p2 = st.get("p2").and_then(Value::as_object).cloned().unwrap_or_default();
    let done = flag(&p2, "done");
    let running = flag(&p2, "running");
    // `form` collects the controls; `chosen` is how many plants are selected.
    let mut form = String::new();
    let mut chosen = 0;
    // Before a run: either offer the Load button or show the plant picker.
    if !done && !running {
        if !flag(&p2, "loaded") {
            h.push_str(&post_form("jp_p2_load", 3, &ui::button("go", "1", "🔍 Load Plants with Missing Fields", false, false), ""));
        } else {
            let all = partials(state, "").await;
            if all.is_empty() {
                form.push_str(&ui::success("No plants with missing fields — nothing to enrich!"));
            } else {
                // Every incomplete plant is pre-selected.
                let opts_: Vec<(String, String)> = all.iter().map(|p| (s(p, "id"), format!("{}  ({} missing)  [ID {}]", s(p, "plant_name"), missing_n(p), s(p, "id")))).collect();
                let every: Vec<String> = opts_.iter().map(|(v, _)| v.clone()).collect();
                form.push_str(&pick_box("pick[]", &format!("🌿 Select plants to enrich ({} with missing fields):", all.len()), &opts_, &every, 0));
                chosen = all.len();
                form.push_str(&ui::caption(&format!("**{chosen}** plants selected for enrichment.")));
            }
        }
    }
    // Web search tick box and the worker-count slider (1 to 16, default 8).
    form.push_str(&ui::columns(&[1, 1], vec![
        ui::checkbox("web", "🌐 Include web search (DuckDuckGo)", false, Some("OFF (default) is much faster — relies on Claude + Perenual. Turn ON for extra location/seasonal context at the cost of speed.")),
        ui::slider("workers", "⚡ Parallel workers", 1, 16, 8),
    ]));
    // The Start button; disabled (by editing its HTML) if running, done or nothing chosen.
    let mut start = ui::button("do", "start", &if chosen > 0 { format!("🚀 Start Batch Enrichment ({chosen} plants)") } else { "🚀 Start Batch Enrichment".into() }, true, false);
    if done || running || chosen == 0 {
        start = start.replacen("<button", "<button disabled", 1);
    }
    // Start and Reset buttons side by side.
    form.push_str(&ui::columns(&[2, 1], vec![start, ui::button("do", "reset", "🔄 Reset", false, false)]));
    h.push_str(&post_form("jp_p2", 3, &form, ""));
    // While running: progress bar, spinner, and a script that reloads after 4000 ms.
    if running {
        let (d, t) = (p2.get("finished").and_then(Value::as_u64).unwrap_or(0), p2.get("total").and_then(Value::as_u64).unwrap_or(0));
        let pct = if t > 0 { d * 100 / t } else { 0 };
        h.push_str(&format!("<p class=\"st-text\">Enriching {d} / {t} plants…</p><div class=\"st-progress\"><div style=\"width:{pct}%\"></div></div>"));
        h.push_str(&format!("<div class=\"st-status\"><span class=\"st-spin\"></span>Enriching {t} plants with {} workers…</div><script>setTimeout(function(){{location.reload()}},4000)</script>", p2.get("workers").and_then(Value::as_u64).unwrap_or(1)));
    }
    // When finished: summary, any errors, a results table and the download link.
    if done {
        let results = p2.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
        let errors = p2.get("errors").and_then(Value::as_array).cloned().unwrap_or_default();
        h.push_str(&ui::success(&format!("✅ Enrichment complete — {} plants processed, {} errors.", results.len(), errors.len())));
        if !errors.is_empty() {
            let body: String = errors.iter().map(|e| ui::error(&format!("ID {} · {}: {}", text(e.get("id")), text(e.get("plant_name")), text(e.get("error"))))).collect();
            h.push_str(&ui::expander(&format!("⚠️ {} plants failed", errors.len()), false, &body));
        }
        if !results.is_empty() {
            let t: Vec<Vec<String>> = results.iter().map(|r| {
                let f: Vec<String> = r.get("_enriched_fields").and_then(Value::as_array).map(|a| a.iter().map(|x| text(Some(x))).collect()).unwrap_or_default();
                vec![ui::cell(&text(r.get("plant_name"))), ui::cell(&f.len().to_string()), ui::cell(&if f.is_empty() { "—".into() } else { f.join(", ") })]
            }).collect();
            h.push_str(&ui::dataframe(&["Plant", "Fields Filled", "Fields"], &t));
            let fname = format!("plants_enriched_{}.json", stamp());
            h.push_str(&format!("<a class=\"st-btn st-primary st-full\" href=\"/streamlit/?export=jp_p2&amp;fn={}\">{}</a>", crate::php::urlencode(&fname), he(&format!("⬇️ Download {fname}  ({} plants)", results.len()))));
            h.push_str(&ui::caption("➡️ Upload this file in **Phase 3** to review and commit changes."));
        }
    }
    h
}

/// The Enrich job: up to `workers` plants at once, saving progress as each
/// one finishes so the page can show it.
/// Takes shared app state (`Arc` is a counted shared pointer, so the
/// background task can keep using it), the plants, whether to use web
/// search, and the worker count. It runs in the background, started by
/// `tokio::spawn` in `action`.
async fn enrich_job(state: Arc<AppState>, plants: Vec<Map<String, Value>>, web: bool, workers: usize) {
    // Brings the `.next()` helper for streams into scope for this function.
    use futures_util::StreamExt;
    // A second shared pointer that is moved into the closure below.
    let st = state.clone();
    // A stream of one async job per plant, run `workers` at a time.
    // `buffer_unordered` yields each result as soon as it is ready.
    let mut stream = futures_util::stream::iter(plants.into_iter().map(move |p| {
        let state = st.clone();
        async move {
            // Per plant: optional web context, Perenual plant data, then ask the AI.
            let name = s(&p, "plant_name");
            let loc = if web { ai::location_context(&state, &name).await } else { String::new() };
            let peren = ai::perenual(&state, &name).await;
            let res = ai::ask(&state, &p, &loc, &peren, false).await;
            (p, res)
        }
    // At least 1 worker, even if 0 was asked for.
    })).buffer_unordered(workers.max(1));
    // Handle each finished plant as it arrives.
    while let Some((p, res)) = stream.next().await {
        // Reload the saved job, add this result, and save again.
        let mut all = load_state(&state);
        let mut p2 = all.get("p2").and_then(Value::as_object).cloned().unwrap_or_default();
        match res {
            // AI worked: fill only fields that are empty (the lock rule).
            Ok(llm) => {
                let mut rec = p.clone();
                let mut filled = Vec::new();
                for (k, v) in llm {
                    if is_empty(p.get(&k)) && !is_empty(Some(&v)) {
                        filled.push(Value::String(k.clone()));
                        rec.insert(k, v);
                    }
                }
                rec.insert("_enriched_fields".into(), Value::Array(filled));
                rec.insert("_proposed_changes".into(), Value::Object(Map::new()));
                push(&mut p2, "results", Value::Object(rec));
            }
            // AI failed: record the error so the page lists it (S-14 in DEVIATIONS.md).
            Err(e) => push(&mut p2, "errors", serde_json::json!({ "id": p.get("id"), "plant_name": s(&p, "plant_name"), "error": e.error })),
        }
        // Count how many plants have finished.
        let n = p2.get("finished").and_then(Value::as_u64).unwrap_or(0) + 1;
        p2.insert("finished".into(), serde_json::json!(n));
        all.insert("p2".into(), Value::Object(p2));
        save_state(&state, &all);
    }
    // All plants handled: mark the job as finished.
    let mut all = load_state(&state);
    if let Some(Value::Object(p2)) = all.get_mut("p2") {
        p2.insert("running".into(), Value::Bool(false));
        p2.insert("done".into(), Value::Bool(true));
    }
    save_state(&state, &all);
}

/// Adds `v` to the list stored under key `k` in map `m`, creating the list if needed.
fn push(m: &mut Map<String, Value>, k: &str, v: Value) {
    match m.get_mut(k) {
        Some(Value::Array(a)) => a.push(v),
        _ => { m.insert(k.into(), Value::Array(vec![v])); }
    }
}

/// Phase 3: upload a JSON file, compare each record with the database, and
/// let the user commit new fills and chosen overwrites. Nothing is written
/// until a commit button is pressed. Gives back HTML.
async fn phase3(state: &AppState, req: &Req) -> String {
    let mut h = ui::subheader("📥 Import & Review")
        + &ui::markdown("Upload a JSON file (from Phase 2 or hand-edited).  Each plant is compared against the current database record.  **New fills** (was empty) are highlighted green.  **Proposed overwrites** (field already had a value) are highlighted amber and require explicit approval.  Nothing is written until you press **Approve & Commit**.")
        + &uploader("jp_upload", 4, "Upload enriched JSON file", "");
    // `up` is the upload's token, kept in the address between page loads.
    let up = req.get("up");
    // No file yet: show a hint and stop.
    let Some(records) = load_upload(state, up) else {
        return h + &ui::info("Upload a JSON file above to begin the review.");
    };
    h.push_str(&ui::success(&format!("Loaded **{}** plant records from JSON.", records.len())));
    h.push_str(&ui::divider());
    // All current plants by id, for comparing.
    let db: HashMap<String, Map<String, Value>> = rows(state, "SELECT * FROM data_source").await.into_iter().map(|r| (s(&r, "id"), r)).collect();
    // Records that would change something: (diff, record).
    let committable: Vec<(Diff, &Map<String, Value>)> = records.iter().filter_map(|r| db.get(&s(r, "id")).map(|d| (diff(r, d), r))).filter(|((n, o), _)| !n.is_empty() || !o.is_empty()).collect();
    // "Approve ALL" button, only when there is something to commit.
    if !committable.is_empty() {
        // Totals of new fills (tn) and overwrites (to) across all plants.
        let (tn, to): (usize, usize) = committable.iter().fold((0, 0), |(a, b), ((n, o), _)| (a + n.len(), b + o.len()));
        let aow = req.get("aow") == "1";
        let note = if aow { format!(" + {to} overwrites") } else { String::new() };
        let all_btn = post_form("jp_p3_all", 4, &ui::button("go", "1", &format!("✅ Approve ALL — commit {tn} new fills{note} across {} plants", committable.len()), true, true), &(hidden("up", up) + &hidden("aow", req.get("aow"))));
        let aow_box = format!("<form method=\"get\" action=\"/streamlit/\" onchange=\"this.submit()\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"4\">{}{}</form>",
            hidden("up", up), ui::checkbox("aow", "Also accept ALL overwrites", aow, Some("When checked, Approve All will also apply every proposed amber overwrite — not just the green new-fills.")));
        h.push_str(&ui::columns(&[3, 2], vec![all_btn, aow_box]));
        h.push_str(&ui::divider());
    }
    // One expander per uploaded record, with its review inside.
    for (i, rec) in records.iter().enumerate() {
        let id = s(rec, "id");
        let name = { let n = s(rec, "plant_name"); if rec.contains_key("plant_name") { n } else { format!("Unknown (index {i})") } };
        let body = match db.get(&id) {
            None => ui::error(&format!("Plant ID {} not found in the database — skipping.", if rec.get("id").map(|v| v.is_null()).unwrap_or(true) { "None".into() } else { id.clone() })),
            Some(d) => review_body(&diff(rec, d), i, up, &name, 4, "jp_p3_commit"),
        };
        h.push_str(&ui::expander(&format!("🌿 {name}  (ID: {})", if id.is_empty() { "None" } else { &id }), false, &body));
    }
    h
}

/// One record's review: green fills, amber overwrites with tick boxes, and
/// the commit / skip buttons.
/// Takes the diff, the record's position (`idx`) in the upload, the upload
/// token, the plant name, the inner-tab number and the action name (which
/// differs for the wishlist). Gives back HTML. The wishlist reuses this.
pub fn review_body((new, ow): &Diff, idx: usize, up: &str, name: &str, sub: usize, act: &str) -> String {
    // Nothing to show for this record.
    if new.is_empty() && ow.is_empty() {
        return ui::info("No differences found between this JSON record and the current database.");
    }
    let mut b = String::new();
    // Green table of new fills.
    if !new.is_empty() {
        b.push_str(&ui::markdown("**✅ New fills** — fields that were empty in the DB:"));
        b.push_str(&fills_table(new));
    }
    let mut boxes = String::new();
    // Amber table of overwrites, then one tick box per overwrite.
    if !ow.is_empty() {
        b.push_str(&ui::markdown("**⚠️ Proposed overwrites** — fields that already have a DB value:"));
        let rows_: String = ow.iter().map(|(f, dv, jv)| {
            let cell = if f == "grow_months" || f == "bloom_months" {
                format!("<div style='opacity:.6'>{}</div><div style='margin-top:4px'>⬇️ {}</div>", month_display(f, dv), month_display(f, jv))
            } else {
                format!("<span style='text-decoration:line-through;opacity:.6'>{}</span> &rarr; {}", he(&py_str(dv).chars().take(100).collect::<String>()), he(&py_str(jv).chars().take(100).collect::<String>()))
            };
            format!("<tr style='background:#fff3cd'><td style='padding:5px 10px;font-weight:600;color:#856404;width:30%'>{}</td><td style='padding:5px 10px;color:#856404'>{cell}</td></tr>", he(f))
        }).collect();
        b.push_str(&format!("<table style='width:100%;border-collapse:collapse;font-size:13px'>{rows_}</table>"));
        boxes.push_str(&ui::markdown("Select which overwrites to **accept**:"));
        for (f, dv, jv) in ow {
            boxes.push_str(&format!("<label class=\"st-check\"><input type=\"checkbox\" name=\"acc[]\" value=\"{}\"> {}</label>", he(f),
                ui::md(&format!("Accept `{f}`: `{}` → `{}`", py_str(dv).chars().take(40).collect::<String>(), py_str(jv).chars().take(40).collect::<String>()))));
        }
    }
    // Button label with a plural "s" when needed.
    let n = new.len();
    let label = format!("✅ Approve & Commit  ({n} new fill{})", if n != 1 { "s" } else { "" });
    let extra = hidden("up", up) + &hidden("idx", &idx.to_string()) + &hidden("name", name);
    // The form with the tick boxes, Commit and Skip buttons.
    b + &post_form(act, sub, &(boxes + &ui::columns(&[3, 1], vec![ui::button("do", "commit", &label, true, true), ui::button("do", "skip", "⏭ Skip", false, true)])), &extra)
}

// ---- downloads --------------------------------------------------------------

/// Uses the file name `f` only if it has safe characters; otherwise `fallback`.
fn safe_fn(f: &str, fallback: &str) -> String {
    if !f.is_empty() && f.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')) { f.to_string() } else { fallback.to_string() }
}

/// Builds a download. `kind` says which one (jp_p1, jp_p2e, jp_p2, jp_schema, jp_wl);
/// `q` holds the address settings. Gives back (file text, file name) or `None`
/// when the plant does not exist. Called from `page` in mod.rs.
pub async fn export(state: &AppState, kind: &str, q: &HashMap<String, String>) -> Option<(String, String)> {
    // Small helper: get an address setting as text ("" if missing).
    let g = |k: &str| q.get(k).map(String::as_str).unwrap_or("");
    match kind {
        // Phase 1: one plant, named after the plant.
        "jp_p1" => {
            let id = php_int(g("id"));
            let r = rows(state, &format!("SELECT * FROM data_source WHERE id = {id}")).await.into_iter().next()?;
            let r = with_missing(r);
            Some((dumps(&Value::Object(r.clone())), format!("{}_missing_{}.json", s(&r, "plant_name").replace(' ', "_").replace('/', "-"), stamp())))
        }
        // Phase 2 export: up to 5 chosen plants as a JSON list.
        "jp_p2e" => {
            let ids = ids_of(g("ids"));
            let all = partials(state, " ORDER BY plant_name ASC").await;
            let chosen: Vec<Value> = ids.iter().take(5).filter_map(|id| all.iter().find(|r| s(r, "id") == *id)).map(|r| Value::Object(r.clone())).collect();
            Some((dumps(&Value::Array(chosen.clone())), safe_fn(g("fn"), &format!("bulk_export_{}plants_{}.json", chosen.len(), stamp()))))
        }
        // Phase 2 enrich: the saved results.
        "jp_p2" => {
            let st = load_state(state);
            let res = st.get("p2").and_then(|p| p.get("results")).cloned().unwrap_or(Value::Array(vec![]));
            Some((dumps(&res), safe_fn(g("fn"), &format!("plants_enriched_{}.json", stamp()))))
        }
        // The VS Code schema from pipeline_ref.rs.
        "jp_schema" => Some((super::pipeline_ref::schema_json(), "plant_schema.json".into())),
        // Wishlist export from pipeline_wl.rs.
        "jp_wl" => super::pipeline_wl::export(state, g("ids"), g("fn")).await,
        _ => None,
    }
}

// ---- the buttons -------------------------------------------------------------

/// Handles every button on this tab (POST). `act` names the button; `f` is the
/// submitted form. Gives back (messages to show, extra address settings so the
/// redirect lands on the same inner tab and upload). Called from `action` in mod.rs.
pub async fn action(state: Arc<AppState>, act: &str, f: &crate::formdata::FormData) -> (Vec<Flash>, Vec<(&'static str, String)>) {
    // Which inner tab to return to.
    let sub = php_int(f.str("sub")).clamp(1, 6);
    // Settings carried over to the next page load.
    let mut next: Vec<(&'static str, String)> = vec![("sub", sub.to_string())];
    for k in ["up", "aow", "wsub", "wup", "waow"] {
        if !f.str(k).is_empty() {
            next.push((k, f.str(k).to_string()));
        }
    }
    // `msgs` is the list of result messages; each branch gives back a list.
    let msgs = match act {
        // Remember Phase 1 was loaded.
        "jp_p1_load" => { set_flag(&state, "p1_loaded", Value::Bool(true)); vec![] }
        "jp_p1_reset" => { let mut st = load_state(&state); st.remove("p1_loaded"); st.remove("p1_done"); save_state(&state, &st); vec![] }
        // Mark a plant done in Phase 1: add its id to the done list.
        "jp_p1_done" => {
            let mut st = load_state(&state);
            let mut d: Vec<Value> = st.get("p1_done").and_then(Value::as_array).cloned().unwrap_or_default();
            d.push(serde_json::json!(php_int(f.str("id"))));
            st.insert("p1_done".into(), Value::Array(d));
            save_state(&state, &st);
            vec![]
        }
        "jp_p2e_load" => { set_flag(&state, "p2e_loaded", Value::Bool(true)); vec![] }
        "jp_p2e_reset" => { set_flag(&state, "p2e_loaded", Value::Bool(false)); vec![] }
        // Enrich: Load button.
        "jp_p2_load" => { set_flag(&state, "p2", serde_json::json!({ "loaded": true })); vec![] }
        // Enrich: Reset button (refused while a job is running).
        "jp_p2" if f.str("do") == "reset" => {
            if flag(load_state(&state).get("p2").and_then(Value::as_object).unwrap_or(&Map::new()), "running") {
                vec![Flash::Warning("The enrichment is still running; reset it when it finishes.".into())]
            } else {
                set_flag(&state, "p2", Value::Object(Map::new()));
                vec![]
            }
        }
        // Enrich: Start. Save the job state first, then start the background task.
        "jp_p2" => {
            let ids = ids_of(f.str("pick[]"));
            let plants: Vec<Map<String, Value>> = partials(&state, "").await.into_iter().filter(|p| ids.contains(&s(p, "id"))).collect();
            if plants.is_empty() {
                vec![Flash::Warning("Select at least one plant.".into())]
            } else {
                let workers = php_int(f.str("workers")).clamp(1, 16) as usize;
                set_flag(&state, "p2", serde_json::json!({ "loaded": true, "running": true, "total": plants.len(), "finished": 0, "workers": workers, "results": [], "errors": [] }));
                // `tokio::spawn` starts the job in the background; this request does not wait.
                tokio::spawn(enrich_job(state.clone(), plants, f.str("web") == "1", workers));
                vec![]
            }
        }
        // A file was uploaded: store it and carry its token to the next page.
        "jp_upload" | "jp_wl_upload" => match store_upload(&state, f) {
            Ok(t) => { next.retain(|(k, _)| *k != if act == "jp_upload" { "up" } else { "wup" }); next.push((if act == "jp_upload" { "up" } else { "wup" }, t)); vec![] }
            Err(e) => vec![Flash::Error(e)],
        },
        // Approve ALL: commit every record's new fills (and all overwrites if ticked).
        "jp_p3_all" => {
            let recs = load_upload(&state, f.str("up")).unwrap_or_default();
            let db: HashMap<String, Map<String, Value>> = rows(&state, "SELECT * FROM data_source").await.into_iter().map(|r| (s(&r, "id"), r)).collect();
            let (mut ok, mut fields_n, mut errs) = (0, 0, Vec::new());
            for r in &recs {
                let Some(d) = db.get(&s(r, "id")) else { continue };
                let (mut fields, ow) = diff(r, d);
                if f.str("aow") == "1" {
                    for (k, _, jv) in ow { fields.insert(k, jv); }
                }
                if fields.is_empty() { continue; }
                match commit(&state, "data_source", php_int(&s(r, "id")), &fields, false).await {
                    Ok(n) => { ok += 1; fields_n += n; }
                    Err(e) => errs.push(format!("{}: {e}", s(r, "plant_name"))),
                }
            }
            let mut v = vec![Flash::Success(format!("✅ Committed {fields_n} field(s) across {ok} plant(s).{}", if errs.is_empty() { String::new() } else { format!("  ⚠️ {} failed.", errs.len()) }))];
            v.extend(errs.into_iter().take(10).map(Flash::Error));
            v
        }
        // Commit or skip one record; only ticked overwrites (`acc[]`) are applied.
        "jp_p3_commit" => {
            let name = f.str("name").to_string();
            if f.str("do") == "skip" {
                vec![Flash::Info(format!("Skipped **{name}**."))]
            } else {
                let recs = load_upload(&state, f.str("up")).unwrap_or_default();
                let Some(r) = recs.get(php_int(f.str("idx")).max(0) as usize) else { return (vec![Flash::Error("That record is no longer loaded.".into())], next) };
                let id = php_int(&s(r, "id"));
                let db = rows(&state, &format!("SELECT * FROM data_source WHERE id = {id}")).await;
                let Some(d) = db.first() else { return (vec![Flash::Error(format!("Plant ID {id} not found in the database — skipping."))], next) };
                let (mut fields, ow) = diff(r, d);
                let acc: Vec<&str> = f.str("acc[]").split(", ").collect();
                for (k, _, jv) in ow {
                    if acc.contains(&k.as_str()) { fields.insert(k, jv); }
                }
                if fields.is_empty() {
                    vec![Flash::Warning("Nothing to commit — all overwrites were unchecked.".into())]
                } else {
                    match commit(&state, "data_source", id, &fields, false).await {
                        Ok(n) => vec![Flash::Success(format!("✅ Committed {n} field(s) for **{name}** (ID {id})."))],
                        Err(e) => vec![Flash::Error(format!("Database error: {e}"))],
                    }
                }
            }
        }
        // Wishlist buttons are handled in pipeline_wl.rs.
        a if a.starts_with("jp_wl_") => super::pipeline_wl::action(&state, a, f).await,
        _ => vec![Flash::Error("Unknown action.".into())],
    };
    (msgs, next)
}

// Tests: run with `cargo test`; they are left out of the real program.
#[cfg(test)]
mod tests {
    use super::*;

    /// The lock rule: filled DB values are kept unless they differ and are approved.
    #[test]
    fn diff_follows_the_lock_rule() {
        let rec: Map<String, Value> = serde_json::from_str(r#"{"id":1,"family":"Lamiaceae","zones":"5-9","average_cost":"8.990","picture":"x.webp","_note":"n"}"#).unwrap();
        let db: Map<String, Value> = serde_json::from_str(r#"{"id":1,"family":null,"zones":"4-8","average_cost":8.99,"picture":""}"#).unwrap();
        let (new, ow) = diff(&rec, &db);
        assert_eq!(new.keys().collect::<Vec<_>>(), vec!["family"]);
        assert_eq!(ow.len(), 1);
        assert_eq!(ow[0].0, "zones");
    }

    /// Month lists become the 12-letter code; Python-style text matches Streamlit.
    #[test]
    fn months_and_python_text() {
        assert_eq!(month_code(&serde_json::json!([5, 6])).as_deref(), Some("....MJ......"));
        assert_eq!(py_str(&serde_json::json!(["a", 1])), "['a', 1]");
    }
}
