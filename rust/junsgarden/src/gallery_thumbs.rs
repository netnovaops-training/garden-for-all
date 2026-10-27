//! The paged gallery: thumbnails that link to the full-size pictures, searched through the database tags.
//!
//! Reads ONLY the new tables (`photo`, `photo_tags`, `plant_tags`; see `db/gallery.sql`), never the old
//! `tags_cache.csv`. Everything is paged with a keyset (`id > after ORDER BY id LIMIT n`), so page 500 costs the same
//! as page 1 and the browser never gets the whole list at once.
//!
//! URLs (JSON):
//!   `GET /api/gallery?after=0&limit=48&q=pink+hydrangea&plant=12`   one page: `{"items":[...],"next":123|null}`
//!   `GET /api/gallery/plant/{id}`                                    all pictures of one plant, cached
//! Each item is `{id, name, plant_id, thumb, full}`: `thumb` is the small WebP, `full` is the ORIGINAL picture
//! (never a resized copy). Both are web addresses under `JUNSGARDEN_THUMBS_URL` (default `/gallery-thumbs`) and
//! `JUNSGARDEN_FULL_URL` (default `/gallery-full`). When `JUNSGARDEN_THUMBS_DIR` / `JUNSGARDEN_PHOTOS_DIR` are set,
//! those folders are served at those addresses (the photos folder is laid out `<set>/<full_rel>`).
//!
//! HOW TO READ THIS FILE. `parse_search` and `build_page` are pure (no database), so the tests cover them.
//! `fetch_page` runs the SQL, `PlantCache` keeps one JSON text per plant until it is invalidated or 5 minutes pass,
//! `PoolSettings` reads the connection-pool numbers from the environment.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySqlPool, Row};
use tower_http::services::ServeDir;

use crate::AppState;

pub const PAGE_DEFAULT: u32 = 48;
pub const PAGE_MAX: u32 = 120;
const MAX_TOKENS: usize = 5;
const MAX_TAG_LEN: usize = 80;
const PLANT_ALL_MAX: u32 = 200;
const PLANT_TTL: Duration = Duration::from_secs(300);

// ---------------------------------------------------------------- search words

/// "Pink  Hydrangea!" -> ["pink","hydrangea"]. Lowercase letters, digits, `-` and `'` stay; anything else splits.
/// At most 5 different words, each at most 80 characters (a longer word is dropped, it could never be a tag).
pub fn parse_search(q: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for w in q
        .to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '\''))
    {
        let w = w.trim_matches(|c| c == '-' || c == '\'');
        if w.is_empty() || w.chars().count() > MAX_TAG_LEN || out.iter().any(|x| x == w) {
            continue;
        }
        out.push(w.to_string());
        if out.len() == MAX_TOKENS {
            break;
        }
    }
    out
}

// ---------------------------------------------------------------- the SQL

/// What the page asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageQuery {
    /// Keyset: only pictures with an id greater than this (0 = from the start).
    pub after: u64,
    pub limit: u32,
    pub tags: Vec<String>,
    pub plant_id: Option<i32>,
}

/// One value to bind, in the order the `?` marks appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bind {
    Text(String),
    Num(u64),
    Int(i32),
}

const COLS: &str = "p.id, p.filename, p.plant_id, p.set_name, p.full_rel, p.thumb_rel, p.media";

/// The SQL and its values. One more row than `limit` is requested so the caller can tell whether a next page exists.
/// Tags are ANDed: the first tag drives the index scan (`photo_tags(tag, photo_id)`), the others are `EXISTS` probes
/// on the primary key. `GROUP BY photo_id` makes a picture tagged by both `auto` and `stag` appear once.
pub fn build_page(q: &PageQuery) -> (String, Vec<Bind>) {
    let limit = q.limit.clamp(1, PAGE_MAX) as u64 + 1;
    let mut binds: Vec<Bind> = Vec::new();
    let mut sql;
    if q.tags.is_empty() {
        sql = format!("SELECT {COLS} FROM photo p WHERE p.status = 'active' AND p.id > ?");
        binds.push(Bind::Num(q.after));
        if let Some(pid) = q.plant_id {
            sql.push_str(" AND p.plant_id = ?");
            binds.push(Bind::Int(pid));
        }
        sql.push_str(" ORDER BY p.id LIMIT ?");
    } else {
        sql = format!(
            "SELECT {COLS} FROM photo_tags t0 JOIN photo p ON p.id = t0.photo_id \
             WHERE t0.tag = ? AND t0.photo_id > ? AND p.status = 'active'"
        );
        binds.push(Bind::Text(q.tags[0].clone()));
        binds.push(Bind::Num(q.after));
        if let Some(pid) = q.plant_id {
            sql.push_str(" AND p.plant_id = ?");
            binds.push(Bind::Int(pid));
        }
        for t in &q.tags[1..] {
            sql.push_str(" AND EXISTS (SELECT 1 FROM photo_tags x WHERE x.photo_id = t0.photo_id AND x.tag = ?)");
            binds.push(Bind::Text(t.clone()));
        }
        sql.push_str(" GROUP BY t0.photo_id ORDER BY t0.photo_id LIMIT ?");
    }
    binds.push(Bind::Num(limit));
    (sql, binds)
}

