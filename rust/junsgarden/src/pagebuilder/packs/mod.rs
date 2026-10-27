//! Feature packs: new looks and parts for the site, delivered as DATA only.
//!
//! A pack is a few small text files (SVG art, CSS, text for sections, colours) listed in a
//! signed index. There is no downloaded program in it: not a script, not Rust, not a template
//! language. Everything is read by the strict readers in this folder, which keep only what a
//! small allowlist permits and write the result out again themselves.
//!
//! HOW TO READ THIS FOLDER.
//! * this file    the index/pack types, their checks, and the trust chain (signature, size, sha256).
//! * `svg.rs`     header and footer art: an allowlist SVG reader/writer.
//! * `css.rs`     styles: an allowlist CSS reader/writer, every rule fenced to the pack's own part.
//! * `blocks.rs`  section templates: a list of plain text blocks, drawn with escaping.
//! * `source.rs`  where packs come from (HTTPS only), and the wizard's downloaded bundle.
//! * `store.rs`   packs in the database: staged, installed, kept previous version, verify.
//! * `../site.rs` which packs are on, what is chosen from them, and the resolved result.
//!
//! THE TRUST CHAIN. The author signs the index (Ed25519). The index lists each pack with each
//! file's sha256 and size. A file is accepted only if its sha256 and size match the index and
//! the index signature matches the author's key. Without a key the pack is "unsigned": it can
//! be looked at, and installed only after the owner ticks a loud warning.

pub mod blocks;
pub mod css;
pub mod source;
pub mod store;
pub mod svg;

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The signed text is this line followed by the index's exact bytes.
pub const MSG_PREFIX: &[u8] = b"junsgarden-packs-v1\n";
/// The channel every index of this garden must name (a different channel is refused, so a test
/// index cannot be mistaken for the real one).
pub const CHANNEL: &str = "garden-for-all";
pub const MAX_INDEX_BYTES: usize = 256 * 1024;
pub const MAX_FILE_BYTES: usize = 64 * 1024;
pub const MAX_FILES: usize = 16;
pub const MAX_PACK_BYTES: usize = 256 * 1024;
pub const MAX_PACKS: usize = 40;

/// Why something was refused, in both languages. Never carries text copied from a pack
/// beyond short checked names.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Problem {
    pub en: String,
    pub ko: String,
}

pub fn problem(en: impl Into<String>, ko: impl Into<String>) -> Problem {
    Problem {
        en: en.into(),
        ko: ko.into(),
    }
}

/// The author's public key for packs (64 hex characters). Empty until the author makes the
/// key (docs/page-builder/sign-packs.sh); `JUNSGARDEN_PACKS_KEY` overrides it. With no key,
/// every pack is "unsigned".
pub const AUTHOR_PACKS_KEY: &str = "";

/// The public key to check against: the setting, else the pinned one, else none.
pub fn key_from(setting: &str) -> Option<VerifyingKey> {
    let hex = if setting.trim().is_empty() {
        AUTHOR_PACKS_KEY
    } else {
        setting.trim()
    };
    VerifyingKey::from_bytes(&crate::address::hex32(hex)?).ok()
}

/// Does `sig_hex` (128 hex characters) sign this index with `key`?
pub fn verify_signature(index: &[u8], sig_hex: &str, key: &VerifyingKey) -> bool {
    let sig_hex = sig_hex.trim();
    if sig_hex.len() != 128 || !sig_hex.is_ascii() {
        return false;
    }
    let mut sig = [0u8; 64];
    for (i, chunk) in sig_hex.as_bytes().chunks(2).enumerate() {
        match std::str::from_utf8(chunk)
            .ok()
            .and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            Some(b) => sig[i] = b,
            None => return false,
        }
    }
    let mut msg = MSG_PREFIX.to_vec();
    msg.extend_from_slice(index);
    key.verify_strict(&msg, &Signature::from_bytes(&sig))
        .is_ok()
}

