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

// `mod name;` tells Rust that a file with that name (name.rs, or a folder name/)
// is part of this program. Without a `mod` line the file is ignored.
mod accounts;
mod brain;
mod formdata;
mod guides;
mod images;
mod layout;
mod normalise;
mod orchestrator;
mod pages;
mod php;
mod rowjson;
mod session;

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
    match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM data_source").fetch_one(&state.db).await {
        Ok(n) => Json(json!({"status": "ok", "plant_count": n})).into_response(),
        Err(e) => {
            tracing::error!("healthz DB query failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "error"}))).into_response()
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
    let p = path.trim_start_matches('/');
    if p.is_empty() || p.contains("..") || p.contains('\\') || p.contains('\0') {
        return false;
    }
    // Hidden files, editor/backup copies, dated copies.
    if p.split('/').any(|seg| seg.starts_with('.')) {
        return false;
    }
    let lower = p.to_ascii_lowercase();
    if lower.contains(".bak") || lower.contains(".pre-") || lower.ends_with('~') || lower.contains(".env") {
        return false;
    }
    // Internal directories: never served, whatever is inside them.
    const PRIVATE: [&str; 6] = ["data/", "database/", "archive/", "tools/", "cashe/", "garden-ingest/"];
    if PRIVATE.iter().any(|d| lower.starts_with(d)) {
        return false;
    }
    let Some((_, ext)) = lower.rsplit_once('.') else { return false };
    matches!(
        ext,
        "css" | "js" | "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" | "ico" | "pdf" | "apk"
            | "mp3" | "mp4" | "mov" | "webm" | "m4v" | "woff" | "woff2" | "ttf" | "html" | "json" | "xmp"
    ) && (ext != "json" || lower.starts_with("sections/cards/") || lower.starts_with("apps/") || lower.starts_with("images/"))
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

/// The headers the PHP site sends on every page it renders. Copied from the
/// live responses: the Content-Security-Policy and the frame, sniffing and
/// referrer protections are part of the site's security, not decoration, and
/// the no-cache set stops a stale copy of a page being shown after an edit.
/// Left out on purpose: `X-Powered-By: PHP/8.3.x`, which only told strangers
/// which PHP version to look up exploits for.
// Takes the finished response, adds the headers below, and gives it back.
async fn page_headers(mut res: Response) -> Response {
    use axum::http::{header::HeaderName, HeaderValue};
    let h = res.headers_mut();
    for (name, value) in [
        ("cache-control", "no-cache, must-revalidate"),
        ("expires", "Thu, 19 Nov 1981 08:52:00 GMT"),
        ("pragma", "no-cache"),
        (
            "content-security-policy",
            "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com; style-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com; img-src 'self' data: blob:; font-src 'self'; connect-src 'self' https://cdnjs.cloudflare.com; frame-ancestors 'none'; base-uri 'self'; form-action 'self';",
        ),
        ("referrer-policy", "strict-origin-when-cross-origin"),
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
    ] {
        // A handler that chose its own Cache-Control (the comments API, the
        // QR images) keeps it, as in PHP where header() runs after the
        // session's default.
        if name == "cache-control" && h.contains_key(name) {
            continue;
        }
        h.insert(HeaderName::from_static(name), HeaderValue::from_static(value));
    }
    res
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
        let v = if is_html { "no-cache, must-revalidate" } else { "public, max-age=31536000, immutable" };
        res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static(v));
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
    // The session and account tables are described in their own files.
    session::ensure_table(db).await?;
    accounts::ensure_tables(db, users).await?;
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
    let db = MySqlPoolOptions::new().max_connections(8).connect(&db_url).await?;
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
    let data = PathBuf::from(common::env_or("JUNSGARDEN_DATA_DIR", &format!("{}/data", www.display())));
    let ai_cache_dir = PathBuf::from(common::env_or("JUNSGARDEN_AI_CACHE_DIR", &format!("{}/cashe/AI", www.display())));
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
        http: reqwest::Client::builder().connect_timeout(std::time::Duration::from_secs(5)).build()?,
        ollama_url: common::env_or("OLLAMA_URL", "http://127.0.0.1:11434/api/generate"),
        ollama_model: common::env_or("OLLAMA_MODEL", "phi4:latest"),
        ai_cache_dir,
        forecast_file: PathBuf::from(common::env_or("JUNSGARDEN_FORECAST_FILE", "/var/www/html/data/latest_prediction.json")),
        weather_latest: PathBuf::from(common::env_or("JUNSGARDEN_WEATHER_LATEST", "/var/www/html/data/weather_latest.json")),
        guides: PathBuf::from(common::env_or("JUNSGARDEN_GUIDES_DIR", "/guides")),
    });

    // The router for static files (CSS, scripts, photos). `ServeDir` hands out files from
    // the site folder. The layers run in a set order: the outer layer `only_servable` first
    // checks the path is allowed, then `asset_headers` adds the cache rules.
    let assets = Router::new()
        .fallback_service(ServeDir::new(&www))
        .layer(middleware::from_fn(asset_headers))
        .layer(middleware::from_fn(only_servable));

    // `use pages::*;` brings every page module (index, blog, catalog, ...) into scope.
    use pages::*;
    // One handler pair reused for several addresses: GET shows the home page, POST handles forms.
    let home = get(index::home).post(index::home_post);
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
        .route("/catalog.php", get(catalog::catalog_get).post(catalog::catalog_post))
        .route("/gallery.php", get(gallery::gallery))
        .route("/comments.php", get(comments::comments).post(comments::comments))
        .route("/plant_actions.php", get(plant_actions::plant_actions).post(plant_actions::plant_actions))
        .route("/data.php", get(data::data_get).post(data::data_get))
        .route("/widget_data.php", get(data::widget_data))
        .route("/app_api.php", get(app_api::app_api).post(app_api::app_api).options(app_api::app_api))
        // The plant admin, converted from the Streamlit app (orchestrator/).
        .route("/streamlit/", get(orchestrator::page).post(orchestrator::action))
        .route("/streamlit", get(|| async { axum::response::Redirect::permanent("/streamlit/") }))
        .route("/streamlit/st.css", get(orchestrator::css))
        .route("/streamlit/st.js", get(orchestrator::js))
        // The app's camera tools, for people logged in (garden_camera.rs).
        .route("/app/garden/{action}", axum::routing::post(garden_camera::garden_camera))
        // The guides, for people logged in (help.rs): /help, /help/images/...
        .merge(help::routes())
        // Layers wrap every route above them. First, work out who the visitor is (session.rs).
        // Then add the security headers to every response.
        .layer(middleware::from_fn_with_state(state.clone(), session::with_visitor))
        .layer(middleware::map_response(page_headers))
        // Photo uploads: several full-size photos in one go.
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024))
        // The cards endpoint never loaded connection.php in the PHP, so it has
        // its own cache rule and none of the page headers: added AFTER the layer.
        .route("/sections/cards/index.php", get(index::cards))
        .with_state(state)
        .fallback_service(assets);

    // Start listening on the port (APP_PORT, default 8021) and handle requests.
    common::serve(app, common::env_u16("APP_PORT", 8021)).await
}

