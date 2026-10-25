//! `data.php` and `widget_data.php` — the Data page.
//!
//! `data.php` does several jobs depending on what is added to the address,
//! checked in the PHP's order: `?ai_predict`, `?discover`, `?warm_ai`,
//! `?ai_fuse` (which is `?cross_tab` plus the model's reading),
//! `?weather_forecast`, `?weather_data`, `?mock_iot`, `?cross_tab`, and
//! otherwise the page. `widget_data.php` is the one-query aggregation the
//! plant widgets load, cached for five minutes.
//!
//! The weather numbers come from the `weather` table in MariaDB (the
//! collector writes it). The PHP's `?weather_data` still carried InfluxDB
//! SQL and returned empty series; this one runs the same two-step
//! day-then-month merge in MariaDB SQL (see DEVIATIONS.md).
//!
//! HOW TO READ THIS FILE (for Jun). Three parts: the endpoints the page's
//! scripts call (`weather_data`, `cross_tab`, `mock_iot`, the AI ones), the
//! numbers behind the 24 plant cards (`build_widget_data`: one `SELECT *`,
//! then one block per card, numbered as in the PHP), and `render_page`,
//! which asks `widgets.rs` for the nine server-drawn cards and fills
//! `templates/data.html`. `data_get` at the bottom is the switchboard.
//! To add a plant card: add a block in `build_widget_data` that inserts a
//! new key, then teach `js/widget-loader.js` to draw it.

// Imports. `use` brings a name from another file or a library into this file so it can be written
// short.
use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Query, State},
    http::{header, HeaderValue},
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde_json::{json, Map, Value};

use crate::brain;
use crate::layout::{self, Page};
use crate::normalise::{garden_clean, garden_normalise, year_in_range};
use crate::pages::widgets;
use crate::php::{filemtime, php_float, php_int, php_round};
use crate::rowjson::row_to_json;
use crate::session::Visitor;
use crate::AppState;

/// Master switch for the AI features on this page. When false, `?ai_predict` answers "AI offline"
/// without calling the model.
///
/// Rust idea: a `const` is a fixed value known at build time; `bool` is true or false.
const AI_ENABLED: bool = true;

/// Send a JSON answer to the browser.
///
/// Takes any JSON value (`Value` is serde_json's type for "any JSON") and gives back an HTTP
/// `Response` with the JSON content type. If the value cannot be turned into text it sends `{}`.
/// Every `?something` call on this page ends here.
/// The text an answer shows for a database failure: never the database's own message (it
/// names tables and columns); that goes to the log.
fn hide(e: impl std::fmt::Display) -> &'static str {
    tracing::error!("data: {e}");
    "The garden data could not be read just now."
}

fn json_response(body: Value) -> Response {
    (
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&body).unwrap_or_else(|_| "{}".into()),
    )
        .into_response()
}

/// Add a `Cache-Control` header to an answer, telling the browser how long it may reuse it. `mut
/// res` means the answer is changed in place; `'static` means the text lives for the whole run of
/// the program.
fn with_cache(mut res: Response, value: &'static str) -> Response {
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    res
}

// --- the weather table ------------------------------------------------------

/// `influxdb_available()`: the weather table is reachable and has data.
///
/// Takes the app data (`state`, which holds the database pool). Gives back true if a one-row test
/// query on the `weather` table works. Rust idea: `async fn` can pause while waiting for the
/// database; callers write `.await`.
pub async fn weather_available(state: &AppState) -> bool {
    sqlx::query("SELECT 1 FROM `weather` LIMIT 1").fetch_optional(&state.db).await.is_ok()
}

/// `influxdb_query()`: rows as JSON objects, empty on any error.
///
/// Takes the SQL text to run. Gives a list of rows, each row a JSON map of column name to value. On
/// a database error it logs the first 200 characters of the SQL and gives an empty list, so the
/// charts show nothing instead of crashing. `Vec` is a growable list; `Map` is a JSON object.
pub async fn weather_query(state: &AppState, sql: &str) -> Vec<Map<String, Value>> {
    match sqlx::query(sql).fetch_all(&state.db).await {
        Ok(rows) => rows
            .iter()
            .filter_map(|r| if let Value::Object(m) = row_to_json(r) { Some(m) } else { None })
            .collect(),
        Err(e) => {
            tracing::error!("weather query failed: {e} — SQL: {}", &sql[..sql.len().min(200)]);
            Vec::new()
        }
    }
}

/// A number out of a weather row, however MariaDB typed it.
///
/// Takes a row and a column name `k`. Gives `Some(number)` or `None` ("maybe nothing"). The `?`
/// after `m.get(k)` means: if the column is missing, return `None` right away. `match` then checks
/// whether the value is a JSON number or a text number.
pub fn num(m: &Map<String, Value>, k: &str) -> Option<f64> {
    match m.get(k)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => Some(php_float(s)),
        _ => None,
    }
}

// --- ?weather_data -----------------------------------------------------------

/// One weather chart the page can show (`?weather_data&metric=...`).
///
/// `om` is the column name for the Open-Meteo reading, `noaa` is the matching NOAA column used to
/// fill days that Open-Meteo is missing (`None` means there is no NOAA column), `sum` is true for
/// totals such as rain and false for averages, and `unit` is the label shown on the chart. Rust
/// idea: a `struct` is a bundle of named fields.
struct Metric {
    om: &'static str,
    noaa: Option<&'static str>,
    sum: bool,
    unit: &'static str,
}

/// Look up a metric by the name sent in the address. Gives `Some(Metric)` for a known name and
/// `None` for anything else, so an unknown name can never reach the SQL.
fn metric(name: &str) -> Option<Metric> {
    Some(match name {
        "temperature" => Metric { om: "temperature", noaa: Some("noaa_temp_avg"), sum: false, unit: "°F" },
        "humidity" => Metric { om: "humidity", noaa: Some("noaa_humidity_avg"), sum: false, unit: "%" },
        "precipitation" => Metric { om: "precipitation", noaa: Some("noaa_precip"), sum: true, unit: "in" },
        "snowfall" => Metric { om: "snowfall", noaa: Some("noaa_snowfall"), sum: true, unit: "in" },
        "wind_speed" => Metric { om: "wind_speed", noaa: Some("noaa_wind_avg"), sum: false, unit: "mph" },
        "uv_index" => Metric { om: "uv_index", noaa: None, sum: false, unit: "" },
        "pressure" => Metric { om: "pressure", noaa: Some("noaa_pressure"), sum: false, unit: "hPa" },
        "dew_point" => Metric { om: "dew_point", noaa: Some("noaa_dew_point"), sum: false, unit: "°F" },
        "soil_temp" => Metric { om: "soil_temp_6in", noaa: None, sum: false, unit: "°F" },
        "soil_moisture" => Metric { om: "soil_moisture_6in", noaa: None, sum: false, unit: "" },
        _ => return None,
    })
}

