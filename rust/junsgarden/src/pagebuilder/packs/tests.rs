//! Tests for the trust chain (signature, size, sha256), the strict index, the sources and the
//! shipped packs. No network, no database: downloads come from a table in memory.

use std::collections::HashMap;

use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};

use super::source::*;
use super::store::{verify_row, Row};
use super::*;
use crate::pagebuilder::site::{self, Loaded};

// ---------------------------------------------------------------- fixtures

fn key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn other_key() -> SigningKey {
    SigningKey::from_bytes(&[9u8; 32])
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sign(k: &SigningKey, index: &[u8]) -> String {
    let mut msg = MSG_PREFIX.to_vec();
    msg.extend_from_slice(index);
    hex(&k.sign(&msg).to_bytes())
}

const BASE: &str = "https://packs.example.org/p";
const ART: &str = "<svg viewBox=\"0 0 10 10\"><path class=\"pk-demo-a\" d=\"M0,0 L10,10\" fill=\"none\" stroke=\"#000\"/></svg>";
const CSS: &str = ".pk-demo-a { stroke-width: 2 }";

/// One pack `demo` 1.0.0 with a header (art + css), as the index entry and the file contents.
fn demo_pack() -> (Value, Vec<(&'static str, &'static str)>) {
    let files = vec![("top.svg", ART), ("demo.css", CSS)];
    let entry = json!({
        "id": "demo", "version": "1.0.0",
        "name_en": "Demo", "name_ko": "데모", "desc_en": "A demo.", "desc_ko": "데모예요.",
        "provides": [{"type": "header", "id": "h", "name_en": "H", "name_ko": "ㅎ", "top": "top.svg", "css": "demo.css"}],
        "files": files.iter().map(|(n, c)| json!({"path": n, "sha256": sha256_hex(c.as_bytes()), "bytes": c.len()})).collect::<Vec<_>>(),
    });
    (entry, files)
}

fn index_text(serial: u64, channel: &str, packs: Vec<Value>) -> String {
    serde_json::to_string_pretty(&json!({
        "schema": 1, "channel": channel, "serial": serial, "released": "2026.12", "packs": packs,
    }))
    .unwrap()
}

/// A web folder with a signed index and the demo pack's files.
fn folder(k: Option<&SigningKey>, serial: u64, channel: &str) -> Memory {
    let (entry, files) = demo_pack();
    let text = index_text(serial, channel, vec![entry]);
    let mut m = HashMap::new();
    m.insert(format!("{BASE}/index.json"), text.clone().into_bytes());
    if let Some(k) = k {
        m.insert(
            format!("{BASE}/index.json.sig"),
            sign(k, text.as_bytes()).into_bytes(),
        );
    }
    for (n, c) in files {
        m.insert(format!("{BASE}/demo/1.0.0/{n}"), c.as_bytes().to_vec());
    }
    Memory(m)
}

struct Memory(HashMap<String, Vec<u8>>);

impl Fetcher for Memory {
    async fn get(&self, url: &str, max: usize) -> Result<Vec<u8>, Problem> {
        match self.0.get(url) {
            None => Err(problem("HTTP 404", "HTTP 404")),
            Some(b) if b.len() > max => Err(problem("too big", "너무 커요")),
            Some(b) => Ok(b.clone()),
        }
    }
}

fn dir_source() -> Source {
    Source {
        name: "gitea".into(),
        kind: "dir".into(),
        url: BASE.into(),
    }
}

// ---------------------------------------------------------------- the index

#[test]
fn a_good_index_is_read() {
    let (entry, _) = demo_pack();
    let ix = parse_index(index_text(3, CHANNEL, vec![entry]).as_bytes()).unwrap();
    assert_eq!(ix.serial, 3);
    assert_eq!(ix.packs[0].provides[0].kind(), "header");
}

#[test]
fn the_index_is_read_strictly() {
    let (entry, _) = demo_pack();
    let with = |f: &dyn Fn(&mut Value)| {
        let mut e = entry.clone();
        f(&mut e);
        parse_index(index_text(1, CHANNEL, vec![e]).as_bytes())
    };
    assert!(with(&|_| {}).is_ok());
    for (what, bad) in [
        ("an unknown field", with(&|e| e["run"] = json!("x"))),
        ("a bad id", with(&|e| e["id"] = json!("Demo Pack"))),
        ("a path-like id", with(&|e| e["id"] = json!("../x"))),
        ("a bad version", with(&|e| e["version"] = json!("1.0"))),
        (
            "markup in a name",
            with(&|e| e["name_en"] = json!("<b>x</b>")),
        ),
        (
            "a file with a folder",
            with(&|e| e["files"][0]["path"] = json!("a/top.svg")),
        ),
        (
            "a file with dots",
            with(&|e| e["files"][0]["path"] = json!("../top.svg")),
        ),
        (
            "an unknown extension",
            with(&|e| e["files"][0]["path"] = json!("top.js")),
        ),
        (
            "a short hash",
            with(&|e| e["files"][0]["sha256"] = json!("abc")),
        ),
        (
            "an upper-case hash",
            with(&|e| e["files"][0]["sha256"] = json!("A".repeat(64))),
        ),
        (
            "a size of zero",
            with(&|e| e["files"][0]["bytes"] = json!(0)),
        ),
        (
            "a huge file",
            with(&|e| e["files"][0]["bytes"] = json!(70_000)),
        ),
        (
            "a part using an unlisted file",
            with(&|e| e["provides"][0]["top"] = json!("other.svg")),
        ),
        (
            "a listed file nobody uses",
            with(&|e| {
                e["files"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"path": "extra.svg", "sha256": "0".repeat(64), "bytes": 5}))
            }),
        ),
        (
            "the same file twice",
            with(&|e| {
                let f = e["files"][0].clone();
                e["files"].as_array_mut().unwrap().push(f);
            }),
        ),
        (
            "an unknown kind of part",
            with(&|e| e["provides"][0]["type"] = json!("script")),
        ),
        ("nothing provided", with(&|e| e["provides"] = json!([]))),
        (
            "a header that is builtin and has art",
            with(&|e| e["provides"][0]["builtin"] = json!(true)),
        ),
    ] {
        assert!(bad.is_err(), "{what} should be refused");
    }
    // Wrong channel, wrong schema, too big, not JSON.
    assert!(parse_index(index_text(1, "garden-test", vec![]).as_bytes()).is_err());
    assert!(parse_index(
        br#"{"schema":2,"channel":"garden-for-all","serial":1,"released":"x","packs":[]}"#
    )
    .is_err());
    assert!(parse_index(&vec![b' '; MAX_INDEX_BYTES + 1]).is_err());
    assert!(parse_index(b"not json").is_err());
    // Two packs with one name.
    let (a, _) = demo_pack();
    assert!(parse_index(index_text(1, CHANNEL, vec![a.clone(), a]).as_bytes()).is_err());
}

