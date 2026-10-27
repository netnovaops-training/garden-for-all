//! Pack sections: a section template is a short list of plain blocks (heading, text, list, link,
//! divider), each with English and Korean words. There is no markup and no template language in
//! it: the site draws the blocks itself and escapes every word.
//!
//! A link goes to a page of this site (`index.php`, `page.php?name=roses`) or to a plain
//! `https://` address. Nothing else (no `javascript:`, no `data:`, no `//host`).

use serde::Deserialize;

use super::{problem, Problem};
use crate::php::he;

const MAX_BLOCKS: usize = 30;
const MAX_TEXT: usize = 600;
const MAX_ITEMS: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading(Words),
    Text(Words),
    List(Vec<Words>),
    Link { href: String, words: Words },
    Divider,
}

/// English and Korean (Korean may be empty).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Words {
    pub en: String,
    #[serde(default)]
    pub ko: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema: u32,
    blocks: Vec<Raw>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Raw {
    Heading {
        en: String,
        #[serde(default)]
        ko: String,
    },
    Text {
        en: String,
        #[serde(default)]
        ko: String,
    },
    List {
        items: Vec<Words>,
    },
    Link {
        href: String,
        en: String,
        #[serde(default)]
        ko: String,
    },
    Divider {},
}

fn words_ok(w: &Words) -> bool {
    let ok = |s: &str| {
        s.chars().count() <= MAX_TEXT && !s.chars().any(|c| c.is_control() || c == '<' || c == '>')
    };
    !w.en.trim().is_empty() && ok(&w.en) && ok(&w.ko)
}

/// A link target: a page of this site, or `https://`.
pub fn href_ok(h: &str) -> bool {
    if h.is_empty() || h.len() > 200 {
        return false;
    }
    if let Some(rest) = h.strip_prefix("https://") {
        return !rest.is_empty()
            && !rest.starts_with('/')
            && !rest.split('/').next().unwrap_or("").contains('@')
            && h.bytes().all(|b| {
                b.is_ascii_alphanumeric()
                    || matches!(
                        b,
                        b'.' | b'_'
                            | b'~'
                            | b':'
                            | b'/'
                            | b'?'
                            | b'#'
                            | b'='
                            | b'&'
                            | b'%'
                            | b'-'
                            | b'+'
                    )
            });
    }
    // A page of this site: letters, digits and `. _ - ? = & /`, no scheme, no `..`, not `//`.
    !h.starts_with('/')
        && !h.contains("..")
        && !h.contains("//")
        && !h.contains(':')
        && h.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'?' | b'=' | b'&' | b'/')
        })
}

/// Read a section template strictly.
pub fn parse(text: &str) -> Result<Vec<Block>, Problem> {
    let bad = |en: &str, ko: &str| {
        problem(
            format!("The section text is not allowed: {en}."),
            format!("섹션 글이 허용되지 않아요: {ko}."),
        )
    };
    let f: File = serde_json::from_str(text).map_err(|_| bad("wrong form", "형식이 맞지 않음"))?;
    if f.schema != 1 || f.blocks.is_empty() || f.blocks.len() > MAX_BLOCKS {
        return Err(bad("wrong form or size", "형식이나 크기가 맞지 않음"));
    }
    let mut out = Vec::new();
    for b in f.blocks {
        out.push(match b {
            Raw::Heading { en, ko } => Block::Heading(Words { en, ko }),
            Raw::Text { en, ko } => Block::Text(Words { en, ko }),
            Raw::List { items } => {
                if items.is_empty() || items.len() > MAX_ITEMS {
                    return Err(bad("a list of the wrong size", "목록 크기가 맞지 않음"));
                }
                Block::List(items)
            }
            Raw::Link { href, en, ko } => {
                if !href_ok(&href) {
                    return Err(bad("a link that is not allowed", "허용되지 않는 링크"));
                }
                Block::Link {
                    href,
                    words: Words { en, ko },
                }
            }
            Raw::Divider {} => Block::Divider,
        });
    }
    let all_ok = out.iter().all(|b| match b {
        Block::Heading(w) | Block::Text(w) | Block::Link { words: w, .. } => words_ok(w),
        Block::List(items) => items.iter().all(words_ok),
        Block::Divider => true,
    });
    if !all_ok {
        return Err(bad("empty or unusual text", "비었거나 이상한 글"));
    }
    Ok(out)
}

fn both(w: &Words) -> String {
    if w.ko.trim().is_empty() {
        he(&w.en)
    } else {
        format!("{} · {}", he(&w.en), he(&w.ko))
    }
}

