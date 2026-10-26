//! The scan queue's addresses: what the phone (and the website's phone page) calls, and what the
//! desktop helper calls. The logic is in `scan_queue.rs`; this file only checks who is asking,
//! reads the request and answers in JSON.
//!
//! THE PHONE (signed in: the app's key or a website session; guests are refused):
//!   GET  /app/scan/beds                      the beds, each with its number and its name now
//!   GET  /app/scan/items                     the newest scans and their status
//!   POST /app/scan/items                     a new scan (JSON with base64 pictures, or a
//!                                            multipart form with files front / back / plant)
//!   POST /app/scan/items/<id>/confirm        one tap on a match (the garden owner only)
//!   POST /app/scan/items/<id>/bed            change the bed of a scan not added yet
//!   POST /app/scan/items/<id>/remove         take a scan out of the queue
//!   POST /app/scan/manual                    type a plant instead (genus, species, name, qty, price, bed)
//!
//! THE DESKTOP HELPER (`Authorization: Worker <key>`, home network only, like /plant-ai/jobs):
//!   GET  /plant-ai/scans?take=N              the next waiting scans (marked "reading")
//!   GET  /plant-ai/scan-photos/<id>          one picture, as JPEG
//!   POST /plant-ai/scans/<id>/progress|result|fail
//!   GET  /plant-ai/catalog                   every plant (current and wish list) in short form,
//!                                            so the PC can search the garden without a database login
//!
//! The helper only makes outgoing requests, so no port is opened anywhere (same as the plant queue).

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, RawQuery, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use serde_json::{json, Value};

use crate::formdata::FormData;
use crate::orchestrator::opts;
use crate::plant_ai::{b64_decode, helper, reply};
use crate::scan_queue as sq;
use crate::session::Visitor;
use crate::AppState;

/// The routes, merged into the main router (main.rs).
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/app/scan/beds", get(beds))
        .route("/app/scan/items", get(list).post(create))
        .route("/app/scan/items/{id}/{what}", post(change))
        .route("/app/scan/manual", post(manual))
        .route("/plant-ai/scans", get(take))
        .route("/plant-ai/scan-photos/{id}", get(photo))
        .route("/plant-ai/scans/{id}/{what}", post(report))
        .route("/plant-ai/catalog", get(catalog))
}

/// Signed-in people only (the app's key or a session).
fn signed_in(v: &Visitor) -> Result<(), Box<Response>> {
    if v.is_admin() {
        Ok(())
    } else {
        Err(Box::new(reply(
            StatusCode::UNAUTHORIZED,
            json!({"error": "Sign in to use the scan queue.", "needs_sign_in": true}),
        )))
    }
}

/// A field from a JSON body or a form, as text.
fn field(f: &FormData, k: &str) -> String {
    if let Some(v) = f.json.as_ref().and_then(|j| j.get(k)) {
        return opts::text(Some(v));
    }
    f.str(k).to_string()
}

fn int_field(f: &FormData, k: &str) -> Option<i64> {
    field(f, k).trim().parse::<i64>().ok().filter(|n| *n > 0)
}

async fn beds(State(state): State<Arc<AppState>>, Extension(v): Extension<Visitor>) -> Response {
    if let Err(r) = signed_in(&v) {
        return *r;
    }
    let beds: Vec<Value> = sq::beds(&state)
        .await
        .into_iter()
        .map(|b| json!({"id": b.id, "name": b.name}))
        .collect();
    reply(StatusCode::OK, json!({ "beds": beds }))
}

async fn list(State(state): State<Arc<AppState>>, Extension(v): Extension<Visitor>) -> Response {
    if let Err(r) = signed_in(&v) {
        return *r;
    }
    let items: Vec<Value> = sq::recent(&state, 100)
        .await
        .iter()
        .map(sq::Item::json)
        .collect();
    let seen = crate::plant_ai::helper_last_seen(&state).await;
    // `owner` tells the phone whether to show the confirm buttons (the server checks it again).
    let owner = crate::plant_ai::is_owner(&state, &v);
    reply(
        StatusCode::OK,
        json!({ "items": items, "helper_last_seen": seen, "owner": owner }),
    )
}

