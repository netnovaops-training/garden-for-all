//! Pack art: an allowlist SVG reader and writer. The pack's SVG text is read element by element;
//! only known drawing elements and known attributes with plain values survive, and the result is
//! written out again by this file (the pack's own bytes are never printed). Anything unknown is a
//! refusal, not a quiet cut: the owner sees "this pack was refused" rather than half an image.
//!
//! No script, no style attribute, no links (`href`), no `use`, no foreign content, no text, no
//! comments kept, no entities. Ids and classes must start with the pack's own prefix
//! (`pk-<pack>-`) so two packs, or a pack and the site, cannot collide. Colours may use
//! `var(--site-...)` so the site's colour themes still tint the art. Animation is the
//! `animate`/`animateTransform` elements with a short list of attributes, and CSS from `css.rs`.

use super::{problem, Problem};

pub const MAX_BYTES: usize = 64 * 1024;
const MAX_DEPTH: usize = 12;
const MAX_NODES: usize = 1500;

const ELEMENTS: &[&str] = &[
    "svg",
    "g",
    "path",
    "circle",
    "ellipse",
    "rect",
    "line",
    "polyline",
    "polygon",
    "defs",
    "linearGradient",
    "radialGradient",
    "stop",
    "clipPath",
    "animate",
    "animateTransform",
];

/// Attributes taken as plain checked numbers, lengths, keywords or lists.
const PLAIN: &[&str] = &[
    "viewBox",
    "preserveAspectRatio",
    "pathLength",
    "x",
    "y",
    "cx",
    "cy",
    "r",
    "rx",
    "ry",
    "x1",
    "y1",
    "x2",
    "y2",
    "width",
    "height",
    "stroke-width",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-miterlimit",
    "opacity",
    "fill-opacity",
    "stroke-opacity",
    "fill-rule",
    "clip-rule",
    "offset",
    "stop-opacity",
    "gradientUnits",
    "clipPathUnits",
    "attributeName",
    "from",
    "to",
    "by",
    "values",
    "keyTimes",
    "keySplines",
    "calcMode",
    "dur",
    "begin",
    "repeatCount",
    "additive",
    "accumulate",
    "type",
];

/// Attributes an `animate` may change (nothing that could point somewhere).
const ANIMATABLE: &[&str] = &[
    "opacity",
    "fill-opacity",
    "stroke-opacity",
    "transform",
    "d",
    "points",
    "r",
    "rx",
    "ry",
    "cx",
    "cy",
    "x",
    "y",
    "width",
    "height",
    "x1",
    "y1",
    "x2",
    "y2",
    "stroke-width",
    "stroke-dashoffset",
    "stroke-dasharray",
    "offset",
    "stop-opacity",
];

fn refuse(en: &str, ko: &str) -> Problem {
    problem(en, ko)
}

/// A value made of these characters only (numbers, units, separators, letters for keywords).
fn plain_chars(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 400
        && v.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b' ' | b'.' | b',' | b'-' | b'+' | b'%' | b';' | b'#' | b'_'
                )
        })
}

/// `d` and `points`: path commands and numbers.
fn path_chars(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 8000
        && v.bytes().all(|b| {
            b.is_ascii_digit()
                || matches!(
                    b,
                    b' ' | b'.'
                        | b','
                        | b'-'
                        | b'+'
                        | b'e'
                        | b'E'
                        | b'M'
                        | b'm'
                        | b'L'
                        | b'l'
                        | b'H'
                        | b'h'
                        | b'V'
                        | b'v'
                        | b'C'
                        | b'c'
                        | b'S'
                        | b's'
                        | b'Q'
                        | b'q'
                        | b'T'
                        | b't'
                        | b'A'
                        | b'a'
                        | b'Z'
                        | b'z'
                )
        })
}

/// `transform`: only known functions with numbers.
fn transform_ok(v: &str) -> bool {
    if v.len() > 300 || v.is_empty() {
        return false;
    }
    let mut rest = v.trim();
    while !rest.is_empty() {
        let Some(open) = rest.find('(') else {
            return false;
        };
        let name = rest[..open].trim();
        if !matches!(
            name,
            "translate" | "scale" | "rotate" | "skewX" | "skewY" | "matrix"
        ) {
            return false;
        }
        let Some(close) = rest[open..].find(')') else {
            return false;
        };
        let args = &rest[open + 1..open + close];
        if args.is_empty()
            || !args.bytes().all(|b| {
                b.is_ascii_digit() || matches!(b, b' ' | b'.' | b',' | b'-' | b'+' | b'e' | b'E')
            })
        {
            return false;
        }
        rest = rest[open + close + 1..].trim_start_matches([' ', ',']);
    }
    true
}

