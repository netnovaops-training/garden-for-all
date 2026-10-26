//! Local plant AI: the queue between the website and the desktop helper.
//!
//! HOW TO READ THIS FILE (for Jun). When a plant is added on the Add plant tab, a JOB is written
//! to the `plant_ai_jobs` table. The desktop helper (plant-ai/ in the garden folder, on the PC
//! with the graphics card) asks this website for jobs every minute, does the AI work on its own
//! computer, and sends the answers back. The website never calls the PC: the PC only makes
//! ordinary outgoing requests, through Caddy, like a browser. So no new port is opened anywhere,
//! and the website can live on another computer (a laptop) while the PC does the heavy work.
//!
//!   GET  /plant-ai/jobs?take=N           the next queued plants (marked "working")
//!   POST /plant-ai/jobs/<id>/progress    {"step": "..."}   for the progress bar
//!   POST /plant-ai/jobs/<id>/result      {"result": {...}, "photos": [...]}
//!   POST /plant-ai/jobs/<id>/fail        {"error": "..."}
//!
//! WHO MAY CALL THEM: a request with `Authorization: Worker <key>`, where the key was made on the
//! Add plant tab ("Desktop helper") and is still switched on. The website keeps only the key's
//! SHA-256 fingerprint, never the key. The request must also come from the home network (or this
//! computer itself), unless JUNSGARDEN_PLANT_AI_ANY_ADDRESS=1.
//!
//! WHAT A RESULT MAY CHANGE: only EMPTY plant-information fields (a value already there is never
//! replaced), the Source link (`product_url`, when that column exists and is empty) and, for a
//! plant with no photo yet, up to 5 photos (one .webp each, 500 to 2000 px, no thumbnails). Before
//! any write, the whole row is copied into `plant_ai_backups`. Approving is a separate step that
//! only the garden owner can do (see `owner_name`).

use std::net::IpAddr;
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, RawQuery, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::orchestrator::opts;
use crate::session::Visitor;
use crate::AppState;

/// Every plant-information field the AI may fill (the same list as plant-ai/plantai/norm.py
/// FIELDS). Names, quantities, beds, prices, photos and the approval columns are not in it.
pub const AI_FIELDS: [&str; 60] = [
    "botanical_name",
    "family",
    "genus",
    "species",
    "plant_type",
    "native_status",
    "plant_length",
    "plant_width",
    "spacing",
    "zones",
    "life_expectancy",
    "growth_rate",
    "sun_exposure",
    "water_needs",
    "soil_ph",
    "soil_type",
    "planting_depth",
    "chilling_hours",
    "foliage_color",
    "stem_color",
    "is_evergreen",
    "leaf_shape",
    "leaf_type",
    "leaf_margin",
    "bloom_color",
    "aromatics",
    "flower_symmetry",
    "flower_shape",
    "flower_size",
    "reproduction_type",
    "fruit_type",
    "fruit_shape",
    "fruit_color",
    "fruit_size",
    "fruit_edibility",
    "root_system",
    "root_morphology",
    "root_function",
    "attracts",
    "deer_resistance",
    "toxicity",
    "invasive_local",
    "propagation_method",
    "cuttings",
    "companion_plants",
    "fertilizer",
    "grow_months",
    "bloom_months",
    "care_notes",
    "maintenance_notes",
    "cleanup_notes",
    "disease_susceptibility",
    "extra_details",
    "season",
    "grow_season",
    "month_start",
    "month_end",
    "planting_month_start",
    "planting_month_end",
    "company",
];

/// The SQL the Approve button runs. Only `approved_jun`: the old second sign-off column
/// (`approved_jason`) is left exactly as it is and is never needed.
pub const APPROVE_SQL: &str = "UPDATE data_source SET approved_jun = ? WHERE id = ?";

