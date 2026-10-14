# Where the Rust site is deliberately NOT identical to the PHP

Each entry: what PHP does, what Rust does, why.

## DEV-1: the legacy search endpoint no longer returns an empty body

- **PHP** (`plant_listing.php?ajax_list_search=1&term=...`): `SELECT *`
  pulls in two binary image columns (`qr_code`, `barcode_image`). PHP cannot
  turn raw image bytes into JSON, so `json_encode` fails and the response is
  empty for every search that matches a plant (checked 2026-10-05: "rose",
  "lily", "hosta" all returned 0 bytes).
- **Rust**: those two fields come back as `null`, the way `catalog.php` and
  `app_api.php` already handle them. The rest of the row is unchanged.
- **Why it matters little today**: the current Quick Search page does not
  call this endpoint any more (it calls `catalog.php?ajax_load`), so no
  visitor sees the bug. It is fixed because anything else that calls it, such
  as an old app version, would otherwise get nothing.
- **Checked**: plant ids and order equal a direct SQL run of the same query;
  future-plants rows are appended after, as in PHP.

## DEV-2: no `X-Powered-By` header

PHP announces its exact version on every response (`PHP/8.3.33`). That only
helps someone looking up attacks for that version. Rust does not send it.

## DEV-3: static files only from an allowlist

Apache never showed PHP source because it ran it. A plain file server would
show it. Rust serves only listed file types and refuses config, source,
`.bak`/`.pre-*` copies, hidden files and the `data/`, `database/`,
`archive/`, `tools/`, `cashe/` folders. This is the one place Rust is
*stricter* than the PHP site was allowed to be.

## DEV-4: post text is escaped after decoding (home page)

- **PHP** (`index.php`): a post is saved with its HTML characters escaped,
  then the home page decodes them again and prints the result **as raw
  HTML**. Anyone who can write a post can therefore run any HTML or script
  on the home page for every visitor.
- **Rust**: the text is decoded and then escaped, so it is shown as text.
- **Checked against the data**: both existing posts contain no HTML, only
  apostrophes, so every post looks the same as before. (Posts that someone
  deliberately wrote tags into would now show the tags as text; none exist.)
- The archive page (`blog.php`) already escaped the text in PHP and still does.

## DEV-5: guestbook limits are enforced by the server

The guestbook form says "20 characters" for the name and "100" for the note,
but only the browser enforced it. The server now does too (name 20, note
100), cuts tags out of both, and writes one entry at a time. The saved line
has exactly the format PHP wrote.

## Observation, not a deviation: the weather box may be blocked by the site's own security policy

`js/weather.js` and `js/home.js` call `api.zippopotam.us` and
`api.open-meteo.com`. The Content-Security-Policy the PHP site sends allows
only `connect-src 'self'` plus cdnjs, which blocks both. The Rust site sends
the same policy, so the behavior is identical. If the weather box never fills
in on the live site, this is the likely cause. Fixing it means adding those
two hosts to `connect-src`; that is a decision for Jason and Jun.

## DEV-6: the plant-by-id lookup works (it is empty on the live site)

- **PHP** (`catalog.php?ajax_load=1&plant_id=N`, used by QR-tag links and the
  plant window): the code calls `json_out()`, but that function is defined
  further down the file, inside an `if`, *after* this call runs. PHP stops
  with "undefined function" and the response is an empty body.
  (Checked 2026-10-05: empty for plants 1, 203 and a wish-list plant.)
- **Rust**: returns the plant, shaped exactly like the same plant in the
  list endpoint, plus `source_table`, or `{"error":"Plant not found"}`.
- **Checked**: 80 plants (current and wish list) compared with the live list
  endpoint's rows, 0 differences (`tools/verify_plant_lookup.py`).
- Worth testing on the live site: scan a QR tag from the garden. If it opens
  nothing today, this is why.

## DEV-7: the visitor cookie has two protective flags

PHP sets `PHPSESSID` with no `HttpOnly` or `SameSite`. Rust sends the same
cookie (same name, same id format, one day) plus `HttpOnly` and
`SameSite=Lax`, and `Secure` when the request came over HTTPS. A visitor who
already has a cookie keeps their favorites and likes, because the database
stores them under `Guest_<that id>`.

## Notes that are not deviations

