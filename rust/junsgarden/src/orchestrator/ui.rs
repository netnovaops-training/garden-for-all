//! Streamlit's widgets, drawn as plain HTML.
//!
//! HOW TO READ THIS FILE (for Jun). The plant admin used to be a Streamlit
//! app, where `st.text_input("Genus", ...)` draws a labelled text box. Each
//! function here is the Rust stand-in for one of those calls and returns the
//! HTML for it: `text_input("genus", "Genus", value)` draws the same labelled
//! box. The look comes from `/streamlit/st.css` (orchestrator/st.css), which
//! copies Streamlit's fonts, colours and spacing. Everything is escaped with
//! `he()`, so a plant name can never inject HTML.
//!
//! Inputs are named after the database column they edit, so a submitted
//! form maps straight onto the save.
//!
//! RUST WORDS USED HERE. Every function takes text in and returns a `String` of HTML. `&str` is a
//! borrowed piece of text that the function only reads; `String` is text the function owns and
//! hands back. `format!` builds a String, with `{}` or `{name}` marking where a value goes.
//! `Option<&str>` is "some text, or nothing". A closure is a small inline function written `|x|
//! ...`. `pub fn` means other files may call it.

// `he` is "HTML escape": it changes < > & and quotes into safe codes. Every outside value goes
// through it before it is placed in HTML.
use crate::php::he;

/// `st.title`: the big page title. Returns an `<h1>` tag.
pub fn h1(s: &str) -> String { format!("<h1>{}</h1>", he(s)) }
/// `st.header`: a section heading (`<h2>`).
pub fn header(s: &str) -> String { format!("<h2 class=\"st-h2\">{}</h2>", he(s)) }
/// `st.subheader`: a smaller heading (`<h3>`).
pub fn subheader(s: &str) -> String { format!("<h3 class=\"st-h3\">{}</h3>", he(s)) }
/// `st.markdown("#### …")`.
pub fn h4(s: &str) -> String { format!("<h4 class=\"st-h4\">{}</h4>", he(s)) }
/// `st.text`: a plain paragraph. The text is escaped, so no HTML or Markdown marks work in it.
pub fn text(s: &str) -> String { format!("<p class=\"st-text\">{}</p>", he(s)) }
/// `st.caption`: small grey text. It may hold the little Markdown that `md` understands.
pub fn caption(s: &str) -> String { format!("<p class=\"st-caption\">{}</p>", md(s)) }
/// `st.divider`: a horizontal line. `.into()` turns the fixed text into a String.
pub fn divider() -> String { "<hr class=\"st-hr\">".into() }

/// The little bit of Markdown the Streamlit app used inline: **bold**,
/// _italic_ and `code`. The text is escaped first, so this can only ever
/// produce these three tags.
pub fn md(s: &str) -> String {
    // Escape first, so nothing in the text can become a real tag. After this, only the tags this
    // function adds can appear.
    let e = he(s);
    // `out` is the finished HTML, built up piece by piece. `rest` is the part of the text not
    // handled yet.
    let mut out = String::new();
    let mut rest = e.as_str();
    // `code` first, so ** inside code stays literal.
    // Look for the next backtick. `find` gives the position, or nothing (an Option); `while let`
    // loops as long as one is found.
    while let Some(i) = rest.find('`') {
        // Split the text at that position: what came before the backtick, and the rest starting
        // with it.
        let (before, after) = rest.split_at(i);
        // The text before the backtick may have bold or italic marks; handle them.
        out.push_str(&bold_italic(before));
        // Skip the backtick itself. `&after[1..]` is a view of the text from byte 1 onward (no
        // copy).
        let after = &after[1..];
        // Look for the closing backtick.
        if let Some(j) = after.find('`') {
            // Found a pair: the text between is shown as code, with no bold or italic handling.
            out.push_str("<code>");
            out.push_str(&after[..j]);
            out.push_str("</code>");
            rest = &after[j + 1..];
        // No closing backtick: keep the backtick as plain text.
        } else {
            out.push('`');
            rest = after;
        }
    }
    // Handle the text after the last backtick.
    out.push_str(&bold_italic(rest));
    out
}

