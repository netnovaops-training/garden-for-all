//! "🌟 Wishlist" (tab_wishlist.py) and "🏷️ Barcodes" (tab_barcode.py +
//! barcode_lookup.py).
//!
//! HOW TO READ THIS FILE (for Jun). The wishlist is the `future_plants`
//! table: plants you don't own yet. Pick one to see what is missing, edit
//! its fields, promote it into the garden (a copy goes into data_source and
//! the wishlist row stays, linked by `future_plant_id`), or remove it.
//! Barcodes looks up UPC codes on UPCitemdb's free trial API (100 a day),
//! remembering every answer in `barcode_cache.json` in the data folder so
//! the same plant never costs a second request. That cache is the same file
//! plant_actions.php's lookup uses.
//!
//! DIFFERENCES FROM STREAMLIT (DEVIATIONS.md): "Yes, promote anyway" works
//! here (S-6; in Streamlit it was a button inside a button and never fired),
//! and "Remove from Wishlist" asks before deleting (S-7).
//!
//! WHERE IT IS USED. `tabs::render` calls `wishlist` for tab 13 and `barcodes` for tab 14.
//! `tabs::action` sends button names starting "wl_" to `action` and "bc_" to `barcode_action`.
//!
//! RUST WORDS USED HERE. `async fn` can pause while it waits for the database or the network;
//! callers write `.await`. `Option<T>` is `Some(value)` or `None`. `Result<T, E>` is `Ok(value)` or
//! `Err(error)`. `?` after a call means "if this failed, return the failure from this function
//! now". `match` picks the first arm whose shape fits. A closure is a small inline function written
//! `|x| ...`. An `enum` is a type that is exactly one of several named choices. `&str` is borrowed
//! text; `String` is owned text.

// `Map` is a JSON object (one database row) and `Value` is any JSON value.
use serde_json::{Map, Value};

// `Flash` is a message shown after an action (Success, Info, Warning or Error).
use super::form::Flash;
use super::opts::{self, text};
use super::ui;
use super::{rows, Req};
use crate::php::{he, php_int};
use crate::AppState;

/// Short helper: field `k` of the row as plain text (empty if missing).
fn s(m: &Map<String, Value>, k: &str) -> String {
    text(m.get(k))
}

/// Hidden form fields that carry the tab, the chosen plant and the filter text, so a button sends
/// the person back to the same view.
fn back_fields(req: &Req, id: &str) -> String {
    format!("<input type=\"hidden\" name=\"tab\" value=\"{}\"><input type=\"hidden\" name=\"p\" value=\"{}\"><input type=\"hidden\" name=\"filter\" value=\"{}\">",
        req.tab, he(id), he(req.get("filter")))
}

