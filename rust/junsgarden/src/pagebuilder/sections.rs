//! Drawing one section. Every section calls the page code that drew it before the page builder
//! (`pages/index.rs`, `pages/blog.rs`, `pages/gallery.rs`): nothing is copied or forked.
//!
//! The permission check is HERE, for every section, every time it is drawn, whatever page it
//! sits on: an admin section drawn for a visitor is an empty string.

use std::sync::Arc;

use super::layout::Cell;
use super::registry::{self, Perm};
use crate::pages::{blog, gallery, index};
use crate::php::he;
use crate::session::Visitor;
use crate::AppState;

/// The ids `render` can draw. The registry test checks this list against `registry::SECTIONS`.
#[cfg(test)]
pub const IMPLEMENTED: &[&str] = &[
    "home.welcome",
    "home.weather",
    "home.tips",
    "home.news",
    "home.guestbook",
    "admin.post_editor",
    "blog.archive",
    "gallery.wall",
];

/// What a section is drawn for.
pub struct Ctx<'a> {
    pub state: &'a Arc<AppState>,
    pub visitor: &'a Visitor,
    /// The `?zip=` of the address (the weather section reads it, as the home page did).
    pub zip: &'a str,
    /// The editor's preview: drawn as a visitor sees it, admin sections shown as a labelled
    /// placeholder so the owner can see where they are.
    pub preview: bool,
}

/// One drawn section: its HTML, and what it needs in `<head>` (added once per page).
#[derive(Default)]
pub struct Piece {
    pub html: String,
    pub head: Option<(&'static str, &'static str)>,
}

/// May this visitor see a section with this permission?
pub fn allowed(perm: Perm, visitor: &Visitor) -> bool {
    match perm {
        Perm::Public => true,
        Perm::Admin => visitor.is_admin(),
    }
}

/// Draw one (already validated) cell.
pub async fn render(ctx: &Ctx<'_>, cell: &Cell) -> Piece {
    let Some(def) = registry::find(&cell.section) else {
        return Piece::default();
    };
    if !allowed(def.perm, ctx.visitor) {
        let html = if ctx.preview {
            format!(
                "<div class=\"pb-locked\">🔒 {} · {}<br><small>Only signed-in gardeners see this. · 로그인한 사람만 볼 수 있어요.</small></div>\n",
                he(def.name_en),
                he(def.name_ko)
            )
        } else {
            String::new()
        };
        return Piece { html, head: None };
    }
    let (state, visitor) = (ctx.state, ctx.visitor);
    let html = match def.id {
        "home.welcome" => index::section_welcome(visitor),
        "home.weather" => index::section_weather(state, ctx.zip, &cell.text("zip")),
        "home.tips" => index::section_tips(),
        "home.news" => index::section_news(state, visitor, cell.int("count")).await,
        "home.guestbook" => index::section_guestbook(state, visitor).await,
        "admin.post_editor" => index::section_post_editor(),
        "blog.archive" => blog::section_archive(state, visitor).await,
        "gallery.wall" => {
            return Piece {
                html: gallery::section_wall(state).await,
                head: Some(("gallery.wall", gallery::GALLERY_HEAD)),
            }
        }
        _ => String::new(),
    };
    Piece { html, head: None }
}