#[test]
fn ids_versions_and_file_names() {
    for ok in ["a", "garden-notes", "x1"] {
        assert!(id_ok(ok), "{ok}");
    }
    for bad in ["", "-a", "A", "a b", "a/b", "a.b", &"a".repeat(41)] {
        assert!(!id_ok(bad), "{bad}");
    }
    assert!(
        version_ok("1.2.3") && !version_ok("1.2") && !version_ok("1.2.x") && !version_ok("1.2.3.4")
    );
    assert!(version_key("1.10.0") > version_key("1.9.9"));
    for ok in ["top.svg", "my-art_2.css", "a.json"] {
        assert!(file_name_ok(ok), "{ok}");
    }
    for bad in [
        "top",
        ".svg",
        ".hidden.svg",
        "a..svg",
        "a/b.svg",
        "A.svg",
        "a b.svg",
        "a.exe",
        "a.svg.exe",
    ] {
        assert!(!file_name_ok(bad), "{bad}");
    }
}

// ---------------------------------------------------------------- the signature

#[test]
fn the_signature_must_be_the_authors_over_exactly_these_bytes() {
    let k = key();
    let vk = k.verifying_key();
    let text = b"{\"a\":1}";
    let sig = sign(&k, text);
    assert!(verify_signature(text, &sig, &vk));
    assert!(verify_signature(text, &format!("{sig}\n"), &vk));
    assert!(
        !verify_signature(b"{\"a\":2}", &sig, &vk),
        "one changed byte"
    );
    assert!(
        !verify_signature(text, &sign(&other_key(), text), &vk),
        "another key"
    );
    assert!(!verify_signature(text, &sig[..126], &vk) && !verify_signature(text, "", &vk));
    assert!(!verify_signature(text, &"zz".repeat(64), &vk));
    let bare = hex(&k.sign(text).to_bytes());
    assert!(
        !verify_signature(text, &bare, &vk),
        "signed without the prefix"
    );
    assert!(key_from("not hex").is_none());
    assert!(key_from(&hex(vk.as_bytes())).is_some());
}

