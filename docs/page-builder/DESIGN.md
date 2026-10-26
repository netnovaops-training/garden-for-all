# Page builder: design

Branch `page-builder`. Status: draft, waiting for the author's review. Nothing here is deployed.

## 1. What it is

The garden owner can build any page out of **sections**, like a 2000s site builder. On a
desktop the owner presses **Edit**. A panel on the left lists the sections. On the right is a
canvas with a big **+** to add sections and a live preview. Sections sit on a simple grid:
rows, each row having 1 to 4 columns. They move up, down, left and right, and come off with a
big **−**.

What stays the same:

- **Nothing changes until the owner edits.** Each current page ships as a *default layout*
  made from the same sections. The default layout produces byte-identical HTML. Tests prove
  it (section 8).
- **Same code, no fork.** A section is a slice of today's page template, rendered by
  today's code with today's data. The page handler now draws "the layout", and the layout
  is a list of those same slices.
- **Phones and the Android app only display.** The saved layout is responsive: rows stack
  into one column below 800 px. The app is native: it reads `app_api.php`, which this work
  does not touch. Editing is refused on narrow screens and for app keys.

## 2. Scope

| In scope now | Handled separately (database rework) |
|---|---|
| Home (`/`, `/index.php`) | Catalog (`/catalog.php`), plant cards |
| Garden archive (`/blog.php`) | Quick Search (`/plant_listing.php`) |
| Gallery (`/gallery.php`) | Data charts (`/data.php`, `/widget_data.php`) |
| New owner pages (`/p/<name>`) | Plant admin (`/streamlit/`, 14 tabs) |
| | `app_api.php`, `plant_actions.php` |
| | Bed planner (`/beds/...`, a separate Python service) |

The database and plant-data area is being reworked separately, so this branch does not
change its code. Its blocks are defined below as **pending sections** (section 3.2). Each has
a planned id, a settings schema and a permission, so registering it later is a small change
(section 5.4).

## 3. Section inventory

### 3.1 Live sections (registered in this branch)

"Max" is how many copies one page may hold. A section whose script looks for a fixed element
id can appear only once.

| id | Name (EN / KO) | Source today | Permission | Settings | Max |
|---|---|---|---|---|---|
| `home.welcome` | Log-in box / 로그인 상자 | `templates/index.html` admin bar, `pages/index.rs` | public | none | 1 |
| `home.weather` | Garden weather outlook / 정원 날씨 예보 | `templates/weather.html` + its box in `index.html`; `site/js/weather.js` | public | none | 1 |
| `home.tips` | Garden & collie tips / 정원·콜리 팁 | `index.html` tips block; `site/js/garden-chronicles.js`; `/sections/cards/index.php` (`index::cards`) | public | none | 1 |
| `home.news` | Garden news (newest posts) / 정원 소식 | `index.html` news block; `index::post_cards`; `site/js/home-blog.js` | public (edit buttons only for people who are signed in, as today) | `count` 1-12, default 3 | 1 |
| `home.guestbook` | Guestbook / 방명록 | `index.html` guestbook block; `index::read_comments` | public (delete buttons only for people who are signed in) | none | 1 |
| `admin.post_editor` | New-post button / 새 글 쓰기 버튼 | `index.html` "Create New Post" and the two post forms | **admin** | none | 1 |
| `blog.archive` | Garden archive (all posts) / 정원 기록 보관소 | `templates/blog.html`, `pages/blog.rs` | public (delete buttons for people who are signed in) | none | 1 |
| `gallery.wall` | Photo gallery / 사진 갤러리 | `templates/gallery_body.html` + `gallery_head.html`, `pages/gallery.rs` | public | none | 1 |

**The radar.** `site/js/home.js` still looks for `#radarFrame` (a Windy map), but no
template draws that element anymore. The weather outlook is the live weather block today, so
`home.weather` is it. Bringing the map back would be a new section (open question 2).

**How a section reuses today's code.** The home page and archive templates mark each block
with `{% block name %}` tags, which print nothing. askama renders one block on its own
(`#[template(path = "index.html", block = "home_news")]`). The post forms moved into
`templates/post_editor.html`, which the news block includes, because askama cannot render a
block nested inside another block. The gallery's body and `<head>` part are used whole. A
golden test renders the whole template and the joined sections and compares them byte for
byte. Another check confirmed that the edited templates, with the tags removed, equal the
templates on `main`.

