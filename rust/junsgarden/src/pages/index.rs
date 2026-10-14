//! `index.php` — the home page: weather box, tip cards, the three newest
//! garden posts, and the guestbook. Also `sections/cards/index.php`, the
//! endpoint the tip cards load from, and `/sso-login`, an old address that
//! still shows the home page (the household sign-in it served is gone).
//!
//! Signed in, the page also lets you write, edit and delete posts (photos
//! are converted to WebP as they arrive) and delete guestbook lines.
//! URLs: `GET /` and `GET /index.php` (the page), `POST /index.php` (guestbook and
//! the admin buttons), `GET /sections/cards/index.php` (the tip cards, JSON), and
//! `/sso-login`, an old address kept so old bookmarks still show the home page.
//!
//! HOW TO READ THIS FILE. Top: constants and the data shapes the templates use.
//! Middle: small helpers for the guestbook text file (strip_tags, read_comments,
//! add_comment, delete_comment) and for building post cards. Then `render_home`
//! (builds the page), `home` (GET handler), `handle_post` and `home_post` (POST),
//! `cards` (JSON for the tip cards), and tests.

// `use` lines bring names into scope. `HashMap` is a key-value table. `Arc` is a
// shared pointer: all requests share one AppState (settings plus database pool).
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
use rand::Rng;
use serde_json::Value;

use crate::formdata::{self, FormData};
use crate::images;
use crate::layout::{self, Page};
use crate::pages::blog::{post_date, PostRow};
use crate::php::{filemtime, he, htmlspecialchars_decode, nl2br, php_int, urlencode};
use crate::session::{self, Visitor};
use crate::AppState;

/// Where uploaded post photos are served from, as a web address.
const IMG_BASE: &str = "/images/post/";
/// The photo shown when a post has no picture of its own.
const NO_PICTURE: &str = "https://images.unsplash.com/photo-1466692476868-aef1dfb1e735";

/// The form's own limits (`maxlength` on the two inputs). The PHP only
/// enforced them in the browser, so anyone could post a megabyte; the
/// server now holds the line the form already draws. See DEVIATIONS.md.
// `const` values are fixed at compile time. `usize` is an unsigned whole number
// used for counts and sizes.
const MAX_NAME_CHARS: usize = 20;
const MAX_MESSAGE_CHARS: usize = 100;

/// One blog post as the home page's template shows it. Every field is already
/// text ready to print, so the template needs no logic. `Vec<T>` is a list.
struct PostCard {
    id: i32,
    theme: String,
    images: Vec<String>,
    title: String,
    date: String,
    author: String,
    content_html: String,
    /// The stored text, for the Edit button's `data-content`.
    content_raw: String,
    share_url: String,
    share_text: String,
}

/// The weather box template (`templates/weather.html`). `weather_v` is the
/// script file's modified time, used to make browsers fetch a fresh copy.
#[derive(Template)]
#[template(path = "weather.html")]
struct WeatherBody {
    theme: String,
    zip: String,
    weather_v: u64,
}

/// The whole home page template (`templates/index.html`). `#[derive(Template)]`
/// is askama writing the code that fills the HTML holes, checked at compile time.
#[derive(Template)]
#[template(path = "index.html")]
struct IndexBody {
    page_theme: String,
    weather: String,
    posts: Vec<PostCard>,
    comments: Vec<String>,
    home_blog_v: u64,
    home_v: u64,
    is_admin: bool,
    current_user: String,
}

/// Pick one of six colour themes at random (`bg-theme-0` to `bg-theme-5`).
/// Takes nothing. Gives back a CSS class name as a `String`. Rust's `format!`
/// builds text from a pattern, like PHP string interpolation.
fn random_theme() -> String {
    format!("bg-theme-{}", rand::thread_rng().gen_range(0..=5))
}

