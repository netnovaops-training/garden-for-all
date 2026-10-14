//! `comments.php` — the comment API behind the gallery lightbox and the
//! plant window.
//!
//! Comments live in two JSON files in the data folder (gallery comments keyed
//! by file name, plant comments keyed by plant id), plus a one-character
//! "someone commented" flag file the admin's bell reads. These are the same
//! files and the same shapes the PHP wrote, so either site can read the
//! other's data.
//! URLs: `/comments.php` only. It answers GET and POST and chooses what to do
//! from the query string or form fields (see the numbered blocks in `comments`):
//!   GET  ?get_comments=1&type=..&key=..  list comments (newest first, max 50)
//!   GET  ?check_notify=1                 does the admin bell have news?
//!   POST dismiss_notify                  admin clears the bell
//!   POST post_comment                    add a comment (anyone, including guests)
//!   GET  ?admin_dump=1                   admin: all comments of one kind
//!   POST delete_comment                  admin: remove one comment
//! The gallery lightbox and the plant window's JavaScript call these.
//!
//! HOW TO READ THIS FILE. Small helpers first (read/write the JSON files, cut
//! text, stamp the date), then `append_comment`, then the big `comments` handler,
//! then tests. Other files (such as the Android API) reuse `append_comment`.

// `use` lines bring names into scope. `Arc` is a shared pointer to one value.
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, Method},
    response::{IntoResponse, Response},
    Extension,
};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::formdata;
use crate::php::{he, php_int};
use crate::session::Visitor;
use crate::AppState;

/// Pick the JSON file for a kind of comment: "gallery" or anything else (plants).
/// Takes the app state (it knows the data folder) and the kind as text.
/// Gives back a `PathBuf`, an owned file path.
fn file_for(state: &AppState, kind: &str) -> PathBuf {
    state.data.join(if kind == "gallery" { "gallery_comments.json" } else { "plant_comments.json" })
}

/// Read a JSON file and return its top-level object as a map.
/// Gives an empty map if the file is missing, unreadable or not an object.
/// That is on purpose: a first-ever comment must work before the file exists.
/// `.ok()` turns a Result into an Option (drops the error); `.and_then` chains a
/// step that runs only if the previous one gave a value; `unwrap_or_default`
/// supplies an empty map for "nothing". `&b` borrows the bytes without copying.
pub fn read_json(path: &std::path::Path) -> Map<String, Value> {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .and_then(|v| if let Value::Object(m) = v { Some(m) } else { None })
        .unwrap_or_default()
}

/// Write the map back to disk as pretty JSON.
/// Takes the file path and the map. Gives back `io::Result<()>`: `Ok` or an
/// I/O error. The `?` on the serialize line returns early with the error if
/// that step fails, so the next line only runs on success.
/// `json_encode($data, JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE)`: four
/// spaces of indent, as PHP writes it.
pub fn write_json(path: &std::path::Path, data: &Map<String, Value>) -> std::io::Result<()> {
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    data.serialize(&mut ser).map_err(std::io::Error::other)?;
    std::fs::write(path, buf)
}

