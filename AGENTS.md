# AGENTS.md: working on Jun's Garden with an AI assistant

This file is for AI coding assistants (Claude, Copilot, Cursor, Codex,
anything that reads `AGENTS.md`) and for Jun, who will be asking them for
help. Read it fully before changing anything. Humans: `docs/rust-port/CODE-TOUR.md`
is the friendlier walk through the code.

## If you are running a personal copy (read this first)

This folder may be Jun's **personal copy** on his own laptop, started with
`setup.sh` and `docker-compose.yml`, at `http://localhost:8030`. In that case:
the guides are in `START-HERE/` (begin with `00-READ-ME-FIRST.md`), the scripts
are in `lab/`, and `GEMINI.md` says how to help Jun. `junsgarden.zapto.org` is
meant to become this copy's address, but only through the switch-over plan in
`START-HERE/04b-take-over-junsgarden-zapto.md`, done together with the owner.
Deploy and server details below describe the old setup on the owner's server,
not the copy.

## What this is

The website at **junsgarden.zapto.org**: Jun's plant catalogue, gallery,
blog, garden data charts and guestbook, plus the small API the Android app
reads. It was a PHP site; on 2026-10-05 it was converted page by page to
**Rust** with the same look, the same URLs (including the `.php` ones, which
printed QR codes and the app still use), the same JavaScript and the same
behaviour. This copy has only the live Rust site: the old PHP pages are not
included. `docs/rust-port/` and `docs/streamlit-port/` describe how each page
was matched to the original.

Stack: Rust 1.99, **axum 0.8** (web server), **askama 0.16** (HTML
templates, checked at compile time), **sqlx 0.9** (MariaDB), tokio. It runs
in Docker on a Linux server behind **Caddy** (the reverse proxy that holds
the HTTPS certificate). Logins are the site's own: see rule 1 below.

## Where things are

```
rust/                     Cargo workspace (build from here)
  Dockerfile              builds one binary: --build-arg APP=junsgarden
  common/                 small shared helpers (env settings, logging)
  junsgarden/src/
    main.rs               settings, database, the URL -> function list, static files
    session.rs            who is asking: visitor cookie, logged-in session, app key
    accounts.rs           the two logins: password hashes, lockout, reset emails
    pages/login.rs        /login.php: log in, forgot password, set password
    layout.rs             the shared page frame (header/footer)
    php.rs                helpers that format output exactly like PHP did
    formdata.rs           reads form posts and uploads
    images.rs             saves/resizes uploaded photos as .webp
    rowjson.rs            database row -> JSON, the way PHP's json_encode did
    normalise.rs          the plant-field clean-up rules
    brain.rs              optional local AI (Ollama); fails quietly if absent
    guides.rs             the START-HERE guides as HTML for /help (pages/help.rs)
    pages/*.rs            one file per page or endpoint (catalog.rs = catalog.php)
    pagebuilder/          the page builder: pages made of sections, owner-only editor
                          (docs/page-builder/DESIGN.md; start with its mod.rs)
    orchestrator/         /streamlit/, the plant admin (was a Streamlit app); 14 tabs,
                          HTML built in Rust (no templates); start with its mod.rs
  junsgarden/templates/   the HTML, one file per page; {{ }} are the holes Rust fills
db/schema.sql             database structure only (no data)
docs/rust-port/           how it was built, verified, deployed; read README.md first
  tools/                  compare/verify/deploy scripts (run on the server)
docs/streamlit-port/      the plant admin's conversion: README.md, SPEC.md (what each
                          tab does), DEVIATIONS.md (S-1...), compare tool, baselines
site/                     the browser files the live site serves (mounted at /www)
  js/ style.css sections/ apps/   scripts, styles, page cards, the app download
garden.sh                 Ubuntu: install, backup, restore, checklist (menu)
windows/Garden-Menu.ps1   Windows: the same jobs, and setup over SSH
docker/tools/             the backup/restore/set-password toolbox container
tools/check-secrets.sh    run before every commit
```

