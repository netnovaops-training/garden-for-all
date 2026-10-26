//! The scan queue: tags and plant photos waiting to be read, matched and added.
//!
//! HOW TO READ THIS FILE (for Jun). Jun photographs a plant tag (front and back) or the plant
//! itself with the phone, picks the garden bed, and goes on to the next one. Each scan becomes an
//! ITEM here (table `scan_items`, its pictures in `scan_photos` and, as files, in the private
//! data folder `scans/`, which is never served as a web page). The desktop helper on the PC with
//! the graphics card (plant-ai/) asks for waiting items, reads the tag, searches the garden's own
//! plants and the internet, and sends back a BEST MATCH plus a few LIKELY MATCHES, each with a
//! confidence. Jun confirms one with one tap; only then is the plant (or its new bed and count)
//! saved and the normal fill-and-check helper queued for it (`plant_ai::queue`).
//!
//! Item status: waiting (on the site, the PC has not taken it), reading (the PC is on it),
//! matched (one clear best match, needs one tap), needs_you (no clear match: pick or type),
//! added (plant saved), failed (the PC could not read it).
//!
//! BEDS ARE KEPT BY NUMBER. A bed is `garden_bed.id`; the name is looked up when it is needed,
//! so renaming a bed never loses a queued scan. The plant row gets the bed's name AT THE TIME OF
//! THE TAP, because the plant tables keep bed names as text.
//!
//! The pieces: this file (tables, checks, saving a confirmed match), `scan_api.rs` (the phone's
//! and the helper's addresses), `orchestrator/scan.rs` (the "Scan queue" tab on the website).

use std::io::Cursor;

use serde_json::{json, Map, Value};
use sqlx::Row;

use crate::orchestrator::opts;
use crate::plant_ai;
use crate::session::Visitor;
use crate::AppState;

/// A best match at or above this confidence, clearly ahead of the next one, is "matched".
pub const MATCH_AT: f64 = 0.75;
/// How far ahead of the second candidate the best one must be to count as clear.
pub const LEAD: f64 = 0.15;
/// The most pictures one item may have (front, back, plant, and a spare).
pub const MAX_PHOTOS: usize = 4;
/// The longest side a stored picture keeps (tags are read well below this).
const MAX_SIDE: u32 = 2400;
/// Where a plant waits until it has a real bed.
pub const WAITING_BED: &str = "To Be Sorted";

/// The tables, made at start-up if they are missing (new tables only).
pub async fn ensure_tables(db: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
    for sql in [
        // One row per scan. `client_uid` is made by the phone, so sending the same scan twice
        // (a weak signal, a retry) never makes two items.
        "CREATE TABLE IF NOT EXISTS `scan_items` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            client_uid VARCHAR(64) NOT NULL,
            kind VARCHAR(10) NOT NULL DEFAULT 'tag',
            bed_id INT NULL,
            qty INT NOT NULL DEFAULT 1,
            barcode_hint VARCHAR(64) NOT NULL DEFAULT '',
            status VARCHAR(12) NOT NULL DEFAULT 'waiting',
            step VARCHAR(40) NOT NULL DEFAULT '',
            added_by VARCHAR(100) NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            claimed_at DATETIME NULL,
            finished_at DATETIME NULL,
            result LONGTEXT NULL,
            error VARCHAR(500) NOT NULL DEFAULT '',
            plant_id INT NULL,
            chosen VARCHAR(40) NOT NULL DEFAULT '',
            UNIQUE KEY k_uid (client_uid),
            KEY k_status (status)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        // The pictures of an item (the files are in <data>/scans/).
        "CREATE TABLE IF NOT EXISTS `scan_photos` (
            id INT AUTO_INCREMENT PRIMARY KEY,
            item_id INT NOT NULL,
            side VARCHAR(10) NOT NULL DEFAULT 'front',
            file VARCHAR(120) NOT NULL,
            width INT NOT NULL DEFAULT 0,
            height INT NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            KEY k_item (item_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    ] {
        sqlx::query(sql).execute(db).await?;
    }
    Ok(())
}

// ---- small checks ---------------------------------------------------------------------------------

/// The phone's id for a scan: 8 to 64 letters, digits, `-` or `_`.
pub fn clean_uid(s: &str) -> Option<String> {
    let s = s.trim();
    (8..=64)
        .contains(&s.len())
        .then_some(s)
        .filter(|s| {
            s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
        .map(str::to_string)
}

/// Which side of a tag (or the plant) a picture shows.
pub fn clean_side(s: &str) -> &'static str {
    match s.trim() {
        "back" => "back",
        "plant" => "plant",
        _ => "front",
    }
}

/// `tag` (front and back of a plant tag) or `plant` (a photo of the plant itself).
pub fn clean_kind(s: &str) -> &'static str {
    if s.trim() == "plant" {
        "plant"
    } else {
        "tag"
    }
}

/// Text made safe to keep and show: no control characters, trimmed, at most `n` characters.
pub fn tidy(s: &str, n: usize) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(n)
        .collect::<String>()
        .trim()
        .to_string()
}

