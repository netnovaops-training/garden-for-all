//! Who is asking: the visitor cookie, the website login, and the app's key.
//!
//! HOW TO READ THIS FILE (for Jun). `Visitor` is the answer to "who is
//! asking?": a cookie id, and a name if they have logged in. `with_visitor`
//! runs before every page and fills that in. `is_admin()` is the one
//! question the pages ask. The passwords themselves are checked in
//! `accounts.rs`; this file only remembers who has already logged in.
//!
//! * Every visitor gets a `PHPSESSID` cookie (same name and id format as
//!   PHP, one day). For somebody not logged in that id IS their identity:
//!   favorites and likes are stored under `Guest_<id>`.
//! * **The website login** (`/login.php`) checks the password and then
//!   starts a NEW session id (`sign_in`), so an id somebody planted before
//!   the login is worthless afterwards. Logged-in sessions last a day from
//!   the last click, as PHP's did.
//! * **The app** logs in through `app_api.php?action=login` and gets a key
//!   (`issue_app_token`, 64 characters, good for 90 days). It sends that key
//!   as `Authorization: Bearer <key>` with every request; the key stands for
//!   the person who logged in. The app's Log out throws the key away.
//!
//! Sessions and keys live in the database (`garden_sessions`), so a restart
//! does not log anybody out and the table is plain to read. Changing a
//! password (`accounts::finish_reset`) deletes all of that person's rows.
//!
//! Until 2026-10-06 this file trusted the household sign-in's `X-Auth-Email`
//! header instead; Jason asked for the garden's own logins back.
// `use` brings names into scope so they can be written short. `Arc` is a shared pointer:
// many parts of the program can hold the same value, which is freed after the last one
// lets go.

use std::sync::Arc;

// axum is the web framework. These are its request, response and middleware types.
use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
// `rand::Rng` gives random numbers. `crate::` below means "from this same program".
use rand::Rng;

use crate::AppState;

/// Who is asking, as far as the site knows.
///
/// Built fresh for each request by `with_visitor` and stored in the request, where page
/// handlers pick it up. `#[derive(...)]` asks Rust to write standard behaviour for us:
/// `Clone` (make a copy), `Debug` (print for logs), `Default` (an empty value).
/// Who is asking, as far as the site knows.
#[derive(Clone, Debug, Default)]
pub struct Visitor {
    /// The session id: the value of the `PHPSESSID` cookie, or the app key.
    pub sid: String,
    /// The logged-in person's name ("Jun", "Jason"), or empty for a visitor.
    pub username: String,
    /// That person's address on file, or empty.
    pub email: String,
    /// Set when the request came from the app with its key, not a cookie.
    pub app_token: Option<String>,
}

/// Methods on `Visitor`. `&self` means a method reads the visitor and does not change it.
/// `bool` is true or false.
impl Visitor {
    /// `$isAdmin`: everybody who can log in is an admin here.
    pub fn is_admin(&self) -> bool {
        !self.username.is_empty()
    }

    /// The name favorites and likes are stored under: the person when logged
    /// in, otherwise `Guest_<session id>`, as the PHP did.
    pub fn db_name(&self) -> String {
        if self.is_admin() {
            self.username.clone()
        } else {
            format!("Guest_{}", self.sid)
        }
    }

    /// May this person upload plant photos? `JUNSGARDEN_UPLOADERS`.
    pub fn can_upload(&self, state: &AppState) -> bool {
        self.is_admin() && state.uploaders.iter().any(|u| u.eq_ignore_ascii_case(&self.username))
    }

    /// May this person see the planning links in the header? Anybody on the
    /// login list (which is everybody who can log in).
    pub fn plan_ok(&self, state: &AppState) -> bool {
        self.is_admin() && crate::accounts::find(state, &self.username).is_some()
    }
}

/// Create the session table if it is not there yet.
pub async fn ensure_table(db: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS `garden_sessions` (
            sid VARCHAR(128) NOT NULL PRIMARY KEY,
            username VARCHAR(100) NOT NULL,
            sso_email VARCHAR(190) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME NOT NULL,
            KEY expires_at (expires_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    )
    .execute(db)
    .await?;
    Ok(())
}

/// PHP accepts session ids made of letters, digits, comma and dash.
fn valid_sid(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b',' || b == b'-')
}

