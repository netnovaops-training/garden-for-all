//! The plant admin's fixed lists and small helpers — `config.py` from the
//! Streamlit app (plant_orchestrator), ported 2026-10-07.
//!
//! HOW TO READ THIS FILE (for Jun). Every drop-down in the plant editor
//! takes its choices from one of the lists below, in exactly the order the
//! Streamlit app showed them ("" = the blank first choice). Change a list
//! here and every tab that uses it changes. `parse_m` / `format_m` turn the
//! 12-letter month code stored in the database ("..MAMJJA....") into month
//! numbers and back. `missing_fields` is the rule that decides whether a
//! plant is complete (Partials, Awaiting Approval, Fully Approved, JSON
//! Pipeline all use it).
//!
//! RUST WORDS USED HERE. A `const` is a value fixed when the program is built; it never changes.
//! `&[&str]` is a read-only list of text pieces. `Vec<u32>` is a growable list of whole numbers
//! that cannot be negative. `Map<String, Value>` is one database row: field name on the left, the
//! value (text, number, yes/no or empty) on the right. `Option<T>` is either `Some(value)` or
//! `None`, which is how Rust says "there may be nothing here".

// `use` brings a name in from another crate (a library) so it can be written short. `Map` and
// `Value` are serde_json's types for JSON data; a database row is turned into a Map elsewhere
// (rowjson.rs).
use serde_json::{Map, Value};

