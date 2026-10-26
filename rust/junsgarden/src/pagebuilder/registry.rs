//! The section registry: every block the page builder can place, with its names (English and
//! Korean), version, the permission it needs and the settings it takes.
//!
//! HOW TO ADD A SECTION. Add one `SectionDef` to `SECTIONS`, add its renderer to
//! `sections.rs` (`render` and `IMPLEMENTED`), and add it to
//! `docs/page-builder/sections-manifest.json`. The tests at the bottom fail if an entry has no
//! renderer or a renderer has no entry. Design: `docs/page-builder/DESIGN.md`.

use serde::Serialize;

/// Who may see a section.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Perm {
    /// Everybody.
    Public,
    /// Only people signed in (GARDEN_USERS). Checked every time it is drawn, on any page.
    Admin,
}

/// The kind and limits of one setting.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Kind {
    /// A whole number between `min` and `max`.
    Int { min: i64, max: i64, default: i64 },
    /// Text of at most `max_len` characters, each one allowed by `charset`, or empty.
    Text {
        max_len: usize,
        charset: Charset,
        default: &'static str,
    },
}

/// Which characters a text setting may hold. Settings never carry markup, so none of these
/// allows `<`, `>`, quotes or `&`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Charset {
    /// 0-9 only (a zip code).
    Digits,
    /// Letters (any language), digits, space and `-_.,!?()`. For the pending sections'
    /// text settings (DESIGN.md 3.2); no live section uses it yet.
    #[allow(dead_code)]
    Plain,
}

impl Charset {
    pub fn allows(self, c: char) -> bool {
        match self {
            Charset::Digits => c.is_ascii_digit(),
            Charset::Plain => c.is_alphanumeric() || " -_.,!?()".contains(c),
        }
    }
}

/// One setting of a section.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Setting {
    pub key: &'static str,
    pub label_en: &'static str,
    pub label_ko: &'static str,
    #[serde(flatten)]
    pub kind: Kind,
}

/// One section the builder can place.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SectionDef {
    pub id: &'static str,
    pub name_en: &'static str,
    pub name_ko: &'static str,
    pub desc_en: &'static str,
    pub desc_ko: &'static str,
    /// Raised when the section's look or settings change.
    pub version: u32,
    /// The garden release that first had it (shown in the editor).
    pub since: &'static str,
    pub perm: Perm,
    /// How many copies one page may hold (1 when its script looks for a fixed element id).
    pub max_per_page: usize,
    pub settings: &'static [Setting],
}