/// Makes a random hex string from `bytes` random bytes (so twice as many characters).
/// Used for session ids (16 bytes -> 32 characters) and app keys (32 -> 64).
/// Not `pub`: only this file uses it.
fn random_hex(bytes: usize) -> String {
    // A random number generator for this thread.
    let mut rng = rand::thread_rng();
    // Make `bytes` random bytes, write each as two hex digits, and join them into a String.
    // `|_|` is a closure (a small unnamed function) that ignores its input.
    (0..bytes).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
}

/// A new random cookie session id: 32 hex characters, like PHP's.
fn new_sid() -> String {
    random_hex(16)
}

/// App keys are 64 hex characters; cookie ids are 32. A cookie that carries
/// a key-shaped id is ignored, so a key only ever works as a key.
fn is_app_token(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Reads the `PHPSESSID` cookie from the request. Returns `Some(id)` if it is there and
/// looks valid, else `None` (`Option` means "a value, or nothing"). Takes `&Request`, a
/// borrowed request, so nothing is copied or consumed.
/// The `?` after a call means: if this gave `None` or an error, stop and return `None`.
fn cookie_sid(req: &Request) -> Option<String> {
    let raw = req.headers().get(header::COOKIE)?.to_str().ok()?;
    // The Cookie header looks like `a=1; PHPSESSID=abc; b=2`. Split it into the pieces.
    raw.split(';')
        .filter_map(|p| p.trim().split_once('='))
        .find(|(k, _)| *k == "PHPSESSID")
        .map(|(_, v)| v.to_string())
        .filter(|v| valid_sid(v) && !is_app_token(v))
}

/// Reads the app key from the `Authorization: Bearer <key>` header. Returns `None` if
/// the header is missing or the key is not 64 hex characters.
fn bearer(req: &Request) -> Option<String> {
    let raw = req.headers().get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = raw.strip_prefix("Bearer ").or_else(|| raw.strip_prefix("bearer "))?.trim();
    // `then(...)` gives `Some(...)` only when the check before it is true.
    is_app_token(token).then(|| token.to_string())
}

/// Reads a request header as text, or "" if missing. The `'a` marks that the returned
/// text is borrowed from the request, so it can only be used while the request exists.
/// Is this name still on the user list (GARDEN_USERS)? Someone taken off the
/// list by editing .env by hand loses a login they still had, at once.
fn still_a_user(state: &AppState, name: &str) -> bool {
    state.users.iter().any(|u| u.name == name)
}

fn header_str<'a>(req: &'a Request, name: &str) -> &'a str {
    req.headers().get(name).and_then(|v| v.to_str().ok()).unwrap_or("")
}

/// Looks up a session id (or app key) in the database. Returns the username and email
/// stored for it, or `None` if there is no such row or it has expired.
async fn load_session(state: &AppState, sid: &str) -> Option<(String, String)> {
    sqlx::query_as::<_, (String, String)>(
        // `?` in the SQL marks a value filled in safely by `.bind` (it is never pasted in).
        "SELECT username, sso_email FROM `garden_sessions` WHERE sid = ? AND expires_at > NOW()",
    )
    .bind(sid)
    .fetch_optional(&state.db)
    .await
    // A database error is treated like "no session": the person is just a visitor.
    .ok()
    .flatten()
}

/// Writes (or renews) a session row that lasts `days` days from now. Errors are logged
/// and otherwise ignored, so a database hiccup does not break the page.
async fn save_session(state: &AppState, sid: &str, username: &str, email: &str, days: i64) {
    // If the id already exists, the ON DUPLICATE KEY part updates that row (renewing the
    // expiry) instead of failing. (A comment cannot go inside the SQL text itself.)
    let r = sqlx::query(
        "INSERT INTO `garden_sessions` (sid, username, sso_email, expires_at)
         VALUES (?, ?, ?, NOW() + INTERVAL ? DAY)
         ON DUPLICATE KEY UPDATE username = VALUES(username), sso_email = VALUES(sso_email),
                                 expires_at = VALUES(expires_at)",
    )
    .bind(sid)
    .bind(username)
    .bind(email)
    .bind(days)
    .execute(&state.db)
    .await;
    // `if let Err(e) = r` runs the block only when saving failed.
    if let Err(e) = r {
        tracing::error!("could not save session: {e}");
    }
}

