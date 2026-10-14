//! `/streamlit/` — the plant admin ("Jun & Kona's Garden Orchestrator"),
//! converted from the Streamlit app plant_orchestrator, 2026-10-07.
//!
//! HOW TO READ THIS FOLDER (for Jun). The Streamlit app was one page with 14
//! tabs; this is the same page, drawn by Rust. `page` answers GET /streamlit/
//! (`?tab=N` picks the tab, `&p=<id>` the plant, `&sub=` an inner tab) and
//! `action` answers the forms (Save, quantity events, bed assignment...).
//! After an action the browser is sent back to the same tab and plant with
//! the result shown at the top, which is what a Streamlit "rerun" looked like.
//!   opts.rs   the drop-down lists and the "missing fields" rule (config.py)
//!   ui.rs     Streamlit's widgets as HTML (st.text_input and friends)
//!   form.rs   the plant editor and its Save (components.py)
//!   tabs.rs   one function per tab, named after its Streamlit file
//!   st.css / st.js   Streamlit's look, and the chip boxes and calendar switch
//!
//! WHO MAY USE IT: the people in GARDEN_USERS (Jun), logged in with the garden login
//! (`visitor.is_admin()`); anybody else is sent to /login.php. The Streamlit
//! version sat behind a password page plus the author's household sign-in.
//!
//! DIFFERENCES FROM THE STREAMLIT APP are listed in
//! docs/streamlit-port/DEVIATIONS.md (S-1, S-2, ...).
//!
//! URLS HANDLED. GET /streamlit/ is `page` (shows a tab, or sends a download or a picture). POST
//! /streamlit/ is `action` (every button that changes something). GET /streamlit/st.css and
//! /streamlit/st.js are `css` and `js`. The routes are registered in the main router elsewhere;
//! this file only holds the handler functions.
//!
//! RUST IDEAS USED IN THIS FILE, explained where they first appear: `use` and `mod` lines,
//! `async`/`.await`, `Arc`, `Option`, `match`, `let ... else`, `if let`, the `?` marker, and
//! closures (small inline functions written `|x| ...`).

// A `HashMap` is a lookup table: give it a key (here a text name) and it gives back the value
// stored for that key.
use std::collections::HashMap;
// `Arc` is a shared pointer. The program keeps one `AppState` (database connection, folders, web
// client) and every request gets a cheap handle to the same one instead of its own copy.
use std::sync::Arc;

