//! The site's own small data files: the guestbook (comments.txt), the gallery and plant
//! comments (two JSON files) and the admin's "someone commented" flag.
//!
//! HOW TO READ THIS FILE (for Jun). Two rules keep these files safe:
//!
//! 1. **A write never leaves half a file.** `write_atomic` writes a new copy next to the
//!    old one and then swaps it in with one rename. If the power goes in the middle, the
//!    old file is still there, whole.
//! 2. **A file that cannot be read is never overwritten.** If a comments file is damaged,
//!    `update_json` refuses to save rather than replace it with just the new comment
//!    (the old code did that, which would have lost every earlier comment). The problem
//!    is logged so it can be fixed by hand.
//!
//! Everything here is plain, blocking file work. Pages call it through
//! `crate::guard::off_thread`, so it never holds up the async server.

use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value};

/// A data file's path that has been checked to be a plain file directly inside its folder.
/// Only [`file_in`] makes one, and every read and write below takes one, so no path built
/// from outside input can reach the disk without that check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafePath(PathBuf);

impl SafePath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Is `name` a plain file name: 1 to 128 of `A-Z a-z 0-9 _ . -`, starting with a letter or
/// digit, and no `..`? (So never a folder, a hidden file, or a step outside the folder.)
fn plain_name(name: &str) -> bool {
    let b = name.as_bytes();
    (1..=128).contains(&b.len())
        && b[0].is_ascii_alphanumeric()
        && b.iter()
            .all(|&c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'-'))
        && !name.contains("..")
}

/// The file `name` inside the folder `dir`, checked: `name` must be a plain file name, and
/// the real path (links followed, folder resolved) must still start with the real `dir`.
/// Anything else is refused with `InvalidInput`. The folder must exist.
pub fn file_in(dir: &Path, name: &str) -> std::io::Result<SafePath> {
    let refuse = || std::io::Error::new(ErrorKind::InvalidInput, "not a plain file in its folder");
    if !plain_name(name) {
        return Err(refuse());
    }
    let base = dir.canonicalize()?;
    let joined = base.join(name);
    // A file that is already there may be a link: follow it, it must stay inside `dir`.
    let real = match joined.canonicalize() {
        Ok(p) => p,
        Err(e) if e.kind() == ErrorKind::NotFound => joined,
        Err(e) => return Err(e),
    };
    if real.starts_with(&base) && real.parent() == Some(base.as_path()) {
        Ok(SafePath(real))
    } else {
        Err(refuse())
    }
}

/// Why a data file could not be used.
#[derive(Debug)]
pub enum StoreError {
    /// The disk said no (permissions, full disk...).
    Io(std::io::Error),
    /// The file is there but is not the JSON object it should be.
    Damaged(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "{e}"),
            StoreError::Damaged(why) => write!(f, "damaged file: {why}"),
        }
    }
}

/// The text of a file; a file that does not exist yet reads as "".
pub fn read_text(path: &SafePath) -> std::io::Result<String> {
    match std::fs::read_to_string(path.as_path()) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(String::new()),
        other => other,
    }
}

/// A JSON file's top-level object. Missing file: an empty object (the first comment ever
/// must work). A file that is there but not a JSON object: `Damaged`.
pub fn read_json(path: &SafePath) -> Result<Map<String, Value>, StoreError> {
    let bytes = match std::fs::read(path.as_path()) {
        Ok(b) => b,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Map::new()),
        Err(e) => return Err(StoreError::Io(e)),
    };
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(m)) => Ok(m),
        Ok(_) => Err(StoreError::Damaged(format!(
            "{} is not a JSON object",
            path.as_path().display()
        ))),
        Err(e) => Err(StoreError::Damaged(format!(
            "{}: {e}",
            path.as_path().display()
        ))),
    }
}

/// Replace a file's contents in one step: write `path.tmp`, flush it to disk, rename it
/// over `path`. A rename within one folder is atomic, so readers see the old file or the
/// new one, never a mix.
///
/// A folder the site may not create files in (only change the existing ones) falls back to
/// writing the file in place, as the PHP always did.
pub fn write_atomic(path: &SafePath, contents: &[u8]) -> std::io::Result<()> {
    let path = path.as_path();
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let mut f = match std::fs::File::create(&tmp) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::PermissionDenied => return std::fs::write(path, contents),
        Err(e) => return Err(e),
    };
    f.write_all(contents)?;
    f.sync_all()?;
    std::fs::rename(&tmp, path)
}

/// `json_encode($data, JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE)`: four spaces of indent,
/// as the PHP wrote these files, so they look the same to anyone opening them.
pub fn pretty_json(data: &Map<String, Value>) -> Vec<u8> {
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    // Writing a Map of Values into memory cannot fail.
    let _ = data.serialize(&mut ser);
    buf
}

/// Read a JSON file, let `change` edit it, write it back atomically. `change` returns
/// `None` to mean "nothing to save" (the file is left alone) or `Some(answer)`.
/// The caller must hold the lock for this file (`AppState::comments_lock`).
pub fn update_json<R>(
    path: &SafePath,
    change: impl FnOnce(&mut Map<String, Value>) -> Option<R>,
) -> Result<Option<R>, StoreError> {
    let mut data = read_json(path)?;
    let Some(answer) = change(&mut data) else {
        return Ok(None);
    };
    write_atomic(path, &pretty_json(&data)).map_err(StoreError::Io)?;
    Ok(Some(answer))
}