/// The three-letter month names, January first. `[&str; 12]` means a fixed array of exactly 12 text
/// pieces. `pub` means other files may use it.
pub const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// Drop-down choices for "Plant Type". The first "" is the blank choice.
pub const PLANT_TYPES: &[&str] = &["", "Perennial", "Annual", "Biennial", "Shrub", "Tree", "Succulent", "Vine", "Unknown"];
/// Choices for foliage retention (`is_evergreen`). No blank choice: this one is a multi-pick box.
pub const EVERGREEN_OPTIONS: &[&str] = &["Evergreen", "Semi-evergreen", "Deciduous", "Deciduous Conifer", "Herbaceous", "Spring Ephemeral", "Summer Dormant"];
/// Drop-down choices for leaf shape. The words in brackets are shown to help the person choose.
pub const LEAF_SHAPES: &[&str] = &["", "Linear (Long, narrow)", "Lanceolate (Long tapering tip)", "Ovate (Egg-shaped)", "Elliptic (Oval tapering ends)", "Cordate (Heart-shaped)", "Palmate (Hand-like lobes)", "Deltoid (Triangular)"];
/// Drop-down choices for leaf type (one blade or a leaf split into leaflets).
pub const LEAF_TYPES: &[&str] = &["", "Simple (Single undivided blade)", "Pinnately Compound (Feather-like leaflets)", "Palmately Compound (Radiating leaflets)"];
/// Drop-down choices for the leaf edge.
pub const LEAF_MARGINS: &[&str] = &["", "Entire (Smooth, no teeth)", "Serrated (Saw-toothed edges)", "Shallowly Serrate", "Lobed (Deep indentations)"];
/// Drop-down choices for flower symmetry.
pub const FLOWER_SYMMETRIES: &[&str] = &["", "None", "Actinomorphic (Radial symmetry)", "Zygomorphic (Bilateral symmetry)"];
/// Drop-down choices for flower shape.
pub const FLOWER_SHAPES: &[&str] = &["", "None", "Rotate (Flat wheel-shaped)", "Stellate (Star-shaped)", "Stellate (Chionodoxa)", "Cruciate (Cross-shaped)", "Campanulate (Bell-shaped)", "Infundibuliform (Funnel-shaped)", "Tubular (Cylindrical tube)", "Salverform (Tube opening flat)", "Bilabiate (Two-lipped)", "Urceolate (Urn-shaped)", "Trumpet/Cup-shaped (Daffodil)", "High-centered", "Double Blossoms"];
/// Drop-down choices for flower size.
pub const FLOWER_SIZES: &[&str] = &["", "None", "Tiny (<1 in)", "Small (1-2 in)", "Small to Large (1-5 in)", "Medium (2-4 in)", "Large (4-6 in)", "Massive (>6 in)"];
/// Drop-down choices for how the plant reproduces.
pub const REPRODUCTION_OPTIONS: &[&str] = &["", "Hermaphrodite", "Monoecious", "Dioecious", "Polygamous", "Unknown"];
/// Drop-down choices for growth rate.
pub const GROWTH_RATES: &[&str] = &["", "Slow", "Moderate", "Fast"];
/// Drop-down choices for the kind of root system.
pub const ROOT_SYSTEM_OPTIONS: &[&str] = &["", "Taproot Systems", "Fibrous/Adventitious Systems", "Specialized Systems", "Unknown"];
/// Drop-down choices for root thickness.
pub const ROOT_MORPHOLOGY_OPTIONS: &[&str] = &["", "Fine Roots (< 2 mm)", "Coarse Roots (> 2 mm)", "Mixed Fine & Coarse", "Unknown"];
/// Drop-down choices for what the roots do.
pub const ROOT_FUNCTIONAL_OPTIONS: &[&str] = &["", "Absorptive Roots (Lower Orders)", "Transport Roots (Higher Orders)", "Mixed Orders", "Unknown"];
/// Drop-down choices for whether the plant is invasive locally.
pub const INVASIVE_OPTIONS: &[&str] = &["", "No", "Yes", "Watchlist"];
/// Drop-down choices for native status.
pub const NATIVE_STATUS_OPTIONS: &[&str] = &["", "Native", "Non-native", "Introduced", "Cultivar", "Unknown"];
/// Drop-down choices for whether the fruit can be eaten.
pub const FRUIT_EDIBILITY_OPTIONS: &[&str] = &["", "Edible", "Inedible", "Toxic", "Wildlife Only", "None"];
/// Drop-down choices for sun exposure. The wishlist tab uses this list too.
pub const SUN_OPTIONS: &[&str] = &["", "Full Sun", "Partial", "Shade", "Full Sun - Partial", "Full Shade", "Full Sun - Full Shade", "Partial Shade - Full Shade", "Full Shade - Partial Shade"];
/// Drop-down choices for deer and rabbit resistance.
pub const RESISTANCE_OPTIONS: &[&str] = &["", "Deer Resistant", "Rabbit Resistant", "Both", "Not Resistant"];
/// Drop-down choices for toxicity.
pub const TOXICITY_OPTIONS: &[&str] = &["", "Non-toxic", "Toxic", "Toxic to Dogs/Cats", "Toxic to Humans", "Toxic to Humans & Pets", "Unknown"];
/// The years a plant can be marked as added. Add the next year here when it is needed.
pub const YEARS_ADDED: &[&str] = &["2022", "2023", "2024", "2025", "2026", "2027"];
/// Fertilizer products a plant can use. Multi-pick box, so there is no blank choice.
pub const FERTILIZER_OPTIONS: &[&str] = &["Azalea-tone", "Berry-tone", "Bio-tone", "Bloom Booster", "Bulb-tone", "Citrus-tone", "Evergreen-tone", "Flower-tone", "Garden-tone", "Holly-tone", "Iron-tone", "Palm-tone", "Plant-tone", "Rose-tone", "Tomato-tone", "Tree-tone", "None"];
/// The bed names, with the curly apostrophe (’) the database uses.
/// These names are saved as text in the database, so they must match it exactly. That includes the
/// curly apostrophe. The first entry is where a plant waits until it has a real bed.
pub const GARDEN_BEDS: &[&str] = &[
    "To Be Sorted",
    "All Animals Go to Heaven", "Annuals", "Aunt Sharon’s Garden",
    "Barwaz Garden", "Boo Boo’s Garden", "China Town Garden",
    "Dead or Removed", "Grandma Emily Barwaz Garden", "Grandpa Duke of Earl Middleton Garden",
    "Jason’s Terrace", "Jimmy John’s Blue’s Garden", "Karta’s Corner Garden",
    "Ko-Ja Garden", "Kona’s Garden", "Korea Garden",
    "Orchard Grove", "River Garden", "Sister Elizabeth’s Garden",
    "The 4 Season Garden", "The Royal Run", "Uncle Dennis Garden",
    "Want (Wishlist)", "Zen Garden",
];

/// "..MAMJJA...." -> [3,4,5,6,7,8]. Anything that is not exactly 12
/// characters is no months, as in Python.
/// The input is `&str`: a borrowed piece of text that this function only reads. The output is a
/// `Vec<u32>`, a new list owned by the caller.
pub fn parse_m(s: &str) -> Vec<u32> {
    // Split the text into single characters so each one can be looked at by position.
    let chars: Vec<char> = s.chars().collect();
    // The code must be exactly 12 characters, one per month. Any other length is treated as "no
    // months".
    if chars.len() != 12 {
        return Vec::new();
    }
    // Walk the characters with their position (`enumerate` counts from 0). Keep every character
    // that is not '.', turn its position into a month number (position + 1), and gather the numbers
    // into a list. `|(_, c)|` is a small inline function that receives each pair; `_` means "ignore
    // this part".
    chars.iter().enumerate().filter(|(_, c)| **c != '.').map(|(i, _)| i as u32 + 1).collect()
}

