//! `comments.php` — the comment API behind the gallery lightbox and the plant window.
//!
//! Comments live in two JSON files in the data folder (gallery comments keyed by photo
//! file name, plant comments keyed by plant number), plus a one-character "someone
//! commented" flag file the admin's bell reads. Same files and shapes as the PHP wrote.
//!
//! URL: `/comments.php`, GET and POST; what it does depends on the field that is present:
//!   GET  ?get_comments=1&type=..&key=..  list comments (newest first, at most 50)
//!   GET  ?check_notify=1                 does the admin bell have news?
//!   POST dismiss_notify                  admin clears the bell
//!   POST post_comment                    add a comment (anyone, including guests)
//!   GET  ?admin_dump=1                   admin: all comments of one kind
//!   POST delete_comment                  admin: remove one comment
//!
//! HOW TO READ THIS FILE (for Jun). `Kind` says which file; `latest`, `new_entry` and
//! `add` are the three things both this page and the app API (`app_api.rs`) need; the
//! handler `comments` at the bottom picks the action. File safety (no half-written files,
//! a damaged file is never overwritten) is in `store.rs`.
//!
//! Guards on posting (new in the Rust site): at most 5 comments per visitor address per
//! 10 minutes, a plant key must be a plant number and a gallery key a photo that exists,
//! and each photo or plant keeps its newest `KEEP_PER_KEY` comments.

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
use serde_json::{json, Value};

use crate::formdata;
use crate::guard::off_thread;
use crate::php::{he, php_int};
use crate::session::Visitor;
use crate::store;
use crate::AppState;

/// How many comments are shown, newest first.
const SHOWN: usize = 50;
/// How many comments one photo or plant keeps; the oldest beyond this are dropped.
const KEEP_PER_KEY: usize = 500;

/// Which comments file: the gallery's or the plants'. Anything but "gallery" means plants,
/// as in the PHP.
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Gallery,
    Plant,
}

impl Kind {
    fn from(s: &str) -> Kind {
        if s == "gallery" {
            Kind::Gallery
        } else {
            Kind::Plant
        }
    }

    fn file(self, state: &AppState) -> PathBuf {
        state.data.join(match self {
            Kind::Gallery => "gallery_comments.json",
            Kind::Plant => "plant_comments.json",
        })
    }
}

