//! Tests for who may use the editor and what visitors see (no network, no database: every
//! path tested here is refused or answered before the database would be asked).

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::{Extension, Router};
use serde_json::{json, Value};
use tower::ServiceExt;

use super::layout::{Cell, Layout, Row};
use super::sections::{self, Ctx};
use crate::accounts::Account;
use crate::session::Visitor;
use crate::AppState;

fn state() -> Arc<AppState> {
    let acc = |n: &str| Account {
        name: n.into(),
        email: format!("{}@example.test", n.to_lowercase()),
    };
    // The first name on the list is the owner.
    crate::test_state(PathBuf::new(), vec![acc("Owner"), acc("Helper")])
}

fn person(name: &str) -> Visitor {
    Visitor {
        sid: format!("sid-{name}"),
        username: name.into(),
        ..Default::default()
    }
}

fn app(st: &Arc<AppState>, who: Visitor) -> Router {
    super::editor::routes()
        .layer(Extension(who))
        .with_state(st.clone())
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, String) {
    let res = app.clone().oneshot(req).await.unwrap();
    let s = res.status();
    let b = to_bytes(res.into_body(), 8 << 20).await.unwrap();
    (s, String::from_utf8_lossy(&b).into_owned())
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn post(uri: &str, ctype: &str, token: Option<&str>, body: Value) -> Request<Body> {
    let mut r = Request::post(uri).header(header::CONTENT_TYPE, ctype);
    if let Some(t) = token {
        r = r.header("x-pb-token", t);
    }
    r.body(Body::from(body.to_string())).unwrap()
}

#[tokio::test]
async fn the_owner_is_the_first_garden_user_unless_named() {
    let st = state();
    assert_eq!(st.pagebuilder.owner, "Owner");
    let users = st.users.clone();
    assert_eq!(super::Builder::new(&users, "helper", "").owner, "Helper");
    // A name that is not on the list makes nobody the owner.
    assert_eq!(super::Builder::new(&users, "Stranger", "").owner, "");
    assert_eq!(super::Builder::new(&[], "", "").owner, "");
    let b = super::Builder::new(&[], "", "");
    assert!(!b.is_owner(&person("")));
}

#[tokio::test]
async fn tokens_belong_to_one_session() {
    let st = state();
    let b = &st.pagebuilder;
    let t = b.token("abc");
    assert_eq!(t.len(), 64);
    assert!(b.token_ok("abc", &t));
    assert!(!b.token_ok("xyz", &t));
    assert!(!b.token_ok("abc", ""));
    assert!(!b.token_ok("abc", &t[..63]));
}

#[tokio::test]
async fn only_the_owner_on_the_website_gets_the_editor() {
    let st = state();
    let mut app_key = person("Owner");
    app_key.app_token = Some("k".into());
    let cases = [
        (person(""), StatusCode::FORBIDDEN),
        (person("Helper"), StatusCode::FORBIDDEN),
        (app_key, StatusCode::FORBIDDEN),
        (person("Owner"), StatusCode::OK),
    ];
    for (case, (who, want)) in cases.into_iter().enumerate() {
        let a = app(&st, who);
        let (s, body) = send(&a, get("/pagebuilder/api/updates")).await;
        assert_eq!(s, want, "case {case}");
        if s == StatusCode::OK {
            assert!(body.contains("not_set_up"));
        } else {
            assert!(body.contains("error_ko"));
        }
        // The registry and a layout are refused the same way (before any database read).
        if want == StatusCode::FORBIDDEN {
            assert_eq!(send(&a, get("/pagebuilder/api/registry")).await.0, want);
            assert_eq!(
                send(&a, get("/pagebuilder/api/layout?page=home")).await.0,
                want
            );
        }
    }
    // The editor page: visitors go to the login, other gardeners are refused.
    let (s, _) = send(&app(&st, person("")), get("/pagebuilder.php?page=blog")).await;
    assert_eq!(s, StatusCode::SEE_OTHER);
    let (s, _) = send(&app(&st, person("Helper")), get("/pagebuilder.php")).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, body) = send(
        &app(&st, person("Owner")),
        get("/pagebuilder.php?page=blog"),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.contains("data-page=\"blog\""));
    assert!(body.contains(&st.pagebuilder.token("sid-Owner")));
    // A made-up page name falls back to home.
    let (_, body) = send(
        &app(&st, person("Owner")),
        get("/pagebuilder.php?page=%3Cx%3E"),
    )
    .await;
    assert!(body.contains("data-page=\"home\""));
}

