//! The plant editor: Streamlit's `render_plant_form` + `save_to_db`.
//!
//! HOW TO READ THIS FILE (for Jun). `render` draws the 14 sections of the
//! editor (Taxonomy, Growth, Soil, Investment, Beds, Foliage, Flower, Fruit,
//! Roots, Care, Calendars, Notes, Photos, Sign-off) with the same labels and
//! layout as the Streamlit app. `save` is what the Save button does: it
//! reads the submitted form, writes the plant row, and handles photo uploads
//! and the "Wipe File" buttons. Nine tabs use this one editor.
//!
//! WHAT IS DELIBERATELY DIFFERENT FROM THE STREAMLIT VERSION (DEVIATIONS.md,
//! S-1): its save silently dropped `qty`, `year_planted` and
//! `additional_pictures`, so a quantity or year edit appeared not to stick
//! and appended photos vanished from the record. Those three are saved here.
//! Used by: tabs 1 to 10 (via `tabs.rs`). The form posts to /streamlit/ with
//! `act=save_plant`, which `action` in mod.rs sends to `save` below. The
//! quantity logger (`qty_logger` / `save_qty_event`) sits under the editor.
//!
//! Rust ideas used often in this file (explained once here):
//!   - `fn name(..) -> String`: a function that gives back text it owns.
//!   - `&str` is a borrowed look at text; `String` is text the code owns.
//!   - `Map<String, Value>` is one JSON object: a plant row as field -> value.
//!   - `Option<T>` is "a value or nothing" (`Some(x)` / `None`).
//!   - `async fn` waits for the database; `.await` pauses until it answers.
//!   - `<'a>` after a name is a "lifetime": it says the borrowed data must
//!     stay alive as long as the struct that holds it.
//!   - `h.push_str(..)` adds text to the end of the page text `h`.
//!   - `vec![..]` makes a growable list; `.iter().map(..).collect()` builds
//!     a new list by changing each item.

use serde_json::{Map, Value};

use super::opts::{self, text};
use super::ui;
use crate::php::he;

/// Everything the editor needs to know besides the plant.
pub struct FormCtx<'a> {
    /// Hidden fields that bring the save back to the same tab and plant.
    pub back: Vec<(&'a str, String)>,
    /// True when the editor is showing an AI result, so labels get AI badges.
    pub is_ai: bool,
    /// Names of fields the AI filled that were empty before (labelled "New").
    pub new_fields: &'a [String],
    /// Fields the AI changed: field name -> the old value (labelled "Was").
    pub changed_fields: &'a Map<String, Value>,
    /// Every plant name, for the companion-plant chooser.
    pub all_names: &'a [String],
    /// Where the plant photos live (www/images/Database), to flag missing files.
    pub images_dir: std::path::PathBuf,
}

/// Reads field `k` of a plant row as plain text ("" when missing or null).
fn s(m: &Map<String, Value>, k: &str) -> String {
    text(m.get(k))
}

/// The label, with Streamlit's AI badges when the editor shows an AI result.
/// Takes the form context, the data shown, the field name `f` and the plain
/// label `n`. Gives back the label text to display. Some fields never get a badge.
fn lbl(ctx: &FormCtx, data: &Map<String, Value>, f: &str, n: &str) -> String {
    // Not an AI view, or a field the AI may not touch: plain label.
    if !ctx.is_ai || matches!(f, "quantity" | "year_added" | "garden_beds") {
        return n.to_string();
    }
    // The AI filled this field and it was empty before.
    if ctx.new_fields.iter().any(|x| x == f) {
        return format!("✨ {n} (New)");
    }
    // The AI changed this field: show the old value, cut to 20 characters.
    if let Some(old) = ctx.changed_fields.get(f) {
        return format!("🔄 {n} (Was: {})", first_n(&text(Some(old)), 20));
    }
    // The AI left an existing value alone: show a short copy of it.
    let v = s(data, f);
    if !v.trim().is_empty() {
        return format!("✨ {n} [AI: {}]", first_n(&v, 20));
    }
    n.to_string()
}

/// The first `n` characters of `v` (by character, not byte, so emoji are safe).
fn first_n(v: &str, n: usize) -> String {
    v.chars().take(n).collect()
}

/// `get_safe_opts`: the current value first; a value not in the list is
/// kept (put first) so legacy data survives.
/// Takes the drop-down's fixed `list` and the saved value `val`. Gives back the
/// options with the saved value first, so the select shows it as chosen.
fn safe_opts(list: &[&str], val: &str) -> Vec<String> {
    // `trim` removes spaces at both ends.
    let v = val.trim();
    // No saved value: use the list as it is.
    if v.is_empty() {
        return list.iter().map(|x| x.to_string()).collect();
    }
    // Find the saved value in the list, ignoring capital letters.
    match list.iter().position(|o| o.eq_ignore_ascii_case(v)) {
        // Not in the list: put the odd value first and keep every list entry after it.
        None => std::iter::once(v.to_string())
            .chain(list.iter().map(|x| x.to_string()))
            .collect(),
        // In the list: put the list's own spelling first, then the others.
        Some(i) => std::iter::once(list[i].to_string())
            .chain(
                list.iter()
                    .filter(|o| **o != list[i])
                    .map(|x| x.to_string()),
            )
            .collect(),
    }
}

/// Draws one drop-down for field `f`. `default` is used only when the plant
/// row has no such column at all. Gives back HTML.
fn select(
    ctx: &FormCtx,
    data: &Map<String, Value>,
    f: &str,
    n: &str,
    list: &[&str],
    default: &str,
) -> String {
    // Current value; fall back to `default` only if the column is missing.
    let mut cur = s(data, f);
    if cur.trim().is_empty() && !data.contains_key(f) {
        cur = default.to_string();
    }
    // Make sure the current value is in the list and first.
    let opts = safe_opts(list, &cur);
    let first = opts.first().cloned().unwrap_or_default();
    ui::select_plain(f, &lbl(ctx, data, f, n), &opts, &first)
}

/// Draws a one-line text box for field `f`, filled with the plant's value.
fn input(ctx: &FormCtx, data: &Map<String, Value>, f: &str, n: &str) -> String {
    ui::text_input(f, &lbl(ctx, data, f, n), &s(data, f), "")
}

/// Draws a multi-line text box for field `f` (height 100).
fn area(ctx: &FormCtx, data: &Map<String, Value>, f: &str, n: &str) -> String {
    ui::text_area(f, &lbl(ctx, data, f, n), &s(data, f), 100)
}