/// Month names, in order, for the chart labels.
const MONTH_ABBR: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// `?weather_data&metric=...&year=...`: monthly weather numbers for the charts, as JSON.
///
/// Takes the address parameters. Gives the monthly series plus yearly and quarterly roll-ups, how
/// many months came from Open-Meteo versus NOAA, and min/max/mean. Answers are cached by the
/// browser for 30 minutes. The PHP version still talked to InfluxDB; this one reads the `weather`
/// table in MariaDB (see DEVIATIONS.md).
async fn weather_data(state: &AppState, q: &HashMap<String, String>) -> Response {
    // Everything runs inside one `async { ... }` block so each early `return json!(...)` inside it
    // gives the finished JSON; the block's result is stored in `res` and sent at the end.
    let res = async {
        if !weather_available(state).await {
            return json!({ "error": "InfluxDB not reachable" });
        }
        // 1. Which metric? Default is temperature. `let ... else` returns an error answer if the
        // name is not known.
        let name = q.get("metric").map(String::as_str).unwrap_or("temperature");
        let Some(m) = metric(name) else { return json!({ "error": "Invalid metric" }) };
        // 2. Totals use SUM, the others AVG. The NOAA side uses MAX for totals and AVG for the
        // others.
        let agg = if m.sum { "SUM" } else { "AVG" };
        let noaa_day = match m.noaa {
            Some(c) => format!("{}({c})", if m.sum { "MAX" } else { "AVG" }),
            None => "CAST(NULL AS DOUBLE)".to_string(),
        };
        // 3. A four-digit `year` shows that year; anything else shows everything since 2020 and
        // clears the year so the answer says so. The digits check means typed text never reaches
        // the SQL.
        let mut year_filter = q.get("year").cloned().unwrap_or_default();
        let where_clause = if year_filter.len() == 4 && year_filter.bytes().all(|b| b.is_ascii_digit()) {
            let y0: i64 = year_filter.parse().unwrap_or(0);
            format!("obs_time >= '{y0}-01-01 00:00:00' AND obs_time < '{}-01-01 00:00:00'", y0 + 1)
        } else {
            year_filter.clear();
            "obs_time >= '2020-01-01 00:00:00'".to_string()
        };
        // Two-step merge: each day collapses to one value (Open-Meteo first,
        // NOAA fills the gaps), then days roll up to months.
        // 4. Build the SQL. The inner query collapses each day to one Open-Meteo value and one NOAA
        // value. The outer query rolls days up to months, using the NOAA value only when the
        // Open-Meteo one is missing (`COALESCE`). `{agg}` is SUM or AVG.
        let sql = format!(
            "SELECT DATE_FORMAT(d, '%Y-%m-01') AS period,
                    {agg}(COALESCE(om_v, noaa_v)) AS value,
                    {agg}(om_v) AS om_value,
                    {agg}(noaa_v) AS noaa_value
             FROM (
                SELECT DATE(obs_time) AS d, {agg}({om}) AS om_v, {noaa_day} AS noaa_v
                FROM weather WHERE {where_clause}
                GROUP BY DATE(obs_time)
             ) daily
             GROUP BY DATE_FORMAT(d, '%Y-%m-01')
             ORDER BY period ASC",
            om = m.om
        );
        // 5. Run it. One row per month, oldest first.
        let rows = weather_query(state, &sql).await;

        // 6. Walk the rows and build the lists the chart needs: month labels, ISO keys, values, and
        // groups for yearly and quarterly roll-ups.
        let mut mon_labels = Vec::new();
        let mut mon_iso = Vec::new();
        let mut mon_data: Vec<f64> = Vec::new();
        let mut year_agg: Vec<(i64, Vec<f64>)> = Vec::new();
        let mut quarter: Vec<((i64, i64), Vec<f64>)> = Vec::new();
        let (mut om_months, mut noaa_only) = (0, 0);
        for r in &rows {
            // Skip a month that has no value.
            let Some(v) = num(r, "value") else { continue };
            let period = r.get("period").and_then(Value::as_str).unwrap_or("");
            // The period is text like `2024-03-01`: characters 0-3 are the year and 5-6 the month.
            let (Some(yr), Some(mo)) = (period.get(0..4).and_then(|s| s.parse::<i64>().ok()), period.get(5..7).and_then(|s| s.parse::<i64>().ok())) else { continue };
            // Round to 2 decimals the way PHP's `round` does.
            let val = php_round(v, 2);
            mon_labels.push(format!("{} {yr}", MONTH_ABBR[(mo - 1) as usize]));
            mon_iso.push(format!("{yr}-{mo:02}"));
            mon_data.push(val);
            push_group(&mut year_agg, yr, val);
            // Quarter number from month: (month + 2) / 3 using whole-number division gives 1 to 4.
            push_group(&mut quarter, (yr, (mo + 2) / 3), val);
            // Count how many months had Open-Meteo data and how many were filled from NOAA only.
            let has_om = r.get("om_value").map_or(false, |v| !v.is_null());
            let has_noaa = r.get("noaa_value").map_or(false, |v| !v.is_null());
            if has_om {
                om_months += 1;
            } else if has_noaa {
                noaa_only += 1;
            }
        }
        // A small function (closure) that rolls a group of monthly values into one number: the sum
        // for totals, the average otherwise.
        let rollup = |vals: &[f64]| -> Option<f64> {
            if vals.is_empty() {
                return None;
            }
            let s: f64 = vals.iter().sum();
            Some(php_round(if m.sum { s } else { s / vals.len() as f64 }, 2))
        };
        year_agg.sort_by_key(|(y, _)| *y);
        quarter.sort_by_key(|(k, _)| *k);
        // 7. Min, max and mean over all months (all `null` when there is no data).
        let stats = if mon_data.is_empty() {
            json!({ "min": null, "max": null, "mean": null, "count": 0 })
        } else {
            let mn = mon_data.iter().cloned().fold(f64::INFINITY, f64::min);
            let mx = mon_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mean = mon_data.iter().sum::<f64>() / mon_data.len() as f64;
            json!({ "min": php_round(mn, 1), "max": php_round(mx, 1), "mean": php_round(mean, 1), "count": mon_data.len() })
        };
        // 8. The JSON the charts read. Key names are fixed by the page's JavaScript.
        json!({
            "metric": name,
            "unit": m.unit,
            "agg": if m.sum { "sum" } else { "avg" },
            "year": year_filter,
            "labels": mon_labels,
            "data": mon_data,
            "monthly": { "labels": mon_labels, "iso": mon_iso, "data": mon_data },
            "yearly": { "labels": year_agg.iter().map(|(y, _)| y.to_string()).collect::<Vec<_>>(), "data": year_agg.iter().map(|(_, v)| rollup(v)).collect::<Vec<_>>() },
            "quarterly": {
                "labels": quarter.iter().map(|((y, qn), _)| format!("{y} Q{qn}")).collect::<Vec<_>>(),
                "data": quarter.iter().map(|(_, v)| rollup(v)).collect::<Vec<_>>(),
                "year": quarter.iter().map(|((y, _), _)| *y).collect::<Vec<_>>(),
                "q": quarter.iter().map(|((_, qn), _)| *qn).collect::<Vec<_>>(),
            },
            "coverage": { "om_months": om_months, "noaa_fill_months": noaa_only, "total_months": mon_data.len() },
            "stats": stats,
        })
    }
    .await;
    // 9. Send with a 30-minute cache. `Vary: Accept` tells caches the answer depends on the Accept
    // header.
    let mut r = with_cache(json_response(res), "public, max-age=1800");
    r.headers_mut().insert(header::VARY, HeaderValue::from_static("Accept"));
    r
}

/// Add `v` to the list belonging to `key`, creating the group if it is new. Keeps first-seen order.
/// The `<K: PartialEq>` part means "works for any key type that can be compared with `==`".
fn push_group<K: PartialEq>(groups: &mut Vec<(K, Vec<f64>)>, key: K, v: f64) {
    if let Some(g) = groups.iter_mut().find(|(k, _)| *k == key) {
        g.1.push(v);
    } else {
        groups.push((key, vec![v]));
    }
}

// --- ?weather_forecast ---------------------------------------------------------

/// The LSTM forecast file, with the imperial conversions added.
///
/// Takes the app data. Gives `Ok(JSON)` or `Err(message)`. Reads the forecast file the model wrote,
/// then adds Fahrenheit highs and lows (C x 9/5 + 32) and rainfall in inches (mm / 25.4). `map_err`
/// swaps a file error for a plain message; `?` returns that message right away; `ok_or` turns a
/// missing value into an error.
pub fn read_forecast(state: &AppState) -> Result<Value, &'static str> {
    let raw = std::fs::read(&state.forecast_file).map_err(|_| "Forecast file not found")?;
    let mut data: Value = serde_json::from_slice(&raw).map_err(|_| "Invalid forecast data")?;
    let p = data.get("predictions").cloned().ok_or("Invalid forecast data")?;
    let g = |k: &str| p.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    data["predictions"]["tmax_f"] = json!(php_round(g("tmax_c") * 9.0 / 5.0 + 32.0, 1));
    data["predictions"]["tmin_f"] = json!(php_round(g("tmin_c") * 9.0 / 5.0 + 32.0, 1));
    data["predictions"]["prcp_in"] = json!(php_round(g("prcp_mm") / 25.4, 2));
    Ok(data)
}

// --- ?mock_iot ----------------------------------------------------------------