/// A paint: `none`, `currentColor`, `#rgb`, `#rrggbb`, `rgb(...)`, a plain colour name,
/// `var(--site-...)` or `url(#<own id>)` (gradients).
fn paint_ok(v: &str, prefix: &str) -> bool {
    let v = v.trim();
    if let Some(id) = v.strip_prefix("url(#").and_then(|r| r.strip_suffix(')')) {
        return id.starts_with(prefix) && id_chars(id);
    }
    // `var(--site-colour)` or `var(--site-colour, #123456)` (a plain colour as the fallback).
    if let Some(inner) = v.strip_prefix("var(--").and_then(|r| r.strip_suffix(')')) {
        let (name, fallback) = match inner.split_once(',') {
            Some((n, f)) => (n, Some(f.trim())),
            None => (inner, None),
        };
        return !name.is_empty()
            && name.len() <= 40
            && name.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
            && fallback
                .is_none_or(|f| !f.contains("var(") && !f.contains("url(") && paint_ok(f, prefix));
    }
    if let Some(hex) = v.strip_prefix('#') {
        return matches!(hex.len(), 3 | 4 | 6 | 8) && hex.bytes().all(|b| b.is_ascii_hexdigit());
    }
    if let Some(args) = v.strip_prefix("rgb(").or_else(|| v.strip_prefix("rgba(")) {
        return args.ends_with(')')
            && args.len() <= 40
            && args[..args.len() - 1]
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b' ' | b',' | b'.' | b'%' | b'/'));
    }
    v.len() <= 20 && v.bytes().all(|b| b.is_ascii_alphabetic())
}

fn id_chars(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 60
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// One attribute: the value to write, or None to refuse.
fn attr(el: &str, name: &str, value: &str, prefix: &str) -> Option<String> {
    match name {
        "id" => (value.starts_with(prefix) && id_chars(value)).then(|| value.to_string()),
        "class" => {
            let ok = !value.trim().is_empty()
                && value.len() <= 200
                && value
                    .split_whitespace()
                    .all(|c| c.starts_with(prefix) && id_chars(c));
            ok.then(|| value.split_whitespace().collect::<Vec<_>>().join(" "))
        }
        "d" | "points" => path_chars(value).then(|| value.to_string()),
        "transform" | "gradientTransform" => transform_ok(value).then(|| value.to_string()),
        "fill" if el == "animate" || el == "animateTransform" => {
            matches!(value, "freeze" | "remove").then(|| value.to_string())
        }
        "fill" | "stroke" | "stop-color" => {
            paint_ok(value, prefix).then(|| value.trim().to_string())
        }
        "clip-path" => (value.trim().starts_with("url(#") && paint_ok(value, prefix))
            .then(|| value.trim().to_string()),
        "xmlns" => (value == "http://www.w3.org/2000/svg").then(String::new),
        "version" => None,
        _ if PLAIN.contains(&name) => {
            let ok = plain_chars(value)
                && match name {
                    "attributeName" => ANIMATABLE.contains(&value),
                    // Times only: no event names, no ids.
                    "begin" | "dur" => value.split(';').all(|t| {
                        let t = t.trim();
                        t == "indefinite" && name == "begin"
                            || t.strip_suffix("ms")
                                .or_else(|| t.strip_suffix('s'))
                                .is_some_and(|n| {
                                    !n.is_empty()
                                        && n.parse::<f64>().is_ok_and(|x| x.is_finite() && x >= 0.0)
                                })
                    }),
                    "repeatCount" => {
                        value == "indefinite" || value.parse::<u32>().is_ok_and(|n| n <= 1000)
                    }
                    "type" => matches!(value, "translate" | "scale" | "rotate" | "skewX" | "skewY"),
                    _ => true,
                };
            ok.then(|| value.to_string())
        }
        _ => None,
    }
}

struct Reader<'a> {
    s: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn peek(&self) -> Option<u8> {
        self.s.get(self.at).copied()
    }
    fn starts(&self, p: &str) -> bool {
        self.s[self.at..].starts_with(p.as_bytes())
    }
    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
            self.at += 1;
        }
    }
    fn name(&mut self) -> Option<String> {
        let from = self.at;
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b':')
        {
            self.at += 1;
        }
        (self.at > from && self.at - from <= 30 && self.s[from].is_ascii_alphabetic())
            .then(|| String::from_utf8_lossy(&self.s[from..self.at]).into_owned())
    }
}

