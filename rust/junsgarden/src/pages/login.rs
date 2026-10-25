//! `/login.php` — the login page: log in, "forgot my password", and the
//! page the emailed link opens to choose a password.
//!
//! HOW TO READ THIS FILE (for Jun). `login_get` decides which of three
//! boxes to show from the address (`?forgot=1`, `?reset=<code>&u=<name>`,
//! or the plain login). `login_post` handles the three buttons:
//! `admin_login` (check the password, start a session, go back to the page
//! you came from), `forgot` (email a link), `set_password` (use the link).
//! The password checking itself is in `accounts.rs`.
//!
//! The small login boxes on the home and catalogue pages post here too, so
//! there is one place where logging in happens.
//! URLs: `GET /login.php` shows the page; `POST /login.php` handles the buttons.
//! The Logout button also posts here (field `logout`).

// `use` lines bring names into scope. `Arc` is a shared pointer to one value,
// here the app's settings and database pool.
use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Extension,
};

use crate::accounts::{self, LoginError};
use crate::formdata;
use crate::layout::{self, Page};
use crate::php::urlencode;
use crate::session::{self, Visitor};
use crate::AppState;

/// The data the login page's HTML template needs. askama (the template library)
/// checks at compile time that `templates/login.html` only uses these fields.
/// `mode` is "login", "forgot" or "reset" and chooses which box the page shows.
/// `error` and `notice` are the red and green messages. `back` is the page to
/// return to after login. `reset_user` and `reset_code` carry the emailed link's
/// values into the "choose a password" box. `String` is owned, changeable text.
#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    mode: String,
    error: String,
    notice: String,
    back: String,
    reset_user: String,
    reset_code: String,
}

// (How `safe_back` decides, in order: starts with one `/`, not `//`, no
// backslash, no `://`, shorter than 300 characters, and not the login page.)
/// Only go back to a page on this site: a path starting with one `/`.
/// Anything else (another website, `//evil.example`) goes to the home page,
/// so the login cannot be used to bounce people somewhere else.
// Takes: a `&str` (borrowed text) from the URL or form. Gives back an owned
// `String` that is safe to redirect to. A web address would be abused as an
// "open redirect" if we sent people wherever the link said.
fn safe_back(s: &str) -> String {
    let ok = s.starts_with('/') && !s.starts_with("//") && !s.contains('\\')
        && !s.contains("://") && s.len() < 300 && !s.starts_with("/login.php");
    if ok { s.to_string() } else { "/".to_string() }
}

/// Make an HTTP redirect (status 302 "Found") to `to`. Takes the target path.
/// Gives back a `Response`. Used after every successful or failed button press.
fn redirect(to: &str) -> Response {
    (StatusCode::FOUND, [(header::LOCATION, to.to_string())]).into_response()
}

/// Render the login template inside the shared page frame.
/// Takes the app state, who is asking and the filled-in template.
/// `unwrap_or_default()` means: if rendering fails, use an empty page body
/// instead of crashing (a template error would be a bug found at compile time).
async fn page(state: &AppState, visitor: &Visitor, t: LoginTemplate) -> Response {
    let body = t.render().unwrap_or_default();
    let mut p = Page::new(body);
    p.title = "Login — Jun's and Kona's Garden".to_string();
    p.show_hero = false;
    Html(layout::render(state, p, visitor).await).into_response()
}

/// Handler for `GET /login.php`. Chooses which box to show from the address.
/// Takes: state, visitor, and the query string as a map of text to text.
/// Gives back the finished HTML page.
pub async fn login_get(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    // 1. A small helper (a closure, written `|k| ...`): look up a query value by
    // name, or give "" when it is missing. `Option` means "maybe nothing".
    let get = |k: &str| q.get(k).map(String::as_str).unwrap_or("");
    // 2. Start with the plain login box.
    let mut t = LoginTemplate {
        mode: "login".into(),
        error: String::new(),
        notice: String::new(),
        back: safe_back(get("back")),
        reset_user: String::new(),
        reset_code: String::new(),
    };

    // 3. `?reset=<code>&u=<name>` is the link from the email. Show the "choose a
    // password" box only if the code is valid (unused, under 30 minutes old).
    // Otherwise fall back to the "forgot" box with an explanation.
    if !get("reset").is_empty() {
        let name = accounts::find(&state, get("u")).map(|a| a.name.clone()).unwrap_or_default();
        if !name.is_empty() && accounts::reset_code_valid(&state, &name, get("reset")).await {
            t.mode = "reset".into();
            t.reset_user = name;
            t.reset_code = get("reset").to_string();
        } else {
            t.mode = "forgot".into();
            t.error = "That link has expired or was already used. Ask for a new one below.".into();
        }
    // 4. `?forgot=1` shows the "ask for a link" box.
    } else if !get("forgot").is_empty() {
        t.mode = "forgot".into();
    }

    // 5. `?msg=` is a one-word status sent by `login_post` after a redirect.
    // `match` compares the value to each text in turn; `_ => {}` means "none of
    // the above, do nothing".
    match get("msg") {
        "invalid" => t.error = "Invalid username or password.".into(),
        "toomany" => t.error = "Too many login attempts. Please wait 15 minutes.".into(),
        "sent" => {
            t.notice = "If that is a garden account, an email with a link is on its way. \
                        It works for 30 minutes. Check the spam folder if it does not arrive."
                .into()
        }
        "set" => t.notice = "Your password is saved. Log in with it now.".into(),
        "out" => t.notice = "You are logged out.".into(),
        _ => {}
    }
    // 6. If already signed in and nothing else to say, tell the person who they are.
    if visitor.is_admin() && t.mode == "login" && t.notice.is_empty() {
        t.notice = format!("You are logged in as {}.", visitor.username);
    }
    // 7. Render the page. (No `;` at the end: this value is the function's result.)
    page(&state, &visitor, t).await
}

