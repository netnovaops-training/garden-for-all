//! The photo work the PHP did with GD: save an upload as WebP, rotate a
//! photo in place, make a small thumbnail.
//!
//! Quality numbers are the PHP's own (82 for blog photos, 87 for plant
//! photos, 80 for thumbnails, 90 when rotating) so the files come out the
//! same size and look.
//!
//! HOW TO READ THIS FILE. The small private helpers come first (`decode`, `encode_webp`,
//! `on_white`). The `pub` functions below use them. Most return `Option`, which is
//! `Some(result)` on success or `None` on failure, so callers must handle failure.
//! The `?` after a call means: if this gave `None`, stop and return `None` from here.

// `use` brings names into scope so they can be written short.
use std::path::Path;

// The `image` library reads and edits pictures.
use image::{DynamicImage, ImageFormat, RgbaImage};

/// Reads image bytes (JPEG, PNG, GIF, WebP...) into a picture in memory. `&[u8]` is a
/// borrowed run of bytes. Returns `None` if the bytes are not a readable image.
fn decode(bytes: &[u8]) -> Option<DynamicImage> {
    image::load_from_memory(bytes).ok()
}

/// Lossy WebP at the given quality.
// A quality number from 0 to 100; higher means a bigger file and a better picture.
fn encode_webp(img: &DynamicImage, quality: f32) -> Option<Vec<u8>> {
    // WebP needs red/green/blue/alpha pixels, so convert first.
    let rgba = img.to_rgba8();
    // Build the encoder from the raw pixel bytes and the picture's width and height.
    let enc = webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
    // Encode at the given quality and copy the result into a `Vec<u8>` (a growable list of bytes).
    Some(enc.encode(quality).to_vec())
}

/// Flatten transparency onto white, as the PHP did for PNG/GIF.
fn on_white(img: DynamicImage) -> DynamicImage {
    // Make sure the picture has an alpha (transparency) channel.
    let rgba = img.to_rgba8();
    // `(w, h)` receives the width and height as a pair.
    let (w, h) = rgba.dimensions();
    // A white canvas the same size...
    let mut out = RgbaImage::from_pixel(w, h, image::Rgba([255, 255, 255, 255]));
    // ...with the picture drawn over it, so see-through areas show white.
    image::imageops::overlay(&mut out, &rgba, 0, 0);
    DynamicImage::ImageRgba8(out)
}

/// A random 32-character hex file name. It hashes 24 random bytes with MD5. The hash is
/// only used to make a name that will not repeat, not for security.
fn random_md5_name() -> String {
    let bytes: [u8; 24] = rand::random();
    format!("{:x}", md5::compute(bytes))
}

/// `saveAsWebP()` from index.php: any image becomes `<md5>.webp` at quality
/// 82 in `dir`; returns the new file name.
pub fn save_as_webp(bytes: &[u8], dir: &Path, quality: f32) -> Option<String> {
    // Read the picture, flatten transparency, encode, and make a name; any step can give `None`.
    let img = on_white(decode(bytes)?);
    let data = encode_webp(&img, quality)?;
    let name = format!("{}.webp", random_md5_name());
    // Make the folder if it is missing, then write the file.
    std::fs::create_dir_all(dir).ok()?;
    std::fs::write(dir.join(&name), data).ok()?;
    Some(name)
}

/// Any image as WebP bytes at `quality`, transparency flattened onto white
/// (the plant admin's uploads: Pillow `convert('RGB')` + WebP 87). A WebP
/// upload is the caller's to keep as is.
pub fn to_webp(bytes: &[u8], quality: f32) -> Option<Vec<u8>> {
    encode_webp(&on_white(decode(bytes)?), quality)
}

/// An already-decoded image as WebP bytes (transparency onto white).
pub fn to_webp_image(img: &DynamicImage, quality: f32) -> Option<Vec<u8>> {
    encode_webp(&on_white(img.clone()), quality)
}

/// Keeps only letters and digits of `s`, so it is safe in a file name. If nothing is
/// left, returns `fallback`.
fn clean_name(s: &str, fallback: &str) -> String {
    let c: String = s.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if c.is_empty() { fallback.to_string() } else { c }
}

/// `process_and_save_image()` from functions.php: a plant photo saved at
/// full size as `<Plant>_<Family>_<NN>.webp` (quality 87); a WebP upload is
/// kept byte for byte.
pub fn process_and_save_image(bytes: &[u8], original_name: &str, dir: &Path, plant: &str, family: &str) -> Option<String> {
    // Empty names become "Unknown"; then everything but letters and digits is removed.
    let plant = clean_name(if plant.is_empty() { "Unknown" } else { plant }, "Plant");
    let family = clean_name(if family.is_empty() { "Unknown" } else { family }, "Family");
    std::fs::create_dir_all(dir).ok()?;

    // Existing files for this plant look like `Plant_Family_NN.webp`; count them.
    let prefix = format!("{plant}_{family}_");
    // `seq` is the highest number to start from; the loop below counts up from it.
    let mut seq = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().to_string();
                    n.starts_with(&prefix) && n.ends_with(".webp")
                })
                .count()
        })
        .unwrap_or(0);
    // Find the first number whose file name is not taken yet. `loop` repeats until `break`.
    let target = loop {
        seq += 1;
        // `{seq:02}` pads the number to two digits (01, 02...).
        let name = format!("{plant}_{family}_{seq:02}.webp");
        if !dir.join(&name).exists() {
            break name;
        }
    };

    // The original file name's extension, lower case, or empty if there is none.
    let ext = original_name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    // WebP is kept as it is; JPEG and PNG are converted; any other type is refused (`None`).
    let data = if ext == "webp" {
        bytes.to_vec()
    } else if ext == "jpg" || ext == "jpeg" || ext == "png" {
        encode_webp(&on_white(decode(bytes)?), 87.0)?
    } else {
        return None;
    };
    std::fs::write(dir.join(&target), data).ok()?;
    Some(target)
}

