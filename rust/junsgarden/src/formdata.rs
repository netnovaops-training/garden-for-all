//! Request bodies the way PHP saw them: `$_POST` and `$_FILES`.
//!
//! The forms on this site post two ways: ordinary `application/x-www-form-
//! urlencoded` (the delete buttons, the guestbook, Logout) and `multipart/
//! form-data` (anything with a photo attached). Both end up here as one
//! map of fields plus a list of files, so a handler does not care which.
//!
//! HOW TO READ THIS FILE. `FormData` is the result. `parse()` fills it in from a request
//! body. Handlers then ask it for fields with `get`, `str` and `has`, or for uploaded
//! files with `files_for`.

// `use` brings names into scope. `HashMap` is a lookup table from a key to a value.
use std::collections::HashMap;

use axum::body::Bytes;
use axum::http::HeaderMap;
use serde_json::Value;

/// One uploaded file: the form field it came in, its original name, bytes.
/// `pub` on a struct and on its fields means other files can read them.
pub struct Upload {
    pub field: String,
    pub filename: String,
    pub bytes: Vec<u8>,
}

/// Everything a form request carried. `#[derive(Default)]` makes Rust write a
/// "start empty" constructor for us: no fields, no files, no JSON.
#[derive(Default)]
pub struct FormData {
    pub fields: HashMap<String, String>,
    pub files: Vec<Upload>,
    /// `php://input` decoded as JSON, when the request was JSON.
    pub json: Option<Value>,
}

/// Lookup helpers on `FormData`. `&self` means each one only reads the data.
impl FormData {
    /// Returns the field's text if it was sent. `Option<&str>` means "text, or nothing";
    /// the text is borrowed from the form, not copied.
    pub fn get(&self, k: &str) -> Option<&str> {
        self.fields.get(k).map(String::as_str)
    }
    /// Like `get`, but gives an empty string when the field is missing. This is how PHP
    /// treats a missing `$_POST` value when it is only printed or compared.
    pub fn str(&self, k: &str) -> &str {
        self.get(k).unwrap_or("")
    }
    /// True when the field was sent at all, even if it was empty (PHP's `isset`).
    pub fn has(&self, k: &str) -> bool {
        self.fields.contains_key(k)
    }
    /// Files from one field (`name="blog_images[]"` arrives as `blog_images[]`).
    pub fn files_for(&self, field: &str) -> Vec<&Upload> {
        self.files
            .iter()
            .filter(|u| {
                u.field == field || u.field.trim_end_matches("[]") == field.trim_end_matches("[]")
            })
            .collect()
    }
}

/// Parse a request body by its content type.
///
/// Takes the request headers (to read Content-Type) and the body bytes. It never fails: a
/// body it cannot read gives an empty `FormData`, as PHP leaves `$_POST` empty.
/// `async` because reading the multipart parts waits for the stream.
pub async fn parse(headers: &HeaderMap, body: Bytes) -> FormData {
    // Read the Content-Type header as text; use an empty string if it is missing or not text.
    let ct = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    // Start with an empty result and fill it in below.
    let mut out = FormData::default();

    // Multipart: a form that may carry files. The boundary text separates the parts.
    if ct.starts_with("multipart/form-data") {
        if let Ok(boundary) = multer::parse_boundary(&ct) {
            // `multer` reads a stream, so the body is wrapped as a stream with one item.
            let stream =
                futures_util::stream::once(async move { Ok::<Bytes, std::io::Error>(body) });
            let mut mp = multer::Multipart::new(stream, boundary);
            // Take the parts one at a time until there are none left.
            while let Ok(Some(field)) = mp.next_field().await {
                // The form field's name, and the file name if this part is a file.
                let name = field.name().unwrap_or("").to_string();
                let filename = field.file_name().map(str::to_string);
                // Read the part's bytes; on a read error, stop reading.
                let data = match field.bytes().await {
                    Ok(b) => b,
                    Err(_) => break,
                };
                // A part with a file name is a file upload; a part without one is plain text.
                match filename {
                    // An empty file input still sends a part with no name.
                    Some(f) if !f.is_empty() => out.files.push(Upload {
                        field: name,
                        filename: f,
                        bytes: data.to_vec(),
                    }),
                    Some(_) => {}
                    None => {
                        let text = String::from_utf8_lossy(&data).to_string();
                        // A multi-select (`flower_bed[]`) sends one part per
                        // choice; PHP joined those with ", " in save_plant.
                        if name.ends_with("[]") {
                            out.fields
                                .entry(name)
                                .and_modify(|v| {
                                    v.push_str(", ");
                                    v.push_str(&text);
                                })
                                .or_insert(text);
                        } else {
                            out.fields.insert(name, text);
                        }
                    }
                }
            }
        }
    // JSON body: store the decoded value. A bad JSON body leaves `json` as `None`.
    } else if ct.starts_with("application/json") {
        out.json = serde_json::from_slice(&body).ok();
    // Ordinary form post (key=value&key=value). The `if let` runs only if decoding worked.
    } else if !body.is_empty() {
        if let Ok(m) = serde_urlencoded::from_bytes::<Vec<(String, String)>>(&body) {
            for (k, v) in m {
                out.fields.insert(k, v);
            }
        }
    }
    out
}
