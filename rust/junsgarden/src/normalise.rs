//! `garden_normalise.php` — one vocabulary for the whole site.
//!
//! The category columns on `data_source` are free text, so the same idea
//! arrives spelled several ways ("Partial" and "Partial Shade"). The
//! dashboard cards and the comparison tool both group through this file so
//! they can never disagree about what a category is. Keyed by DATABASE
//! COLUMN. Wording that is not recognised is returned unchanged, never
//! forced into a bucket.
//!
//! HOW TO READ THIS FILE. Start at `garden_normalise` at the bottom. It cleans a value
//! with `garden_clean`, then picks one of the small `normalise_*` functions by column
//! name. Each of those maps messy wording to one standard label. Unknown wording is
//! returned unchanged. No function here touches the database; they only change text.
//!
//! Rust ideas used: `&str` is borrowed text, `String` is text the function owns and
//! returns. `match` picks the arm that fits. `Option` is "a value, or nothing".
//! `|x| ...` is a closure, a small unnamed function.

/// Collapse whitespace, straighten curly quotes, trim (including a trailing
/// full stop).
// Step 1: split on any whitespace and join with single spaces. This removes extra
// spaces, tabs and line breaks.
pub fn garden_clean(v: &str) -> String {
    let collapsed = v.split_whitespace().collect::<Vec<_>>().join(" ");
    // Step 2: replace curly single quotes (U+2018, U+2019) with a straight apostrophe, and
    // curly double quotes (U+201C, U+201D) with a straight double quote.
    let straight = collapsed
        .replace(['\u{2018}', '\u{2019}'], "'")
        .replace(['\u{201C}', '\u{201D}'], "\"");
    // Step 3: trim these characters from both ends: space, tab, newline, carriage return,
    // null, vertical tab, and the full stop. Same set the PHP `trim` call used.
    straight.trim_matches(|c: char| c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\0' || c == '\u{0B}' || c == '.').to_string()
}

/// Sun words people write, as (what they wrote in lower case, standard name, rank).
/// A higher rank means brighter. `[... ; 9]` is a fixed-size list of 9 items. `u8` is a
/// small whole number (0 to 255). `const` is a value fixed at build time.
const SUN_TERMS: [(&str, &str, u8); 9] = [
    ("full sun", "Full Sun", 3),
    ("sun", "Full Sun", 3),
    ("partial shade", "Partial Shade", 2),
    ("partial sun", "Partial Shade", 2),
    ("partial", "Partial Shade", 2),
    ("part shade", "Partial Shade", 2),
    ("part sun", "Partial Shade", 2),
    ("full shade", "Full Shade", 1),
    ("shade", "Full Shade", 1),
];

/// Three terms, alone or as a range of two, brightest first.
fn normalise_sun(v: &str) -> String {
    // Any kind of dash, with optional spaces around it.
    let parts: Vec<&str> = v
        .split(|c| c == '-' || c == '\u{2013}' || c == '\u{2014}')
        .map(str::trim)
        .collect();
    // Terms found so far, as (standard name, rank), without repeats.
    let mut seen: Vec<(&str, u8)> = Vec::new();
    // Check each piece of the range on its own (e.g. "Full Sun" and "Partial").
    for p in parts {
        // Compare in lower case, because the table is lower case.
        let k = p.to_lowercase();
        // `let Some(..) = .. else { .. }` takes the match if there is one; otherwise it runs the
        // else block. Here an unrecognised piece makes the whole value come back unchanged.
        let Some((_, name, rank)) = SUN_TERMS.iter().find(|(t, _, _)| *t == k) else {
            return v.to_string();
        };
        // If this standard name was already seen, just update its rank; otherwise add it.
        if let Some(e) = seen.iter_mut().find(|(n, _)| n == name) {
            e.1 = *rank;
        } else {
            seen.push((name, *rank));
        }
    }
    // Nothing was recognised: return the original value.
    if seen.is_empty() {
        return v.to_string();
    }
    // Sort by rank, highest first, so the brightest term comes first.
    seen.sort_by(|a, b| b.1.cmp(&a.1));
    // Join the standard names with " - ".
    seen.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(" - ")
}

/// True if `word` appears in `s` as a whole word (not inside a longer word).
/// This copies the regular expression `word` between word-boundary markers. A letter,
/// digit or underscore next to the match means it is part of a longer word.
fn has_word(s: &str, word: &str) -> bool {
    // `\bword\b`
    // Work with bytes so a position can be checked next to the match.
    let bytes = s.as_bytes();
    // `start` is where to search next. Searching may find a match that is not a whole word.
    let mut start = 0;
    // `s[start..]` is the text from `start` onward. `find` gives the position of a match.
    while let Some(pos) = s[start..].find(word) {
        let i = start + pos;
        // Whole word on the left: at the very start, or the byte before is not a word character.
        let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
        let end = i + word.len();
        // Whole word on the right: at the very end, or the byte after is not a word character.
        let after_ok = end >= s.len() || !bytes[end].is_ascii_alphanumeric() && bytes[end] != b'_';
        if before_ok && after_ok {
            return true;
        }
        // Not a whole word here. Search again starting one byte further on.
        start = i + 1;
    }
    false
}

/// True if a word in `s` begins with `prefix` (the left edge must be a word boundary,
/// the right edge is free, so "average" also matches "averagely").
fn starts_word(s: &str, prefix: &str) -> bool {
    // `\bprefix` (no trailing boundary)
    let bytes = s.as_bytes();
    let mut start = 0;
    while let Some(pos) = s[start..].find(prefix) {
        let i = start + pos;
        if i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_' {
            return true;
        }
        start = i + 1;
    }
    false
}

