//! The short path for a new plant: tab 15 "Add plant", tab 16 "Receipt", tab 17 "Pictures".
//!
//! HOW TO READ THIS FILE (for Jun). Add plant asks for six things only (genus, species, name,
//! bed, quantity, price), saves the plant, and puts it in the AI queue (crate::plant_ai). The
//! desktop helper fills in what it can find; the same tab then shows the review: what was
//! filled, what needs a person, what is still missing, and the Approve button (garden owner
//! only). Receipt keeps the receipt with the plant; Pictures adds photos (one .webp each).
//! Everything a person reads here is in English or Korean (lang.rs).
//!
//!   render_add / render_receipt / render_pictures   draw the three tabs
//!   action                                          their buttons (names start with "pa_")
//!   missing_export                                  the "Export what's still missing" file
//!   receipt_file                                    a receipt, for its download link

use serde_json::{json, Map, Value};

use super::form::Flash;
use super::lang::Lang;
use super::opts::{self, text};
use super::{rows, ui, Req};
use crate::php::{he, php_int};
use crate::plant_ai;
use crate::session::Visitor;
use crate::AppState;

/// Field `k` of a row as text.
fn s(m: &Map<String, Value>, k: &str) -> String {
    text(m.get(k))
}

/// Labels for the three new tabs, and the Advanced menu, in the chosen language.
pub fn tab_label(n: usize, l: Lang) -> Option<&'static str> {
    Some(match (n, l) {
        (15, Lang::En) => "🌱 Add plant",
        (15, Lang::Ko) => "🌱 식물 추가",
        (16, Lang::En) => "🧾 Receipt",
        (16, Lang::Ko) => "🧾 영수증",
        (17, Lang::En) => "🖼️ Pictures",
        (17, Lang::Ko) => "🖼️ 사진",
        _ => return None,
    })
}

/// A hidden form field.
fn hidden(k: &str, v: &str) -> String {
    format!(
        "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
        he(k),
        he(v)
    )
}

/// The start of a POST form for one of these tabs.
fn post_form(act: &str, tab: usize, p: &str, multipart: bool) -> String {
    format!(
        "<form method=\"post\" action=\"/streamlit/\"{}>{}{}{}",
        if multipart {
            " enctype=\"multipart/form-data\""
        } else {
            ""
        },
        hidden("act", act),
        hidden("tab", &tab.to_string()),
        if p.is_empty() {
            String::new()
        } else {
            hidden("p", p)
        }
    )
}

/// The plant picker for Receipt and Pictures: newest first, the chosen one kept.
async fn picker(state: &AppState, req: &Req) -> (String, Option<Map<String, Value>>) {
    let l = req.lang;
    let list = rows(state, "SELECT id, plant_name, botanical_name, picture, qty, average_cost FROM data_source ORDER BY id DESC").await;
    if list.is_empty() {
        return (
            ui::info(l.t(
                "No plants yet. Add one on the Add plant tab first.",
                "아직 식물이 없어요. 먼저 '식물 추가' 탭에서 추가하세요.",
            )),
            None,
        );
    }
    let want = req.get("p");
    let chosen = list
        .iter()
        .find(|p| s(p, "id") == want)
        .unwrap_or(&list[0])
        .clone();
    let opts_: Vec<(String, String)> = list
        .iter()
        .map(|p| {
            (
                s(p, "id"),
                format!("{} (ID {})", s(p, "plant_name"), s(p, "id")),
            )
        })
        .collect();
    let h = format!(
        "<form method=\"get\" action=\"/streamlit/\">{}{}</form>",
        hidden("tab", &req.tab.to_string()),
        ui::selectbox("p", l.t("Plant", "식물"), &opts_, &s(&chosen, "id"), true)
    );
    (h, Some(chosen))
}

/// One photo, shown from its one .webp: sized by the page (CSS), loaded only when scrolled to.
/// No separate thumbnail file is asked for.
fn photo(name: &str) -> String {
    format!(
        "<img class=\"pa-photo\" loading=\"lazy\" decoding=\"async\" src=\"/images/Database/{}\" alt=\"\">",
        ui::path_url(name)
    )
}

/// The photo names of a plant ("a.webp, b.webp").
fn pics_of(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty() && !matches!(x.to_lowercase().as_str(), "none" | "null" | "[]"))
        .collect()
}

/// How far along a job is, 0-100, and its words.
fn progress(status: &str, step: &str, l: Lang) -> (u32, String) {
    let (pct, en, ko) = match (status, step) {
        ("queued", _) => (
            5,
            "waiting for the desktop helper",
            "데스크톱 도우미를 기다리는 중",
        ),
        ("working", "searching") => (25, "searching the web", "웹에서 찾는 중"),
        ("working", "filling") => (50, "filling in the facts (phi4)", "정보 채우는 중 (phi4)"),
        ("working", "photos") => (65, "finding photos", "사진 찾는 중"),
        ("working", "checking") => (
            85,
            "checking every value (gemma)",
            "모든 값을 확인하는 중 (gemma)",
        ),
        ("working", _) => (10, "started", "시작함"),
        ("review", _) | ("approved", _) => (100, "ready for your review", "검토할 준비가 됐어요"),
        ("failed", _) => (100, "stopped", "멈춤"),
        _ => (0, "", ""),
    };
    (pct, l.t(en, ko).to_string())
}

fn bar(pct: u32) -> String {
    format!("<div class=\"pa-bar\"><span style=\"width:{pct}%\"></span></div>")
}

/// Plain words for the checker's verdict.
fn verdict(d: &str, l: Lang) -> &'static str {
    match d {
        "Accept" => l.t("looks right", "맞아 보여요"),
        "Reject" => l.t(
            "probably a different plant: nothing was written",
            "다른 식물 같아요: 아무것도 쓰지 않았어요",
        ),
        _ => l.t("needs a look", "확인이 필요해요"),
    }
}

// ---- tab 15: Add plant -----------------------------------------------------------------------