/// Cut `s` to at most `n` bytes without splitting a multi-byte character.
/// Takes text and a byte limit. Gives back a new `String`. Plain `s[..n]`
/// would crash on a letter such as "e with accent" that takes two bytes, so we
/// step back to the nearest character boundary first.
/// `substr($s, 0, $n)`, but never cutting a character in half.
pub fn substr(s: &str, n: usize) -> String {
    if s.len() <= n {
        return s.to_string();
    }
    let mut end = n;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// Gives true for "" and "0", which PHP's `empty()` also treats as empty.
/// Kept so the Rust site accepts and rejects the same input as the PHP did.
/// PHP's `empty()` for a string.
fn empty(s: &str) -> bool {
    s.is_empty() || s == "0"
}

/// Turn a JSON value into an HTTP reply. Sets `Cache-Control: no-store` so the
/// browser never shows an old comment list from its cache.
/// Falls back to `{}` if the value cannot be written as text (it always can).
fn json_reply(body: Value) -> Response {
    let mut res = (
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&body).unwrap_or_else(|_| "{}".into()),
    )
        .into_response();
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// Make the date text and the Unix timestamp stored with a new comment.
/// Takes nothing. Gives back a pair: (for example "Oct 5, 2026", seconds count).
/// The format string is the Rust form of PHP's `date('M j, Y')`.
/// `date('M j, Y')` and `time()` for a new comment.
pub fn comment_stamp() -> (String, i64) {
    let now = common::now_local();
    let date = now
        .format(&time::macros::format_description!("[month repr:short] [day padding:none], [year]"))
        .unwrap_or_default();
    (date, now.unix_timestamp())
}

/// Takes: app state, the JSON file's path, the key (photo name or plant id) and
/// the comment as JSON. Gives back nothing. Steps: wait for the lock so two
/// people posting at once cannot overwrite each other, read the file, push the
/// comment onto that key's list, write the file, raise the bell flag.
/// `.lock().await` waits for our turn; the lock is released when `_guard`
/// goes out of scope at the end of the function.
/// Append one comment under `key` and raise the admin's flag.
pub async fn append_comment(state: &AppState, path: &std::path::Path, key: &str, entry: Value) {
    // 1. Take the lock. Only one task at a time may edit the comment files.
    let _guard = state.comments_lock.lock().await;
    // 2. Read the whole file (it is small).
    let mut all = read_json(path);
    // 3. Find this key's list, creating an empty one when the key is new.
    let list = all.entry(key.to_string()).or_insert_with(|| json!([]));
    // 4. `if let` runs its block only when the value has that shape (an array).
    if let Value::Array(a) = list {
        a.push(entry);
    }
    // 5. Write the file. On failure log it; the visitor still gets an answer.
    if let Err(e) = write_json(path, &all) {
        tracing::error!("could not save comment to {}: {e}", path.display());
    }
    // 6. Write "1" into the flag file so the admin's bell lights up.
    // `let _ =` deliberately ignores the result: a failed flag is not worth an error.
    let _ = std::fs::write(state.data.join("comment_notify.txt"), "1");
}

/// The route handler for `/comments.php`. One function serves every comment
/// action; it tests the HTTP method and which field is present, in order.
/// Takes: state, the visitor (admin or guest), the HTTP method, the query
/// string as a map, the headers and the raw body. Gives back a JSON reply.
pub async fn comments(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    method: Method,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Read the form fields from the body (form posts or file posts alike).
    let f = formdata::parse(&headers, body).await;
    let is_admin = visitor.is_admin();
    let notify_file = state.data.join("comment_notify.txt");

    // 1. GET ?get_comments: load comments for one photo or plant.
    // `Option` means maybe-nothing; `unwrap_or("plant")` supplies the default.
    // Load the comments for one photo or plant: newest first, at most 50.
    if method == Method::GET && q.contains_key("get_comments") {
        let kind = q.get("type").map(String::as_str).unwrap_or("plant");
        let key = q.get("key").map(|k| k.trim()).unwrap_or("");
        if empty(key) {
            return json_reply(json!({ "comments": [] }));
        }
        let all = read_json(&file_for(&state, kind));
        // `.rev()` newest first, `.take(50)` at most 50, `.cloned()` copies each one.
        let rows: Vec<Value> = all
            .get(key)
            .and_then(Value::as_array)
            .map(|a| a.iter().rev().take(50).cloned().collect())
            .unwrap_or_default();
        return json_reply(json!({ "comments": rows }));
    }

    // 2. GET ?check_notify: the admin's bell. Guests always get "false" so they
    // cannot learn whether anyone has commented.
    // The admin's bell.
    if method == Method::GET && q.contains_key("check_notify") {
        if !is_admin {
            return json_reply(json!({ "notify": false }));
        }
        let flag = std::fs::read_to_string(&notify_file).map(|s| s.trim().to_string()).unwrap_or_else(|_| "0".into());
        return json_reply(json!({ "notify": flag == "1" }));
    }
    // 3. POST dismiss_notify: switch the bell off. Admin only.
    if method == Method::POST && f.has("dismiss_notify") {
        if !is_admin {
            return json_reply(json!({ "status": "denied" }));
        }
        let _ = std::fs::write(&notify_file, "0");
        return json_reply(json!({ "status": "ok" }));
    }

    // 4. POST post_comment: anyone may comment, but the text is escaped with `he`
    // (like PHP's htmlspecialchars) so a comment cannot inject HTML. Names are
    // cut to 30 bytes and messages to 200, as in the PHP.
    // Add a comment.
    if method == Method::POST && f.has("post_comment") {
        let kind = f.get("type").unwrap_or("plant").to_string();
        let key = f.str("key").trim().to_string();
        let name = he(f.get("name").map(str::trim).unwrap_or("Guest"));
        let msg = he(f.str("message").trim());
        if empty(&key) || empty(&msg) {
            return json_reply(json!({ "status": "error", "message": "key and message required" }));
        }
        let name = substr(if empty(&name) { "Guest" } else { &name }, 30);
        let msg = substr(&msg, 200);
        let (date, ts) = comment_stamp();
        let entry = json!({ "name": name, "message": msg, "date": date, "ts": ts });
        append_comment(&state, &file_for(&state, &kind), &key, entry.clone()).await;
        return json_reply(json!({ "status": "ok", "entry": entry }));
    }

    // 5. Admin tools: dump every comment of a kind, or delete one by position.
    // The admin's review tools.
    if method == Method::GET && q.contains_key("admin_dump") && is_admin {
        let kind = q.get("type").map(String::as_str).unwrap_or("plant");
        return json_reply(Value::Object(read_json(&file_for(&state, kind))));
    }
    if method == Method::POST && f.has("delete_comment") && is_admin {
        let kind = f.get("type").unwrap_or("plant").to_string();
        let key = f.str("key").trim().to_string();
        // `index` is the position in the list; -1 means "not given" and is rejected.
        let index = f.get("index").map(php_int).unwrap_or(-1);
        let path = file_for(&state, &kind);
        // Take the lock before reading so the delete cannot collide with a new post.
        let _guard = state.comments_lock.lock().await;
        let mut all = read_json(&path);
        // Remove the comment, and the whole key when its list becomes empty.
        // `Some(Value::Array(a)) if ...` matches only when the key holds a list and
        // the index is inside it; `_` catches everything else.
        let found = match all.get_mut(&key) {
            Some(Value::Array(a)) if index >= 0 && (index as usize) < a.len() => {
                a.remove(index as usize);
                if a.is_empty() {
                    all.remove(&key);
                }
                true
            }
            _ => false,
        };
        if found {
            let _ = write_json(&path, &all);
            return json_reply(json!({ "status": "ok" }));
        }
        return json_reply(json!({ "status": "error", "message": "Not found" }));
    }

    // 6. Nothing matched: tell the caller the request was not understood.
    json_reply(json!({ "status": "error", "message": "Unknown request" }))
}

// Tests run only with `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    // Checks that cutting text never splits a character: "e with accent" takes two
    // bytes, so asking for 2 bytes of "a" plus it must give just "a".
    #[test]
    fn substr_never_splits_a_character() {
        assert_eq!(substr("abcdef", 3), "abc");
        assert_eq!(substr("aé", 2), "a");
        assert_eq!(substr("short", 30), "short");
    }

    // Writes a small file to a scratch folder and checks the 4-space indent.
    #[test]
    fn written_json_uses_four_space_indent() {
        let dir = std::env::temp_dir().join(format!("jg-test-{}", std::process::id())); // nosemgrep: rust.lang.security.temp-dir.temp-dir -- unit-test scratch folder, never runs on the server
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("c.json");
        let mut m = Map::new();
        m.insert("k".into(), json!([{"a": 1}]));
        write_json(&p, &m).unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("\n    \"k\": ["), "{s}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
