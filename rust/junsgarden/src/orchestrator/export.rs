//! Downloads: the printable plant sheet (`html_export.py`) and the
//! "missing fields" JSON from the Partials tab.
//!
//! HOW TO READ THIS FILE (for Jun). `plant_html` builds a self-contained
//! web page for one plant, laid out in the same 12 sections as the editor,
//! with a "Print / Save as PDF" button; open it in a browser and print.
//! When it shows an AI result, filled fields are green and proposed changes
//! amber, exactly like the Streamlit export. `missing_json` is the JSON stub
//! Partials offers ("_missing_fields" first, then every column).
//!
//! WHERE IT IS USED. GET /streamlit/?export=html&kind=partial|manual&id=N (the Partials and Manual
//! Editor download buttons) and ?export=missing_json&id=N call these through `download` in mod.rs.
//! The AI tabs call `plant_html` too, with the AI result as the shown data.
//!
//! RUST WORDS USED HERE. `Option<&Value>` is "a JSON value, or nothing" and the `&` means it is
//! only borrowed. `&[String]` is a borrowed list of text values. `Map<String, Value>` is a JSON
//! object (one database row). `match` picks the first arm whose shape fits. A closure is a small
//! inline function written `|x| ...`.

// `Map` is a JSON object and `Value` is any JSON value.
use serde_json::{Map, Value};

// `super` means the parent module (the orchestrator folder). `text` turns a JSON value into plain
// text.
use super::opts::{self, text};
// `he` escapes text for HTML so a plant name cannot add tags to the page.
use crate::php::he;

/// Shows a value in the sheet. Empty values and placeholders such as "none", "null", "[]", twelve
/// dots and zero print as a long dash "—", like the Streamlit export did. `v` is the field value,
/// or None if the column is missing.
fn safe(v: Option<&Value>) -> String {
    // Read the value as text and remove spaces at both ends.
    let s = text(v).trim().to_string();
    // Placeholders that mean "no data". `matches!` checks the lower-case text against the list.
    if s.is_empty() || matches!(s.to_lowercase().as_str(), "none" | "null" | "[]" | "............" | "0" | "0.0" | "0.00") {
        "—".into()
    } else {
        s
    }
}

/// Turns the 12-letter month code into a range like "Mar – Aug". Only the first and last month are
/// shown, so a gap in the middle is not visible. No months gives "—".
fn months_label(bitmap: &str) -> String {
    // `parse_m` gives the month numbers (1 to 12).
    let m = opts::parse_m(bitmap);
    // `first` and `last` give the first and last number, each an Option. Matching both together as
    // a pair lets the arms cover every case.
    match (m.first(), m.last()) {
        // The list was empty: no months.
        (None, _) => "—".into(),
        // One month only. `*a - 1` reads the number and moves to a position counted from 0 in the
        // MONTHS list.
        (Some(a), Some(b)) if a == b => opts::MONTHS[(*a - 1) as usize].into(),
        // Two different months: "first – last".
        (Some(a), Some(b)) => format!("{} – {}", opts::MONTHS[(*a - 1) as usize], opts::MONTHS[(*b - 1) as usize]),
        // Cannot happen, but `match` must cover every case.
        _ => "—".into(),
    }
}

/// Shows an average cost as dollars with two decimals ("$12.50"). Zero or text that is not a number
/// gives "—".
fn cost(v: Option<&Value>) -> String {
    // `parse::<f64>()` reads text as a decimal number and gives `Ok(number)` or `Err`. The arm
    // `Ok(f) if f != 0.0` accepts only a non-zero number.
    match text(v).trim().parse::<f64>() {
        Ok(f) if f != 0.0 => format!("${f:.2}"),
        _ => "—".into(),
    }
}

/// One table row: the label on the left, the value on the right. If the field name is in
/// `new_fields` the row is green with an "AI filled" badge. If it is in `changed` the row is amber
/// and the badge shows the old value ("was: ..."), cut to 80 characters. Otherwise it is plain.
/// `key` is empty for rows that are not database fields, like the AI notes.
fn row(key: &str, label: &str, value: &str, new_fields: &[String], changed: &Map<String, Value>) -> String {
    // Pick the CSS class and badge in one step: `if / else if / else` gives back a pair, which is
    // split into two variables.
    let (class, badge) = if !key.is_empty() && new_fields.iter().any(|f| f == key) {
        ("row-new", " <span class=\"badge badge-new\">✨ AI filled</span>".to_string())
    // The field has an old value in `changed`. `filter` makes this ignore an empty key.
    } else if let Some(old) = changed.get(key).filter(|_| !key.is_empty()) {
        // The old value, as text, limited to its first 80 characters (`take(80)`).
        let o: String = text(Some(old)).chars().take(80).collect();
        ("row-changed", format!(" <span class=\"badge badge-changed\">🔄 Changed &nbsp;·&nbsp; was: {}</span>", he(&o)))
    } else {
        ("", String::new())
    };
    format!("\n        <tr class=\"{class}\">\n            <td class=\"field-label\">{}</td>\n            <td class=\"field-value\">{}{badge}</td>\n        </tr>", he(label), he(value))
}