/// `rotate_image` in catalog.php: turn the file a quarter turn clockwise and
/// write it back in its own format.
pub fn rotate_clockwise_in_place(path: &Path) -> bool {
    // `let Ok(..) = .. else { return false }` unwraps a good result or leaves the function.
    let Ok(bytes) = std::fs::read(path) else { return false };
    let Some(img) = decode(&bytes) else { return false };
    // GD's imagerotate(-90) is clockwise.
    // A quarter turn clockwise.
    let rotated = img.rotate90();
    // The file's extension decides the format it is written back in.
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let out: Option<Vec<u8>> = match ext.as_str() {
        // JPEG at quality 90.
        "jpg" | "jpeg" => {
            let mut buf = Vec::new();
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 90);
            enc.encode_image(&rotated.to_rgb8()).ok().map(|_| buf)
        }
        // PNG has no quality setting.
        "png" => {
            let mut buf = std::io::Cursor::new(Vec::new());
            rotated.write_to(&mut buf, ImageFormat::Png).ok().map(|_| buf.into_inner())
        }
        // WebP at quality 90.
        "webp" => encode_webp(&rotated, 90.0),
        // Any other extension: do nothing.
        _ => None,
    };
    // Write the new bytes over the old file. Returns true only if the write worked.
    match out {
        Some(data) => std::fs::write(path, data).is_ok(),
        None => false,
    }
}

/// `app_api.php?action=thumb`: scale to at most `max_w` wide, WebP quality 80.
pub fn thumbnail_webp(bytes: &[u8], max_w: u32) -> Option<Vec<u8>> {
    let img = decode(bytes)?;
    // Original width and height.
    let (sw, sh) = (img.width(), img.height());
    // New width: at most `max_w`, never larger than the original, and at least 1.
    let nw = max_w.min(sw).max(1);
    // New height keeps the same shape: width times the original height/width ratio.
    let nh = ((nw as f64) * (sh as f64 / sw.max(1) as f64)).round().max(1.0) as u32;
    // `Triangle` is a smoothing method for shrinking.
    let small = img.resize_exact(nw, nh, image::imageops::FilterType::Triangle);
    encode_webp(&small, 80.0)
}

// Tests: only compiled with `cargo test`. They use a temporary scratch folder.
#[cfg(test)]
mod tests {
    use super::*;

    /// Makes a small solid-colour PNG in memory for the tests to use.
    fn png_bytes(w: u32, h: u32) -> Vec<u8> {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, image::Rgba([10, 200, 30, 255])));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Png).unwrap();
        buf.into_inner()
    }

    #[test]
    fn saves_webp_and_names_files_like_php() {
        let dir = std::env::temp_dir().join(format!("jg-img-{}", std::process::id())); // nosemgrep: rust.lang.security.temp-dir.temp-dir -- unit-test scratch folder, never runs on the server
        let _ = std::fs::remove_dir_all(&dir);
        let name = save_as_webp(&png_bytes(8, 6), &dir, 82.0).unwrap();
        assert!(name.ends_with(".webp") && name.len() == 37);
        let a = process_and_save_image(&png_bytes(8, 6), "x.png", &dir, "Lil' Kim", "Hibiscus").unwrap();
        let b = process_and_save_image(&png_bytes(8, 6), "x.png", &dir, "Lil' Kim", "Hibiscus").unwrap();
        assert_eq!(a, "LilKim_Hibiscus_01.webp");
        assert_eq!(b, "LilKim_Hibiscus_02.webp");
        assert!(process_and_save_image(b"notanimage", "x.png", &dir, "a", "b").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn thumbnail_keeps_aspect_and_rotation_swaps_sides() {
        let t = thumbnail_webp(&png_bytes(40, 20), 10).unwrap();
        let img = decode(&t).unwrap();
        assert_eq!((img.width(), img.height()), (10, 5));

        let dir = std::env::temp_dir().join(format!("jg-rot-{}", std::process::id())); // nosemgrep: rust.lang.security.temp-dir.temp-dir -- unit-test scratch folder, never runs on the server
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("a.png");
        std::fs::write(&p, png_bytes(40, 20)).unwrap();
        assert!(rotate_clockwise_in_place(&p));
        let r = decode(&std::fs::read(&p).unwrap()).unwrap();
        assert_eq!((r.width(), r.height()), (20, 40));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
