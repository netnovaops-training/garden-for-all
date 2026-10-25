//! Small functions that behave exactly like the PHP ones the site was
//! written with.
//!
//! The port has to produce the same page, so the places where PHP's
//! behaviour is specific (how it escapes, how it encodes JSON, how it turns
//! newlines into line breaks) are copied here rather than approximated with
//! whatever the Rust ecosystem happens to do.
//! HOW TO READ THIS FILE. Each function is named after the PHP function it copies and
//! is short and independent. Read the `///` line above each one: it says which PHP
//! function it imitates. The tests at the bottom show example input and output.
//! Nearly every function takes a `&str` (a borrowed piece of text: it can be read but
//! not changed or kept) and returns a `String` (text that the function owns).
//!

// `use` brings a name into scope so the code can write `Path` instead of `std::path::Path`.
use std::path::Path;

/// `htmlspecialchars($s, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8')`, which is
/// what the site's `he()` helper is. Single quotes become `&#039;`.
// `pub fn` means other files may call it. `s: &str` is the text to escape. `-> String` is
// the result. `String::with_capacity` reserves room first so the string does not regrow.
pub fn he(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    // Walk through the text one character at a time and swap the five special ones.
    for c in s.chars() {
        // `match` compares the character with each pattern and runs the first that fits.
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#039;"),
            // Every other character is copied as it is (`_` means "anything else").
            _ => out.push(c),
        }
    }
    out
}

/// `htmlspecialchars_decode($s)` with PHP's default flags: undoes exactly
/// the five entities `he()` produces, and nothing else.
pub fn htmlspecialchars_decode(s: &str) -> String {
    // The order matters: the ampersand entity is undone LAST, so the text `&amp;lt;` turns
    // into `&lt;` and not into `<`, exactly as PHP does.
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&amp;", "&")
}

