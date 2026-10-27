//! Packs in the database: downloaded and checked ("staged"), installed (with the one
//! replaced version kept for a one-click rollback), and a plain re-check of everything kept
//! ("Verify my install"). Pack files are kept as rows, never as files, so no file path is ever
//! made from anything a pack says.
//!
//! HOW TO READ THIS FILE. `status` is `staged`, `installed` or `previous`. Install is one
//! transaction: the old `previous` row goes, the `installed` row becomes `previous`, the staged
//! row becomes `installed`; either all of that happens or none of it. Every row keeps the signed
//! index it came from, so it can be checked against the author's key again at any time.

use std::collections::BTreeMap;

use ed25519_dalek::VerifyingKey;
use serde::Serialize;
use sqlx::Row as _;

use super::source::{check_signature, Checked, Downloaded};
use super::{
    check_files, parse_index, problem, sha256_hex, version_key, FileReport, Pack, Problem,
};
use crate::AppState;

/// One stored pack.
#[derive(Clone, Debug)]
pub struct Row {
    pub id: i64,
    pub status: String,
    pub source: String,
    pub signed: bool,
    pub pack: Pack,
    pub index_text: String,
    pub index_sig: String,
    pub files: Vec<(String, Vec<u8>)>,
    /// sha256 of each file when it was installed.
    pub hashes: BTreeMap<String, String>,
    pub installed_by: String,
}

/// Create the tables if they are not there yet (called at start-up).
pub async fn ensure_tables(db: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
    for sql in [
        "CREATE TABLE IF NOT EXISTS `page_packs` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            pack_id VARCHAR(40) NOT NULL,
            version VARCHAR(20) NOT NULL,
            status VARCHAR(10) NOT NULL,
            source VARCHAR(20) NOT NULL,
            signed TINYINT NOT NULL DEFAULT 0,
            pack MEDIUMTEXT NOT NULL,
            index_text MEDIUMTEXT NOT NULL,
            index_sig VARCHAR(200) NOT NULL,
            files MEDIUMTEXT NOT NULL,
            hashes TEXT NOT NULL,
            staged_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            installed_at TIMESTAMP NULL,
            installed_by VARCHAR(100) NULL,
            KEY pack_status (pack_id, status)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `page_site_state` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            version INT NOT NULL,
            state TEXT NOT NULL,
            saved_by VARCHAR(100) NOT NULL,
            note VARCHAR(40) NOT NULL DEFAULT 'save',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY state_version (version)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `page_builder_settings` (
            k VARCHAR(40) NOT NULL PRIMARY KEY,
            v TEXT NOT NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    ] {
        sqlx::query(sql).execute(db).await?;
    }
    Ok(())
}

// ---------------------------------------------------------------- settings

pub async fn setting(state: &AppState, key: &str) -> Option<String> {
    sqlx::query_scalar::<_, String>("SELECT v FROM page_builder_settings WHERE k = ?")
        .bind(key)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
}

pub async fn set_setting(state: &AppState, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO page_builder_settings (k, v) VALUES (?, ?) ON DUPLICATE KEY UPDATE v = VALUES(v)",
    )
    .bind(key)
    .bind(value)
    .execute(&state.db)
    .await
    .map(|_| ())
}

