//! The image manager — Streamlit's `image_manager.py`, used by the
//! "📸 Manual Image Replace" tab and under the editor in "⚠️ Partials".
//!
//! HOW TO READ THIS FILE (for Jun). `render` draws the two columns: the
//! plant's photos on the left (remove one, mark the gallery verified, wipe
//! all), and on the right "Option 1: Fetch from Web" (Wikipedia first, then
//! DuckDuckGo images; no key needed) and "Option 2: Upload Your Own".
//! `action` does what those buttons ask. A fetched picture waits as a
//! preview file in the data folder (`orchestrator-preview/<id>.webp`) until
//! "Confirm & Save" or "Discard"; Streamlit kept it in memory instead.
//!
//! Like the Streamlit widget, this one stores every photo comma-separated
//! in `picture` (the editor uses picture + additional_pictures).
//!
//! BUTTONS AND THEIR `act` NAMES. Every button is a small form that posts to /streamlit/ with a
//! hidden `act` field: img_remove, img_verify, img_unverify, img_wipe, img_fetch, img_confirm,
//! img_discard and img_upload. `tabs::action` hands any name starting with "img_" to `action`
//! below.
//!
//! RUST WORDS USED HERE. `Option<T>` is `Some(value)` or `None`. `Result<T, E>` is `Ok(value)` or
//! `Err(error)`. `async fn` can pause while it waits for the database or the network; callers write
//! `.await` to wait. `let Some(x) = ... else { ... }` stops early when the pattern does not match.
//! `?` after a call means "if this failed, return the failure from this function now". `&str` is
//! borrowed text, `String` is owned text.

// `Map` is a JSON object (one database row) and `Value` is any JSON value.
use serde_json::{Map, Value};

// `file_safe` makes a plant name safe for a file name, and `Flash` is a message shown after an
// action (Success, Info, Warning or Error).
use super::form::{file_safe, Flash};
use super::opts::text;
use super::ui;
use crate::php::{he, php_int};
use crate::AppState;

/// The User-Agent header sent when asking Wikipedia, which asks every program to say who it is.
const UA: &str = "JunsGardenBot/1.0";

/// Short helper: field `k` of the plant row as plain text (empty if missing).
fn s(m: &Map<String, Value>, k: &str) -> String {
    text(m.get(k))
}

/// Splits the stored photo list ("a.webp, b.webp") into separate file names. It trims spaces and
/// drops empty items and the words "none" and "null".
fn pics_of(raw: &str) -> Vec<String> {
    raw.split(',').map(|f| f.trim().to_string()).filter(|f| !f.is_empty() && !matches!(f.to_lowercase().as_str(), "none" | "null")).collect()
}

/// The folder (inside the data folder) where a fetched picture waits for "Confirm & Save".
/// `PathBuf` is an owned file path.
fn preview_dir(state: &AppState) -> std::path::PathBuf {
    state.data.join("orchestrator-preview")
}

/// The form header every button form shares: which action, which plant,
/// and where to come back to.
/// Starts a button form: the opening `<form>` tag plus hidden fields for the action name (`act`),
/// the plant id, and everything in `back` (the tab, plant and filter to return to). The caller adds
/// the button and the closing `</form>`. `multipart` is needed for file uploads.
fn form_open(act: &str, pid: &str, back: &[(&str, String)], multipart: bool) -> String {
    // Build the first part of the form. The `{act}` and `{}` marks are filled from the values after
    // the text.
    let mut h = format!(
        "<form method=\"post\" action=\"/streamlit/\"{}><input type=\"hidden\" name=\"act\" value=\"{act}\"><input type=\"hidden\" name=\"id\" value=\"{}\">",
        if multipart { " enctype=\"multipart/form-data\"" } else { "" }, he(pid)
    );
    // Add one hidden field per "back" setting, escaped with `he`.
    for (k, v) in back {
        h.push_str(&format!("<input type=\"hidden\" name=\"{}\" value=\"{}\">", he(k), he(v)));
    }
    h
}

