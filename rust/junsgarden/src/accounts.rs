//! The garden's own logins: the people in GARDEN_USERS (Jun), a password each, and "forgot my
//! password" by email.
//!
//! HOW TO READ THIS FILE (for Jun). There are only two accounts, listed in
//! the setting `JUNSGARDEN_USERS` as `Name=email,Name=email`. Nobody can
//! create another one from the website. Each account's password is stored
//! only as an Argon2 HASH in the `garden_users` table: a one-way scramble
//! the site can check a password against but can never turn back into the
//! password. So reading the database does not reveal anybody's password.
//!
//! The pieces, top to bottom:
//! * `ensure_tables` makes the tables and adds the accounts at startup.
//! * `check_password` is the login: it answers "is this one of the garden's accounts?",
//!   and counts failures so guessing is slow (5 wrong tries from one
//!   address, or 10 against one name, and it waits 15 minutes).
//! * `start_reset` / `finish_reset` are "forgot my password": a link with a
//!   one-time code is emailed to the address on file and works for 30
//!   minutes. That is also how each person sets their FIRST password.
//! * `send_mail` sends that email through an ordinary SMTP account (Gmail
//!   with an app password works; see SETUP-ON-A-NEW-MACHINE.md).
//!
//! History: until 2026-09-19 the PHP site had a password login much like
//! this (bcrypt hashes in a file). It was replaced by the household sign-in,
//! and on 2026-10-06 the author asked for passwords back, for Jun and himself
//! only, with resets going to their own addresses.
// `use` brings names into scope so they can be written short. `OnceLock` holds a value
// that is created once, the first time it is needed, and then reused.

use std::sync::OnceLock;

// The argon2 library hashes and checks passwords. sha2 makes SHA-256 hashes.
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand::Rng;
use sha2::{Digest, Sha256};

// `crate::` means "from this same program".
use crate::AppState;

/// Who may log in, when `JUNSGARDEN_USERS` is not set.
///
/// `pub const` is a fixed value other files can read. Your install always sets
/// `JUNSGARDEN_USERS` from `.env` (GARDEN_USERS). Without it, NOBODY can log in:
/// there is no built-in account, so a copy of this public code that someone
/// else runs never has a login (or sends reset emails) for Jun.
pub const DEFAULT_USERS: &str = "";

/// How long a reset link works, and the lockout window.
/// Minutes a reset link stays valid.
const RESET_MINUTES: i64 = 30;
/// The lockout window, in seconds (900 seconds = 15 minutes).
const WINDOW_SECONDS: i64 = 900;
/// Failed logins allowed from one address inside the window before login is blocked.
const MAX_FAILS_PER_IP: i64 = 5;
/// Failed logins allowed against one account name inside the window.
const MAX_FAILS_PER_NAME: i64 = 10;

/// One login account: a display name and the email address on file.
/// `#[derive(Clone, Debug)]` asks Rust to write code for copying the value and for
/// printing it in logs.
#[derive(Clone, Debug)]
pub struct Account {
    /// The account's name, e.g. what is typed in the login box.
    pub name: String,
    /// The address reset emails go to. Stored in lower case.
    pub email: String,
}

/// Parse `Name=email,Name=email`.
// `&str` is borrowed text. `Vec<Account>` is a growable list of accounts. The chain below
// reads like a pipeline: split at commas, split each piece at `=`, build an `Account`,
// drop the invalid ones, and collect what is left into the list.
pub fn parse_users(spec: &str) -> Vec<Account> {
    spec.split(',')
        // `filter_map` keeps only pieces that have an `=` (`split_once` gives `Some` or `None`).
        .filter_map(|p| p.split_once('='))
        .map(|(n, e)| Account { name: n.trim().to_string(), email: e.trim().to_ascii_lowercase() })
        // Keep only entries with a name and an address that has an @ in it.
        .filter(|a| !a.name.is_empty() && a.email.contains('@'))
        .collect()
}

/// Find an account by its name, ignoring capitals ("jun" is Jun).
///
/// `'a` is a lifetime: it says the returned account is borrowed from `state`, so it can
/// only be used while `state` exists. `Option` is "an account, or nothing".
pub fn find<'a>(state: &'a AppState, name: &str) -> Option<&'a Account> {
    let name = name.trim();
    state.users.iter().find(|a| a.name.eq_ignore_ascii_case(name))
}

/// Find an account by name OR email address, for "forgot my password".
fn find_by_name_or_email<'a>(state: &'a AppState, who: &str) -> Option<&'a Account> {
    let who = who.trim();
    state.users.iter().find(|a| a.name.eq_ignore_ascii_case(who) || a.email.eq_ignore_ascii_case(who))
}

