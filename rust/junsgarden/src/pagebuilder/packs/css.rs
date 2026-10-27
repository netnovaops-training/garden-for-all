//! Pack styles: an allowlist CSS reader and writer. A pack's CSS is read rule by rule; only
//! known properties with plain values survive, and the result is written out again here.
//!
//! Every selector is FENCED: it is put behind the pack part's own root class (`.pk-<pack>-<part>`,
//! which the site puts on the pack's art or on the catalog cards section), so a pack cannot
//! style the menu, the login box or the editor, only what it was given. The reader refuses
//! `url()`, `@import`, `@font-face`, escapes, `expression`, ids, attribute selectors, `html`,
//! `body` and any property that is not on the list. Keyframe names must carry the pack's prefix.

use super::{problem, Problem};

pub const MAX_BYTES: usize = 64 * 1024;
const MAX_RULES: usize = 200;
const MAX_DECLS: usize = 1200;

/// Elements a selector may name.
const ELEMENTS: &[&str] = &[
    "div",
    "span",
    "p",
    "a",
    "img",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "ul",
    "ol",
    "li",
    "section",
    "header",
    "footer",
    "article",
    "table",
    "thead",
    "tbody",
    "tr",
    "td",
    "th",
    "svg",
    "path",
    "g",
    "circle",
    "ellipse",
    "rect",
    "line",
    "polyline",
    "polygon",
    "stop",
    "strong",
    "em",
    "b",
    "i",
    "small",
    "button",
    "label",
    "figure",
    "figcaption",
    "details",
    "summary",
];

const PSEUDO: &[&str] = &[
    ":hover",
    ":focus",
    ":focus-visible",
    ":active",
    ":first-child",
    ":last-child",
    ":only-child",
    ":nth-child(odd)",
    ":nth-child(even)",
    "::before",
    "::after",
];

const PROPS: &[&str] = &[
    "display",
    "position",
    "color",
    "background",
    "background-color",
    "background-image",
    "background-size",
    "background-position",
    "background-repeat",
    "border",
    "border-top",
    "border-right",
    "border-bottom",
    "border-left",
    "border-color",
    "border-width",
    "border-style",
    "border-radius",
    "border-collapse",
    "border-spacing",
    "box-shadow",
    "text-shadow",
    "margin",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "padding",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "gap",
    "row-gap",
    "column-gap",
    "grid-template-columns",
    "grid-template-rows",
    "grid-column",
    "grid-row",
    "grid-auto-flow",
    "flex",
    "flex-direction",
    "flex-wrap",
    "flex-grow",
    "flex-shrink",
    "flex-basis",
    "justify-content",
    "justify-items",
    "align-items",
    "align-self",
    "align-content",
    "order",
    "font-size",
    "font-weight",
    "font-style",
    "line-height",
    "letter-spacing",
    "text-align",
    "text-transform",
    "text-decoration",
    "text-overflow",
    "white-space",
    "word-break",
    "opacity",
    "visibility",
    "overflow",
    "overflow-x",
    "overflow-y",
    "object-fit",
    "object-position",
    "aspect-ratio",
    "table-layout",
    "transform",
    "transform-origin",
    "transform-box",
    "transition",
    "animation",
    "animation-name",
    "animation-duration",
    "animation-delay",
    "animation-iteration-count",
    "animation-timing-function",
    "animation-direction",
    "animation-fill-mode",
    "filter",
    "fill",
    "stroke",
    "stroke-width",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
];

const FUNCTIONS: &[&str] = &[
    "rgb",
    "rgba",
    "hsl",
    "hsla",
    "calc",
    "var",
    "min",
    "max",
    "clamp",
    "translate",
    "translateX",
    "translateY",
    "scale",
    "scaleX",
    "scaleY",
    "rotate",
    "skewX",
    "skewY",
    "matrix",
    "cubic-bezier",
    "steps",
    "linear-gradient",
    "radial-gradient",
    "repeat",
    "minmax",
    "blur",
    "brightness",
    "contrast",
    "grayscale",
    "saturate",
    "drop-shadow",
];

