//! Which pages the builder manages, and what each page tells the frame about itself.
//!
//! HOW TO READ THIS FILE. `BUILTIN` lists the site's own pages (their address, their name in
//! the top menu and whether they are in it today). The owner's pages are `p-<name>` and live
//! at `/page.php?name=<name>`. `Meta` is the part of a saved layout that is about the page
//! and not about its sections: the titles, the menu switch, the banner switch, "removed".

use super::layout::{self, Layout};

/// One of the site's own pages.
pub struct Builtin {
    pub id: &'static str,
    pub href: &'static str,
    pub label_en: &'static str,
    pub label_ko: &'static str,
    /// In the top menu until the owner hides it.
    pub in_menu: bool,
}

/// The site's own pages, in top-menu order.
pub const BUILTIN: &[Builtin] = &[
    Builtin {
        id: "home",
        href: "index.php",
        label_en: "Home",
        label_ko: "처음 화면",
        in_menu: true,
    },
    Builtin {
        id: "search",
        href: "plant_listing.php",
        label_en: "Quick Search",
        label_ko: "빠른 검색",
        in_menu: true,
    },
    Builtin {
        id: "catalog",
        href: "catalog.php",
        label_en: "Full Catalog",
        label_ko: "전체 카탈로그",
        in_menu: true,
    },
    Builtin {
        id: "data",
        href: "data.php",
        label_en: "Data",
        label_ko: "데이터",
        in_menu: true,
    },
    Builtin {
        id: "gallery",
        href: "gallery.php",
        label_en: "Gallery",
        label_ko: "갤러리",
        in_menu: true,
    },
    Builtin {
        id: "blog",
        href: "blog.php",
        label_en: "Garden archive",
        label_ko: "정원 기록 보관소",
        in_menu: false,
    },
];

/// The site's own page with this id.
pub fn builtin(id: &str) -> Option<&'static Builtin> {
    BUILTIN.iter().find(|b| b.id == id)
}

/// The address of a page, relative to the site (pages and the editor live at the top level).
pub fn href(page: &str) -> String {
    match builtin(page) {
        Some(b) => b.href.to_string(),
        None => format!("page.php?name={}", layout::custom_slug(page).unwrap_or("")),
    }
}

/// What a saved layout says about its page (everything optional; absent = the default).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meta {
    pub title_en: String,
    pub title_ko: String,
    /// `None` = the page's default (in the menu for new pages and most site pages).
    pub menu: Option<bool>,
    /// `None` = the page's default banner.
    pub hero: Option<bool>,
    pub removed: bool,
}

impl Meta {
    pub fn of(l: &Layout) -> Meta {
        Meta {
            title_en: l.title_en.clone().unwrap_or_default(),
            title_ko: l.title_ko.clone().unwrap_or_default(),
            menu: l.menu,
            hero: l.hero,
            removed: l.removed.unwrap_or(false),
        }
    }

    /// Does the top menu list this page? (Removed pages never; a new page by default.)
    pub fn in_menu(&self, page: &str) -> bool {
        if self.removed {
            return false;
        }
        self.menu
            .unwrap_or_else(|| builtin(page).is_none_or(|b| b.in_menu))
    }

    /// The page's name in each language (for the editor's page list): the owner's title, else
    /// the site page's own name (Korean falls back to the English one for owner pages).
    pub fn names(&self, page: &str) -> (String, String) {
        let b = builtin(page);
        let en = if !self.title_en.is_empty() {
            self.title_en.clone()
        } else if let Some(b) = b {
            b.label_en.to_string()
        } else {
            layout::custom_slug(page).unwrap_or("").to_string()
        };
        let ko = if !self.title_ko.is_empty() {
            self.title_ko.clone()
        } else if let Some(b) = b.filter(|_| self.title_en.is_empty()) {
            b.label_ko.to_string()
        } else {
            en.clone()
        };
        (en, ko)
    }

    /// The menu name: the owner's title (English, then Korean), else the page's own name.
    /// A page with both shows "English · 한국어".
    pub fn label(&self, page: &str) -> String {
        let own = match (self.title_en.is_empty(), self.title_ko.is_empty()) {
            (false, false) => format!("{} · {}", self.title_en, self.title_ko),
            (false, true) => self.title_en.clone(),
            (true, false) => self.title_ko.clone(),
            (true, true) => String::new(),
        };
        if !own.is_empty() {
            return own;
        }
        match builtin(page) {
            Some(b) => b.label_en.to_string(),
            None => layout::custom_slug(page).unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_pages_have_addresses_and_owner_pages_use_page_php() {
        assert_eq!(href("home"), "index.php");
        assert_eq!(href("search"), "plant_listing.php");
        assert_eq!(href("p-roses"), "page.php?name=roses");
        assert_eq!(href("p-"), "page.php?name=");
    }

    #[test]
    fn menu_defaults_and_labels() {
        let none = Meta::default();
        assert!(none.in_menu("home") && none.in_menu("p-new"));
        assert!(!none.in_menu("blog"), "the archive was never in the menu");
        let hidden = Meta {
            menu: Some(false),
            ..Meta::default()
        };
        assert!(!hidden.in_menu("p-new") && !hidden.in_menu("data"));
        let shown = Meta {
            menu: Some(true),
            ..Meta::default()
        };
        assert!(shown.in_menu("blog"));
        let gone = Meta {
            removed: true,
            menu: Some(true),
            ..Meta::default()
        };
        assert!(!gone.in_menu("p-new"));
        assert_eq!(none.label("catalog"), "Full Catalog");
        assert_eq!(none.label("p-roses"), "roses");
        let both = Meta {
            title_en: "Roses".into(),
            title_ko: "장미".into(),
            ..Meta::default()
        };
        assert_eq!(both.label("p-roses"), "Roses · 장미");
    }
}