/// Handler for `POST /login.php`. Handles four buttons, tested in this order:
/// `logout`, `admin_login`, `forgot` and `set_password`. Anything else goes
/// back to the login page.
/// Takes: state, visitor, headers and the raw body. Gives back a redirect (or
/// the "choose a password" page again if the new password is rejected).
pub async fn login_post(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Read the form fields; find the caller's IP address (for the lockout count).
    let f = formdata::parse(&headers, body).await;
    let ip = accounts::client_ip(&headers);

    // 1. Logout: delete the session row and go to the login page with "logged out".
    if f.has("logout") {
        session::destroy(&state, &visitor.sid).await;
        return redirect("/login.php?msg=out");
    }

    // 2. Log in. `accounts::check_password` returns `Ok(name)` or an `Err`.
    // `.await` waits for the database. `return match ...` returns the result of
    // the match directly.
    if f.has("admin_login") {
        let back = safe_back(f.str("back"));
        return match accounts::check_password(&state, f.str("admin_user"), f.str("admin_password"), &ip).await {
            Ok(name) => {
                // Security: the old session id is destroyed and a new one issued, so an id an
                // attacker may have seen before login is useless afterwards.
                // A brand-new session id at login, and the old one is dropped.
                session::destroy(&state, &visitor.sid).await;
                let sid = session::sign_in(&state, &name).await;
                let mut res = redirect(&back);
                // Set the session cookie. `if let Some(c)` runs only when a cookie was built.
                if let Some(c) = session::session_cookie(&sid, session::is_https(&headers)) {
                    res.headers_mut().append(header::SET_COOKIE, c);
                }
                res
            }
            // Too many failures: back to the form, with the `back` page kept in the address.
            Err(LoginError::TooMany) => redirect(&format!("/login.php?msg=toomany&back={}", urlencode(&back))),
            Err(LoginError::Invalid) => redirect(&format!("/login.php?msg=invalid&back={}", urlencode(&back))),
        };
    }

    // 3. Forgot password: email a one-time link.
    if f.has("forgot") {
        accounts::start_reset(&state, f.str("who")).await;
        // The reply is identical whether or not the name exists, so nobody can use
        // this form to learn which accounts exist.
        // The same answer whether or not the name exists.
        return redirect("/login.php?forgot=1&msg=sent");
    }

    // 4. Set a new password from the emailed link.
    if f.has("set_password") {
        let name = f.str("u").to_string();
        let code = f.str("reset").to_string();
        // The two typed passwords must match; then try to save the new one. A message
        // starting "That link" means the code expired, so start over; any other message
        // is shown in the "choose a password" box again.
        let problem = if f.str("password") != f.str("password2") {
            "The two passwords are not the same."
        } else {
            match accounts::finish_reset(&state, &name, &code, f.str("password")).await {
                Ok(_) => return redirect("/login.php?msg=set"),
                Err(e) if e.starts_with("That link") => return redirect("/login.php?reset=expired&u=x"),
                Err(e) => e,
            }
        };
        let t = LoginTemplate {
            mode: "reset".into(),
            error: problem.to_string(),
            notice: String::new(),
            back: "/".into(),
            reset_user: accounts::find(&state, &name).map(|a| a.name.clone()).unwrap_or_default(),
            reset_code: code,
        };
        return page(&state, &visitor, t).await;
    }

    // 5. Nothing matched: go to the login page.
    redirect("/login.php")
}

// Tests run only with `cargo test`.
#[cfg(test)]
mod tests {
    use super::safe_back;

    // Checks that `safe_back` accepts only paths on this site and rejects foreign
    // addresses, backslash tricks and the login page itself.
    #[test]
    fn back_stays_on_this_site() {
        assert_eq!(safe_back("/catalog.php"), "/catalog.php");
        assert_eq!(safe_back("/"), "/");
        assert_eq!(safe_back("//evil.example/x"), "/");
        assert_eq!(safe_back("https://evil.example"), "/");
        assert_eq!(safe_back("/\\evil.example"), "/");
        assert_eq!(safe_back(""), "/");
        assert_eq!(safe_back("/login.php?x"), "/");
    }
}