// axum is the web framework. These are the pieces used here: `Bytes` is the raw body of a request;
// `RawQuery` is the part of the address after "?"; `State` and `Extension` pull shared things into
// a handler; `HeaderMap` and `HeaderValue` are HTTP headers; `Html` and `Response` build the
// answer; `StatusCode` is a number such as 200 or 404.
use axum::{
    body::Bytes,
    extract::{RawQuery, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Extension,
};
// serde_json reads and writes JSON. `Map` is a JSON object and `Value` is any JSON value.
use serde_json::{Map, Value};

// `crate::` means "from this same program". `he` escapes text for HTML (so < becomes &lt;),
// `urlencode` makes text safe inside an address. They are named after the PHP functions they copy.
// `Visitor` is the logged-in person (set by the sign-in code), and `AppState` is the shared state
// described above.
use crate::php::{he, urlencode};
use crate::session::Visitor;
use crate::AppState;

/// Each `mod x;` line pulls in the file `x.rs` (or folder `x/`) next to this one. Without `pub`,
/// the module is private to the orchestrator folder. This one is tab 1, "Add Plant".
mod add;
/// Calls to the free cloud AI models, and the prompt rules.
mod ai;
/// The three AI tabs (Manual Review, Fill Missing Data, AI Missing Only).
mod ai_tabs;
/// The printable plant sheet and the missing-fields JSON.
mod export;
/// The plant editor and its Save button. `pub` lets other parts of the site use it too.
pub mod form;
/// The image manager (fetch, upload, remove photos).
mod gallery;
/// The drop-down lists and the "missing fields" rule.
pub mod opts;
/// Tab 11, the JSON Pipeline (export, AI in a file, import).
mod pipeline;
/// Reference text used by the JSON Pipeline.
mod pipeline_ref;
/// The wishlist side of the JSON Pipeline.
mod pipeline_wl;
/// One function per remaining tab, and the tab picker.
mod tabs;
/// Streamlit's widgets drawn as HTML.
pub mod ui;
/// Tab 13 (Wishlist) and tab 14 (Barcodes).
mod wishlist;

/// The tabs, exactly as Streamlit labelled them, in order.
/// Tab number N in the address (`?tab=N`) is entry N-1 in this list, because lists start counting
/// at 0. `[&str; 14]` means exactly 14 text pieces.
pub const TABS: [&str; 14] = [
    "🌱 Add Plant", "📝 Manual Review (AI)", "✨ Fill Missing Data (AI)", "✍️ Manual Editor",
    "📸 Manual Image Replace", "⚠️ Partials", "🔍 Awaiting Approval", "🏆 Fully Approved & Verified",
    "🚩 Flagged for Review", "🔒 AI — Missing Only", "🔄 JSON Pipeline", "🗺️ Bed Assignment",
    "🌟 Wishlist", "🏷️ Barcodes",
];

/// GET /streamlit/st.css. Sends the stylesheet. `async fn` is a function that can pause while it
/// waits (for the network or the database) without blocking other requests; callers use `.await` to
/// wait for it. `include_str!` copies the text of st.css into the program when it is built.
/// "no-cache" makes the browser re-check the file each time so changes show up.
pub async fn css() -> Response {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8"), (header::CACHE_CONTROL, "no-cache")], include_str!("st.css")).into_response()
}

/// GET /streamlit/st.js. Sends the small script that draws the chip boxes and switches the
/// calendar. Same method as `css`.
pub async fn js() -> Response {
    ([(header::CONTENT_TYPE, "application/javascript; charset=utf-8"), (header::CACHE_CONTROL, "no-cache")], include_str!("st.js")).into_response()
}

/// The request's own settings: which tab, plant, inner tab, and anything
/// else on the address.
/// A `struct` groups named fields into one value. This one is made once per page request.
pub struct Req {
    /// The tab number, 1 to 14.
    pub tab: usize,
    /// Every setting from the address, such as `p` (plant id) or `sub` (inner tab).
    pub q: HashMap<String, String>,
}

/// An `impl` block holds the functions (methods) that belong to a type.
impl Req {
    /// The value of setting `k`, or an empty string if it is not there. That way the callers never
    /// have to deal with "missing".
    pub fn get(&self, k: &str) -> &str {
        // `get` on a HashMap gives an Option: `Some(value)` or `None`. `.map(String::as_str)` turns
        // an owned String into a borrowed `&str`. `unwrap_or("")` uses "" when there was nothing.
        self.q.get(k).map(String::as_str).unwrap_or("")
    }
}

/// Every row of a query, as field maps.
/// Runs a read-only SQL query and gives back each row as a field map. On a database error it logs
/// the problem and returns an empty list, so one broken query shows "nothing found" instead of
/// crashing the page. `&AppState` borrows the shared state without copying it.
pub async fn rows(state: &AppState, sql: &str) -> Vec<Map<String, Value>> {
    // Send the query and wait (`.await`) for all rows. The result is either `Ok(rows)` or
    // `Err(error)`; `match` handles both.
    match sqlx::query(sql).fetch_all(&state.db).await {
        // Turn each database row into JSON; keep it only if it came out as a JSON object
        // (`filter_map` drops the others). `collect()` gathers what is left into a Vec.
        Ok(rs) => rs.iter().filter_map(|r| match crate::rowjson::row_to_json(r) {
            Value::Object(m) => Some(m),
            _ => None,
        }).collect(),
        // The query failed: write the error in the server log and return an empty list.
        Err(e) => {
            tracing::error!("orchestrator query failed: {e}");
            Vec::new()
        }
    }
}

/// `get_all_plant_names`: every plant name, sorted, once each.
/// The plant name picker list used by the companion-plant box. Reads every `plant_name`, drops
/// empty ones, sorts them, and removes repeats.
pub async fn all_plant_names(state: &AppState) -> Vec<String> {
    // `query_scalar` reads one column. The column can be empty, so each item is an
    // `Option<String>`; `flatten()` drops the empty ones, and `filter` drops names that are only
    // spaces. `unwrap_or_default()` gives an empty list if the query failed.
    let mut v: Vec<String> = sqlx::query_scalar::<_, Option<String>>("SELECT plant_name FROM data_source")
        .fetch_all(&state.db).await.unwrap_or_default().into_iter().flatten().filter(|s| !s.trim().is_empty()).collect();
    // `dedup` removes neighbours that are equal, which after sorting removes all repeats.
    v.sort();
    v.dedup();
    v
}

/// Builds a reply that sends the browser to another address. Status 303 "See Other" tells the
/// browser to load the new address with GET. This is used after a POST so that refreshing the page
/// does not send the form again.
fn redirect(to: &str) -> Response {
    (StatusCode::SEE_OTHER, [(header::LOCATION, to.to_string())]).into_response()
}

// A "flash" is a message shown once on the next page (like Streamlit's green success box). It is
// kept in a short-lived cookie named `st_flash`.
// ---- results shown after an action (a "flash", kept in a short cookie) ----

/// Packs a list of messages into a Set-Cookie header value. Each message becomes `["success",
/// "text"]` in a JSON list, which is then percent-encoded so it is safe in a cookie. Returns `None`
/// if the header text turns out to be invalid.
fn flash_cookie(msgs: &[form::Flash]) -> Option<HeaderValue> {
    // Turn each message into a two-item JSON list: the kind, then the text.
    let v: Vec<Value> = msgs.iter().map(|m| match m {
        form::Flash::Success(s) => serde_json::json!(["success", s]),
        form::Flash::Info(s) => serde_json::json!(["info", s]),
        form::Flash::Warning(s) => serde_json::json!(["warning", s]),
        form::Flash::Error(s) => serde_json::json!(["error", s]),
    }).collect();
    // Encode the whole JSON text for use inside a cookie.
    let enc = urlencode(&Value::Array(v).to_string());
    // Path=/streamlit limits the cookie to this tool. Max-Age=120 deletes it after two minutes.
    // HttpOnly hides it from page scripts. SameSite=Lax stops other sites from sending it along.
    // `.ok()` turns a possible error into `None`.
    HeaderValue::from_str(&format!("st_flash={enc}; Path=/streamlit; Max-Age=120; HttpOnly; SameSite=Lax")).ok()
}

/// Reads the `st_flash` cookie from the request and returns the HTML of the messages (empty text if
/// there is none or it is damaged).
fn read_flash(headers: &HeaderMap) -> String {
    // The Cookie header is one line like `a=1; b=2`. Read it as text, or "" if absent.
    let raw = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()).unwrap_or("");
    // `let ... else` tries to match a pattern and runs the `else` block if it does not match. Here:
    // split on ";", split each piece at the first "=", and find the pair named st_flash.
    let Some(val) = raw.split(';').filter_map(|p| p.trim().split_once('=')).find(|(k, _)| *k == "st_flash").map(|(_, v)| v) else {
        return String::new();
    };
    // Undo the percent-encoding.
    let decoded: String = url_decode(val);
    // Parse the text as JSON and require a list; anything else gives an empty result.
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(&decoded) else { return String::new() };
    // For each message, read the kind (item 0) and the text (item 1). The `?` after a call means
    // "if this gave nothing, stop and skip this message".
    items.iter().filter_map(|it| {
        let kind = it.get(0)?.as_str()?;
        let msg = it.get(1)?.as_str()?;
        // Pick the HTML box for that kind. Anything unknown is drawn as an error box.
        Some(match kind {
            "success" => ui::success(msg),
            "info" => ui::info(msg),
            "warning" => ui::warning(msg),
            _ => ui::error(msg),
        })
    }).collect()
}

