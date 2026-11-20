//! `/help` — the garden's guides on the website, in English and Korean.
//!
//! The guides are START-HERE/*.md (and START-HERE/ko/*.md), the same files the
//! PDFs and the handbook are made from, read from `JUNSGARDEN_GUIDES_DIR`
//! (docker-compose.yml mounts the garden folder's START-HERE there, read-only).
//! Edit a guide and the page shows the change at once; no rebuild.
//!
//! Who can see it: only people who are logged in (the accounts in
//! GARDEN_USERS). The guides describe this garden's own setup (folders, menus,
//! the router, backups), so they are not for strangers. A visitor is sent to
//! the login page and back here afterwards, as the plant admin (/streamlit/) does.
//!
//! Addresses (all GET):
//!   /help                         every guide in reading order, with a search box
//!   /help?guide=NAME[&lang=ko]    one whole guide
//!   /help/ID[?lang=ko]            one help id (help-ids.tsv, the one registry the
//!                                 setup wizard's Help button uses too): the brief
//!                                 overview, "Read the full guide", and the full
//!                                 reading of that section with its pictures. Its
//!                                 #hot words open that id.
//!   /help/NAME                    a guide's name redirects to /help?guide=NAME
//!   /help?id=ID, /help/id/ID      older forms; redirect to /help/ID
//!   /help/images/FILE             a picture from START-HERE/images
//!
//! The shared page frame links its stylesheet and menu with relative addresses
//! (`style.css`, `index.php`), which only work from the top level; `framed` makes
//! them absolute so /help/ID works too. Guide names start with a digit and ids
//! never do, so a name and an id can never be the same (a test checks it).
//!
//! HOW TO READ THIS FILE. `routes` lists the addresses. `help` is the switchboard
//! for /help (index, guide or brief). The Markdown work is in `guides.rs`.

use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};

use crate::guides::{self, HelpEntry, Lang};
use crate::layout::{self, Page};
use crate::php::{he, urlencode};
use crate::session::Visitor;
use crate::AppState;

/// The help addresses, added to the main router in main.rs.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/help", get(help))
        .route("/help/", get(|| async { redirect("/help") }))
        .route("/help/images/{name}", get(image))
        .route("/help/id/{id}", get(old_id_path))
        .route("/help/{what}", get(help_what))
}

/// A 303 redirect to a path on this site.
fn redirect(to: &str) -> Response {
    (StatusCode::SEE_OTHER, [(header::LOCATION, to.to_string())]).into_response()
}

/// Visitors who are not logged in go to the login page, then back to `uri`.
fn need_login(visitor: &Visitor, uri: &Uri) -> Option<Response> {
    if visitor.is_admin() {
        return None;
    }
    let back = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/help");
    Some(redirect(&format!("/login.php?back={}", urlencode(back))))
}

