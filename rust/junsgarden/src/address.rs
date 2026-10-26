//! The garden's own address: where it lives, how labels and the phone app find it, and how
//! they keep finding it after the address changes.
//!
//! HOW TO READ THIS FILE (for Jun). The address is ONE setting, `JUNSGARDEN_PUBLIC_URL`
//! (the wizard writes it): a domain, and optionally a folder after it, for example
//! `https://example.com` or `https://example.com/garden`. Everything below is built from it:
//!
//! * **Sub-folder.** When the garden lives in a folder (`/garden`), `under_prefix` adds that
//!   folder to the site's own links in every page, script and stylesheet, and to redirects and
//!   the login cookie. With no folder it does nothing at all.
//! * **Labels that never go stale.** A QR code on a printed label holds a SHORT path, `/q/12`
//!   for plant 12 or `/b/3` for bed 3, never a page address. The garden answers those with
//!   the current page. A bed's QR holds the bed's number, so renaming the bed keeps it working.
//!   Labels printed before (they hold `catalog.php?plant=12` or `?bed=Name`) keep working too,
//!   as long as the old domain forwards to the new one (the wizard sets that up).
//! * **The phone app.** `/app/config.json` tells the app where the garden is now. It is
//!   signed with the garden's key (`JUNSGARDEN_SIGNING_KEY`, made by the wizard); the app got
//!   the matching public key when Jun scanned the setup QR, and it only moves to a new
//!   address that carries a valid signature. So if an old domain ever lapses and a stranger
//!   takes it, the app will not follow them.
//! * `/app/setup` is the page the wizard shows: one QR to download the app, one to set it up.

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    extract::{Path, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Extension, Json, Router,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};

use crate::layout::{self, Page};
use crate::php::{he, urlencode};
use crate::session::Visitor;
use crate::AppState;

/// The garden's address and, when the wizard made one, its signing key.
pub struct SiteAddress {
    /// The full address without a trailing slash: `https://example.com/garden`.
    pub url: String,
    /// The folder part (`/garden`), or "" when the garden is at the top of its domain.
    pub prefix: String,
    key: Option<SigningKey>,
}

impl SiteAddress {
    /// From the two settings. A missing or malformed key simply means "not signed".
    pub fn new(public_url: &str, signing_key_hex: &str) -> Self {
        let url = public_url.trim().trim_end_matches('/').to_string();
        let prefix = url
            .split_once("://")
            .and_then(|(_, rest)| rest.find('/').map(|i| rest[i..].to_string()))
            .unwrap_or_default();
        let key = hex32(signing_key_hex.trim()).map(|seed| SigningKey::from_bytes(&seed));
        SiteAddress { url, prefix, key }
    }

    /// The full address of a path on this garden: `link("/q/12")`.
    pub fn link(&self, path: &str) -> String {
        format!("{}{path}", self.url)
    }

    /// The public half of the signing key, as 64 hex characters (what the setup QR carries).
    pub fn public_key_hex(&self) -> Option<String> {
        self.key.as_ref().map(|k| hex(k.verifying_key().as_bytes()))
    }

    /// What `/app/config.json` says, signed when there is a key. The signed text is
    /// `junsgarden-config-v1\n<base>\n<issued>`; the app checks exactly that.
    pub fn config(&self, issued: i64) -> Value {
        let sig = self.key.as_ref().map(|k| {
            hex(&k
                .sign(config_message(&self.url, issued).as_bytes())
                .to_bytes())
        });
        json!({
            "v": 1,
            "base": self.url,
            "api": self.link("/app_api.php"),
            "version": self.link("/apps/version.json"),
            "apk": self.link("/apps/junsgarden.apk"),
            "issued": issued,
            "key": self.public_key_hex(),
            "sig": sig,
        })
    }

    /// The link the setup QR holds; the app opens it and saves the address and key.
    pub fn setup_link(&self) -> String {
        let mut s = format!("junsgarden://setup?base={}", urlencode(&self.url));
        if let Some(k) = self.public_key_hex() {
            s.push_str(&format!("&key={k}"));
        }
        s
    }
}