/// Tab 13. Shows a count, a name filter, a plant picker, a summary and a list of missing key
/// fields, an "Edit Fields" box, the Promote and Remove buttons (each with a confirmation step),
/// and the full table. Returns the tab's HTML.
pub async fn wishlist(state: &AppState, req: &Req) -> String {
    // Heading and caption.
    let mut h = ui::header("🌟 Wishlist / Future Plants") + &ui::caption("Browse, edit, and promote plants from the future_plants table into your garden.");
    // Every wishlist plant by name.
    let all = rows(state, "SELECT * FROM future_plants ORDER BY plant_name").await;
    if all.is_empty() {
        return h + &ui::info("No plants in the wishlist. Add plants via the INSERT SQL or web scraping.");
    }
    // A card with the total.
    h.push_str(&ui::metric("Wishlist Plants", &all.len().to_string()));
    // The filter text, lower case.
    let term = req.get("filter").trim().to_lowercase();
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"{}\">{}</form>", req.tab, ui::text_input("filter", "🔍 Filter by name", req.get("filter"), "")));
    // Plants whose name, botanical name or company contains the filter text. These are references
    // (`&`) into `all`, so nothing is copied.
    let list: Vec<&Map<String, Value>> = all.iter().filter(|p| term.is_empty()
        || s(p, "plant_name").to_lowercase().contains(&term) || s(p, "botanical_name").to_lowercase().contains(&term) || s(p, "company").to_lowercase().contains(&term)).collect();
    if !term.is_empty() {
        h.push_str(&ui::caption(&format!("{} match(es)", list.len())));
    }
    if list.is_empty() {
        return h;
    }
    // The picker choices: name, company and id.
    let opts_: Vec<(String, String)> = list.iter().map(|p| (s(p, "id"), format!("{}  ({})  [ID {}]", s(p, "plant_name"), s(p, "company"), s(p, "id")))).collect();
    // The plant chosen in the address, or the first one. `.copied()` turns a reference to a
    // reference into a plain reference.
    let fp = list.iter().find(|p| s(p, "id") == req.get("p")).copied().unwrap_or(list[0]);
    let id = s(fp, "id");
    h.push_str(&format!("<form method=\"get\" action=\"/streamlit/\"><input type=\"hidden\" name=\"tab\" value=\"{}\"><input type=\"hidden\" name=\"filter\" value=\"{}\">{}</form>",
        req.tab, he(req.get("filter")), ui::selectbox("p", "Select a plant", &opts_, &id, true)));

    // Python's fp.get(k, '—') printed an empty (NULL) column as "None".
    // Show a value the way the Python app did: a column that is missing shows a dash, an empty
    // (NULL) column shows "None".
    let shown = |k: &str| match fp.get(k) { None => "—".to_string(), Some(Value::Null) => "None".to_string(), Some(_) => s(fp, k) };
    // Left side: the name and four facts.
    let left = ui::subheader(&s(fp, "plant_name"))
        + &ui::markdown(&format!("**Botanical:** {}", shown("botanical_name")))
        + &ui::markdown(&format!("**Company:** {}", shown("company")))
        + &ui::markdown(&format!("**Family:** {}", shown("family")))
        + &ui::markdown(&format!("**Added:** {}", shown("date_added")));
    // The key fields that should be filled in.
    let keys = ["botanical_name", "company", "family", "genus", "species", "plant_type", "zones", "sun_exposure", "water_needs", "bloom_color", "foliage_color", "size"];
    // The key fields that are empty.
    let missing: Vec<&str> = keys.iter().filter(|k| s(fp, k).is_empty()).copied().collect();
    // Right side: a green note if nothing is missing, else a yellow note naming what is.
    let right = if missing.is_empty() { ui::success("All key fields populated") } else { ui::warning(&format!("Missing {} key fields: {}", missing.len(), missing.join(", "))) };
    h.push_str(&ui::columns(&[1, 1], vec![left, right]));

    // ✏️ Edit Fields
    // A closure that draws a drop-down from a list. If the current value is not in the list, the
    // first choice is used.
    let sel = |name: &str, label: &str, list: &[&str], cur: &str| {
        let opts: Vec<String> = list.iter().map(|x| x.to_string()).collect();
        let pick = if list.contains(&cur) { cur.to_string() } else { list[0].to_string() };
        ui::select_plain(name, label, &opts, &pick)
    };
    // The edit form posts `wl_save`.
    let mut edit = format!("<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"wl_save\"><input type=\"hidden\" name=\"id\" value=\"{}\">{}", he(&id), back_fields(req, &id));
    // A closure for a text box filled from the plant.
    let t = |k: &str, l: &str| ui::text_input(k, l, &s(fp, k), "");
    // Two columns of editable fields. `+` joins the pieces of HTML.
    edit.push_str(&ui::columns(&[1, 1], vec![
        t("plant_name", "Plant Name") + &t("botanical_name", "Botanical Name") + &t("company", "Company") + &t("family", "Family") + &t("genus", "Genus") + &t("species", "Species") + &t("plant_type", "Plant Type"),
        t("zones", "Zones") + &sel("sun_exposure", "Sun Exposure", opts::SUN_OPTIONS, &s(fp, "sun_exposure")) + &t("water_needs", "Water Needs") + &t("size", "Size") + &t("bloom_color", "Bloom Color") + &t("foliage_color", "Foliage Color")
            + &sel("deer_resistance", "Deer Resistance", opts::RESISTANCE_OPTIONS, &s(fp, "deer_resistance")),
    ]));
    // The bed choices with a blank one first (`once("")` makes a one-item list, `chain` adds the
    // rest).
    let beds: Vec<&str> = std::iter::once("").chain(opts::GARDEN_BEDS.iter().copied()).collect();
    edit.push_str(&sel("bed", "Garden Bed", &beds, &s(fp, "flower_bed")));
    edit.push_str(&ui::button("save", "1", "💾 Save Changes", false, false));
    edit.push_str("</form>");
    // Put the form in a box that is closed at first.
    h.push_str(&ui::expander("✏️ Edit Fields", false, &edit));
    h.push_str(&ui::divider());

    // Promote / Remove (with the confirmations, see the file header)
    // Is a plant with this name already in the garden table? The count is more than zero if so.
    let in_garden: bool = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM data_source WHERE plant_name = ?")
        .bind(s(fp, "plant_name").trim()).fetch_one(&state.db).await.unwrap_or(0) > 0;
    // `confirm=promote` or `confirm=delete` in the address means a confirmation is being shown.
    let confirm = req.get("confirm");
    // A closure that builds a link back to this view with an extra setting added.
    let link = |extra: &str| format!("/streamlit/?tab={}&p={}&filter={}{extra}", req.tab, crate::php::urlencode(&id), crate::php::urlencode(req.get("filter")));
    // A closure that builds a tiny form with one button that posts the action `act`.
    let post = |act: &str, label: &str, primary: bool| format!("<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"{act}\"><input type=\"hidden\" name=\"id\" value=\"{}\">{}{}</form>", he(&id), back_fields(req, &id), ui::button("go", "1", label, primary, false));
    // If the plant is already in the garden, "Promote" only reloads the page with a warning
    // (nothing is copied yet). Otherwise it promotes at once.
    let mut col1 = if in_garden {
        format!("<a class=\"st-btn st-primary\" href=\"{}\">🌱 Promote to Garden</a>", he(&link("&confirm=promote")))
    } else {
        post("wl_promote", "🌱 Promote to Garden", true)
    };
    // The warning plus a "Yes, promote anyway" button, which posts `wl_promote_force`.
    if confirm == "promote" {
        col1.push_str(&ui::warning(&format!("⚠️ '{}' already exists in data_source! Promote anyway?", s(fp, "plant_name"))));
        col1.push_str(&post("wl_promote_force", "Yes, promote anyway", false));
    }
    // "Remove" first only reloads with `confirm=delete`.
    let mut col2 = format!("<a class=\"st-btn\" href=\"{}\">🗑️ Remove from Wishlist</a>", he(&link("&confirm=delete")));
    // Ask, then the "Yes, remove" button posts `wl_delete`.
    if confirm == "delete" {
        col2.push_str(&ui::warning(&format!("Remove **{}** from the wishlist? This cannot be undone.", s(fp, "plant_name"))));
        col2.push_str(&post("wl_delete", "✅ Yes, remove", true));
    }
    // The two buttons in the first two columns; the third stays empty.
    h.push_str(&ui::columns(&[1, 1, 1], vec![col1, col2, String::new()]));
    h.push_str(&ui::divider());
    // The full table of all wishlist plants, in a box that is closed at first.
    let cols = ["id", "plant_name", "botanical_name", "company", "family", "plant_type", "zones", "sun_exposure", "date_added"];
    let table: Vec<Vec<String>> = all.iter().map(|p| cols.iter().map(|c| ui::cell(&s(p, c))).collect()).collect();
    h.push_str(&ui::expander("📋 Full Wishlist Table", false, &ui::dataframe(&cols, &table)));
    h
}

