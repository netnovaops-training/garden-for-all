//! A page layout: rows of 1-4 sections, stored as JSON (DESIGN.md section 4).
//!
//! HOW TO READ THIS FILE. `Layout`/`Row`/`Cell` are the JSON shape. `page_id_ok` says which
//! page names exist. `default_layout` is each page as it was before the builder (the same
//! sections in the same order). `validate` checks a layout against the registry before it is
//! saved or drawn. `join_rows` puts the drawn sections together; a one-section row adds no
//! wrapper, which is what keeps the default pages byte-identical.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::pages;
use super::registry::{self, Kind};

/// The only layout format there is so far.
pub const SCHEMA: u32 = 1;
pub const MAX_ROWS: usize = 40;
pub const MAX_COLS: usize = 4;
/// Largest layout JSON accepted (also the editor API's body limit).
pub const MAX_JSON: usize = 64 * 1024;
/// Longest page title (menu name), in characters.
pub const MAX_TITLE: usize = 40;
/// How many copies of one pack section a page may hold (they are plain text and pictures).
pub const PACK_MAX_PER_PAGE: usize = 3;

/// A page: its sections, and (all optional) what it says about itself: titles, the top-menu
/// and banner switches, and "removed" (owner pages only). Older saved layouts have none of
/// these and still load.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub schema: u32,
    pub page: String,
    pub rows: Vec<Row>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_en: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_ko: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub menu: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
}

impl Layout {
    /// A layout with sections only (no titles, switches or removal).
    pub fn new(page: &str, rows: Vec<Row>) -> Self {
        Layout {
            schema: SCHEMA,
            page: page.to_string(),
            rows,
            title_en: None,
            title_ko: None,
            menu: None,
            hero: None,
            removed: None,
        }
    }
}

/// Sections that come from enabled feature packs (id -> names for messages). They carry no
/// settings. Plain `validate`/`sanitize` know none.
pub type Extra = HashMap<String, ExtraDef>;

#[derive(Clone, Debug, PartialEq)]
pub struct ExtraDef {
    pub name_en: String,
    pub name_ko: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub cols: Vec<Cell>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub section: String,
    #[serde(default)]
    pub settings: Map<String, Value>,
}

impl Cell {
    fn new(section: &str) -> Self {
        Cell {
            section: section.to_string(),
            settings: Map::new(),
        }
    }

    /// A whole-number setting (already validated), or its default.
    pub fn int(&self, key: &str) -> i64 {
        self.settings
            .get(key)
            .and_then(Value::as_i64)
            .or_else(|| default_of(&self.section, key).and_then(|v| v.as_i64()))
            .unwrap_or(0)
    }

    /// A text setting (already validated), or its default.
    pub fn text(&self, key: &str) -> String {
        self.settings
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| default_of(&self.section, key).and_then(|v| v.as_str().map(str::to_string)))
            .unwrap_or_default()
    }
}

fn default_of(section: &str, key: &str) -> Option<Value> {
    let st = registry::find(section)?
        .settings
        .iter()
        .find(|s| s.key == key)?;
    Some(match st.kind {
        Kind::Int { default, .. } => Value::from(default),
        Kind::Text { default, .. } => Value::from(default),
    })
}

/// The pages the builder manages: the site's own (`pages::BUILTIN`), and the owner's own
/// `p-<name>` pages (`/page.php?name=<name>`), name = 1-40 of `a-z 0-9 -`.
pub fn page_id_ok(id: &str) -> bool {
    pages::builtin(id).is_some() || custom_slug(id).is_some()
}

