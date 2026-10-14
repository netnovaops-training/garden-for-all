//! JSON Pipeline → "📋 Field Reference": every field a plant JSON file can
//! hold, its type, its allowed values and an example, plus the VS Code JSON
//! Schema download (`?export=jp_schema`). Copied from FIELD_DEFS in
//! tab_json_pipeline.py.
//!
//! URL: /streamlit/?tab=11&sub=6 (tab 11 is "JSON Pipeline", inner tab 6).
//!
//! HOW TO READ THIS FILE. Start with `FIELD_DEFS` below: one line per plant
//! field. `render` turns that list into a filterable table plus lists of
//! the allowed values. `schema_json` turns the same list into a JSON Schema
//! file that VS Code can use to check a plant JSON file. Both functions
//! only read the list; nothing here touches the database.
//!
//! Rust idea: `use` brings a name into scope so it can be written short.
//! `super` means the parent folder module (`orchestrator`).

use serde_json::{Map, Value};

use super::opts::*;
use super::ui;
use super::Req;
use crate::php::he;

/// One row of the field reference. It is a tuple (a fixed group of values
/// that is not named): (field name, type description, allowed values if the
/// field has a drop-down list, example text).
/// `&'static str` is text that lives for the whole run of the program, such
/// as a literal typed in the code. `Option<..>` means "a value or nothing";
/// `None` means the field has no fixed list of allowed values.
type Def = (&'static str, &'static str, Option<&'static [&'static str]>, &'static str);

/// Allowed-value text for companion_plants. It is a hint, not a real list.
/// `&[&str]` is a borrowed list (a slice) of text pieces.
const COMPANIONS: &[&str] = &["any plant_name from DB"];
/// Hint text for grow_months (shown in place of a list of allowed values).
const GROW: &[&str] = &["e.g. '....MAMJJAS..'  means May–Sep active"];
/// Hint text for bloom_months (same idea as `GROW`).
const BLOOM: &[&str] = &["e.g. '......JJA...'  means Jun–Aug blooming"];