- **Favorites and likes change data through a GET request** (`?toggle_favorite=5`).
  Kept, because `js/app.js` calls them that way and the JavaScript is not
  being rewritten. Another website could make a visitor's browser add or
  remove their own favorites. Low impact; the fix is to change both the
  script and the endpoint to POST together, a decision for later.
- **`toggle_review` is called by `js/app.js` but does not exist in
  `catalog.php`**, on either site. It returns the page, not JSON.

## DEV-8: the weather overlay on the Data page has data again

- **PHP** (`data.php?weather_data=1`): still carries the InfluxDB form of the
  query (`DATE_TRUNC`, a `time` column) from before the weather moved to
  MariaDB, so it fails and returns empty series. The comparison tool's
  "Overlay Weather Data" and the Plant Weather Detective's chart have been
  empty on the live site since then.
- **Rust**: the same two-step merge (each day collapsed, Open-Meteo first and
  NOAA filling gaps, then days rolled up to months) written for MariaDB
  against the `weather` table the collector writes.

## DEV-9: times are shown in the house's time zone

The PHP container has no time zone set, so it runs on UTC: the guestbook's
"Oct 5", the app's `ping` and `export` timestamps, the "Updated:" line on the
Air box. The Rust container runs with `TZ=America/New_York` (Jason's rule for
the Rust apps), so those read in local time. Dates stored in the database are
printed as stored on both.

## DEV-10: order among equal counts in the comparison tool

When two rows of a cross-tab have the same count, the database hands them
back in an order it does not promise, and it happens to hand the PHP's
prepared statement a different order than it hands the Rust one. The counts,
labels and totals are identical; only the order of tied rows can differ. It
was never defined on the PHP side either.

## DEV-11: `&` is written as `&#38;` in page text

The template engine escapes `&` as `&#38;` where PHP writes `&amp;`. Browsers
read both the same; the DOM comparison treats them as equal.

## DEV-12: tables are created at startup, not on every request

`plant_actions.php` and `index.php` ran `CREATE TABLE IF NOT EXISTS` on every
request. Rust does it once when it starts (and adds the sign-in tables
`garden_sessions`, `garden_users` and `login_attempts`).

## DEV-13: the barcode cache lives in the data folder, not /tmp

`plant_actions.php` kept `barcode_cache.json` in the system temp folder under
a fixed name; any other program on the machine could create or replace that
file first (Semgrep rule `rust.lang.security.temp-dir`). Rust keeps it in
`JUNSGARDEN_DATA_DIR`, which only the site writes and which is never served.
The lookups and answers are unchanged; the first lookup after the switch
simply starts with an empty cache.

## DEV-14: the garden has its own password login again (2026-10-06)

The PHP site had a password login until 2026-09-19, then used the household
sign-in (SSO). On 2026-10-06 Jason asked for passwords back, for Jun and
himself only, with password resets by email: that is `login.php` and
`accounts.rs` (see "The sign-in" in README.md). The old `app_api.php` login
(a plain username, with no proof needed on writes) is **not** restored:
writes from the app need the key the app gets when it logs in.

## Observations about the live site, kept as they are

- **Jun could not upload plant photos on the PHP site.** The uploader list
  in `catalog.php` was `Jason, Joshua`; the household sign-in named Jun's
  account "Jun", so the upload box told him only authorised admins may
  upload. On the Rust site the default `JUNSGARDEN_UPLOADERS` is now
  `Jason,Joshua,Jun`, so Jun can upload photos.
- "Set as default photo" is Jason-only (`plant_actions.php`). Kept.
- The "Views" filters, the Beds & Planner links and the quantity logger show
  only when signed in. Kept.
- `app_api.php?action=like` trusts the `username` the app sends. Kept; the
  app's contract depends on it.
- The forecast widget reads `/home/jason/AIWebsite/.../latest_prediction.json`
  and the Air box reads `/home/jason/iot/weather_latest.json`. Neither exists
  on the NUC, so the forecast box shows "file not found" and the Air box shows
  its sample numbers, on both sites. The paths are settings now
  (`JUNSGARDEN_FORECAST_FILE`, `JUNSGARDEN_WEATHER_LATEST`).
- The AI boxes depend on an Ollama model on the desktop PC (`OLLAMA_URL`).
  When the PC is gone they say the AI is offline, on both sites.