/// Turns **bold** and *italic* marks into `<strong>` and `<em>` tags. It splits the text on "**"
/// and flips between "inside bold" and "outside bold" at each mark.
fn bold_italic(s: &str) -> String {
    let mut out = String::new();
    // `split("**")` cuts the text at every double star. The pieces alternate: normal, bold, normal,
    // bold...
    let mut parts = s.split("**");
    // Tracks whether the next piece is bold.
    let mut bold = false;
    // The first piece is always outside bold.
    if let Some(first) = parts.next() {
        out.push_str(&italic(first));
    }
    // Each later piece comes right after a "**": open a tag if we were outside bold, close it if we
    // were inside, then flip.
    for p in parts {
        out.push_str(if bold { "</strong>" } else { "<strong>" });
        bold = !bold;
        out.push_str(&italic(p));
    }
    // An odd number of "**" leaves a tag open, so close it to keep the HTML valid.
    if bold {
        out.push_str("</strong>");
    }
    out
}

/// `*text*` -> <em>text</em> (a lone `*` stays as it is).
/// Takes a lone `*` pair as italic. A single unmatched star stays as a star. The text is cut at
/// each "*"; odd-numbered gaps are inside italic.
fn italic(s: &str) -> String {
    // `Vec<&str>` is a list of borrowed slices of the original text.
    let parts: Vec<&str> = s.split('*').collect();
    // Fewer than 3 pieces means there are fewer than 2 stars, so there is no pair; return the text
    // unchanged.
    if parts.len() < 3 {
        return s.to_string();
    }
    let mut out = String::new();
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            // Even positions close an italic span; odd positions open one.
            let closing = i % 2 == 0;
            // an unmatched last star is literal
            if !closing && i == parts.len() - 1 {
                out.push('*');
            } else {
                out.push_str(if closing { "</em>" } else { "<em>" });
            }
        }
        out.push_str(p);
    }
    out
}

/// `st.file_uploader`: Streamlit's drop zone, with the real file input
/// behind "Browse files".
/// `types` are the allowed extensions without a dot (["jpg", "png"]), `multiple` allows several
/// files. The real `<input type="file">` is hidden inside a label that looks like Streamlit's drop
/// box. st.js shows the chosen files.
pub fn file_uploader(name: &str, label: &str, types: &[&str], multiple: bool) -> String {
    // The browser's `accept` list needs ".jpg"; the shown list needs "JPG".
    let accept: Vec<String> = types.iter().map(|t| format!(".{t}")).collect();
    let shown: Vec<String> = types.iter().map(|t| t.to_uppercase()).collect();
    // Build the drop box. Every value is escaped with `he`. The text "Limit 200MB per file" is only
    // a label; the real limit is set elsewhere.
    format!(
        "<div class=\"st-field\"><span class=\"st-label\">{}</span><label class=\"st-drop\"><span class=\"st-drop-icon\">☁</span><span><span class=\"st-drop-t\">Drag and drop {} here</span><span class=\"st-drop-s\">Limit 200MB per file • {}</span></span><span class=\"st-btn st-drop-b\">Browse files</span><input type=\"file\" name=\"{}\" accept=\"{}\"{}></label></div>",
        he(label), if multiple { "files" } else { "file" }, he(&shown.join(", ")), he(name), he(&accept.join(",")), if multiple { " multiple" } else { "" }
    )
}

/// Markdown paragraph (`st.markdown("**x** …")`).
/// `st.markdown`: a paragraph that understands the bold, italic and code marks.
pub fn markdown(s: &str) -> String { format!("<p class=\"st-text\">{}</p>", md(s)) }

/// A coloured message box. `kind` is the CSS class suffix: success, info, warning or error.
fn alert(kind: &str, s: &str) -> String {
    format!("<div class=\"st-alert st-{kind}\">{}</div>", md(s))
}
/// `st.success`: green box.
pub fn success(s: &str) -> String { alert("success", s) }
/// `st.info`: blue box.
pub fn info(s: &str) -> String { alert("info", s) }
/// `st.warning`: yellow box.
pub fn warning(s: &str) -> String { alert("warning", s) }
/// `st.error`: red box.
pub fn error(s: &str) -> String { alert("error", s) }

