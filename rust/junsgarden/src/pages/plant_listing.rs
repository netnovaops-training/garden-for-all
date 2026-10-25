//! `plant_listing.php` — Quick Search.
//!
//! One URL, two jobs, exactly like the PHP: with `?ajax_list_search=1` it
//! answers the search box with JSON; otherwise it is the page itself. The
//! response shape is `{"results": [...whole plant rows...]}` and the page's
//! JavaScript depends on every column name in it.
//! HOW TO READ THIS FILE. Top to bottom: `normalise` cleans the search text,
//! `search` runs the database queries and returns JSON, `plant_listing` is the
//! handler that picks between "JSON answer" and "whole page". The tiny test at
//! the bottom checks `normalise`. The printed QR codes and the page's JavaScript
//! call this exact URL, so the name and the JSON shape must not change.

// `use` lines bring names into scope. `Arc` is a shared pointer, `HashMap` is a
// key-value table.
use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Query, State},
    http::header,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde_json::{json, Value};
use sqlx::mysql::MySqlRow;

use crate::layout::{self, Page};
use crate::php::filemtime;
use crate::rowjson::row_to_json;
use crate::session::Visitor;
use crate::AppState;

/// Where plant photos live on the website. Handed to the page's JavaScript.
/// `const` is a value fixed at compile time; `&str` is borrowed text.
const IMG_BASE: &str = "/images/Database/";

/// The page body, filled in from `templates/plant_listing.html`. The struct has
/// no fields because the page has no holes to fill; `#[derive(Template)]` is
/// askama writing the rendering code for us at compile time.
#[derive(Template)]
#[template(path = "plant_listing.html")]
struct ListingBody;

/// `preg_replace("/[^a-zA-Z0-9\s]/", " ")` then collapse runs of whitespace
/// and trim, so "Lil Kim" finds "Lil' Kim". PHP works on bytes here, which
/// gives the same result once the runs are collapsed.
/// Input: the raw text typed in the search box (`&str`, borrowed text).
/// Output: a new owned `String` with only letters, digits and single spaces.
fn normalise(raw: &str) -> String {
    let spaced: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c.is_ascii_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();
    spaced
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Search the plant tables and build the JSON the search box needs.
/// Takes the app state (it holds the database pool) and the typed words.
/// Gives back a `serde_json::Value` shaped like `{"results": [...]}`.
/// It returns a result even when the database fails (empty list), as the PHP did.
async fn search(state: &AppState, term_raw: &str) -> Value {
    // 1. Clean the typed words. `trim` removes spaces at both ends.
    let raw = term_raw.trim();
    let norm = normalise(raw);
    // 2. Build the LIKE pattern. `%` means "anything here", so `%rose%` matches any
    // text containing "rose". If cleaning left nothing, use the raw text.
    let term = format!("%{}%", if norm.is_empty() { raw } else { norm.as_str() });

    // 3. The SQL below. Each `?` is a placeholder; the value is bound separately
    // by sqlx, never glued into the text, so typing quotes cannot break the query.
    // REGEXP_REPLACE on name and family so punctuation stored in the
    // database is ignored too, same as the PHP.
    let sql = "SELECT * FROM `data_source`
        WHERE REGEXP_REPLACE(REGEXP_REPLACE(plant_name, '[^a-zA-Z0-9 ]', ' '), ' +', ' ') LIKE ?
           OR REGEXP_REPLACE(REGEXP_REPLACE(family,     '[^a-zA-Z0-9 ]', ' '), ' +', ' ') LIKE ?
           OR flower_bed LIKE ? OR company LIKE ?
           OR zones LIKE ? OR water_needs LIKE ?
           OR deer_resistance LIKE ? OR year_planted LIKE ?
        ORDER BY plant_name ASC LIMIT 50";

    // 4. Bind the same pattern to all 8 placeholders. There are 8 `?` in the SQL.
    let mut q = sqlx::query(sql);
    for _ in 0..8 {
        q = q.bind(&term);
    }
    // 5. Run the query. `.await` waits for the database. A `match` handles both
    // results: on error we log it and carry on with an empty list.
    let rows: Vec<MySqlRow> = match q.fetch_all(&state.db).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("plant search failed: {e}");
            Vec::new()
        }
    };
    // 6. Turn each database row into JSON (`row_to_json` mimics PHP's json_encode).
    let mut results: Vec<Value> = rows.iter().map(row_to_json).collect();

    // 7. Also search the wish-list table (limit 20). Each hit is tagged with
    // `source_table` so the page can tell it from an owned plant.
    // The plants Jun does not own yet. The table may not exist; the PHP
    // swallows that, and so does this.
    let future = sqlx::query(
        "SELECT * FROM `future_plants`
         WHERE plant_name LIKE ? OR botanical_name LIKE ? OR company LIKE ?
         ORDER BY plant_name ASC LIMIT 20",
    )
    .bind(&term)
    .bind(&term)
    .bind(&term)
    .fetch_all(&state.db)
    .await;
    // `if let Ok(rows) = future` runs only if the query worked; a failure
    // (for example a missing table) is skipped on purpose.
    if let Ok(rows) = future {
        for row in &rows {
            let mut v = row_to_json(row);
            if let Value::Object(m) = &mut v {
                m.insert("source_table".to_string(), json!("future_plants"));
            }
            results.push(v);
        }
    }
    json!({ "results": results })
}

/// Handler for `/plant_listing.php`. Takes the shared state, who is asking, and
/// the URL's query string as a map (`?term=rose&ajax_list_search=1` becomes
/// {"term": "rose", "ajax_list_search": "1"}). Gives back JSON or the full page.
pub async fn plant_listing(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // Case 1: the search box's JavaScript asked for data. Answer with JSON only.
    // `.map(String::as_str).unwrap_or("")` turns "maybe a String" into text,
    // using an empty text when the key is missing.
    if params.contains_key("ajax_list_search") {
        let term = params.get("term").map(String::as_str).unwrap_or("");
        let body = search(&state, term).await;
        return (
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&body).unwrap_or_else(|_| "{\"results\":[]}".into()),
        )
            .into_response();
    }

    // Case 2: a normal visit. Render the template, then wrap it in the shared
    // page frame (header, footer) via `layout::render`.
    let body = ListingBody.render().unwrap_or_default();
    let mut page = Page::new(body);
    // The settings below tell the frame what is special about this page.
    page.title = "Quick Search - Jun's Garden".to_string();
    page.show_hero = false;
    page.body_class = "plant-listing-page".to_string();
    // Load the page's script. `?v=` plus the file's modified time makes browsers
    // fetch a fresh copy after the file changes.
    page.extra_head = format!(
        "<script>window.PLANT_LISTING_CONFIG={{imgBase:\"{IMG_BASE}\"}};</script>\
         <script type=\"module\" src=\"js/plant-listing.js?v={}\"></script>",
        filemtime(&state.www, "js/plant-listing.js")
    );
    page.after_header = "\n<div class=\"qs-search-bar\">\n    <input type=\"text\" id=\"list-search-box\" class=\"qs-search-input\" placeholder=\"Quick search plants...\" autofocus>\n</div>".to_string();
    Html(layout::render(&state, page, &visitor).await).into_response()
}

// Tests run only with `cargo test`; they are not part of the live site.
#[cfg(test)]
mod tests {
    use super::normalise;

    // This test feeds `normalise` messy text and checks the cleaned result.
    #[test]
    fn punctuation_is_ignored() {
        assert_eq!(normalise("Lil' Kim"), "Lil Kim");
        assert_eq!(normalise("  a   b  "), "a b");
        assert_eq!(normalise("!!!"), "");
    }
}
