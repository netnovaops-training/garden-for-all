//! One function per tab, named after the Streamlit file it replaces
//! (`tab_flagged.py` -> `flagged`). Each returns the tab's HTML.
//!
//! HOW TO READ THIS FILE (for Jun). `render` picks the tab from `?tab=N`.
//! Most tabs are "a table, a plant picker, the editor": that pattern is
//! `list_and_edit`, and the tab functions just say which plants, which
//! columns and which words. `action` handles the tab-specific buttons
//! (bed assignment, ...); the editor's own Save is in form.rs.
//!
//! TABS HANDLED HERE. 4 Manual Editor, 5 Manual Image Replace, 6 Partials, 7 Awaiting Approval, 8
//! Fully Approved, 9 Flagged for Review, 12 Bed Assignment. Tab 1 is add.rs, tabs 2, 3 and 10 are
//! ai_tabs.rs, tab 11 is pipeline.rs, tabs 13 and 14 are wishlist.rs.
//!
//! RUST WORDS USED HERE. `async fn` can pause while it waits for the database; callers write
//! `.await`. `&str` is borrowed text and `String` is text the function owns. `Vec<T>` is a growable
//! list. `Option<T>` is `Some(value)` or `None`. `match` picks the first arm whose shape fits. A
//! closure is a small inline function written `|x| ...`. `Map<String, Value>` is one plant row:
//! column name to value. The lifetime mark `<'a>` says a returned reference points into one of the
//! inputs.

// `Map` is a JSON object (one database row); `Value` is any JSON value.
use serde_json::{Map, Value};

// `Flash` is a message shown after an action. `FormCtx` bundles the settings the plant editor
// needs.
use super::form::{self, Flash, FormCtx};
// `text` turns a JSON value into plain text.
use super::opts::{self, text};
use super::ui;
// Helpers from mod.rs: the plant name list, the SQL row reader, and `Req` (the request's tab and
// settings).
use super::{all_plant_names, rows, Req};
use crate::php::he;
use crate::AppState;

/// Draws the page for the tab in `req.tab`. Returns the tab's HTML. The tab numbers match the TABS
/// list in mod.rs.
pub async fn render(state: &AppState, req: &Req) -> String {
    match req.tab {
        1 => super::add::render(state, req).await,
        2 | 3 | 10 => super::ai_tabs::render(state, req).await,
        4 => manual_editor(state, req).await,
        5 => image_replace(state, req).await,
        6 => partials(state, req).await,
        7 => approval(state, req, false).await,
        8 => approval(state, req, true).await,
        9 => flagged(state, req).await,
        11 => super::pipeline::render(state, req).await,
        12 => bed_assign(state, req).await,
        13 => super::wishlist::wishlist(state, req).await,
        14 => super::wishlist::barcodes(state, req).await,
        15 => super::quick::render_add(state, req).await,
        16 => super::quick::render_receipt(state, req).await,
        17 => super::quick::render_pictures(state, req).await,
        // A tab that has not been converted yet shows a short note. `super::TABS[n - 1]` is the
        // tab's name (lists count from 0).
        n => ui::info(&format!(
            "**{}** is being converted from Streamlit and will appear here soon.",
            super::TABS[n - 1]
        )),
    }
}

/// Runs a button that belongs to these tabs. Names starting "img_" go to the image manager, "wl_"
/// to the wishlist, "bc_" to barcodes, and `save_beds` is here. Returns the messages to show.
pub async fn action(state: &AppState, act: &str, f: &crate::formdata::FormData) -> Vec<Flash> {
    match act {
        "save_beds" => save_beds(state, f).await,
        a if a.starts_with("img_") => super::gallery::action(state, a, f).await,
        a if a.starts_with("wl_") => super::wishlist::action(state, a, f).await,
        a if a.starts_with("bc_") => super::wishlist::barcode_action(state, a, f).await,
        _ => vec![Flash::Error("Unknown action.".into())],
    }
}

/// Short helper: field `k` of the plant row as plain text (empty if missing).
fn s(m: &Map<String, Value>, k: &str) -> String {
    text(m.get(k))
}

/// The drop-down text for a plant: "Name (ID: 12)".
fn plant_label(p: &Map<String, Value>) -> String {
    format!("{} (ID: {})", s(p, "plant_name"), s(p, "id"))
}