/// The highest list number the garden has accepted (0 = none yet).
pub async fn last_serial(state: &AppState) -> u64 {
    setting(state, "packs_serial")
        .await
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

// ---------------------------------------------------------------- rows

fn row_of(r: &sqlx::mysql::MySqlRow) -> Option<Row> {
    let pack: Pack = serde_json::from_str(&r.try_get::<String, _>("pack").ok()?).ok()?;
    let files: BTreeMap<String, String> =
        serde_json::from_str(&r.try_get::<String, _>("files").ok()?).ok()?;
    let hashes: BTreeMap<String, String> =
        serde_json::from_str(&r.try_get::<String, _>("hashes").ok()?).ok()?;
    Some(Row {
        id: r.try_get::<i32, _>("id").ok()? as i64,
        status: r.try_get("status").ok()?,
        source: r.try_get("source").ok()?,
        signed: r.try_get::<i8, _>("signed").ok()? != 0,
        pack,
        index_text: r.try_get("index_text").ok()?,
        index_sig: r.try_get("index_sig").ok()?,
        files: files
            .into_iter()
            .map(|(k, v)| (k, v.into_bytes()))
            .collect(),
        hashes,
        installed_by: r
            .try_get::<Option<String>, _>("installed_by")
            .ok()?
            .unwrap_or_default(),
    })
}

/// Every stored pack of one status (`staged`, `installed` or `previous`), by pack name.
pub async fn rows(state: &AppState, status: &str) -> Vec<Row> {
    sqlx::query("SELECT * FROM page_packs WHERE status = ? ORDER BY pack_id")
        .bind(status)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .iter()
        .filter_map(row_of)
        .collect()
}

pub async fn row(state: &AppState, pack_id: &str, status: &str) -> Option<Row> {
    sqlx::query(
        "SELECT * FROM page_packs WHERE pack_id = ? AND status = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(pack_id)
    .bind(status)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .as_ref()
    .and_then(row_of)
}

/// Keep a download that passed every check, waiting for the owner. One staged row per pack.
pub async fn stage(
    state: &AppState,
    checked: &Checked,
    d: &Downloaded,
    user: &str,
) -> Result<(), sqlx::Error> {
    let files: BTreeMap<String, String> = d
        .files
        .iter()
        .map(|(k, v)| (k.clone(), String::from_utf8_lossy(v).into_owned()))
        .collect();
    let hashes: BTreeMap<String, String> = d
        .files
        .iter()
        .map(|(k, v)| (k.clone(), sha256_hex(v)))
        .collect();
    let mut tx = state.db.begin().await?;
    sqlx::query("DELETE FROM page_packs WHERE pack_id = ? AND status = 'staged'")
        .bind(&d.pack.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO page_packs (pack_id, version, status, source, signed, pack, index_text, index_sig, files, hashes, installed_by)
         VALUES (?, ?, 'staged', ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&d.pack.id)
    .bind(&d.pack.version)
    .bind(&checked.source.name)
    .bind(i8::from(checked.signed))
    .bind(serde_json::to_string(&d.pack).unwrap_or_default())
    .bind(&checked.text)
    .bind(&checked.sig)
    .bind(serde_json::to_string(&files).unwrap_or_default())
    .bind(serde_json::to_string(&hashes).unwrap_or_default())
    .bind(user)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

pub async fn discard_staged(state: &AppState, pack_id: &str) {
    let _ = sqlx::query("DELETE FROM page_packs WHERE pack_id = ? AND status = 'staged'")
        .bind(pack_id)
        .execute(&state.db)
        .await;
}

/// The checks that run again on a stored row: the files against the pack's entry in the signed
/// index, that entry against the index text, the index signature against the author's key (if
/// there is a key), and the fingerprints recorded when the row was made.
pub fn verify_row(r: &Row, key: Option<&VerifyingKey>) -> (Vec<FileReport>, Result<bool, Problem>) {
    let mut report = check_files(&r.pack, &r.files);
    for rep in &mut report {
        if rep.ok && r.hashes.get(&rep.path).is_some_and(|h| *h != rep.sha256) {
            rep.ok = false;
            rep.why_en = "it changed after it was kept".into();
            rep.why_ko = "보관한 뒤에 바뀌었어요".into();
        }
    }
    let sig = check_signature(r.index_text.as_bytes(), &r.index_sig, key).and_then(|signed| {
        // The pack's entry must be exactly the one in the signed list.
        let ix = parse_index(r.index_text.as_bytes())?;
        let same = ix.packs.iter().any(|p| {
            p.id == r.pack.id && serde_json::to_value(p).ok() == serde_json::to_value(&r.pack).ok()
        });
        if same {
            Ok(signed)
        } else {
            Err(problem(
                "The kept pack is not the one in the signed list.",
                "보관된 기능이 서명된 목록의 것과 달라요.",
            ))
        }
    });
    (report, sig)
}

/// Why an install did not happen.
#[derive(Debug)]
pub enum InstallError {
    Refused(Problem),
    Db(sqlx::Error),
}

impl From<sqlx::Error> for InstallError {
    fn from(e: sqlx::Error) -> Self {
        InstallError::Db(e)
    }
}

/// Install the staged pack: re-check it, then in ONE transaction make it the installed version
/// and keep the replaced one as `previous`. An unsigned pack needs `accept_unsigned`.
pub async fn install(
    state: &AppState,
    pack_id: &str,
    user: &str,
    accept_unsigned: bool,
) -> Result<Pack, InstallError> {
    let mut tx = state.db.begin().await?;
    let found = sqlx::query(
        "SELECT * FROM page_packs WHERE pack_id = ? AND status = 'staged' ORDER BY id DESC LIMIT 1 FOR UPDATE",
    )
    .bind(pack_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(staged) = found.as_ref().and_then(row_of) else {
        return Err(InstallError::Refused(problem(
            "There is nothing downloaded for that feature.",
            "그 기능은 내려받은 것이 없어요.",
        )));
    };
    let (report, sig) = verify_row(&staged, state.pagebuilder.packs_key.as_ref());
    if let Some(bad) = report.iter().find(|r| !r.ok) {
        return Err(InstallError::Refused(problem(
            format!(
                "A file failed its check ({}). Nothing was installed.",
                bad.path
            ),
            format!(
                "파일 검사에 실패했어요 ({}). 아무것도 설치하지 않았어요.",
                bad.path
            ),
        )));
    }
    let signed = sig.map_err(InstallError::Refused)?;
    if !signed && !accept_unsigned {
        return Err(InstallError::Refused(problem(
            "This feature is not signed by the author. Tick the box to accept it anyway.",
            "이 기능은 제작자의 서명이 없어요. 그래도 받으려면 칸에 표시하세요.",
        )));
    }
    let old = sqlx::query(
        "SELECT * FROM page_packs WHERE pack_id = ? AND status = 'installed' ORDER BY id DESC LIMIT 1 FOR UPDATE",
    )
    .bind(pack_id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(old) = old.as_ref().and_then(row_of) {
        if version_key(&staged.pack.version) < version_key(&old.pack.version) {
            return Err(InstallError::Refused(problem(
                "That is an older version than the one installed. Use Roll back instead.",
                "설치된 것보다 옛 버전이에요. 대신 '되돌리기'를 쓰세요.",
            )));
        }
    }
    sqlx::query("DELETE FROM page_packs WHERE pack_id = ? AND status = 'previous'")
        .bind(pack_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE page_packs SET status = 'previous' WHERE pack_id = ? AND status = 'installed'",
    )
    .bind(pack_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE page_packs SET status = 'installed', installed_at = NOW(), installed_by = ? WHERE id = ?",
    )
    .bind(user)
    .bind(staged.id as i32)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(staged.pack)
}

/// Swap the installed version and the kept previous one (a second press swaps back).
pub async fn rollback(state: &AppState, pack_id: &str) -> Result<Pack, InstallError> {
    let mut tx = state.db.begin().await?;
    let cur = sqlx::query(
        "SELECT * FROM page_packs WHERE pack_id = ? AND status = 'installed' ORDER BY id DESC LIMIT 1 FOR UPDATE",
    )
    .bind(pack_id)
    .fetch_optional(&mut *tx)
    .await?;
    let prev = sqlx::query(
        "SELECT * FROM page_packs WHERE pack_id = ? AND status = 'previous' ORDER BY id DESC LIMIT 1 FOR UPDATE",
    )
    .bind(pack_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (Some(cur), Some(prev)) = (
        cur.as_ref().and_then(row_of),
        prev.as_ref().and_then(row_of),
    ) else {
        return Err(InstallError::Refused(problem(
            "There is no earlier version kept for that feature.",
            "그 기능에는 보관된 이전 버전이 없어요.",
        )));
    };
    sqlx::query("UPDATE page_packs SET status = 'previous' WHERE id = ?")
        .bind(cur.id as i32)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE page_packs SET status = 'installed' WHERE id = ?")
        .bind(prev.id as i32)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(prev.pack)
}

/// Take a pack away entirely (its kept previous version too).
pub async fn remove(state: &AppState, pack_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM page_packs WHERE pack_id = ? AND status IN ('installed', 'previous')")
        .bind(pack_id)
        .execute(&state.db)
        .await
        .map(|_| ())
}

/// One line about a stored pack for the editor's list.
#[derive(Serialize)]
pub struct Summary {
    pub id: String,
    pub version: String,
    pub status: String,
    pub signed: bool,
    pub source: String,
    /// Who downloaded or installed it.
    pub by: String,
    pub name_en: String,
    pub name_ko: String,
    pub desc_en: String,
    pub desc_ko: String,
    pub parts: Vec<serde_json::Value>,
}

pub fn summary(r: &Row) -> Summary {
    Summary {
        id: r.pack.id.clone(),
        version: r.pack.version.clone(),
        status: r.status.clone(),
        signed: r.signed,
        source: r.source.clone(),
        by: r.installed_by.clone(),
        name_en: r.pack.name_en.clone(),
        name_ko: r.pack.name_ko.clone(),
        desc_en: r.pack.desc_en.clone(),
        desc_ko: r.pack.desc_ko.clone(),
        parts: r
            .pack
            .provides
            .iter()
            .map(|p| {
                let (en, ko) = p.names();
                serde_json::json!({ "kind": p.kind(), "id": p.id(), "name_en": en, "name_ko": ko })
            })
            .collect(),
    }
}
