//! The editor's Features API: look for new features, see what they would change, download,
//! install, turn on, roll back, verify. Owner only, like the rest of the editor.
//!
//! The order the owner sees is the order of the checks: `check` reads only the small signed
//! list; `download` fetches one pack's files into the database as "staged" and shows PASS or
//! FAIL per file (a FAIL keeps nothing); `install` re-checks and swaps it in as one step, keeping
//! the replaced version for `rollback`; `site/save` turns parts of installed packs on or off
//! (with undo). `verify` re-checks everything kept against the signed list.

#![allow(clippy::result_large_err)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use super::editor::{body, fail, owner_only, owner_post};
use super::packs::source::{self, Checked, Https, Source};
use super::packs::store::{self, InstallError, Row};
use super::packs::{self, css, svg, Pack, Problem, Provide};
use super::site::{self, Loaded};
use crate::session::Visitor;
use crate::AppState;

/// How long the last look at the list can be used for downloads.
const CHECK_TTL: Duration = Duration::from_secs(30 * 60);
/// The wizard's downloaded bundle, in the garden's data folder (a fixed name: no path is ever
/// made from a request).
const WIZARD_BUNDLE: &str = "packs/bundle.json";

fn problem_response(p: &Problem, status: StatusCode) -> Response {
    fail(status, &p.en, &p.ko)
}

/// The sources in use: the owner's saved list, else the garden's defaults.
async fn sources(state: &AppState) -> Vec<Source> {
    store::setting(state, "packs_sources")
        .await
        .and_then(|t| serde_json::from_str::<Vec<Source>>(&t).ok())
        .filter(|v| !v.is_empty() && v.iter().all(|s| source::source_ok(s).is_ok()))
        .unwrap_or_else(|| state.pagebuilder.default_sources.clone())
}

/// What a part does to the site, in plain words (English, Korean).
fn describe(p: &Provide, pack: &Pack, files: &HashMap<String, String>) -> (String, String) {
    let rules = |file: &str| {
        files
            .get(file)
            .and_then(|t| css::sanitize(t, &pack.id, "pk-x").ok())
            .map_or(0, |c| c.matches('{').count())
    };
    match p {
        Provide::Header {
            builtin: true, ..
        } => (
            "Uses the site's own waves at the top and the bottom (as the site looks today).".into(),
            "위와 아래에 사이트 기본 물결을 써요 (지금 사이트 모습).".into(),
        ),
        Provide::Header {
            top,
            bottom,
            hero,
            css: c,
            ..
        } => {
            let mut en = String::from("Replaces the wave under the top bar");
            let mut ko = String::from("맨 위 막대 아래 물결");
            if bottom.is_some() {
                en.push_str(", the wave above the footer");
                ko.push_str(", 바닥글 위 물결");
            }
            if hero.is_some() {
                en.push_str(" and the edge of the photo banner");
                ko.push_str(", 사진 배너 가장자리");
            }
            let _ = top;
            en.push_str(" with its own drawing");
            ko.push_str("을(를) 이 기능의 그림으로 바꿔요");
            if let Some(c) = c {
                en.push_str(&format!(" and {} style rules for it", rules(c)));
                ko.push_str(&format!(" (스타일 규칙 {}개)", rules(c)));
            }
            (en + ".", ko + ".")
        }
        Provide::CatalogLayout { css: c, .. } => (
            format!("Changes how the plant cards look wherever the \"Catalog: plant cards\" section is ({} style rules). Nothing else on the site.", rules(c)),
            format!("\"카탈로그: 식물 카드\" 섹션이 있는 곳의 식물 카드 모습을 바꿔요 (스타일 규칙 {}개). 사이트의 다른 곳은 그대로예요.", rules(c)),
        ),
        Provide::Theme { vars, .. } => (
            format!("Changes {} site colour(s) on every page.", vars.len()),
            format!("모든 페이지의 사이트 색 {}개를 바꿔요.", vars.len()),
        ),
        Provide::Section { .. } => (
            "Adds a section you can put on any page. It shows only text and links.".into(),
            "어느 페이지에나 놓을 수 있는 섹션을 더해요. 글과 링크만 보여요.".into(),
        ),
    }
}