/// Creates the tables the login needs and makes sure each configured account has a row.
/// Safe to run on every start: every command says `IF NOT EXISTS` or updates in place.
/// Returns `Ok(())` on success. `Result<(), sqlx::Error>` is "nothing, or a database error",
/// and the `?` after a call means: on error, stop and return that error.
pub async fn ensure_tables(db: &sqlx::MySqlPool, users: &[Account]) -> Result<(), sqlx::Error> {
    // The accounts table. `pass_hash` stays empty until the person sets a password.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS `garden_users` (
            username VARCHAR(50) NOT NULL PRIMARY KEY,
            email VARCHAR(190) NOT NULL,
            pass_hash VARCHAR(255) NULL,
            reset_hash CHAR(64) NULL,
            reset_expires DATETIME NULL,
            reset_sent_at DATETIME NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    )
    .execute(db)
    .await?;
    // The same table the PHP login used, plus the name that was tried.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS `login_attempts` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            ip VARCHAR(45) NOT NULL,
            attempted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_ip_time (ip, attempted_at)
        ) ENGINE=InnoDB",
    )
    .execute(db)
    .await?;
    // An older install may lack the `username` column; add it only if it is missing.
    sqlx::query("ALTER TABLE `login_attempts` ADD COLUMN IF NOT EXISTS username VARCHAR(50) NULL")
        .execute(db)
        .await?;
    // The accounts themselves. A password is never set here: each person
    // sets their own through the emailed link.
    // One row per account. If the row exists, only refresh the email address.
    for a in users {
        sqlx::query(
            "INSERT INTO `garden_users` (username, email) VALUES (?, ?)
             ON DUPLICATE KEY UPDATE email = VALUES(email)",
        )
        .bind(&a.name)
        .bind(&a.email)
        .execute(db)
        .await?;
    }
    Ok(())
}

// --- passwords ----------------------------------------------------------------

/// Argon2 is deliberately slow (that is what makes guessing expensive), so
/// it runs off the async threads.
// `async fn` can pause while waiting. `pw: String` is text this function owns (it is moved
// into the closure below). Returns `Some(hash)` or `None` if hashing failed.
async fn hash_password(pw: String) -> Option<String> {
    // `spawn_blocking` runs slow work on a separate thread. `move ||` is a closure (a small
    // unnamed function) that takes ownership of the values it uses.
    tokio::task::spawn_blocking(move || {
        // A random salt makes two identical passwords hash differently.
        let salt = SaltString::generate(&mut OsRng);
        // Hash with default Argon2 settings and turn the hash into text.
        Argon2::default().hash_password(pw.as_bytes(), &salt).ok().map(|h| h.to_string())
    })
    .await
    .ok()
    .flatten()
}

/// Checks a typed password against a stored hash. Returns true only if they match.
/// Like `hash_password`, it runs on a blocking thread because Argon2 is slow on purpose.
async fn verify_password(pw: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        // Read the stored hash text; if it is not a valid hash, the answer is false.
        PasswordHash::new(&hash)
            .map(|h| Argon2::default().verify_password(pw.as_bytes(), &h).is_ok())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

/// A hash of nothing in particular, checked when the name is unknown or has
/// no password yet, so a wrong NAME takes as long as a wrong PASSWORD and
/// the timing does not reveal which names exist.
fn dummy_hash() -> String {
    // A value created once, the first time this runs, and kept for the whole program.
    static H: OnceLock<String> = OnceLock::new();
    H.get_or_init(|| {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(b"not-a-real-password", &salt)
            .map(|h| h.to_string())
            .unwrap_or_default()
    })
    .clone()
}

/// Why a new password is not acceptable, or None if it is fine.
// `&'static str` is text written into the program itself, which lives for the whole run.
pub fn password_problem(name: &str, pw: &str) -> Option<&'static str> {
    // Passwords too common to allow. All have at least 10 characters, so the length rule
    // below does not already catch them.
    const TOO_COMMON: [&str; 12] = [
        "password", "password1", "passw0rd", "1234567890", "qwertyuiop", "letmein123",
        "welcome123", "iloveyou12", "gardengarden", "junsgarden", "konasgarden", "changeme12",
    ];
    // Count characters, not bytes.
    let n = pw.chars().count();
    // Minimum length 10.
    if n < 10 {
        return Some("Use at least 10 characters. A short sentence is easy to remember and hard to guess.");
    }
    // Maximum length 128, to keep hashing time reasonable.
    if n > 128 {
        return Some("That is longer than 128 characters.");
    }
    // Not the same as the account name, and not on the common list (compared ignoring capitals).
    if pw.eq_ignore_ascii_case(name) || TOO_COMMON.iter().any(|c| pw.eq_ignore_ascii_case(c)) {
        return Some("That one is too easy to guess. Pick something else.");
    }
    // `None` here means no problem was found.
    None
}