/// The blocks as HTML, in the look of the site's cards. Every word is escaped.
pub fn render(blocks: &[Block]) -> String {
    let mut h = String::from("<div class=\"cartoony-card pk-section\" style=\"padding:20px; border:4px solid #333; border-radius:15px; box-shadow:6px 6px 0 rgba(0,0,0,0.8); margin:20px auto 30px; max-width:900px; width:95%; box-sizing:border-box; background:#fff;\">\n");
    for b in blocks {
        match b {
            Block::Heading(w) => h.push_str(&format!(
                "<h2 class=\"blog-cartoony-text\" style=\"text-align:center;\">{}</h2>\n",
                both(w)
            )),
            Block::Text(w) => {
                h.push_str(&format!("<p>{}</p>\n", he(&w.en)));
                if !w.ko.trim().is_empty() {
                    h.push_str(&format!("<p lang=\"ko\">{}</p>\n", he(&w.ko)));
                }
            }
            Block::List(items) => {
                h.push_str("<ul>\n");
                for w in items {
                    h.push_str(&format!("<li>{}</li>\n", both(w)));
                }
                h.push_str("</ul>\n");
            }
            Block::Link { href, words } => h.push_str(&format!(
                "<p><a href=\"{}\" rel=\"noopener noreferrer\">{}</a></p>\n",
                he(href),
                both(words)
            )),
            Block::Divider => h.push_str("<hr>\n"),
        }
    }
    h.push_str("</div>\n");
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_are_drawn_with_every_word_escaped() {
        let b = parse(
            r#"{"schema":1,"blocks":[
            {"type":"heading","en":"Hello & welcome","ko":"안녕"},
            {"type":"text","en":"Plain \"words\"","ko":"글"},
            {"type":"list","items":[{"en":"one"},{"en":"two","ko":"둘"}]},
            {"type":"link","href":"page.php?name=roses","en":"Roses","ko":"장미"},
            {"type":"divider"}]}"#,
        )
        .unwrap();
        let h = render(&b);
        assert!(h.contains("<h2 class=\"blog-cartoony-text\" style=\"text-align:center;\">Hello &amp; welcome · 안녕</h2>"));
        assert!(h.contains("<p>Plain &quot;words&quot;</p>"));
        assert!(h.contains("<li>one</li>") && h.contains("<li>two · 둘</li>"));
        assert!(h.contains(
            "<a href=\"page.php?name=roses\" rel=\"noopener noreferrer\">Roses · 장미</a>"
        ));
        assert!(h.contains("<hr>") && h.ends_with("</div>\n"));
    }

    #[test]
    fn markup_and_unknown_things_are_refused() {
        for bad in [
            r#"{"schema":1,"blocks":[{"type":"heading","en":"<script>alert(1)</script>"}]}"#,
            r#"{"schema":1,"blocks":[{"type":"text","en":"a","ko":"<b>"}]}"#,
            r#"{"schema":1,"blocks":[{"type":"html","en":"x"}]}"#,
            r#"{"schema":1,"blocks":[{"type":"text","en":"x","onclick":"y"}]}"#,
            r#"{"schema":1,"blocks":[{"type":"text","en":""}]}"#,
            r#"{"schema":1,"blocks":[]}"#,
            r#"{"schema":2,"blocks":[{"type":"divider"}]}"#,
            r#"{"schema":1,"blocks":[{"type":"text","en":"a"}],"x":1}"#,
            r#"{"schema":1,"blocks":[{"type":"list","items":[]}]}"#,
            r#"{"schema":1,"blocks":[{"type":"text","en":"line\u0007bell"}]}"#,
            "not json",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        let many = format!(
            r#"{{"schema":1,"blocks":[{}]}}"#,
            vec![r#"{"type":"divider"}"#; 31].join(",")
        );
        assert!(parse(&many).is_err());
        let long = format!(
            r#"{{"schema":1,"blocks":[{{"type":"text","en":"{}"}}]}}"#,
            "a".repeat(601)
        );
        assert!(parse(&long).is_err());
    }

    #[test]
    fn links_go_to_this_site_or_to_https_only() {
        for ok in [
            "index.php",
            "page.php?name=roses",
            "catalog.php?x=1&y=2",
            "https://example.org/a/b?c=d#e",
            "https://example.org",
        ] {
            assert!(href_ok(ok), "{ok}");
        }
        for bad in [
            "",
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html,x",
            "http://example.org",
            "//evil.test/x",
            "/etc/passwd",
            "../x",
            "a/../b",
            "https://user@evil.test/",
            "https:///x",
            "https://x.test/\" onmouseover=\"y",
            "https://x.test/ a",
            "page.php?name=<x>",
            "vbscript:x",
            "index.php\n",
            &format!("https://x.test/{}", "a".repeat(200)),
        ] {
            assert!(!href_ok(bad), "{bad}");
        }
    }
}