/// A stored list ("a, b", "['a','b']", "a / b") as its items.
/// Splits `raw` at the character `sep` into trimmed items, dropping empty items, none and null.
fn list_of(raw: &str, sep: char) -> Vec<String> {
    // Remove list punctuation first: [ ] ' " so "['a','b']" becomes "a,b".
    raw.replace(['[', ']', '\'', '"'], "")
        .split(sep)
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty() && !matches!(x.to_lowercase().as_str(), "none" | "null"))
        .collect()
}

/// Joins a fixed list and a list from the plant, sorted, with no repeats.
/// Used so a saved value that is not in the standard list still appears.
fn sorted_union(a: &[&str], b: &[String]) -> Vec<String> {
    let mut v: Vec<String> = a
        .iter()
        .map(|x| x.to_string())
        .chain(b.iter().cloned())
        .collect();
    // `sort` orders the items; `dedup` removes neighbours that are equal.
    v.sort();
    v.dedup();
    v
}

/// Splits a comma list of picture file names, dropping blanks and "none"/"null"/"[]".
fn pic_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty() && !matches!(x.to_lowercase().as_str(), "none" | "null" | "[]"))
        .collect()
}

/// Draws the coloured month bar (one box per month) with its 12-letter code.
/// `bitmap` is the code (a dot means not active), `color` the box colour,
/// `emoji` and `label` the heading. Gives back HTML.
fn cal_bar(bitmap: &str, color: &str, emoji: &str, label: &str) -> String {
    // One letter per month, in order.
    let labels: Vec<char> = "JFMAMJJASOND".chars().collect();
    let bits: Vec<char> = bitmap.chars().collect();
    let mut cells = String::new();
    // Go through the 12 months; `bits.get(i)` is the code's character for that month.
    for (i, lab) in labels.iter().enumerate() {
        let ch = bits.get(i).copied().unwrap_or('.');
        // Active month: a coloured box with the month letter.
        if ch != '.' {
            cells.push_str(&format!("<span style=\"display:inline-block;width:24px;height:24px;line-height:24px;text-align:center;background:{color};color:#fff;border-radius:4px;margin:0 1px;font-size:12px;font-weight:700;\">{lab}</span>"));
        // Inactive month: a grey dot.
        } else {
            cells.push_str("<span style=\"display:inline-block;width:24px;height:24px;line-height:24px;text-align:center;color:#bbb;font-size:12px;margin:0 1px;\">·</span>");
        }
    }
    // Put the pieces together; `he` makes the code safe to show as HTML.
    format!("<div style=\"display:flex;align-items:center;gap:8px;margin:4px 0;\"><span style=\"font-size:14px;\">{emoji}</span><strong style=\"width:50px;font-size:13px;color:{color};\">{label}</strong><div>{cells}</div><code style=\"font-size:11px;color:#888;margin-left:8px;\">{}</code></div>", he(bitmap))
}