const ANIMATION_WORDS: &[&str] = &[
    "infinite",
    "alternate",
    "alternate-reverse",
    "normal",
    "reverse",
    "forwards",
    "backwards",
    "both",
    "none",
    "running",
    "paused",
    "ease",
    "ease-in",
    "ease-out",
    "ease-in-out",
    "linear",
    "step-start",
    "step-end",
];

fn id_chars(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 60
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn refuse(en: &str, ko: &str) -> Problem {
    problem(
        format!("The styles are not allowed: {en}."),
        format!("스타일이 허용되지 않아요: {ko}."),
    )
}

/// Remove `/* ... */` comments (an unfinished one is a refusal).
fn strip_comments(s: &str) -> Result<String, Problem> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        let Some(j) = rest[i + 2..].find("*/") else {
            return Err(refuse("an unfinished comment", "끝나지 않은 주석"));
        };
        out.push(' ');
        rest = &rest[i + 2 + j + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

/// One selector (no commas), fenced behind `.scope`. None if anything in it is not allowed.
fn selector(sel: &str, scope: &str) -> Option<String> {
    let sel = sel.trim();
    if sel.is_empty() || sel.len() > 200 {
        return None;
    }
    let b = sel.as_bytes();
    let mut out = format!(".{scope}");
    let mut i = 0;
    let mut first = true;
    // `:scope` as the first compound is the fence's own element.
    if sel.starts_with(":scope") {
        i = 6;
        first = false;
    }
    while i < b.len() {
        match b[i] {
            b' ' => {
                // A run of spaces is one descendant combinator, unless an explicit one follows.
                while i < b.len() && b[i] == b' ' {
                    i += 1;
                }
                if i < b.len() && !matches!(b[i], b'>' | b'+' | b'~') {
                    out.push(' ');
                }
                first = true;
            }
            c @ (b'>' | b'+' | b'~') => {
                out.push(' ');
                out.push(c as char);
                out.push(' ');
                i += 1;
                while i < b.len() && b[i] == b' ' {
                    i += 1;
                }
                first = true;
            }
            b'.' => {
                let from = i + 1;
                i = from;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_')
                {
                    i += 1;
                }
                let class = &sel[from..i];
                if !id_chars(class) {
                    return None;
                }
                if first && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push('.');
                out.push_str(class);
                first = false;
            }
            b':' => {
                let rest = &sel[i..];
                let p = PSEUDO
                    .iter()
                    .filter(|p| rest.starts_with(**p))
                    .max_by_key(|p| p.len())?;
                // The pseudo name must end here (not `:hoverx`).
                let after = rest.as_bytes().get(p.len());
                if after.is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'-' || *c == b'(') {
                    return None;
                }
                if first && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(p);
                i += p.len();
                first = false;
            }
            c if c.is_ascii_lowercase() || c.is_ascii_digit() => {
                let from = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric()) {
                    i += 1;
                }
                let el = &sel[from..i];
                if !first || !ELEMENTS.contains(&el) {
                    return None;
                }
                if !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(el);
                first = false;
            }
            b'*' if first => {
                if !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push('*');
                i += 1;
                first = false;
            }
            _ => return None,
        }
    }
    // Do not end on a combinator.
    (!out.ends_with(' ') && !out.ends_with('>') && !out.ends_with('+') && !out.ends_with('~'))
        .then_some(out)
}