/// The pictures of a new scan, from JSON (`photos: [{side, data}]`) or form files.
fn photos_of(f: &FormData, kind: &str) -> Result<Vec<sq::NewPhoto>, String> {
    let mut out = Vec::new();
    if let Some(a) = f
        .json
        .as_ref()
        .and_then(|j| j.get("photos"))
        .and_then(Value::as_array)
    {
        for p in a {
            let data = p.get("data").and_then(Value::as_str).and_then(b64_decode);
            let bytes = data.ok_or("a picture is not base64")?;
            out.push(sq::NewPhoto {
                side: opts::text(p.get("side")),
                bytes,
            });
        }
    }
    for u in &f.files {
        let f_name = u.field.trim_end_matches("[]");
        let side = match f_name {
            "back" => "back",
            "plant" => "plant",
            "front" => "front",
            // Anything else: the first picture of a tag is the front, the second the back.
            _ if kind == "plant" => "plant",
            _ if out.is_empty() => "front",
            _ => "back",
        };
        out.push(sq::NewPhoto {
            side: side.into(),
            bytes: u.bytes.clone(),
        });
    }
    Ok(out)
}

async fn create(
    State(state): State<Arc<AppState>>,
    Extension(v): Extension<Visitor>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(r) = signed_in(&v) {
        return *r;
    }
    let f = crate::formdata::parse(&headers, body).await;
    let Some(uid) = sq::clean_uid(&field(&f, "client_uid")) else {
        return reply(
            StatusCode::BAD_REQUEST,
            json!({"error": "client_uid must be 8 to 64 letters, digits, - or _"}),
        );
    };
    let kind = field(&f, "kind");
    let photos = match photos_of(&f, sq::clean_kind(&kind)) {
        Ok(p) => p,
        Err(e) => return reply(StatusCode::BAD_REQUEST, json!({"error": e})),
    };
    let qty = int_field(&f, "qty").unwrap_or(1);
    let barcode = field(&f, "barcode");
    let new = sq::NewItem {
        who: &v.username,
        uid: &uid,
        kind: &kind,
        bed_id: int_field(&f, "bed_id"),
        qty,
        barcode: &barcode,
    };
    match sq::create_item(&state, new, photos).await {
        Ok(c) => {
            let item = sq::item(&state, c.id).await;
            reply(
                if c.duplicate {
                    StatusCode::OK
                } else {
                    StatusCode::CREATED
                },
                json!({"ok": true, "duplicate": c.duplicate, "id": c.id,
                       "item": item.as_ref().map(sq::Item::json)}),
            )
        }
        Err(e) => reply(StatusCode::BAD_REQUEST, json!({"error": e})),
    }
}

