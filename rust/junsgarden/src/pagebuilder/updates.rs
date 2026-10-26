//! "Are there new sections?" A signed list of sections, published by the author in the GitHub
//! repo, read when the owner opens the editor (DESIGN.md section 7).
//!
//! HOW TO READ THIS FILE. `fetch` reads `rust/junsgarden/sections-manifest.json` and its `.sig`
//! from the repo,
//! `verify` checks the Ed25519 signature against the author's public key pinned below (the same
//! signature kind as the garden's signed `/app/config.json`, see address.rs), `parse` reads it
//! strictly, and `announced` lists the sections a newer garden release brings. Nothing here
//! ever places a section on a page or installs code: the editor only shows a badge.

use std::time::{Duration, Instant};

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::registry::SectionDef;
use crate::AppState;

/// The author's public key for the sections list (64 hex characters). Empty until the author
/// makes the key (docs/page-builder/sign-manifest.sh); `JUNSGARDEN_SECTIONS_KEY` overrides it.
/// With no key the check answers "not set up" and reads nothing.
pub const AUTHOR_SECTIONS_KEY: &str = "";

/// Where the author publishes the list (and `<this>.sig`).
pub const MANIFEST_URL: &str = "https://raw.githubusercontent.com/netnovaops-training/garden-for-all/main/rust/junsgarden/sections-manifest.json";

/// The signed text is this line followed by the manifest's exact bytes.
const MSG_PREFIX: &[u8] = b"junsgarden-sections-v1\n";
const MAX_BYTES: usize = 256 * 1024;
const TIMEOUT: Duration = Duration::from_secs(10);
/// How long one answer is reused.
pub const TTL: Duration = Duration::from_secs(6 * 3600);

/// The published list.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub released: String,
    pub sections: Vec<RemoteSection>,
}

/// One section in the published list.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteSection {
    pub id: String,
    pub version: u32,
    pub since: String,
    pub name_en: String,
    pub name_ko: String,
    pub desc_en: String,
    pub desc_ko: String,
}

/// The public key to check against: the setting, else the pinned one, else none.
pub fn key_from(setting: &str) -> Option<VerifyingKey> {
    let hex = if setting.trim().is_empty() {
        AUTHOR_SECTIONS_KEY
    } else {
        setting.trim()
    };
    VerifyingKey::from_bytes(&crate::address::hex32(hex)?).ok()
}

/// Does `sig_hex` (128 hex characters) sign this manifest with `key`?
pub fn verify(manifest: &[u8], sig_hex: &str, key: &VerifyingKey) -> bool {
    let sig_hex = sig_hex.trim();
    if sig_hex.len() != 128 {
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
    msg.extend_from_slice(manifest);
    key.verify_strict(&msg, &Signature::from_bytes(&sig))
        .is_ok()
}

/// Read the (already verified) list strictly: known keys only, plain ids, short texts.
pub fn parse(manifest: &[u8]) -> Option<Manifest> {
    let m: Manifest = serde_json::from_slice(manifest).ok()?;
    let id_ok = |s: &str| {
        !s.is_empty()
            && s.len() <= 64
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'_')
    };
    let text_ok = |s: &str| s.chars().count() <= 200 && !s.chars().any(char::is_control);
    let ok = m.schema == 1
        && m.sections.len() <= 200
        && text_ok(&m.released)
        && m.sections.iter().all(|s| {
            id_ok(&s.id)
                && [&s.since, &s.name_en, &s.name_ko, &s.desc_en, &s.desc_ko]
                    .iter()
                    .all(|t| text_ok(t))
        });
    ok.then_some(m)
}

/// Sections the list has that this garden does not, or has in an older version: they come
/// with the next garden update. Shown greyed out; they cannot be placed.
pub fn announced(local: &[SectionDef], m: &Manifest) -> Vec<RemoteSection> {
    m.sections
        .iter()
        .filter(|r| match local.iter().find(|l| l.id == r.id) {
            None => true,
            Some(l) => r.version > l.version,
        })
        .cloned()
        .collect()
}

/// Read the list and its signature from the repo.
async fn fetch(state: &AppState) -> Result<(Vec<u8>, String), String> {
    let get = |url: String| async move {
        let res = state
            .http
            .get(&url)
            .timeout(TIMEOUT)
            .send()
            .await
            .map_err(|e| e.without_url().to_string())?;
        if !res.status().is_success() {
            return Err(format!("HTTP {}", res.status().as_u16()));
        }
        if res.content_length().is_some_and(|n| n > MAX_BYTES as u64) {
            return Err("too big".to_string());
        }
        let body = res.bytes().await.map_err(|e| e.without_url().to_string())?;
        if body.len() > MAX_BYTES {
            return Err("too big".to_string());
        }
        Ok(body.to_vec())
    };
    let manifest = get(MANIFEST_URL.to_string()).await?;
    let sig = get(format!("{MANIFEST_URL}.sig")).await?;
    Ok((manifest, String::from_utf8_lossy(&sig).into_owned()))
}