/// The help pages' own look, on top of the site's style.css.
const STYLE: &str = r#"<style>
.help-page { max-width: 860px; margin: 0 auto; padding: 18px 16px 48px; box-sizing: border-box; font-size: 1.05rem; line-height: 1.65; }
.help-page [hidden] { display: none !important; }
.help-top { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 10px; }
.help-top h1 { margin: 6px 0; font-size: 1.7rem; }
.help-lang { display: flex; gap: 6px; }
.help-lang a { padding: 6px 14px; border-radius: 20px; border: 2px solid var(--site-row-border, #c0d8a5); text-decoration: none; color: inherit; background: rgba(255,255,255,.7); }
.help-lang a[aria-current] { background: var(--site-accent, #2e7d32); border-color: var(--site-accent, #2e7d32); color: #fff; font-weight: 700; }
.help-back { font-weight: 600; }
.help-intro, .help-where, .help-small { color: var(--site-row-label, #5a7a50); }
.help-where { margin: 14px 0 0; font-size: .95rem; }
.help-note { background: #fff4dc; border-left: 4px solid #8a5a00; padding: 10px 14px; border-radius: 8px; }
.help-search { display: block; margin: 14px 0 6px; font-weight: 600; }
.help-search input { display: block; width: 100%; box-sizing: border-box; margin-top: 6px; padding: 12px 14px; font-size: 1.1rem; border-radius: 12px; border: 2px solid var(--site-row-border, #c0d8a5); background: rgba(255,255,255,.9); }
.help-step h2 { font-size: 1.25rem; margin: 26px 0 8px; display: flex; align-items: center; gap: 10px; }
.help-num { display: inline-flex; min-width: 2rem; height: 2rem; padding: 0 6px; box-sizing: border-box; align-items: center; justify-content: center; border-radius: 1rem; background: var(--site-accent, #2e7d32); color: #fff; font-size: 1rem; }
.help-guides { list-style: none; margin: 0; padding: 0; display: grid; gap: 10px; }
.help-guides li { background: rgba(255,255,255,.85); border: 1px solid var(--site-row-border, #c0d8a5); border-radius: 14px; padding: 12px 16px; }
.help-guides p { margin: 4px 0 0; }
.help-title { font-size: 1.2rem; font-weight: 700; text-decoration: none; color: var(--site-accent, #2e7d32); }
.help-title:hover { text-decoration: underline; }
.help-tags span, .help-tags a { font-weight: 700; text-decoration: none; color: inherit; display: inline-block; margin: 2px 6px 0 0; padding: 1px 8px; border-radius: 10px; background: var(--site-chip-bg, rgba(46,125,50,.1)); font-size: .85rem; }
.help-toc { margin: 10px 0 18px; background: rgba(255,255,255,.85); border: 1px solid var(--site-row-border, #c0d8a5); border-radius: 12px; padding: 8px 14px; }
.help-toc summary { cursor: pointer; font-weight: 700; }
.help-guide { background: rgba(255,255,255,.9); border-radius: 14px; padding: 6px 20px 18px; overflow-wrap: break-word; }
.help-guide h2 { margin-top: 1.8em; border-bottom: 2px solid var(--site-row-border, #c0d8a5); padding-bottom: 4px; }
.help-guide img { max-width: 100%; height: auto; border-radius: 8px; display: block; margin: 10px auto; }
.help-guide pre { background: #eef2ee; padding: 10px 12px; border-radius: 8px; overflow-x: auto; white-space: pre-wrap; }
.help-guide code { background: #eef2ee; padding: 0 4px; border-radius: 4px; font-size: .92em; }
.help-guide pre code { padding: 0; }
.help-guide table { border-collapse: collapse; display: block; overflow-x: auto; max-width: 100%; }
.help-guide th, .help-guide td { border: 1px solid var(--site-row-border, #c0d8a5); padding: 4px 8px; vertical-align: top; text-align: left; }
.help-guide blockquote { border-left: 4px solid #8a5a00; background: #fff4dc; margin: 10px 0; padding: 6px 12px; border-radius: 6px; }
.help-guide :target { scroll-margin-top: 80px; }
.help-pager { display: flex; justify-content: space-between; gap: 12px; margin-top: 22px; }
.help-pager a { flex: 1; max-width: 48%; padding: 10px 14px; border-radius: 12px; border: 1px solid var(--site-row-border, #c0d8a5); background: rgba(255,255,255,.85); text-decoration: none; }
.help-pager small { display: block; color: var(--site-row-label, #5a7a50); }
.help-next { text-align: right; }
.help-brief { background: rgba(255,255,255,.9); border-radius: 14px; padding: 12px 20px 18px; margin-top: 8px; }
.help-brief h1 { font-size: 1.4rem; }
.help-btn { display: inline-block; padding: 10px 18px; border-radius: 12px; font-weight: 700; text-decoration: none; }
@media (max-width: 600px) { .help-page { font-size: 1.1rem; padding: 12px 10px 40px; } .help-guide { padding: 4px 12px 14px; } .help-pager { flex-direction: column; } .help-pager a { max-width: none; } }
</style>"#;

/// Wrap a help body in the site's frame (no hero banner: these are pages to read).
fn framed(state: &AppState, visitor: &Visitor, title: &str, body: String) -> Html<String> {
    let mut p = Page::new(body);
    p.title = format!("{title} — {}", layout::DEFAULT_TITLE);
    p.show_hero = false;
    p.extra_head = STYLE.to_string();
    let page = layout::render_now(state, p, visitor);
    // The frame's own links are relative to the site's top level; make them
    // absolute so a page below /help/ finds its stylesheet, scripts and menu.
    let mut page = page;
    for (from, to) in [
        ("href=\"style.css", "href=\"/style.css"),
        ("src=\"js/", "src=\"/js/"),
        ("href=\"index.php\"", "href=\"/index.php\""),
        ("href=\"plant_listing.php\"", "href=\"/plant_listing.php\""),
        ("href=\"catalog.php\"", "href=\"/catalog.php\""),
        ("href=\"data.php\"", "href=\"/data.php\""),
        ("href=\"gallery.php\"", "href=\"/gallery.php\""),
        ("fetch('comments.php", "fetch('/comments.php"),
    ] {
        page = page.replace(from, to);
    }
    Html(page)
}

/// The 404 page for an unknown guide or help id.
fn not_found(state: &AppState, visitor: &Visitor, lang: Lang) -> Response {
    let (msg, back) = match lang {
        Lang::En => ("There is no such help page.", "All guides"),
        Lang::Ko => ("그런 도움말 페이지는 없어요.", "모든 안내서"),
    };
    let q = if lang == Lang::Ko { "?lang=ko" } else { "" };
    let body = format!(
        "<div class=\"page-content no-hero help-page\"><h1>{}</h1><p><a href=\"/help{q}\">&larr; {}</a></p></div>",
        he(msg),
        he(back)
    );
    (StatusCode::NOT_FOUND, framed(state, visitor, "Help", body)).into_response()
}

// ---------------------------------------------------------------- /help

/// `/help`: the index, one guide (`?guide=`) or one section in brief (`?id=`).
pub async fn help(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
    uri: Uri,
) -> Response {
    if let Some(r) = need_login(&visitor, &uri) {
        return r;
    }
    let lang = Lang::from_param(q.get("lang").map(String::as_str));
    if let Some(name) = q.get("guide") {
        return guide_page(&state, &visitor, name, lang);
    }
    if let Some(id) = q.get("id") {
        if !guides::id_ok(id) {
            return not_found(&state, &visitor, lang);
        }
        // `&full=1` goes straight to the section inside the guide.
        if q.contains_key("full") {
            return go_full(&state, &visitor, id, lang);
        }
        return redirect(&format!("/help/{id}{}", lang.qs()));
    }
    index_page(&state, &visitor, lang)
}

/// One guide in the index.
struct GuideView {
    name: &'static str,
    title: String,
    summary: String,
    tags: Vec<(String, String)>,
    /// Lower-case words the search box looks in.
    search: String,
    /// There is a file in the chosen language (Korean may be missing).
    in_lang: bool,
}

struct StepView {
    num: &'static str,
    label: &'static str,
    guides: Vec<GuideView>,
}

struct TopicView {
    id: String,
    brief: String,
    guide_title: String,
    tags: Vec<String>,
    search: String,
}

#[derive(Template)]
#[template(path = "help_index.html")]
struct IndexTemplate {
    lang: &'static str,
    q_lang: &'static str,
    ko: bool,
    amp: &'static str,
    missing: bool,
    steps: Vec<StepView>,
    topics: Vec<TopicView>,
}

/// A guide's text in `lang`, falling back to English. Gives the text and
/// whether it is in the language asked for.
fn read_guide(state: &AppState, name: &str, lang: Lang) -> Option<(String, bool)> {
    if let Some(p) = guides::guide_file(&state.guides, name, lang) {
        return std::fs::read_to_string(p).ok().map(|t| (t, true));
    }
    let p = guides::guide_file(&state.guides, name, Lang::En)?;
    std::fs::read_to_string(p)
        .ok()
        .map(|t| (t, lang == Lang::En))
}

fn split_tags(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_string).collect()
}

fn index_page(state: &AppState, visitor: &Visitor, lang: Lang) -> Response {
    let reg = guides::load_registry(&state.guides);
    let mut titles: HashMap<&str, String> = HashMap::new();
    let mut steps = Vec::new();
    for s in guides::STEPS {
        let mut list = Vec::new();
        for &name in s.guides {
            let Some((text, in_lang)) = read_guide(state, name, lang) else {
                continue;
            };
            let title = guides::title_of(&text);
            let summary = guides::summary_of(&text);
            // Each #word opens the help id it was written for.
            let mut tags: Vec<(String, String)> = Vec::new();
            for e in reg.iter().filter(|e| e.guide == name) {
                for t in split_tags(e.tags(lang)) {
                    tags.push((t, e.id.clone()));
                }
            }
            let words: Vec<&str> = tags.iter().map(|t| t.0.as_str()).collect();
            let search = format!("{title} {summary} {} {name}", words.join(" ")).to_lowercase();
            titles.insert(name, title.clone());
            list.push(GuideView {
                name,
                title,
                summary,
                tags,
                search,
                in_lang,
            });
        }
        if !list.is_empty() {
            steps.push(StepView {
                num: s.num,
                label: if lang == Lang::Ko { s.ko } else { s.en },
                guides: list,
            });
        }
    }
    let topics = reg
        .iter()
        .filter(|e| titles.contains_key(e.guide.as_str()))
        .map(|e| {
            let guide_title = titles.get(e.guide.as_str()).cloned().unwrap_or_default();
            let tags = split_tags(e.tags(lang));
            let search = format!(
                "{} {} {} {}",
                e.brief(lang),
                tags.join(" "),
                guide_title,
                e.id
            )
            .to_lowercase();
            TopicView {
                id: e.id.clone(),
                brief: e.brief(lang).to_string(),
                guide_title,
                tags,
                search,
            }
        })
        .collect();
    let t = IndexTemplate {
        lang: lang.code(),
        q_lang: lang.qs(),
        ko: lang == Lang::Ko,
        amp: lang.amp(),
        missing: steps.is_empty(),
        steps,
        topics,
    };
    let title = if lang == Lang::Ko {
        "도움말"
    } else {
        "Help"
    };
    framed(state, visitor, title, t.render().unwrap_or_default()).into_response()
}

// ---------------------------------------------------------------- one guide

#[derive(Template)]
#[template(path = "help_guide.html")]
struct GuideTemplate {
    lang: &'static str,
    ko: bool,
    amp: &'static str,
    q_lang: &'static str,
    name: &'static str,
    has_ko: bool,
    english_only: bool,
    step_num: &'static str,
    step_label: &'static str,
    title: String,
    sections: Vec<(String, String)>,
    body: String,
    prev: Option<(&'static str, String)>,
    next: Option<(&'static str, String)>,
}

/// The title of a neighbouring guide, for "Previous" / "Next".
fn neighbour(
    state: &AppState,
    name: Option<&'static str>,
    lang: Lang,
) -> Option<(&'static str, String)> {
    let name = name?;
    let (text, _) = read_guide(state, name, lang)?;
    Some((name, guides::title_of(&text)))
}

fn guide_page(state: &AppState, visitor: &Visitor, asked: &str, lang: Lang) -> Response {
    // The name is looked up in the fixed list; from here on only the list's own
    // text is used, never what was typed in the address.
    let Some(name) = guides::order().find(|g| *g == asked) else {
        return not_found(state, visitor, lang);
    };
    let Some((text, in_lang)) = read_guide(state, name, lang) else {
        return not_found(state, visitor, lang);
    };
    let shown = if in_lang { lang } else { Lang::En };
    let r = guides::render(&text, shown);
    let order: Vec<&'static str> = guides::order()
        .filter(|g| guides::guide_file(&state.guides, g, Lang::En).is_some())
        .collect();
    let i = order.iter().position(|g| *g == name);
    let prev = i
        .and_then(|i| i.checked_sub(1))
        .and_then(|j| order.get(j).copied());
    let next = i.and_then(|i| order.get(i + 1).copied());
    let step = guides::step_of(name);
    let title = guides::title_of(&text);
    let t = GuideTemplate {
        lang: lang.code(),
        ko: lang == Lang::Ko,
        amp: lang.amp(),
        q_lang: if lang == Lang::Ko { "?lang=ko" } else { "" },
        name,
        has_ko: guides::guide_file(&state.guides, name, Lang::Ko).is_some(),
        english_only: !in_lang,
        step_num: step.map(|s| s.num).unwrap_or(""),
        step_label: step
            .map(|s| if lang == Lang::Ko { s.ko } else { s.en })
            .unwrap_or(""),
        title: title.clone(),
        sections: r.sections,
        body: r.html,
        prev: neighbour(state, prev, lang),
        next: neighbour(state, next, lang),
    };
    framed(state, visitor, &title, t.render().unwrap_or_default()).into_response()
}

// ---------------------------------------------------------------- one help id

#[derive(Template)]
#[template(path = "help_brief.html")]
struct BriefTemplate {
    lang: &'static str,
    ko: bool,
    q_lang: &'static str,
    id: String,
    guide_title: String,
    brief: String,
    tags: Vec<String>,
    full_href: String,
    /// The section itself, as HTML the code made from the guide.
    section: String,
}

/// Where a help id's full text is: the guide in `lang` (English when that
/// guide has no Korean file) at the id's anchor.
pub fn full_href(state: &AppState, e: &HelpEntry, lang: Lang) -> String {
    let lang = if guides::guide_file(&state.guides, &e.guide, lang).is_some() {
        lang
    } else {
        Lang::En
    };
    format!("/help?guide={}{}#{}", e.guide, lang.amp(), e.anchor(lang))
}

/// `/help?id=ID&full=1`: straight to the section inside its guide.
fn go_full(state: &AppState, visitor: &Visitor, id: &str, lang: Lang) -> Response {
    let reg = guides::load_registry(&state.guides);
    match reg.iter().find(|e| e.id == id) {
        Some(e) => redirect(&full_href(state, e, lang)),
        None => not_found(state, visitor, lang),
    }
}

/// `/help/ID`: the brief, the way to the full guide, and the full reading.
fn id_page(state: &AppState, visitor: &Visitor, id: &str, lang: Lang, _full: bool) -> Response {
    let reg = guides::load_registry(&state.guides);
    let Some(e) = reg.iter().find(|e| e.id == id) else {
        return not_found(state, visitor, lang);
    };
    let Some((text, in_lang)) = read_guide(state, &e.guide, lang) else {
        return not_found(state, visitor, lang);
    };
    let shown = if in_lang { lang } else { Lang::En };
    let section = guides::section_html(&guides::render(&text, shown).html, &e.anchor(shown));
    let t = BriefTemplate {
        lang: lang.code(),
        ko: lang == Lang::Ko,
        q_lang: if lang == Lang::Ko { "?lang=ko" } else { "" },
        id: e.id.clone(),
        guide_title: guides::title_of(&text),
        brief: e.brief(lang).to_string(),
        tags: split_tags(e.tags(lang)),
        full_href: full_href(state, e, lang),
        section,
    };
    framed(state, visitor, "Help", t.render().unwrap_or_default()).into_response()
}

// ---------------------------------------------------------------- short forms and pictures

/// `/help/WHAT`: a help id shows its page; a guide's name goes to the guide;
/// anything else is a 404.
pub async fn help_what(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Path(what): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    uri: Uri,
) -> Response {
    if let Some(r) = need_login(&visitor, &uri) {
        return r;
    }
    let lang = Lang::from_param(q.get("lang").map(String::as_str));
    if let Some(g) = guides::order().find(|g| *g == what) {
        return redirect(&format!("/help?guide={g}{}", lang.amp()));
    }
    if !guides::id_ok(&what) {
        return not_found(&state, &visitor, lang);
    }
    id_page(&state, &visitor, &what, lang, false)
}

/// `/help/id/ID` (an older form) -> `/help/ID`.
pub async fn old_id_path(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Path(id): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    uri: Uri,
) -> Response {
    if let Some(r) = need_login(&visitor, &uri) {
        return r;
    }
    let lang = Lang::from_param(q.get("lang").map(String::as_str));
    if !guides::id_ok(&id) {
        return not_found(&state, &visitor, lang);
    }
    redirect(&format!("/help/{id}{}", lang.qs()))
}

/// `/help/images/FILE`: a picture from START-HERE/images (plain names only).
pub async fn image(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Path(name): Path<String>,
) -> Response {
    if !visitor.is_admin() {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some((path, mime)) = guides::image_file(&state.guides, &name) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match tokio::fs::read(&path).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, mime),
                (header::CACHE_CONTROL, "private, max-age=86400"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

// The tests are in help_tests.rs (a folder named help/ is kept out of git:
// the menu's help files use that name).
#[cfg(test)]
#[path = "help_tests.rs"]
mod tests;
