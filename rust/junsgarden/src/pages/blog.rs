//! `blog.php` — the complete garden archive.
//!
//! Visitors read it; a signed-in person also gets a Delete Post button on
//! each post and the Logout button works here too, as in the PHP.
//! URLs: `GET /blog.php` (the page) and `POST /blog.php` (Delete Post, Logout).
//! The posts are rows of the `garden_blog` table in the database.
//!
//! HOW TO READ THIS FILE. Types first (`PostRow` = a database row, `PostView` =
//! what the template shows), then small formatting helpers, then `render` which
//! builds the page, then the two handlers `blog` (GET) and `blog_post` (POST).

// `use` lines bring names into scope. `Arc` is a shared pointer to one value,
// here the app's settings and database pool.
use std::sync::Arc;

use askama::Template;
use axum::{
    body::Bytes,
    extract::{OriginalUri, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Extension,
};

use crate::formdata;
use crate::php::{he, nl2br, php_int, urlencode};
use crate::session::{self, Visitor};
use crate::AppState;

// `const` values are fixed at compile time. `&str` is borrowed text.
/// The photo that stands in when a post has none.
const NO_PICTURE: &str = "https://images.unsplash.com/photo-1466692476868-aef1dfb1e735";
/// Folder (as a web address) where uploaded post photos are stored.
const IMG_BASE: &str = "/images/post/";

/// One row of the `garden_blog` table. `#[derive(sqlx::FromRow)]` makes sqlx
/// fill this struct from a database row by matching column names to field names.
/// `Option<T>` means the column may be empty (NULL in SQL). `pub` makes a field
/// or item usable from other files.
#[derive(sqlx::FromRow)]
pub struct PostRow {
    pub id: i32,
    pub author: Option<String>,
    pub title: String,
    pub content: String,
    pub picture: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

/// `date("F j, Y", strtotime($created_at))`.
/// Takes the date as an `Option` (maybe nothing). Gives back the text, such as
/// "October 5, 2026". `match` picks one branch depending on whether a date exists.
pub fn post_date(created: &Option<chrono::NaiveDateTime>) -> String {
    match created {
        Some(d) => d.format("%B %-d, %Y").to_string(),
        // strtotime(null) is false, and date() of false is the epoch.
        None => "January 1, 1970".to_string(),
    }
}

/// The post's photos as the page's `src` values: full URLs pass through,
/// bare filenames live under `/images/post/`.
/// Takes the stored `picture` column: empty, one name, or a comma list of names.
/// Gives back a list of web addresses. `as_deref` borrows the text inside the
/// Option, and `filter` treats an empty string like "no picture".
pub fn post_images(picture: &Option<String>) -> Vec<String> {
    match picture.as_deref().filter(|p| !p.is_empty()) {
        Some(p) => p
            .split(',')
            .map(|img| {
                if img.starts_with("http") {
                    img.trim().to_string()
                } else {
                    format!("{IMG_BASE}{}", img.trim())
                }
            })
            .collect(),
        None => vec![NO_PICTURE.to_string()],
    }
}

/// One post as the page's template wants it: all text is ready to print.
/// This is separate from `PostRow` so the template needs no logic of its own.
#[derive(Clone)]
struct PostView {
    id: i32,
    title: String,
    author: String,
    date: String,
    images: Vec<String>,
    /// `nl2br(he(content))`: the PHP escapes the stored text again here,
    /// which is why an `&amp;` saved in a post shows as `&amp;` on this page.
    content_html: String,
    share_url: String,
    share_text: String,
}

/// The page's body template (`templates/blog.html`): the list of posts and
/// whether the viewer is signed in (admins see the Delete Post button).
/// Kept for the test that proves the page builder's pieces add up to this page.
#[cfg(test)]
#[derive(Template)]
#[template(path = "blog.html")]
struct BlogBody {
    posts: Vec<PostView>,
    is_admin: bool,
}

/// The page's opening wrapper (page shell; also used by the owner's own pages).
#[derive(Template)]
#[template(path = "blog.html", block = "shell_open")]
struct BlogOpen;

/// The page's closing wrapper (page shell).
#[derive(Template)]
#[template(path = "blog.html", block = "shell_close")]
struct BlogClose;

/// Section `blog.archive`.
#[derive(Template)]
#[template(path = "blog.html", block = "blog_archive")]
struct BlogArchive {
    posts: Vec<PostView>,
    is_admin: bool,
}

/// Draw the whole blog page through the page builder (default layout = the page as before).
async fn render(state: &Arc<AppState>, visitor: &Visitor) -> Html<String> {
    Html(crate::pagebuilder::render_page(state, visitor, "blog", "").await)
}

/// The page shell: the archive's opening and closing wrappers.
pub(crate) fn shell() -> (String, String) {
    (
        BlogOpen.render().unwrap_or_default(),
        BlogClose.render().unwrap_or_default(),
    )
}

/// Section `blog.archive`: read all posts and draw them.
/// `async` means it can wait for the database; `.await` is where it waits.
pub(crate) async fn section_archive(state: &AppState, visitor: &Visitor) -> String {
    // 1. Ask the database for every post, newest first. `query_as::<_, PostRow>`
    // tells sqlx to turn each row into a `PostRow`.
    let rows = sqlx::query_as::<_, PostRow>(
        "SELECT id, author, title, content, picture, created_at
         FROM garden_blog ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    // If the query fails, log the error and use an empty list (no crash).
    .unwrap_or_else(|e| {
        // The PHP catches the exception and shows "no posts yet".
        tracing::error!("blog query failed: {e}");
        Vec::new()
    });

    // 2. Convert each row into a `PostView`. `.iter().map(...)` runs the closure
    // (the `|p| ...` function) on each row; `.collect()` gathers the results.
    // `.clone()` makes a copy of text we only borrow. The author defaults to "Jun".
    let posts = rows
        .iter()
        .map(|p| PostView {
            id: p.id,
            title: p.title.clone(),
            author: p.author.clone().unwrap_or_else(|| "Jun".to_string()),
            date: post_date(&p.created_at),
            images: post_images(&p.picture),
            // Escape the HTML first (`he`), then turn line breaks into `<br>` (`nl2br`).
            content_html: nl2br(&he(&p.content)),
            share_url: urlencode("https://junsgarden.zapto.org/"),
            share_text: urlencode(&format!(
                "Check out this update from Jun's Garden: {}",
                p.title
            )),
        })
        .collect();

    // 3. Render the section's block.
    BlogArchive {
        posts,
        is_admin: visitor.is_admin(),
    }
    .render()
    .unwrap_or_default()
}

/// Handler for `GET /blog.php`. Takes the state and visitor (both supplied by
/// axum). Gives back the page. All the work is in `render`.
pub async fn blog(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
) -> Html<String> {
    render(&state, &visitor).await
}

/// Delete Post, and Logout.
/// Handler for `POST /blog.php`. Takes: state, visitor, the original address,
/// headers and the raw body. Gives back a redirect, or the page again.
/// Only an admin can delete. The form field names are the ones the PHP used.
pub async fn blog_post(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    OriginalUri(uri): OriginalUri,
    Query(q): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // 1. Read the posted form.
    let f = formdata::parse(&headers, body).await;
    // 2. Logout button: end the session and return to this page.
    if f.has("logout") {
        return session::logout_redirect(&state, &visitor, uri.path()).await;
    }
    // 3. Delete Post: only for a signed-in admin. `php_int` reads the id like PHP's
    // `(int)`, giving 0 for junk. The SQL uses `?` so the id is passed as data,
    // never pasted into the text. `let _ =` ignores the result on purpose.
    if visitor.is_admin() && f.has("delete_blog") {
        let id = php_int(f.str("blog_id"));
        if id != 0 {
            let _ = sqlx::query("DELETE FROM garden_blog WHERE id = ?")
                .bind(id)
                .execute(&state.db)
                .await;
        }
        // Go back to the blog page so a refresh does not repeat the delete.
        return (StatusCode::FOUND, [(header::LOCATION, "blog.php")]).into_response();
    }
    // 4. A form of a home-page section the owner placed on this page (guestbook, new post,
    // delete a guestbook line): handled by the home page's own code.
    if ["user_message", "post_blog", "edit_blog", "delete_comment"]
        .iter()
        .any(|k| f.has(k))
    {
        return crate::pages::index::shared_post(&state, &visitor, uri.path(), &q, &headers, &f)
            .await;
    }
    // 5. Anything else: show the page.
    render(&state, &visitor).await.into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(id: i32, title: &str) -> PostView {
        PostView {
            id,
            title: title.into(),
            author: "Gardener".into(),
            date: "October 5, 2026".into(),
            images: post_images(&Some("a.webp,b.webp".into())),
            content_html: nl2br(&he("one\n<two>")),
            share_url: urlencode("https://example.test/"),
            share_text: urlencode(title),
        }
    }

    /// The page builder's promise for the archive: shell + section = the whole page.
    #[test]
    fn archive_shell_and_section_add_up_to_the_whole_page() {
        for is_admin in [false, true] {
            for posts in [
                Vec::new(),
                vec![view(2, "Tulips & <roses>"), view(1, "Rain")],
            ] {
                let whole = BlogBody {
                    posts: posts.clone(),
                    is_admin,
                }
                .render()
                .unwrap();
                let (open, close) = shell();
                let archive = BlogArchive { posts, is_admin }.render().unwrap();
                assert_eq!(format!("{open}{archive}{close}"), whole, "admin={is_admin}");
                assert!(whole.starts_with("<div class=\"page-content\""));
                assert!(whole.ends_with("</div>\n</div>\n"));
            }
        }
    }
}