/// A GET form whose drop-down reloads the tab with the chosen plant.
/// `opts` are the choices as (value, shown text) pairs, `selected` the current value, and `sub` the
/// inner tab to keep. The form has no button: choosing a plant sends it at once, and the page
/// reloads with `p=<id>`.
fn picker(req: &Req, label: &str, opts: &[(String, String)], selected: &str, sub: &str) -> String {
    // Start the form with the tab number as a hidden field.
    let mut h = format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"{}\">", req.tab);
    // Keep the inner tab number if there is one.
    if !sub.is_empty() {
        h.push_str(&format!(
            "<input type=\"hidden\" name=\"sub\" value=\"{}\">",
            he(sub)
        ));
    }
    // The drop-down named "p" with auto-submit.
    h.push_str(&ui::selectbox("p", label, opts, selected, true));
    h.push_str("</form>");
    h
}

/// The plant the address asked for, else the first.
/// Finds the plant in `list` whose id equals `p` in the address, or the first one. `list[0]` would
/// fail on an empty list, so callers check for empty first.
fn chosen<'a>(req: &Req, list: &'a [Map<String, Value>]) -> &'a Map<String, Value> {
    // The id from the address.
    let want = req.get("p");
    list.iter().find(|p| s(p, "id") == want).unwrap_or(&list[0])
}

/// One table cell for a column, the thumbnail column drawn as an image.
/// One table cell for column `col` of plant `p`. "Thumbnail" is not a real column: it draws the
/// first photo. Others are plain text cells.
fn cell_for(p: &Map<String, Value>, col: &str) -> String {
    if col == "Thumbnail" {
        ui::thumb(&s(p, "picture"))
    } else {
        ui::cell(&s(p, col))
    }
}

/// The column header shown for `col`: "Thumbnail" is shown as "Image".
fn header_for(col: &str) -> &str {
    if col == "Thumbnail" {
        "Image"
    } else {
        col
    }
}

/// Table + picker + editor + quantity logger.
/// The shared pattern for most tabs: `list` is the plants, `cols` the table columns, `pick_label`
/// the drop-down label, `sub` an inner tab number and `between` extra HTML shown between the table
/// and the picker.
async fn list_and_edit(
    state: &AppState,
    req: &Req,
    list: &[Map<String, Value>],
    cols: &[&str],
    pick_label: &str,
    sub: &str,
    between: &str,
) -> String {
    // Header names for each column.
    let headers: Vec<&str> = cols.iter().map(|c| header_for(c)).collect();
    // One row per plant, one cell per column.
    let table: Vec<Vec<String>> = list
        .iter()
        .map(|p| cols.iter().map(|c| cell_for(p, c)).collect())
        .collect();
    // Draw the table, then the extra HTML.
    let mut h = ui::dataframe(&headers, &table);
    h.push_str(between);
    // The picker choices.
    let opts: Vec<(String, String)> = list.iter().map(|p| (s(p, "id"), plant_label(p))).collect();
    // The plant chosen in the address, or the first.
    let p = chosen(req, list);
    // Then the picker and the editor for that plant.
    h.push_str(&picker(req, pick_label, &opts, &s(p, "id"), sub));
    h.push_str(&editor(state, req, p, sub).await);
    h
}

/// The editor and logger for one plant, returning to this tab and plant.
/// Draws the plant editor (form.rs) and the quantity logger for one plant. `req` and `sub` are used
/// to build the "back" settings, so each button returns to this tab, plant and inner tab.
async fn editor(state: &AppState, req: &Req, p: &Map<String, Value>, sub: &str) -> String {
    // All plant names, for the companion-plants box.
    let names = all_plant_names(state).await;
    // The settings every button will send back.
    let mut back = vec![("tab", req.tab.to_string()), ("p", s(p, "id"))];
    if !sub.is_empty() {
        back.push(("sub", sub.to_string()));
    }
    // No AI changes to show: this editor is not an AI result.
    let empty = Map::new();
    // Bundle the editor's settings.
    let ctx = FormCtx {
        back: back.clone(),
        is_ai: false,
        new_fields: &[],
        changed_fields: &empty,
        all_names: &names,
        images_dir: state.www.join("images/Database"),
    };
    // Draw the editor. The plant is passed twice because the editor shows one version and compares
    // with the original.
    let mut h = form::render(p, p, &ctx);
    // The plant id as a number.
    let id = crate::php::php_int(&s(p, "id"));
    h.push_str(&form::qty_logger(id, &back));
    h
}

