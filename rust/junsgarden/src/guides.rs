//! The garden's guides (START-HERE/*.md and START-HERE/ko/*.md), read for the
//! website's help pages (`pages/help.rs`).
//!
//! The same Markdown files are what the PDFs and the handbook are made from
//! (tools/release/make_pdfs.py, make_handbook.py), so the help pages, the
//! handbook and the PDFs always say the same thing. Nothing is copied: the
//! garden folder's START-HERE is mounted into the website's container (see
//! docker-compose.yml) and read when a page is asked for.
//!
//! HOW TO READ THIS FILE (for the reader). `STEPS` is the reading order (the same list
//! as STEPS in make_pdfs.py; a test checks they match). `guide_file` is the
//! ONLY way a guide's file name is made, and it only accepts names on that
//! list, so an address can never reach another file. `parse_registry` reads
//! help-ids.tsv. `render` turns a guide into HTML: it gives every heading the
//! same anchor the handbook gives it (`slugify`, the rules of Python-Markdown),
//! points links at the help pages, and shows any other HTML as plain text.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use pulldown_cmark::{html, CowStr, Event, HeadingLevel, LinkType, Options, Parser, Tag, TagEnd};
use unicode_normalization::UnicodeNormalization;

/// The language a guide is read in. English is the default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Ko,
}

impl Lang {
    /// `?lang=ko` means Korean; anything else (or nothing) means English.
    pub fn from_param(v: Option<&str>) -> Lang {
        if v == Some("ko") {
            Lang::Ko
        } else {
            Lang::En
        }
    }

    /// The `lang` attribute and the value of `?lang=`.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ko => "ko",
        }
    }

    /// What to add to an address with no `?` yet: "" or "?lang=ko".
    pub fn qs(self) -> &'static str {
        match self {
            Lang::En => "",
            Lang::Ko => "?lang=ko",
        }
    }

    /// What to add to an address that already has `?...`: "" or "&lang=ko".
    pub fn amp(self) -> &'static str {
        match self {
            Lang::En => "",
            Lang::Ko => "&lang=ko",
        }
    }
}

/// One step of the reading order: its number, its name in both languages,
/// and its guides (file names in START-HERE/ without `.md`).
pub struct Step {
    pub num: &'static str,
    pub en: &'static str,
    pub ko: &'static str,
    pub guides: &'static [&'static str],
}

/// The reading order. The same as STEPS in tools/release/make_pdfs.py (the
/// test `steps_match_make_pdfs` fails if the two drift apart).
pub const STEPS: &[Step] = &[
    Step {
        num: "0",
        en: "Start here",
        ko: "여기서 시작",
        guides: &[
            "00-OVERVIEW",
            "00-READ-ME-FIRST",
            "00b-before-you-start",
            "23-full-setup",
            "02a-what-is-docker",
        ],
    },
    Step {
        num: "1",
        en: "Ubuntu on the computer",
        ko: "컴퓨터에 Ubuntu 설치",
        guides: &["01-ubuntu-laptop"],
    },
    Step {
        num: "2",
        en: "Docker and the install",
        ko: "Docker와 설치",
        guides: &["02-docker", "03-run-your-garden"],
    },
    Step {
        num: "3",
        en: "Going online",
        ko: "인터넷에 올리기",
        guides: &[
            "04-go-public-no-ip-caddy",
            "04b-take-over-junsgarden-zapto",
            "07-sso-gate",
            "14-reserve-your-ip",
            "20-ssh-and-remote-desktop",
        ],
    },
    Step {
        num: "4",
        en: "Email, keys and AI",
        ko: "이메일, 키, AI",
        guides: &[
            "16-ai-studio",
            "05-gemini-prompts",
            "08-gemini-setup",
            "15-local-ai",
            "17-android-app-and-ai-coding",
        ],
    },
    Step {
        num: "5",
        en: "Backups and your data",
        ko: "백업과 내 데이터",
        guides: &["13-backups-and-folders", "12-your-database"],
    },
    Step {
        num: "6",
        en: "Everyday use and help",
        ko: "평소에 쓰기와 도움",
        guides: &[
            "19-every-menu-option",
            "25-common-questions",
            "26-rename-move-and-switch",
            "09-troubleshooting",
            "11-maintain-your-site",
            "10-the-plan",
            "18-review-the-code",
            "06-shopping-list",
        ],
    },
    Step {
        num: "6b",
        en: "The terminal way (bash), harder but not tough",
        ko: "터미널로 하기(bash), 조금 어렵지만 힘들지 않아요",
        guides: &["22-bash-walkthrough", "24-optional-tools"],
    },
];