/// One numbered box of the sheet: a heading and a two-column table. `title` is already HTML (it
/// contains &amp; codes), `rows` are the finished row strings. The columns are 28% and 72% wide.
fn section(title: &str, rows: Vec<String>) -> String {
    format!("\n    <div class=\"section\">\n        <h2>{title}</h2>\n        <table>\n            <colgroup>\n                <col style=\"width:28%\">\n                <col style=\"width:72%\">\n            </colgroup>\n            {}\n        </table>\n    </div>", rows.join("\n"))
}

/// The sheet's own CSS, put inside the downloaded page so it works alone, with no link to the site.
/// The r#"..."# marks are a "raw string": quotes and backslashes inside need no escaping. Colours:
/// dark green header, green rows for AI-filled fields, amber rows for changed fields. The `@media
/// print` part hides the print button and tightens spacing when the page is printed.
const STYLE: &str = r#"
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: "Segoe UI", Arial, sans-serif; font-size: 13px; color: #212529; background: #f8f9fa; padding: 24px; }
  .header { background: #1B4332; color: white; border-radius: 8px; padding: 18px 24px 14px; margin-bottom: 12px; }
  .header h1 { font-size: 22px; font-weight: 700; margin-bottom: 4px; }
  .header .meta { font-size: 11px; color: #B7E4C7; }
  .draft-banner { background: #FFF9C4; border: 1px solid #FFEEBA; border-radius: 6px; padding: 10px 16px; margin-bottom: 10px; font-size: 12px; color: #856404; line-height: 1.6; }
  .legend { display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 14px; font-size: 11px; }
  .leg-item { padding: 4px 10px; border-radius: 4px; border: 1px solid #dee2e6; }
  .leg-new { background: #D4EDDA; color: #155724; }
  .leg-changed { background: #FFF3CD; color: #856404; }
  .leg-plain { background: white; color: #495057; }
  .print-btn { display: inline-block; margin-bottom: 16px; padding: 8px 20px; background: #2D6A4F; color: white; border: none; border-radius: 6px; font-size: 13px; cursor: pointer; font-weight: 600; letter-spacing: 0.3px; }
  .print-btn:hover { background: #1B4332; }
  .section { background: white; border: 1px solid #D8F3DC; border-radius: 6px; margin-bottom: 14px; overflow: hidden; page-break-inside: avoid; }
  .section h2 { background: #D8F3DC; color: #1B4332; font-size: 12px; font-weight: 700; padding: 7px 14px; border-bottom: 1px solid #B7E4C7; letter-spacing: 0.3px; }
  table { width: 100%; border-collapse: collapse; }
  td { padding: 6px 14px; border-bottom: 1px solid #f0f0f0; vertical-align: top; line-height: 1.5; }
  .field-label { font-weight: 600; color: #2D6A4F; white-space: nowrap; font-size: 11.5px; }
  .field-value { color: #212529; font-size: 12.5px; word-break: break-word; }
  .row-new td { background: #D4EDDA; }
  .row-changed td { background: #FFF3CD; }
  .badge { display: inline-block; font-size: 10px; padding: 1px 7px; border-radius: 12px; margin-left: 6px; font-weight: 600; vertical-align: middle; }
  .badge-new { background: #C3E6CB; color: #155724; }
  .badge-changed { background: #FFE5A0; color: #856404; }
  .footer { margin-top: 16px; font-size: 10px; color: #6C757D; text-align: center; border-top: 1px solid #dee2e6; padding-top: 10px; }
  @media print {
    body { background: white; padding: 8px; font-size: 11px; }
    .print-btn { display: none; }
    .header { border-radius: 0; }
    .section { border-radius: 0; page-break-inside: avoid; }
    .section h2 { font-size: 11px; }
    td { padding: 4px 10px; font-size: 11px; }
    .field-value { font-size: 11px; }
    .badge { font-size: 9px; }
  }
"#;

/// `generate_plant_html`. `d` is what is shown (a row or an AI result), `o`
/// the database row for the fields AI may not touch (name, beds, quantity,
/// years). Months are read from the stored 12-letter codes.
/// `d` is the data shown (a database row or an AI result). `o` is the original database row, used
/// for fields the AI may not touch (name, beds, quantity, years). `title_suffix` goes in the title
/// line. `new_fields` are the fields the AI filled; `changed` maps field name to its old value.
/// Returns the whole web page as one String.
pub fn plant_html(d: &Map<String, Value>, o: &Map<String, Value>, title_suffix: &str, new_fields: &[String], changed: &Map<String, Value>) -> String {
    // Two small helper closures. `r` makes one row with the AI highlighting rules. `ds` reads a
    // field from the shown data and cleans it with `safe`.
    let r = |k: &str, l: &str, v: String| row(k, l, &v, new_fields, changed);
    let ds = |k: &str| safe(d.get(k));
    // The plant name comes from the original row, or from the shown data if it is missing there.
    let name = safe(o.get("plant_name").or_else(|| d.get("plant_name")));
    // The time stamp for the page header: month, day, year and a 12-hour clock, in the site's time
    // zone.
    let now = {
        let n = common::now_local();
        // Convert the 24-hour clock to 12-hour: hour 0 becomes 12.
        let h12 = match n.hour() % 12 { 0 => 12, h => h };
        format!("{} {:02}, {}  {:02}:{:02} {}", n.month(), n.day(), n.year(), h12, n.minute(), if n.hour() < 12 { "AM" } else { "PM" })
    };
    // If any field was filled or changed, the sheet is an AI draft and gets a warning banner, a
    // colour legend and an AI metadata box.
    let has_ai = !new_fields.is_empty() || !changed.is_empty();
    // The 12 sections, in the same order as the editor. Each `r(key, label, value)` is one row.
    let sections = [
        section("🧬 Section 1: Taxonomy &amp; Core Identity", vec![
            r("plant_name", "Common Name", safe(o.get("plant_name"))), r("botanical_name", "Botanical Name", ds("botanical_name")),
            r("family", "Family", ds("family")), r("genus", "Genus", ds("genus")), r("species", "Species", ds("species")),
            r("plant_type", "Plant Type", ds("plant_type")), r("native_status", "Native Status", ds("native_status")), r("company", "Company / Brand", ds("company")),
        ]),
        section("📏 Section 2: Growth Metrics, Spacing &amp; Beds", vec![
            r("plant_length", "Target Height / Length", ds("plant_length")), r("plant_width", "Target Width / Spread", ds("plant_width")),
            r("spacing", "Spacing", ds("spacing")), r("zones", "Hardiness Zones", ds("zones")),
            r("growth_rate", "Growth Rate", ds("growth_rate")), r("life_expectancy", "Life Expectancy", ds("life_expectancy")),
        ]),
        section("🌦️ Section 3: Microclimatic Profiles &amp; Soil Requirements", vec![
            r("sun_exposure", "Sun Exposure", ds("sun_exposure")), r("water_needs", "Water Needs", ds("water_needs")),
            r("soil_ph", "Soil pH", ds("soil_ph")), r("soil_type", "Soil Type", ds("soil_type")),
            r("planting_depth", "Planting Depth", ds("planting_depth")), r("chilling_hours", "Chilling Hours", ds("chilling_hours")),
        ]),
        section("💵 Section 4: Investment &amp; Procurement", vec![
            r("average_cost", "Avg. 1-Gallon Retail Cost", cost(d.get("average_cost"))),
            r("quantity", "Quantity Planted", safe(o.get("quantity"))), r("year_added", "Year(s) Added", safe(o.get("year_added"))),
        ]),
        section("📍 Section 5: Garden Bed Location", vec![r("garden_beds", "Assigned Beds", safe(o.get("garden_beds")))]),
        section("🌿 Section 6: Foliage &amp; Stem Morphologies", vec![
            r("foliage_color", "Foliage Color", ds("foliage_color")), r("stem_color", "Stem Color", ds("stem_color")),
            r("is_evergreen", "Foliage Retention", ds("is_evergreen")), r("leaf_shape", "Leaf Shape", ds("leaf_shape")),
            r("leaf_type", "Leaf Type", ds("leaf_type")), r("leaf_margin", "Leaf Margin", ds("leaf_margin")),
        ]),
        section("🌸 Section 7: Flower, Scent &amp; Reproduction", vec![
            r("bloom_color", "Bloom Color", ds("bloom_color")), r("aromatics", "Aromatics / Scent", ds("aromatics")),
            r("flower_symmetry", "Flower Symmetry", ds("flower_symmetry")), r("flower_shape", "Flower Shape", ds("flower_shape")),
            r("flower_size", "Flower Size", ds("flower_size")), r("reproduction_type", "Reproduction Type", ds("reproduction_type")),
        ]),
        section("🍒 Section 8: Fruit &amp; Berry Yields", vec![
            r("fruit_type", "Fruit Type", ds("fruit_type")), r("fruit_shape", "Fruit Shape", ds("fruit_shape")),
            r("fruit_color", "Fruit Color", ds("fruit_color")), r("fruit_size", "Fruit Size", ds("fruit_size")), r("fruit_edibility", "Edibility", ds("fruit_edibility")),
        ]),
        section("🪹 Section 9: Root Systems", vec![
            r("root_system", "Root System", ds("root_system")), r("root_morphology", "Root Morphology", ds("root_morphology")), r("root_function", "Root Function", ds("root_function")),
        ]),
        section("🛡️ Section 10: Care, Hazards, Propagation &amp; Companions", vec![
            r("attracts", "Attracts", ds("attracts")), r("deer_resistance", "Deer/Rabbit Resist.", ds("deer_resistance")),
            r("toxicity", "Toxicity", ds("toxicity")), r("invasive_local", "Local Invasive Status", ds("invasive_local")),
            r("propagation_method", "Propagation Method", ds("propagation_method")), r("cuttings", "Cuttings Notes", ds("cuttings")),
            r("companion_plants", "Companion Plants", ds("companion_plants")), r("fertilizer", "Fertilizer Regimens", ds("fertilizer")),
        ]),
        section("📅 Section 11: Growth Calendars", vec![
            r("grow_months", "Vegetative Growth Season", months_label(&text(d.get("grow_months")))),
            r("bloom_months", "Bloom / Flower Season", months_label(&text(d.get("bloom_months")))),
        ]),
        section("📝 Section 12: Maintenance Journals", vec![
            r("care_notes", "Seasonal Care & Watering", ds("care_notes")), r("maintenance_notes", "Pruning & Fertilizer", ds("maintenance_notes")),
            r("cleanup_notes", "Winterization & Cleanup", ds("cleanup_notes")), r("disease_susceptibility", "Disease & Insect Vulns.", ds("disease_susceptibility")),
            r("extra_details", "Extra / Culinary Details", ds("extra_details")),
        ]),
    ];
    // These stay empty for a plain export (no AI result).
    let mut ai_meta = String::new();
    let (mut draft, mut legend) = (String::new(), String::new());
    // Build the AI-only parts: the metadata box (counts and names of filled and changed fields, the
    // image search term, and whether the plant is flagged), the colour legend and the draft banner.
    if has_ai {
        let none = Map::new();
        let nl = if new_fields.is_empty() { "None".to_string() } else { new_fields.join(", ") };
        let cl = if changed.is_empty() { "None".to_string() } else { changed.keys().cloned().collect::<Vec<_>>().join(", ") };
        ai_meta = section("🤖 AI Session Metadata", vec![
            row("", "Fields AI Filled (was blank)", &format!("{} — {nl}", new_fields.len()), &[], &none),
            row("", "Fields AI Proposed Changes", &format!("{} — {cl}", changed.len()), &[], &none),
            row("", "Image Search Term", &ds("image_search_term"), &[], &none),
            row("", "Flagged for Review", if matches!(text(d.get("needs_review")).trim(), "1" | "true" | "True") { "Yes" } else { "No" }, &[], &none),
        ]);
        legend = "\n        <div class=\"legend\">\n            <span class=\"leg-item leg-new\">✨ Green = AI filled blank field</span>\n            <span class=\"leg-item leg-changed\">🔄 Amber = AI proposed change to existing value</span>\n            <span class=\"leg-item leg-plain\">White = unchanged / pre-existing data</span>\n        </div>".into();
        draft = "\n        <div class=\"draft-banner\">\n            ⚠️  DRAFT — PENDING YOUR VERIFICATION<br>\n            <small>Verify all highlighted fields against an external source (Google, plant databases, etc.) before committing to the database.</small>\n        </div>".into();
    }
    let suffix = he(title_suffix);
    // Put everything into the page template. The `{name}` marks are filled from the named values
    // after the template, e.g. `n =` is the plant name. The name and suffix are escaped with `he`.
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n<title>{n} — {t}</title>\n<style>{STYLE}</style>\n</head>\n<body>\n<div class=\"header\">\n  <h1>🌱 {n}</h1>\n  <div class=\"meta\">\n    {sfx}Generated: {now}\n  </div>\n</div>\n{draft}\n{legend}\n<button class=\"print-btn\" onclick=\"window.print()\">🖨️ Print / Save as PDF</button>\n{body}\n{ai_meta}\n<div class=\"footer\">\n  Jun &amp; Kona's Garden Orchestrator &nbsp;·&nbsp; {now} &nbsp;·&nbsp;\n  {end}\n</div>\n</body>\n</html>",
        n = he(&name),
        t = if title_suffix.is_empty() { "Plant Record".to_string() } else { suffix.clone() },
        sfx = if title_suffix.is_empty() { String::new() } else { format!("{suffix} &nbsp;·&nbsp; ") },
        now = he(&now),
        body = sections.join("\n"),
        end = if has_ai { "Verify all highlighted fields before committing to the database." } else { "Exported for reference." },
    )
}

/// JSON the way Python's `json.dumps(..., indent=2)` writes it: two-space
/// indent and every non-ASCII character as \uXXXX.
/// This keeps the downloaded files identical to the Streamlit ones.
pub fn python_json(v: &Value) -> String {
    // `to_string_pretty` writes JSON with indentation. If it fails, use empty text
    // (`unwrap_or_default`).
    let pretty = serde_json::to_string_pretty(v).unwrap_or_default();
    let mut out = String::with_capacity(pretty.len());
    // Go through the characters one by one.
    for c in pretty.chars() {
        // Plain characters are copied unchanged.
        if c.is_ascii() {
            out.push(c);
        } else {
            // Other characters are written as \uXXXX using UTF-16 (a character outside the basic
            // range becomes two such codes, as Python does). `{u:04x}` prints the number in hex
            // with at least 4 digits.
            let mut buf = [0u16; 2];
            for u in c.encode_utf16(&mut buf) {
                out.push_str(&format!("\\u{u:04x}"));
            }
        }
    }
    out
}

/// "📋 Export Missing Fields as JSON" (Partials).
/// `row` is the plant's database row. Returns JSON text with `_missing_fields` first, then a
/// `_note`, then every column of the row. The Partials tab offers this as a download; it can be
/// edited and brought back through the JSON Pipeline.
pub fn missing_json(row: &Map<String, Value>) -> String {
    // The names of the missing fields, as JSON text values.
    let missing: Vec<Value> = opts::missing_fields(row).into_iter().map(|f| Value::String(f.into())).collect();
    // Build the output map in order: the two `_` keys first, then the plant's columns.
    let mut m = Map::new();
    m.insert("_missing_fields".into(), Value::Array(missing));
    m.insert("_note".into(), Value::String("Fields listed in _missing_fields are blank or unrecognised. Fill/correct them, then import via JSON Pipeline → Phase 3.".into()));
    // Copy each column. `.clone()` makes an owned copy.
    for (k, v) in row {
        m.insert(k.clone(), v.clone());
    }
    python_json(&Value::Object(m))
}

/// The file name the Streamlit downloads used: spaces to _, / to -.
/// `plant` is the plant name, `suffix` the end such as "_Manual_Export.html". Spaces become "_" and
/// "/" becomes "-" so the name is safe as a file name.
pub fn download_name(plant: &str, suffix: &str) -> String {
    format!("{}{suffix}", plant.replace(' ', "_").replace('/', "-"))
}

/// Tests, built only for `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    /// Checks the Python-style JSON: two-space indent, and a curly apostrophe written as ’.
    #[test]
    fn python_style_json() {
        let v = serde_json::json!({"a": "Kona’s", "b": [1]});
        assert_eq!(python_json(&v), "{\n  \"a\": \"Kona\\u2019s\",\n  \"b\": [\n    1\n  ]\n}");
    }

    /// Checks the month range text, including one month and none.
    #[test]
    fn month_labels() {
        assert_eq!(months_label("..MAMJJA...."), "Mar – Aug");
        assert_eq!(months_label("......J....."), "Jul");
        assert_eq!(months_label(""), "—");
    }
}
