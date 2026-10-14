//! The three AI tabs: "📝 Manual Review (AI)" (tab_manual_review.py),
//! "✨ Fill Missing Data (AI)" (tab_fill_missing.py) and "🔒 AI — Missing
//! Only" (tab_ai_safe_fill.py).
//!
//! HOW TO READ THIS FILE (for Jun). A button picks the plants (3, or 1 for
//! Missing Only) and starts a background job; the page reloads itself every
//! few seconds while the job works through them, exactly like Streamlit's
//! "⏳ 3 plants in queue..." loop. Each answer is shown in the normal editor
//! with the AI's values filled in; nothing is written until you press Save
//! Changes. The job's progress and answers are kept in
//! data/orchestrator-ai/tab<N>.json, so a reload (or the other person
//! opening the tab) shows the same results. The AI itself is ai.rs.
//!
//! DIFFERENCES FROM STREAMLIT (DEVIATIONS.md): free cloud models instead of
//! Ollama/Anthropic (S-4); "Next Plant" on Missing Only moves on to the next
//! plant instead of fetching the same unsaved one again (S-9); "Fetch &
//! Stage Image" attaches the photo to the plant straight away (S-10).
//! Tab numbers: 2 = Manual Review, 3 = Fill Missing Data, 10 = AI Missing Only.
//! The job file for tab N is data/orchestrator-ai/tab<N>.json. A job holds:
//!   queue    plants still waiting;   batch   finished answers;
//!   working  the plant being done now;   stage   a short progress note;
//!   done     (tab 10 only) true when no plant needs filling;
//!   last_id  (tab 10 only) the last plant looked at.
//!
//! Rust ideas used often in this file (explained once here):
//!   - `async fn` waits (for the database, the AI, the disk); `.await`
//!     pauses until the wait is over.
//!   - `tokio::spawn(..)` starts a task in the background; the request that
//!     started it does not wait for it.
//!   - `Arc<T>` is a shared pointer, so the page and the background task can
//!     both hold the app state. `Mutex<T>` lets only one thread at a time
//!     change a value; `.lock()` takes the turn.
//!   - `Option<T>` is "a value or nothing" (`Some(x)` / `None`).
//!   - `let Some(x) = .. else { .. }` continues only when the value is there.
//!   - `&str` is a borrowed look at text; `String` is text the code owns.
//!   - `Map<String, Value>` is one JSON object (field -> value).
//!   - `'a` is a "lifetime": it says a result borrows from an input.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use serde_json::{Map, Value};

use super::ai;
use super::export;
use super::form::{self, Flash, FormCtx};
use super::opts::text;
use super::ui;
use super::{all_plant_names, rows, Req};
use crate::AppState;

/// The set of tab numbers whose background job is running right now.
/// A `static` lives for the whole run of the program. It is inside a Mutex so
/// two requests cannot change it at the same moment. It starts as `None`
/// (no set yet); `set_running` creates the set the first time it is needed.
static RUNNING: Mutex<Option<HashSet<usize>>> = Mutex::new(None);

/// True when the job for `tab` is running. If the lock cannot be taken, say false.
fn running(tab: usize) -> bool {
    RUNNING.lock().map(|g| g.as_ref().map(|s| s.contains(&tab)).unwrap_or(false)).unwrap_or(false)
}

/// Marks the job for `tab` as running (`on` = true) or finished (false).
/// Gives back true if the set changed. Starting a second job on the same
/// tab gives false, which is how `action` avoids starting it twice.
fn set_running(tab: usize, on: bool) -> bool {
    let Ok(mut g) = RUNNING.lock() else { return false };
    // Create the set on first use.
    let s = g.get_or_insert_with(HashSet::new);
    if on { s.insert(tab) } else { s.remove(&tab) }
}

/// The job file for a tab: data/orchestrator-ai/tab<N>.json.
/// `PathBuf` is an owned file path. `state.data` is the app's data folder.
fn path(state: &AppState, tab: usize) -> std::path::PathBuf {
    state.data.join("orchestrator-ai").join(format!("tab{tab}.json"))
}