fn parts_json(pack: &Pack, files: &HashMap<String, String>) -> Vec<Value> {
    pack.provides
        .iter()
        .map(|p| {
            let (en, ko) = p.names();
            let (what_en, what_ko) = describe(p, pack, files);
            let mut v = json!({
                "kind": p.kind(), "id": p.id(), "name_en": en, "name_ko": ko,
                "what_en": what_en, "what_ko": what_ko,
            });
            // The art, after the strict reader (so the preview shows what would be used).
            if let Provide::Header {
                top, bottom, hero, ..
            } = p
            {
                let art: Vec<Value> = [("top", top), ("bottom", bottom), ("hero", hero)]
                    .into_iter()
                    .filter_map(|(slot, f)| {
                        let t = files.get(f.as_deref()?)?;
                        Some(match svg::sanitize(t, &pack.id) {
                            Ok(s) => json!({ "slot": slot, "svg": s }),
                            Err(e) => json!({ "slot": slot, "error": e.en, "error_ko": e.ko }),
                        })
                    })
                    .collect();
                v["art"] = json!(art);
            }
            v
        })
        .collect()
}

/// One pack of a checked list, as the owner first sees it (before any download).
fn list_item(pack: &Pack, installed: Option<&Row>) -> Value {
    let status = match installed {
        None => "new",
        Some(r)
            if r.pack.version == pack.version
                && r.pack.files.len() == pack.files.len()
                && r.pack
                    .files
                    .iter()
                    .zip(&pack.files)
                    .all(|(a, b)| a.sha256 == b.sha256) =>
        {
            "same"
        }
        Some(r) if packs::version_key(&pack.version) < packs::version_key(&r.pack.version) => {
            "older"
        }
        Some(_) => "changed",
    };
    let (empty, files): (HashMap<String, String>, _) = (HashMap::new(), &pack.files);
    json!({
        "id": pack.id, "version": pack.version, "status": status,
        "installed_version": installed.map(|r| r.pack.version.clone()),
        "name_en": pack.name_en, "name_ko": pack.name_ko,
        "desc_en": pack.desc_en, "desc_ko": pack.desc_ko,
        "bytes": files.iter().map(|f| f.bytes).sum::<usize>(),
        "files": files.iter().map(|f| json!({ "path": f.path, "bytes": f.bytes, "sha256": f.sha256 })).collect::<Vec<_>>(),
        "parts": pack.provides.iter().map(|p| {
            let (en, ko) = p.names();
            let (what_en, what_ko) = describe(p, pack, &empty);
            json!({ "kind": p.kind(), "id": p.id(), "name_en": en, "name_ko": ko, "what_en": what_en, "what_ko": what_ko })
        }).collect::<Vec<_>>(),
    })
}

/// The answer to "look for new features" (and to reading the wizard's bundle).
async fn list_answer(state: &AppState, c: &Checked, notes: &[(String, Problem)]) -> Value {
    let installed = store::rows(state, "installed").await;
    let items: Vec<Value> = c
        .index
        .packs
        .iter()
        .map(|p| list_item(p, installed.iter().find(|r| r.pack.id == p.id)))
        .collect();
    let removed: Vec<Value> = installed
        .iter()
        .filter(|r| !c.index.packs.iter().any(|p| p.id == r.pack.id))
        .map(|r| json!({ "id": r.pack.id, "version": r.pack.version, "name_en": r.pack.name_en, "name_ko": r.pack.name_ko }))
        .collect();
    json!({
        "ok": true,
        "source": c.source.name, "signed": c.signed, "released": c.index.released,
        "serial": c.index.serial, "packs": items, "removed": removed,
        "notes": notes.iter().map(|(n, p)| json!({ "source": n, "why": p.en, "why_ko": p.ko })).collect::<Vec<_>>(),
    })
}

fn remember(state: &AppState, c: &Checked) {
    if let Ok(mut m) = state.pagebuilder.checked.lock() {
        *m = Some((Instant::now(), Arc::new(c.clone())));
    }
}

async fn remember_serial(state: &AppState, c: &Checked) {
    // Only a list the author signed may move the "not older than" mark.
    if c.signed && c.index.serial > store::last_serial(state).await {
        let _ = store::set_setting(state, "packs_serial", &c.index.serial.to_string()).await;
    }
}

// ---------------------------------------------------------------- handlers

/// `GET /pagebuilder/api/features`: what is installed, downloaded and chosen.
pub async fn api_features(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
) -> Response {
    if let Err(r) = owner_only(&state, &visitor) {
        return r;
    }
    let (version, st) = site::current(&state).await;
    let installed: Vec<Value> = store::rows(&state, "installed")
        .await
        .iter()
        .map(|r| serde_json::to_value(store::summary(r)).unwrap_or_default())
        .collect();
    let previous: Vec<Value> = store::rows(&state, "previous")
        .await
        .iter()
        .map(|r| json!({ "id": r.pack.id, "version": r.pack.version }))
        .collect();
    let staged: Vec<Value> = store::rows(&state, "staged")
        .await
        .iter()
        .map(|r| serde_json::to_value(store::summary(r)).unwrap_or_default())
        .collect();
    let bundle_waiting = tokio::fs::metadata(state.data.join(WIZARD_BUNDLE))
        .await
        .is_ok();
    Json(json!({
        "installed": installed, "previous": previous, "staged": staged,
        "site": { "version": version, "state": st, "history": site::history(&state).await },
        "sources": sources(&state).await,
        "key_set": state.pagebuilder.packs_key.is_some(),
        "serial": store::last_serial(&state).await,
        "bundle_waiting": bundle_waiting,
    }))
    .into_response()
}