/// `nl2br($s)`: a `<br />` before every newline, newline kept.
pub fn nl2br(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    // `peekable` lets the loop look at the next character without taking it, so a
    // two-character line ending (CR LF or LF CR) is treated as one break.
    let mut chars = s.chars().peekable();
    // `while let Some(c) = ...` repeats while there is another character.
    while let Some(c) = chars.next() {
        match c {
            // A lone carriage return: add the tag, keep the character, and swallow a following LF.
            '\r' => {
                out.push_str("<br />\r");
                if chars.peek() == Some(&'\n') {
                    out.push('\n');
                    chars.next();
                }
            }
            // A newline: same idea, and a following CR is swallowed.
            '\n' => {
                out.push_str("<br />\n");
                if chars.peek() == Some(&'\r') {
                    out.push('\r');
                    chars.next();
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// `urlencode($s)`: spaces become `+`, and only `A-Za-z0-9-_.` stay as they are.
pub fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    // Work on bytes, not characters, because PHP encodes each byte of a UTF-8 character.
    for b in s.bytes() {
        match b {
            // Letters, digits and - _ . are kept as they are.
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(b as char),
            // A space becomes a plus sign.
            b' ' => out.push('+'),
            // Every other byte becomes % and two capital hex digits (an apostrophe gives %27).
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `json_encode($string)` with no flags: `/` is written `\/`, and anything
/// outside ASCII is written `\uXXXX` (as surrogate pairs above U+FFFF).
pub fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    // Go through each character and write its JSON form, with PHP's escapes.
    out.push('"');
    for c in s.chars() {
        match c {
            // Backslash and quote must be escaped in any JSON.
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            // PHP also escapes the forward slash (a slash becomes backslash + slash).
            '/' => out.push_str("\\/"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Other control characters (below 0x20) become backslash-u plus four hex digits.
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            // Plain ASCII is written as it is.
            c if c.is_ascii() => out.push(c),
            // Anything else becomes backslash-u plus four hex digits. Characters above
            // U+FFFF do not fit in 16 bits, so they are written as two 16-bit pieces
            // (a surrogate pair), as PHP does.
            c => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
        }
    }
    out.push('"');
    out
}

/// `json_encode($value)` with PHP's default flags: `/` escaped, non-ASCII
/// as `\uXXXX`, whole floats written with `.0`, object keys in order.
pub fn json_encode(v: &serde_json::Value) -> String {
    // `serde_json::Value` is any JSON value. This function writes it out as PHP would.
    use serde_json::Value;
    // Each kind of value is handled in turn.
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        // PHP writes a float that is a whole number with `.0` (for example `3.0`).
        // Integers stay as integers. 1e15 is the limit below which the number prints in full.
        Value::Number(n) => {
            if let Some(f) = n.as_f64().filter(|_| !n.is_i64() && !n.is_u64()) {
                if f.fract() == 0.0 && f.abs() < 1e15 {
                    format!("{f:.1}")
                } else {
                    format!("{f}")
                }
            } else {
                n.to_string()
            }
        }
        Value::String(s) => json_string(s),
        // Arrays: encode each item, join with commas, wrap in square brackets.
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(json_encode).collect::<Vec<_>>().join(",")
        ),
        // Objects: key and value pairs, in the order they were stored, wrapped in braces.
        Value::Object(m) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| format!("{}:{}", json_string(k), json_encode(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

/// A float printed the way PHP prints it in a string: no trailing `.0`.
pub fn php_num(v: f64) -> String {
    // A whole number prints without a decimal part. Otherwise trailing zeros and a
    // trailing dot are removed.
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// `json_encode($list_of_strings)`.
pub fn json_string_list(items: &[String]) -> String {
    // `.map(...).collect()` turns each string into its JSON form and gathers them in a list.
    let parts: Vec<String> = items.iter().map(|s| json_string(s)).collect();
    format!("[{}]", parts.join(","))
}

/// `filemtime()`, in seconds, as the `?v=` cache-buster the pages append to
/// their stylesheet and scripts. 1 if the file is missing, like the PHP.
pub fn filemtime(root: &Path, rel: &str) -> u64 {
    // Ask the file system for the file's last-modified time and convert it to seconds
    // since 1970. Each `.and_then` / `.ok()` step passes a failure along as `None`;
    // `.unwrap_or(1)` supplies 1 if any step failed. `Option` is Rust's "a value, or nothing".
    std::fs::metadata(root.join(rel))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(1)
}

/// `preg_quote($s, '/')`: backslash-escape every character that means
/// something in a regular expression. Used to build the "attracts" match.
pub fn preg_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    // Put a backslash before any character that has a meaning in a regular expression.
    for c in s.chars() {
        if ".\\+*?[^]$(){}=!<>|:-#/".contains(c) {
            out.push('\\');
        }
        // PHP writes a null byte as backslash and 000.
        if c == '\0' {
            out.push_str("\\000");
        } else {
            out.push(c);
        }
    }
    out
}

/// `(int)$value`: leading digits (with an optional sign), anything else is 0.
pub fn php_int(s: &str) -> i64 {
    // Skip leading spaces, then read an optional + or - sign.
    let t = s.trim_start();
    let (sign, rest) = match t.strip_prefix('-') {
        Some(r) => (-1, r),
        None => (1, t.strip_prefix('+').unwrap_or(t)),
    };
    // Keep the run of digits at the front; a number that is too big or empty gives 0.
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    sign * digits.parse::<i64>().unwrap_or(0)
}

/// `(float)$value`: the leading number, else 0.
pub fn php_float(s: &str) -> f64 {
    // Scan from the start while the characters could belong to a number (digits, a dot,
    // a sign in first place, or the letter e for exponents), remembering where it ends.
    let t = s.trim();
    let mut end = 0;
    for (i, c) in t.char_indices() {
        if c.is_ascii_digit()
            || c == '.'
            || (i == 0 && (c == '-' || c == '+'))
            || c == 'e'
            || c == 'E'
        {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    // Parse that front part. If it is not a valid number, PHP gives 0, so this does too.
    t[..end].parse::<f64>().unwrap_or(0.0)
}

/// `round($v, $places)` the way PHP does it: half away from zero.
pub fn php_round(v: f64, places: i32) -> f64 {
    // Shift the decimal point by `places`, round, then shift back.
    let m = 10f64.powi(places);
    let x = v * m;
    let r = if x >= 0.0 {
        (x + 0.5).floor()
    } else {
        (x - 0.5).ceil()
    };
    // Values like 2.675 * 100 = 267.49999 in binary; PHP pre-rounds to the
    // nearest representable value first, which this approximates.
    let r2 = (x * 1e9).round() / 1e9;
    let r2 = if r2 >= 0.0 {
        (r2 + 0.5).floor()
    } else {
        (r2 - 0.5).ceil()
    };
    (if (r - r2).abs() > 0.0 { r2 } else { r }) / m
}

/// `date('c')`: ISO-8601 with the local offset.
pub fn date_c() -> String {
    // `common::now_local()` is the current time with the server's offset from UTC.
    common::now_local()
        .format(&time::macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
        ))
        .unwrap_or_default()
}

/// `date('Y-m-d')` today, local.
pub fn date_ymd() -> String {
    common::now_local()
        .format(&time::macros::format_description!("[year]-[month]-[day]"))
        .unwrap_or_default()
}

/// The search-term clean-up every search in the PHP does:
/// `preg_replace("/[^a-zA-Z0-9\s]/", " ")`, collapse whitespace, trim, and
/// fall back to the raw term if nothing is left (`?:` also treats "0" as
/// nothing, which is kept).
pub fn search_norm(raw: &str) -> String {
    // Replace every character that is not a letter, digit or space with a space.
    let spaced: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c.is_ascii_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();
    // Split on whitespace and rejoin with single spaces; this collapses runs and trims.
    let collapsed = spaced
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    // If nothing useful is left, use the original term (PHP's `?:` treats "0" as empty).
    if collapsed.is_empty() || collapsed == "0" {
        raw.to_string()
    } else {
        collapsed
    }
}

// Tests: only compiled with `cargo test`. Each checks one function against PHP's output.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preg_quote_escapes_regex_characters() {
        assert_eq!(preg_quote("a.b*c"), "a\\.b\\*c");
        assert_eq!(preg_quote("Birds"), "Birds");
        assert_eq!(preg_quote("x/y-z"), "x\\/y\\-z");
    }

    #[test]
    fn search_norm_matches_php() {
        assert_eq!(search_norm("Lil' Kim"), "Lil Kim");
        assert_eq!(search_norm("!!!"), "!!!");
        assert_eq!(search_norm("  a   b "), "a b");
    }

    #[test]
    fn he_matches_php_quote_style() {
        assert_eq!(
            he("Jun's \"garden\" <b>&"),
            "Jun&#039;s &quot;garden&quot; &lt;b&gt;&amp;"
        );
    }

    #[test]
    fn decode_undoes_he() {
        let s = "a & b <c> \"d\" 'e'";
        assert_eq!(htmlspecialchars_decode(&he(s)), s);
    }

    #[test]
    fn nl2br_keeps_the_newline() {
        assert_eq!(nl2br("a\nb\r\nc"), "a<br />\nb<br />\r\nc");
    }

    #[test]
    fn urlencode_matches_php() {
        assert_eq!(
            urlencode("Check out Jun's Garden: x/y"),
            "Check+out+Jun%27s+Garden%3A+x%2Fy"
        );
        assert_eq!(
            urlencode("https://junsgarden.zapto.org/"),
            "https%3A%2F%2Fjunsgarden.zapto.org%2F"
        );
    }

    #[test]
    fn json_matches_php_default_flags() {
        assert_eq!(json_string("images/a.webp"), "\"images\\/a.webp\"");
        assert_eq!(
            json_string("café — 🌿"),
            "\"caf\\u00e9 \\u2014 \\ud83c\\udf3f\""
        );
    }
}
