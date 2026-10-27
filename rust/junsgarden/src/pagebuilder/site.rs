//! The site-wide choices made from feature packs, and the result that pages are drawn with.
//!
//! HOW TO READ THIS FILE. `State` is what the owner chose: which installed packs are ON, and
//! which header art, catalog layout and colour theme are picked from them. It is saved as
//! versions in `page_site_state` (every change adds a version; undo adds a copy of the one
//! before), like page layouts. `build_active` turns a State plus the installed packs into
//! `Active`: ready-to-print art, one style block and the pack sections. It runs every file
//! through the strict readers again, and a part that fails is left out (logged), never half
//! used. `active` caches the result; it is dropped on every change.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::layout::{Extra, ExtraDef};
use super::packs::{blocks, css, scope_class, section_id, store, svg, Pack, Problem, Provide};
use crate::AppState;

const KEEP_VERSIONS: i32 = 100;

/// What the owner chose.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub schema: u32,
    /// Packs that are ON (their sections can be placed; their parts can be picked).
    #[serde(default)]
    pub enabled: Vec<String>,
    /// `<pack>/<part>` of the header art, or none for the site's own waves.
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub catalog_layout: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
}

/// A pack that is installed, with its files as text (already checked when it was kept).
#[derive(Clone, Debug)]
pub struct Loaded {
    pub pack: Pack,
    pub files: HashMap<String, String>,
}

/// A section a pack brings, drawn from its blocks.
#[derive(Clone, Debug)]
pub struct ActiveSection {
    pub name_en: String,
    pub name_ko: String,
    pub desc_en: String,
    pub desc_ko: String,
    pub html: String,
}

/// What pages are drawn with.
#[derive(Clone, Debug, Default)]
pub struct Active {
    /// The header's under-edge, the footer's top edge and the photo banner's edge, as ready
    /// markup; `None` = the site's own waves.
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub hero: Option<String>,
    /// One `<style>` block (or empty) for `<head>`.
    pub head: String,
    /// The class that wraps the catalog's cards section when a layout is picked.
    pub catalog_class: Option<String>,
    pub sections: HashMap<String, ActiveSection>,
    pub extra: Extra,
}

fn text(l: &Loaded, name: &str) -> Option<String> {
    l.files.get(name).cloned()
}

fn find<'a>(packs: &'a [Loaded], pick: &Option<String>) -> Option<(&'a Loaded, &'a Provide)> {
    let (pack, part) = pick.as_deref()?.split_once('/')?;
    let l = packs.iter().find(|l| l.pack.id == pack)?;
    let p = l.pack.provides.iter().find(|p| p.id() == part)?;
    Some((l, p))
}

fn skip(what: &str, pack: &str, why: &Problem) {
    tracing::warn!("pack {pack}: {what} left out: {}", why.en);
}