/// `?mock_iot&sensor=...&bed=...`: made-up demo sensor numbers for the IoT charts. There are no
/// real sensors; the twelve numbers per bed are fixed sample values for Jul 25 to Jun 26.
///
/// Takes the address parameters. Gives chart JSON for one bed or all beds, or an error naming the
/// valid sensors.
fn mock_iot(q: &HashMap<String, String>) -> Value {
    let sensor = q.get("sensor").map(String::as_str).unwrap_or("soil_moisture");
    let bed = q.get("bed").map(String::as_str).unwrap_or("");
    let months = ["Jul 25", "Aug 25", "Sep 25", "Oct 25", "Nov 25", "Dec 25", "Jan 26", "Feb 26", "Mar 26", "Apr 26", "May 26", "Jun 26"];
    // Table of sensors: key, chart title, unit, and (bed name, twelve monthly values) pairs.
    let data: Vec<(&str, &str, &str, Vec<(&str, [i64; 12])>)> = vec![
        ("soil_moisture", "Soil Moisture (%)", "%", vec![
            ("Zen Garden", [42, 45, 40, 35, 30, 25, 20, 22, 28, 38, 45, 48]),
            ("River Garden", [55, 58, 52, 45, 38, 30, 25, 28, 35, 48, 55, 60]),
            ("Ko-Ja Garden", [38, 40, 35, 30, 25, 20, 18, 20, 25, 35, 42, 45]),
            ("Korea Garden", [33, 35, 30, 28, 22, 18, 15, 17, 22, 30, 38, 40]),
            ("The Royal Run", [28, 30, 25, 22, 18, 15, 12, 14, 20, 28, 35, 38]),
            ("Jason's Terrace", [45, 48, 42, 38, 32, 28, 22, 25, 30, 40, 48, 50]),
        ]),
        ("soil_temp", "Soil Temperature (°F)", "°F", vec![
            ("Zen Garden", [72, 70, 62, 50, 38, 30, 28, 30, 38, 50, 62, 70]),
            ("River Garden", [70, 68, 60, 48, 36, 28, 26, 28, 36, 48, 60, 68]),
            ("Ko-Ja Garden", [74, 72, 64, 52, 40, 32, 30, 32, 40, 52, 64, 72]),
            ("Korea Garden", [68, 66, 58, 46, 34, 26, 24, 26, 34, 46, 58, 66]),
            ("The Royal Run", [76, 74, 66, 54, 42, 34, 32, 34, 42, 54, 66, 74]),
        ]),
        ("soil_npk", "NPK Index (N+P+K avg mg/kg)", "", vec![
            ("Zen Garden", [45, 44, 42, 40, 38, 36, 35, 35, 37, 40, 43, 46]),
            ("River Garden", [40, 39, 38, 36, 34, 32, 31, 31, 33, 36, 39, 41]),
            ("Ko-Ja Garden", [52, 50, 48, 45, 42, 40, 38, 39, 42, 46, 50, 53]),
            ("Korea Garden", [55, 53, 50, 48, 45, 42, 40, 41, 44, 48, 52, 56]),
            ("The Royal Run", [48, 46, 44, 42, 40, 38, 36, 37, 40, 43, 47, 49]),
        ]),
        ("irrigation", "Water Usage (gal/week)", " gal", vec![
            ("Zen Garden", [45, 50, 40, 25, 10, 0, 0, 0, 5, 20, 35, 48]),
            ("River Garden", [32, 38, 28, 18, 8, 0, 0, 0, 4, 15, 28, 35]),
            ("Ko-Ja Garden", [38, 42, 35, 22, 10, 0, 0, 0, 5, 18, 32, 40]),
            ("Korea Garden", [52, 58, 45, 30, 12, 0, 0, 0, 6, 25, 42, 55]),
            ("The Royal Run", [28, 32, 25, 15, 6, 0, 0, 0, 3, 12, 22, 30]),
        ]),
        ("light_lux", "Light Level (klux)", " klux", vec![
            ("Zen Garden", [12, 11, 9, 6, 3, 2, 2, 3, 5, 8, 11, 13]),
            ("River Garden", [8, 7, 6, 4, 2, 1, 1, 2, 4, 6, 8, 9]),
            ("Ko-Ja Garden", [15, 14, 12, 8, 4, 2, 2, 3, 6, 10, 14, 16]),
            ("Korea Garden", [18, 17, 14, 10, 5, 3, 3, 4, 7, 12, 16, 19]),
            ("The Royal Run", [20, 19, 16, 11, 6, 3, 3, 5, 8, 14, 18, 21]),
        ]),
    ];
    // Find the sensor asked for, or answer with an error.
    let Some((_, label, unit, beds)) = data.iter().find(|(s, _, _, _)| *s == sensor) else {
        return json!({ "error": "Invalid sensor. Valid: soil_moisture, soil_temp, soil_npk, irrigation, light_lux" });
    };
    // One bed if a valid bed was named, otherwise every bed.
    let datasets: Vec<Value> = match beds.iter().find(|(b, _)| !bed.is_empty() && *b == bed) {
        Some((b, v)) => vec![json!({ "label": b, "data": v })],
        None => beds.iter().map(|(b, v)| json!({ "label": b, "data": v })).collect(),
    };
    json!({ "labels": months, "datasets": datasets, "sensor": sensor, "sensorLabel": label, "unit": unit })
}

// --- ?cross_tab ------------------------------------------------------------------

/// One thing the cross-tab can count plants by (a "dimension"), such as sun exposure or bed.
///
/// `col` is the SQL used to group (it may be an expression), `label` is the nice name shown to the
/// person, and `raw` is the plain column used for filtering and for cleaning values. For the
/// computed Bloom Season dimension `raw` is an expression, so it never matches a cleaning
/// vocabulary, the same as in the PHP.
struct Dim {
    col: &'static str,
    label: &'static str,
    raw: &'static str,
}

/// SQL that turns bloom data into one Bloom Season label: "Does not flower" for non-ornamental
/// plants, else the `season` column, else `month_start` to `month_end`, else "See month data".
const BLOOM_SEASON_COL: &str = "CASE
            WHEN LOWER(TRIM(COALESCE(bloom_color,''))) IN ('not ornamental','none','insignificant','non-flowering','inconspicuous','n/a') THEN 'Does not flower'
            WHEN COALESCE(TRIM(season),'') != '' THEN TRIM(season)
            WHEN COALESCE(TRIM(month_start),'') != '' THEN CONCAT(TRIM(month_start), ' – ', TRIM(COALESCE(month_end, month_start)))
            WHEN bloom_months IS NOT NULL AND bloom_months REGEXP '[^. ]' THEN 'See month data'
            ELSE 'Does not flower'
        END";
/// The same bloom information as plain values, used for filtering with `LIKE`.
const BLOOM_SEASON_RAW: &str = "COALESCE(NULLIF(TRIM(season),''), NULLIF(TRIM(month_start),''), CASE WHEN bloom_months REGEXP '[^. ]' THEN bloom_months END, bloom_color)";

/// Look up a dimension by the key sent in the address (`year`, `sun`, `bed`, ...). Gives `None` for
/// an unknown key. Only these fixed column names are ever put into SQL, never typed text.
fn dim(key: &str) -> Option<Dim> {
    let d = |col, label, raw| Some(Dim { col, label, raw });
    match key {
        "year" => d("LEFT(year_planted,4)", "Year Planted", "year_planted"),
        "sun" => d("sun_exposure", "Sun Exposure", "sun_exposure"),
        "water" => d("water_needs", "Water Needs", "water_needs"),
        "family" => d("family", "Plant Family", "family"),
        "company" => d("company", "Brand / Company", "company"),
        "bed" => d("flower_bed", "Garden Bed", "flower_bed"),
        "deer" => d("deer_resistance", "Deer Resistance", "deer_resistance"),
        "zones" => d("zones", "Hardiness Zone", "zones"),
        "fertilizer" => d("fertilizer", "Fertilizer", "fertilizer"),
        "bloom_season" => d(BLOOM_SEASON_COL, "Bloom Season", BLOOM_SEASON_RAW),
        "grow_season" => d("grow_season", "Grow Season", "grow_season"),
        "toxicity" => d("toxicity", "Toxicity", "toxicity"),
        "plant_type" => d("plant_type", "Plant Type", "plant_type"),
        "store" => d("local_store", "Local Store", "local_store"),
        "growth_rate" => d("growth_rate", "Growth Rate", "growth_rate"),
        "native" => d("native_status", "Native Status", "native_status"),
        "evergreen" => d("is_evergreen", "Evergreen/Deciduous", "is_evergreen"),
        "flower_shape" => d("flower_shape", "Flower Shape", "flower_shape"),
        "root_system" => d("root_system", "Root System", "root_system"),
        "reproduction" => d("reproduction_type", "Reproduction", "reproduction_type"),
        "edibility" => d("fruit_edibility", "Fruit Edibility", "fruit_edibility"),
        "invasive" => d("invasive_local", "Invasive Status", "invasive_local"),
        "plant_name" => d("plant_name", "Plant Name", "plant_name"),
        "bloom_color" => d("bloom_color", "Bloom Color", "bloom_color"),
        "foliage_color" => d("foliage_color", "Foliage Color", "foliage_color"),
        "leaf_shape" => d("leaf_shape", "Leaf Shape", "leaf_shape"),
        "flower_size" => d("flower_size", "Flower Size", "flower_size"),
        "soil_ph" => d("soil_ph", "Soil pH", "soil_ph"),
        _ => None,
    }
}