/// Inner tabs (st.tabs inside a tab), as links.
/// Inner tabs (Streamlit's `st.tabs` inside a tab) drawn as links. `current` is the number (from 1)
/// of the open one.
fn subtabs(req: &Req, labels: &[&str], current: usize) -> String {
    let mut h = String::from("<nav class=\"st-tabs\">");
    for (i, l) in labels.iter().enumerate() {
        h.push_str(&format!(
            "<a href=\"/streamlit/?tab={}&amp;sub={}\"{}>{}</a>",
            req.tab,
            i + 1,
            if i + 1 == current {
                " class=\"on\""
            } else {
                ""
            },
            he(l)
        ));
    }
    h.push_str("</nav>");
    h
}

// ---- 🚩 Flagged for Review (tab_flagged.py) --------------------------------

/// Tab 9: plants marked "needs review". Shows them in a table with an editor. If none are flagged,
/// a green note says so.
async fn flagged(state: &AppState, req: &Req) -> String {
    // The heading and a one-line description.
    let mut h = ui::header("🚩 Flagged for Review")
        + &ui::text("Plants manually marked with the 'Needs Review' flag.");
    // Load the flagged plants.
    let list = rows(state, "SELECT * FROM data_source WHERE needs_review = 1").await;
    // Nothing flagged: return the note now (`h + ...` adds text to a String).
    if list.is_empty() {
        return h + &ui::success("No plants are currently flagged for review.");
    }
    // Extra heading shown above the picker.
    let between = ui::divider() + &ui::subheader("Edit a Flagged Plant");
    // Table columns: id, thumbnail, names and the flag.
    h.push_str(
        &list_and_edit(
            state,
            req,
            &list,
            &[
                "id",
                "Thumbnail",
                "plant_name",
                "botanical_name",
                "needs_review",
            ],
            "Select flagged plant to review:",
            "",
            &between,
        )
        .await,
    );
    h
}

// ---- 🔍 Awaiting Approval / 🏆 Fully Approved (two files, one shape) -------

/// Tabs 7 and 8 share one shape. `done` false is "Awaiting Approval": plants that have no missing
/// fields but are not approved yet. `done` true is "Fully Approved": complete plants the garden
/// owner approved. Approval is the owner's alone (`plant_ai::is_approved`, the `approved_jun`
/// column); the old second sign-off column is not asked for. Two inner tabs split perennials from
/// annuals.
async fn approval(state: &AppState, req: &Req, done: bool) -> String {
    let l = req.lang;
    // Heading and description for the chosen tab.
    let mut h = if done {
        ui::header("🏆 Fully Approved & Verified")
            + &ui::text(l.t("Browse the 100% completed, print-ready catalog. The garden owner approved this data.", "100% 완성된, 인쇄할 수 있는 목록이에요. 정원 주인이 이 정보를 승인했어요."))
    } else {
        ui::header("🔍 Awaiting Approval")
            + &ui::text(l.t("These plants have 100% of their data complete and wait for the garden owner's Approve.", "이 식물들은 정보가 100% 채워졌고 정원 주인의 승인을 기다려요."))
    };
    // Inner tab 1 (perennials) or 2 (annuals).
    let sub = if req.get("sub") == "2" { 2 } else { 1 };
    h.push_str(&subtabs(req, &["🌷 Perennials", "🌻 Annuals"], sub));
    // The database view to read: `Perennials` or `Annuals`. The text is chosen by the code, never
    // typed by a visitor, so it is safe to place in SQL.
    let view = if sub == 1 { "Perennials" } else { "Annuals" };
    // Keep plants with no missing fields whose both-approved status equals `done`.
    let list: Vec<Map<String, Value>> = rows(state, &format!("SELECT * FROM {view}"))
        .await
        .into_iter()
        .filter(|p| opts::missing_fields(p).is_empty() && crate::plant_ai::is_approved(p) == done)
        .collect();
    let kind = if sub == 1 { "perennial" } else { "annual" };
    // Nothing to show: a short note.
    if list.is_empty() {
        return h + &ui::info(&match (done, sub) {
            (true, _) => format!("No 100% approved {kind}s found yet."),
            (false, _) => format!("All complete {kind}s have been fully approved."),
        });
    }
    // Awaiting Approval shows who approved; Fully Approved shows size and spacing.
    let cols: &[&str] = if done {
        &[
            "Thumbnail",
            "plant_name",
            "botanical_name",
            "genus",
            "species",
            "plant_length",
            "spacing",
        ]
    } else {
        &["Thumbnail", "plant_name", "botanical_name", "approved_jun"]
    };
    // The picker label.
    let label = if done {
        format!("Edit fully approved {kind}:")
    } else {
        format!("Review & approve {kind}:")
    };
    // Awaiting Approval: the owner's Approve button for the chosen plant, above its editor.
    let between = if done {
        String::new()
    } else {
        approve_button(req, &s(chosen(req, &list), "id"))
    };
    h.push_str(&list_and_edit(state, req, &list, cols, &label, &sub.to_string(), &between).await);
    h
}

