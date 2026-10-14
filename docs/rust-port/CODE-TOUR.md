# A tour of the code, for Jun

This is how the site is put together, file by file, in the order that makes
sense to read it. Every file also starts with a comment saying what it does.
Read this once; then when something on the site needs changing, the table at
the bottom says which file to open.

## The shape of it

A visitor's browser asks for a page, say `/catalog.php`. The program
(`rust/junsgarden/`) answers in four steps:

1. **`main.rs`** receives the request and decides which page function
   handles that address.
2. **`session.rs`** works out who is asking: a visitor (identified by a
   cookie) or Jun/Jason (logged in with their garden password, or the app
   with its key).
3. The **page function** (in `src/pages/`) asks the database what it needs
   and fills in a **template** (in `templates/`), which is the page's HTML
   with holes in it.
4. **`layout.rs`** wraps the page body in the frame every page shares
   (the head, the green header, the hero photos, the footer) and sends it.

Static files (the stylesheet, the scripts, the photos, the PDFs) are not in
the program at all. They sit in the site folder on disk, and `main.rs` serves
them as they are, if their type is on the allowed list.

## The files, in reading order

### `src/main.rs`: the front door
- Reads the settings (environment variables; the list is at the top).
- Opens the database connection and creates the few tables the site makes
  for itself (`ensure_tables`).
- `servable()`: the allowed list of file types. Anything not on it is a 404,
  which is how the source code and the backup copies in the site folder stay
  private.
- The **router**: one line per address, `.route("/blog.php", get(blog::blog)...)`.
  To add a page, add a line here and a function in `src/pages/`.
- `page_headers`: the security headers every page carries.

### `src/session.rs`: who is asking
- The `PHPSESSID` cookie (same name as PHP used, so old cookies still work).
- Sessions: after a good login a row goes into `garden_sessions`, under a
  **new** session id, and that person can edit. `Visitor::is_admin()` is the
  one question every page asks.
- App keys: the Android app logs in once and then sends its 64-character
  key as `Authorization: Bearer <key>`; this file checks it.
- It no longer trusts any `X-Auth-Email` header (that was the household
  sign-in, used until 2026-10-06).
- `logout_redirect`: the Logout button.

### `src/accounts.rs`: the two accounts and their passwords
- The accounts come from the `JUNSGARDEN_USERS` setting (GARDEN_USERS in `.env`; Jun only by
  default). Nobody can create one from the website.
- The password check (passwords are stored only as Argon2 hashes in
  `garden_users`).
- The lockout: failed logins go into `login_attempts`; 5 from one address
  or 10 against one name within 15 minutes locks login for 15 minutes.
- The "Forgot password? / First time?" emails: a one-time link, good for 30
  minutes, stored only as a SHA-256 hash, sent by SMTP (the
  `JUNSGARDEN_SMTP_*` settings). With no mail settings the email is not sent
  and the log says "no outbox configured".

### `src/pages/login.rs` and `templates/login.html`: the login page
`/login.php`: log in, and "Forgot password? / First time?" (ask for a link,
then choose a password of at least 10 characters).

### `src/layout.rs` and `templates/layout.html`, `header.html`, `footer.html`
The frame. `Page` is what a page hands over (title, body, whether to show
the hero banner). The hero photos come from `images/HeroBanner/`, the "Did
you know?" facts from `data/facts.txt`, the app version from
`apps/version.json`.

### `src/php.rs`: small helpers that behave exactly like PHP's
`he()` (escape text for HTML), `nl2br`, `urlencode`, `json_encode` with
PHP's quirks, `php_int` (how PHP reads a number out of a form field). They
exist so the Rust pages produce the same bytes the PHP pages did.

### `src/formdata.rs`: what a form sent
Turns a request body into `fields` (text inputs) and `files` (photos),
whether the form was plain or had attachments.