/// Tab 15: the six boxes, the queue, the review of one plant, and the desktop helper.
pub async fn render_add(state: &AppState, req: &Req) -> String {
    let l = req.lang;
    let mut h = ui::header(l.t("🌱 Add a plant", "🌱 식물 추가"));
    h.push_str(&helper_line(state, l).await);
    // The six boxes.
    let beds: Vec<String> = opts::GARDEN_BEDS.iter().map(|b| b.to_string()).collect();
    h.push_str(&post_form("pa_add", 15, "", false));
    h.push_str(&ui::columns(
        &[1, 1],
        vec![
            ui::text_input(
                "genus",
                l.t(
                    "Genus (first word of the Latin name)",
                    "속 (학명의 첫 단어)",
                ),
                "",
                "Salvia",
            ),
            ui::text_input(
                "species",
                l.t(
                    "Species (second word; 'hybrid' if it is one)",
                    "종 (두 번째 단어; 교배종이면 'hybrid')",
                ),
                "",
                "nemorosa",
            ),
        ],
    ));
    h.push_str(&ui::text_input(
        "name",
        l.t("Name (as on the tag)", "이름 (이름표에 쓰인 대로)"),
        "",
        "May Night Salvia",
    ));
    h.push_str(&ui::columns(
        &[2, 1, 1],
        vec![
            ui::select_plain("bed", l.t("Bed", "화단"), &beds, "To Be Sorted"),
            ui::number_input("qty", l.t("Quantity", "수량"), "1", Some(0), "1"),
            ui::number_input(
                "price",
                l.t("Price each ($)", "개당 가격 ($)"),
                "",
                Some(0),
                "0.01",
            ),
        ],
    ));
    h.push_str(&ui::checkbox(
        "ai",
        l.t(
            "Let the desktop AI fill in the rest",
            "나머지는 데스크톱 AI가 채우게 하기",
        ),
        true,
        None,
    ));
    h.push_str(&ui::button(
        "go",
        "1",
        l.t("➕ Add plant", "➕ 식물 추가"),
        true,
        false,
    ));
    h.push_str("</form>");
    // The queue: the newest jobs, each a link to its review.
    let jobs = plant_ai::recent_jobs(state, 12).await;
    let mut pending = false;
    if !jobs.is_empty() {
        h.push_str(&ui::subheader(
            l.t("Plants with the AI", "AI가 작업 중인 식물"),
        ));
        let mut t = String::from("<div class=\"st-df\"><table><thead><tr>");
        for c in [
            l.t("Plant", "식물"),
            l.t("Progress", "진행"),
            l.t("Added", "추가한 때"),
        ] {
            t.push_str(&format!("<th>{}</th>", he(c)));
        }
        t.push_str("</tr></thead><tbody>");
        for j in &jobs {
            let name: String =
                sqlx::query_scalar("SELECT plant_name FROM data_source WHERE id = ?")
                    .bind(j.plant_id)
                    .fetch_optional(&state.db)
                    .await
                    .ok()
                    .flatten()
                    .flatten()
                    .unwrap_or_default();
            let (pct, words) = progress(&j.status, &j.step, l);
            pending |= matches!(j.status.as_str(), "queued" | "working");
            t.push_str(&format!(
                "<tr><td><a href=\"/streamlit/?tab=15&amp;p={}\">{}</a></td><td>{}{}</td><td>{}</td></tr>",
                j.plant_id,
                he(&name),
                bar(pct),
                he(&words),
                he(&j.created)
            ));
        }
        t.push_str("</tbody></table></div>");
        h.push_str(&t);
    }
    // One plant's review.
    let p = php_int(req.get("p"));
    if p > 0 {
        h.push_str(&ui::divider());
        h.push_str(&review(state, req, p).await);
    }
    // While something is still being worked on, the page looks again every 15 seconds.
    if pending {
        h.push_str(&ui::caption(l.t(
            "This page refreshes by itself while the AI works.",
            "AI가 작업하는 동안 이 페이지는 저절로 새로 고쳐져요.",
        )));
        h.push_str("<script>setTimeout(function(){location.reload()},15000)</script>");
    }
    h.push_str(&ui::divider());
    h.push_str(&helper_box(state, req).await);
    h
}

/// One line: is the desktop helper on?
async fn helper_line(state: &AppState, l: Lang) -> String {
    let seen = plant_ai::helper_last_seen(state).await;
    let fresh: bool = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM plant_ai_keys WHERE revoked = 0 AND last_seen > NOW() - INTERVAL 10 MINUTE")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0)
        > 0;
    if fresh {
        ui::success(&format!(
            "{} {seen}",
            l.t(
                "The desktop helper is on. Last asked for work:",
                "데스크톱 도우미가 켜져 있어요. 마지막으로 일을 물어본 때:"
            )
        ))
    } else if !seen.is_empty() {
        ui::info(&format!(
            "{} {seen}). {}",
            l.t(
                "The desktop helper is off (last seen",
                "데스크톱 도우미가 꺼져 있어요 (마지막으로 본 때"
            ),
            l.t(
                "New plants wait in the queue and are filled when it is on.",
                "새 식물은 대기열에서 기다리다가 도우미가 켜지면 채워져요."
            )
        ))
    } else {
        ui::info(l.t(
            "No desktop helper yet. Plants are saved now and wait in the queue; the AI fills them once a helper is set up (Desktop helper, below). The AI runs only on a Windows PC with an NVIDIA card with 12 GB or more (like an RTX 4070 Ti).",
            "아직 데스크톱 도우미가 없어요. 식물은 지금 저장되고 대기열에서 기다려요. 도우미를 설정하면 AI가 채워요 (아래 '데스크톱 도우미'). AI는 12GB 이상의 NVIDIA 그래픽 카드(예: RTX 4070 Ti)가 있는 Windows PC에서만 돌아가요.",
        ))
    }
}

