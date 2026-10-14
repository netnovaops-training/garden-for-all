//! The page frame every page shares: head, header, hero banner, footer.
//!
//! This is `layout_open.php` + `header.php` + `layout_close.php` +
//! `footer.php` from the PHP site, as one template. A page supplies its
//! body and a few settings; everything around it is identical to the PHP.
//!
//! HOW TO READ THIS FILE. A handler builds a `Page` (title, body and options) and calls
//! `render`. `render` collects the extra values the frame needs (file versions, hero
//! images, facts, the user) into `LayoutTemplate` and fills the `layout.html` template.

// `use` brings names into scope. `Path` is a borrowed file-system path.
use std::path::Path;

// askama turns an HTML template file into Rust code at build time.
use askama::Template;
use rand::seq::SliceRandom;

// `crate::` means from this same program.
use crate::php::{filemtime, json_string_list};
use crate::session::Visitor;
use crate::AppState;

/// The description used in the page head when a page does not give its own.
const DEFAULT_DESC: &str =
    "Jun's and Kona's Garden — a plant catalog, garden journal, and growing guide for our home garden.";
/// The title used when a page does not give its own.
pub const DEFAULT_TITLE: &str = "Jun's and Kona's Garden";

/// What one page tells the frame about itself. The PHP pages did this by
/// setting variables before including `layout_open.php`; the field names
/// follow those variables.
pub struct Page {
    pub title: String,
    pub meta_desc: String,
    pub show_hero: bool,
    pub body_class: String,
    /// Raw HTML for the end of `<head>` (`$extraHead`).
    pub extra_head: String,
    /// Raw HTML between the header bar and the wave (`$afterHeaderBar`).
    pub after_header: String,
    pub body: String,
}

/// Methods on `Page`. `Self` means `Page`.
impl Page {
    /// Makes a `Page` around a finished body, with default title, description and hero banner.
    pub fn new(body: String) -> Self {
        Page {
            title: DEFAULT_TITLE.to_string(),
            meta_desc: DEFAULT_DESC.to_string(),
            show_hero: true,
            body_class: String::new(),
            extra_head: String::new(),
            after_header: String::new(),
            body,
        }
    }
}

/// The values the `layout.html` template uses. `#[derive(Template)]` and the `#[template]`
/// line below make askama connect this struct to that file: each field here is a name the
/// template can print. The `_v` fields are file modified times, added to script and style
/// links (`?v=...`) so a browser fetches a new copy only when the file changes.
#[derive(Template)]
#[template(path = "layout.html")]
struct LayoutTemplate {
    page_title: String,
    meta_desc: String,
    show_hero: bool,
    hero_initial: Vec<String>,
    style_v: u64,
    site_theme_v: u64,
    hero_v: u64,
    acc_v: u64,
    extra_head: String,
    body_class: String,
    after_header: String,
    is_admin: bool,
    plan_ok: bool,
    app_version: String,
    facts_json: String,
    deferred_json: String,
    current_user: String,
    year: i32,
    body: String,
}

/// Hero banner images: everything in `images/HeroBanner/` with one of the
/// extensions the PHP globbed for, in random order, as site-relative paths.
fn hero_images(www: &Path) -> Vec<String> {
    // The file types the PHP looked for. Capital versions are listed because file
    // names are case-sensitive on Linux.
    const EXTS: [&str; 8] = ["webp", "WEBP", "jpg", "jpeg", "png", "JPG", "JPEG", "PNG"];
    // Read the folder. `.flatten()` skips entries that could not be read.
    let mut found: Vec<String> = std::fs::read_dir(www.join("images/HeroBanner"))
        .map(|rd| {
            // Keep names whose extension is on the list.
            rd.flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| n.rsplit_once('.').map(|(_, ext)| EXTS.contains(&ext)).unwrap_or(false))
                .map(|n| format!("images/HeroBanner/{n}"))
                .collect()
        })
        .unwrap_or_default();
    // No images found: use a fallback photo from the web.
    if found.is_empty() {
        return vec!["https://images.unsplash.com/photo-1466692476868-aef1dfb1e735".to_string()];
    }
    // Sort first so the order is predictable, then shuffle for a random order each load.
    found.sort();
    found.shuffle(&mut rand::thread_rng());
    found
}

/// The "Did you know?" lines, as the JSON array the hero script wants.
fn facts_json(www: &Path) -> String {
    // Read the file, turn bytes into text (bad bytes become a placeholder), split into lines.
    let lines: Vec<String> = std::fs::read(www.join("data/facts.txt"))
        .map(|bytes| {
            String::from_utf8_lossy(&bytes)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default();
    // An empty or missing file gives a single welcome line.
    if lines.is_empty() {
        return json_string_list(&["Welcome to the Garden!".to_string()]);
    }
    json_string_list(&lines)
}

/// The version shown on the Android-app link, from `apps/version.json`.
/// Empty when the file is missing or has no `versionName`, like the PHP.
fn app_version(www: &Path) -> String {
    // Read the file, parse it as JSON, and take the `versionName` text. Any failure gives "".
    std::fs::read(www.join("apps/version.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| v.get("versionName").and_then(|n| n.as_str()).map(str::to_string))
        .unwrap_or_default()
}

/// The body class the PHP built: `admin-logged-in` when signed in, then the
/// page's own class.
fn body_class(is_admin: bool, own: &str) -> String {
    // Build a list of class names and join them with spaces.
    let mut parts: Vec<&str> = Vec::new();
    if is_admin {
        parts.push("admin-logged-in");
    }
    if !own.is_empty() {
        parts.push(own);
    }
    parts.join(" ")
}

/// Wrap a page body in the shared frame and return the full HTML.
pub fn render(state: &AppState, page: Page, visitor: &Visitor) -> String {
    let www = &state.www;
    // The hero images only matter when the page shows the banner.
    let mut images = if page.show_hero { hero_images(www) } else { Vec::new() };
    // The first 3 images load right away; the rest are "deferred" and loaded later by the
    // script. `split_off(3)` removes items from position 3 onward and returns them.
    let deferred = if images.len() > 3 { images.split_off(3) } else { Vec::new() };
    let is_admin = visitor.is_admin();

    // Fill in the template values.
    let t = LayoutTemplate {
        page_title: page.title,
        meta_desc: page.meta_desc,
        show_hero: page.show_hero,
        hero_initial: images,
        style_v: filemtime(www, "style.css"),
        site_theme_v: filemtime(www, "js/site-theme.js"),
        hero_v: filemtime(www, "js/hero.js"),
        acc_v: filemtime(www, "js/accessibility.js"),
        extra_head: page.extra_head,
        body_class: body_class(is_admin, &page.body_class),
        after_header: page.after_header,
        is_admin,
        plan_ok: visitor.plan_ok(state),
        app_version: app_version(www),
        facts_json: facts_json(www),
        deferred_json: json_string_list(&deferred),
        current_user: visitor.username.clone(),
        year: common::current_year(),
        body: page.body,
    };
    // Render the template to HTML. If that fails, log it and show a plain error message.
    t.render().unwrap_or_else(|e| {
        tracing::error!("layout render failed: {e}");
        "<h1>Something went wrong</h1>".to_string()
    })
}