/// The editor's answer: `{"status": "not_set_up" | "ok" | "failed", "announced": [...]}`,
/// reused for `TTL`.
pub async fn check(state: &AppState) -> Value {
    let Some(key) = state.pagebuilder.sections_key.as_ref() else {
        return json!({ "status": "not_set_up", "announced": [] });
    };
    if let Some(v) = state.pagebuilder.updates.lock().ok().and_then(|c| {
        c.as_ref()
            .filter(|(at, _)| at.elapsed() < TTL)
            .map(|(_, v)| v.clone())
    }) {
        return v;
    }
    let answer = match fetch(state).await {
        Err(e) => {
            tracing::warn!("section update check: {e}");
            json!({ "status": "failed", "announced": [] })
        }
        Ok((m, sig)) if !verify(&m, &sig, key) => {
            tracing::warn!("section update check: the signature does not match; ignored");
            json!({ "status": "failed", "announced": [] })
        }
        Ok((m, _)) => match parse(&m) {
            None => json!({ "status": "failed", "announced": [] }),
            Some(m) => json!({
                "status": "ok",
                "released": m.released,
                "announced": announced(super::registry::SECTIONS, &m),
            }),
        },
    };
    if let Ok(mut c) = state.pagebuilder.updates.lock() {
        *c = Some((Instant::now(), answer.clone()));
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    /// A throwaway test key (a fixed seed, only ever used in this test).
    fn test_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn sign(key: &SigningKey, manifest: &[u8]) -> String {
        let mut msg = MSG_PREFIX.to_vec();
        msg.extend_from_slice(manifest);
        key.sign(&msg)
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    const LIST: &str = r#"{"schema":1,"released":"2026.12","sections":[
        {"id":"home.tips","version":1,"since":"2026.11","name_en":"Tips","name_ko":"팁","desc_en":"d","desc_ko":"설명"},
        {"id":"home.news","version":2,"since":"2026.12","name_en":"News","name_ko":"소식","desc_en":"d","desc_ko":"설명"},
        {"id":"home.radar","version":1,"since":"2026.12","name_en":"Radar","name_ko":"레이더","desc_en":"d","desc_ko":"설명"}
    ]}"#;

    #[test]
    fn a_good_signature_passes_and_anything_else_fails() {
        let k = test_key();
        let vk = k.verifying_key();
        let sig = sign(&k, LIST.as_bytes());
        assert!(verify(LIST.as_bytes(), &sig, &vk));
        assert!(verify(LIST.as_bytes(), &format!("{sig}\n"), &vk));
        // One changed byte, a wrong key, a broken or empty signature: refused.
        let changed = LIST.replace("Radar", "Rader");
        assert!(!verify(changed.as_bytes(), &sig, &vk));
        let other = SigningKey::from_bytes(&[9u8; 32]).verifying_key();
        assert!(!verify(LIST.as_bytes(), &sig, &other));
        assert!(!verify(LIST.as_bytes(), &sig[..126], &vk));
        assert!(!verify(LIST.as_bytes(), "", &vk));
        assert!(!verify(LIST.as_bytes(), &"zz".repeat(64), &vk));
        // Signed without the prefix: refused (a signature made for something else).
        let bare: String = k
            .sign(LIST.as_bytes())
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(!verify(LIST.as_bytes(), &bare, &vk));
    }

    #[test]
    fn no_key_means_not_set_up() {
        assert!(AUTHOR_SECTIONS_KEY.is_empty() || key_from("").is_some());
        assert!(key_from("not hex").is_none());
        let hex: String = test_key()
            .verifying_key()
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(key_from(&hex).is_some());
    }

    #[test]
    fn announced_lists_only_new_or_newer_sections() {
        let m = parse(LIST.as_bytes()).unwrap();
        let ids: Vec<String> = announced(super::super::registry::SECTIONS, &m)
            .into_iter()
            .map(|s| s.id)
            .collect();
        assert_eq!(ids, ["home.news", "home.radar"]);
    }

    #[test]
    fn parse_is_strict() {
        assert!(parse(b"not json").is_none());
        assert!(parse(br#"{"schema":2,"released":"x","sections":[]}"#).is_none());
        assert!(parse(br#"{"schema":1,"released":"x","sections":[],"extra":1}"#).is_none());
        assert!(parse(br#"{"schema":1,"released":"x","sections":[{"id":"Bad Id","version":1,"since":"","name_en":"","name_ko":"","desc_en":"","desc_ko":""}]}"#).is_none());
        let long = "a".repeat(201);
        let j = format!(
            r#"{{"schema":1,"released":"x","sections":[{{"id":"a.b","version":1,"since":"","name_en":"{long}","name_ko":"","desc_en":"","desc_ko":""}}]}}"#
        );
        assert!(parse(j.as_bytes()).is_none());
    }

    #[test]
    fn the_published_list_matches_this_garden() {
        let text = include_str!("../../sections-manifest.json");
        let m = parse(text.as_bytes()).expect("the repo's manifest parses");
        for def in super::super::registry::SECTIONS {
            let r = m
                .sections
                .iter()
                .find(|r| r.id == def.id)
                .unwrap_or_else(|| panic!("{} missing from the manifest", def.id));
            assert_eq!(r.version, def.version, "{}", def.id);
            assert_eq!(r.name_en, def.name_en, "{}", def.id);
            assert_eq!(r.name_ko, def.name_ko, "{}", def.id);
        }
        assert!(announced(super::super::registry::SECTIONS, &m).is_empty());
    }
}