/// Draw the editor. `p` is the database row; `data` is what is shown (the
/// row itself, or an AI result laid over it).
pub fn render(p: &Map<String, Value>, data: &Map<String, Value>, ctx: &FormCtx) -> String {
    // Start the form. `data-plant-form` lets st.js find it.
    // It posts as multipart so photo uploads work.
    let mut h = String::from("<form method=\"post\" action=\"/streamlit/\" enctype=\"multipart/form-data\" data-plant-form>");
    // Hidden fields: which action this is, and which plant.
    h.push_str(&format!("<input type=\"hidden\" name=\"act\" value=\"save_plant\"><input type=\"hidden\" name=\"id\" value=\"{}\">", he(&s(p, "id"))));
    // More hidden fields so the browser comes back to the same tab and plant after Save.
    for (k, v) in &ctx.back {
        h.push_str(&format!(
            "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
            he(k),
            he(v)
        ));
    }

    // The editor has 14 sections. Each one draws its inputs in columns.
    // Section 1
    h.push_str(&ui::h4("🧬 Section 1: Taxonomy & Core Identity"));
    h.push_str(&ui::columns(
        &[1, 1, 1, 1],
        vec![
            input(ctx, data, "botanical_name", "Botanical Name")
                + &input(ctx, data, "company", "Company / Brand")
                // The grower's product page; checked by weblink::check on Save.
                + &ui::text_input(
                    "product_url",
                    &lbl(ctx, data, "product_url", "Source"),
                    &s(data, "product_url"),
                    "https://...",
                ),
            input(ctx, data, "family", "Family") + &input(ctx, data, "genus", "Genus"),
            input(ctx, data, "species", "Species")
                + &select(
                    ctx,
                    data,
                    "plant_type",
                    "Plant Type",
                    opts::PLANT_TYPES,
                    "Perennial",
                ),
            select(
                ctx,
                data,
                "native_status",
                "Native Status",
                opts::NATIVE_STATUS_OPTIONS,
                "",
            ),
        ],
    ));

    // Section 2
    h.push_str(&ui::h4("📏 Section 2: Growth Metrics, Spacing & Beds"));
    h.push_str(&ui::columns(
        &[2, 2],
        vec![
            input(ctx, data, "plant_length", "Target Length / Height")
                + &input(ctx, data, "plant_width", "Target Width / Spread")
                + &input(ctx, data, "life_expectancy", "Life Expectancy"),
            input(ctx, data, "spacing", "Plant Spacing Limits")
                + &input(
                    ctx,
                    data,
                    "zones",
                    "Global Species Hardiness Zones spectrum",
                )
                + &select(
                    ctx,
                    data,
                    "growth_rate",
                    "Growth Rate Assumption",
                    opts::GROWTH_RATES,
                    "",
                ),
        ],
    ));

    // Section 3
    h.push_str(&ui::h4(
        "🌦️ Section 3: Microclimatic Profiles & Soil Requirements",
    ));
    h.push_str(&ui::columns(
        &[1, 1, 1],
        vec![
            select(
                ctx,
                data,
                "sun_exposure",
                "Sun Exposure Requirement",
                opts::SUN_OPTIONS,
                "",
            ) + &input(ctx, data, "water_needs", "Hydration Needs"),
            input(ctx, data, "soil_ph", "Soil Target pH Scale")
                + &input(ctx, data, "soil_type", "Soil Consistency Profile"),
            input(ctx, data, "planting_depth", "Planting Depth Requirement")
                + &input(ctx, data, "chilling_hours", "Chilling Hours Requirement"),
        ],
    ));

    // Section 4: cost, quantity (qty first, then quantity), years
    h.push_str(&ui::h4("💵 Section 4: Investment & Procurement Logistics"));
    // Cost as a number (0 when blank or not a number).
    let cost = s(data, "average_cost").trim().parse::<f64>().unwrap_or(0.0);
    // The newer `qty` column wins over the older `quantity` when it has a value.
    let raw_qty = if p.get("qty").map(|v| !v.is_null()).unwrap_or(false) {
        s(p, "qty")
    } else {
        s(p, "quantity")
    };
    // Whole number, never below 0; default 1.
    let qty = raw_qty
        .trim()
        .parse::<f64>()
        .map(|x| x as i64)
        .unwrap_or(1)
        .max(0);
    // The year list: `year_planted` if set, otherwise `year_added`.
    let raw_years = {
        let y = s(p, "year_planted");
        if y.trim().is_empty() {
            s(p, "year_added")
        } else {
            y
        }
    };
    // Years may be separated by "/" or ",".
    let years = list_of(&raw_years, if raw_years.contains('/') { '/' } else { ',' });
    h.push_str(&ui::columns(
        &[1, 1, 1],
        vec![
            ui::number_input(
                "average_cost",
                &lbl(
                    ctx,
                    data,
                    "average_cost",
                    "Estimated 1-Gallon Regional Retail Cost ($)",
                ),
                &format!("{cost:.2}"),
                None,
                "0.01",
            ),
            ui::number_input(
                "quantity",
                "Quantity Planted (Manual Override Only)",
                &qty.to_string(),
                Some(0),
                "1",
            ),
            ui::multiselect(
                "year_added[]",
                "Year(s) Added to Garden (Manual Override Only)",
                &sorted_union(opts::YEARS_ADDED, &years),
                &years,
            ),
        ],
    ));
    // A stored 0 means "no year", so show an empty box.
    let died = s(p, "died_year");
    let died = if died.trim() == "0" {
        String::new()
    } else {
        died
    };
    h.push_str(&ui::text_input(
        "died_year",
        &lbl(
            ctx,
            data,
            "died_year",
            "Year Plant Died (optional, leave blank if alive)",
        ),
        died.trim(),
        "e.g. 2024",
    ));

    // Section 5: beds (only known bed names count; none = To Be Sorted)
    h.push_str(&ui::h4("📍 Section 5: Extended Human Bed Location Manager"));
    // Keep only bed names that really exist; none left means "To Be Sorted".
    let mut beds: Vec<String> = list_of(&s(p, "garden_beds"), ',')
        .into_iter()
        .filter(|b| opts::GARDEN_BEDS.contains(&b.as_str()))
        .collect();
    if beds.is_empty() {
        beds.push("To Be Sorted".into());
    }
    let bed_opts: Vec<String> = opts::GARDEN_BEDS.iter().map(|b| b.to_string()).collect();
    h.push_str(&ui::multiselect(
        "garden_beds[]",
        "Assign Plant to Bed (AI Prohibited Field)",
        &bed_opts,
        &beds,
    ));

    // Section 6
    h.push_str(&ui::h4("🌿 Section 6: Foliage & Stem Morphologies"));
    let evg = list_of(&s(data, "is_evergreen"), ',');
    h.push_str(&ui::columns(
        &[1, 1, 1],
        vec![
            input(ctx, data, "foliage_color", "Foliage Color Spec")
                + &input(ctx, data, "stem_color", "Stem Color Spec")
                + &ui::multiselect(
                    "is_evergreen[]",
                    &lbl(ctx, data, "is_evergreen", "Foliage Retention (Tags)"),
                    &sorted_union(opts::EVERGREEN_OPTIONS, &evg),
                    &evg,
                ),
            select(
                ctx,
                data,
                "leaf_shape",
                "Leaf Structural Shape",
                opts::LEAF_SHAPES,
                "",
            ) + &select(
                ctx,
                data,
                "leaf_type",
                "Leaf Blade Type",
                opts::LEAF_TYPES,
                "",
            ),
            select(
                ctx,
                data,
                "leaf_margin",
                "Leaf Margin / Edge Profile",
                opts::LEAF_MARGINS,
                "",
            ),
        ],
    ));

    // Section 7
    h.push_str(&ui::h4("🌸 Section 7: Flower, Scent & Reproduction"));
    h.push_str(&ui::columns(
        &[1, 1, 1, 1],
        vec![
            input(ctx, data, "bloom_color", "Flower/Bloom Color")
                + &input(ctx, data, "aromatics", "Aromatics / Scent Descriptors"),
            select(
                ctx,
                data,
                "flower_symmetry",
                "Flower Symmetry Layout",
                opts::FLOWER_SYMMETRIES,
                "",
            ) + &select(
                ctx,
                data,
                "flower_shape",
                "Flower / Corolla Shape",
                opts::FLOWER_SHAPES,
                "",
            ),
            select(
                ctx,
                data,
                "flower_size",
                "Flower Size Class",
                opts::FLOWER_SIZES,
                "",
            ),
            select(
                ctx,
                data,
                "reproduction_type",
                "Reproduction Mechanism",
                opts::REPRODUCTION_OPTIONS,
                "",
            ),
        ],
    ));

    // Section 8
    h.push_str(&ui::h4("🍒 Section 8: Fruit & Berry Yields"));
    h.push_str(&ui::columns(
        &[1, 1, 1],
        vec![
            input(ctx, data, "fruit_type", "Fruit/Berry Type")
                + &input(ctx, data, "fruit_shape", "Fruit Structural Shape"),
            input(ctx, data, "fruit_color", "Fruit Color")
                + &input(ctx, data, "fruit_size", "Fruit Dimension Size"),
            select(
                ctx,
                data,
                "fruit_edibility",
                "Fruit Edibility Class",
                opts::FRUIT_EDIBILITY_OPTIONS,
                "",
            ),
        ],
    ));

    // Section 9
    h.push_str(&ui::h4("🪹 Section 9: Subterranean Anatomy & Root Systems"));
    h.push_str(&ui::columns(
        &[1, 1, 1],
        vec![
            select(
                ctx,
                data,
                "root_system",
                "Primary Root Structure",
                opts::ROOT_SYSTEM_OPTIONS,
                "",
            ),
            select(
                ctx,
                data,
                "root_morphology",
                "Root Dimensional Morphology",
                opts::ROOT_MORPHOLOGY_OPTIONS,
                "",
            ),
            select(
                ctx,
                data,
                "root_function",
                "Root Order & Function Limits",
                opts::ROOT_FUNCTIONAL_OPTIONS,
                "",
            ),
        ],
    ));

    // Section 10
    h.push_str(&ui::h4(
        "🛡️ Section 10: Care, Hazards, Propagation & Companions",
    ));
    // Companions are split on commas only: plant names keep their quotes
    // ("Mint 'Kudos Ambrosia'"), unlike beds and years.
    let comps: Vec<String> = s(data, "companion_plants")
        .split(',')
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty() && !matches!(c.to_lowercase().as_str(), "none" | "null" | "[]"))
        .collect();
    let mut comp_opts: Vec<String> = ctx
        .all_names
        .iter()
        .cloned()
        .chain(comps.iter().cloned())
        .collect();
    comp_opts.sort();
    comp_opts.dedup();
    let ferts: Vec<String> = list_of(&s(data, "fertilizer"), ',')
        .into_iter()
        .filter(|f| opts::FERTILIZER_OPTIONS.contains(&f.as_str()))
        .collect();
    let fert_opts: Vec<String> = opts::FERTILIZER_OPTIONS
        .iter()
        .map(|x| x.to_string())
        .collect();
    h.push_str(&ui::columns(
        &[1, 1, 1],
        vec![
            input(ctx, data, "attracts", "Attracts Pollinators/Wildlife")
                + &select(
                    ctx,
                    data,
                    "deer_resistance",
                    "Deer/Rabbit Pest Resistance",
                    opts::RESISTANCE_OPTIONS,
                    "",
                )
                + &select(
                    ctx,
                    data,
                    "toxicity",
                    "Toxicity Verification Classification",
                    opts::TOXICITY_OPTIONS,
                    "",
                )
                + &select(
                    ctx,
                    data,
                    "invasive_local",
                    "Local Invasive Status",
                    opts::INVASIVE_OPTIONS,
                    "",
                ),
            input(ctx, data, "propagation_method", "Propagation Method")
                + &input(ctx, data, "cuttings", "Cuttings Instructions")
                + &ui::multiselect(
                    "companion_plants[]",
                    &lbl(
                        ctx,
                        data,
                        "companion_plants",
                        "Assigned Companion Plant Tokens",
                    ),
                    &comp_opts,
                    &comps,
                ),
            ui::multiselect(
                "fertilizer[]",
                &lbl(ctx, data, "fertilizer", "Assigned Fertilizer Regimens"),
                &fert_opts,
                &ferts,
            ),
        ],
    ));

    // Section 11: calendars
    // The calendars: month-bar previews, and a switch between a simple
    // start/end range and ticking each month.
    h.push_str(&ui::h4("📅 Section 11: Growth Calendars"));
    // The toggle is edited so st.js can find it (`data-cal-multi`).
    h.push_str(
        &ui::toggle(
            "multi",
            "🌸 Multi-Period Mode (non-contiguous months)",
            false,
            Some("Use this for plants that bloom in separate periods, e.g. Apr–May then Jul–Oct"),
        )
        .replace(
            "<input type=\"checkbox\"",
            "<input data-cal-multi type=\"checkbox\"",
        ),
    );
    // `parse_m` turns the stored 12-letter code into month numbers 1..12.
    let g = opts::parse_m(&s(data, "grow_months"));
    let b = opts::parse_m(&s(data, "bloom_months"));
    let month_opts: Vec<String> = opts::MONTHS.iter().map(|m| m.to_string()).collect();
    // `mname`: first or last month in the list as a short name, "Jan" if none.
    let mname = |v: &[u32], last: bool| -> String {
        let pick = if last { v.last() } else { v.first() };
        pick.map(|m| opts::MONTHS[(*m - 1) as usize].to_string())
            .unwrap_or_else(|| "Jan".into())
    };
    // `boxes`: twelve tick boxes named prefix0..prefix11, used by Multi-Period Mode.
    let boxes = |prefix: &str, on: &[u32]| -> String {
        let cells: Vec<String> = (0..12)
            .map(|i| {
                ui::checkbox(
                    &format!("{prefix}{i}"),
                    opts::MONTHS[i],
                    on.contains(&(i as u32 + 1)),
                    None,
                )
            })
            .collect();
        ui::columns(&[1; 12], cells)
    };
    let mut grow = String::new();
    grow.push_str("<div data-cal=\"range\">");
    if g.is_empty() {
        grow.push_str(&ui::warning("⚠️ Grow months not set."));
    }
    grow.push_str(&ui::columns(
        &[1, 1],
        vec![
            ui::select_plain(
                "gs",
                "Vegetative Grow Start",
                &month_opts,
                &mname(&g, false),
            ),
            ui::select_plain("ge", "Vegetative Grow End", &month_opts, &mname(&g, true)),
        ],
    ));
    grow.push_str("</div><div data-cal=\"multi\" style=\"display:none\">");
    grow.push_str(&ui::markdown("**🌱 Grow Months** — tick each active month"));
    grow.push_str(&boxes("gm", &g));
    grow.push_str("</div>");
    // A plant with these bloom colors is treated as flowerless.
    let bc = s(data, "bloom_color").trim().to_lowercase();
    let nobloom = matches!(bc.as_str(), "not ornamental" | "none" | "insignificant");
    let mut bloom = ui::checkbox(
        "nobloom",
        "🚫 Does not bloom",
        nobloom,
        Some(
            "Mark flowerless (conifer / fern / foliage-only). Bloom months are no longer required.",
        ),
    )
    .replace(
        "<input type=\"checkbox\"",
        "<input data-cal-nobloom type=\"checkbox\"",
    );
    bloom.push_str(&format!(
        "<div data-cal=\"flowerless\"{}>{}</div>",
        if nobloom {
            ""
        } else {
            " style=\"display:none\""
        },
        ui::caption("🌲 Flowerless — bloom months cleared and no longer required.")
    ));
    bloom.push_str(&format!(
        "<div data-cal=\"bloom\"{}><div data-cal=\"range\">",
        if nobloom {
            " style=\"display:none\""
        } else {
            ""
        }
    ));
    if b.is_empty() {
        bloom.push_str(&ui::warning("⚠️ Bloom months not set."));
    }
    bloom.push_str(&ui::columns(
        &[1, 1],
        vec![
            ui::select_plain(
                "bs",
                "Reproductive Bloom Start",
                &month_opts,
                &mname(&b, false),
            ),
            ui::select_plain(
                "be",
                "Reproductive Bloom End",
                &month_opts,
                &mname(&b, true),
            ),
        ],
    ));
    bloom.push_str("</div><div data-cal=\"multi\" style=\"display:none\">");
    bloom.push_str(&ui::markdown(
        "**🌸 Bloom Months** — tick each active month",
    ));
    bloom.push_str(&boxes("bm", &b));
    bloom.push_str("</div></div>");
    h.push_str(&ui::columns(&[1, 1], vec![grow, bloom]));
    let gb = opts::format_m(&g);
    let bb = opts::format_m(&if nobloom { Vec::new() } else { b.clone() });
    // Live preview bars, using the colours of the website card.
    h.push_str(&format!("<div style=\"background:#1e1e1e;border-radius:10px;padding:12px 16px;margin:8px 0;\">{}{}</div>",
        cal_bar(&gb, "#2e7d32", "🌱", "Grow"), cal_bar(&bb, "#d81b60", "🌸", "Bloom")));
    h.push_str(&ui::caption("↑ Preview matches the website card. The `code` shows the DB bitmap value that will be saved."));

    // Section 12
    h.push_str(&ui::h4(
        "📝 Section 12: Long-Form Maintenance Journals & Search Overrides",
    ));
    let mut right = area(
        ctx,
        data,
        "disease_susceptibility",
        "Disease & Insect Vulnerabilities",
    ) + &area(
        ctx,
        data,
        "extra_details",
        "Extra Details / Culinary Edibility Profile",
    );
    if ctx.is_ai {
        right.push_str(&ui::text_input(
            "image_search_term",
            "Engine Image Search Target Override",
            &s(data, "image_search_term"),
            "",
        ));
    }
    // The Flag box is ticked when needs_review is 1 or true.
    let flagged = matches!(s(p, "needs_review").trim(), "1" | "true" | "True");
    right.push_str(&ui::checkbox(
        "needs_review",
        "⚠️ Flag Entry for Further Technical Review",
        flagged,
        None,
    ));
    h.push_str(&ui::columns(
        &[1, 1],
        vec![
            area(ctx, data, "care_notes", "Seasonal Care & Watering Notes")
                + &area(ctx, data, "maintenance_notes", "Pruning & Fertilizer Notes")
                + &area(ctx, data, "cleanup_notes", "Winterization & Cleanup Notes"),
            right,
        ],
    ));

    // Section 13: photos (picture first, then additional_pictures)
    h.push_str(&ui::divider());
    h.push_str(&ui::subheader(
        "🖼️ Section 13: Digital Asset Gallery Management",
    ));
    // Collect the picture file names: `picture` first, then `additional_pictures`.
    let mut pics = Vec::new();
    let main = s(p, "picture").trim().to_string();
    // picture may itself be a comma list (the image manager writes that)
    pics.extend(pic_list(&main));
    pics.extend(pic_list(&s(p, "additional_pictures")));
    if pics.is_empty() {
        h.push_str(&ui::info(
            "📁 No images are currently tracked or linked for this record container row.",
        ));
    } else {
        h.push_str(&ui::markdown("**Active Vault Graphics:**"));
        h.push_str("<div class=\"st-gallery\">");
        for (i, f) in pics.iter().enumerate() {
            // Show the thumbnail only if the file exists on disk;
            // otherwise show a red "Missing File" box.
            let shown = if ctx.images_dir.join(f).is_file() {
                format!("<img src=\"/images/Database/{}\" alt=\"\" loading=\"lazy\"><div class=\"st-img-cap\">{}</div>", ui::path_url(f), he(f))
            } else {
                ui::error(&format!("Missing File: {f}"))
            };
            // The wipe button sends its position (`i`) as `purge_img`, which `save` reads.
            h.push_str(&format!(
                "<div>{shown}{}</div>",
                ui::button(
                    "purge_img",
                    &i.to_string(),
                    "🗑 *Wipe File from Vol*",
                    false,
                    true
                )
            ));
        }
        h.push_str("</div>");
    }
    h.push_str(&ui::caption("🗑 removes the photo from this plant's gallery. The file itself is kept (see DEVIATIONS.md S-2)."));
    if ctx.is_ai {
        h.push_str(&ui::button(
            "stage_img",
            "1",
            "🌐 Fetch & Stage Image from Web Search",
            false,
            false,
        ));
    }
    h.push_str(&ui::file_uploader(
        "uploads[]",
        "Upload custom photo(s) to gallery collection",
        &["jpg", "jpeg", "png", "webp"],
        true,
    ));
    if !pics.is_empty() {
        h.push_str(&ui::radio(
            "up_act",
            "Upload Processing Engine Action",
            &["Append to existing photos", "Replace all existing photos"],
            "Append to existing photos",
            true,
            false,
        ));
    }

    // Section 14
    h.push_str(&ui::divider());
    h.push_str(&ui::subheader(
        "✅ Section 14: Validation Signing Authority",
    ));
    // Approval is the garden owner's Approve button (Add plant review, Awaiting Approval), not a
    // switch here, so saving this form never changes it. Only the owner's approval is shown;
    // nothing else is asked for.
    h.push_str(&ui::markdown(
        if crate::plant_ai::is_approved(p) {
            "✅ **Approved by the garden owner.** / 정원 주인이 승인했어요."
        } else {
            "⏳ **Not approved yet.** The garden owner approves with the ✅ Approve button (Add plant tab, or Awaiting Approval). / 아직 승인되지 않았어요. 정원 주인이 ✅ 승인 버튼으로 승인해요."
        },
    ));
    h.push_str(&ui::button("save", "1", "Save Changes", true, true));
    h.push_str("</form>");
    h
}