// ---------------------------------------------------------------- addresses

/// Percent-encode one path piece (everything except `A-Za-z0-9-_.~`).
fn enc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `a/b c/d.webp` -> `a/b%20c/d.webp` (each piece encoded, the slashes kept).
fn enc_path(p: &str) -> String {
    p.replace('\\', "/")
        .split('/')
        .filter(|s| !s.is_empty() && *s != "..")
        .map(enc)
        .collect::<Vec<_>>()
        .join("/")
}

/// Where thumbnails and originals are served from (web addresses).
#[derive(Debug, Clone)]
pub struct Urls {
    pub thumbs: String,
    pub full: String,
}

impl Urls {
    pub fn from_env() -> Urls {
        Urls {
            thumbs: common::env_or("JUNSGARDEN_THUMBS_URL", "/gallery-thumbs"),
            full: common::env_or("JUNSGARDEN_FULL_URL", "/gallery-full"),
        }
    }
}

/// One picture as the page's script wants it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub plant_id: Option<i32>,
    pub thumb: String,
    pub full: String,
    /// `image`, `gif` or `video`. A video has no thumbnail (`thumb` is empty): the page shows a placeholder.
    pub media: String,
}

/// Build an item from the database columns. A picture with no thumbnail uses the original as its own thumbnail.
pub fn item(
    urls: &Urls,
    id: u32,
    filename: &str,
    plant_id: Option<i32>,
    set_name: &str,
    full_rel: &str,
    thumb_rel: Option<&str>,
) -> Item {
    let full = format!(
        "{}/{}/{}",
        urls.full.trim_end_matches('/'),
        enc(set_name),
        enc_path(full_rel)
    );
    let thumb = match thumb_rel {
        Some(t) if !t.is_empty() => {
            format!("{}/{}", urls.thumbs.trim_end_matches('/'), enc_path(t))
        }
        _ => full.clone(),
    };
    let name = filename
        .rsplit_once('.')
        .map_or(filename, |(n, _)| n)
        .to_string();
    Item {
        id,
        name,
        plant_id,
        thumb,
        full,
        media: "image".to_string(),
    }
}

/// Set the media type; a video that has no thumbnail gets an empty `thumb` instead of its own address.
pub fn with_media(mut it: Item, media: &str) -> Item {
    if media == "video" && it.thumb == it.full {
        it.thumb = String::new();
    }
    it.media = media.to_string();
    it
}

/// A page of items and the `after` value for the next page (None on the last page).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Page {
    pub items: Vec<Item>,
    pub next: Option<u32>,
}

/// Cut the (limit + 1) rows down to `limit` and set `next` to the last kept id when a row was left over.
pub fn finish_page(mut items: Vec<Item>, limit: u32) -> Page {
    let limit = limit.clamp(1, PAGE_MAX) as usize;
    let more = items.len() > limit;
    items.truncate(limit);
    let next = if more {
        items.last().map(|i| i.id)
    } else {
        None
    };
    Page { items, next }
}

/// Run the page query.
pub async fn fetch_page(db: &MySqlPool, urls: &Urls, q: &PageQuery) -> Result<Page, sqlx::Error> {
    let (sql, binds) = build_page(q);
    // Audited: `build_page` joins only fixed text and `?` marks (the tests check this); every
    // value, search words included, travels in `binds`, never inside the SQL text.
    let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()));
    for b in &binds {
        query = match b {
            Bind::Text(s) => query.bind(s.as_str()),
            Bind::Num(n) => query.bind(*n),
            Bind::Int(i) => query.bind(*i),
        };
    }
    let rows = query.fetch_all(db).await?;
    let mut items = Vec::with_capacity(rows.len());
    for r in &rows {
        let it = item(
            urls,
            r.try_get::<u32, _>("id")?,
            &r.try_get::<String, _>("filename")?,
            r.try_get::<Option<i32>, _>("plant_id")?,
            &r.try_get::<String, _>("set_name")?,
            &r.try_get::<String, _>("full_rel")?,
            r.try_get::<Option<String>, _>("thumb_rel")?.as_deref(),
        );
        items.push(with_media(it, &r.try_get::<String, _>("media")?));
    }
    Ok(finish_page(items, q.limit))
}