/// Every field a plant JSON file may hold, in the same order as the original
/// Streamlit FIELD_DEFS. The option lists (PLANT_TYPES and so on) come from
/// `opts.rs`, so the drop-down lists stay in one place.
/// A type text that contains "read-only" or "metadata" is shown but must
/// not be changed by hand. `schema_json` and `render` both check for those
/// words.
const FIELD_DEFS: &[Def] = &[
    ("id", "integer (read-only)", None, "42"),
    ("plant_name", "string  (read-only)", None, "Lavender"),
    ("botanical_name", "string", None, "Lavandula angustifolia"),
    ("family", "string", None, "Lamiaceae"),
    ("genus", "string", None, "Lavandula"),
    ("species", "string", None, "angustifolia"),
    ("company", "string", None, "Proven Winners"),
    ("plant_type", "string — one of list", Some(PLANT_TYPES), "Perennial"),
    ("native_status", "string — one of list", Some(NATIVE_STATUS_OPTIONS), "Native"),
    ("plant_length", "string", None, "18-24 in"),
    ("plant_width", "string", None, "12-18 in"),
    ("spacing", "string", None, "12-15 in apart"),
    ("zones", "string", None, "5-9"),
    ("life_expectancy", "string", None, "10+ years"),
    ("growth_rate", "string — one of list", Some(GROWTH_RATES), "Moderate"),
    ("sun_exposure", "string — one of list", Some(SUN_OPTIONS), "Full Sun"),
    ("water_needs", "string", None, "Low — drought tolerant once established"),
    ("soil_ph", "string", None, "6.5 – 7.5"),
    ("soil_type", "string", None, "Well-draining, sandy or loamy"),
    ("planting_depth", "string", None, "Crown at soil level"),
    ("chilling_hours", "string", None, "0  (or number of hours)"),
    ("average_cost", "number (float)", None, "8.99"),
    ("quantity", "integer (read-only)", None, "3"),
    ("year_added", "comma-separated string (read-only)", Some(YEARS_ADDED), "2024"),
    ("garden_beds", "comma-separated string (read-only)", Some(GARDEN_BEDS), "Zen Garden, River Garden"),
    ("foliage_color", "string", None, "Silver-green"),
    ("stem_color", "string", None, "Grey-green"),
    ("is_evergreen", "comma-separated string", Some(EVERGREEN_OPTIONS), "Evergreen"),
    ("leaf_shape", "string — one of list", Some(LEAF_SHAPES), "Linear (Long, narrow)"),
    ("leaf_type", "string — one of list", Some(LEAF_TYPES), "Simple (Single undivided blade)"),
    ("leaf_margin", "string — one of list", Some(LEAF_MARGINS), "Entire (Smooth, no teeth)"),
    ("bloom_color", "string", None, "Purple, Lavender"),
    ("aromatics", "string", None, "Strong floral, herbal"),
    ("flower_symmetry", "string — one of list", Some(FLOWER_SYMMETRIES), "Zygomorphic (Bilateral symmetry)"),
    ("flower_shape", "string — one of list", Some(FLOWER_SHAPES), "Tubular (Cylindrical tube)"),
    ("flower_size", "string — one of list", Some(FLOWER_SIZES), "Small (1-2 in)"),
    ("reproduction_type", "string — one of list", Some(REPRODUCTION_OPTIONS), "Hermaphrodite"),
    ("fruit_type", "string", None, "Nutlet"),
    ("fruit_shape", "string", None, "Oval"),
    ("fruit_color", "string", None, "Brown"),
    ("fruit_size", "string", None, "< 1 mm"),
    ("fruit_edibility", "string — one of list", Some(FRUIT_EDIBILITY_OPTIONS), "Inedible"),
    ("root_system", "string — one of list", Some(ROOT_SYSTEM_OPTIONS), "Fibrous/Adventitious Systems"),
    ("root_morphology", "string — one of list", Some(ROOT_MORPHOLOGY_OPTIONS), "Fine Roots (< 2 mm)"),
    ("root_function", "string — one of list", Some(ROOT_FUNCTIONAL_OPTIONS), "Absorptive Roots (Lower Orders)"),
    ("attracts", "string", None, "Bees, Butterflies, Hummingbirds"),
    ("deer_resistance", "string — one of list", Some(RESISTANCE_OPTIONS), "Deer Resistant"),
    ("toxicity", "string — one of list", Some(TOXICITY_OPTIONS), "Non-toxic"),
    ("invasive_local", "string — one of list", Some(INVASIVE_OPTIONS), "No"),
    ("propagation_method", "string", None, "Stem cuttings, division, seed"),
    ("cuttings", "string", None, "Take 4 in softwood cuttings in summer"),
    ("companion_plants", "comma-separated string", Some(COMPANIONS), "Rosemary, Sage, Thyme"),
    ("fertilizer", "comma-separated string", Some(FERTILIZER_OPTIONS), "Rose-tone, Flower-tone"),
    ("grow_months", "12-char string (J=Jan … D=Dec, dot=absent)", Some(GROW), "....MAMJJAS.."),
    ("bloom_months", "12-char string (same encoding as grow_months)", Some(BLOOM), "......JJA..."),
    ("care_notes", "string (free text)", None, "Water deeply once a week; reduce in winter."),
    ("maintenance_notes", "string (free text)", None, "Prune lightly after first bloom flush."),
    ("cleanup_notes", "string (free text)", None, "Cut back by 1/3 in early spring."),
    ("disease_susceptibility", "string (free text)", None, "Root rot in wet soils; watch for aphids."),
    ("extra_details", "string (free text)", None, "Culinary herb; edible flowers."),
    ("picture", "comma-separated filenames (read-only)", None, "Lavender_42_1717000000.jpg"),
    ("image_search_term", "string (read-only)", None, "Lavandula angustifolia purple"),
    ("approved_jason", "0 or 1 (read-only)", None, "0"),
    ("approved_jun", "0 or 1 (read-only)", None, "0"),
    ("needs_review", "0 or 1 (read-only)", None, "0"),
    ("image_verified", "0 or 1 (read-only)", None, "0"),
    ("_missing_fields", "list of strings (pipeline metadata)", None, "[\"family\", \"zones\"]"),
    ("_enriched_fields", "list of strings (pipeline metadata)", None, "[\"family\"]"),
];

