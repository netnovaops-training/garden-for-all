//! Saved layouts in the database: versions, undo, reset, and a backup file before each save
//! (DESIGN.md section 6).
//!
//! HOW TO READ THIS FILE. Every change (save, undo, reset) ADDS a new version row; nothing is
//! overwritten. The newest version of a page is what the page shows; no row at all means the
//! default layout. `current` is cached in memory and the cache is dropped on every change.

use std::collections::HashSet;

use serde_json::{json, Value};
use sqlx::Row;

use super::layout::{self, Layout};
use crate::AppState;

/// Versions kept per page (older rows are removed), and backup files kept per page.
const KEEP_VERSIONS: i32 = 200;
const KEEP_BACKUPS: usize = 200;

/// Create the two tables if they are not there yet (called at start-up).
pub async fn ensure_tables(db: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
    for sql in [
        "CREATE TABLE IF NOT EXISTS `page_layouts` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            page VARCHAR(64) NOT NULL,
            version INT NOT NULL,
            layout MEDIUMTEXT NOT NULL,
            saved_by VARCHAR(100) NOT NULL,
            note VARCHAR(40) NOT NULL DEFAULT 'save',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY page_version (page, version)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `page_builder_seen` (
            section_id VARCHAR(64) NOT NULL PRIMARY KEY,
            seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    ] {
        sqlx::query(sql).execute(db).await?;
    }
    Ok(())
}

/// The page's newest saved layout and its version, or None (= the default layout).
pub async fn current(state: &AppState, page: &str) -> Option<(i32, Layout)> {
    if let Some(hit) = state
        .pagebuilder
        .cache
        .lock()
        .ok()
        .and_then(|c| c.get(page).cloned())
    {
        return hit;
    }
    let row = sqlx::query(
        "SELECT version, layout FROM page_layouts WHERE page = ? ORDER BY version DESC LIMIT 1",
    )
    .bind(page)
    .fetch_optional(&state.db)
    .await;
    let found = match row {
        Ok(r) => r.and_then(|r| {
            let v: i32 = r.try_get("version").ok()?;
            let text: String = r.try_get("layout").ok()?;
            let l: Layout = serde_json::from_str(&text).ok()?;
            Some((v, layout::sanitize(l, page)))
        }),
        // No table, or the database is away: show the default page, do not cache.
        Err(e) => {
            tracing::warn!("page layout for {page} not read: {e}");
            return None;
        }
    };
    if let Ok(mut c) = state.pagebuilder.cache.lock() {
        c.insert(page.to_string(), found.clone());
    }
    found
}

/// One saved version (0 = the default layout), sanitized for drawing.
pub async fn at(state: &AppState, page: &str, version: i32) -> Option<Layout> {
    if version == 0 {
        return Some(layout::default_layout(page));
    }
    let text: String =
        sqlx::query_scalar("SELECT layout FROM page_layouts WHERE page = ? AND version = ?")
            .bind(page)
            .bind(version)
            .fetch_optional(&state.db)
            .await
            .ok()??;
    let l: Layout = serde_json::from_str(&text).ok()?;
    Some(layout::sanitize(l, page))
}

/// The newest 20 versions of a page, for the editor's history list.
pub async fn history(state: &AppState, page: &str) -> Vec<Value> {
    sqlx::query(
        "SELECT version, note, saved_by, DATE_FORMAT(created_at, '%Y-%m-%d %H:%i') AS at
         FROM page_layouts WHERE page = ? ORDER BY version DESC LIMIT 20",
    )
    .bind(page)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .iter()
    .map(|r| {
        json!({
            "version": r.try_get::<i32, _>("version").unwrap_or(0),
            "note": r.try_get::<String, _>("note").unwrap_or_default(),
            "by": r.try_get::<String, _>("saved_by").unwrap_or_default(),
            "at": r.try_get::<String, _>("at").unwrap_or_default(),
        })
    })
    .collect()
}

/// The owner's own pages that have been saved at least once (`p-...`).
pub async fn custom_pages(state: &AppState) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT page FROM page_layouts WHERE page LIKE 'p-%' ORDER BY page",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .filter(|p| layout::page_id_ok(p))
    .collect()
}

/// Why a save did not happen.
#[derive(Debug)]
pub enum SaveError {
    /// Somebody (another editor tab) saved in between; reload first.
    Conflict {
        now: i32,
    },
    Db(sqlx::Error),
}

impl From<sqlx::Error> for SaveError {
    fn from(e: sqlx::Error) -> Self {
        SaveError::Db(e)
    }
}