/// Why a login attempt failed. An `enum` is a type that is exactly one of several named
/// choices. The login page uses this to pick which message to show.
pub enum LoginError {
    /// Wrong name or wrong password (deliberately not saying which).
    Invalid,
    /// Too many failures recently; wait.
    TooMany,
}

/// The login. Returns the account's proper name ("Jun") when the password
/// is right.
// What it takes in: the shared app state, the typed name, the typed password and the
// visitor's address. Gives back `Ok(proper name)` or `Err(LoginError)`.
pub async fn check_password(state: &AppState, typed_name: &str, password: &str, ip: &str) -> Result<String, LoginError> {
    // `&` borrows the database pool from the state instead of copying it.
    let db = &state.db;
    // Step 1: forget failed attempts older than the lockout window.
    let _ = sqlx::query("DELETE FROM `login_attempts` WHERE attempted_at < NOW() - INTERVAL ? SECOND")
        .bind(WINDOW_SECONDS)
        .execute(db)
        .await;

    // Step 2: look up the account. It may be `None` for an unknown name.
    let account = find(state, typed_name);
    // Count recent failures from this address inside the window.
    let by_ip: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM `login_attempts` WHERE ip = ? AND attempted_at > NOW() - INTERVAL ? SECOND",
    )
    .bind(ip)
    .bind(WINDOW_SECONDS)
    .fetch_one(db)
    .await
    .unwrap_or(0);
    // Count recent failures against this account name (zero for an unknown name).
    let by_name: i64 = match account {
        Some(a) => sqlx::query_scalar(
            "SELECT COUNT(*) FROM `login_attempts` WHERE username = ? AND attempted_at > NOW() - INTERVAL ? SECOND",
        )
        .bind(&a.name)
        .bind(WINDOW_SECONDS)
        .fetch_one(db)
        .await
        .unwrap_or(0),
        None => 0,
    };
    // Step 3: too many failures means refuse right away, without checking the password.
    if by_ip >= MAX_FAILS_PER_IP || by_name >= MAX_FAILS_PER_NAME {
        return Err(LoginError::TooMany);
    }

    // Step 4: get this account's stored password hash. There may be none if never set.
    let stored: Option<String> = match account {
        Some(a) => sqlx::query_scalar("SELECT pass_hash FROM `garden_users` WHERE username = ?")
            .bind(&a.name)
            .fetch_optional(db)
            .await
            .ok()
            .flatten()
            .flatten(),
        None => None,
    };
    // Step 5: check the password. For an unknown name or an account with no password yet, check
    // against a dummy hash anyway so the response takes the same time (a timing defence).
    let ok = match (&account, stored) {
        (Some(_), Some(h)) => verify_password(password.to_string(), h).await,
        _ => {
            let _ = verify_password(password.to_string(), dummy_hash()).await;
            false
        }
    };

    // Step 6a: success. Clear this address's failure count and return the account's name.
    if ok {
        let _ = sqlx::query("DELETE FROM `login_attempts` WHERE ip = ?").bind(ip).execute(db).await;
        Ok(account.map(|a| a.name.clone()).unwrap_or_default())
    // Step 6b: failure. Record it, then answer with the same message for any cause.
    } else {
        let _ = sqlx::query("INSERT INTO `login_attempts` (ip, username) VALUES (?, ?)")
            .bind(ip)
            .bind(account.map(|a| a.name.clone()))
            .execute(db)
            .await;
        Err(LoginError::Invalid)
    }
}

// --- forgot my password -------------------------------------------------------

