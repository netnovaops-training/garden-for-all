//! `gallery.php` — the photo gallery.
//!
//! The page is a big self-contained block of styles and script (copied
//! as-is into `templates/gallery_head.html` and `gallery_body.html`); the
//! only data it needs is the list of photos, which the PHP read from
//! `images/Gallery/tags_cache.csv` (written offline by
//! `images/Gallery/build_gallery_cache.py`) and shuffled on every load.
//! Two values are injected: the photo count and that list as JSON.
//!
//! Not ported: the PHP's fallback for when the CSV is missing (a directory
//! scan with no tags). In production the CSV exists; if it does not, the
//! gallery shows no photos and says why in the log.
//! URL: `GET /gallery.php`. It has no POST.
//!
//! HOW TO READ THIS FILE. `rawurlencode` and `parse_csv` are small helpers,
//! `load_photos` reads the CSV and builds one JSON object per photo, and
//! `gallery` is the handler: load, shuffle, render, send. Tests are at the end.

// `use` lines bring names into scope. `Arc` is a shared pointer to one value.
use std::sync::Arc;

use crate::session::Visitor;
use askama::Template;
use axum::{extract::State, response::Html, Extension};
use rand::seq::SliceRandom;
use serde_json::{json, Value};

use crate::AppState;

/// Folder (as a web address) where gallery photos are served from.
const WEB_BASE: &str = "images/Gallery/";

/// The gallery body template. `total` is the photo count and `photos_json` is
/// the whole list as JSON text that the page's script reads.
#[derive(Template)]
#[template(path = "gallery_body.html")]
struct GalleryBody {
    total: usize,
    photos_json: String,
}

/// `rawurlencode()`: percent-encode everything except `A-Za-z0-9-_.~`.
/// Takes `&str` (borrowed text). Gives back a new `String`. Needed so file names
/// with spaces or accents work inside a web address.
fn rawurlencode(s: &str) -> String {
    // Go through the text byte by byte. Safe characters are kept; every other byte
    // becomes `%` plus two hex digits (for example a space becomes `%20`).
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Parse CSV the way PHP's `fgetcsv` reads what `fputcsv` writes: comma
/// separated, fields optionally in double quotes, `""` inside quotes for a
/// quote, and newlines allowed inside quotes. A blank line is one empty row.
/// Takes the whole CSV file as text. Gives back rows, each a list of fields
/// (a `Vec` is a growable list). `peekable` lets us look at the next character
/// without using it, needed to tell `""` (a quote inside quotes) from the end of
/// a quoted field. `std::mem::take` moves a value out and leaves an empty one.
pub fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();
    let mut any = false;
    // Walk the text one character at a time. Inside quotes, `,` and newlines are
    // ordinary text. Outside quotes, `,` ends a field and a newline ends a row.
    while let Some(c) = chars.next() {
        any = true;
        if in_quotes {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => in_quotes = false,
                _ => field.push(c),
            }
        } else {
            match c {
                '"' => in_quotes = true,
                ',' => row.push(std::mem::take(&mut field)),
                '\r' => {}
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    any = false;
                }
                _ => field.push(c),
            }
        }
    }
    // After the last character: keep a final row that had no trailing newline.
    if any || !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// The photos, in file order, as the page's script wants them.
