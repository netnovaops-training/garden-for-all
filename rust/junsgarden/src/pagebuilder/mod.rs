//! The page builder: pages made of sections that the garden owner arranges on a desktop
//! (design and reasons: docs/page-builder/DESIGN.md; how to use it: docs/page-builder/GUIDE.md).
//!
//! HOW TO READ THIS FOLDER.
//! * `registry.rs`  every section: names (EN/KO), version, permission, settings.
//! * `layout.rs`    the layout JSON, the default layouts (= the pages as they were), checks.
//! * `sections.rs`  draws one section with the page code that drew it before.
//! * `saved.rs`     layouts in the database: versions, undo, reset, backup files.
//! * `updates.rs`   the author's signed list of sections (the "new" badges).
//! * `editor.rs`    the editor page and its API, for the owner only.
//! * this file      `Builder` (the builder's part of `AppState`) and `render_page`, which every
//!   built-in page (home, archive, gallery) and every owner page goes through.

pub mod editor;
pub mod layout;
pub mod registry;
pub mod saved;
pub mod sections;
#[cfg(test)]
mod tests;
pub mod updates;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::layout::Page;
use crate::pages::{blog, index};
use crate::php::{filemtime, he};
use crate::session::Visitor;
use crate::AppState;
use layout::Layout;

/// The page builder's part of the shared state.
pub struct Builder {
    /// The one person who may edit pages (a GARDEN_USERS name), or empty for nobody.
    pub owner: String,
    /// Made fresh at every start; the editor's anti-forgery token is derived from it.
    secret: [u8; 32],
    /// The newest layout of each page (None = default), dropped on every change.
    pub cache: Mutex<HashMap<String, Option<(i32, Layout)>>>,
    /// The last answer of the signed sections check, and when it was fetched.
    pub updates: Mutex<Option<(Instant, serde_json::Value)>>,
    /// The author's public key for the sections list (none = check not set up).
    pub sections_key: Option<ed25519_dalek::VerifyingKey>,
}