/// A row of columns (`st.columns(n)` or weighted `st.columns([2,1])`).
/// The CSS grid is told the width ratio of each column (`2fr 1fr` means the first is twice as
/// wide). `weights` has one number per column and `cells` the finished HTML of each. `Vec<String>`
/// is a list of owned text values.
pub fn columns(weights: &[u32], cells: Vec<String>) -> String {
    // Turn each weight into a CSS size like "2fr".
    let tmpl: Vec<String> = weights.iter().map(|w| format!("{w}fr")).collect();
    let mut s = format!("<div class=\"st-cols\" style=\"grid-template-columns:{}\">", tmpl.join(" "));
    // Wrap each cell's HTML in its own column box.
    for c in cells {
        s.push_str("<div class=\"st-col\">");
        s.push_str(&c);
        s.push_str("</div>");
    }
    s.push_str("</div>");
    s
}

/// The common wrapper for an input: a `<label>` holding the label text, an optional "?" help badge
/// (its tooltip is the `help` text), and the input itself (`inner`). Putting the input inside the
/// label means clicking the label focuses the input.
fn field(label: &str, inner: String, help: Option<&str>) -> String {
    // `help` is an Option. `map` runs only when there is text; `unwrap_or_default` gives an empty
    // String when there is not.
    let h = help.map(|h| format!(" <span class=\"st-help\" title=\"{}\">?</span>", he(h))).unwrap_or_default();
    format!("<label class=\"st-field\"><span class=\"st-label\">{}{h}</span>{inner}</label>", he(label))
}

/// `st.text_input`: a one-line text box. `name` is the form field name (the database column),
/// `value` what it starts with.
pub fn text_input(name: &str, label: &str, value: &str, placeholder: &str) -> String {
    field(label, format!(
        "<input class=\"st-input\" type=\"text\" name=\"{}\" value=\"{}\" placeholder=\"{}\">",
        he(name), he(value), he(placeholder)), None)
}

/// `st.text_area`: a multi-line box of `height` pixels.
pub fn text_area(name: &str, label: &str, value: &str, height: u32) -> String {
    field(label, format!(
        "<textarea class=\"st-input\" name=\"{}\" style=\"height:{height}px\">{}</textarea>",
        he(name), he(value)), None)
}

/// `st.number_input`: a number box. `min` is the smallest allowed (or none), `step` the size of
/// each arrow click. `value` is kept as text, so the box can show what the database holds.
pub fn number_input(name: &str, label: &str, value: &str, min: Option<i64>, step: &str) -> String {
    let m = min.map(|m| format!(" min=\"{m}\"")).unwrap_or_default();
    field(label, format!(
        "<input class=\"st-input\" type=\"number\" step=\"{step}\" name=\"{}\" value=\"{}\"{m}>",
        he(name), he(value)), None)
}

/// `st.selectbox`. `options` are (value, shown text). With `autosubmit`,
/// choosing an option reloads the page with it, the way Streamlit reruns.
/// The `autosubmit` flag adds a small script (`onchange`) that sends the form as soon as a choice
/// is made.
pub fn selectbox(name: &str, label: &str, options: &[(String, String)], selected: &str, autosubmit: bool) -> String {
    // Start the `<select>` tag.
    let mut s = format!(
        "<select class=\"st-select\" name=\"{}\"{}>",
        he(name),
        if autosubmit { " onchange=\"this.form.submit()\"" } else { "" }
    );
    // One `<option>` per choice. The chosen one gets `selected`.
    for (v, t) in options {
        s.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            he(v),
            if v == selected { " selected" } else { "" },
            he(t)
        ));
    }
    s.push_str("</select>");
    field(label, s, None)
}

/// A selectbox whose value and shown text are the same.
/// Shortcut when each choice's value and shown text are the same. It clones the list into pairs and
/// calls `selectbox`.
pub fn select_plain(name: &str, label: &str, options: &[String], selected: &str) -> String {
    let opts: Vec<(String, String)> = options.iter().map(|o| (o.clone(), o.clone())).collect();
    selectbox(name, label, &opts, selected, false)
}