/// The words of a value other than functions (functions are checked on their own).
fn value_ok(prop: &str, v: &str, pack: &str) -> Option<String> {
    let v = v.trim();
    if v.is_empty() || v.len() > 400 {
        return None;
    }
    let lower = v.to_ascii_lowercase();
    if [
        "url",
        "expression",
        "javascript",
        "image-set",
        "attr(",
        "env(",
        "@",
        "\\",
        "<",
        ">",
        "{",
        "}",
        ";",
        "\"",
        "'",
        "!",
    ]
    .iter()
    .any(|bad| lower.contains(bad))
    {
        return None;
    }
    if !v.bytes().all(|b| {
        b.is_ascii_alphanumeric()
            || matches!(
                b,
                b' ' | b'#' | b'%' | b'.' | b',' | b'(' | b')' | b'-' | b'+' | b'*' | b'/' | b'_'
            )
    }) {
        return None;
    }
    // Every function name must be on the list; the brackets must balance.
    let mut depth = 0i32;
    let mut word = String::new();
    let mut bare = String::new();
    for c in v.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            word.push(c);
            continue;
        }
        if c == '(' {
            if !FUNCTIONS.contains(&word.as_str()) {
                return None;
            }
            word.clear();
            depth += 1;
            continue;
        }
        if !word.is_empty() && depth == 0 {
            bare.push_str(&word);
            bare.push(' ');
        }
        word.clear();
        if c == ')' {
            depth -= 1;
            if depth < 0 {
                return None;
            }
        }
    }
    if depth != 0 {
        return None;
    }
    if !word.is_empty() {
        bare.push_str(&word);
    }
    match prop {
        "display" => {
            let ok = matches!(
                lower.as_str(),
                "block" | "inline" | "inline-block" | "flex" | "inline-flex" | "grid" | "none"
            );
            ok.then_some(lower)
        }
        "position" => matches!(lower.as_str(), "static" | "relative").then_some(lower),
        "animation" | "animation-name" => {
            let ok = bare.split_whitespace().all(|w| {
                ANIMATION_WORDS.contains(&w)
                    || w.bytes().all(|b| b.is_ascii_digit() || b == b'.')
                    || w.strip_suffix("ms")
                        .or_else(|| w.strip_suffix('s'))
                        .is_some_and(|n| n.parse::<f64>().is_ok())
                    || (w.starts_with(&format!("pk-{pack}-")) && id_chars(w))
            });
            ok.then(|| v.to_string())
        }
        _ => Some(v.to_string()),
    }
}

/// `prop: value; prop: value` -> `prop:value;prop:value`.
fn declarations(body: &str, pack: &str, count: &mut usize) -> Result<String, Problem> {
    let mut out = String::new();
    for d in body.split(';') {
        let d = d.trim();
        if d.is_empty() {
            continue;
        }
        *count += 1;
        if *count > MAX_DECLS {
            return Err(refuse("too many settings", "설정이 너무 많음"));
        }
        let Some((prop, value)) = d.split_once(':') else {
            return Err(refuse("a setting without a value", "값 없는 설정"));
        };
        let prop = prop.trim();
        if !PROPS.contains(&prop) {
            return Err(refuse(
                "a setting that is not on the list",
                "목록에 없는 설정",
            ));
        }
        let (value, important) = match value.trim().strip_suffix("!important") {
            Some(v) => (v.trim(), true),
            None => (value.trim(), false),
        };
        // `!` was removed with the one allowed `!important`; value_ok refuses any other.
        let Some(v) = value_ok(prop, value, pack) else {
            return Err(refuse("a value that is not allowed", "허용되지 않는 값"));
        };
        out.push_str(prop);
        out.push(':');
        out.push_str(&v);
        if important {
            out.push_str("!important");
        }
        out.push(';');
    }
    Ok(out)
}

/// `(max-width: 600px) and (prefers-reduced-motion: reduce)` and the like.
fn media_ok(cond: &str) -> Option<String> {
    let mut parts = Vec::new();
    for p in cond.split(" and ") {
        let p = p.trim();
        let inner = p.strip_prefix('(')?.strip_suffix(')')?;
        let (k, v) = inner.split_once(':')?;
        let (k, v) = (k.trim(), v.trim());
        let ok = match k {
            "max-width" | "min-width" => v
                .strip_suffix("px")
                .is_some_and(|n| n.parse::<u32>().is_ok_and(|n| n <= 4000)),
            "prefers-reduced-motion" => matches!(v, "reduce" | "no-preference"),
            _ => false,
        };
        if !ok {
            return None;
        }
        parts.push(format!("({k}:{v})"));
    }
    (!parts.is_empty()).then(|| parts.join(" and "))
}

struct Cur<'a> {
    s: &'a str,
    at: usize,
}