/// A barcode or QR text from the phone: digits, letters and a few separators, at most 64.
pub fn clean_barcode(s: &str) -> String {
    s.trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | ':' | '/' | '_'))
        .take(64)
        .collect()
}

/// A picture for the queue: opened (so it must be a real picture), turned upright by its EXIF
/// flag, shrunk to 2400 px on the long side if bigger, and written again as JPEG quality 90. The
/// new file has no EXIF, so no GPS position from the phone is ever kept.
pub fn jpeg_for_storage(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    use image::ImageDecoder;
    let reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "not a picture".to_string())?;
    let mut dec = reader
        .into_decoder()
        .map_err(|_| "not a picture".to_string())?;
    // A picture that claims to be enormous is refused before it is unpacked in memory.
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12_000);
    limits.max_image_height = Some(12_000);
    limits.max_alloc = Some(512 * 1024 * 1024);
    dec.set_limits(limits)
        .map_err(|_| "picture too large".to_string())?;
    let orient = dec.orientation().ok();
    let mut img =
        image::DynamicImage::from_decoder(dec).map_err(|_| "not a picture".to_string())?;
    if let Some(o) = orient {
        img.apply_orientation(o);
    }
    if img.width().max(img.height()) < 200 {
        return Err("too small to read (200 px needed)".into());
    }
    if img.width().max(img.height()) > MAX_SIDE {
        img = img.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Lanczos3);
    }
    let (w, h) = (img.width(), img.height());
    let mut out = Vec::new();
    let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90);
    img.to_rgb8()
        .write_with_encoder(enc)
        .map_err(|_| "could not write the picture".to_string())?;
    Ok((out, w, h))
}

/// The private folder of scan pictures (inside the data folder, never a web page).
pub fn photo_dir(state: &AppState) -> std::path::PathBuf {
    state.data.join("scans")
}

// ---- beds ----------------------------------------------------------------------------------------

/// A garden bed: its number (never changes) and its name now.
#[derive(Clone, Debug, PartialEq)]
pub struct Bed {
    pub id: i64,
    pub name: String,
}

/// The beds to pick from, by number. Every bed name the plants use gets a number first if it has
/// none (the Labels page does the same), and "To Be Sorted" always has one. "Dead or Removed" is
/// not offered: nothing new goes there.
pub async fn beds(state: &AppState) -> Vec<Bed> {
    let _ = sqlx::query(
        "INSERT IGNORE INTO garden_bed (name, shape)
         SELECT DISTINCT TRIM(flower_bed), '' FROM data_source
         WHERE TRIM(COALESCE(flower_bed, '')) <> '' AND died = 0",
    )
    .execute(&state.db)
    .await;
    let _ = sqlx::query("INSERT IGNORE INTO garden_bed (name, shape) VALUES (?, '')")
        .bind(WAITING_BED)
        .execute(&state.db)
        .await;
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM garden_bed WHERE name <> 'Dead or Removed' ORDER BY (name = 'To Be Sorted') DESC, name",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, name)| Bed { id, name })
    .collect()
}