**Forms in moved sections.** The guestbook, Logout and the post forms post to the page they
are on. `/gallery.php` and `/page.php` now send such POSTs to the home page's own handler
(`index::home_post`), and `/blog.php` does the same after its own buttons. Before, those POSTs
were refused (gallery) or ignored. These three addresses also get the photo-upload body limit
that the home page has, for the post editor.

The page frame is not a section and stays as it is: header, menu, hero banner, footer
(`layout.rs`, `layout.html`). Each page's outer wrapper (`<div class="page-content ...">`)
and its page-wide scripts belong to the page "shell" (section 4.3).

### 3.2 Pending sections (database rework; defined here, registered later)

| Planned id | Name (EN / KO) | Source today | Permission | Settings schema |
|---|---|---|---|---|
| `catalog.browse` | Plant catalog / 식물 목록 | `pages/catalog.rs`, `templates/catalog.html`, `site/js/app.js` | public (edits inside need admin) | `page_size` int 6-60 (default 12); `filter` string ≤ 80 |
| `catalog.cards` | Plant cards (a fixed pick) / 식물 카드 | `catalog.rs` card markup | public | `plant_ids` int list ≤ 24; `columns` int 1-4 |
| `search.quick` | Quick search / 빠른 검색 | `pages/plant_listing.rs`, `site/js/plant-listing.js`, `search.js` | public | `placeholder_en`/`_ko` string ≤ 60 |
| `data.widgets` | Garden charts / 정원 차트 | `pages/data.rs`, `widgets.rs`, `templates/data.html`, `site/js/widget-loader.js` | public | `widgets` list of widget ids (from `build_widget_data`) |
| `data.compare` | Cross-reference tool / 교차 비교 도구 | `templates/data.html` compare block, `site/js/chart-compare.js` | public | none |
| `data.table` | Data table / 데이터 표 | new, over the reworked data API | public | `columns` list, `sort` string |
| `admin.plant_admin` | Plant admin tab / 식물 관리 탭 | `orchestrator/` (14 tabs) | **admin** | `tab` int 1-14 |
| `beds.planner` | Garden beds / 화단 | `beds/plan_page.py` (separate service) | **admin** (planting) / public (view) | `bed` string ≤ 120 |

## 4. Layout JSON

### 4.1 Shape

```json
{
  "schema": 1,
  "page": "home",
  "rows": [
    { "cols": [ { "section": "home.weather", "settings": {} } ] },
    { "cols": [ { "section": "home.tips", "settings": {} },
                { "section": "home.news", "settings": { "count": 2 } } ] }
  ]
}
```

### 4.2 Validation (server side, on every save and preview)

- `schema` must be `1`. `page` must match the page being saved. Unknown keys anywhere are
  refused (`deny_unknown_fields`).
- 0-40 rows, 1-4 columns per row, the whole JSON at most 64 KB.
- `section` must be a **registered** id. Pending ids and ids that only appear in the
  remote manifest are refused.
- `settings` must match that section's schema: only known keys, the right type, inside
  min/max, and strings within their length limit. Missing keys take the default.
- A section with `max: 1` may appear once per page.
- Settings never reach HTML unescaped. Numbers are used as numbers, and strings go through
  askama's escaping. No section takes HTML, script, CSS or a URL from settings.

### 4.3 Rendering

`pagebuilder::render(page, layout)` walks the rows:

- A row with one column prints the section's HTML **as is**, with no wrapper. That is why a
  default layout is byte-identical to today's page.
- A row with 2-4 columns prints `<div class="pb-row pb-cols-N">` with one `<div class="pb-col">`
  per column. `site/pagebuilder/pb.css` (loaded only when such a row exists) makes it a grid
  that stacks below 800 px.
- Each built-in page keeps its shell: title, hero on/off, outer wrapper, page scripts. New
  `/p/<name>` pages use a plain shell (the archive's green background, no hero).
- Each section's `<head>` additions (the gallery's styles) are added once per page.

## 5. Registry and permissions

### 5.1 The manifest

The registry is `rust/junsgarden/src/pagebuilder/registry.rs`, one entry per section:

```rust
SectionDef { id: "home.news", name_en: "Garden news", name_ko: "정원 소식",
             desc_en: "...", desc_ko: "...", version: 1, since: "2026.11",
             permission: Perm::Public, max_per_page: 1,
             settings: &[Setting::int("count", 1, 12, 3)] }
```

`GET /pagebuilder/api/registry` serves the same list as JSON to the editor.

### 5.2 Who may do what

