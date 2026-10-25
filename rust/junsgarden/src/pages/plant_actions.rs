//! `plant_actions.php` — the small endpoint the catalog's plant window and
//! the Tags view call.
//!
//! Public: the QR-code and barcode PNG images stored in the database (the QR tags
//! printed in the garden point at them). Signed in: saved preferences, the
//! done list, photo position, approve, mark done, default photo, save or
//! promote a wish-list plant, and the barcode lookup. Same order of checks
//! and same answers as the PHP.
//! URL: `/plant_actions.php` only (GET or POST). Which job to do is chosen by the
//! `action` value, sent as a form field, in the address, or inside a JSON body.
//! The printed QR tags and the Android app use this exact address.
//!
//! Actions without sign-in (the printed QR tags need them):
//!   bed_qr_code, qr_code, barcode        send a stored PNG picture.
//! Actions for any signed-in person:
//!   save_pref, load_prefs                remember small settings per user.
//! Actions for a signed-in admin only:
//!   list_done, toggle_done               the per-user "done" checklist.
//!   save_photo_position, toggle_image_approved, set_default   photo settings.
//!   save_future, promote_future          edit or promote a wish-list plant.
//!   lookup_barcode                       look up a barcode number online.
//!
//! HOW TO READ THIS FILE. The top has small reply helpers and SQL helpers.
//! `lookup_barcode` is the one big helper. The handler `plant_actions` is a long
//! list of `if action == "..."` blocks, in the same order as the PHP, each one
//! ending in `return`. Tests are at the end.

// `use` lines bring names into scope. `HashMap` is a key-value table. `Arc` is a
// shared pointer: all requests share one AppState (settings plus database pool).
use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use serde_json::{json, Value};
use sqlx::Row;

use crate::formdata::{self, FormData};
use crate::php::php_int;
use crate::rowjson::row_to_json;
use crate::session::Visitor;
use crate::AppState;

/// Build a JSON reply with a chosen HTTP status.
/// Takes a status code and a JSON value. Gives back a `Response`. If the value
/// cannot be turned into text it sends `{}` rather than failing.
fn json_status(status: StatusCode, body: Value) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&body).unwrap_or_else(|_| "{}".into()),
    )
        .into_response()
}

/// Shorthand for a JSON reply with status 200. The PHP returned errors such as
/// "Missing plant_id" with status 200 too, so many replies use this on purpose.
fn ok(body: Value) -> Response {
    json_status(StatusCode::OK, body)
}

/// Send picture bytes as a PNG. `Vec<u8>` is a list of raw bytes. The reply may be
/// cached by browsers for a day (86400 seconds) because QR codes rarely change.
fn png(bytes: Vec<u8>) -> Response {
    let mut res = (StatusCode::OK, [(header::CONTENT_TYPE, "image/png")], bytes).into_response();
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );
    res
}

/// The two kinds of value that `blob` can match in its SQL `?`: a number id or a
/// text name. An `enum` is a type that is exactly one of several listed forms.
enum BindValue {
    Id(i64),
    Text(String),
}

/// One binary column of one row, or None.
/// Takes the app state, an SQL text with one `?`, and the value for that `?`.
/// Gives back `Option<Vec<u8>>`: `Some(bytes)` or `None` (nothing found or empty).
async fn blob(state: &AppState, sql: &str, bind: BindValue) -> Option<Vec<u8>> {
    // 1. Start the query. 2. `match` on which kind of value we got and bind it to
    // the `?` (binding keeps typed text from changing the meaning of the SQL).
    let q = sqlx::query(sqlx::AssertSqlSafe(sql));
    let q = match bind {
        BindValue::Id(i) => q.bind(i),
        BindValue::Text(t) => q.bind(t),
    };
    // 3. Fetch at most one row. The double `?` means: if the database call failed
    // return `None`, and if there was no row return `None` (`?` on an Option
    // returns early with `None`).
    let row = q.fetch_optional(&state.db).await.ok()??;
    // 4. Read column 0 as raw bytes (the picture). A NULL or empty value gives `None`.
    row.try_get_unchecked::<Option<Vec<u8>>, _>(0)
        .ok()
        .flatten()
        .filter(|b| !b.is_empty())
}

