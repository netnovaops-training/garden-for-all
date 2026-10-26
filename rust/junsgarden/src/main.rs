//! junsgarden.zapto.org — the public garden site, in Rust.
//!
//! A faithful, page-by-page conversion of the PHP site: each page here
//! produces the same HTML, uses the same stylesheet and scripts, and
//! answers the same URLs (including the `.php` ones the Android app and
//! printed QR codes use). Nothing is redesigned. See docs/rust-port/.
//!
//! The site's files (CSS, JS, images, PDFs) are not copied into the Rust
//! build: the PHP site's own directory is mounted at `/www` and served
//! as-is, so the assets are identical by construction. The same directory
//! receives uploaded photos, as it did under PHP.
//!
//! Settings, all by environment variable:
//!   JUNSGARDEN_DATABASE_URL   mysql://user:pass@host/db          (required)
//!   APP_PORT                  default 8021
//!   JUNSGARDEN_WWW_DIR        the site folder, default /www
//!   JUNSGARDEN_DATA_DIR       the guestbook/comments folder, default <www>/data
//!   JUNSGARDEN_USERS          Name=email,... who may log in (see accounts.rs)
//!   JUNSGARDEN_PUBLIC_URL     the site's address, for reset links,
//!                             default https://junsgarden.zapto.org
//!   JUNSGARDEN_SMTP_HOST/_PORT/_USER/_PASSWORD, JUNSGARDEN_MAIL_FROM
//!                             the outbox for reset emails (accounts.rs)
//!   JUNSGARDEN_UPLOADERS      names who may upload photos, default Jun
//!   JUNSGARDEN_INGEST_URL     garden-ingest, for the app's camera tools,
//!                             default http://junsgarden-ingest:8025
//!   OLLAMA_URL, OLLAMA_MODEL  the local AI model, see brain.rs
//!   JUNSGARDEN_AI_CACHE_DIR   where AI readings are cached, default <www>/cashe/AI
//!   JUNSGARDEN_FORECAST_FILE  the LSTM forecast JSON, see data.rs
//!   JUNSGARDEN_WEATHER_LATEST the collector's latest-reading JSON, see data.rs
//!   JUNSGARDEN_GUIDES_DIR     the guides (the garden folder's START-HERE) for the
//!                             /help pages, default /guides (pages/help.rs)
//! IMPORTANT for anyone running their own copy of this site: set JUNSGARDEN_USERS and
//! JUNSGARDEN_PUBLIC_URL yourself. If you leave them unset, the program falls back to the
//! defaults written in the code, which belong to the live site. Your copy would then send
//! reset emails to the live site's people with links to the live site's address.
//! Always set both, so a copy never uses the live site's address.
//!
//!
//! HOW TO READ THIS FILE (for Jun). Start at `fn run()` near the bottom:
//! it reads the settings, connects to the database, and builds the
//! "router", the list that says which function answers which address
//! (`.route("/blog.php", get(blog::blog))` means "a GET of /blog.php runs
//! the `blog` function in pages/blog.rs"). To add a page: write a function
//! in src/pages/, add one `.route` line here. The `servable()` function above
//! it is the list of file types the site will hand out from the site folder;
//! anything else is a 404, which keeps the code and backups private.

/// SQL text put together while the program runs, for the few queries whose table or column
/// NAME varies (`format!("SELECT ... FROM `{table}`")`). sqlx refuses such text unless it is
/// marked as checked, which is what `AssertSqlSafe` says. The rule everywhere it is used: only
/// names from this program's own fixed lists (or the database's own column list) go into the
/// text; every value a visitor typed is a `?` filled by `.bind(...)`.
macro_rules! sql_text {
    ($($fmt:tt)*) => { sqlx::AssertSqlSafe(format!($($fmt)*)) };
}

// `mod name;` tells Rust that a file with that name (name.rs, or a folder name/)
// is part of this program. Without a `mod` line the file is ignored.
mod accounts;
mod address;
mod brain;
mod formdata;
mod guard;
mod guides;
mod images;
mod layout;
mod normalise;
mod orchestrator;
mod pages;
mod php;
mod plant_ai;
mod rowjson;
mod session;
mod store;
mod weblink;

