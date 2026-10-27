//! Where packs come from, and how a download is checked before anything is kept.
//!
//! Two kinds of source, both over HTTPS only:
//! * `dir`: a web folder with `index.json`, `index.json.sig` and `<id>/<version>/<file>`
//!   (the author's Gitea or, later, GitHub raw files);
//! * `bundle`: ONE file holding the index, its signature and every pack file (the plain
//!   download link of the author's Google Drive copy, or the wizard's downloaded file).
//!
//! The steps are separate on purpose, so the owner sees the list before anything is downloaded:
//! `fetch_index` reads only the small signed index; `download_pack` then fetches that one pack's
//! files and checks each against the index. Nothing is stored here.

use std::collections::HashMap;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use super::{
    check_files, parse_index, problem, verify_signature, FileReport, Index, Pack, Problem,
    MAX_FILE_BYTES, MAX_INDEX_BYTES,
};
use ed25519_dalek::VerifyingKey;

/// The author's private Gitea (the packs sit in the repo, under `rust/junsgarden/packs`).
/// Later the same folder on GitHub can be added as another source.
pub const DEFAULT_GITEA: &str =
    "https://jasonagibson.zapto.org/git/Jason/garden-for-all/raw/branch/main/rust/junsgarden/packs";
/// One bundle file may be this big (the index plus every file of every pack).
pub const MAX_BUNDLE_BYTES: usize = 1536 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub name: String,
    /// `dir` or `bundle`.
    pub kind: String,
    /// Empty = this source is not set up yet (it is skipped).
    pub url: String,
}

/// The sources a new garden starts with: the author's Gitea first, then the Drive bundle
/// (empty until its link is set).
pub fn default_sources(gitea: &str, drive: &str) -> Vec<Source> {
    vec![
        Source {
            name: "gitea".into(),
            kind: "dir".into(),
            url: if gitea.trim().is_empty() {
                DEFAULT_GITEA.into()
            } else {
                gitea.trim().into()
            },
        },
        Source {
            name: "drive".into(),
            kind: "bundle".into(),
            url: drive.trim().into(),
        },
    ]
}

/// Is this a web address the garden may read? `https://`, a real host name (not an address
/// number, `localhost` or a name that only exists inside a home network), no password in it.
pub fn url_ok(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    if url.len() > 400 || url.bytes().any(|b| b <= b' ' || b == b'\\' || b >= 0x7f) {
        return false;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.contains('@') || authority.contains('[') {
        return false;
    }
    let (host, port) = match authority.split_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (authority, None),
    };
    if port.is_some_and(|p| p.parse::<u16>().map_or(true, |n| n == 0)) {
        return false;
    }
    let host = host.to_ascii_lowercase();
    let numeric = host.bytes().all(|b| b.is_ascii_digit() || b == b'.');
    host.contains('.')
        && !numeric
        && !host.starts_with('.')
        && !host.ends_with('.')
        && !host.contains("..")
        && ![
            ".local",
            ".localhost",
            ".internal",
            ".lan",
            ".home",
            ".corp",
        ]
        .iter()
        .any(|s| host.ends_with(s))
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
}