impl Builder {
    /// `owner_setting` is JUNSGARDEN_PAGE_OWNER; empty means the first GARDEN_USERS name.
    /// `key_setting` is JUNSGARDEN_SECTIONS_KEY (empty: the key pinned in updates.rs).
    pub fn new(users: &[crate::accounts::Account], owner_setting: &str, key_setting: &str) -> Self {
        let owner = match owner_setting.trim() {
            "" => users.first().map(|a| a.name.clone()).unwrap_or_default(),
            name => users
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case(name))
                .map(|a| a.name.clone())
                .unwrap_or_default(),
        };
        Builder {
            owner,
            secret: rand::random(),
            cache: Mutex::new(HashMap::new()),
            updates: Mutex::new(None),
            sections_key: updates::key_from(key_setting),
        }
    }

    /// Is this the owner, signed in on the website (an app key never edits pages)?
    pub fn is_owner(&self, v: &Visitor) -> bool {
        v.is_admin()
            && v.app_token.is_none()
            && !self.owner.is_empty()
            && v.username.eq_ignore_ascii_case(&self.owner)
    }

    /// The anti-forgery token for one session: SHA-256 of the start-up secret and the
    /// session id. Another site cannot read it, so it cannot send it.
    pub fn token(&self, sid: &str) -> String {
        let mut h = Sha256::new();
        h.update(self.secret);
        h.update([0u8]);
        h.update(sid.as_bytes());
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Compare a sent token with the right one, in constant time.
    pub fn token_ok(&self, sid: &str, given: &str) -> bool {
        let want = self.token(sid);
        want.len() == given.len()
            && want
                .bytes()
                .zip(given.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0
    }
}

/// The frame settings and outer wrapper of each page (not sections: they stay as they were).
struct Shell {
    title: String,
    show_hero: bool,
    open: String,
    close: String,
}

fn shell(state: &AppState, page: &str) -> Shell {
    match page {
        "home" => {
            let (open, close) = index::shell(state);
            Shell {
                title: "Jun's and Kona's Garden Dashboard".into(),
                show_hero: true,
                open,
                close,
            }
        }
        "blog" => {
            let (open, close) = blog::shell();
            Shell {
                title: "Garden Archives — Jun's & Kona's Garden".into(),
                show_hero: false,
                open,
                close,
            }
        }
        "gallery" => Shell {
            title: "Gallery | Jun's and Kona's Garden".into(),
            show_hero: false,
            open: String::new(),
            close: String::new(),
        },
        // The owner's own pages: the archive's plain green wrapper, no hero.
        other => {
            let (open, close) = blog::shell();
            Shell {
                title: format!(
                    "{} | {}",
                    layout::custom_slug(other).unwrap_or(other),
                    crate::layout::DEFAULT_TITLE
                ),
                show_hero: false,
                open,
                close,
            }
        }
    }
}

/// The address of a page, relative to the site (pages and the editor live at the top level).
pub fn page_href(page: &str) -> String {
    match page {
        "home" => "index.php".into(),
        "blog" => "blog.php".into(),
        "gallery" => "gallery.php".into(),
        other => format!("page.php?name={}", layout::custom_slug(other).unwrap_or("")),
    }
}

/// Draw a page from its saved layout (or the default one, which is the page as it was).
pub async fn render_page(
    state: &Arc<AppState>,
    visitor: &Visitor,
    page: &str,
    zip: &str,
) -> String {
    let l = saved::current(state, page)
        .await
        .map(|(_, l)| l)
        .unwrap_or_else(|| layout::default_layout(page));
    draw(state, visitor, page, &l, zip, false).await
}

/// Draw a page from a given (validated) layout. `preview` = the editor's preview: drawn as a
/// visitor sees it, admin sections as labelled placeholders, no Edit button.
pub async fn draw(
    state: &Arc<AppState>,
    visitor: &Visitor,
    page: &str,
    l: &Layout,
    zip: &str,
    preview: bool,
) -> String {
    let ctx = sections::Ctx {
        state,
        visitor,
        zip,
        preview,
    };
    let mut rows = Vec::with_capacity(l.rows.len());
    let mut heads: Vec<(&str, &str)> = Vec::new();
    for row in &l.rows {
        let mut cols = Vec::with_capacity(row.cols.len());
        for cell in &row.cols {
            let piece = sections::render(&ctx, cell).await;
            if let Some((id, head)) = piece.head {
                if !heads.iter().any(|(h, _)| *h == id) {
                    heads.push((id, head));
                }
            }
            cols.push(piece.html);
        }
        rows.push(cols);
    }
    let (body, grid) = layout::join_rows(rows);
    let sh = shell(state, page);
    let mut html = format!("{}{}{}", sh.open, body, sh.close);
    if !preview && state.pagebuilder.is_owner(visitor) {
        html.push_str(&edit_button(page));
    }
    let mut extra_head: String = heads.iter().map(|(_, h)| *h).collect();
    if grid || preview {
        extra_head.push_str(&format!(
            "\n<link rel=\"stylesheet\" href=\"pagebuilder/pb.css?v={}\">\n",
            filemtime(&state.www, "pagebuilder/pb.css")
        ));
    }
    let mut p = Page::new(html);
    p.title = sh.title;
    p.show_hero = sh.show_hero;
    p.extra_head = extra_head;
    crate::layout::render(state, p, visitor).await
}

/// The owner's Edit button (fixed in a corner; hidden on phones, where editing is off).
fn edit_button(page: &str) -> String {
    format!(
        "\n<a class=\"pb-edit-button\" href=\"pagebuilder.php?page={}\" style=\"position:fixed; left:20px; bottom:20px; z-index:99997; background:#FFD700; color:#333; border:3px solid #333; border-radius:20px; padding:10px 18px; font-weight:900; text-decoration:none; box-shadow:3px 3px 0 rgba(0,0,0,0.6);\">✏️ Edit page · 페이지 편집</a>\n<style>@media (max-width: 900px) {{ .pb-edit-button {{ display:none; }} }}</style>\n",
        he(page)
    )
}