/// The version a change must start from, and the version it becomes.
pub fn next_version(newest: i32, base: i32) -> Result<i32, SaveError> {
    if newest == base {
        Ok(newest + 1)
    } else {
        Err(SaveError::Conflict { now: newest })
    }
}

/// Store a validated layout as the page's next version. `base` is the version the editor
/// started from (0 = default); a different newest version means another save came first.
/// The current layout is written to a backup file before anything changes.
pub async fn save(
    state: &AppState,
    page: &str,
    l: &Layout,
    base: i32,
    user: &str,
    note: &str,
) -> Result<i32, SaveError> {
    // A Layout is plain data: turning it into JSON cannot fail.
    let json = serde_json::to_string(l).unwrap_or_default();
    let mut tx = state.db.begin().await?;
    let newest: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM page_layouts WHERE page = ? FOR UPDATE",
    )
    .bind(page)
    .fetch_one(&mut *tx)
    .await?;
    let version = next_version(newest, base)?;
    // Backup first: the layout that is about to be replaced (the default if none was saved).
    let before = if newest == 0 {
        Some(layout::default_layout(page))
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT layout FROM page_layouts WHERE page = ? AND version = ?",
        )
        .bind(page)
        .bind(newest)
        .fetch_optional(&mut *tx)
        .await?
        .and_then(|t| serde_json::from_str(&t).ok())
    };
    if let Some(before) = before {
        backup(state, page, newest, &before).await;
    }
    sqlx::query(
        "INSERT INTO page_layouts (page, version, layout, saved_by, note) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(page)
    .bind(version)
    .bind(&json)
    .bind(user)
    .bind(note)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM page_layouts WHERE page = ? AND version <= ?")
        .bind(page)
        .bind(version - KEEP_VERSIONS)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    forget(state, page);
    Ok(version)
}

/// Drop the cached layout of a page (after any change).
pub fn forget(state: &AppState, page: &str) {
    if let Ok(mut c) = state.pagebuilder.cache.lock() {
        c.remove(page);
    }
}

/// Write `<data>/page-layouts/<page>-v<version>-<unix time>.json` and keep the newest
/// `KEEP_BACKUPS` of that page. A failed backup is logged; it does not stop the save.
async fn backup(state: &AppState, page: &str, version: i32, l: &Layout) {
    let dir = state.data.join("page-layouts");
    let page = page.to_string();
    let body = serde_json::to_vec_pretty(l).unwrap_or_default();
    crate::guard::off_thread(move || {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return tracing::error!("page layout backup folder: {e}");
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let name = format!("{page}-v{version}-{stamp}.json");
        match crate::store::file_in(&dir, &name) {
            Ok(p) => {
                if let Err(e) = crate::store::write_atomic(&p, &body) {
                    tracing::error!("page layout backup {name}: {e}");
                }
            }
            Err(e) => tracing::error!("page layout backup {name}: {e}"),
        }
        // Keep the newest backups of this page (names sort by version, then time).
        let prefix = format!("{page}-v");
        let mut mine: Vec<(i64, String)> = std::fs::read_dir(&dir)
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| e.file_name().into_string().ok())
                    .filter_map(|n| {
                        let v = n.strip_prefix(&prefix)?.split('-').next()?.parse().ok()?;
                        Some((v, n))
                    })
                    .collect()
            })
            .unwrap_or_default();
        mine.sort();
        let extra = mine.len().saturating_sub(KEEP_BACKUPS);
        for (_, old) in mine.into_iter().take(extra) {
            if let Ok(p) = crate::store::file_in(&dir, &old) {
                let _ = std::fs::remove_file(p.as_path());
            }
        }
    })
    .await
}

/// Section ids the owner has already been shown (no "new" badge).
pub async fn seen(state: &AppState) -> Option<HashSet<String>> {
    sqlx::query_scalar::<_, String>("SELECT section_id FROM page_builder_seen")
        .fetch_all(&state.db)
        .await
        .ok()
        .map(|v| v.into_iter().collect())
}

/// Mark sections as seen (only ids from the registry are ever stored).
pub async fn mark_seen(state: &AppState, ids: &[&str]) {
    for id in ids {
        let _ = sqlx::query("INSERT IGNORE INTO page_builder_seen (section_id) VALUES (?)")
            .bind(id)
            .execute(&state.db)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_change_must_start_from_the_newest_version() {
        assert_eq!(next_version(0, 0).unwrap(), 1);
        assert_eq!(next_version(4, 4).unwrap(), 5);
        assert!(matches!(
            next_version(5, 4),
            Err(SaveError::Conflict { now: 5 })
        ));
        assert!(matches!(
            next_version(0, 3),
            Err(SaveError::Conflict { now: 0 })
        ));
    }
}