/// The Approve button (garden owner only; others see who can approve). The same action as the
/// Add plant review (quick.rs, `pa_approve`).
fn approve_button(req: &Req, id: &str) -> String {
    let l = req.lang;
    if !req.owner {
        return ui::caption(l.t(
            "Only the garden owner can approve.",
            "승인은 정원 주인만 할 수 있어요.",
        ));
    }
    format!(
        "<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"pa_approve\"><input type=\"hidden\" name=\"tab\" value=\"{}\"><input type=\"hidden\" name=\"p\" value=\"{}\"><input type=\"hidden\" name=\"sub\" value=\"{}\">{}</form>",
        req.tab,
        he(id),
        he(req.get("sub")),
        ui::button("go", "1", l.t("✅ Approve", "✅ 승인"), true, false)
    )
}

// ---- ⚠️ Partials (tab_partials.py) -----------------------------------------

/// Tab 6: plants missing required fields. Adds a "Missing Data" column to each row, lists them,
/// offers exports, then shows the editor and the image manager for the chosen plant.
async fn partials(state: &AppState, req: &Req) -> String {
    let mut h = ui::header("⚠️ Incomplete Datasets (Partials)")
        + &ui::text("Plants missing vital fields required for the catalog.");
    // All plants. `retain` then keeps only those with missing fields.
    let mut list = rows(state, "SELECT * FROM data_source").await;
    list.retain(|p| !opts::missing_fields(p).is_empty());
    // Add the "Missing Data" column: the names of the missing fields joined with commas.
    for p in list.iter_mut() {
        let m = opts::missing_fields(p).join(", ");
        p.insert("Missing Data".into(), Value::String(m));
    }
    if list.is_empty() {
        return h + &ui::success("No partials found! Everything is complete.");
    }
    h.push_str(&ui::warning(&format!(
        "Found {} plants missing core data.",
        list.len()
    )));
    // Columns for the table.
    let cols = [
        "id",
        "Thumbnail",
        "plant_name",
        "genus",
        "species",
        "Missing Data",
    ];
    let headers: Vec<&str> = cols.iter().map(|c| header_for(c)).collect();
    let table: Vec<Vec<String>> = list
        .iter()
        .map(|p| cols.iter().map(|c| cell_for(p, c)).collect())
        .collect();
    h.push_str(&ui::dataframe(&headers, &table));
    h.push_str(&ui::divider());
    h.push_str(&ui::subheader("Edit a Partial Plant"));
    let opts_: Vec<(String, String)> = list.iter().map(|p| (s(p, "id"), plant_label(p))).collect();
    let p = chosen(req, &list);
    h.push_str(&picker(
        req,
        "Select plant to manually edit:",
        &opts_,
        &s(p, "id"),
        "",
    ));
    let id = s(p, "id");
    // Three buttons: the HTML export, the missing-fields JSON export and a link to the JSON
    // Pipeline's import tab.
    h.push_str(&ui::columns(
        &[3, 3, 2],
        vec![
            ui::link_button(
                &format!("/streamlit/?export=html&kind=partial&id={id}"),
                "📄 Export as HTML",
            ),
            ui::link_button(
                &format!("/streamlit/?export=missing_json&id={id}"),
                "📋 Export Missing Fields as JSON",
            ),
            ui::link_button("/streamlit/?tab=11&sub=4", "📥 Go to JSON Import →"),
        ],
    ));
    h.push_str(&ui::info("💡 The JSON export contains **all current data** for this plant, with missing fields flagged at the top under `_missing_fields`. Review or correct any values, then head to **🔄 JSON Pipeline → 📥 Phase 3 — Import & Review** to bring it back in."));
    // Then the editor, then the image manager.
    h.push_str(&ui::divider());
    h.push_str(&editor(state, req, p, "").await);
    h.push_str(&ui::divider());
    h.push_str(&ui::subheader("📸 Image Management"));
    let back = vec![("tab", req.tab.to_string()), ("p", s(p, "id"))];
    h.push_str(&super::gallery::render(
        state,
        p,
        &back,
        req.get("wipe") == "1",
    ));
    h
}