/// Finds which water levels the text names: 1 = Low, 2 = Average, 3 = High.
/// Returns a list because a text such as "Low to Average" names two levels.
fn water_ranks(s: &str) -> Vec<u8> {
    let mut r = Vec::new();
    // "low" must be a whole word, so words that merely contain it do not count.
    if has_word(s, "low") {
        r.push(1);
    }
    // Any of these words (or words starting with them) mean Average.
    if ["average", "medium", "moderate", "regular", "consistent"].iter().any(|w| starts_word(s, w)) {
        r.push(2);
    }
    // These mean High.
    if ["high", "wet", "boggy"].iter().any(|w| starts_word(s, w)) {
        r.push(3);
    }
    r
}

/// The level named in the FIRST clause; the advice after it is ignored.
fn normalise_water(v: &str) -> String {
    // Lower case so "Low" and "low" are treated the same.
    let s = v.to_lowercase();
    // No text at all: label it Unspecified.
    if s.is_empty() {
        return "Unspecified".to_string();
    }
    // Find where the first clause ends: at a comma, semicolon or full stop, or at a dash with
    // a space on both sides. The advice after that point is ignored.
    // `[,;.]|\s[-–—]\s`
    // Default: the first clause is the whole text.
    let mut head_end = s.len();
    // Look at each character and its byte position.
    for (i, c) in s.char_indices() {
        if c == ',' || c == ';' || c == '.' {
            head_end = i;
            break;
        }
        // A dash only ends the clause when it has whitespace on both sides.
        if (c == '-' || c == '\u{2013}' || c == '\u{2014}') && i > 0 {
            // The character before and the character after the dash, if any.
            let before = s[..i].chars().last();
            let after = s[i + c.len_utf8()..].chars().next();
            if before.map_or(false, char::is_whitespace) && after.map_or(false, char::is_whitespace) {
                // The clause ends just before the space that comes before the dash.
                head_end = i - 1;
                break;
            }
        }
    }
    // `head` is the first clause. Try it first.
    let head = &s[..head_end];
    let mut ranks = water_ranks(head);
    // Nothing named in the first clause: look at the whole text.
    if ranks.is_empty() {
        ranks = water_ranks(&s);
    }
    if ranks.is_empty() {
        // Still nothing: dry or drought -> Low, moist, wet or soak -> High, and Average for
        // anything else.
        ranks = if has_word(&s, "drought") || has_word(&s, "dry") {
            vec![1]
        } else if starts_word(&s, "moist") || starts_word(&s, "wet") || starts_word(&s, "soak") {
            vec![3]
        } else {
            vec![2]
        };
    }
    // Turn a rank number into its label. This is a closure stored in a variable.
    let name = |r: u8| match r {
        1 => "Low",
        2 => "Average",
        _ => "High",
    };
    // Lowest and highest rank found. `unwrap` is safe because the list is never empty here.
    let lo = *ranks.iter().min().unwrap();
    let hi = *ranks.iter().max().unwrap();
    // One level gives its name; two levels give "Low to High" style text.
    if lo == hi { name(lo).to_string() } else { format!("{} to {}", name(lo), name(hi)) }
}