// ---------------------------------------------------------------- looking, then downloading

#[tokio::test]
async fn a_signed_list_from_the_web_folder_is_accepted() {
    let k = key();
    let m = folder(Some(&k), 5, CHANNEL);
    let c = fetch_index(&m, &dir_source(), Some(&k.verifying_key()), 0)
        .await
        .unwrap();
    assert!(c.signed && c.index.serial == 5 && c.index.packs.len() == 1);
    // The same serial again is fine (a second look); a newer one too.
    assert!(fetch_index(&m, &dir_source(), Some(&k.verifying_key()), 5)
        .await
        .is_ok());
}

#[tokio::test]
async fn a_list_signed_by_someone_else_is_refused() {
    let m = folder(Some(&other_key()), 5, CHANNEL);
    let e = fetch_index(&m, &dir_source(), Some(&key().verifying_key()), 0)
        .await
        .unwrap_err();
    assert!(e.en.contains("signature") && e.ko.contains("서명"));
}

#[tokio::test]
async fn with_a_key_an_unsigned_or_altered_list_is_refused() {
    let k = key();
    let vk = k.verifying_key();
    // No .sig file.
    assert!(
        fetch_index(&folder(None, 5, CHANNEL), &dir_source(), Some(&vk), 0)
            .await
            .is_err()
    );
    // The list changed after it was signed (one pack's name).
    let mut m = folder(Some(&k), 5, CHANNEL);
    let url = format!("{BASE}/index.json");
    let text = String::from_utf8(m.0[&url].clone())
        .unwrap()
        .replace("\"Demo\"", "\"Demo!\"");
    m.0.insert(url, text.into_bytes());
    assert!(fetch_index(&m, &dir_source(), Some(&vk), 0).await.is_err());
}

#[tokio::test]
async fn without_a_key_the_list_is_accepted_but_marked_unsigned() {
    let c = fetch_index(&folder(None, 1, CHANNEL), &dir_source(), None, 0)
        .await
        .unwrap();
    assert!(!c.signed);
}

#[tokio::test]
async fn a_list_for_another_channel_is_refused_even_when_signed() {
    let k = key();
    let m = folder(Some(&k), 5, "garden-test");
    let e = fetch_index(&m, &dir_source(), Some(&k.verifying_key()), 0)
        .await
        .unwrap_err();
    assert!(e.en.contains("channel"));
}

#[tokio::test]
async fn an_older_signed_list_cannot_be_replayed() {
    let k = key();
    let m = folder(Some(&k), 4, CHANNEL);
    let e = fetch_index(&m, &dir_source(), Some(&k.verifying_key()), 5)
        .await
        .unwrap_err();
    assert!(e.en.contains("older"));
}

#[tokio::test]
async fn a_good_download_passes_every_file() {
    let k = key();
    let m = folder(Some(&k), 1, CHANNEL);
    let c = fetch_index(&m, &dir_source(), Some(&k.verifying_key()), 0)
        .await
        .unwrap();
    let d = download_pack(&m, &c, "demo").await.unwrap();
    assert!(d.ok && d.files.len() == 2 && d.report.iter().all(|r| r.ok));
    assert!(download_pack(&m, &c, "nope").await.is_err());
}