**The browser files are not in the Rust build.** CSS, JavaScript, images and
PDFs are served unchanged from `site/` (`SITE_DIR` in `.env`, mounted at
`/www`). To change a script or the stylesheet, edit it in `site/` and reload
the page; no rebuild is needed. Photos, comments and the AI cache live in
`local/` (outside git) and are mounted over `/www/images`, `/www/data` and
`/www/cashe`.

**Which file answers a URL:** find the `.route("/that.php", ...)` line in
`main.rs`; it names the function in `pages/`. Most pages switch on a query
parameter (`?ajax_load`, `?action=list`): the `dispatch`/`*_get`/`*_post`
function at the bottom of each page file is that switchboard.

## Build, test, run

Building needs Linux (the code uses Unix signals). On Windows or a Mac, use
Docker. From the repo root:

```sh
# unit tests (42 today; all must pass)
cd rust && docker run --rm -v "$PWD":/build -w /build rust:1.99-trixie cargo test --locked -p junsgarden

# build the image
docker build --build-arg APP=junsgarden -t junsgarden/rs rust
```

`--locked` means `Cargo.lock` must already match `Cargo.toml`. After adding
a dependency, run `cargo build` once without `--locked` (in the same
container) and commit the updated `Cargo.lock`.

Running it needs a MariaDB with the schema in `db/schema.sql` and the site
folder. Settings are environment variables only (never put them in code):

| Variable | Meaning |
|---|---|
| `JUNSGARDEN_DATABASE_URL` | `mysql://user:pass@host/db`, required, **secret** |
| `APP_PORT` | port inside the container (deploy scripts use 8030) |
| `JUNSGARDEN_WWW_DIR` | the site folder, default `/www` |
| `JUNSGARDEN_DATA_DIR` | guestbook/comments folder, default `<www>/data` |
| `JUNSGARDEN_USERS` | `Name=email,...` the only accounts (from GARDEN_USERS in `.env`); fallback: Jun only |
| `JUNSGARDEN_PUBLIC_URL` | the site's address, used in reset-link emails |
| `JUNSGARDEN_SMTP_HOST`, `_PORT`, `_USER`, `_PASSWORD`, `JUNSGARDEN_MAIL_FROM` | the outbox for "forgot password" emails, **secret** |
| `JUNSGARDEN_UPLOADERS` | names who may upload photos; fallback `Jun` |
| `OLLAMA_URL`, `OLLAMA_MODEL` | optional local AI |
| `JUNSGARDEN_AI_CACHE_DIR` | AI cache, default `<www>/cashe/AI` (sic, the PHP spelling) |
| `JUNSGARDEN_GUIDES_DIR` | the guides (`START-HERE`) for the `/help` pages, default `/guides` (mounted read-only by `docker-compose.yml`) |
| `TZ` | `America/New_York`; dates on pages depend on it |
| `JUNSGARDEN_PAGE_OWNER` | who may edit pages with the page builder; default: the first JUNSGARDEN_USERS name |
| `JUNSGARDEN_SECTIONS_KEY` | optional: the author's public key for the signed sections list (pagebuilder/updates.rs) |

Keep the real values in an `.env` file next to the deploy scripts, never in
the repo (`.gitignore` blocks `*.env`).

## The rules (for the assistant)

1. **Visitors can see everything; only the people in GARDEN_USERS can edit.** There are
   only the accounts listed in `JUNSGARDEN_USERS` (Jun adds people with the
   menu, `bash garden.sh users`) and no way to create more from
   the site. The login is in `accounts.rs` (Argon2 password hashes, a
   lockout after 5 failures per address or 10 per name in 15 minutes) and
   `pages/login.rs` (`/login.php`: log in, "forgot password" by a one-time
   emailed link to the address on file, which is also how a first password
   is set). The website keeps a `PHPSESSID` session, renewed with a new id at
   login; the app gets a 64-character key from `app_api.php?action=login`
   and sends `Authorization: Bearer <key>` (`session.rs`). Keep it that
   way: never store or log a password or a reset code in plain text, never
   accept a password in a URL, never trust a header or a `username=` field
   as proof of who someone is, and never add a second way in (no master
   password, no "remember me forever", no SSO header) unless Jun asks.