/// Reads a tab's job. Gives back an empty job when the file is missing or broken.
fn load(state: &AppState, tab: usize) -> Map<String, Value> {
    // read file -> bytes -> JSON. `.ok()` turns a failure into "nothing", and
    // `.and_then(..)` runs the next step only if the earlier one worked.
    match std::fs::read(path(state, tab)).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()) {
        Some(Value::Object(m)) => m,
        _ => Map::new(),
    }
}

/// Saves a tab's job. It writes a temporary file and then renames it, so a
/// reader never sees a half-written file.
fn store(state: &AppState, tab: usize, job: &Map<String, Value>) {
    let p = path(state, tab);
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let tmp = p.with_extension("tmp");
    if std::fs::write(&tmp, Value::Object(job.clone()).to_string()).is_ok() {
        let _ = std::fs::rename(tmp, p);
    }
}

/// Reads field `k` of a row as plain text ("" when missing or null).
fn s(m: &Map<String, Value>, k: &str) -> String {
    text(m.get(k))
}

/// Gets the list stored under key `k` as a list of objects (empty if absent).
/// The result borrows from `m`, which the `'a` marks show.
fn arr<'a>(m: &'a Map<String, Value>, k: &str) -> Vec<&'a Map<String, Value>> {
    m.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_object).collect()).unwrap_or_default()
}

/// Gets the list stored under key `k` as a list of text values (empty if absent).
fn strings(m: &Map<String, Value>, k: &str) -> Vec<String> {
    m.get(k).and_then(Value::as_array).map(|a| a.iter().map(|x| text(Some(x))).collect()).unwrap_or_default()
}

/// SQL for tab 10: a plant needs filling when any key field is NULL or blank
/// (TRIM removes spaces). The company field also counts "unknown" as blank.
/// It is joined into a SELECT in `action`; it holds no user input.
const MISSING_ONLY_WHERE: &str = "(botanical_name IS NULL OR TRIM(botanical_name) = '')
   OR (family IS NULL OR TRIM(family) = '')
   OR (genus IS NULL OR TRIM(genus) = '')
   OR (species IS NULL OR TRIM(species) = '')
   OR (plant_length IS NULL OR TRIM(plant_length) = '')
   OR (plant_width IS NULL OR TRIM(plant_width) = '')
   OR (spacing IS NULL OR TRIM(spacing) = '')
   OR (soil_ph IS NULL OR TRIM(soil_ph) = '')
   OR (bloom_color IS NULL OR TRIM(bloom_color) = '')
   OR (sun_exposure IS NULL OR TRIM(sun_exposure) = '')
   OR (water_needs IS NULL OR TRIM(water_needs) = '')
   OR (foliage_color IS NULL OR TRIM(foliage_color) = '')
   OR (zones IS NULL OR TRIM(zones) = '')
   OR (company IS NULL OR TRIM(company) = '' OR LOWER(TRIM(company)) IN ('unknown', 'unknown only'))";