#[tokio::test]
async fn a_tampered_file_fails_and_nothing_is_kept() {
    let k = key();
    let mut m = folder(Some(&k), 1, CHANNEL);
    let c = fetch_index(&m, &dir_source(), Some(&k.verifying_key()), 0)
        .await
        .unwrap();
    // Same size, different content.
    let evil = ART.replace("L10,10", "L99,99");
    assert_eq!(evil.len(), ART.len());
    m.0.insert(format!("{BASE}/demo/1.0.0/top.svg"), evil.into_bytes());
    let d = download_pack(&m, &c, "demo").await.unwrap();
    assert!(!d.ok && d.files.is_empty());
    let bad = d.report.iter().find(|r| !r.ok).unwrap();
    assert_eq!(bad.path, "top.svg");
    assert!(bad.why_en.contains("fingerprint") && bad.why_ko.contains("지문"));
    assert!(
        d.report.iter().any(|r| r.ok && r.path == "demo.css"),
        "the other file still shows PASS"
    );
}

#[tokio::test]
async fn a_cut_short_download_fails() {
    let k = key();
    let mut m = folder(Some(&k), 1, CHANNEL);
    let c = fetch_index(&m, &dir_source(), Some(&k.verifying_key()), 0)
        .await
        .unwrap();
    m.0.insert(
        format!("{BASE}/demo/1.0.0/top.svg"),
        ART.as_bytes()[..40].to_vec(),
    );
    let d = download_pack(&m, &c, "demo").await.unwrap();
    assert!(!d.ok && d.files.is_empty());
    assert!(d.report.iter().any(|r| !r.ok && r.why_en.contains("size")));
}

#[tokio::test]
async fn a_longer_file_or_a_missing_file_fails() {
    let k = key();
    let c = fetch_index(
        &folder(Some(&k), 1, CHANNEL),
        &dir_source(),
        Some(&k.verifying_key()),
        0,
    )
    .await
    .unwrap();
    let mut longer = folder(Some(&k), 1, CHANNEL);
    longer.0.insert(
        format!("{BASE}/demo/1.0.0/demo.css"),
        format!("{CSS}\n.x{{}}").into_bytes(),
    );
    let d = download_pack(&longer, &c, "demo").await.unwrap();
    assert!(!d.ok && d.files.is_empty());
    let mut gone = folder(Some(&k), 1, CHANNEL);
    gone.0.remove(&format!("{BASE}/demo/1.0.0/top.svg"));
    let d = download_pack(&gone, &c, "demo").await.unwrap();
    assert!(!d.ok && d.files.is_empty());
    assert!(d
        .report
        .iter()
        .any(|r| !r.ok && r.path == "top.svg" && r.why_en.contains("404")));
}

#[tokio::test]
async fn the_second_source_is_asked_only_when_the_first_fails() {
    let k = key();
    let vk = k.verifying_key();
    let m = folder(Some(&k), 1, CHANNEL);
    let first = Source {
        name: "gitea".into(),
        kind: "dir".into(),
        url: "https://down.example.org/p".into(),
    };
    let unset = Source {
        name: "drive".into(),
        kind: "bundle".into(),
        url: String::new(),
    };
    let (c, notes) = fetch_first(
        &m,
        &[first.clone(), unset.clone(), dir_source()],
        Some(&vk),
        0,
    )
    .await
    .unwrap();
    assert_eq!(c.source.url, BASE);
    assert_eq!(
        notes.len(),
        1,
        "only the first failed; the unset one was skipped"
    );
    // The first works: the others are not asked at all.
    let (c, notes) = fetch_first(&m, &[dir_source(), first], Some(&vk), 0)
        .await
        .unwrap();
    assert_eq!(c.source.name, "gitea");
    assert!(notes.is_empty());
    assert!(fetch_first(&m, &[unset], Some(&vk), 0).await.is_err());
}