/// The valid dimension keys, in order, for the error message. `[&str; 28]` is a fixed-size list of
/// 28 pieces of text.
const DIM_KEYS: [&str; 28] = [
    "year", "sun", "water", "family", "company", "bed", "deer", "zones", "fertilizer", "bloom_season", "grow_season",
    "toxicity", "plant_type", "store", "growth_rate", "native", "evergreen", "flower_shape", "root_system", "reproduction",
    "edibility", "invasive", "plant_name", "bloom_color", "foliage_color", "leaf_shape", "flower_size", "soil_ph",
];

/// Turn any JSON value into text: NULL becomes an empty string, text stays as it is, and numbers or
/// others are written out.
fn text_value(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Add a count under a label, keeping first-seen order (`$merged[$lbl] += ...`).
///
/// Add `n` to the count under `key`, keeping first-seen order.
fn add_count(list: &mut Vec<(String, i64)>, key: String, n: i64) {
    if let Some(e) = list.iter_mut().find(|(k, _)| *k == key) {
        e.1 += n;
    } else {
        list.push((key, n));
    }
}

/// `?cross_tab&dim_a=...&dim_b=...`: count plants by two properties at once (for example sun by
/// bed) and return a grid.
///
/// Takes the address parameters and `with_ai` (true for `?ai_fuse`, which also asks the local model
/// to read the table). Gives JSON `rowLabels`, `colLabels`, `matrix` and totals, limited to 15 rows
/// and 15 columns. Other modes: `single` (one dimension only) and `get_filter_values` (the values a
/// filter can take). Up to two filters can narrow the plants counted.
async fn cross_tab(state: &AppState, q: &HashMap<String, String>, with_ai: bool) -> Value {
    // 1. Read the two dimensions. Both must be known, or an error is returned.
    let dim_a_key = q.get("dim_a").map(String::as_str).unwrap_or("");
    let dim_b_key = q.get("dim_b").map(String::as_str).unwrap_or("");
    let (Some(da), Some(db_)) = (dim(dim_a_key), dim(dim_b_key)) else {
        return json!({ "error": format!("Invalid dimensions. Valid: {}", DIM_KEYS.join(", ")) });
    };
    let dim_filter = q.get("dim_filter").map(String::as_str).unwrap_or("");
    let filter_val = q.get("filter_val").map(String::as_str).unwrap_or("");
    let dim_filter2 = q.get("dim_filter2").map(String::as_str).unwrap_or("");
    let filter_val2 = q.get("filter_val2").map(String::as_str).unwrap_or("");

    // 2. Build the optional filters. Only `raw` column names from our fixed list go in the SQL; the
    // typed value is a `?` placeholder.
    let mut filter_where = String::new();
    let mut filter_params: Vec<String> = Vec::new();
    for (fk, fv) in [(dim_filter, filter_val), (dim_filter2, filter_val2)] {
        if !fk.is_empty() && !fv.is_empty() && fv != "0" {
            if let Some(fd) = dim(fk) {
                filter_where.push_str(&format!(" AND {} LIKE ?", fd.raw));
                filter_params.push(format!("%{fv}%"));
            }
        }
    }

    // Single dimension mode.
    if q.contains_key("single") {
        let order_by = if dim_a_key == "year" {
            "lbl DESC".to_string()
        } else if dim_a_key == "bloom_season" || dim_a_key == "grow_season" {
            "CASE
                WHEN lbl LIKE '%Spring%' AND lbl NOT LIKE '%Summer%' AND lbl NOT LIKE '%Fall%' AND lbl NOT LIKE '%Winter%' THEN 1
                WHEN lbl LIKE '%Summer%' AND lbl NOT LIKE '%Fall%' AND lbl NOT LIKE '%Winter%' THEN 2
                WHEN lbl LIKE '%Fall%'   AND lbl NOT LIKE '%Winter%' THEN 3
                WHEN lbl LIKE '%Winter%' THEN 4
                WHEN lbl LIKE '%See month%' THEN 5
                WHEN lbl LIKE '%Does not%' THEN 6
                ELSE 5 END, cnt DESC"
                .to_string()
        } else {
            // Ties in name order: without it MariaDB returns equal counts in no fixed order.
            "cnt DESC, lbl".to_string()
        };
        let sql = format!(
            "SELECT {} AS lbl, COUNT(*) AS cnt FROM data_source WHERE {} IS NOT NULL AND TRIM({}) != ''{filter_where} GROUP BY lbl ORDER BY {order_by} LIMIT 500",
            da.col, da.raw, da.raw
        );
        // Run the query, binding each filter value to its `?`.
        let mut qy = sqlx::query(&sql);
        for p in &filter_params {
            qy = qy.bind(p);
        }
        let rows = match qy.fetch_all(&state.db).await {
            Ok(r) => r,
            Err(e) => return json!({ "error": hide(e) }),
        };
        // Clean each label with `garden_normalise`, merge labels that become the same, and keep at
        // most 20.
        let mut merged: Vec<(String, i64)> = Vec::new();
        for r in &rows {
            let m = match row_to_json(r) {
                Value::Object(m) => m,
                _ => continue,
            };
            let lbl = garden_normalise(da.raw, &text_value(m.get("lbl").unwrap_or(&Value::Null)));
            if lbl.is_empty() {
                continue;
            }
            add_count(&mut merged, lbl, num(&m, "cnt").unwrap_or(0.0) as i64);
        }
        merged.truncate(20);
        let totals: Vec<i64> = merged.iter().map(|(_, c)| *c).collect();
        return json!({
            "dim_a_label": da.label,
            "dim_b_label": da.label,
            "rowLabels": merged.iter().map(|(l, _)| php_key(l)).collect::<Vec<_>>(),
            "colLabels": [da.label],
            "matrix": totals.iter().map(|t| vec![*t]).collect::<Vec<_>>(),
            "rowTotals": totals,
            "colTotals": [totals.iter().sum::<i64>()],
        });
    }

    // The values a filter dimension can take.
    if q.contains_key("get_filter_values") && !dim_filter.is_empty() {
        if let Some(fd) = dim(dim_filter) {
            let sql = format!("SELECT DISTINCT {} AS val FROM data_source WHERE {} IS NOT NULL AND TRIM({}) != '' ORDER BY val", fd.col, fd.raw, fd.raw);
            let vals: Vec<Value> = sqlx::query(&sql)
                .fetch_all(&state.db)
                .await
                .unwrap_or_default()
                .iter()
                .map(|r| match row_to_json(r) {
                    Value::Object(m) => m.get("val").cloned().unwrap_or(Value::Null),
                    _ => Value::Null,
                })
                .collect();
            return json!({ "values": vals });
        }
    }

    // 3. Fertilizer holds comma-separated lists, so it is split into separate values below.
    let needs_split_a = dim_a_key == "fertilizer";
    let needs_split_b = dim_b_key == "fertilizer";
    // 4. Count plants per (a, b) pair in SQL, most common first.
    let sql = format!(
        "SELECT {} AS dim_a, {} AS dim_b, {} AS raw_a, {} AS raw_b, COUNT(*) AS cnt FROM data_source
         WHERE ({} IS NOT NULL AND TRIM({}) != '') AND ({} IS NOT NULL AND TRIM({}) != ''){filter_where}
         GROUP BY dim_a, dim_b ORDER BY cnt DESC, dim_a, dim_b",
        da.col, db_.col, da.raw, db_.raw, da.raw, da.raw, db_.raw, db_.raw
    );
    let mut qy = sqlx::query(&sql);
    for p in &filter_params {
        qy = qy.bind(p);
    }
    let rows = match qy.fetch_all(&state.db).await {
        Ok(r) => r,
        Err(e) => return json!({ "error": hide(e) }),
    };

    // Expand comma-separated fields, normalise, and count every (a, b).
    // 5. Clean each pair and add up counts that now match (cleaning can merge two labels).
    let mut expanded: Vec<((String, String), i64)> = Vec::new();
    for r in &rows {
        let m = match row_to_json(r) {
            Value::Object(m) => m,
            _ => continue,
        };
        let cnt = num(&m, "cnt").unwrap_or(0.0) as i64;
        let a_raw = text_value(m.get("dim_a").unwrap_or(&Value::Null));
        let b_raw = text_value(m.get("dim_b").unwrap_or(&Value::Null));
        let a_vals: Vec<String> = if needs_split_a { a_raw.split(',').map(|s| s.trim().to_string()).collect() } else { vec![a_raw] };
        let b_vals: Vec<String> = if needs_split_b { b_raw.split(',').map(|s| s.trim().to_string()).collect() } else { vec![b_raw] };
        for a in &a_vals {
            for b in &b_vals {
                let a = garden_normalise(da.raw, a);
                let b = garden_normalise(db_.raw, b);
                if a.is_empty() || b.is_empty() {
                    continue;
                }
                if let Some(e) = expanded.iter_mut().find(|(k, _)| k.0 == a && k.1 == b) {
                    e.1 += cnt;
                } else {
                    expanded.push(((a, b), cnt));
                }
            }
        }
    }
    // 6. Total each row label and column label separately.
    let mut row_set: Vec<(String, i64)> = Vec::new();
    let mut col_set: Vec<(String, i64)> = Vec::new();
    for ((a, b), c) in &expanded {
        add_count(&mut row_set, a.clone(), *c);
        add_count(&mut col_set, b.clone(), *c);
    }
    if dim_a_key == "year" {
        // krsort: reverse by key. Years are numeric strings; compare as such.
        row_set.sort_by(|a, b| b.0.parse::<i64>().unwrap_or(0).cmp(&a.0.parse::<i64>().unwrap_or(0)).then_with(|| b.0.cmp(&a.0)));
    } else {
        row_set.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    }
    // Biggest first; equal counts in name order, so the grid is the same on every load (the
    // PHP's order for ties depended on how the database happened to group the rows).
    col_set.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    // 7. Keep the 15 biggest rows and columns and fill the grid; a pair that never appeared is 0.
    let row_labels: Vec<String> = row_set.iter().take(15).map(|(l, _)| l.clone()).collect();
    let col_labels: Vec<String> = col_set.iter().take(15).map(|(l, _)| l.clone()).collect();
    let grid: Vec<Vec<i64>> = row_labels
        .iter()
        .map(|r| col_labels.iter().map(|c| expanded.iter().find(|((a, b), _)| a == r && b == c).map(|(_, n)| *n).unwrap_or(0)).collect())
        .collect();
    let row_labels_json: Vec<Value> = row_labels.iter().map(|l| php_key(l)).collect();
    let col_labels_json: Vec<Value> = col_labels.iter().map(|l| php_key(l)).collect();
    // 8. Build the JSON. `php_key` makes numeric labels such as years numbers, as PHP did.
    let mut out = json!({
        "dim_a_label": da.label,
        "dim_b_label": db_.label,
        "rowLabels": row_labels_json,
        "colLabels": col_labels_json,
        "matrix": grid,
        "rowTotals": row_labels.iter().map(|r| row_set.iter().find(|(l, _)| l == r).map(|(_, n)| *n).unwrap_or(0)).collect::<Vec<_>>(),
        "colTotals": col_labels.iter().map(|c| col_set.iter().find(|(l, _)| l == c).map(|(_, n)| *n).unwrap_or(0)).collect::<Vec<_>>(),
    });
    // 9. For `?ai_fuse`: describe the weather scenario, ask the local model to comment, and add its
    // answer. If it does not answer, send a clear message; the counts are still correct.
    if with_ai {
        let temp = q.get("temp_scenario").map(String::as_str).filter(|s| !s.is_empty());
        let rain = q.get("rain_scenario").map(String::as_str).filter(|s| !s.is_empty());
        out["scenario"] = json!(brain::scenario_label(temp, rain));
        let ai = brain::fuse_ask(state, &out, temp, rain, q.get("filter_val").map(String::as_str).filter(|s| !s.is_empty())).await;
        out["ai"] = ai.unwrap_or_else(|| json!({ "error": "The local model did not answer. The counts above are still correct." }));
    }
    out
}

/// PHP turns array keys that are canonical integers back into integers:
/// a "2024" label comes out as the number 2024 in the JSON.
///
/// A label counts as a PHP integer key if it is "0" or digits with no leading zero (a minus sign is
/// allowed). So "007" and "6.5" stay text. The page's scripts depend on this.
fn php_key(l: &str) -> Value {
    let canonical = l == "0" || (l.trim_start_matches('-').chars().all(|c| c.is_ascii_digit()) && !l.trim_start_matches('-').starts_with('0') && !l.is_empty() && l != "-");
    if canonical {
        if let Ok(n) = l.parse::<i64>() {
            return json!(n);
        }
    }
    json!(l)
}

// --- widget_data.php ---------------------------------------------------------------

/// Count how many rows have each value of one column, for the widgets.
///
/// Takes the rows, the column, a label for empty values ("" means skip empties), a maximum number
/// of results (0 means no limit) and a sort order (`cnt_desc`, `key_asc`, `key_desc`). Gives two
/// lists of the same length: labels and counts. Values are cleaned with `garden_normalise` first.
fn group_count(rows: &[Map<String, Value>], col: &str, empty: &str, limit: usize, sort: &str) -> (Vec<String>, Vec<i64>) {
    let mut map: Vec<(String, i64)> = Vec::new();
    for r in rows {
        let v = garden_normalise(col, &text_value(r.get(col).unwrap_or(&Value::Null)));
        if v.is_empty() && empty.is_empty() {
            continue;
        }
        let key = if v.is_empty() { empty.to_string() } else { v };
        add_count(&mut map, key, 1);
    }
    match sort {
        "cnt_desc" => map.sort_by(|a, b| b.1.cmp(&a.1)),
        "key_asc" => map.sort_by(|a, b| a.0.cmp(&b.0)),
        "key_desc" => map.sort_by(|a, b| b.0.cmp(&a.0)),
        _ => {}
    }
    if limit > 0 {
        map.truncate(limit);
    }
    (map.iter().map(|(k, _)| k.clone()).collect(), map.iter().map(|(_, v)| *v).collect())
}

/// Build the JSON for one chart card.
///
/// `labels`/`data` are the points, `chart` is the type (bar or doughnut), `id` is the HTML id,
/// `label` names the data set, `axis` is `x` or `y` (`y` draws horizontal bars), `title` and `icon`
/// are shown on the card, and `tags` are search words for the widget picker. Key names must match
/// what `js/widget-loader.js` reads.
fn widget(labels: Vec<String>, data: Vec<Value>, chart: &str, id: &str, label: &str, axis: &str, title: &str, icon: &str, tags: &str) -> Value {
    json!({
        "labels": labels, "data": data, "chart": chart, "id": id, "label": label,
        "indexAxis": axis, "title": title, "icon": icon, "tags": tags,
    })
}

/// Turn a list of whole numbers into a list of JSON numbers.
fn ints(v: Vec<i64>) -> Vec<Value> {
    v.into_iter().map(Value::from).collect()
}

/// Year labels are integers in PHP's JSON (numeric array keys).
fn year_labels(v: &[String]) -> Vec<Value> {
    v.iter().map(|y| y.parse::<i64>().map(Value::from).unwrap_or_else(|_| json!(y))).collect()
}

/// `widget_data.php`: every aggregation from one `SELECT *`.
///
/// Takes the app data. Gives a map of card name to chart JSON, plus `_summary`, or a database
/// error. Steps: 1 read every plant once, 2 build 24 cards one block at a time (numbered as in the
/// PHP), 3 add the totals. The caller caches the result for five minutes. `Result<A, B>` means
/// "good answer A or error B".
pub async fn build_widget_data(state: &AppState) -> Result<Map<String, Value>, sqlx::Error> {
    // 1. One `SELECT *`; every card is worked out in Rust from these rows. The trailing `?` returns
    // the error early.
    let rows: Vec<Map<String, Value>> = sqlx::query("SELECT * FROM `data_source`")
        .fetch_all(&state.db)
        .await?
        .iter()
        .filter_map(|r| if let Value::Object(m) = row_to_json(r) { Some(m) } else { None })
        .collect();
    // Two small helpers: `s` reads a column as text, `i` reads a column as a whole number (text
    // numbers are converted like PHP's (int)).
    let s = |r: &Map<String, Value>, k: &str| text_value(r.get(k).unwrap_or(&Value::Null));
    let i = |r: &Map<String, Value>, k: &str| r.get(k).map(|v| v.as_i64().unwrap_or_else(|| php_int(&text_value(v)))).unwrap_or(0);
    // `w` is the output map; each card is added with `w.insert("name", ...)`.
    let mut w = Map::new();

    // 1. growth by year
    let mut year_map: Vec<(String, i64)> = Vec::new();
    for r in &rows {
        let yr: String = s(r, "year_planted").chars().take(4).collect();
        if year_in_range(&yr) {
            add_count(&mut year_map, yr, 1);
        }
    }
    year_map.sort_by(|a, b| a.0.cmp(&b.0));
    // The growth card is built in two steps: make the card, then replace its labels with numbers so
    // years are numbers in the JSON, like PHP.
    w.insert("growth".into(), {
        let mut v = widget(vec![], ints(year_map.iter().map(|(_, c)| *c).collect()), "bar", "wgt-growth", "New Varieties", "x", "Collection Growth by Year", "📈", "growth timeline");
        v["labels"] = json!(year_labels(&year_map.iter().map(|(y, _)| y.clone()).collect::<Vec<_>>()));
        v
    });

    // 2. sun
    let (l, d) = group_count(&rows, "sun_exposure", "", 0, "cnt_desc");
    w.insert("sun".into(), widget(l, ints(d), "doughnut", "wgt-sun", "Sun Exposure", "x", "Sun Exposure Breakdown", "☀️", "sun light"));
    // 3. families
    let (l, d) = group_count(&rows, "family", "", 10, "cnt_desc");
    w.insert("families".into(), widget(l, ints(d), "bar", "wgt-families", "Varieties", "y", "Top Plant Families", "🌳", "families taxonomy"));
    // 4. fertilizer
    // Fertilizer: ignore blanks, "None", and values containing `~`; keep the top 8.
    let mut fert: Vec<(String, i64)> = Vec::new();
    for r in &rows {
        let v = s(r, "fertilizer").trim().to_string();
        if v.is_empty() || v == "None" || v.contains('~') {
            continue;
        }
        add_count(&mut fert, v, 1);
    }
    fert.sort_by(|a, b| b.1.cmp(&a.1));
    fert.truncate(8);
    w.insert("fertilizer".into(), widget(fert.iter().map(|(k, _)| k.clone()).collect(), ints(fert.iter().map(|(_, c)| *c).collect()), "doughnut", "wgt-fertilizer", "Fertilizers", "x", "Common Fertilizers Used", "🧪", "fertilizer care"));
    // 5. wildlife
    // Wildlife: split each `attracts` text into single animals (on commas and "and"), clean them,
    // capitalise the first letter, and keep the top 8.
    let mut wild: Vec<(String, i64)> = Vec::new();
    for r in &rows {
        let v = garden_clean(&s(r, "attracts"));
        if v.is_empty() {
            continue;
        }
        for p in split_and(&v) {
            let p = garden_clean(&p);
            if p.is_empty() {
                continue;
            }
            let lower = p.to_lowercase();
            let mut c = lower.chars();
            let cap = match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            };
            add_count(&mut wild, cap, 1);
        }
    }
    wild.sort_by(|a, b| b.1.cmp(&a.1));
    wild.truncate(8);
    w.insert("wildlife".into(), widget(wild.iter().map(|(k, _)| k.clone()).collect(), ints(wild.iter().map(|(_, c)| *c).collect()), "bar", "wgt-wildlife", "Plants Attracting", "x", "Wildlife & Pollinators Attracted", "🦋", "wildlife pollinators"));
    // 6. vendors
    let (l, d) = group_count(&rows, "company", "", 10, "cnt_desc");
    w.insert("vendors".into(), widget(l, ints(d), "bar", "wgt-vendors", "Varieties Bought", "y", "Where Plants Were Purchased", "🏬", "vendors purchase"));
    // 7. beds
    let (l, d) = group_count(&rows, "flower_bed", "Unassigned", 0, "cnt_desc");
    w.insert("beds".into(), widget(l, ints(d), "bar", "wgt-beds", "Varieties", "y", "Plants per Flower Bed", "🏡", "layout beds"));
    // 8. water
    let (l, d) = group_count(&rows, "water_needs", "Unspecified", 0, "cnt_desc");
    w.insert("water".into(), widget(l, ints(d), "doughnut", "wgt-water", "Water Needs", "x", "Water Needs Breakdown", "💧", "water irrigation"));
    // 9. zones
    // Zones: values like `5-9` count once in every zone from 5 to 9; plain numbers like `6` count
    // once; anything else is skipped.
    let mut zones: Vec<(String, i64)> = Vec::new();
    for r in &rows {
        let v = s(r, "zones").trim().to_string();
        if v.is_empty() {
            continue;
        }
        for p in v.split([',', '/']) {
            let p = p.trim();
            if let Some((a, b)) = p.split_once('-') {
                if let (Ok(a), Ok(b)) = (a.trim().parse::<i64>(), b.trim().parse::<i64>()) {
                    if a.to_string() == a.to_string().trim() {
                        for z in a..=b {
                            add_count(&mut zones, format!("Zone {z}"), 1);
                        }
                    }
                    continue;
                }
            }
            if !p.is_empty() && p.parse::<f64>().is_ok() {
                add_count(&mut zones, format!("Zone {p}"), 1);
            }
        }
    }
    zones.sort_by(|a, b| a.0.cmp(&b.0));
    w.insert("zones".into(), widget(zones.iter().map(|(k, _)| k.clone()).collect(), ints(zones.iter().map(|(_, c)| *c).collect()), "bar", "wgt-zones", "Plants", "x", "Hardiness Zone Distribution", "🗺️", "zones hardiness"));
    // 10. deer
    let (l, d) = group_count(&rows, "deer_resistance", "Not Specified", 0, "cnt_desc");
    w.insert("deer".into(), widget(l, ints(d), "doughnut", "wgt-deer", "Deer Resistance", "x", "Deer & Rabbit Resistance", "🦌", "deer wildlife"));
    // 11. season
    let (l, d) = group_count(&rows, "season", "Unspecified", 0, "cnt_desc");
    w.insert("season".into(), widget(l, ints(d), "doughnut", "wgt-season", "Bloom Season", "x", "Bloom Season", "🌸", "bloom season"));
    // 12. grow season
    let (l, d) = group_count(&rows, "grow_season", "Unspecified", 0, "cnt_desc");
    w.insert("grow-season".into(), widget(l, ints(d), "doughnut", "wgt-grow-season", "Grow Season", "x", "Grow Season", "🌱", "grow season"));
    // 13. edibility
    // Edibility: every plant is put in one bucket. Wildlife-only first, then edible and non-toxic,
    // then by who the toxicity text mentions (humans, pets, both). Empty buckets are dropped at the
    // end.
    let mut ed: Vec<(&str, i64)> = vec![
        ("Fully Edible", 0), ("Non-toxic", 0), ("Wildlife Only", 0), ("Caution (Pets)", 0), ("Caution (Humans)", 0),
        ("Caution (Humans & Pets)", 0), ("Toxic", 0), ("Not Specified", 0),
    ];
    for r in &rows {
        let tox = garden_clean(&s(r, "toxicity")).to_lowercase();
        let e = garden_clean(&s(r, "fruit_edibility")).to_lowercase();
        let safe = tox.contains("non-toxic") || tox.contains("nontoxic");
        let humans = !safe && tox.contains("human");
        let pets = !safe && (tox.contains("dog") || tox.contains("cat") || tox.contains("pet"));
        let bucket = if e == "wildlife only" {
            "Wildlife Only"
        } else if e.contains("edible") && safe {
            "Fully Edible"
        } else if safe {
            "Non-toxic"
        } else if humans && pets {
            "Caution (Humans & Pets)"
        } else if humans {
            "Caution (Humans)"
        } else if pets {
            "Caution (Pets)"
        } else if !tox.is_empty() {
            "Toxic"
        } else {
            "Not Specified"
        };
        if let Some(b) = ed.iter_mut().find(|(k, _)| *k == bucket) {
            b.1 += 1;
        }
    }
    let ed: Vec<(&str, i64)> = ed.into_iter().filter(|(_, c)| *c != 0).collect();
    w.insert("edibility".into(), widget(ed.iter().map(|(k, _)| k.to_string()).collect(), ints(ed.iter().map(|(_, c)| *c).collect()), "doughnut", "wgt-edibility", "Edibility", "x", "Edible vs Toxic", "🥗", "edibility toxicity food"));
    // 14. losses
    // Losses: a plant with quantity above 0 is still growing; the rest are gone and are counted per
    // bed (top 6).
    let (mut active, mut gone) = (0i64, 0i64);
    let mut loss_bed: Vec<(String, i64)> = Vec::new();
    for r in &rows {
        if i(r, "qty") > 0 {
            active += 1;
        } else {
            gone += 1;
            let bed = s(r, "flower_bed").trim().to_string();
            add_count(&mut loss_bed, if bed.is_empty() || bed == "0" { "Unassigned".into() } else { bed }, 1);
        }
    }
    loss_bed.sort_by(|a, b| b.1.cmp(&a.1));
    loss_bed.truncate(6);
    w.insert("losses".into(), json!({
        "labels": ["Still Growing", "Gone (qty=0)"], "data": [active, gone],
        "bedLabels": loss_bed.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(), "bedData": loss_bed.iter().map(|(_, c)| *c).collect::<Vec<_>>(),
        "chart": "doughnut", "id": "wgt-losses", "label": "Losses vs Survivors", "indexAxis": "x",
        "title": "Plant Losses vs Survivors", "icon": "📉", "tags": "losses survival",
    }));
    // 15. top plants by qty (later rows with the same name overwrite earlier ones)
    // Top plants: highest quantity first, top 12. If a name appears twice the later row wins, as in
    // the PHP.
    let mut top: Vec<(String, i64)> = Vec::new();
    for r in &rows {
        let qty = i(r, "qty");
        if qty > 0 {
            let name = s(r, "plant_name");
            if let Some(e) = top.iter_mut().find(|(k, _)| *k == name) {
                e.1 = qty;
            } else {
                top.push((name, qty));
            }
        }
    }
    top.sort_by(|a, b| b.1.cmp(&a.1));
    top.truncate(12);
    w.insert("top-plants".into(), widget(top.iter().map(|(k, _)| k.clone()).collect(), ints(top.iter().map(|(_, c)| *c).collect()), "bar", "wgt-top-plants", "Qty in Garden", "y", "Top Plants by Quantity", "🏆", "top quantity"));
    // 16. store
    // Store: total quantity per store and number of varieties per store; empty or "0" store is
    // "Unknown Store".
    let mut store_qty: Vec<(String, i64)> = Vec::new();
    let mut store_var: Vec<(String, i64)> = Vec::new();
    for r in &rows {
        let st = s(r, "local_store").trim().to_string();
        let st = if st.is_empty() || st == "0" { "Unknown Store".to_string() } else { st };
        add_count(&mut store_qty, st.clone(), i(r, "qty"));
        add_count(&mut store_var, st, 1);
    }
    store_qty.sort_by(|a, b| b.1.cmp(&a.1));
    store_qty.truncate(12);
    let varieties: Vec<i64> = store_qty.iter().map(|(k, _)| store_var.iter().find(|(s2, _)| s2 == k).map(|(_, c)| *c).unwrap_or(0)).collect();
    w.insert("store".into(), json!({
        "labels": store_qty.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(), "data": store_qty.iter().map(|(_, c)| *c).collect::<Vec<_>>(),
        "varieties": varieties, "chart": "doughnut", "id": "wgt-store", "label": "Total Plants", "indexAxis": "x",
        "title": "Local Store Breakdown", "icon": "🛒", "tags": "store purchase",
    }));
    // 17. investment
    // Investment: for each year, add up `average_cost x qty` for plants with a cost above 0.
    let mut invest: Vec<(String, f64)> = Vec::new();
    for r in &rows {
        let yr: String = s(r, "year_planted").chars().take(4).collect();
        let cost = php_float(&s(r, "average_cost"));
        let qty = i(r, "qty");
        if !year_in_range(&yr) || cost <= 0.0 {
            continue;
        }
        if let Some(e) = invest.iter_mut().find(|(k, _)| *k == yr) {
            e.1 += cost * qty as f64;
        } else {
            invest.push((yr, cost * qty as f64));
        }
    }
    invest.sort_by(|a, b| a.0.cmp(&b.0));
    w.insert("investment".into(), json!({
        "labels": year_labels(&invest.iter().map(|(y, _)| y.clone()).collect::<Vec<_>>()),
        "data": invest.iter().map(|(_, v)| json_number(*v)).collect::<Vec<_>>(),
        "chart": "bar", "id": "wgt-investment", "label": "Total Spent ($)", "indexAxis": "x",
        "title": "Investment by Year", "icon": "💰", "tags": "investment cost money",
    }));
    // 18. toxicity
    let (l, d) = group_count(&rows, "toxicity", "", 0, "cnt_desc");
    w.insert("toxicity".into(), widget(l, ints(d), "doughnut", "wgt-toxicity", "Count", "x", "Toxicity Safety Map", "☠️", "toxicity safety"));
    // 19. survival
    // Survival: for each year planted, how many plants are alive and how many have `died = 1`.
    let mut surv: Vec<(String, i64, i64)> = Vec::new();
    for r in &rows {
        let yr: String = s(r, "year_planted").chars().take(4).collect();
        if !year_in_range(&yr) {
            continue;
        }
        let died = i(r, "died") == 1;
        if let Some(e) = surv.iter_mut().find(|(k, _, _)| *k == yr) {
            if died { e.2 += 1 } else { e.1 += 1 }
        } else {
            surv.push((yr, (!died) as i64, died as i64));
        }
    }
    surv.sort_by(|a, b| a.0.cmp(&b.0));
    w.insert("survival".into(), json!({
        "labels": year_labels(&surv.iter().map(|(y, _, _)| y.clone()).collect::<Vec<_>>()),
        "alive": surv.iter().map(|(_, a, _)| *a).collect::<Vec<_>>(),
        "died": surv.iter().map(|(_, _, d)| *d).collect::<Vec<_>>(),
        "chart": "bar", "id": "wgt-survival", "stacked": true, "label": "Survived", "indexAxis": "x",
        "title": "Survival Rate by Year", "icon": "💪", "tags": "survival mortality",
    }));
    // 20. flower shapes
    let (l, d) = group_count(&rows, "flower_shape", "", 12, "cnt_desc");
    w.insert("flower-shapes".into(), widget(l, ints(d), "bar", "wgt-flower-shapes", "Count", "y", "Flower Shape Diversity", "🌺", "flower shapes"));
    // 21. roots
    let (l, d) = group_count(&rows, "root_system", "", 0, "cnt_desc");
    w.insert("roots".into(), widget(l, ints(d), "doughnut", "wgt-roots", "Count", "x", "Root System Balance", "🌿", "roots underground"));
    // 22. evergreen
    let (l, d) = group_count(&rows, "is_evergreen", "", 0, "cnt_desc");
    w.insert("evergreen".into(), widget(l, ints(d), "bar", "wgt-evergreen", "Count", "x", "Evergreen Coverage", "🌲", "evergreen deciduous"));
    // 23. growth rate
    // Growth rate: counted, then ordered Slow, Moderate, Fast by the small `rank` helper (anything
    // else last).
    let mut gr: Vec<(String, i64)> = Vec::new();
    for r in &rows {
        let v = s(r, "growth_rate").trim().to_string();
        if !v.is_empty() {
            add_count(&mut gr, v, 1);
        }
    }
    let rank = |k: &str| match k {
        "Slow" => 1,
        "Moderate" => 2,
        "Fast" => 3,
        _ => 99,
    };
    gr.sort_by_key(|(k, _)| rank(k));
    w.insert("growth-rate".into(), widget(gr.iter().map(|(k, _)| k.clone()).collect(), ints(gr.iter().map(|(_, c)| *c).collect()), "bar", "wgt-growth-rate", "Count", "x", "Growth Rate Distribution", "📊", "growth rate speed"));
    // 24. native
    let (l, d) = group_count(&rows, "native_status", "", 0, "cnt_desc");
    w.insert("native".into(), widget(l, ints(d), "doughnut", "wgt-native", "Count", "x", "Native vs Cultivar", "🏔️", "native cultivar"));

    // summary
    // Summary: ignore dead plants; count one variety per live row and add up its quantity, which
    // may be a list like "3+2".
    let (mut plant_count, mut variety_count) = (0i64, 0i64);
    for r in &rows {
        if i(r, "died") == 1 || s(r, "flower_bed").to_lowercase().contains("dead") {
            continue;
        }
        variety_count += 1;
        let qty = s(r, "qty");
        let qty = if qty.is_empty() { "0".to_string() } else { qty };
        for part in qty.split([',', '+']) {
            plant_count += php_int(part.trim());
        }
    }
    w.insert("_summary".into(), json!({ "plantCount": plant_count, "varietyCount": variety_count }));
    Ok(w)
}