/// Draws the image manager for one plant. `plant` is its database row, `back` the settings to
/// return to after a button, and `wipe_confirm` is true when the "Wipe ALL images" confirmation
/// should show. Returns HTML in two columns: the left shows the photos and their buttons, the right
/// has the two ways to add a photo.
pub fn render(state: &AppState, plant: &Map<String, Value>, back: &[(&str, String)], wipe_confirm: bool) -> String {
    // The plant id, the list of photo names, and the folder where photos live.
    let pid = s(plant, "id");
    let pics = pics_of(&s(plant, "picture"));
    let dir = state.www.join("images/Database");

    // Build the left column.
    let mut left = String::new();
    // No photos yet: show a blue note.
    if pics.is_empty() {
        left.push_str(&ui::info("No images yet — use the options on the right."));
    } else {
        // For each photo, show it (if the file exists) with its name, then a "Remove" button. The
        // hidden `idx` is the photo's position in the list, so the action knows which one to
        // remove.
        for (i, f) in pics.iter().enumerate() {
            if dir.join(f).is_file() {
                left.push_str(&format!("<img src=\"/images/Database/{}\" width=\"280\" alt=\"\"><div class=\"st-img-cap\" style=\"text-align:left\">{}</div>", ui::path_url(f), he(f)));
            } else {
                left.push_str(&ui::caption(&format!("⚠️ File not found: {f}")));
            }
            left.push_str(&form_open("img_remove", &pid, back, false));
            left.push_str(&format!("<input type=\"hidden\" name=\"idx\" value=\"{i}\">"));
            left.push_str(&ui::button("go", "1", "🗑️ Remove this image", false, true));
            left.push_str("</form>");
        }
        // Whether the gallery is already marked verified; the button then offers the opposite
        // change.
        let verified = matches!(s(plant, "image_verified").trim(), "1" | "true" | "True");
        // The Verify / Unverify button.
        left.push_str(&form_open(if verified { "img_unverify" } else { "img_verify" }, &pid, back, false));
        left.push_str(&ui::button("go", "1", if verified { "❌ Mark as Unverified" } else { "✅ Gallery Looks Good! (Mark as Verified)" }, false, true));
        left.push_str("</form>");
        // First click on "Wipe ALL images" is only a link that reloads the page with `wipe=1`;
        // nothing is deleted yet.
        if !wipe_confirm {
            // Rebuild the address settings as `key=value` pairs, each value percent-encoded.
            let mut q: Vec<String> = back.iter().map(|(k, v)| format!("{k}={}", crate::php::urlencode(v))).collect();
            q.push("wipe=1".into());
            left.push_str(&format!("<a class=\"st-btn st-full\" href=\"/streamlit/?{}\">🗑️ Wipe ALL images</a>", he(&q.join("&"))));
        // Second step: ask "Are you sure?" with a Yes button (which posts `img_wipe`) and a Cancel
        // link back to the page without `wipe=1`.
        } else {
            left.push_str(&ui::warning(&format!("⚠️ Wipe ALL images for **{}**? Cannot be undone.", s(plant, "plant_name"))));
            let cancel: Vec<String> = back.iter().map(|(k, v)| format!("{k}={}", crate::php::urlencode(v))).collect();
            left.push_str(&ui::columns(&[1, 1], vec![
                form_open("img_wipe", &pid, back, false) + &ui::button("go", "1", "✅ Yes, wipe", true, true) + "</form>",
                format!("<a class=\"st-btn st-full\" href=\"/streamlit/?{}\">❌ Cancel</a>", he(&cancel.join("&"))),
            ]));
        }
    }

    // A closure that shows "N/A" for an empty value.
    let or_na = |v: String| if v.trim().is_empty() { "N/A".to_string() } else { v };
    // Build the right column: a summary line for the plant.
    let mut right = ui::markdown(&format!("**{}**  |  Botanical: {}  |  Family: {}", s(plant, "plant_name"), or_na(s(plant, "botanical_name")), or_na(s(plant, "family"))));
    // Option 1: fetch a picture from the web.
    right.push_str(&ui::subheader("Option 1: Fetch from Web"));
    right.push_str(&ui::caption("Searches Wikipedia first, then DuckDuckGo — no API key."));
    // The search term starts as the saved `image_search_term`, or the plant name if that is empty.
    let term = { let t = s(plant, "image_search_term"); if t.trim().is_empty() { s(plant, "plant_name") } else { t } };
    // Search form: a term box and the fetch button.
    right.push_str(&form_open("img_fetch", &pid, back, false));
    right.push_str(&ui::text_input("term", "Search term:", &term, ""));
    right.push_str(&ui::button("go", "1", "🔍 Fetch Image Preview", false, true));
    right.push_str("</form>");
    // If a fetched picture is waiting, show it with Confirm and Discard buttons. The picture is
    // served by the `?preview=<id>` address in mod.rs.
    if preview_dir(state).join(format!("{pid}.webp")).is_file() {
        right.push_str(&ui::divider());
        right.push_str(&ui::h4("🖼️ Preview — does this look right?"));
        right.push_str(&format!("<img src=\"/streamlit/?preview={}\" width=\"340\" alt=\"\">", he(&pid)));
        right.push_str(&ui::columns(&[1, 1], vec![
            form_open("img_confirm", &pid, back, false) + &ui::button("go", "1", "✅ Confirm & Save", true, true) + "</form>",
            form_open("img_discard", &pid, back, false) + &ui::button("go", "1", "❌ Discard", false, true) + "</form>",
        ]));
    }
    // Option 2: upload your own photos. The file box accepts several files.
    right.push_str(&ui::divider());
    right.push_str(&ui::subheader("Option 2: Upload Your Own"));
    right.push_str(&form_open("img_upload", &pid, back, true));
    right.push_str(&ui::file_uploader("uploads[]", "Upload photos (multiple allowed)", &["jpg", "jpeg", "png", "webp"], true));
    // The radio choices, with the more likely one first: a plant with no photos defaults to
    // "replace", one with photos defaults to "append".
    let choices: [&str; 2] = if pics.is_empty() { ["Wipe gallery and replace", "Append to current gallery"] } else { ["Append to current gallery", "Wipe gallery and replace"] };
    right.push_str(&ui::radio("up_act", "Upload action:", &choices, choices[0], false, false));
    // st.js fills this box with a preview of the chosen files and shows the hidden save button only
    // after files are chosen.
    right.push_str("<div data-upload-preview></div>");
    // The save button is hidden at first (`display:none`) by changing its HTML.
    right.push_str(&ui::button("go", "1", "✅ Process & Save", true, true).replace("class=\"st-btn", "data-upload-save style=\"display:none\" class=\"st-btn"));
    right.push_str("</form>");

    // Left column one share wide, right column two shares.
    ui::columns(&[1, 2], vec![left, right])
}