/// Is `name` one of the guides in `STEPS`? Only these can ever be opened.
pub fn is_guide(name: &str) -> bool {
    STEPS.iter().any(|s| s.guides.contains(&name))
}

/// The step a guide belongs to.
pub fn step_of(name: &str) -> Option<&'static Step> {
    STEPS.iter().find(|s| s.guides.contains(&name))
}

/// Every guide in reading order.
pub fn order() -> impl Iterator<Item = &'static str> {
    STEPS.iter().flat_map(|s| s.guides.iter().copied())
}

/// The file of one guide in one language, if the name is on the list and the
/// file is there. The path is built from the list's own text, never from the
/// address, so `../` or any other trick cannot reach a different file.
pub fn guide_file(dir: &Path, name: &str, lang: Lang) -> Option<PathBuf> {
    let known = order().find(|g| *g == name)?;
    let p = match lang {
        Lang::En => dir.join(format!("{known}.md")),
        Lang::Ko => dir.join("ko").join(format!("{known}.md")),
    };
    p.is_file().then_some(p)
}

/// A picture in START-HERE/images that may be shown, and its content type.
/// Only plain names (letters, digits, `.`, `-`, `_`; no folders, no leading
/// dot) with a picture extension.
pub fn image_file(dir: &Path, name: &str) -> Option<(PathBuf, &'static str)> {
    if !image_name_ok(name) {
        return None;
    }
    let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    let p = dir.join("images").join(name);
    p.is_file().then_some((p, mime))
}