// `use` brings a name into scope so it can be written short. `use std::path::PathBuf;`
// lets the code say `PathBuf` instead of `std::path::PathBuf`.
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use tower_http::services::ServeDir;

/// The shared state of the whole program: everything a page handler may need.
///
/// One `AppState` is built at start-up and wrapped in an `Arc` (a counted shared
/// pointer: many request handlers can hold the same value at once, and it is freed
/// when the last one lets go). Every handler receives it through axum's `State`.
/// `pub` means other files may use it. The `///` lines above a field explain it.
pub struct AppState {
    /// The database connection pool (a small set of ready connections to MariaDB).
    pub db: MySqlPool,
    /// The PHP site's directory: assets, the files pages read, uploads.
    pub www: PathBuf,
    /// Where the guestbook and comment files live and are written. The
    /// live site: the real data folder. The test site: a COPY, so testing
    /// never touches real entries.
    pub data: PathBuf,
    /// One guestbook/comment-file write at a time.
    pub comments_lock: tokio::sync::Mutex<()>,
    /// Who may log in (name and address on file), and who may upload photos.
    pub users: Vec<accounts::Account>,
    /// Names (as logged in) who may upload photos.
    pub uploaders: Vec<String>,
    /// The site's own address, used in reset-link emails.
    pub public_url: String,
    /// garden-ingest, which reads tag/plant/receipt photos for the app.
    pub ingest_url: String,
    /// The dashboard's widget data, rebuilt at most every five minutes and
    /// thrown away whenever a plant is edited (`invalidate_widget_cache`).
    pub widget_cache: std::sync::Mutex<Option<(std::time::Instant, String)>>,
    /// One shared HTTP client, for calls out to garden-ingest and the AI model.
    pub http: reqwest::Client,
    /// Address of the local AI model (Ollama) that brain.rs talks to.
    pub ollama_url: String,
    /// Name of the Ollama model to ask.
    pub ollama_model: String,
    /// Folder where AI answers are saved so the same question is not asked twice.
    pub ai_cache_dir: PathBuf,
    /// Path of the weather-forecast JSON file written by the forecasting job.
    pub forecast_file: PathBuf,
    /// Path of the newest sensor-reading JSON file written by the weather collector.
    pub weather_latest: PathBuf,
    /// The guides folder (START-HERE: the .md files, ko/, images/, help-ids.tsv),
    /// read by the /help pages. Read-only.
    pub guides: PathBuf,
    /// The page frame's files (hero photos, facts, versions), re-read every 10 s (layout.rs).
    pub frame: layout::FrameCache,
    /// Guestbook lines and plant comments anybody may write: per visitor address.
    pub write_limit: guard::RateLimit,
    /// The gallery's photo list (tags_cache.csv), parsed once until the file changes.
    pub gallery: store::Parsed<Vec<serde_json::Value>>,
    /// A fresh (forced) AI run on the data page: once per 10 minutes for the whole site.
    pub ai_recompute: guard::RateLimit,
    /// AI questions on the data page: 20 per visitor address per 10 minutes.
    pub ai_per_visitor: guard::RateLimit,
    /// The garden's address (domain and optional folder) and signing key (address.rs).
    pub address: address::SiteAddress,
}

/// Methods that belong to `AppState`. `&self` means a method only reads the value.
impl AppState {
    /// `invalidateWidgetCache()`: the next dashboard load recomputes.
    pub fn invalidate_widget_cache(&self) {
        if let Ok(mut c) = self.widget_cache.lock() {
            *c = None;
        }
    }
}

/// Health check page at `/healthz`. Asks the database to count the plants and
/// answers `{"status":"ok", ...}`, or a 500 error if the database does not answer.
/// Docker and monitoring call it to see whether the site is alive.
/// `async fn` means the function can pause while it waits (here, for the database)
/// and let other requests run meanwhile. `.await` is the place where it waits.
async fn healthz(State(state): State<Arc<AppState>>) -> Response {
    // `match` looks at a `Result` (success or error) and runs the arm that fits.
    // `Ok(n)` is success carrying the count; `Err(e)` is failure carrying the error.
    match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM data_source")
        .fetch_one(&state.db)
        .await
    {
        Ok(n) => Json(json!({"status": "ok", "plant_count": n})).into_response(),
        Err(e) => {
            tracing::error!("healthz DB query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error"})),
            )
                .into_response()
        }
    }
}