// ---- 📸 Manual Image Replace (tab_image_replace.py) -------------------------

/// The five choices for "Show plants:" in the image tab.
const SHOW_MODES: [&str; 5] = [
    "All plants",
    "Unverified images",
    "Verified images",
    "No image yet",
    "Only one image",
];

/// Tab 5. A radio chooses which plants to list, a text box filters by name, family or botanical
/// name, and a drop-down picks a plant. The image manager (gallery.rs) is shown for that plant.
async fn image_replace(state: &AppState, req: &Req) -> String {
    let mut h = ui::header("📸 Manual Image Replacement Gallery");
    // The chosen mode from the address, or "All plants" if it is unknown.
    let mode = SHOW_MODES
        .iter()
        .find(|m| **m == req.get("show"))
        .copied()
        .unwrap_or("All plants");
    let filter = req.get("filter");
    // A GET form: changing the radio sends it at once.
    let mut top = format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"{}\">", req.tab);
    top.push_str(&ui::columns(
        &[2, 3],
        vec![
            ui::radio("show", "Show plants:", &SHOW_MODES, mode, false, true),
            ui::text_input(
                "filter",
                "🔍 Filter by name / family / botanical name:",
                filter,
                "Start typing…",
            ),
        ],
    ));
    top.push_str("</form>");
    h.push_str(&top);
    // The columns read from the database.
    let base = "SELECT id, plant_name, botanical_name, picture, additional_pictures, image_search_term, family, zones, image_verified FROM data_source";
    // The SQL filter for each mode. "None" and empty text both mean no photo. "Only one image"
    // means there is no comma in `picture` and no extra pictures. Only fixed text is used here.
    let where_ = match mode {
        "Verified images" => " WHERE image_verified = 1",
        "Unverified images" => " WHERE picture IS NOT NULL AND TRIM(picture) != '' AND picture != 'None' AND (image_verified = 0 OR image_verified IS NULL)",
        "No image yet" => " WHERE picture IS NULL OR TRIM(picture) = '' OR picture = 'None'",
        "Only one image" => " WHERE picture IS NOT NULL AND TRIM(picture) != '' AND picture != 'None' AND picture NOT LIKE '%,%' AND (additional_pictures IS NULL OR TRIM(additional_pictures) = '' OR additional_pictures = 'None')",
        _ => "",
    };
    // The filter text, lower case.
    let term = filter.trim().to_lowercase();
    // Load, then keep plants where the filter appears in the name, botanical name or family.
    let list: Vec<Map<String, Value>> =
        rows(state, &format!("{base}{where_} ORDER BY plant_name ASC"))
            .await
            .into_iter()
            .filter(|p| {
                term.is_empty()
                    || s(p, "plant_name").to_lowercase().contains(&term)
                    || s(p, "botanical_name").to_lowercase().contains(&term)
                    || s(p, "family").to_lowercase().contains(&term)
            })
            .collect();
    if list.is_empty() {
        return h + &ui::info("No plants match that filter.");
    }
    // A small mark for the picker: verified, has a photo, or none.
    let status = |p: &Map<String, Value>| {
        if matches!(s(p, "image_verified").trim(), "1" | "true") {
            "✅"
        } else if !matches!(s(p, "picture").trim(), "" | "None") {
            "📷"
        } else {
            "❌ no image"
        }
    };
    // The picker choices (the trailing `_` avoids a clash with the name `opts`).
    let opts_: Vec<(String, String)> = list
        .iter()
        .map(|p| {
            (
                s(p, "id"),
                format!("{}  [ID {}]  {}", s(p, "plant_name"), s(p, "id"), status(p)),
            )
        })
        .collect();
    let p = chosen(req, &list);
    // The picker form keeps the mode and filter as hidden fields.
    let mut pick = format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"{}\"><input type=\"hidden\" name=\"show\" value=\"{}\"><input type=\"hidden\" name=\"filter\" value=\"{}\">", req.tab, he(mode), he(filter));
    pick.push_str(&ui::selectbox(
        "p",
        &format!("Select a plant ({} shown):", list.len()),
        &opts_,
        &s(p, "id"),
        true,
    ));
    pick.push_str("</form>");
    h.push_str(&pick);
    h.push_str(&ui::divider());
    // The settings the image buttons send back.
    let back = vec![
        ("tab", req.tab.to_string()),
        ("p", s(p, "id")),
        ("show", mode.to_string()),
        ("filter", filter.to_string()),
    ];
    h.push_str(&super::gallery::render(
        state,
        p,
        &back,
        req.get("wipe") == "1",
    ));
    h
}

