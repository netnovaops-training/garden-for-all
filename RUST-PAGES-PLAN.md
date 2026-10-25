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
| unit tests | 94 pass (was 86) |
| screenshots | public pages identical; full re-run on sample data pending |
| fmt, clippy -D warnings | todo (45 lints, most existed before) |

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

## Remaining, in order
1. Screenshots on sample data: every page 0.000% (gallery and random parts pinned).
2. Review and lean-up: data.rs/widgets.rs, app_api.rs, garden_camera.rs, orchestrator/, php.rs
   (-> plain Rust helpers), brain.rs, images.rs; fmt + clippy clean; cargo audit.
3. Dependencies to latest stable (Cargo, Python in beds/, Docker images by digest, Actions by SHA).
4. Plant admin vs the original Streamlit app; list the features that were hard to match.
5. Android app: dead code from the household version removed on a local branch; build + tests.
6. Open TODO items for Jun's project (broken first).
7. Installer test from scratch (Ubuntu 24.04 throwaway, Windows lint + dry run, Pi arm64);
   how the slim download's pin moves (commit a67990d is never rewritten).
8. Fingerprints regenerated; robot opens a PR.
