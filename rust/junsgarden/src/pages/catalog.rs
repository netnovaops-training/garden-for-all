//! `catalog.php` — the Full Catalog.
//!
//! One URL, many jobs, chosen by what is added to the address, in the same
//! order the PHP checks them: `?autocomplete`, `?letter_counts`, `?qr_tags`,
//! `?ajax_load` (the plant list and search; also the Quick Search page's
//! data), favorites and likes, the admin's image and plant actions, the
//! notes and quantity calls, and finally the page itself. The plant cards
//! are drawn in the browser by js/app.js from the JSON below, so the JSON
//! has to match the PHP exactly.
//!
//! Signed in, the page shows the Add / Edit Plant form, Bulk Move and the
//! quantity log, and every admin action here works: save a plant (with
//! photos), bulk-move beds, delete, upload / rotate / delete gallery
//! photos, log quantity events, set a died year, and personal notes.
//!
//! HOW TO READ THIS FILE (for Jun). Start at `dispatch_get` and
//! `catalog_post` at the bottom: they are the switchboard, one `if` per
//! `?something` on the address, in the order the PHP checked them. Each
//! `if` calls one function above it, named after the PHP switch it answers
//! (`ajax_load`, `letter_counts`, `save_plant`, ...). `plant_json` is the
//! shape of one plant as `js/app.js` expects it; change a key there and the
//! cards stop drawing. Nothing in this file writes HTML except
//! `render_page`, which fills `templates/catalog.html`.

