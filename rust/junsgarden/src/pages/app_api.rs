//! `app_api.php` — what the Android app talks to. Nothing else does.
//!
//! THIS IS A CONTRACT. The installed phone app expects these exact
//! `?action=` names and these exact JSON keys; changing one breaks the app
//! for everybody until a new APK is built. Every action here answers with
//! the same shape the PHP did, including the odd ones (wish-list rows with
//! negative ids).
//!
//! Writes need somebody logged in: the app logs in with `action=login`
//! (username + password, POST), gets a key, and sends it as
//! `Authorization: Bearer <key>`; anyone else gets a 401 with
//! `needs_sign_in`. Guests can read everything, exactly like the website.
//!
//! HOW TO READ THIS FILE (for Jun). `app_api` is a `match` on the
//! `?action=` name, one line per action, each calling a function below.
//! The app only READS (`list`, `detail`, `bed_counts`, `barcode_lookup`,
//! `thumb`); the write actions exist because the PHP had them. If the app
//! ever needs a new field, add it to the JSON here, never rename or remove
//! an existing one, and ship the app change after the site change.

// Imports. `use` brings a name from another file or a library into this file so it can be written
// short.
use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use serde_json::{json, Map, Value};
use sqlx::Row;

use crate::formdata::{self, FormData};
use crate::images;
use crate::pages::comments::{append_comment, comment_stamp, read_json, substr};
use crate::php::{date_c, he, php_int};
use crate::rowjson::row_to_json;
use crate::session::Visitor;
use crate::AppState;

/// The web path in front of every plant photo name. The app builds picture addresses from this plus
/// the file name stored in the database.
///
/// Rust idea: a `const` is a fixed value known at build time. `&str` is a borrowed piece of text
/// that is never changed (a `String` is text you own and can change).
const IMG_BASE: &str = "/images/Database/";
/// How many plants one page of the app's list holds. `i64` is a whole number. The list asks the
/// database for one extra row to learn if a next page exists.
const PAGE_SIZE: i64 = 20;

/// Build a JSON answer with an HTTP status code (200 OK, 401 unauthorised, and so on).
///
/// Every app answer goes through here, so every answer carries the same headers: JSON content type,
/// `no-store` (do not cache) and permission for any web page to read it (CORS). Takes the status
/// and a JSON `Value`; gives back a `Response`.
fn reply(status: StatusCode, body: Value) -> Response {
    let mut res = (
        status,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        serde_json::to_string(&body).unwrap_or_else(|_| "{}".into()),
    )
        .into_response();
    let h = res.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    h.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET, OPTIONS"));
    res
}

/// Shorthand for a successful `reply` with status 200. Most actions end with `ok(json!({...}))`;
/// the `json!` macro writes a JSON object directly in Rust.
fn ok(body: Value) -> Response {
    reply(StatusCode::OK, body)
}

/// Refuse a write by somebody not signed in. Valid JSON, always.
///
/// Takes the visitor. Gives back `Ok(username)` if signed in, or `Err(response)` holding the 401
/// answer. Rust idea: `Result<A, B>` is "good answer A or error B"; each write action does `match`
/// on it and returns the error response straight away, so a write can never run for a guest.
fn require_actor(visitor: &Visitor) -> Result<String, Response> {
    if visitor.username.is_empty() {
        return Err(reply(
            StatusCode::UNAUTHORIZED,
            json!({
                "ok": false,
                "error": "Log in with a garden account to change the catalogue.",
                "needs_sign_in": true,
            }),
        ));
    }
    Ok(visitor.username.clone())
}

/// A de-duplicated list of image web paths for a plant row.
///
/// Takes one plant row as a JSON map. Reads `picture` first, then `additional_pictures`
/// (comma-separated file names), skips blanks and "0", and builds `/images/Database/<name>`. Order
/// matters: the app shows the first image as the cover.
fn plant_image_urls(p: &Map<String, Value>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for col in ["picture", "additional_pictures"] {
        if let Some(s) = p.get(col).and_then(Value::as_str).filter(|s| !s.is_empty() && *s != "0") {
            for name in s.split(',') {
                let name = name.trim();
                if !name.is_empty() && !out.iter().any(|o| o == &format!("{IMG_BASE}{name}")) {
                    out.push(format!("{IMG_BASE}{name}"));
                }
            }
        }
    }
    out
}

/// Strip trademark glyphs that show up oddly on mobile, and trim.
fn clean_strings(row: &mut Map<String, Value>) {
    for (_, v) in row.iter_mut() {
        if let Value::String(s) = v {
            *s = s.replace(['™', '®'], "").trim().to_string();
        }
    }
}

/// Read one request field. Looks in the posted form first, then in the address (`?name=value`), and
/// gives "" if neither has it.
///
/// The app sometimes sends a value one way and sometimes the other, and the PHP accepted both. The
/// `'a` marks tell Rust the answer lives as long as the inputs do.
fn pick<'a>(f: &'a FormData, q: &'a HashMap<String, String>, k: &str) -> &'a str {
    f.get(k).or_else(|| q.get(k).map(String::as_str)).unwrap_or("")
}

/// True if the field was sent at all (form or address), even when empty. Used to tell "clear the
/// barcode" apart from "do not touch the barcode".
fn is_set(f: &FormData, q: &HashMap<String, String>, k: &str) -> bool {
    f.has(k) || q.contains_key(k)
}

/// Columns of a table that are not binary.
///
/// Used by the `export` action so raw image bytes are never put into JSON. Asks MariaDB's
/// `information_schema` for the column names and types, then drops any type containing "blob" or
/// "binary".
async fn text_columns(state: &AppState, table: &str) -> Vec<String> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT COLUMN_NAME, COLUMN_TYPE FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .filter(|(_, t)| {
        let t = t.to_lowercase();
        !t.contains("blob") && !t.contains("binary")
    })
    .map(|(n, _)| n)
    .collect()
}