/// `preg_split('/[,]| and /i')`.
///
/// Splits first on commas, then on the word " and " in any upper or lower case, so "Bees,
/// Butterflies and Hummingbirds" becomes three items.
fn split_and(v: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in v.split(',') {
        let mut rest = part;
        loop {
            let lower = rest.to_lowercase();
            match lower.find(" and ") {
                Some(i) => {
                    out.push(rest[..i].to_string());
                    rest = &rest[i + 5..];
                }
                None => {
                    out.push(rest.to_string());
                    break;
                }
            }
        }
    }
    out
}

/// PHP prints a float with no fraction as an integer in JSON.
///
/// Whole numbers such as 40.0 become 40; others stay decimals. `v.fract()` is the part after the
/// decimal point.
fn json_number(v: f64) -> Value {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        json!(v as i64)
    } else {
        json!(v)
    }
}

/// `widget_data.php` with its five-minute cache.
///
/// Route handler for `GET /widget_data.php`. Takes the shared app data (`State`, an `Arc`, which is
/// a shared pointer many requests can use) and the address parameters. Steps: 1 `?invalidate`
/// clears the cache, 2 reuse the cached JSON if under 300 seconds old, 3 otherwise rebuild it with
/// `build_widget_data` and store it, 4 send either everything or just the card named in `?w=`.
pub async fn widget_data(State(state): State<Arc<AppState>>, Query(q): Query<HashMap<String, String>>) -> Response {
    let reply = |body: String| with_cache(([(header::CONTENT_TYPE, "application/json")], body).into_response(), "public, max-age=60");
    if q.contains_key("invalidate") {
        state.invalidate_widget_cache();
        return reply("{\"status\":\"cache_cleared\"}".into());
    }
    // The cache is a `Mutex` (only one request may use it at a time). `lock()` takes it,
    // `elapsed()` is the age of the stored answer.
    let cached: Option<String> = state
        .widget_cache
        .lock()
        .ok()
        .and_then(|c| c.as_ref().filter(|(t, _)| t.elapsed().as_secs() < 300).map(|(_, s)| s.clone()));
    let all = match cached {
        Some(s) => s,
        None => match build_widget_data(&state).await {
            Ok(w) => {
                let s = serde_json::to_string(&w).unwrap_or_else(|_| "{}".into());
                if let Ok(mut c) = state.widget_cache.lock() {
                    *c = Some((std::time::Instant::now(), s.clone()));
                }
                s
            }
            Err(e) => {
                tracing::error!("widget_data: {e}");
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "application/json")], "{\"error\":\"DB error\"}").into_response();
            }
        },
    };
    // `?w=name` returns one card; no `w` returns all of them.
    match q.get("w").filter(|w| !w.is_empty()) {
        Some(wanted) => {
            let v: Value = serde_json::from_str(&all).unwrap_or(Value::Null);
            let one = v.get(wanted).cloned().unwrap_or_else(|| json!({ "error": "Unknown widget" }));
            reply(serde_json::to_string(&one).unwrap_or_default())
        }
        None => reply(all),
    }
}