impl<'a> Cur<'a> {
    fn skip_ws(&mut self) {
        while self.s[self.at..].starts_with(|c: char| c.is_whitespace()) {
            self.at += self.s[self.at..].chars().next().map_or(1, char::len_utf8);
        }
    }
    fn done(&self) -> bool {
        self.at >= self.s.len()
    }
    /// Text up to the next `{` (consumed).
    fn head(&mut self) -> Option<&'a str> {
        let rest = &self.s[self.at..];
        let i = rest.find('{')?;
        let h = &rest[..i];
        if h.contains('}') || h.contains(';') {
            return None;
        }
        self.at += i + 1;
        Some(h.trim())
    }
    /// Text up to the next `}` (consumed); no `{` allowed inside.
    fn body(&mut self) -> Option<&'a str> {
        let rest = &self.s[self.at..];
        let i = rest.find('}')?;
        let b = &rest[..i];
        if b.contains('{') {
            return None;
        }
        self.at += i + 1;
        Some(b)
    }
}

/// One plain rule: `selector, selector { decls }` after the head was read.
fn rule(
    head: &str,
    c: &mut Cur,
    scope: &str,
    pack: &str,
    decls: &mut usize,
) -> Result<String, Problem> {
    let sels: Option<Vec<String>> = head.split(',').map(|s| selector(s, scope)).collect();
    let Some(sels) = sels.filter(|s| !s.is_empty() && s.len() <= 12) else {
        return Err(refuse(
            "a selector that is not allowed",
            "허용되지 않는 선택자",
        ));
    };
    let Some(body) = c.body() else {
        return Err(refuse("a rule that does not close", "닫히지 않은 규칙"));
    };
    let d = declarations(body, pack, decls)?;
    Ok(format!("{}{{{}}}\n", sels.join(","), d))
}

/// Read and re-write a pack's CSS. `pack` is the pack's id, `scope` the root class that fences it.
pub fn sanitize(text: &str, pack: &str, scope: &str) -> Result<String, Problem> {
    if text.len() > MAX_BYTES {
        return Err(refuse("the file is too big", "파일이 너무 큼"));
    }
    let clean = strip_comments(text)?;
    let mut c = Cur { s: &clean, at: 0 };
    let mut out = String::new();
    let (mut rules, mut decls) = (0usize, 0usize);
    let prefix = format!("pk-{pack}-");
    loop {
        c.skip_ws();
        if c.done() {
            break;
        }
        rules += 1;
        if rules > MAX_RULES {
            return Err(refuse("too many rules", "규칙이 너무 많음"));
        }
        let Some(head) = c.head() else {
            return Err(refuse("a broken rule", "깨진 규칙"));
        };
        if let Some(name) = head.strip_prefix("@keyframes") {
            let name = name.trim();
            if !name.starts_with(&prefix) || !id_chars(name) {
                return Err(refuse(
                    "an animation name without the pack's prefix",
                    "묶음 접두어가 없는 움직임 이름",
                ));
            }
            out.push_str(&format!("@keyframes {name}{{"));
            loop {
                c.skip_ws();
                if c.s[c.at..].starts_with('}') {
                    c.at += 1;
                    break;
                }
                let Some(step) = c.head() else {
                    return Err(refuse("a broken animation", "깨진 움직임"));
                };
                let ok = step.split(',').all(|p| {
                    let p = p.trim();
                    p == "from"
                        || p == "to"
                        || p.strip_suffix('%').is_some_and(|n| {
                            n.parse::<f64>().is_ok_and(|x| (0.0..=100.0).contains(&x))
                        })
                });
                if !ok {
                    return Err(refuse("a bad animation step", "잘못된 움직임 단계"));
                }
                let Some(body) = c.body() else {
                    return Err(refuse(
                        "an animation step that does not close",
                        "닫히지 않은 단계",
                    ));
                };
                let d = declarations(body, pack, &mut decls)?;
                let steps: Vec<&str> = step.split(',').map(str::trim).collect();
                out.push_str(&format!("{}{{{}}}", steps.join(","), d));
            }
            out.push_str("}\n");
        } else if let Some(cond) = head.strip_prefix("@media") {
            let Some(cond) = media_ok(cond.trim()) else {
                return Err(refuse(
                    "a screen condition that is not allowed",
                    "허용되지 않는 화면 조건",
                ));
            };
            out.push_str(&format!("@media {cond}{{"));
            loop {
                c.skip_ws();
                if c.s[c.at..].starts_with('}') {
                    c.at += 1;
                    break;
                }
                let Some(h) = c.head() else {
                    return Err(refuse("a broken rule", "깨진 규칙"));
                };
                if h.starts_with('@') {
                    return Err(refuse("a rule inside a rule", "규칙 안의 규칙"));
                }
                out.push_str(&rule(h, &mut c, scope, pack, &mut decls)?);
            }
            out.push_str("}\n");
        } else if head.starts_with('@') {
            return Err(refuse(
                "an @-rule that is not allowed",
                "허용되지 않는 @ 규칙",
            ));
        } else {
            out.push_str(&rule(head, &mut c, scope, pack, &mut decls)?);
        }
    }
    Ok(out)
}