/// [3,4,5] -> "..MAM.......".
/// Here the input is a borrowed list of month numbers (`&[u32]`) and the output is a new `String`
/// (text this function owns).
pub fn format_m(months: &[u32]) -> String {
    // The letter for each month, January first. Some letters repeat (M for March and May, J for
    // June and July), so only the position tells them apart.
    const CHARS: [char; 12] = ['J', 'F', 'M', 'A', 'M', 'J', 'J', 'A', 'S', 'O', 'N', 'D'];
    // For each of the 12 positions: use that month's letter if its number (position + 1) is in the
    // list, otherwise a dot. `collect()` joins the characters into one String.
    (0..12).map(|i| if months.contains(&(i as u32 + 1)) { CHARS[i] } else { '.' }).collect()
}

/// Start..end month numbers, wrapping past December (Nov..Feb = 11,12,1,2).
pub fn month_range(start: u32, end: u32) -> Vec<u32> {
    // One month only.
    if start == end {
        vec![start]
    // A normal range inside one year, for example March to August.
    } else if start < end {
        (start..=end).collect()
    // The range crosses New Year, for example November to February. Take start..=12, then 1..=end.
    // `chain` joins two ranges into one. `..=` includes the last number.
    } else {
        (start..=12).chain(1..=end).collect()
    }
}

/// A value the way Python's `str(val)` shows it (None -> "").
/// Converts a JSON value to plain text. A database row holds mixed kinds of values, but the code
/// mostly wants text.
pub fn text(v: Option<&Value>) -> String {
    // `match` looks at the value and runs the first arm (line) whose shape fits. `Some(...)` and
    // `None` are the two cases of an Option.
    match v {
        // No value, or an empty database field (NULL): show nothing, as Python's None became an
        // empty string.
        None | Some(Value::Null) => String::new(),
        // Text: copy it. `.clone()` makes an owned copy of borrowed text.
        Some(Value::String(s)) => s.clone(),
        // Yes/no: Python printed these as True and False, so the same words are used here. `*b`
        // reads the true/false behind the reference `b`.
        Some(Value::Bool(b)) => if *b { "True".into() } else { "False".into() },
        // Anything else, such as a number: use JSON's own text form (3 becomes "3").
        Some(other) => other.to_string(),
    }
}

/// The 56 fields every finished plant must have filled in. `missing_fields` checks a plant against
/// this list. The order is the order they are reported in. Copied from the Streamlit app's
/// config.py.
const REQUIRED: [&str; 56] = [
    "botanical_name", "company", "family", "genus", "species", "plant_type",
    "plant_length", "plant_width", "spacing", "zones", "sun_exposure", "water_needs", "soil_ph",
    "soil_type", "bloom_color", "foliage_color", "stem_color", "attracts",
    "deer_resistance", "toxicity", "care_notes", "maintenance_notes",
    "cleanup_notes", "disease_susceptibility", "extra_details", "fertilizer",
    "propagation_method", "cuttings", "picture", "grow_months", "bloom_months",
    "leaf_shape", "leaf_type", "leaf_margin", "flower_symmetry", "flower_shape", "flower_size",
    "fruit_type", "fruit_shape", "fruit_color", "fruit_size", "fruit_edibility",
    "invasive_local", "native_status", "growth_rate", "planting_depth",
    "chilling_hours", "garden_beds", "aromatics", "is_evergreen", "life_expectancy",
    "reproduction_type", "root_system", "root_morphology", "root_function",
    "average_cost",
];

/// Values that always mean "not filled in yet".
pub fn is_blank(s: &str) -> bool {
    // `matches!` is a short test: is the value equal to any of the listed choices? It is applied to
    // the text after `trim()` removes spaces at both ends and `to_lowercase()` ignores capital
    // letters. These words all mean "nobody has filled this in": empty, "unknown", "[]" (an empty
    // list) and 12 dots (a month code with no months).
    matches!(s.trim().to_lowercase().as_str(), "" | "unknown" | "unknown only" | "[]" | "............")
}

