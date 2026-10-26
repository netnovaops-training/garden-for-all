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
    "home.radar",
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
        "home.radar" => radar(cell, ctx.preview || visitor.is_admin()),
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

/// A latitude/longitude setting as a number, or None if it is not one or out of range.
fn coord(s: &str, limit: f64) -> Option<f64> {
    s.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && v.abs() <= limit)
}

/// `home.radar`: the Windy rain map that `site/js/home.js` used to fill (`#radarFrame`), with
/// the same map options. The address is built only from parsed numbers, never from the text.
/// Without a valid place, visitors see nothing; the owner sees how to set it.
pub(crate) fn radar(cell: &Cell, show_hint: bool) -> String {
    let (Some(lat), Some(lon)) = (
        coord(&cell.text("lat"), 90.0),
        coord(&cell.text("lon"), 180.0),
    ) else {
        return if show_hint {
            "<div class=\"cartoony-card\" style=\"padding:20px; border:4px solid #333; border-radius:15px; text-align:center;\">🌧️ Rain radar map: add its latitude and longitude in the page editor. · 비 레이더 지도: 페이지 편집에서 위도와 경도를 넣어 주세요.</div>\n".into()
        } else {
            String::new()
        };
    };
    let zoom = cell.int("zoom");
    let src = format!(
        "https://embed.windy.com/embed2.html?lat={lat:.4}&lon={lon:.4}&detailLat={lat:.4}&detailLon={lon:.4}&width=650&height=450&zoom={zoom}&level=surface&overlay=rain&product=ecmwf&menu=&message=&marker=&calendar=now&pressure=&type=map&location=coordinates&detail=&metricWind=default&metricTemp=default&radarRange=-1"
    );
    format!(
        "<div class=\"cartoony-card\" style=\"padding:20px; border:4px solid #333; border-radius:15px; box-shadow:6px 6px 0 rgba(0,0,0,0.8);\">\n<h2 class=\"blog-cartoony-text\" style=\"text-align:center;\">🌧️ Rain radar · 비 레이더</h2>\n<iframe class=\"pb-radar\" src=\"{}\" title=\"Rain radar map\" loading=\"lazy\" referrerpolicy=\"strict-origin-when-cross-origin\" sandbox=\"allow-scripts allow-same-origin allow-popups\" style=\"width:100%; height:450px; border:3px solid #333; border-radius:12px;\"></iframe>\n</div>\n",
        he(&src)
    )
}