/// A colour for a theme: `#rgb`, `#rrggbb`, or `rgb(r,g,b)` with numbers.
pub fn color_ok(v: &str) -> bool {
    let v = v.trim();
    if let Some(h) = v.strip_prefix('#') {
        return matches!(h.len(), 3 | 6) && h.bytes().all(|b| b.is_ascii_hexdigit());
    }
    v.strip_prefix("rgb(")
        .and_then(|r| r.strip_suffix(')'))
        .is_some_and(|a| {
            let n: Vec<&str> = a.split(',').collect();
            n.len() == 3 && n.iter().all(|x| x.trim().parse::<u8>().is_ok())
        })
}

/// The custom properties a theme may set (the ones the site's colour themes set).
pub const THEME_VARS: &[&str] = &[
    "--pw-green",
    "--primary-green",
    "--site-body-bg",
    "--site-body-text",
    "--site-accent",
    "--site-accent-hover",
    "--site-tint",
    "--site-row-border",
    "--site-footer-bg",
    "--site-wave-back",
    "--site-wave-front",
];

/// `:root{...}` for a theme, or a refusal.
pub fn theme(vars: &std::collections::BTreeMap<String, String>) -> Result<String, Problem> {
    if vars.is_empty() || vars.len() > THEME_VARS.len() {
        return Err(refuse(
            "a theme needs some colours",
            "테마에는 색이 있어야 해요",
        ));
    }
    let mut out = String::from(":root{");
    for (k, v) in vars {
        if !THEME_VARS.contains(&k.as_str()) || !color_ok(v) {
            return Err(refuse("a colour that is not allowed", "허용되지 않는 색"));
        }
        out.push_str(&format!("{k}:{};", v.trim()));
    }
    out.push_str("}\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(s: &str) -> String {
        sanitize(s, "demo", "pk-demo-x").unwrap_or_else(|e| panic!("{}: {s}", e.en))
    }
    fn refused(s: &str) {
        assert!(
            sanitize(s, "demo", "pk-demo-x").is_err(),
            "should be refused: {s}"
        );
    }

    #[test]
    fn rules_are_fenced_behind_the_packs_own_class() {
        let out = ok("/* a table */\n.plant-grid { grid-template-columns: 1fr; gap: 6px }\n\
                      .plant-card > .svd-card, .svd-photo img:hover { display: flex; border-radius: 4px !important }\n\
                      :scope { background: linear-gradient(to right, #fff, rgba(0,0,0,.5)) }");
        assert!(out.contains(".pk-demo-x .plant-grid{grid-template-columns:1fr;gap:6px;}"));
        assert!(out.contains(".pk-demo-x .plant-card > .svd-card,.pk-demo-x .svd-photo img:hover{display:flex;border-radius:4px!important;}"));
        assert!(
            out.contains(".pk-demo-x{background:linear-gradient(to right, #fff, rgba(0,0,0,.5));}")
        );
        // Each rule is fenced exactly once, and nothing of the pack's own wording is kept.
        assert_eq!(out.matches("pk-demo-x").count(), 4);
    }

    #[test]
    fn animation_names_need_the_packs_prefix() {
        let out = ok("@keyframes pk-demo-grow { from { stroke-dashoffset: 100 } 50%, 80% { opacity: .5 } to { stroke-dashoffset: 0 } }\n\
                      .vine { animation: pk-demo-grow 3s ease-in-out infinite alternate }\n\
                      @media (prefers-reduced-motion: reduce) { .vine { animation: none } }\n\
                      @media (max-width: 600px) and (min-width: 100px) { .vine { width: 50% } }");
        assert!(out.contains("@keyframes pk-demo-grow{from{stroke-dashoffset:100;}50%,80%{opacity:.5;}to{stroke-dashoffset:0;}}"));
        assert!(out.contains("animation:pk-demo-grow 3s ease-in-out infinite alternate;"));
        assert!(out.contains("@media (prefers-reduced-motion:reduce){"));
        refused("@keyframes edgeWaveSlide { from { opacity: 0 } to { opacity: 1 } }");
        refused("@keyframes pk-other-x { from { opacity: 0 } }");
        refused(".a { animation: edgeWaveSlide 1s }");
        refused(".a { animation-name: pk-other-x }");
    }

    #[test]
    fn nothing_that_loads_runs_or_escapes_the_fence_gets_through() {
        for bad in [
            ".a { background: url(https://x.test/a.png) }",
            ".a { background-image: URL( 'x' ) }",
            ".a { background: image-set(x 1x) }",
            "@import 'x.css';",
            "@import url(x.css);",
            "@font-face { font-family: x }",
            "@charset 'x';",
            "@namespace x;",
            "@supports (display:grid) { .a { display: grid } }",
            "@media (max-width: 600px) { @media (min-width: 1px) { .a { width: 1px } } }",
            "@keyframes pk-demo-x { from { opacity: 0 } } @page { margin: 0 }",
            "html { color: red }",
            "body { display: none }",
            ":root { --pw-green: red }",
            "#menu { display: none }",
            "a[href^='x'] { color: red }",
            ".a { behavior: url(x) }",
            ".a { width: expression(alert(1)) }",
            ".a { color: red; position: fixed }",
            ".a { position: absolute }",
            ".a { content: 'x' }",
            ".a { z-index: 99999 }",
            ".a { cursor: pointer }",
            ".a { font-family: x }",
            ".a { color: \\72ed }",
            ".a { color: red !ie }",
            ".a { width: calc(100% - attr(x px)) }",
            ".a { width: env(safe-area-inset-top) }",
            ".a { color: javascript:x }",
            ".a { filter: url(#x) }",
            ".a { transform: unknownfn(1) }",
            ".a { display: table-cell }",
            ".a { margin: 1px } }",
            ".a { margin: 1px",
            ".a { margin }",
            ".a:not(.b) { margin: 1px }",
            ".a::selection { color: red }",
            "a:hoverx { color: red }",
            "script { display: none }",
            "input { width: 1px }",
            ".a >  { width: 1px }",
            "/* open",
            ".a { margin: 1px; /* x */ padding: 2px; } .a { width: 1px; } }",
        ] {
            refused(bad);
        }
    }

    #[test]
    fn size_and_count_are_limited() {
        refused(&format!(".a {{ margin: 1px }}{}", " ".repeat(70_000)));
        let many: String = (0..201)
            .map(|i| format!(".c{i} {{ margin: 1px }}\n"))
            .collect();
        refused(&many);
    }

    #[test]
    fn themes_set_only_the_listed_colours() {
        let mut v = std::collections::BTreeMap::new();
        v.insert("--pw-green".to_string(), "#2e7d32".to_string());
        v.insert(
            "--site-body-bg".to_string(),
            "rgb(255, 255, 240)".to_string(),
        );
        assert_eq!(
            theme(&v).unwrap(),
            ":root{--pw-green:#2e7d32;--site-body-bg:rgb(255, 255, 240);}\n"
        );
        for (k, val) in [
            ("--evil", "#fff"),
            ("--pw-green", "url(x)"),
            ("--pw-green", "red"),
            ("--pw-green", "#12345"),
            ("--pw-green", "rgb(1,2,300)"),
            ("color", "#fff"),
        ] {
            let mut bad = std::collections::BTreeMap::new();
            bad.insert(k.to_string(), val.to_string());
            assert!(theme(&bad).is_err(), "{k}={val}");
        }
        assert!(theme(&std::collections::BTreeMap::new()).is_err());
    }
}