/// Which files in the site directory may be served to anyone.
///
/// The PHP site's directory also holds source code, database settings and
/// dated backup copies of both. Apache never showed PHP source because it
/// ran it; a plain file server would. So this is an ALLOWLIST by extension,
/// with the known-dangerous shapes refused first. A file type not listed
/// is a 404, not a download.
fn servable(path: &str) -> bool {
    // Judge the path the way the file server will read it: with the %XX codes decoded
    // ("%2e" is ".", "%69mages" is "images"). A path that does not decode is refused.
    let Some(decoded) = percent_decode(path) else {
        return false;
    };
    let p = decoded.trim_start_matches('/');
    if p.is_empty() || p.contains("..") || p.contains('\\') || p.contains('\0') {
        return false;
    }
    // Hidden files, editor/backup copies, dated copies.
    if p.split('/').any(|seg| seg.starts_with('.')) {
        return false;
    }
    let lower = p.to_ascii_lowercase();
    if lower.contains(".bak")
        || lower.contains(".pre-")
        || lower.ends_with('~')
        || lower.contains(".env")
    {
        return false;
    }
    // Internal directories: never served, whatever is inside them.
    const PRIVATE: [&str; 6] = [
        "data/",
        "database/",
        "archive/",
        "tools/",
        "cashe/",
        "garden-ingest/",
    ];
    if PRIVATE.iter().any(|d| lower.starts_with(d)) {
        return false;
    }
    let Some((_, ext)) = lower.rsplit_once('.') else {
        return false;
    };
    // The photos folder (mounted at /www/images) holds the garden's own files: only photos and
    // videos are served from it. A PDF, text, zip, json or script file there is a 404, whatever
    // its name looks like, and however the address is written.
    if lower.starts_with("images/") {
        return IMAGE_TYPES.contains(&ext);
    }
    matches!(
        ext,
        "css"
            | "js"
            | "png"
            | "jpg"
            | "jpeg"
            | "webp"
            | "gif"
            | "svg"
            | "ico"
            | "pdf"
            | "apk"
            | "mp3"
            | "mp4"
            | "mov"
            | "webm"
            | "m4v"
            | "woff"
            | "woff2"
            | "ttf"
            | "html"
            | "json"
            | "xmp"
    ) && (ext != "json" || lower.starts_with("sections/cards/") || lower.starts_with("apps/"))
}

/// The only file types served from the photos folder (/images/...): photos and videos.
const IMAGE_TYPES: [&str; 13] = [
    "jpg", "jpeg", "png", "gif", "webp", "avif", "heic", "heif", "ico", "mp4", "webm", "mov", "m4v",
];

/// Decode the %XX codes of an address path. None if a code is broken, the result is not
/// text, or it hides a slash or backslash ("%2f", "%5c"), which no real file name needs.
fn percent_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hex = std::str::from_utf8(b.get(i + 1..i + 3)?).ok()?;
            let v = u8::from_str_radix(hex, 16).ok()?;
            if v == b'/' || v == b'\\' || v == 0 {
                return None;
            }
            out.push(v);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The router for static files: the site folder behind the allowlist (`servable`).
fn static_files(www: &std::path::Path) -> Router {
    Router::new()
        .fallback_service(ServeDir::new(www))
        .layer(middleware::from_fn(asset_headers))
        .layer(middleware::from_fn(only_servable))
}