// ---------------------------------------------------------------- per-plant cache

/// One JSON text per plant, kept until `invalidate`/`invalidate_all` or `ttl` passes.
pub struct PlantCache {
    ttl: Duration,
    map: Mutex<HashMap<i32, (Instant, Arc<String>)>>,
}

impl PlantCache {
    pub fn new(ttl: Duration) -> Self {
        PlantCache {
            ttl,
            map: Mutex::new(HashMap::new()),
        }
    }
    pub fn get(&self, plant: i32) -> Option<Arc<String>> {
        let m = self.map.lock().ok()?;
        m.get(&plant)
            .filter(|(t, _)| t.elapsed() < self.ttl)
            .map(|(_, v)| v.clone())
    }
    pub fn put(&self, plant: i32, json: String) -> Arc<String> {
        let v = Arc::new(json);
        if let Ok(mut m) = self.map.lock() {
            m.insert(plant, (Instant::now(), v.clone()));
        }
        v
    }
    /// Drop one plant's JSON (for a future "photo added to plant N" hook; today every plant edit clears all).
    #[allow(dead_code)]
    pub fn invalidate(&self, plant: i32) {
        if let Ok(mut m) = self.map.lock() {
            m.remove(&plant);
        }
    }
    pub fn invalidate_all(&self) {
        if let Ok(mut m) = self.map.lock() {
            m.clear();
        }
    }
}

static PLANT_CACHE: LazyLock<PlantCache> = LazyLock::new(|| PlantCache::new(PLANT_TTL));

/// Called when a plant (or its pictures) changes: the next detail page asks the database again.
pub fn invalidate_all() {
    PLANT_CACHE.invalidate_all();
}

// ---------------------------------------------------------------- pool settings

/// Connection-pool numbers. Defaults suit one small server; each can be set in the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolSettings {
    pub max: u32,
    pub min: u32,
    pub acquire: Duration,
    pub idle: Duration,
    pub lifetime: Duration,
}

impl PoolSettings {
    /// `get` looks a variable up (the real environment, or a map in tests). Bad or out-of-range values fall back.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> PoolSettings {
        let num = |k: &str, d: u64, lo: u64, hi: u64| {
            get(k)
                .and_then(|v| v.trim().parse::<u64>().ok())
                .filter(|n| (lo..=hi).contains(n))
                .unwrap_or(d)
        };
        let max = num("JUNSGARDEN_DB_MAX_CONNECTIONS", 8, 1, 64) as u32;
        let min = (num("JUNSGARDEN_DB_MIN_CONNECTIONS", 1, 0, 64) as u32).min(max);
        PoolSettings {
            max,
            min,
            acquire: Duration::from_secs(num("JUNSGARDEN_DB_ACQUIRE_SECS", 5, 1, 120)),
            idle: Duration::from_secs(num("JUNSGARDEN_DB_IDLE_SECS", 600, 10, 86_400)),
            lifetime: Duration::from_secs(num("JUNSGARDEN_DB_LIFETIME_SECS", 1800, 60, 86_400)),
        }
    }
    pub fn from_env() -> PoolSettings {
        PoolSettings::from_lookup(|k| std::env::var(k).ok())
    }
    pub fn options(&self) -> MySqlPoolOptions {
        MySqlPoolOptions::new()
            .max_connections(self.max)
            .min_connections(self.min)
            .acquire_timeout(self.acquire)
            .idle_timeout(self.idle)
            .max_lifetime(self.lifetime)
    }
}

// ---------------------------------------------------------------- handlers

#[derive(Deserialize)]
pub struct PageParams {
    after: Option<u64>,
    limit: Option<u32>,
    q: Option<String>,
    plant: Option<i32>,
}

fn json_error(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

async fn api_page(State(state): State<Arc<AppState>>, Query(p): Query<PageParams>) -> Response {
    let q = PageQuery {
        after: p.after.unwrap_or(0),
        limit: p.limit.unwrap_or(PAGE_DEFAULT).clamp(1, PAGE_MAX),
        tags: parse_search(
            &p.q.unwrap_or_default()
                .chars()
                .take(200)
                .collect::<String>(),
        ),
        plant_id: p.plant,
    };
    match fetch_page(&state.db, &Urls::from_env(), &q).await {
        Ok(page) => ([(header::CACHE_CONTROL, "no-store")], Json(page)).into_response(),
        Err(e) => {
            tracing::error!("gallery page query failed: {e}");
            json_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "gallery is not available right now",
            )
        }
    }
}