/// A bed's name right now, or None when there is no such bed.
pub async fn bed_name(state: &AppState, id: i64) -> Option<String> {
    sqlx::query_scalar::<_, String>("SELECT name FROM garden_bed WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
}

// ---- what the helper sends back ---------------------------------------------------------------------

/// Check and tidy a result from the helper, and decide the item's status. Candidates are kept in
/// order of confidence (best first, at most 5, named "c1".."c5"); a confidence is clamped to
/// 0..1; words are cleaned. `matched` needs the best at MATCH_AT or more and a LEAD over the
/// second; everything else (including no candidates) is `needs_you`.
pub fn judge(v: &Value) -> (&'static str, Value) {
    let mut cands: Vec<Map<String, Value>> = v
        .get("candidates")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(clean_candidate).collect())
        .unwrap_or_default();
    cands.sort_by(|a, b| {
        let f = |m: &Map<String, Value>| m.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
        f(b).partial_cmp(&f(a)).unwrap_or(std::cmp::Ordering::Equal)
    });
    cands.truncate(5);
    for (i, c) in cands.iter_mut().enumerate() {
        c.insert("id".into(), json!(format!("c{}", i + 1)));
    }
    let conf = |i: usize| {
        cands
            .get(i)
            .and_then(|c| c.get("confidence"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    let clear = !cands.is_empty() && conf(0) >= MATCH_AT && conf(0) - conf(1) >= LEAD;
    let ocr = v.get("ocr").and_then(Value::as_object).map(|o| {
        let mut m = Map::new();
        for k in ["name", "botanical", "size", "vendor", "barcode", "engine"] {
            m.insert(k.into(), json!(tidy(&opts::text(o.get(k)), 120)));
        }
        let lines: Vec<Value> = o
            .get("lines")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .take(40)
                    .map(|l| json!(tidy(&opts::text(Some(l)), 120)))
                    .collect()
            })
            .unwrap_or_default();
        m.insert("lines".into(), Value::Array(lines));
        Value::Object(m)
    });
    let notes: Vec<Value> = v
        .get("notes")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .take(6)
                .map(|n| json!(tidy(&opts::text(Some(n)), 200)))
                .collect()
        })
        .unwrap_or_default();
    let clean = json!({
        "candidates": cands,
        "best": if clear { json!("c1") } else { Value::Null },
        "ocr": ocr.unwrap_or(Value::Null),
        "notes": notes,
    });
    (if clear { "matched" } else { "needs_you" }, clean)
}

/// One candidate, cleaned; None when it has no usable name.
fn clean_candidate(c: &Value) -> Option<Map<String, Value>> {
    let o = c.as_object()?;
    let name = tidy(&opts::text(o.get("name")), 200);
    let botanical = tidy(&opts::text(o.get("botanical")), 150);
    if name.is_empty() && botanical.is_empty() {
        return None;
    }
    let conf = o
        .get("confidence")
        .and_then(Value::as_f64)
        .filter(|x| x.is_finite())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let plant_id = o.get("plant_id").and_then(Value::as_i64).filter(|n| *n > 0);
    let set = match o.get("set").and_then(Value::as_str) {
        Some("current") => "current",
        Some("future") => "future",
        _ => "",
    };
    let source = o
        .get("source")
        .and_then(Value::as_str)
        .filter(|s| matches!(*s, "garden" | "wishlist" | "image" | "web" | "tag"))
        .unwrap_or("tag");
    let photo = tidy(&opts::text(o.get("photo")), 300);
    let photo = if photo.starts_with("http://") || photo.starts_with("https://") {
        plant_ai::clean_url(&photo).unwrap_or_default()
    } else {
        // A picture of the garden itself: the first plain file name, no folders.
        let first = photo.split(',').next().unwrap_or("");
        let last = first.rsplit(['/', '\\']).next().unwrap_or("");
        last.chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ' '))
            .collect::<String>()
            .trim_start_matches('.')
            .trim()
            .to_string()
    };
    let mut m = Map::new();
    m.insert(
        "name".into(),
        json!(if name.is_empty() {
            botanical.clone()
        } else {
            name
        }),
    );
    m.insert("botanical".into(), json!(botanical));
    m.insert("genus".into(), json!(tidy(&opts::text(o.get("genus")), 60)));
    m.insert(
        "species".into(),
        json!(tidy(&opts::text(o.get("species")), 60)),
    );
    m.insert("confidence".into(), json!((conf * 1000.0).round() / 1000.0));
    m.insert("source".into(), json!(source));
    m.insert("set".into(), json!(set));
    m.insert(
        "plant_id".into(),
        plant_id.map(|n| json!(n)).unwrap_or(Value::Null),
    );
    m.insert("photo".into(), json!(photo));
    m.insert(
        "url".into(),
        json!(plant_ai::clean_url(&opts::text(o.get("url"))).unwrap_or_default()),
    );
    m.insert("why".into(), json!(tidy(&opts::text(o.get("why")), 240)));
    Some(m)
}

// ---- making an item ------------------------------------------------------------------------------------

/// One picture of a new item: which side, and the bytes as the phone sent them.
pub struct NewPhoto {
    pub side: String,
    pub bytes: Vec<u8>,
}

/// What `create_item` gives back.
pub struct Created {
    pub id: i64,
    /// True when this scan was already in the queue (the same `client_uid`).
    pub duplicate: bool,
}

