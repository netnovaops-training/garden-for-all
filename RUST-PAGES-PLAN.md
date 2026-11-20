# Official Rust pages: plan and resumable checklist

Branch `rust-pages`. Merged into `main` only when every step below passes, including the
installer (wizard) test, and the author approves.

## Goal
- Every page works and looks exactly as before; the code becomes lean, documented, secure, fast Rust.
- No PHP anywhere. The `.php` *addresses* stay (printed QR tags and the Android app use them);
  they are only names, nothing runs PHP.
- Help pages: not rewritten, tested side by side.
- The plant admin (`/streamlit/`, 14 tabs) and the Android app's requests are included.
- Dependencies at their latest stable versions, proven by the same tests.
- When done: `CODE-FINGERPRINTS.txt` regenerated; the fingerprints robot opens a PR instead of
  pushing to `main`.

## How it is tested (`docs/rust-pages/`)
A lab on any Linux machine with Docker (`GARDEN_LAB=user@host sh docs/rust-pages/sync.sh`):
- `gp-db`: throwaway MariaDB on an internal network; two databases, each = `db/schema.sql` +
  made-up sample data (`make_sample.py`). No real data anywhere.
- Old code (`fixes`) on 127.0.0.1:18101, new code (this branch) on 127.0.0.1:18102.
- `sh lab.sh round`: fresh databases, both sides, every list -> one PASS/FAIL line each.
  Lists: `urls-port.txt` (pages and APIs), `urls-extra.txt` (login, help, admin tabs),
  `urls-app.txt` (every Android app request, with its login key), `urls-post.txt` (form posts).
  `expect:` lines name intended fixes; `expect: maybe:` marks places the old code is random.
- `sh lab.sh shots`: screenshots at 390 and 1366 px, pixel-compared (random parts pinned).
- `sh lab.sh check`: fmt, clippy `-D warnings`, tests.

## Status
| Check | Result |
|---|---|
| round: urls-port / extra / app / post | PASS / PASS / PASS / PASS (all differences are listed fixes) |
| unit tests | 97 pass |
| screenshots | 30/36 identical; the other 6 are gallery (photo wall drifts with time) and data.php (random demo weather): both proven by the exact HTML comparison instead |
| fmt, clippy -D warnings, cargo audit | PASS (Rust 1.99) |
| dependencies | sqlx 0.9, reqwest 0.13, all crates latest; Rust 1.99 + Debian 13; Python locks refreshed; compose images by digest |
| address (`lab.sh address`) | 14/14: folder and no folder, `/q` `/b` codes, renamed bed, signed config, old QR after a name change |
| front door (`tools/release/tests/test-caddy-address.sh`) | 14/14: garden.sh's own Caddyfile validated and run; old name forwards pages (no double folder) and answers phone addresses |
| installer from scratch (`docs/rust-pages/wizard_test.sh`) | throwaway Ubuntu 24.04, the wizard's exact settings, nothing typed: see the language rows in the report |
| Windows wizard (`tools/release/tests/test-wizard-flow.ps1`) | 62/62 (real window, nothing installed), including the folder and its Korean label; PSScriptAnalyzer count unchanged |
| app | builds, lint, 4 unit tests (signature pinned to the Rust side) |

## Changes so far
- `guard.rs` (new): security headers (+ Permissions-Policy, COOP, CORP); POSTs started by another
  website refused (closes login CSRF); per-address rate limit; blocking work moved off the async threads.
- `store.rs` (new): atomic file writes; a damaged data file is never overwritten; parse-once file cache.
- `layout.rs`: the page frame's files cached for 10 s (was 7 blocking disk reads per page).
- `session.rs`: session renewal written at most hourly (was every click).
- `main.rs`: 64 MB request limit only on photo-upload addresses, 2 MB elsewhere.
- `comments.rs` (rewritten, shared with the app API): fixed a data-loss bug (a damaged comments
  file was overwritten by the next comment); keys must be real plants or gallery photos; 5 per
  address per 10 minutes; 500 kept per key.
- `index.rs`: guestbook writes atomic, rate-limited, 500 lines kept, redirect after posting.
- `gallery.rs`: photo list parsed once until the file changes.
- `catalog.rs`: database error text no longer shown; letter counts 26 queries -> 1; transactions
  for promote-to-garden and both quantity logs; photo work off the async threads.
- `plant_actions.rs`: promote-from-wish-list is one transaction and now moves the plant's barcodes.
- `app_api.rs`: like/favourite marks belong to whoever asks (anyone could edit Jun's favourites
  through `username=`); database error text no longer shown.
- `data.rs`: cross-table ties sorted by name (were random); error text no longer shown.
- `address.rs` (new): the garden's own address (`PUBLIC_URL`, optional folder); `/q/<id>`, `/b/<id>`
  label codes; `/labels`; `/app/setup` (EN + KO) and the Ed25519-signed `/app/config.json`.
- App: `GardenAddress` (set by the setup QR, follows only a signed move); updater checks sha256.
- `garden.sh`: `labels`, `app`, `folder` (menu More 19-21), web-name swap keeps the old name
  (pages forward, phones answered). Installer fixes found by the from-scratch test: a step failed
  in "both languages" mode when its last line had no Korean (the firewall step); `USER` unset
  outside a login; "Scheduled" printed with no cron (now installs cron, or stops).