pub fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ---------------------------------------------------------------- the index

/// The signed list of packs.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    pub schema: u32,
    /// Must be `CHANNEL`.
    pub channel: String,
    /// Goes up with every release; an index with a lower number than one already accepted is
    /// refused (an old signed list cannot be replayed).
    pub serial: u64,
    pub released: String,
    pub packs: Vec<Pack>,
}

/// One pack: its names, what it provides and every file with its fingerprint.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub id: String,
    pub version: String,
    pub name_en: String,
    pub name_ko: String,
    pub desc_en: String,
    pub desc_ko: String,
    pub provides: Vec<Provide>,
    pub files: Vec<FileEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: usize,
}

/// What a pack gives. Each kind has its own small set of fields.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Provide {
    /// Header and footer art. `builtin` = the site's own waves (no files). Otherwise `top`
    /// (under the header bar), `bottom` (above the footer) and optional `hero` (the photo
    /// banner's edge) are SVG files, and `css` is optional.
    Header {
        id: String,
        name_en: String,
        name_ko: String,
        #[serde(default)]
        builtin: bool,
        #[serde(default)]
        top: Option<String>,
        #[serde(default)]
        bottom: Option<String>,
        #[serde(default)]
        hero: Option<String>,
        #[serde(default)]
        css: Option<String>,
    },
    /// A way the catalog's plant cards are laid out (CSS fenced to the cards section).
    CatalogLayout {
        id: String,
        name_en: String,
        name_ko: String,
        css: String,
    },
    /// Site colours: custom properties from a short allowed list.
    Theme {
        id: String,
        name_en: String,
        name_ko: String,
        vars: std::collections::BTreeMap<String, String>,
    },
    /// A section the owner can place on any page (a file of text blocks).
    Section {
        id: String,
        name_en: String,
        name_ko: String,
        desc_en: String,
        desc_ko: String,
        template: String,
    },
}

impl Provide {
    pub fn id(&self) -> &str {
        match self {
            Provide::Header { id, .. }
            | Provide::CatalogLayout { id, .. }
            | Provide::Theme { id, .. }
            | Provide::Section { id, .. } => id,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Provide::Header { .. } => "header",
            Provide::CatalogLayout { .. } => "catalog_layout",
            Provide::Theme { .. } => "theme",
            Provide::Section { .. } => "section",
        }
    }

    /// The names (English, Korean) of this part.
    pub fn names(&self) -> (&str, &str) {
        match self {
            Provide::Header {
                name_en, name_ko, ..
            }
            | Provide::CatalogLayout {
                name_en, name_ko, ..
            }
            | Provide::Theme {
                name_en, name_ko, ..
            }
            | Provide::Section {
                name_en, name_ko, ..
            } => (name_en, name_ko),
        }
    }

    /// The files this part uses.
    pub fn files(&self) -> Vec<&str> {
        match self {
            Provide::Header {
                top,
                bottom,
                hero,
                css,
                ..
            } => [top, bottom, hero, css]
                .into_iter()
                .flatten()
                .map(String::as_str)
                .collect(),
            Provide::CatalogLayout { css, .. } => vec![css],
            Provide::Theme { .. } => Vec::new(),
            Provide::Section { template, .. } => vec![template],
        }
    }
}

/// `a-z 0-9 -`, 1 to 40, starting with a letter or digit.
pub fn id_ok(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 40
        && !s.starts_with('-')
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// `1.2.3` (three whole numbers).
pub fn version_ok(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    s.len() <= 20
        && parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 6 && p.bytes().all(|b| b.is_ascii_digit()))
}

/// A version as numbers, to compare.
pub fn version_key(s: &str) -> (u32, u32, u32) {
    let mut it = s.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
    )
}

/// A file name inside a pack: no folders, a known extension.
pub fn file_name_ok(s: &str) -> bool {
    let Some((stem, ext)) = s.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty()
        && s.len() <= 40
        && matches!(ext, "svg" | "css" | "json")
        && s.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_' || b == b'.'
        })
        && !s.starts_with('.')
        && !s.contains("..")
}