/// What a new scan says about itself (its pictures come separately).
#[derive(Clone, Copy)]
pub struct NewItem<'a> {
    /// Who scanned it (the account name).
    pub who: &'a str,
    /// The phone's id for this scan; the same id sent twice is one scan.
    pub uid: &'a str,
    pub kind: &'a str,
    pub bed_id: Option<i64>,
    pub qty: i64,
    /// A barcode or QR text the phone read itself ("" when none).
    pub barcode: &'a str,
}

/// Save a new scan. Idempotent on `uid`. Pictures are checked and re-saved (`jpeg_for_storage`)
/// BEFORE anything is written, so a bad picture leaves no half item behind.
pub async fn create_item(
    state: &AppState,
    item: NewItem<'_>,
    photos: Vec<NewPhoto>,
) -> Result<Created, String> {
    let NewItem {
        who,
        uid,
        kind,
        bed_id,
        qty,
        barcode,
    } = item;
    if photos.is_empty() {
        return Err("a scan needs at least one picture".into());
    }
    if photos.len() > MAX_PHOTOS {
        return Err(format!("at most {MAX_PHOTOS} pictures per scan"));
    }
    if let Some(id) = sqlx::query_scalar::<_, i64>("SELECT id FROM scan_items WHERE client_uid = ?")
        .bind(uid)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(Created {
            id,
            duplicate: true,
        });
    }
    if let Some(b) = bed_id {
        if bed_name(state, b).await.is_none() {
            return Err("that bed does not exist (its number is not in the garden)".into());
        }
    }
    let mut ready = Vec::new();
    for (i, p) in photos.iter().enumerate() {
        let (jpg, w, h) =
            jpeg_for_storage(&p.bytes).map_err(|e| format!("picture {}: {e}", i + 1))?;
        ready.push((clean_side(&p.side), jpg, w, h));
    }
    let r = sqlx::query(
        "INSERT INTO scan_items (client_uid, kind, bed_id, qty, barcode_hint, added_by) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(uid)
    .bind(clean_kind(kind))
    .bind(bed_id)
    .bind(qty.clamp(1, 1000))
    .bind(clean_barcode(barcode))
    .bind(tidy(who, 100))
    .execute(&state.db)
    .await;
    let id = match r {
        Ok(r) => r.last_insert_id() as i64,
        // Two copies of the same scan arrived together: the second finds the first.
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            let id = sqlx::query_scalar::<_, i64>("SELECT id FROM scan_items WHERE client_uid = ?")
                .bind(uid)
                .fetch_one(&state.db)
                .await
                .map_err(|e| e.to_string())?;
            return Ok(Created {
                id,
                duplicate: true,
            });
        }
        Err(e) => return Err(e.to_string()),
    };
    // The pictures: files first, then their rows. If any of it fails, the whole item is taken
    // back out (rows and files), so a scan is either complete or not there at all.
    let dir = photo_dir(state);
    let saved = async {
        std::fs::create_dir_all(&dir).map_err(|e| format!("could not make the scans folder: {e}"))?;
        for (n, (side, jpg, w, h)) in ready.into_iter().enumerate() {
            let file = format!("{id}_{}_{side}.jpg", n + 1);
            std::fs::write(dir.join(&file), jpg).map_err(|e| format!("could not save a picture: {e}"))?;
            sqlx::query("INSERT INTO scan_photos (item_id, side, file, width, height) VALUES (?, ?, ?, ?, ?)")
                .bind(id)
                .bind(side)
                .bind(&file)
                .bind(w as i64)
                .bind(h as i64)
                .execute(&state.db)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok::<(), String>(())
    }
    .await;
    if let Err(e) = saved {
        let _ = remove_item(state, id).await;
        if let Ok(rd) = std::fs::read_dir(&dir) {
            let prefix = format!("{id}_");
            for f in rd
                .flatten()
                .filter(|f| f.file_name().to_string_lossy().starts_with(&prefix))
            {
                let _ = std::fs::remove_file(f.path());
            }
        }
        return Err(e);
    }
    Ok(Created {
        id,
        duplicate: false,
    })
}