#[test]
fn only_https_to_a_real_website_is_a_source() {
    for ok in [
        "https://jasonagibson.zapto.org/git/Jason/garden-for-all/raw/branch/main/rust/junsgarden/packs",
        "https://raw.githubusercontent.com/o/r/main/rust/junsgarden/packs",
        "https://drive.google.com/uc?export=download&id=abc123",
        "https://example.org:8443/x",
    ] {
        assert!(url_ok(ok), "{ok}");
    }
    for bad in [
        "http://example.org/x",
        "ftp://example.org/x",
        "file:///etc/passwd",
        "//example.org/x",
        "https://",
        "https://localhost/x",
        "https://127.0.0.1/x",
        "https://192.168.1.5/x",
        "https://[::1]/x",
        "https://user:pass@example.org/x",
        "https://example.org@evil.test/x",
        "https://nas.local/x",
        "https://router.lan/x",
        "https://example.org:0/x",
        "https://example.org:99999/x",
        "https://exa mple.org/x",
        "https://example/x",
        "https://.example.org/x",
        "https://example.org\\x",
        "javascript:alert(1)",
        "",
    ] {
        assert!(!url_ok(bad), "{bad}");
    }
    let s = |name: &str, kind: &str, url: &str| Source {
        name: name.into(),
        kind: kind.into(),
        url: url.into(),
    };
    assert!(source_ok(&s("gitea", "dir", "https://example.org/p")).is_ok());
    assert!(
        source_ok(&s("drive", "bundle", "")).is_ok(),
        "an unset source is allowed"
    );
    assert!(source_ok(&s("Bad Name", "dir", "")).is_err());
    assert!(source_ok(&s("x", "ftp", "")).is_err());
    assert!(source_ok(&s("x", "dir", "http://example.org")).is_err());
}

// ---------------------------------------------------------------- the single-file bundle

fn bundle_bytes(k: Option<&SigningKey>, tamper: bool, channel: &str) -> Vec<u8> {
    let (entry, files) = demo_pack();
    let text = index_text(1, channel, vec![entry]);
    let sig = k.map(|k| sign(k, text.as_bytes())).unwrap_or_default();
    let mut f: HashMap<String, String> = files
        .iter()
        .map(|(n, c)| (format!("demo/1.0.0/{n}"), c.to_string()))
        .collect();
    if tamper {
        f.insert("demo/1.0.0/top.svg".into(), ART.replace("L10,10", "L99,99"));
    }
    serde_json::to_vec(&json!({"index": text, "sig": sig, "files": f})).unwrap()
}

fn bundle_source() -> Source {
    Source {
        name: "drive".into(),
        kind: "bundle".into(),
        url: "https://drive.example.org/b".into(),
    }
}

#[tokio::test]
async fn a_bundle_is_checked_like_a_web_folder() {
    let k = key();
    let vk = k.verifying_key();
    let mut m = HashMap::new();
    m.insert(
        "https://drive.example.org/b".to_string(),
        bundle_bytes(Some(&k), false, CHANNEL),
    );
    let m = Memory(m);
    let c = fetch_index(&m, &bundle_source(), Some(&vk), 0)
        .await
        .unwrap();
    assert!(c.signed);
    let d = download_pack(&m, &c, "demo").await.unwrap();
    assert!(d.ok && d.files.len() == 2);
}

#[tokio::test]
async fn a_bundle_with_a_changed_file_fails_that_file() {
    let k = key();
    let c = read_bundle(
        &bundle_bytes(Some(&k), true, CHANNEL),
        bundle_source(),
        Some(&k.verifying_key()),
        0,
    )
    .unwrap();
    let d = download_pack(&Memory(HashMap::new()), &c, "demo")
        .await
        .unwrap();
    assert!(!d.ok && d.files.is_empty());
}