| Who | Sees | May edit |
|---|---|---|
| Visitor | Public sections; admin sections render **nothing** | No |
| Signed-in garden user (GARDEN_USERS) | Public and admin sections, with today's edit buttons | No |
| **Owner** (signed in on the website, not the app) | Everything, plus the **Edit** button | Yes |

The owner is `JUNSGARDEN_PAGE_OWNER` (a name on the GARDEN_USERS list). When it is unset,
the owner is the first name on that list.

### 5.3 Checks on the editor API (all under `/pagebuilder/`)

| Method and path | What it does |
|---|---|
| `GET /pagebuilder/edit?page=` | the editor page |
| `GET /pagebuilder/api/registry` | sections, with "new" badges |
| `GET /pagebuilder/api/layout?page=` | current layout and version list |
| `POST /pagebuilder/api/preview` | renders a layout without saving |
| `POST /pagebuilder/api/save` | validates, backs up, saves a new version |
| `POST /pagebuilder/api/undo` | goes back to the previous version |
| `POST /pagebuilder/api/reset` | goes back to the default layout |
| `POST /pagebuilder/api/seen` | clears "new" badges |
| `GET /pagebuilder/api/updates` | the signed remote manifest check |

Every route checks:

1. The visitor is the owner (`is_owner`), on a website session. An app key is refused (403).
2. A POST must also carry `X-PB-Token`: SHA-256 of a secret made at start-up and the
   session id, compared in constant time. It must be `Content-Type: application/json` (a
   plain HTML form cannot send that). The site's existing `same_site_posts` guard refuses
   cross-site POSTs before any of this.
3. Bodies are at most 64 KB, and the layout is validated (4.2) before anything is stored.

Admin sections enforce their own permission **at render time**, in `render`, whatever page
they are placed on. A visitor gets an empty string. Tests prove this.

**The preview** is drawn by the server as a visitor sees it: no session, admin sections shown
as a labelled placeholder, no Edit button. It goes into an `<iframe srcdoc>` sandboxed with
`allow-scripts allow-same-origin`, so the page's own scripts run. There is no `allow-forms`
and no top navigation. Its HTML is the site's own, and no setting carries markup. The editor
puts all server text on the page with `textContent`.

### 5.4 Adding a section later (incl. the pending ones)