/// Middleware (a step every request passes through) that only lets a request on to
/// the file server when `servable()` says the path is allowed. Otherwise it answers
/// 404 Not Found, the same answer as for a file that does not exist.
async fn only_servable(req: Request, next: Next) -> Response {
    if servable(req.uri().path()) {
        next.run(req).await
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// Cache rules for the site's files, as the live site serves them: HTML is
/// always revalidated; everything else is cached for a year, which is safe
/// because the pages add `?v=<file modified time>` to every stylesheet and
/// script URL.
async fn asset_headers(req: Request, next: Next) -> Response {
    use axum::http::{header, HeaderValue};
    let is_html = req.uri().path().to_ascii_lowercase().ends_with(".html");
    let mut res = next.run(req).await;
    if res.status().is_success() {
        let v = if is_html {
            "no-cache, must-revalidate"
        } else {
            "public, max-age=31536000, immutable"
        };
        res.headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(v));
    }
    res
}

/// Tables the PHP created on the fly (`CREATE TABLE IF NOT EXISTS` on every
/// request in plant_actions.php and index.php). Done once at startup here.
// What it takes in: the database pool, and the accounts list (so accounts.rs can
// create its own tables). `anyhow::Result<()>` means "nothing on success, or any error".
// The `?` after a call means: if it failed, stop here and return the error.
async fn ensure_tables(db: &MySqlPool, users: &[accounts::Account]) -> anyhow::Result<()> {
    // Each string is one SQL command; `IF NOT EXISTS` makes running it again harmless.
    for sql in [
        "CREATE TABLE IF NOT EXISTS garden_blog (
            id INT AUTO_INCREMENT PRIMARY KEY, author VARCHAR(50) DEFAULT 'Jun',
            title VARCHAR(255) NOT NULL, content TEXT NOT NULL, picture TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP)",
        "CREATE TABLE IF NOT EXISTS `user_done` (plant_id INT NOT NULL, username VARCHAR(100) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, PRIMARY KEY (plant_id, username)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_prefs` (username VARCHAR(100) NOT NULL, pref_key VARCHAR(50) NOT NULL,
            pref_value VARCHAR(200) NOT NULL, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (username, pref_key)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `flower_bed_qr` (bed_name VARCHAR(120) NOT NULL PRIMARY KEY, qr_code MEDIUMBLOB NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    ] {
        // `.execute(db).await?` sends the command to the database and waits for the answer.
        sqlx::query(sql).execute(db).await?;
    }
    // The plant "Website" link (the grower's product page), added 2026-10. Older installs
    // lack the column; `ADD COLUMN IF NOT EXISTS` adds it once and keeps every row.
    // A missing table (an empty database before schema.sql is loaded) only logs a warning.
    for sql in [
        "ALTER TABLE `data_source` ADD COLUMN IF NOT EXISTS product_url VARCHAR(500) DEFAULT NULL",
        "ALTER TABLE `future_plants` ADD COLUMN IF NOT EXISTS product_url VARCHAR(500) DEFAULT NULL",
    ] {
        if let Err(e) = sqlx::query(sql).execute(db).await {
            tracing::warn!("could not add the product_url column: {e}");
        }
    }
    // The session and account tables are described in their own files.
    session::ensure_table(db).await?;
    accounts::ensure_tables(db, users).await?;
    // The local plant AI's queue, helper keys and backups (new tables only). A problem here
    // only switches that feature off; the rest of the site starts as usual.
    if let Err(e) = plant_ai::ensure_tables(db).await {
        tracing::warn!("could not make the plant AI tables: {e}");
    }
    Ok(())
}

/// Starts the whole site: reads settings, connects to the database, builds the
/// router (address -> handler list), and serves requests until the program stops.
/// It returns an error if something needed at start-up fails (for example no
/// database address was set).
async fn run() -> anyhow::Result<()> {
    // Set up log messages for this program.
    common::init_tracing("junsgarden");

    // The database address is required; the program stops here with an error if missing.
    let db_url = common::env_required("JUNSGARDEN_DATABASE_URL")?;
    // Open a pool of at most 8 connections to the database.
    let db = MySqlPoolOptions::new()
        .max_connections(8)
        .connect(&db_url)
        .await?;
    // Who may log in. `env_or` reads the setting, or uses the fallback written in the code.
    // Anyone running a copy must set JUNSGARDEN_USERS themselves (see the top of this file).
    let users = accounts::parse_users(&common::env_or("JUNSGARDEN_USERS", accounts::DEFAULT_USERS));
    if users.is_empty() {
        // Nobody can log in until JUNSGARDEN_USERS (GARDEN_USERS in .env) names someone.
        tracing::warn!("no users: set GARDEN_USERS in .env (Name=email), then restart. The site runs read-only until then.");
    }
    // Make sure the tables the site creates for itself exist.
    ensure_tables(&db, &users).await?;

    // Folders on disk: the site files, the data folder, and the AI answer cache.
    let www = PathBuf::from(common::env_or("JUNSGARDEN_WWW_DIR", "/www"));
    let data = PathBuf::from(common::env_or(
        "JUNSGARDEN_DATA_DIR",
        &format!("{}/data", www.display()),
    ));
    let ai_cache_dir = PathBuf::from(common::env_or(
        "JUNSGARDEN_AI_CACHE_DIR",
        &format!("{}/cashe/AI", www.display()),
    ));
    // Made once here; the AI pages only ever use files inside it (brain::cache_file).
    if let Err(e) = std::fs::create_dir_all(&ai_cache_dir) {
        tracing::warn!(
            "AI cache folder {} not usable ({e}): AI answers will not be cached",
            ai_cache_dir.display()
        );
    }
    tracing::info!("serving site files from {}", www.display());

    // Build the shared state. `Arc::new` wraps it so all handlers can share it.
    let state = Arc::new(AppState {
        db,
        www: www.clone(),
        data,
        comments_lock: tokio::sync::Mutex::new(()),
        users,
        // The address put in reset-link emails. Set JUNSGARDEN_PUBLIC_URL for a copy, so its
        // emails link to the copy and not to the live site.
        public_url: common::env_or("JUNSGARDEN_PUBLIC_URL", "https://junsgarden.zapto.org"),
        ingest_url: common::env_or("JUNSGARDEN_INGEST_URL", "http://junsgarden-ingest:8025"),
        // A comma list such as "A,B,C" becomes a list of names: split at commas, trim spaces,
        // drop empty pieces.
        uploaders: common::env_or("JUNSGARDEN_UPLOADERS", "Jun")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        widget_cache: std::sync::Mutex::new(None),
        http: reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()?,
        ollama_url: common::env_or("OLLAMA_URL", "http://127.0.0.1:11434/api/generate"),
        ollama_model: common::env_or("OLLAMA_MODEL", "phi4:latest"),
        ai_cache_dir,
        forecast_file: PathBuf::from(common::env_or(
            "JUNSGARDEN_FORECAST_FILE",
            "/var/www/html/data/latest_prediction.json",
        )),
        weather_latest: PathBuf::from(common::env_or(
            "JUNSGARDEN_WEATHER_LATEST",
            "/var/www/html/data/weather_latest.json",
        )),
        guides: PathBuf::from(common::env_or("JUNSGARDEN_GUIDES_DIR", "/guides")),
        frame: layout::FrameCache::default(),
        // 5 guestbook lines or comments per visitor address per 10 minutes.
        write_limit: guard::RateLimit::new(5, std::time::Duration::from_secs(600)),
        gallery: Default::default(),
        ai_recompute: guard::RateLimit::new(1, std::time::Duration::from_secs(600)),
        ai_per_visitor: guard::RateLimit::new(20, std::time::Duration::from_secs(600)),
        // The wizard writes both: the address, and a signing key so the phone app can trust
        // an address change (address.rs). Without a key, the app's address is not followed.
        address: address::SiteAddress::new(
            &common::env_or("JUNSGARDEN_PUBLIC_URL", "https://junsgarden.zapto.org"),
            &common::env_or("JUNSGARDEN_SIGNING_KEY", ""),
        ),
    });

    // The router for static files (CSS, scripts, photos). `ServeDir` hands out files from
    // the site folder. The layers run in a set order: the outer layer `only_servable` first
    // checks the path is allowed, then `asset_headers` adds the cache rules.
    let assets = static_files(&www);

    // `use pages::*;` brings every page module (index, blog, catalog, ...) into scope.
    use pages::*;
    // How much a request may send. Ordinary forms are small; only the addresses that take
    // photos get the big allowance, so nobody can make every page hold 64 MB in memory.
    let uploads = DefaultBodyLimit::max(64 * 1024 * 1024);
    // One handler pair reused for several addresses: GET shows the home page, POST handles forms.
    let home = get(index::home).post(index::home_post).layer(uploads);
    // The main router. Each `.route(address, handler)` says which function answers which
    // address. `get(...)` answers GET requests; `.post(...)` adds POST for the same address.
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/", home.clone())
        .route("/index.php", home.clone())
        // The login page: log in, forgot my password, set a password.
        .route("/login.php", get(login::login_get).post(login::login_post))
        // The old household sign-in address, kept so bookmarks still land
        // somewhere: it is just the home page now.
        .route("/sso-login", home.clone())
        .route("/sso-login/{*rest}", home)
        .route("/blog.php", get(blog::blog).post(blog::blog_post))
        .route("/plant_listing.php", get(plant_listing::plant_listing))
        .route(
            "/catalog.php",
            get(catalog::catalog_get)
                .post(catalog::catalog_post)
                .layer(uploads),
        )
        .route("/gallery.php", get(gallery::gallery))
        .route(
            "/comments.php",
            get(comments::comments).post(comments::comments),
        )
        .route(
            "/plant_actions.php",
            get(plant_actions::plant_actions).post(plant_actions::plant_actions),
        )
        .route("/data.php", get(data::data_get).post(data::data_get))
        .route("/widget_data.php", get(data::widget_data))
        .route(
            "/app_api.php",
            get(app_api::app_api)
                .post(app_api::app_api)
                .options(app_api::app_api)
                .layer(uploads),
        )
        // The plant admin, converted from the Streamlit app (orchestrator/).
        .route(
            "/streamlit/",
            get(orchestrator::page)
                .post(orchestrator::action)
                .layer(uploads),
        )
        .route(
            "/streamlit",
            get(|| async { axum::response::Redirect::permanent("/streamlit/") }),
        )
        .route("/streamlit/st.css", get(orchestrator::css))
        .route("/streamlit/st.js", get(orchestrator::js))
        // The app's camera tools, for people logged in (garden_camera.rs).
        .route(
            "/app/garden/{action}",
            axum::routing::post(garden_camera::garden_camera).layer(uploads),
        )
        // The guides, for people logged in (help.rs): /help, /help/images/...
        .merge(help::routes())
        // Labels, short QR paths, and the app's address and setup page (address.rs).
        .merge(address::routes())
        // The desktop helper's queue (plant_ai.rs): its own key, home network only. A result
        // carries up to 5 photos, so it gets the photo-sized allowance.
        .merge(plant_ai::routes().layer(uploads))
        // Layers wrap every route above them; the LAST one added is the OUTERMOST. So, in
        // order: refuse a post another website sent, work out who the visitor is (session.rs),
        // run the page; and every answer, a refusal included, gets the security headers.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session::with_visitor,
        ))
        .layer(middleware::from_fn(guard::same_site_posts))
        .layer(middleware::map_response(guard::security_headers))
        // Every other address: 2 MB is plenty for any form without photos.
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        // The cards endpoint never loaded connection.php in the PHP, so it has
        // its own cache rule and none of the page headers: added AFTER the layer.
        .route("/sections/cards/index.php", get(index::cards))
        .with_state(state.clone())
        .fallback_service(assets)
        // Outermost, around pages AND files: a garden living in a folder (example.com/garden)
        // gets that folder added to its own links (address.rs). No folder: nothing changes.
        .layer(middleware::from_fn_with_state(state, address::under_prefix));

    // Start listening on the port (APP_PORT, default 8021) and handle requests.
    common::serve(app, common::env_u16("APP_PORT", 8021)).await
}