/// SHA-256 of the text, written as 64 hex characters. Used to store reset codes without
/// storing the codes themselves.
fn sha256_hex(s: &str) -> String {
    Sha256::digest(s.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// A new random reset code: 32 random bytes written as 64 hex characters. This is the
/// secret part of the emailed link.
fn new_reset_code() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// "Forgot my password": email a one-time link to the address on file. Says
/// nothing back about whether the name existed (the page shows the same
/// message either way), and sends at most one email every two minutes per
/// account so the button cannot be used to flood anybody's inbox.
pub async fn start_reset(state: &AppState, who: &str) {
    // `let Some(..) = .. else { return }`: take the account if found; otherwise stop silently.
    // Nothing is said either way, so nobody can learn which names exist.
    let Some(account) = find_by_name_or_email(state, who) else { return };
    // Rate limit: has an email already been sent for this account in the last 2 minutes?
    let recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM `garden_users` WHERE username = ? AND reset_sent_at > NOW() - INTERVAL 2 MINUTE",
    )
    .bind(&account.name)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);
    if recent > 0 {
        return;
    }
    // Make a new secret code.
    let code = new_reset_code();
    // Only the HASH of the code is stored, like a password: somebody reading
    // the database cannot use a link that is still waiting in an inbox.
    let r = sqlx::query(
        "UPDATE `garden_users` SET reset_hash = ?, reset_expires = NOW() + INTERVAL ? MINUTE,
         reset_sent_at = NOW() WHERE username = ?",
    )
    .bind(sha256_hex(&code))
    .bind(RESET_MINUTES)
    .bind(&account.name)
    .execute(&state.db)
    .await;
    if let Err(e) = r {
        tracing::error!("could not store a reset code: {e}");
        return;
    }
    // The link always points at the site's own address (a setting), never
    // at whatever Host the request claimed, so a forged request cannot make
    // the email send somebody's code to another website.
    // Build the link: the site's own address, the code, and the account name.
    let link = format!(
        "{}/login.php?reset={}&u={}",
        state.public_url.trim_end_matches('/'),
        code,
        crate::php::urlencode(&account.name)
    );
    // The email text. `{name}` is replaced by the account name. A backslash at the end of
    // a line joins it to the next line.
    let body = format!(
        "Hello {name},\n\n\
         Somebody (hopefully you) asked to set or reset the password for\n\
         {name}'s login on Jun's and Kona's Garden.\n\n\
         Open this link within {RESET_MINUTES} minutes and choose a password:\n\n\
         {link}\n\n\
         The link works once. If you did not ask for this, ignore this email:\n\
         nothing changes until the link is used.\n",
        name = account.name,
    );
    // Send it and log what happened. Neither the code nor the link is logged.
    match send_mail(&account.email, "Jun's Garden: set your password", &body).await {
        Ok(true) => tracing::info!("sent a password link to {}'s address", account.name),
        Ok(false) => tracing::warn!(
            "no outbox configured (JUNSGARDEN_SMTP_*), so {}'s password link was not sent",
            account.name
        ),
        Err(e) => tracing::error!("could not send {}'s password link: {e}", account.name),
    }
}

/// Is this a live, unused reset link for this account?
pub async fn reset_code_valid(state: &AppState, name: &str, code: &str) -> bool {
    // Unknown account name: not valid.
    let Some(account) = find(state, name) else { return false };
    // A real code is exactly 64 hex characters; reject anything else before touching the database.
    if code.len() != 64 || !code.bytes().all(|b| b.is_ascii_hexdigit()) {
        return false;
    }
    // Count rows with this name, this code's hash, and an expiry still in the future.
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM `garden_users` WHERE username = ? AND reset_hash = ? AND reset_expires > NOW()",
    )
    .bind(&account.name)
    .bind(sha256_hex(code))
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);
    n > 0
}

/// Use a reset link: set the new password, spend the link, and sign the
/// account out everywhere (a stolen session should not survive a reset).
// Returns `Ok(account name)` or `Err(message to show the person)`.
pub async fn finish_reset(state: &AppState, name: &str, code: &str, new_password: &str) -> Result<String, &'static str> {
    // Each early `return Err(..)` stops with a plain-words message.
    let Some(account) = find(state, name) else { return Err("That link is not valid.") };
    // The code must still be valid and unused.
    if !reset_code_valid(state, &account.name, code).await {
        return Err("That link has expired or was already used. Ask for a new one.");
    }
    // The new password must pass the rules in `password_problem`.
    if let Some(problem) = password_problem(&account.name, new_password) {
        return Err(problem);
    }
    // Hash the new password. The plain password is never stored.
    let Some(hash) = hash_password(new_password.to_string()).await else {
        return Err("Something went wrong saving the password. Try again.");
    };
    // Save the new hash and clear the reset code so the link works only once.
    sqlx::query(
        "UPDATE `garden_users` SET pass_hash = ?, reset_hash = NULL, reset_expires = NULL WHERE username = ?",
    )
    .bind(&hash)
    .bind(&account.name)
    .execute(&state.db)
    .await
    .map_err(|_| "Something went wrong saving the password. Try again.")?;
    // Sign this person out everywhere by deleting all their session rows.
    let _ = sqlx::query("DELETE FROM `garden_sessions` WHERE username = ?")
        .bind(&account.name)
        .execute(&state.db)
        .await;
    Ok(account.name.clone())
}