/// The name rule for pictures (see `image_file`).
fn image_name_ok(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 120
        && !name.starts_with('.')
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// A help id is small letters, numbers and `-` (the rule check_help_ids.py checks).
pub fn id_ok(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 60
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--")
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

// ---------------------------------------------------------------- help-ids.tsv

/// One row of START-HERE/help-ids.tsv.
#[derive(Clone, Debug, Default)]
pub struct HelpEntry {
    pub id: String,
    pub guide: String,
    /// The headings' own anchors. The site links to the stable `help-ID`
    /// anchors instead; the tests check these against the rendered pages.
    #[cfg_attr(not(test), allow(dead_code))]
    pub anchor_en: String,
    #[cfg_attr(not(test), allow(dead_code))]
    pub anchor_ko: String,
    pub tags_en: String,
    pub tags_ko: String,
    pub brief_en: String,
    pub brief_ko: String,
}

impl HelpEntry {
    pub fn tags(&self, lang: Lang) -> &str {
        if lang == Lang::Ko {
            &self.tags_ko
        } else {
            &self.tags_en
        }
    }
    pub fn brief(&self, lang: Lang) -> &str {
        if lang == Lang::Ko {
            &self.brief_ko
        } else {
            &self.brief_en
        }
    }
    /// The anchor on the guide's page: `help-ID` in English, `help-ID-ko` in
    /// Korean, as in the handbook.
    pub fn anchor(&self, lang: Lang) -> String {
        match lang {
            Lang::En => format!("help-{}", self.id),
            Lang::Ko => format!("help-{}-ko", self.id),
        }
    }
}

/// Read the registry's text. Lines starting with `#` are notes; the first
/// other line names the columns. A row with a bad id, an unknown guide or the
/// wrong number of columns is left out (check_help_ids.py reports those).
pub fn parse_registry(text: &str) -> Vec<HelpEntry> {
    let mut lines = text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'));
    let Some(head) = lines.next() else {
        return Vec::new();
    };
    let cols: Vec<&str> = head.split('\t').collect();
    let col = |name: &str| cols.iter().position(|c| *c == name);
    let (Some(id), Some(guide)) = (col("id"), col("guide")) else {
        return Vec::new();
    };
    let get = |f: &[&str], name: &str| {
        col(name)
            .and_then(|i| f.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    for l in lines {
        let f: Vec<&str> = l.split('\t').collect();
        if f.len() != cols.len() || !id_ok(f[id]) || !is_guide(f[guide]) {
            continue;
        }
        out.push(HelpEntry {
            id: f[id].to_string(),
            guide: f[guide].to_string(),
            anchor_en: get(&f, "anchor_en"),
            anchor_ko: get(&f, "anchor_ko"),
            tags_en: get(&f, "tags_en"),
            tags_ko: get(&f, "tags_ko"),
            brief_en: get(&f, "brief_en"),
            brief_ko: get(&f, "brief_ko"),
        });
    }
    out
}

/// help-ids.tsv from the guides folder; empty when it is missing.
pub fn load_registry(dir: &Path) -> Vec<HelpEntry> {
    std::fs::read_to_string(dir.join("help-ids.tsv"))
        .map(|t| parse_registry(&t))
        .unwrap_or_default()
}

// ---------------------------------------------------------------- titles and summaries

/// The guide's title: its first line without `# ` and without the number
/// (`# 03. Run your garden` -> `Run your garden`), like make_handbook.py.
pub fn title_of(md: &str) -> String {
    let first = md
        .lines()
        .next()
        .unwrap_or("")
        .trim_start_matches('#')
        .trim();
    // Drop "03. " / "04b. ": digits, maybe one small letter, a dot, a space.
    let digits = first.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 {
        let rest = &first[digits..];
        let rest = rest
            .strip_prefix(|c: char| c.is_ascii_lowercase())
            .unwrap_or(rest);
        if let Some(r) = rest.strip_prefix('.') {
            if r.starts_with(char::is_whitespace) {
                return r.trim().to_string();
            }
        }
    }
    first.to_string()
}

/// A one-line description: the first point of the guide's short version
/// ("Quick version" / "짧은 버전"), or else its first paragraph, as plain text.
pub fn summary_of(md: &str) -> String {
    let mut in_quick = false;
    let mut first_para = None;
    for line in md.lines().skip(1) {
        let t = line.trim();
        if let Some(h) = t.strip_prefix("## ") {
            if in_quick {
                break;
            }
            in_quick = h.starts_with("Quick version") || h.starts_with("짧은 버전");
            continue;
        }
        if in_quick {
            if let Some(b) = t.strip_prefix("- ") {
                return clip(&plain_text(b), 180);
            }
        } else if first_para.is_none()
            && !t.is_empty()
            && !t.starts_with(['#', '>', '-', '*', '|', '<', '`'])
        {
            first_para = Some(t.to_string());
        }
    }
    first_para
        .map(|p| clip(&plain_text(&p), 180))
        .unwrap_or_default()
}

/// Markdown text without its marks (`**`, backticks, link targets).
fn plain_text(md: &str) -> String {
    let mut s = String::new();
    for ev in Parser::new(md) {
        match ev {
            Event::Text(t) | Event::Code(t) => s.push_str(&t),
            Event::SoftBreak | Event::HardBreak => s.push(' '),
            _ => {}
        }
    }
    s.trim().to_string()
}

/// At most `max` characters, cut at a space, with "…" when cut.
fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    let cut = cut.rsplit_once(' ').map(|(a, _)| a).unwrap_or(&cut);
    format!("{}…", cut.trim_end_matches([',', ';', ':', ' ']))
}

// ---------------------------------------------------------------- anchors

/// A heading's anchor, by the rules of Python-Markdown's `toc` extension,
/// which the handbook and check_help_ids.py use: English (`unicode = false`)
/// is folded to plain ASCII first (é -> e, Korean letters dropped); Korean
/// (`unicode = true`, `slugify_unicode`) keeps every letter. Then anything
/// but letters, digits, `_`, spaces and `-` is removed, the ends trimmed,
/// everything lower-cased, and each run of spaces and `-` becomes one `-`.
pub fn slugify(text: &str, unicode: bool) -> String {
    let folded: String = if unicode {
        text.to_string()
    } else {
        text.nfkd().filter(char::is_ascii).collect()
    };
    let kept: String = folded
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || c.is_whitespace())
        .collect();
    let lower = kept.trim().to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut in_run = false;
    for c in lower.chars() {
        if c == '-' || c.is_whitespace() {
            if !in_run {
                out.push('-');
            }
            in_run = true;
        } else {
            out.push(c);
            in_run = false;
        }
    }
    out
}

/// Python-Markdown's `unique`: a repeated anchor gets `_1`, `_2`, ...
fn unique(id: String, used: &mut HashSet<String>) -> String {
    let mut id = id;
    while id.is_empty() || used.contains(&id) {
        id = match id.rsplit_once('_') {
            Some((base, n)) if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => {
                format!("{base}_{}", n.parse::<u64>().unwrap_or(0) + 1)
            }
            _ => format!("{id}_1"),
        };
    }
    used.insert(id.clone());
    id
}

/// `<a id="help-ID"></a>` on its own: the only HTML a guide may contain.
fn help_anchor(html: &str) -> Option<&str> {
    let id = html
        .trim()
        .strip_prefix("<a id=\"help-")?
        .strip_suffix("\"></a>")?;
    id_ok(id).then_some(id)
}

/// An opening `<a id="help-...">` and the `</a>` right after it, as one piece.
fn joined<'a>(h: CowStr<'a>, rest: &mut std::iter::Peekable<Parser<'a>>) -> CowStr<'a> {
    let opens = h.trim().starts_with("<a id=\"help-") && !h.contains("</a>");
    if opens && matches!(rest.peek(), Some(Event::InlineHtml(c)) if c.trim() == "</a>") {
        rest.next();
        return format!("{h}</a>").into();
    }
    h
}

/// One section of a rendered guide: from the `<a id="ANCHOR"></a>` through its
/// heading and everything under it, up to the next heading of the same or a
/// higher level (so a `##` section includes its `###` parts). Empty when the
/// anchor is not in the page.
pub fn section_html(html: &str, anchor: &str) -> String {
    let marker = format!("<a id=\"{anchor}\"></a>");
    let Some(at) = html.find(&marker) else {
        return String::new();
    };
    let rest = &html[at + marker.len()..];
    let level_at = |s: &str| -> Option<(usize, u32)> {
        let mut from = 0;
        while let Some(i) = s[from..].find("<h") {
            let p = from + i;
            if let Some(d) = s[p + 2..].chars().next().and_then(|c| c.to_digit(10)) {
                return Some((p, d));
            }
            from = p + 2;
        }
        None
    };
    let Some((start, level)) = level_at(rest) else {
        return String::new();
    };
    let mut end = rest.len();
    let mut from = start + 3;
    while let Some((p, l)) = level_at(&rest[from..]).map(|(p, l)| (from + p, l)) {
        if l <= level {
            end = p;
            break;
        }
        from = p + 3;
    }
    format!("{marker}{}", &rest[..end])
}

// ---------------------------------------------------------------- links

/// Join a relative link to the folder the guide is in (`""` or `"ko"`).
/// `None` when it climbs out of START-HERE.
fn resolve(base: &str, rel: &str) -> Option<String> {
    let mut parts: Vec<&str> = if base.is_empty() {
        Vec::new()
    } else {
        vec![base]
    };
    for seg in rel.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

/// Where a link in a guide should go on the site, or `None` when it points
/// at something the help pages do not show (the projects, the PC's own
/// files): those become plain text.
fn rewrite_href(url: &str, lang: Lang) -> Option<String> {
    if url.starts_with("https://") || url.starts_with("http://") || url.starts_with("mailto:") {
        return Some(url.to_string());
    }
    if let Some(frag) = url.strip_prefix('#') {
        return Some(format!("#{}", fix_frag(frag, lang)));
    }
    let (path, frag) = url
        .split_once('#')
        .map_or((url, None), |(p, f)| (p, Some(f)));
    let base = if lang == Lang::Ko { "ko" } else { "" };
    let target = resolve(base, path)?;
    let (to_lang, stem) = match target.strip_prefix("ko/") {
        Some(rest) => (Lang::Ko, rest),
        None => (Lang::En, target.as_str()),
    };
    let stem = stem.strip_suffix(".md")?;
    if !is_guide(stem) {
        return None;
    }
    let frag = frag
        .map(|f| format!("#{}", fix_frag(f, to_lang)))
        .unwrap_or_default();
    Some(format!("/help?guide={stem}{}{frag}", to_lang.amp()))
}

/// In a Korean page the help anchors are `help-ID-ko` (as in the handbook).
fn fix_frag(frag: &str, lang: Lang) -> String {
    match frag.strip_prefix("help-") {
        Some(id) if lang == Lang::Ko && id_ok(id) && !id.ends_with("-ko") => {
            format!("help-{id}-ko")
        }
        _ => frag.to_string(),
    }
}

/// A picture's address on the site, or `None` (then its words are shown).
fn rewrite_src(url: &str, lang: Lang) -> Option<String> {
    let base = if lang == Lang::Ko { "ko" } else { "" };
    let name = resolve(base, url)?.strip_prefix("images/")?.to_string();
    image_name_ok(&name).then(|| format!("/help/images/{name}"))
}

// ---------------------------------------------------------------- rendering

/// A guide as HTML, with the list of its sections for "On this page".
pub struct Rendered {
    pub html: String,
    /// (anchor, words) of every `##` heading.
    pub sections: Vec<(String, String)>,
}

/// Turn one guide into HTML. The title line (`# ...`) is left out: the page
/// shows the title itself, and the handbook leaves it out too, so the
/// anchors match.
pub fn render(md: &str, lang: Lang) -> Rendered {
    let body = if md.starts_with("# ") {
        md.split_once('\n').map_or("", |(_, b)| b)
    } else {
        md
    };
    let mut used = HashSet::new();
    let mut sections = Vec::new();
    let mut out: Vec<Event> = Vec::new();
    // Links and pictures that are not shown as such: their end must match.
    let mut link_kept: Vec<bool> = Vec::new();
    let mut image_kept: Vec<bool> = Vec::new();
    // While inside a heading: its level, its events, its words.
    let mut heading: Option<(HeadingLevel, Vec<Event>, String)> = None;

    let mut events = Parser::new_ext(body, Options::ENABLE_TABLES).peekable();
    while let Some(ev) = events.next() {
        let ev = match ev {
            Event::Start(Tag::Heading { level, .. }) => {
                heading = Some((level, Vec::new(), String::new()));
                continue;
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, inner, words)) = heading.take() {
                    let id = unique(slugify(&words, lang == Lang::Ko), &mut used);
                    if level == HeadingLevel::H2 {
                        sections.push((id.clone(), words.trim().to_string()));
                    }
                    out.push(Event::Start(Tag::Heading {
                        level,
                        id: Some(id.into()),
                        classes: Vec::new(),
                        attrs: Vec::new(),
                    }));
                    out.extend(inner);
                    out.push(Event::End(TagEnd::Heading(level)));
                }
                continue;
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let new = if link_type == LinkType::Email {
                    Some(dest_url.to_string())
                } else {
                    rewrite_href(&dest_url, lang)
                };
                link_kept.push(new.is_some());
                match new {
                    Some(u) => Event::Start(Tag::Link {
                        link_type,
                        dest_url: u.into(),
                        title,
                        id,
                    }),
                    None => Event::InlineHtml(CowStr::Borrowed("<span class=\"help-offsite\">")),
                }
            }
            Event::End(TagEnd::Link) => {
                if link_kept.pop().unwrap_or(true) {
                    Event::End(TagEnd::Link)
                } else {
                    Event::InlineHtml(CowStr::Borrowed("</span>"))
                }
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let new = rewrite_src(&dest_url, lang);
                image_kept.push(new.is_some());
                match new {
                    Some(u) => Event::Start(Tag::Image {
                        link_type,
                        dest_url: u.into(),
                        title,
                        id,
                    }),
                    None => Event::InlineHtml(CowStr::Borrowed("<span class=\"help-noimg\">")),
                }
            }
            Event::End(TagEnd::Image) => {
                if image_kept.pop().unwrap_or(true) {
                    Event::End(TagEnd::Image)
                } else {
                    Event::InlineHtml(CowStr::Borrowed("</span>"))
                }
            }
            // HTML written in a guide: only the help anchors pass; anything else
            // is shown as the text it is, never run by the browser.
            // `<a id="help-ID"></a>` on a line of its own is a paragraph to
            // CommonMark, and comes as two pieces: `<a id="help-ID">`, `</a>`.
            Event::Html(h) | Event::InlineHtml(h) => {
                let h = joined(h, &mut events);
                match help_anchor(&h) {
                    Some(id) => {
                        let suffix = if lang == Lang::Ko { "-ko" } else { "" };
                        Event::Html(format!("<a id=\"help-{id}{suffix}\"></a>").into())
                    }
                    None => Event::Text(h),
                }
            }
            other => other,
        };
        match heading.as_mut() {
            Some((_, inner, words)) => {
                if let Event::Text(t) | Event::Code(t) = &ev {
                    words.push_str(t);
                }
                inner.push(ev);
            }
            None => out.push(ev),
        }
    }
    let mut html_out = String::with_capacity(body.len() * 3 / 2);
    html::push_html(&mut html_out, out.into_iter());
    Rendered {
        html: html_out,
        sections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_follow_python_markdown() {
        assert_eq!(
            slugify(
                "Part 9: Your web name at No-IP, and two checks (champagne)",
                false
            ),
            "part-9-your-web-name-at-no-ip-and-two-checks-champagne"
        );
        assert_eq!(
            slugify("L) Language: English / 한국어 / both", false),
            "l-language-english-both"
        );
        assert_eq!(slugify("Café — résumé", false), "cafe-resume");
        assert_eq!(
            slugify(
                "9 부분: No-IP의 내 웹 이름, 그리고 확인 두 가지(샴페인)",
                true
            ),
            "9-부분-no-ip의-내-웹-이름-그리고-확인-두-가지샴페인"
        );
        let mut used = HashSet::new();
        assert_eq!(unique("a".into(), &mut used), "a");
        assert_eq!(unique("a".into(), &mut used), "a_1");
        assert_eq!(unique("a".into(), &mut used), "a_2");
    }

    #[test]
    fn only_listed_guides_and_plain_picture_names() {
        let dir = std::env::temp_dir().join(format!("jg-guides-{}", std::process::id())); // nosemgrep: rust.lang.security.temp-dir.temp-dir (test only: a per-process throwaway folder, or a path that is never created)
        std::fs::create_dir_all(dir.join("ko")).unwrap();
        std::fs::create_dir_all(dir.join("images")).unwrap();
        std::fs::write(dir.join("02-docker.md"), "# 02. Docker\n").unwrap();
        std::fs::write(dir.join("MY-WINS.md"), "# private\n").unwrap();
        std::fs::write(dir.join("images/a.png"), b"x").unwrap();
        assert!(guide_file(&dir, "02-docker", Lang::En).is_some());
        assert!(
            guide_file(&dir, "02-docker", Lang::Ko).is_none(),
            "no Korean file"
        );
        for bad in [
            "MY-WINS",
            "../02-docker",
            "02-docker.md",
            "ko/02-docker",
            "..%2F02-docker",
            "/etc/passwd",
            "",
            "help-ids.tsv",
        ] {
            assert!(
                guide_file(&dir, bad, Lang::En).is_none(),
                "{bad} must not open"
            );
        }
        assert!(image_file(&dir, "a.png").is_some());
        for bad in [
            "../02-docker.md",
            "..",
            ".hidden.png",
            "a.png/..",
            "sub/a.png",
            "a.md",
            "a.PNG.txt",
            "%2e%2e",
        ] {
            assert!(image_file(&dir, bad).is_none(), "{bad} must not open");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn registry_rows_are_read_and_bad_rows_skipped() {
        let t = "# note\nid\tguide\tanchor_en\tanchor_ko\ttags_en\ttags_ko\tbrief_en\tbrief_ko\twizard\n\
                 level\t23-full-setup\teasy\t쉽게\t#easy #level\t#쉽게 #단계\tEasy or custom.\t쉽게 또는 직접.\tlevel\n\
                 Bad_ID\t23-full-setup\ta\tb\t#a #b\t#a #b\tx\ty\t\n\
                 other\tMY-WINS\ta\tb\t#a #b\t#a #b\tx\ty\t\n";
        let r = parse_registry(t);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].brief(Lang::Ko), "쉽게 또는 직접.");
        assert_eq!(r[0].anchor(Lang::Ko), "help-level-ko");
        assert_eq!(r[0].anchor(Lang::En), "help-level");
    }

    #[test]
    fn korean_guide_renders_with_its_anchors_and_links() {
        let md = "# 03. 정원 실행하기\n\n> 영어 원문: [../03-run-your-garden.md](../03-run-your-garden.md)\n\n\
                  <a id=\"help-summary\"></a>\n\n## 요약, 설치, 끝\n\n자세한 것은 [백업](13-backups-and-folders.md#help-backup-schedule)과 \
                  [위](#help-summary). ![마법사](../images/wizard-1-welcome.png) <script>alert(1)</script> [p](../projects/P01.md)\n";
        assert_eq!(title_of(md), "정원 실행하기");
        let r = render(md, Lang::Ko);
        assert!(
            r.html.contains("<a id=\"help-summary-ko\"></a>"),
            "{}",
            r.html
        );
        assert!(
            r.html
                .contains("<h2 id=\"요약-설치-끝\">요약, 설치, 끝</h2>"),
            "{}",
            r.html
        );
        assert_eq!(
            r.sections,
            vec![("요약-설치-끝".to_string(), "요약, 설치, 끝".to_string())]
        );
        assert!(
            r.html.contains("href=\"/help?guide=03-run-your-garden\""),
            "the English original"
        );
        assert!(
            r.html.contains(
                "href=\"/help?guide=13-backups-and-folders&amp;lang=ko#help-backup-schedule-ko\""
            ),
            "{}",
            r.html
        );
        assert!(r.html.contains("href=\"#help-summary-ko\""));
        assert!(r.html.contains("src=\"/help/images/wizard-1-welcome.png\""));
        assert!(
            r.html.contains("&lt;script&gt;"),
            "HTML in a guide is shown, not run"
        );
        assert!(!r.html.contains("<script>"));
        assert!(r.html.contains("<span class=\"help-offsite\">p</span>"));
    }

    #[test]
    fn a_section_runs_to_the_next_heading_of_its_level() {
        let h = "<h2 id=\"a\">A</h2><p>x</p><a id=\"help-b\"></a>
<h2 id=\"b\">B</h2><p>y</p><h3 id=\"c\">C</h3><p>z</p><h2 id=\"d\">D</h2><p>w</p>";
        let s = section_html(h, "help-b");
        assert!(s.contains("<p>y</p>") && s.contains("<h3 id=\"c\">") && s.contains("<p>z</p>"));
        assert!(!s.contains("id=\"d\"") && !s.contains("<p>x</p>"));
        assert_eq!(section_html(h, "help-none"), "");
    }

    #[test]
    fn summary_is_the_first_quick_point() {
        let md = "# 01. X\n\n## Quick version\n*No time?*\n\n- Install **Ubuntu** with `Rufus`, see [B](#b).\n- Two\n";
        assert_eq!(summary_of(md), "Install Ubuntu with Rufus, see B.");
        assert_eq!(
            summary_of("# T\n\n> note\n\nFirst words here.\n"),
            "First words here."
        );
    }
}