/// The program's entry point. Sets up the clock offset and an async runtime, then
/// runs `run()` to the end. The async runtime is what lets `async` functions run.
fn main() -> anyhow::Result<()> {
    common::init_local_offset();
    common::runtime()?.block_on(run())
}

// Tests: only compiled when running `cargo test`; they check `servable()`.
#[cfg(test)]
mod tests {
    use super::servable;

    // Ordinary public files must be allowed.
    #[test]
    fn serves_ordinary_assets() {
        for ok in ["/style.css", "/js/hero.js", "/images/Database/a.webp", "/privacy.html", "/sections/cards/card-01.json", "/images/Gallery/x.mp4"] {
            assert!(servable(ok), "{ok} should be served");
        }
    }

    // Source code, settings, backups and private folders must be refused.
    #[test]
    fn refuses_source_config_and_backups() {
        for bad in [
            "/connection.php", "/connection.php.bak-presso", "/index.php.pre-sso", "/.env",
            "/data/Data-Source.csv", "/data/comments.txt", "/garden-users.env", "/../etc/passwd",
            "/images/../connection.php", "/database/x.sql", "/app_api.php", "/sections/other.json",
            "/js/app.js.bak-before-scrollfix", "/cashe/AI/health.json", "/garden-ingest/vocab.py",
        ] {
            assert!(!servable(bad), "{bad} must not be served");
        }
    }
}