/// `POST /pagebuilder/api/features/check`: read the signed list from the sources (one try per
/// click; the second source only when the first fails). Nothing is downloaded or kept.
pub async fn api_check(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let srcs = sources(&state).await;
    let last = store::last_serial(&state).await;
    match source::fetch_first(
        &Https::new(),
        &srcs,
        state.pagebuilder.packs_key.as_ref(),
        last,
    )
    .await
    {
        Ok((c, notes)) => {
            remember(&state, &c);
            remember_serial(&state, &c).await;
            Json(list_answer(&state, &c, &notes).await).into_response()
        }
        Err(p) => problem_response(&p, StatusCode::BAD_GATEWAY),
    }
}

/// `POST /pagebuilder/api/features/import`: read the bundle the wizard downloaded (the same
/// checks as a download from the web).
pub async fn api_import(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let path = state.data.join(WIZARD_BUNDLE);
    let bytes = match tokio::fs::read(&path).await {
        Ok(b) if b.len() <= source::MAX_BUNDLE_BYTES => b,
        Ok(_) => {
            return fail(
                StatusCode::BAD_REQUEST,
                "The wizard's file is too big.",
                "마법사 파일이 너무 커요.",
            )
        }
        Err(_) => {
            return fail(
                StatusCode::NOT_FOUND,
                "The wizard has not downloaded anything yet.",
                "마법사가 아직 아무것도 내려받지 않았어요.",
            )
        }
    };
    let src = Source {
        name: "wizard".into(),
        kind: "bundle".into(),
        url: String::new(),
    };
    let last = store::last_serial(&state).await;
    match source::read_bundle(&bytes, src, state.pagebuilder.packs_key.as_ref(), last) {
        Ok(c) => {
            remember(&state, &c);
            remember_serial(&state, &c).await;
            Json(list_answer(&state, &c, &[]).await).into_response()
        }
        Err(p) => problem_response(&p, StatusCode::BAD_REQUEST),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdBody {
    id: String,
    #[serde(default)]
    accept_unsigned: bool,
}

fn id_arg(id: &str) -> Result<(), Response> {
    if packs::id_ok(id) {
        Ok(())
    } else {
        Err(fail(
            StatusCode::BAD_REQUEST,
            "Unknown feature.",
            "알 수 없는 기능이에요.",
        ))
    }
}

/// `POST /pagebuilder/api/features/download`: fetch one feature's files, check each against
/// the list, and keep them only if all pass.
pub async fn api_download(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: IdBody = body(&bytes)?;
        id_arg(&b.id)?;
        let checked = state
            .pagebuilder
            .checked
            .lock()
            .ok()
            .and_then(|c| c.clone())
            .filter(|(at, _)| at.elapsed() < CHECK_TTL)
            .map(|(_, c)| c)
            .ok_or_else(|| {
                fail(
                    StatusCode::CONFLICT,
                    "Look for new features again first (the list is too old).",
                    "먼저 새 기능을 다시 찾아 보세요 (목록이 너무 오래됐어요).",
                )
            })?;
        let d = source::download_pack(&Https::new(), &checked, &b.id)
            .await
            .map_err(|p| problem_response(&p, StatusCode::BAD_REQUEST))?;
        if d.ok {
            if let Err(e) = store::stage(&state, &checked, &d, &visitor.username).await {
                tracing::error!("feature not kept: {e}");
                return Err(fail(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Not kept: the database did not answer.",
                    "저장하지 못했어요: 데이터베이스가 응답하지 않아요.",
                ));
            }
        }
        Ok::<_, Response>(
            Json(json!({ "ok": d.ok, "report": d.report, "signed": checked.signed }))
                .into_response(),
        )
    };
    run.await.unwrap_or_else(|r| r)
}

/// `GET /pagebuilder/api/features/review?id=`: what a downloaded feature would change, and the
/// result of every check on it, before the owner decides.
pub async fn api_review(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    if let Err(r) = owner_only(&state, &visitor) {
        return r;
    }
    let id = q.get("id").map(String::as_str).unwrap_or("");
    if let Err(r) = id_arg(id) {
        return r;
    }
    let Some(r) = store::row(&state, id, "staged").await else {
        return fail(
            StatusCode::NOT_FOUND,
            "Nothing downloaded for that feature.",
            "그 기능은 내려받은 것이 없어요.",
        );
    };
    let installed = store::row(&state, id, "installed").await;
    let (report, sig) = store::verify_row(&r, state.pagebuilder.packs_key.as_ref());
    let files: HashMap<String, String> = r
        .files
        .iter()
        .map(|(k, v)| (k.clone(), String::from_utf8_lossy(v).into_owned()))
        .collect();
    Json(json!({
        "id": r.pack.id, "version": r.pack.version,
        "name_en": r.pack.name_en, "name_ko": r.pack.name_ko,
        "desc_en": r.pack.desc_en, "desc_ko": r.pack.desc_ko,
        "source": r.source, "signed": r.signed,
        "signature": match &sig { Ok(true) => "signed", Ok(false) => "unsigned", Err(_) => "fail" },
        "signature_why": sig.as_ref().err().map(|p| p.en.clone()),
        "signature_why_ko": sig.as_ref().err().map(|p| p.ko.clone()),
        "status": list_item(&r.pack, installed.as_ref())["status"],
        "installed_version": installed.map(|i| i.pack.version),
        "report": report,
        "parts": parts_json(&r.pack, &files),
    }))
    .into_response()
}

/// `POST /pagebuilder/api/features/install`: re-check, then install as one step (the replaced
/// version is kept for a roll back).
pub async fn api_install(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: IdBody = body(&bytes)?;
        id_arg(&b.id)?;
        match store::install(&state, &b.id, &visitor.username, b.accept_unsigned).await {
            Ok(pack) => {
                site::forget(&state);
                Ok::<_, Response>(
                    Json(json!({ "ok": true, "id": pack.id, "version": pack.version }))
                        .into_response(),
                )
            }
            Err(e) => Err(install_error(e)),
        }
    };
    run.await.unwrap_or_else(|r| r)
}