/// The buttons: "Search Next 3 Plants", "Find 3 Partially Filled Plants",
/// "Find Next Plant" / "Next Plant", and "Check Again".
/// Takes shared app state, the button (`act`) and the tab number. Gives back
/// messages to show. It picks the plants, saves them as a queue in the job
/// file, and starts `work` in the background.
pub async fn action(state: Arc<AppState>, act: &str, tab: usize) -> Vec<Flash> {
    // Only the three AI tabs are valid.
    if !matches!(tab, 2 | 3 | 10) {
        return vec![Flash::Error("Unknown action.".into())];
    }
    // "Check Again": throw the old job away.
    if act == "ai_reset" {
        store(&state, tab, &Map::new());
        return vec![];
    }
    // Do not start a second job on the same tab.
    if running(tab) {
        return vec![Flash::Warning("The AI is still working on the last request. Wait for it to finish.".into())];
    }
    // Which plants to pick depends on the tab.
    let picked = match tab {
        // Tab 2: up to 3 plants with no botanical name.
        2 => rows(&state, "SELECT * FROM data_source WHERE botanical_name IS NULL OR TRIM(botanical_name) = '' LIMIT 3").await,
        // Tab 3: up to 3 plants missing size, spacing, soil pH, bloom colour or company.
        3 => rows(&state, "SELECT * FROM data_source
            WHERE (plant_length IS NULL OR TRIM(plant_length) = '')
               OR (plant_width IS NULL OR TRIM(plant_width) = '')
               OR (spacing IS NULL OR TRIM(spacing) = '')
               OR (soil_ph IS NULL OR TRIM(soil_ph) = '')
               OR (bloom_color IS NULL OR TRIM(bloom_color) = '')
               OR (company IS NULL OR TRIM(company) = '' OR LOWER(TRIM(company)) IN ('unknown', 'unknown only'))
            LIMIT 3").await,
        // Tab 10: ONE plant, the next one after `last_id`. If none is left after it,
        // start again from the beginning.
        _ => {
            let last = crate::php::php_int(&s(&load(&state, tab), "last_id"));
            let mut r = rows(&state, &format!("SELECT * FROM data_source WHERE id > {last} AND ({MISSING_ONLY_WHERE}) ORDER BY id LIMIT 1")).await;
            if r.is_empty() && last > 0 {
                r = rows(&state, &format!("SELECT * FROM data_source WHERE {MISSING_ONLY_WHERE} ORDER BY id LIMIT 1")).await;
            }
            r
        }
    };
    // Start a fresh job.
    let mut job = Map::new();
    // Tab 10 with nothing to fill: remember that, so the page says it is finished.
    if tab == 10 && picked.is_empty() {
        job.insert("done".into(), Value::Bool(true));
        store(&state, tab, &job);
        return vec![];
    }
    // Tab 10 remembers where it stopped, so "Next Plant" moves on (S-9 in DEVIATIONS.md).
    if let Some(p) = picked.first().filter(|_| tab == 10) {
        job.insert("last_id".into(), p.get("id").cloned().unwrap_or(Value::Null));
    }
    // The queue is the picked plants; the batch (finished answers) starts empty.
    job.insert("queue".into(), Value::Array(picked.into_iter().map(Value::Object).collect()));
    job.insert("batch".into(), Value::Array(Vec::new()));
    store(&state, tab, &job);
    // `set_running` is true only if this tab was not already running.
    if set_running(tab, true) {
        // Start the background worker. The page returns at once.
        tokio::spawn(work(state, tab));
    }
    vec![]
}

/// Work through the queue one plant at a time, saving after each.
/// Takes shared app state and the tab. Loops until the queue is empty: take
/// the first plant, show it as "working", ask the AI, and save the answer in
/// the batch. Clears the running flag at the end.
async fn work(state: Arc<AppState>, tab: usize) {
    // `loop` repeats until a `break`.
    loop {
        let mut job = load(&state, tab);
        // Take the first plant off the queue; if the queue is empty, stop the loop.
        let Some(Value::Object(row)) = job.get_mut("queue").and_then(Value::as_array_mut).and_then(|q| if q.is_empty() { None } else { Some(q.remove(0)) }) else { break };
        // The queue as it is after removing that plant.
        let queue_left = job.get("queue").cloned();
        // Show the plant being worked on, still counted in the queue.
        let mut showing = job.clone();
        if let Some(Value::Array(q)) = showing.get_mut("queue") {
            q.insert(0, Value::Object(row.clone()));
        }
        showing.insert("working".into(), Value::String(s(&row, "plant_name")));
        store(&state, tab, &showing);
        // Ask the AI. The closure (small function) saves a progress note for the page.
        let item = process(&state, tab, row, |stage| {
            let mut st = showing.clone();
            st.insert("stage".into(), Value::String(stage.into()));
            store(&state, tab, &st);
        }).await;
        // Load again (the stage notes changed the file), then record the finished plant.
        let mut job = load(&state, tab);
        if let Some(q) = queue_left {
            job.insert("queue".into(), q);
        }
        job.remove("working");
        job.remove("stage");
        if let Some(Value::Array(b)) = job.get_mut("batch") {
            b.push(Value::Object(item));
        } else {
            job.insert("batch".into(), Value::Array(vec![Value::Object(item)]));
        }
        store(&state, tab, &job);
    }
    set_running(tab, false);
}

/// `_is_empty` (tab_ai_safe_fill.py): 'none', 'n/a' and '0' are real answers.
fn safe_empty(v: Option<&Value>) -> bool {
    matches!(text(v).trim().to_lowercase().as_str(), "" | "null" | "[]" | "............" | "unknown" | "unknown only")
}

/// `is_emp` (tab_fill_missing.py).
fn fill_empty(v: Option<&Value>) -> bool {
    matches!(text(v).trim().to_lowercase().as_str(), "" | "unknown" | "unknown only" | "[]" | "............")
}

/// Asks the AI about one plant and builds the finished answer for the batch.
/// Takes the app state, the tab, the plant row and `stage`, a function that is
/// called to report a progress note. Gives back an answer object holding the
/// row, the AI's values (`llm`), and extras for the tab: `new_fields` and
/// `changed_fields` (tab 3), `merged` and `ai_filled` (tab 10), or `error` and
/// `raw` when the AI failed. `impl Fn(&str)` means "any function that takes text".
async fn process(state: &AppState, tab: usize, row: Map<String, Value>, stage: impl Fn(&str)) -> Map<String, Value> {
    // Weather/location context and Perenual plant data are given to the AI.
    let name = s(&row, "plant_name");
    let loc = ai::location_context(state, &name).await;
    let peren = ai::perenual(state, &name).await;
    let mut item = Map::new();
    // Only tab 3 asks for the long version of the answer.
    let res = ai::ask(state, &row, &loc, &peren, tab == 3).await;
    // `Err` = the AI failed (S-14 in DEVIATIONS.md): keep the reason and raw text.
    match res {
        Err(e) => {
            item.insert("error".into(), Value::String(e.error));
            item.insert("raw".into(), Value::String(e.raw));
        }
        Ok(llm) => {
            // Tab 3: compare the AI's values with the row.
                // Was empty, now filled = "new"; different value = "changed".
            if tab == 3 {
                let mut new = Vec::new();
                let mut changed = Map::new();
                for (k, nv) in &llm {
                    let old = row.get(k);
                    if fill_empty(old) && !fill_empty(Some(nv)) {
                        new.push(Value::String(k.clone()));
                    } else if !fill_empty(old) && !fill_empty(Some(nv)) && text(old).trim() != text(Some(nv)).trim() {
                        changed.insert(k.clone(), Value::String(text(old)));
                    }
                }
                item.insert("new_fields".into(), Value::Array(new));
                item.insert("changed_fields".into(), Value::Object(changed));
            }
            // Tab 10: the LOCK RULE. Only fields that are empty in the row are filled.
            if tab == 10 {
                let mut merged = row.clone();
                let mut filled = Vec::new();
                for (k, v) in &llm {
                    if safe_empty(row.get(k)) && !safe_empty(Some(v)) {
                        filled.push(k.clone());
                        merged.insert(k.clone(), v.clone());
                    }
                }
                // If companion plants are still empty, ask for them, using the
                // garden's own plant names.
                if safe_empty(merged.get("companion_plants")) {
                    stage("companions");
                    let comps = ai::companions(state, &row, &all_plant_names(state).await).await;
                    if !comps.is_empty() {
                        merged.insert("companion_plants".into(), Value::String(comps.join(", ")));
                        if !filled.iter().any(|f| f == "companion_plants") {
                            filled.push("companion_plants".into());
                        }
                    }
                }
                item.insert("ai_filled".into(), Value::Array(filled.into_iter().map(Value::String).collect()));
                item.insert("merged".into(), Value::Object(merged));
            }
            item.insert("llm".into(), Value::Object(llm));
        }
    }
    // Keep the original row beside the answer for the page.
    item.insert("row".into(), Value::Object(row));
    item
}

/// The plant as the database has it now (photos, flags and sign-offs may
/// have changed since the AI looked), else the copy the job took.
/// Takes the plant snapshot from the job. Looks the plant up by id.
async fn fresh(state: &AppState, snap: &Map<String, Value>) -> Map<String, Value> {
    let id = crate::php::php_int(&s(snap, "id"));
    match sqlx::query("SELECT * FROM data_source WHERE id = ?").bind(id).fetch_optional(&state.db).await {
        Ok(Some(r)) => match crate::rowjson::row_to_json(&r) { Value::Object(m) => m, _ => snap.clone() },
        _ => snap.clone(),
    }
}

/// A "start" button that posts `ai_start` for this tab. `label` is the button text.
fn start_button(tab: usize, label: &str) -> String {
    format!("<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"ai_start\"><input type=\"hidden\" name=\"tab\" value=\"{tab}\">{}</form>", ui::button("go", "1", label, true, false))
}

/// A spinner with a message.
fn status(label: &str) -> String {
    format!("<div class=\"st-status\"><span class=\"st-spin\"></span>{}</div>", ui::md(label))
}

/// While the job works: the queue count, what it is doing, and a reload.
/// Takes the job and the tab. Gives back HTML for the queue count, the status
/// message, and a script that reloads the page every 4000 ms while the queue
/// is not empty. Gives back nothing when the queue is empty.
fn progress(job: &Map<String, Value>, tab: usize) -> String {
    let queue = arr(job, "queue");
    if queue.is_empty() {
        return String::new();
    }
    // The plant being worked on, or the first in the queue.
    let name = { let w = s(job, "working"); if w.is_empty() { s(queue[0], "plant_name") } else { w } };
    let mut h = String::new();
    // Tab 10 shows one plant at a time, so no queue count there.
    if tab != 10 {
        h.push_str(&ui::caption(&format!("⏳ {} plants in queue...", queue.len())));
    }
    // The status text depends on the tab (and on the companion stage for tab 10).
    h.push_str(&status(&match tab {
        2 => format!("🤖 Fetching {name} via free cloud models..."),
        3 => format!("🤖 Finding missing data for {name}..."),
        _ if s(job, "stage") == "companions" => format!("🔗 Linking companion plants for **{name}** from your database..."),
        _ => format!("🔒 Safely filling missing fields for **{name}**..."),
    }));
    h.push_str("<script>setTimeout(function(){location.reload()},4000)</script>");
    h
}

/// Draws the normal plant editor (form.rs) with the AI's values laid over the row.
/// `new` = fields the AI filled; `changed` = fields it would change (old values).
async fn editor(state: &AppState, tab: usize, p: &Map<String, Value>, data: &Map<String, Value>, new: &[String], changed: &Map<String, Value>) -> String {
    let names = all_plant_names(state).await;
    let ctx = FormCtx { back: vec![("tab", tab.to_string())], is_ai: true, new_fields: new, changed_fields: changed, all_names: &names, images_dir: state.www.join("images/Database") };
    form::render(p, data, &ctx)
}

/// A link button to download one AI result as HTML (`export` below).
fn export_button(tab: usize, i: usize, label: &str) -> String {
    ui::link_button(&format!("/streamlit/?export=ai_html&tab={tab}&i={i}"), label).replace("class=\"st-btn\"", "class=\"st-btn st-full\"")
}

/// Draws one of the three AI tabs. `req.tab` says which. Gives back HTML.
/// Tabs 2 and 3 show up to 3 results; tab 10 shows one.
pub async fn render(state: &AppState, req: &Req) -> String {
    let tab = req.tab;
    // The job saved for this tab (may be empty).
    let job = load(state, tab);
    let empty = Map::new();
    match tab {
        // Tabs 2 and 3: a start button, one expander per finished plant, then the progress.
        2 | 3 => {
            let mut h = if tab == 2 {
                ui::header("Search & Review Data (AI Only)") + &start_button(tab, "Search Next 3 Plants")
            } else {
                ui::header("✨ Fill Missing Data (Review & Approve)") + &start_button(tab, "Find 3 Partially Filled Plants")
            };
            // Number each result (`i`) so its export link can find it.
            for (i, it) in arr(&job, "batch").into_iter().enumerate() {
                let row = it.get("row").and_then(Value::as_object).unwrap_or(&empty);
                let name = s(row, "plant_name");
                let err = s(it, "error");
                let new = strings(it, "new_fields");
                let chg = it.get("changed_fields").and_then(Value::as_object).cloned().unwrap_or_default();
                // Expander title depends on the tab and on whether the AI failed.
                let title = if tab == 2 {
                    format!("{} {name}", if err.is_empty() { "✅" } else { "⚠️" })
                } else {
                    format!("{} {name} - {} New, {} Proposed", if err.is_empty() { "✨" } else { "⚠️" }, new.len(), chg.len())
                };
                // Failed: show the error (tab 2 also shows the raw text).
                let body = if !err.is_empty() {
                    let mut b = ui::error(&err);
                    if tab == 2 {
                        b.push_str(&ui::warning(&format!("Raw Output: {}", s(it, "raw"))));
                    }
                    b
                // Worked: an export button and the editor with the AI's values.
                } else {
                    let llm = it.get("llm").and_then(Value::as_object).unwrap_or(&empty);
                    let label = if tab == 2 {
                        "📄 Export this result as HTML (open in browser → Print → Save as PDF)".to_string()
                    } else {
                        format!("📄 Export this result as HTML  ({} new · {} changed)  — open in browser → Print → Save as PDF", new.len(), chg.len())
                    };
                    // Re-read the plant: photos or sign-offs may have changed since the AI looked.
                    let p = fresh(state, row).await;
                    export_button(tab, i, &label) + &ui::divider() + &editor(state, tab, &p, llm, &new, &chg).await
                };
                h.push_str(&ui::expander(&title, true, &body));
            }
            h.push_str(&progress(&job, tab));
            h
        }
        // Tab 10: strict mode, one plant at a time.
        _ => {
            let mut h = ui::header("🔒 AI — Missing Fields Only");
            h.push_str(&format!("<div class=\"st-alert st-info\"><p>🛡️</p>{}</div>", ui::md("**Strict read-protect mode.** The AI can only fill fields that are completely empty. Any field that already has a value is skipped — the AI never overwrites or proposes changes to existing data. Plants are reviewed **one at a time**. Save or discard, then fetch the next one.")));
            // No plants left: say so and offer "Check Again".
            if job.get("done").and_then(Value::as_bool) == Some(true) {
                h.push_str(&ui::success("🎉 No more plants with missing fields were found in the database!"));
                h.push_str(&format!("<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"ai_reset\"><input type=\"hidden\" name=\"tab\" value=\"{tab}\">{}</form>", ui::button("go", "1", "🔄 Check Again", false, false)));
                return h;
            }
            // `busy` = a plant is still in the queue.
            let batch = arr(&job, "batch");
            let busy = !arr(&job, "queue").is_empty();
            let has_plant = !batch.is_empty() || busy;
            h.push_str(&start_button(tab, if has_plant { "➡️ Next Plant" } else { "🔍 Find Next Plant with Missing Fields" }));
            h.push_str(&progress(&job, tab));
            // Nothing finished yet: stop after the button and progress.
            let Some(it) = batch.first() else { return h };
            let row = it.get("row").and_then(Value::as_object).unwrap_or(&empty);
            let name = s(row, "plant_name");
            h.push_str(&ui::divider());
            if !s(it, "error").is_empty() {
                h.push_str(&ui::error(&format!("AI error for **{name}**: {}", s(it, "error"))));
                h.push_str(&ui::warning(&format!("Raw output: {}", s(it, "raw"))));
                return h;
            }
            // `filled` = the fields the AI filled; `merged` = the row with those values added.
            let filled = strings(it, "ai_filled");
            let merged = it.get("merged").and_then(Value::as_object).unwrap_or(row);
            h.push_str(&ui::columns(&[3, 1], vec![ui::subheader(&format!("🔒 {name}")), export_button(tab, 0, "📄 Export as HTML")]));
            if filled.is_empty() {
                h.push_str(&ui::warning("The AI found no additional missing data to fill for this plant. You can still edit fields manually below, or click **Next Plant** to skip."));
            } else {
                h.push_str(&ui::success(&format!("✅ AI filled **{}** previously empty field(s): `{}`", filled.len(), filled.join("`, `"))));
            }
            h.push_str(&ui::caption("🛡️ Fields with existing data were locked — the AI could not touch them. Only green-highlighted fields were filled by the AI."));
            h.push_str(&ui::divider());
            // Show the freshest copy of the plant, with the AI's values and green marks.
            let p = fresh(state, row).await;
            h.push_str(&editor(state, tab, &p, merged, &filled, &empty).await);
            h
        }
    }
}

/// `?export=ai_html&tab=N&i=I`: one AI result as the printable HTML page.
/// Takes the tab and the position `i` of the result. Gives (HTML text, file
/// name), or `None` if there is no such result. Called from `page` in mod.rs.
pub fn export(state: &AppState, tab: usize, i: usize) -> Option<(String, String)> {
    let job = load(state, tab);
    // `*` copies the reference out of the list so `it` can be used on its own.
    let it = *arr(&job, "batch").get(i)?;
    let row = it.get("row").and_then(Value::as_object)?;
    let name = s(row, "plant_name");
    let empty = Map::new();
    let new = strings(it, "new_fields");
    // The page title differs per tab.
    Some(match tab {
        2 => (export::plant_html(it.get("llm")?.as_object()?, row, "AI Review — Tab 1", &[], &empty), export::download_name(&name, "_AI_Review.html")),
        3 => {
            let chg = it.get("changed_fields").and_then(Value::as_object).cloned().unwrap_or_default();
            (export::plant_html(it.get("llm")?.as_object()?, row, &format!("Fill Missing — {} new · {} changed", new.len(), chg.len()), &new, &chg), export::download_name(&name, "_Fill_Review.html"))
        }
        10 => {
            let filled = strings(it, "ai_filled");
            (export::plant_html(it.get("merged")?.as_object()?, row, &format!("Safe Fill — {} field(s) filled", filled.len()), &filled, &empty), export::download_name(&name, "_SafeFill.html"))
        }
        _ => return None,
    })
}

/// "🌐 Fetch & Stage Image from Web Search" in an AI result's editor.
/// Handles "Fetch & Stage Image": finds a free web image for the plant, saves
/// it as WebP in images/Database and adds it to the plant's picture list at
/// once (S-10 in DEVIATIONS.md). Takes the app state and the submitted form;
/// gives back a message. Called from `action` in mod.rs.
pub async fn stage_image(state: &AppState, f: &crate::formdata::FormData) -> Vec<Flash> {
    let id = crate::php::php_int(f.str("id"));
    // The plant's name and current picture list.
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as("SELECT plant_name, picture FROM data_source WHERE id = ?").bind(id).fetch_optional(&state.db).await.ok().flatten();
    let Some((name, picture)) = row else { return vec![Flash::Error(format!("Plant {id} was not found."))] };
    let name = name.unwrap_or_default();
    // Search for a free image using the botanical name, the name and any search term.
    let Some(bytes) = super::gallery::fetch_free_image(state, f.str("botanical_name"), &name, f.str("image_search_term")).await else {
        return vec![Flash::Error("No suitable imagery found online.".into())];
    };
    // Seconds since 1970, to make the file name unique.
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let fname = format!("{}_{id}_{ts}.webp", form::file_safe(&name));
    if let Err(e) = std::fs::write(state.www.join("images/Database").join(&fname), bytes) {
        return vec![Flash::Error(format!("Could not save the image: {e}"))];
    }
    // Existing pictures as a clean list, then the new file is added at the end.
    let mut pics: Vec<String> = picture.unwrap_or_default().split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty() && !matches!(x.to_lowercase().as_str(), "none" | "null" | "[]")).collect();
    pics.push(fname);
    let _ = sqlx::query("UPDATE data_source SET picture = ? WHERE id = ?").bind(pics.join(", ")).bind(id).execute(&state.db).await;
    state.invalidate_widget_cache();
    vec![Flash::Success("Staged image! It is now in this plant's gallery; click Save Changes to keep the AI's other values.".into())]
}

// Tests: run with `cargo test`; they are left out of the real program.
#[cfg(test)]
mod tests {
    use super::*;

    /// Each tab has its own idea of "empty": tab 10 treats "None" as a real answer.
    #[test]
    fn empties_follow_each_tab() {
        assert!(safe_empty(Some(&Value::String("Unknown".into()))));
        assert!(!safe_empty(Some(&Value::String("None".into()))));
        assert!(fill_empty(Some(&Value::String("............".into()))));
        assert!(!fill_empty(Some(&Value::String("0".into()))));
    }
}