#[test]
fn a_bundle_from_another_source_or_a_broken_one_is_refused() {
    let k = key();
    let vk = k.verifying_key();
    // Signed by a different key (a list from a different source).
    assert!(read_bundle(
        &bundle_bytes(Some(&other_key()), false, CHANNEL),
        bundle_source(),
        Some(&vk),
        0
    )
    .is_err());
    // Not signed at all, while a key is set.
    assert!(read_bundle(
        &bundle_bytes(None, false, CHANNEL),
        bundle_source(),
        Some(&vk),
        0
    )
    .is_err());
    // Another channel.
    assert!(read_bundle(
        &bundle_bytes(Some(&k), false, "garden-test"),
        bundle_source(),
        Some(&vk),
        0
    )
    .is_err());
    // Not a bundle.
    assert!(read_bundle(b"{}", bundle_source(), Some(&vk), 0).is_err());
    assert!(read_bundle(b"not json", bundle_source(), None, 0).is_err());
    // Without a key it is read, as unsigned.
    assert!(
        !read_bundle(
            &bundle_bytes(None, false, CHANNEL),
            bundle_source(),
            None,
            0
        )
        .unwrap()
        .signed
    );
}

// ---------------------------------------------------------------- kept packs ("Verify my install")

fn kept(k: Option<&SigningKey>) -> Row {
    let (entry, files) = demo_pack();
    let text = index_text(1, CHANNEL, vec![entry.clone()]);
    Row {
        id: 1,
        status: "installed".into(),
        source: "gitea".into(),
        signed: k.is_some(),
        pack: serde_json::from_value(entry).unwrap(),
        index_sig: k.map(|k| sign(k, text.as_bytes())).unwrap_or_default(),
        index_text: text,
        files: files
            .iter()
            .map(|(n, c)| (n.to_string(), c.as_bytes().to_vec()))
            .collect(),
        hashes: files
            .iter()
            .map(|(n, c)| (n.to_string(), sha256_hex(c.as_bytes())))
            .collect(),
        installed_by: "Owner".into(),
    }
}

#[test]
fn verify_my_install_passes_for_what_was_kept() {
    let k = key();
    let (report, sig) = verify_row(&kept(Some(&k)), Some(&k.verifying_key()));
    assert!(report.iter().all(|r| r.ok));
    assert!(sig.unwrap());
    let (_, sig) = verify_row(&kept(None), None);
    assert!(!sig.unwrap(), "unsigned, not failed");
}

#[test]
fn verify_my_install_catches_a_file_changed_after_it_was_kept() {
    let k = key();
    let mut r = kept(Some(&k));
    r.files[0].1 = ART.replace("L10,10", "L99,99").into_bytes();
    let (report, _) = verify_row(&r, Some(&k.verifying_key()));
    assert!(report.iter().any(|f| !f.ok));
    // Even if someone also edited the pack entry in the table, the recorded fingerprints and the
    // signed list still disagree with it.
    let mut r = kept(Some(&k));
    r.pack.files[0].sha256 = sha256_hex(b"x");
    let (_, sig) = verify_row(&r, Some(&k.verifying_key()));
    assert!(sig.is_err(), "the entry is not the one in the signed list");
}

#[test]
fn verify_my_install_catches_a_wrong_key_and_a_swapped_list() {
    let k = key();
    let r = kept(Some(&k));
    let (_, sig) = verify_row(&r, Some(&other_key().verifying_key()));
    assert!(sig.is_err());
    let mut swapped = kept(Some(&k));
    swapped.index_text = swapped.index_text.replace("2026.12", "2027.01");
    let (_, sig) = verify_row(&swapped, Some(&k.verifying_key()));
    assert!(sig.is_err());
    // A recorded fingerprint that no longer matches (the database itself was edited).
    let mut r = kept(Some(&k));
    r.hashes.insert("demo.css".into(), sha256_hex(b"other"));
    let (report, _) = verify_row(&r, Some(&k.verifying_key()));
    assert!(report.iter().any(|f| f.path == "demo.css" && !f.ok));
}

// ---------------------------------------------------------------- the packs that ship with the garden

const SHIPPED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/packs");

fn shipped() -> (Index, Vec<Loaded>) {
    let text = std::fs::read(format!("{SHIPPED}/index.json")).expect("packs/index.json");
    let ix = parse_index(&text).expect("the shipped index parses");
    let loaded = ix
        .packs
        .iter()
        .map(|p| Loaded {
            pack: p.clone(),
            files: p
                .files
                .iter()
                .map(|f| {
                    let path = format!("{SHIPPED}/{}/{}/{}", p.id, p.version, f.path);
                    (
                        f.path.clone(),
                        std::fs::read_to_string(path).expect("a listed file"),
                    )
                })
                .collect(),
        })
        .collect();
    (ix, loaded)
}