- Windows wizard: folder field, phone-app and labels buttons (Korean from the same `lang/ko.tsv`).
- `code-fingerprints.yml`: the robot proposes a pull request instead of pushing to main.
- `pi-arm64.yml` (new): every image built on GitHub's arm64 machine; every pinned image has arm64.
- After PR #31 (branch `address-one-step`): the web address changes in ONE step from three places
  (`garden.sh webname NAME [/folder]`, the Windows wizard's last page, the labels page's form via the
  address watcher `junsgarden-address.path`): settings, front door, old name forwarding, rebuilt
  site, signed app config, labels page. Data files only through `store::file_in` (CodeQL clean).
  Camera tools say "turned off" (503) when their part is not installed. `labels` stays inside a
  folder. Jun's real data (a throwaway copy of the 2026-10-25 archive) walked on the new code.

## Pages
| Address | Rust file | Status |
|---|---|---|
| `/`, `/index.php`, `/sections/cards/index.php` | pages/index.rs | hardened, same* |
| `/login.php` | pages/login.rs | same* |
| `/blog.php` | pages/blog.rs | same |
| `/plant_listing.php` | pages/plant_listing.rs | same, lean-up todo |
| `/catalog.php` | pages/catalog.rs | hardened, same |
| `/gallery.php` | pages/gallery.rs | same |
| `/comments.php` | pages/comments.rs | rewritten, same* |
| `/plant_actions.php` | pages/plant_actions.rs | hardened, same* |
| `/data.php`, `/widget_data.php` | pages/data.rs, widgets.rs | partly reviewed |
| `/app_api.php` | pages/app_api.rs | partly reviewed, same* |
| `/app/garden/{action}` | pages/garden_camera.rs | same, review todo |
| `/streamlit/` | orchestrator/ | same, review todo |
| `/help...` | help.rs, guides.rs | test only, same |

`same*` = identical except the listed fixes.

## Open TODO items for Jun's project (triage 2026-10-25)
| Item | Verdict |
|---|---|
| Plant admin (Streamlit) to Rust; its basic-auth | Done (Rust, behind the garden login, since 2026-10-07) |
| Retire `garden-users.env` | Done: nothing reads it (a test only proves it is never served) |
| Credentials in tracked files / private history | Repo scans clean (gitleaks, trufflehog); old private copies are being archived |
| Keep public pages public | True (tested: catalogue, gallery, blog need no login) |
| Tag photo / barcode / identify from the app | Done in app 3.1+ camera tools (`/app/garden/{action}`) |
| Household chat buttons, Hermes vocabulary, moderation chat | Not Jun's garden (household stack); removed from the app |
| Soil / attracts / fruit_color clean-up mappings | Needs the author's review of the proposed mappings (data change) |
| Jun accepts invites; locked old account removal | Needs Jun / the author (live accounts) |
| Rotate credentials that leaked into a transcript | Needs the author (passwords) |

## Remaining, in order
1. Screenshots on sample data: every page 0.000% (gallery and random parts pinned).
2. Review and lean-up: data.rs/widgets.rs, app_api.rs, garden_camera.rs, orchestrator/, php.rs
   (-> plain Rust helpers), brain.rs, images.rs; fmt + clippy clean; cargo audit.
3. Dependencies to latest stable (Cargo, Python in beds/, Docker images by digest, Actions by SHA).
4. Done: plant admin = the 2026-10-07 Rust port (0 missing lines vs the original Streamlit) =
   this branch (all 14 tabs identical side by side). Hard-to-match features: DEVIATIONS.md S-1..S-14.
5. Done: the current app is `android-app/` (3.2, code 18); it builds, unit tests and lint pass.
6. Done: TODO triage above.
7. The garden's own address, end to end (each option in all THREE places: the QR/label script,
   the GUI wizard and the bash wizard `garden.sh` menu + subcommands; every screen, prompt and
   guide in English AND Korean):
   - Base address = domain plus optional sub-path (`example.com/garden`); nothing hard-codes a host.
   - Server publishes it (`/app/config.json`, signed); old domain keeps answering/redirecting.
   - QR codes and bed labels encode short stable paths (`/q/<plant id>`, `/b/<bed id>`) that the
     server resolves; a renamed bed keeps its QR; printed labels survive a domain change.
   - Android app: one APK, set up at first run from a QR/link on the wizard page; follows a
     domain change on next contact; self-update follows too; old app versions keep working
     (no removed/renamed endpoints).
   - Domain-change option (wizard + bash) regenerates labels on request.
   - Tests: base URL with/without sub-path, renamed bed, old printed QR after a domain change,
     app 3.1 + new app x old/new domain x updated/not updated, both languages.
8. Installer test from scratch (Ubuntu 24.04 throwaway, Windows lint + dry run, Pi arm64 build);
   how the slim download's pin moves (commit a67990d is never rewritten). What Jun does: run the
   wizard, scan the app QR / install the app, set his own password.
9. Guides (START-HERE, EN + KO): Rust 1.99 / Debian 13 mentions, new options; PDFs regenerated.
10. Fingerprints regenerated; robot opens a PR.