#[tokio::test]
async fn posts_need_the_owner_json_and_the_token() {
    let st = state();
    let layout = json!({"page":"home","base":0,"layout":{"schema":1,"page":"home","rows":[]}});
    let good = st.pagebuilder.token("sid-Owner");
    let helper_token = st.pagebuilder.token("sid-Helper");
    let json = "application/json";
    for path in [
        "/pagebuilder/api/save",
        "/pagebuilder/api/restore",
        "/pagebuilder/api/preview",
        "/pagebuilder/api/seen",
    ] {
        // Other gardeners, even with their own valid token.
        let (s, _) = send(
            &app(&st, person("Helper")),
            post(path, json, Some(&helper_token), layout.clone()),
        )
        .await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{path}");
        // The owner without a token, with another session's token, or as a plain form.
        let owner = app(&st, person("Owner"));
        let status = |r| {
            let owner = owner.clone();
            async move { send(&owner, r).await.0 }
        };
        let cases = [
            (
                post(path, json, None, layout.clone()),
                StatusCode::FORBIDDEN,
            ),
            (
                post(path, json, Some(&helper_token), layout.clone()),
                StatusCode::FORBIDDEN,
            ),
            (
                post(
                    path,
                    "application/x-www-form-urlencoded",
                    Some(&good),
                    layout.clone(),
                ),
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ),
            (
                post(path, "text/plain", Some(&good), layout.clone()),
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ),
        ];
        for (req, want) in cases {
            assert_eq!(status(req).await, want, "{path}");
        }
    }
    // With everything right, a bad layout is still refused before anything is stored.
    let owner = app(&st, person("Owner"));
    let save = |body: Value| post("/pagebuilder/api/save", json, Some(&good), body);
    let bad = json!({"page":"home","base":0,"layout":{"schema":1,"page":"home","rows":[{"cols":[{"section":"data.widgets"}]}]}});
    let (s, body) = send(&owner, save(bad)).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(body.contains("database") && body.contains("error_ko"));
    let away = json!({"page":"../x","base":0,"layout":{"schema":1,"page":"../x","rows":[]}});
    assert_eq!(send(&owner, save(away)).await.0, StatusCode::BAD_REQUEST);
    let extra = json!({"page":"home","base":0,"layout":{},"extra":1});
    assert_eq!(send(&owner, save(extra)).await.0, StatusCode::BAD_REQUEST);
    // Too big a body is refused.
    let huge = json!({"page":"home","base":0,"layout":{"schema":1,"page":"home","rows":[]},"x":"a".repeat(70_000)});
    let (s, _) = send(&owner, save(huge)).await;
    assert_eq!(s, StatusCode::PAYLOAD_TOO_LARGE);
}

fn one_per_row(ids: &[&str]) -> Layout {
    Layout {
        schema: 1,
        page: "p-test".into(),
        rows: ids
            .iter()
            .map(|id| Row {
                cols: vec![Cell {
                    section: id.to_string(),
                    settings: Default::default(),
                }],
            })
            .collect(),
    }
}

/// Admin sections check their own permission whenever they are drawn, on any page.
#[tokio::test]
async fn admin_sections_show_nothing_to_visitors_wherever_they_are_placed() {
    let st = state();
    let editor = Cell {
        section: "admin.post_editor".into(),
        settings: Default::default(),
    };
    let draw_cell = |who: Visitor, preview: bool| {
        let st = st.clone();
        let cell = editor.clone();
        async move {
            let ctx = Ctx {
                state: &st,
                visitor: &who,
                zip: "",
                preview,
            };
            sections::render(&ctx, &cell).await.html
        }
    };
    assert_eq!(draw_cell(person(""), false).await, "");
    let admin = draw_cell(person("Helper"), false).await;
    assert!(admin.contains("id=\"blog-modal\""));
    // In the editor's preview (drawn as a visitor): a placeholder, never the form.
    let ph = draw_cell(person(""), true).await;
    assert!(ph.contains("pb-locked") && !ph.contains("blog-modal"));

    // Whole pages: an owner page and the home page with the editor added.
    for page in ["p-test", "home"] {
        let mut l = one_per_row(&["home.tips", "admin.post_editor"]);
        l.page = page.into();
        let visitor_html = super::draw(&st, &person(""), page, &l, "", false).await;
        assert!(!visitor_html.contains("blog-modal"), "{page}");
        assert!(!visitor_html.contains("name=\"post_blog\""), "{page}");
        let tips = visitor_html.contains("garden-chronicles-container");
        assert!(tips, "{page}");
        let admin_html = super::draw(&st, &person("Helper"), page, &l, "", false).await;
        assert!(admin_html.contains("blog-modal"), "{page}");
        // The Edit button: only for the owner.
        assert!(!admin_html.contains("pb-edit-button"));
        let owner_html = super::draw(&st, &person("Owner"), page, &l, "", false).await;
        assert!(owner_html.contains("pb-edit-button"));
    }
}

#[tokio::test]
async fn side_by_side_rows_get_the_grid_and_its_stylesheet() {
    let st = state();
    let l = Layout {
        schema: 1,
        page: "p-test".into(),
        rows: vec![Row {
            cols: vec![
                Cell {
                    section: "home.tips".into(),
                    settings: Default::default(),
                },
                Cell {
                    section: "home.welcome".into(),
                    settings: Default::default(),
                },
            ],
        }],
    };
    let html = super::draw(&st, &person(""), "p-test", &l, "", false).await;
    assert!(html.contains("<div class=\"pb-row pb-cols-2\">"));
    assert!(html.contains("pagebuilder/pb.css"));
    // A page without such a row does not load it (the default pages stay unchanged).
    let one = one_per_row(&["home.tips"]);
    let plain = super::draw(&st, &person(""), "p-test", &one, "", false).await;
    assert!(!plain.contains("pb.css") && !plain.contains("pb-row"));
}

#[tokio::test]
async fn a_preview_is_drawn_as_a_visitor_sees_it() {
    let st = state();
    let good = st.pagebuilder.token("sid-Owner");
    let body = json!({"page":"p-test","layout":{"schema":1,"page":"p-test","rows":[
        {"cols":[{"section":"home.tips"}]},{"cols":[{"section":"admin.post_editor"}]}]}});
    let (s, out) = send(
        &app(&st, person("Owner")),
        post(
            "/pagebuilder/api/preview",
            "application/json",
            Some(&good),
            body,
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let html = serde_json::from_str::<Value>(&out).unwrap()["html"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(html.contains("garden-chronicles-container"));
    assert!(html.contains("pb-locked") && !html.contains("blog-modal"));
    // Drawn as a visitor: no Edit button, the login box instead of the owner's name.
    assert!(!html.contains("pb-edit-button"));
}
