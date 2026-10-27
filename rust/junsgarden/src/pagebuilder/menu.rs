//! The top menu's page links. The site's own pages come first (as they always did), then the
//! owner's pages in the order they were made. A page the owner hid, renamed or removed shows
//! that way; with no change at all the links are exactly the old hand-written ones.

use std::collections::HashMap;
use std::sync::Arc;

use super::pages::{self, Meta};
use super::saved;
use crate::php::he;
use crate::AppState;

/// The site's own links as they were written in `header.html` before the page builder
/// (including their odd spacing: the first line's indent is in the template).
pub const DEFAULT_NAV: &str = "<li><a href=\"index.php\">Home</a></li>\n                <li><a href=\"plant_listing.php\">Quick Search</a></li> \n\t\t<li><a href=\"catalog.php\">Full Catalog</a></li>\n\t\t<li><a href=\"data.php\">Data</a></li>\n\t\t<li><a href=\"gallery.php\">Gallery</a></li>";

/// The links for the saved page settings. `customs` are the owner's page ids, oldest first.
pub fn build(metas: &HashMap<String, Meta>, customs: &[String]) -> String {
    let mut items: Vec<String> = Vec::new();
    let mut changed = false;
    for b in pages::BUILTIN {
        let meta = metas.get(b.id).cloned().unwrap_or_default();
        let shown = meta.in_menu(b.id);
        let label = meta.label(b.id);
        if shown != b.in_menu || (shown && label != b.label_en) {
            changed = true;
        }
        if shown {
            items.push(item(b.href, &label));
        }
    }
    let before = items.len();
    for id in customs {
        let meta = metas.get(id).cloned().unwrap_or_default();
        if meta.in_menu(id) {
            items.push(item(&pages::href(id), &meta.label(id)));
        }
    }
    if !changed && items.len() == before {
        return DEFAULT_NAV.to_string();
    }
    items.join("\n                ")
}

fn item(href: &str, label: &str) -> String {
    format!("<li><a href=\"{}\">{}</a></li>", he(href), he(label))
}

/// The menu links now (cached; the cache is dropped on every page change). The database not
/// answering gives the old links and is not cached.
pub async fn nav(state: &AppState) -> Arc<String> {
    if let Some(hit) = cached(state) {
        return hit;
    }
    let Some(metas) = saved::page_metas(state).await else {
        return Arc::new(DEFAULT_NAV.to_string());
    };
    let customs: Vec<String> = metas
        .iter()
        .map(|(id, _)| id.clone())
        .filter(|id| pages::builtin(id).is_none())
        .collect();
    let map: HashMap<String, Meta> = metas.into_iter().collect();
    let html = Arc::new(build(&map, &customs));
    if let Ok(mut c) = state.pagebuilder.menu.lock() {
        *c = Some(html.clone());
    }
    html
}

/// The cached links, if there are any (for code that cannot wait).
pub fn cached(state: &AppState) -> Option<Arc<String>> {
    state.pagebuilder.menu.lock().ok().and_then(|c| c.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(menu: Option<bool>, title: &str) -> Meta {
        Meta {
            menu,
            title_en: title.into(),
            ..Meta::default()
        }
    }

    #[test]
    fn nothing_changed_gives_the_old_links_byte_for_byte() {
        assert_eq!(build(&HashMap::new(), &[]), DEFAULT_NAV);
        // A saved layout that says nothing about the menu changes nothing either.
        let mut m = HashMap::new();
        m.insert("home".to_string(), Meta::default());
        assert_eq!(build(&m, &[]), DEFAULT_NAV);
        // A removed owner page and a hidden one add nothing.
        let mut m = HashMap::new();
        m.insert(
            "p-gone".to_string(),
            Meta {
                removed: true,
                ..Meta::default()
            },
        );
        m.insert("p-quiet".to_string(), meta(Some(false), ""));
        let customs = ["p-gone".to_string(), "p-quiet".to_string()];
        assert_eq!(build(&m, &customs), DEFAULT_NAV);
    }

    #[test]
    fn a_new_page_is_in_the_menu_by_default_and_a_switch_hides_it() {
        let mut m = HashMap::new();
        m.insert("p-roses".to_string(), meta(None, "Roses"));
        let html = build(&m, &["p-roses".to_string()]);
        assert!(html.starts_with(DEFAULT_NAV.split('\n').next().unwrap()));
        assert!(html.ends_with("<li><a href=\"page.php?name=roses\">Roses</a></li>"));
        m.insert("p-roses".to_string(), meta(Some(false), "Roses"));
        assert_eq!(build(&m, &["p-roses".to_string()]), DEFAULT_NAV);
    }

    #[test]
    fn site_pages_can_be_hidden_renamed_or_added() {
        let mut m = HashMap::new();
        m.insert("data".to_string(), meta(Some(false), ""));
        m.insert("catalog".to_string(), meta(None, "Plants"));
        m.insert("blog".to_string(), meta(Some(true), ""));
        let html = build(&m, &[]);
        assert!(!html.contains("data.php"));
        assert!(html.contains("<a href=\"catalog.php\">Plants</a>"));
        assert!(html.contains("<a href=\"blog.php\">Garden archive</a>"));
    }

    /// The frame prints the old hand-written links (as in `header.html` on `main`) when nothing
    /// was changed: same bytes, tabs and the trailing space included.
    #[tokio::test]
    async fn the_frame_prints_the_old_links_unchanged() {
        let old = "            <ul>\n                <li><a href=\"index.php\">Home</a></li>\n                <li><a href=\"plant_listing.php\">Quick Search</a></li> \n\t\t<li><a href=\"catalog.php\">Full Catalog</a></li>\n\t\t<li><a href=\"data.php\">Data</a></li>\n\t\t<li><a href=\"gallery.php\">Gallery</a></li>\n                <li><a href=\"https://wiki.junsgarden.duckdns.org/\"\n";
        let st = crate::test_state(std::path::PathBuf::new(), Vec::new());
        let html = crate::layout::render_now(
            &st,
            crate::layout::Page::new("x".into()),
            &crate::session::Visitor::default(),
        );
        assert!(html.contains(old), "the menu links changed");
    }

    #[test]
    fn titles_are_escaped() {
        let mut m = HashMap::new();
        m.insert("p-x".to_string(), meta(None, "A&B"));
        assert!(build(&m, &["p-x".to_string()]).contains(">A&amp;B<"));
    }
}