// ---- ✍️ Manual Editor (tab_manual_editor.py) -------------------------------

/// Tab 4: pick any plant and edit it. Also lists other plants with the same botanical name (or the
/// same genus if there is no botanical name) and a button to export the plant as HTML.
async fn manual_editor(state: &AppState, req: &Req) -> String {
    let mut h = ui::header("✍️ Global Manual Editor")
        + &ui::text("Instantly find and edit any plant manually. No AI involved.");
    // All plants by name.
    let list = rows(state, "SELECT * FROM data_source ORDER BY plant_name ASC").await;
    if list.is_empty() {
        return h;
    }
    let opts_: Vec<(String, String)> = list.iter().map(|p| (s(p, "id"), plant_label(p))).collect();
    let p = chosen(req, &list);
    h.push_str(&picker(
        req,
        "Search database for a plant to edit:",
        &opts_,
        &s(p, "id"),
        "",
    ));
    // The plant id as a number.
    let id = crate::php::php_int(&s(p, "id"));
    // The botanical name and genus.
    let (b, g) = (s(p, "botanical_name"), s(p, "genus"));
    // Related plants: same botanical name, else same genus, never the plant itself. Each result is
    // (id, name, beds). `query_as` reads rows into a tuple, `.bind` fills the `?`.
    let matches: Vec<(i64, String, Option<String>)> = if !b.trim().is_empty() {
        sqlx::query_as("SELECT id, plant_name, garden_beds FROM data_source WHERE botanical_name = ? AND id != ?")
            .bind(&b).bind(id).fetch_all(&state.db).await.unwrap_or_default()
    } else if !g.trim().is_empty() {
        sqlx::query_as(
            "SELECT id, plant_name, garden_beds FROM data_source WHERE genus = ? AND id != ?",
        )
        .bind(&g)
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };
    h.push_str(&ui::divider());
    h.push_str(&ui::subheader("🌿 Related Plants in Database"));
    // The text for the read-only box: one line per plant with its bed, or "To Be Sorted" if none.
    let text_ = if matches.is_empty() {
        "No other plants found with this Botanical Name or Genus.".to_string()
    } else {
        matches
            .iter()
            .map(|(i, n, beds)| {
                format!(
                    "• {n} (ID: {i}) - Location: {}",
                    beds.as_deref()
                        .filter(|b| !b.is_empty())
                        .unwrap_or("To Be Sorted")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    h.push_str(&format!("<label class=\"st-field\"><span class=\"st-label\">Botanical Matches (Read Only)</span><textarea class=\"st-input\" style=\"height:100px\" disabled>{}</textarea></label>", he(&text_)));
    h.push_str(&ui::subheader("✏️ Edit Active Plant"));
    // The export link, made full width.
    h.push_str(&format!(
        "<div>{}</div>",
        ui::link_button(
            &format!("/streamlit/?export=html&kind=manual&id={id}"),
            "📄 Export current data as HTML (open in browser → Print → Save as PDF)"
        )
        .replace("class=\"st-btn\"", "class=\"st-btn st-full\"")
    ));
    h.push_str(&ui::divider());
    h.push_str(&editor(state, req, p, "").await);
    h
}

// ---- 🗺️ Bed Assignment (tab_bed_assign.py) ---------------------------------

/// Tab 12: find a plant (optional filter), then pick its garden beds from a multi-pick list. The
/// form posts `save_beds`. Beds that are saved on the plant but are not in the standard list are
/// kept in hidden fields so they are not lost.
async fn bed_assign(state: &AppState, req: &Req) -> String {
    let mut h = ui::header("🗺️ Garden Bed Assignment")
        + &ui::markdown("Find any plant, then assign or reassign it to one or more garden beds using the multiselect below.  Changes are saved directly to the database.");
    // Load the plants with the columns this tab needs.
    let all = rows(state, "SELECT id, plant_name, botanical_name, family, garden_beds, flower_bed, picture FROM data_source ORDER BY plant_name").await;
    if all.is_empty() {
        return h + &ui::warning("No plants found in the database.");
    }
    // The filter text.
    let term = req.get("filter").trim().to_lowercase();
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"{}\">{}</form>",
        req.tab, ui::text_input("filter", "🔍 Filter plants:", req.get("filter"), "Type a name, family, or botanical name…")));
    // Apply the filter to name, botanical name and family.
    let list: Vec<Map<String, Value>> = all
        .into_iter()
        .filter(|p| {
            term.is_empty()
                || s(p, "plant_name").to_lowercase().contains(&term)
                || s(p, "botanical_name").to_lowercase().contains(&term)
                || s(p, "family").to_lowercase().contains(&term)
        })
        .collect();
    if list.is_empty() {
        return h + &ui::info("No plants match that search.");
    }
    // The picker choices.
    let opts_: Vec<(String, String)> = list
        .iter()
        .map(|p| {
            (
                s(p, "id"),
                format!("{}  [ID {}]", s(p, "plant_name"), s(p, "id")),
            )
        })
        .collect();
    let p = chosen(req, &list);
    // The picker form keeps the filter.
    let mut pick = format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"{}\"><input type=\"hidden\" name=\"filter\" value=\"{}\">", req.tab, he(req.get("filter")));
    pick.push_str(&ui::selectbox(
        "p",
        &format!("Select a plant ({} shown):", list.len()),
        &opts_,
        &s(p, "id"),
        true,
    ));
    pick.push_str("</form>");
    h.push_str(&pick);
    h.push_str(&ui::divider());

    // The plant's photo names.
    let pics: Vec<String> = s(p, "picture")
        .split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty() && !matches!(x.to_lowercase().as_str(), "none" | "null"))
        .collect();
    // The first photo, or a note if there is none or the file is missing.
    let mut info = match pics.first() {
        Some(f) if state.www.join("images/Database").join(f).is_file() => format!(
            "<img src=\"/images/Database/{}\" width=\"200\" alt=\"\">",
            ui::path_url(f)
        ),
        Some(_) => ui::caption("📷 Image file not found on disk"),
        None => ui::caption("📷 No image uploaded yet"),
    };
    // A closure that shows a dash for empty values.
    let or_dash = |v: String| {
        if v.trim().is_empty() {
            "—".to_string()
        } else {
            v
        }
    };
    info.push_str(&ui::markdown(&format!("**ID:** {}", s(p, "id"))));
    info.push_str(&ui::markdown(&format!(
        "**Botanical:** {}",
        or_dash(s(p, "botanical_name"))
    )));
    info.push_str(&ui::markdown(&format!(
        "**Family:** {}",
        or_dash(s(p, "family"))
    )));
    // The old `flower_bed` value is shown if it has one.
    let legacy = s(p, "flower_bed");
    if !legacy.trim().is_empty() && !matches!(legacy.to_lowercase().as_str(), "none" | "null") {
        info.push_str(&ui::markdown(&format!("**Legacy flower_bed:** `{legacy}`")));
    }
    // The plant's saved beds: remove brackets and quotes, split at commas, drop empty items and
    // "none".
    let current: Vec<String> = s(p, "garden_beds")
        .replace(['[', ']', '\'', '"'], "")
        .split(',')
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty() && !matches!(b.to_lowercase().as_str(), "none" | "null"))
        .collect();
    // Saved beds that are in the standard list: shown as picked.
    let valid: Vec<String> = current
        .iter()
        .filter(|b| opts::GARDEN_BEDS.contains(&b.as_str()))
        .cloned()
        .collect();
    // Saved beds not in the standard list: kept in hidden fields.
    let extra: Vec<String> = current
        .iter()
        .filter(|b| !opts::GARDEN_BEDS.contains(&b.as_str()))
        .cloned()
        .collect();
    // The standard list as text.
    let beds: Vec<String> = opts::GARDEN_BEDS.iter().map(|b| b.to_string()).collect();
    // The multi-pick box. The label is replaced to add a "?" help badge.
    let mut bed_col = ui::multiselect("beds[]", "📍 Assign to garden bed(s):", &beds, &valid)
        .replace("<span class=\"st-label\">📍 Assign to garden bed(s):</span>", "<span class=\"st-label\">📍 Assign to garden bed(s): <span class=\"st-help\" title=\"Select one or more beds. Hold Ctrl / Cmd to pick multiple.\">?</span></span>");
    // Tell the person about extra values.
    if !extra.is_empty() {
        bed_col.push_str(&ui::caption(&format!(
            "ℹ️ Also has legacy bed value(s) not in the standard list: {}",
            extra
                .iter()
                .map(|b| format!("`{b}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    // The save form (multipart, as the multi-pick box needs).
    let mut form_ = format!("<form method=\"post\" action=\"/streamlit/\" enctype=\"multipart/form-data\"><input type=\"hidden\" name=\"act\" value=\"save_beds\"><input type=\"hidden\" name=\"tab\" value=\"{}\"><input type=\"hidden\" name=\"p\" value=\"{}\"><input type=\"hidden\" name=\"filter\" value=\"{}\">",
        req.tab, he(&s(p, "id")), he(req.get("filter")));
    // One hidden field for each extra bed.
    for e in &extra {
        form_.push_str(&format!(
            "<input type=\"hidden\" name=\"extra[]\" value=\"{}\">",
            he(e)
        ));
    }
    // Info on the left, bed picker on the right, then the Save button.
    form_.push_str(&ui::columns(&[1, 2], vec![info, bed_col]));
    form_.push_str(&ui::divider());
    form_.push_str(&ui::button(
        "save",
        "1",
        "💾 Save Bed Assignment",
        true,
        true,
    ));
    form_.push_str("</form>");
    h.push_str(&form_);
    h
}

/// "💾 Save Bed Assignment". Also writes `flower_bed`, which the website
/// reads; the Streamlit version did not, so the site drifted (S-3).
/// `f` is the submitted form. Joins the picked beds and the hidden extra ones, saves them to both
/// columns, and returns a success or error message.
async fn save_beds(state: &AppState, f: &crate::formdata::FormData) -> Vec<Flash> {
    // The plant id.
    let id = crate::php::php_int(f.str("p"));
    // Picked beds. The multi-pick box sends them as one text joined with ", ", so it is split
    // there.
    let mut combined: Vec<String> = f
        .get("beds[]")
        .unwrap_or("")
        .split(", ")
        .map(str::trim)
        .filter(|b| !b.is_empty())
        .map(String::from)
        .collect();
    // Add the hidden extra beds.
    combined.extend(
        f.get("extra[]")
            .unwrap_or("")
            .split(", ")
            .map(str::trim)
            .filter(|b| !b.is_empty())
            .map(String::from),
    );
    // No beds means "To Be Sorted".
    let val = if combined.is_empty() {
        "To Be Sorted".to_string()
    } else {
        combined.join(", ")
    };
    // The plant's name, for the message.
    let name: String = sqlx::query_scalar("SELECT plant_name FROM data_source WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    // Save to garden_beds and flower_bed. `.bind` fills each `?` in order.
    match sqlx::query("UPDATE data_source SET garden_beds = ?, flower_bed = ? WHERE id = ?")
        .bind(&val)
        .bind(&val)
        .bind(id)
        .execute(&state.db)
        .await
    {
        Ok(_) => {
            // Clear the site's cache so public pages update.
            state.invalidate_widget_cache();
            // The beds in bold for the message.
            let shown = if combined.is_empty() {
                "**To Be Sorted**".to_string()
            } else {
                combined
                    .iter()
                    .map(|b| format!("**{b}**"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            vec![Flash::Success(format!(
                "✅ **{name}** assigned to: {shown}"
            ))]
        }
        Err(e) => vec![Flash::Error(format!("Save failed: {e}"))],
    }
}