#[test]
fn the_shipped_index_matches_the_shipped_files() {
    let (ix, loaded) = shipped();
    assert!(ix.packs.len() >= 4);
    for l in &loaded {
        let files: Vec<(String, Vec<u8>)> = l
            .files
            .iter()
            .map(|(k, v)| (k.clone(), v.clone().into_bytes()))
            .collect();
        let report = check_files(&l.pack, &files);
        assert!(
            report.iter().all(|r| r.ok),
            "{}: {:?}",
            l.pack.id,
            report.iter().filter(|r| !r.ok).collect::<Vec<_>>()
        );
        // Nothing else lies in the version folder.
        let dir = format!("{SHIPPED}/{}/{}", l.pack.id, l.pack.version);
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let n = e.file_name().to_string_lossy().into_owned();
                assert!(
                    l.pack.files.iter().any(|f| f.path == n),
                    "{n} is in {dir} but not in the index"
                );
            }
        }
    }
}

#[test]
fn every_shipped_file_passes_its_strict_reader() {
    let (_, loaded) = shipped();
    let mut art = 0;
    for l in &loaded {
        for p in &l.pack.provides {
            match p {
                Provide::Header {
                    top,
                    bottom,
                    hero,
                    css: c,
                    id,
                    ..
                } => {
                    for f in [top, bottom, hero].into_iter().flatten() {
                        svg::sanitize(&l.files[f], &l.pack.id)
                            .unwrap_or_else(|e| panic!("{f}: {}", e.en));
                        art += 1;
                    }
                    if let Some(c) = c {
                        css::sanitize(&l.files[c], &l.pack.id, &scope_class(&l.pack.id, id))
                            .unwrap_or_else(|e| panic!("{c}: {}", e.en));
                    }
                }
                Provide::CatalogLayout { css: c, id, .. } => {
                    css::sanitize(&l.files[c], &l.pack.id, &scope_class(&l.pack.id, id))
                        .unwrap_or_else(|e| panic!("{c}: {}", e.en));
                }
                Provide::Section { template, .. } => {
                    blocks::parse(&l.files[template])
                        .unwrap_or_else(|e| panic!("{template}: {}", e.en));
                }
                Provide::Theme { vars, .. } => {
                    css::theme(vars).unwrap();
                }
            }
        }
    }
    assert!(art >= 3, "the vines come with three drawings");
}

// ---------------------------------------------------------------- what the choices switch on

fn st(enabled: &[&str], header: Option<&str>, catalog: Option<&str>) -> site::State {
    site::State {
        schema: 1,
        enabled: enabled.iter().map(|s| s.to_string()).collect(),
        header: header.map(String::from),
        catalog_layout: catalog.map(String::from),
        theme: None,
    }
}

#[test]
fn nothing_is_switched_on_until_the_owner_does_it() {
    let (_, loaded) = shipped();
    let a = site::build_active(&st(&[], None, None), &loaded);
    assert!(
        a.top.is_none() && a.head.is_empty() && a.sections.is_empty() && a.catalog_class.is_none()
    );
    // Installed but off: still nothing, even when a choice names it.
    let a = site::build_active(
        &st(
            &[],
            Some("header-vines/vines"),
            Some("catalog-table/compact"),
        ),
        &loaded,
    );
    assert!(a.top.is_none() && a.catalog_class.is_none());
}