/// Delete an item and its picture files.
pub async fn remove_item(state: &AppState, id: i64) -> Result<(), String> {
    let files: Vec<String> = sqlx::query_scalar("SELECT file FROM scan_photos WHERE item_id = ?")
        .bind(id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    for f in files {
        // The names were made here (`<id>_<n>_<side>.jpg`); a name with a slash is ignored.
        if !f.contains('/') && !f.contains('\\') && !f.contains("..") {
            let _ = std::fs::remove_file(photo_dir(state).join(f));
        }
    }
    sqlx::query("DELETE FROM scan_photos WHERE item_id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM scan_items WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---- reading items --------------------------------------------------------------------------------------

/// One item as the phone and the Scan queue tab show it.
#[derive(Clone, Debug)]
pub struct Item {
    pub id: i64,
    pub client_uid: String,
    pub kind: String,
    pub bed_id: Option<i64>,
    pub bed_name: String,
    pub qty: i64,
    pub status: String,
    pub step: String,
    pub created: String,
    pub result: Option<Value>,
    pub error: String,
    pub plant_id: Option<i64>,
    pub photos: i64,
}

impl Item {
    /// The item as JSON for the phone (never the picture bytes).
    pub fn json(&self) -> Value {
        let cands = self
            .result
            .as_ref()
            .and_then(|r| r.get("candidates"))
            .cloned()
            .unwrap_or_else(|| json!([]));
        json!({
            "id": self.id, "client_uid": self.client_uid, "kind": self.kind,
            "bed_id": self.bed_id, "bed_name": self.bed_name, "qty": self.qty,
            "status": self.status, "step": self.step, "created": self.created,
            "candidates": cands, "plant_id": self.plant_id, "error": self.error,
            "photos": self.photos,
        })
    }
}

const ITEM_SQL: &str =
    "SELECT i.id, i.client_uid, i.kind, i.bed_id, b.name AS bed_name, i.qty, i.status, i.step,
        DATE_FORMAT(i.created_at, '%Y-%m-%d %H:%i') AS created, i.result, i.error, i.plant_id,
        (SELECT COUNT(*) FROM scan_photos p WHERE p.item_id = i.id) AS photos
     FROM scan_items i LEFT JOIN garden_bed b ON b.id = i.bed_id";

fn item_of(r: &sqlx::mysql::MySqlRow) -> Item {
    Item {
        id: r.try_get("id").unwrap_or(0),
        client_uid: r.try_get("client_uid").unwrap_or_default(),
        kind: r.try_get("kind").unwrap_or_default(),
        bed_id: r.try_get("bed_id").ok().flatten(),
        bed_name: r
            .try_get::<Option<String>, _>("bed_name")
            .ok()
            .flatten()
            .unwrap_or_default(),
        qty: r.try_get("qty").unwrap_or(1),
        status: r.try_get("status").unwrap_or_default(),
        step: r.try_get("step").unwrap_or_default(),
        created: r.try_get("created").unwrap_or_default(),
        result: r
            .try_get::<Option<String>, _>("result")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok()),
        error: r.try_get("error").unwrap_or_default(),
        plant_id: r.try_get("plant_id").ok().flatten(),
        photos: r.try_get("photos").unwrap_or(0),
    }
}

/// The newest `n` items, newest first.
pub async fn recent(state: &AppState, n: i64) -> Vec<Item> {
    let sql = format!("{ITEM_SQL} ORDER BY i.id DESC LIMIT ?");
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(n)
        .fetch_all(&state.db)
        .await
        .map(|rs| rs.iter().map(item_of).collect())
        .unwrap_or_default()
}

/// One item by number.
pub async fn item(state: &AppState, id: i64) -> Option<Item> {
    let sql = format!("{ITEM_SQL} WHERE i.id = ?");
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|r| item_of(&r))
}

// ---- saving a confirmed match -----------------------------------------------------------------------------

/// What the person chose for an item.
pub enum Pick {
    /// One of the helper's candidates, by its id ("c1"...).
    Candidate(String),
    /// Typed by hand: genus, species (may be empty), name.
    Manual {
        genus: String,
        species: String,
        name: String,
    },
}

/// What happened when a match was confirmed.
pub struct Confirmed {
    pub plant_id: i64,
    pub name: String,
    pub bed: String,
    /// True when the plant was already in the garden (its count and bed were updated instead).
    pub merged: bool,
}

/// A genus word or species word: letters and `-` only, 1 to 60 characters.
pub fn word_ok(w: &str) -> bool {
    !w.is_empty() && w.len() <= 60 && w.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
}

/// Genus and species from what a candidate carries: the fields when they are words, else the first
/// two words of the botanical name ("Salvia nemorosa 'May Night'"). Species is "" when unknown.
pub fn genus_species(genus: &str, species: &str, botanical: &str) -> Option<(String, String)> {
    let mut words = botanical
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '-'));
    let first = words.next().unwrap_or("");
    let second = words.next().unwrap_or("");
    let g = if word_ok(genus.trim()) {
        genus.trim()
    } else {
        first
    };
    if !word_ok(g) {
        return None;
    }
    let sp = if word_ok(species.trim()) {
        species.trim()
    } else if g.eq_ignore_ascii_case(first)
        && word_ok(second)
        && second.chars().all(|c| c.is_ascii_lowercase() || c == '-')
    {
        second
    } else {
        ""
    };
    let mut c = g.chars();
    let g = c
        .next()
        .map(|x| x.to_ascii_uppercase().to_string() + &c.as_str().to_lowercase())
        .unwrap_or_default();
    Some((g, sp.to_lowercase()))
}

