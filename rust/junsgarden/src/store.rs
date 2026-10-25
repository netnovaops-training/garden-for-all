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
use std::path::Path;

use serde::Serialize;
use serde_json::{Map, Value};

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
pub fn read_text(path: &Path) -> std::io::Result<String> {
    match std::fs::read_to_string(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(String::new()),
        other => other,
    }
}

/// A JSON file's top-level object. Missing file: an empty object (the first comment ever
/// must work). A file that is there but not a JSON object: `Damaged`.
pub fn read_json(path: &Path) -> Result<Map<String, Value>, StoreError> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Map::new()),
        Err(e) => return Err(StoreError::Io(e)),
    };
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(m)) => Ok(m),
        Ok(_) => Err(StoreError::Damaged(format!(
            "{} is not a JSON object",
            path.display()
        ))),
        Err(e) => Err(StoreError::Damaged(format!("{}: {e}", path.display()))),
    }
}

/// Replace a file's contents in one step: write `path.tmp`, flush it to disk, rename it
/// over `path`. A rename within one folder is atomic, so readers see the old file or the
/// new one, never a mix.
///
/// A folder the site may not create files in (only change the existing ones) falls back to
/// writing the file in place, as the PHP always did.
pub fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
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
    path: &Path,
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
        assert_eq!(read_text(&dir.join("none.txt")).unwrap(), "");
        assert!(read_json(&dir.join("none.json")).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_damaged_file_is_never_overwritten() {
        let dir = scratch("damaged");
        let p = dir.join("c.json");
        std::fs::write(&p, b"{\"276\": [half a file").unwrap();
        let r = update_json(&p, |m| {
            m.insert("1".into(), json!([]));
            Some(())
        });
        assert!(matches!(r, Err(StoreError::Damaged(_))));
        assert_eq!(std::fs::read(&p).unwrap(), b"{\"276\": [half a file");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_writes_four_space_json_and_leaves_no_temp_file() {
        let dir = scratch("update");
        let p = dir.join("c.json");
        update_json(&p, |m| {
            m.insert("k".into(), json!([{"a": 1}]));
            Some(())
        })
        .unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("\n    \"k\": ["), "{s}");
        assert!(!dir.join("c.json.tmp").exists());
        // "nothing to save" leaves the file exactly as it was
        let before = std::fs::read(&p).unwrap();
        assert_eq!(update_json(&p, |_| None::<()>).unwrap(), None);
        assert_eq!(std::fs::read(&p).unwrap(), before);
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
