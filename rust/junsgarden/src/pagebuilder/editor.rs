//! The editor page and its API: for the owner only, on a desktop browser.
//!
//! HOW TO READ THIS FILE. `routes` lists the addresses. Every handler starts with a gate:
//! `owner_only` (GET) or `owner_post` (POST: owner + JSON + the anti-forgery token), and
//! nothing is read from the request before the gate has passed. Bodies are at most 64 KB and
//! every layout is checked against the registry (`layout::validate`) before it is used.
//! The page itself is `templates/pagebuilder_editor.html`; the work in the browser is
//! `site/pagebuilder/editor.js`.

// The gates return the refusal itself (`Err(Response)`): a response is large, but it is made
// once per refused request, and early returns keep every handler readable.
#![allow(clippy::result_large_err)]

use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use super::layout::{self, Layout};
use super::{registry, saved, updates};
use crate::php::{filemtime, urlencode};
use crate::session::Visitor;
use crate::AppState;

/// The editor's addresses. The editor page sits at the top level so the site's relative
/// links (`style.css`, `js/...`) work in its preview.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/pagebuilder.php", get(editor_page))
        .route("/pagebuilder/api/registry", get(api_registry))
        .route("/pagebuilder/api/layout", get(api_layout))
        .route("/pagebuilder/api/updates", get(api_updates))
        .route("/pagebuilder/api/preview", post(api_preview))
        .route("/pagebuilder/api/save", post(api_save))
        .route("/pagebuilder/api/restore", post(api_restore))
        .route("/pagebuilder/api/seen", post(api_seen))
        .layer(DefaultBodyLimit::max(layout::MAX_JSON))
}

/// A refusal or error as JSON, in both languages.
fn fail(status: StatusCode, en: &str, ko: &str) -> Response {
    (status, Json(json!({ "error": en, "error_ko": ko }))).into_response()
}

fn not_owner() -> Response {
    fail(
        StatusCode::FORBIDDEN,
        "Only the garden owner, signed in on a computer, can edit pages.",
        "페이지 편집은 정원 주인이 컴퓨터에서 로그인했을 때만 할 수 있어요.",
    )
}

/// GET gate: the owner on a website session.
fn owner_only(state: &AppState, visitor: &Visitor) -> Result<(), Response> {
    if state.pagebuilder.is_owner(visitor) {
        Ok(())
    } else {
        Err(not_owner())
    }
}

/// POST gate: the owner, a JSON body (a plain HTML form cannot send one), and the token.
/// (Cross-site posts were already refused by `guard::same_site_posts`.)
fn owner_post(state: &AppState, visitor: &Visitor, headers: &HeaderMap) -> Result<(), Response> {
    owner_only(state, visitor)?;
    let ctype = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !ctype.to_ascii_lowercase().starts_with("application/json") {
        return Err(fail(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Send JSON.",
            "JSON으로 보내 주세요.",
        ));
    }
    let token = headers
        .get("x-pb-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !state.pagebuilder.token_ok(&visitor.sid, token) {
        return Err(fail(
            StatusCode::FORBIDDEN,
            "The page is out of date. Reload the editor.",
            "페이지가 오래됐어요. 편집기를 새로 고쳐 주세요.",
        ));
    }
    Ok(())
}

/// Read a JSON body into `T`, refusing anything that does not fit.
fn body<T: for<'de> Deserialize<'de>>(bytes: &Bytes) -> Result<T, Response> {
    serde_json::from_slice(bytes).map_err(|_| {
        fail(
            StatusCode::BAD_REQUEST,
            "That request is not understood.",
            "요청을 이해하지 못했어요.",
        )
    })
}

/// A page name from the request, checked.
fn page_arg(page: &str) -> Result<String, Response> {
    if layout::page_id_ok(page) {
        Ok(page.to_string())
    } else {
        Err(fail(
            StatusCode::BAD_REQUEST,
            "Unknown page. Owner pages are named with a-z, 0-9 and -.",
            "알 수 없는 페이지예요. 새 페이지 이름에는 a-z, 0-9, - 만 쓸 수 있어요.",
        ))
    }
}