/// The result of a State and the installed packs. Pure: no database, no clock.
pub fn build_active(st: &State, packs: &[Loaded]) -> Active {
    let on: Vec<&Loaded> = packs
        .iter()
        .filter(|l| st.enabled.contains(&l.pack.id))
        .collect();
    let mut a = Active::default();
    let mut styles = String::new();
    // Sections of every pack that is on.
    for l in &on {
        for p in &l.pack.provides {
            let Provide::Section {
                id,
                name_en,
                name_ko,
                desc_en,
                desc_ko,
                template,
            } = p
            else {
                continue;
            };
            let parsed = text(l, template)
                .ok_or_else(|| super::packs::problem("missing", "없음"))
                .and_then(|t| blocks::parse(&t));
            match parsed {
                Ok(b) => {
                    let sid = section_id(&l.pack.id, id);
                    a.extra.insert(
                        sid.clone(),
                        ExtraDef {
                            name_en: name_en.clone(),
                            name_ko: name_ko.clone(),
                        },
                    );
                    a.sections.insert(
                        sid,
                        ActiveSection {
                            name_en: name_en.clone(),
                            name_ko: name_ko.clone(),
                            desc_en: desc_en.clone(),
                            desc_ko: desc_ko.clone(),
                            html: blocks::render(&b),
                        },
                    );
                }
                Err(e) => skip("a section", &l.pack.id, &e),
            }
        }
    }
    // Header art (only from a pack that is on).
    if let Some((
        l,
        Provide::Header {
            id,
            builtin,
            top,
            bottom,
            hero,
            css: css_file,
            ..
        },
    )) = find(packs, &st.header).filter(|(l, _)| st.enabled.contains(&l.pack.id))
    {
        if !builtin {
            let scope = scope_class(&l.pack.id, id);
            let art = |file: &Option<String>| -> Option<String> {
                let t = text(l, file.as_deref()?)?;
                match svg::sanitize(&t, &l.pack.id) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        skip("art", &l.pack.id, &e);
                        None
                    }
                }
            };
            let (t, b, h) = (art(top), art(bottom), art(hero));
            if t.is_some() || b.is_some() || h.is_some() {
                a.top = t.map(|s| format!("<div class=\"edge-wave edge-wave--header pk-art {scope}\" aria-hidden=\"true\">{s}</div>"));
                a.bottom = b.map(|s| format!("<div class=\"edge-wave edge-wave--footer pk-art {scope}\" aria-hidden=\"true\">{s}</div>"));
                a.hero = h.map(|s| {
                    format!(
                        "<div class=\"hero-wave pk-art {scope}\" aria-hidden=\"true\">{s}</div>"
                    )
                });
                if let Some(c) = css_file.as_deref().and_then(|f| text(l, f)) {
                    match css::sanitize(&c, &l.pack.id, &scope) {
                        Ok(s) => styles.push_str(&s),
                        Err(e) => skip("header styles", &l.pack.id, &e),
                    }
                }
            }
        }
    }
    // Catalog layout.
    if let Some((l, Provide::CatalogLayout { id, css: file, .. })) =
        find(packs, &st.catalog_layout).filter(|(l, _)| st.enabled.contains(&l.pack.id))
    {
        let scope = scope_class(&l.pack.id, id);
        match text(l, file).map(|c| css::sanitize(&c, &l.pack.id, &scope)) {
            Some(Ok(s)) => {
                styles.push_str(&s);
                a.catalog_class = Some(scope);
            }
            Some(Err(e)) => skip("catalog layout", &l.pack.id, &e),
            None => {}
        }
    }
    // Colour theme.
    if let Some((l, Provide::Theme { vars, .. })) =
        find(packs, &st.theme).filter(|(l, _)| st.enabled.contains(&l.pack.id))
    {
        match css::theme(vars) {
            Ok(s) => styles.push_str(&s),
            Err(e) => skip("theme", &l.pack.id, &e),
        }
    }
    if !styles.is_empty() {
        a.head = format!("\n<style id=\"pb-pack-style\">\n{styles}</style>\n");
    }
    a
}

