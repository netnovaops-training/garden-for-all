//! Tab 18, "Scan queue": the website's phone page for scanning tags and plants.
//!
//! HOW TO READ THIS FILE (for Jun). On a phone, the picture boxes below open the camera. Take the
//! tag's front (and back), or one photo of the plant; pick the bed and the count; press the big
//! button. The scan joins the queue, you can scan the next one at once. The PC's helper reads
//! them one after another (it works while you keep adding). When a scan has a best match, one tap
//! on "This one" adds the plant; the garden owner only (as with Approve). The Android app does the
//! same with its own screens and keeps its queue on the phone when there is no signal.
//!
//!   render        draws the tab
//!   action        its buttons (names start with "sq_")
//!   tab_label     the tab's name in English and Korean
//!
//! The work itself (saving a scan, saving a confirmed match) is in `crate::scan_queue`.

use serde_json::Value;

use super::form::Flash;
use super::lang::Lang;
use super::opts::text;
use super::{ui, Req};
use crate::php::{he, php_int};
use crate::scan_queue as sq;
use crate::session::Visitor;
use crate::AppState;

/// The tab's name.
pub fn tab_label(n: usize, l: Lang) -> Option<&'static str> {
    (n == 18).then(|| l.t("📷 Scan queue", "📷 스캔 대기열"))
}

fn hidden(k: &str, v: &str) -> String {
    format!(
        "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
        he(k),
        he(v)
    )
}

fn form(act: &str, id: i64, multipart: bool) -> String {
    format!(
        "<form method=\"post\" action=\"/streamlit/\"{}>{}{}{}",
        if multipart {
            " enctype=\"multipart/form-data\""
        } else {
            ""
        },
        hidden("act", act),
        hidden("tab", "18"),
        if id > 0 {
            hidden("p", &id.to_string())
        } else {
            String::new()
        }
    )
}

/// Words and a colour class for an item's status.
fn status_words(status: &str, step: &str, l: Lang) -> (&'static str, String) {
    let (class, en, ko) = match (status, step) {
        ("waiting", _) => ("sq-wait", "waiting", "대기 중"),
        ("reading", "ocr") | ("reading", "reading tag") => {
            ("sq-work", "reading the tag", "이름표 읽는 중")
        }
        ("reading", "photo") | ("reading", "matching photo") => {
            ("sq-work", "looking at the photo", "사진 살펴보는 중")
        }
        ("reading", "lookup") | ("reading", "looking up") => {
            ("sq-work", "looking it up", "찾아보는 중")
        }
        ("reading", _) => ("sq-work", "reading", "읽는 중"),
        ("matched", _) => ("sq-ok", "matched: one tap", "찾았어요: 한 번만 누르세요"),
        ("needs_you", _) => ("sq-you", "needs you", "확인이 필요해요"),
        ("added", _) => ("sq-done", "added", "추가했어요"),
        ("failed", _) => ("sq-bad", "failed", "실패"),
        _ => ("sq-wait", "", ""),
    };
    (class, l.t(en, ko).to_string())
}

fn source_words(src: &str, l: Lang) -> &'static str {
    match src {
        "garden" => l.t("already in the garden", "이미 정원에 있어요"),
        "wishlist" => l.t("on the wish list", "위시리스트에 있어요"),
        "image" => l.t("looks like (photo)", "사진이 비슷해요"),
        "web" => l.t("found on the internet", "인터넷에서 찾았어요"),
        _ => l.t("read from the tag", "이름표에서 읽었어요"),
    }
}

/// Is any item still waiting for the PC (so the page should look again by itself)?
fn pending(items: &[sq::Item]) -> bool {
    items
        .iter()
        .any(|i| matches!(i.status.as_str(), "waiting" | "reading"))
}