/// PHP's `empty()` for a request value.
/// Gives true for "" and "0", which PHP's `empty()` also treats as empty.
fn empty(s: &str) -> bool {
    s.is_empty() || s == "0"
}

/// Split a comma list and keep each name once, in order.
/// Takes the two picture columns of a plant (each may be NULL or a comma list).
/// Gives back one list with every name once, in order, main picture first.
fn merged_images(picture: &Option<String>, additional: &Option<String>) -> Vec<String> {
    // `Vec` is a growable list. `&Option<String>` is a borrowed "maybe text".
    let mut out: Vec<String> = Vec::new();
    for s in [picture, additional].into_iter().flatten() {
        for f in s.split(',') {
            let f = f.trim();
            if !f.is_empty() && !out.iter().any(|x| x == f) {
                out.push(f.to_string());
            }
        }
    }
    out
}

/// A JSON body as `(column, value)` pairs, in the order given.
/// Takes a JSON value. Gives back its top-level fields as a list of pairs, or an
/// empty list if the value is not a JSON object.
fn json_pairs(v: &Value) -> Vec<(String, Value)> {
    match v {
        Value::Object(m) => m.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        _ => Vec::new(),
    }
}

/// Column names must be plain identifiers before they go into SQL.
/// Takes a column name. Gives true if it is safe to paste into SQL: not empty, at
/// most 64 characters, only letters, digits and `_`. Column names cannot be
/// bound with `?`, so this check is what stops SQL injection through them.
fn safe_column(c: &str) -> bool {
    !c.is_empty() && c.len() <= 64 && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Attach one JSON value to an SQL query as the next `?`, choosing the matching
/// database type. Takes the query so far and the value; gives back the query.
/// Null and empty text become SQL NULL (the PHP did the same). `'q` is a
/// lifetime name: it says the borrowed value must live as long as the query.
fn bind_json<'q>(
    q: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    v: &'q Value,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    // Choose the database type from the JSON type. Booleans become 0 or 1.
    // Whole numbers stay integers; other numbers become floating point.
    match v {
        Value::Null => q.bind(Option::<String>::None),
        Value::Bool(b) => q.bind(*b as i64),
        Value::Number(n) => match n.as_i64() {
            Some(i) => q.bind(i),
            None => q.bind(n.as_f64().unwrap_or(0.0)),
        },
        Value::String(s) => {
            // The PHP turns '' into NULL for these updates.
            if s.is_empty() {
                q.bind(Option::<String>::None)
            } else {
                q.bind(s.as_str())
            }
        }
        // Anything else (a list or object) is stored as its JSON text.
        other => q.bind(other.to_string()),
    }
}

/// The barcode lookup's cache file. The PHP kept it in the shared temp
/// folder under a fixed name, which any other program on the machine could
/// pre-create or overwrite (Semgrep rule temp-dir). It lives in the site's
/// data folder instead, which only this program writes and which is never
/// served to browsers. Same contents, same behaviour.
/// Takes the app state. Gives back the path of the cache file in the data folder.
fn barcode_cache_path(state: &AppState) -> std::path::PathBuf {
    state.data.join("barcode_cache.json")
}

/// Make the cache key for a plant: name and company, lower-case, letters and
/// digits only, joined with `|`. "Lil' Kim" by "Proven Winners" gives
/// "lilkim|provenwinners". The closure `clean` does the cleaning for both.
fn cache_key(plant_name: &str, company: &str) -> String {
    let clean = |s: &str| {
        s.to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
    };
    format!("{}|{}", clean(plant_name), clean(company))
}