/// Columns both tables have, for copying a wishlist row into the garden.
/// Names of the columns that exist in both `future_plants` and `data_source`, except `id` and
/// `date_added`, in table order. The SQL joins the database's own column list for the two tables
/// (`information_schema.COLUMNS`). It keeps the copy working if a table gains a column.
async fn shared_columns(state: &AppState) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT f.COLUMN_NAME FROM information_schema.COLUMNS f
         JOIN information_schema.COLUMNS d ON d.TABLE_SCHEMA = f.TABLE_SCHEMA AND d.TABLE_NAME = 'data_source' AND d.COLUMN_NAME = f.COLUMN_NAME
         WHERE f.TABLE_SCHEMA = DATABASE() AND f.TABLE_NAME = 'future_plants' AND f.COLUMN_NAME NOT IN ('id', 'date_added')
         ORDER BY f.ORDINAL_POSITION",
    ).fetch_all(&state.db).await.unwrap_or_default()
}

/// `promote_future_plant`: copy the wishlist row into data_source (the
/// wishlist row is kept), bed "To Be Sorted", linked by future_plant_id.
/// `fp_id` is the wishlist row, `force` skips the "already in garden" check. Returns the new garden
/// row's id, or an error message. Steps: find the plant, make sure `data_source` has the
/// `future_plant_id` link column (adding it if not), refuse if already linked or same name (unless
/// forced), then copy the shared columns with one `INSERT ... SELECT`.
pub async fn promote(state: &AppState, fp_id: i64, force: bool) -> Result<i64, String> {
    // The wishlist plant's name. `map_err` turns a database error into text; `?` returns it at
    // once.
    let name: Option<String> = sqlx::query_scalar("SELECT plant_name FROM future_plants WHERE id = ?").bind(fp_id).fetch_optional(&state.db).await.map_err(|e| e.to_string())?.flatten();
    // No such wishlist row: return an error.
    let Some(name) = name else { return Err("Future plant not found".into()) };
    // Count columns named `future_plant_id` in the garden table (0 or 1).
    let has_link: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'data_source' AND COLUMN_NAME = 'future_plant_id'")
        .fetch_one(&state.db).await.unwrap_or(0);
    // Add the link column and an index for it the first time.
    if has_link == 0 {
        sqlx::query("ALTER TABLE data_source ADD COLUMN future_plant_id INT NULL, ADD INDEX idx_future_plant_id (future_plant_id)").execute(&state.db).await.map_err(|e| e.to_string())?;
    }
    // Look for the plant already in the garden: first by the link, then by the same name. If found,
    // report where it lives (its bed).
    if !force {
        let by_link: Option<(Option<String>,)> = sqlx::query_as("SELECT flower_bed FROM data_source WHERE future_plant_id = ? LIMIT 1").bind(fp_id).fetch_optional(&state.db).await.map_err(|e| e.to_string())?;
        let (hit, how) = match by_link {
            Some(b) => (Some(b), "link"),
            None => (sqlx::query_as("SELECT flower_bed FROM data_source WHERE plant_name = ? LIMIT 1").bind(&name).fetch_optional(&state.db).await.map_err(|e| e.to_string())?, "name"),
        };
        if let Some((bed,)) = hit {
            return Err(format!("{} is already in the garden ({}), matched by {how}", if name.is_empty() { "That plant" } else { &name }, bed.filter(|b| !b.is_empty()).unwrap_or_else(|| "no bed".into())));
        }
    }
    // The columns to copy: all shared ones except the bed columns and the link, which are set
    // separately.
    let cols: Vec<String> = shared_columns(state).await.into_iter().filter(|c| !matches!(c.as_str(), "flower_bed" | "garden_beds" | "future_plant_id")).collect();
    // The column names in backticks, joined by commas. These come from the database itself, not
    // from a visitor.
    let list = cols.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ");
    // Copy the row: the shared columns, plus bed "To Be Sorted" in both bed columns and the link
    // back to the wishlist id. The wishlist row stays.
    let sql = format!("INSERT INTO data_source ({list}, flower_bed, garden_beds, future_plant_id) SELECT {list}, 'To Be Sorted', 'To Be Sorted', id FROM future_plants WHERE id = ?");
    // Run it. `?` returns an error message on failure.
    let r = sqlx::query(&sql).bind(fp_id).execute(&state.db).await.map_err(|e| e.to_string())?;
    // Tell the public pages to rebuild.
    state.invalidate_widget_cache();
    // The id of the row just created.
    Ok(r.last_insert_id() as i64)
}

