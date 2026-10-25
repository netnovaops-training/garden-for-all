//! The site's guards: things every request passes that keep it safe and quick.
//!
//! HOW TO READ THIS FILE (for Jun). Four small tools, each used from `main.rs` or a page:
//!
//! * `security_headers` adds the browser safety rules to every page (what may load, no
//!   framing, no sniffing). They were copied from the old PHP site and are kept exactly;
//!   three newer ones are added (see the function).
//! * `same_site_posts` refuses a form post that another website's page sent. The login
//!   cookie is already `SameSite=Strict`, so a stranger's page cannot act as Jun; this
//!   also stops it filling the guestbook or logging a visitor in to a different account.
//! * `RateLimit` counts attempts per visitor address in memory ("5 guestbook lines per
//!   10 minutes"), for the forms anybody may use.
//! * `off_thread` runs slow, blocking work (reading or writing files, resizing photos)
//!   on a helper thread, so one big photo never holds up everybody else's page.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::{
    extract::Request,
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// The headers every rendered page gets. The first seven are the PHP site's own (copied
/// from its live responses); `X-Powered-By` is left out on purpose, it only told strangers
/// which version to attack. The last three are current practice and change nothing visible:
/// no camera/microphone/location for any page, the page cannot be grabbed by a window
/// another site opened, and the site's files cannot be embedded by other sites.
const PAGE_HEADERS: [(&str, &str); 10] = [
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
    ("permissions-policy", "camera=(), microphone=(), geolocation=(), payment=(), usb=()"),
    ("cross-origin-opener-policy", "same-origin"),
    ("cross-origin-resource-policy", "same-origin"),
];

/// Adds `PAGE_HEADERS` to a finished page. A page that chose its own Cache-Control
/// (the comments API, the QR images) keeps it.
pub async fn security_headers(mut res: Response) -> Response {
    let h = res.headers_mut();
    for (name, value) in PAGE_HEADERS {
        if name == "cache-control" && h.contains_key(name) {
            continue;
        }
        h.insert(HeaderName::from_static(name), HeaderValue::from_static(value));
    }
    res
}

/// Refuse a POST that another website started.
///
/// Browsers say where a request came from: `Sec-Fetch-Site` (all current browsers) and
/// `Origin` (every POST from a page). A post is let through when it is from this site, or
/// carries neither header (the Android app, curl, very old browsers; the app proves who it
/// is with its key, and the Strict cookie covers the old browsers).
pub async fn same_site_posts(req: Request, next: Next) -> Response {
    if req.method() == Method::POST && !from_this_site(req.headers()) {
        tracing::warn!("refused a cross-site POST to {}", req.uri().path());
        return (StatusCode::FORBIDDEN, "This form must be sent from the garden site itself.").into_response();
    }
    next.run(req).await
}

/// True when the request was not started by another website (see `same_site_posts`).
fn from_this_site(h: &HeaderMap) -> bool {
    let text = |name: &str| h.get(name).and_then(|v| v.to_str().ok());
    if let Some(site) = text("sec-fetch-site") {
        return matches!(site, "same-origin" | "none");
    }
    match (text("origin"), text("x-forwarded-host").or_else(|| text(header::HOST.as_str()))) {
        // "null" is what a sandboxed or file:// page sends: not this site.
        (Some(origin), Some(host)) => origin.split_once("://").map(|(_, rest)| rest) == Some(host),
        (Some(_), None) => false,
        (None, _) => true,
    }
}

/// "At most `max` tries per `window` per visitor address", kept in memory.
///
/// A restart forgets the counts, which is fine for what it guards (guestbook lines,
/// comments): it slows a flood down, it is not an account lockout (that one is in the
/// database, see accounts.rs).
pub struct RateLimit {
    max: usize,
    window: Duration,
    hits: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimit {
    pub fn new(max: usize, window: Duration) -> Self {
        RateLimit { max, window, hits: Mutex::new(HashMap::new()) }
    }

    /// Count one try from `who`; false when that address has used up its tries.
    pub fn allow(&self, who: &str) -> bool {
        let now = Instant::now();
        let Ok(mut hits) = self.hits.lock() else { return true };
        // Forget addresses whose tries are all old, so the table cannot grow forever.
        if hits.len() > 10_000 {
            hits.retain(|_, t| t.iter().any(|at| now.duration_since(*at) < self.window));
        }
        let tries = hits.entry(who.to_string()).or_default();
        tries.retain(|at| now.duration_since(*at) < self.window);
        if tries.len() >= self.max {
            return false;
        }
        tries.push(now);
        true
    }
}

/// Run blocking work (files, photo resizing) on Tokio's helper threads and wait for it.
/// If the work itself panics, that is logged and `T::default()` is returned, so a bad file
/// gives the same "nothing there" answer the page would give for a missing one.
pub async fn off_thread<T, F>(work: F) -> T
where
    T: Default + Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(work).await.unwrap_or_else(|e| {
        tracing::error!("background work failed: {e}");
        T::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(*k, HeaderValue::from_static(v));
        }
        h
    }

    #[test]
    fn posts_from_the_site_itself_or_the_app_pass() {
        assert!(from_this_site(&headers(&[("sec-fetch-site", "same-origin")])));
        assert!(from_this_site(&headers(&[("sec-fetch-site", "none")])));
        assert!(from_this_site(&headers(&[]))); // the app, curl
        assert!(from_this_site(&headers(&[("origin", "https://junsgarden.zapto.org"), ("host", "junsgarden.zapto.org")])));
        assert!(from_this_site(&headers(&[
            ("origin", "https://junsgarden.zapto.org"),
            ("host", "127.0.0.1:8031"),
            ("x-forwarded-host", "junsgarden.zapto.org"),
        ])));
    }

    #[test]
    fn posts_from_other_sites_are_refused() {
        assert!(!from_this_site(&headers(&[("sec-fetch-site", "cross-site")])));
        assert!(!from_this_site(&headers(&[("sec-fetch-site", "same-site")])));
        assert!(!from_this_site(&headers(&[("origin", "https://evil.example"), ("host", "junsgarden.zapto.org")])));
        assert!(!from_this_site(&headers(&[("origin", "null"), ("host", "junsgarden.zapto.org")])));
    }

    #[test]
    fn rate_limit_counts_per_address_and_window() {
        let rl = RateLimit::new(2, Duration::from_secs(60));
        assert!(rl.allow("a"));
        assert!(rl.allow("a"));
        assert!(!rl.allow("a"));
        assert!(rl.allow("b"));
        let quick = RateLimit::new(1, Duration::from_millis(1));
        assert!(quick.allow("a"));
        std::thread::sleep(Duration::from_millis(5));
        assert!(quick.allow("a"));
    }

    #[tokio::test]
    async fn off_thread_returns_the_work_and_survives_a_panic() {
        assert_eq!(off_thread(|| 2 + 2).await, 4);
        let v: Vec<u8> = off_thread(|| panic!("bad file")).await;
        assert!(v.is_empty());
    }
}