/// Takes the site folder. Gives back a list of JSON values (`Vec<Value>`).
/// On a missing or old-format file it logs the problem and returns an empty list,
/// so the page loads with no photos instead of crashing.
fn load_photos(www: &std::path::Path) -> Vec<Value> {
    let path = www.join("images/Gallery/tags_cache.csv");
    // 1. Read the whole CSV file. `match` handles success (`Ok`) and failure (`Err`).
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(
                "gallery: cannot read {}: {e}. Run build_gallery_cache.py",
                path.display()
            );
            return Vec::new();
        }
    };
    // 2. The first row is the header. It must have 9 columns (the last two are
    // width and height); an older file without them is rejected.
    let mut rows = parse_csv(&text).into_iter();
    match rows.next() {
        Some(h) if h.len() >= 9 => {}
        _ => {
            tracing::error!(
                "gallery: {} has no width/height columns. Run build_gallery_cache.py",
                path.display()
            );
            return Vec::new();
        }
    }
    let mut photos = Vec::new();
    // 3. Each later row is one photo. Columns: 0 file name, 1 display name,
    // 2 to 4 thumbnails (web, tablet, mobile), 5 download name, 6 tags separated by
    // `|`, 7 width, 8 height. Rows with fewer than 7 columns are skipped.
    for row in rows {
        if row.len() < 7 {
            continue;
        }
        // `get` reads a column by number, giving "" when it is absent.
        let get = |i: usize| row.get(i).cloned().unwrap_or_default();
        let (fn_, name, t_web, t_tab, t_mob, dl, tag_str) =
            (get(0), get(1), get(2), get(3), get(4), get(5), get(6));
        // The type comes from the file extension: video, gif, or image.
        let ext = fn_
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        let kind = if ["mp4", "mov", "webm", "m4v"].contains(&ext.as_str()) {
            "video"
        } else if ext == "gif" {
            "gif"
        } else {
            "image"
        };
        // `php_int` here reads a number from a column, 0 when empty or not a number.
        // Size 0 would break the layout, so use a default (video 16:9, photo 4:3).
        let php_int = |i: usize| {
            row.get(i)
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0)
        };
        let (mut w, mut h) = (php_int(7), php_int(8));
        if w == 0 || h == 0 {
            (w, h) = if kind == "video" {
                (1920, 1080)
            } else {
                (800, 600)
            };
        }
        // Build the web address of the photo, then the tag list (dropping empty and "0").
        let src = format!("{WEB_BASE}{}", rawurlencode(&fn_));
        // A thumbnail value of "" or "0" means "none"; JSON then carries `null`.
        let non_empty = |s: String| {
            if s.is_empty() || s == "0" {
                None
            } else {
                Some(s)
            }
        };
        let tags: Vec<&str> = tag_str
            .split('|')
            .filter(|t| !t.is_empty() && *t != "0")
            .collect();
        // 4. Build the JSON object the gallery script reads. Key names are the ones the
        // page's JavaScript expects, so they must not change.
        photos.push(json!({
            "fn": fn_,
            "name": name,
            "src": src,
            "thumb": non_empty(t_web).unwrap_or_else(|| src.clone()),
            "thumbTab": non_empty(t_tab),
            "thumbMob": non_empty(t_mob),
            "dl": dl,
            "tags": tags,
            "type": kind,
            "w": w,
            "h": h,
        }));
    }
    photos
}

/// Handler for `GET /gallery.php`. Takes the shared state and the visitor.
/// Gives back the finished HTML page.
/// Steps: load the photos (the CSV is parsed once and kept until it changes, see
/// `store::Parsed`), shuffle them (a new order on every visit, as the PHP did), render the
/// template with the count and the JSON, then wrap it in the page frame. `include_str!`
/// pastes a file's text into the program when it is built; here it is the page's style and
/// script block.
pub async fn gallery(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
) -> Html<String> {
    // The page builder draws the layout; the default one is this page exactly as before.
    Html(crate::pagebuilder::render_page(&state, &visitor, "gallery", "").await)
}

/// The gallery's style and script block for `<head>` (the section's head part).
pub(crate) const GALLERY_HEAD: &str = include_str!("../../templates/gallery_head.html");

/// Section `gallery.wall`: the photo wall's HTML (its `<head>` part is `GALLERY_HEAD`).
pub(crate) async fn section_wall(state: &Arc<AppState>) -> String {
    let st = state.clone();
    let mut photos: Vec<Value> = crate::guard::off_thread(move || {
        let www = st.www.clone();
        st.gallery
            .get(&www.join("images/Gallery/tags_cache.csv"), |_| {
                load_photos(&www)
            })
            .as_ref()
            .clone()
    })
    .await;
    photos.shuffle(&mut rand::rng());
    GalleryBody {
        total: photos.len(),
        photos_json: serde_json::to_string(&photos).unwrap_or_else(|_| "[]".to_string()),
    }
    .render()
    .unwrap_or_default()
}

// Tests run only with `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    // Checks quoted fields, commas inside quotes, doubled quotes and empty fields.
    #[test]
    fn csv_handles_quotes_commas_and_blank_fields() {
        let rows = parse_csv("a,b,c\n\"x, y\",\"he said \"\"hi\"\"\",\n1,2,3\n");
        assert_eq!(rows[0], vec!["a", "b", "c"]);
        assert_eq!(rows[1], vec!["x, y", "he said \"hi\"", ""]);
        assert_eq!(rows[2], vec!["1", "2", "3"]);
        assert_eq!(rows.len(), 3);
    }

    // Checks the encoding matches PHP's `rawurlencode`, including accented letters.
    #[test]
    fn rawurlencode_matches_php() {
        assert_eq!(rawurlencode("a b+c.webp"), "a%20b%2Bc.webp");
        assert_eq!(rawurlencode("Zoë~.jpg"), "Zo%C3%AB~.jpg");
    }
}