/// Find a barcode number for a plant using the free UPCitemdb web service.
/// Takes the app state and the JSON sent by the page (plant_name, company, table,
/// id). Gives back a JSON reply. The steps: check the cache file first (to save the
/// service's small daily limit), otherwise ask the web, pick the best match, store
/// the result in the cache, and write the barcode into the plant's row.
async fn lookup_barcode(state: &AppState, data: &Value) -> Response {
    // 1. Read the request. Only the two known tables are allowed (never a table name
    // taken from the user). `pid` is the plant's row id, 0 if none was given.
    let plant_name = data
        .get("plant_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let company = data
        .get("company")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let table = if data.get("table").and_then(Value::as_str) == Some("future_plants") {
        "future_plants"
    } else {
        "data_source"
    };
    let pid = data
        .get("id")
        .map(|v| {
            v.as_i64()
                .unwrap_or_else(|| php_int(v.as_str().unwrap_or("")))
        })
        .unwrap_or(0);
    // A plant name is required.
    if plant_name.is_empty() {
        return ok(json!({ "error": "plant_name required" }));
    }

    // 2. Load the cache file, or start a new empty cache if it is missing (or damaged: it
    // is only a cache, so starting over is fine).
    let path = barcode_cache_path(state);
    let p = path.clone();
    let loaded = crate::guard::off_thread(move || crate::store::read_json(&p).ok()).await;
    let mut cache: Value = loaded
        .map(Value::Object)
        .unwrap_or_else(|| json!({ "lookups": {} }));
    // 3. Make the cache key, a small helper to save the cache (whole file at once, off the
    // async threads), and a timestamp.
    let key = cache_key(&plant_name, &company);
    let save_cache = |cache: &Value| {
        let (p, body) = (
            path.clone(),
            serde_json::to_vec_pretty(cache).unwrap_or_default(),
        );
        async move {
            crate::guard::off_thread(move || {
                if let Err(e) = crate::store::write_atomic(&p, &body) {
                    tracing::error!("barcode cache not saved: {e}");
                }
            })
            .await
        }
    };
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 4. Cache hit: reuse the stored number, write it to the plant row, and reply
    // "cached". `if let Some(x) = ...` runs only when the value exists.
    if let Some(hit) = cache.get("lookups").and_then(|l| l.get(&key)) {
        if let Some(upc) = hit
            .get("upc")
            .and_then(Value::as_str)
            .filter(|u| !u.is_empty())
        {
            if pid != 0 {
                let _ = sqlx::query(sql_text!("UPDATE `{table}` SET barcode = ? WHERE id = ?"))
                    .bind(upc)
                    .bind(pid)
                    .execute(&state.db)
                    .await;
            }
            return ok(
                json!({ "status": "cached", "upc": upc, "title": hit.get("title").and_then(Value::as_str).unwrap_or("") }),
            );
        }
    }

    // 5. Cache miss: build the web address. Name and brand are URL-encoded.
    let url = format!(
        "https://api.upcitemdb.com/prod/trial/search?s={}{}&match_mode=0&type=product",
        crate::php::urlencode(&plant_name),
        if company.is_empty() {
            String::new()
        } else {
            format!("&brand={}", crate::php::urlencode(&company))
        }
    );
    // 6. Ask the service, waiting at most 10 seconds. `r.json().await.ok()` turns the
    // answer into JSON, or `None` if it is not valid JSON.
    let resp = state
        .http
        .get(&url)
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    let body: Option<Value> = match resp {
        Ok(r) => r.json().await.ok(),
        Err(_) => None,
    };
    // 7. If there was no usable answer, remember "not found" so we do not ask again
    // right away, and reply "not_found".
    let Some(api) = body else {
        cache["lookups"][&key] = json!({ "not_found": true, "looked_up": now });
        save_cache(&cache).await;
        return ok(json!({ "status": "not_found" }));
    };
    // 8. Pick the best product: first one whose title contains the plant name,
    // otherwise the first one in the list. A barcode is its EAN, else its UPC.
    let items = api
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let first_code = |item: &Value| -> String {
        let ean = item
            .get("ean")
            .and_then(|e| e.get(0))
            .and_then(Value::as_str)
            .unwrap_or("");
        if !ean.is_empty() {
            return ean.to_string();
        }
        item.get("upc")
            .and_then(|e| e.get(0))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let mut upc = String::new();
    let mut title = String::new();
    let name_lower = plant_name.to_lowercase();
    for item in &items {
        let t = item.get("title").and_then(Value::as_str).unwrap_or("");
        if t.to_lowercase().contains(&name_lower) {
            upc = first_code(item);
            title = t.to_string();
            break;
        }
    }
    if upc.is_empty() {
        if let Some(item) = items.first() {
            upc = first_code(item);
            title = item
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
        }
    }
    // 9. Store a found number in the cache and in the plant row, then reply.
    if !upc.is_empty() {
        cache["lookups"][&key] = json!({ "upc": upc, "title": title, "looked_up": now });
        save_cache(&cache).await;
        if pid != 0 {
            let _ = sqlx::query(sql_text!("UPDATE `{table}` SET barcode = ? WHERE id = ?"))
                .bind(&upc)
                .bind(pid)
                .execute(&state.db)
                .await;
        }
        ok(json!({ "status": "found", "upc": upc, "title": title }))
    // Nothing found: remember that and reply "not_found".
    } else {
        cache["lookups"][&key] = json!({ "not_found": true, "looked_up": now });
        save_cache(&cache).await;
        ok(json!({ "status": "not_found" }))
    }
}

/// Handler for `/plant_actions.php`. Takes the state, the visitor, the query
/// string, the headers and the raw body. Gives back a picture or a JSON reply.
/// Only the first matching `action` block runs, because each ends with `return`.
pub async fn plant_actions(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // 1. Read the posted data. Then `pick` looks for a value in the form first, then
    // in the address, then in the JSON body, which is the order the PHP used.
    let f: FormData = formdata::parse(&headers, body).await;
    let json_body = f.json.clone();
    // The PHP reads POST first, then GET, then the JSON body.
    let pick = |k: &str| -> String {
        f.get(k)
            .map(str::to_string)
            .or_else(|| q.get(k).cloned())
            .or_else(|| {
                json_body.as_ref().and_then(|j| j.get(k)).map(|v| {
                    v.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| v.to_string())
                })
            })
            .unwrap_or_default()
    };
    // 2. The three values almost every action needs.
    let action = pick("action");
    let plant_id = php_int(&pick("plant_id"));
    let user = visitor.username.clone();

    // 3. Public actions: QR code of a flower bed, QR code of a plant, barcode image.
    // These send a PNG, or a 404 JSON error when nothing is stored.
    if action == "bed_qr_code" {
        let bed = pick("bed").trim().to_string();
        if !bed.is_empty() {
            if let Some(b) = blob(
                &state,
                "SELECT qr_code FROM `flower_bed_qr` WHERE bed_name = ?",
                BindValue::Text(bed),
            )
            .await
            {
                return png(b);
            }
        }
        return json_status(
            StatusCode::NOT_FOUND,
            json!({ "error": "No QR code for this bed" }),
        );
    }
    if action == "qr_code" && plant_id > 0 {
        return match blob(
            &state,
            "SELECT qr_code FROM `data_source` WHERE id = ?",
            BindValue::Id(plant_id),
        )
        .await
        {
            Some(b) => png(b),
            None => json_status(
                StatusCode::NOT_FOUND,
                json!({ "error": "No QR code generated for this plant" }),
            ),
        };
    }
    if action == "barcode" && plant_id > 0 {
        return match blob(
            &state,
            "SELECT barcode_image FROM `plant_barcodes`
             WHERE plant_id = ? AND source_table = 'data_source' AND barcode_image IS NOT NULL
             ORDER BY id LIMIT 1",
            BindValue::Id(plant_id),
        )
        .await
        {
            Some(b) => png(b),
            None => json_status(
                StatusCode::NOT_FOUND,
                json!({ "error": "No barcode for this plant" }),
            ),
        };
    }

    // 4. Preferences (any signed-in person). `save_pref` cuts the key to 50 and the
    // value to 200 characters. The SQL inserts a row, or updates it when that user
    // and key already exist (`ON DUPLICATE KEY UPDATE`).
    // Preferences: any signed-in person, before the admin check.
    if action == "save_pref" && !user.is_empty() {
        let key: String = f.str("key").trim().chars().take(50).collect();
        let val: String = f.str("value").trim().chars().take(200).collect();
        if !key.is_empty() {
            let _ = sqlx::query(
                "INSERT INTO `user_prefs` (username, pref_key, pref_value) VALUES (?, ?, ?)
                 ON DUPLICATE KEY UPDATE pref_value = VALUES(pref_value)",
            )
            .bind(&user)
            .bind(&key)
            .bind(&val)
            .execute(&state.db)
            .await;
        }
        return ok(json!({ "status": "ok" }));
    }
    // `load_prefs` returns all of this user's saved settings as one JSON object.
    if action == "load_prefs" && !user.is_empty() {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT pref_key, pref_value FROM `user_prefs` WHERE username = ?",
        )
        .bind(&user)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        let mut prefs = serde_json::Map::new();
        for (k, v) in rows {
            prefs.insert(k, json!(v));
        }
        return ok(json!({ "status": "ok", "prefs": prefs }));
    }

    // 5. Everything below needs an admin. Others get 403 "Not authorized".
    if !visitor.is_admin() {
        return json_status(StatusCode::FORBIDDEN, json!({ "error": "Not authorized" }));
    }

    // A closure that logs a database error and builds the 500 reply. Every block
    // below calls it so errors are logged in one place.
    let db_error = |e: sqlx::Error| {
        tracing::error!("plant_actions: {e}");
        json_status(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "Database error" }),
        )
    };

    // 6. list_done: which plants this person has marked done (default: the caller).
    if action == "list_done" {
        let who = {
            let w = pick("user").trim().to_string();
            if w.is_empty() {
                user.clone()
            } else {
                w
            }
        };
        if who.is_empty() {
            return ok(json!({ "done": [] }));
        }
        return match sqlx::query_scalar::<_, i64>(
            "SELECT plant_id FROM `user_done` WHERE username = ?",
        )
        .bind(&who)
        .fetch_all(&state.db)
        .await
        {
            Ok(ids) => ok(json!({ "user": who, "done": ids })),
            Err(e) => db_error(e),
        };
    }

    // 7. All remaining actions work on one plant, so a plant id is required.
    if plant_id == 0 {
        return ok(json!({ "error": "Missing plant_id" }));
    }

    // 8. save_photo_position: where the photo is centred, such as "50% 50%", and
    // whether it is flipped. The check accepts exactly two values like `12.5%` with
    // at most 3 digits before the point, so odd text never reaches the database.
    if action == "save_photo_position" {
        let pos = f
            .get("position")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("50% 50%")
            .to_string();
        let flip = if php_int(f.str("flip")) != 0 { 1 } else { 0 };
        let valid = {
            let parts: Vec<&str> = pos.split_whitespace().collect();
            parts.len() == 2
                && parts.iter().all(|p| {
                    let num = p.strip_suffix('%').unwrap_or("");
                    !num.is_empty()
                        && num.parse::<f64>().is_ok()
                        && num
                            .split('.')
                            .next()
                            .is_some_and(|d| d.len() <= 3 && d.chars().all(|c| c.is_ascii_digit()))
                })
        };
        if !valid {
            return ok(json!({ "error": "Invalid position format" }));
        }
        if let Err(e) =
            sqlx::query("UPDATE `data_source` SET photo_position = ?, photo_flip = ? WHERE id = ?")
                .bind(&pos)
                .bind(flip)
                .bind(plant_id)
                .execute(&state.db)
                .await
        {
            return db_error(e);
        }
        // Clear the cached widget data so the change shows up at once.
        state.invalidate_widget_cache();
        return ok(json!({ "status": "ok", "position": pos, "flip": flip }));
    }

    // 9. toggle_image_approved: mark the plant's photo approved or not (1 or 0).
    if action == "toggle_image_approved" {
        let approved = if php_int(f.str("approved")) != 0 {
            1
        } else {
            0
        };
        if let Err(e) = sqlx::query("UPDATE `data_source` SET image_approved = ? WHERE id = ?")
            .bind(approved)
            .bind(plant_id)
            .execute(&state.db)
            .await
        {
            return db_error(e);
        }
        state.invalidate_widget_cache();
        return ok(json!({ "status": "ok", "image_approved": approved }));
    }

    // 10. toggle_done: add or remove this person from the plant's done list.
    // `INSERT IGNORE` does nothing if the row exists already.
    if action == "toggle_done" {
        let done = f.get("done").is_some_and(|d| !empty(d));
        let r = if done {
            sqlx::query("INSERT IGNORE INTO `user_done` (plant_id, username) VALUES (?, ?)")
                .bind(plant_id)
                .bind(&user)
                .execute(&state.db)
                .await
        } else {
            sqlx::query("DELETE FROM `user_done` WHERE plant_id = ? AND username = ?")
                .bind(plant_id)
                .bind(&user)
                .execute(&state.db)
                .await
        };
        return match r {
            Ok(_) => ok(json!({ "status": "ok", "done": done as i64 })),
            Err(e) => db_error(e),
        };
    }

    // 11. set_default: make one of the plant's photos the main picture. Any
    // logged-in gardener may (checked above). The old main picture moves into
    // the extra list. (The old site allowed only one named user here.)
    if action == "set_default" {
        let filename = f.str("filename").trim().to_string();
        if filename.is_empty() {
            return ok(json!({ "error": "Missing filename" }));
        }
        let row = sqlx::query_as::<_, (Option<String>, Option<String>)>(
            "SELECT picture, additional_pictures FROM `data_source` WHERE id = ?",
        )
        .bind(plant_id)
        .fetch_optional(&state.db)
        .await;
        // Read the row: `None` means no such plant. The odd-looking `let ... else`
        // returns the "Plant not found" reply when the row is missing.
        let Some((picture, additional)) = (match row {
            Ok(r) => r,
            Err(e) => return db_error(e),
        }) else {
            return ok(json!({ "error": "Plant not found" }));
        };
        let imgs = merged_images(&picture, &additional);
        // The chosen file must already belong to this plant.
        if !imgs.contains(&filename) {
            return ok(json!({ "error": "That image is not part of this plant" }));
        }
        let rest: Vec<String> = imgs.into_iter().filter(|i| *i != filename).collect();
        let new_additional = rest.join(",");
        if let Err(e) = sqlx::query(
            "UPDATE `data_source` SET picture = ?, additional_pictures = ? WHERE id = ?",
        )
        .bind(&filename)
        .bind(&new_additional)
        .bind(plant_id)
        .execute(&state.db)
        .await
        {
            return db_error(e);
        }
        state.invalidate_widget_cache();
        return ok(
            json!({ "status": "ok", "picture": filename, "additional_pictures": new_additional }),
        );
    }

    // 12. save_future: update a wish-list plant from the JSON body. Only fields that
    // pass `safe_column` and are not in the `skip` list are used; their names are
    // placed into the SQL and their values are bound.
    if action == "save_future" {
        let Some(data) = json_body.as_ref().filter(|d| d.get("id").is_some()) else {
            return ok(json!({ "error": "Missing plant data" }));
        };
        let fp_id = data
            .get("id")
            .map(|v| {
                v.as_i64()
                    .unwrap_or_else(|| php_int(v.as_str().unwrap_or("")))
            })
            .unwrap_or(0);
        let skip = ["id", "action", "source_table", "date_added"];
        let sets: Vec<(String, Value)> = json_pairs(data)
            .into_iter()
            .filter(|(k, _)| !skip.contains(&k.as_str()) && safe_column(k))
            .collect();
        if sets.is_empty() {
            return ok(json!({ "error": "No fields to update" }));
        }
        let clause = sets
            .iter()
            .map(|(k, _)| format!("`{k}` = ?"))
            .collect::<Vec<_>>()
            .join(", ");
        let up_sql = format!("UPDATE `future_plants` SET {clause} WHERE id = ?");
        let mut qy = sqlx::query(sqlx::AssertSqlSafe(up_sql.as_str()));
        for (_, v) in &sets {
            qy = bind_json(qy, v);
        }
        return match qy.bind(fp_id).execute(&state.db).await {
            Ok(_) => ok(json!({ "status": "ok", "message": "Future plant updated" })),
            Err(e) => db_error(e),
        };
    }

    // 13. promote_future: move a wish-list plant into the main table. Copy every
    // column except `id` and `date_added`, insert it, then delete the old row.
    if action == "promote_future" {
        let fp_id = json_body
            .as_ref()
            .and_then(|d| d.get("id"))
            .map(|v| {
                v.as_i64()
                    .unwrap_or_else(|| php_int(v.as_str().unwrap_or("")))
            })
            .unwrap_or(plant_id);
        if fp_id <= 0 {
            return ok(json!({ "error": "Missing future plant ID" }));
        }
        let row = match sqlx::query("SELECT * FROM `future_plants` WHERE id = ?")
            .bind(fp_id)
            .fetch_optional(&state.db)
            .await
        {
            Ok(r) => r,
            Err(e) => return db_error(e),
        };
        let Some(row) = row else {
            return ok(json!({ "error": "Future plant not found" }));
        };
        let pairs: Vec<(String, Value)> = json_pairs(&row_to_json(&row))
            .into_iter()
            .filter(|(k, _)| k != "id" && k != "date_added")
            .collect();
        let cols = pairs
            .iter()
            .map(|(k, _)| format!("`{k}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let ph = vec!["?"; pairs.len()].join(", ");
        let ins_sql = format!("INSERT INTO `data_source` ({cols}) VALUES ({ph})");
        let mut qy = sqlx::query(sqlx::AssertSqlSafe(ins_sql.as_str()));
        for (_, v) in &pairs {
            qy = bind_json(qy, v);
        }
        // Add the garden row, move the plant's barcodes to it, remove the wish-list row: all
        // or none (a transaction). The PHP left the barcodes pointing at the deleted wish-list
        // row; this moves them, as the catalogue's own promote always did.
        let promoted: Result<i64, sqlx::Error> = async {
            let mut tx = state.db.begin().await?;
            let new_id = qy.execute(&mut *tx).await?.last_insert_id() as i64;
            sqlx::query("UPDATE `plant_barcodes` SET plant_id = ?, source_table = 'data_source' WHERE plant_id = ? AND source_table = 'future_plants'")
                .bind(new_id)
                .bind(fp_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM `future_plants` WHERE id = ?").bind(fp_id).execute(&mut *tx).await?;
            tx.commit().await?;
            Ok(new_id)
        }
        .await;
        let new_id = match promoted {
            Ok(id) => id,
            Err(e) => return db_error(e),
        };
        state.invalidate_widget_cache();
        let name = pairs
            .iter()
            .find(|(k, _)| k == "plant_name")
            .and_then(|(_, v)| v.as_str())
            .unwrap_or("");
        return ok(
            json!({ "status": "ok", "message": "Plant promoted to garden!", "new_id": new_id, "plant_name": name }),
        );
    }

    // 14. lookup_barcode: ask the online service (see `lookup_barcode` above).
    if action == "lookup_barcode" {
        let data = json_body.clone().unwrap_or(Value::Null);
        return lookup_barcode(&state, &data).await;
    }

    // No action matched: say so.
    ok(json!({ "error": "Unknown action" }))
}

// Tests run only with `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    // Checks that two comma lists merge with each name once, in order.
    #[test]
    fn merged_images_dedupes_in_order() {
        let got = merged_images(
            &Some("a.webp, b.webp".into()),
            &Some("b.webp,c.webp".into()),
        );
        assert_eq!(got, vec!["a.webp", "b.webp", "c.webp"]);
    }

    // Checks that punctuation and case are dropped from the cache key.
    #[test]
    fn cache_keys_strip_punctuation() {
        assert_eq!(
            cache_key("Lil' Kim", "Proven Winners"),
            "lilkim|provenwinners"
        );
    }

    // Checks that a name with a quote or `;--` is refused, because it could be SQL.
    #[test]
    fn only_plain_identifiers_reach_sql() {
        assert!(safe_column("plant_name"));
        assert!(!safe_column("plant_name`;--"));
        assert!(!safe_column(""));
    }
}