/// Runs a wishlist button: `wl_save`, `wl_promote`, `wl_promote_force` or `wl_delete`. `f` is the
/// submitted form. Returns messages to show.
pub async fn action(state: &AppState, act: &str, f: &crate::formdata::FormData) -> Vec<Flash> {
    // The wishlist plant id.
    let id = php_int(f.str("id"));
    // Its name, for messages.
    let name: String = sqlx::query_scalar("SELECT plant_name FROM future_plants WHERE id = ?").bind(id).fetch_optional(&state.db).await.ok().flatten().flatten().unwrap_or_default();
    match act {
        // Save the edit form. Empty boxes are saved as empty (NULL) values.
        "wl_save" => {
            // A list of (column, value) pairs. The column names are fixed in the code, never from
            // the visitor.
            let mut sets: Vec<(&str, Option<String>)> = ["plant_name", "botanical_name", "company", "family", "genus", "species", "plant_type", "zones", "sun_exposure", "water_needs", "size", "bloom_color", "foliage_color", "deer_resistance"]
                .iter().map(|k| (*k, Some(f.str(k).to_string()).filter(|v| !v.is_empty()))).collect();
            // A chosen bed is written to both bed columns.
            if !f.str("bed").is_empty() {
                sets.push(("flower_bed", Some(f.str("bed").to_string())));
                sets.push(("garden_beds", Some(f.str("bed").to_string())));
            }
            // Build `UPDATE ... SET col = ?, ...`. Values are bound separately, in the same order.
            let sql = format!("UPDATE future_plants SET {} WHERE id = ?", sets.iter().map(|(k, _)| format!("`{k}` = ?")).collect::<Vec<_>>().join(", "));
            // Bind each value in turn.
            let mut q = sqlx::query(&sql);
            for (_, v) in &sets {
                q = q.bind(v.clone());
            }
            // The last `?` is the row id. Show success or a plain failure message.
            match q.bind(id).execute(&state.db).await {
                Ok(_) => vec![Flash::Success(format!("Updated {}", f.str("plant_name")))],
                Err(_) => vec![Flash::Error("Failed to update".into())],
            }
        }
        // Promote. The forced version has a shorter message.
        "wl_promote" | "wl_promote_force" => match promote(state, id, act == "wl_promote_force").await {
            Ok(new_id) if act == "wl_promote_force" => vec![Flash::Success(format!("✅ Promoted! New ID: {new_id}"))],
            Ok(new_id) => vec![Flash::Success(format!("✅ '{name}' promoted to garden! New ID: {new_id}"))],
            Err(e) => vec![Flash::Error(format!("Failed: {e}"))],
        },
        // Delete the wishlist row.
        "wl_delete" => match sqlx::query("DELETE FROM future_plants WHERE id = ?").bind(id).execute(&state.db).await {
            Ok(_) => vec![Flash::Success(format!("Removed {name} from wishlist"))],
            Err(_) => vec![Flash::Error("Failed to delete".into())],
        },
        _ => vec![Flash::Error("Unknown action.".into())],
    }
}