/// Is this State allowed, given what is installed? (Every name must exist.)
pub fn validate(st: &State, installed: &[Loaded]) -> Result<(), Problem> {
    let bad = |en: &str, ko: &str| Err(super::packs::problem(en, ko));
    if st.schema != 1 || st.enabled.len() > 40 {
        return bad("Unknown settings format.", "알 수 없는 설정 형식이에요.");
    }
    for (i, id) in st.enabled.iter().enumerate() {
        if !installed.iter().any(|l| l.pack.id == *id) || st.enabled[..i].contains(id) {
            return bad(
                "A feature that is not installed cannot be turned on.",
                "설치되지 않은 기능은 켤 수 없어요.",
            );
        }
    }
    for (pick, kind) in [
        (&st.header, "header"),
        (&st.catalog_layout, "catalog_layout"),
        (&st.theme, "theme"),
    ] {
        let Some(p) = pick else { continue };
        let ok = p.split_once('/').is_some_and(|(pack, part)| {
            st.enabled.iter().any(|e| e == pack)
                && installed.iter().any(|l| {
                    l.pack.id == pack
                        && l.pack
                            .provides
                            .iter()
                            .any(|x| x.id() == part && x.kind() == kind)
                })
        });
        if !ok {
            return bad(
                "That choice is not from a feature that is turned on.",
                "그 선택은 켜져 있는 기능에서 온 것이 아니에요.",
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- database

/// The installed packs, as text (a pack whose stored files fail their check is left out).
pub async fn loaded(state: &AppState) -> Vec<Loaded> {
    let key = state.pagebuilder.packs_key.as_ref();
    store::rows(state, "installed")
        .await
        .into_iter()
        .filter_map(|r| {
            let (report, _) = store::verify_row(&r, key);
            if report.iter().any(|f| !f.ok) {
                tracing::warn!("pack {} not used: a stored file fails its check", r.pack.id);
                return None;
            }
            let files = r
                .files
                .iter()
                .map(|(k, v)| (k.clone(), String::from_utf8_lossy(v).into_owned()))
                .collect();
            Some(Loaded {
                pack: r.pack,
                files,
            })
        })
        .collect()
}

/// The newest saved State and its version (0 = none: everything off).
pub async fn current(state: &AppState) -> (i32, State) {
    let row =
        sqlx::query("SELECT version, state FROM page_site_state ORDER BY version DESC LIMIT 1")
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
    row.and_then(|r| {
        let v: i32 = r.try_get("version").ok()?;
        let s: State = serde_json::from_str(&r.try_get::<String, _>("state").ok()?).ok()?;
        Some((v, s))
    })
    .unwrap_or((
        0,
        State {
            schema: 1,
            ..State::default()
        },
    ))
}

/// The earlier version `base - 1`'s State (for undo).
pub async fn at(state: &AppState, version: i32) -> Option<State> {
    if version == 0 {
        return Some(State {
            schema: 1,
            ..State::default()
        });
    }
    let t: String = sqlx::query_scalar("SELECT state FROM page_site_state WHERE version = ?")
        .bind(version)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    serde_json::from_str(&t).ok()
}

pub async fn history(state: &AppState) -> Vec<serde_json::Value> {
    sqlx::query(
        "SELECT version, note, saved_by, DATE_FORMAT(created_at, '%Y-%m-%d %H:%i') AS at
         FROM page_site_state ORDER BY version DESC LIMIT 10",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .iter()
    .map(|r| {
        serde_json::json!({
            "version": r.try_get::<i32, _>("version").unwrap_or(0),
            "note": r.try_get::<String, _>("note").unwrap_or_default(),
            "by": r.try_get::<String, _>("saved_by").unwrap_or_default(),
            "at": r.try_get::<String, _>("at").unwrap_or_default(),
        })
    })
    .collect()
}

/// A stale editor (another window saved first) is refused, like a page save.
pub enum SaveError {
    Conflict { now: i32 },
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveError {
    fn from(e: sqlx::Error) -> Self {
        SaveError::Db(e)
    }
}

/// Store a validated State as the next version. `base` is the version the owner started from.
pub async fn save(
    state: &AppState,
    st: &State,
    base: i32,
    user: &str,
    note: &str,
) -> Result<i32, SaveError> {
    let mut tx = state.db.begin().await?;
    let newest: i32 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM page_site_state FOR UPDATE")
            .fetch_one(&mut *tx)
            .await?;
    if newest != base {
        return Err(SaveError::Conflict { now: newest });
    }
    let version = newest + 1;
    sqlx::query("INSERT INTO page_site_state (version, state, saved_by, note) VALUES (?, ?, ?, ?)")
        .bind(version)
        .bind(serde_json::to_string(st).unwrap_or_default())
        .bind(user)
        .bind(note)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM page_site_state WHERE version <= ?")
        .bind(version - KEEP_VERSIONS)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    forget(state);
    Ok(version)
}

// ---------------------------------------------------------------- the cache

/// What pages are drawn with now (cached; the database not answering gives "all off" and is
/// not cached).
pub async fn active(state: &AppState) -> Arc<Active> {
    if let Some(hit) = cached(state) {
        return hit;
    }
    let (_, st) = current(state).await;
    let packs = loaded(state).await;
    let a = Arc::new(build_active(&st, &packs));
    if let Ok(mut c) = state.pagebuilder.site.lock() {
        *c = Some(a.clone());
    }
    a
}

pub fn cached(state: &AppState) -> Option<Arc<Active>> {
    state.pagebuilder.site.lock().ok().and_then(|c| c.clone())
}

/// Drop what was worked out (after any change to packs or choices); page layouts are drawn
/// again too, because which pack sections exist may have changed.
pub fn forget(state: &AppState) {
    if let Ok(mut c) = state.pagebuilder.site.lock() {
        *c = None;
    }
    if let Ok(mut c) = state.pagebuilder.cache.lock() {
        c.clear();
    }
}