/// The program's entry point. Sets up the clock offset and an async runtime, then
/// runs `run()` to the end. The async runtime is what lets `async` functions run.
fn main() -> anyhow::Result<()> {
    common::init_local_offset();
    common::runtime()?.block_on(run())
}

// Tests: only compiled when running `cargo test`; they check `servable()` and the static router.
#[cfg(test)]
mod tests {
    use super::{servable, static_files};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    // Ordinary public files must be allowed.
    #[test]
    fn serves_ordinary_assets() {
        for ok in [
            "/style.css",
            "/js/hero.js",
            "/images/Database/a.webp",
            "/privacy.html",
            "/sections/cards/card-01.json",
            "/images/Gallery/x.mp4",
            "/images/Gallery/My%20Photo.JPG",
            "/images/plan/a.jpeg",
            "/images/Database/b.png",
            "/images/Gallery/c.gif",
            "/images/Gallery/d.MOV",
            "/images/Gallery/e.heic",
            "/images/Database/f.avif",
        ] {
            assert!(servable(ok), "{ok} should be served");
        }
    }

    // Source code, settings, backups and private folders must be refused.
    #[test]
    fn refuses_source_config_and_backups() {
        for bad in [
            "/connection.php",
            "/connection.php.bak-presso",
            "/index.php.pre-sso",
            "/.env",
            "/data/Data-Source.csv",
            "/data/comments.txt",
            "/garden-users.env",
            "/../etc/passwd",
            "/images/../connection.php",
            "/database/x.sql",
            "/app_api.php",
            "/sections/other.json",
            "/js/app.js.bak-before-scrollfix",
            "/cashe/AI/health.json",
            "/garden-ingest/vocab.py",
        ] {
            assert!(!servable(bad), "{bad} must not be served");
        }
    }

