//! English or Korean for the plant admin's newer screens (Add plant, Receipt, Pictures, the
//! review and the Advanced menu). The choice is a small cookie, `st_lang`, set by the
//! "English / 한국어" links in the sidebar (`?setlang=ko`), so every page and every message
//! after a button press uses the same language. The older Streamlit tabs stay in English.

use axum::http::{header, HeaderMap};

/// The two languages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    Ko,
}

impl Lang {
    /// The language from the `st_lang` cookie (anything but "ko" is English).
    pub fn from_headers(h: &HeaderMap) -> Lang {
        let raw = h
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let ko = raw
            .split(';')
            .filter_map(|p| p.trim().split_once('='))
            .any(|(k, v)| k == "st_lang" && v == "ko");
        if ko {
            Lang::Ko
        } else {
            Lang::En
        }
    }

    /// Pick the words for this language.
    pub fn t<'a>(self, en: &'a str, ko: &'a str) -> &'a str {
        match self {
            Lang::En => en,
            Lang::Ko => ko,
        }
    }

    /// "en" or "ko".
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ko => "ko",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_picks_the_language() {
        let mut h = HeaderMap::new();
        assert_eq!(Lang::from_headers(&h), Lang::En);
        h.insert(header::COOKIE, "a=1; st_lang=ko".parse().unwrap());
        assert_eq!(Lang::from_headers(&h), Lang::Ko);
        assert_eq!(Lang::Ko.t("Approve", "승인"), "승인");
    }
}