/// Text columns the editor writes when the form sent them (save_to_db's
/// list, minus the ones handled specially below).
/// Text columns copied straight from the form into the database when sent.
const TEXT_FIELDS: [&str; 51] = [
    "company",
    "family",
    "genus",
    "botanical_name",
    "species",
    "plant_type",
    "sun_exposure",
    "cleanup_notes",
    "care_notes",
    "cuttings",
    "maintenance_notes",
    "attracts",
    "zones",
    "water_needs",
    "deer_resistance",
    "soil_ph",
    "soil_type",
    "toxicity",
    "bloom_color",
    "foliage_color",
    "stem_color",
    "disease_susceptibility",
    "propagation_method",
    "plant_length",
    "plant_width",
    "extra_details",
    "image_search_term",
    "spacing",
    "leaf_shape",
    "leaf_type",
    "leaf_margin",
    "flower_symmetry",
    "flower_shape",
    "flower_size",
    "fruit_type",
    "fruit_shape",
    "fruit_color",
    "fruit_size",
    "fruit_edibility",
    "invasive_local",
    "native_status",
    "growth_rate",
    "planting_depth",
    "chilling_hours",
    "aromatics",
    "life_expectancy",
    "reproduction_type",
    "root_system",
    "root_morphology",
    "root_function",
    "plant_name",
];
/// Multiselects: always written ("" when nothing is chosen).
const LIST_FIELDS: [&str; 4] = [
    "fertilizer",
    "companion_plants",
    "garden_beds",
    "is_evergreen",
];