/// A layout from the request, checked against the registry (and its defaults filled in).
fn checked(mut l: Layout, page: &str) -> Result<Layout, Response> {
    layout::validate(&mut l, page).map(|_| l).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.en, "error_ko": e.ko })),
        )
            .into_response()
    })
}

#[derive(Template)]
#[template(path = "pagebuilder_editor.html")]
struct EditorPage {
    page: String,
    token: String,
    editor_js_v: u64,
    editor_css_v: u64,
}

/// `GET /pagebuilder.php?page=home`: the editor. Visitors are sent to the login.
async fn editor_page(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let page = q
        .get("page")
        .filter(|p| layout::page_id_ok(p))
        .cloned()
        .unwrap_or_else(|| "home".to_string());
    if !visitor.is_admin() {
        let back = format!("/pagebuilder.php?page={page}");
        return Redirect::to(&format!("/login.php?back={}", urlencode(&back))).into_response();
    }
    if let Err(r) = owner_only(&state, &visitor) {
        return r;
    }
    let t = EditorPage {
        page,
        token: state.pagebuilder.token(&visitor.sid),
        editor_js_v: filemtime(&state.www, "pagebuilder/editor.js"),
        editor_css_v: filemtime(&state.www, "pagebuilder/editor.css"),
    };
    match t.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("editor page: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// `GET /pagebuilder/api/registry`: every section, with a "new" mark for ones the owner has
/// not seen yet. On the very first visit everything there is counts as seen.
async fn api_registry(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
) -> Response {
    if let Err(r) = owner_only(&state, &visitor) {
        return r;
    }
    let seen = match saved::seen(&state).await {
        Some(s) if s.is_empty() => {
            let all: Vec<&str> = registry::SECTIONS.iter().map(|s| s.id).collect();
            saved::mark_seen(&state, &all).await;
            None
        }
        other => other,
    };
    let sections: Vec<Value> = registry::SECTIONS
        .iter()
        .map(|s| {
            let mut v = serde_json::to_value(s).unwrap_or_default();
            v["new"] = json!(seen.as_ref().is_some_and(|seen| !seen.contains(s.id)));
            v
        })
        .collect();
    Json(json!({ "sections": sections, "pending": registry::PENDING })).into_response()
}

/// `GET /pagebuilder/api/layout?page=home`: the page's layout, version, history, and the
/// list of pages.
async fn api_layout(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    if let Err(r) = owner_only(&state, &visitor) {
        return r;
    }
    let page = match page_arg(q.get("page").map(String::as_str).unwrap_or("home")) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let (version, l) = saved::current(&state, &page)
        .await
        .unwrap_or_else(|| (0, layout::default_layout(&page)));
    let mut pages = vec![
        "home".to_string(),
        "blog".to_string(),
        "gallery".to_string(),
    ];
    pages.extend(saved::custom_pages(&state).await);
    Json(json!({
        "page": page,
        "version": version,
        "layout": l,
        "history": saved::history(&state, &page).await,
        "pages": pages,
        "view": super::page_href(&page),
    }))
    .into_response()
}

/// `GET /pagebuilder/api/updates`: the signed sections list (updates.rs).
async fn api_updates(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
) -> Response {
    if let Err(r) = owner_only(&state, &visitor) {
        return r;
    }
    Json(updates::check(&state).await).into_response()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviewBody {
    page: String,
    layout: Layout,
}

/// `POST /pagebuilder/api/preview`: the page drawn from an unsaved layout, as a visitor sees it.
async fn api_preview(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: PreviewBody = body(&bytes)?;
        let page = page_arg(&b.page)?;
        let l = checked(b.layout, &page)?;
        let html = super::draw(&state, &Visitor::default(), &page, &l, "", true).await;
        Ok::<_, Response>(Json(json!({ "html": html })).into_response())
    };
    run.await.unwrap_or_else(|r| r)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveBody {
    page: String,
    base: i32,
    layout: Layout,
}

/// The answer to a save, undo or reset.
fn saved_answer(r: Result<i32, saved::SaveError>) -> Response {
    match r {
        Ok(v) => Json(json!({ "version": v })).into_response(),
        Err(saved::SaveError::Conflict { now }) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "This page was changed in another window. Reload to see it.",
                "error_ko": "다른 창에서 이 페이지가 바뀌었어요. 새로 고쳐서 확인하세요.",
                "now": now,
            })),
        )
            .into_response(),
        Err(saved::SaveError::Db(e)) => {
            tracing::error!("page layout not saved: {e}");
            fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Not saved: the database did not answer.",
                "저장하지 못했어요: 데이터베이스가 응답하지 않아요.",
            )
        }
    }
}