/// A source the owner typed or saved.
pub fn source_ok(s: &Source) -> Result<(), Problem> {
    let name_ok = !s.name.is_empty()
        && s.name.len() <= 20
        && s.name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !name_ok || !matches!(s.kind.as_str(), "dir" | "bundle") {
        return Err(problem(
            "That source has a bad name or kind.",
            "그 출처의 이름이나 종류가 맞지 않아요.",
        ));
    }
    if !s.url.is_empty() && !url_ok(&s.url) {
        return Err(problem(
            "A source must be a web address that starts with https:// (a real website name).",
            "출처는 https:// 로 시작하는 웹 주소여야 해요 (실제 웹사이트 이름).",
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------- fetching

/// Reads bytes from an address, at most `max` of them. The real one is `Https`; tests use a
/// table in memory.
#[allow(async_fn_in_trait)]
pub trait Fetcher {
    async fn get(&self, url: &str, max: usize) -> Result<Vec<u8>, Problem>;
}

/// The real reader: HTTPS only (also after a redirect), a short timeout, a size limit that is
/// enforced while reading, and no more than three redirects.
pub struct Https(reqwest::Client);

impl Https {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::limited(3))
            .connect_timeout(std::time::Duration::from_secs(8))
            .timeout(std::time::Duration::from_secs(20))
            .user_agent("junsgarden-packs/1")
            .build()
            .unwrap_or_default();
        Https(client)
    }
}

impl Default for Https {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher for Https {
    async fn get(&self, url: &str, max: usize) -> Result<Vec<u8>, Problem> {
        if !url_ok(url) {
            return Err(problem(
                "That address is not allowed (https:// only).",
                "허용되지 않는 주소예요 (https:// 만 돼요).",
            ));
        }
        let down = |why: String| {
            problem(
                format!("Could not read the source ({why})."),
                format!("출처를 읽지 못했어요 ({why})."),
            )
        };
        let res = self
            .0
            .get(url)
            .send()
            .await
            .map_err(|e| down(e.without_url().to_string()))?;
        if !res.status().is_success() {
            return Err(down(format!("HTTP {}", res.status().as_u16())));
        }
        if res.content_length().is_some_and(|n| n > max as u64) {
            return Err(problem("The file is too big.", "파일이 너무 커요."));
        }
        let mut body = Vec::new();
        let mut stream = res.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| down(e.without_url().to_string()))?;
            if body.len() + chunk.len() > max {
                return Err(problem("The file is too big.", "파일이 너무 커요."));
            }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }
}

// ---------------------------------------------------------------- the index

/// A checked index: what the owner is shown before anything is downloaded.
#[derive(Clone, Debug)]
pub struct Checked {
    pub index: Index,
    /// The index's exact text and its signature (kept, so "Verify my install" can check again).
    pub text: String,
    pub sig: String,
    /// The signature matched the author's key.
    pub signed: bool,
    pub source: Source,
    /// For a bundle: every file in it, by `<id>/<version>/<file>`.
    pub bundle: Option<HashMap<String, Vec<u8>>>,
}

/// The index's signature against the key. With a key: it must match. Without one the index is
/// accepted as "unsigned" (`Ok(false)`).
pub fn check_signature(
    text: &[u8],
    sig: &str,
    key: Option<&VerifyingKey>,
) -> Result<bool, Problem> {
    match key {
        None => Ok(false),
        Some(k) if verify_signature(text, sig, k) => Ok(true),
        Some(_) => Err(problem(
            "The signature on the list does not match the author's key. Nothing was used.",
            "목록의 서명이 제작자의 키와 맞지 않아요. 아무것도 쓰지 않았어요.",
        )),
    }
}

/// An index older than one already accepted is refused (an old signed list cannot be replayed).
pub fn check_serial(serial: u64, last_accepted: u64) -> Result<(), Problem> {
    if serial < last_accepted {
        return Err(problem(
            "That list is older than the one you already have. Nothing was used.",
            "그 목록은 이미 받은 것보다 오래됐어요. 아무것도 쓰지 않았어요.",
        ));
    }
    Ok(())
}

/// A bundle file: the index text, its signature and all the files.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleFile {
    index: String,
    sig: String,
    files: HashMap<String, String>,
}

/// Read a bundle's bytes into a checked index (signature, form, channel) with its files.
pub fn read_bundle(
    bytes: &[u8],
    source: Source,
    key: Option<&VerifyingKey>,
    last_serial: u64,
) -> Result<Checked, Problem> {
    let b: BundleFile = serde_json::from_slice(bytes).map_err(|_| {
        problem(
            "That file is not a feature bundle.",
            "그 파일은 기능 묶음이 아니에요.",
        )
    })?;
    let signed = check_signature(b.index.as_bytes(), &b.sig, key)?;
    let index = parse_index(b.index.as_bytes())?;
    check_serial(index.serial, last_serial)?;
    let files = b
        .files
        .into_iter()
        .map(|(k, v)| (k, v.into_bytes()))
        .collect();
    Ok(Checked {
        index,
        text: b.index,
        sig: b.sig,
        signed,
        source,
        bundle: Some(files),
    })
}

/// Step 1: read only the small signed index from a source.
pub async fn fetch_index<F: Fetcher>(
    f: &F,
    source: &Source,
    key: Option<&VerifyingKey>,
    last_serial: u64,
) -> Result<Checked, Problem> {
    source_ok(source)?;
    if source.url.is_empty() {
        return Err(problem(
            "This source is not set up yet.",
            "이 출처는 아직 설정되지 않았어요.",
        ));
    }
    if source.kind == "bundle" {
        let bytes = f.get(&source.url, MAX_BUNDLE_BYTES).await?;
        return read_bundle(&bytes, source.clone(), key, last_serial);
    }
    let base = source.url.trim_end_matches('/');
    let text = f
        .get(&format!("{base}/index.json"), MAX_INDEX_BYTES)
        .await?;
    // An unsigned source has no .sig file; a missing one is only a problem when a key is set.
    let sig = match f.get(&format!("{base}/index.json.sig"), 1024).await {
        Ok(s) => String::from_utf8_lossy(&s).trim().to_string(),
        Err(e) if key.is_some() => return Err(e),
        Err(_) => String::new(),
    };
    let signed = check_signature(&text, &sig, key)?;
    let index = parse_index(&text)?;
    check_serial(index.serial, last_serial)?;
    Ok(Checked {
        index,
        text: String::from_utf8_lossy(&text).into_owned(),
        sig,
        signed,
        source: source.clone(),
        bundle: None,
    })
}

/// Try the sources in order and use the first that answers (the second is only asked when the
/// first fails). Returns the answer and a note per source that failed.
pub async fn fetch_first<F: Fetcher>(
    f: &F,
    sources: &[Source],
    key: Option<&VerifyingKey>,
    last_serial: u64,
) -> Result<(Checked, Vec<(String, Problem)>), Problem> {
    let mut failed = Vec::new();
    for s in sources.iter().filter(|s| !s.url.is_empty()) {
        match fetch_index(f, s, key, last_serial).await {
            Ok(c) => return Ok((c, failed)),
            Err(e) => failed.push((s.name.clone(), e)),
        }
    }
    Err(failed.pop().map(|(_, e)| e).unwrap_or_else(|| {
        problem(
            "No source is set up to look for new features.",
            "새 기능을 찾을 출처가 설정되어 있지 않아요.",
        )
    }))
}

/// What a download gave: the files (kept only when every check passed) and one PASS/FAIL line
/// per file.
#[derive(Debug)]
pub struct Downloaded {
    pub pack: Pack,
    pub files: Vec<(String, Vec<u8>)>,
    pub report: Vec<FileReport>,
    pub ok: bool,
}

/// Step 2: fetch one pack's files (from the web folder, or out of the bundle) and check each
/// against the index. If anything fails, `files` is empty: nothing is kept.
pub async fn download_pack<F: Fetcher>(
    f: &F,
    checked: &Checked,
    id: &str,
) -> Result<Downloaded, Problem> {
    let pack = checked
        .index
        .packs
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or_else(|| {
            problem(
                "That feature is not in the list.",
                "그 기능은 목록에 없어요.",
            )
        })?;
    let mut got: Vec<(String, Vec<u8>)> = Vec::new();
    let mut missing: Vec<FileReport> = Vec::new();
    for e in &pack.files {
        let key = format!("{}/{}/{}", pack.id, pack.version, e.path);
        let bytes = match &checked.bundle {
            Some(files) => files
                .get(&key)
                .cloned()
                .ok_or_else(|| problem("A file is not in the bundle.", "묶음에 파일이 없어요.")),
            None => {
                let base = checked.source.url.trim_end_matches('/');
                // One byte more than allowed, so a longer file is seen as wrong, not cut.
                f.get(
                    &format!("{base}/{key}"),
                    (e.bytes + 1).min(MAX_FILE_BYTES + 1),
                )
                .await
            }
        };
        match bytes {
            Ok(b) => got.push((e.path.clone(), b)),
            Err(why) => missing.push(FileReport {
                path: e.path.clone(),
                ok: false,
                bytes: 0,
                sha256: String::new(),
                why_en: why.en,
                why_ko: why.ko,
            }),
        }
    }
    let mut report = check_files(&pack, &got);
    // A file that could not be fetched replaces its "missing" line with the real reason.
    for m in missing {
        match report.iter_mut().find(|r| r.path == m.path) {
            Some(r) => *r = m,
            None => report.push(m),
        }
    }
    let ok = report.iter().all(|r| r.ok);
    Ok(Downloaded {
        pack,
        files: if ok { got } else { Vec::new() },
        report,
        ok,
    })
}