/// The one entry point for `app_api.php`, the address the Android app calls.
///
/// Takes the shared app data, the visitor (set by the sign-in layer from the app's Bearer key), the
/// HTTP method, the address parameters, headers and body. Gives back JSON. It reads `?action=` and
/// hands the work to one function per action; an unknown or missing action runs `list`, as the PHP
/// did.
///
/// Rust idea: `async` means this function can pause while waiting for the database, and `.await` is
/// where it pauses. The arguments are axum "extractors" that pull each piece out of the request.
/// `Arc` is a shared pointer, so many requests can use one copy of the app data.
pub async fn app_api(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    method: Method,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // 1. A browser's "may I call this?" pre-check (CORS OPTIONS) gets an empty 204 answer.
    if method == Method::OPTIONS {
        return reply(StatusCode::NO_CONTENT, Value::Null);
    }
    // 2. Read the body as a form (works for normal and file-upload forms) and find which action was
    // asked for.
    let f = formdata::parse(&headers, body).await;
    let action = q.get("action").map(String::as_str).unwrap_or("list").to_string();

    // 3. `match` picks the code for the action name. Each line below is `"name" => function`. The
    // `_` line at the end catches everything else.
    match action.as_str() {
        "ping" => ok(json!({ "ok": true, "time": date_c() })),
        "thumb" => thumb(&state, &q).await,
        // POST action=login with username + password (form fields, never
        // the address, so a password does not end up in anybody's logs).
        // Answers with a KEY the app keeps and sends as
        // `Authorization: Bearer <key>` from then on (see session.rs).
        "login" => {
            if method != Method::POST {
                return reply(StatusCode::METHOD_NOT_ALLOWED, json!({ "ok": false, "error": "Use POST." }));
            }
            let ip = crate::accounts::client_ip(&headers);
            match crate::accounts::check_password(&state, f.str("username"), f.str("password"), &ip).await {
                Ok(name) => {
                    let token = crate::session::issue_app_token(&state, &name).await;
                    ok(json!({ "ok": true, "username": name, "token": token }))
                }
                Err(crate::accounts::LoginError::TooMany) => reply(
                    StatusCode::TOO_MANY_REQUESTS,
                    json!({ "ok": false, "error": "Too many login attempts. Please wait 15 minutes." }),
                ),
                Err(crate::accounts::LoginError::Invalid) => reply(
                    StatusCode::UNAUTHORIZED,
                    json!({ "ok": false, "error": "Invalid username or password" }),
                ),
            }
        }
        // The app's Log out: throw this key away.
        "logout" => {
            if let Some(t) = &visitor.app_token {
                crate::session::destroy(&state, t).await;
            }
            ok(json!({ "ok": true }))
        }
        // Who does this key belong to? Lets the app check its saved key at start.
        "whoami" => {
            if visitor.username.is_empty() {
                reply(StatusCode::UNAUTHORIZED, json!({ "ok": false, "needs_sign_in": true }))
            } else {
                ok(json!({ "ok": true, "username": visitor.username }))
            }
        }
        "add" => add_plant(&state, &visitor, &f, &q).await,
        "future_search" => future_search(&state, &q).await,
        "future_promote" => future_promote(&state, &visitor, &f, &q).await,
        "barcode_lookup" => barcode_lookup(&state, &f, &q).await,
        "barcodes" => barcodes(&state, &f, &q).await,
        "barcode_image" => barcode_image(&state, &f, &q).await,
        "add_barcode" => add_barcode(&state, &visitor, &f, &q).await,
        "delete_barcode" => delete_barcode(&state, &visitor, &f, &q).await,
        "set_barcode" => set_barcode(&state, &visitor, &f, &q).await,
        "export" => export(&state).await,
        "catalog_signature" => catalog_signature(&state).await,
        "bed_counts" => bed_counts(&state).await,
        "update_plant" => update_plant(&state, &visitor, &f, &q).await,
        "like" | "favorite" => like_or_favorite(&state, &action, &f, &q).await,
        // Read comments for a plant from the JSON file on disk: newest first, at most 50. (Comments
        // are kept in a file, not the database, as in the PHP.)
        "comments" => {
            let pid = q.get("plant_id").map(|p| php_int(p)).unwrap_or(0);
            let all = read_json(&state.data.join("plant_comments.json"));
            let rows: Vec<Value> = all
                .get(&pid.to_string())
                .and_then(Value::as_array)
                .map(|a| a.iter().rev().take(50).cloned().collect())
                .unwrap_or_default();
            ok(json!({ "comments": rows }))
        }
        // Add a comment. Text is HTML-escaped (`he`) and cut to 30 characters for the name and 200
        // for the message, so nobody can put markup in the page.
        "comment" => {
            let pid = php_int(pick(&f, &q, "plant_id"));
            let name = {
                let n = substr(&he(pick(&f, &q, "name").trim()), 30);
                if n.is_empty() { "Guest".to_string() } else { n }
            };
            let name = if f.has("name") || q.contains_key("name") { name } else { "Guest".to_string() };
            let msg = substr(&he(pick(&f, &q, "message").trim()), 200);
            if pid <= 0 || msg.is_empty() {
                return ok(json!({ "ok": false, "error": "A message is required." }));
            }
            // Stamp the time, build the comment, append it to the file, and send it back so the app
            // can show it.
            let (date, ts) = comment_stamp();
            let entry = json!({ "name": name, "message": msg, "date": date, "ts": ts });
            append_comment(&state, &state.data.join("plant_comments.json"), &pid.to_string(), entry.clone()).await;
            ok(json!({ "ok": true, "entry": entry }))
        }
        "autocomplete" => autocomplete(&state, &q).await,
        "detail" => detail(&state, &q).await,
        _ => list(&state, &q).await,
    }
}

/// `?action=thumb&img=<filename>&w=<width>`: a small cached WebP.
///
/// Steps: 1 clean the file name, 2 clamp the width, 3 serve a cached copy if it is newer than the
/// original, 4 otherwise make a new WebP thumbnail, save it, and serve it.
async fn thumb(state: &AppState, q: &HashMap<String, String>) -> Response {
    // 1. Read `img` and `w` from the address.
    let img = q.get("img").map(|s| s.trim()).unwrap_or("");
    // basename() blocks path traversal.
    // Keep only the part after the last slash. This stops `../../` tricks from reading other files.
    let img = img.rsplit(['/', '\\']).next().unwrap_or("").to_string();
    // Width defaults to 300 and is held between 40 and 1200 so nobody asks for a huge image. `as
    // u32` converts the number type.
    let w = q.get("w").map(|w| php_int(w)).unwrap_or(300).clamp(40, 1200) as u32;
    let src = state.www.join("images/Database").join(&img);
    if img.is_empty() || !src.is_file() {
        return StatusCode::NOT_FOUND.into_response();
    }
    // 2. Work out where the cached thumbnail lives (`.thumbs` folder) and when each file was last
    // changed.
    let cache_dir = state.www.join("images/Database/.thumbs");
    let stem = img.rsplit_once('.').map(|(s, _)| s).unwrap_or(&img);
    let cache_file = cache_dir.join(format!("{stem}_w{w}.webp"));
    let src_m = std::fs::metadata(&src).and_then(|m| m.modified()).ok();
    let cache_m = std::fs::metadata(&cache_file).and_then(|m| m.modified()).ok();
    // A small helper that builds the image answer with a one-week browser cache.
    let image_reply = |bytes: Vec<u8>, ct: &'static str| {
        let mut res = (StatusCode::OK, [(header::CONTENT_TYPE, ct)], bytes).into_response();
        res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=604800"));
        res
    };
    // 3. A cached thumbnail at least as new as the original is reused.
    if let (Some(c), Some(s)) = (cache_m, src_m) {
        if c >= s {
            if let Ok(b) = std::fs::read(&cache_file) {
                return image_reply(b, "image/webp");
            }
        }
    }
    // 4. Otherwise read the original and shrink it. `let ... else` returns 404 if the read fails.
    let Ok(data) = std::fs::read(&src) else { return StatusCode::NOT_FOUND.into_response() };
    match images::thumbnail_webp(&data, w) {
        Some(out) => {
            let _ = std::fs::create_dir_all(&cache_dir);
            let _ = std::fs::write(&cache_file, &out);
            image_reply(out, "image/webp")
        }
        // The PHP falls back to the original file when it cannot decode it.
        None => image_reply(data, "image/webp"),
    }
}