// Barcodes: UPC lookups with a local cache.
// ---- 🏷️ Barcodes ------------------------------------------------------------

/// Where the barcode cache file lives: `barcode_cache.json` in the data folder.
fn cache_path(state: &AppState) -> std::path::PathBuf {
    state.data.join("barcode_cache.json")
}

/// Reads the cache file as JSON. If the file is missing, damaged, or has no `lookups` object, a
/// fresh empty cache is used. Shape: `{"lookups": {key: result}, "last_updated": time}`.
fn load_cache(state: &AppState) -> Value {
    std::fs::read(cache_path(state)).ok().and_then(|b| serde_json::from_slice(&b).ok())
        .filter(|v: &Value| v.get("lookups").map(Value::is_object).unwrap_or(false))
        .unwrap_or_else(|| serde_json::json!({ "lookups": {}, "last_updated": null }))
}

/// Stamps `last_updated` with the current time in the site's time zone and writes the cache file,
/// indented. A failed write is ignored because the cache is only a convenience. `&mut Value` means
/// the function may change the value it borrows.
fn save_cache(state: &AppState, cache: &mut Value) {
    let now = common::now_local();
    cache["last_updated"] = Value::String(format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), u8::from(now.month()), now.day(), now.hour(), now.minute(), now.second()));
    if let Ok(b) = serde_json::to_vec_pretty(cache) {
        let _ = std::fs::write(cache_path(state), b);
    }
}

/// Makes the cache key from name and company: lower case, letters and digits only, joined with "|".
/// The same key is made by the older PHP lookup, so both share the file.
fn cache_key(name: &str, company: &str) -> String {
    // A closure that keeps only a-z and 0-9.
    let clean = |s: &str| s.to_lowercase().chars().filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit()).collect::<String>();
    format!("{}|{}", clean(name), clean(company))
}

/// The result of one barcode search.
enum Upc {
    /// A barcode was found: its number, the product title and the brand.
    Found { upc: String, title: String, brand: String },
    /// The service has nothing for this plant.
    NotFound,
    /// The free daily limit was reached; stop asking.
    RateLimited,
    /// The request failed; the text says why.
    Error(String),
}

