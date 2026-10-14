//! A database row as a JSON object, the way PHP's `PDO::FETCH_ASSOC` +
//! `json_encode` gives it: integers stay integers, decimals and dates are
//! strings, NULL is null. The plant search endpoint returns whole rows
//! (`SELECT *`, ~88 columns), so it needs this rather than a fixed struct.
//!
//! HOW TO READ THIS FILE. There is one function, `row_to_json`. It looks at the database
//! type of each column and picks how to turn that column's value into JSON.

// `use` brings names into scope. `json!` builds a JSON value. `Value` is any JSON value.
use serde_json::{json, Value};
use sqlx::mysql::MySqlRow;
use sqlx::{Column, Row, TypeInfo};

/// Turns one database row into a JSON object (column name -> value).
///
/// Takes `&MySqlRow` (a borrowed row: this function only reads it). Gives back a JSON
/// `Value`. It exists because the plant search returns whole rows with about 88 columns,
/// and the PHP site's output for them has to be matched type by type.
pub fn row_to_json(row: &MySqlRow) -> Value {
    // The object being built: column name -> value, in column order.
    let mut map = serde_json::Map::new();
    // Go through the columns by position `i`; `enumerate` gives the position with each column.
    for (i, col) in row.columns().iter().enumerate() {
        // The database's name for the column type, in capitals, e.g. INT or BIGINT UNSIGNED.
        let ty = col.type_info().name().to_ascii_uppercase();
        // Unsigned integers may be larger than a signed 64-bit number, so they are read as u64.
        let unsigned = ty.contains("UNSIGNED");
        let base = ty.replace(" UNSIGNED", "");
        // Choose how to read the value from the column type.
        let value: Value = match base.as_str() {
            // Whole numbers. `try_get_unchecked::<Option<i64>>` reads "a number or NULL"
            // without sqlx's strict type check. `.ok().flatten()` turns any failure or NULL into
            // `None`, and `.unwrap_or(Value::Null)` turns `None` into JSON null.
            "BOOLEAN" | "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" | "YEAR" => {
                if unsigned {
                    row.try_get_unchecked::<Option<u64>, _>(i).ok().flatten().map(|v| json!(v))
                } else {
                    row.try_get_unchecked::<Option<i64>, _>(i).ok().flatten().map(|v| json!(v))
                }
                .unwrap_or(Value::Null)
            }
            // Decimal numbers with a fraction part (FLOAT is 32-bit, DOUBLE is 64-bit).
            "FLOAT" => row
                .try_get_unchecked::<Option<f32>, _>(i)
                .ok()
                .flatten()
                .map(|v| json!(v))
                .unwrap_or(Value::Null),
            "DOUBLE" => row
                .try_get_unchecked::<Option<f64>, _>(i)
                .ok()
                .flatten()
                .map(|v| json!(v))
                .unwrap_or(Value::Null),
            // A date is written as text, year-month-day, as PHP gives it.
            "DATE" => row
                .try_get_unchecked::<Option<chrono::NaiveDate>, _>(i)
                .ok()
                .flatten()
                .map(|d| json!(d.format("%Y-%m-%d").to_string()))
                .unwrap_or(Value::Null),
            // A date and time is written as text, like 2026-10-12 08:30:00.
            "DATETIME" | "TIMESTAMP" => row
                .try_get_unchecked::<Option<chrono::NaiveDateTime>, _>(i)
                .ok()
                .flatten()
                .map(|d| json!(d.format("%Y-%m-%d %H:%M:%S").to_string()))
                .unwrap_or(Value::Null),
            // Binary columns (the QR-tag and barcode PNGs) are not JSON-safe.
            // The PHP pages that return plant rows (catalog.php, app_api.php)
            // blank them for exactly that reason; plant_listing.php forgot
            // to, and its search has returned an empty body ever since a plant
            // got a QR code. Nothing reads them from this response -- the
            // pages fetch the image from plant_actions.php instead.
            "TINYBLOB" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" => Value::Null,
            // DECIMAL arrives as its text form; CHAR/TEXT/ENUM/SET/JSON are
            // already text. try_get_unchecked skips sqlx's strict type match.
            // Bytes that are not valid UTF-8 would make PHP's json_encode
            // fail; catalog.php's json_out() swaps them for "?", so this does.
            _ => row
                .try_get_unchecked::<Option<Vec<u8>>, _>(i)
                .ok()
                .flatten()
                .map(|b| Value::String(String::from_utf8_lossy(&b).replace('\u{FFFD}', "?")))
                .unwrap_or(Value::Null),
        };
        // Store this column's value under its name.
        map.insert(col.name().to_string(), value);
    }
    Value::Object(map)
}