/// A short, single-line, plain text (names and descriptions in the index).
fn text_ok(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.chars().count() <= max
        && !s.chars().any(|c| c.is_control() || "<>\"&`".contains(c))
}

/// Read the index strictly (after its signature, if any, was checked).
pub fn parse_index(bytes: &[u8]) -> Result<Index, Problem> {
    if bytes.len() > MAX_INDEX_BYTES {
        return Err(problem(
            "The list of features is too big.",
            "기능 목록이 너무 커요.",
        ));
    }
    let bad = || {
        problem(
            "The list of features is not in the expected form.",
            "기능 목록의 형식이 맞지 않아요.",
        )
    };
    let ix: Index = serde_json::from_slice(bytes).map_err(|_| bad())?;
    if ix.schema != 1 || ix.packs.len() > MAX_PACKS || !text_ok(&ix.released, 40) {
        return Err(bad());
    }
    if ix.channel != CHANNEL {
        return Err(problem(
            "That list is for something else (wrong channel).",
            "다른 용도의 목록이에요 (채널이 맞지 않아요).",
        ));
    }
    for (i, p) in ix.packs.iter().enumerate() {
        check_pack(p).map_err(|why| {
            problem(
                format!("Feature #{}: {}", i + 1, why.en),
                format!("기능 #{}: {}", i + 1, why.ko),
            )
        })?;
        if ix.packs[..i].iter().any(|o| o.id == p.id) {
            return Err(problem(
                "Two features have the same name in the list.",
                "목록에 같은 이름의 기능이 둘 있어요.",
            ));
        }
    }
    Ok(ix)
}

/// One pack's own consistency: names, parts, files.
pub fn check_pack(p: &Pack) -> Result<(), Problem> {
    let no = |en: &str, ko: &str| Err(problem(en, ko));
    if !id_ok(&p.id) || !version_ok(&p.version) {
        return no("bad name or version", "이름이나 버전이 맞지 않아요");
    }
    for t in [&p.name_en, &p.name_ko] {
        if !text_ok(t, 60) {
            return no("bad name text", "이름 글이 맞지 않아요");
        }
    }
    for t in [&p.desc_en, &p.desc_ko] {
        if !text_ok(t, 300) {
            return no("bad description text", "설명 글이 맞지 않아요");
        }
    }
    if p.provides.is_empty() || p.provides.len() > 8 {
        return no(
            "it provides nothing or too much",
            "내용이 없거나 너무 많아요",
        );
    }
    if p.files.len() > MAX_FILES {
        return no("too many files", "파일이 너무 많아요");
    }
    let mut total = 0;
    for (i, f) in p.files.iter().enumerate() {
        let hex_ok = f.sha256.len() == 64
            && f.sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !file_name_ok(&f.path) || !hex_ok || f.bytes == 0 || f.bytes > MAX_FILE_BYTES {
            return no("a file entry is not allowed", "파일 항목이 허용되지 않아요");
        }
        if p.files[..i].iter().any(|o| o.path == f.path) {
            return no("a file is listed twice", "파일이 두 번 적혀 있어요");
        }
        total += f.bytes;
    }
    if total > MAX_PACK_BYTES {
        return no(
            "the files are too big together",
            "파일이 모두 합쳐 너무 커요",
        );
    }
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for pr in &p.provides {
        let id = pr.id();
        if !id_ok(id) || seen.contains(&(pr.kind(), id)) {
            return no(
                "a part has a bad or repeated name",
                "부분의 이름이 맞지 않아요",
            );
        }
        seen.push((pr.kind(), id));
        let (en, ko) = pr.names();
        if !text_ok(en, 60) || !text_ok(ko, 60) {
            return no("a part has bad name text", "부분의 이름 글이 맞지 않아요");
        }
        for f in pr.files() {
            if !p.files.iter().any(|e| e.path == f) {
                return no(
                    "a part uses a file that is not listed",
                    "부분이 목록에 없는 파일을 써요",
                );
            }
        }
        match pr {
            Provide::Header {
                builtin,
                top,
                bottom,
                hero,
                css,
                ..
            } => {
                let has_art = top.is_some() || bottom.is_some() || hero.is_some() || css.is_some();
                if *builtin == has_art || (!builtin && top.is_none() && bottom.is_none()) {
                    return no(
                        "a header is either the built-in waves or has art files",
                        "머리글은 기본 물결이거나 그림 파일이 있어야 해요",
                    );
                }
            }
            Provide::Section {
                desc_en, desc_ko, ..
            } if !text_ok(desc_en, 300) || !text_ok(desc_ko, 300) => {
                return no("a section has bad text", "섹션의 글이 맞지 않아요");
            }
            _ => {}
        }
    }
    // Every listed file must be used by some part (no hidden extra files).
    for f in &p.files {
        if !p
            .provides
            .iter()
            .any(|pr| pr.files().contains(&f.path.as_str()))
        {
            return no(
                "a listed file is not used",
                "목록의 파일을 쓰는 곳이 없어요",
            );
        }
    }
    Ok(())
}