/// Maps toxicity wording to one of: Non-toxic, Toxic to Humans & Pets, Toxic to Humans,
/// Toxic to Dogs/Cats. Wording that names nobody is returned unchanged.
fn normalise_toxicity(v: &str) -> String {
    let s = v.to_lowercase();
    // "non-toxic" is checked first, because it also contains the word "toxic".
    if s.contains("non-toxic") || s.contains("nontoxic") {
        return "Non-toxic".to_string();
    }
    // Does the text mention people? Does it mention pets?
    let humans = s.contains("human") || s.contains("people");
    let pets = s.contains("dog") || s.contains("cat") || s.contains("pet");
    // Choose the label by the pair (people at risk, pets at risk).
    match (humans, pets) {
        (true, true) => "Toxic to Humans & Pets".to_string(),
        (true, false) => "Toxic to Humans".to_string(),
        (false, true) => "Toxic to Dogs/Cats".to_string(),
        _ => v.to_string(),
    }
}

/// Groups root-system wording into three standard groups. Unknown wording is unchanged.
fn normalise_roots(v: &str) -> String {
    match v.to_lowercase().as_str() {
        "fibrous" | "adventitious" | "fibrous/adventitious" => "Fibrous/Adventitious Systems".to_string(),
        "taproot" => "Taproot Systems".to_string(),
        "tuberous" | "bulbous" | "rhizomatous" => "Specialized Systems".to_string(),
        _ => v.to_string(),
    }
}

/// Standardises the evergreen/herbaceous wording. Unknown wording is unchanged.
fn normalise_evergreen(v: &str) -> String {
    match v.to_lowercase().as_str() {
        "herbaceous perennial" | "herbaceous annual" => "Herbaceous".to_string(),
        "semi evergreen" | "semi-evergreen" => "Semi-evergreen".to_string(),
        _ => v.to_string(),
    }
}

/// Standardises native status. The order of the checks matters: "non-native" contains
/// the word "native", so it must be tested before plain "native".
fn normalise_native(v: &str) -> String {
    let s = v.to_lowercase();
    if s.contains("cultivar") {
        "Cultivar".to_string()
    } else if s.contains("non-native") || s.contains("nonnative") {
        "Non-native".to_string()
    } else if s.contains("introduced") {
        "Introduced".to_string()
    } else if s.contains("native") {
        "Native".to_string()
    } else {
        v.to_string()
    }
}

/// Maps spellings of one garden bed's name to its standard name.
fn normalise_beds(v: &str) -> String {
    match v.to_lowercase().as_str() {
        "koja garden bed" | "ko-ja garden bed" | "koja garden" => "Ko-Ja Garden".to_string(),
        _ => v.to_string(),
    }
}