/// Walks through `FIELD_DEFS` one entry at a time.
/// Returns an iterator (something you can loop over) of references to rows.
/// `impl Iterator<..>` means "some type that can be looped over"; the
/// exact type is not important to the caller.
fn defs() -> impl Iterator<Item = &'static Def> {
    FIELD_DEFS.iter()
}

/// Makes the short "Allowed Values" cell text for the table.
/// Takes the optional list of allowed values. Gives back "—" when there is
/// no list; otherwise the first 6 values joined by commas, plus a count of
/// how many more there are. The full lists are shown lower on the page.
/// `let Some(a) = a else { .. }` means: if the value is there, name it `a`;
/// if not, run the `else` block, which must leave the function.
fn allowed_text(a: Option<&[&str]>) -> String {
    // No list for this field: show a dash.
    let Some(a) = a else { return "—".into() };
    // Copy the values, dropping empty strings (some lists start with a blank choice).
    let vals: Vec<&str> = a.iter().copied().filter(|v| !v.is_empty()).collect();
    // 6 is the most values shown in one table cell; more would make the row too tall.
    if vals.len() > 6 {
        format!("{}  … (+{} more)", vals[..6].join(", "), vals.len() - 6)
    } else {
        vals.join(", ")
    }
}

/// Draws the "Field Reference" page body as HTML text.
/// Takes the request (`req`, which holds the address's settings such as the
/// filter box text). Gives back the HTML as a `String` (text that this
/// function owns and can grow).
pub fn render(req: &Req) -> String {
    // Page title and the intro paragraph.
    let mut h = ui::subheader("📋 JSON Field Reference")
        + &ui::markdown("Use this table when filling in a JSON file by hand.  Every field exported by Phase 1 is listed here with its expected type, allowed values (for dropdown fields), and an example.  Download the **JSON Schema** to get live validation and autocomplete inside VS Code — just point the schema at your JSON file.");
    // The text typed in the filter box (`rf`), made lower case so that
    // matching ignores capital letters.
    let q = req.get("rf").to_lowercase();
    // The filter box is a GET form: pressing Enter reloads this same tab and
    // sub-tab with `rf=` in the address. The two hidden inputs keep the tab.
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"11\"><input type=\"hidden\" name=\"sub\" value=\"6\">{}</form>",
        ui::text_input("rf", "🔍 Filter fields", req.get("rf"), "e.g. bloom, root, string…")));
    // Keep a row when the filter is empty, or when the filter text appears in
    // the field name, the type text, or any allowed value. Each kept row
    // becomes a list of table cells. `.map(..)` changes each item;
    // `.filter(..)` keeps only the items for which the test is true;
    // `.collect()` gathers the results into a list.
    let t: Vec<Vec<String>> = defs().filter(|(f, ty, a, _)| q.is_empty() || f.to_lowercase().contains(&q) || ty.to_lowercase().contains(&q) || a.map(|a| a.iter().any(|v| v.to_lowercase().contains(&q))).unwrap_or(false))
        .map(|(f, ty, a, e)| vec![ui::cell(f), ui::cell(ty), ui::cell(&allowed_text(*a)), ui::cell(e)]).collect();
    // Draw the table.
    h.push_str(&ui::dataframe(&["Field", "Type", "Allowed Values", "Example"], &t));
    h.push_str(&ui::divider());
    h.push_str(&ui::h4("Full Option Lists (for dropdown / comma-separated fields)"));
    // Fields that get a full list under the table: more than one allowed value,
    // and not a read-only or metadata field.
    let lists: Vec<&Def> = defs().filter(|(_, ty, a, _)| a.map(|a| a.len() > 1).unwrap_or(false) && !ty.contains("read-only") && !ty.contains("metadata")).collect();
    // Two columns of expanders; `i % 2` sends items alternately to left and right.
    let mut cols = [String::new(), String::new()];
    for (i, (f, _, a, _)) in lists.iter().enumerate() {
        // One bullet per value. `he` turns characters such as < and & into safe
        // HTML so that option text cannot break the page.
        let body: String = a.unwrap_or(&[]).iter().filter(|v| !v.is_empty()).map(|v| format!("<li><code>{}</code></li>", he(v))).collect();
        cols[i % 2].push_str(&ui::expander(&format!("`{f}`"), false, &format!("<ul>{body}</ul>")));
    }
    // Lay the two columns out side by side with equal width ([1, 1]).
    h.push_str(&ui::columns(&[1, 1], cols.to_vec()));
    h.push_str(&ui::divider());
    // Download section for the VS Code schema file.
    h.push_str(&ui::h4("Download JSON Schema (VS Code validation)"));
    h.push_str(&ui::markdown("Save this file as `plant_schema.json`, then add `\"$schema\": \"./plant_schema.json\"` as the first key in any exported plant JSON array wrapper to get red-underline validation and autocomplete in VS Code."));
    // This link is answered by `page` in mod.rs (`?export=jp_schema`).
    h.push_str("<a class=\"st-btn st-primary st-full\" href=\"/streamlit/?export=jp_schema\">⬇️ Download plant_schema.json</a>");
    h
}