/// `strip_tags()` for the guestbook: drop anything that looks like a tag.
/// The result is escaped afterwards, so this is about not storing markup,
/// not about safety by itself.
/// Takes text, gives back text with `<...>` tags removed. How it works: copy
/// each character unless we are inside a tag. `chars.get(i + 1)` looks at the next
/// character (an `Option`, maybe nothing); `map_or(true, ..)` says "if there is no
/// next character, or it is whitespace, this is not a tag".
fn strip_tags(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    // `for (i, &c) in ... .enumerate()` gives each character with its position.
    for (i, &c) in chars.iter().enumerate() {
        // `match` tests `c` against each pattern in order; `if` after a pattern is an
        // extra condition. Arms: start a tag, end a tag, keep the character, or skip it.
        match c {
            // Like PHP, a "<" followed by a space is just a less-than sign.
            '<' if !in_tag && !chars.get(i + 1).map_or(true, |n| n.is_whitespace()) => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// PHP's `empty()` on a string: nothing, or exactly "0".
/// Takes `&str` (borrowed text). Gives back true or false.
fn php_empty(s: &str) -> bool {
    s.is_empty() || s == "0"
}

/// Path of the guestbook text file inside the data folder. Takes the app state.
/// Gives back a `PathBuf` (an owned file path).
fn comments_path(state: &AppState) -> std::path::PathBuf {
    state.data.join("comments.txt")
}

/// The guestbook lines, newest first, at most 20, blanks skipped.
/// Takes the app state. Gives back up to 20 lines as a list of `String`.
/// A missing file gives an empty list: `unwrap_or_default` means "use the empty
/// value if there was an error".
fn read_comments(state: &AppState) -> Vec<String> {
    std::fs::read_to_string(comments_path(state))
        .map(|t| t.lines().filter(|l| !l.trim().is_empty()).take(20).map(str::to_string).collect())
        .unwrap_or_default()
}

/// Add one guestbook line at the top of the file, in the exact format the
/// PHP wrote: `<span class='c-date'>Oct 5</span> <strong>Name:</strong> text`.
/// Takes the visitor's name and message (borrowed text). Gives back nothing.
/// `async fn` can wait without blocking others; `.await` marks each wait.
async fn add_comment(state: &AppState, name: &str, message: &str) {
    // 1. Strip tags, cut to the form's length limit (counted in characters, not
    // bytes), then escape with `he` (like PHP's htmlspecialchars).
    let name = he(&strip_tags(name).chars().take(MAX_NAME_CHARS).collect::<String>());
    let msg = he(&strip_tags(message).chars().take(MAX_MESSAGE_CHARS).collect::<String>());
    // 2. An empty name becomes "Guest".
    let name = if php_empty(&name) { "Guest".to_string() } else { name };
    // 3. Today's date as text such as "Oct 5". `common::now_local` is the server's
    // local time.
    let date = common::now_local()
        .format(&time::macros::format_description!("[month repr:short] [day padding:none]"))
        .unwrap_or_default();
    // 4. The one line to store, in the exact HTML format the PHP wrote.
    let entry = format!("<span class='c-date'>{date}</span> <strong>{name}:</strong> {msg}\n");

    // 5. Take the lock so two visitors writing at once cannot lose a line. It is
    // released when `_guard` goes out of scope at the end of the function.
    let _guard = state.comments_lock.lock().await;
    // 6. Read the old file, then write the new line followed by the old lines.
    // `{entry}{existing}` is text interpolation. New lines go at the top.
    let path = comments_path(state);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if let Err(e) = std::fs::write(&path, format!("{entry}{existing}")) {
        tracing::error!("could not save guestbook entry to {}: {e}", path.display());
    }
}

/// The admin's delete button: remove line N (counting non-blank lines, as
/// PHP's `file()` with FILE_SKIP_EMPTY_LINES does).
/// Takes the app state and the line number to remove (0 is the first line).
async fn delete_comment(state: &AppState, index: i64) {
    // 1. Lock, then read the file. `let Ok(text) = ... else { return }` leaves the
    // function quietly if the file cannot be read.
    let _guard = state.comments_lock.lock().await;
    let path = comments_path(state);
    let Ok(text) = std::fs::read_to_string(&path) else { return };
    // 2. Collect the non-blank lines (blank lines do not count, like the PHP).
    let mut lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    // 3. Reject a number that is negative or beyond the last line.
    if index < 0 || index as usize >= lines.len() {
        return;
    }
    // 4. Remove that line and write the rest back (an empty file when none remain).
    lines.remove(index as usize);
    let out = if lines.is_empty() { String::new() } else { format!("{}\n", lines.join("\n")) };
    let _ = std::fs::write(&path, out);
}

/// Turn database rows into the cards the template prints.
/// Takes a slice of rows (`&[PostRow]`: a borrowed list). Gives back a list of
/// `PostCard`. The position `i` picks each card's colour theme (0 to 5, repeating).
fn post_cards(rows: &[PostRow]) -> Vec<PostCard> {
    rows.iter()
        .enumerate()
        .map(|(i, p)| {
            // Photos: a comma list. Full web addresses pass through; bare file names are
            // placed under `/images/post/`. No photo means the stand-in picture.
            let images = match p.picture.as_deref().filter(|s| !s.is_empty()) {
                Some(pic) => pic
                    .split(',')
                    .map(|img| if img.starts_with("http") { img.to_string() } else { format!("{IMG_BASE}{}", img.trim()) })
                    .collect(),
                None => vec![NO_PICTURE.to_string()],
            };
            // "Joshua" and an empty author are both shown as "Jun" (kept from the PHP).
            let author = match p.author.as_deref() {
                Some("Joshua") | None => "Jun".to_string(),
                Some(a) => a.to_string(),
            };
            PostCard {
                id: p.id,
                theme: format!("bg-theme-{}", i % 6),
                images,
                title: p.title.clone(),
                date: post_date(&p.created_at),
                author,
                // The PHP prints the stored text with its HTML entities
                // decoded and NOT re-escaped, so anything typed into a post
                // would run as HTML. Decoding and then escaping shows the
                // same text for every real post and closes that hole.
                content_html: nl2br(&he(&htmlspecialchars_decode(&p.content))),
                content_raw: p.content.clone(),
                share_url: urlencode("https://junsgarden.zapto.org/"),
                share_text: urlencode(&format!("Check out this update from Jun's Garden: {}", p.title)),
            }
        })
        .collect()
}

/// Build the whole home page.
/// Takes the app state, the visitor and the `zip` text from `?zip=` (for weather).
/// Gives back the page as HTML.
async fn render_home(state: &AppState, visitor: &Visitor, zip: &str) -> Html<String> {
    // 1. Read the 3 newest posts. On a database error, log it and show none.
    let rows = sqlx::query_as::<_, PostRow>(
        "SELECT id, author, title, content, picture, created_at
         FROM garden_blog ORDER BY created_at DESC LIMIT 3",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_else(|e| {
        tracing::error!("home page posts query failed: {e}");
        Vec::new()
    });

    // `sections/weather.php`: only a five-digit zip is accepted.
    // 2. Weather box. Only a five-digit zip is accepted (letters and long text are
    // dropped); anything else becomes an empty zip.
    let zip = if zip.len() == 5 && zip.bytes().all(|b| b.is_ascii_digit()) { zip } else { "" };
    let weather = WeatherBody {
        theme: random_theme(),
        zip: zip.to_string(),
        weather_v: filemtime(&state.www, "js/weather.js"),
    }
    .render()
    .unwrap_or_default();

    // 3. Fill the home template. The `*_v` numbers are file modified times used in
    // `?v=` on script addresses, so browsers fetch fresh scripts after edits.
    let body = IndexBody {
        page_theme: random_theme(),
        weather,
        posts: post_cards(&rows),
        comments: read_comments(state),
        home_blog_v: filemtime(&state.www, "js/home-blog.js"),
        home_v: filemtime(&state.www, "js/home.js"),
        is_admin: visitor.is_admin(),
        current_user: visitor.username.clone(),
    }
    .render()
    .unwrap_or_default();

    // 4. Wrap the body in the shared page frame (header, footer, hero) and send it.
    let mut page = Page::new(body);
    page.title = "Jun's and Kona's Garden Dashboard".to_string();
    Html(layout::render(state, page, visitor))
}

/// Handler for `GET /` and `GET /index.php`. Takes the shared state, the visitor
/// and the query string (`?zip=12345`). Gives back the page. The work is in
/// `render_home`.
pub async fn home(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
) -> Html<String> {
    render_home(&state, &visitor, q.get("zip").map(String::as_str).unwrap_or("")).await
}

/// Save the photos attached to a post; returns the new file names.
/// Takes the app state and the uploaded files. Gives back the new file names.
/// Each photo is converted to WebP at quality 82 and saved under `images/post`.
/// `filter_map` keeps only the ones that saved successfully.
fn save_post_images(state: &AppState, files: &[&formdata::Upload]) -> Vec<String> {
    let dir = state.www.join("images/post");
    files
        .iter()
        .filter_map(|u| images::save_as_webp(&u.bytes, &dir, 82.0))
        .collect()
}

/// Everything the home page accepts by POST: the guestbook (anyone), and
/// for a signed-in person: new post, edit post, delete post, delete a
/// guestbook line, Logout. Same order of checks as the PHP.
/// Takes the state, the visitor, the parsed form and the page path. Gives back
/// `Some(response)` when it handled the form (usually a redirect), or `None`
/// when the form was not one of these buttons so the caller carries on.
async fn handle_post(state: &AppState, visitor: &Visitor, f: &FormData, path: &str) -> Option<Response> {
    // `redirect` is a closure (a small function stored in a variable) that makes a
    // 302 redirect response to the given address.
    let redirect = |to: &str| Some((StatusCode::FOUND, [(header::LOCATION, to.to_string())]).into_response());

    // 1. Logout button: works for anyone.
    if f.has("logout") {
        return Some(session::logout_redirect(state, visitor, path).await);
    }
    // 2. Everything below is for signed-in people only. A guest gets `None`.
    if !visitor.is_admin() {
        return None;
    }

    // 3. New post: escape the text, convert and save photos, then INSERT a row.
    // The `?` marks in the SQL are filled by `.bind(...)` so typed text can never
    // change the meaning of the query.
    if f.has("post_blog") {
        let title = he(f.str("title"));
        let content = he(f.str("content"));
        let uploaded = save_post_images(state, &f.files_for("blog_images[]"));
        let picture = if uploaded.is_empty() { None } else { Some(uploaded.join(",")) };
        // Posts by "Joshua" are stored as "Jun" (kept from the PHP).
        // Joshua posts as Jun.
        let author = if visitor.username == "Joshua" { "Jun".to_string() } else { visitor.username.clone() };
        let r = sqlx::query("INSERT INTO garden_blog (author, title, content, picture) VALUES (?, ?, ?, ?)")
            .bind(author)
            .bind(title)
            .bind(content)
            .bind(picture)
            .execute(&state.db)
            .await;
        if let Err(e) = r {
            tracing::error!("post_blog failed: {e}");
        }
        // Redirect so refreshing the page does not post again.
        return redirect("index.php");
    }

    // 4. Edit post: update title and text. If new photos were attached they
    // replace the old ones; otherwise the photos are left alone.
    if f.has("edit_blog") {
        let id = php_int(f.str("blog_id"));
        let title = he(f.str("title"));
        let content = he(f.str("content"));
        let files = f.files_for("edit_blog_images[]");
        let r = if !files.is_empty() {
            let uploaded = save_post_images(state, &files);
            sqlx::query("UPDATE garden_blog SET title=?, content=?, picture=? WHERE id=?")
                .bind(title)
                .bind(content)
                .bind(uploaded.join(","))
                .bind(id)
                .execute(&state.db)
                .await
        } else {
            sqlx::query("UPDATE garden_blog SET title=?, content=? WHERE id=?")
                .bind(title)
                .bind(content)
                .bind(id)
                .execute(&state.db)
                .await
        };
        if let Err(e) = r {
            tracing::error!("edit_blog failed: {e}");
        }
        return redirect("index.php");
    }

    // 5. Delete post by id.
    if f.has("delete_blog") {
        let id = php_int(f.str("blog_id"));
        let _ = sqlx::query("DELETE FROM garden_blog WHERE id=?").bind(id).execute(&state.db).await;
        return redirect("index.php");
    }

    // 6. Delete one guestbook line by its position.
    if f.has("delete_comment") {
        delete_comment(state, php_int(f.str("comment_index"))).await;
        return redirect("index.php#guestbook");
    }
    // Not one of these buttons: tell the caller to carry on.
    None
}

/// Handler for `POST /index.php`. Takes the state, visitor, original address,
/// query string, headers and raw body. Gives back a redirect or the page again.
pub async fn home_post(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    OriginalUri(uri): OriginalUri,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // 1. Read the form. 2. Try the admin buttons. If one handled it, we are done.
    let f = formdata::parse(&headers, body).await;
    if let Some(r) = handle_post(&state, &visitor, &f, uri.path()).await {
        return r;
    }
    // 3. Otherwise it may be a guestbook message. Anyone can leave one.
    let message = f.str("user_message");
    if !php_empty(message) {
        add_comment(&state, f.str("user_name"), message).await;
    }
    // 4. Show the home page again (with the new guestbook line).
    render_home(&state, &visitor, q.get("zip").map(String::as_str).unwrap_or("")).await.into_response()
}

/// `sections/cards/index.php`: every `card-*.json`, as one array, sorted by
/// each card's `id`. The tip cards on the home page load this.
pub async fn cards(State(state): State<Arc<AppState>>) -> Response {
    // 1. Find the cards folder. 2. Read each `card-*.json` file. 3. Keep only
    // files that hold a JSON object. 4. Sort by each card's `id` text.
    // 5. Send as JSON, cached by browsers for 5 minutes (`max-age=300`).
    // `if let Ok(x) = ...` runs only when the step succeeded; a bad file is skipped.
    let dir = state.www.join("sections/cards");
    let mut cards: Vec<Value> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut files: Vec<_> = rd
            .flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("card-") && n.ends_with(".json")
            })
            .collect();
        files.sort_by_key(|e| e.file_name());
        for f in files {
            if let Ok(raw) = std::fs::read(f.path()) {
                if let Ok(v @ Value::Object(_)) = serde_json::from_slice::<Value>(&raw) {
                    cards.push(v);
                }
            }
        }
    }
    // A closure that reads a card's `id` as text, or "" if it has none.
    let id = |v: &Value| v.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    cards.sort_by_key(id);
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=300"),
        ],
        serde_json::to_string(&cards).unwrap_or_else(|_| "[]".to_string()),
    )
        .into_response()
}

// Tests run only with `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    // Checks that tags are removed but a lone "<" followed by a space is kept.
    #[test]
    fn strip_tags_removes_markup() {
        assert_eq!(strip_tags("hi <b>there</b> <script>x</script>"), "hi there x");
        assert_eq!(strip_tags("a < b"), "a < b");
    }

    // Checks that "" and "0" count as empty but "00" does not.
    #[test]
    fn php_empty_treats_zero_as_empty() {
        assert!(php_empty(""));
        assert!(php_empty("0"));
        assert!(!php_empty("00"));
    }
}