/// A file name the way the Streamlit app made them: the plant name with
/// everything but ASCII letters and digits removed.
pub fn file_safe(name: &str) -> String {
    name.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// Reads the month numbers from the form. In Multi-Period Mode it reads the 12
/// tick boxes (`boxes` + 0..11); otherwise it reads the start and end drop-downs
/// named in `range` and fills every month between them.
fn months_from_form(f: &crate::formdata::FormData, range: (&str, &str), boxes: &str) -> Vec<u32> {
    // `f.has(name)` is true when the form sent that field.
    if f.has("multi") {
        (0..12)
            .filter(|i| f.has(&format!("{boxes}{i}")))
            .map(|i| i as u32 + 1)
            .collect()
    } else {
        // Position of the chosen month name in the month list, as 1..12 (default 1).
        let idx = |k: &str| {
            opts::MONTHS
                .iter()
                .position(|m| *m == f.str(k))
                .map(|i| i as u32 + 1)
                .unwrap_or(1)
        };
        opts::month_range(idx(range.0), idx(range.1))
    }
}

/// A message shown at the top of the page after an action (a "flash").
/// The four kinds match Streamlit's success, info, warning and error boxes.
/// An `enum` is a type that is exactly one of several named choices; each
/// choice here carries the message text.
pub enum Flash {
    Success(String),
    Info(String),
    Warning(String),
    Error(String),
}

/// The Save button (and the 🗑 buttons, which submit the same form).
/// Takes the database connection (inside `state`) and the submitted form `f`.
/// Gives back the messages to show. Steps: read the current gallery, handle a
/// wipe button, collect every column value, handle photo uploads, run one
/// UPDATE, then read the calendars back to confirm them.
pub async fn save(state: &crate::AppState, f: &crate::formdata::FormData) -> Vec<Flash> {
    // php_int turns text into a whole number the way PHP does (0 when not a number).
    let id = crate::php::php_int(f.str("id"));
    if id <= 0 {
        return vec![Flash::Error("No plant was given.".into())];
    }
    // Load the plant's current name and pictures.
    let row = sqlx::query(
        "SELECT plant_name, picture, additional_pictures FROM data_source WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;
    // `let Ok(Some(row)) = row else { .. }`: continue only if the query worked and found a row.
    let Ok(Some(row)) = row else {
        return vec![Flash::Error(format!("Plant {id} was not found."))];
    };
    // Brings `try_get` (read a column from a row) into scope.
    use sqlx::Row;
    // A column may be NULL, so read it as Option and use "" when missing.
    let name: String = row
        .try_get::<Option<String>, _>("plant_name")
        .ok()
        .flatten()
        .unwrap_or_default();
    let cur_pic: String = row
        .try_get::<Option<String>, _>("picture")
        .ok()
        .flatten()
        .unwrap_or_default();
    let cur_extra: String = row
        .try_get::<Option<String>, _>("additional_pictures")
        .ok()
        .flatten()
        .unwrap_or_default();
    // The gallery is `picture` followed by `additional_pictures`.
    let mut gallery: Vec<String> = pic_list(&cur_pic);
    gallery.extend(pic_list(&cur_extra));

    // 🗑 Wipe File: take one photo out of the gallery (the file is kept).
    if f.has("purge_img") {
        let i = crate::php::php_int(f.str("purge_img")).max(0) as usize;
        // `purge_img` is the position of the photo to remove.
        let Some(gone) = gallery.get(i).cloned() else {
            return vec![];
        };
        gallery.remove(i);
        let main = gallery.first().cloned().unwrap_or_default();
        let extras = gallery
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        // Save the first photo as `picture` and the rest as `additional_pictures`.
        return match sqlx::query(
            "UPDATE data_source SET picture = ?, additional_pictures = ? WHERE id = ?",
        )
        .bind(&main)
        .bind(&extras)
        .bind(id)
        .execute(&state.db)
        .await
        {
            Ok(_) => vec![Flash::Success(format!("🗑 '{gone}' removed from gallery."))],
            Err(e) => vec![Flash::Error(format!(
                "Failed to remove image from database: {e}"
            ))],
        };
    }

    // `sets` is the list of (column, new value) pairs for one UPDATE.
    // Every column and its new value.
    let mut sets: Vec<(&str, Option<String>)> = Vec::new();
    // The Source link: https only, empty = NULL. A bad address stops the whole
    // save so nothing is half-written.
    if f.has("product_url") {
        match crate::weblink::check(f.str("product_url")) {
            Ok(v) => sets.push(("product_url", v)),
            Err(msg) => return vec![Flash::Error(msg.into())],
        }
    }
    // Text columns: write only those the form sent.
    for k in TEXT_FIELDS {
        if f.has(k) {
            sets.push((k, Some(f.str(k).to_string())));
        }
    }
    // Set below when "Does not bloom" is ticked.
    let mut bloom_color_override = false;
    // Pick-lists are always written, as "a, b, c" (empty when nothing is chosen).
    for k in LIST_FIELDS {
        sets.push((k, Some(f.get(&format!("{k}[]")).unwrap_or("").to_string())));
    }
    // The editor's bed list is also kept in the older `flower_bed` column.
    let beds = f.get("garden_beds[]").unwrap_or("").to_string();
    sets.push(("flower_bed", Some(beds)));
    // Cost as a number; bad text becomes 0.
    let cost = f.str("average_cost").trim().parse::<f64>().unwrap_or(0.0);
    sets.push(("average_cost", Some(format!("{cost}"))));
    // Quantity: whole number, at least 0, default 1.
    let qty = f.str("quantity").trim().parse::<i64>().unwrap_or(1).max(0);
    sets.push(("quantity", Some(qty.to_string())));
    sets.push(("qty", Some(qty.to_string()))); // S-1: Streamlit dropped this
                                               // Years are stored as "2023, 2024" and also as "2023 / 2024" in year_planted.
    let years = f.get("year_added[]").unwrap_or("").to_string();
    let planted = years
        .split(',')
        .map(str::trim)
        .filter(|y| !y.is_empty())
        .collect::<Vec<_>>()
        .join(" / ");
    sets.push(("year_added", Some(years)));
    sets.push(("year_planted", Some(planted))); // S-1
                                                // Year died: digits only, otherwise NULL.
    let died = f.str("died_year").trim();
    sets.push((
        "died_year",
        if !died.is_empty() && died.chars().all(|c| c.is_ascii_digit()) {
            Some(died.to_string())
        } else {
            None
        },
    ));
    // Tick boxes: only sent by the browser when ticked, so absent means 0. The approval columns
    // are not written here: only the owner's Approve button changes `approved_jun`, and the old
    // second sign-off column is left exactly as it is.
    let flag = if f.has("needs_review") { "1" } else { "0" };
    sets.push(("needs_review", Some(flag.to_string())));
    // Calendar months from the form.
    let grow = months_from_form(f, ("gs", "ge"), "gm");
    // "Does not bloom" clears the bloom months and sets bloom_color to "Not ornamental".
    let bloom = if f.has("nobloom") {
        bloom_color_override = true;
        Vec::new()
    } else {
        months_from_form(f, ("bs", "be"), "bm")
    };
    if bloom_color_override {
        sets.retain(|(k, _)| *k != "bloom_color");
        sets.push(("bloom_color", Some("Not ornamental".into())));
    }
    // Months are saved as the 12-letter code.
    sets.push(("grow_months", Some(opts::format_m(&grow))));
    sets.push(("bloom_months", Some(opts::format_m(&bloom))));

    // Uploads: <Plant>_<id>_manual_<time+i>.webp, WebP kept as is, anything
    // else converted at quality 87.
    // Photo uploads from the multi-file box.
    let uploads = f.files_for("uploads[]");
    if !uploads.is_empty() {
        let dir = state.www.join("images/Database");
        // Seconds since 1970; used to make file names unique.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut new_files = Vec::new();
        for (i, u) in uploads.iter().enumerate() {
            let fname = format!("{}_{}_manual_{}.webp", file_safe(&name), id, now + i as u64);
            let is_webp = u.filename.to_lowercase().ends_with(".webp");
            // WebP files are kept as they are; others are converted to WebP at quality 87.
            let bytes = if is_webp {
                Some(u.bytes.clone())
            } else {
                crate::images::to_webp(&u.bytes, 87.0)
            };
            // Skip a file that could not be converted.
            let Some(bytes) = bytes else { continue };
            if std::fs::write(dir.join(&fname), bytes).is_ok() {
                new_files.push(fname);
            }
        }
        // Replace all, or add after the existing photos.
        let (main, extras) = if f.str("up_act") == "Replace all existing photos" {
            let mut it = new_files.into_iter();
            (it.next().unwrap_or_default(), it.collect::<Vec<_>>())
        } else if let Some(first) = gallery.first().cloned() {
            (
                first,
                gallery.iter().skip(1).cloned().chain(new_files).collect(),
            )
        } else {
            let mut it = new_files.into_iter();
            (it.next().unwrap_or_default(), it.collect())
        };
        sets.push(("picture", Some(main)));
        sets.push(("additional_pictures", Some(extras.join(", ")))); // S-1
        sets.push(("image_verified", Some("1".into())));
    }

    // Build: UPDATE data_source SET `a` = ?, `b` = ? ... WHERE id = ?
    let sql = format!(
        "UPDATE data_source SET {} WHERE id = ?",
        sets.iter()
            .map(|(k, _)| format!("`{k}` = ?"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    // Bind each value in order. sqlx sends values apart from the SQL text,
    // so a value can never change the SQL.
    let mut q = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()));
    for (_, v) in &sets {
        q = q.bind(v.clone());
    }
    // The last bind is the id; run it. On failure return an error message.
    if let Err(e) = q.bind(id).execute(&state.db).await {
        return vec![Flash::Error(format!("Save failed: {e}"))];
    }
    // Home-page widgets keep a cache of plant data; mark it out of date.
    state.invalidate_widget_cache();

    // Read the calendars back, as the Streamlit app did.
    let mut out = Vec::new();
    if let Ok(Some(r)) =
        sqlx::query("SELECT grow_months, bloom_months FROM data_source WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.db)
            .await
    {
        let g: String = r
            .try_get::<Option<String>, _>("grow_months")
            .ok()
            .flatten()
            .unwrap_or_default();
        let b: String = r
            .try_get::<Option<String>, _>("bloom_months")
            .ok()
            .flatten()
            .unwrap_or_default();
        // A calendar is good when it has 12 characters and is not all dots.
        let ok = |s: &str| s.chars().count() == 12 && s != "............";
        // Show NULL when the stored value is empty.
        let or_null = |s: &str| {
            if s.is_empty() {
                "NULL".to_string()
            } else {
                s.to_string()
            }
        };
        match (ok(&g), ok(&b)) {
            (true, true) => out.push(Flash::Success(format!(
                "✅ Verified — Grow: `{g}` | Bloom: `{b}` — both saved & will display on website."
            ))),
            (true, false) => {
                out.push(Flash::Success(format!("✅ Grow saved: `{g}`")));
                out.push(Flash::Warning(format!(
                    "⚠️ Bloom still empty in DB: `{}` — website will show 'Not Specified'.",
                    or_null(&b)
                )));
            }
            (false, true) => {
                out.push(Flash::Warning(format!(
                    "⚠️ Grow still empty in DB: `{}`",
                    or_null(&g)
                )));
                out.push(Flash::Success(format!("✅ Bloom saved: `{b}`")));
            }
            (false, false) => out.push(Flash::Warning(format!(
                "⚠️ Both calendars empty — Grow: `{}` | Bloom: `{}`",
                or_null(&g),
                or_null(&b)
            ))),
        }
    }
    out.push(Flash::Success(
        "Successfully committed to target instances!".into(),
    ));
    out
}

/// The last 172 Mondays, newest first, labelled as the Streamlit picker did.
/// Takes today's date. Gives back 172 (label, Monday date) pairs, newest first.
/// 172 weeks is how far back the Streamlit picker went.
fn weeks(today: chrono::NaiveDate) -> Vec<(String, chrono::NaiveDate)> {
    // Brings `.weekday()` into scope.
    use chrono::Datelike;
    // The Monday of this week.
    let monday = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    (0..172)
        .map(|w| {
            let mon = monday - chrono::Duration::weeks(w);
            let sun = mon + chrono::Duration::days(6);
            let md = |d: chrono::NaiveDate| d.format("%b %-d").to_string();
            // Week 0 and 1 get friendly names; older weeks show the date range.
            let label = match w {
                0 => format!("This week  ({} – {})", md(mon), md(sun)),
                1 => format!("Last week  ({} – {})", md(mon), md(sun)),
                _ => format!("{} – {}", md(mon), sun.format("%b %-d, %Y")),
            };
            (label, mon)
        })
        .collect()
}

/// "📊 Log a Quantity Event (optional)", under the editor.
/// Takes the plant id and the hidden fields for coming back. Gives back HTML:
/// a small closed-by-default form for logging a purchase, death or removal.
pub fn qty_logger(id: i64, back: &[(&str, String)]) -> String {
    let mut b = String::from("<form method=\"post\" action=\"/streamlit/\">");
    b.push_str(&format!("<input type=\"hidden\" name=\"act\" value=\"qty_event\"><input type=\"hidden\" name=\"id\" value=\"{id}\">"));
    for (k, v) in back {
        b.push_str(&format!(
            "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
            he(k),
            he(v)
        ));
    }
    b.push_str(&ui::caption("Record a purchase, loss, or removal for this plant. Updates the current qty automatically."));
    // (value, label) pairs for the Event type drop-down.
    let types: Vec<(String, String)> = [
        ("purchased", "🛒 Purchased / Added"),
        ("died", "💀 Died"),
        ("removed", "🗑️ Removed"),
        ("other", "📝 Other"),
    ]
    .iter()
    .map(|(a, c)| (a.to_string(), c.to_string()))
    .collect();
    b.push_str(&ui::columns(
        &[1, 1],
        vec![
            ui::selectbox("event_type", "Event type", &types, "purchased", false),
            ui::number_input("qty_amt", "How many plants", "1", Some(1), "1"),
        ],
    ));
    // Week choices; the value is the position in the list, the label is the date range.
    let w: Vec<(String, String)> = weeks(super::opts::today())
        .into_iter()
        .enumerate()
        .map(|(i, (l, _))| (i.to_string(), l))
        .collect();
    b.push_str(&ui::selectbox("week", "Week", &w, "0", false));
    b.push_str(&ui::text_input(
        "notes",
        "Notes (optional)",
        "",
        "e.g. bought from Hewitt's",
    ));
    b.push_str(&ui::button(
        "save",
        "1",
        "💾 Save Quantity Event",
        false,
        true,
    ));
    b.push_str("</form>");
    ui::expander("📊 Log a Quantity Event (optional)", false, &b)
}

/// Handles the quantity form: writes a row in plant_quantity_log, changes the
/// plant's `qty`, and marks it dead when the quantity reaches 0 after a death
/// or removal. Gives back the messages to show.
pub async fn save_qty_event(state: &crate::AppState, f: &crate::formdata::FormData) -> Vec<Flash> {
    use chrono::Datelike;
    let id = crate::php::php_int(f.str("id"));
    // Only the four known event kinds are accepted; anything else is "other".
    let kind = match f.str("event_type") {
        k @ ("purchased" | "died" | "removed" | "other") => k,
        _ => "other",
    };
    // At least 1 plant.
    let n = crate::php::php_int(f.str("qty_amt")).max(1);
    let wk = weeks(super::opts::today());
    // Week position, kept inside the list.
    let idx = (crate::php::php_int(f.str("week")).max(0) as usize).min(wk.len() - 1);
    let date = wk[idx].1;
    // Week of the year, counting from 1 (the %W format starts at 0).
    let week: i64 = date.format("%W").to_string().parse::<i64>().unwrap_or(0) + 1;
    let notes = f.str("notes").trim();
    // The plant's name, for the confirmation message.
    let name: String = sqlx::query_scalar("SELECT plant_name FROM data_source WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    // Insert the log row. The `?` marks are filled in by the `.bind(..)` calls below.
    let r = sqlx::query(
        "INSERT INTO plant_quantity_log (plant_id, event_type, quantity, event_date, event_week, event_month, event_year, notes, recorded_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'orchestrator')",
    )
    .bind(id).bind(kind).bind(n).bind(date.to_string()).bind(week).bind(date.month()).bind(date.year())
    .bind(if notes.is_empty() { None } else { Some(notes.to_string()) })
    .execute(&state.db).await;
    // If the table is missing, tell the user to run the migration; else show the error.
    if let Err(e) = r {
        let e = e.to_string();
        return vec![if e.contains("plant_quantity_log") {
            Flash::Warning("⚠️ Quantity log table not found — run the migration first: `tools/plant_tracking_migration.sql`".into())
        } else {
            Flash::Error(format!("Error saving event: {e}"))
        }];
    }
    // A purchase adds to the quantity; anything else subtracts.
    let delta = if kind == "purchased" { n } else { -n };
    // GREATEST(0, ..) keeps the quantity from going below zero.
    let _ = sqlx::query("UPDATE data_source SET qty = GREATEST(0, qty + ?) WHERE id = ?")
        .bind(delta)
        .bind(id)
        .execute(&state.db)
        .await;
    // Read the new quantity back.
    let now: Option<i64> = sqlx::query_scalar("SELECT qty FROM data_source WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten();
    // If nothing is left after a death or removal, mark the plant as died.
    if now == Some(0) && matches!(kind, "died" | "removed") {
        let _ = sqlx::query("UPDATE data_source SET died = 1 WHERE id = ?")
            .bind(id)
            .execute(&state.db)
            .await;
    }
    state.invalidate_widget_cache();
    vec![Flash::Success(format!(
        "✅ Logged: {kind} ×{n} for **{name}** (week of {})",
        date.format("%b %-d, %Y")
    ))]
}

// Tests: run with `cargo test`; they are left out of the real program.
#[cfg(test)]
mod tests {
    use super::*;

    /// The week list has 172 entries and its labels match the Streamlit picker.
    #[test]
    fn week_labels_like_streamlit() {
        let w = weeks(chrono::NaiveDate::from_ymd_opt(2026, 10, 7).unwrap());
        assert_eq!(w.len(), 172);
        assert_eq!(w[0].0, "This week  (Oct 5 – Oct 11)");
        assert_eq!(w[1].0, "Last week  (Sep 28 – Oct 4)");
        assert_eq!(w[2].0, "Sep 21 – Sep 27, 2026");
    }

    /// Plant names become file-safe: only ASCII letters and digits stay.
    #[test]
    fn file_names_like_streamlit() {
        assert_eq!(file_safe("Lil' Kim® Rose"), "LilKimRose");
    }

    /// A saved value that is not in the list is kept, first.
    #[test]
    fn safe_opts_keeps_legacy_values_first() {
        assert_eq!(
            safe_opts(&["", "Slow", "Fast"], "fast"),
            vec!["Fast", "", "Slow"]
        );
        assert_eq!(safe_opts(&["", "Slow"], "Weird"), vec!["Weird", "", "Slow"]);
        assert_eq!(safe_opts(&["", "Slow"], ""), vec!["", "Slow"]);
    }

    /// List text with Python-style punctuation is split into clean items.
    #[test]
    fn lists_parse_like_python() {
        assert_eq!(list_of("['a', 'b'], None", ','), vec!["a", "b"]);
        assert_eq!(list_of("2023 / 2024", '/'), vec!["2023", "2024"]);
    }
}