#[test]
fn the_vines_header_brings_art_for_all_three_places_and_fenced_styles() {
    let (_, loaded) = shipped();
    let a = site::build_active(
        &st(&["header-vines"], Some("header-vines/vines"), None),
        &loaded,
    );
    let top = a.top.unwrap();
    assert!(top.starts_with("<div class=\"edge-wave edge-wave--header pk-art pk-header-vines-vines\" aria-hidden=\"true\"><svg "));
    assert!(a.bottom.unwrap().contains("edge-wave--footer"));
    assert!(a
        .hero
        .unwrap()
        .starts_with("<div class=\"hero-wave pk-art "));
    assert!(a.head.starts_with("\n<style id=\"pb-pack-style\">"));
    // Every rule in the style block is fenced behind the pack's own class.
    let rules = a
        .head
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with(['<', '@', '}']));
    let mut n = 0;
    for line in rules {
        assert!(line.starts_with(".pk-header-vines-vines "), "{line}");
        n += 1;
    }
    assert!(n >= 10, "the plain rules were found ({n})");
    // Inside a screen condition too.
    assert!(a
        .head
        .contains("@media (prefers-reduced-motion:reduce){.pk-header-vines-vines "));
    assert!(!a.head.contains("url(") && !a.head.contains("<script"));
}

#[test]
fn the_builtin_waves_choice_changes_nothing() {
    let (_, loaded) = shipped();
    let a = site::build_active(
        &st(&["header-waves"], Some("header-waves/waves"), None),
        &loaded,
    );
    assert!(a.top.is_none() && a.bottom.is_none() && a.hero.is_none() && a.head.is_empty());
}

#[test]
fn a_catalog_layout_and_pack_sections_are_switched_on_by_name() {
    let (_, loaded) = shipped();
    let a = site::build_active(
        &st(
            &["catalog-table", "garden-notes"],
            None,
            Some("catalog-table/compact"),
        ),
        &loaded,
    );
    assert_eq!(a.catalog_class.as_deref(), Some("pk-catalog-table-compact"));
    assert!(a
        .head
        .contains(".pk-catalog-table-compact .plant-grid{grid-template-columns:1fr;"));
    let s = &a.sections["pack.garden-notes.welcome-note"];
    assert!(s
        .html
        .contains("Welcome to the garden · 정원에 오신 것을 환영해요"));
    assert!(a.extra.contains_key("pack.garden-notes.welcome-note"));
}

#[test]
fn a_part_that_fails_its_reader_is_left_out_and_the_rest_still_works() {
    let (_, mut loaded) = shipped();
    for l in &mut loaded {
        if l.pack.id == "header-vines" {
            l.files.insert(
                "top.svg".into(),
                "<svg viewBox=\"0 0 1 1\"><script>x</script></svg>".into(),
            );
        }
        if l.pack.id == "catalog-table" {
            l.files.insert(
                "compact.css".into(),
                ".a { background: url(https://evil.test/x) }".into(),
            );
        }
    }
    let a = site::build_active(
        &st(
            &["header-vines", "catalog-table", "garden-notes"],
            Some("header-vines/vines"),
            Some("catalog-table/compact"),
        ),
        &loaded,
    );
    assert!(a.top.is_none(), "the bad top art is not used");
    assert!(a.bottom.is_some(), "the others are");
    assert!(a.catalog_class.is_none() && !a.head.contains("evil"));
    assert_eq!(a.sections.len(), 1);
}

#[test]
fn choices_must_come_from_installed_features_that_are_on() {
    let (_, loaded) = shipped();
    let ok = |s: &site::State| site::validate(s, &loaded).is_ok();
    assert!(ok(&st(&[], None, None)));
    assert!(ok(&st(&["header-vines"], Some("header-vines/vines"), None)));
    assert!(!ok(&st(&["not-installed"], None, None)), "not installed");
    assert!(
        !ok(&st(&["header-vines", "header-vines"], None, None)),
        "twice"
    );
    assert!(
        !ok(&st(&[], Some("header-vines/vines"), None)),
        "chosen but off"
    );
    assert!(
        !ok(&st(&["header-vines"], Some("header-vines/nope"), None)),
        "no such part"
    );
    assert!(
        !ok(&st(&["catalog-table"], Some("catalog-table/compact"), None)),
        "a layout is not a header"
    );
    assert!(!ok(&st(&["header-vines"], Some("../x"), None)));
    let mut bad = st(&[], None, None);
    bad.schema = 2;
    assert!(!ok(&bad));
}
