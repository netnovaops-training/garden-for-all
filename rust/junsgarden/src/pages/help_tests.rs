//! Tests for the /help pages: requests go straight to the router (no network,
//! no database: the pool is never used by these pages).

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::{Extension, Router};
use tower::ServiceExt;

use super::routes;
use crate::guides::{self, Lang};
use crate::session::Visitor;
use crate::AppState;

/// A small guides folder: one guide in both languages, a picture, a registry.
fn fixture() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "jg-help-{}-{}",
        std::process::id(),
        rand::random::<u32>()
    )); // nosemgrep: rust.lang.security.temp-dir.temp-dir (test only: a per-process throwaway folder, or a path that is never created)
    std::fs::create_dir_all(dir.join("ko")).unwrap();
    std::fs::create_dir_all(dir.join("images")).unwrap();
    std::fs::write(
        dir.join("02-docker.md"),
        "# 02. Docker\n\n## Quick version\n- Docker runs the garden.\n\n<a id=\"help-docker-what\"></a>\n\n## What Docker is\n\n![a](images/a.png)\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("ko/02-docker.md"),
        "# 02. 도커\n\n## 짧은 버전\n- 도커가 정원을 실행해요.\n\n<a id=\"help-docker-what\"></a>\n\n## 도커란 무엇인가\n",
    )
    .unwrap();
    std::fs::write(dir.join("images/a.png"), b"\x89PNG").unwrap();
    std::fs::write(
        dir.join("help-ids.tsv"),
        "# notes\nid\tguide\tanchor_en\tanchor_ko\ttags_en\ttags_ko\tbrief_en\tbrief_ko\twizard\n\
         docker-what\t02-docker\twhat-docker-is\t도커란-무엇인가\t#docker #boxes\t#도커 #상자\tWhat Docker is.\t도커가 무엇인지.\t\n",
    )
    .unwrap();
    dir
}

fn state(guides: PathBuf) -> Arc<AppState> {
    Arc::new(AppState {
        // A pool that never connects: nothing here asks the database.
        db: sqlx::mysql::MySqlPoolOptions::new()
            .connect_lazy("mysql://test@127.0.0.1:9/test")
            .unwrap(),
        www: std::env::temp_dir().join("jg-help-no-www"), // nosemgrep: rust.lang.security.temp-dir.temp-dir (test only: a per-process throwaway folder, or a path that is never created)
        data: std::env::temp_dir().join("jg-help-no-data"), // nosemgrep: rust.lang.security.temp-dir.temp-dir (test only: a per-process throwaway folder, or a path that is never created)
        comments_lock: tokio::sync::Mutex::new(()),
        users: Vec::new(),
        uploaders: Vec::new(),
        public_url: "http://localhost".into(),
        ingest_url: "http://127.0.0.1:9".into(),
        widget_cache: std::sync::Mutex::new(None),
        http: reqwest::Client::new(),
        ollama_url: "http://127.0.0.1:9".into(),
        ollama_model: "none".into(),
        ai_cache_dir: std::env::temp_dir().join("jg-help-no-cache"), // nosemgrep: rust.lang.security.temp-dir.temp-dir (test only: a per-process throwaway folder, or a path that is never created)
        forecast_file: PathBuf::new(),
        weather_latest: PathBuf::new(),
        guides,
        frame: Default::default(),
        write_limit: crate::guard::RateLimit::new(5, std::time::Duration::from_secs(600)),
        gallery: Default::default(),
        ai_recompute: crate::guard::RateLimit::new(1, std::time::Duration::from_secs(600)),
        ai_per_visitor: crate::guard::RateLimit::new(20, std::time::Duration::from_secs(600)),
    })
}

fn app(dir: PathBuf, signed_in: bool) -> Router {
    let visitor = Visitor {
        username: if signed_in {
            "tester".into()
        } else {
            String::new()
        },
        ..Default::default()
    };
    routes().layer(Extension(visitor)).with_state(state(dir))
}

/// GET `uri`; gives the status, the Location header and the body.
async fn get(app: &Router, uri: &str) -> (StatusCode, String, String) {
    let res = app
        .clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let loc = res
        .headers()
        .get(header::LOCATION)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let body = to_bytes(res.into_body(), 4 << 20).await.unwrap();
    (status, loc, String::from_utf8_lossy(&body).into_owned())
}

#[tokio::test]
async fn visitors_are_sent_to_the_login() {
    let dir = fixture();
    let a = app(dir.clone(), false);
    let (s, loc, _) = get(&a, "/help?guide=02-docker").await;
    assert_eq!(s, StatusCode::SEE_OTHER);
    assert_eq!(loc, "/login.php?back=%2Fhelp%3Fguide%3D02-docker");
    assert_eq!(get(&a, "/help/02-docker").await.0, StatusCode::SEE_OTHER);
    assert_eq!(get(&a, "/help/images/a.png").await.0, StatusCode::FORBIDDEN);
    std::fs::remove_dir_all(dir).ok();
}