/// Save a new plant row (the six things the Add plant tab asks for). Gives its number.
/// Used by the Add plant tab, the phone's typed add and a confirmed scan, so all three make the
/// same row.
pub async fn insert_plant(
    state: &AppState,
    genus: &str,
    species: &str,
    name: &str,
    bed: &str,
    qty: i64,
    price: Option<f64>,
) -> Result<i64, String> {
    let botanical = if species.is_empty() {
        genus.to_string()
    } else {
        format!("{genus} {species}")
    };
    let year = common::current_year().to_string();
    let r = sqlx::query("INSERT INTO data_source (plant_name, genus, species, botanical_name, garden_beds, flower_bed, qty, average_cost, year_added) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(name)
        .bind(genus)
        .bind(if species.is_empty() { None } else { Some(species.to_string()) })
        .bind(&botanical)
        .bind(bed)
        .bind(bed)
        .bind(qty)
        .bind(price)
        .bind(&year)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;
    state.invalidate_widget_cache();
    Ok(r.last_insert_id() as i64)
}

/// A plant that is already in the garden: more plants of it, in this bed. Backs the row up first,
/// adds to the count (and a "purchased" line in the quantity log) and adds the bed to the plant's
/// bed list when it is not there.
async fn merge_into(
    state: &AppState,
    plant_id: i64,
    qty: i64,
    bed: &str,
    who: &str,
) -> Result<String, String> {
    use chrono::Datelike;
    let row = plant_ai::plant_row(state, plant_id)
        .await
        .ok_or("that plant is not in the garden any more")?;
    plant_ai::backup(state, plant_id, None, "scan queue: more of this plant").await?;
    let mut beds: Vec<String> = opts::text(row.get("garden_beds"))
        .split(',')
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty() && !b.eq_ignore_ascii_case(WAITING_BED))
        .collect();
    if !beds.iter().any(|b| b == bed) {
        beds.push(bed.to_string());
    }
    let joined = beds.join(", ");
    sqlx::query("UPDATE data_source SET qty = COALESCE(qty, 0) + ?, garden_beds = ?, flower_bed = ?, died = 0 WHERE id = ?")
        .bind(qty)
        .bind(&joined)
        .bind(&joined)
        .bind(plant_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;
    let d = opts::today();
    let week: i64 = d.format("%W").to_string().parse::<i64>().unwrap_or(0) + 1;
    // The quantity log is optional (older gardens may not have it): a failure is not an error.
    let _ = sqlx::query(
        "INSERT INTO plant_quantity_log (plant_id, event_type, quantity, event_date, event_week, event_month, event_year, notes, recorded_by)
         VALUES (?, 'purchased', ?, ?, ?, ?, ?, 'scan queue', ?)",
    )
    .bind(plant_id)
    .bind(qty)
    .bind(d.to_string())
    .bind(week)
    .bind(d.month())
    .bind(d.year())
    .bind(tidy(who, 100))
    .execute(&state.db)
    .await;
    state.invalidate_widget_cache();
    Ok(opts::text(row.get("plant_name")))
}

/// Confirm an item: the garden owner only. Saves the plant (or adds to the one already there) in
/// the item's bed, as the bed is named now, then queues the normal fill-and-check helper.
/// Tapping twice is safe: the second tap finds the item already added.
pub async fn confirm(
    state: &AppState,
    who: &Visitor,
    item_id: i64,
    pick: Pick,
) -> Result<Confirmed, String> {
    if !plant_ai::is_owner(state, who) {
        return Err("only the garden owner can confirm a match".into());
    }
    let it = item(state, item_id)
        .await
        .ok_or("that scan is not in the queue")?;
    if it.status == "added" {
        if let Some(pid) = it.plant_id {
            let name = plant_ai::plant_row(state, pid)
                .await
                .map(|r| opts::text(r.get("plant_name")))
                .unwrap_or_default();
            return Ok(Confirmed {
                plant_id: pid,
                name,
                bed: it.bed_name.clone(),
                merged: false,
            });
        }
    }
    // Take the item (a second tap at the same moment finds it taken).
    let taken = sqlx::query("UPDATE scan_items SET status = 'adding' WHERE id = ? AND status IN ('waiting', 'matched', 'needs_you', 'failed')")
        .bind(item_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    if taken.rows_affected() != 1 {
        return Err("this scan is being added or is not ready".into());
    }
    let out = confirm_inner(state, who, &it, pick).await;
    match out {
        Ok((c, chosen)) => {
            sqlx::query("UPDATE scan_items SET status = 'added', plant_id = ?, chosen = ?, finished_at = NOW(), step = 'done', error = '' WHERE id = ?")
                .bind(c.plant_id)
                .bind(tidy(&chosen, 40))
                .bind(item_id)
                .execute(&state.db)
                .await
                .map_err(|e| e.to_string())?;
            Ok(c)
        }
        Err(e) => {
            // Put the item back as it was, so the person can try again.
            let _ =
                sqlx::query("UPDATE scan_items SET status = ? WHERE id = ? AND status = 'adding'")
                    .bind(&it.status)
                    .bind(item_id)
                    .execute(&state.db)
                    .await;
            Err(e)
        }
    }
}

/// The work of `confirm` once the item is taken. Gives the result and what was chosen.
async fn confirm_inner(
    state: &AppState,
    who: &Visitor,
    it: &Item,
    pick: Pick,
) -> Result<(Confirmed, String), String> {
    let bed = match it.bed_id {
        Some(b) => bed_name(state, b)
            .await
            .unwrap_or_else(|| WAITING_BED.to_string()),
        None => WAITING_BED.to_string(),
    };
    let qty = it.qty.clamp(1, 1000);
    let (genus, species, name, existing, chosen) = match pick {
        Pick::Manual {
            genus,
            species,
            name,
        } => {
            let name = tidy(&name, 255);
            let (g, s) =
                genus_species(&genus, &species, "").ok_or("type a genus (letters only)")?;
            if name.is_empty() {
                return Err("type a name".into());
            }
            (g, s, name, None, "manual".to_string())
        }
        Pick::Candidate(cid) => {
            if !matches!(it.status.as_str(), "matched" | "needs_you") {
                return Err("this scan has no matches yet".into());
            }
            let c = it
                .result
                .as_ref()
                .and_then(|r| r.get("candidates"))
                .and_then(Value::as_array)
                .and_then(|a| {
                    a.iter()
                        .find(|c| c.get("id").and_then(Value::as_str) == Some(cid.as_str()))
                })
                .ok_or("that match is not in the list")?;
            let text = |k: &str| opts::text(c.get(k));
            let existing = (text("set") == "current")
                .then(|| c.get("plant_id").and_then(Value::as_i64))
                .flatten();
            let (g, s) = genus_species(&text("genus"), &text("species"), &text("botanical"))
                .unwrap_or_default();
            (g, s, text("name"), existing, cid)
        }
    };
    if let Some(pid) = existing {
        let name = merge_into(state, pid, qty, &bed, &who.username).await?;
        return Ok((
            Confirmed {
                plant_id: pid,
                name,
                bed,
                merged: true,
            },
            chosen,
        ));
    }
    if !word_ok(&genus) {
        return Err("this match has no genus: use \"type it instead\"".into());
    }
    let pid = insert_plant(state, &genus, &species, &name, &bed, qty, None).await?;
    // The normal fill-and-check helper completes the data. A queue problem is not fatal: the
    // plant is saved and "Ask the AI (again)" on the Add plant tab can queue it later.
    if let Err(e) = plant_ai::queue(state, pid, &who.username).await {
        tracing::warn!("scan queue: could not queue the AI for plant {pid}: {e}");
    }
    Ok((
        Confirmed {
            plant_id: pid,
            name,
            bed,
            merged: false,
        },
        chosen,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(name: &str, conf: f64) -> Value {
        json!({"name": name, "botanical": "Salvia nemorosa", "confidence": conf, "source": "garden"})
    }

    #[test]
    fn uid_side_kind() {
        assert!(clean_uid("abcd-1234_ef").is_some());
        assert!(clean_uid("short").is_none());
        assert!(clean_uid("has space in it!").is_none());
        assert_eq!(clean_side("back"), "back");
        assert_eq!(clean_side("../x"), "front");
        assert_eq!(clean_kind("plant"), "plant");
        assert_eq!(clean_kind("anything"), "tag");
        assert_eq!(clean_barcode(" 0123 4567;DROP "), "01234567DROP");
    }

    #[test]
    fn matched_needs_a_clear_best() {
        let (s, v) = judge(&json!({"candidates": [cand("A", 0.9), cand("B", 0.4)]}));
        assert_eq!(s, "matched");
        assert_eq!(v["best"], "c1");
        // Two close candidates: Jun chooses.
        let (s, v) = judge(&json!({"candidates": [cand("A", 0.85), cand("B", 0.80)]}));
        assert_eq!(s, "needs_you");
        assert!(v["best"].is_null());
        // A weak best: needs Jun.
        assert_eq!(
            judge(&json!({"candidates": [cand("A", 0.5)]})).0,
            "needs_you"
        );
        // Nothing at all: needs Jun.
        assert_eq!(judge(&json!({})).0, "needs_you");
    }

    #[test]
    fn candidates_are_sorted_capped_and_cleaned() {
        let mut list: Vec<Value> = (0..8)
            .map(|i| cand(&format!("P{i}"), 0.1 * i as f64))
            .collect();
        list.push(json!({"name": "", "botanical": ""}));
        list.push(json!({"name": "Odd\u{0007}name", "confidence": 7.5, "photo": "../../etc/passwd", "url": "javascript:x"}));
        let (_, v) = judge(&json!({ "candidates": list }));
        let c = v["candidates"].as_array().unwrap();
        assert_eq!(c.len(), 5);
        assert_eq!(c[0]["name"], "Oddname");
        assert_eq!(c[0]["confidence"], 1.0);
        assert_eq!(c[0]["id"], "c1");
        assert_eq!(c[0]["photo"], "passwd");
        assert_eq!(c[0]["url"], "");
        assert!(c
            .windows(2)
            .all(|w| w[0]["confidence"].as_f64() >= w[1]["confidence"].as_f64()));
    }

    #[test]
    fn genus_and_species_come_from_the_botanical_name() {
        let g = |a: &str, b: &str, c: &str| genus_species(a, b, c);
        assert_eq!(
            g("", "", "Salvia nemorosa 'May Night'"),
            Some(("Salvia".into(), "nemorosa".into()))
        );
        assert_eq!(
            g("salvia", "NEMOROSA", ""),
            Some(("Salvia".into(), "nemorosa".into()))
        );
        // A cultivar word is not a species.
        assert_eq!(
            g("", "", "Hosta 'Blue Angel'"),
            Some(("Hosta".into(), "".into()))
        );
        assert_eq!(g("", "", ""), None);
        assert_eq!(g("", "", "123 456"), None);
    }

    #[test]
    fn pictures_are_checked_and_made_plain_jpeg() {
        let mut png = Vec::new();
        image::DynamicImage::new_rgb8(3000, 1000)
            .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let (jpg, w, h) = jpeg_for_storage(&png).unwrap();
        assert_eq!((w, h), (2400, 800));
        assert_eq!(&jpg[..2], &[0xFF, 0xD8]);
        assert!(jpeg_for_storage(b"not a picture at all").is_err());
        let mut tiny = Vec::new();
        image::DynamicImage::new_rgb8(50, 50)
            .write_to(&mut Cursor::new(&mut tiny), image::ImageFormat::Png)
            .unwrap();
        assert!(jpeg_for_storage(&tiny).unwrap_err().contains("too small"));
    }

    /// A phone photo says "turn me by 90 degrees" in its EXIF and may hold a GPS position: the
    /// stored copy is turned already and holds no EXIF at all.
    #[test]
    fn exif_turn_is_applied_and_the_exif_is_gone() {
        let mut plain = Vec::new();
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut plain, 90);
        image::DynamicImage::new_rgb8(400, 200)
            .to_rgb8()
            .write_with_encoder(enc)
            .unwrap();
        // An EXIF block with Orientation = 6 (rotate 90 degrees clockwise), then the picture.
        let tiff: Vec<u8> = [
            &b"II*\0"[..],
            &[8, 0, 0, 0],
            &[1, 0],
            &[0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0],
            &[0, 0, 0, 0],
        ]
        .concat();
        let mut exif = b"Exif\0\0".to_vec();
        exif.extend(&tiff);
        let len = (exif.len() + 2) as u16;
        let mut tagged = vec![0xFF, 0xD8, 0xFF, 0xE1];
        tagged.extend(len.to_be_bytes());
        tagged.extend(&exif);
        tagged.extend(&plain[2..]);
        let (jpg, w, h) = jpeg_for_storage(&tagged).unwrap();
        assert_eq!((w, h), (200, 400), "the picture was turned upright");
        assert!(
            !jpg.windows(4).any(|x| x == b"Exif"),
            "no EXIF block is kept"
        );
    }
}