/// Every section in this version of the garden.
pub const SECTIONS: &[SectionDef] = &[
    SectionDef {
        id: "home.welcome",
        name_en: "Log-in box",
        name_ko: "로그인 상자",
        desc_en: "The log-in form for visitors; a welcome and Logout button once signed in.",
        desc_ko: "방문자에게는 로그인 양식, 로그인한 사람에게는 환영 인사와 로그아웃 버튼.",
        version: 1,
        since: "2026.11",
        perm: Perm::Public,
        max_per_page: 1,
        settings: &[],
    },
    SectionDef {
        id: "home.weather",
        name_en: "Garden weather outlook",
        name_ko: "정원 날씨 예보",
        desc_en: "The 48-hour daytime outlook for a zip code.",
        desc_ko: "우편번호 지역의 48시간 낮 날씨 예보.",
        version: 1,
        since: "2026.11",
        perm: Perm::Public,
        max_per_page: 1,
        settings: &[Setting {
            key: "zip",
            label_en: "Zip code shown first (5 digits, or empty)",
            label_ko: "처음 보여 줄 우편번호 (숫자 5개, 또는 비움)",
            kind: Kind::Text {
                max_len: 5,
                charset: Charset::Digits,
                default: "",
            },
        }],
    },
    SectionDef {
        id: "home.tips",
        name_en: "Garden & collie tips",
        name_ko: "정원·콜리 팁",
        desc_en: "The tip cards, shuffled on every visit.",
        desc_ko: "방문할 때마다 섞이는 팁 카드.",
        version: 1,
        since: "2026.11",
        perm: Perm::Public,
        max_per_page: 1,
        settings: &[],
    },
    SectionDef {
        id: "home.news",
        name_en: "Garden news",
        name_ko: "정원 소식",
        desc_en: "The newest garden posts. Signed-in people also get Edit and Delete.",
        desc_ko: "최신 정원 글. 로그인한 사람에게는 수정·삭제 버튼도 보여요.",
        version: 1,
        since: "2026.11",
        perm: Perm::Public,
        max_per_page: 1,
        settings: &[Setting {
            key: "count",
            label_en: "How many posts",
            label_ko: "글 개수",
            kind: Kind::Int {
                min: 1,
                max: 12,
                default: 3,
            },
        }],
    },
    SectionDef {
        id: "home.guestbook",
        name_en: "Guestbook",
        name_ko: "방명록",
        desc_en: "Visitors leave a short note; signed-in people can delete notes.",
        desc_ko: "방문자가 짧은 글을 남겨요. 로그인한 사람은 글을 지울 수 있어요.",
        version: 1,
        since: "2026.11",
        perm: Perm::Public,
        max_per_page: 1,
        settings: &[],
    },
    SectionDef {
        id: "admin.post_editor",
        name_en: "New-post button",
        name_ko: "새 글 쓰기 버튼",
        desc_en: "The Create New Post button and form. Only signed-in people see it.",
        desc_ko: "새 글 쓰기 버튼과 양식. 로그인한 사람에게만 보여요.",
        version: 1,
        since: "2026.11",
        perm: Perm::Admin,
        max_per_page: 1,
        settings: &[],
    },
    SectionDef {
        id: "blog.archive",
        name_en: "Garden archive",
        name_ko: "정원 기록 보관소",
        desc_en: "Every garden post, newest first.",
        desc_ko: "모든 정원 글, 최신 글부터.",
        version: 1,
        since: "2026.11",
        perm: Perm::Public,
        max_per_page: 1,
        settings: &[],
    },
    SectionDef {
        id: "gallery.wall",
        name_en: "Photo gallery",
        name_ko: "사진 갤러리",
        desc_en: "The photo wall with tags, search and the slideshow.",
        desc_ko: "태그·검색·슬라이드쇼가 있는 사진 벽.",
        version: 1,
        since: "2026.11",
        perm: Perm::Public,
        max_per_page: 1,
        settings: &[],
    },
];

/// Planned sections that wait for the database rework (DESIGN.md 3.2). Not placeable yet; a
/// layout naming one is refused with a clear message.
pub const PENDING: &[&str] = &[
    "catalog.browse",
    "catalog.cards",
    "search.quick",
    "data.widgets",
    "data.compare",
    "data.table",
    "admin.plant_admin",
    "beds.planner",
];

/// The section with this id.
pub fn find(id: &str) -> Option<&'static SectionDef> {
    SECTIONS.iter().find(|s| s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_every_entry_has_a_renderer() {
        for (i, s) in SECTIONS.iter().enumerate() {
            assert!(
                SECTIONS[i + 1..].iter().all(|o| o.id != s.id),
                "{} twice",
                s.id
            );
            assert!(
                super::super::sections::IMPLEMENTED.contains(&s.id),
                "{} has no renderer",
                s.id
            );
            assert!(!PENDING.contains(&s.id), "{} is also pending", s.id);
        }
        for id in super::super::sections::IMPLEMENTED {
            assert!(find(id).is_some(), "renderer {id} has no registry entry");
        }
    }

    #[test]
    fn every_name_and_description_is_in_english_and_korean() {
        let hangul = |s: &str| s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c));
        for s in SECTIONS {
            assert!(!s.name_en.is_empty() && !s.desc_en.is_empty(), "{}", s.id);
            assert!(hangul(s.name_ko) && hangul(s.desc_ko), "{} Korean", s.id);
            for st in s.settings {
                assert!(
                    !st.label_en.is_empty() && hangul(st.label_ko),
                    "{}.{}",
                    s.id,
                    st.key
                );
            }
            assert!(s.max_per_page >= 1);
        }
    }

    #[test]
    fn text_settings_never_allow_markup() {
        for c in ['<', '>', '"', '\'', '&', '/', '\\', '`', '='] {
            assert!(!Charset::Plain.allows(c), "{c}");
            assert!(!Charset::Digits.allows(c), "{c}");
        }
    }
}