/// `session_unset()` / `session_destroy()`: forget this session or key.
pub async fn destroy(state: &AppState, sid: &str) {
    // `let _ =` throws the result away on purpose: forgetting a session cannot fail usefully.
    let _ = sqlx::query("DELETE FROM `garden_sessions` WHERE sid = ?")
        .bind(sid)
        .execute(&state.db)
        .await;
}

/// After a correct password on the website: a brand-new session id for this
/// person. The caller sends it back with `session_cookie`.
pub async fn sign_in(state: &AppState, username: &str) -> String {
    // The email on file for this person, or empty if they have no account.
    let email = crate::accounts::find(state, username).map(|a| a.email.clone()).unwrap_or_default();
    // A new random id each login, so an id planted before the login is useless.
    let sid = new_sid();
    // Lasts 1 day.
    save_session(state, &sid, username, &email, 1).await;
    sid
}

/// After a correct password in the app: a key good for 90 days.
pub async fn issue_app_token(state: &AppState, username: &str) -> String {
    let email = crate::accounts::find(state, username).map(|a| a.email.clone()).unwrap_or_default();
    // 32 random bytes = a 64-character key.
    let token = random_hex(32);
    // App keys last 90 days.
    save_session(state, &token, username, &email, 90).await;
    token
}

/// Occasionally sweep expired rows so the table does not grow forever.
// About once in 200 requests (a random number 0..200 equals 0), delete expired rows.
async fn sweep(state: &AppState) {
    if rand::thread_rng().gen_range(0..200) == 0 {
        let _ = sqlx::query("DELETE FROM `garden_sessions` WHERE expires_at < NOW()")
            .execute(&state.db)
            .await;
    }
}

/// The `Set-Cookie` value for a session id, with a one-day life. PHP sends
/// no HttpOnly or SameSite flag; both are added: the site's own scripts never
/// read this cookie, and SameSite=Strict means the browser never sends it on
/// a request that another website started (not even a plain link), so a
/// stranger's page cannot make the site act as the logged-in person.
/// (Lax still sent it on links, and some actions here accept a link.)
/// Secure is added when the request came over HTTPS.
pub fn session_cookie(sid: &str, https: bool) -> Option<HeaderValue> {
    // The cookie expires one day from now, written as a UTC date in the format cookies use.
    let expires = common::now_local()
        .saturating_add(time::Duration::days(1))
        .to_offset(time::UtcOffset::UTC)
        .format(&time::macros::format_description!(
            "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT"
        ))
        .unwrap_or_default();
    // Build the cookie text. HttpOnly hides it from scripts; SameSite=Strict stops other
    // sites using it; Max-Age=86400 is one day in seconds; Secure only over HTTPS.
    HeaderValue::from_str(&format!(
        "PHPSESSID={sid}; expires={expires}; Max-Age=86400; path=/; HttpOnly; SameSite=Strict{}",
        if https { "; Secure" } else { "" }
    ))
    .ok()
}

/// Was this request made over HTTPS (as Caddy reports it)?
pub fn is_https(headers: &axum::http::HeaderMap) -> bool {
    // Caddy (the front web server) tells us the original scheme in this header.
    headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https")
}