/// Tab 18.
pub async fn render(state: &AppState, req: &Req) -> String {
    let l = req.lang;
    let mut h = ui::header(l.t("📷 Scan queue", "📷 스캔 대기열"));
    h.push_str(&super::quick::helper_line(state, l).await);
    h.push_str(&ui::caption(l.t(
        "Take a photo of a plant tag (front, then back) or of the plant, pick the bed, and scan the next one. The desktop helper reads them while you keep adding.",
        "식물 이름표(앞면, 뒷면) 또는 식물 사진을 찍고, 화단을 고른 다음 바로 다음 것을 스캔하세요. 데스크톱 도우미가 읽는 동안 계속 추가할 수 있어요.",
    )));
    // The new scan: three picture boxes (the phone opens its camera), the bed, the count.
    let beds = sq::beds(state).await;
    let opts_: Vec<(String, String)> = beds
        .iter()
        .map(|b| (b.id.to_string(), b.name.clone()))
        .collect();
    let first = beds
        .iter()
        .find(|b| b.name == sq::WAITING_BED)
        .map(|b| b.id.to_string())
        .unwrap_or_default();
    let pick = |name: &str, label: &str| {
        format!(
            "<div class=\"st-field\"><span class=\"st-label\">{}</span><input class=\"sq-file\" type=\"file\" name=\"{}\" accept=\"image/*\" capture=\"environment\"></div>",
            he(label),
            he(name)
        )
    };
    h.push_str(&form("sq_add", 0, true));
    h.push_str(&pick("front", l.t("📷 Tag, front", "📷 이름표 앞면")));
    h.push_str(&pick(
        "back",
        l.t(
            "📷 Tag, back (if you can turn it over)",
            "📷 이름표 뒷면 (뒤집을 수 있을 때)",
        ),
    ));
    h.push_str(&pick(
        "plant",
        l.t(
            "🌿 OR a photo of the plant itself",
            "🌿 또는 식물 자체 사진",
        ),
    ));
    h.push_str(&ui::columns(
        &[3, 1],
        vec![
            ui::selectbox("bed_id", l.t("Bed", "화단"), &opts_, &first, false),
            ui::number_input("qty", l.t("How many", "몇 개"), "1", Some(1), "1"),
        ],
    ));
    h.push_str(&ui::button(
        "go",
        "1",
        l.t("➕ Add to the queue", "➕ 대기열에 추가"),
        true,
        true,
    ));
    h.push_str("</form>");

    let items = sq::recent(state, 40).await;
    let owner = crate::plant_ai::owner_name(state).unwrap_or_default();
    if items.is_empty() {
        h.push_str(&ui::info(
            l.t("The queue is empty.", "대기열이 비어 있어요."),
        ));
    }
    for it in &items {
        h.push_str(&card(it, &beds, req, &owner));
    }
    if pending(&items) {
        h.push_str(&ui::caption(l.t(
            "This page refreshes by itself while the helper works.",
            "도우미가 일하는 동안 이 페이지는 저절로 새로 고쳐져요.",
        )));
        h.push_str("<script>setTimeout(function(){location.reload()},15000)</script>");
    }
    h
}

/// One scan of the queue: status, bed, the matches and the buttons.
fn card(it: &sq::Item, beds: &[sq::Bed], req: &Req, owner: &str) -> String {
    let l = req.lang;
    let (class, words) = status_words(&it.status, &it.step, l);
    let kind = if it.kind == "plant" {
        l.t("plant photo", "식물 사진")
    } else {
        l.t("tag", "이름표")
    };
    let mut h = format!(
        "<div class=\"sq-card\"><div class=\"sq-head\"><span class=\"sq-chip {class}\">{}</span> <b>#{}</b> · {} · {} × {} · {}</div>",
        he(&words),
        it.id,
        he(kind),
        he(if it.bed_name.is_empty() { sq::WAITING_BED } else { &it.bed_name }),
        it.qty,
        he(&it.created)
    );
    if it.status == "failed" && !it.error.is_empty() {
        h.push_str(&ui::warning(&it.error));
    }
    if matches!(it.status.as_str(), "matched" | "needs_you") {
        h.push_str(&candidates(it, req, owner));
    }
    if it.status == "added" {
        if let Some(pid) = it.plant_id {
            h.push_str(&ui::link_button(
                &format!("/streamlit/?tab=15&p={pid}"),
                l.t("Open this plant (review)", "이 식물 열기 (검토)"),
            ));
        }
    }
    if it.status != "added" {
        // Move to another bed, or take it out of the queue.
        let opts_: Vec<(String, String)> = beds
            .iter()
            .map(|b| (b.id.to_string(), b.name.clone()))
            .collect();
        let cur = it.bed_id.map(|b| b.to_string()).unwrap_or_default();
        let mut f = form("sq_bed", it.id, false);
        f.push_str(&ui::selectbox(
            "bed_id",
            l.t("Bed", "화단"),
            &opts_,
            &cur,
            false,
        ));
        f.push_str(&ui::button(
            "go",
            "1",
            l.t("Change bed", "화단 바꾸기"),
            false,
            false,
        ));
        f.push_str("</form>");
        let mut r = form("sq_remove", it.id, false);
        r.push_str(&ui::button(
            "go",
            "1",
            l.t("🗑 Remove from the queue", "🗑 대기열에서 빼기"),
            false,
            false,
        ));
        r.push_str("</form>");
        h.push_str(&ui::expander(
            l.t("More", "더 보기"),
            false,
            &format!("{f}{r}"),
        ));
    }
    h.push_str("</div>");
    h
}