/// Asks UPCitemdb's free "trial" search for a product named `name` (and made by `company`, when
/// given). Picks the first item whose title contains the name, or else the first item. Uses its EAN
/// code, or its UPC if there is no EAN.
async fn search_upc(state: &AppState, name: &str, company: &str) -> Upc {
    // The query settings. `match_mode=0` is a loose match.
    let mut q = vec![("s", name.to_string()), ("match_mode", "0".into()), ("type", "product".into())];
    if !company.is_empty() {
        q.push(("brand", company.to_string()));
    }
    // Send the request with a 10 second limit.
    let r = state.http.get("https://api.upcitemdb.com/prod/trial/search").query(&q)
        .header("Accept", "application/json").header("key_type", "3scale")
        .timeout(std::time::Duration::from_secs(10)).send().await;
    // A network failure becomes `Upc::Error`.
    let r = match r { Ok(r) => r, Err(e) => return Upc::Error(e.to_string()) };
    // HTTP 429 means "too many requests".
    if r.status().as_u16() == 429 {
        return Upc::RateLimited;
    }
    // Any other failure: treat as nothing found.
    if !r.status().is_success() {
        return Upc::NotFound;
    }
    // Read the answer as JSON.
    let Ok(data) = r.json::<Value>().await else { return Upc::NotFound };
    // The list of products.
    let items = data.get("items").and_then(Value::as_array).cloned().unwrap_or_default();
    if items.is_empty() {
        return Upc::NotFound;
    }
    // The name in lower case, for matching.
    let lower = name.to_lowercase();
    // Prefer an item whose title contains the name.
    let best = items.iter().find(|i| i.get("title").and_then(Value::as_str).unwrap_or("").to_lowercase().contains(&lower)).unwrap_or(&items[0]);
    // A closure: the first non-empty text in the list stored under key `k`.
    let first = |k: &str| best.get(k).and_then(Value::as_array).and_then(|a| a.iter().filter_map(Value::as_str).find(|s| !s.is_empty()).map(String::from));
    // Found a code: build the result. Otherwise nothing.
    match first("ean").or_else(|| first("upc")) {
        Some(upc) => Upc::Found {
            upc,
            title: best.get("title").and_then(Value::as_str).unwrap_or("").to_string(),
            brand: best.get("brand").and_then(Value::as_str).unwrap_or("").to_string(),
        },
        None => Upc::NotFound,
    }
}

/// Cache first; otherwise wait 2.5 s (the free API's pace) and ask.
/// Looks in the cache first. A cached barcode, or a cached "not found", is returned at once with no
/// web request. Otherwise it waits 2.5 seconds (the free service's pace), asks, and stores the
/// answer in the cache (not for rate limits).
async fn lookup(state: &AppState, cache: &mut Value, name: &str, company: &str) -> Upc {
    // The cache key.
    let key = cache_key(name, company);
    // A cached entry.
    if let Some(c) = cache["lookups"].get(&key) {
        if let Some(u) = c.get("upc").and_then(Value::as_str) {
            return Upc::Found { upc: u.into(), title: c.get("title").and_then(Value::as_str).unwrap_or("").into(), brand: String::new() };
        }
        if c.get("not_found").and_then(Value::as_bool) == Some(true) {
            return Upc::NotFound;
        }
    }
    // Wait without blocking other requests.
    tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
    // Ask the service.
    let res = search_upc(state, name, company).await;
    // The current time as text.
    let stamp = { let n = common::now_local(); format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", n.year(), u8::from(n.month()), n.day(), n.hour(), n.minute(), n.second()) };
    // Save the result in the cache. A rate-limit answer returns at once and is not saved.
    match &res {
        Upc::RateLimited => return res,
        Upc::Found { upc, title, brand } => cache["lookups"][&key] = serde_json::json!({ "upc": upc, "title": title, "brand": brand, "looked_up": stamp }),
        _ => cache["lookups"][&key] = serde_json::json!({ "not_found": true, "looked_up": stamp }),
    }
    // Write the cache file.
    save_cache(state, cache);
    res
}

/// Only two table names are accepted. Anything else becomes `data_source`. This stops a visitor
/// from naming any table, because the name is placed into SQL.
fn table_of(f: &str) -> &'static str {
    if f == "future_plants" { "future_plants" } else { "data_source" }
}