/// `st.multiselect`: chips in a box. Without JavaScript it is a plain
/// multiple select; `st.js` turns it into Streamlit's chip box.
/// `selected` is the list of choices that start picked.
pub fn multiselect(name: &str, label: &str, options: &[String], selected: &[String]) -> String {
    // `multiple` allows several picks; `data-ms` tells st.js to draw the chip box on top. The form
    // sends one value per picked option.
    let mut s = format!("<select class=\"st-multi\" multiple data-ms name=\"{}\">", he(name));
    for o in options {
        s.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            he(o),
            if selected.contains(o) { " selected" } else { "" },
            he(o)
        ));
    }
    s.push_str("</select>");
    field(label, s, None)
}

/// `st.checkbox`. The value sent when ticked is "1"; when it is not ticked, browsers send nothing
/// at all. `on` is whether it starts ticked.
pub fn checkbox(name: &str, label: &str, on: bool, help: Option<&str>) -> String {
    let h = help.map(|h| format!(" <span class=\"st-help\" title=\"{}\">?</span>", he(h))).unwrap_or_default();
    format!(
        "<label class=\"st-check\"><input type=\"checkbox\" name=\"{}\" value=\"1\"{}> {}{h}</label>",
        he(name), if on { " checked" } else { "" }, md(label)
    )
}

/// `st.toggle`: a checkbox drawn as a switch.
pub fn toggle(name: &str, label: &str, on: bool, help: Option<&str>) -> String {
    let h = help.map(|h| format!(" <span class=\"st-help\" title=\"{}\">?</span>", he(h))).unwrap_or_default();
    format!(
        "<label class=\"st-toggle\"><input type=\"checkbox\" name=\"{}\" value=\"1\"{}><span class=\"st-switch\"></span> {}{h}</label>",
        he(name), if on { " checked" } else { "" }, he(label)
    )
}

/// `st.radio`: a set of round buttons where one choice is picked. `horizontal` lays them in a row;
/// `autosubmit` sends the form when the choice changes.
pub fn radio(name: &str, label: &str, options: &[&str], selected: &str, horizontal: bool, autosubmit: bool) -> String {
    // Start the group.
    let mut s = format!("<div class=\"st-radio{}\">", if horizontal { " st-radio-h" } else { "" });
    // One radio button per option. `*o == selected` compares the option text with the chosen one.
    for o in options {
        s.push_str(&format!(
            "<label><input type=\"radio\" name=\"{}\" value=\"{}\"{}{}> {}</label>",
            he(name), he(o),
            if *o == selected { " checked" } else { "" },
            if autosubmit { " onchange=\"this.form.submit()\"" } else { "" },
            he(o)
        ));
    }
    s.push_str("</div>");
    format!("<div class=\"st-field\"><span class=\"st-label\">{}</span>{s}</div>", he(label))
}

/// A submit button. `primary` is Streamlit's red button; `name`/`value`
/// tell the handler which button was pressed.
pub fn button(name: &str, value: &str, label: &str, primary: bool, full_width: bool) -> String {
    format!(
        "<button type=\"submit\" class=\"st-btn{}{}\" name=\"{}\" value=\"{}\">{}</button>",
        if primary { " st-primary" } else { "" },
        if full_width { " st-full" } else { "" },
        he(name), he(value), md(label)
    )
}

/// `st.download_button` / `st.link_button`: a link that looks like a button.
pub fn link_button(href: &str, label: &str) -> String {
    format!("<a class=\"st-btn\" href=\"{}\">{}</a>", he(href), md(label))
}

/// `st.expander`: a box that opens and closes, built from the browser's own `<details>` tag. `open`
/// says if it starts open, `body` is its finished HTML.
pub fn expander(title: &str, open: bool, body: &str) -> String {
    format!(
        "<details class=\"st-expander\"{}><summary>{}</summary><div class=\"st-exp-body\">{body}</div></details>",
        if open { " open" } else { "" }, md(title)
    )
}