/// The best match and the likely ones, each with its confidence, and the buttons.
fn candidates(it: &sq::Item, req: &Req, owner: &str) -> String {
    let l = req.lang;
    let mut h = String::new();
    let cands = it
        .result
        .as_ref()
        .and_then(|r| r.get("candidates"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if cands.is_empty() {
        h.push_str(&ui::info(l.t(
            "Nothing matched. Type the plant in by hand below.",
            "맞는 것을 찾지 못했어요. 아래에 직접 입력하세요.",
        )));
    }
    for (i, c) in cands.iter().enumerate() {
        let cid = text(c.get("id"));
        let conf = c.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
        let pct = (conf * 100.0).round() as u32;
        let best = i == 0 && it.status == "matched";
        let name = text(c.get("name"));
        let bot = text(c.get("botanical"));
        h.push_str(&format!(
            "<div class=\"sq-cand{}\"><b>{}{}</b>{} <span class=\"sq-pct\">{pct}%</span><div class=\"pa-bar\"><span style=\"width:{pct}%\"></span></div><small>{}{}</small>",
            if best { " sq-best" } else { "" },
            if best { he(l.t("Best match: ", "가장 가까운 것: ")) } else { String::new() },
            he(&name),
            if bot.is_empty() { String::new() } else { format!(" <i>({})</i>", he(&bot)) },
            he(source_words(&text(c.get("source")), l)),
            { let w = text(c.get("why")); if w.is_empty() { String::new() } else { format!(" · {}", he(&w)) } },
        ));
        if req.owner {
            let mut f = form("sq_confirm", it.id, false);
            f.push_str(&hidden("pick", &cid));
            f.push_str(&ui::button(
                "go",
                "1",
                if text(c.get("set")) == "current" {
                    l.t(
                        "✅ Same plant: add to its count",
                        "✅ 같은 식물: 수량에 더하기",
                    )
                } else {
                    l.t("✅ This one", "✅ 이거예요")
                },
                best,
                false,
            ));
            f.push_str("</form>");
            h.push_str(&f);
        }
        h.push_str("</div>");
    }
    if req.owner {
        let mut f = form("sq_manual", it.id, false);
        f.push_str(&ui::columns(
            &[1, 1],
            vec![
                ui::text_input("genus", l.t("Genus", "속"), "", "Salvia"),
                ui::text_input(
                    "species",
                    l.t("Species (if known)", "종 (알면)"),
                    "",
                    "nemorosa",
                ),
            ],
        ));
        f.push_str(&ui::text_input(
            "name",
            l.t("Name", "이름"),
            "",
            "May Night Salvia",
        ));
        f.push_str(&ui::button(
            "go",
            "1",
            l.t("Add it this way", "이렇게 추가"),
            false,
            false,
        ));
        f.push_str("</form>");
        h.push_str(&ui::expander(
            l.t(
                "None of these: type it instead",
                "맞는 게 없어요: 직접 입력",
            ),
            cands.is_empty(),
            &f,
        ));
    } else {
        h.push_str(&ui::caption(&format!(
            "{} {owner}",
            l.t(
                "Only the garden owner can confirm:",
                "확인은 정원 주인만 할 수 있어요:"
            )
        )));
    }
    h
}

/// The buttons of this tab. Gives the messages and the address settings to go back to.
pub async fn action(
    state: &AppState,
    who: &Visitor,
    act: &str,
    f: &crate::formdata::FormData,
    l: Lang,
) -> (Vec<Flash>, Vec<(&'static str, String)>) {
    let id = php_int(f.str("p"));
    let err = |m: String| (vec![Flash::Error(m)], vec![]);
    match act {
        "sq_add" => {
            let bed = php_int(f.str("bed_id"));
            let qty = php_int(f.str("qty")).clamp(1, 1000);
            let mut photos = Vec::new();
            for side in ["front", "back", "plant"] {
                for u in f
                    .files_for(side)
                    .into_iter()
                    .filter(|u| !u.bytes.is_empty())
                {
                    photos.push(sq::NewPhoto {
                        side: side.into(),
                        bytes: u.bytes.clone(),
                    });
                }
            }
            let kind = if photos.iter().any(|p| p.side == "plant")
                && !photos.iter().any(|p| p.side == "front")
            {
                "plant"
            } else {
                "tag"
            };
            // The browser has no id of its own: a random one per press.
            let uid = format!("web-{:032x}", rand::random::<u128>());
            let new = sq::NewItem {
                who: &who.username,
                uid: &uid,
                kind,
                bed_id: (bed > 0).then_some(bed),
                qty,
                barcode: "",
            };
            match sq::create_item(state, new, photos).await {
                Ok(_) => (
                    vec![Flash::Success(
                        l.t(
                            "✅ In the queue. You can scan the next one.",
                            "✅ 대기열에 들어갔어요. 다음 것을 스캔하세요.",
                        )
                        .into(),
                    )],
                    vec![],
                ),
                Err(e) => err(e),
            }
        }
        "sq_confirm" | "sq_manual" => {
            let pick = if act == "sq_confirm" {
                sq::Pick::Candidate(f.str("pick").to_string())
            } else {
                sq::Pick::Manual {
                    genus: f.str("genus").to_string(),
                    species: f.str("species").to_string(),
                    name: f.str("name").to_string(),
                }
            };
            match sq::confirm(state, who, id, pick).await {
                Ok(c) => {
                    let m = if c.merged {
                        l.t("✅ Added to the count of", "✅ 수량에 더했어요:")
                    } else {
                        l.t("✅ Added:", "✅ 추가했어요:")
                    };
                    (
                        vec![Flash::Success(format!("{m} **{}** ({})", c.name, c.bed))],
                        vec![],
                    )
                }
                Err(e) => err(e),
            }
        }
        "sq_bed" => {
            let bed = php_int(f.str("bed_id"));
            if sq::bed_name(state, bed).await.is_none() {
                return err(l.t("No such bed.", "그런 화단이 없어요.").into());
            }
            let _ =
                sqlx::query("UPDATE scan_items SET bed_id = ? WHERE id = ? AND status <> 'added'")
                    .bind(bed)
                    .bind(id)
                    .execute(&state.db)
                    .await;
            (
                vec![Flash::Success(
                    l.t("Bed changed.", "화단을 바꿨어요.").into(),
                )],
                vec![],
            )
        }
        "sq_remove" => match sq::item(state, id).await {
            Some(it) if it.status != "added" => match sq::remove_item(state, id).await {
                Ok(()) => (
                    vec![Flash::Info(
                        l.t("Removed from the queue.", "대기열에서 뺐어요.").into(),
                    )],
                    vec![],
                ),
                Err(e) => err(e),
            },
            _ => err(l
                .t("That scan cannot be removed.", "그 스캔은 뺄 수 없어요.")
                .into()),
        },
        _ => err("Unknown action.".into()),
    }
}