/// The exact text the config signature covers.
pub fn config_message(base: &str, issued: i64) -> String {
    format!("junsgarden-config-v1\n{base}\n{issued}")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 64 hex characters -> 32 bytes (a key). Also used by the page builder's signed sections list.
pub(crate) fn hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
    }
    Some(out)
}

// ---------------------------------------------------------------- a garden in a sub-folder

/// The site's own top-level names. Only links that start with one of these get the folder
/// added, so links to other websites and to `//cdn...` are never touched.
const OWN_PATHS: [&str; 32] = [
    "pagebuilder",
    "page.php",
    "images/",
    "js/",
    "sections/",
    "apps/",
    "app/",
    "q/",
    "b/",
    "qr/",
    "streamlit",
    "help",
    "app_api.php",
    "comments.php",
    "catalog.php",
    "index.php",
    "data.php",
    "widget_data.php",
    "plant_actions.php",
    "plant_listing.php",
    "gallery.php",
    "blog.php",
    "login.php",
    "style.css",
    "user-notes.css",
    "healthz",
    "sso-login",
    "privacy.html",
    "loader-preview.html",
    "favicon.svg",
    "ic_launcher_playstore.png",
    "labels",
];

/// Add `prefix` to every root-relative link of the site's own in `text`: a quote, bracket or
/// backtick, then `/`, then one of `OWN_PATHS` (or just `/` on its own, the home page).
pub fn add_prefix(text: &str, prefix: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 64);
    let mut last = 0;
    let mut i = 0;
    while i + 1 < b.len() {
        let opener = b[i];
        if matches!(opener, b'"' | b'\'' | b'(' | b'`') && b[i + 1] == b'/' {
            let rest = &text[i + 2..];
            let home = rest
                .as_bytes()
                .first()
                .is_some_and(|c| *c == opener || (opener == b'(' && *c == b')'));
            let own = OWN_PATHS.iter().any(|p| {
                rest.starts_with(p)
                    && (p.ends_with('/')
                        || !rest[p.len()..].starts_with(|c: char| {
                            c.is_ascii_alphanumeric() || c == '_' || c == '-'
                        }))
            });
            if home || own {
                out.push_str(&text[last..=i]);
                out.push_str(prefix);
                last = i + 1;
            }
        }
        i += 1;
    }
    out.push_str(&text[last..]);
    out
}

/// Middleware: when the garden lives in a folder, add it to links in pages, scripts and
/// stylesheets, to redirects, and to the login cookie's path. No folder: untouched.
pub async fn under_prefix(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let prefix = state.address.prefix.clone();
    let res = next.run(req).await;
    if prefix.is_empty() {
        return res;
    }
    let (mut parts, body) = res.into_parts();
    if let Some(loc) = parts
        .headers
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
    {
        if loc.starts_with('/') && !loc.starts_with("//") && !loc.starts_with(&format!("{prefix}/"))
        {
            if let Ok(v) = HeaderValue::from_str(&format!("{prefix}{loc}")) {
                parts.headers.insert(header::LOCATION, v);
            }
        }
    }
    let cookies: Vec<HeaderValue> = parts
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|c| {
            HeaderValue::from_str(&c.replace("; path=/;", &format!("; path={prefix}/;"))).ok()
        })
        .collect();
    if !cookies.is_empty() {
        parts.headers.remove(header::SET_COOKIE);
        for c in cookies {
            parts.headers.append(header::SET_COOKIE, c);
        }
    }
    let ct = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let text_like =
        ct.starts_with("text/html") || ct.contains("javascript") || ct.starts_with("text/css");
    if !text_like {
        return Response::from_parts(parts, body);
    }
    // Pages, scripts and stylesheets are small; anything over 16 MB passes unchanged.
    let Ok(bytes) = to_bytes(body, 16 * 1024 * 1024).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "page too large").into_response();
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Response::from_parts(parts, Body::from(bytes));
    };
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(add_prefix(text, &prefix)))
}

// ---------------------------------------------------------------- QR codes and short paths