/// `st.slider` for whole numbers, with the value shown above it.
pub fn slider(name: &str, label: &str, min: i64, max: i64, value: i64) -> String {
    format!(
        "<label class=\"st-field\"><span class=\"st-label\">{}</span><output class=\"st-slider-v\">{value}</output><input class=\"st-slider\" type=\"range\" name=\"{}\" min=\"{min}\" max=\"{max}\" value=\"{value}\" oninput=\"this.previousElementSibling.value=this.value\"><span class=\"st-slider-ends\"><span>{min}</span><span>{max}</span></span></label>",
        he(label), he(name)
    )
}

/// `st.metric`: a small card with a label and a large value.
pub fn metric(label: &str, value: &str) -> String {
    format!("<div class=\"st-metric\"><div class=\"st-metric-l\">{}</div><div class=\"st-metric-v\">{}</div></div>", he(label), he(value))
}

/// `st.dataframe`: column names as given, each cell already HTML (use
/// `cell()` for text). Scrolls inside a fixed height like Streamlit's grid.
/// Builds a `<table>`. `cols` are the headers, `rows` the cells (each already HTML).
pub fn dataframe(cols: &[&str], rows: &[Vec<String>]) -> String {
    // Start with the table and header row.
    let mut s = String::from("<div class=\"st-df\"><table><thead><tr>");
    // One header cell per column name.
    for c in cols {
        s.push_str(&format!("<th>{}</th>", he(c)));
    }
    s.push_str("</tr></thead><tbody>");
    // One table row per item, one `<td>` per cell.
    for r in rows {
        s.push_str("<tr>");
        for c in r {
            s.push_str(&format!("<td>{c}</td>"));
        }
        s.push_str("</tr>");
    }
    s.push_str("</tbody></table></div>");
    s
}

/// One plain-text dataframe cell; numbers align right like Streamlit's.
pub fn cell(v: &str) -> String {
    // A cell counts as a number if it is not empty and `parse::<f64>()` can read it as a decimal
    // number (`is_ok()` is true when that worked).
    let numeric = !v.is_empty() && v.parse::<f64>().is_ok();
    if numeric {
        format!("<span class=\"st-num\">{}</span>", he(v))
    } else {
        he(v)
    }
}

/// A photo's file name as a URL path: each folder part encoded on its own
/// so "web/abc.jpg" stays a sub-folder and spaces become %20.
pub fn path_url(f: &str) -> String {
    // Split the path at "/", percent-encode each part by itself, turn "+" into "%20" (PHP's
    // urlencode writes spaces as "+", which is wrong inside a path), then join the parts with "/"
    // again.
    f.split('/').map(|part| crate::php::urlencode(part).replace('+', "%20")).collect::<Vec<_>>().join("/")
}

/// The 80px thumbnail Streamlit's tables showed, from the site's own
/// resizer (`app_api.php?action=thumb`), or nothing.
pub fn thumb(picture: &str) -> String {
    // Take the first picture in the comma list: trim spaces, skip empty items and the words none,
    // null and "[]".
    let first = picture.split(',').map(str::trim).find(|s| !s.is_empty() && !matches!(s.to_lowercase().as_str(), "none" | "null" | "[]"));
    // Three cases: a photo in a sub-folder, a plain file name, or no photo at all.
    match first {
        // The resizer reads only a file name, so a photo in a sub-folder
        // (web/...) is shown from its own address instead.
        Some(f) if f.contains('/') => format!("<img class=\"st-thumb\" loading=\"lazy\" src=\"/images/Database/{}\" alt=\"\">", path_url(f)),
        Some(f) => format!(
            "<img class=\"st-thumb\" loading=\"lazy\" src=\"/app_api.php?action=thumb&amp;img={}&amp;w=80\" alt=\"\">",
            crate::php::urlencode(f)
        ),
        None => String::new(),
    }
}

/// Tests, built only for `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    /// Checks the Markdown rules: marks work, HTML is escaped first, code keeps its stars, and a
    /// lone star stays.
    #[test]
    fn markdown_is_escaped_then_formatted() {
        assert_eq!(md("**a** <b> `c**d`"), "<strong>a</strong> &lt;b&gt; <code>c**d</code>");
        assert_eq!(md("no marks"), "no marks");
        assert_eq!(md("🗑 *Wipe File from Vol*"), "🗑 <em>Wipe File from Vol</em>");
        assert_eq!(md("a * b"), "a * b");
    }
}