/// `POST /pagebuilder/api/save`: check, back up, store as the next version.
async fn api_save(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: SaveBody = body(&bytes)?;
        let page = page_arg(&b.page)?;
        let l = checked(b.layout, &page)?;
        let ids: Vec<&str> = l
            .rows
            .iter()
            .flat_map(|r| r.cols.iter())
            .filter_map(|c| registry::find(&c.section).map(|d| d.id))
            .collect();
        let r = saved::save(&state, &page, &l, b.base, &visitor.username, "save").await;
        if r.is_ok() {
            // A section once placed is no longer "new".
            saved::mark_seen(&state, &ids).await;
        }
        Ok::<_, Response>(saved_answer(r))
    };
    run.await.unwrap_or_else(|r| r)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RestoreBody {
    page: String,
    base: i32,
    /// "undo" (the version before `base`), "reset" (the default layout) or "version".
    action: String,
    #[serde(default)]
    version: i32,
}

/// `POST /pagebuilder/api/restore`: undo, reset to the original page, or bring back an older
/// version. It is stored as a NEW version, so it can itself be undone.
async fn api_restore(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: RestoreBody = body(&bytes)?;
        let page = page_arg(&b.page)?;
        let target = match b.action.as_str() {
            "undo" if b.base > 0 => b.base - 1,
            "reset" => 0,
            "version" if (0..b.base.max(1)).contains(&b.version) => b.version,
            _ => {
                return Err(fail(
                    StatusCode::BAD_REQUEST,
                    "There is nothing to undo.",
                    "되돌릴 것이 없어요.",
                ))
            }
        };
        let Some(l) = saved::at(&state, &page, target).await else {
            return Err(fail(
                StatusCode::NOT_FOUND,
                "That version is no longer kept.",
                "그 버전은 더 이상 보관되어 있지 않아요.",
            ));
        };
        let note = if b.action == "version" {
            "restore"
        } else {
            b.action.as_str()
        };
        Ok::<_, Response>(saved_answer(
            saved::save(&state, &page, &l, b.base, &visitor.username, note).await,
        ))
    };
    run.await.unwrap_or_else(|r| r)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SeenBody {
    ids: Vec<String>,
}

/// `POST /pagebuilder/api/seen`: the owner pressed OK on "new" sections.
async fn api_seen(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let b: SeenBody = match body(&bytes) {
        Ok(b) => b,
        Err(r) => return r,
    };
    // Only ids from the registry are ever stored.
    let ids: Vec<&str> = registry::SECTIONS
        .iter()
        .filter(|s| b.ids.iter().any(|i| i == s.id))
        .map(|s| s.id)
        .collect();
    saved::mark_seen(&state, &ids).await;
    Json(json!({ "ok": true })).into_response()
}

/// `GET /page.php?name=roses`: one of the owner's own pages. Its forms post to the same
/// address (`index::home_post`, see main.rs).
pub async fn owner_page(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let id = format!("p-{}", q.get("name").map(String::as_str).unwrap_or(""));
    if !layout::page_id_ok(&id) || saved::current(&state, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Html("<!DOCTYPE html><meta charset=\"utf-8\"><p>No such page. · 그런 페이지는 없어요. <a href=\"index.php\">Home · 처음으로</a></p>"),
        )
            .into_response();
    }
    let zip = q.get("zip").map(String::as_str).unwrap_or("");
    Html(super::render_page(&state, &visitor, &id, zip).await).into_response()
}