/// `p-roses` -> `roses`.
pub fn custom_slug(id: &str) -> Option<&str> {
    let slug = id.strip_prefix("p-")?;
    (!slug.is_empty()
        && slug.len() <= 40
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'))
    .then_some(slug)
}

/// Each page as it was before the page builder. Owner pages start empty.
pub fn default_layout(page: &str) -> Layout {
    let one = |ids: &[&str]| -> Vec<Row> {
        ids.iter()
            .map(|id| Row {
                cols: vec![Cell::new(id)],
            })
            .collect()
    };
    let rows = match page {
        "home" => one(&[
            "home.welcome",
            "home.weather",
            "home.tips",
            "home.news",
            "home.guestbook",
        ]),
        "blog" => one(&["blog.archive"]),
        "gallery" => one(&["gallery.wall"]),
        "search" => one(&["search.results"]),
        "catalog" => one(&[
            "catalog.manage",
            "catalog.stats",
            "catalog.search",
            "catalog.browse",
        ]),
        "data" => one(&["data.stats", "data.picker", "data.compare", "data.widgets"]),
        _ => Vec::new(),
    };
    Layout::new(page, rows)
}

/// Why a layout was refused, in both languages (shown in the editor).
#[derive(Debug, PartialEq)]
pub struct Invalid {
    pub en: String,
    pub ko: String,
}

fn invalid(en: impl Into<String>, ko: impl Into<String>) -> Invalid {
    Invalid {
        en: en.into(),
        ko: ko.into(),
    }
}

/// Check a layout before it is saved or previewed, and fill in missing settings with their
/// defaults. Everything not in the registry is refused.
#[cfg(test)]
pub fn validate(layout: &mut Layout, page: &str) -> Result<(), Invalid> {
    validate_with(layout, page, &Extra::new())
}

/// The page's own words (titles) must be short plain text.
fn check_titles(layout: &Layout, page: &str) -> Result<(), Invalid> {
    for t in [&layout.title_en, &layout.title_ko].into_iter().flatten() {
        let ok = t.chars().count() <= MAX_TITLE
            && t.chars().all(|c| registry::Charset::Plain.allows(c))
            && t.trim() == t;
        if !ok {
            return Err(invalid(
                format!("A page title is at most {MAX_TITLE} plain characters."),
                format!("페이지 제목은 평범한 글자로 {MAX_TITLE}자까지예요."),
            ));
        }
    }
    if layout.removed == Some(true) && custom_slug(page).is_none() {
        return Err(invalid(
            "The site's own pages cannot be removed.",
            "사이트 기본 페이지는 지울 수 없어요.",
        ));
    }
    Ok(())
}

/// `validate`, also accepting the sections that enabled feature packs bring.
pub fn validate_with(layout: &mut Layout, page: &str, extra: &Extra) -> Result<(), Invalid> {
    if layout.schema != SCHEMA {
        return Err(invalid(
            "Unknown layout format.",
            "알 수 없는 레이아웃 형식이에요.",
        ));
    }
    if layout.page != page || !page_id_ok(page) {
        return Err(invalid("Wrong page name.", "페이지 이름이 맞지 않아요."));
    }
    if layout.rows.len() > MAX_ROWS {
        return Err(invalid(
            format!("At most {MAX_ROWS} rows."),
            format!("줄은 최대 {MAX_ROWS}개예요."),
        ));
    }
    check_titles(layout, page)?;
    let mut counts: Vec<(&str, usize)> = Vec::new();
    let mut pack_counts: HashMap<String, usize> = HashMap::new();
    for row in &mut layout.rows {
        if row.cols.is_empty() || row.cols.len() > MAX_COLS {
            return Err(invalid(
                format!("A row holds 1 to {MAX_COLS} sections."),
                format!("한 줄에는 섹션이 1~{MAX_COLS}개 들어가요."),
            ));
        }
        for cell in &mut row.cols {
            if let Some(x) = extra.get(&cell.section) {
                if !cell.settings.is_empty() {
                    return Err(invalid(
                        format!("\"{}\" has no settings.", x.name_en),
                        format!("\"{}\" 섹션에는 설정이 없어요.", x.name_ko),
                    ));
                }
                let n = pack_counts.entry(cell.section.clone()).or_insert(0);
                *n += 1;
                if *n > PACK_MAX_PER_PAGE {
                    return Err(invalid(
                        format!(
                            "\"{}\" can be on a page only {PACK_MAX_PER_PAGE} times.",
                            x.name_en
                        ),
                        format!(
                            "\"{}\" 섹션은 한 페이지에 {PACK_MAX_PER_PAGE}번만 놓을 수 있어요.",
                            x.name_ko
                        ),
                    ));
                }
                continue;
            }
            let Some(def) = registry::find(&cell.section) else {
                return Err(if registry::PENDING.contains(&cell.section.as_str()) {
                    invalid(
                        format!(
                            "\"{}\" comes with the database update; it cannot be placed yet.",
                            cell.section
                        ),
                        format!(
                            "\"{}\" 섹션은 데이터베이스 업데이트와 함께 와요. 아직 놓을 수 없어요.",
                            cell.section
                        ),
                    )
                } else {
                    invalid("Unknown section.", "알 수 없는 섹션이에요.")
                });
            };
            match counts.iter_mut().find(|(id, _)| *id == def.id) {
                Some((_, n)) => *n += 1,
                None => counts.push((def.id, 1)),
            }
            if counts
                .iter()
                .any(|(id, n)| *id == def.id && *n > def.max_per_page)
            {
                return Err(invalid(
                    format!(
                        "\"{}\" can be on a page only {} time(s).",
                        def.name_en, def.max_per_page
                    ),
                    format!(
                        "\"{}\" 섹션은 한 페이지에 {}번만 놓을 수 있어요.",
                        def.name_ko, def.max_per_page
                    ),
                ));
            }
            cell.settings = clean_settings(def, &cell.settings)?;
        }
    }
    // Parts that only work together (the catalog's cards and its search, for example).
    for (id, _) in &counts {
        if let Some(need) = registry::requires(id) {
            if !counts.iter().any(|(c, _)| *c == need) {
                let name = |lang_ko: bool| {
                    registry::find(need)
                        .map_or(need, |d| if lang_ko { d.name_ko } else { d.name_en })
                };
                let me = registry::find(id);
                return Err(invalid(
                    format!(
                        "\"{}\" needs \"{}\" on the same page.",
                        me.map_or(*id, |d| d.name_en),
                        name(false)
                    ),
                    format!(
                        "\"{}\" 섹션은 같은 페이지에 \"{}\" 섹션이 있어야 해요.",
                        me.map_or(*id, |d| d.name_ko),
                        name(true)
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Only known keys, each the right type and inside its limits; missing keys get the default.
fn clean_settings(
    def: &registry::SectionDef,
    given: &Map<String, Value>,
) -> Result<Map<String, Value>, Invalid> {
    if let Some(k) = given
        .keys()
        .find(|k| !def.settings.iter().any(|s| s.key == *k))
    {
        return Err(invalid(
            format!(
                "\"{}\" has no setting called \"{}\".",
                def.name_en,
                short(k)
            ),
            format!(
                "\"{}\" 섹션에는 \"{}\" 설정이 없어요.",
                def.name_ko,
                short(k)
            ),
        ));
    }
    let mut out = Map::new();
    for st in def.settings {
        let bad = || {
            invalid(
                format!("\"{}\": \"{}\" is not allowed.", def.name_en, st.label_en),
                format!(
                    "\"{}\": \"{}\" 값이 허용되지 않아요.",
                    def.name_ko, st.label_ko
                ),
            )
        };
        let v = given.get(st.key);
        let clean = match st.kind {
            Kind::Int { min, max, default } => match v {
                None => Value::from(default),
                Some(v) => match v.as_i64() {
                    Some(n) if (min..=max).contains(&n) => Value::from(n),
                    _ => return Err(bad()),
                },
            },
            Kind::Text {
                max_len,
                charset,
                default,
            } => match v {
                None => Value::from(default),
                Some(Value::String(s))
                    if s.chars().count() <= max_len && s.chars().all(|c| charset.allows(c)) =>
                {
                    Value::from(s.as_str())
                }
                Some(_) => return Err(bad()),
            },
        };
        out.insert(st.key.to_string(), clean);
    }
    Ok(out)
}

/// A key as it may be quoted back in a message: short, and only plain characters.
fn short(k: &str) -> String {
    k.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(30)
        .collect()
}

/// A layout read back from the database, made safe to draw: cells whose section no longer
/// exists (or whose settings no longer fit) are dropped, never shown half-checked.
#[cfg(test)]
pub fn sanitize(layout: Layout, page: &str) -> Layout {
    sanitize_with(layout, page, &Extra::new())
}

/// `sanitize`, keeping the sections of enabled feature packs (a disabled pack's sections are
/// dropped from what is drawn, but stay in the saved layout, so enabling it again brings them
/// back).
pub fn sanitize_with(mut layout: Layout, page: &str, extra: &Extra) -> Layout {
    layout.page = page.to_string();
    layout.schema = SCHEMA;
    if check_titles(&layout, page).is_err() {
        layout.title_en = None;
        layout.title_ko = None;
        if custom_slug(page).is_none() {
            layout.removed = None;
        }
    }
    let mut kept = Vec::new();
    for row in layout.rows.drain(..).take(MAX_ROWS) {
        let cols: Vec<Cell> = row
            .cols
            .into_iter()
            .take(MAX_COLS)
            .filter_map(|mut c| {
                if extra.contains_key(&c.section) {
                    c.settings = Map::new();
                    return Some(c);
                }
                let def = registry::find(&c.section)?;
                c.settings = clean_settings(def, &c.settings).ok()?;
                Some(c)
            })
            .collect();
        if !cols.is_empty() {
            kept.push(Row { cols });
        }
    }
    layout.rows = kept;
    // The per-page limits: drop later copies beyond `max_per_page`.
    let mut seen: Vec<(String, usize)> = Vec::new();
    for row in &mut layout.rows {
        row.cols.retain(|c| {
            let max = registry::find(&c.section).map_or(PACK_MAX_PER_PAGE, |d| d.max_per_page);
            match seen.iter_mut().find(|(id, _)| *id == c.section) {
                Some((_, n)) if *n >= max => false,
                Some((_, n)) => {
                    *n += 1;
                    true
                }
                None => {
                    seen.push((c.section.clone(), 1));
                    true
                }
            }
        });
    }
    layout.rows.retain(|r| !r.cols.is_empty());
    layout
}

/// Put drawn sections together. `rows[i][j]` is the HTML of row i, column j. A one-section row
/// is printed as is; a wider row gets a grid wrapper (site/pagebuilder/pb.css), so the second
/// value says whether that stylesheet is needed.
pub fn join_rows(rows: Vec<Vec<String>>) -> (String, bool) {
    let mut out = String::new();
    let mut grid = false;
    for row in rows {
        if row.len() == 1 {
            out.push_str(&row[0]);
            continue;
        }
        grid = true;
        out.push_str(&format!(
            "<div class=\"pb-row pb-cols-{}\">\n",
            row.len().min(MAX_COLS)
        ));
        for cell in row {
            out.push_str("<div class=\"pb-col\">\n");
            out.push_str(&cell);
            out.push_str("\n</div>\n");
        }
        out.push_str("</div>\n");
    }
    (out, grid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(v: Value) -> Result<Layout, serde_json::Error> {
        serde_json::from_value(v)
    }

    #[test]
    fn default_layouts_validate_and_hold_the_old_page_order() {
        for page in [
            "home", "blog", "gallery", "search", "catalog", "data", "p-roses",
        ] {
            let mut l = default_layout(page);
            validate(&mut l, page).unwrap();
        }
        let ids: Vec<_> = default_layout("home")
            .rows
            .iter()
            .map(|r| r.cols[0].section.clone())
            .collect();
        assert_eq!(
            ids,
            [
                "home.welcome",
                "home.weather",
                "home.tips",
                "home.news",
                "home.guestbook"
            ]
        );
        assert_eq!(default_layout("home").rows[3].cols[0].int("count"), 3);
    }

    #[test]
    fn page_names() {
        for ok in ["home", "blog", "gallery", "p-roses", "p-a-1"] {
            assert!(page_id_ok(ok), "{ok}");
        }
        for bad in [
            "",
            "wiki",
            "p-",
            "p-Roses",
            "p-../x",
            "p-a b",
            "p-ㄱ",
            &format!("p-{}", "a".repeat(41)),
        ] {
            assert!(!page_id_ok(bad), "{bad}");
        }
    }

    #[test]
    fn refuses_what_the_registry_does_not_know() {
        let cases = [
            json!({"schema":2,"page":"home","rows":[]}),
            json!({"schema":1,"page":"blog","rows":[]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"nope"}]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"catalog.cards"}]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"home.news","settings":{"count":13}}]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"home.news","settings":{"count":"3"}}]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"home.news","settings":{"html":"x"}}]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"home.weather","settings":{"zip":"<script>"}}]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"home.weather","settings":{"zip":"123456"}}]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"home.tips"}]},{"cols":[{"section":"home.tips"}]}]}),
            json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"home.tips"},{"section":"home.news"},{"section":"home.guestbook"},{"section":"home.welcome"},{"section":"blog.archive"}]}]}),
        ];
        for c in cases {
            let mut l = parse(c.clone()).unwrap();
            assert!(validate(&mut l, "home").is_err(), "{c}");
        }
        let too_many: Vec<Value> = (0..41)
            .map(|_| json!({"cols":[{"section":"home.tips"}]}))
            .collect();
        let mut l = parse(json!({"schema":1,"page":"home","rows":too_many})).unwrap();
        assert!(validate(&mut l, "home").is_err());
        // Unknown keys anywhere are refused by the parser itself.
        assert!(parse(json!({"schema":1,"page":"home","rows":[],"x":1})).is_err());
        assert!(parse(json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"home.tips","onload":"x"}]}]})).is_err());
    }

    #[test]
    fn pending_sections_get_a_clear_message_in_both_languages() {
        let mut l =
            parse(json!({"schema":1,"page":"home","rows":[{"cols":[{"section":"data.table"}]}]}))
                .unwrap();
        let e = validate(&mut l, "home").unwrap_err();
        assert!(e.en.contains("database"));
        assert!(e.ko.contains("데이터베이스"));
    }

    #[test]
    fn accepts_a_mix_and_fills_defaults() {
        let mut l = parse(json!({"schema":1,"page":"p-mix","rows":[
            {"cols":[{"section":"home.weather","settings":{"zip":"12065"}},{"section":"admin.post_editor"}]},
            {"cols":[{"section":"home.news"}]}
        ]})).unwrap();
        validate(&mut l, "p-mix").unwrap();
        assert_eq!(l.rows[1].cols[0].settings["count"], json!(3));
        assert_eq!(l.rows[0].cols[0].text("zip"), "12065");
    }

    #[test]
    fn sanitize_drops_what_no_longer_fits() {
        let l = parse(json!({"schema":1,"page":"x","rows":[
            {"cols":[{"section":"gone.section"}]},
            {"cols":[{"section":"home.tips"},{"section":"home.news","settings":{"count":99}}]},
            {"cols":[{"section":"home.tips"}]}
        ]}))
        .unwrap();
        let s = sanitize(l, "home");
        assert_eq!(s.page, "home");
        assert_eq!(s.rows.len(), 1);
        assert_eq!(s.rows[0].cols.len(), 1);
        assert_eq!(s.rows[0].cols[0].section, "home.tips");
    }

    #[test]
    fn one_section_rows_are_printed_as_is() {
        let (html, grid) = join_rows(vec![vec!["A\n".into()], vec!["B".into()]]);
        assert_eq!(html, "A\nB");
        assert!(!grid);
        let (html, grid) = join_rows(vec![vec!["A".into(), "B".into()]]);
        assert!(grid);
        assert!(html.starts_with("<div class=\"pb-row pb-cols-2\">"));
        assert_eq!(html.matches("<div class=\"pb-col\">").count(), 2);
    }

    #[test]
    fn the_new_default_layouts_hold_each_page_in_its_old_order() {
        let ids = |p: &str| -> Vec<String> {
            default_layout(p)
                .rows
                .iter()
                .map(|r| r.cols[0].section.clone())
                .collect()
        };
        assert_eq!(ids("search"), ["search.results"]);
        assert_eq!(
            ids("catalog"),
            [
                "catalog.manage",
                "catalog.stats",
                "catalog.search",
                "catalog.browse"
            ]
        );
        assert_eq!(
            ids("data"),
            ["data.stats", "data.picker", "data.compare", "data.widgets"]
        );
    }

    #[test]
    fn parts_that_work_together_must_stay_together() {
        let mut l = parse(json!({"schema":1,"page":"catalog","rows":[
            {"cols":[{"section":"catalog.search"}]}]}))
        .unwrap();
        let e = validate(&mut l, "catalog").unwrap_err();
        assert!(e.en.contains("needs") && e.ko.contains("있어야"));
        let mut ok = parse(json!({"schema":1,"page":"p-x","rows":[
            {"cols":[{"section":"catalog.search"}]},{"cols":[{"section":"catalog.browse"}]}]}))
        .unwrap();
        validate(&mut ok, "p-x").unwrap();
        // The cards alone are fine.
        let mut alone = parse(json!({"schema":1,"page":"p-x","rows":[
            {"cols":[{"section":"catalog.browse"}]}]}))
        .unwrap();
        validate(&mut alone, "p-x").unwrap();
    }

    #[test]
    fn page_words_are_short_plain_text_and_only_owner_pages_can_be_removed() {
        let mk = |page: &str, extra: Value| {
            let mut v = json!({"schema":1,"page":page,"rows":[]});
            v.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            parse(v).unwrap()
        };
        let mut good = mk(
            "p-roses",
            json!({"title_en":"Roses","title_ko":"장미","menu":false,"hero":true}),
        );
        validate(&mut good, "p-roses").unwrap();
        for bad in [
            json!({"title_en":"<b>x</b>"}),
            json!({"title_en":"a&b"}),
            json!({"title_ko":"x".repeat(41)}),
            json!({"title_en":" padded"}),
        ] {
            let mut l = mk("p-roses", bad.clone());
            assert!(validate(&mut l, "p-roses").is_err(), "{bad}");
        }
        let mut gone = mk("home", json!({"removed":true}));
        assert!(validate(&mut gone, "home").is_err());
        let mut mine = mk("p-roses", json!({"removed":true}));
        validate(&mut mine, "p-roses").unwrap();
        // Unknown page fields are still refused.
        assert!(parse(json!({"schema":1,"page":"home","rows":[],"nav":"x"})).is_err());
    }

    #[test]
    fn a_layout_without_page_words_is_saved_as_before() {
        let l = default_layout("home");
        let text = serde_json::to_string(&l).unwrap();
        assert!(!text.contains("title") && !text.contains("menu") && !text.contains("removed"));
        // And an old saved layout (no such keys) still loads.
        let old = r#"{"schema":1,"page":"home","rows":[{"cols":[{"section":"home.tips","settings":{}}]}]}"#;
        let back: Layout = serde_json::from_str(old).unwrap();
        assert_eq!(back.menu, None);
    }

    #[test]
    fn pack_sections_are_accepted_only_when_the_pack_is_enabled() {
        let mut extra = Extra::new();
        extra.insert(
            "pack.demo.note".into(),
            ExtraDef {
                name_en: "Note".into(),
                name_ko: "메모".into(),
            },
        );
        let json =
            json!({"schema":1,"page":"p-x","rows":[{"cols":[{"section":"pack.demo.note"}]}]});
        let mut l = parse(json.clone()).unwrap();
        assert!(validate(&mut l, "p-x").is_err(), "not enabled: refused");
        let mut l = parse(json.clone()).unwrap();
        validate_with(&mut l, "p-x", &extra).unwrap();
        // Settings are refused, and four copies are too many.
        let mut s = parse(json!({"schema":1,"page":"p-x","rows":[{"cols":[{"section":"pack.demo.note","settings":{"a":1}}]}]})).unwrap();
        assert!(validate_with(&mut s, "p-x", &extra).is_err());
        let cell = json!({"cols":[{"section":"pack.demo.note"}]});
        let mut four =
            parse(json!({"schema":1,"page":"p-x","rows":[cell,cell,cell,cell]})).unwrap();
        assert!(validate_with(&mut four, "p-x", &extra).is_err());
        // Read back from the database: dropped while disabled, kept while enabled.
        let l = parse(json).unwrap();
        assert!(sanitize(l.clone(), "p-x").rows.is_empty());
        assert_eq!(sanitize_with(l, "p-x", &extra).rows.len(), 1);
    }
}