    // From the photos folder only photos and videos are served, however the address is written.
    #[test]
    fn photos_folder_serves_only_media() {
        for bad in [
            "/images/My%20Note.pdf",
            "/images/note.PDF",
            "/images/Gallery/x.pdf",
            "/images/a/b/c/d.pdf",
            "/images/x.pdf/",
            "/images/x.pdf.",
            "/images/x.jpg.pdf",
            "/images/x.pdf%2ejpg.pdf",
            "/images/x%2Epdf",
            "/%69mages/x.pdf",
            "/Images/x.pdf",
            "/images/x.txt",
            "/images/x.zip",
            "/images/x.json",
            "/images/x.php",
            "/images/x.py",
            "/images/x.csv",
            "/images/x.xmp",
            "/images/x.svg",
            "/images/x.html",
            "/images/x.js",
            "/images/x.mp3",
            "/images/x",
            "/images/x.pdf%00.jpg",
            "/images/a%2fb.pdf",
            "/images//x.pdf",
            "/images/x.p%64f",
            "/images/%zz.jpg",
        ] {
            assert!(!servable(bad), "{bad} must not be served");
        }
    }

    // The real router: a media file is served, a PDF is a 404 even when the file is on the disk.
    #[tokio::test]
    async fn router_serves_media_and_refuses_pdf_in_photos() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../target")
            .join(format!("garden-static-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("images/Gallery/deep")).unwrap();
        std::fs::write(dir.join("images/Gallery/a.jpg"), b"jpg").unwrap();
        std::fs::write(dir.join("images/Gallery/b.mp4"), b"mp4").unwrap();
        std::fs::write(dir.join("images/My Note.pdf"), b"%PDF").unwrap();
        std::fs::write(dir.join("images/Gallery/deep/x.pdf"), b"%PDF").unwrap();
        std::fs::write(dir.join("images/notes.txt"), b"t").unwrap();
        std::fs::write(dir.join("style.css"), b"a{}").unwrap();
        std::fs::write(dir.join("guide.pdf"), b"%PDF").unwrap();
        let get = |path: &'static str| {
            let app = static_files(&dir);
            async move {
                app.oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                    .await
                    .unwrap()
                    .status()
            }
        };
        for (path, want) in [
            ("/images/Gallery/a.jpg", StatusCode::OK),
            ("/images/Gallery/b.mp4", StatusCode::OK),
            ("/style.css", StatusCode::OK),
            ("/guide.pdf", StatusCode::OK), // other folders are unchanged
            ("/images/My%20Note.pdf", StatusCode::NOT_FOUND),
            ("/images/Gallery/deep/x.pdf", StatusCode::NOT_FOUND),
            ("/images/My%20Note.PDF", StatusCode::NOT_FOUND),
            ("/images/My%20Note.pdf/", StatusCode::NOT_FOUND),
            ("/%69mages/My%20Note.pdf", StatusCode::NOT_FOUND),
            ("/images/My%20Note%2epdf", StatusCode::NOT_FOUND),
            ("/images/notes.txt", StatusCode::NOT_FOUND),
        ] {
            assert_eq!(get(path).await, want, "{path}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
