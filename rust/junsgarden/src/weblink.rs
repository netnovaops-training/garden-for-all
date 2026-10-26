//! The plant "Source" link: the grower's product page, kept in the
//! `product_url` column of `data_source` and `future_plants`.
//!
//! HOW TO READ THIS FILE. `check` is the one rule every path uses (admin save,
//! JSON import, and the pages before they draw the link). `link_html` draws the
//! link itself. Nothing here touches the database.
//!
//! Why so strict: the value ends up in an `href`, so only a plain `https://`
//! address with a real host is allowed. `javascript:`, `data:`, `http://`,
//! relative paths, user:password@ parts, spaces and control characters are refused.
//! (Not `source_url`: that column holds the image folder the photos came from.)

use crate::php::he;

/// The longest address the column holds (`varchar(500)`).
pub const MAX_LEN: usize = 500;

/// The message shown when an address is refused, in English and Korean.
pub const INVALID_MSG: &str = "Source link must be a full https:// address (for example https://www.example.com/plant), with no spaces, at most 500 characters. / 출처 링크는 https://로 시작하는 전체 주소여야 해요(예: https://www.example.com/plant). 띄어쓰기 없이 500자까지 쓸 수 있어요.";

/// Checks one address. Gives back `Ok(None)` for an empty value (stored as NULL),
/// `Ok(Some(url))` for a good one (spaces at the ends trimmed), or `Err` with the
/// EN/KO message for anything else.
pub fn check(raw: &str) -> Result<Option<String>, &'static str> {
    let v = raw.trim();
    if v.is_empty() {
        return Ok(None);
    }
    // Length in characters, the way MariaDB counts varchar(500).
    if v.chars().count() > MAX_LEN {
        return Err(INVALID_MSG);
    }
    // No spaces or control characters anywhere, and no characters that only
    // make sense when someone is trying to break out of an HTML attribute.
    if v.chars()
        .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '"' | '<' | '>' | '\\' | '`'))
    {
        return Err(INVALID_MSG);
    }
    // Scheme: https only (any capitals, e.g. HTTPS://, are accepted).
    let Some(rest) = v
        .get(..8)
        .filter(|s| s.eq_ignore_ascii_case("https://"))
        .map(|_| &v[8..])
    else {
        return Err(INVALID_MSG);
    };
    // Host: everything up to the first / ? or #. Letters, digits, dots, dashes
    // and an optional :port. It needs a dot, so a bare "localhost" fails, and "@"
    // is not allowed, so a "user:password@" part fails too.
    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..host_end];
    let (name, port) = match host.split_once(':') {
        Some((n, p)) => (n, Some(p)),
        None => (host, None),
    };
    let name_ok = !name.is_empty()
        && name.contains('.')
        && !name.starts_with(['.', '-'])
        && !name.ends_with(['.', '-'])
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'));
    let port_ok =
        port.is_none_or(|p| !p.is_empty() && p.len() <= 5 && p.chars().all(|c| c.is_ascii_digit()));
    if !name_ok || !port_ok {
        return Err(INVALID_MSG);
    }
    Ok(Some(v.to_string()))
}

/// The good address in `raw`, or `None` when it is empty or fails `check`.
/// Used before drawing, so an old or bad row simply shows no link.
pub fn safe(raw: Option<&str>) -> Option<String> {
    raw.and_then(|r| check(r).ok().flatten())
}

/// The "Source" link as HTML, or "" when the address is not good.
/// The address goes only into the escaped `href`; the visible text is always
/// "Source" (the same word in the English and Korean pages).
pub fn link_html(raw: Option<&str>) -> String {
    match safe(raw) {
        Some(u) => format!(
            "<a class=\"plant-source\" href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer nofollow\">Source</a>",
            he(&u)
        ),
        None => String::new(),
    }
}

/// For a plant row on its way to the browser as JSON: keeps `product_url` only
/// when it passes `check`, otherwise sets it to null. Rows without the column
/// are left alone.
pub fn clean_json(m: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(v) = m.get_mut("product_url") {
        *v = match safe(v.as_str()) {
            Some(u) => serde_json::Value::String(u),
            None => serde_json::Value::Null,
        };
    }
}

// Tests run with `cargo test`; this module is left out of the real program.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_https_addresses() {
        assert_eq!(
            check("https://x.y/path"),
            Ok(Some("https://x.y/path".into()))
        );
        assert_eq!(
            check("  https://www.example.com/plants/a-b?c=1#d  "),
            Ok(Some("https://www.example.com/plants/a-b?c=1#d".into()))
        );
        assert!(check("HTTPS://example.com").is_ok());
        assert!(check("https://example.com:8443/x").is_ok());
    }

    #[test]
    fn empty_is_none() {
        assert_eq!(check(""), Ok(None));
        assert_eq!(check("   "), Ok(None));
        assert_eq!(safe(None), None);
    }

    #[test]
    fn refuses_everything_else() {
        for bad in [
            "http://example.com/x",
            "javascript:alert(1)",
            "data:text/html,<b>x</b>",
            "/plants/lavender",
            "example.com/plants",
            "https://",
            "https:///path",
            "https://exa mple.com/",
            "https://example.com/a b",
            "https://example.com/\tx",
            "https://example.com/\u{0}",
            "https://user:pw@example.com/",
            "https://localhost/",
            "https://example.com:/x",
            "https://example.com:99999999/x",
            "https://example.com/\"onmouseover=\"x",
            "https://example.com/<script>",
            "//example.com/x",
        ] {
            assert_eq!(check(bad), Err(INVALID_MSG), "should refuse {bad:?}");
        }
        // 501 characters is one too many; 500 is fine.
        let base = "https://example.com/";
        let ok = format!("{base}{}", "a".repeat(MAX_LEN - base.len()));
        assert!(check(&ok).is_ok());
        assert!(check(&format!("{ok}a")).is_err());
    }

    #[test]
    fn link_has_the_safe_attributes_and_no_url_text() {
        let h = link_html(Some("https://www.example.com/plants/x?a=1&b=2"));
        assert!(h.contains("href=\"https://www.example.com/plants/x?a=1&amp;b=2\""));
        assert!(h.contains("target=\"_blank\""));
        assert!(h.contains("rel=\"noopener noreferrer nofollow\""));
        assert!(h.ends_with(">Source</a>"));
        // The address appears only once: inside href, never as visible text.
        assert_eq!(h.matches("example.com").count(), 1);
    }

    #[test]
    fn json_rows_keep_only_good_links() {
        let mut m: serde_json::Map<String, serde_json::Value> = serde_json::from_str(
            r#"{"id":1,"product_url":"javascript:alert(1)","source_url":"https://x.y/img/"}"#,
        )
        .unwrap();
        clean_json(&mut m);
        assert!(m["product_url"].is_null());
        // source_url (the image folder) is not touched.
        assert_eq!(m["source_url"], "https://x.y/img/");
        let mut m: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(r#"{"product_url":" https://x.y/p "}"#).unwrap();
        clean_json(&mut m);
        assert_eq!(m["product_url"], "https://x.y/p");
        let mut m = serde_json::Map::new();
        clean_json(&mut m);
        assert!(!m.contains_key("product_url"));
    }

    #[test]
    fn bad_rows_draw_nothing() {
        assert_eq!(link_html(Some("javascript:alert(1)")), "");
        assert_eq!(link_html(Some("http://example.com")), "");
        assert_eq!(link_html(Some("")), "");
        assert_eq!(link_html(None), "");
    }
}
