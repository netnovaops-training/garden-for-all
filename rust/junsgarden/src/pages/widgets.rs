//! The Data page's server-rendered widgets (`sections/data/widget_*.php`).
//!
//! Each widget is one template copied from its PHP file plus the numbers
//! it needs, computed here the same way. The weather ones read the
//! `weather` table; the "IoT" soil and water-use ones are the PHP's own
//! mock data (there are no sensors) and say so on the card.
//!
//! HOW TO READ THIS FILE (for Jun). One section per card, each with a
//! `#[derive(Template)]` struct (the holes in `templates/w_*.html`) and a
//! function that computes the numbers and fills them. `render_all` at the
//! bottom runs them all for the page. The numbers that end up inside
//! `<script>` tags are turned into JSON with `json_encode`, which writes
//! them the way PHP did so the page's scripts see no difference.
//! URLs: this file answers no URL by itself. `pages/data.rs` (the Data page,
//! `/data.php`) calls `render_all` at the bottom and puts each finished card of
//! HTML into its page. The same cards are also asked for one at a time by the
//! page's scripts.
//!
//! Rust ideas used all through this file, explained once here:
//!   `String` is text the program owns; `&str` is borrowed text it only looks at.
//!   `Option<T>` is "maybe nothing": `Some(value)` or `None`. `unwrap_or(x)`
//!   means "use x when it is nothing"; `unwrap_or_default()` uses the empty value.
//!   `Vec<T>` is a growable list. `Map<String, Value>` is a JSON object.
//!   `async fn` can wait (for the database) without blocking others, and `.await`
//!   is the place where it waits.
//!   `x.render()` fills an askama template (the HTML with holes) with the struct.
//!   A `closure` such as `|k| ...` is a small function written in place.
//!   `format!("...{x}...")` builds text by putting values into the pattern.
//!   `php_num` and `php_round` print and round numbers the way PHP did, so the
//!   page shows the same digits as the old site.

// `use` lines bring names into scope so they can be written short.
use askama::Template;
use rand::RngExt;
use serde_json::{json, Map, Value};

use crate::brain;
use crate::pages::data::{num, read_forecast, weather_available, weather_query};
use crate::php::{he, json_encode, php_num, php_round};
use crate::rowjson::row_to_json;
use crate::AppState;

/// All ten finished cards of the Data page, as ready-to-send HTML text. One
/// field per card. `pub` means other files may read these fields. `render_all`
/// fills this and `pages/data.rs` places each field in the page.
pub struct Rendered {
    pub ai_insights: String,
    pub weather_forecast: String,
    pub iot_current: String,
    pub plant_weather: String,
    pub garden_intelligence: String,
    pub iot_heatgrid: String,
    pub iot_moisture: String,
    pub iot_soil: String,
    pub iot_weather: String,
    pub iot_water_use: String,
}

/// Make a short random id such as "wgt_air_4821". Takes a prefix. Gives back text.
/// A card uses it to give its HTML elements unique names, because the same card
/// can appear twice on a page.
fn uid(prefix: &str) -> String {
    format!("{prefix}{}", rand::rng().random_range(1000..=9999))
}

/// `round(floatval($x), $p)` for a value that may be missing.
/// Takes a number that may be missing and the decimal places to keep. Gives back
/// the rounded number, treating "missing" as 0 (PHP's floatval does the same).
fn r(v: Option<f64>, places: i32) -> f64 {
    php_round(v.unwrap_or(0.0), places)
}

/// A date string shown as `M j, Y`.
/// Takes a date as text, either "2026-10-05..." or a full timestamp with time
/// zone. Gives back a date such as "Oct 5, 2026". If the text is not a date it is
/// returned unchanged. `s.get(0..10)` takes the first ten characters safely.
fn date_mjy(s: &str) -> String {
    let d = chrono::NaiveDate::parse_from_str(s.get(0..10).unwrap_or(""), "%Y-%m-%d")
        .ok()
        .or_else(|| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|d| d.date_naive())
        });
    match d {
        Some(d) => d.format("%b %-d, %Y").to_string(),
        None => s.to_string(),
    }
}

/// Card 1: the weather forecast. Template fields: an error flag and message, the
/// dates, the three predicted numbers with plain-word descriptions, and the
/// position and width of the temperature bar.
// --- 1. weather forecast ----------------------------------------------------

// The struct below holds the "holes" of `templates/w_weather_forecast.html`.
#[derive(Template)]
#[template(path = "w_weather_forecast.html")]
struct WeatherForecast {
    has_err: bool,
    err: String,
    title_suffix: String,
    gen_at: String,
    tmax_f: String,
    tmin_f: String,
    prcp_in: String,
    temp_desc: String,
    prcp_desc: String,
    bar_left: String,
    bar_width: String,
    win_end: String,
}

/// Build card 1 from the forecast file written by the weather pipeline.
/// Takes the app state (it knows where the file is). Gives back the card's HTML.
/// Starts as an error card and turns into a normal card only if the file reads.
fn weather_forecast(state: &AppState) -> String {
    // 1. Start with an error card and neutral placeholder text.
    let mut t = WeatherForecast {
        has_err: true,
        err: String::new(),
        title_suffix: String::new(),
        gen_at: "—".into(),
        tmax_f: String::new(),
        tmin_f: String::new(),
        prcp_in: String::new(),
        temp_desc: String::new(),
        prcp_desc: String::new(),
        bar_left: String::new(),
        bar_width: String::new(),
        win_end: "—".into(),
    };
    // 2. Read the forecast file. `match` picks a branch by the result: file missing,
    // file unreadable, or good data. In the good case the JSON looks like
    // `{"predictions": {"tmax_f": .., "tmin_f": .., "prcp_in": ..}, ...}`.
    match read_forecast(state) {
        Err("Forecast file not found") => {
            t.err = "Forecast file not found. Run the pipeline to generate predictions.".into()
        }
        Err(_) => t.err = "Could not parse forecast file.".into(),
        Ok(data) => {
            t.has_err = false;
            // Read the three predicted numbers, using 0 when one is missing.
            let p = &data["predictions"];
            let tmax = p["tmax_f"].as_f64().unwrap_or(0.0);
            let tmin = p["tmin_f"].as_f64().unwrap_or(0.0);
            let prcp = p["prcp_in"].as_f64().unwrap_or(0.0);
            // Date the prediction is for, and when it was generated (shown in local time).
            t.title_suffix = format!(
                " — {}",
                date_mjy(data["prediction_for"].as_str().unwrap_or("—"))
            );
            if let Some(g) = data["generated_at"].as_str() {
                t.gen_at = chrono::DateTime::parse_from_rfc3339(g)
                    .map(|d| {
                        d.with_timezone(&chrono::Local)
                            .format("%b %-d, %Y %-I:%M %p")
                            .to_string()
                    })
                    .unwrap_or_else(|_| "—".into());
            }
            t.win_end = date_mjy(data["window_end"].as_str().unwrap_or("—"));
            // Rain wording from inches predicted: 0 dry, under 0.1 drizzle, under 0.3 light,
            // under 0.75 moderate, otherwise heavy.
            t.prcp_desc = if prcp == 0.0 {
                "☀️ Dry"
            } else if prcp < 0.1 {
                "🌦️ Light drizzle"
            } else if prcp < 0.3 {
                "🌧️ Light rain"
            } else if prcp < 0.75 {
                "🌧️ Moderate rain"
            } else {
                "⛈️ Heavy rain"
            }
            .into();
            // Temperature wording from the average of high and low (degrees F).
            let avg = (tmax + tmin) / 2.0;
            t.temp_desc = if avg < 32.0 {
                "🥶 Freezing"
            } else if avg < 45.0 {
                "🧥 Cold"
            } else if avg < 60.0 {
                "🍂 Cool"
            } else if avg < 75.0 {
                "😊 Comfortable"
            } else if avg < 85.0 {
                "☀️ Warm"
            } else {
                "🔥 Hot"
            }
            .into();
            // The bar runs from the low to the high, kept between 0 and 100 degrees, and is
            // at least 2 percent wide so it is always visible.
            let bar_min = tmin.clamp(0.0, 100.0);
            let bar_max = tmax.clamp(0.0, 100.0);
            t.bar_left = php_num(bar_min);
            t.bar_width = php_num((bar_max - bar_min).max(2.0));
            t.tmax_f = php_num(tmax);
            t.tmin_f = php_num(tmin);
            t.prcp_in = php_num(prcp);
        }
    }
    // 3. Fill the template with the values. `render()` can fail, so the result is
    // turned into empty text rather than stopping the whole page.
    t.render().unwrap_or_default()
}