### `src/images.rs`: photos
Converting an upload to WebP, naming plant photos `Plant_Family_01.webp`,
rotating a photo, making the app's thumbnails. Same quality numbers as the
PHP used.

### `src/rowjson.rs`
A database row as JSON, typed the way PHP's `json_encode` would type it.
Used by every endpoint that returns plant rows.

### `src/normalise.rs`: one vocabulary
The category words ("Partial Shade", "Low to Average") so the dashboard
cards and the comparison tool agree. Read the comments in it before
changing a chart.

### `src/brain.rs`: the AI
Talks to the local model (Ollama). Every call fails soft: no model, no AI
box, nothing else affected. Also the "discover" scan (plain statistics).

### `src/pages/`: one file per page
| File | Page | What is in it |
|---|---|---|
| `index.rs` | Home (`/`, `index.php`, `/sso-login`) | posts, weather box, tip cards, guestbook, the login box, and the post editor for logged-in people; `/sso-login` is only kept so old bookmarks land here |
| `login.rs` | `login.php` | the login page and "Forgot password? / First time?" |
| `blog.rs` | `blog.php` | the archive of posts |
| `plant_listing.rs` | `plant_listing.php` | Quick Search |
| `catalog.rs` | `catalog.php` | the Full Catalog and every call `js/app.js` makes: the plant list, search modes, favorites, likes, notes, the plant editor, photo uploads, quantity log |
| `gallery.rs` | `gallery.php` | the photo gallery (list from `images/Gallery/tags_cache.csv`) |
| `data.rs` | `data.php`, `widget_data.php` | the Data page: cross-tab, weather series, the 24 plant widgets' numbers |
| `widgets.rs` | the Data page's nine server-drawn cards | weather forecast, air, moisture, heat grid, trends, plant detective, garden intelligence |
| `comments.rs` | `comments.php` | comments on photos and plants (JSON files in `data/`) |
| `plant_actions.rs` | `plant_actions.php` | QR/barcode images, done list, preferences, photo position, wish-list save/promote |
| `app_api.rs` | `app_api.php` | the Android app's API: **a contract**, do not rename anything it returns |

### `templates/`
One HTML file per page (`index.html`, `catalog.html`, ...) plus the widget
cards (`w_*.html`). They were copied from the PHP pages, so the markup is the
same; the `{{ name }}` and `{% if %}` bits are the holes. `|safe` after a
name means "this is HTML, put it in as it is"; without it the text is
escaped, which is what you want for anything a person typed.

## Changing things safely

1. Edit, then build and run the tests on the NUC
   (`docs/rust-port/tools/verify.sh` does the build; `cargo test -p
   junsgarden` runs the tests). The compiler is strict: a template that names
   a field that does not exist will not build, which is a feature.
2. Put it on the test site first (`deploy_test.sh`), look at it, then the
   live one (`deploy_live.sh`, then the two Caddy lines).
3. The PHP original is not part of Jun's copy (only the live Rust site is).
   The tools in `docs/rust-port/tools/` were used on the server to compare
   the two during the conversion.

## Which file do I open when…

| I want to… | Open |
|---|---|
| change wording on the home page | `templates/index.html` |
| change what the header links to | `templates/header.html` |
| let one more person log in | the `JUNSGARDEN_USERS` setting (no code); they then set a password with "First time?" on `login.php` |
| change how the login or password emails work | `src/accounts.rs`, and `src/pages/login.rs` with `templates/login.html` |
| let somebody upload plant photos | the `JUNSGARDEN_UPLOADERS` setting (no code) |
| add a search filter to the catalog | `src/pages/catalog.rs`, `ajax_load` |
| add a card to the Data page | `src/pages/data.rs`, `build_widget_data` (the number) and `js/widget-loader.js` (the card) |
| change how a category is grouped | `src/normalise.rs` |
| change what the app receives | don't, without a new app build: `src/pages/app_api.rs` |
| serve a new kind of file | `servable()` in `src/main.rs` |