// --- email --------------------------------------------------------------------

/// Send one plain-text email. `Ok(false)` means no outbox is configured.
///
/// Settings: JUNSGARDEN_SMTP_HOST (e.g. smtp.gmail.com), JUNSGARDEN_SMTP_PORT
/// (587), JUNSGARDEN_SMTP_USER, JUNSGARDEN_SMTP_PASSWORD (for Gmail, an APP
/// password, not the account password), JUNSGARDEN_MAIL_FROM.
// `anyhow::Result<bool>`: `Ok(true)` sent, `Ok(false)` not configured, or an error.
pub async fn send_mail(to: &str, subject: &str, body: &str) -> anyhow::Result<bool> {
    // `use` inside a function makes the names available only here.
    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

    // Read the outbox settings. The password setting is read here and never logged.
    let host = common::env_or("JUNSGARDEN_SMTP_HOST", "");
    let user = common::env_or("JUNSGARDEN_SMTP_USER", "");
    let password = common::env_or("JUNSGARDEN_SMTP_PASSWORD", "");
    // Without a host, user and password there is no outbox: report "not sent" without error.
    if host.is_empty() || user.is_empty() || password.is_empty() {
        return Ok(false);
    }
    // The From address defaults to the SMTP user.
    let from = common::env_or("JUNSGARDEN_MAIL_FROM", &user);
    // Build the message. Each `.parse()?` checks an address is valid.
    let message = Message::builder()
        .from(format!("Jun's and Kona's Garden <{from}>").parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())?;
    // STARTTLS on 587: encrypted, and the port every provider documents.
    let mailer: AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)?
        .port(common::env_or("JUNSGARDEN_SMTP_PORT", "587").parse().unwrap_or(587))
        .credentials(Credentials::new(user, password))
        .timeout(Some(std::time::Duration::from_secs(20)))
        .build();
    mailer.send(message).await?;
    Ok(true)
}

/// The visitor's address, for the lockout. Caddy puts the real one in
/// X-Forwarded-For; the last entry is the one Caddy itself saw.
pub fn client_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.rsplit(',').next())
        .map(|s| s.trim().chars().take(45).collect::<String>())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn users_parse() {
        let u = parse_users("Jun=A@B.com, Mina=c@d.com ,bad,=x@y.z");
        assert_eq!(u.len(), 2);
        assert_eq!(u[0].name, "Jun");
        assert_eq!(u[0].email, "a@b.com");
        // No built-in account: without GARDEN_USERS nobody can log in.
        assert!(parse_users(DEFAULT_USERS).is_empty());
    }

    #[test]
    fn password_rules() {
        assert!(password_problem("Jun", "short").is_some());
        assert!(password_problem("Jun", "password1").is_some());
        assert!(password_problem("Minamina", "MINAMINA").is_some());
        assert!(password_problem("Jun", "Password1").is_some());
        assert!(password_problem("Jun", "the hostas by the porch").is_none());
    }

    #[test]
    fn reset_codes_are_64_hex_and_hash_to_64_hex() {
        let c = new_reset_code();
        assert_eq!(c.len(), 64);
        assert!(c.bytes().all(|b| b.is_ascii_hexdigit()));
        let h = sha256_hex(&c);
        assert_eq!(h.len(), 64);
        assert_ne!(h, c);
    }

    #[test]
    fn hashes_verify_and_reject() {
        let salt = SaltString::generate(&mut OsRng);
        let h = Argon2::default().hash_password(b"the hostas by the porch", &salt).unwrap().to_string();
        let parsed = PasswordHash::new(&h).unwrap();
        assert!(Argon2::default().verify_password(b"the hostas by the porch", &parsed).is_ok());
        assert!(Argon2::default().verify_password(b"the hostas by the porch!", &parsed).is_err());
        assert!(!h.contains("hostas"));
    }

    #[test]
    fn client_ip_takes_the_last_forwarded_entry() {
        let mut h = axum::http::HeaderMap::new();
        assert_eq!(client_ip(&h), "unknown");
        h.insert("x-forwarded-for", "1.2.3.4, 5.6.7.8".parse().unwrap());
        assert_eq!(client_ip(&h), "5.6.7.8");
    }
}