1. Add a template slice (or reuse the page's own) and a `load`/`render` pair in
   `pagebuilder/sections.rs`.
2. Add one `SectionDef` to `registry.rs`.
3. Add it to the signed manifest (`docs/page-builder/sections-manifest.json`) and sign it
   (section 7).

The registry test fails if an entry has no renderer, or a renderer has no entry.

## 6. Storage, versions, undo, backups

```sql
CREATE TABLE IF NOT EXISTS page_layouts (
  id INT AUTO_INCREMENT PRIMARY KEY,
  page VARCHAR(64) NOT NULL,
  version INT NOT NULL,
  layout MEDIUMTEXT NOT NULL,          -- the validated JSON
  saved_by VARCHAR(100) NOT NULL,
  note VARCHAR(40) NOT NULL DEFAULT 'save',  -- save | undo | reset
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY page_version (page, version)
);
CREATE TABLE IF NOT EXISTS page_layout_backups (   -- a copy before each save
  id INT AUTO_INCREMENT PRIMARY KEY,
  page VARCHAR(64) NOT NULL,
  version INT NOT NULL,
  layout MEDIUMTEXT NOT NULL,
  backed_up_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  KEY page (page)
);
CREATE TABLE IF NOT EXISTS page_builder_seen (
  section_id VARCHAR(64) NOT NULL PRIMARY KEY,
  seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

- **The current layout** is the highest version of the page. If there is no row, the page
  uses the default layout.
- **Save** inserts version N+1 inside a transaction. The client sends the version it
  started from, and a stale editor gets 409 instead of overwriting.
- **Backup before each save.** In the same transaction, a copy of the layout being replaced
  (the default one included) goes into its own table, `page_layout_backups`, which keeps the
  newest 1000 copies. They go to the database, not to files, so no file path is ever built
  from a page name. The database backups (`garden.sh`) cover the table too.
- **Undo** inserts a copy of version N-1 as N+1, so history only grows and an undo can
  itself be undone. **Reset** inserts the default layout. **Bring back** (the history list)
  inserts any kept older version. Unsaved changes are dropped by Undo after a question.
- The tables are made at start-up (`saved::ensure_tables`, like the site's other own
  tables). `db/schema.sql` is left alone, because the database rework owns it. Adding the three
  tables there is part of that rework (open question 5).
- The current layouts are cached in memory and dropped on every save, undo or reset. A page
  view costs one database read only after a restart.

## 7. Update check for new sections

- Sections arrive with garden updates (new code). The garden also asks the author's
  GitHub repo whether newer releases bring sections it does not have yet.
- It reads `https://raw.githubusercontent.com/netnovaops-training/garden-for-all/main/rust/junsgarden/sections-manifest.json`
  and `...json.sig`. The list sits inside the Rust crate, so a test can check that it matches
  the registry (the usual `cargo test` command mounts only `rust/`).
- The signature is Ed25519 (the same `ed25519-dalek` crate and hex format the signed
  `/app/config.json` uses). The signed message is `junsgarden-sections-v1\n` followed by the
  manifest bytes. The public key is pinned in code (`AUTHOR_SECTIONS_KEY`), and
  `JUNSGARDEN_SECTIONS_KEY` can override it. With no key, the check says "not set up" and
  does nothing.
- The check runs only when the owner opens the editor on a desktop. The answer is cached
  for 6 hours, with a 10 s timeout and a 256 KB limit. No new open ports are needed: it is
  an outgoing HTTPS request.
- **Installed but never seen** sections show a **new** badge until the owner adds one or
  presses "OK". **Announced but not installed** sections show greyed out with "comes with
  the next update". Nothing is ever placed on a page by itself.
- The author signs with `docs/page-builder/sign-manifest.sh` (OpenSSL 3, Ed25519 key
  outside the repo).

## 8. Test plan

| Test | Where | Proves |
|---|---|---|
| Golden HTML: home, archive and gallery sections joined = the original templates (kept test-only in `templates/legacy/`), for visitor and admin, posts or none, comments or none | `cargo test` | default layouts are byte-identical |
| Pixel and HTML side by side: `main` vs `page-builder` on the lab's sample data (`docs/rust-pages/lab.sh round` and `shots`) | lab | same pages, same screenshots |
| Admin section as visitor → empty; as admin → drawn; on a public page and on `/p/...` | `cargo test` | permission at render time |
| Editor API: visitor 403, admin non-owner 403, app key 403, no or wrong token 403, form content type 415 | `cargo test` (router `oneshot`) | owner-only, CSRF |
| Validation: unknown section, pending section, unknown setting, out of range, too many rows/cols, duplicate `max:1`, `<script>` in a string setting stays escaped | `cargo test` | layout JSON is safe |
| Undo/reset/version conflict | `cargo test` on the pure version logic; lab for the SQL | history behaves |
| Signed manifest: good signature passes; changed byte, wrong key, no key fail; "new" badge logic | `cargo test` | update check is safe |
| Registry: every entry has a renderer, names in EN and KO, unique ids | `cargo test` | registry is complete |
| fmt, clippy `-D warnings`, gitleaks, trufflehog, semgrep | CI and `tools/check-secrets.sh` | repo rules |

**Results (2026-10-26, on the made-up sample data):**

- `cargo test`: 127 pass, 26 of them new. `fmt` and `clippy -D warnings` are clean.
- `main` vs `page-builder`, HTML: 12/12 pages byte-identical. These were home, home with a
  zip, archive, gallery, catalog, quick search, login and the tip cards, logged out and in.
  `data.php` is left out because it prints random demo values on every load; its code is
  unchanged.
- Screenshots at 390 and 1366 px: 8/10 identical. The two others are the gallery, whose wall
  drifts with time (known; its HTML is identical).
- In a browser, end to end, 11/11: Edit button for the owner only; the default layout; drag
  and drop side by side; save, undo; Korean; on a new page, a visitor sees no admin section;
  a phone shows the page stacked.

## 9. Open questions for the author

1. **Owner**: is "the first GARDEN_USERS name, or `JUNSGARDEN_PAGE_OWNER`" right, or
   should every signed-in garden user be allowed to edit?
2. **Radar**: bring the Windy radar map back as a section (it needs `frame-src` in the
   CSP), or leave it out?
3. **Signing key**: the author creates the Ed25519 key and puts its public half in
   `AUTHOR_SECTIONS_KEY`. Until then, the update check says "not set up".
4. **Owner pages** (`/page.php?name=<name>`): should they show in the top menu?
5. **`db/schema.sql`**: add `page_layouts`, `page_layout_backups` and `page_builder_seen`
   there as part of the database rework?