/// The review of one plant: progress, or filled / needs review / still missing, then Approve.
async fn review(state: &AppState, req: &Req, pid: i64) -> String {
    let l = req.lang;
    let Some(row) = plant_ai::plant_row(state, pid).await else {
        return ui::error(l.t("That plant was not found.", "그 식물을 찾지 못했어요."));
    };
    let mut h = ui::subheader(&format!(
        "{} {}",
        l.t("Review:", "검토:"),
        s(&row, "plant_name")
    ));
    let job = plant_ai::latest_job(state, pid).await;
    let id = pid.to_string();
    match job.as_ref().map(|j| j.status.as_str()) {
        None => h.push_str(&ui::info(l.t(
            "The AI was not asked for this plant. Press 'Ask the AI' to queue it.",
            "이 식물은 AI에게 맡기지 않았어요. 'AI에게 맡기기'를 누르면 대기열에 들어가요.",
        ))),
        Some("queued") | Some("working") => {
            let j = job
                .as_ref()
                .map(|j| (j.status.clone(), j.step.clone()))
                .unwrap_or_default();
            let (pct, words) = progress(&j.0, &j.1, l);
            h.push_str(&bar(pct));
            h.push_str(&ui::markdown(&format!("**{}**", words)));
        }
        Some("failed") => {
            let e = job.as_ref().map(|j| j.error.clone()).unwrap_or_default();
            h.push_str(&ui::warning(&format!(
                "{} {e}",
                l.t(
                    "The AI stopped for this plant:",
                    "이 식물에서 AI가 멈췄어요:"
                )
            )));
        }
        Some(_) => {}
    }
    // The answers, when they are in.
    if let Some(res) = job.as_ref().and_then(|j| j.result.clone()) {
        h.push_str(&answers(&row, &res, l, pid));
    }
    // Still missing (the site's own list of required fields), and the export for them.
    let missing: Vec<&str> = opts::missing_fields(&row)
        .into_iter()
        .filter(|f| !EXPORT_SKIP.contains(f))
        .collect();
    h.push_str(&ui::h4(&format!(
        "{} ({})",
        l.t("Still missing", "아직 비어 있음"),
        missing.len()
    )));
    if missing.is_empty() {
        h.push_str(&ui::success(
            l.t("Nothing is missing.", "빠진 것이 없어요."),
        ));
    } else {
        h.push_str(&format!(
            "<p class=\"pa-chips\">{}</p>",
            missing
                .iter()
                .map(|f| format!("<code>{}</code>", he(f)))
                .collect::<Vec<_>>()
                .join(" ")
        ));
        h.push_str(&ui::link_button(
            &format!("/streamlit/?export=pa_missing&id={pid}"),
            l.t(
                "📤 Export what's still missing",
                "📤 아직 비어 있는 것 내보내기",
            ),
        ));
        h.push_str(&ui::caption(l.t(
            "A small file with only the empty fields, their allowed values and a ready prompt. Give it to your own Claude, save its answer as a .json file, and bring it back with Advanced → JSON Pipeline → Import (you see every value before it is saved; filled fields are never overwritten).",
            "비어 있는 칸, 허용되는 값, 바로 쓸 수 있는 프롬프트만 담은 작은 파일이에요. 내 Claude에게 주고, 답을 .json 파일로 저장한 뒤 고급 → JSON Pipeline → 가져오기로 넣으세요 (저장 전에 모든 값을 보여 주고, 이미 채워진 칸은 절대 덮어쓰지 않아요).",
        )));
        h.push_str(&ui::link_button(
            "/streamlit/?tab=11&sub=4",
            l.t("📥 Advanced: JSON import", "📥 고급: JSON 가져오기"),
        ));
    }
    // Approve: the garden owner only.
    h.push_str(&ui::h4(l.t("Approve", "승인")));
    let approved = plant_ai::is_approved(&row);
    if approved {
        h.push_str(&ui::success(l.t(
            "✅ Approved by the garden owner.",
            "✅ 정원 주인이 승인했어요.",
        )));
    }
    if req.owner {
        h.push_str(&post_form(
            if approved {
                "pa_unapprove"
            } else {
                "pa_approve"
            },
            15,
            &id,
            false,
        ));
        h.push_str(&ui::button(
            "go",
            "1",
            if approved {
                l.t("Undo approval", "승인 취소")
            } else {
                l.t("✅ Approve", "✅ 승인")
            },
            !approved,
            false,
        ));
        h.push_str("</form>");
    } else {
        let owner = plant_ai::owner_name(state).unwrap_or_default();
        h.push_str(&ui::caption(&format!(
            "{} {owner}",
            l.t(
                "Only the garden owner can approve:",
                "승인은 정원 주인만 할 수 있어요:"
            )
        )));
    }
    // Other steps for this plant.
    let mut more = post_form("pa_requeue", 15, &id, false);
    more.push_str(&ui::button(
        "go",
        "1",
        l.t("🔁 Ask the AI (again)", "🔁 AI에게 (다시) 맡기기"),
        false,
        false,
    ));
    more.push_str("</form>");
    h.push_str(&ui::columns(
        &[1, 1, 1, 1],
        vec![
            more,
            ui::link_button(
                &format!("/streamlit/?tab=16&p={pid}"),
                l.t("🧾 Receipt", "🧾 영수증"),
            ),
            ui::link_button(
                &format!("/streamlit/?tab=17&p={pid}"),
                l.t("🖼️ Pictures", "🖼️ 사진"),
            ),
            ui::link_button(
                &format!("/streamlit/?tab=4&p={pid}"),
                l.t("✍️ Full editor", "✍️ 전체 편집기"),
            ),
        ],
    ));
    h
}

/// Fields the "still missing" list and its export leave out: photos, beds and the price come
/// from the person (Pictures tab, Add plant), never from an AI.
const EXPORT_SKIP: [&str; 3] = ["picture", "garden_beds", "average_cost"];