/// Writes the photo list back to the database: the names joined with ", ", or empty (NULL) if there
/// are none. A gallery with photos is marked verified; an empty one is not. Also clears the site's
/// widget cache so the public pages show the change. Returns `Ok(())` or the database error.
async fn save_pics(state: &AppState, pid: i64, pics: &[String]) -> Result<(), sqlx::Error> {
    // `?` marks in SQL are filled by `.bind(...)` in order, so values never get mixed into the SQL
    // text. This prevents SQL injection.
    sqlx::query("UPDATE data_source SET picture = ?, image_verified = ? WHERE id = ?")
        .bind(if pics.is_empty() { None } else { Some(pics.join(", ")) })
        .bind(if pics.is_empty() { 0 } else { 1 })
        .bind(pid)
        .execute(&state.db).await?;
    // The public pages keep a cache; tell it to rebuild.
    state.invalidate_widget_cache();
    Ok(())
}

/// Seconds since 1970 (the Unix time). Used to make unique file names. If the clock reads before
/// 1970 it gives 0.
fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Runs one image button. `act` is the button's name and `f` the submitted form (including uploaded
/// files). Returns a list of messages to show. An empty list means "done, nothing to say".
pub async fn action(state: &AppState, act: &str, f: &crate::formdata::FormData) -> Vec<Flash> {
    // The plant id from the hidden field.
    let pid = php_int(f.str("id"));
    // Load the plant's name and photo list. The `.ok().flatten()` steps turn "query failed" and "no
    // row" into the same `None`.
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as("SELECT plant_name, picture FROM data_source WHERE id = ?")
        .bind(pid).fetch_optional(&state.db).await.ok().flatten();
    // If the plant does not exist, report that and stop.
    let Some((name, picture)) = row else { return vec![Flash::Error(format!("Plant {pid} was not found."))] };
    let name = name.unwrap_or_default();
    let pics = pics_of(&picture.unwrap_or_default());
    // A closure that wraps a database error into a message list, used wherever a query can fail.
    let fail = |e: sqlx::Error| vec![Flash::Error(format!("Database error: {e}"))];
    // Choose what to do from the button name.
    match act {
        // Remove one photo from the list. The file itself stays on disk; only the list changes.
        "img_remove" => {
            // The position to remove. `.max(0)` stops a negative number, and `as usize` makes it a
            // list position.
            let i = php_int(f.str("idx")).max(0) as usize;
            // If the position does not exist, do nothing.
            let Some(gone) = pics.get(i).cloned() else { return vec![] };
            // Keep every photo except the one at position `i`.
            let remaining: Vec<String> = pics.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, p)| p.clone()).collect();
            // Save the new list; on failure show the error.
            if let Err(e) = save_pics(state, pid, &remaining).await { return fail(e) }
            vec![Flash::Success(format!("🗑️ Removed `{gone}`{}", if remaining.is_empty() { " — gallery now empty." } else { "" }))]
        }
        // Mark the gallery as verified or not verified.
        "img_verify" | "img_unverify" => {
            let v = if act == "img_verify" { 1 } else { 0 };
            match sqlx::query("UPDATE data_source SET image_verified = ? WHERE id = ?").bind(v).bind(pid).execute(&state.db).await {
                Ok(_) => vec![],
                Err(e) => fail(e),
            }
        }
        // Remove all photo names for the plant and clear the verified flag. The photo files stay on
        // disk.
        "img_wipe" => match sqlx::query("UPDATE data_source SET picture = NULL, additional_pictures = NULL, image_verified = 0 WHERE id = ?").bind(pid).execute(&state.db).await {
            Ok(_) => { state.invalidate_widget_cache(); vec![Flash::Success("🗑️ All images wiped.".into())] }
            Err(e) => fail(e),
        },
        // Look for a picture on the web and keep it as a preview file.
        "img_fetch" => {
            let term = f.str("term").trim().to_string();
            // Read the botanical and common names to use as search terms.
            let (bot, plant_name): (Option<String>, Option<String>) = sqlx::query_as("SELECT botanical_name, plant_name FROM data_source WHERE id = ?")
                .bind(pid).fetch_one(&state.db).await.unwrap_or((None, None));
            // The result is `Some(bytes)` when a picture was found. The bytes are saved as
            // `<id>.webp`, and the search term as `<id>.term`. `let _ =` ignores a failure to
            // write, because the preview is only a convenience. If nothing was found, any old
            // preview is deleted and an error is shown.
            match fetch_free_image(state, bot.as_deref().unwrap_or(""), plant_name.as_deref().unwrap_or(""), &term).await {
                Some(bytes) => {
                    let dir = preview_dir(state);
                    let _ = std::fs::create_dir_all(&dir);
                    let _ = std::fs::write(dir.join(format!("{pid}.webp")), bytes);
                    let _ = std::fs::write(dir.join(format!("{pid}.term")), term);
                    vec![]
                }
                None => {
                    let _ = std::fs::remove_file(preview_dir(state).join(format!("{pid}.webp")));
                    vec![Flash::Error("No image found — try a different search term.".into())]
                }
            }
        }
        // Delete the waiting preview and its term.
        "img_discard" => {
            let _ = std::fs::remove_file(preview_dir(state).join(format!("{pid}.webp")));
            let _ = std::fs::remove_file(preview_dir(state).join(format!("{pid}.term")));
            vec![]
        }
        // Make the preview a real photo: save it in the photo folder under a unique name (plant
        // name, id, time), add it to the list, remember the search term, and delete the preview
        // files.
        "img_confirm" => {
            let dir = preview_dir(state);
            // No preview file: nothing to do.
            let Ok(bytes) = std::fs::read(dir.join(format!("{pid}.webp"))) else { return vec![] };
            let term = std::fs::read_to_string(dir.join(format!("{pid}.term"))).unwrap_or_default();
            let fname = format!("{}_{}_{}.webp", file_safe(&name), pid, now());
            if let Err(e) = std::fs::write(state.www.join("images/Database").join(&fname), bytes) {
                return vec![Flash::Error(format!("Could not save the image: {e}"))];
            }
            let mut all = pics.clone();
            all.push(fname.clone());
            if let Err(e) = save_pics(state, pid, &all).await { return fail(e) }
            let _ = sqlx::query("UPDATE data_source SET image_search_term = ? WHERE id = ?").bind(&term).bind(pid).execute(&state.db).await;
            let _ = std::fs::remove_file(dir.join(format!("{pid}.webp")));
            let _ = std::fs::remove_file(dir.join(format!("{pid}.term")));
            vec![Flash::Success(format!("✅ Saved `{fname}`"))]
        }
        // Save the uploaded files as WebP photos.
        "img_upload" => {
            // The uploaded files. No files means nothing to do.
            let ups = f.files_for("uploads[]");
            if ups.is_empty() {
                return vec![];
            }
            // One time stamp for the batch; each file adds its position so names stay different.
            let t = now();
            let mut new_files = Vec::new();
            // For each upload: files already in WebP are saved as they are; other formats are
            // converted to WebP with quality 87. A file that cannot be read or written is skipped.
            for (i, u) in ups.iter().enumerate() {
                let fname = format!("{}_{}_manual_{}.webp", file_safe(&name), pid, t + i as u64);
                let bytes = if u.filename.to_lowercase().ends_with(".webp") { Some(u.bytes.clone()) } else { crate::images::to_webp(&u.bytes, 87.0) };
                if let Some(b) = bytes {
                    if std::fs::write(state.www.join("images/Database").join(&fname), b).is_ok() {
                        new_files.push(fname);
                    }
                }
            }
            // "Append" keeps the old photos and adds the new ones; otherwise the new ones replace
            // the old list.
            let fin: Vec<String> = if f.str("up_act") == "Append to current gallery" { pics.iter().cloned().chain(new_files.iter().cloned()).collect() } else { new_files.clone() };
            if let Err(e) = save_pics(state, pid, &fin).await { return fail(e) }
            vec![Flash::Success(format!("✅ {} image(s) saved: {}", new_files.len(), new_files.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ")))]
        }
        // Any other button name.
        _ => vec![Flash::Error("Unknown action.".into())],
    }
}

/// The preview file, for <img src="/streamlit/?preview=<id>">.
/// Reads the waiting preview picture for plant `pid`, or `None` if there is none. The address
/// `?preview=<id>` in mod.rs sends it to the browser.
pub fn preview_bytes(state: &AppState, pid: i64) -> Option<Vec<u8>> {
    std::fs::read(preview_dir(state).join(format!("{pid}.webp"))).ok()
}

// ---- finding a picture on the web (no key) --------------------------------

/// Downloads `url` and returns the bytes only if the answer was a success and its content type
/// starts with "image". Gives up after 10 seconds. Returns `None` on any problem.
async fn get_bytes(state: &AppState, url: &str, ua: &str) -> Option<Vec<u8>> {
    // `?` here returns `None` early if the request failed (this function returns an Option, so `?`
    // works on Options and Results alike).
    let r = state.http.get(url).header("User-Agent", ua).timeout(std::time::Duration::from_secs(10)).send().await.ok()?;
    if !r.status().is_success() {
        return None;
    }
    // The Content-Type header: it must say the answer is an image, not a web page.
    let ct = r.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    if !ct.starts_with("image") {
        return None;
    }
    r.bytes().await.ok().map(|b| b.to_vec())
}

/// Asks Wikipedia's API for a page matching `term`, then for that page's lead picture (640 pixels
/// wide), and downloads it. Returns the picture bytes or `None`.
async fn wikipedia(state: &AppState, term: &str) -> Option<Vec<u8>> {
    // Both questions go to the same API address.
    let api = "https://en.wikipedia.org/w/api.php";
    // First question: search for the best matching page title (`srlimit=1` means one result).
    let search: Value = state.http.get(api).header("User-Agent", UA).timeout(std::time::Duration::from_secs(8))
        .query(&[("action", "query"), ("list", "search"), ("srsearch", term), ("format", "json"), ("srlimit", "1")])
        .send().await.ok()?.json().await.ok()?;
    // Read the title with a JSON pointer: `/query/search/0/title` means query, then search, then
    // first item, then title.
    let title = search.pointer("/query/search/0/title")?.as_str()?.to_string();
    // Second question: the picture ("pageimages") of that page.
    let pages: Value = state.http.get(api).header("User-Agent", UA).timeout(std::time::Duration::from_secs(8))
        .query(&[("action", "query"), ("titles", title.as_str()), ("prop", "pageimages"), ("format", "json"), ("pithumbsize", "640")])
        .send().await.ok()?.json().await.ok()?;
    // Try each returned page's thumbnail and use the first one that downloads.
    for page in pages.pointer("/query/pages")?.as_object()?.values() {
        if let Some(src) = page.pointer("/thumbnail/source").and_then(Value::as_str) {
            if let Some(b) = get_bytes(state, src, UA).await {
                return Some(b);
            }
        }
    }
    None
}

/// DuckDuckGo image search the way the duckduckgo_search library does it:
/// get the page's `vqd` token, then ask `i.js` for results.
/// Returns the picture bytes of the first result that downloads, or `None`.
async fn duckduckgo(state: &AppState, term: &str) -> Option<Vec<u8>> {
    // The search text is the term plus the word "plant", so results show plants.
    let q = format!("{term} plant");
    // Step 1: load DuckDuckGo's image search page.
    let page = state.http.get("https://duckduckgo.com/").header("User-Agent", "Mozilla/5.0")
        .query(&[("q", q.as_str()), ("iax", "images"), ("ia", "images")])
        .timeout(std::time::Duration::from_secs(10)).send().await.ok()?.text().await.ok()?;
    // Step 2: pull the `vqd` token out of the page text: take what follows "vqd=", then keep only
    // letters, digits and "-". The `i.js` request needs this token.
    let vqd = page.split("vqd=").nth(1).map(|r| r.trim_start_matches(['"', '\'']).chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect::<String>())?;
    if vqd.is_empty() {
        return None;
    }
    // Step 3: ask `i.js` for the image results in JSON.
    let res: Value = state.http.get("https://duckduckgo.com/i.js").header("User-Agent", "Mozilla/5.0").header("Referer", "https://duckduckgo.com/")
        .query(&[("l", "us-en"), ("o", "json"), ("q", q.as_str()), ("vqd", vqd.as_str()), ("f", ",,,,,"), ("p", "1")])
        .timeout(std::time::Duration::from_secs(10)).send().await.ok()?.json().await.ok()?;
    // Step 4: try the first 8 results; use the first image that downloads.
    for hit in res.get("results")?.as_array()?.iter().take(8) {
        let url = hit.get("image").or_else(|| hit.get("thumbnail")).and_then(Value::as_str).unwrap_or("");
        if !url.is_empty() {
            if let Some(b) = get_bytes(state, url, "Mozilla/5.0").await {
                return Some(b);
            }
        }
    }
    None
}

/// `fetch_free_image`: Wikipedia for the botanical name, the search term,
/// then the plant name; then DuckDuckGo. Shrunk to fit 900x900, WebP 87.
/// Takes the botanical name, plant name and search term. Returns a WebP picture as bytes (or
/// `None`). No key is needed.
pub async fn fetch_free_image(state: &AppState, botanical: &str, plant_name: &str, term: &str) -> Option<Vec<u8>> {
    // `found` will hold the picture once one is downloaded.
    let mut found = None;
    // Try Wikipedia with each name in turn: botanical name, search term, plant name. Skip empty
    // names and "none" or "null".
    for t in [botanical, term, plant_name] {
        let t = t.trim();
        if !t.is_empty() && !matches!(t.to_lowercase().as_str(), "none" | "null") {
            if let Some(b) = wikipedia(state, t).await {
                found = Some(b);
                break;
            }
        }
    }
    // No Wikipedia picture: try DuckDuckGo with the search term, or the plant name if the term is
    // empty.
    if found.is_none() {
        found = duckduckgo(state, if term.trim().is_empty() { plant_name } else { term }).await;
    }
    // Decode the downloaded bytes as a picture (`found?` stops with `None` if there is no picture).
    let img = image::load_from_memory(&found?).ok()?;
    // Shrink it to fit inside 900 by 900 pixels, keeping its proportions.
    let img = img.thumbnail(900, 900);
    // Save it as WebP, quality 87.
    crate::images::to_webp_image(&img, 87.0)
}