/// `?action=add` (needs sign-in): add a new plant from the app's quick-add form.
///
/// Needs `plant_name`. If a living plant already has that name (ignoring case) it answers `exists:
/// true` with the old id unless `force` is set. The new row is flagged `needs_review = 1` so Jun
/// can finish the details. A barcode, if sent, is also saved in `plant_barcodes`. Gives `{ok, id}`.
async fn add_plant(state: &AppState, visitor: &Visitor, f: &FormData, q: &HashMap<String, String>) -> Response {
    // 1. Refuse guests. The `_` in `_user` means the name is not used here.
    let _user = match require_actor(visitor) {
        Ok(u) => u,
        Err(r) => return r,
    };
    // 2. Read the fields. `pick` looks in the form, then the address.
    let name = pick(f, q, "plant_name").trim().to_string();
    let bot = pick(f, q, "botanical_name").trim().to_string();
    let bed = pick(f, q, "garden_beds").trim().to_string();
    let year = pick(f, q, "year").trim().to_string();
    let barcode = pick(f, q, "barcode").trim().to_string();
    let qty_raw = pick(f, q, "qty");
    let qty = if qty_raw.is_empty() { 1 } else { php_int(qty_raw).max(0) };
    let force = !matches!(pick(f, q, "force"), "" | "0");
    if name.is_empty() {
        return ok(json!({ "ok": false, "error": "A common name is required." }));
    }
    // 3. Look for a living plant with the same name.
    let existing = sqlx::query_scalar::<_, i64>("SELECT id FROM `data_source` WHERE LOWER(plant_name) = LOWER(?) AND died = 0 LIMIT 1")
        .bind(&name)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();
    if let Some(id) = existing {
        if !force {
            return ok(json!({ "ok": false, "exists": true, "id": id, "error": "A plant with that name already exists." }));
        }
    }
    // A small helper: an empty string becomes NULL (nothing) in the database.
    let none_if_empty = |s: &str| if s.is_empty() { None } else { Some(s.to_string()) };
    // 4. Insert the plant. Bed and year are each saved twice because the table has two columns for
    // each (older and newer names).
    let r = sqlx::query(
        "INSERT INTO `data_source` (plant_name, botanical_name, flower_bed, garden_beds, year_planted, year_added, qty, barcode, needs_review)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(&name)
    .bind(none_if_empty(&bot))
    .bind(none_if_empty(&bed))
    .bind(none_if_empty(&bed))
    .bind(none_if_empty(&year))
    .bind(none_if_empty(&year))
    .bind(qty)
    .bind(none_if_empty(&barcode))
    .execute(&state.db)
    .await;
    let new_id = match r {
        Ok(r) => r.last_insert_id() as i64,
        Err(e) => return reply(StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": e.to_string() })),
    };
    // 5. Remember the barcode in its own table too, so scanning finds this plant later.
    if !barcode.is_empty() {
        let _ = sqlx::query("INSERT INTO `plant_barcodes` (plant_id, source_table, size_label, barcode) VALUES (?, 'data_source', NULL, ?)")
            .bind(new_id)
            .bind(&barcode)
            .execute(&state.db)
            .await;
    }
    // 6. Clear the cached widget numbers, then answer.
    state.invalidate_widget_cache();
    ok(json!({ "ok": true, "id": new_id }))
}

/// `?action=future_search&q=...`: the app's search over the wish list (`future_plants`), up to 8
/// results. Used when Jun scans or types something that is not in the garden yet. Gives `{results:
/// [...]}`.
async fn future_search(state: &AppState, q: &HashMap<String, String>) -> Response {
    let term = q.get("q").map(|s| s.trim()).unwrap_or("");
    if term.chars().count() < 2 {
        return ok(json!({ "results": [] }));
    }
    let like = format!("%{term}%");
    let rows = sqlx::query(
        "SELECT id, plant_name, botanical_name, garden_beds, flower_bed, year_added, year_planted, qty
         FROM `future_plants` WHERE plant_name LIKE ? OR botanical_name LIKE ? ORDER BY plant_name LIMIT 8",
    )
    .bind(&like)
    .bind(&like)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    // Build each result with fixed key names. The bed uses `garden_beds` if filled, else
    // `flower_bed`; the year uses `year_added`, else `year_planted`.
    let out: Vec<Value> = rows
        .iter()
        .map(|r| {
            let mut m = match row_to_json(r) {
                Value::Object(m) => m,
                _ => Map::new(),
            };
            clean_strings(&mut m);
            let non_empty = |k: &str| m.get(k).filter(|v| !v.is_null() && v.as_str().map_or(true, |s| !s.is_empty() && s != "0")).cloned();
            json!({
                "id": m.get("id").and_then(Value::as_i64).unwrap_or(0),
                "plant_name": m.get("plant_name").cloned().unwrap_or(Value::Null),
                "botanical_name": m.get("botanical_name").cloned().unwrap_or(Value::Null),
                "garden_beds": non_empty("garden_beds").or_else(|| m.get("flower_bed").cloned()).unwrap_or(Value::Null),
                "year": non_empty("year_added").or_else(|| m.get("year_planted").cloned()).unwrap_or(Value::Null),
                "qty": m.get("qty").map(|v| v.as_i64().map(Value::from).unwrap_or(Value::Null)).unwrap_or(Value::Null),
            })
        })
        .collect();
    ok(json!({ "results": out }))
}

/// `?action=future_promote` (needs sign-in): move a wish-list plant into the real garden.
///
/// Copies every column that both tables share into `data_source`, applies any new name, bed, year
/// or quantity the app sent, points its barcodes at the new row, and deletes the wish-list row.
/// This is done in one database transaction so it all happens or none of it does.
async fn future_promote(state: &AppState, visitor: &Visitor, f: &FormData, q: &HashMap<String, String>) -> Response {
    if let Err(r) = require_actor(visitor) {
        return r;
    }
    let fid = php_int(pick(f, q, "id"));
    if fid <= 0 {
        return ok(json!({ "ok": false, "error": "Missing id" }));
    }
    let name = pick(f, q, "plant_name").trim().to_string();
    let bot = pick(f, q, "botanical_name").trim().to_string();
    let beds = pick(f, q, "beds").trim().to_string();
    let years = pick(f, q, "years").trim().to_string();
    let qty_raw = pick(f, q, "qty").to_string();

    // Everything below runs in one `async { ... }` block so `?` can be used. `?` means "on error,
    // stop and return that error". The block's result is checked after `.await`.
    let result: Result<Option<i64>, sqlx::Error> = async {
        // Start a transaction (`begin`): nothing is kept until `commit` at the end.
        let mut tx = state.db.begin().await?;
        // Read the wish-list row; if it is gone, finish with `Ok(None)` ("not found").
        let Some(row) = sqlx::query("SELECT * FROM `future_plants` WHERE id = ?").bind(fid).fetch_optional(&mut *tx).await? else {
            return Ok(None);
        };
        // Ask which columns `data_source` has, so only shared columns are copied.
        let ds_cols: Vec<String> = sqlx::query_scalar("SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'data_source'")
            .fetch_all(&mut *tx)
            .await?;
        // Make the list of (column, value) pairs to copy, leaving out `id` and `date_added`.
        let pairs: Vec<(String, Value)> = match row_to_json(&row) {
            Value::Object(m) => m.into_iter().filter(|(k, _)| k != "id" && k != "date_added" && ds_cols.contains(k)).collect(),
            _ => Vec::new(),
        };
        if pairs.is_empty() {
            return Err(sqlx::Error::Protocol("No compatible columns".into()));
        }
        // Build `INSERT INTO data_source (cols) VALUES (?, ?, ...)` and bind each JSON value by its
        // type.
        let cols = pairs.iter().map(|(k, _)| format!("`{k}`")).collect::<Vec<_>>().join(", ");
        let ph = vec!["?"; pairs.len()].join(", ");
        let ins_sql = format!("INSERT INTO `data_source` ({cols}) VALUES ({ph})");
        let mut ins = sqlx::query(&ins_sql);
        for (_, v) in &pairs {
            ins = match v {
                Value::Null => ins.bind(Option::<String>::None),
                Value::Number(n) => match n.as_i64() {
                    Some(i) => ins.bind(i),
                    None => ins.bind(n.as_f64().unwrap_or(0.0)),
                },
                Value::String(s) => ins.bind(s.clone()),
                other => ins.bind(other.to_string()),
            };
        }
        let new_id = ins.execute(&mut *tx).await?.last_insert_id() as i64;

        // Now apply what the app sent: build `UPDATE ... SET a=?, b=?` from only the fields that
        // were filled in.
        let mut set: Vec<&str> = Vec::new();
        let mut params: Vec<String> = Vec::new();
        if !name.is_empty() {
            set.push("plant_name=?");
            params.push(name.clone());
        }
        if !bot.is_empty() {
            set.push("botanical_name=?");
            params.push(bot.clone());
        }
        if !beds.is_empty() {
            set.push("flower_bed=?");
            params.push(beds.clone());
            set.push("garden_beds=?");
            params.push(beds.clone());
        }
        if !years.is_empty() {
            set.push("year_added=?");
            params.push(years.clone());
            set.push("year_planted=?");
            params.push(years.clone());
        }
        if !qty_raw.is_empty() {
            set.push("qty=?");
            params.push(php_int(&qty_raw).max(0).to_string());
        }
        if !set.is_empty() {
            let up_sql = format!("UPDATE `data_source` SET {} WHERE id=?", set.join(", "));
            let mut up = sqlx::query(&up_sql);
            for p in &params {
                up = up.bind(p);
            }
            up.bind(new_id).execute(&mut *tx).await?;
        }
        // Point barcodes at the new row, delete the wish-list row, and save everything with
        // `commit`.
        let _ = sqlx::query("UPDATE `plant_barcodes` SET plant_id = ?, source_table = 'data_source' WHERE plant_id = ? AND source_table = 'future_plants'")
            .bind(new_id)
            .bind(fid)
            .execute(&mut *tx)
            .await;
        sqlx::query("DELETE FROM `future_plants` WHERE id = ?").bind(fid).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(Some(new_id))
    }
    .await;
    // Clear the cache, then turn the result into JSON: new id, not found, or the error text.
    state.invalidate_widget_cache();
    match result {
        Ok(Some(id)) => ok(json!({ "ok": true, "id": id })),
        Ok(None) => ok(json!({ "ok": false, "error": "Future plant not found" })),
        Err(e) => ok(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// `?action=barcode_lookup&code=...`: find which plant a scanned barcode belongs to. This is what
/// the app calls after the camera reads a code.
///
/// Looks in `plant_barcodes` first, then the `barcode` column of the garden, then of the wish list.
/// Gives `{found, id, plant_name, table, is_future}` or `{found: false}`.
async fn barcode_lookup(state: &AppState, f: &FormData, q: &HashMap<String, String>) -> Response {
    let code = q.get("code").map(String::as_str).or_else(|| f.get("code")).unwrap_or("").trim().to_string();
    if code.is_empty() {
        return ok(json!({ "found": false }));
    }
    if let Ok(Some((pid, table))) = sqlx::query_as::<_, (i64, String)>("SELECT plant_id, source_table FROM `plant_barcodes` WHERE barcode = ? LIMIT 1")
        .bind(&code)
        .fetch_optional(&state.db)
        .await
    {
        let tbl = if table == "future_plants" { "future_plants" } else { "data_source" };
        let name = sqlx::query_scalar::<_, Option<String>>(&format!("SELECT plant_name FROM `{tbl}` WHERE id = ? LIMIT 1"))
            .bind(pid)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .flatten()
            .unwrap_or_default();
        return ok(json!({ "found": true, "id": pid, "plant_name": name, "table": tbl, "is_future": tbl == "future_plants" }));
    }
    if let Ok(Some((id, name))) = sqlx::query_as::<_, (i64, Option<String>)>("SELECT id, plant_name FROM `data_source` WHERE barcode = ? AND died = 0 LIMIT 1")
        .bind(&code)
        .fetch_optional(&state.db)
        .await
    {
        return ok(json!({ "found": true, "id": id, "plant_name": name, "table": "data_source", "is_future": false }));
    }
    if let Ok(Some((id, name))) = sqlx::query_as::<_, (i64, Option<String>)>("SELECT id, plant_name FROM `future_plants` WHERE barcode = ? LIMIT 1")
        .bind(&code)
        .fetch_optional(&state.db)
        .await
    {
        return ok(json!({ "found": true, "id": id, "plant_name": name, "table": "future_plants", "is_future": true }));
    }
    ok(json!({ "found": false }))
}

/// Allow only the two real plant table names. Returns `Some(name)` for a good one and `None`
/// (nothing) otherwise. Table names cannot be `?` placeholders in SQL, so this fixed list keeps
/// typed text out of the query.
fn valid_table(t: &str) -> Option<&'static str> {
    match t {
        "data_source" => Some("data_source"),
        "future_plants" => Some("future_plants"),
        _ => None,
    }
}

/// `?action=barcodes&id=N&table=...`: list the barcodes saved for one plant (id, code, size label,
/// and whether an image exists). Gives `{barcodes: [...]}`.
async fn barcodes(state: &AppState, f: &FormData, q: &HashMap<String, String>) -> Response {
    let id = php_int(pick(f, q, "id"));
    let table = {
        let t = pick(f, q, "table").trim();
        if t.is_empty() { "data_source" } else { t }
    };
    let Some(table) = valid_table(table) else { return ok(json!({ "barcodes": [] })) };
    if id <= 0 {
        return ok(json!({ "barcodes": [] }));
    }
    let rows = sqlx::query("SELECT id, barcode, size_label, (barcode_image IS NOT NULL) AS has_image FROM `plant_barcodes` WHERE plant_id = ? AND source_table = ? ORDER BY id")
        .bind(id)
        .bind(table)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    let out: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.try_get_unchecked::<i64, _>("id").unwrap_or(0),
                "barcode": r.try_get_unchecked::<Option<String>, _>("barcode").unwrap_or(None),
                "size_label": r.try_get_unchecked::<Option<String>, _>("size_label").unwrap_or(None),
                "has_image": r.try_get_unchecked::<i64, _>("has_image").unwrap_or(0) != 0,
            })
        })
        .collect();
    ok(json!({ "barcodes": out }))
}