// Imports. `use` brings a name from another file or a library into this file so it can be written
// short. Everything below is a name this file needs.
use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::{
    body::Bytes,
    extract::{OriginalUri, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde_json::{json, Map, Value};
use sqlx::mysql::MySqlRow;
use sqlx::Row;

use crate::formdata::{self, FormData};
use crate::guard::off_thread;
use crate::images;
use crate::layout::{self, Page};
use crate::php::{date_ymd, filemtime, php_int, preg_quote, search_norm};
use crate::rowjson::row_to_json;
use crate::session::{self, Visitor};
use crate::AppState;

/// The data handed to the page template `templates/catalog.html`.
///
/// Rust idea: a `struct` is a bundle of named fields. `i64` is a whole number that may be negative,
/// `u64` and `usize` are whole numbers that cannot be negative, `bool` is true or false, and
/// `String` is text this struct owns. The `Template` derive (askama) checks at build time that
/// every name the HTML file uses exists here, so a typo fails the build instead of the page.
#[derive(Template)]
#[template(path = "catalog.html")]
struct CatalogBody {
    plant_count: i64,
    variety_count: usize,
    app_v: u64,
    search_v: u64,
    is_admin: bool,
    current_user: String,
    can_upload: bool,
}

/// Send a JSON answer to the browser.
///
/// Takes any JSON value (`Value` is serde_json's type for "any JSON"). Gives back an HTTP
/// `Response` with the `application/json` header. If the value somehow cannot be turned into text,
/// it sends `[]` instead of crashing. The page's JavaScript reads every `?something` call on this
/// file as JSON, so they all end here.
fn json_response(body: Value) -> Response {
    (
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&body).unwrap_or_else(|_| "[]".to_string()),
    )
        .into_response()
}

/// A database failure, answered the way the PHP did (`{"error": ...}`) but without the
/// database's own message: that text names tables and columns, which is nobody's business.
/// The full message goes to the log.
fn db_error(e: sqlx::Error) -> Response {
    tracing::error!("catalog: {e}");
    json_response(json!({ "error": "The catalogue could not be read just now. Please try again." }))
}

/// Send the browser back to `catalog.php` with a 302 "Found" redirect.
///
/// The admin forms (save plant, bulk move, delete) are normal form posts, so after they finish the
/// PHP sent the person back to the catalog. This does the same.
fn redirect_catalog() -> Response {
    (StatusCode::FOUND, [(header::LOCATION, "catalog.php")]).into_response()
}

/// Run a query with string binds and return every row.
///
/// Rust idea: `async fn` is a function that can pause while it waits (here, for the database). A
/// caller must write `.await` to run it and get its result. `Result<A, B>` means "either a good
/// answer A or an error B". `&` means "borrow this, do not take it".
async fn rows(state: &AppState, sql: &str, binds: &[String]) -> Result<Vec<MySqlRow>, sqlx::Error> {
    let mut q = sqlx::query(sql);
    for b in binds {
        q = q.bind(b);
    }
    q.fetch_all(&state.db).await
}

// --- who has favorited / liked what -----------------------------------------

/// `plant_id -> [usernames]`, in table order, from `user_favorites` or
/// `user_likes`.
///
/// Note: `format!` builds a String; `{table}` is replaced by the name. The table name is only ever
/// a fixed word from this file (never typed by a visitor), because SQL cannot take a table name as
/// a `?` placeholder. The `?` after `.await` means "if this failed, stop and return the error to my
/// caller". `.map(|row| ...)` runs a small unnamed function (a closure) on every row. Reading a
/// column by position (0, 1) avoids sqlx's strict type checks.
async fn user_map(state: &AppState, table: &str) -> Result<Vec<(i64, String)>, sqlx::Error> {
    let r = sqlx::query(&format!("SELECT plant_id, username FROM {table}"))
        .fetch_all(&state.db)
        .await?;
    Ok(r.iter()
        .map(|row| {
            (
                row.try_get_unchecked::<i64, _>(0).unwrap_or(0),
                row.try_get_unchecked::<String, _>(1).unwrap_or_default(),
            )
        })
        .collect())
}

/// All usernames in `map` that belong to plant `id`, in table order.
///
/// Used to fill the `favorited_by` and `liked_by` lists on each plant. `.filter` keeps matching
/// items, `.map` changes each one, `.collect` gathers them into a list.
fn users_for(map: &[(i64, String)], id: i64) -> Vec<String> {
    map.iter()
        .filter(|(p, _)| *p == id)
        .map(|(_, u)| u.clone())
        .collect()
}

/// Plant ids favorited by one named person, in table order.
fn ids_for_user(map: &[(i64, String)], name: &str) -> Vec<i64> {
    map.iter()
        .filter(|(_, u)| u == name)
        .map(|(p, _)| *p)
        .collect()
}

/// Plant ids favorited by anyone who is not Joshua, the author or Kona, each
/// once, in order of first appearance (`array_unique`).
fn community_ids(map: &[(i64, String)]) -> Vec<i64> {
    let mut out: Vec<i64> = Vec::new();
    for (p, u) in map {
        if !["Joshua", "Jason", "Kona"].contains(&u.as_str()) && !out.contains(p) {
            out.push(*p);
        }
    }
    out
}

/// One plant as the page's JavaScript expects it: the row, then who
/// favorited and liked it, with the binary tag image dropped and the ™/®
/// marks stripped from every text value.
///
/// Steps: 1 start from the database row as JSON, 2 note which table it came from (the wish list or
/// the garden), 3 add who favorited it and how many, 4 add who liked it, 5 remove the binary QR
/// image, 6 strip the trademark marks.
fn plant_json(
    row: &MySqlRow,
    favs: &[(i64, String)],
    likes: &[(i64, String)],
    me: &str,
    source_table: Option<&str>,
) -> Value {
    // 1. Turn the whole database row into a JSON object (a map of column name to value).
    let mut v = row_to_json(row);
    // `let ... else` unpacks it as an object; if it is somehow not an object, give it back
    // unchanged. `&mut` means "borrow so it can be changed".
    let Value::Object(m) = &mut v else { return v };
    // 2. `Option` means "maybe nothing". If the caller said which table the row came from, record
    // it so the page knows whether to treat it as a wish-list plant.
    if let Some(t) = source_table {
        m.insert("source_table".into(), json!(t));
    }
    // 3. Read the plant's id back out of the JSON (0 if missing) to look up its favorites.
    let id = m.get("id").and_then(Value::as_i64).unwrap_or(0);

    let fav_by = users_for(favs, id);
    m.insert("favorited_by".into(), json!(fav_by));
    m.insert("fav_count".into(), json!(fav_by.len()));
    m.insert("is_favorited".into(), json!(fav_by.iter().any(|u| u == me)));

    // 4. The same for likes.
    let like_by = users_for(likes, id);
    m.insert("liked_by".into(), json!(like_by));
    m.insert("like_count".into(), json!(like_by.len()));
    m.insert("is_liked".into(), json!(like_by.iter().any(|u| u == me)));

    // 5. Drop the QR image column: it is raw binary and the page does not need it in the JSON.
    m.shift_remove("qr_code");
    // 6. Walk every value; for text values, remove the ™ and ® marks (the PHP did this too).
    for (_, val) in m.iter_mut() {
        if let Value::String(s) = val {
            if s.contains('™') || s.contains('®') {
                *s = s.replace(['™', '®'], "");
            }
        }
    }
    v
}

// --- ?ajax_load -------------------------------------------------------------

/// `?ajax_load`: the plant list and search, as JSON `{ "results": [...] }`.
///
/// Takes the database (`state`), the address parameters (`q`), and the visitor's database name
/// (`me`, used for their own favorites). Gives back JSON. The Catalog page, the Quick Search page
/// and the plant window all call this, so its output must match the PHP exactly.
///
/// Three ways in: `plant_id=N` (one plant, used by QR links), a paged search (9 plants a page, 3 on
/// mobile), and special search words like `edible:full`, `view:jun_done`, `^A-B` (the A-Z bar) and
/// the Favs buttons.
async fn ajax_load(state: &AppState, q: &HashMap<String, String>, me: &str) -> Response {
    // A database error becomes a JSON error answer, as in the PHP (minus the database's text).
    let err = db_error;

    // 1. Direct lookup by id (QR-code links, the plant modal). `source=future` picks the wish-list
    // table. Only two fixed table names can reach the SQL.
    // Direct lookup by id (QR-code links, the plant modal).
    if let Some(pid) = q.get("plant_id") {
        let pid = php_int(pid);
        let table = if q.get("source").map(String::as_str) == Some("future") {
            "future_plants"
        } else {
            "data_source"
        };
        let found = sqlx::query(&format!("SELECT * FROM `{table}` WHERE id = ?"))
            .bind(pid)
            .fetch_optional(&state.db)
            .await;
        // `match` checks each possible outcome: database error, no row found, or a row found.
        return match found {
            Err(e) => err(e),
            Ok(None) => json_response(json!({ "error": "Plant not found" })),
            Ok(Some(row)) => {
                let (favs, likes) = match (
                    user_map(state, "user_favorites").await,
                    user_map(state, "user_likes").await,
                ) {
                    (Ok(f), Ok(l)) => (f, l),
                    (Err(e), _) | (_, Err(e)) => return err(e),
                };
                json_response(
                    json!({ "results": [plant_json(&row, &favs, &likes, me, Some(table))] }),
                )
            }
        };
    }

    // 2. Read paging and search settings. `.trim()` removes spaces at both ends. `php_int` reads a
    // number the way PHP's (int) cast does.
    let search_raw = q
        .get("search")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let page = q.get("page").map(|p| php_int(p)).unwrap_or(1);
    let limit: i64 = if q.contains_key("mobile") { 3 } else { 9 };
    // Page 1 starts at row 0, page 2 at row `limit`, and so on.
    let offset = (page - 1) * limit;
    if offset < 0 {
        // MariaDB refuses a negative OFFSET; the PHP reports the error as JSON.
        return json_response(json!({ "error": "invalid page" }));
    }

    // 3. A search that starts with `^` is the A-Z bar, such as `^A-B`: split on `-` into letters,
    // dropping blanks and "0".
    let az_letters: Vec<String> = if let Some(rest) = search_raw.strip_prefix('^') {
        rest.split('-')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "0")
            .collect()
    } else {
        Vec::new()
    };
    // `edible:full` and similar searches. `search_norm` makes the text safe to compare: special
    // characters become spaces, as in the SQL below.
    let edibility = search_raw.strip_prefix("edible:").unwrap_or("").to_string();
    let search = search_norm(&search_raw);

    // 4. Load who favorited and liked what once, so every plant can be marked.
    let (favs, likes) = match (
        user_map(state, "user_favorites").await,
        user_map(state, "user_likes").await,
    ) {
        (Ok(f), Ok(l)) => (f, l),
        (Err(e), _) | (_, Err(e)) => return err(e),
    };

    // `tail` is the paging part of the SQL. It holds only numbers made by this code, so it is safe
    // to put in the text.
    let tail = format!("LIMIT {limit} OFFSET {offset}");
    let order = "ORDER BY flower_bed, plant_name";
    let mut future_rows: Vec<MySqlRow> = Vec::new();

    // A helper for the four Favs buttons: build a query for "these plant ids" with one `?` per id,
    // or nothing when the list is empty.
    let fav_branch = |ids: Vec<i64>| -> Option<(String, Vec<String>)> {
        if ids.is_empty() {
            return None;
        }
        let ph = vec!["?"; ids.len()].join(",");
        Some((
            format!("SELECT * FROM `data_source` WHERE id IN ({ph}) {order} {tail}"),
            ids.iter().map(|i| i.to_string()).collect(),
        ))
    };

    // 5. Choose which query runs, in the same order the PHP decides. Each branch gives the SQL text
    // and the values for its `?` placeholders (placeholders keep typed text from being run as SQL).
    // Which query runs, in the same order the PHP decides.
    let plan: Option<(String, Vec<String>)> = if !edibility.is_empty() {
        // The edibility filters. The WHERE text is picked from a fixed list; an unknown word
        // matches nothing (`1=0`).
        let w = match edibility.as_str() {
            "full" => "fruit_edibility = 'Edible' AND LOWER(toxicity) LIKE 'non-toxic%'",
            "semi" => "fruit_edibility = 'Edible' AND LOWER(toxicity) LIKE '%dog%'",
            "wildlife" => "fruit_edibility = 'Wildlife Only'",
            "toxic" => {
                "(fruit_edibility IN ('Toxic','Inedible/Toxic') OR LOWER(toxicity) LIKE '%humans%')"
            }
            _ => "1=0",
        };
        Some((
            format!("SELECT * FROM `data_source` WHERE ({w}) AND died = 0 {order} {tail}"),
            vec![],
        ))
    // `view:` filters: image review, who has marked a plant done, and "not done by me" (a LEFT JOIN
    // that keeps plants with no matching done row).
    } else if let Some(view) = search_raw.strip_prefix("view:") {
        match view {
            "image_approved" => Some((format!("SELECT * FROM `data_source` WHERE image_approved = 1 AND died = 0 {order} {tail}"), vec![])),
            "image_not_approved" => Some((format!("SELECT * FROM `data_source` WHERE (image_approved = 0 OR image_approved IS NULL) AND died = 0 {order} {tail}"), vec![])),
            "jun_done" => Some((format!("SELECT d.* FROM `data_source` d INNER JOIN `user_done` ud ON d.id = ud.plant_id AND ud.username = 'Joshua' WHERE d.died = 0 ORDER BY d.flower_bed, d.plant_name {tail}"), vec![])),
            "jason_done" => Some((format!("SELECT d.* FROM `data_source` d INNER JOIN `user_done` ud ON d.id = ud.plant_id AND ud.username = 'Jason' WHERE d.died = 0 ORDER BY d.flower_bed, d.plant_name {tail}"), vec![])),
            "not_done" => Some((format!("SELECT d.* FROM `data_source` d LEFT JOIN `user_done` ud ON d.id = ud.plant_id AND ud.username = ? WHERE ud.plant_id IS NULL AND d.died = 0 ORDER BY d.flower_bed, d.plant_name {tail}"), vec![me.to_string()])),
            _ => None,
        }
    // The Favs buttons are matched by their exact label text, emoji included.
    } else if search_raw == "⭐ Jun's Favs" {
        fav_branch(ids_for_user(&favs, "Joshua"))
    } else if search_raw == "⭐ Author's favs" {
        fav_branch(ids_for_user(&favs, "Jason"))
    } else if search_raw == "🐾 Kona's Favs" {
        fav_branch(ids_for_user(&favs, "Kona"))
    } else if search_raw == "💖 Community Favs" {
        fav_branch(community_ids(&favs))
    // A-Z bar: plant names that start with any chosen letter, leaving out dead plants.
    } else if !az_letters.is_empty() {
        let clauses = vec!["plant_name LIKE ?"; az_letters.len()].join(" OR ");
        Some((
            format!("SELECT * FROM `data_source` WHERE ({clauses}) AND flower_bed NOT LIKE '%Dead%' ORDER BY plant_name ASC {tail}"),
            az_letters.iter().map(|l| format!("{}%", l.to_uppercase())).collect(),
        ))
    // Default: the normal text search across many columns. The first two columns are compared after
    // replacing symbols with spaces and squeezing repeated spaces, so "o'brien" still matches "o
    // brien".
    } else {
        let sql = format!(
            "SELECT * FROM `data_source`
             WHERE REGEXP_REPLACE(REGEXP_REPLACE(plant_name,     '[^a-zA-Z0-9 ]', ' '), ' +', ' ') LIKE ?
                OR REGEXP_REPLACE(REGEXP_REPLACE(botanical_name, '[^a-zA-Z0-9 ]', ' '), ' +', ' ') LIKE ?
                OR family LIKE ? OR flower_bed LIKE ?
                OR company LIKE ? OR local_store LIKE ? OR care_notes LIKE ? OR maintenance_notes LIKE ?
                OR attracts REGEXP ? OR sun_exposure LIKE ? OR fertilizer LIKE ?
                OR size_spacing LIKE ? OR cuttings LIKE ? OR cleanup_notes LIKE ?
                OR zones LIKE ? OR water_needs LIKE ? OR deer_resistance LIKE ?
                OR season LIKE ? OR month_start LIKE ? OR month_end LIKE ?
                OR spacing LIKE ? OR new_name LIKE ? OR year_planted LIKE ?
                OR admin_todo LIKE ? OR size LIKE ? OR grow_months LIKE ? OR bloom_months LIKE ?
                OR soil_ph LIKE ? OR soil_type LIKE ? OR toxicity LIKE ?
                OR bloom_color LIKE ? OR foliage_color LIKE ? OR disease_susceptibility LIKE ?
                OR propagation_method LIKE ? OR species LIKE ?
             ORDER BY flower_bed, plant_name {tail}"
        );
        // `binds` holds one value per `?`. Almost all use the raw text; the first two use the
        // normalised text; the ninth (`attracts`) is a regular expression that matches the word as
        // a whole item in a comma list.
        let term_norm = format!("%{search}%");
        let term_raw = format!("%{search_raw}%");
        let attracts = format!("(^|,)[ ]*{}[ ]*(,|$)", preg_quote(&search_raw));
        let mut binds = vec![term_raw.clone(); 35];
        binds[0] = term_norm.clone();
        binds[1] = term_norm;
        binds[8] = attracts;

        // The wish list is searched too, but only when something was typed. A failure here is
        // ignored, as in the PHP.
        // The wish list is searched too, but only when something was typed.
        if !search_raw.trim().is_empty() {
            let fut = rows(
                state,
                &format!("SELECT * FROM `future_plants` WHERE plant_name LIKE ? OR botanical_name LIKE ? OR company LIKE ? ORDER BY plant_name {tail}"),
                &[term_raw.clone(), term_raw.clone(), term_raw],
            )
            .await;
            if let Ok(r) = fut {
                future_rows = r;
            }
        }
        Some((sql, binds))
    };

    // 6. Run the chosen query. `None` means "no query" (for example an empty Favs list), so there
    // are no rows.
    let main_rows = match plan {
        None => Vec::new(),
        Some((sql, binds)) => match rows(state, &sql, &binds).await {
            Ok(r) => r,
            Err(e) => return err(e),
        },
    };

    // 7. Turn every row into JSON and add the wish-list rows at the end.
    let mut results: Vec<Value> = main_rows
        .iter()
        .map(|r| plant_json(r, &favs, &likes, me, None))
        .collect();
    results.extend(
        future_rows
            .iter()
            .map(|r| plant_json(r, &favs, &likes, me, Some("future_plants"))),
    );
    json_response(json!({ "results": results }))
}