/// Maps retailer spellings to standard names. Apostrophes are removed before comparing,
/// so "Lowe's" and "Lowes" match.
fn normalise_retailer(v: &str) -> String {
    match v.replace('\'', "").to_lowercase().as_str() {
        "lowes" => "Lowe's".to_string(),
        "home depot" | "the home depot" => "Home Depot".to_string(),
        "kulaks nursery" => "Kulak's Nursery".to_string(),
        "hewitts garden center" => "Hewitt's Garden Center".to_string(),
        _ => v.to_string(),
    }
}

/// The window the year charts cover: 2021 up to this year.
pub const YEAR_FROM: i32 = 2021;

/// True if `yr` is a four-digit style year between YEAR_FROM and the current year.
/// Rejects empty text and anything that is not only digits.
pub fn year_in_range(yr: &str) -> bool {
    if yr.is_empty() || !yr.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    // Turn the digits into a number; a failure becomes 0, which is out of range.
    let y: i64 = yr.parse().unwrap_or(0);
    y >= YEAR_FROM as i64 && y <= common::current_year() as i64
}

/// Takes the DATABASE COLUMN and a raw value, returns the canonical label.
pub fn garden_normalise(column: &str, value: &str) -> String {
    // First tidy the text.
    let value = garden_clean(value);
    // An empty value stays empty.
    if value.is_empty() {
        return String::new();
    }
    // Pick the rule by database column name. `&value` lends the text without giving it away.
    match column {
        "sun_exposure" => normalise_sun(&value),
        "water_needs" => normalise_water(&value),
        "toxicity" => normalise_toxicity(&value),
        "root_system" => normalise_roots(&value),
        "is_evergreen" => normalise_evergreen(&value),
        "native_status" => normalise_native(&value),
        "flower_bed" => normalise_beds(&value),
        "company" | "local_store" => normalise_retailer(&value),
        // A year outside the chart window becomes empty.
        "year_planted" => {
            if year_in_range(&value) {
                value
            } else {
                String::new()
            }
        }
        // Any other column: just the cleaned text.
        _ => value,
    }
}

// Tests: only compiled with `cargo test`. They check examples of messy wording.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_ranges_collapse_brightest_first() {
        assert_eq!(garden_normalise("sun_exposure", "Partial"), "Partial Shade");
        assert_eq!(garden_normalise("sun_exposure", "Full Sun - Partial"), "Full Sun - Partial Shade");
        assert_eq!(garden_normalise("sun_exposure", "Partial Shade - Full Sun"), "Full Sun - Partial Shade");
        assert_eq!(garden_normalise("sun_exposure", "Dappled"), "Dappled");
    }

    #[test]
    fn water_reads_the_first_clause() {
        assert_eq!(garden_normalise("water_needs", "Average, drought tolerant once established"), "Average");
        assert_eq!(garden_normalise("water_needs", "Low to Average"), "Low to Average");
        assert_eq!(garden_normalise("water_needs", "1-2 inches per week"), "Average");
        assert_eq!(garden_normalise("water_needs", "Keep soil moist"), "High");
        assert_eq!(garden_normalise("water_needs", "Medium; keep soil evenly moist"), "Average");
    }

    #[test]
    fn toxicity_by_who_is_at_risk() {
        assert_eq!(garden_normalise("toxicity", "Non-toxic, pet and wildlife safe"), "Non-toxic");
        assert_eq!(garden_normalise("toxicity", "Toxic to pets and humans if ingested"), "Toxic to Humans & Pets");
        assert_eq!(garden_normalise("toxicity", "Toxic"), "Toxic");
    }

    #[test]
    fn clean_handles_quotes_and_full_stops() {
        assert_eq!(garden_clean("Birds."), "Birds");
        assert_eq!(garden_clean("Jason\u{2019}s  Terrace"), "Jason's Terrace");
        assert_eq!(garden_normalise("company", "Lowes"), "Lowe's");
        assert_eq!(garden_normalise("year_planted", "2020"), "");
        assert_eq!(garden_normalise("year_planted", "2024"), "2024");
    }
}