2. **Every editing action checks `visitor.is_admin()`** (or `can_upload`)
   before it touches the database or disk. A new editing endpoint without
   that check is a bug, whatever else it does.
3. **SQL only with bound parameters** (`sqlx::query(...).bind(x)`). Never
   build SQL by pasting user input into a string. Column or table names that
   must vary come from a fixed list in the code, never from the request.
4. **Output is escaped.** askama escapes `{{ }}` by default; `|safe` only on
   HTML the code itself built. In Rust-built HTML use `php::he()`.
5. **Do not break the app.** The Android app (its code is the Gitea repo
   `junsgarden-app`) reads `app_api.php` with `action=list`, `detail`,
   `bed_counts`, `barcode_lookup`, logs in with `login` / `whoami` /
   `logout`, and reads `/images/...` and `/apps/version.json`. Add fields if
   needed; never rename or remove one, or change a type. Change the site
   first, then ship the app that uses the change.
6. **Same output as before unless Jun asks for a change.** Pages, JSON key
   order and number formats match the PHP site on purpose (the scripts in
   `js/` depend on them). `php.rs` exists for this. Known, deliberate
   differences are listed in `docs/rust-port/DEVIATIONS.md`; add to it when
   you make one.
7. **Static files go through `servable()` in `main.rs`**, an allowlist of
   file types. Do not widen it to source, `.bak`, `.env`, `.sql`, or the
   `data/`, `database/`, `tools/`, `archive/` folders.
8. **No secrets in the repo, ever:** no passwords, tokens, database URLs with
   a password, private keys, `.env` files. Settings come from the
   environment. Run `sh tools/check-secrets.sh` before every commit.
9. **Small changes, one at a time, tested.** Run the unit tests, try it on
   the test site, then live. Explain changes in plain words: Jun is learning.

## Recipes

**Change what a page shows.** Edit its template in `templates/` (HTML) and,
if it needs new data, the struct and query in `pages/<page>.rs`. askama
fails the build if a template uses a field the struct does not have; that
is the safety net, read the error.

**Add a page.** New file `pages/mypage.rs` with an async handler returning
`Response`; add `pub mod mypage;` to `pages/mod.rs`; add
`.route("/mypage.php", get(pages::mypage::handler))` in `main.rs`; add a
template and render it with `layout::render` so it gets the header/footer.

**Add an editing action.** Follow an existing one in `pages/plant_actions.rs`:
the `if !visitor.is_admin() { ... 403 ... }` gate comes before any write; read
the form with `formdata`, bind every value, and answer in the shape that
file's other actions use (`{"status":"ok",...}`, or `{"error":"..."}` with a
4xx/5xx status), because the page's JavaScript reads exactly those keys.

**Add a database column.** `ALTER TABLE` on the server (back up first),
update `db/schema.sql`, then read it in Rust (`row.try_get::<Option<T>,_>`
handles NULLs).

**Add a chart card to the Data page.** Compute it in `build_widget_data` in
`pages/data.rs` (one block per card, numbered), then draw it in
`js/widget-loader.js` in the site folder.

## Before you commit

```sh
sh tools/check-secrets.sh      # gitleaks + semgrep, both must be clean
cd rust && docker run --rm -v "$PWD":/build -w /build rust:1.99-trixie cargo test --locked -p junsgarden
```

Commit messages: one line saying what changed and why, in plain words.

## Deploying

The scripts in `docs/rust-port/tools/` run on the server: `deploy_test.sh`
(test site on copies of the data), `golive.sh` / `undo-golive.sh` (switch the
live site and back), `refresh_test_db.sh`. `docs/rust-port/README.md`
explains each, and `SETUP-ON-A-NEW-MACHINE.md` covers running it all on a
new machine (a Raspberry Pi or a Linux laptop).