fn install_error(e: InstallError) -> Response {
    match e {
        InstallError::Refused(p) => problem_response(&p, StatusCode::BAD_REQUEST),
        InstallError::Db(e) => {
            tracing::error!("install: {e}");
            fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Not installed: the database did not answer.",
                "설치하지 못했어요: 데이터베이스가 응답하지 않아요.",
            )
        }
    }
}

/// `POST /pagebuilder/api/features/rollback`: go back to the kept earlier version (again: forward).
pub async fn api_rollback(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: IdBody = body(&bytes)?;
        id_arg(&b.id)?;
        match store::rollback(&state, &b.id).await {
            Ok(pack) => {
                site::forget(&state);
                Ok::<_, Response>(
                    Json(json!({ "ok": true, "version": pack.version })).into_response(),
                )
            }
            Err(e) => Err(install_error(e)),
        }
    };
    run.await.unwrap_or_else(|r| r)
}

/// `POST /pagebuilder/api/features/discard`: throw a downloaded feature away.
pub async fn api_discard(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: IdBody = body(&bytes)?;
        id_arg(&b.id)?;
        store::discard_staged(&state, &b.id).await;
        Ok::<_, Response>(Json(json!({ "ok": true })).into_response())
    };
    run.await.unwrap_or_else(|r| r)
}

/// `POST /pagebuilder/api/features/remove`: take an installed feature away (not while it is on).
pub async fn api_remove(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: IdBody = body(&bytes)?;
        id_arg(&b.id)?;
        let (_, st) = site::current(&state).await;
        if st.enabled.contains(&b.id) {
            return Err(fail(
                StatusCode::CONFLICT,
                "Turn the feature off first.",
                "먼저 그 기능을 끄세요.",
            ));
        }
        let _ = store::remove(&state, &b.id).await;
        site::forget(&state);
        Ok::<_, Response>(Json(json!({ "ok": true })).into_response())
    };
    run.await.unwrap_or_else(|r| r)
}