// --- the page ---------------------------------------------------------------------

/// Data for `templates/data_head.html`: version numbers for the page's scripts. The number is each
/// file's last-changed time, so browsers load the new file after a change (cache busting).
#[derive(Template)]
#[template(path = "data_head.html")]
struct DataHead {
    chart_theme_v: u64,
    data_widgets_v: u64,
    widget_loader_v: u64,
}

/// Data for `templates/data.html`: the year list for the year picker, a script version number, and
/// the HTML of the ten cards drawn on the server (made by `widgets.rs`).
#[derive(Template)]
#[template(path = "data.html")]
struct DataBody {
    years: Vec<i32>,
    chart_compare_v: u64,
    w_ai_insights: String,
    w_weather_forecast: String,
    w_iot_current: String,
    w_plant_weather: String,
    w_garden_intelligence: String,
    w_iot_heatgrid: String,
    w_iot_moisture: String,
    w_iot_soil: String,
    w_iot_weather: String,
    w_iot_water_use: String,
}

/// Draw the Data page itself (used when no `?something` matched).
///
/// Asks `widgets::render_all` for the server-drawn cards, fills the two templates, and wraps them
/// in the site layout. Takes the app data and the visitor; gives an HTML `Response`.
async fn render_page(state: &AppState, visitor: &Visitor) -> Response {
    let ws = widgets::render_all(state, false).await;
    let body = DataBody {
        // Years from this year down to 2020 for the year picker (`rev` reverses the order).
        years: (2020..=common::current_year()).rev().collect(),
        chart_compare_v: filemtime(&state.www, "js/chart-compare.js"),
        w_ai_insights: ws.ai_insights,
        w_weather_forecast: ws.weather_forecast,
        w_iot_current: ws.iot_current,
        w_plant_weather: ws.plant_weather,
        w_garden_intelligence: ws.garden_intelligence,
        w_iot_heatgrid: ws.iot_heatgrid,
        w_iot_moisture: ws.iot_moisture,
        w_iot_soil: ws.iot_soil,
        w_iot_weather: ws.iot_weather,
        w_iot_water_use: ws.iot_water_use,
    }
    .render()
    .unwrap_or_default();
    let mut page = Page::new(body);
    page.title = "Garden Data - Analytics".to_string();
    page.show_hero = false;
    page.extra_head = DataHead {
        chart_theme_v: filemtime(&state.www, "js/chart-theme.js"),
        data_widgets_v: filemtime(&state.www, "js/data-widgets.js"),
        widget_loader_v: filemtime(&state.www, "js/widget-loader.js"),
    }
    .render()
    .unwrap_or_default();
    Html(layout::render(state, page, visitor).await).into_response()
}