/// Tab 14. Shows cache counts, how many plants in each table have a barcode (progress bars), the
/// single lookup form, the batch lookup form, and the table of the last batch.
pub async fn barcodes(state: &AppState, req: &Req) -> String {
    // Heading and caption.
    let mut h = ui::header("🏷️ Barcode Lookup") + &ui::caption("Automatically look up UPC barcodes from UPCitemdb (free: 100 requests/day). Results are cached locally so repeated lookups don't burn API calls.");
    // The cache.
    let cache = load_cache(state);
    // The `lookups` object, copied (`cloned`).
    let lk = cache["lookups"].as_object().cloned().unwrap_or_default();
    // Entries with a barcode.
    let found = lk.values().filter(|v| v.get("upc").is_some()).count();
    // Entries marked not found.
    let nf = lk.values().filter(|v| v.get("not_found").is_some()).count();
    // Three count cards.
    h.push_str(&ui::columns(&[1, 1, 1], vec![ui::metric("Cached Lookups", &lk.len().to_string()), ui::metric("Found", &found.to_string()), ui::metric("Not Found", &nf.to_string())]));
    if let Some(t) = cache["last_updated"].as_str() {
        h.push_str(&ui::caption(&format!("Cache last updated: {t}")));
    }
    h.push_str(&ui::divider());
    // Garden: total living plants and how many have a barcode. `CAST(... AS SIGNED)` makes the sum
    // a plain integer. A failed query gives (0, 0).
    let ds: (i64, Option<i64>) = sqlx::query_as("SELECT COUNT(*), CAST(SUM(CASE WHEN barcode IS NOT NULL AND barcode != '' THEN 1 ELSE 0 END) AS SIGNED) FROM data_source WHERE died = 0")
        .fetch_one(&state.db).await.unwrap_or((0, Some(0)));
    // Wishlist: the same counts.
    let fp: (i64, Option<i64>) = sqlx::query_as("SELECT COUNT(*), CAST(SUM(CASE WHEN barcode IS NOT NULL AND barcode != '' THEN 1 ELSE 0 END) AS SIGNED) FROM future_plants")
        .fetch_one(&state.db).await.unwrap_or((0, Some(0)));
    // A closure that draws one progress bar. `total.max(1)` avoids dividing by zero.
    let bar = |title: &str, (total, with): (i64, Option<i64>)| {
        let w = with.unwrap_or(0);
        let pct = w * 100 / total.max(1);
        ui::subheader(title) + &format!("<div class=\"st-progress\"><div style=\"width:{pct}%\"></div></div><p class=\"st-text\">{w} / {total} have barcodes ({pct}%)</p>")
    };
    h.push_str(&ui::columns(&[1, 1], vec![bar("Garden (data_source)", ds), bar("Wishlist (future_plants)", fp)]));
    h.push_str(&ui::divider());
    // The two table choices.
    let tables = vec!["data_source".to_string(), "future_plants".to_string()];
    h.push_str(&ui::subheader("Single Plant Lookup"));
    // Single lookup form: a table choice and a plant id.
    h.push_str(&format!("<form method=\"post\" action=\"/streamlit/\"><input type=\"hidden\" name=\"act\" value=\"bc_single\"><input type=\"hidden\" name=\"tab\" value=\"{}\">", req.tab));
    h.push_str(&ui::columns(&[1, 1], vec![ui::select_plain("table", "Table", &tables, "data_source"), ui::number_input("pid", "Plant ID", "1", Some(1), "1")]));
    h.push_str(&ui::button("go", "1", "🔍 Look Up Barcode", false, false));
    h.push_str("</form>");
    h.push_str(&ui::divider());
    // Batch lookup form: a table choice and a limit between 5 and 90. The free service allows about
    // 100 requests a day. The page shows a busy message because it takes about 2.5 seconds per
    // plant.
    h.push_str(&ui::subheader("Batch Lookup (Missing Barcodes)"));
    h.push_str(&format!("<form method=\"post\" action=\"/streamlit/\" data-busy=\"Looking up barcodes… (about 2.5 seconds per plant)\"><input type=\"hidden\" name=\"act\" value=\"bc_batch\"><input type=\"hidden\" name=\"tab\" value=\"{}\"><input type=\"hidden\" name=\"batch\" value=\"1\">", req.tab));
    h.push_str(&ui::columns(&[1, 1], vec![ui::select_plain("table", "Table", &tables, "data_source"), ui::slider("limit", "Max plants to look up", 5, 90, 30)]));
    h.push_str(&ui::caption("⚠️ Free API: 100 requests/day. ~2.5s delay between requests."));
    h.push_str(&ui::button("go", "1", "🚀 Start Batch Lookup", true, false));
    h.push_str("</form>");
    // After a batch, the saved results are shown as a table.
    // The last batch's table (Streamlit showed it right after the run).
    if let Ok(b) = std::fs::read(state.data.join("barcode-last-batch.json")) {
        if let Ok(Value::Array(rs)) = serde_json::from_slice::<Value>(&b) {
            if !rs.is_empty() && req.get("batch") == "1" {
                let cols = ["id", "name", "upc", "title"];
                let t: Vec<Vec<String>> = rs.iter().map(|r| cols.iter().map(|c| ui::cell(&text(r.get(*c)))).collect()).collect();
                h.push_str(&ui::dataframe(&cols, &t));
            }
        }
    }
    h
}