/// `POST /pagebuilder/api/features/verify`: check every kept feature again (files against the
/// signed list, the list against the author's key, and what was recorded at install).
pub async fn api_verify(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let mut out = Vec::new();
    for status in ["installed", "previous"] {
        for r in store::rows(&state, status).await {
            let (report, sig) = store::verify_row(&r, state.pagebuilder.packs_key.as_ref());
            let files_ok = report.iter().all(|f| f.ok);
            out.push(json!({
                "id": r.pack.id, "version": r.pack.version, "status": status,
                "files": report, "files_ok": files_ok,
                "signature": match &sig { Ok(true) => "signed", Ok(false) => "unsigned", Err(_) => "fail" },
                "signature_why": sig.as_ref().err().map(|p| p.en.clone()),
                "signature_why_ko": sig.as_ref().err().map(|p| p.ko.clone()),
                "ok": files_ok && sig.is_ok(),
            }));
        }
    }
    Json(json!({ "ok": out.iter().all(|v| v["ok"] == true), "packs": out })).into_response()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcesBody {
    sources: Vec<Source>,
}

/// `POST /pagebuilder/api/features/sources`: where to look for new features (a short list;
/// the first that answers is used). An empty list goes back to the defaults.
pub async fn api_sources(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: SourcesBody = body(&bytes)?;
        if b.sources.len() > 5 {
            return Err(fail(
                StatusCode::BAD_REQUEST,
                "At most 5 sources.",
                "출처는 5개까지예요.",
            ));
        }
        for s in &b.sources {
            source::source_ok(s).map_err(|p| problem_response(&p, StatusCode::BAD_REQUEST))?;
        }
        let text = if b.sources.is_empty() {
            String::new()
        } else {
            serde_json::to_string(&b.sources).unwrap_or_default()
        };
        store::set_setting(&state, "packs_sources", &text)
            .await
            .map_err(|_| {
                fail(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Not saved.",
                    "저장하지 못했어요.",
                )
            })?;
        Ok::<_, Response>(
            Json(json!({ "ok": true, "sources": sources(&state).await })).into_response(),
        )
    };
    run.await.unwrap_or_else(|r| r)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SiteBody {
    base: i32,
    state: site::State,
}

fn site_answer(r: Result<i32, site::SaveError>) -> Response {
    match r {
        Ok(v) => Json(json!({ "ok": true, "version": v })).into_response(),
        Err(site::SaveError::Conflict { now }) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "The site settings were changed in another window. Reload to see them.",
                "error_ko": "다른 창에서 사이트 설정이 바뀌었어요. 새로 고쳐서 확인하세요.",
                "now": now,
            })),
        )
            .into_response(),
        Err(site::SaveError::Db(e)) => {
            tracing::error!("site settings not saved: {e}");
            fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Not saved: the database did not answer.",
                "저장하지 못했어요: 데이터베이스가 응답하지 않아요.",
            )
        }
    }
}

/// The installed packs as `Loaded` (for checking a choice).
async fn installed_loaded(state: &AppState) -> Vec<Loaded> {
    site::loaded(state).await
}

/// `POST /pagebuilder/api/site/save`: turn features on or off and pick header, catalog layout
/// and colours from them. Stored as a new version, so it can be undone.
pub async fn api_site_save(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: SiteBody = body(&bytes)?;
        site::validate(&b.state, &installed_loaded(&state).await)
            .map_err(|p| problem_response(&p, StatusCode::BAD_REQUEST))?;
        Ok::<_, Response>(site_answer(
            site::save(&state, &b.state, b.base, &visitor.username, "save").await,
        ))
    };
    run.await.unwrap_or_else(|r| r)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UndoBody {
    base: i32,
}

/// `POST /pagebuilder/api/site/undo`: the settings as they were one version ago, as a new version.
pub async fn api_site_undo(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(r) = owner_post(&state, &visitor, &headers) {
        return r;
    }
    let run = async {
        let b: UndoBody = body(&bytes)?;
        if b.base < 1 {
            return Err(fail(
                StatusCode::BAD_REQUEST,
                "There is nothing to undo.",
                "되돌릴 것이 없어요.",
            ));
        }
        let Some(mut before) = site::at(&state, b.base - 1).await else {
            return Err(fail(
                StatusCode::NOT_FOUND,
                "That version is no longer kept.",
                "그 버전은 더 이상 보관되어 있지 않아요.",
            ));
        };
        // Things removed since then are dropped from the old choices instead of failing.
        let have = installed_loaded(&state).await;
        before
            .enabled
            .retain(|id| have.iter().any(|l| l.pack.id == *id));
        for pick in [
            &mut before.header,
            &mut before.catalog_layout,
            &mut before.theme,
        ] {
            if pick.as_ref().is_some_and(|p| {
                !p.split_once('/')
                    .is_some_and(|(pack, _)| before.enabled.iter().any(|e| e == pack))
            }) {
                *pick = None;
            }
        }
        Ok::<_, Response>(site_answer(
            site::save(&state, &before, b.base, &visitor.username, "undo").await,
        ))
    };
    run.await.unwrap_or_else(|r| r)
}