/// A file read and parsed once, then reused until the file changes (its modified time or
/// size). For files the pages read on every visit but that change rarely (the gallery list).
pub struct Parsed<T> {
    slot: std::sync::Mutex<Option<Stamped<T>>>,
}

/// A parsed file with the modified time and size it had when it was read.
type Stamped<T> = (Option<std::time::SystemTime>, u64, std::sync::Arc<T>);

impl<T> Default for Parsed<T> {
    fn default() -> Self {
        Parsed {
            slot: std::sync::Mutex::new(None),
        }
    }
}

impl<T> Parsed<T> {
    /// The parsed file: the kept copy if the file is unchanged, else `load(path)` again.
    /// Blocking (it looks at the file); call it inside `off_thread`.
    pub fn get(&self, path: &Path, load: impl FnOnce(&Path) -> T) -> std::sync::Arc<T> {
        let meta = std::fs::metadata(path).ok();
        let stamp = (
            meta.as_ref().and_then(|m| m.modified().ok()),
            meta.as_ref().map_or(0, |m| m.len()),
        );
        if let Ok(slot) = self.slot.lock() {
            if let Some((when, len, value)) = slot.as_ref() {
                if (*when, *len) == stamp && stamp.0.is_some() {
                    return value.clone();
                }
            }
        }
        let value = std::sync::Arc::new(load(path));
        if let Ok(mut slot) = self.slot.lock() {
            *slot = Some((stamp.0, stamp.1, value.clone()));
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../target")
            .join(format!("jg-store-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_files_read_as_empty() {
        let dir = scratch("missing");
        assert_eq!(read_text(&file_in(&dir, "none.txt").unwrap()).unwrap(), "");
        assert!(read_json(&file_in(&dir, "none.json").unwrap())
            .unwrap()
            .is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_plain_files_inside_the_folder_are_allowed() {
        let dir = scratch("confined");
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        for bad in [
            "../outside.json",
            "..",
            "a/../../x",
            "sub/x.json",
            "/etc/passwd",
            "..\\x",
            ".hidden",
            "",
            "a\0b",
            "x..json",
            &"a".repeat(129),
        ] {
            let e = file_in(&dir, bad).unwrap_err();
            assert_eq!(e.kind(), ErrorKind::InvalidInput, "{bad:?} was let through");
        }
        let ok = file_in(&dir, "plant_comments.json").unwrap();
        assert_eq!(
            ok.as_path().parent(),
            Some(dir.canonicalize().unwrap().as_path())
        );
        assert!(file_in(&dir, "insights_0123456789abcdef0123456789abcdef.json").is_ok());
        // A folder that is not there: an error, never a path somewhere else.
        assert!(file_in(&dir.join("missing"), "a.json").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_link_out_of_the_folder_is_refused() {
        let dir = scratch("link");
        let outside = scratch("link-outside");
        std::fs::write(outside.join("secret.txt"), "x").unwrap();
        std::os::unix::fs::symlink(outside.join("secret.txt"), dir.join("comments.txt")).unwrap();
        assert_eq!(
            file_in(&dir, "comments.txt").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn a_damaged_file_is_never_overwritten() {
        let dir = scratch("damaged");
        let p = file_in(&dir, "c.json").unwrap();
        std::fs::write(p.as_path(), b"{\"276\": [half a file").unwrap();
        let r = update_json(&p, |m| {
            m.insert("1".into(), json!([]));
            Some(())
        });
        assert!(matches!(r, Err(StoreError::Damaged(_))));
        assert_eq!(
            std::fs::read(p.as_path()).unwrap(),
            b"{\"276\": [half a file"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_writes_four_space_json_and_leaves_no_temp_file() {
        let dir = scratch("update");
        let p = file_in(&dir, "c.json").unwrap();
        update_json(&p, |m| {
            m.insert("k".into(), json!([{"a": 1}]));
            Some(())
        })
        .unwrap();
        let s = std::fs::read_to_string(p.as_path()).unwrap();
        assert!(s.contains("\n    \"k\": ["), "{s}");
        assert!(!dir.join("c.json.tmp").exists());
        // "nothing to save" leaves the file exactly as it was
        let before = std::fs::read(p.as_path()).unwrap();
        assert_eq!(update_json(&p, |_| None::<()>).unwrap(), None);
        assert_eq!(std::fs::read(p.as_path()).unwrap(), before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parsed_is_reused_until_the_file_changes() {
        let dir = scratch("parsed");
        let p = dir.join("list.csv");
        std::fs::write(&p, "a").unwrap();
        let cache: Parsed<String> = Parsed::default();
        let loads = std::cell::Cell::new(0);
        let load = |path: &Path| {
            loads.set(loads.get() + 1);
            std::fs::read_to_string(path).unwrap()
        };
        assert_eq!(*cache.get(&p, load), "a");
        assert_eq!(*cache.get(&p, load), "a");
        assert_eq!(loads.get(), 1);
        std::fs::write(&p, "abc").unwrap(); // a different size: read again
        assert_eq!(*cache.get(&p, load), "abc");
        assert_eq!(loads.get(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