/// Builds the JSON Schema text (draft-07) that VS Code uses to check a plant
/// JSON file. It is made from `FIELD_DEFS`, the way the Streamlit app did it.
/// Takes nothing; gives back the JSON as a `String`.
/// Every field is allowed to be null, so the checker does not complain
/// about empty values.
/// The draft-07 schema, built from FIELD_DEFS the way the Streamlit app did.
pub fn schema_json() -> String {
    // `props` will hold one entry per field. `Map` is a list of name -> value pairs.
    let mut props = Map::new();
    // `for (..) in ..` runs the block once for every field definition.
    for (f, ty, a, e) in defs() {
        // `p` is the schema entry for this one field.
        let mut p = Map::new();
        // Pick the JSON type from the words in the type description.
        // `json!([..])` builds a JSON value from a literal.
        if ty.contains("integer") {
            p.insert("type".into(), serde_json::json!(["integer", "null"]));
        } else if ty.contains("number") {
            p.insert("type".into(), serde_json::json!(["number", "null"]));
        // Fields shown as "0 or 1" are whole numbers that may only be 0, 1 or null.
        } else if ty.contains("0 or 1") {
            p.insert("type".into(), serde_json::json!(["integer", "null"]));
            p.insert("enum".into(), serde_json::json!([0, 1, null]));
        } else {
            p.insert("type".into(), serde_json::json!(["string", "null"]));
        }
        // `if let Some(a) = a` runs only when the field has a list of allowed values.
        if let Some(a) = a {
            // Only plain drop-down fields get a strict list. Read-only, metadata,
            // comma-separated and month-string fields are free-form, so no list.
            if !ty.contains("read-only") && !ty.contains("metadata") && !ty.contains("comma-separated") && !ty.contains("12-char") {
                let mut en: Vec<Value> = a.iter().filter(|v| !v.is_empty()).map(|v| Value::String(v.to_string())).collect();
                // null and the empty string are also accepted, because fields may be blank.
                en.push(Value::Null);
                en.push(Value::String(String::new()));
                p.insert("enum".into(), Value::Array(en));
            }
        }
        // Show the type text as the hover description, and the example as a hint.
        p.insert("description".into(), Value::String(ty.to_string()));
        p.insert("examples".into(), serde_json::json!([e]));
        props.insert(f.to_string(), Value::Object(p));
    }
    // Wrap the properties in a schema: an array of plant objects.
    let schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Jun & Kona Garden Plant Record",
        "description": "Schema for plant records exported from the Garden Data Orchestrator.",
        "type": "array",
        "items": { "type": "object", "properties": Value::Object(props) },
    });
    // `dumps` (in pipeline.rs) writes JSON text in the same style as Python's
    // json.dumps, so the file matches what the Streamlit app produced.
    super::pipeline::dumps(&schema)
}

// Tests run with `cargo test`; this module is left out of the real program.
#[cfg(test)]
mod tests {
    /// Checks that the drop-down lists are the same count and order as in Streamlit.
    #[test]
    fn option_lists_match_streamlit_order() {
        // Collect the names of fields that would get a full option list.
        let lists: Vec<&str> = super::defs().filter(|(_, ty, a, _)| a.map(|a| a.len() > 1).unwrap_or(false) && !ty.contains("read-only") && !ty.contains("metadata")).map(|d| d.0).collect();
        assert_eq!(lists.len(), 20);
        assert_eq!(lists[0], "plant_type");
        assert_eq!(lists[19], "fertilizer");
    }
}