// --- the small endpoints ----------------------------------------------------

/// `?autocomplete&q=...`: suggestions for the search box, as a JSON list of `{value, type}`.
///
/// Searches plant names, wish-list names, families and brands, then removes repeats (ignoring case)
/// and keeps at most 10. Fewer than 2 typed letters gives an empty list.
async fn autocomplete(state: &AppState, q: &HashMap<String, String>) -> Response {
    // 1. Read and clean the typed text.
    let raw = q.get("q").map(|s| s.trim().to_string()).unwrap_or_default();
    let term = search_norm(&raw);
    if term.chars().count() < 2 {
        return json_response(json!([]));
    }
    // `like` matches anywhere in the name; `starts` matches only the beginning, so names that start
    // with the text are listed first.
    let like = format!("%{term}%");
    let starts = format!("{term}%");

    // 2. Five small queries, one per kind of suggestion, each with its own limit.
    let mut found: Vec<Value> = Vec::new();
    let queries: [(&'static str, Vec<String>); 5] = [
        (
            "SELECT DISTINCT plant_name AS value, 'plant' AS type FROM `data_source`
             WHERE REGEXP_REPLACE(plant_name, '[^a-zA-Z0-9 ]', ' ') LIKE ? AND died = 0
             ORDER BY CASE WHEN REGEXP_REPLACE(plant_name, '[^a-zA-Z0-9 ]', ' ') LIKE ? THEN 0 ELSE 1 END, plant_name
             LIMIT 6",
            vec![like.clone(), starts],
        ),
        (
            "SELECT DISTINCT plant_name AS value, 'future' AS type FROM `future_plants`
             WHERE plant_name LIKE ? ORDER BY plant_name LIMIT 4",
            vec![like.clone()],
        ),
        (
            "SELECT DISTINCT TRIM(family) AS value, 'family' AS type FROM `data_source`
             WHERE family LIKE ? AND died = 0 AND TRIM(family) != '' ORDER BY family LIMIT 3",
            vec![like.clone()],
        ),
        (
            "SELECT DISTINCT company AS value, 'brand' AS type FROM `data_source`
             WHERE company LIKE ? AND died = 0 AND company != '' AND company IS NOT NULL ORDER BY company LIMIT 3",
            vec![like.clone()],
        ),
        (
            "SELECT DISTINCT company AS value, 'brand' AS type FROM `future_plants`
             WHERE company LIKE ? AND company != '' AND company IS NOT NULL ORDER BY company LIMIT 3",
            vec![like],
        ),
    ];
    // 3. Run them in order and collect the rows.
    for (i, (sql, binds)) in queries.into_iter().enumerate() {
        match rows(state, sql, &binds).await {
            Ok(r) => found.extend(r.iter().map(row_to_json)),
            // Only the wish-list lookup is allowed to fail quietly in the PHP;
            // any other failure returns an empty list.
            Err(_) if i == 1 => {}
            Err(_) => return json_response(json!([])),
        }
    }

    // 4. Remove repeated suggestions. A `HashSet` remembers what was seen; `insert` returns false
    // for a repeat, so `filter` drops it.
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<Value> = found
        .into_iter()
        .filter(|r| {
            let key = r
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_lowercase();
            seen.insert(key)
        })
        .take(10)
        .collect();
    json_response(Value::Array(unique))
}

/// `?letter_counts`: how many living plants start with each letter A to Z, as JSON. The A-Z bar
/// uses it to show counts and grey out empty letters.
///
/// One query (the PHP asked 26 times), with the very same test per letter: `plant_name LIKE
/// 'A%'`, which follows the database's collation (any case, and accented "Á" counts as A).
async fn letter_counts(state: &AppState) -> Response {
    let letters: Vec<char> = ('A'..='Z').collect();
    let sums = letters
        .iter()
        .map(|l| format!("CAST(COALESCE(SUM(plant_name LIKE '{l}%'), 0) AS SIGNED)"))
        .collect::<Vec<_>>();
    // The letters are this function's own A-Z, never visitor text, so they may be in the SQL.
    let sql = format!(
        "SELECT {} FROM `data_source` WHERE died = 0 AND flower_bed NOT LIKE '%Dead%'",
        sums.join(", ")
    );
    let row = match sqlx::query(&sql).fetch_one(&state.db).await {
        Ok(r) => r,
        Err(e) => return db_error(e),
    };
    let m: Map<String, Value> = letters
        .iter()
        .enumerate()
        .map(|(i, l)| (l.to_string(), json!(row.try_get::<i64, _>(i).unwrap_or(0))))
        .collect();
    json_response(Value::Object(m))
}

/// `?qr_tags`: the plants that have a QR code or barcode image, for the printable tag sheet.
///
/// `LEFT JOIN` keeps a plant even without a barcode row. `(x IS NOT NULL) AS has_qr` turns "is
/// there an image" into 1 or 0 without sending the image itself.
async fn qr_tags(state: &AppState) -> Response {
    let r = sqlx::query(
        "SELECT d.id, d.plant_name, d.flower_bed,
                (d.qr_code IS NOT NULL) AS has_qr,
                pb.barcode,
                (pb.id IS NOT NULL) AS has_barcode
         FROM `data_source` d
         LEFT JOIN `plant_barcodes` pb
           ON pb.plant_id = d.id AND pb.source_table = 'data_source' AND pb.barcode_image IS NOT NULL
         WHERE d.died = 0 AND d.flower_bed NOT LIKE '%Dead%'
           AND (d.qr_code IS NOT NULL OR pb.id IS NOT NULL)
         ORDER BY d.flower_bed, d.plant_name",
    )
    .fetch_all(&state.db)
    .await;
    match r {
        Ok(rows) => {
            json_response(json!({ "tags": rows.iter().map(row_to_json).collect::<Vec<_>>() }))
        }
        Err(e) => db_error(e),
    }
}

/// `?toggle_favorite=ID` / `?toggle_like=ID`: add the person's mark, or
/// take it away if it is already there.
async fn toggle(state: &AppState, table: &str, plant_id: i64, me: &str) -> Response {
    // 1. Does this person already have the mark on this plant?
    let exists = sqlx::query(&format!(
        "SELECT id FROM {table} WHERE username = ? AND plant_id = ?"
    ))
    .bind(me)
    .bind(plant_id)
    .fetch_optional(&state.db)
    .await;
    // 2. Yes: remove it. No: add it. A database error is logged and gives a 500 answer.
    let status = match exists {
        Ok(Some(_)) => {
            let _ = sqlx::query(&format!(
                "DELETE FROM {table} WHERE username = ? AND plant_id = ?"
            ))
            .bind(me)
            .bind(plant_id)
            .execute(&state.db)
            .await;
            "removed"
        }
        Ok(None) => {
            let _ = sqlx::query(&format!(
                "INSERT INTO {table} (username, plant_id) VALUES (?, ?)"
            ))
            .bind(me)
            .bind(plant_id)
            .execute(&state.db)
            .await;
            "added"
        }
        Err(e) => {
            tracing::error!("{table} toggle failed: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    json_response(json!({ "status": status }))
}

// --- the admin's image actions ----------------------------------------------

/// The folder that holds uploaded plant photos (`images/Database` under the web folder).
fn upload_dir(state: &AppState) -> std::path::PathBuf {
    state.www.join("images/Database")
}

/// A file name that stays inside the photo folder.
///
/// Gives back `Some(name)` if the name is safe, or `None` (nothing) if it has a slash, a `..`, or a
/// leading dot. This blocks someone sending `../../secret` to reach files outside the photo folder.
fn safe_filename(name: &str) -> Option<String> {
    let n = name.trim();
    if n.is_empty() || n.contains('/') || n.contains('\\') || n.contains("..") || n.starts_with('.')
    {
        return None;
    }
    Some(n.to_string())
}

/// `?rotate_image&filename=...` (admin only): turn one photo 90 degrees clockwise in place.
///
/// Gives back JSON with `status` of `success` or `error`. The default placeholder picture can never
/// be rotated.
async fn rotate_image(state: &AppState, q: &HashMap<String, String>) -> Response {
    let name = q
        .get("filename")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if name.is_empty() || name == "default.webp" {
        return json_response(
            json!({ "status": "error", "message": "Invalid or missing filename." }),
        );
    }
    let Some(safe) = safe_filename(&name) else {
        return json_response(json!({ "status": "error", "message": "File not found." }));
    };
    let path = upload_dir(state).join(&safe);
    if !path.is_file() {
        return json_response(json!({ "status": "error", "message": "File not found." }));
    }
    if off_thread(move || images::rotate_clockwise_in_place(&path)).await {
        json_response(json!({ "status": "success", "rotated": safe }))
    } else {
        json_response(json!({ "status": "error", "message": "Failed to process image." }))
    }
}

/// `?delete_image&plant_id=N&filename=...` (admin only): remove one gallery photo.
///
/// Steps: delete the file from disk, then remove its name from the plant's comma-separated
/// `additional_pictures` list, then clear the widget cache.
async fn delete_image(state: &AppState, q: &HashMap<String, String>) -> Response {
    let plant_id = q.get("plant_id").map(|p| php_int(p)).unwrap_or(0);
    let name = q
        .get("filename")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if plant_id == 0 || name.is_empty() {
        return json_response(json!({ "status": "error", "message": "Missing ID or filename." }));
    }
    if let Some(safe) = safe_filename(&name) {
        let _ = std::fs::remove_file(upload_dir(state).join(safe));
    }
    // Read the plant's current comma-separated list of extra pictures (`flatten` removes the nested
    // `Option` layers; "0" or empty means none).
    let existing = sqlx::query_scalar::<_, Option<String>>(
        "SELECT additional_pictures FROM `data_source` WHERE id = ?",
    )
    .bind(plant_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .flatten();
    match existing.filter(|s| !s.is_empty() && s != "0") {
        Some(all) => {
            // Keep every name except the one deleted.
            let kept: Vec<&str> = all.split(',').filter(|p| p.trim() != name).collect();
            let _ = sqlx::query("UPDATE `data_source` SET additional_pictures = ? WHERE id = ?")
                .bind(kept.join(","))
                .bind(plant_id)
                .execute(&state.db)
                .await;
            state.invalidate_widget_cache();
            json_response(json!({ "status": "success", "deleted": name }))
        }
        None => {
            json_response(json!({ "status": "error", "message": "Plant has no gallery images." }))
        }
    }
}

/// Resize and save the photos posted under `field`, named from the plant's name and family
/// (`images::process_and_save_image`), on a helper thread. Gives the saved file names, in the
/// order they were sent; a photo that cannot be read is skipped.
async fn save_photos(
    state: &AppState,
    f: &FormData,
    field: &str,
    name: String,
    family: String,
) -> Vec<String> {
    let dir = upload_dir(state);
    let files: Vec<(Vec<u8>, String)> = f
        .files_for(field)
        .iter()
        .map(|u| (u.bytes.clone(), u.filename.clone()))
        .collect();
    off_thread(move || {
        files
            .iter()
            .filter_map(|(bytes, filename)| {
                images::process_and_save_image(bytes, filename, &dir, &name, &family)
            })
            .collect()
    })
    .await
}

/// Upload gallery photos for a plant (POST `?upload_gallery=1`, admin only).
///
/// Each file is resized and saved by `images::process_and_save_image`, named from the plant's name
/// and family. The new names are added to `additional_pictures`. Gives JSON `{status, new_images}`.
async fn upload_gallery(state: &AppState, visitor: &Visitor, f: &FormData) -> Response {
    if !visitor.can_upload(state) {
        return json_response(json!({ "status": "error", "message": "Not authorized to upload." }));
    }
    // 1. Look up the plant's name and family, used to name the saved files.
    let plant_id = php_int(f.str("plant_id"));
    let info = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT plant_name, family FROM `data_source` WHERE id = ?",
    )
    .bind(plant_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();
    let (p_name, p_fam) = match info {
        Some((n, fam)) => (
            n.unwrap_or_else(|| "Unknown".into()),
            fam.unwrap_or_else(|| "Unknown".into()),
        ),
        None => ("Unknown".to_string(), "Unknown".to_string()),
    };
    // 2. Process and save each uploaded file (on a helper thread: resizing is slow).
    let uploaded = save_photos(state, f, "gallery_images[]", p_name, p_fam).await;
    if uploaded.is_empty() {
        return json_response(
            json!({ "status": "error", "message": "No files moved. Check folder permissions." }),
        );
    }
    // 3. Read the current picture list, add the new names, and save it back (blank and "0" entries
    // are dropped).
    let existing = sqlx::query_scalar::<_, Option<String>>(
        "SELECT additional_pictures FROM `data_source` WHERE id = ?",
    )
    .bind(plant_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_default();
    let mut all: Vec<String> = if existing.is_empty() || existing == "0" {
        Vec::new()
    } else {
        existing.split(',').map(str::to_string).collect()
    };
    all.extend(uploaded.iter().cloned());
    let joined = all
        .into_iter()
        .filter(|s| !s.is_empty() && s != "0")
        .collect::<Vec<_>>()
        .join(",");
    let _ = sqlx::query("UPDATE `data_source` SET additional_pictures = ? WHERE id = ?")
        .bind(joined)
        .bind(plant_id)
        .execute(&state.db)
        .await;
    state.invalidate_widget_cache();
    json_response(json!({ "status": "success", "new_images": uploaded }))
}

// --- the admin's plant actions ----------------------------------------------

/// Bulk Move (admin only): move several plants to one bed. Comes from the Bulk Move form.
///
/// Moving to "Dead or Removed" also sets the quantity to 0 and marks the plants died. Always ends
/// with a redirect back to the catalog.
async fn bulk_update_bed(state: &AppState, f: &FormData) -> Response {
    // Ids arrive as "3,7,9"; split them and ignore zeros.
    let ids: Vec<i64> = f
        .str("selected_ids")
        .split(',')
        .map(php_int)
        .filter(|i| *i != 0)
        .collect();
    let new_bed = f.str("new_bed_name").trim().to_string();
    if !ids.is_empty() && !new_bed.is_empty() && new_bed != "0" {
        let qty_update = if new_bed == "Dead or Removed" {
            ", qty = 0, died = 1"
        } else {
            ""
        };
        // One `?` per id for the `IN (...)` list.
        let ph = vec!["?"; ids.len()].join(",");
        let bulk_sql =
            format!("UPDATE `data_source` SET flower_bed = ? {qty_update} WHERE id IN ({ph})");
        let mut q = sqlx::query(&bulk_sql).bind(&new_bed);
        for id in &ids {
            q = q.bind(id);
        }
        let _ = q.execute(&state.db).await;
        state.invalidate_widget_cache();
    }
    redirect_catalog()
}

/// Bind one JSON value the way PDO would bind the PHP value.
///
/// The `'q` marks only tell Rust that the query and the value live equally long; read the signature
/// as "take a query and a value, give back the query with the value added". `bind` attaches one
/// value to a `?`. Used when column names come from the database at run time.
fn bind_value<'q>(
    q: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    v: &'q Value,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    match v {
        Value::Null => q.bind(Option::<String>::None),
        Value::Bool(b) => q.bind(*b as i64),
        Value::Number(n) => match n.as_i64() {
            Some(i) => q.bind(i),
            None => q.bind(n.as_f64().unwrap_or(0.0)),
        },
        Value::String(s) => q.bind(s.as_str()),
        other => q.bind(other.to_string()),
    }
}

/// `save_plant`: the Add / Edit Plant form, including promoting a wish-list
/// plant into the garden when it is given a real bed.
///
/// Takes the posted form (`f`). Steps: 1 read the fields, 2 save any uploaded photos, 3 either
/// promote a wish-list plant, edit a plant, or add a new one, 4 clear the widget cache, 5 redirect
/// (or show an error page).
async fn save_plant(state: &AppState, visitor: &Visitor, f: &FormData) -> Response {
    // 1. Which plant is being edited? 0 or missing means a new plant.
    let edit_id = php_int(f.str("edit_id"));
    let edit_id = if edit_id == 0 { None } else { Some(edit_id) };
    let edit_source = f.str("edit_source").trim().to_string();
    // Keep only letters, digits and spaces in the name (the PHP did the same).
    let p_name: String = f
        .get("plant_name")
        .unwrap_or("Unknown")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace())
        .collect();

    // `flower_bed[]` is a multi-select; the form parser keeps the last value
    // for a repeated name, so the beds were joined there (see formdata.rs).
    let bed_value = {
        let v = f
            .get("flower_bed[]")
            .or_else(|| f.get("flower_bed"))
            .unwrap_or("To Be Sorted");
        if v.is_empty() {
            "To Be Sorted".to_string()
        } else {
            v.to_string()
        }
    };
    // Quantity and year are only changed when they were filled in.
    let new_qty: Option<i64> = f.get("p_qty_edit").filter(|s| !s.is_empty()).map(php_int);
    let new_year: Option<String> = f
        .get("p_year_added")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    // 2. Save uploaded photos (only if this person may upload). The first becomes the main picture,
    // the rest go to the gallery.
    let mut additional: Vec<String> = if visitor.can_upload(state) {
        save_photos(
            state,
            f,
            "plant_image[]",
            p_name.clone(),
            "General".to_string(),
        )
        .await
    } else {
        Vec::new()
    };
    let picture: Option<String> = if additional.is_empty() {
        None
    } else {
        Some(additional.remove(0))
    };

    // 3. Do the database work inside one `async { ... }` block so the `?` operator can be used on
    // each step. The block's result is checked after it.
    let result: Result<(), sqlx::Error> = async {
        // 3a. Editing a wish-list plant.
        if let (Some(id), "future_plants") = (edit_id, edit_source.as_str()) {
            if bed_value != "Want" {
                // A real bed: copy the wish-list row into the garden.
                if let Some(row) = sqlx::query("SELECT * FROM `future_plants` WHERE id = ?").bind(id).fetch_optional(&state.db).await? {
                    // Ask which columns `data_source` has, so only columns that exist in both
                    // tables are copied.
                    let ds_cols: Vec<String> = sqlx::query_scalar(
                        "SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'data_source'",
                    )
                    .fetch_all(&state.db)
                    .await?;
                    let mut ins: Vec<(String, Value)> = Vec::new();
                    if let Value::Object(m) = row_to_json(&row) {
                        for (col, val) in m {
                            if col != "id" && ds_cols.contains(&col) {
                                ins.push((col, val));
                            }
                        }
                    }
                    // The small helper `set` changes a column in the list to copy, or adds it if
                    // missing.
                    let fallback = ins.iter().find(|(c, _)| c == "plant_name").and_then(|(_, v)| v.as_str()).unwrap_or("").to_string();
                    let mut set = |k: &str, v: Value| {
                        if let Some(e) = ins.iter_mut().find(|(c, _)| c == k) {
                            e.1 = v;
                        } else {
                            ins.push((k.to_string(), v));
                        }
                    };
                    set("flower_bed", json!(bed_value));
                    set("garden_beds", json!(bed_value));
                    set("plant_name", json!(if p_name.is_empty() { fallback } else { p_name.clone() }));
                    if let Some(q) = new_qty {
                        set("qty", json!(q));
                    }
                    if let Some(y) = &new_year {
                        set("year_added", json!(y));
                    }
                    if let Some(p) = &picture {
                        set("picture", json!(p));
                    }
                    set("died", json!(0));

                    // Build `INSERT INTO data_source (cols) VALUES (?,?,...)` from the list and
                    // bind each value. The column names come from the database's own list
                    // (`ds_cols`), never from the form.
                    let cols = ins.iter().map(|(c, _)| format!("`{c}`")).collect::<Vec<_>>().join(",");
                    let ph = vec!["?"; ins.len()].join(",");
                    let ins_sql = format!("INSERT INTO `data_source` ({cols}) VALUES ({ph})");
                    let mut q = sqlx::query(&ins_sql);
                    for (_, v) in &ins {
                        q = bind_value(q, v);
                    }
                    // Add the garden row, point any barcode at it, remove the wish-list row: all
                    // three or none (a transaction), so a failure halfway cannot leave the plant
                    // in both lists.
                    let mut tx = state.db.begin().await?;
                    let new_id = q.execute(&mut *tx).await?.last_insert_id() as i64;
                    sqlx::query("UPDATE `plant_barcodes` SET plant_id = ?, source_table = 'data_source' WHERE plant_id = ? AND source_table = 'future_plants'")
                        .bind(new_id)
                        .bind(id)
                        .execute(&mut *tx)
                        .await?;
                    sqlx::query("DELETE FROM `future_plants` WHERE id = ?").bind(id).execute(&mut *tx).await?;
                    tx.commit().await?;
                }
            // Bed is still "Want": only update name, quantity and year in the wish list.
            // `COALESCE(NULLIF(?,''), plant_name)` keeps the old name if the new one is empty.
            } else {
                let mut sql = "UPDATE `future_plants` SET plant_name = COALESCE(NULLIF(?,''), plant_name)".to_string();
                if new_qty.is_some() {
                    sql.push_str(", qty=?");
                }
                if new_year.is_some() {
                    sql.push_str(", year_added=?");
                }
                sql.push_str(" WHERE id = ?");
                let mut q = sqlx::query(&sql).bind(&p_name);
                if let Some(n) = new_qty {
                    q = q.bind(n);
                }
                if let Some(y) = &new_year {
                    q = q.bind(y);
                }
                q.bind(id).execute(&state.db).await?;
            }
            return Ok(());
        }

        // 3b. Editing a garden plant.
        if let Some(id) = edit_id {
            let mut sql = "UPDATE `data_source` SET flower_bed=?, garden_beds=?, plant_name=COALESCE(NULLIF(?,''), plant_name)".to_string();
            let mut extra_binds: Vec<String> = Vec::new();
            if let Some(n) = new_qty {
                sql.push_str(", qty=?");
                extra_binds.push(n.to_string());
            }
            if let Some(y) = &new_year {
                sql.push_str(", year_added=?");
                extra_binds.push(y.clone());
            }
            // With a new photo: it becomes the main picture only if the plant has none, otherwise
            // it joins the gallery.
            if let Some(pic) = &picture {
                let existing = sqlx::query_as::<_, (Option<String>, Option<String>)>("SELECT picture, additional_pictures FROM `data_source` WHERE id = ?")
                    .bind(id)
                    .fetch_optional(&state.db)
                    .await?;
                let (cur_pic, cur_add) = existing.unwrap_or((None, None));
                let mut all_extra: Vec<String> = cur_add
                    .filter(|s| !s.is_empty() && s != "0")
                    .map(|s| s.split(',').map(str::to_string).collect())
                    .unwrap_or_default();
                if cur_pic.as_deref().is_none_or(|p| p.is_empty() || p == "0") {
                    sql.push_str(", picture=?");
                    extra_binds.push(pic.clone());
                    all_extra.extend(additional.iter().cloned());
                } else {
                    all_extra.push(pic.clone());
                    all_extra.extend(additional.iter().cloned());
                }
                if !all_extra.is_empty() {
                    sql.push_str(", additional_pictures=?");
                    extra_binds.push(all_extra.into_iter().filter(|s| !s.is_empty() && s != "0").collect::<Vec<_>>().join(","));
                }
            }
            sql.push_str(" WHERE id=?");
            let mut q = sqlx::query(&sql).bind(&bed_value).bind(&bed_value).bind(&p_name);
            for b in &extra_binds {
                q = q.bind(b);
            }
            q.bind(id).execute(&state.db).await?;
        // 3c. No id: add a brand new plant.
        } else {
            sqlx::query("INSERT INTO `data_source` (plant_name, flower_bed, garden_beds, picture, additional_pictures) VALUES (?, ?, ?, ?, ?)")
                .bind(&p_name)
                .bind(&bed_value)
                .bind(&bed_value)
                .bind(&picture)
                .bind(additional.join(","))
                .execute(&state.db)
                .await?;
        }
        Ok(())
    }
    .await;

    // 4. The cached widget numbers may now be wrong, so clear them.
    state.invalidate_widget_cache();
    // 5. Success redirects to the catalog; failure shows the error text (escaped with `he` so it
    // cannot inject HTML).
    match result {
        Ok(()) => redirect_catalog(),
        Err(e) => (
            StatusCode::OK,
            Html(format!(
                "<h2>⚠️ Error Saving</h2><p>{}</p>",
                crate::php::he(&e.to_string())
            )),
        )
            .into_response(),
    }
}

/// ISO week, month and year for a date string, as PHP's `W`, `n`, `Y`.
fn week_parts(date: &str) -> (i32, i32, i32) {
    use chrono::Datelike;
    let d = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::NaiveDate::parse_from_str(&date_ymd(), "%Y-%m-%d").unwrap());
    (d.iso_week().week() as i32, d.month() as i32, d.year())
}

/// The quantity logger on the plant form.
///
/// Body fields: plant id, event type (purchased, died, removed, other), amount, date, notes. Adds a
/// row to `plant_quantity_log`, changes the plant's `qty` (never below 0), and marks the plant died
/// if the quantity reaches 0 after a death or removal. Gives JSON `{status, delta}`.
async fn log_qty_event_catalog(state: &AppState, visitor: &Visitor, f: &FormData) -> Response {
    let plant_id = php_int(f.str("plant_id"));
    let event_type = f.get("event_type").unwrap_or("other").to_string();
    let qty_amt = php_int(f.str("qty_amt")).max(1);
    let event_date = f
        .get("event_date")
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(date_ymd);
    let notes = f
        .get("notes")
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "0")
        .map(str::to_string);
    let (week, month, year) = week_parts(&event_date);
    let recorded = if visitor.username.is_empty() {
        "catalog".to_string()
    } else {
        visitor.username.clone()
    };
    let delta = if event_type == "purchased" {
        qty_amt
    } else {
        -qty_amt
    };

    // The log row and the quantity change go together or not at all (a transaction).
    let r: Result<(), sqlx::Error> = async {
        let mut tx = state.db.begin().await?;
        sqlx::query(
            "INSERT INTO plant_quantity_log (plant_id, event_type, quantity, event_date, event_week, event_month, event_year, notes, recorded_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(plant_id)
        .bind(&event_type)
        .bind(qty_amt)
        .bind(&event_date)
        .bind(week)
        .bind(month)
        .bind(year)
        .bind(&notes)
        .bind(&recorded)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE `data_source` SET qty = GREATEST(0, qty + ?) WHERE id = ?").bind(delta).bind(plant_id).execute(&mut *tx).await?;
        if event_type == "died" || event_type == "removed" {
            let current: i64 = sqlx::query_scalar::<_, Option<i64>>("SELECT CAST(qty AS SIGNED) FROM `data_source` WHERE id=?")
                .bind(plant_id)
                .fetch_optional(&mut *tx)
                .await?
                .flatten()
                .unwrap_or(1);
            if current == 0 {
                sqlx::query("UPDATE `data_source` SET died=1 WHERE id=?").bind(plant_id).execute(&mut *tx).await?;
            }
        }
        tx.commit().await
    }
    .await;
    state.invalidate_widget_cache();
    match r {
        Ok(()) => json_response(json!({ "status": "ok", "delta": delta })),
        Err(e) => {
            tracing::error!("catalog quantity log: {e}");
            json_response(json!({ "status": "error", "message": "The change could not be saved." }))
        }
    }
}

/// The quantity log in the plant window (`log_qty_event`): snaps the date
/// to the Monday of its week, as the PHP does.
///
/// Admin only. Steps: 1 read and check the fields, 2 move the date back to that week's Monday, 3
/// write the log row, 4 change `qty` (purchased adds, others subtract), 5 mark died if it reached
/// 0. Gives JSON `{status, new_qty, week_of}`.
async fn log_qty_event(state: &AppState, visitor: &Visitor, f: &FormData) -> Response {
    use chrono::Datelike;
    let plant_id = php_int(f.str("plant_id"));
    let event_type = match f.str("event_type") {
        t @ ("purchased" | "died" | "removed" | "other") => t.to_string(),
        _ => "other".to_string(),
    };
    let quantity = php_int(f.str("quantity"));
    let notes: String = f.str("notes").trim().chars().take(500).collect();
    let by = if visitor.username.is_empty() {
        "admin".to_string()
    } else {
        visitor.username.clone()
    };

    // Snap the date to the Monday of its week. A bad date means today.
    // `weekday().number_from_monday()` is 1 for Monday.
    let raw = f.str("event_date");
    let mut d = if raw.len() == 10 {
        chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok()
    } else {
        None
    }
    .unwrap_or_else(|| chrono::NaiveDate::parse_from_str(&date_ymd(), "%Y-%m-%d").unwrap());
    let dow = d.weekday().number_from_monday() as i64;
    if dow > 1 {
        d -= chrono::Duration::days(dow - 1);
    }
    let event_date = d.format("%Y-%m-%d").to_string();
    let (week, month, year) = (d.iso_week().week() as i32, d.month() as i32, d.year());

    // A plant and a non-zero quantity are required.
    if plant_id == 0 || quantity == 0 {
        return json_response(
            json!({ "status": "error", "message": "plant_id and quantity required" }),
        );
    }
    // The log row and the quantity change go together or not at all (a transaction).
    let r: Result<i64, sqlx::Error> = async {
        let mut tx = state.db.begin().await?;
        sqlx::query(
            "INSERT INTO plant_quantity_log (plant_id, event_type, quantity, event_date, event_week, event_month, event_year, notes, recorded_by)
             VALUES (?,?,?,?,?,?,?,?,?)",
        )
        .bind(plant_id)
        .bind(&event_type)
        .bind(quantity)
        .bind(&event_date)
        .bind(week)
        .bind(month)
        .bind(year)
        .bind(if notes.is_empty() { None } else { Some(notes.clone()) })
        .bind(&by)
        .execute(&mut *tx)
        .await?;
        let delta = if event_type == "purchased" { quantity.abs() } else { -quantity.abs() };
        sqlx::query("UPDATE `data_source` SET qty = GREATEST(0, qty + ?) WHERE id = ?").bind(delta).bind(plant_id).execute(&mut *tx).await?;
        let current: i64 = sqlx::query_scalar::<_, Option<i64>>("SELECT CAST(qty AS SIGNED) FROM `data_source` WHERE id = ?")
            .bind(plant_id)
            .fetch_optional(&mut *tx)
            .await?
            .flatten()
            .unwrap_or(0);
        if current == 0 && (event_type == "died" || event_type == "removed") {
            sqlx::query("UPDATE `data_source` SET died = 1 WHERE id = ?").bind(plant_id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(current)
    }
    .await;
    state.invalidate_widget_cache();
    match r {
        Ok(current) => {
            json_response(json!({ "status": "ok", "new_qty": current, "week_of": event_date }))
        }
        Err(e) => {
            tracing::error!("catalog quantity log: {e}");
            json_response(json!({ "status": "error", "message": "The change could not be saved." }))
        }
    }
}

/// `?qty_history=ID` (admin only): the quantity log for one plant, newest first, as JSON.
///
/// `DATE_FORMAT` turns the saved time into text like `2026-10-05 14:30`.
async fn qty_history(state: &AppState, plant_id: i64) -> Response {
    if plant_id == 0 {
        return json_response(json!([]));
    }
    let r = sqlx::query(
        "SELECT id, event_type, quantity, event_date, event_week, event_month, event_year, notes, recorded_by,
                DATE_FORMAT(created_at,'%Y-%m-%d %H:%i') AS logged_at
         FROM plant_quantity_log WHERE plant_id = ? ORDER BY event_date DESC, id DESC",
    )
    .bind(plant_id)
    .fetch_all(&state.db)
    .await;
    match r {
        Ok(rows) => json_response(Value::Array(rows.iter().map(row_to_json).collect())),
        Err(_) => json_response(json!([])),
    }
}

// --- the page ---------------------------------------------------------------

/// The "You have N plants (across M varieties)" line: every live plant's
/// quantity, where a quantity can be a list such as "3+2" or "1,4".
///
/// Returns (total plants, number of varieties). A quantity may be a list like "3+2" or "1,4", so it
/// is split on `,` and `+` and each piece is added up.
async fn plant_stats(state: &AppState) -> (i64, usize) {
    let r = sqlx::query("SELECT CAST(qty AS CHAR) AS qty FROM `data_source` WHERE died = 0 AND flower_bed NOT LIKE '%Dead%'")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    let mut total = 0i64;
    for row in &r {
        let qty = row
            .try_get_unchecked::<Option<String>, _>(0)
            .ok()
            .flatten()
            .unwrap_or_else(|| "0".into());
        for part in qty.split([',', '+']) {
            total += php_int(part.trim());
        }
    }
    (total, r.len())
}

/// Draw the Catalog page itself (the last step when no `?something` matched).
///
/// Fills the template with counts, cache-busting version numbers for the two scripts (their file
/// modified time, so browsers reload them after a change), and whether the visitor is admin. Then
/// wraps it in the site layout and adds the Quill note editor files.
async fn render_page(state: &AppState, visitor: &Visitor) -> Response {
    let (plant_count, variety_count) = plant_stats(state).await;
    let body = CatalogBody {
        plant_count,
        variety_count,
        app_v: filemtime(&state.www, "js/app.js"),
        search_v: filemtime(&state.www, "js/search.js"),
        is_admin: visitor.is_admin(),
        current_user: visitor.username.clone(),
        can_upload: visitor.can_upload(state),
    }
    .render()
    .unwrap_or_default();

    let mut page = Page::new(body);
    page.title = "Jun's and Kona's Garden Catalog".to_string();
    page.show_hero = false;
    // The notes editor and its stylesheet; the PHP busts the cache with the
    // current time on every request.
    let now = common::now_local().unix_timestamp();
    page.extra_head = format!(
        "\n<link href=\"https://cdnjs.cloudflare.com/ajax/libs/quill/1.3.7/quill.snow.min.css\" rel=\"stylesheet\">\n\
         <script src=\"https://cdnjs.cloudflare.com/ajax/libs/quill/1.3.7/quill.min.js\" defer></script>\n\
         <link rel=\"stylesheet\" href=\"user-notes.css?v={now}\">\n"
    );
    Html(layout::render(state, page, visitor).await).into_response()
}

/// Every GET, in the PHP's order.
///
/// One `if` per `?something` on the address; the first match answers and `return` ends the
/// function. Admin-only switches check `admin` first. If nothing matched, the full page is drawn.
async fn dispatch_get(
    state: &AppState,
    visitor: &Visitor,
    q: &HashMap<String, String>,
) -> Response {
    let me = visitor.db_name();
    let admin = visitor.is_admin();
    if q.contains_key("autocomplete") {
        return autocomplete(state, q).await;
    }
    if q.contains_key("letter_counts") {
        return letter_counts(state).await;
    }
    if q.contains_key("qr_tags") {
        return qr_tags(state).await;
    }
    if q.contains_key("ajax_load") {
        return ajax_load(state, q, &me).await;
    }
    if let Some(id) = q.get("toggle_favorite") {
        return toggle(state, "user_favorites", php_int(id), &me).await;
    }
    if let Some(id) = q.get("toggle_like") {
        return toggle(state, "user_likes", php_int(id), &me).await;
    }
    if admin && q.contains_key("rotate_image") {
        return rotate_image(state, q).await;
    }
    if admin && q.contains_key("delete_image") {
        return delete_image(state, q).await;
    }
    if let Some(id) = q.get("get_note") {
        if !admin {
            return json_response(json!({ "note_html": "" }));
        }
        let note = sqlx::query_scalar::<_, Option<String>>(
            "SELECT note_html FROM user_notes WHERE plant_id = ? AND username = ?",
        )
        .bind(php_int(id))
        .bind(&visitor.username)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten()
        .unwrap_or_default();
        return json_response(json!({ "note_html": note }));
    }
    if q.contains_key("notes_index") {
        if !admin {
            return json_response(json!([]));
        }
        let ids = sqlx::query_scalar::<_, i64>(
            "SELECT plant_id FROM user_notes WHERE username = ? AND note_html != ''",
        )
        .bind(&visitor.username)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        return json_response(json!(ids));
    }
    if let Some(id) = q.get("qty_history") {
        if !admin {
            return json_response(json!([]));
        }
        return qty_history(state, php_int(id)).await;
    }
    render_page(state, visitor).await
}

/// Route handler for `GET /catalog.php`.
///
/// axum calls this for every GET. The arguments are "extractors": `State` is the shared app data
/// (`Arc` is a shared pointer so many requests can use one copy), `Extension` is the visitor found
/// by the sign-in layer, and `Query` is the address parameters as a map of text to text. It just
/// hands over to `dispatch_get`.
pub async fn catalog_get(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    dispatch_get(&state, &visitor, &q).await
}

/// Route handler for `POST /catalog.php`: every form and admin action.
///
/// Reads the form body (`formdata::parse` handles both normal and file-upload forms), then checks
/// switches in the PHP's order. Admin-only actions are skipped for anyone not signed in as admin.
/// Notes are saved per signed-in person. If no POST switch matches, it treats the address as a GET
/// switch, as the PHP did.
pub async fn catalog_post(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    OriginalUri(uri): OriginalUri,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let f = formdata::parse(&headers, body).await;
    // Is this visitor an admin? Checked before each admin action.
    let admin = visitor.is_admin();
    let not_logged_in = || json_response(json!({ "status": "error", "message": "Not logged in." }));

    if f.has("logout") {
        return session::logout_redirect(&state, &visitor, uri.path()).await;
    }
    // The PHP checks the GET switches first even on a POST; the upload is a
    // POST with `?upload_gallery=1` on the address.
    if admin && q.contains_key("upload_gallery") {
        return upload_gallery(&state, &visitor, &f).await;
    }
    if admin && f.has("bulk_update_bed") {
        return bulk_update_bed(&state, &f).await;
    }
    if admin && f.has("save_plant") {
        return save_plant(&state, &visitor, &f).await;
    }
    if admin && f.has("log_qty_event_catalog") {
        return log_qty_event_catalog(&state, &visitor, &f).await;
    }
    if admin && f.has("delete_plant") {
        let _ = sqlx::query("DELETE FROM `data_source` WHERE id=?")
            .bind(php_int(f.str("id")))
            .execute(&state.db)
            .await;
        state.invalidate_widget_cache();
        return redirect_catalog();
    }
    if f.has("save_note") {
        if !admin {
            return not_logged_in();
        }
        let r = sqlx::query(
            "INSERT INTO user_notes (plant_id, username, note_html) VALUES (?, ?, ?)
             ON DUPLICATE KEY UPDATE note_html = VALUES(note_html), updated_at = NOW()",
        )
        .bind(php_int(f.str("plant_id")))
        .bind(&visitor.username)
        .bind(f.str("note_html"))
        .execute(&state.db)
        .await;
        return match r {
            Ok(_) => json_response(json!({ "status": "saved" })),
            Err(e) => {
                tracing::error!("catalog note: {e}");
                json_response(
                    json!({ "status": "error", "message": "The note could not be saved." }),
                )
            }
        };
    }
    if f.has("delete_note") {
        if !admin {
            return not_logged_in();
        }
        let _ = sqlx::query("DELETE FROM user_notes WHERE plant_id = ? AND username = ?")
            .bind(php_int(f.str("plant_id")))
            .bind(&visitor.username)
            .execute(&state.db)
            .await;
        return json_response(json!({ "status": "deleted" }));
    }
    if admin && f.has("log_qty_event") {
        return log_qty_event(&state, &visitor, &f).await;
    }
    if admin && f.has("set_died_year") {
        let plant_id = php_int(f.str("plant_id"));
        let year = f
            .get("died_year")
            .filter(|s| !s.is_empty() && *s != "0")
            .map(php_int);
        let _ = sqlx::query("UPDATE `data_source` SET died_year = ? WHERE id = ?")
            .bind(year)
            .bind(plant_id)
            .execute(&state.db)
            .await;
        return json_response(json!({ "status": "ok", "died_year": year }));
    }
    // A GET-style switch sent by POST (the PHP reads $_GET on any method).
    dispatch_get(&state, &visitor, &q).await
}

// Tests. `cargo test` runs each `#[test]` function; `assert_eq!` fails the test if the two sides
// differ.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn community_favourites_exclude_the_three_named_people() {
        let map = vec![
            (1, "Joshua".to_string()),
            (2, "Guest_abc".to_string()),
            (2, "Guest_def".to_string()),
            (3, "Kona".to_string()),
            (4, "Someone".to_string()),
        ];
        assert_eq!(community_ids(&map), vec![2, 4]);
        assert_eq!(ids_for_user(&map, "Joshua"), vec![1]);
    }

    #[test]
    fn filenames_stay_inside_the_photo_folder() {
        assert_eq!(safe_filename("a.webp").as_deref(), Some("a.webp"));
        assert!(safe_filename("../x").is_none());
        assert!(safe_filename("sub/x.webp").is_none());
        assert!(safe_filename(".hidden").is_none());
    }

    #[test]
    fn weeks_snap_like_php() {
        assert_eq!(week_parts("2026-10-05"), (41, 10, 2026));
    }
}