#[tokio::test]
async fn only_listed_guides_and_pictures_open() {
    let dir = fixture();
    let a = app(dir.clone(), true);
    for bad in [
        "/help?guide=../../etc/passwd",
        "/help?guide=..%2F02-docker",
        "/help?guide=02-docker.md",
        "/help?guide=ko%2F02-docker",
        "/help?guide=MY-WINS",
        "/help/..%2F..%2Fetc%2Fpasswd",
        "/help/help-ids.tsv",
        "/help/images/..%2Fhelp-ids.tsv",
        "/help/images/..%2F02-docker.md",
        "/help/images/.hidden.png",
        "/help/images/missing.png",
        "/help/id/..%2Fx",
        "/help/nope",
        "/help/Bad_ID",
    ] {
        let (s, loc, _) = get(&a, bad).await;
        assert_eq!(
            s,
            StatusCode::NOT_FOUND,
            "{bad} must be a 404 (went to {loc:?})"
        );
    }
    let res = a
        .clone()
        .oneshot(
            Request::get("/help/images/a.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()[header::CONTENT_TYPE], "image/png");
    std::fs::remove_dir_all(dir).ok();
}

#[tokio::test]
async fn index_guide_and_korean_pages() {
    let dir = fixture();
    let a = app(dir.clone(), true);
    let (s, _, body) = get(&a, "/help").await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        body.contains("href=\"/help?guide=02-docker\">Docker</a>"),
        "{body}"
    );
    assert!(
        body.contains("Docker runs the garden."),
        "the one-line description"
    );
    assert!(
        body.contains("href=\"/help/docker-what\">#docker</a>"),
        "the tags from help-ids.tsv"
    );
    assert!(body.contains("id=\"help-q\""), "the search box");

    let (s, _, body) = get(&a, "/help?lang=ko").await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        body.contains("href=\"/help?guide=02-docker&#38;lang=ko\">도커</a>"),
        "{body}"
    );
    assert!(body.contains("도커가 정원을 실행해요."));

    let (s, _, body) = get(&a, "/help?guide=02-docker&lang=ko").await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.contains("<h1>도커</h1>"));
    assert!(body.contains("<a id=\"help-docker-what-ko\"></a>"));
    assert!(
        body.contains("<h2 id=\"도커란-무엇인가\">도커란 무엇인가</h2>"),
        "{body}"
    );

    let (_, _, body) = get(&a, "/help?guide=02-docker").await;
    assert!(body.contains("<h2 id=\"what-docker-is\">What Docker is</h2>"));
    assert!(body.contains("src=\"/help/images/a.png\""));
    std::fs::remove_dir_all(dir).ok();
}

#[tokio::test]
async fn every_help_id_has_its_own_page_with_brief_full_reading_and_tags() {
    let dir = fixture();
    let a = app(dir.clone(), true);
    let (s, _, body) = get(&a, "/help/docker-what?lang=ko").await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.contains("도커가 무엇인지."), "the Korean brief");
    assert!(
        body.contains("href=\"/help?guide=02-docker&#38;lang=ko#help-docker-what-ko\""),
        "Read the full guide: {body}"
    );
    assert!(
        body.contains("<h2 id=\"도커란-무엇인가\">"),
        "the full reading is on the page"
    );
    assert!(
        body.contains("href=\"/help/docker-what?lang=ko\">#도커</a>"),
        "a #hot word opens its id"
    );
    assert!(
        body.contains("href=\"/style.css"),
        "the page finds its stylesheet from below /help/"
    );

    let (_, _, body) = get(&a, "/help/docker-what").await;
    assert!(body.contains("What Docker is."));
    assert!(body.contains("href=\"/help?guide=02-docker#help-docker-what\""));
    assert!(
        body.contains("<h2 id=\"what-docker-is\">") && body.contains("src=\"/help/images/a.png\""),
        "full reading with its picture"
    );
    assert!(body.contains("href=\"/help/docker-what\">#docker</a>"));

    // The index's #words open the id they were written for.
    let (_, _, idx) = get(&a, "/help?lang=ko").await;
    assert!(
        idx.contains("href=\"/help/docker-what?lang=ko\">#상자</a>"),
        "{idx}"
    );

    // Older forms lead to the same place.
    let (s, loc, _) = get(&a, "/help?id=docker-what&lang=ko").await;
    assert_eq!(
        (s, loc.as_str()),
        (StatusCode::SEE_OTHER, "/help/docker-what?lang=ko")
    );
    let (s, loc, _) = get(&a, "/help/id/docker-what").await;
    assert_eq!(
        (s, loc.as_str()),
        (StatusCode::SEE_OTHER, "/help/docker-what")
    );
    let (s, loc, _) = get(&a, "/help?id=docker-what&lang=ko&full=1").await;
    assert_eq!(
        (s, loc.as_str()),
        (
            StatusCode::SEE_OTHER,
            "/help?guide=02-docker&lang=ko#help-docker-what-ko"
        )
    );
    let (s, loc, _) = get(&a, "/help/02-docker").await;
    assert_eq!(
        (s, loc.as_str()),
        (StatusCode::SEE_OTHER, "/help?guide=02-docker")
    );
    std::fs::remove_dir_all(dir).ok();
}