/// `?action=barcode_image&id=N`: the PNG picture of one barcode, kept in the database. The app
/// draws it for printing. 404 if there is none.
async fn barcode_image(state: &AppState, f: &FormData, q: &HashMap<String, String>) -> Response {
    let bid = php_int(pick(f, q, "id"));
    if bid <= 0 {
        return StatusCode::NOT_FOUND.into_response();
    }
    let img = sqlx::query_scalar::<_, Option<Vec<u8>>>("SELECT barcode_image FROM `plant_barcodes` WHERE id = ? LIMIT 1")
        .bind(bid)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten()
        .filter(|b| !b.is_empty());
    match img {
        Some(b) => {
            let mut res = (StatusCode::OK, [(header::CONTENT_TYPE, "image/png")], b).into_response();
            res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400"));
            res
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `?action=add_barcode` (needs sign-in): attach a barcode to a plant. A barcode may belong to only
/// one plant; a duplicate gives a friendly error instead of a crash.
async fn add_barcode(state: &AppState, visitor: &Visitor, f: &FormData, q: &HashMap<String, String>) -> Response {
    if let Err(r) = require_actor(visitor) {
        return r;
    }
    let id = php_int(pick(f, q, "id"));
    let table = {
        let t = pick(f, q, "table").trim();
        if t.is_empty() { "data_source" } else { t }
    };
    let Some(table) = valid_table(table) else { return ok(json!({ "ok": false, "error": "Invalid table" })) };
    let barcode = pick(f, q, "barcode").trim().to_string();
    let size = pick(f, q, "size_label").trim().to_string();
    if id <= 0 || barcode.is_empty() {
        return ok(json!({ "ok": false, "error": "Missing id or barcode" }));
    }
    match sqlx::query("INSERT INTO `plant_barcodes` (plant_id, source_table, size_label, barcode) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(table)
        .bind(if size.is_empty() { None } else { Some(size) })
        .bind(&barcode)
        .execute(&state.db)
        .await
    {
        Ok(r) => ok(json!({ "ok": true, "id": r.last_insert_id() as i64 })),
        Err(e) => {
            // MariaDB reports a duplicate with error code 23000 (a unique-key violation).
            let dup = e.as_database_error().map_or(false, |d| d.code().as_deref() == Some("23000") || d.is_unique_violation());
            if dup {
                ok(json!({ "ok": false, "error": "That barcode is already assigned to a plant." }))
            } else {
                ok(json!({ "ok": false, "error": e.to_string() }))
            }
        }
    }
}

/// `?action=delete_barcode` (needs sign-in): remove one barcode row by `barcode_id`.
async fn delete_barcode(state: &AppState, visitor: &Visitor, f: &FormData, q: &HashMap<String, String>) -> Response {
    if let Err(r) = require_actor(visitor) {
        return r;
    }
    let bid = php_int(pick(f, q, "barcode_id"));
    if bid <= 0 {
        return ok(json!({ "ok": false, "error": "Missing barcode_id" }));
    }
    match sqlx::query("DELETE FROM `plant_barcodes` WHERE id = ?").bind(bid).execute(&state.db).await {
        Ok(_) => ok(json!({ "ok": true })),
        Err(e) => ok(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// `?action=set_barcode` (needs sign-in): set or clear the single `barcode` column on a plant. An
/// empty value clears it.
async fn set_barcode(state: &AppState, visitor: &Visitor, f: &FormData, q: &HashMap<String, String>) -> Response {
    if let Err(r) = require_actor(visitor) {
        return r;
    }
    let id = php_int(pick(f, q, "id"));
    let table = {
        let t = pick(f, q, "table").trim();
        if t.is_empty() { "data_source" } else { t }
    };
    let Some(table) = valid_table(table) else { return ok(json!({ "ok": false, "error": "Invalid table" })) };
    if id <= 0 {
        return ok(json!({ "ok": false, "error": "Missing id" }));
    }
    let barcode = pick(f, q, "barcode").trim().to_string();
    match sqlx::query(&format!("UPDATE `{table}` SET barcode = ? WHERE id = ?"))
        .bind(if barcode.is_empty() { None } else { Some(barcode) })
        .bind(id)
        .execute(&state.db)
        .await
    {
        Ok(_) => ok(json!({ "ok": true })),
        Err(e) => ok(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// `?action=export`: every non-binary column of the three tables.
///
/// Build one JSON object with a list of rows per table, skipping image columns.
async fn export(state: &AppState) -> Response {
    let mut out = Map::new();
    out.insert("generated_at".into(), json!(date_c()));
    for t in ["data_source", "future_plants", "plant_barcodes"] {
        let cols = text_columns(state, t).await;
        let rows: Vec<Value> = if cols.is_empty() {
            Vec::new()
        } else {
            let list = cols.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ");
            sqlx::query(&format!("SELECT {list} FROM `{t}`"))
                .fetch_all(&state.db)
                .await
                .unwrap_or_default()
                .iter()
                .map(row_to_json)
                .collect()
        };
        out.insert(t.into(), Value::Array(rows));
    }
    ok(Value::Object(out))
}

/// A number that changes whenever any plant, wish-list entry, like,
/// favorite or barcode changes.
///
/// The app asks this cheaply to learn whether anything changed; if the number is the same as last
/// time it keeps its saved copy. How: `CRC32` turns each row's key fields joined with `|` into a
/// number, `SUM` adds them, and the five totals are added together. Gives `{signature: "..."}`.
async fn catalog_signature(state: &AppState) -> Response {
    let mut sig = 0f64;
    // One query per table: plants, wish list, favorites, likes, barcodes.
    for sql in [
        "SELECT COALESCE(SUM(CRC32(CONCAT_WS('|',
            id, plant_name, botanical_name, family, genus, flower_bed, garden_beds,
            sun_exposure, water_needs, fruit_edibility, toxicity, plant_type,
            native_status, zones, average_cost, qty, barcode, picture,
            year_added, year_planted, COALESCE(died,0)
         ))), 0) FROM `data_source`",
        "SELECT COALESCE(SUM(CRC32(CONCAT_WS('|',
            id, plant_name, botanical_name, garden_beds, flower_bed,
            year_added, year_planted, qty, barcode
         ))), 0) FROM `future_plants`",
        "SELECT COALESCE(SUM(CRC32(CONCAT_WS('|', plant_id, username))), 0) FROM `user_favorites`",
        "SELECT COALESCE(SUM(CRC32(CONCAT_WS('|', plant_id, username))), 0) FROM `user_likes`",
        "SELECT COALESCE(SUM(CRC32(CONCAT_WS('|', id, plant_id, source_table, barcode))), 0) FROM `plant_barcodes`",
    ] {
        // Read the sum as text then as a decimal number; a failed query adds nothing.
        if let Ok(row) = sqlx::query(sql).fetch_one(&state.db).await {
            let v: f64 = row
                .try_get_unchecked::<Option<String>, _>(0)
                .ok()
                .flatten()
                .map(|s| crate::php::php_float(&s))
                .unwrap_or(0.0);
            sig += v;
        }
    }
    // PHP prints a float that is a whole number without a decimal point.
    let text = if sig.fract() == 0.0 && sig.abs() < 1e15 { format!("{}", sig as i64) } else { format!("{sig}") };
    ok(json!({ "signature": text }))
}

/// `?action=bed_counts`: how many living plants with a picture are in each bed. Gives `{total,
/// beds: [{name, count}]}`. A plant listed in several comma-separated beds counts once in each.
async fn bed_counts(state: &AppState) -> Response {
    let rows = sqlx::query_scalar::<_, Option<String>>(
        "SELECT flower_bed FROM `data_source` WHERE died = 0 AND flower_bed NOT LIKE '%Dead%' AND picture IS NOT NULL AND TRIM(picture) <> ''",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    // Count by bed name in order of first appearance, then sort without regard to upper or lower
    // case.
    let mut counts: Vec<(String, i64)> = Vec::new();
    for fb in rows.into_iter().flatten() {
        for b in fb.split(',') {
            let b = b.trim();
            if b.is_empty() {
                continue;
            }
            if let Some(e) = counts.iter_mut().find(|(n, _)| n == b) {
                e.1 += 1;
            } else {
                counts.push((b.to_string(), 1));
            }
        }
    }
    // ksort with SORT_FLAG_CASE | SORT_STRING: case-insensitive.
    counts.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    let total: i64 = counts.iter().map(|(_, c)| c).sum();
    let beds: Vec<Value> = counts.iter().map(|(n, c)| json!({ "name": n, "count": c })).collect();
    ok(json!({ "total": total, "beds": beds }))
}

/// `?action=update_plant` (needs sign-in): change bed, year, quantity or barcode on a plant.
///
/// Only the fields sent are changed. Changing the quantity also writes a "correction" row to the
/// quantity log, with the signed-in person's name and today's date.
async fn update_plant(state: &AppState, visitor: &Visitor, f: &FormData, q: &HashMap<String, String>) -> Response {
    let user = match require_actor(visitor) {
        Ok(u) => u,
        Err(r) => return r,
    };
    let id = php_int(pick(f, q, "id"));
    if id <= 0 {
        return ok(json!({ "ok": false, "error": "Missing id" }));
    }
    let beds = pick(f, q, "beds").trim().to_string();
    let years = pick(f, q, "years").trim().to_string();
    let qty_raw = pick(f, q, "qty").to_string();
    let qty: Option<i64> = if qty_raw.is_empty() { None } else { Some(php_int(&qty_raw).max(0)) };
    let barcode_set = is_set(f, q, "barcode");
    let barcode = pick(f, q, "barcode").trim().to_string();

    // Build the `SET` list and values one field at a time. `Option<String>` allows NULL (nothing)
    // for clearing the barcode.
    let mut set: Vec<&str> = Vec::new();
    let mut params: Vec<Option<String>> = Vec::new();
    if !beds.is_empty() {
        set.push("flower_bed = ?");
        params.push(Some(beds.clone()));
        set.push("garden_beds = ?");
        params.push(Some(beds.clone()));
    }
    if !years.is_empty() {
        set.push("year_added = ?");
        params.push(Some(years.clone()));
        set.push("year_planted = ?");
        params.push(Some(years.clone()));
    }
    if let Some(qv) = qty {
        set.push("qty = ?");
        params.push(Some(qv.to_string()));
    }
    if barcode_set {
        set.push("barcode = ?");
        params.push(if barcode.is_empty() { None } else { Some(barcode.clone()) });
    }
    if set.is_empty() {
        return ok(json!({ "ok": false, "error": "Nothing to update" }));
    }
    let up_sql = format!("UPDATE `data_source` SET {} WHERE id = ?", set.join(", "));
    let mut up = sqlx::query(&up_sql);
    for p in &params {
        up = up.bind(p.clone());
    }
    if let Err(e) = up.bind(id).execute(&state.db).await {
        return reply(StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": e.to_string() }));
    }
    // Log the quantity change. `iso_week` is the ISO week number, as the PHP's `W`.
    if let Some(qv) = qty {
        use chrono::Datelike;
        let d = chrono::Local::now().date_naive();
        let _ = sqlx::query(
            "INSERT INTO plant_quantity_log (plant_id, event_type, quantity, event_date, event_week, event_month, event_year, notes, recorded_by)
             VALUES (?, 'correction', ?, ?, ?, ?, ?, 'Set via app', ?)",
        )
        .bind(id)
        .bind(qv)
        .bind(d.format("%Y-%m-%d").to_string())
        .bind(d.iso_week().week() as i32)
        .bind(d.month() as i32)
        .bind(d.year())
        .bind(&user)
        .execute(&state.db)
        .await;
    }
    state.invalidate_widget_cache();
    ok(json!({ "ok": true }))
}

/// `?action=like` / `?action=favorite`: toggle a mark for the named user on a plant. If it exists
/// it is removed; if not it is added. Gives `{ok, active, count}`.
///
/// The user name comes from the request and needs no sign-in, as in the PHP. This only changes a
/// like or favorite, never a plant.
async fn like_or_favorite(state: &AppState, action: &str, f: &FormData, q: &HashMap<String, String>) -> Response {
    let table = if action == "like" { "user_likes" } else { "user_favorites" };
    let user = pick(f, q, "username").trim().to_string();
    let pid = php_int(pick(f, q, "plant_id"));
    if user.is_empty() || pid <= 0 {
        return ok(json!({ "ok": false, "error": "Missing user or plant." }));
    }
    let exists = sqlx::query(&format!("SELECT id FROM `{table}` WHERE username = ? AND plant_id = ?"))
        .bind(&user)
        .bind(pid)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .is_some();
    // Toggle the mark, then count how many people now have it.
    let active = if exists {
        let _ = sqlx::query(&format!("DELETE FROM `{table}` WHERE username = ? AND plant_id = ?")).bind(&user).bind(pid).execute(&state.db).await;
        false
    } else {
        let _ = sqlx::query(&format!("INSERT INTO `{table}` (username, plant_id) VALUES (?, ?)")).bind(&user).bind(pid).execute(&state.db).await;
        true
    };
    let count = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM `{table}` WHERE plant_id = ?"))
        .bind(pid)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    ok(json!({ "ok": true, "active": active, "count": count }))
}

/// `?action=autocomplete&q=...`: name suggestions for the app's search box, up to 8 garden names
/// (`suggestions`) and 8 wish-list names (`future`). Names that start with the typed text come
/// first. Needs 2 or more letters.
async fn autocomplete(state: &AppState, q: &HashMap<String, String>) -> Response {
    let term = q.get("q").map(|s| s.trim()).unwrap_or("");
    if term.chars().count() < 2 {
        return ok(json!({ "suggestions": [] }));
    }
    let like = format!("%{term}%");
    let starts = format!("{term}%");
    // A small helper: drop NULLs (`flatten`), strip trademark marks, and remove blank or "0" names.
    let clean = |names: Vec<Option<String>>| -> Vec<String> {
        names
            .into_iter()
            .flatten()
            .map(|n| n.replace(['™', '®'], "").trim().to_string())
            .filter(|n| !n.is_empty() && n != "0")
            .collect()
    };
    let names = sqlx::query_scalar::<_, Option<String>>(
        "SELECT DISTINCT plant_name FROM `data_source` WHERE plant_name LIKE ? AND died = 0 AND flower_bed NOT LIKE '%Dead%'
         ORDER BY CASE WHEN plant_name LIKE ? THEN 0 ELSE 1 END, plant_name LIMIT 8",
    )
    .bind(&like)
    .bind(&starts)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let future = sqlx::query_scalar::<_, Option<String>>(
        "SELECT DISTINCT plant_name FROM `future_plants` WHERE plant_name LIKE ?
         ORDER BY CASE WHEN plant_name LIKE ? THEN 0 ELSE 1 END, plant_name LIMIT 8",
    )
    .bind(&like)
    .bind(&starts)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    ok(json!({ "suggestions": clean(names), "future": clean(future) }))
}

/// `?action=detail&id=N`: everything about one plant for the app's detail screen.
///
/// Gives `{plant: {...}}` with all columns except the QR image, an `images` list (see
/// `plant_image_urls`), like and favorite counts, who favorited it, and whether the optional `user`
/// has liked or favorited it. 404 if the plant does not exist.
async fn detail(state: &AppState, q: &HashMap<String, String>) -> Response {
    let id = q.get("id").map(|i| php_int(i)).unwrap_or(0);
    if id <= 0 {
        return ok(json!({ "error": "Missing id" }));
    }
    let row = sqlx::query("SELECT * FROM `data_source` WHERE id = ?").bind(id).fetch_optional(&state.db).await.ok().flatten();
    let Some(row) = row else {
        return reply(StatusCode::NOT_FOUND, json!({ "error": "Plant not found" }));
    };
    let mut p = match row_to_json(&row) {
        Value::Object(m) => m,
        _ => Map::new(),
    };
    p.shift_remove("qr_code");
    clean_strings(&mut p);
    let images = plant_image_urls(&p);

    // A helper that runs a "COUNT" query for this plant id and returns the number (0 on error).
    let count = |sql: &'static str| async move { sqlx::query_scalar::<_, i64>(sql).bind(id).fetch_one(&state.db).await.unwrap_or(0) };
    let like_count = count("SELECT COUNT(*) FROM user_likes WHERE plant_id = ?").await;
    let fav_count = count("SELECT COUNT(*) FROM user_favorites WHERE plant_id = ?").await;

    // `user` is the app's current person; it is only used to fill `is_liked` and `is_favorited`.
    let viewer = q.get("user").map(|s| s.trim()).unwrap_or("");
    let (mut is_liked, mut is_fav) = (false, false);
    if !viewer.is_empty() {
        is_liked = sqlx::query("SELECT 1 FROM user_likes WHERE plant_id = ? AND username = ? LIMIT 1")
            .bind(id)
            .bind(viewer)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .is_some();
        is_fav = sqlx::query("SELECT 1 FROM user_favorites WHERE plant_id = ? AND username = ? LIMIT 1")
            .bind(id)
            .bind(viewer)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .is_some();
    }
    // The app wants one `images` list, so the two raw picture columns are removed.
    p.shift_remove("picture");
    p.shift_remove("additional_pictures");
    let fav_by = sqlx::query_scalar::<_, String>("SELECT username FROM user_favorites WHERE plant_id = ?")
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    p.insert("images".into(), json!(images));
    p.insert("like_count".into(), json!(like_count));
    p.insert("fav_count".into(), json!(fav_count));
    p.insert("is_liked".into(), json!(is_liked));
    p.insert("is_favorited".into(), json!(is_fav));
    p.insert("favorited_by".into(), json!(fav_by));
    ok(json!({ "plant": p }))
}

/// The plant grid, paged, with the wish list folded in on a search.
///
/// Takes `page`, `search`, `bed`, `user` and `sort` from the address. Gives `{page, hasMore, count,
/// results}`. Only living plants that have a picture are listed. A one-letter search means "names
/// starting with this letter". Wish-list rows are added only on page 1 of a real search and have
/// NEGATIVE ids so the app can tell them from garden plants (`is_future: true`).
async fn list(state: &AppState, q: &HashMap<String, String>) -> Response {
    // 1. Work out the page window: one extra row is fetched to learn whether another page exists.
    let page = q.get("page").map(|p| php_int(p)).unwrap_or(1).max(1);
    let offset = (page - 1) * PAGE_SIZE;
    let fetch = PAGE_SIZE + 1;
    let search = q.get("search").map(|s| s.trim().to_string()).unwrap_or_default();
    let bed = q.get("bed").map(|s| s.trim().to_string()).unwrap_or_default();
    let viewer = q.get("user").map(|s| s.trim().to_string()).unwrap_or_default();

    // 2. Build the WHERE text and the list of values for its `?` placeholders. Only fixed pieces of
    // SQL are added; typed text is always a placeholder value.
    let mut where_ = "died = 0 AND flower_bed NOT LIKE '%Dead%' AND picture IS NOT NULL AND TRIM(picture) <> ''".to_string();
    let mut params: Vec<String> = Vec::new();
    let single_letter = search.len() == 1 && search.chars().all(|c| c.is_ascii_alphabetic());
    if !search.is_empty() {
        if single_letter {
            where_.push_str(" AND plant_name LIKE ?");
            params.push(format!("{search}%"));
        } else {
            let like = format!("%{search}%");
            where_.push_str(" AND (plant_name LIKE ? OR botanical_name LIKE ? OR family LIKE ? OR flower_bed LIKE ? OR company LIKE ? OR genus LIKE ?)");
            for _ in 0..6 {
                params.push(like.clone());
            }
        }
    }
    if !bed.is_empty() {
        where_.push_str(" AND flower_bed LIKE ?");
        params.push(format!("%{bed}%"));
    }
    // 3. Sort by bed when asked, otherwise by name. These are fixed words chosen by this code.
    let order_by = if q.get("sort").map(String::as_str) == Some("bed") { "flower_bed, plant_name" } else { "plant_name ASC" };
    let sql = format!(
        "SELECT id, plant_name, botanical_name, family, genus, flower_bed,
                picture, sun_exposure, water_needs, fruit_edibility, toxicity,
                bloom_color, plant_type, native_status, zones, average_cost, qty
         FROM `data_source` WHERE {where_} ORDER BY {order_by} LIMIT {fetch} OFFSET {offset}"
    );
    let mut qy = sqlx::query(&sql);
    for p in &params {
        qy = qy.bind(p);
    }
    // 4. Run it. On error send a 500 with the message.
    let mut rows = match qy.fetch_all(&state.db).await {
        Ok(r) => r,
        Err(e) => return reply(StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": e.to_string() })),
    };
    // If the extra row came back there is a next page; remove that row.
    let has_more = rows.len() as i64 > PAGE_SIZE;
    if has_more {
        rows.pop();
    }

    // 5. Load all favorites once so each plant can show who favorited it.
    let favs = sqlx::query_as::<_, (i64, String)>("SELECT plant_id, username FROM user_favorites")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    // 6. Build the JSON for each garden plant with fixed key names the app expects.
    let get = |m: &Map<String, Value>, k: &str| m.get(k).cloned().unwrap_or(Value::Null);
    let mut results: Vec<Value> = Vec::new();
    for r in &rows {
        let mut m = match row_to_json(r) {
            Value::Object(m) => m,
            _ => continue,
        };
        clean_strings(&mut m);
        let imgs = plant_image_urls(&m);
        let id = m.get("id").and_then(Value::as_i64).unwrap_or(0);
        let fav_by: Vec<&String> = favs.iter().filter(|(p, _)| *p == id).map(|(_, u)| u).collect();
        results.push(json!({
            "id": id,
            "plant_name": get(&m, "plant_name"),
            "botanical_name": get(&m, "botanical_name"),
            "family": get(&m, "family"),
            "genus": get(&m, "genus"),
            "flower_bed": get(&m, "flower_bed"),
            "sun_exposure": get(&m, "sun_exposure"),
            "water_needs": get(&m, "water_needs"),
            "fruit_edibility": get(&m, "fruit_edibility"),
            "toxicity": get(&m, "toxicity"),
            "plant_type": get(&m, "plant_type"),
            "native_status": get(&m, "native_status"),
            "zones": get(&m, "zones"),
            "average_cost": get(&m, "average_cost"),
            "qty": m.get("qty").and_then(Value::as_i64),
            "image": imgs.first(),
            "fav_count": fav_by.len(),
            "is_favorited": !viewer.is_empty() && fav_by.iter().any(|u| **u == viewer),
            "favorited_by": fav_by,
            "is_future": false,
        }));
    }

    // 7. Wish-list items only appear when the person is actively searching.
    // Wish-list items only appear when the person is actively searching.
    if page == 1 && !search.is_empty() {
        let mut fwhere = "1=1".to_string();
        let mut fparams: Vec<String> = Vec::new();
        if single_letter {
            fwhere.push_str(" AND plant_name LIKE ?");
            fparams.push(format!("{search}%"));
        } else {
            let flike = format!("%{search}%");
            fwhere.push_str(" AND (plant_name LIKE ? OR botanical_name LIKE ? OR family LIKE ? OR genus LIKE ?)");
            for _ in 0..4 {
                fparams.push(flike.clone());
            }
        }
        if !bed.is_empty() {
            fwhere.push_str(" AND (flower_bed LIKE ? OR garden_beds LIKE ?)");
            fparams.push(format!("%{bed}%"));
            fparams.push(format!("%{bed}%"));
        }
        let fq_sql = format!(
            "SELECT id, plant_name, botanical_name, family, genus, flower_bed, garden_beds,
                    sun_exposure, water_needs, fruit_edibility, toxicity, plant_type,
                    native_status, zones, average_cost, qty
             FROM `future_plants` WHERE {fwhere} ORDER BY plant_name LIMIT 50"
        );
        let mut fq = sqlx::query(&fq_sql);
        for p in &fparams {
            fq = fq.bind(p);
        }
        for r in fq.fetch_all(&state.db).await.unwrap_or_default() {
            let mut m = match row_to_json(&r) {
                Value::Object(m) => m,
                _ => continue,
            };
            clean_strings(&mut m);
            // `non_empty` returns a field only when it is not NULL, blank or "0".
            let non_empty = |k: &str| m.get(k).filter(|v| !v.is_null() && v.as_str().map_or(true, |s| !s.is_empty() && s != "0")).cloned();
            results.push(json!({
                // The id is made negative on purpose (see the doc comment on this function).
                "id": -m.get("id").and_then(Value::as_i64).unwrap_or(0),
                "plant_name": get(&m, "plant_name"),
                "botanical_name": get(&m, "botanical_name"),
                "family": get(&m, "family"),
                "genus": get(&m, "genus"),
                "flower_bed": non_empty("flower_bed").or_else(|| m.get("garden_beds").cloned()).unwrap_or(Value::Null),
                "sun_exposure": get(&m, "sun_exposure"),
                "water_needs": get(&m, "water_needs"),
                "fruit_edibility": get(&m, "fruit_edibility"),
                "toxicity": get(&m, "toxicity"),
                "plant_type": get(&m, "plant_type"),
                "native_status": get(&m, "native_status"),
                "zones": get(&m, "zones"),
                "average_cost": get(&m, "average_cost"),
                "qty": m.get("qty").and_then(Value::as_i64),
                "image": Value::Null,
                "fav_count": 0,
                "is_favorited": false,
                "favorited_by": [],
                "is_future": true,
            }));
        }
    }

    ok(json!({ "page": page, "hasMore": has_more, "count": results.len(), "results": results }))
}

// Tests. `cargo test` runs each `#[test]` function; `assert_eq!` fails the test if the two sides
// differ.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_urls_dedupe_and_skip_blanks() {
        let mut m = Map::new();
        m.insert("picture".into(), json!("a.webp, b.webp"));
        m.insert("additional_pictures".into(), json!("b.webp,,c.webp"));
        assert_eq!(plant_image_urls(&m), vec!["/images/Database/a.webp", "/images/Database/b.webp", "/images/Database/c.webp"]);
    }

    #[test]
    fn trademarks_are_stripped() {
        let mut m = Map::new();
        m.insert("plant_name".into(), json!("Proven Winners® Lil' Kim™ "));
        clean_strings(&mut m);
        assert_eq!(m["plant_name"], json!("Proven Winners Lil' Kim"));
    }
}