/// `get_missing_fields`: which of the 56 required fields a plant lacks.
/// Flowerless plants (bloom colour "Not ornamental", "None"…) do not need
/// bloom months or the flower_* fields. average_cost of 0 counts as
/// missing. "None"/"0" count as a deliberate answer everywhere else, as the
/// Python did in practice.
pub fn missing_fields(p: &Map<String, Value>) -> Vec<&'static str> {
    // Some plants have no flowers. If the bloom colour says so, bloom months and the flower_*
    // fields are not required. The input is one plant row; the output is a list of field names.
    // `&'static str` means the names are fixed text built into the program, so they live for the
    // whole run.
    let flowerless = matches!(
        text(p.get("bloom_color")).trim().to_lowercase().as_str(),
        "not ornamental" | "none" | "insignificant" | "non-flowering" | "inconspicuous"
    );
    // The fields that flowerless plants may leave blank.
    let flower_optional = ["bloom_months", "flower_symmetry", "flower_shape", "flower_size"];
    // `let mut` makes a variable that can be changed later. `out` collects the names of missing
    // fields.
    let mut out = Vec::new();
    // Check each required field in turn.
    for f in REQUIRED {
        // A flowerless plant does not need this field, so skip to the next one (`continue`).
        if flowerless && flower_optional.contains(&f) {
            continue;
        }
        // Read the field as plain text; a missing column reads as an empty string.
        let s = text(p.get(f));
        // Blank or "unknown": record it as missing and move on.
        if is_blank(&s) {
            out.push(f);
            continue;
        }
        // A cost of 0 also counts as missing, even though "0" is a real answer for other fields.
        // `parse::<f64>()` tries to read the text as a decimal number; `if let Ok(x)` runs only
        // when that worked.
        if f == "average_cost" {
            if let Ok(x) = s.trim().parse::<f64>() {
                if x == 0.0 {
                    out.push(f);
                }
            }
        }
    }
    out
}

/// Today's date on the site's clock (TZ, America/New_York).
pub fn today() -> chrono::NaiveDate {
    // `common::now_local()` is the site's shared clock helper.
    let now = common::now_local();
    // Build a date from year, month and day. This can fail, so it gives an Option; if it ever does,
    // fall back to 2026-01-01 instead of stopping (`unwrap_or_else`).
    chrono::NaiveDate::from_ymd_opt(now.year(), u8::from(now.month()) as u32, now.day() as u32)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
}

/// A filename-safe version of a plant name: letters and digits only, as the
/// Python's `"".join(c for c in name if c.isalnum())`.
pub fn safe_name(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Tests. `#[cfg(test)]` means this block is only built when running `cargo test`; it is not part
/// of the running website.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Checks that month codes turn into numbers and back again, including the wrap past December.
    #[test]
    fn months_round_trip() {
        assert_eq!(parse_m("..MAMJJA...."), vec![3, 4, 5, 6, 7, 8]);
        assert_eq!(format_m(&[3, 4, 5, 6, 7, 8]), "..MAMJJA....");
        assert_eq!(parse_m("short"), Vec::<u32>::new());
        assert_eq!(format_m(&[]), "............");
        assert_eq!(month_range(11, 2), vec![11, 12, 1, 2]);
        assert_eq!(month_range(4, 4), vec![4]);
    }

    /// Checks the "missing fields" rule: a full plant has none missing, blank/unknown/zero cost are
    /// caught, and flowerless plants skip the flower fields.
    #[test]
    fn missing_fields_rules() {
        // Build a plant where every required field holds "x" (filled in).
        let mut p = Map::new();
        for f in REQUIRED {
            p.insert(f.into(), json!("x"));
        }
        assert!(missing_fields(&p).is_empty());
        p.insert("average_cost".into(), json!("0.00"));
        p.insert("company".into(), json!("Unknown"));
        assert_eq!(missing_fields(&p), vec!["company", "average_cost"]);
        p.insert("bloom_color".into(), json!("Not ornamental"));
        p.insert("bloom_months".into(), json!(""));
        assert!(!missing_fields(&p).contains(&"bloom_months"));
        p.insert("chilling_hours".into(), json!("0"));
        assert!(!missing_fields(&p).contains(&"chilling_hours"));
    }

    /// Checks that punctuation and symbols are removed from a name.
    #[test]
    fn safe_names() {
        assert_eq!(safe_name("Lil' Kim® Rose"), "LilKimRose");
    }
}