/// The real guides: START-HERE next to the rust folder. Skipped (with a note)
/// when only rust/ is mounted into the test container.
fn real_guides() -> Option<PathBuf> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../START-HERE");
    if d.join("help-ids.tsv").is_file() {
        Some(d)
    } else {
        eprintln!("SKIPPED: START-HERE is not next to rust/ (mount the whole garden folder to run this test)");
        None
    }
}

#[test]
fn every_help_id_lands_on_its_anchor_in_both_languages() {
    let Some(dir) = real_guides() else { return };
    let reg = guides::load_registry(&dir);
    let lines = std::fs::read_to_string(dir.join("help-ids.tsv")).unwrap();
    let rows = lines
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .count()
        - 1;
    assert_eq!(reg.len(), rows, "every row of help-ids.tsv is read");
    assert!(!reg.is_empty());
    for e in &reg {
        for (lang, heading) in [(Lang::En, &e.anchor_en), (Lang::Ko, &e.anchor_ko)] {
            let file = guides::guide_file(&dir, &e.guide, lang)
                .unwrap_or_else(|| panic!("{}: no {lang:?} guide", e.id));
            let html = guides::render(&std::fs::read_to_string(file).unwrap(), lang).html;
            let anchor = format!("<a id=\"{}\"></a>", e.anchor(lang));
            let at = html
                .find(&anchor)
                .unwrap_or_else(|| panic!("{}: no {anchor} in {} ({lang:?})", e.id, e.guide));
            // The heading right after the help anchor has the registry's own anchor.
            let next_heading = html[at..].find("<h").map(|i| &html[at + i..]).unwrap_or("");
            assert!(
                next_heading.contains(&format!("id=\"{heading}\""))
                    && next_heading.find(&format!("id=\"{heading}\"")) < next_heading.find('>'),
                "{}: the heading after {anchor} should have id=\"{heading}\" ({lang:?})",
                e.id
            );
        }
    }
}

#[test]
fn no_id_is_a_guide_name_and_every_id_has_a_section_with_text() {
    let Some(dir) = real_guides() else { return };
    for e in guides::load_registry(&dir) {
        assert!(!guides::is_guide(&e.id), "{} is also a guide name", e.id);
        for lang in [Lang::En, Lang::Ko] {
            let file = guides::guide_file(&dir, &e.guide, lang).unwrap();
            let html = guides::render(&std::fs::read_to_string(file).unwrap(), lang).html;
            let sec = guides::section_html(&html, &e.anchor(lang));
            assert!(
                sec.len() > 200 && sec.contains("<h"),
                "{} ({lang:?}): the section is empty",
                e.id
            );
        }
    }
}

#[test]
fn every_guide_in_the_reading_order_exists_and_renders() {
    let Some(dir) = real_guides() else { return };
    for name in guides::order() {
        let f = guides::guide_file(&dir, name, Lang::En)
            .unwrap_or_else(|| panic!("{name}.md is missing"));
        let text = std::fs::read_to_string(f).unwrap();
        assert!(!guides::title_of(&text).is_empty(), "{name} has a title");
        assert!(
            !guides::render(&text, Lang::En).html.contains("<script"),
            "{name}"
        );
        if let Some(k) = guides::guide_file(&dir, name, Lang::Ko) {
            assert!(
                !guides::render(&std::fs::read_to_string(k).unwrap(), Lang::Ko)
                    .html
                    .is_empty()
            );
        }
    }
}

#[test]
fn steps_match_make_pdfs() {
    let Some(dir) = real_guides() else { return };
    let py = std::fs::read_to_string(dir.join("../tools/release/make_pdfs.py")).unwrap();
    let block = &py[py.find("STEPS = [").unwrap()..];
    let block = &block[..block.find("\n]").unwrap()];
    // Each step line: ("0 - Start here", "0 - 여기서 시작", ["00-READ-ME-FIRST", ...]),
    let mut found = Vec::new();
    for line in block.lines().filter(|l| l.trim_start().starts_with("(\"")) {
        let names = &line[line.find('[').unwrap() + 1..line.rfind(']').unwrap()];
        let names: Vec<String> = names
            .split(',')
            .map(|n| n.trim().trim_matches('"').to_string())
            .filter(|n| !n.is_empty())
            .collect();
        let labels: Vec<&str> = line.split('"').collect();
        found.push((labels[1].to_string(), labels[3].to_string(), names));
    }
    let ours: Vec<(String, String, Vec<String>)> = guides::STEPS
        .iter()
        .map(|s| {
            (
                format!("{} - {}", s.num, s.en),
                format!("{} - {}", s.num, s.ko),
                s.guides.iter().map(|g| g.to_string()).collect(),
            )
        })
        .collect();
    assert_eq!(
        ours, found,
        "guides::STEPS must be the same as STEPS in tools/release/make_pdfs.py"
    );
}