/// Reverses percent-encoding: "+" becomes a space and "%41" becomes the byte 0x41. Written by hand
/// because it is small. Bytes that are not valid text become the replacement character instead of
/// failing.
fn url_decode(s: &str) -> String {
    // Forms write a space as "+".
    let s = s.replace('+', " ");
    // Work on raw bytes so that a "%xx" code can be turned into one byte.
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    // Walk through the bytes with the position `i`.
    while i < bytes.len() {
        // A "%" followed by at least two more bytes may be an encoded byte.
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            // Read the next two characters as a base-16 (hex) number. If that works, store the byte
            // and skip 3 positions.
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        // Not an encoded byte: copy it as it is.
        out.push(bytes[i]);
        i += 1;
    }
    // Turn the bytes back into text.
    String::from_utf8_lossy(&out).into_owned()
}

/// Wraps a tab's HTML in the whole page: the head with the stylesheet and script, the sidebar (who
/// is logged in, Logout, the AI model note) and the row of 14 tab links. `body` is the tab's HTML
/// and `flash` the messages from the last action. Returns the page as a `String`.
fn shell(visitor: &Visitor, tab: usize, flash: &str, body: &str) -> String {
    // Build the row of tab links. `mut` allows the String to grow.
    let mut tabs = String::new();
    // `enumerate` gives each tab its position; tab numbers start at 1, so the link uses i + 1. The
    // current tab gets the class "on" so it is highlighted.
    for (i, t) in TABS.iter().enumerate() {
        tabs.push_str(&format!("<a href=\"/streamlit/?tab={}\"{}>{}</a>", i + 1, if i + 1 == tab { " class=\"on\"" } else { "" }, he(t)));
    }
    // `format!` builds a String; `{name}` marks where a value is placed. Everything that comes from
    // outside (the user name) is escaped with `he()`.
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
<title>Garden Data Orchestrator</title><link rel=\"stylesheet\" href=\"/streamlit/st.css\"><script src=\"/streamlit/st.js\" defer></script></head>\
<body><div class=\"st-decoration\"></div><div class=\"st-app\"><aside class=\"st-sidebar\"><h2>⚙️ Settings</h2>\
<p>👤 <strong>Logged in as: {user}</strong></p>\
<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"logout\"><button class=\"st-btn\" type=\"submit\">Logout</button></form>\
<label class=\"st-field\"><span class=\"st-label\">AI model</span><input class=\"st-input\" value=\"free cloud models (Gemini, OpenRouter)\" disabled></label>\
<p class=\"st-caption\">The Streamlit app's \"Claude Model Name\" box. The AI tabs now use free cloud models only (DEVIATIONS.md S-4).</p>\
<p class=\"st-caption\"><a href=\"/\">← Jun's Garden</a></p></aside>\
<main class=\"st-main\">{h1}<nav class=\"st-tabs\">{tabs}</nav>{flash}{body}</main></div></body></html>",
        user = he(&visitor.username),
        h1 = ui::h1("🌱 Jun & Kona's Garden Orchestrator"),
    )
}