/// Middleware for the pages: work out who is asking, make it available to
/// the handler, and (re)send the cookie the way PHP does.
pub async fn with_visitor(State(state): State<Arc<AppState>>, mut req: Request, next: Next) -> Response {
    // Did the request arrive over HTTPS?
    let https = header_str(&req, "x-forwarded-proto") == "https";

    // The app, with its key. Nothing else about the request matters.
    // The app's key wins: if it is there, the cookie is not looked at.
    if let Some(token) = bearer(&req) {
        // `..Default::default()` fills all other fields with empty values.
        let mut visitor = Visitor { sid: token.clone(), ..Default::default() };
        // Only a key found in the database (and not expired) makes this a logged-in visitor.
        if let Some((name, email)) = load_session(&state, &token).await {
            if still_a_user(&state, &name) {
                visitor.username = name;
                visitor.email = email;
                visitor.app_token = Some(token);
            }
        }
        // Attach the visitor to the request so handlers can read it, then run the page.
        req.extensions_mut().insert(visitor);
        return next.run(req).await;
    }

    // Use the cookie's id, or make a new one. `unwrap_or_else` runs the fallback only if needed.
    let sid = cookie_sid(&req).unwrap_or_else(new_sid);
    let mut visitor = Visitor { sid: sid.clone(), ..Default::default() };
    // A request that ANOTHER website started (a link or form on a stranger's
    // page)? Then treat it as a visitor, even with a login cookie: a strange
    // page must never be able to act as the logged-in person. Browsers say so
    // in Sec-Fetch-Site; "same-origin" (this site) and "none" (typed, a
    // bookmark) are fine. Older browsers send nothing, and the Strict cookie
    // covers them.
    let from_elsewhere = matches!(header_str(&req, "sec-fetch-site"), "cross-site" | "same-site");
    // A known, unexpired session: this visitor is logged in.
    if let Some((name, email)) = if from_elsewhere { None } else { load_session(&state, &sid).await } {
        // Only while the person is still on the user list (GARDEN_USERS).
        if still_a_user(&state, &name) {
            // Keep the session alive for another day from this click, as PHP's
            // session files are.
            save_session(&state, &sid, &name, &email, 1).await;
            visitor.username = name;
            visitor.email = email;
        }
    }
    // Now and then, clean out old rows.
    sweep(&state).await;

    // Run the rest of the request (the page handler) and get its response.
    req.extensions_mut().insert(visitor);
    let mut res = next.run(req).await;

    // A login (or logout) sets its own, new cookie; do not overwrite it.
    let already = res
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .any(|v| v.to_str().map(|s| s.starts_with("PHPSESSID=")).unwrap_or(false));
    if !already {
        // No new cookie from the page: send the usual one, which keeps the id and renews the day.
        if let Some(v) = session_cookie(&sid, https) {
            res.headers_mut().append(header::SET_COOKIE, v);
        }
    }
    res
}

/// The Logout button: forget the session and go back to the same page
/// without its query string, as `connection.php` does.
pub async fn logout_redirect(state: &AppState, visitor: &Visitor, path: &str) -> Response {
    // Delete the session row so the cookie no longer logs anyone in.
    destroy(state, &visitor.sid).await;
    // Keep only the part of the address before any `?`.
    let to = path.split('?').next().unwrap_or("/").to_string();
    // 302 Found plus a Location header sends the browser to that address.
    (StatusCode::FOUND, [(header::LOCATION, to)]).into_response()
}

// Tests: only compiled with `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_ids_follow_php_rules() {
        assert!(valid_sid("f07b2c8ec9f216ded2138c757da1932f"));
        assert!(valid_sid("a,b-c"));
        assert!(!valid_sid(""));
        assert!(!valid_sid("a b"));
        assert!(!valid_sid("a;b"));
    }

    #[test]
    fn new_ids_are_32_hex_and_keys_64() {
        let s = new_sid();
        assert_eq!(s.len(), 32);
        assert!(s.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(!is_app_token(&s));
        let k = random_hex(32);
        assert!(is_app_token(&k));
    }

    #[test]
    fn guest_and_logged_in_names() {
        let v = Visitor { sid: "abc".into(), ..Default::default() };
        assert_eq!(v.db_name(), "Guest_abc");
        assert!(!v.is_admin());
        let j = Visitor { sid: "abc".into(), username: "Jun".into(), ..Default::default() };
        assert_eq!(j.db_name(), "Jun");
        assert!(j.is_admin());
    }
}