// These two helpers read the latest sensor reading the weather collector wrote
// to a small JSON file.
// --- the collector's latest reading -----------------------------------------

/// Read the collector's latest reading. Takes the app state. Gives back
/// `Some(map)` of reading name to value, or `None` if the file is missing or not
/// a JSON object.
fn latest_reading(state: &AppState) -> Option<Map<String, Value>> {
    std::fs::read(&state.weather_latest)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .and_then(|v| {
            if let Value::Object(m) = v {
                Some(m)
            } else {
                None
            }
        })
}

/// Made-up reading used when the real file is missing, so the cards still show
/// something. Takes a flag: true gives a full reading with temperature and
/// pollen, false gives only the moisture values. Gives back a JSON object.
fn mock_reading(with_temperature: bool) -> Map<String, Value> {
    let mut m = Map::new();
    // `for (k, v) in [...]` loops over name and number pairs and inserts each into
    // the map as JSON.
    if with_temperature {
        for (k, v) in [
            ("temperature", 72.0),
            ("feels_like", 70.0),
            ("humidity", 55.0),
            ("dew_point", 52.0),
            ("precipitation", 0.0),
            ("rain", 0.0),
            ("snowfall", 0.0),
            ("wind_speed", 8.0),
            ("wind_gusts", 14.0),
            ("wind_direction", 220.0),
            ("cloud_cover", 30.0),
            ("pressure", 1015.2),
            ("uv_index", 5.0),
            ("visibility", 10.0),
            ("soil_temp_6in", 65.0),
            ("soil_moisture_6in", 0.32),
            ("grass_pollen", 12.0),
            ("tree_pollen", 8.0),
            ("weed_pollen", 3.0),
            ("aqi", 35.0),
        ] {
            m.insert(k.into(), json!(v));
        }
        // A timestamp for "now" (local time).
        m.insert(
            "timestamp".into(),
            json!(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        );
    } else {
        for (k, v) in [
            ("humidity", 55.0),
            ("dew_point", 52.0),
            ("precipitation", 0.0),
            ("rain", 0.0),
            ("snowfall", 0.0),
            ("soil_moisture_6in", 0.32),
            ("pressure", 1015.2),
        ] {
            m.insert(k.into(), json!(v));
        }
    }
    m
}

/// One small tile on the "air & temperature" and "water" cards: an icon, a name,
/// a value, a note below it, and a background colour.
struct Metric {
    icon: String,
    label: String,
    value: String,
    sub: String,
    color: String,
}

/// A named group of tiles. `default_on` is whether the card shows the group
/// open when the page loads.
struct Group {
    key: String,
    label: String,
    default_on: bool,
    metrics: Vec<Metric>,
}

/// Build one `Metric` from borrowed text and owned text. `.into()` converts the
/// `&str` arguments into owned `String` values.
fn metric(icon: &str, label: &str, value: String, sub: String, color: &str) -> Metric {
    Metric {
        icon: icon.into(),
        label: label.into(),
        value,
        sub,
        color: color.into(),
    }
}

// The next struct is the card 2 template and holds the values it prints.
// --- 2. air & temperature ---------------------------------------------------

/// Card 2: air and temperature (`templates/w_iot_current.html`).
#[derive(Template)]
#[template(path = "w_iot_current.html")]
struct IotCurrent {
    uid: String,
    has_timestamp: bool,
    timestamp: String,
    temp: String,
    feels: String,
    soil_t: String,
    zone_status: String,
    zone_icon: String,
    zone_color: String,
    soil_note: String,
    groups: Vec<Group>,
}

/// Build card 2 from the latest reading, or from the made-up reading if the real
/// one has no temperature. Takes the app state. Gives back the card's HTML.
fn iot_current(state: &AppState) -> String {
    // 1. Pick the reading. `filter` keeps it only if it has a temperature.
    let c = latest_reading(state)
        .filter(|c| c.contains_key("temperature"))
        .unwrap_or_else(|| mock_reading(true));
    // 2. `g("name")` reads one number, 0 if absent. Then round the main values.
    let g = |k: &str| num(&c, k).unwrap_or(0.0);
    let temp = r(Some(g("temperature")), 0);
    let feels = r(Some(g("feels_like")), 0);
    let soil_t = r(Some(g("soil_temp_6in")), 0);
    // 3. The frost zone from the air temperature in degrees F: 28 or lower is a
    // hard freeze, 32 frost, 40 cold, 55 cool, 85 ideal, 95 heat stress, hotter is
    // extreme. Each branch gives (message, colour, icon).
    let (zone_status, zone_color, zone_icon) = if temp <= 28.0 {
        ("HARD FREEZE", "#880E4F", "🥶")
    } else if temp <= 32.0 {
        ("FROST WARNING", "#C62828", "❄️")
    } else if temp <= 40.0 {
        ("Cold — protect tender plants", "#E65100", "🧥")
    } else if temp <= 55.0 {
        ("Cool — slow growth", "#F9A825", "🌤️")
    } else if temp <= 85.0 {
        ("Ideal growing range", "#2E7D32", "🌱")
    } else if temp <= 95.0 {
        ("Heat stress — water deeply", "#E65100", "🥵")
    } else {
        ("EXTREME HEAT", "#C62828", "🔥")
    };
    // 4. UV label: up to 2 low, up to 5 moderate, up to 7 high, above that very high.
    let uv = r(Some(g("uv_index")), 0);
    let uv_label = if uv <= 2.0 {
        "Low"
    } else if uv <= 5.0 {
        "Moderate"
    } else if uv <= 7.0 {
        "High"
    } else {
        "Very High"
    };
    let wind = r(Some(g("wind_speed")), 0);
    let gusts = r(Some(g("wind_gusts")), 0);
    let wind_dir = r(Some(g("wind_direction")), 0);
    // 5. Wind direction. A compass has 16 points of 22.5 degrees each, so divide by
    // 22.5, round, and wrap around with `rem_euclid(16)` so 360 degrees gives "N".
    let dirs = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    let compass = dirs[(php_round(wind_dir / 22.5, 0) as i64).rem_euclid(16) as usize];
    let cloud = r(Some(g("cloud_cover")), 0);
    let pressure = r(Some(g("pressure")), 1);
    let vis = r(Some(num(&c, "visibility").unwrap_or(10.0)), 1);
    let hum = r(Some(g("humidity")), 0);
    let dew = r(Some(g("dew_point")), 0);
    let grass = r(Some(g("grass_pollen")), 0);
    let tree = r(Some(g("tree_pollen")), 0);
    let weed = r(Some(g("weed_pollen")), 0);
    // 6. Pollen is the sum of grass, tree and weed. Over 50 is high, over 20 moderate.
    // Air quality: up to 50 good, up to 100 moderate, above that unhealthy.
    let total_pollen = grass + tree + weed;
    let pollen_level = if total_pollen > 50.0 {
        "High"
    } else if total_pollen > 20.0 {
        "Moderate"
    } else {
        "Low"
    };
    let aqi = r(Some(g("aqi")), 0);
    let aqi_label = if aqi <= 50.0 {
        "Good"
    } else if aqi <= 100.0 {
        "Moderate"
    } else {
        "Unhealthy"
    };

    // 7. Build three groups of tiles: Air, Wind & Atmosphere, Ground & Soil.
    // Colours are hex codes picked to match the old PHP card.
    let groups = vec![
        Group {
            key: "air".into(),
            label: "🌡️ Air".into(),
            default_on: true,
            metrics: vec![
                metric(
                    "🌡️",
                    "Temperature",
                    format!("{}°F", php_num(temp)),
                    format!("Feels {}°F", php_num(feels)),
                    "#FFF3E0",
                ),
                metric(
                    "💧",
                    "Humidity",
                    format!("{}%", php_num(hum)),
                    format!("Dew {}°F", php_num(dew)),
                    "#E3F2FD",
                ),
                metric(
                    "☀️",
                    "UV Index",
                    php_num(uv),
                    uv_label.into(),
                    if uv > 5.0 { "#FFF3E0" } else { "#F1F8E9" },
                ),
                metric(
                    "🌾",
                    "Pollen",
                    pollen_level.into(),
                    format!(
                        "G:{} T:{} W:{}",
                        php_num(grass),
                        php_num(tree),
                        php_num(weed)
                    ),
                    if total_pollen > 50.0 {
                        "#FFEBEE"
                    } else {
                        "#F3E5F5"
                    },
                ),
                metric(
                    "🫁",
                    "Air Quality",
                    php_num(aqi),
                    aqi_label.into(),
                    "#E0F7FA",
                ),
            ],
        },
        Group {
            key: "wind".into(),
            label: "💨 Wind & Atmosphere".into(),
            default_on: false,
            metrics: vec![
                metric(
                    "💨",
                    "Wind",
                    format!("{} mph", php_num(wind)),
                    format!("{compass} gusts {}", php_num(gusts)),
                    "#E8F5E9",
                ),
                metric(
                    "🔻",
                    "Pressure",
                    format!("{} hPa", php_num(pressure)),
                    if pressure > 1020.0 {
                        "High (stable)"
                    } else {
                        "Normal"
                    }
                    .into(),
                    "#FCE4EC",
                ),
                metric(
                    "☁️",
                    "Cloud Cover",
                    format!("{}%", php_num(cloud)),
                    if cloud < 20.0 {
                        "Clear sky"
                    } else {
                        "Overcast"
                    }
                    .into(),
                    "#F5F5F5",
                ),
                metric(
                    "👁️",
                    "Visibility",
                    format!("{} mi", php_num(vis)),
                    if vis > 5.0 { "Clear" } else { "Low visibility" }.into(),
                    "#ECEFF1",
                ),
            ],
        },
        Group {
            key: "ground".into(),
            label: "🌱 Ground & Soil".into(),
            default_on: false,
            metrics: vec![metric(
                "🌱",
                "Soil Temp",
                format!("{}°F", php_num(soil_t)),
                if soil_t < 45.0 {
                    "Too cold"
                } else if soil_t < 60.0 {
                    "Cool crops OK"
                } else {
                    "Warm crops OK"
                }
                .into(),
                "#EFEBE9",
            )],
        },
    ];
    // 8. Fill the template and render it.
    IotCurrent {
        uid: uid("wgt_air_"),
        has_timestamp: c.contains_key("timestamp"),
        timestamp: he(&c
            .get("timestamp")
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_default()),
        temp: php_num(temp),
        feels: php_num(feels),
        soil_t: php_num(soil_t),
        zone_status: zone_status.into(),
        zone_icon: zone_icon.into(),
        zone_color: zone_color.into(),
        soil_note: if soil_t >= 60.0 {
            "✅ Warm enough for all crops"
        } else if soil_t >= 45.0 {
            "🌿 Cool-season crops OK"
        } else {
            "❄️ Too cold to plant"
        }
        .into(),
        groups,
    }
    .render()
    .unwrap_or_default()
}

// --- 3. water & moisture ----------------------------------------------------

/// Card 3: water and moisture (`templates/w_iot_moisture.html`), with a 7-day
/// chart. The `trend_*` fields hold JSON text that the card's script reads.
#[derive(Template)]
#[template(path = "w_iot_moisture.html")]
struct IotMoisture {
    uid: String,
    has_timestamp: bool,
    timestamp: String,
    groups: Vec<Group>,
    trend_labels: String,
    trend_rain: String,
    trend_snow: String,
    trend_hum: String,
}

/// Build card 3. Takes the app state. Gives back the card's HTML.
async fn iot_moisture(state: &AppState) -> String {
    // 1. Pick the reading (real or made up) and round the values.
    let c = latest_reading(state)
        .filter(|c| c.contains_key("humidity"))
        .unwrap_or_else(|| mock_reading(false));
    let g = |k: &str| num(&c, k).unwrap_or(0.0);
    let hum = r(Some(g("humidity")), 0);
    let dew = r(Some(g("dew_point")), 0);
    let precip = r(Some(g("precipitation")), 2);
    let rain = r(Some(g("rain")), 2);
    let snow = r(Some(g("snowfall")), 1);
    let soil_m = r(Some(g("soil_moisture_6in") * 100.0), 0);
    // 2. Build the tile groups: Precipitation, and Moisture & Humidity.
    let groups = vec![
        Group {
            key: "precipitation".into(),
            label: "🌧️ Precipitation".into(),
            default_on: true,
            metrics: vec![
                metric(
                    "🌧️",
                    "Rain",
                    format!("{}\"", php_num(rain)),
                    if rain > 0.5 { "Heavy" } else { "Light" }.into(),
                    "#E3F2FD",
                ),
                metric(
                    "❄️",
                    "Snowfall",
                    format!("{}\"", php_num(snow)),
                    if snow > 0.0 { "Active" } else { "None" }.into(),
                    if snow > 0.0 { "#E8EAF6" } else { "#FAFAFA" },
                ),
                metric(
                    "💧",
                    "Total Precip",
                    format!("{}\"", php_num(precip)),
                    "Combined".into(),
                    "#E1F5FE",
                ),
            ],
        },
        Group {
            key: "moisture".into(),
            label: "💦 Moisture & Humidity".into(),
            default_on: true,
            metrics: vec![
                metric(
                    "💧",
                    "Humidity",
                    format!("{}%", php_num(hum)),
                    if hum > 80.0 {
                        "Very humid"
                    } else if hum > 60.0 {
                        "Comfortable"
                    } else {
                        "Dry"
                    }
                    .into(),
                    "#E3F2FD",
                ),
                metric(
                    "💦",
                    "Soil Moisture",
                    format!("{}%", php_num(soil_m)),
                    if soil_m < 30.0 {
                        "Needs water"
                    } else if soil_m > 70.0 {
                        "Saturated"
                    } else {
                        "Good"
                    }
                    .into(),
                    if soil_m < 30.0 { "#FFF3E0" } else { "#E0F7FA" },
                ),
                metric(
                    "💨",
                    "Dew Point",
                    format!("{}°F", php_num(dew)),
                    if dew > 65.0 {
                        "Muggy"
                    } else if dew > 55.0 {
                        "Comfortable"
                    } else {
                        "Dry air"
                    }
                    .into(),
                    "#ECEFF1",
                ),
            ],
        },
    ];

    // 3. The chart data: one entry per day for the last 7 days.
    let mut labels: Vec<Value> = Vec::new();
    let mut rain_v: Vec<Value> = Vec::new();
    let mut snow_v: Vec<Value> = Vec::new();
    let mut hum_v: Vec<Value> = Vec::new();
    // Only ask the database if the `weather` table exists and has data.
    if weather_available(state).await {
        // Daily average humidity and total rain and snow. `DATE(obs_time)` cuts the
        // timestamp to its day; `GROUP BY day` makes one row per day.
        let rows = weather_query(
            state,
            "SELECT DATE(obs_time) AS day, AVG(humidity) AS avg_hum, SUM(precipitation) AS total_precip, SUM(snowfall) AS total_snow
             FROM weather WHERE obs_time > NOW() - INTERVAL 7 DAY GROUP BY day ORDER BY day ASC",
        )
        .await;
        for row in &rows {
            let day = row.get("day").and_then(Value::as_str).unwrap_or("");
            let label = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d")
                .map(|d| d.format("%a").to_string())
                .unwrap_or_else(|_| chrono::Local::now().format("%a").to_string());
            labels.push(json!(label));
            rain_v.push(json!(r(num(row, "total_precip"), 2)));
            snow_v.push(json!(r(num(row, "total_snow"), 1)));
            hum_v.push(json!(r(num(row, "avg_hum"), 0)));
        }
    }
    // 4. If there is no real data, show a sample week so the chart is not empty.
    if labels.is_empty() {
        labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            .iter()
            .map(|d| json!(d))
            .collect();
        rain_v = [0.1, 0.0, 0.3, 0.8, 0.0, 0.0, 0.2]
            .iter()
            .map(|v| json!(v))
            .collect();
        snow_v = vec![json!(0); 7];
        hum_v = [62, 58, 70, 82, 55, 48, 60]
            .iter()
            .map(|v| json!(v))
            .collect();
    }
    // 5. Fill the template. `json_encode` writes lists as JSON text, as PHP did.
    IotMoisture {
        uid: uid("wgt_water_"),
        has_timestamp: c.contains_key("timestamp"),
        timestamp: he(&c
            .get("timestamp")
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_default()),
        groups,
        trend_labels: json_encode(&Value::Array(labels)),
        trend_rain: json_encode(&Value::Array(rain_v)),
        trend_snow: json_encode(&Value::Array(snow_v)),
        trend_hum: json_encode(&Value::Array(hum_v)),
    }
    .render()
    .unwrap_or_default()
}

// Card 4 below uses only made-up numbers (there are no soil sensors).
// --- 4. soil moisture (mock) -------------------------------------------------

/// Card 4: soil moisture by bed (`templates/w_iot_soil.html`). Mock data.
#[derive(Template)]
#[template(path = "w_iot_soil.html")]
struct IotSoil {
    labels_json: String,
    current_json: String,
    optimal_json: String,
    colors_json: String,
}

/// Build card 4. Takes nothing. Gives back the card's HTML. Each bar is blue at
/// 70 or more, green at 50 or more, orange below that.
fn iot_soil() -> String {
    let labels = [
        "Zen Garden",
        "River Garden",
        "Ko-Ja Garden",
        "Kona's Garden",
        "Korea Garden",
        "The Royal Run",
        "Jason's Terrace",
        "The 4 Season Garden",
        "Boo Boo's Garden",
        "Front Garden",
    ];
    let current = [72, 58, 65, 80, 44, 38, 71, 55, 62, 48];
    let optimal = [70, 65, 65, 70, 60, 55, 65, 60, 65, 60];
    let colors: Vec<Value> = current
        .iter()
        .map(|v| {
            json!(if *v >= 70 {
                "rgba(33,150,243,0.7)"
            } else if *v >= 50 {
                "rgba(76,175,80,0.7)"
            } else {
                "rgba(255,152,0,0.7)"
            })
        })
        .collect();
    IotSoil {
        labels_json: json_encode(&json!(labels)),
        current_json: json_encode(&json!(current)),
        optimal_json: json_encode(&json!(optimal)),
        colors_json: json_encode(&Value::Array(colors)),
    }
    .render()
    .unwrap_or_default()
}

// Card 5 shows monthly temperature and rain from the database.
// --- 5. weather trends --------------------------------------------------------

/// Card 5: monthly weather trends (`templates/w_iot_weather.html`). `live` tells
/// the card whether the numbers came from the database or are sample numbers.
#[derive(Template)]
#[template(path = "w_iot_weather.html")]
struct IotWeather {
    live: bool,
    months_json: String,
    temps_json: String,
    rain_json: String,
    snow_json: String,
    point_colors_json: String,
}

/// Build card 5. Takes the app state. Gives back the card's HTML.
/// Starts with a sample year, and replaces it if the database has rows.
async fn iot_weather(state: &AppState) -> String {
    let mut live = false;
    let mut months: Vec<Value> = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .map(|m| json!(m))
    .collect();
    let mut temps: Vec<f64> = vec![
        24.0, 27.0, 36.0, 49.0, 60.0, 70.0, 75.0, 73.0, 65.0, 53.0, 41.0, 29.0,
    ];
    let mut temps_json: Vec<Value> = temps.iter().map(|t| json!(*t as i64)).collect();
    let mut rain: Vec<Value> = [2.4, 2.1, 3.2, 3.5, 3.8, 3.9, 4.1, 3.7, 3.5, 3.3, 3.0, 2.6]
        .iter()
        .map(|v| json!(v))
        .collect();
    let mut snow: Vec<Value> = [
        12.5f64, 10.2, 8.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 3.0, 9.8,
    ]
    .iter()
    .map(|v| {
        if v.fract() == 0.0 {
            json!(*v as i64)
        } else {
            json!(v)
        }
    })
    .collect();
    // Monthly averages for the last 365 days. `DATE_FORMAT(obs_time, '%Y-%m-01')`
    // turns every timestamp into the first day of its month, so `GROUP BY month`
    // makes one row per month.
    if weather_available(state).await {
        let rows = weather_query(
            state,
            "SELECT DATE_FORMAT(obs_time, '%Y-%m-01') AS month, AVG(temperature) AS avg_temp, SUM(precipitation) AS total_rain, SUM(snowfall) AS total_snow
             FROM weather WHERE obs_time > NOW() - INTERVAL 365 DAY GROUP BY month ORDER BY month ASC",
        )
        .await;
        if !rows.is_empty() {
            live = true;
            months.clear();
            temps.clear();
            temps_json.clear();
            rain.clear();
            snow.clear();
            for row in &rows {
                let m = row.get("month").and_then(Value::as_str).unwrap_or("");
                months.push(json!(chrono::NaiveDate::parse_from_str(m, "%Y-%m-%d")
                    .map(|d| d.format("%b %Y").to_string())
                    .unwrap_or_default()));
                let t = r(num(row, "avg_temp"), 1);
                temps.push(t);
                temps_json.push(json!(t));
                rain.push(json!(r(num(row, "total_rain"), 2)));
                snow.push(json!(r(num(row, "total_snow"), 1)));
            }
        }
    }
    // Point colour: purple below 35 degrees, red otherwise.
    let colors: Vec<Value> = temps
        .iter()
        .map(|t| json!(if *t < 35.0 { "#9C27B0" } else { "#F44336" }))
        .collect();
    IotWeather {
        live,
        months_json: json_encode(&Value::Array(months)),
        temps_json: json_encode(&Value::Array(temps_json)),
        rain_json: json_encode(&Value::Array(rain)),
        snow_json: json_encode(&Value::Array(snow)),
        point_colors_json: json_encode(&Value::Array(colors)),
    }
    .render()
    .unwrap_or_default()
}

// Card 6 uses made-up water numbers (no real sensors).
// --- 6. water usage (mock) ----------------------------------------------------

/// Card 6: water use by bed (`templates/w_iot_water_use.html`). Mock data.
#[derive(Template)]
#[template(path = "w_iot_water_use.html")]
struct IotWaterUse {
    beds_json: String,
    actual_json: String,
    target_json: String,
    colors_json: String,
    total_weekly: i64,
    total_target: i64,
    over_under_text: String,
    monthly_est: i64,
    annual_est: String,
}

/// Build card 6. Takes nothing. Gives back the card's HTML. The weekly total is
/// the sum of the beds; the monthly estimate is that times 4 and the yearly
/// estimate is that times 26, both copied from the PHP.
fn iot_water_use() -> String {
    let beds = [
        "Zen Garden",
        "River Garden",
        "Ko-Ja Garden",
        "The Royal Run",
        "Korea Garden",
        "Kona's Garden",
        "The 4 Season Garden",
        "Jason's Terrace",
    ];
    let actual = [45, 32, 38, 28, 52, 41, 35, 22];
    let target = [40, 35, 35, 30, 40, 38, 30, 25];
    let total_weekly: i64 = actual.iter().sum();
    let total_target: i64 = target.iter().sum();
    let over = total_weekly - total_target;
    // A bar is red when a bed used more than its target, blue otherwise.
    let colors: Vec<Value> = actual
        .iter()
        .zip(target.iter())
        .map(|(a, t)| {
            json!(if a > t {
                "rgba(244,67,54,0.6)"
            } else {
                "rgba(33,150,243,0.6)"
            })
        })
        .collect();
    let annual = total_weekly * 26;
    IotWaterUse {
        beds_json: json_encode(&json!(beds)),
        actual_json: json_encode(&json!(actual)),
        target_json: json_encode(&json!(target)),
        colors_json: json_encode(&Value::Array(colors)),
        total_weekly,
        total_target,
        over_under_text: if over > 0 {
            format!("⚠️ +{over}")
        } else {
            format!("✅ {over}")
        },
        monthly_est: total_weekly * 4,
        annual_est: number_format(annual),
    }
    .render()
    .unwrap_or_default()
}

/// `number_format($n)`: thousands separated by commas.
/// Takes a whole number. Gives back text with commas, for example 12345 becomes
/// "12,345". It walks the digits and adds a comma whenever the number of digits
/// still to come is a multiple of 3.
fn number_format(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

// Card 7: a heat grid of average weather by hour of the day.
// --- 7. heat grid ---------------------------------------------------------------

/// Card 7: heat grid (`templates/w_iot_heatgrid.html`). The two `*_json` fields
/// hold a 30-day by-weekday grid and a 12-month grid.
#[derive(Template)]
#[template(path = "w_iot_heatgrid.html")]
struct IotHeatgrid {
    uid: String,
    live: bool,
    data30_json: String,
    data12_json: String,
}

/// `json_encode` of a PHP array with integer keys: a list when the keys are
/// 0..n-1 in order, otherwise an object keyed by the numbers.
/// Takes (number, value) pairs. Gives back a JSON list when the numbers are
/// exactly 0,1,2... in order, or a JSON object keyed by the numbers otherwise.
/// PHP's `json_encode` makes the same choice, and the page's script relies on it.
fn php_int_keyed(entries: Vec<(i64, Value)>) -> Value {
    let sequential = entries.iter().enumerate().all(|(i, (k, _))| *k == i as i64);
    if sequential {
        Value::Array(entries.into_iter().map(|(_, v)| v).collect())
    } else {
        let mut m = Map::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Value::Object(m)
    }
}

/// Turn one database row into the five numbers one grid cell shows: temperature,
/// humidity, rain, wind and UV, each rounded the way PHP rounded them.
fn heat_cell(row: &Map<String, Value>) -> Value {
    json!({
        "temp": r(num(row, "avg_temp"), 1),
        "hum": r(num(row, "avg_hum"), 0),
        "precip": r(num(row, "total_precip"), 2),
        "wind": r(num(row, "avg_wind"), 1),
        "uv": r(num(row, "avg_uv"), 1),
    })
}

/// Rows keyed (outer, hour) folded into the nested PHP array shape.
/// Takes the rows, the name of the column that groups them (weekday or month),
/// and a flag for subtracting 1 (months are 1 to 12 but the grid counts from 0).
/// Gives back a JSON structure: group, then hour of the day, then the cell.
fn heat_grid(rows: &[Map<String, Value>], outer_key: &str, zero_based: bool) -> Value {
    let mut outer: Vec<(i64, Vec<(i64, Value)>)> = Vec::new();
    for row in rows {
        // Work out which group (weekday or month) and which hour this row belongs to.
        let o = num(row, outer_key).unwrap_or(0.0) as i64 - if zero_based { 1 } else { 0 };
        let h = num(row, "hr").unwrap_or(0.0) as i64;
        let cell = heat_cell(row);
        // Find the group, creating it if it is new; replace or add the hour inside it.
        if let Some(e) = outer.iter_mut().find(|(k, _)| *k == o) {
            if let Some(c) = e.1.iter_mut().find(|(hk, _)| *hk == h) {
                c.1 = cell;
            } else {
                e.1.push((h, cell));
            }
        } else {
            outer.push((o, vec![(h, cell)]));
        }
    }
    php_int_keyed(
        outer
            .into_iter()
            .map(|(k, hours)| (k, php_int_keyed(hours)))
            .collect(),
    )
}

/// Build card 7. Takes the app state. Gives back the card's HTML.
/// It uses real data when the database has it, otherwise a made-up daily pattern.
async fn iot_heatgrid(state: &AppState) -> String {
    let mut live = false;
    let mut data30 = Value::Array(vec![]);
    let mut data12 = Value::Array(vec![]);
    if weather_available(state).await {
        // 30 days by weekday: `DAYOFWEEK` gives 1 for Sunday, so `- 1` makes Sunday 0.
        let rows30 = weather_query(
            state,
            "SELECT (DAYOFWEEK(obs_time) - 1) AS dow, HOUR(obs_time) AS hr, AVG(temperature) AS avg_temp, AVG(humidity) AS avg_hum,
                    SUM(precipitation) AS total_precip, AVG(wind_speed) AS avg_wind, AVG(uv_index) AS avg_uv
             FROM weather WHERE obs_time > NOW() - INTERVAL 30 DAY GROUP BY dow, hr ORDER BY dow, hr",
        )
        .await;
        // 12 months by month number, for the last 365 days.
        let rows12 = weather_query(
            state,
            "SELECT MONTH(obs_time) AS mon, HOUR(obs_time) AS hr, AVG(temperature) AS avg_temp, AVG(humidity) AS avg_hum,
                    SUM(precipitation) AS total_precip, AVG(wind_speed) AS avg_wind, AVG(uv_index) AS avg_uv
             FROM weather WHERE obs_time > NOW() - INTERVAL 365 DAY GROUP BY mon, hr ORDER BY mon, hr",
        )
        .await;
        // Real data exists only if the weekday query returned rows.
        if !rows30.is_empty() {
            live = true;
            data30 = heat_grid(&rows30, "dow", false);
            data12 = heat_grid(&rows12, "mon", true);
        }
    }
    if !live {
        // The sample numbers follow a sine wave: warm in the afternoon, cool at night.
        // The PHP's demo pattern, with its small random wobble.
        // `rng` makes small random wobbles so the sample looks natural, like the PHP.
        let mut rng = rand::rng();
        let pi = std::f64::consts::PI;
        let mut d30 = Vec::new();
        for d in 0..7 {
            let mut hours = Vec::new();
            for h in 0..24 {
                let hf = h as f64;
                let base =
                    55.0 + 15.0 * ((hf - 6.0) * pi / 12.0).sin() + rng.random_range(-3..=3) as f64;
                hours.push((h, json!({
                    "temp": php_round(base, 1),
                    "hum": php_round(70.0 - 20.0 * ((hf - 6.0) * pi / 12.0).sin() + rng.random_range(-5..=5) as f64, 0),
                    "precip": php_round((rng.random_range(0..=10) as f64 / 100.0).max(0.0), 2),
                    "wind": php_round(5.0 + 8.0 * (hf * pi / 12.0).sin().abs() + rng.random_range(-2..=2) as f64, 1),
                    "uv": php_round((8.0 * ((hf - 6.0).max(0.0) * pi / 12.0).sin()).max(0.0), 1),
                })));
            }
            d30.push((d, php_int_keyed(hours)));
        }
        // Typical average temperature for each month of the year, January first.
        let month_temps = [
            25.0, 28.0, 38.0, 52.0, 63.0, 73.0, 78.0, 75.0, 66.0, 54.0, 42.0, 30.0,
        ];
        let mut d12 = Vec::new();
        for (m, mt) in month_temps.iter().enumerate() {
            let mut hours = Vec::new();
            for h in 0..24 {
                let hf = h as f64;
                hours.push((h, json!({
                    "temp": php_round(mt + 10.0 * ((hf - 6.0) * pi / 12.0).sin(), 1),
                    "hum": php_round(65.0 - 15.0 * ((hf - 6.0) * pi / 12.0).sin(), 0),
                    "precip": php_round((rng.random_range(0..=20) as f64 / 100.0).max(0.0), 2),
                    "wind": php_round(6.0 + 5.0 * (hf * pi / 12.0).sin().abs(), 1),
                    "uv": php_round(((2.0 + m as f64 * 0.8) * ((hf - 6.0).max(0.0) * pi / 12.0).sin()).max(0.0), 1),
                })));
            }
            d12.push((m as i64, php_int_keyed(hours)));
        }
        data30 = php_int_keyed(d30);
        data12 = php_int_keyed(d12);
    }
    IotHeatgrid {
        uid: uid("heatgrid_"),
        live,
        data30_json: json_encode(&data30),
        data12_json: json_encode(&data12),
    }
    .render()
    .unwrap_or_default()
}

// --- 8. plant weather detective ---------------------------------------------------

/// One plant in the drop-down on the Plant Weather card: id, name, and whether it
/// has died.
struct PlantOpt {
    id: i64,
    name: String,
    died: bool,
}

/// Card 8: plant weather detective (`templates/w_plant_weather.html`).
/// `plants_json` carries the full plant details for the card's script.
#[derive(Template)]
#[template(path = "w_plant_weather.html")]
struct PlantWeather {
    uid: String,
    has_ai: bool,
    has_influx: bool,
    plants: Vec<PlantOpt>,
    plants_json: String,
}

/// Build card 8. Takes the app state and whether the local AI is available.
/// Gives back the card's HTML. It reads every plant from `data_source` (the main
/// plant table), sorted by name.
async fn plant_weather(state: &AppState, has_ai: bool) -> String {
    let rows = sqlx::query(
        "SELECT id, plant_name, year_planted, bloom_months, grow_months, sun_exposure, water_needs, zones, flower_bed, died,
                month_start, month_end, family, care_notes, deer_resistance, native_status, is_evergreen, growth_rate, plant_type, toxicity
         FROM data_source ORDER BY plant_name ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    // Turn each row into JSON, then pick out the id, name and died flag for the list.
    let all: Vec<Value> = rows.iter().map(row_to_json).collect();
    let plants: Vec<PlantOpt> = all
        .iter()
        .map(|p| PlantOpt {
            id: p["id"].as_i64().unwrap_or(0),
            name: p["plant_name"].as_str().unwrap_or("").to_string(),
            died: p["died"].as_i64().unwrap_or(0) != 0,
        })
        .collect();
    PlantWeather {
        uid: uid("pw_"),
        has_ai,
        has_influx: weather_available(state).await,
        plants,
        plants_json: json_encode(&Value::Array(all)),
    }
    .render()
    .unwrap_or_default()
}

// --- 9. garden intelligence ---------------------------------------------------------

/// One finding on the Garden Intelligence card: icon, colour, the sentence, and
/// a suggested action.
struct Insight {
    icon: String,
    color: String,
    text: String,
    action: String,
}

/// Card 9: garden intelligence (`templates/w_garden_intelligence.html`). It
/// joins weather per year with plant survival, and may include an AI summary.
#[derive(Template)]
#[template(path = "w_garden_intelligence.html")]
struct GardenIntelligence {
    live: bool,
    years_count: usize,
    plants_total: i64,
    has_ai: bool,
    ai_model: String,
    ai_headline: String,
    ai_findings: Vec<String>,
    ai_advice: Vec<String>,
    wins: Vec<String>,
    risks: Vec<String>,
    insights: Vec<Insight>,
    years_json: String,
    surv_json: String,
    wx_json: String,
}

/// Weather totals for one calendar year. `Option` fields are empty when no value
/// was measured. `#[derive(Default, Clone)]` lets Rust make an all-zero value and
/// copy it.
#[derive(Default, Clone)]
struct YearWx {
    avg_temp: f64,
    min_temp: f64,
    max_temp: f64,
    total_rain: f64,
    total_snow: f64,
    avg_humidity: f64,
    avg_wind: f64,
    frost_hours: Option<i64>,
    frost_days: Option<f64>,
    heat_hours: Option<i64>,
}

/// Takes text. Gives back the text with anything between `<` and `>` removed.
/// Used to clean AI replies before showing them.
fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Build card 9. Takes the app state and a flag saying whether the slow AI step
/// is allowed. Gives back the card's HTML.
async fn garden_intelligence(state: &AppState, allow_slow: bool) -> String {
    let mut wx: Vec<(String, YearWx)> = Vec::new();
    let mut dropped: Vec<String> = Vec::new();
    if weather_available(state).await {
        // 1. Weather per year for the last two years.
        let rows = weather_query(
            state,
            "SELECT YEAR(obs_time) AS yr, AVG(temperature) AS avg_temp, MIN(temperature) AS min_temp, MAX(temperature) AS max_temp,
                    SUM(precipitation) AS total_rain, SUM(snowfall) AS total_snow, AVG(humidity) AS avg_humidity, AVG(wind_speed) AS avg_wind,
                    COUNT(DISTINCT DATE(obs_time)) AS days_covered
             FROM weather WHERE obs_time > NOW() - INTERVAL 730 DAY GROUP BY yr ORDER BY yr DESC",
        )
        .await;
        for row in &rows {
            let yr = num(row, "yr")
                .map(|y| (y as i64).to_string())
                .unwrap_or_default();
            let days = num(row, "days_covered").unwrap_or(0.0) as i64;
            if yr.is_empty() {
                continue;
            }
            // Years with fewer than 300 days of readings are left out and listed in
            // `dropped`, so a part-year does not look like a cold or dry year.
            if days < 300 {
                dropped.push(format!("{yr} ({days} days)"));
                continue;
            }
            wx.push((
                yr,
                YearWx {
                    avg_temp: r(num(row, "avg_temp"), 1),
                    min_temp: r(num(row, "min_temp"), 1),
                    max_temp: r(num(row, "max_temp"), 1),
                    total_rain: r(num(row, "total_rain"), 1),
                    total_snow: r(num(row, "total_snow"), 1),
                    avg_humidity: r(num(row, "avg_humidity"), 0),
                    avg_wind: r(num(row, "avg_wind"), 1),
                    ..Default::default()
                },
            ));
        }
        // 2. Hours below freezing (32 F) per year. 24 frost hours count as one frost day.
        for row in &weather_query(state, "SELECT YEAR(obs_time) AS yr, COUNT(*) AS frost_hours FROM weather WHERE temperature < 32 AND obs_time > NOW() - INTERVAL 730 DAY GROUP BY yr").await {
            let yr = num(row, "yr").map(|y| (y as i64).to_string()).unwrap_or_default();
            let hours = num(row, "frost_hours").unwrap_or(0.0) as i64;
            if let Some((_, w)) = wx.iter_mut().find(|(y, _)| *y == yr) {
                w.frost_hours = Some(hours);
                w.frost_days = Some(php_round(hours as f64 / 24.0, 0));
            }
        }
        // 3. Hours above 90 F per year.
        for row in &weather_query(state, "SELECT YEAR(obs_time) AS yr, COUNT(*) AS heat_hours FROM weather WHERE temperature > 90 AND obs_time > NOW() - INTERVAL 730 DAY GROUP BY yr").await {
            let yr = num(row, "yr").map(|y| (y as i64).to_string()).unwrap_or_default();
            if let Some((_, w)) = wx.iter_mut().find(|(y, _)| *y == yr) {
                w.heat_hours = Some(num(row, "heat_hours").unwrap_or(0.0) as i64);
            }
        }
    }
    let live = !wx.is_empty();

    // 4. Plant survival by the year each plant was planted. `LEFT(year_planted,4)`
    // keeps the first four characters (the year). Each `CASE WHEN` counts rows that
    // meet a condition (died or alive). Each tuple kept is (year, alive, died, total,
    // percent alive). The `for row in <query>...` loop runs once per result row.
    // Plant outcomes.
    let mut surv: Vec<(String, i64, i64, i64, f64)> = Vec::new(); // yr, alive, died, total, rate
    for row in sqlx::query(
        "SELECT LEFT(year_planted,4) AS yr, SUM(CASE WHEN died=1 THEN 1 ELSE 0 END) AS died, SUM(CASE WHEN died=0 THEN 1 ELSE 0 END) AS alive, COUNT(*) AS total
         FROM data_source WHERE year_planted != '' GROUP BY yr ORDER BY yr DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .iter()
    {
        // Turn the row into a JSON object; skip it (`continue`) if it is not one.
        let m = match row_to_json(row) {
            Value::Object(m) => m,
            _ => continue,
        };
        let yr = m.get("yr").map(|v| v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string())).unwrap_or_default();
        let died = num(&m, "died").unwrap_or(0.0) as i64;
        let alive = num(&m, "alive").unwrap_or(0.0) as i64;
        let total = num(&m, "total").unwrap_or(0.0) as i64;
        // Survival percent, guarded so a total of 0 cannot divide by zero.
        let rate = if total > 0 { php_round(alive as f64 / total as f64 * 100.0, 0) } else { 0.0 };
        surv.push((yr, alive, died, total, rate));
    }
    // 5. The 20 most recently planted plants that died, for the sun-exposure check.
    // `filter_map` keeps only the rows that turned into a JSON object.
    let dead: Vec<Map<String, Value>> = sqlx::query(
        "SELECT plant_name, family, sun_exposure, water_needs, zones, year_planted, flower_bed FROM data_source WHERE died = 1 ORDER BY year_planted DESC LIMIT 20",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .iter()
    .filter_map(|r| if let Value::Object(m) = row_to_json(r) { Some(m) } else { None })
    .collect();
    // 6. Death counts grouped by sun exposure and by water needs (see `by_col`).
    let by_sun = by_col("sun_exposure", state).await;
    let by_water = by_col("water_needs", state).await;

    // A closure giving a colour for a severity word. `_` matches anything else.
    let sev_color = |s: &str| match s {
        "high" => "#C62828",
        "medium" => "#E65100",
        "low" => "#2E7D32",
        _ => "#888",
    };
    // 7. Build the findings. `insights` are weather and plant links, `wins` are good
    // news, `risks` are bad news.
    let mut insights: Vec<Insight> = Vec::new();
    let mut wins: Vec<String> = Vec::new();
    let mut risks: Vec<String> = Vec::new();
    // Closure: how many plants died among those planted in year `yr`.
    let deaths_for = |yr: &str| {
        surv.iter()
            .find(|(y, ..)| y == yr)
            .map(|(_, _, d, _, _)| *d)
            .unwrap_or(0)
    };

    // Only compare weather with plants when real weather data exists.
    if live {
        // Frost years: a year with frost days and plant deaths. Severity: more than 5
        // deaths is high, more than 2 medium, otherwise low. The advice depends on the
        // lowest temperature that year.
        for (yr, w) in &wx {
            let deaths = deaths_for(yr);
            let frost_days = w.frost_days.unwrap_or(0.0);
            let min_temp = w.min_temp;
            if deaths > 0 && frost_days > 0.0 {
                let sev = if deaths > 5 {
                    "high"
                } else if deaths > 2 {
                    "medium"
                } else {
                    "low"
                };
                insights.push(Insight {
                    icon: "❄️".into(),
                    color: sev_color(sev).into(),
                    text: format!("<strong>{yr}</strong> had <strong>{} frost days</strong> (low of {}°F) — <strong>{deaths} plants died</strong>.", php_num(frost_days), php_num(min_temp)),
                    action: if min_temp < 10.0 {
                        "Consider cold frames or row covers for Zone 6+ plants."
                    } else if min_temp < 20.0 {
                        "Mulch heavily before December for root protection."
                    } else {
                        "Most losses may be from other causes — check drainage."
                    }
                    .into(),
                });
            }
        }
        // Hot years: more than 50 hours above 90 F. Count the dead full-sun plants
        // planted that year. More than 200 hot hours is high severity.
        for (yr, w) in &wx {
            let heat = w.heat_hours.unwrap_or(0);
            if heat > 50 {
                let full_sun_died = dead
                    .iter()
                    .filter(|p| {
                        p.get("year_planted")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .starts_with(yr.as_str())
                            && p.get("sun_exposure")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_lowercase()
                                .contains("full sun")
                    })
                    .count();
                insights.push(Insight {
                    icon: "🔥".into(),
                    color: sev_color(if heat > 200 { "high" } else { "medium" }).into(),
                    text: format!(
                        "<strong>{yr}</strong> had <strong>{} days above 90°F</strong>{}",
                        php_num(php_round(heat as f64 / 24.0, 0)),
                        if full_sun_died > 0 { format!(" — {full_sun_died} full-sun plants died.") } else { ".".into() }
                    ),
                    action: "Deep water early morning on 90°F+ days. Shade cloth helps sensitive plants.".into(),
                });
            }
        }
        // Rain years: under 30 inches is dry (under 20 is high severity); over 50
        // inches is wet. 35 inches is the Clifton Park average the PHP card used.
        for (yr, w) in &wx {
            let rain = w.total_rain;
            if rain < 30.0 {
                insights.push(Insight {
                    icon: "🏜️".into(),
                    color: sev_color(if rain < 20.0 { "high" } else { "medium" }).into(),
                    text: format!("<strong>{yr}</strong> had only <strong>{}\" of rain</strong> — below the 35\" Clifton Park average.", php_num(rain)),
                    action: "High water-need plants may have struggled. Consider drip irrigation for dry years.".into(),
                });
            } else if rain > 50.0 {
                insights.push(Insight {
                    icon: "🌊".into(),
                    color: sev_color("medium").into(),
                    text: format!("<strong>{yr}</strong> had <strong>{}\" of rain</strong> — well above average.", php_num(rain)),
                    action: "Check for root rot in poorly-drained beds. Fungal issues more likely.".into(),
                });
            }
        }
        // Best and worst year by survival rate. Years with fewer than 3 plants planted
        // (`s.3 < 3`) are ignored because the rate would mean little. `s.3` and `s.4`
        // are the fourth and fifth items of the tuple.
        let mut best: Option<&(String, i64, i64, i64, f64)> = None;
        let mut worst: Option<&(String, i64, i64, i64, f64)> = None;
        for s in &surv {
            if s.3 < 3 {
                continue;
            }
            if best.is_none_or(|b| s.4 > b.4) {
                best = Some(s);
            }
            if worst.is_none_or(|w| s.4 < w.4) {
                worst = Some(s);
            }
        }
        if let Some(b) = best {
            if let Some((_, w)) = wx.iter().find(|(y, _)| *y == b.0) {
                wins.push(format!(
                    "🏆 <strong>Best year: {}</strong> ({}% survival) — Avg {}°F, {}\" rain.",
                    b.0,
                    php_num(b.4),
                    php_num(w.avg_temp),
                    php_num(w.total_rain)
                ));
            }
        }
        if let Some(wst) = worst {
            if best.is_none_or(|b| b.0 != wst.0) {
                if let Some((_, w)) = wx.iter().find(|(y, _)| *y == wst.0) {
                    risks.push(format!("📉 <strong>Toughest year: {}</strong> ({}% survival) — Avg {}°F, {}\" rain.", wst.0, php_num(wst.4), php_num(w.avg_temp), php_num(w.total_rain)));
                }
            }
        }
        // Groups with at least 5 plants and a death rate over 15 percent are risks.
        for (k, cnt, died) in &by_sun {
            if *cnt >= 5 && *died > 0 {
                let rate = php_round(*died as f64 / *cnt as f64 * 100.0, 0);
                if rate > 15.0 {
                    risks.push(format!("☀️ <strong>{k}</strong> plants have a <strong>{}% death rate</strong> ({died}/{cnt}). Check if placement matches actual sun exposure.", php_num(rate)));
                }
            }
        }
        for (k, cnt, died) in &by_water {
            if *cnt >= 5 && *died > 0 {
                let rate = php_round(*died as f64 / *cnt as f64 * 100.0, 0);
                if rate > 15.0 {
                    risks.push(format!("💧 <strong>{k}</strong> water-need plants: <strong>{}% death rate</strong> ({died}/{cnt}). May need adjusted irrigation.", php_num(rate)));
                }
            }
        }
    // No real weather: show one note asking for a weather source.
    } else {
        insights.push(Insight {
            icon: "📡".into(),
            color: sev_color("low").into(),
            text: "Connect InfluxDB to see weather-correlated plant insights.".into(),
            action: "Weather data enables frost analysis, heat stress detection, and rainfall correlation.".into(),
        });
    }

    // 8. Optional: ask the local AI (Ollama, through `brain.rs`) to summarise the
    // facts above. It fails quietly, and `ai` stays `None` if it is not running.
    // `strip_tags` removes our HTML bold tags so the AI reads plain sentences.
    // The model's reading of the facts above.
    let mut ai: Option<Value> = None;
    if live && brain::available(state).await {
        let mut ctx: Vec<(String, Vec<String>)> = Vec::new();
        if !wins.is_empty() {
            ctx.push((
                "What went well".into(),
                wins.iter().map(|w| strip_tags(w)).collect(),
            ));
        }
        if !risks.is_empty() {
            ctx.push((
                "What went badly".into(),
                risks.iter().map(|w| strip_tags(w)).collect(),
            ));
        }
        if !insights.is_empty() {
            ctx.push((
                "Weather correlations found".into(),
                insights.iter().map(|i| strip_tags(&i.text)).collect(),
            ));
        }
        ctx.push((
            "Weather by year".into(),
            wx.iter()
                .map(|(yr, w)| {
                    format!(
                        "{yr}: average {} F, low {} F, high {} F, {} in rain, {} in snow",
                        php_num(w.avg_temp),
                        php_num(w.min_temp),
                        php_num(w.max_temp),
                        php_num(w.total_rain),
                        php_num(w.total_snow)
                    )
                })
                .collect(),
        ));
        ctx.push((
            "Survival by year".into(),
            surv.iter()
                .map(|(yr, _, _, total, rate)| {
                    format!("{yr}: {}% of {total} plants survived", php_num(*rate))
                })
                .collect(),
        ));
        if !dropped.is_empty() {
            ctx.push((
                "Years excluded as incomplete".into(),
                dropped
                    .iter()
                    .map(|y| {
                        format!(
                            "{y} of weather recorded - too little of the year to compare fairly"
                        )
                    })
                    .collect(),
            ));
        }
        // The reply is JSON with a model name, a headline, findings and advice.
        ai = brain::insights(state, &ctx, allow_slow).await;
    }

    // 9. Turn the numbers into JSON text for the card's script, keyed by year.
    let years_json = json_encode(&Value::Array(
        wx.iter()
            .map(|(y, _)| json!(y.parse::<i64>().unwrap_or(0)))
            .collect(),
    ));
    let mut surv_map = Map::new();
    for (yr, alive, died, total, rate) in &surv {
        surv_map.insert(
            yr.clone(),
            json!({ "alive": alive, "died": died, "total": total, "rate": rate }),
        );
    }
    let mut wx_map = Map::new();
    for (yr, w) in &wx {
        let mut m = Map::new();
        m.insert("avg_temp".into(), json!(w.avg_temp));
        m.insert("min_temp".into(), json!(w.min_temp));
        m.insert("max_temp".into(), json!(w.max_temp));
        m.insert("total_rain".into(), json!(w.total_rain));
        m.insert("total_snow".into(), json!(w.total_snow));
        m.insert("avg_humidity".into(), json!(w.avg_humidity));
        m.insert("avg_wind".into(), json!(w.avg_wind));
        if let Some(h) = w.frost_hours {
            m.insert("frost_hours".into(), json!(h));
            m.insert("frost_days".into(), json!(w.frost_days.unwrap_or(0.0)));
        }
        if let Some(h) = w.heat_hours {
            m.insert("heat_hours".into(), json!(h));
        }
        wx_map.insert(yr.clone(), Value::Object(m));
    }
    // A closure that turns an optional JSON list into a list of text lines.
    let str_list = |v: Option<&Value>| {
        v.and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    // 10. Fill the card's template and render it.
    GardenIntelligence {
        live,
        years_count: wx.len(),
        plants_total: surv.iter().map(|s| s.3).sum(),
        has_ai: ai.is_some(),
        ai_model: ai
            .as_ref()
            .and_then(|a| a.get("model"))
            .and_then(Value::as_str)
            .unwrap_or("local AI")
            .to_string(),
        ai_headline: ai
            .as_ref()
            .and_then(|a| a.get("headline"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        ai_findings: str_list(ai.as_ref().and_then(|a| a.get("findings"))),
        ai_advice: str_list(ai.as_ref().and_then(|a| a.get("advice"))),
        wins,
        risks,
        insights,
        years_json,
        surv_json: json_encode(&Value::Object(surv_map)),
        wx_json: json_encode(&Value::Object(wx_map)),
    }
    .render()
    .unwrap_or_default()
}

// Card 10 is hidden on the page, so it only has a stub.
// --- 10. AI insights (hidden on the page) -------------------------------------------

/// One bed's AI accuracy row on card 10 (colours included).
struct BedStat {
    name: String,
    plants: i64,
    acc: i64,
    bg: String,
    acc_color: String,
    acc_bg: String,
    conf: i64,
}

/// One plant the AI was unsure about on card 10.
struct Uncertain {
    plant_name: String,
    flower_bed: String,
    actual: String,
    predicted: String,
    conf: i64,
}

/// Card 10: AI insights (`templates/w_ai_insights.html`). Hidden on the page.
#[derive(Template)]
#[template(path = "w_ai_insights.html")]
struct AiInsights {
    brain_up: bool,
    target: String,
    has_overall_acc: bool,
    overall_acc: i64,
    total_plants: i64,
    classes: Vec<String>,
    bed_stats: Vec<BedStat>,
    uncertain: Vec<Uncertain>,
}

/// Build card 10. Takes whether the AI service is up. Gives back the card's HTML.
fn ai_insights(brain_up: bool) -> String {
    // The old classifier's per-bed accuracy no longer exists (brain.php
    // answers /health with a status only), so the lists are empty, as on
    // the live site. The widget is hidden on the page in any case.
    AiInsights {
        brain_up,
        target: "water_needs".into(),
        has_overall_acc: false,
        overall_acc: 0,
        total_plants: 0,
        classes: Vec::new(),
        bed_stats: Vec::new(),
        uncertain: Vec::new(),
    }
    .render()
    .unwrap_or_default()
}

/// Every widget's HTML. `allow_slow` is only true for the warmer.
/// Takes the app state and a flag. Gives back a `Rendered` with all ten cards.
/// Each `.await` is a wait for the database or the AI service.
pub async fn render_all(state: &AppState, allow_slow: bool) -> Rendered {
    let brain_up = brain::available(state).await;
    Rendered {
        ai_insights: ai_insights(brain_up),
        weather_forecast: weather_forecast(state),
        iot_current: iot_current(state),
        plant_weather: plant_weather(state, brain_up).await,
        garden_intelligence: garden_intelligence(state, allow_slow).await,
        iot_heatgrid: iot_heatgrid(state).await,
        iot_moisture: iot_moisture(state).await,
        iot_soil: iot_soil(),
        iot_weather: iot_weather(state).await,
        iot_water_use: iot_water_use(),
    }
}

/// Death counts grouped by one column (sun exposure or water needs).
/// Takes the column to group by (a fixed name, never from a visitor, because it
/// picks the SQL text) and the app state. Gives back (value, count, died) tuples,
/// most common first. Used by card 9.
async fn by_col(col: &'static str, state: &AppState) -> Vec<(String, i64, i64)> {
    let sql = if col == "sun_exposure" {
        "SELECT sun_exposure AS k, COUNT(*) AS cnt, SUM(CASE WHEN died=1 THEN 1 ELSE 0 END) AS died FROM data_source WHERE sun_exposure != '' GROUP BY sun_exposure ORDER BY cnt DESC"
    } else {
        "SELECT water_needs AS k, COUNT(*) AS cnt, SUM(CASE WHEN died=1 THEN 1 ELSE 0 END) AS died FROM data_source WHERE water_needs IS NOT NULL AND TRIM(water_needs) != '' GROUP BY water_needs ORDER BY cnt DESC"
    };
    // Run the query, turn each row into JSON, then read the three numbers from it.
    sqlx::query(sql)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .iter()
        .filter_map(|r| {
            if let Value::Object(m) = row_to_json(r) {
                Some(m)
            } else {
                None
            }
        })
        .map(|m| {
            (
                m.get("k")
                    .map(|v| {
                        v.as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| v.to_string())
                    })
                    .unwrap_or_default(),
                num(&m, "cnt").unwrap_or(0.0) as i64,
                num(&m, "died").unwrap_or(0.0) as i64,
            )
        })
        .collect()
}

// Tests run only with `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    // Checks the list-versus-object choice, as PHP's `json_encode` makes it.
    #[test]
    fn int_keyed_arrays_become_lists_only_when_sequential() {
        assert_eq!(
            php_int_keyed(vec![(0, json!(1)), (1, json!(2))]),
            json!([1, 2])
        );
        assert_eq!(
            php_int_keyed(vec![(0, json!(1)), (2, json!(2))]),
            json!({ "0": 1, "2": 2 })
        );
    }

    // Checks comma grouping of numbers.
    #[test]
    fn number_format_adds_commas() {
        assert_eq!(number_format(7618), "7,618");
        assert_eq!(number_format(999), "999");
        assert_eq!(number_format(1234567), "1,234,567");
    }

    // Checks the water card still shows the same totals as the PHP did.
    #[test]
    fn water_use_constants_match_the_php() {
        let html = iot_water_use();
        assert!(html.contains("Weekly Total: 293 gal"));
        assert!(html.contains("⚠️ +20 gal"));
        assert!(html.contains("~7,618 gal"));
    }
}