/// The AI's answers for one plant: verdict, Source link, photos, filled, needs review.
fn answers(row: &Map<String, Value>, res: &Value, l: Lang, pid: i64) -> String {
    let mut h = String::new();
    let dec = res.get("decision").and_then(Value::as_str).unwrap_or("");
    let reason = res.get("reason").and_then(Value::as_str).unwrap_or("");
    h.push_str(&ui::markdown(&format!(
        "**{}** {} — {}",
        l.t("The checker says:", "확인 결과:"),
        verdict(dec, l),
        reason
    )));
    if let Some(u) = res
        .get("product_url")
        .and_then(Value::as_str)
        .and_then(plant_ai::clean_url)
    {
        h.push_str(&format!(
            "<p class=\"st-text\">{} <a href=\"{}\" rel=\"noopener noreferrer\" target=\"_blank\">{}</a></p>",
            he(l.t("Source:", "출처:")),
            he(&u),
            he(&u)
        ));
    }
    // Photos saved for this plant (each its one .webp; credits under them).
    let saved: Vec<String> = res
        .get("photos_saved")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if !saved.is_empty() {
        h.push_str(&ui::h4(&format!(
            "{} ({})",
            l.t("Photos found", "찾은 사진"),
            saved.len()
        )));
        h.push_str(&format!(
            "<div class=\"pa-photos\">{}</div>",
            saved.iter().map(|n| photo(n)).collect::<String>()
        ));
        let credits: Vec<String> = res
            .get("photo_credits")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .map(|c| text(c.get("credit")))
                    .filter(|c| !c.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        if !credits.is_empty() {
            h.push_str(&ui::caption(&credits.join(" · ")));
        }
    }
    // Filled: the values that were written (shown as they are in the garden now).
    let written: Vec<String> = res
        .get("written")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .filter(|f| f != "picture")
                .collect()
        })
        .unwrap_or_default();
    h.push_str(&ui::h4(&format!(
        "{} ({})",
        l.t("Filled by the AI", "AI가 채운 것"),
        written.len()
    )));
    if !written.is_empty() {
        let rows_: Vec<Vec<String>> = written
            .iter()
            .map(|f| {
                let quote = res
                    .pointer(&format!("/quotes/{f}"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let src = res
                    .pointer(&format!("/sources/{f}/0"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                vec![
                    he(f),
                    he(&s(row, f)),
                    format!(
                        "<span class=\"pa-quote\">{}</span>{}",
                        he(&quote.chars().take(160).collect::<String>()),
                        source_link(src)
                    ),
                ]
            })
            .collect();
        h.push_str(&ui::dataframe(
            &[
                l.t("Field", "칸"),
                l.t("Value", "값"),
                l.t("Where it says so", "근거"),
            ],
            &rows_,
        ));
    }
    // Needs review: what the checker could not confirm. Never written until a person ticks it.
    let nr = res
        .get("needs_review")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    h.push_str(&ui::h4(&format!(
        "{} ({})",
        l.t("Needs review", "검토 필요"),
        nr.len()
    )));
    if !nr.is_empty() {
        let mut f = post_form("pa_use", 15, &pid.to_string(), false);
        let mut t = String::from("<div class=\"st-df\"><table><thead><tr><th></th>");
        for c in [
            l.t("Field", "칸"),
            l.t("Suggested", "제안 값"),
            l.t("Why it needs a look", "확인이 필요한 이유"),
        ] {
            t.push_str(&format!("<th>{}</th>", he(c)));
        }
        t.push_str("</tr></thead><tbody>");
        for (k, x) in &nr {
            let filled = !opts::is_blank(&s(row, k));
            t.push_str(&format!(
                "<tr><td><input type=\"checkbox\" name=\"use_{}\" value=\"1\"{}></td><td>{}</td><td>{}</td><td>{} <span class=\"pa-quote\">{}</span>{}</td></tr>",
                he(k),
                if filled { " disabled" } else { "" },
                he(k),
                he(&text(x.get("v"))),
                he(&text(x.get("why")).chars().take(160).collect::<String>()),
                he(&text(x.get("quote")).chars().take(160).collect::<String>()),
                source_link(x.pointer("/sources/0").and_then(Value::as_str).unwrap_or(""))
            ));
        }
        t.push_str("</tbody></table></div>");
        f.push_str(&t);
        f.push_str(&ui::button(
            "go",
            "1",
            l.t("Use the ticked values", "체크한 값 쓰기"),
            false,
            false,
        ));
        f.push_str("</form>");
        h.push_str(&f);
        h.push_str(&ui::caption(l.t(
            "Only empty fields can be ticked; a value already there is never replaced.",
            "빈 칸만 체크할 수 있어요. 이미 있는 값은 절대 바꾸지 않아요.",
        )));
    }
    h
}

/// A small "open the page" link for a source address.
fn source_link(u: &str) -> String {
    match plant_ai::clean_url(u) {
        Some(u) => format!(
            " <a href=\"{}\" rel=\"noopener noreferrer\" target=\"_blank\">↗</a>",
            he(&u)
        ),
        None => String::new(),
    }
}

/// The desktop helper box: what it is, its keys, a button for a new key.
async fn helper_box(state: &AppState, req: &Req) -> String {
    let l = req.lang;
    let mut b = ui::markdown(l.t(
        "The AI work runs on a **Windows PC with an NVIDIA card (12 GB or more, like an RTX 4070 Ti)**, set up by the Windows setup wizard's 'Local plant AI' step. The website can run anywhere (a laptop is fine): the PC asks the website for work every minute, over your home network, through the website's usual address. Nothing is opened on either computer. Plants added while the PC is off wait in the queue.",
        "AI 작업은 **NVIDIA 그래픽 카드(12GB 이상, 예: RTX 4070 Ti)가 있는 Windows PC**에서 돌아가요. Windows 설치 마법사의 '로컬 식물 AI' 단계에서 설정해요. 웹사이트는 어디에서 돌아가도 돼요 (노트북도 괜찮아요): PC가 1분마다 집 네트워크를 통해 웹사이트의 평소 주소로 일을 물어봐요. 두 컴퓨터 모두 새로 여는 포트가 없어요. PC가 꺼져 있을 때 추가한 식물은 대기열에서 기다려요.",
    ));
    let keys: Vec<(i64, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, label, DATE_FORMAT(created_at, '%Y-%m-%d'), DATE_FORMAT(last_seen, '%Y-%m-%d %H:%i') FROM plant_ai_keys WHERE revoked = 0 ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    for (id, label, made, seen) in keys {
        let mut f = post_form("pa_key_off", 15, "", false);
        f.push_str(&hidden("key", &id.to_string()));
        f.push_str(&format!(
            "<p class=\"st-text\">🔑 <strong>{}</strong> · {} {} · {} {} {}</p>",
            he(&label),
            he(l.t("made", "만든 날")),
            he(&made),
            he(l.t("last used", "마지막 사용")),
            he(&seen.unwrap_or_else(|| l.t("never", "없음").into())),
            ui::button("go", "1", l.t("Switch off", "끄기"), false, false)
        ));
        f.push_str("</form>");
        b.push_str(&f);
    }
    if req.owner {
        let mut f = post_form("pa_key_new", 15, "", false);
        f.push_str(&ui::text_input(
            "label",
            l.t(
                "Name for the new key (which PC)",
                "새 키 이름 (어느 PC인지)",
            ),
            "Desktop PC",
            "",
        ));
        f.push_str(&ui::button(
            "go",
            "1",
            l.t("🔑 Make a helper key", "🔑 도우미 키 만들기"),
            false,
            false,
        ));
        f.push_str("</form>");
        b.push_str(&f);
    }
    ui::expander(
        l.t(
            "🖥️ Desktop helper (local plant AI)",
            "🖥️ 데스크톱 도우미 (로컬 식물 AI)",
        ),
        false,
        &b,
    )
}

/// The page shown once after "Make a helper key": the key itself, never shown again.
pub fn key_page(key: &str, l: Lang) -> String {
    let mut h = ui::header(l.t("🔑 Your new helper key", "🔑 새 도우미 키"));
    h.push_str(&ui::warning(l.t(
        "This key is shown ONCE. Copy it now into the Windows setup wizard's 'Local plant AI' page on the PC (or into plant-ai\\worker-key.txt there). The website keeps only its fingerprint; if it is lost, make a new one and switch the old one off.",
        "이 키는 딱 한 번만 보여요. 지금 PC의 Windows 설치 마법사 '로컬 식물 AI' 페이지(또는 그 PC의 plant-ai\\worker-key.txt)에 복사하세요. 웹사이트는 키의 지문만 보관해요. 잃어버리면 새로 만들고 예전 키는 끄세요.",
    )));
    h.push_str(&format!(
        "<p><input class=\"st-input pa-key\" readonly value=\"{}\" onclick=\"this.select()\"></p>",
        he(key)
    ));
    h.push_str(&ui::link_button("/streamlit/?tab=15", l.t("Done", "완료")));
    h
}

// ---- tab 16: Receipt ---------------------------------------------------------------------------

/// Tab 16: keep a receipt with a plant (photo or PDF), with the store, date, quantity and price.
pub async fn render_receipt(state: &AppState, req: &Req) -> String {
    let l = req.lang;
    let mut h = ui::header(l.t("🧾 Receipt", "🧾 영수증"));
    h.push_str(&ui::text(l.t(
        "Keep the receipt with the plant. It stays private: receipts are not on the public website.",
        "영수증을 식물과 함께 보관해요. 영수증은 공개 웹사이트에 나오지 않아요.",
    )));
    let (pick, chosen) = picker(state, req).await;
    h.push_str(&pick);
    let Some(p) = chosen else { return h };
    let id = s(&p, "id");
    h.push_str(&post_form("pa_receipt", 16, &id, true));
    h.push_str(&ui::columns(
        &[2, 1],
        vec![
            ui::text_input("store", l.t("Store", "가게"), "", ""),
            format!("<label class=\"st-field\"><span class=\"st-label\">{}</span><input class=\"st-input\" type=\"date\" name=\"date\" value=\"{}\"></label>", he(l.t("Date bought", "산 날짜")), opts::today()),
        ],
    ));
    h.push_str(&ui::columns(
        &[1, 1],
        vec![
            ui::number_input("qty", l.t("Quantity", "수량"), &s(&p, "qty"), Some(0), "1"),
            ui::number_input(
                "price",
                l.t("Price each ($)", "개당 가격 ($)"),
                &s(&p, "average_cost"),
                Some(0),
                "0.01",
            ),
        ],
    ));
    h.push_str(&ui::file_uploader(
        "receipt",
        l.t("Receipt photo or PDF", "영수증 사진 또는 PDF"),
        &["jpg", "jpeg", "png", "webp", "heic", "pdf"],
        false,
    ));
    h.push_str(&ui::button(
        "go",
        "1",
        l.t("💾 Save receipt", "💾 영수증 저장"),
        true,
        false,
    ));
    h.push_str("</form>");
    // The receipts kept for this plant.
    let list: Vec<ReceiptRow> = sqlx::query_as(
        "SELECT id, DATE_FORMAT(bought_on, '%Y-%m-%d'), store, quantity, CAST(unit_price AS CHAR), receipt_ref FROM plant_purchase WHERE plant_id = ? ORDER BY id DESC",
    )
    .bind(php_int(&id))
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    if !list.is_empty() {
        h.push_str(&ui::subheader(
            l.t("Receipts for this plant", "이 식물의 영수증"),
        ));
        let rows_: Vec<Vec<String>> = list
            .into_iter()
            .map(|(rid, d, st, q, pr, rf)| {
                let file = if rf.as_deref().is_some_and(safe_receipt_name) {
                    format!(
                        "<a href=\"/streamlit/?export=pa_receipt&amp;id={rid}\">{}</a>",
                        he(l.t("open", "열기"))
                    )
                } else {
                    String::new()
                };
                vec![
                    he(&d.unwrap_or_default()),
                    he(&st.unwrap_or_default()),
                    q.to_string(),
                    he(&pr.unwrap_or_default()),
                    file,
                ]
            })
            .collect();
        h.push_str(&ui::dataframe(
            &[
                l.t("Date", "날짜"),
                l.t("Store", "가게"),
                l.t("Qty", "수량"),
                l.t("Price", "가격"),
                l.t("File", "파일"),
            ],
            &rows_,
        ));
    }
    h
}

/// One kept receipt: (id, date, store, quantity, price, file name).
type ReceiptRow = (
    i64,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    Option<String>,
);

/// A receipt file name this tab made: `<plant id>_<time>.<ext>`, nothing else.
fn safe_receipt_name(n: &str) -> bool {
    let ok_ext = [".jpg", ".jpeg", ".png", ".webp", ".heic", ".pdf"]
        .iter()
        .any(|e| n.to_lowercase().ends_with(e));
    ok_ext
        && n.len() < 80
        && n.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        && !n.contains("..")
}

/// The receipts folder: inside the private data folder (never served as a web page).
fn receipts_dir(state: &AppState) -> std::path::PathBuf {
    state.data.join("receipts")
}

/// `?export=pa_receipt&id=<purchase id>`: the receipt file (signed-in people only, checked in mod.rs).
pub async fn receipt_file(state: &AppState, id: i64) -> Option<(Vec<u8>, &'static str, String)> {
    let r: Option<String> =
        sqlx::query_scalar("SELECT receipt_ref FROM plant_purchase WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .ok()?;
    let name = r.filter(|n| safe_receipt_name(n))?;
    let bytes = std::fs::read(receipts_dir(state).join(&name)).ok()?;
    let mime = match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "webp" => "image/webp",
        "heic" => "image/heic",
        _ => "image/jpeg",
    };
    Some((bytes, mime, name))
}

// ---- tab 17: Pictures --------------------------------------------------------------------------

/// Tab 17: add one or many photos to a plant. Each becomes one .webp (500 to 2000 px).
pub async fn render_pictures(state: &AppState, req: &Req) -> String {
    let l = req.lang;
    let mut h = ui::header(l.t("🖼️ Pictures", "🖼️ 사진"));
    let (pick, chosen) = picker(state, req).await;
    h.push_str(&pick);
    let Some(p) = chosen else { return h };
    let pics = pics_of(&s(&p, "picture"));
    if pics.is_empty() {
        h.push_str(&ui::info(l.t("No pictures yet.", "아직 사진이 없어요.")));
    } else {
        h.push_str(&format!(
            "<div class=\"pa-photos\">{}</div>",
            pics.iter().map(|n| photo(n)).collect::<String>()
        ));
        h.push_str(&ui::caption(l.t("The first picture is the main one. To remove or reorder pictures, use Manual Image Replace.", "첫 번째 사진이 대표 사진이에요. 사진을 지우거나 순서를 바꾸려면 'Manual Image Replace' 탭을 쓰세요.")));
    }
    h.push_str(&post_form("pa_pics", 17, &s(&p, "id"), true));
    h.push_str(&ui::file_uploader(
        "pics[]",
        l.t(
            "Add pictures (one or many)",
            "사진 추가 (한 장 또는 여러 장)",
        ),
        &["jpg", "jpeg", "png", "webp"],
        true,
    ));
    h.push_str(&ui::button(
        "go",
        "1",
        l.t("➕ Add pictures", "➕ 사진 추가"),
        true,
        false,
    ));
    h.push_str("</form>");
    h.push_str(&ui::caption(l.t(
        "Each picture is kept once, as a .webp of at most 2000 px (no small copies). Pictures under 500 px are not used.",
        "사진마다 최대 2000px의 .webp 파일 하나로 보관해요 (작은 사본은 만들지 않아요). 500px보다 작은 사진은 쓰지 않아요.",
    )));
    h
}

// ---- the buttons -------------------------------------------------------------------------------

/// The buttons of these tabs. Gives the messages and the address settings to go back to.
pub async fn action(
    state: &AppState,
    who: &Visitor,
    act: &str,
    f: &crate::formdata::FormData,
    l: Lang,
) -> (Vec<Flash>, Vec<(&'static str, String)>) {
    let pid = php_int(f.str("p"));
    // Back to the same plant (and inner tab, for Awaiting Approval).
    let sub = f.str("sub").to_string();
    let back = |p: i64| {
        let mut v = vec![("p", p.to_string())];
        if !sub.is_empty() {
            v.push(("sub", sub.clone()));
        }
        v
    };
    match act {
        "pa_add" => add(state, who, f, l).await,
        // Approve and Undo: the garden owner only. Checked before anything else is read.
        "pa_approve" | "pa_unapprove" => {
            if !plant_ai::is_owner(state, who) {
                return (
                    vec![Flash::Error(
                        l.t(
                            "Only the garden owner can approve plants.",
                            "식물 승인은 정원 주인만 할 수 있어요.",
                        )
                        .into(),
                    )],
                    back(pid),
                );
            }
            let on = i64::from(act == "pa_approve");
            match sqlx::query(plant_ai::APPROVE_SQL)
                .bind(on)
                .bind(pid)
                .execute(&state.db)
                .await
            {
                Ok(r) if r.rows_affected() > 0 || on == 0 => {
                    state.invalidate_widget_cache();
                    let m = if on == 1 {
                        l.t("✅ Approved.", "✅ 승인했어요.")
                    } else {
                        l.t("Approval undone.", "승인을 취소했어요.")
                    };
                    (vec![Flash::Success(m.into())], back(pid))
                }
                Ok(_) => (
                    vec![Flash::Error(
                        l.t("That plant was not found.", "그 식물을 찾지 못했어요.")
                            .into(),
                    )],
                    back(pid),
                ),
                Err(e) => (
                    vec![Flash::Error(format!("Database error: {e}"))],
                    back(pid),
                ),
            }
        }
        "pa_requeue" => match plant_ai::queue(state, pid, &who.username).await {
            Ok(_) => (
                vec![Flash::Success(
                    l.t("Queued for the AI.", "AI 대기열에 넣었어요.").into(),
                )],
                back(pid),
            ),
            Err(e) => (
                vec![Flash::Error(format!("Database error: {e}"))],
                back(pid),
            ),
        },
        "pa_use" => (use_reviewed(state, pid, f, l).await, back(pid)),
        "pa_key_off" => {
            let _ = sqlx::query("UPDATE plant_ai_keys SET revoked = 1 WHERE id = ?")
                .bind(php_int(f.str("key")))
                .execute(&state.db)
                .await;
            (
                vec![Flash::Success(
                    l.t("That helper key is switched off.", "그 도우미 키를 껐어요.")
                        .into(),
                )],
                vec![],
            )
        }
        "pa_receipt" => (save_receipt(state, who, pid, f, l).await, back(pid)),
        "pa_pics" => (save_pictures(state, pid, f, l).await, back(pid)),
        _ => (vec![Flash::Error("Unknown action.".into())], vec![]),
    }
}

/// Make a new helper key (the owner only). Gives the key to show once, or the refusal.
pub async fn make_key(
    state: &AppState,
    who: &Visitor,
    label: &str,
    l: Lang,
) -> Result<String, String> {
    if !plant_ai::is_owner(state, who) {
        return Err(l
            .t(
                "Only the garden owner can make helper keys.",
                "도우미 키는 정원 주인만 만들 수 있어요.",
            )
            .into());
    }
    let key = plant_ai::new_key();
    let label: String = label.trim().chars().take(100).collect();
    sqlx::query("INSERT INTO plant_ai_keys (label, key_sha256, created_by) VALUES (?, ?, ?)")
        .bind(if label.is_empty() {
            "Desktop PC".to_string()
        } else {
            label
        })
        .bind(plant_ai::sha256_hex(&key))
        .bind(&who.username)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;
    Ok(key)
}

/// "➕ Add plant": check the six boxes, save the plant, queue the AI.
async fn add(
    state: &AppState,
    who: &Visitor,
    f: &crate::formdata::FormData,
    l: Lang,
) -> (Vec<Flash>, Vec<(&'static str, String)>) {
    let genus = f.str("genus").trim().to_string();
    let species = f.str("species").trim().to_lowercase();
    let name = f.str("name").trim().to_string();
    let word = |w: &str| {
        !w.is_empty() && w.len() <= 60 && w.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
    };
    if name.is_empty() || !word(&genus) || !(species.is_empty() || word(&species)) {
        return (
            vec![Flash::Error(
                l.t(
                    "Type a name and a genus (letters only; the species too if you know it).",
                    "이름과 속을 입력하세요 (글자만; 종을 알면 종도).",
                )
                .into(),
            )],
            vec![],
        );
    }
    let genus = {
        let mut c = genus.chars();
        c.next()
            .map(|x| x.to_ascii_uppercase().to_string() + &c.as_str().to_lowercase())
            .unwrap_or_default()
    };
    let bed = f.str("bed").trim();
    let bed = if opts::GARDEN_BEDS.contains(&bed) {
        bed
    } else {
        "To Be Sorted"
    };
    let qty = php_int(f.str("qty")).clamp(0, 100_000);
    let price: Option<f64> = f
        .str("price")
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|p| p.is_finite() && *p >= 0.0 && *p < 1_000_000.0);
    let botanical = if species.is_empty() {
        genus.clone()
    } else {
        format!("{genus} {species}")
    };
    let year = common::current_year().to_string();
    let r = sqlx::query("INSERT INTO data_source (plant_name, genus, species, botanical_name, garden_beds, flower_bed, qty, average_cost, year_added) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&name)
        .bind(&genus)
        .bind(if species.is_empty() { None } else { Some(species.clone()) })
        .bind(&botanical)
        .bind(bed)
        .bind(bed)
        .bind(qty)
        .bind(price)
        .bind(&year)
        .execute(&state.db)
        .await;
    let id = match r {
        Ok(r) => r.last_insert_id() as i64,
        Err(e) => return (vec![Flash::Error(format!("Database error: {e}"))], vec![]),
    };
    state.invalidate_widget_cache();
    let mut msgs = vec![Flash::Success(format!(
        "✅ **{name}** {}",
        l.t("is saved.", "저장했어요.")
    ))];
    if f.has("ai") {
        match plant_ai::queue(state, id, &who.username).await {
            Ok(_) => msgs.push(Flash::Info(
                l.t(
                    "The AI will fill in the rest; this page shows how far it is.",
                    "나머지는 AI가 채워요. 진행 상황은 이 페이지에 나와요.",
                )
                .into(),
            )),
            Err(e) => msgs.push(Flash::Warning(format!("AI queue: {e}"))),
        }
    }
    (msgs, vec![("p", id.to_string())])
}

/// "Use the ticked values": the reviewed suggestions the person ticked, into EMPTY fields only,
/// after a backup.
async fn use_reviewed(
    state: &AppState,
    pid: i64,
    f: &crate::formdata::FormData,
    l: Lang,
) -> Vec<Flash> {
    let Some(job) = plant_ai::latest_job(state, pid).await else {
        return vec![];
    };
    let Some(row) = plant_ai::plant_row(state, pid).await else {
        return vec![];
    };
    let nr = job
        .result
        .as_ref()
        .and_then(|r| r.get("needs_review"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut picked = Map::new();
    for (k, x) in &nr {
        if f.has(&format!("use_{k}")) {
            picked.insert(k.clone(), x.get("v").cloned().unwrap_or(Value::Null));
        }
    }
    if picked.is_empty() {
        return vec![Flash::Info(
            l.t("Nothing was ticked.", "체크한 것이 없어요.").into(),
        )];
    }
    let cols = plant_ai::columns(state).await;
    let (write, skipped) = plant_ai::plan_writes(&row, &picked, &cols);
    if write.is_empty() {
        return vec![Flash::Warning(
            l.t(
                "Nothing to write: those fields are already filled.",
                "쓸 것이 없어요: 그 칸들은 이미 채워져 있어요.",
            )
            .into(),
        )];
    }
    if let Err(e) = plant_ai::backup(state, pid, Some(job.id), "reviewed values").await {
        return vec![Flash::Error(e)];
    }
    match plant_ai::write_fields(state, pid, &write).await {
        Ok(n) => {
            let mut m = vec![Flash::Success(format!(
                "✅ {n} {}",
                l.t("value(s) saved.", "개 값을 저장했어요.")
            ))];
            if !skipped.is_empty() {
                m.push(Flash::Info(format!(
                    "{} {}",
                    l.t("Kept as they were:", "그대로 둔 칸:"),
                    skipped.keys().cloned().collect::<Vec<_>>().join(", ")
                )));
            }
            m
        }
        Err(e) => vec![Flash::Error(e)],
    }
}

/// "💾 Save receipt": the file into the private receipts folder, one purchase line, and the price
/// on the plant if it had none.
async fn save_receipt(
    state: &AppState,
    who: &Visitor,
    pid: i64,
    f: &crate::formdata::FormData,
    l: Lang,
) -> Vec<Flash> {
    if plant_ai::plant_row(state, pid).await.is_none() {
        return vec![Flash::Error(
            l.t("That plant was not found.", "그 식물을 찾지 못했어요.")
                .into(),
        )];
    }
    let mut file_ref = String::new();
    if let Some(u) = f
        .files_for("receipt")
        .into_iter()
        .find(|u| !u.bytes.is_empty())
    {
        let ext = u.filename.rsplit('.').next().unwrap_or("").to_lowercase();
        if !["jpg", "jpeg", "png", "webp", "heic", "pdf"].contains(&ext.as_str()) {
            return vec![Flash::Error(
                l.t(
                    "A receipt must be a photo (JPG, PNG, WebP, HEIC) or a PDF.",
                    "영수증은 사진(JPG, PNG, WebP, HEIC) 또는 PDF여야 해요.",
                )
                .into(),
            )];
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let name = format!("{pid}_{stamp}.{ext}");
        let dir = receipts_dir(state);
        if std::fs::create_dir_all(&dir)
            .and_then(|_| std::fs::write(dir.join(&name), &u.bytes))
            .is_err()
        {
            return vec![Flash::Error(
                l.t(
                    "Could not save the receipt file.",
                    "영수증 파일을 저장하지 못했어요.",
                )
                .into(),
            )];
        }
        file_ref = name;
    }
    let qty = php_int(f.str("qty")).clamp(0, 100_000);
    let price: Option<f64> = f
        .str("price")
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|p| p.is_finite() && *p >= 0.0);
    let date = f.str("date").trim();
    let date = (date.len() == 10 && chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok())
        .then(|| date.to_string());
    let store: String = f.str("store").trim().chars().take(200).collect();
    let line = format!("{} (Receipt tab)", l.t("Receipt", "영수증"));
    let r = sqlx::query("INSERT INTO plant_purchase (plant_id, line_text, quantity, unit_price, line_total, store, bought_on, receipt_ref, added_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(pid)
        .bind(&line)
        .bind(qty.max(1))
        .bind(price)
        .bind(price.map(|p| p * qty.max(1) as f64))
        .bind(&store)
        .bind(date)
        .bind(&file_ref)
        .bind(&who.username)
        .execute(&state.db)
        .await;
    if let Err(e) = r {
        return vec![Flash::Error(format!("Database error: {e}"))];
    }
    // The plant's price, only when it has none yet.
    if let Some(p) = price {
        let _ = sqlx::query("UPDATE data_source SET average_cost = ? WHERE id = ? AND (average_cost IS NULL OR average_cost = 0)")
            .bind(p)
            .bind(pid)
            .execute(&state.db)
            .await;
    }
    vec![Flash::Success(
        l.t("🧾 Receipt saved.", "🧾 영수증을 저장했어요.").into(),
    )]
}

/// "➕ Add pictures": each upload becomes one .webp, added after the plant's pictures.
async fn save_pictures(
    state: &AppState,
    pid: i64,
    f: &crate::formdata::FormData,
    l: Lang,
) -> Vec<Flash> {
    let Some(row) = plant_ai::plant_row(state, pid).await else {
        return vec![Flash::Error(
            l.t("That plant was not found.", "그 식물을 찾지 못했어요.")
                .into(),
        )];
    };
    let ups = f.files_for("pics[]");
    if ups.is_empty() {
        return vec![Flash::Info(
            l.t(
                "Choose one or more pictures first.",
                "먼저 사진을 한 장 이상 고르세요.",
            )
            .into(),
        )];
    }
    let base = format!(
        "{}_{}_pic_{}",
        super::form::file_safe(&s(&row, "plant_name")),
        pid,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let dir = state.www.join("images/Database");
    let (mut saved, mut skipped) = (Vec::new(), Vec::new());
    for (i, u) in ups.iter().enumerate() {
        match plant_ai::fit_photo(&u.bytes) {
            Ok(webp) => {
                let name = format!("{base}_{}.webp", i + 1);
                if std::fs::write(dir.join(&name), webp).is_ok() {
                    saved.push(name);
                } else {
                    skipped.push(format!("{}: could not save", u.filename));
                }
            }
            Err(e) => skipped.push(format!("{}: {e}", u.filename)),
        }
    }
    let mut msgs = Vec::new();
    if !saved.is_empty() {
        let mut all = pics_of(&s(&row, "picture"));
        all.extend(saved.iter().cloned());
        if let Err(e) = sqlx::query("UPDATE data_source SET picture = ? WHERE id = ?")
            .bind(all.join(", "))
            .bind(pid)
            .execute(&state.db)
            .await
        {
            return vec![Flash::Error(format!("Database error: {e}"))];
        }
        state.invalidate_widget_cache();
        msgs.push(Flash::Success(format!(
            "✅ {} {}",
            saved.len(),
            l.t("picture(s) added.", "장의 사진을 추가했어요.")
        )));
    }
    if !skipped.is_empty() {
        msgs.push(Flash::Warning(format!(
            "{} {}",
            l.t("Not used:", "쓰지 않은 사진:"),
            skipped.join("; ")
        )));
    }
    msgs
}

// ---- "Export what's still missing" -----------------------------------------------------------

/// The ready prompt for the owner's own Claude (or any assistant), in both languages.
pub const PROMPT_EN: &str = "You are helping fill in a garden plant database. The file below lists plants and, for each, ONLY the fields that are still empty, with the allowed values and the expected format. Rules: 1) Answer with ONLY a JSON array (a top-level [ ... ]), one object per plant, with its \"id\" and \"plant_name\" exactly as given. 2) Include only the empty fields listed for that plant; never add or change any other field. 3) For a field with \"allowed\" values, use one of them exactly, letter for letter (for lists, several of them joined with \", \"). 4) Follow each field's format and example (months as the 12-letter code like \"....MJJAS...\", sizes like \"24-36 in\", zones like \"4-8\"). 5) If you are not sure of a value, leave it out (or null): an empty field is better than a wrong one. 6) No comments, no text before or after the JSON.";
pub const PROMPT_KO: &str = "정원 식물 데이터베이스를 채우는 일을 도와주세요. 아래 파일에는 식물마다 아직 비어 있는 칸만, 허용되는 값과 형식과 함께 들어 있어요. 규칙: 1) 답은 JSON 배열(맨 바깥이 [ ... ])만 쓰고, 식물마다 객체 하나, \"id\"와 \"plant_name\"은 주어진 그대로 쓰세요. 2) 그 식물에 적힌 빈 칸만 넣고, 다른 칸은 절대 더하거나 바꾸지 마세요. 3) \"allowed\" 값이 있는 칸은 그중 하나를 글자 그대로 쓰세요 (여러 개면 \", \"로 이어서). 4) 각 칸의 형식과 예를 따르세요 (달은 \"....MJJAS...\" 같은 12글자 코드, 크기는 \"24-36 in\", 존은 \"4-8\"). 5) 확실하지 않으면 빼거나 null로 두세요: 틀린 값보다 빈 칸이 나아요. 6) JSON 앞뒤에 설명이나 주석을 쓰지 마세요.";

/// The export as a JSON value (kept separate so it can be tested without a database).
pub fn missing_doc(row: &Map<String, Value>) -> Value {
    let missing: Vec<&str> = opts::missing_fields(row)
        .into_iter()
        .filter(|f| !EXPORT_SKIP.contains(f))
        .collect();
    let mut fields = Map::new();
    let mut template = Map::new();
    template.insert("id".into(), row.get("id").cloned().unwrap_or(Value::Null));
    template.insert(
        "plant_name".into(),
        row.get("plant_name").cloned().unwrap_or(Value::Null),
    );
    for f in &missing {
        let (ty, allowed, example) =
            super::pipeline_ref::field_def(f).unwrap_or(("string", None, ""));
        let mut d = Map::new();
        d.insert("type".into(), json!(ty));
        if let Some(a) = allowed {
            let a: Vec<&str> = a
                .iter()
                .copied()
                .filter(|x| !x.is_empty() && *x != "Unknown")
                .collect();
            d.insert("allowed".into(), json!(a));
        }
        if !example.is_empty() {
            d.insert("example".into(), json!(example));
        }
        fields.insert((*f).to_string(), Value::Object(d));
        template.insert((*f).to_string(), Value::Null);
    }
    json!({
        "prompt_en": PROMPT_EN,
        "prompt_ko": PROMPT_KO,
        "how_to_use_en": "Give this whole file to your assistant with the prompt above. Save its answer (the JSON array only) as a .json file, then in the plant admin open Advanced -> JSON Pipeline -> Import & Review. You see every value before it is saved; filled fields are never overwritten.",
        "how_to_use_ko": "이 파일 전체를 위의 프롬프트와 함께 도우미에게 주세요. 답(JSON 배열만)을 .json 파일로 저장한 뒤, 식물 관리 화면의 고급 -> JSON Pipeline -> Import & Review에서 가져오세요. 저장하기 전에 모든 값을 보여 주고, 이미 채워진 칸은 절대 덮어쓰지 않아요.",
        "plants": [{
            "id": row.get("id").cloned().unwrap_or(Value::Null),
            "plant_name": row.get("plant_name").cloned().unwrap_or(Value::Null),
            "botanical_name": row.get("botanical_name").cloned().unwrap_or(Value::Null),
            "genus": row.get("genus").cloned().unwrap_or(Value::Null),
            "species": row.get("species").cloned().unwrap_or(Value::Null),
            "empty_fields": Value::Object(fields),
        }],
        "answer_template": [Value::Object(template)],
    })
}

/// `?export=pa_missing&id=`: (file text, file name).
pub async fn missing_export(state: &AppState, id: i64) -> Option<(String, String)> {
    let row = plant_ai::plant_row(state, id).await?;
    let name = super::export::download_name(&s(&row, "plant_name"), "_still_missing.json");
    Some((serde_json::to_string_pretty(&missing_doc(&row)).ok()?, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_lists_only_empty_fields_with_allowed_values() {
        let mut row = Map::new();
        row.insert("id".into(), json!(7));
        row.insert("plant_name".into(), json!("Sample Sage"));
        row.insert("zones".into(), json!("4-8"));
        row.insert("sun_exposure".into(), Value::Null);
        let d = missing_doc(&row);
        let empty = d
            .pointer("/plants/0/empty_fields")
            .and_then(Value::as_object)
            .unwrap();
        assert!(empty.contains_key("sun_exposure"));
        assert!(
            !empty.contains_key("zones"),
            "a filled field is never asked for"
        );
        assert!(!empty.contains_key("picture") && !empty.contains_key("average_cost"));
        let allowed = empty["sun_exposure"]["allowed"].as_array().unwrap();
        assert!(allowed.iter().any(|v| v == "Full Sun") && !allowed.iter().any(|v| v == ""));
        // The answer is a top-level array, and the prompts say so in both languages.
        assert!(d["answer_template"].is_array());
        assert!(PROMPT_EN.contains("top-level") && PROMPT_KO.contains("JSON 배열"));
        assert!(!d.to_string().contains("approved_jason"));
    }

    /// A site state with two accounts (the first is the owner) and a database that is never
    /// reached (connections to port 9 fail at once).
    fn state() -> AppState {
        let tmp = std::env::temp_dir().join("jg-quick-test"); // nosemgrep: rust.lang.security.temp-dir.temp-dir (test only: a path that is never created)
        AppState {
            db: sqlx::mysql::MySqlPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_secs(3))
                .connect_lazy("mysql://test@127.0.0.1:9/test")
                .unwrap(),
            www: tmp.clone(),
            data: tmp.clone(),
            comments_lock: tokio::sync::Mutex::new(()),
            users: crate::accounts::parse_users(
                "Gardener=gardener@example.test,Helper=helper@example.test",
            ),
            uploaders: Vec::new(),
            public_url: "http://localhost".into(),
            ingest_url: "http://127.0.0.1:9".into(),
            widget_cache: std::sync::Mutex::new(None),
            http: reqwest::Client::new(),
            ollama_url: "http://127.0.0.1:9".into(),
            ollama_model: "none".into(),
            ai_cache_dir: tmp,
            forecast_file: std::path::PathBuf::new(),
            weather_latest: std::path::PathBuf::new(),
            guides: std::path::PathBuf::new(),
            frame: Default::default(),
            write_limit: crate::guard::RateLimit::new(5, std::time::Duration::from_secs(600)),
            gallery: Default::default(),
            ai_recompute: crate::guard::RateLimit::new(1, std::time::Duration::from_secs(600)),
            ai_per_visitor: crate::guard::RateLimit::new(20, std::time::Duration::from_secs(600)),
            address: crate::address::SiteAddress::new("http://localhost", ""),
        }
    }

    fn who(name: &str) -> Visitor {
        Visitor {
            sid: "test".into(),
            username: name.into(),
            ..Default::default()
        }
    }

    fn form(pairs: &[(&str, &str)]) -> crate::formdata::FormData {
        let mut f = crate::formdata::FormData::default();
        for (k, v) in pairs {
            f.fields.insert((*k).into(), (*v).into());
        }
        f
    }

    #[tokio::test]
    async fn a_non_owner_cannot_approve() {
        let st = state();
        let f = form(&[("act", "pa_approve"), ("tab", "15"), ("p", "1")]);
        for name in ["Helper", ""] {
            let (msgs, _) = action(&st, &who(name), "pa_approve", &f, Lang::En).await;
            assert!(
                matches!(msgs.as_slice(), [Flash::Error(m)] if m.contains("Only the garden owner")),
                "{name:?} must be refused before anything is written"
            );
        }
        // The owner gets past the check (here the test database is unreachable, so the next
        // thing that happens is a database error, not a refusal).
        let (msgs, _) = action(&st, &who("Gardener"), "pa_approve", &f, Lang::En).await;
        assert!(matches!(msgs.as_slice(), [Flash::Error(m)] if m.starts_with("Database error")));
        // Making a helper key is the owner's too.
        assert!(make_key(&st, &who("Helper"), "PC", Lang::En).await.is_err());
    }

    #[test]
    fn receipt_names_are_only_our_own() {
        assert!(safe_receipt_name("12_1790000000.pdf"));
        assert!(!safe_receipt_name("../.env"));
        assert!(!safe_receipt_name("12_1.exe"));
    }
}