/// The tables, made at start-up if they are missing (new tables only; nothing existing changes).
pub async fn ensure_tables(db: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
    for sql in [
        // One row per AI job. status: queued, working, review (answers are in), failed.
        "CREATE TABLE IF NOT EXISTS `plant_ai_jobs` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            plant_id INT NOT NULL,
            status VARCHAR(20) NOT NULL DEFAULT 'queued',
            step VARCHAR(40) NOT NULL DEFAULT '',
            added_by VARCHAR(100) NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            claimed_at DATETIME NULL,
            finished_at DATETIME NULL,
            result LONGTEXT NULL,
            error VARCHAR(500) NOT NULL DEFAULT '',
            KEY k_plant (plant_id),
            KEY k_status (status)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        // The desktop helper keys: only a SHA-256 fingerprint of each key is kept.
        "CREATE TABLE IF NOT EXISTS `plant_ai_keys` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            label VARCHAR(100) NOT NULL DEFAULT '',
            key_sha256 CHAR(64) NOT NULL,
            created_by VARCHAR(100) NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_seen DATETIME NULL,
            revoked TINYINT(1) NOT NULL DEFAULT 0,
            UNIQUE KEY k_hash (key_sha256)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        // A copy of a plant's whole row, made just before this feature writes to it.
        "CREATE TABLE IF NOT EXISTS `plant_ai_backups` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            plant_id INT NOT NULL,
            job_id INT NULL,
            reason VARCHAR(100) NOT NULL DEFAULT '',
            row_json LONGTEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            KEY k_plant (plant_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    ] {
        sqlx::query(sql).execute(db).await?;
    }
    Ok(())
}

// ---- who is the garden owner -------------------------------------------------------------

/// The owner's account name: JUNSGARDEN_OWNER when it names one of the accounts, otherwise the
/// FIRST account in JUNSGARDEN_USERS (the install writes the owner first). `None` = no accounts.
pub fn owner_from(users: &[crate::accounts::Account], setting: Option<&str>) -> Option<String> {
    let wanted = setting.map(str::trim).filter(|s| !s.is_empty());
    if let Some(w) = wanted {
        if let Some(a) = users.iter().find(|a| a.name.eq_ignore_ascii_case(w)) {
            return Some(a.name.clone());
        }
    }
    users.first().map(|a| a.name.clone())
}

/// The owner of this garden, from the settings.
pub fn owner_name(state: &AppState) -> Option<String> {
    owner_from(
        &state.users,
        std::env::var("JUNSGARDEN_OWNER").ok().as_deref(),
    )
}

/// Is the signed-in person the garden owner? Only the owner may press Approve.
pub fn is_owner(state: &AppState, v: &Visitor) -> bool {
    v.is_admin() && owner_name(state).is_some_and(|o| o.eq_ignore_ascii_case(v.username.trim()))
}

/// "Approved" in this garden means the owner approved it. Nothing else is asked for.
pub fn is_approved(row: &Map<String, Value>) -> bool {
    matches!(
        opts::text(row.get("approved_jun")).trim(),
        "1" | "true" | "True"
    )
}

// ---- small helpers -----------------------------------------------------------------------------

/// Hex SHA-256 of a text.
pub fn sha256_hex(s: &str) -> String {
    Sha256::digest(s.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A new helper key: "gw_" and 40 random hex characters. The prefix keeps it from ever looking
/// like the phone app's 64-character key.
pub fn new_key() -> String {
    let b: [u8; 20] = rand::random();
    format!(
        "gw_{}",
        b.iter().map(|x| format!("{x:02x}")).collect::<String>()
    )
}

/// The key from `Authorization: Worker <key>`, when it has the right shape.
fn worker_key(h: &HeaderMap) -> Option<String> {
    let raw = h.get(header::AUTHORIZATION)?.to_str().ok()?;
    let k = raw.strip_prefix("Worker ")?.trim();
    (k.len() == 43 && k.starts_with("gw_") && k[3..].bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| k.to_string())
}

/// Is this address at home (or this computer)? Private IPv4 ranges, loopback, Tailscale's
/// 100.64.0.0/10, and IPv6 loopback, link-local and private (fc00::/7) addresses.
pub fn is_home_address(ip: &str) -> bool {
    // No forwarded address at all: the request did not come through Caddy, so it came from this
    // computer (the website's own port only listens on 127.0.0.1).
    if ip == "unknown" {
        return true;
    }
    match ip.parse::<IpAddr>() {
        Ok(IpAddr::V4(a)) => {
            let o = a.octets();
            a.is_private() || a.is_loopback() || (o[0] == 100 && (64..128).contains(&o[1]))
        }
        Ok(IpAddr::V6(a)) => {
            let s = a.segments()[0];
            a.is_loopback() || (s & 0xfe00) == 0xfc00 || (s & 0xffc0) == 0xfe80
        }
        Err(_) => false,
    }
}

/// Base64 (standard alphabet, with or without `=` padding) to bytes; None if it is not base64.
/// Written here because it is small, so the site needs no extra library for it.
pub fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let val = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => (c - b'A') as u32,
            b'a'..=b'z' => (c - b'a' + 26) as u32,
            b'0'..=b'9' => (c - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        })
    };
    let clean: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let body = clean
        .strip_suffix(b"==")
        .or_else(|| clean.strip_suffix(b"="))
        .unwrap_or(&clean);
    let mut out = Vec::with_capacity(body.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for &c in body {
        acc = (acc << 6) | val(c)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// A web address the Source link may hold: http(s), no spaces or quotes, at most 500 characters.
pub fn clean_url(u: &str) -> Option<String> {
    let u = u.trim();
    let ok = (u.starts_with("https://") || u.starts_with("http://"))
        && u.len() <= 500
        && u.len() > 10
        && !u
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | '\\'));
    ok.then(|| u.to_string())
}

/// One photo for the garden: a picture the image library can read, long side at least 500 px,
/// shrunk once to 2000 px when bigger, saved as ONE WebP (quality 90). No thumbnail is made: the
/// pages show this one file at the size they need (CSS) and load it lazily.
pub fn fit_photo(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|_| "not a picture".to_string())?;
    let long = img.width().max(img.height());
    if long < 500 {
        return Err(format!(
            "too small ({}x{} px; 500 px needed)",
            img.width(),
            img.height()
        ));
    }
    let img = if long > 2000 {
        img.resize(2000, 2000, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    crate::images::to_webp_image(&img, 90.0).ok_or_else(|| "could not write WebP".to_string())
}

/// The fields a result may write: in AI_FIELDS, a real column, and EMPTY in the row now. Gives
/// (field -> value to write, field -> why it was skipped). Never overwrites a value.
pub fn plan_writes(
    row: &Map<String, Value>,
    fields: &Map<String, Value>,
    cols: &[String],
) -> (Map<String, Value>, Map<String, Value>) {
    let (mut write, mut skip) = (Map::new(), Map::new());
    for (k, v) in fields {
        let text = opts::text(Some(v));
        let why = if !AI_FIELDS.contains(&k.as_str()) {
            "not a field the AI may fill"
        } else if !cols.iter().any(|c| c == k) {
            "no such column"
        } else if !opts::is_blank(&opts::text(row.get(k))) {
            "already filled (kept)"
        } else if text.trim().is_empty() || text.chars().count() > 2000 {
            "empty or too long"
        } else {
            write.insert(k.clone(), Value::String(text.trim().to_string()));
            continue;
        };
        skip.insert(k.clone(), Value::String(why.into()));
    }
    (write, skip)
}

/// The row as JSON, without the two picture blobs (QR and barcode images).
pub async fn plant_row(state: &AppState, id: i64) -> Option<Map<String, Value>> {
    let r = sqlx::query("SELECT * FROM data_source WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()??;
    match crate::rowjson::row_to_json(&r) {
        Value::Object(mut m) => {
            m.remove("qr_code");
            m.remove("barcode_image");
            Some(m)
        }
        _ => None,
    }
}

/// The real column names of data_source.
pub async fn columns(state: &AppState) -> Vec<String> {
    sqlx::query_scalar("SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'data_source'")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
}

/// Copy the plant's whole row into plant_ai_backups before a write. A failed copy stops the write.
pub async fn backup(
    state: &AppState,
    id: i64,
    job: Option<i64>,
    reason: &str,
) -> Result<(), String> {
    let row = plant_row(state, id).await.ok_or("plant not found")?;
    sqlx::query(
        "INSERT INTO plant_ai_backups (plant_id, job_id, reason, row_json) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(job)
    .bind(reason)
    .bind(Value::Object(row).to_string())
    .execute(&state.db)
    .await
    .map_err(|e| format!("backup failed, nothing written: {e}"))?;
    Ok(())
}

/// Write `fields` (already checked by `plan_writes`) to the plant. Gives how many were written.
pub async fn write_fields(
    state: &AppState,
    id: i64,
    fields: &Map<String, Value>,
) -> Result<usize, String> {
    if fields.is_empty() {
        return Ok(0);
    }
    crate::orchestrator::commit_fields(state, id, fields).await
}

/// Queue the AI for a plant (a new job; an older unfinished job for it is closed first).
pub async fn queue(state: &AppState, plant_id: i64, who: &str) -> Result<i64, sqlx::Error> {
    sqlx::query("UPDATE plant_ai_jobs SET status = 'failed', error = 'replaced by a newer job' WHERE plant_id = ? AND status IN ('queued', 'working')")
        .bind(plant_id)
        .execute(&state.db)
        .await?;
    let r = sqlx::query("INSERT INTO plant_ai_jobs (plant_id, added_by) VALUES (?, ?)")
        .bind(plant_id)
        .bind(who)
        .execute(&state.db)
        .await?;
    Ok(r.last_insert_id() as i64)
}

/// One job, as the pages show it.
#[derive(Clone, Debug, Default)]
pub struct Job {
    pub id: i64,
    pub plant_id: i64,
    pub status: String,
    pub step: String,
    pub created: String,
    pub result: Option<Value>,
    pub error: String,
}

type JobRow = (i64, i64, String, String, String, Option<String>, String);

fn job_of(r: JobRow) -> Job {
    Job {
        id: r.0,
        plant_id: r.1,
        status: r.2,
        step: r.3,
        created: r.4,
        result: r.5.and_then(|s| serde_json::from_str(&s).ok()),
        error: r.6,
    }
}

const JOB_COLS: &str =
    "id, plant_id, status, step, DATE_FORMAT(created_at, '%Y-%m-%d %H:%i'), result, error";

/// The newest job for a plant, if any.
pub async fn latest_job(state: &AppState, plant_id: i64) -> Option<Job> {
    let sql =
        format!("SELECT {JOB_COLS} FROM plant_ai_jobs WHERE plant_id = ? ORDER BY id DESC LIMIT 1");
    sqlx::query_as::<_, JobRow>(sqlx::AssertSqlSafe(sql))
        .bind(plant_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(job_of)
}

/// The newest jobs (one line each on the Add plant tab).
pub async fn recent_jobs(state: &AppState, n: i64) -> Vec<Job> {
    let sql = format!("SELECT {JOB_COLS} FROM plant_ai_jobs ORDER BY id DESC LIMIT ?");
    sqlx::query_as::<_, JobRow>(sqlx::AssertSqlSafe(sql))
        .bind(n)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(job_of)
        .collect()
}

/// When a helper key was last used ("" = never), for the "is the desktop helper on?" line.
pub async fn helper_last_seen(state: &AppState) -> String {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT DATE_FORMAT(MAX(last_seen), '%Y-%m-%d %H:%i') FROM plant_ai_keys WHERE revoked = 0",
    )
    .fetch_one(&state.db)
    .await
    .ok()
    .flatten()
    .unwrap_or_default()
}

// ---- the helper's addresses ------------------------------------------------------------------

/// /plant-ai/... routes, merged into the main router (main.rs).
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/plant-ai/jobs", get(take))
        .route("/plant-ai/jobs/{id}/{what}", post(report))
}

fn reply(status: StatusCode, body: Value) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response()
}

/// The helper's key and address are checked here. Gives the key's id, or the refusal to send
/// (boxed: a whole reply is large to pass around).
async fn helper(state: &AppState, h: &HeaderMap) -> Result<i64, Box<Response>> {
    let refuse = |s: StatusCode, msg: &str| Box::new(reply(s, json!({ "error": msg })));
    let any = common::env_bool("JUNSGARDEN_PLANT_AI_ANY_ADDRESS", false);
    if !any && !is_home_address(&crate::accounts::client_ip(h)) {
        return Err(refuse(
            StatusCode::FORBIDDEN,
            "the desktop helper works from the home network only",
        ));
    }
    let Some(k) = worker_key(h) else {
        return Err(refuse(StatusCode::UNAUTHORIZED, "a helper key is needed"));
    };
    let id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM plant_ai_keys WHERE key_sha256 = ? AND revoked = 0")
            .bind(sha256_hex(&k))
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
    let Some(id) = id else {
        return Err(refuse(
            StatusCode::FORBIDDEN,
            "unknown or switched-off helper key",
        ));
    };
    let _ = sqlx::query("UPDATE plant_ai_keys SET last_seen = NOW() WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await;
    Ok(id)
}

/// GET /plant-ai/jobs?take=N: hand out up to N queued plants (1 to 20) and mark them working.
/// A job left "working" for 3 hours (the PC was switched off mid-way) is handed out again.
async fn take(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
) -> Response {
    if let Err(r) = helper(&state, &headers).await {
        return *r;
    }
    let n = serde_urlencoded::from_str::<Vec<(String, String)>>(q.as_deref().unwrap_or(""))
        .unwrap_or_default()
        .into_iter()
        .find(|(k, _)| k == "take")
        .and_then(|(_, v)| v.parse::<i64>().ok())
        .unwrap_or(10)
        .clamp(1, 20);
    let _ = sqlx::query("UPDATE plant_ai_jobs SET status = 'queued', step = '' WHERE status = 'working' AND claimed_at < NOW() - INTERVAL 3 HOUR")
        .execute(&state.db)
        .await;
    let ids: Vec<(i64, i64)> = match async {
        let mut tx = state.db.begin().await?;
        let ids: Vec<(i64, i64)> = sqlx::query_as("SELECT id, plant_id FROM plant_ai_jobs WHERE status = 'queued' ORDER BY id LIMIT ? FOR UPDATE")
            .bind(n)
            .fetch_all(&mut *tx)
            .await?;
        for (id, _) in &ids {
            sqlx::query("UPDATE plant_ai_jobs SET status = 'working', step = 'waiting', claimed_at = NOW() WHERE id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok::<_, sqlx::Error>(ids)
    }
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("plant-ai take failed: {e}");
            return reply(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": "database"}));
        }
    };
    let mut jobs = Vec::new();
    for (id, pid) in ids {
        match plant_row(&state, pid).await {
            Some(row) => jobs.push(json!({"id": id, "plant": Value::Object(row)})),
            None => {
                let _ = sqlx::query("UPDATE plant_ai_jobs SET status = 'failed', error = 'plant was deleted' WHERE id = ?")
                    .bind(id)
                    .execute(&state.db)
                    .await;
            }
        }
    }
    reply(StatusCode::OK, json!({"jobs": jobs}))
}

/// POST /plant-ai/jobs/<id>/progress | result | fail.
async fn report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, what)): Path<(i64, String)>,
    body: Bytes,
) -> Response {
    if let Err(r) = helper(&state, &headers).await {
        return *r;
    }
    let Ok(v) = serde_json::from_slice::<Value>(&body) else {
        return reply(StatusCode::BAD_REQUEST, json!({"error": "JSON expected"}));
    };
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM plant_ai_jobs WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
    if status.as_deref() != Some("working") {
        return reply(
            StatusCode::CONFLICT,
            json!({"error": "this job is not waiting for an answer"}),
        );
    }
    match what.as_str() {
        "progress" => {
            let step: String = opts::text(v.get("step"))
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
                .take(40)
                .collect();
            let _ = sqlx::query("UPDATE plant_ai_jobs SET step = ? WHERE id = ?")
                .bind(step)
                .bind(id)
                .execute(&state.db)
                .await;
            reply(StatusCode::OK, json!({"ok": true}))
        }
        "fail" => {
            let err: String = opts::text(v.get("error")).chars().take(500).collect();
            let _ = sqlx::query("UPDATE plant_ai_jobs SET status = 'failed', error = ?, finished_at = NOW() WHERE id = ?")
                .bind(err)
                .bind(id)
                .execute(&state.db)
                .await;
            reply(StatusCode::OK, json!({"ok": true}))
        }
        "result" => match apply(&state, id, &v).await {
            Ok(summary) => reply(StatusCode::OK, summary),
            Err(e) => {
                let _ = sqlx::query("UPDATE plant_ai_jobs SET status = 'failed', error = ?, finished_at = NOW() WHERE id = ?")
                    .bind(e.chars().take(500).collect::<String>())
                    .bind(id)
                    .execute(&state.db)
                    .await;
                reply(StatusCode::UNPROCESSABLE_ENTITY, json!({"error": e}))
            }
        },
        _ => reply(StatusCode::NOT_FOUND, json!({"error": "unknown"})),
    }
}

/// Save a result: back up, write the empty fields, the Source link and the photos, keep the
/// whole answer on the job for the review screen.
async fn apply(state: &AppState, job: i64, v: &Value) -> Result<Value, String> {
    let pid: i64 = sqlx::query_scalar("SELECT plant_id FROM plant_ai_jobs WHERE id = ?")
        .bind(job)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let row = plant_row(state, pid).await.ok_or("plant not found")?;
    let res = v
        .get("result")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let fields = res
        .get("fields")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let cols = columns(state).await;
    let (mut write, skipped) = plan_writes(&row, &fields, &cols);
    // The Source link: only into an empty product_url column (the column comes with the
    // "website link" change; without it the link stays on the review screen only).
    if cols.iter().any(|c| c == "product_url")
        && opts::is_blank(&opts::text(row.get("product_url")))
    {
        if let Some(u) = res
            .get("product_url")
            .and_then(Value::as_str)
            .and_then(clean_url)
        {
            write.insert("product_url".into(), Value::String(u));
        }
    }
    // Photos: only for a plant with none yet; one .webp each.
    let mut saved: Vec<String> = Vec::new();
    let mut photo_notes: Vec<Value> = Vec::new();
    let no_photo = opts::is_blank(&opts::text(row.get("picture")))
        || opts::text(row.get("picture"))
            .trim()
            .eq_ignore_ascii_case("none");
    let photos = v
        .get("photos")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if no_photo {
        let dir = state.www.join("images/Database");
        let base = format!(
            "{}_{}_ai_{}",
            crate::orchestrator::form::file_safe(&opts::text(row.get("plant_name"))),
            pid,
            job
        );
        for (i, p) in photos.iter().take(5).enumerate() {
            let data = p.get("data").and_then(Value::as_str).and_then(b64_decode);
            match data.as_deref().map(fit_photo) {
                Some(Ok(webp)) => {
                    let name = format!("{base}_{}.webp", i + 1);
                    if std::fs::write(dir.join(&name), webp).is_ok() {
                        saved.push(name);
                    }
                }
                Some(Err(e)) => photo_notes.push(json!({"photo": i + 1, "skipped": e})),
                None => photo_notes.push(json!({"photo": i + 1, "skipped": "not base64"})),
            }
        }
        if !saved.is_empty() {
            write.insert("picture".into(), Value::String(saved.join(", ")));
        }
    } else if !photos.is_empty() {
        photo_notes.push(json!({"skipped": "the plant already has photos (kept as they are)"}));
    }
    if !write.is_empty() {
        backup(state, pid, Some(job), "ai result").await?;
        write_fields(state, pid, &write).await?;
    }
    // What the review screen needs, without the photo bytes.
    let mut keep = res.clone();
    keep.insert(
        "written".into(),
        Value::Array(write.keys().map(|k| Value::String(k.clone())).collect()),
    );
    keep.insert("skipped".into(), Value::Object(skipped));
    keep.insert("photos_saved".into(), json!(saved));
    keep.insert("photo_notes".into(), Value::Array(photo_notes));
    let credits: Vec<Value> = photos
        .iter()
        .take(5)
        .map(|p| json!({"credit": opts::text(p.get("credit")), "source": opts::text(p.get("source"))}))
        .collect();
    keep.insert("photo_credits".into(), Value::Array(credits));
    sqlx::query("UPDATE plant_ai_jobs SET status = 'review', step = 'done', result = ?, finished_at = NOW(), error = '' WHERE id = ?")
        .bind(Value::Object(keep).to_string())
        .bind(job)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    state.invalidate_widget_cache();
    Ok(json!({"ok": true, "written": write.len(), "photos": saved.len()}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::Account;

    fn acct(n: &str) -> Account {
        Account {
            name: n.into(),
            email: format!("{}@example.test", n.to_lowercase()),
        }
    }

    #[test]
    fn owner_is_the_setting_or_the_first_account() {
        let users = vec![acct("Gardener"), acct("Helper")];
        assert_eq!(owner_from(&users, None).as_deref(), Some("Gardener"));
        assert_eq!(
            owner_from(&users, Some("helper")).as_deref(),
            Some("Helper")
        );
        // A name that is not an account falls back to the first account.
        assert_eq!(
            owner_from(&users, Some("Stranger")).as_deref(),
            Some("Gardener")
        );
        assert_eq!(owner_from(&[], None), None);
    }

    #[test]
    fn approval_needs_only_the_owner() {
        let mut row = Map::new();
        row.insert("approved_jun".into(), json!(1));
        row.insert("approved_jason".into(), json!(0));
        assert!(is_approved(&row));
        row.insert("approved_jun".into(), json!(0));
        row.insert("approved_jason".into(), json!(1));
        assert!(!is_approved(&row));
        assert!(!APPROVE_SQL.contains("approved_jason"));
    }

    #[test]
    fn writes_only_empty_allowed_fields() {
        let mut row = Map::new();
        row.insert("zones".into(), json!("5-9"));
        row.insert("family".into(), Value::Null);
        row.insert("bloom_color".into(), json!("Unknown"));
        let mut f = Map::new();
        f.insert("zones".into(), json!("4-8"));
        f.insert("family".into(), json!("Lamiaceae"));
        f.insert("bloom_color".into(), json!("Violet"));
        f.insert("approved_jun".into(), json!("1"));
        f.insert("plant_name".into(), json!("Other"));
        let cols: Vec<String> = [
            "zones",
            "family",
            "bloom_color",
            "approved_jun",
            "plant_name",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let (w, s) = plan_writes(&row, &f, &cols);
        assert_eq!(w.get("family"), Some(&json!("Lamiaceae")));
        assert_eq!(w.get("bloom_color"), Some(&json!("Violet")));
        assert!(!w.contains_key("zones") && s.contains_key("zones"));
        assert!(!w.contains_key("approved_jun") && !w.contains_key("plant_name"));
    }

    #[test]
    fn home_addresses_only() {
        for ok in [
            "unknown",
            "127.0.0.1",
            "192.168.1.20",
            "10.0.0.5",
            "172.20.1.1",
            "100.101.2.3",
            "::1",
            "fd00::1",
            "fe80::1",
        ] {
            assert!(is_home_address(ok), "{ok}");
        }
        for bad in [
            "8.8.8.8",
            "172.32.0.1",
            "100.128.0.1",
            "2001:db8::1",
            "nonsense",
        ] {
            assert!(!is_home_address(bad), "{bad}");
        }
    }

    #[test]
    fn keys_and_base64() {
        let k = new_key();
        assert_eq!(k.len(), 43);
        let mut h = HeaderMap::new();
        h.insert(
            header::AUTHORIZATION,
            format!("Worker {k}").parse().unwrap(),
        );
        assert_eq!(worker_key(&h), Some(k.clone()));
        h.insert(
            header::AUTHORIZATION,
            format!("Bearer {k}").parse().unwrap(),
        );
        assert_eq!(worker_key(&h), None);
        assert_eq!(b64_decode("UklGRg==").unwrap(), b"RIFF");
        assert_eq!(b64_decode("aGk").unwrap(), b"hi");
        assert!(b64_decode("no*base64").is_none());
        assert_eq!(sha256_hex("abc").len(), 64);
    }

    #[test]
    fn photos_one_webp_within_limits() {
        let png = |w: u32, h: u32| {
            let mut b = Vec::new();
            image::DynamicImage::new_rgb8(w, h)
                .write_to(&mut std::io::Cursor::new(&mut b), image::ImageFormat::Png)
                .unwrap();
            b
        };
        assert!(fit_photo(&png(400, 300)).unwrap_err().contains("too small"));
        let big = fit_photo(&png(3000, 1500)).unwrap();
        let out = image::load_from_memory(&big).unwrap();
        assert_eq!((out.width(), out.height()), (2000, 1000));
        assert_eq!(&big[..4], b"RIFF");
        assert!(clean_url("https://example.test/plant").is_some());
        assert!(clean_url("javascript:alert(1)").is_none());
    }
}