/// The registry id of a pack's section.
pub fn section_id(pack: &str, part: &str) -> String {
    format!("pack.{pack}.{part}")
}

/// The class that fences a pack part's CSS and marks the elements it may style.
pub fn scope_class(pack: &str, part: &str) -> String {
    format!("pk-{pack}-{part}")
}

// ---------------------------------------------------------------- checking a download

/// One file's check, for the plain PASS/FAIL list.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FileReport {
    pub path: String,
    pub ok: bool,
    pub bytes: usize,
    pub sha256: String,
    /// Why not, in both languages (empty when ok).
    pub why_en: String,
    pub why_ko: String,
}

/// Check downloaded files against the pack's entry in the index: every listed file present,
/// the right size, the right sha256, and nothing extra. The text is checked as UTF-8.
/// Returns one report per listed file (and per extra file).
pub fn check_files(pack: &Pack, files: &[(String, Vec<u8>)]) -> Vec<FileReport> {
    let mut out = Vec::new();
    for e in &pack.files {
        let got = files.iter().find(|(n, _)| *n == e.path).map(|(_, d)| d);
        let (ok, bytes, sha, why) = match got {
            None => (false, 0, String::new(), ("missing", "빠져 있어요")),
            Some(d) if d.len() != e.bytes => (
                false,
                d.len(),
                sha256_hex(d),
                (
                    "the size is not the size in the list (cut short or changed)",
                    "크기가 목록과 달라요 (잘렸거나 바뀌었어요)",
                ),
            ),
            Some(d) if sha256_hex(d) != e.sha256 => (
                false,
                d.len(),
                sha256_hex(d),
                (
                    "the fingerprint is not the one in the list (the file was changed)",
                    "지문이 목록과 달라요 (파일이 바뀌었어요)",
                ),
            ),
            Some(d) if std::str::from_utf8(d).is_err() => (
                false,
                d.len(),
                sha256_hex(d),
                ("it is not plain text", "평범한 글이 아니에요"),
            ),
            Some(d) => (true, d.len(), sha256_hex(d), ("", "")),
        };
        out.push(FileReport {
            path: e.path.clone(),
            ok,
            bytes,
            sha256: sha,
            why_en: why.0.into(),
            why_ko: why.1.into(),
        });
    }
    for (n, d) in files {
        if !pack.files.iter().any(|e| e.path == *n) {
            out.push(FileReport {
                path: n
                    .chars()
                    .filter(|c| c.is_ascii_graphic())
                    .take(40)
                    .collect(),
                ok: false,
                bytes: d.len(),
                sha256: sha256_hex(d),
                why_en: "not in the list".into(),
                why_ko: "목록에 없는 파일이에요".into(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests;