/// confirm | bed | remove, for one item.
async fn change(
    State(state): State<Arc<AppState>>,
    Extension(v): Extension<Visitor>,
    Path((id, what)): Path<(i64, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(r) = signed_in(&v) {
        return *r;
    }
    let f = crate::formdata::parse(&headers, body).await;
    match what.as_str() {
        "confirm" => {
            let pick = if let Some(m) = f.json.as_ref().and_then(|j| j.get("manual")) {
                sq::Pick::Manual {
                    genus: opts::text(m.get("genus")),
                    species: opts::text(m.get("species")),
                    name: opts::text(m.get("name")),
                }
            } else if !field(&f, "pick").is_empty() {
                sq::Pick::Candidate(field(&f, "pick"))
            } else {
                return reply(
                    StatusCode::BAD_REQUEST,
                    json!({"error": "pick or manual is needed"}),
                );
            };
            match sq::confirm(&state, &v, id, pick).await {
                Ok(c) => reply(
                    StatusCode::OK,
                    json!({"ok": true, "plant_id": c.plant_id, "name": c.name, "bed": c.bed, "merged": c.merged}),
                ),
                Err(e) => {
                    let owner_only = !crate::plant_ai::is_owner(&state, &v);
                    reply(
                        if owner_only {
                            StatusCode::FORBIDDEN
                        } else {
                            StatusCode::CONFLICT
                        },
                        json!({"error": e, "owner_only": owner_only}),
                    )
                }
            }
        }
        "bed" => {
            let Some(bed) = int_field(&f, "bed_id") else {
                return reply(
                    StatusCode::BAD_REQUEST,
                    json!({"error": "bed_id is needed"}),
                );
            };
            if sq::bed_name(&state, bed).await.is_none() {
                return reply(StatusCode::BAD_REQUEST, json!({"error": "no such bed"}));
            }
            let r =
                sqlx::query("UPDATE scan_items SET bed_id = ? WHERE id = ? AND status <> 'added'")
                    .bind(bed)
                    .bind(id)
                    .execute(&state.db)
                    .await;
            match r {
                Ok(r) if r.rows_affected() == 1 => reply(StatusCode::OK, json!({"ok": true})),
                Ok(_) => reply(
                    StatusCode::CONFLICT,
                    json!({"error": "already added (or no such scan)"}),
                ),
                Err(e) => reply(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": e.to_string()}),
                ),
            }
        }
        "remove" => match sq::item(&state, id).await {
            Some(it) if it.status != "added" && it.status != "adding" => {
                match sq::remove_item(&state, id).await {
                    Ok(()) => reply(StatusCode::OK, json!({"ok": true})),
                    Err(e) => reply(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
                }
            }
            Some(_) => reply(StatusCode::CONFLICT, json!({"error": "already added"})),
            None => reply(StatusCode::NOT_FOUND, json!({"error": "no such scan"})),
        },
        _ => reply(StatusCode::NOT_FOUND, json!({"error": "unknown"})),
    }
}

/// "Type it instead": the same six things as the Add plant tab (any signed-in person, as there).
async fn manual(
    State(state): State<Arc<AppState>>,
    Extension(v): Extension<Visitor>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(r) = signed_in(&v) {
        return *r;
    }
    let f = crate::formdata::parse(&headers, body).await;
    let name = sq::tidy(&field(&f, "name"), 255);
    let Some((genus, species)) = sq::genus_species(&field(&f, "genus"), &field(&f, "species"), "")
    else {
        return reply(
            StatusCode::BAD_REQUEST,
            json!({"error": "a genus is needed (letters only)"}),
        );
    };
    if name.is_empty() {
        return reply(
            StatusCode::BAD_REQUEST,
            json!({"error": "a name is needed"}),
        );
    }
    let bed = match int_field(&f, "bed_id") {
        Some(b) => match sq::bed_name(&state, b).await {
            Some(n) => n,
            None => return reply(StatusCode::BAD_REQUEST, json!({"error": "no such bed"})),
        },
        None => sq::WAITING_BED.to_string(),
    };
    let qty = int_field(&f, "qty").unwrap_or(1).clamp(1, 100_000);
    let price = field(&f, "price")
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|p| p.is_finite() && (0.0..1_000_000.0).contains(p));
    match sq::insert_plant(&state, &genus, &species, &name, &bed, qty, price).await {
        Ok(pid) => {
            let ai = crate::plant_ai::queue(&state, pid, &v.username)
                .await
                .is_ok();
            reply(
                StatusCode::CREATED,
                json!({"ok": true, "plant_id": pid, "bed": bed, "ai_queued": ai}),
            )
        }
        Err(e) => reply(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
    }
}

// ---- the desktop helper's side ------------------------------------------------------------------------

/// GET /plant-ai/scans?take=N: up to N waiting scans (1 to 10), marked "reading". A scan left
/// "reading" for 3 hours (the PC was switched off) goes back to waiting.
async fn take(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(q): RawQuery,
) -> Response {
    if let Err(r) = helper(&state, &headers).await {
        return *r;
    }
    let n = serde_urlencoded::from_str::<Vec<(String, String)>>(q.as_deref().unwrap_or(""))
        .unwrap_or_default()
        .into_iter()
        .find(|(k, _)| k == "take")
        .and_then(|(_, v)| v.parse::<i64>().ok())
        .unwrap_or(5)
        .clamp(1, 10);
    let _ = sqlx::query("UPDATE scan_items SET status = 'waiting', step = '' WHERE status = 'reading' AND claimed_at < NOW() - INTERVAL 3 HOUR")
        .execute(&state.db)
        .await;
    let taken = async {
        let mut tx = state.db.begin().await?;
        let ids: Vec<(i64,)> = sqlx::query_as("SELECT id FROM scan_items WHERE status = 'waiting' ORDER BY id LIMIT ? FOR UPDATE")
            .bind(n)
            .fetch_all(&mut *tx)
            .await?;
        for (id,) in &ids {
            sqlx::query("UPDATE scan_items SET status = 'reading', step = 'waiting', claimed_at = NOW() WHERE id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok::<_, sqlx::Error>(ids)
    }
    .await;
    let ids = match taken {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("scan queue take failed: {e}");
            return reply(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "database"}),
            );
        }
    };
    let mut items = Vec::new();
    for (id,) in ids {
        let Some(it) = sq::item(&state, id).await else {
            continue;
        };
        let photos: Vec<(i64, String, i64, i64)> = sqlx::query_as(
            "SELECT id, side, width, height FROM scan_photos WHERE item_id = ? ORDER BY id",
        )
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        let hint: String = sqlx::query_scalar("SELECT barcode_hint FROM scan_items WHERE id = ?")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .unwrap_or_default();
        items.push(json!({
            "id": id, "kind": it.kind, "qty": it.qty, "barcode": hint,
            "photos": photos.iter().map(|(pid, side, w, h)| json!({"id": pid, "side": side, "w": w, "h": h})).collect::<Vec<_>>(),
        }));
    }
    reply(StatusCode::OK, json!({ "items": items }))
}

/// GET /plant-ai/scan-photos/<id>: one picture of a scan the helper has taken.
async fn photo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if let Err(r) = helper(&state, &headers).await {
        return *r;
    }
    let file: Option<String> = sqlx::query_scalar(
        "SELECT p.file FROM scan_photos p JOIN scan_items i ON i.id = p.item_id WHERE p.id = ? AND i.status = 'reading'",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();
    let Some(file) = file.filter(|f| !f.contains('/') && !f.contains('\\') && !f.contains(".."))
    else {
        return reply(
            StatusCode::NOT_FOUND,
            json!({"error": "no such picture, or its scan is not being read"}),
        );
    };
    match std::fs::read(sq::photo_dir(&state).join(file)) {
        Ok(b) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/jpeg"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            b,
        )
            .into_response(),
        Err(_) => reply(
            StatusCode::NOT_FOUND,
            json!({"error": "the picture file is missing"}),
        ),
    }
}