async fn api_plant(State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> Response {
    let body = match PLANT_CACHE.get(id) {
        Some(b) => b,
        None => {
            let q = PageQuery {
                after: 0,
                limit: PLANT_ALL_MAX.min(PAGE_MAX),
                tags: vec![],
                plant_id: Some(id),
            };
            match fetch_page(&state.db, &Urls::from_env(), &q).await {
                Ok(page) => PLANT_CACHE.put(
                    id,
                    serde_json::to_string(&page).unwrap_or_else(|_| "{}".into()),
                ),
                Err(e) => {
                    tracing::error!("gallery plant query failed: {e}");
                    return json_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "gallery is not available right now",
                    );
                }
            }
        }
    };
    (
        [(header::CONTENT_TYPE, "application/json")],
        body.as_str().to_string(),
    )
        .into_response()
}

/// The routes. Thumbnails and originals are served only when their folders are set in the environment.
pub fn routes() -> Router<Arc<AppState>> {
    let mut r = Router::new()
        .route("/api/gallery", get(api_page))
        .route("/api/gallery/plant/{id}", get(api_plant));
    if let Ok(dir) = std::env::var("JUNSGARDEN_THUMBS_DIR") {
        let base = Urls::from_env().thumbs;
        r = r.nest_service(&base, ServeDir::new(dir));
    }
    if let Ok(dir) = std::env::var("JUNSGARDEN_PHOTOS_DIR") {
        let base = Urls::from_env().full;
        r = r.nest_service(&base, ServeDir::new(dir));
    }
    r
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn q(tags: &[&str], plant: Option<i32>) -> PageQuery {
        PageQuery {
            after: 0,
            limit: 48,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            plant_id: plant,
        }
    }

    #[test]
    fn search_words_are_lowercased_split_deduped_and_capped() {
        assert_eq!(
            parse_search("  Pink  Hydrangea!! "),
            vec!["pink", "hydrangea"]
        );
        assert_eq!(parse_search("cat CAT cat"), vec!["cat"]);
        assert_eq!(parse_search("a b c d e f g"), vec!["a", "b", "c", "d", "e"]);
        assert_eq!(
            parse_search("'; DROP TABLE photo; --"),
            vec!["drop", "table", "photo"]
        );
        assert!(parse_search("").is_empty());
        assert!(parse_search(&"x".repeat(81)).is_empty());
    }

    #[test]
    fn plain_page_uses_keyset_and_asks_for_one_extra_row() {
        let (sql, b) = build_page(&q(&[], None));
        assert_eq!(
            sql,
            "SELECT p.id, p.filename, p.plant_id, p.set_name, p.full_rel, p.thumb_rel, p.media FROM photo p \
             WHERE p.status = 'active' AND p.id > ? ORDER BY p.id LIMIT ?"
        );
        assert_eq!(b, vec![Bind::Num(0), Bind::Num(49)]);
    }

    #[test]
    fn plant_page_binds_in_marker_order() {
        let (sql, b) = build_page(&PageQuery {
            after: 7,
            ..q(&[], Some(12))
        });
        assert!(sql.contains("p.id > ? AND p.plant_id = ? ORDER BY p.id LIMIT ?"));
        assert_eq!(b, vec![Bind::Num(7), Bind::Int(12), Bind::Num(49)]);
    }

    #[test]
    fn tag_search_drives_from_the_first_tag_and_probes_the_rest() {
        let (sql, b) = build_page(&q(&["pink", "hydrangea"], Some(3)));
        assert_eq!(sql.matches('?').count(), b.len());
        assert!(sql.starts_with("SELECT p.id, p.filename, p.plant_id, p.set_name, p.full_rel, p.thumb_rel, p.media FROM photo_tags t0 JOIN photo p"));
        assert!(sql.contains("WHERE t0.tag = ? AND t0.photo_id > ? AND p.status = 'active' AND p.plant_id = ? AND EXISTS"));
        assert!(sql.ends_with("GROUP BY t0.photo_id ORDER BY t0.photo_id LIMIT ?"));
        assert_eq!(
            b,
            vec![
                Bind::Text("pink".into()),
                Bind::Num(0),
                Bind::Int(3),
                Bind::Text("hydrangea".into()),
                Bind::Num(49)
            ]
        );
    }

    #[test]
    fn user_text_never_lands_in_the_sql_text() {
        let words = parse_search("x' OR '1'='1");
        let (sql, _) = build_page(&q(
            &words.iter().map(String::as_str).collect::<Vec<_>>(),
            None,
        ));
        assert!(!sql.contains("OR '1'"));
        assert_eq!(sql.matches('?').count(), 3 + words.len() - 1);
    }

    #[test]
    fn limit_is_clamped() {
        let (_, b) = build_page(&PageQuery {
            limit: 100_000,
            ..q(&[], None)
        });
        assert_eq!(b.last(), Some(&Bind::Num(PAGE_MAX as u64 + 1)));
        let (_, b) = build_page(&PageQuery {
            limit: 0,
            ..q(&[], None)
        });
        assert_eq!(b.last(), Some(&Bind::Num(2)));
    }

    fn it(id: u32) -> Item {
        item(
            &Urls {
                thumbs: "/t".into(),
                full: "/f/".into(),
            },
            id,
            "A b.webp",
            Some(1),
            "current",
            "A b.webp",
            Some("current/1/A b.webp"),
        )
    }

    #[test]
    fn items_link_thumbnail_and_the_original() {
        let i = it(5);
        assert_eq!(i.thumb, "/t/current/1/A%20b.webp");
        assert_eq!(i.full, "/f/current/A%20b.webp");
        assert_eq!(i.name, "A b");
        let no_thumb = item(
            &Urls {
                thumbs: "/t".into(),
                full: "/f".into(),
            },
            6,
            "x.jpg",
            None,
            "s",
            "x.jpg",
            None,
        );
        assert_eq!(no_thumb.thumb, no_thumb.full);
        assert_eq!(no_thumb.media, "image");
    }

    #[test]
    fn a_video_without_thumbnail_has_an_empty_thumb_but_a_gif_keeps_its_own() {
        let urls = Urls {
            thumbs: "/t".into(),
            full: "/f".into(),
        };
        let v = with_media(item(&urls, 1, "c.mp4", None, "s", "c.mp4", None), "video");
        assert_eq!((v.thumb.as_str(), v.media.as_str()), ("", "video"));
        assert_eq!(v.full, "/f/s/c.mp4");
        let g = with_media(
            item(&urls, 2, "a.gif", None, "s", "a.gif", Some("s/a.webp")),
            "gif",
        );
        assert_eq!((g.thumb.as_str(), g.media.as_str()), ("/t/s/a.webp", "gif"));
    }

    #[test]
    fn paths_cannot_climb_out() {
        assert_eq!(enc_path("../../etc/passwd"), "etc/passwd");
        assert_eq!(enc_path("a\\b/..\\c"), "a/b/c");
    }

    #[test]
    fn next_page_is_set_only_when_a_row_was_left_over() {
        let p = finish_page((1..=49).map(it).collect(), 48);
        assert_eq!(p.items.len(), 48);
        assert_eq!(p.next, Some(48));
        let last = finish_page((1..=10).map(it).collect(), 48);
        assert_eq!(last.items.len(), 10);
        assert_eq!(last.next, None);
    }

    #[test]
    fn plant_cache_keeps_expires_and_invalidates() {
        let c = PlantCache::new(Duration::from_secs(60));
        assert!(c.get(1).is_none());
        c.put(1, "{}".into());
        c.put(2, "[]".into());
        assert_eq!(c.get(1).as_deref().map(String::as_str), Some("{}"));
        c.invalidate(1);
        assert!(c.get(1).is_none() && c.get(2).is_some());
        c.invalidate_all();
        assert!(c.get(2).is_none());
        let short = PlantCache::new(Duration::from_millis(0));
        short.put(3, "x".into());
        assert!(short.get(3).is_none());
    }

    #[test]
    fn pool_settings_default_clamp_and_fall_back() {
        let none = PoolSettings::from_lookup(|_| None);
        assert_eq!((none.max, none.min), (8, 1));
        assert_eq!(none.acquire, Duration::from_secs(5));
        let set = PoolSettings::from_lookup(|k| match k {
            "JUNSGARDEN_DB_MAX_CONNECTIONS" => Some("20".into()),
            "JUNSGARDEN_DB_MIN_CONNECTIONS" => Some("50".into()),
            "JUNSGARDEN_DB_ACQUIRE_SECS" => Some("junk".into()),
            _ => None,
        });
        assert_eq!(set.max, 20);
        assert_eq!(set.min, 20, "min never exceeds max");
        assert_eq!(set.acquire, Duration::from_secs(5));
        let big = PoolSettings::from_lookup(|k| {
            (k == "JUNSGARDEN_DB_MAX_CONNECTIONS").then(|| "5000".into())
        });
        assert_eq!(big.max, 8, "out of range -> default");
    }
}