/// Cut `s` to at most `n` bytes without splitting a character (`substr($s, 0, $n)`).
pub fn substr(s: &str, n: usize) -> String {
    let mut end = s.len().min(n);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// PHP's `empty()` for a string: nothing, or exactly "0".
fn empty(s: &str) -> bool {
    s.is_empty() || s == "0"
}

/// A new comment as it is stored: name and message escaped (so no markup ever reaches a
/// page) and cut to 30 and 200 bytes, the date as "Oct 5, 2026" and the Unix time.
/// `None` when the message is empty.
pub fn new_entry(name: &str, message: &str) -> Option<Value> {
    let name = he(name.trim());
    let msg = he(message.trim());
    if empty(&msg) {
        return None;
    }
    let name = substr(if empty(&name) { "Guest" } else { &name }, 30);
    let now = common::now_local();
    let date = now
        .format(&time::macros::format_description!("[month repr:short] [day padding:none], [year]"))
        .unwrap_or_default();
    Some(json!({ "name": name, "message": substr(&msg, 200), "date": date, "ts": now.unix_timestamp() }))
}

/// The newest comments for one photo or plant (at most 50, newest first). A damaged file
/// shows no comments (and is logged) rather than an error.
pub async fn latest(state: &AppState, kind: Kind, key: &str) -> Vec<Value> {
    let (path, key) = (kind.file(state), key.to_string());
    off_thread(move || match store::read_json(&path) {
        Ok(all) => all
            .get(&key)
            .and_then(Value::as_array)
            .map(|a| a.iter().rev().take(SHOWN).cloned().collect())
            .unwrap_or_default(),
        Err(e) => {
            tracing::error!("comments: {e}");
            Vec::new()
        }
    })
    .await
}

/// Is `key` something that can be commented on? A plant number, or the file name of a
/// photo that is in the gallery folder. Stops anybody filling the file with made-up keys.
fn valid_key(state: &AppState, kind: Kind, key: &str) -> bool {
    match kind {
        Kind::Plant => !key.is_empty() && key.len() <= 10 && key.bytes().all(|b| b.is_ascii_digit()),
        Kind::Gallery => {
            key.len() <= 255
                && !key.starts_with('.')
                && !key.contains(['/', '\\', '\0'])
                && state.www.join("images/Gallery").join(key).is_file()
        }
    }
}

/// Append one comment under `key`, keep the newest `KEEP_PER_KEY`, raise the admin's
/// flag. Returns false (and logs why) when it could not be saved.
pub async fn add(state: &Arc<AppState>, kind: Kind, key: &str, entry: Value) -> bool {
    let _one_at_a_time = state.comments_lock.lock().await;
    let (path, key, flag) = (kind.file(state), key.to_string(), state.data.join("comment_notify.txt"));
    off_thread(move || {
        let saved = store::update_json(&path, |all| {
            let list = all.entry(key).or_insert_with(|| json!([]));
            let Value::Array(a) = list else { return None };
            a.push(entry);
            if a.len() > KEEP_PER_KEY {
                a.drain(..a.len() - KEEP_PER_KEY);
            }
            Some(())
        });
        match saved {
            Ok(Some(())) => {
                // A failed flag only means the bell stays dark; not worth failing the comment.
                let _ = store::write_atomic(&flag, b"1");
                true
            }
            Ok(None) => false,
            Err(e) => {
                tracing::error!("could not save a comment: {e}");
                false
            }
        }
    })
    .await
}

/// A JSON answer that is never cached by the browser (an old comment list must not show).
fn json_reply(body: Value) -> Response {
    let mut res = ([(header::CONTENT_TYPE, "application/json")], body.to_string()).into_response();
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

fn error(message: &str) -> Response {
    json_reply(json!({ "status": "error", "message": message }))
}

/// The handler for `/comments.php`: tests the method and which field is present, in order.
pub async fn comments(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    method: Method,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let f = formdata::parse(&headers, body).await;
    let is_admin = visitor.is_admin();
    let get = method == Method::GET;
    let post = method == Method::POST;
    let query = |k: &str| q.get(k).map(String::as_str).unwrap_or("");
    let notify_file = state.data.join("comment_notify.txt");

    // Load the comments for one photo or plant.
    if get && q.contains_key("get_comments") {
        let key = query("key").trim();
        if empty(key) {
            return json_reply(json!({ "comments": [] }));
        }
        let kind = Kind::from(q.get("type").map(String::as_str).unwrap_or("plant"));
        return json_reply(json!({ "comments": latest(&state, kind, key).await }));
    }

    // The admin's bell. Guests always get "false", so they cannot tell whether anyone commented.
    if get && q.contains_key("check_notify") {
        let on = is_admin && off_thread(move || store::read_text(&notify_file).unwrap_or_default()).await.trim() == "1";
        return json_reply(json!({ "notify": on }));
    }
    if post && f.has("dismiss_notify") {
        if !is_admin {
            return json_reply(json!({ "status": "denied" }));
        }
        off_thread(move || {
            if let Err(e) = store::write_atomic(&notify_file, b"0") {
                tracing::error!("could not clear the comment bell: {e}");
            }
        })
        .await;
        return json_reply(json!({ "status": "ok" }));
    }

    // Add a comment. Anyone may; the text is escaped and cut in `new_entry`.
    if post && f.has("post_comment") {
        let kind = Kind::from(f.get("type").unwrap_or("plant"));
        let key = f.str("key").trim();
        let entry = if empty(key) { None } else { new_entry(f.get("name").unwrap_or("Guest"), f.str("message")) };
        let Some(entry) = entry else { return error("key and message required") };
        if !valid_key(&state, kind, key) {
            return error("Unknown photo or plant");
        }
        if !is_admin && !state.write_limit.allow(&crate::accounts::client_ip(&headers)) {
            return error("Too many comments at once. Please wait a few minutes.");
        }
        if !add(&state, kind, key, entry.clone()).await {
            return error("The comment could not be saved");
        }
        return json_reply(json!({ "status": "ok", "entry": entry }));
    }

    // The admin's review tools: every comment of a kind, or delete one by position.
    if get && q.contains_key("admin_dump") && is_admin {
        let path = Kind::from(q.get("type").map(String::as_str).unwrap_or("plant")).file(&state);
        let all = off_thread(move || store::read_json(&path).unwrap_or_default()).await;
        return json_reply(Value::Object(all));
    }
    if post && f.has("delete_comment") && is_admin {
        let path = Kind::from(f.get("type").unwrap_or("plant")).file(&state);
        let key = f.str("key").trim().to_string();
        // -1 means "not given" and matches nothing.
        let index = f.get("index").map(php_int).unwrap_or(-1);
        let _one_at_a_time = state.comments_lock.lock().await;
        let removed = off_thread(move || {
            store::update_json(&path, |all| {
                let Some(Value::Array(a)) = all.get_mut(&key) else { return None };
                let i = usize::try_from(index).ok().filter(|i| *i < a.len())?;
                a.remove(i);
                if a.is_empty() {
                    all.remove(&key);
                }
                Some(())
            })
            .map_err(|e| tracing::error!("could not delete a comment: {e}"))
            .ok()
            .flatten()
            .is_some()
        })
        .await;
        return if removed { json_reply(json!({ "status": "ok" })) } else { error("Not found") };
    }

    error("Unknown request")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substr_never_splits_a_character() {
        assert_eq!(substr("abcdef", 3), "abc");
        assert_eq!(substr("aé", 2), "a");
        assert_eq!(substr("short", 30), "short");
    }

    #[test]
    fn entries_are_escaped_cut_and_need_a_message() {
        let e = new_entry("<b>Kona</b>", " hi <script> ").unwrap();
        assert_eq!(e["name"], "&lt;b&gt;Kona&lt;/b&gt;");
        assert_eq!(e["message"], "hi &lt;script&gt;");
        assert_eq!(new_entry("", "x").unwrap()["name"], "Guest");
        assert_eq!(new_entry("0", "x").unwrap()["name"], "Guest");
        assert!(new_entry("Kona", "  ").is_none());
        assert!(new_entry("Kona", "0").is_none());
        assert_eq!(new_entry(&"n".repeat(99), &"m".repeat(999)).unwrap()["message"].as_str().unwrap().len(), 200);
    }
}