/// Read and re-write one SVG. `pack` is the pack's id (it fences ids and classes).
/// The result has one root `<svg>` that fills its box (`width` and `height` 100%, and
/// `preserveAspectRatio="none"` unless the art sets its own).
pub fn sanitize(text: &str, pack: &str) -> Result<String, Problem> {
    if text.len() > MAX_BYTES {
        return Err(refuse("The art file is too big.", "그림 파일이 너무 커요."));
    }
    let prefix = format!("pk-{pack}-");
    let mut r = Reader {
        s: text.as_bytes(),
        at: 0,
    };
    let mut out = String::new();
    let mut stack: Vec<String> = Vec::new();
    let mut nodes = 0usize;
    let mut roots = 0usize;
    let bad = |what: &str, ko: &str| {
        refuse(
            &format!("The art is not allowed: {what}."),
            &format!("그림이 허용되지 않아요: {ko}."),
        )
    };
    // A byte order mark is the only thing allowed before the first tag.
    if r.starts("\u{feff}") {
        r.at += 3;
    }
    loop {
        // Text between tags: white space only.
        while r.peek().is_some_and(|b| b != b'<') {
            if !r.peek().is_some_and(|b| b.is_ascii_whitespace()) {
                return Err(bad("text inside the art", "그림 안의 글"));
            }
            r.at += 1;
        }
        if r.peek().is_none() {
            break;
        }
        if r.starts("<!--") {
            let Some(end) = text[r.at + 4..].find("-->") else {
                return Err(bad("an unfinished comment", "끝나지 않은 주석"));
            };
            r.at += 4 + end + 3;
            continue;
        }
        if r.starts("<!") || r.starts("<?") {
            return Err(bad("a declaration or instruction", "선언이나 지시문"));
        }
        r.at += 1; // '<'
        if r.peek() == Some(b'/') {
            r.at += 1;
            let name = r.name().ok_or_else(|| bad("a broken tag", "깨진 태그"))?;
            r.skip_ws();
            if r.peek() != Some(b'>') {
                return Err(bad("a broken tag", "깨진 태그"));
            }
            r.at += 1;
            if stack.pop().as_deref() != Some(name.as_str()) {
                return Err(bad("tags that do not match", "짝이 맞지 않는 태그"));
            }
            out.push_str("</");
            out.push_str(&name);
            out.push('>');
            continue;
        }
        let name = r.name().ok_or_else(|| bad("a broken tag", "깨진 태그"))?;
        if !ELEMENTS.contains(&name.as_str()) {
            return Err(refuse(
                "The art uses a part that is not allowed.",
                "그림에 허용되지 않는 부분이 들어 있어요.",
            ));
        }
        nodes += 1;
        if nodes > MAX_NODES || stack.len() >= MAX_DEPTH {
            return Err(bad("too many parts", "부분이 너무 많음"));
        }
        if stack.is_empty() {
            roots += 1;
            if roots > 1 || name != "svg" {
                return Err(bad("it must be one <svg>", "<svg> 하나여야 함"));
            }
        } else if name == "svg" {
            return Err(bad("an svg inside an svg", "svg 안의 svg"));
        }
        out.push('<');
        out.push_str(&name);
        let mut seen: Vec<String> = Vec::new();
        let mut aspect: Option<String> = None;
        let self_closed;
        loop {
            r.skip_ws();
            match r.peek() {
                Some(b'>') => {
                    r.at += 1;
                    self_closed = false;
                    break;
                }
                Some(b'/') if r.starts("/>") => {
                    r.at += 2;
                    self_closed = true;
                    break;
                }
                None => return Err(bad("an unfinished tag", "끝나지 않은 태그")),
                _ => {}
            }
            let an = r
                .name()
                .ok_or_else(|| bad("a broken attribute", "깨진 속성"))?;
            r.skip_ws();
            if r.peek() != Some(b'=') {
                return Err(bad("an attribute without a value", "값 없는 속성"));
            }
            r.at += 1;
            r.skip_ws();
            let q = r.peek().filter(|b| *b == b'"' || *b == b'\'');
            let Some(q) = q else {
                return Err(bad("an unquoted value", "따옴표 없는 값"));
            };
            r.at += 1;
            let from = r.at;
            while r.peek().is_some_and(|b| b != q) {
                if matches!(r.peek(), Some(b'<' | b'&' | b'\\')) {
                    return Err(bad("a strange character in a value", "값 안의 이상한 문자"));
                }
                r.at += 1;
            }
            if r.peek().is_none() {
                return Err(bad("an unfinished value", "끝나지 않은 값"));
            }
            // Line breaks and tabs in a long path count as spaces.
            let value = text[from..r.at].replace(['\n', '\r', '\t'], " ");
            let value = value.as_str();
            r.at += 1;
            if seen.contains(&an) {
                return Err(bad("an attribute twice", "같은 속성이 두 번"));
            }
            seen.push(an.clone());
            // On the root, size and aspect are set below.
            if name == "svg" && (an == "width" || an == "height") {
                continue;
            }
            let Some(v) = attr(&name, &an, value, &prefix) else {
                return Err(refuse(
                    "The art uses a setting that is not allowed.",
                    "그림에 허용되지 않는 설정이 들어 있어요.",
                ));
            };
            if an == "xmlns" {
                continue;
            }
            // On the root, the aspect setting is written last (always the same order).
            if name == "svg" && an == "preserveAspectRatio" {
                aspect = Some(v);
                continue;
            }
            out.push(' ');
            out.push_str(&an);
            out.push_str("=\"");
            out.push_str(&v);
            out.push('"');
        }
        if name == "svg" {
            if !seen.iter().any(|a| a == "viewBox") {
                return Err(bad("the <svg> needs a viewBox", "<svg>에 viewBox가 필요함"));
            }
            out.push_str(" width=\"100%\" height=\"100%\" preserveAspectRatio=\"");
            out.push_str(aspect.as_deref().unwrap_or("none"));
            out.push('"');
        }
        if (name == "animate" || name == "animateTransform")
            && !seen.iter().any(|a| a == "attributeName")
        {
            return Err(bad("an animation without a target", "대상 없는 움직임"));
        }
        if self_closed {
            out.push_str("/>");
        } else {
            out.push('>');
            stack.push(name);
        }
    }
    if !stack.is_empty() || roots != 1 {
        return Err(bad(
            "it must be one complete <svg>",
            "완전한 <svg> 하나여야 함",
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(s: &str) -> String {
        sanitize(s, "demo").unwrap_or_else(|e| panic!("{}: {s}", e.en))
    }

    fn refused(s: &str) {
        assert!(sanitize(s, "demo").is_err(), "should be refused: {s}");
    }

    #[test]
    fn a_plain_drawing_is_written_out_again() {
        let out = ok(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1440 60">
            <!-- a comment is dropped -->
            <defs><linearGradient id="pk-demo-g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2e7d32"/><stop offset="1" stop-color="var(--site-wave-front)"/></linearGradient></defs>
            <path class="pk-demo-vine" d="M0,30 C240,8 480,56 720,30 L1440,60 H0 Z" fill="url(#pk-demo-g)" stroke="none" stroke-width="2"/>
            <circle cx="10" cy="10" r="3" fill="rgb(10, 20, 30)" transform="translate(5 5) rotate(10)"/>
        </svg>"##,
        );
        assert!(out.starts_with("<svg viewBox=\"0 0 1440 60\" width=\"100%\" height=\"100%\" preserveAspectRatio=\"none\">"));
        assert!(!out.contains("xmlns"));
        assert!(out.contains(
            "<path class=\"pk-demo-vine\" d=\"M0,30 C240,8 480,56 720,30 L1440,60 H0 Z\""
        ));
        assert!(!out.contains("comment") && !out.contains("xmlns"));
        assert!(out.ends_with("</svg>"));
        // Writing it again changes nothing (stable).
        assert_eq!(ok(&out), out);
        // A long path split over lines is joined with spaces.
        let multi = ok("<svg viewBox=\"0 0 1 1\"><path d=\"M0,0\n\t L1,1\"/></svg>");
        assert!(multi.contains("d=\"M0,0   L1,1\""));
    }

    #[test]
    fn animation_is_allowed_only_in_its_safe_form() {
        let out = ok(
            r##"<svg viewBox="0 0 10 10"><path d="M0,0 L10,10" stroke-dasharray="20" stroke-dashoffset="20" fill="none" stroke="#000">
            <animate attributeName="stroke-dashoffset" from="20" to="0" dur="3s" begin="0s" fill="freeze" repeatCount="indefinite"/></path>
            <g><animateTransform attributeName="transform" type="rotate" from="0" to="5" dur="4s" repeatCount="3"/></g></svg>"##,
        );
        assert!(out.contains("<animate attributeName=\"stroke-dashoffset\""));
        for bad in [
            r##"<animate attributeName="href" to="javascript:alert(1)" dur="1s"/>"##,
            r##"<animate attributeName="xlink:href" to="x" dur="1s"/>"##,
            r##"<animate attributeName="opacity" dur="1s" begin="click"/>"##,
            r##"<animate attributeName="opacity" dur="1s" begin="a.end"/>"##,
            r##"<animate to="1" dur="1s"/>"##,
            r##"<set attributeName="opacity" to="1"/>"##,
            r##"<animate attributeName="opacity" dur="1s" repeatCount="999999"/>"##,
        ] {
            refused(&format!("<svg viewBox=\"0 0 1 1\"><g>{bad}</g></svg>"));
        }
    }

    #[test]
    fn everything_that_could_run_or_leave_is_refused() {
        for bad in [
            r##"<svg viewBox="0 0 1 1"><script>alert(1)</script></svg>"##,
            r##"<svg viewBox="0 0 1 1" onload="alert(1)"/>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" onclick="x"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" style="fill:red"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><a href="https://x.test"><path d="M0,0"/></a></svg>"##,
            r##"<svg viewBox="0 0 1 1"><use href="#a"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><image href="x.png"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><foreignObject><div/></foreignObject></svg>"##,
            r##"<svg viewBox="0 0 1 1"><style>*{}</style></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" fill="url(https://x.test/a.svg)"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" fill="url(#other)"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" xlink:href="x"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" fill="javascript:alert(1)"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0 alert(1)"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" transform="url(x)"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" id="menu"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" class="plant-card"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0" class="pk-other-x"/></svg>"##,
            r##"<svg viewBox="0 0 1 1">hello</svg>"##,
            r##"<svg viewBox="0 0 1 1"><![CDATA[x]]></svg>"##,
            r##"<!DOCTYPE svg><svg viewBox="0 0 1 1"/>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0&#44;0"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"/><svg viewBox="0 0 1 1"/>"##,
            r##"<svg viewBox="0 0 1 1"><svg viewBox="0 0 1 1"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0"></g></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d="M0,0"/>"##,
            r##"<g><path d="M0,0"/></g>"##,
            r##"<svg><path d="M0,0"/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d='M0,0' d='M1,1'/></svg>"##,
            r##"<svg viewBox="0 0 1 1"><path d=M0,0/></svg>"##,
            r##"<svg viewBox="0 0 1 1" xmlns="http://evil.test/"/>"##,
            r##"<SVG viewBox="0 0 1 1"/>"##,
        ] {
            refused(bad);
        }
    }

    #[test]
    fn size_and_depth_are_limited() {
        let big = format!(
            "<svg viewBox=\"0 0 1 1\"><path d=\"{}\"/></svg>",
            "M0,0 ".repeat(20_000)
        );
        refused(&big);
        let deep = format!(
            "<svg viewBox=\"0 0 1 1\">{}{}</svg>",
            "<g>".repeat(20),
            "</g>".repeat(20)
        );
        refused(&deep);
        let many = format!(
            "<svg viewBox=\"0 0 1 1\">{}</svg>",
            "<circle r=\"1\"/>".repeat(2000)
        );
        refused(&many);
    }

    #[test]
    fn the_root_always_fills_its_box() {
        let out = ok(
            r##"<svg viewBox="0 0 100 20" width="5000" height="9" preserveAspectRatio="xMidYMid slice"><rect x="0" y="0" width="10" height="10"/></svg>"##,
        );
        assert!(out.contains("width=\"100%\" height=\"100%\""));
        assert!(!out.contains("5000"));
        assert!(
            out.contains("width=\"100%\" height=\"100%\" preserveAspectRatio=\"xMidYMid slice\">")
        );
        // A rect keeps its own size.
        assert!(out.contains("<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\"/>"));
    }
}