/// A QR code for `text` as a PNG: black on white, 8 pixels per square, the standard
/// 4-square quiet border. None only if the text is too long for any QR code.
pub fn qr_png(text: &str) -> Option<Vec<u8>> {
    let code = qrcode::QrCode::new(text.as_bytes()).ok()?;
    let (width, colors) = (code.width(), code.to_colors());
    const SCALE: u32 = 8;
    const QUIET: u32 = 4;
    let side = (width as u32 + 2 * QUIET) * SCALE;
    let img = image::GrayImage::from_fn(side, side, |x, y| {
        let (mx, my) = (x / SCALE, y / SCALE);
        let dark = mx >= QUIET && my >= QUIET && {
            let (cx, cy) = ((mx - QUIET) as usize, (my - QUIET) as usize);
            cx < width && cy < width && colors[cy * width + cx] == qrcode::Color::Dark
        };
        image::Luma([if dark { 0 } else { 255 }])
    });
    let mut png = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    Some(png)
}

fn png_response(png: Option<Vec<u8>>) -> Response {
    match png {
        Some(b) => (
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "public, max-age=86400"),
            ],
            b,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `/q/{id}`: a plant's label. Goes to the catalogue with that plant open.
async fn plant_short(Path(id): Path<String>) -> Response {
    match id.parse::<u32>() {
        Ok(n) => Redirect::to(&format!("/catalog.php?plant={n}")).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `/b/{id}`: a bed's label. Looks the bed's CURRENT name up by its number, so a renamed bed
/// keeps its label.
async fn bed_short(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let Ok(n) = id.parse::<u32>() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match sqlx::query_scalar::<_, String>("SELECT name FROM garden_bed WHERE id = ?")
        .bind(n)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(name)) => {
            Redirect::to(&format!("/catalog.php?bed={}", urlencode(&name))).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("bed label {n}: {e}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

async fn plant_qr(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let Ok(n) = id.trim_end_matches(".png").parse::<u32>() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    png_response(qr_png(&state.address.link(&format!("/q/{n}"))))
}

async fn bed_qr(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let Ok(n) = id.trim_end_matches(".png").parse::<u32>() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    png_response(qr_png(&state.address.link(&format!("/b/{n}"))))
}

async fn setup_qr(State(state): State<Arc<AppState>>) -> Response {
    png_response(qr_png(&state.address.setup_link()))
}

async fn apk_qr(State(state): State<Arc<AppState>>) -> Response {
    png_response(qr_png(&state.address.link("/apps/junsgarden.apk")))
}

/// `/app/config.json`: where the garden is now, signed (see the top of this file).
async fn config(State(state): State<Arc<AppState>>) -> Response {
    let mut res = Json(state.address.config(common::now_local().unix_timestamp())).into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// `/app/setup`: the two QR codes the wizard shows, in English and Korean.
async fn setup_page(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
) -> Html<String> {
    let body = format!(
        "<div class=\"page-content no-hero\" style=\"max-width:760px;margin:0 auto;padding:16px\">\
         <h1>📱 The garden app / 정원 앱</h1>\
         <p><b>1.</b> Scan this with the phone's camera to download the app, then install it.<br>\
         <b>1.</b> 휴대폰 카메라로 찍어서 앱을 내려받고 설치하세요.</p>\
         <p><img src=\"/qr/app-apk.png\" alt=\"QR: download the app\" width=\"240\" height=\"240\"></p>\
         <p><b>2.</b> Open the app once, then scan this one: it tells the app where your garden is.<br>\
         <b>2.</b> 앱을 한 번 연 다음 이것을 찍으세요. 앱이 내 정원의 주소를 알게 돼요.</p>\
         <p><img src=\"/qr/app-setup.png\" alt=\"QR: set up the app\" width=\"240\" height=\"240\"></p>\
         <p>Garden address / 정원 주소: <code>{}</code></p>\
         <p><b>3.</b> In the app, log in with your garden name and password.<br>\
         <b>3.</b> 앱에서 정원 이름과 비밀번호로 로그인하세요.</p></div>",
        he(&state.address.url)
    );
    let mut page = Page::new(body);
    page.title = "Garden app / 정원 앱".to_string();
    page.show_hero = false;
    Html(layout::render(&state, page, &visitor).await)
}

/// `/labels`: a printable sheet of bed and plant labels with their QR codes (logged in only).
/// Every bed name the plants use gets a bed number first, if it has none yet, so every label
/// can hold a number instead of a name.
async fn labels(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
) -> Response {
    if !visitor.is_admin() {
        return Redirect::to("/login.php?back=/labels").into_response();
    }
    let _ = sqlx::query(
        "INSERT IGNORE INTO garden_bed (name, shape)
         SELECT DISTINCT TRIM(flower_bed), '' FROM data_source
         WHERE TRIM(COALESCE(flower_bed, '')) <> '' AND died = 0",
    )
    .execute(&state.db)
    .await;
    let beds = sqlx::query_as::<_, (i64, String)>("SELECT id, name FROM garden_bed ORDER BY name")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    let plants = sqlx::query_as::<_, (i64, Option<String>, Option<String>)>(
        "SELECT id, plant_name, flower_bed FROM data_source WHERE died = 0 ORDER BY flower_bed, plant_name",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let card = |qr: String, title: &str, sub: &str| {
        format!(
            "<div class=\"lbl\"><img src=\"{qr}\" alt=\"\" width=\"120\" height=\"120\"><b>{}</b><small>{}</small></div>",
            he(title),
            he(sub)
        )
    };
    let mut body = String::from(
        "<style>.lbls{display:flex;flex-wrap:wrap;gap:10px}.lbl{width:150px;border:1px solid #999;border-radius:8px;\
         padding:8px;text-align:center;break-inside:avoid}.lbl b,.lbl small{display:block}\
         @media print{header,footer,.no-print{display:none!important}}</style>\
         <div class=\"page-content no-hero\" style=\"padding:16px\"><h1>🏷️ Labels / 이름표</h1>\
         <p class=\"no-print\">Print this page. The codes point to the garden's current address, and keep working after a \
         domain change or a bed rename.<br>이 페이지를 인쇄하세요. 주소가 바뀌거나 화단 이름을 바꿔도 코드는 계속 작동해요.</p>",
    );
    body.push_str(&address_panel(&state).await);
    body.push_str("<h2>Beds / 화단</h2><div class=\"lbls\">");
    for (id, name) in &beds {
        body.push_str(&card(format!("/qr/bed/{id}.png"), name, &format!("#{id}")));
    }
    body.push_str("</div><h2>Plants / 식물</h2><div class=\"lbls\">");
    for (id, name, bed) in &plants {
        body.push_str(&card(
            format!("/qr/plant/{id}.png"),
            name.as_deref().unwrap_or(""),
            bed.as_deref().unwrap_or(""),
        ));
    }
    body.push_str("</div></div>");
    let mut page = Page::new(body);
    page.title = "Labels / 이름표".to_string();
    page.show_hero = false;
    Html(layout::render(&state, page, &visitor).await).into_response()
}

// --- Changing the web address from the admin page --------------------------------------------
//
// The website cannot change the garden computer's front door (Caddy) or its settings file
// itself, and it should not be able to. So the admin page only FILES A REQUEST: a small file
// `address-request.json` in the data folder with the new web name and folder. On the garden
// computer a systemd path unit (set up by `garden.sh address-watch`, part of the install)
// notices the file within a second and runs `garden.sh address-request`, which checks the name
// again and does the same one-step change as `garden.sh webname` (settings, front door, old
// name forwarding, restart, labels). It writes the result to `address-status.json`, shown here.

/// The request file the admin page writes, and the status file `garden.sh` writes back.
const ADDRESS_REQUEST: &str = "address-request.json";
const ADDRESS_STATUS: &str = "address-status.json";

/// A web name someone may ask for: 4 to 253 of `a-z 0-9 . -`, at least one dot, every part
/// between dots 1 to 63 long and not starting or ending with `-`, the last part two or more
/// letters. (The same rule as garden.sh's valid_webname, which checks it again.)
pub fn valid_web_name(s: &str) -> bool {
    (4..=253).contains(&s.len())
        && s.rsplit('.')
            .next()
            .is_some_and(|tld| tld.len() >= 2 && tld.bytes().all(|b| b.is_ascii_lowercase()))
        && s.contains('.')
        && s.split('.').all(|p| {
            !p.is_empty()
                && p.len() <= 63
                && !p.starts_with('-')
                && !p.ends_with('-')
                && p.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

/// A folder after the web name: empty (the top) or `/name`, with `a-z 0-9 _ -`, at most 40.
pub fn valid_folder(s: &str) -> bool {
    let Some(rest) = s.strip_prefix('/') else {
        return s.is_empty();
    };
    (1..=40).contains(&rest.len())
        && rest
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && rest
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// The address box on the labels page: the address now, the last change, a waiting request,
/// and the form to ask for a new one. Bilingual, like the rest of the page; never printed.
async fn address_panel(state: &Arc<AppState>) -> String {
    let data = state.data.clone();
    let (request, status) = crate::guard::off_thread(move || {
        let read = |name: &str| {
            crate::store::file_in(&data, name)
                .ok()
                .and_then(|p| crate::store::read_json(&p).ok())
                .filter(|m| !m.is_empty())
        };
        (read(ADDRESS_REQUEST), read(ADDRESS_STATUS))
    })
    .await;
    let s = |m: &serde_json::Map<String, Value>, k: &str| {
        he(m.get(k).and_then(Value::as_str).unwrap_or(""))
    };
    let mut out = format!(
        "<div class=\"no-print\" style=\"border:1px solid #9a9;border-radius:10px;padding:12px;margin:12px 0\">\
         <h2 style=\"margin-top:0\">🌐 Web address / 웹 주소</h2>\
         <p>Now / 지금: <b>{}</b></p>",
        he(&state.address.url)
    );
    if let Some(st) = &status {
        let (at, old, new) = (s(st, "at"), s(st, "old"), s(st, "new"));
        out.push_str(&if st.get("state").and_then(Value::as_str) == Some("done") {
            format!(
                "<p>✅ Changed on {at} from <b>{old}</b> to <b>{new}</b>. Every code on this page uses the \
                 new address; labels printed before keep working.<br>{at}에 <b>{old}</b>에서 <b>{new}</b>(으)로 \
                 바뀌었어요. 이 페이지의 모든 코드는 새 주소를 써요. 전에 인쇄한 이름표도 계속 작동해요.</p>"
            )
        } else {
            let why = s(st, "message");
            format!(
                "<p>⚠️ The last change did not finish ({at}): {why}<br>지난번 변경이 끝나지 않았어요({at}): {why}</p>"
            )
        });
    }
    if let Some(rq) = &request {
        let to = format!("{}{}", s(rq, "name"), s(rq, "folder"));
        out.push_str(&format!(
            "<p>⏳ Waiting for the garden computer to switch to <b>{to}</b> (usually under a minute; \
             reload this page).<br>정원 컴퓨터가 <b>{to}</b>(으)로 바꾸는 중이에요(보통 1분 안. 이 페이지를 \
             새로 고치세요).</p>"
        ));
    }
    out.push_str(
        "<form method=\"post\" action=\"/labels/address\" style=\"display:flex;flex-wrap:wrap;gap:8px;align-items:end\">\
         <label>New web name / 새 웹 이름<br><input name=\"name\" required placeholder=\"name.zapto.org\" \
         pattern=\"[a-z0-9.-]+\" autocapitalize=\"off\" spellcheck=\"false\"></label>\
         <label>Folder (optional) / 폴더(선택)<br><input name=\"folder\" placeholder=\"/garden\" \
         pattern=\"(/[a-z0-9][a-z0-9_-]*)?\"></label>\
         <button type=\"submit\">Change the address / 주소 바꾸기</button></form>\
         <p><small>In one step: the settings, the front door, the codes on this page and the phone app's \
         address all change; the old name keeps forwarding. The new name must be in your No-IP account \
         first (guide 04).<br>한 번에: 설정, 현관문, 이 페이지의 코드, 휴대폰 앱의 주소가 모두 바뀌고, 예전 \
         이름은 계속 새 이름으로 넘겨줘요. 새 이름은 먼저 내 No-IP 계정에 있어야 해요(안내서 04).</small></p></div>",
    );
    out
}

/// `POST /labels/address`: the admin asks for a new web address. Checked here, written to
/// `address-request.json` for `garden.sh` to carry out (and checked there again).
async fn ask_new_address(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    if !visitor.is_admin() {
        return Redirect::to("/login.php?back=/labels").into_response();
    }
    let f = crate::formdata::parse(&headers, body).await;
    let name = f.str("name").trim().to_ascii_lowercase();
    let folder = f
        .str("folder")
        .trim()
        .trim_end_matches('/')
        .to_ascii_lowercase();
    let back = "<p><a href=\"/labels\">Back / 돌아가기</a></p>";
    if !valid_web_name(&name) || !valid_folder(&folder) {
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                "<p>That does not look like a web name (like name.zapto.org) and an optional folder \
                 (like /garden). / 웹 이름(예: name.zapto.org)과 폴더(예: /garden)가 맞지 않아요.</p>{back}"
            )),
        )
            .into_response();
    }
    let request = json!({
        "name": name,
        "folder": folder,
        "by": visitor.db_name(),
        "at": common::now_local().unix_timestamp(),
    });
    let data = state.data.clone();
    // `None` = saved; `Some(why)` = not saved.
    let failed = crate::guard::off_thread(move || {
        crate::store::file_in(&data, ADDRESS_REQUEST)
            .and_then(|p| {
                crate::store::write_atomic(
                    &p,
                    &serde_json::to_vec_pretty(&request).unwrap_or_default(),
                )
            })
            .err()
            .map(|e| e.to_string())
    })
    .await;
    if let Some(e) = failed {
        tracing::error!("address request not saved: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "<p>The request could not be saved. / 요청을 저장하지 못했어요.</p>{back}"
            )),
        )
            .into_response();
    }
    tracing::info!("web address change requested: {name}{folder}");
    Redirect::to("/labels").into_response()
}

/// The routes this file answers.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/q/{id}", get(plant_short))
        .route("/b/{id}", get(bed_short))
        .route("/qr/plant/{id}", get(plant_qr))
        .route("/qr/bed/{id}", get(bed_qr))
        .route("/qr/app-setup.png", get(setup_qr))
        .route("/qr/app-apk.png", get(apk_qr))
        .route("/app/config.json", get(config))
        .route("/app/setup", get(setup_page))
        .route("/labels", get(labels))
        .route("/labels/address", axum::routing::post(ask_new_address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Verifier};

    const SEED: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";

    #[test]
    fn address_with_and_without_a_folder() {
        let top = SiteAddress::new("https://example.com/", "");
        assert_eq!(
            (top.url.as_str(), top.prefix.as_str()),
            ("https://example.com", "")
        );
        assert_eq!(top.link("/q/12"), "https://example.com/q/12");
        let sub = SiteAddress::new("https://example.com/garden/", "");
        assert_eq!(
            (sub.url.as_str(), sub.prefix.as_str()),
            ("https://example.com/garden", "/garden")
        );
        assert_eq!(sub.link("/b/3"), "https://example.com/garden/b/3");
        let port = SiteAddress::new("http://192.0.2.7:8030", "");
        assert_eq!(port.prefix, "");
    }

    #[test]
    fn only_the_sites_own_links_get_the_folder() {
        let html = r#"<a href="/">Home</a><a href="/help">h</a><img src="/images/a.jpg"><link href="/style.css?v=1">
<script>fetch('/comments.php?x=1');u=`/q/${id}`</script><a href="//cdn.example/x.js"></a><a href="/helpful">no</a>
<a href="https://other.example/index.php">ext</a><div style="background:url(/images/b.png)"></div><a href="catalog.php">rel</a>"#;
        let out = add_prefix(html, "/garden");
        for want in [
            r#"href="/garden/""#,
            r#"href="/garden/help""#,
            r#"src="/garden/images/a.jpg""#,
            r#"href="/garden/style.css?v=1""#,
            "fetch('/garden/comments.php",
            "`/garden/q/",
            "url(/garden/images/b.png)",
        ] {
            assert!(out.contains(want), "{want} in {out}");
        }
        for keep in [
            r#"href="//cdn.example/x.js""#,
            r#"href="/helpful""#,
            r#"href="https://other.example/index.php""#,
            r#"href="catalog.php""#,
        ] {
            assert!(out.contains(keep), "{keep} unchanged");
        }
        assert_eq!(add_prefix("nothing here", "/garden"), "nothing here");
    }

    #[test]
    fn config_is_signed_and_the_signature_checks() {
        let a = SiteAddress::new("https://example.com/garden", SEED);
        let c = a.config(1790000000);
        assert_eq!(c["base"], "https://example.com/garden");
        // Pinned: the Android app's GardenAddressTest checks this exact signature, so the two
        // sides can never drift apart on the signed text.
        assert_eq!(
            c["sig"],
            "9932dc7d968058367785afbcff754936a057e9bbd672df8ae39afc7bb1614cac2417e53999c1ce944247ee699ac17be6b85282fc856456b3151ad13f003ba40b"
        );
        assert_eq!(c["api"], "https://example.com/garden/app_api.php");
        let key =
            ed25519_dalek::VerifyingKey::from_bytes(&hex32(c["key"].as_str().unwrap()).unwrap())
                .unwrap();
        let sig_bytes: Vec<u8> = (0..64)
            .map(|i| u8::from_str_radix(&c["sig"].as_str().unwrap()[i * 2..i * 2 + 2], 16).unwrap())
            .collect();
        let sig = Signature::from_slice(&sig_bytes).unwrap();
        assert!(key
            .verify(
                config_message("https://example.com/garden", 1790000000).as_bytes(),
                &sig
            )
            .is_ok());
        // A different address does not verify with that signature.
        assert!(key
            .verify(
                config_message("https://evil.example", 1790000000).as_bytes(),
                &sig
            )
            .is_err());
        // No key: unsigned, and the setup link carries no key.
        let plain = SiteAddress::new("https://example.com", "");
        assert!(plain.config(1)["sig"].is_null());
        assert_eq!(
            plain.setup_link(),
            "junsgarden://setup?base=https%3A%2F%2Fexample.com"
        );
        assert!(a
            .setup_link()
            .ends_with(&format!("&key={}", a.public_key_hex().unwrap())));
    }

    #[test]
    fn qr_codes_are_png_images() {
        let png = qr_png("https://example.com/garden/q/12").unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!(img.width(), img.height());
    }

    #[test]
    fn only_plain_web_names_and_folders_can_be_asked_for() {
        for ok in [
            "name.zapto.org",
            "my-garden.example.com",
            "a1.bc",
            "xn--bcher-kva.example",
        ] {
            assert!(valid_web_name(ok), "{ok}");
        }
        for bad in [
            "",
            "nodot",
            "Name.zapto.org",
            "a1.b2",
            "a.c",
            "-a.org",
            "a-.org",
            "a..org",
            ".a.org",
            "a.org.",
            "a.org/x",
            "a.org;rm -rf",
            "a.org\n",
            "a b.org",
            "https://a.org",
            &"a".repeat(64),
        ] {
            assert!(!valid_web_name(bad), "{bad:?}");
        }
        for ok in ["", "/garden", "/g", "/my_garden-2"] {
            assert!(valid_folder(ok), "{ok}");
        }
        for bad in [
            "garden",
            "/",
            "/Garden",
            "/_x",
            "/a/b",
            "/../x",
            "/a b",
            "/a;b",
            &format!("/{}", "a".repeat(41)),
        ] {
            assert!(!valid_folder(bad), "{bad:?}");
        }
    }

    #[test]
    fn labels_stay_inside_the_folder() {
        assert_eq!(
            add_prefix("<a href=\"/labels\">", "/garden"),
            "<a href=\"/garden/labels\">"
        );
        assert_eq!(
            add_prefix("action=\"/labels/address\"", "/garden"),
            "action=\"/garden/labels/address\""
        );
    }
}