/// POST /plant-ai/scans/<id>/progress | result | fail.
async fn report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, what)): Path<(i64, String)>,
    body: Bytes,
) -> Response {
    if let Err(r) = helper(&state, &headers).await {
        return *r;
    }
    let Ok(v) = serde_json::from_slice::<Value>(&body) else {
        return reply(StatusCode::BAD_REQUEST, json!({"error": "JSON expected"}));
    };
    let status: Option<String> = sqlx::query_scalar("SELECT status FROM scan_items WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();
    if status.as_deref() != Some("reading") {
        return reply(
            StatusCode::CONFLICT,
            json!({"error": "this scan is not waiting for an answer"}),
        );
    }
    match what.as_str() {
        "progress" => {
            let step: String = opts::text(v.get("step"))
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
                .take(40)
                .collect();
            let _ = sqlx::query("UPDATE scan_items SET step = ? WHERE id = ?")
                .bind(step)
                .bind(id)
                .execute(&state.db)
                .await;
            reply(StatusCode::OK, json!({"ok": true}))
        }
        "fail" => {
            let err = sq::tidy(&opts::text(v.get("error")), 500);
            let _ = sqlx::query("UPDATE scan_items SET status = 'failed', error = ?, finished_at = NOW() WHERE id = ?")
                .bind(err)
                .bind(id)
                .execute(&state.db)
                .await;
            reply(StatusCode::OK, json!({"ok": true}))
        }
        "result" => {
            let (status, clean) = sq::judge(v.get("result").unwrap_or(&Value::Null));
            let r = sqlx::query("UPDATE scan_items SET status = ?, step = 'done', result = ?, finished_at = NOW(), error = '' WHERE id = ?")
                .bind(status)
                .bind(clean.to_string())
                .bind(id)
                .execute(&state.db)
                .await;
            match r {
                Ok(_) => reply(StatusCode::OK, json!({"ok": true, "status": status})),
                Err(e) => reply(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": e.to_string()}),
                ),
            }
        }
        _ => reply(StatusCode::NOT_FOUND, json!({"error": "unknown"})),
    }
}

/// The first picture file name in a plant's `picture` text ("a.webp, b.webp").
fn first_photo(raw: &str) -> String {
    raw.split(',')
        .map(str::trim)
        .find(|x| !x.is_empty() && !matches!(x.to_lowercase().as_str(), "none" | "null" | "[]"))
        .unwrap_or("")
        .to_string()
}

/// GET /plant-ai/catalog: every plant in short form (name, botanical name, genus, species,
/// grower, barcodes, first picture), current plants and the wish list, for the helper's matching.
async fn catalog(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(r) = helper(&state, &headers).await {
        return *r;
    }
    let mut barcodes: std::collections::HashMap<(String, i64), Vec<String>> = Default::default();
    for r in crate::orchestrator::rows(
        &state,
        "SELECT plant_id, source_table, barcode FROM plant_barcodes",
    )
    .await
    {
        let t = if opts::text(r.get("source_table")) == "future_plants" {
            "future"
        } else {
            "current"
        };
        let pid = opts::text(r.get("plant_id")).parse::<i64>().unwrap_or(0);
        barcodes
            .entry((t.to_string(), pid))
            .or_default()
            .push(opts::text(r.get("barcode")));
    }
    let mut plants = Vec::new();
    for (set, table) in [("current", "data_source"), ("future", "future_plants")] {
        // The two table names are fixed here, never from the request.
        let sql = format!(
            "SELECT id, plant_name, botanical_name, genus, species, company, barcode, picture, died FROM {table}"
        );
        let sql = if table == "future_plants" {
            sql.replace(", died FROM", ", 0 AS died FROM")
        } else {
            sql
        };
        for r in crate::orchestrator::rows(&state, &sql).await {
            let id = opts::text(r.get("id")).parse::<i64>().unwrap_or(0);
            let mut codes = barcodes
                .get(&(set.to_string(), id))
                .cloned()
                .unwrap_or_default();
            let own = opts::text(r.get("barcode"));
            if !own.trim().is_empty() && !codes.contains(&own) {
                codes.push(own);
            }
            plants.push(json!({
                "id": id, "set": set,
                "name": opts::text(r.get("plant_name")),
                "botanical": opts::text(r.get("botanical_name")),
                "genus": opts::text(r.get("genus")),
                "species": opts::text(r.get("species")),
                "company": opts::text(r.get("company")),
                "barcodes": codes,
                "photo": first_photo(&opts::text(r.get("picture"))),
                "died": opts::text(r.get("died")) == "1",
            }));
        }
    }
    reply(StatusCode::OK, json!({ "plants": plants }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_photo_skips_blanks_and_none() {
        assert_eq!(first_photo("a.webp, b.webp"), "a.webp");
        assert_eq!(first_photo("none"), "");
        assert_eq!(first_photo(" , x.webp"), "x.webp");
    }

    #[test]
    fn form_files_get_their_side() {
        let mut f = FormData::default();
        f.files.push(crate::formdata::Upload {
            field: "front".into(),
            filename: "a.jpg".into(),
            bytes: vec![1],
        });
        f.files.push(crate::formdata::Upload {
            field: "photos[]".into(),
            filename: "b.jpg".into(),
            bytes: vec![2],
        });
        let p = photos_of(&f, "tag").unwrap();
        assert_eq!(p[0].side, "front");
        // The second unnamed picture of a tag is its back.
        assert_eq!(p[1].side, "back");
        let q = photos_of(&f, "plant").unwrap();
        assert_eq!(q[1].side, "plant");
    }

    #[test]
    fn json_photos_must_be_base64() {
        let with = |j: Value| FormData {
            json: Some(j),
            ..Default::default()
        };
        let bad = with(json!({"photos": [{"side": "front", "data": "no*base64"}]}));
        assert!(photos_of(&bad, "tag").is_err());
        let good = with(json!({"photos": [{"side": "back", "data": "aGk"}]}));
        assert_eq!(photos_of(&good, "tag").unwrap()[0].bytes, b"hi");
    }
}