/// GET /streamlit/
/// Answers GET /streamlit/. The parameters are filled in by axum: the shared state, the signed-in
/// visitor, the raw address settings and the request headers. Returns `Response`, which is anything
/// that can be sent to the browser. Besides drawing a tab, it also serves small side requests: the
/// image preview, the tag guide picture and the download buttons (`?export=...`).
pub async fn page(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    RawQuery(raw): RawQuery,
    headers: HeaderMap,
) -> Response {
    // Only the garden's accounts (GARDEN_USERS) may use this tool. Anyone else is sent to the login page and then back
    // here.
    if !visitor.is_admin() {
        return redirect("/login.php?back=%2Fstreamlit%2F");
    }
    // The address's settings. A multi-pick box (`pick[]`) sends one value per
    // choice; those are joined with ", " the way formdata.rs joins a POST.
    let mut q: HashMap<String, String> = HashMap::new();
    // Read each setting from the address. A repeated name ending in "[]" (a multi-pick box) has its
    // values joined with ", " into one text.
    for (k, v) in serde_urlencoded::from_str::<Vec<(String, String)>>(raw.as_deref().unwrap_or("")).unwrap_or_default() {
        match q.get_mut(&k) {
            // `Some(old) if k.ends_with("[]")` means "the key already exists and is a list-style
            // name": add to the old value.
            Some(old) if k.ends_with("[]") => { old.push_str(", "); old.push_str(&v); }
            // Otherwise store or replace the value.
            _ => { q.insert(k, v); }
        }
    }
    // The image manager's "does this look right?" preview picture.
    // `?preview=<id>`: send the waiting preview picture as WebP, or 404 (not found) if there is
    // none.
    if let Some(pid) = q.get("preview") {
        return match gallery::preview_bytes(&state, crate::php::php_int(pid)) {
            Some(b) => ([(header::CONTENT_TYPE, "image/webp"), (header::CACHE_CONTROL, "no-store")], b).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        };
    }
    // `?tagguide`: send the plant tag guide picture from the data folder (`std::fs::read` reads a
    // whole file into memory).
    if q.contains_key("tagguide") {
        return match std::fs::read(state.data.join("plant_tag_guide.jpg")) {
            Ok(b) => ([(header::CONTENT_TYPE, "image/jpeg")], b).into_response(),
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        };
    }
    // Plant id for the export links. `php_int` reads text as a whole number the PHP way (bad text
    // becomes 0).
    let export_id = crate::php::php_int(q.get("id").map(String::as_str).unwrap_or(""));
    // `?export=<kind>` asks for a file download. Each kind has its own source; `return` leaves the
    // function with that file right away.
    match q.get("export").map(String::as_str) {
        // Plant schema reference text file.
        Some("schema_ref") => return attachment(add::REFERENCE.to_string(), "text/plain; charset=utf-8", "plant_schema_reference.txt"),
        // The Add Plant tab's schema JSON for one plant.
        Some("add_schema") => return match add::schema_json(&state, export_id).await {
            Some((body, name)) => attachment(body, "application/json", &name),
            None => StatusCode::NOT_FOUND.into_response(),
        },
        // The Add Plant tab's missing-fields JSON for one plant.
        Some("add_missing") => return match add::missing_json(&state, export_id).await {
            Some((body, name)) => attachment(body, "application/json", &name),
            None => StatusCode::NOT_FOUND.into_response(),
        },
        // JSON Pipeline downloads; their names start with "jp_".
        Some(k) if k.starts_with("jp_") => return match pipeline::export(&state, k, &q).await {
            Some((body, name)) => attachment(body, "application/json", &name),
            None => StatusCode::NOT_FOUND.into_response(),
        },
        // Any other kind: carry on below.
        _ => {}
    }
    // `ai_html`: the printable sheet for an AI result. `n` is a small closure (inline function)
    // that reads a setting as a number, giving 0 if it is missing or not a number.
    if q.get("export").map(String::as_str) == Some("ai_html") {
        let n = |k: &str| q.get(k).and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
        return match ai_tabs::export(&state, n("tab"), n("i")) {
            Some((html, name)) => attachment(html, "text/html; charset=utf-8", &name),
            None => StatusCode::NOT_FOUND.into_response(),
        };
    }
    // Remaining kinds (the plain HTML and missing-fields buttons) are handled by `download`.
    if let Some(kind) = q.get("export") {
        return download(&state, kind, &q).await;
    }
    // Pick the tab: read `tab`, parse it as a number, keep it only if it is 1 to 14, otherwise use
    // tab 1.
    let tab = q.get("tab").and_then(|t| t.parse::<usize>().ok()).filter(|t| (1..=TABS.len()).contains(t)).unwrap_or(1);
    // Bundle the tab and settings for the tab functions.
    let req = Req { tab, q };
    // Draw the tab. This waits on the database, hence `.await`.
    let body = tabs::render(&state, &req).await;
    // Messages left by the previous action, as HTML.
    let flash = read_flash(&headers);
    // Build the page reply.
    let mut res = Html(shell(&visitor, tab, &flash, &body)).into_response();
    // The messages were shown, so delete the cookie (Max-Age=0) so they do not appear again.
    if !flash.is_empty() {
        res.headers_mut().append(header::SET_COOKIE, HeaderValue::from_static("st_flash=; Path=/streamlit; Max-Age=0"));
    }
    res
}