/// Route handler for `GET /data.php`.
///
/// The arguments are axum "extractors": `State` is the shared app data, `Extension` is the visitor
/// found by the sign-in layer, `Query` is the address parameters as a map. One `if` per
/// `?something`, in the PHP's order; the first match answers and `return` ends the function. If
/// none matches, the page is drawn.
pub async fn data_get(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    // `?ai_predict&plant_id=N`: ask the local model for a prediction about one plant.
    if q.contains_key("ai_predict") {
        if !AI_ENABLED || !brain::available(&state).await {
            return json_response(json!({ "error": "AI offline" }));
        }
        let pid = q.get("plant_id").map(|p| php_int(p)).unwrap_or(0);
        if pid == 0 {
            return json_response(json!({ "error": "plant_id required" }));
        }
        let row = sqlx::query("SELECT * FROM data_source WHERE id = ?").bind(pid).fetch_optional(&state.db).await;
        return match row {
            Err(e) => json_response(json!({ "error": hide(e) })),
            Ok(None) => json_response(json!({ "error": "Plant not found" })),
            Ok(Some(r)) => {
                let m = match row_to_json(&r) {
                    Value::Object(m) => m,
                    _ => Map::new(),
                };
                json_response(brain::predict(&state, &m).await.unwrap_or_else(|| json!({ "error": "AI returned no data" })))
            }
        };
    }
    // `?discover`: the model's garden discoveries; `warm` asks it to recompute.
    if q.contains_key("discover") {
        let warm = q.get("warm").map_or(false, |w| !w.is_empty() && w != "0");
        return json_response(brain::disc_run(&state, warm).await);
    }
    // `?warm_ai`: pre-fill the AI caches and report how many seconds it took.
    if q.contains_key("warm_ai") {
        let started = std::time::Instant::now();
        let ws = widgets::render_all(&state, true).await;
        let ok = !ws.garden_intelligence.is_empty();
        let d = brain::disc_run(&state, true).await;
        let disc_ok = !d["ai"].is_null();
        return json_response(json!({ "warmed": ok, "discoveries": disc_ok, "seconds": php_round(started.elapsed().as_secs_f64(), 1) }));
    }
    // `?ai_fuse`: the cross-tab plus the model's reading of it.
    if q.contains_key("ai_fuse") {
        return json_response(cross_tab(&state, &q, true).await);
    }
    // `?weather_forecast`: the saved forecast file, cached for five minutes.
    if q.contains_key("weather_forecast") {
        let body = match read_forecast(&state) {
            Ok(v) => v,
            Err(e) => json!({ "error": e }),
        };
        return with_cache(json_response(body), "public, max-age=300");
    }
    // Weather charts, demo sensor data, and the plain cross-tab.
    if q.contains_key("weather_data") {
        return weather_data(&state, &q).await;
    }
    if q.contains_key("mock_iot") {
        return json_response(mock_iot(&q));
    }
    if q.contains_key("cross_tab") {
        return json_response(cross_tab(&state, &q, false).await);
    }
    // Nothing matched: draw the page.
    render_page(&state, &visitor).await
}

// Tests. `cargo test` runs each `#[test]` function; `assert_eq!` fails the test if the two sides
// differ.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_and_matches_php() {
        assert_eq!(split_and("Bees, Butterflies and Hummingbirds"), vec!["Bees", " Butterflies", "Hummingbirds"]);
    }

    #[test]
    fn mock_iot_single_bed() {
        let mut q = HashMap::new();
        q.insert("sensor".to_string(), "soil_temp".to_string());
        q.insert("bed".to_string(), "Zen Garden".to_string());
        let v = mock_iot(&q);
        assert_eq!(v["datasets"].as_array().unwrap().len(), 1);
        assert_eq!(v["datasets"][0]["data"][0], json!(72));
        q.insert("sensor".to_string(), "bogus".to_string());
        assert!(mock_iot(&q)["error"].is_string());
    }

    #[test]
    fn integer_labels_become_numbers() {
        assert_eq!(php_key("2024"), json!(2024));
        assert_eq!(php_key("0"), json!(0));
        assert_eq!(php_key("007"), json!("007"));
        assert_eq!(php_key("6.5"), json!("6.5"));
        assert_eq!(php_key("Full Sun"), json!("Full Sun"));
    }
}