/// Runs `bc_single` (one plant) or `bc_batch` (many plants lacking a barcode). Each found barcode
/// is saved on the plant. Returns messages.
pub async fn barcode_action(state: &AppState, act: &str, f: &crate::formdata::FormData) -> Vec<Flash> {
    // A safe table name.
    let table = table_of(f.str("table"));
    // The cache, loaded once and updated as lookups happen.
    let mut cache = load_cache(state);
    match act {
        // One plant. The table name is placed in SQL with backticks; it is safe because `table_of`
        // allows only two values.
        "bc_single" => {
            let id = php_int(f.str("pid"));
            let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(&format!("SELECT plant_name, company FROM `{table}` WHERE id = ?"))
                .bind(id).fetch_optional(&state.db).await.ok().flatten();
            // Plant not found: warn.
            let Some((name, company)) = row else { return vec![Flash::Warning("No barcode found for this plant.".into())] };
            let name = name.unwrap_or_default();
            // Show the result: success and product title, rate limit error, other error, or a
            // warning.
            match lookup(state, &mut cache, &name, company.as_deref().unwrap_or("")).await {
                Upc::Found { upc, title, .. } => {
                    let _ = sqlx::query(&format!("UPDATE `{table}` SET barcode = ? WHERE id = ?")).bind(&upc).bind(id).execute(&state.db).await;
                    let mut v = vec![Flash::Success(format!("Found: **{name}** → `{upc}`"))];
                    if !title.is_empty() {
                        v.push(Flash::Success(format!("Product: {title}")));
                    }
                    v
                }
                Upc::RateLimited => vec![Flash::Error("Error: rate_limited".into())],
                Upc::Error(e) => vec![Flash::Error(format!("Error: {e}"))],
                Upc::NotFound => vec![Flash::Warning("No barcode found for this plant.".into())],
            }
        }
        // Plants without a barcode, in name order, up to the limit.
        "bc_batch" => {
            // Keep the limit between 5 and 90.
            let limit = php_int(f.str("limit")).clamp(5, 90);
            let plants: Vec<(i64, Option<String>, Option<String>)> = sqlx::query_as(&format!("SELECT id, plant_name, company FROM `{table}` WHERE (barcode IS NULL OR barcode = '') ORDER BY plant_name LIMIT ?"))
                .bind(limit).fetch_all(&state.db).await.unwrap_or_default();
            // Counters for the final message.
            let (mut found, mut nf, mut errors) = (0, 0, 0);
            // The found barcodes, saved for display.
            let mut results = Vec::new();
            // Look up each plant. A rate limit stops the loop; a found barcode is saved on the
            // plant; anything else counts as not found.
            for (id, name, company) in plants {
                let name = name.unwrap_or_default();
                match lookup(state, &mut cache, &name, company.as_deref().unwrap_or("")).await {
                    Upc::RateLimited => { errors += 1; break; }
                    Upc::Found { upc, title, .. } => {
                        let _ = sqlx::query(&format!("UPDATE `{table}` SET barcode = ? WHERE id = ?")).bind(&upc).bind(id).execute(&state.db).await;
                        found += 1;
                        results.push(serde_json::json!({ "id": id, "name": name, "upc": upc, "title": title }));
                    }
                    _ => nf += 1,
                }
            }
            // Save the batch results so the page can show them. A failed write is ignored.
            let _ = std::fs::write(state.data.join("barcode-last-batch.json"), Value::Array(results).to_string());
            vec![Flash::Success(format!("Done! Found: {found}, Not found: {nf}, Errors: {errors}"))]
        }
        _ => vec![Flash::Error("Unknown action.".into())],
    }
}

/// Tests, built only for `cargo test`.
#[cfg(test)]
mod tests {
    use super::cache_key;

    /// Checks that the cache key matches the one the Streamlit app made.
    #[test]
    fn cache_keys_match_streamlit() {
        assert_eq!(cache_key("Lil' Kim® Rose", "Proven Winners"), "lilkimrose|provenwinners");
    }
}