/// Makes a file-download reply. `Content-Disposition: attachment` makes the browser save the file
/// under `name` instead of showing it. Quotes, backslashes and line breaks in the name are replaced
/// with "_" so they cannot break the header.
fn attachment(body: String, mime: &str, name: &str) -> Response {
    // The header text for the file name.
    let disp = format!("attachment; filename=\"{}\"", name.replace(['"', '\\', '\r', '\n'], "_"));
    // The reply with the content type set (`mime` tells the browser what kind of file it is).
    let mut res = ([(header::CONTENT_TYPE, mime.to_string())], body).into_response();
    // Add the header. If the name has characters a header cannot hold, fall back to a plain name
    // that keeps only the file's extension.
    if let Ok(v) = HeaderValue::from_str(&disp) {
        res.headers_mut().insert(header::CONTENT_DISPOSITION, v);
    } else if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"plant{}\"", name.rsplit_once('.').map(|(_, e)| format!(".{e}")).unwrap_or_default())) {
        res.headers_mut().insert(header::CONTENT_DISPOSITION, v);
    }
    res
}

/// The download buttons: `?export=html&kind=partial|manual&id=` and
/// `?export=missing_json&id=`.
/// Handles the plain download buttons: `export=html` (a printable sheet for a Partial or a Manual
/// Editor plant) and `export=missing_json`. `kind` is the download kind and `q` all the address
/// settings. A missing plant gives a 404.
async fn download(state: &AppState, kind: &str, q: &HashMap<String, String>) -> Response {
    // The plant id.
    let id = crate::php::php_int(q.get("id").map(String::as_str).unwrap_or(""));
    // Load that plant's whole row. `fetch_optional` gives `Some(row)` or `None`. `?` is not used;
    // every failure here returns 404.
    let row = match sqlx::query("SELECT * FROM data_source WHERE id = ?").bind(id).fetch_optional(&state.db).await {
        Ok(Some(r)) => match crate::rowjson::row_to_json(&r) { Value::Object(m) => m, _ => return StatusCode::NOT_FOUND.into_response() },
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    // The plant name, used for the file name.
    let name = opts::text(row.get("plant_name"));
    // Pick the download.
    match kind {
        "html" => {
            // A Partial export says which fields are missing in its title; the Manual Editor one
            // says "Manual Editor Export". `suffix` is the title text and `file` the end of the
            // file name.
            let (suffix, file) = if q.get("kind").map(String::as_str) == Some("partial") {
                (format!("Partial Export — Missing: {}", opts::missing_fields(&row).join(", ")), "_Partial_Export.html")
            } else {
                ("Manual Editor Export".to_string(), "_Manual_Export.html")
            };
            // An empty map: no AI "changed" fields here.
            let none = Map::new();
            attachment(export::plant_html(&row, &row, &suffix, &[], &none), "text/html; charset=utf-8", &export::download_name(&name, file))
        }
        "missing_json" => attachment(export::missing_json(&row), "application/json", &export::download_name(&name, "_missing_fields.json")),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// POST /streamlit/ — every button that changes something.
/// Answers POST /streamlit/. Reads the submitted form, runs the matching action (named by the
/// hidden field `act`), then redirects back to the same tab and plant with the result messages in
/// the flash cookie. The messages come back as a list of `Flash` values (success, info, warning or
/// error).
pub async fn action(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Same login check as `page`.
    if !visitor.is_admin() {
        return redirect("/login.php?back=%2Fstreamlit%2F");
    }
    // Read the submitted form (plain or multipart with file uploads).
    let f = crate::formdata::parse(&headers, body).await;
    // Logout: destroy the server-side session, then go to the login page with a message.
    if f.str("act") == "logout" {
        crate::session::destroy(&state, &visitor.sid).await;
        return redirect("/login.php?msg=out");
    }
    // Where the Add Plant steps go next (the new record, the AI answer...).
    // `next` holds extra address settings that an action wants in the redirect.
    let mut next: Vec<(&str, String)> = Vec::new();
    // Pick the code to run from the action name. The names start with a prefix that tells which
    // area handles it: "add_" Add Plant, "jp_" JSON Pipeline, "ai_" the AI tabs, "wl_" wishlist,
    // "bc_" barcodes, "img_" image manager.
    let msgs = match f.str("act") {
        // An arm with `if` is a match arm with an extra condition.
        a if a.starts_with("add_") => {
            let (m, n) = add::action(&state, a, &f).await;
            next = n;
            m
        }
        a if a.starts_with("jp_") => {
            let (m, n) = pipeline::action(state.clone(), a, &f).await;
            next = n;
            m
        }
        // Save from the AI tabs, when the "stage image" box is on: only the picture is staged.
        "save_plant" if f.str("stage_img") == "1" => ai_tabs::stage_image(&state, &f).await,
        // The editor's normal Save.
        "save_plant" => form::save(&state, &f).await,
        // `a @ (...)` matches either name and keeps it in `a`.
        a @ ("ai_start" | "ai_reset") => ai_tabs::action(state.clone(), a, crate::php::php_int(f.str("tab")).max(0) as usize).await,
        // The quantity logger (plants added, lost, moved).
        "qty_event" => form::save_qty_event(&state, &f).await,
        // Everything else is handled in tabs.rs.
        other => tabs::action(&state, other, &f).await,
    };
    // Back to the same tab, plant and inner tab.
    // Build the address to return to: the tab (kept between 1 and 14), then either what the action
    // asked for or the usual settings. `clamp(1, 14)` forces the number into that range.
    let mut to = format!("/streamlit/?tab={}", crate::php::php_int(f.str("tab")).clamp(1, TABS.len() as i64));
    if !next.is_empty() {
        for (k, v) in &next {
            to.push_str(&format!("&{k}={}", urlencode(v)));
        }
    } else {
        // These settings are carried back so the same plant and inner tab are shown again.
        for k in ["p", "sub", "show", "filter", "batch", "new", "full", "ai", "qc"] {
            if !f.str(k).is_empty() {
                to.push_str(&format!("&{k}={}", urlencode(f.str(k))));
            }
        }
    }
    // Redirect, and attach the flash cookie so the next page shows the messages.
    let mut res = redirect(&to);
    if let Some(c) = flash_cookie(&msgs) {
        res.headers_mut().append(header::SET_COOKIE, c);
    }
    res
}
