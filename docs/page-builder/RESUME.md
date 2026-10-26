# Page builder: where the work stopped (resume notes)

Branch `page-builder`, draft PR #37 (do not merge; the author approves it himself).
Last session: 2026-10-26, **local commits only, nothing pushed** (the GitHub account was
suspended that day; work from the local clone, no `gh`, no fetch/pull/push).

## State
- Phase 1-4 done: `DESIGN.md`, the MVP (registry, default layouts equal to today's pages,
  owner-only editor, history and undo, owner pages), the signed sections list (not set up until
  the author adds a key), `GUIDE.md` / `GUIDE.ko.md`.
- **Main merged in** (commit `9f061b8`): Source link (#36) and Manual v2 (#38), from the last
  fetched `origin/main` (`93823c4`). The local `main` working copy was still at the branch
  base. The local plant AI (#39) is **not** in main yet, so it is not in this branch either.
  The only conflict was `CODE-FINGERPRINTS.txt`, made again with `make_fingerprints.py`.
- **Answers from the author applied** (commit `f80f085`):
  - Only Jun (the owner) edits pages. The code already did this; DESIGN.md 5.2 and 9 say so.
  - The rain radar comes back as section `home.radar` (DESIGN.md 3.1). The owner types a
    latitude and longitude; the Windy address is built from the parsed numbers only. The one
    CSP change: `frame-src 'self' https://embed.windy.com` in `guard.rs`
    (`docs/rust-port/DEVIATIONS.md` DEV-15).
- **CodeQL "path from user input"**: not re-run (the CodeQL CLI is not installed here and
  GitHub cannot be used). Manual review instead: `pagebuilder/` has no file system call at all
  (no `std::fs`, `tokio::fs`, `File`, `Path`/`PathBuf` join); the branch's changes to `pages/`,
  `main.rs` and `address.rs` add none either (only `test_state`, test-only temp paths). Backups
  are rows in `page_layout_backups` (commit `7624cbc`); every SQL value is bound. Page names
  reach only SQL binds and `page.php?name=` links, after `custom_slug` (a-z, 0-9, `-`, 40 max).
  Confirm with the real CodeQL check once CI runs again.

## Test results (2026-10-26)
- `cargo fmt --check`: clean. `cargo clippy --workspace --all-targets -D warnings`: clean.
- `cargo test --workspace`: **138 pass**, 0 fail (127 before + the merged Source-link tests +
  2 radar tests).
- Node: `editor.js` syntax OK; `test-source-link.mjs`, `test-chart-compare-escape.js`,
  `test-guest-thumbs.js` pass.
- Scans: gitleaks (diff and commit hook) no leaks; trufflehog on the changed files none;
  semgrep (p/rust, p/javascript, p/secrets) on the page builder files clean. The repo-wide
  `tools/check-secrets.sh` reports one old finding that is also on main
  (`pages/help_tests.rs:19`, test-only `temp_dir`, its `nosemgrep` sits on line 23), and its
  gitleaks part cannot read history from this worktree (`.git` is a file pointing outside).
- **Not run**: the side-by-side lab and the browser end-to-end tests. `docs/rust-pages/lab.sh`
  runs on the server (not to be touched), and the browser scripts were kept outside the repo.
  Rerun both before asking the author to merge.

## Open questions for the author (do not guess)
1. Should new owner pages (`/page.php?name=...`) appear in the top menu automatically?
2. Should the builder's 3 tables (`page_layouts`, `page_layout_backups`, `page_builder_seen`)
   fold into the separate database rework (and `db/schema.sql`)?
3. (Still open from DESIGN.md 9) The Ed25519 signing key for the sections list.

## NEXT goal (added by the author): the whole site in sections
Break **every page and script** of Jun's site into sections that work together, so Jun can
add parts, add whole pages and customise the banner. Plan only; not built yet.

**Inventory.** Pages: home, archive, gallery (done), catalog (`catalog.rs`, `app.js`),
plant listing / search (`plant_listing.rs`, `plant-listing.js`, `search.js`), data page
(`data.rs`, `widgets.rs`, 11 `w_*.html` widgets, `widget-loader.js`, `chart-compare.js`),
help (`help.rs`, 3 templates), garden camera, plant admin (`orchestrator/`, 14 tabs). Frame:
header, menu, hero banner (`hero.js`, hero photo folder, facts), footer (`layout.rs`).
Stay outside the builder on purpose: login, `app_api.php` (the phone app's contract), the
`.php` addresses QR codes use, `/healthz`.

**Order** (each step: golden test "default layout = today's page", then the editor):
1. Section contract: each section lists its scripts and styles (loaded once per page, only
   when used) and draws inside its own root element; scripts that look for a fixed id keep
   `max_per_page: 1` until they are changed to work inside their root.
2. Data page: one section per widget and the comparison tool (read-only, many small parts).
3. Help pages, garden camera (read-only).
4. Catalog and plant listing: the pending sections in DESIGN.md 3.2. These wait for the
   database rework or are done together with it.
5. Banner: a page-frame part of the layout (banner on/off per page, which hero photos,
   title/subtitle EN/KO, facts on/off), with the same checks as section settings.
6. Whole new pages from a starting set of sections (needs open question 1 answered).
7. Plant admin tabs last (admin-only, the most code).

**Risks.** The same-output rule (AGENTS.md rule 6): every default layout must stay byte for
byte equal. Scripts written for one page (fixed ids, page-wide globals) can clash when two
sections share a page. Every new outside host needs a CSP change and a DEVIATIONS entry.
Admin parts must keep the per-draw permission check. The database rework overlaps steps 2
and 4. Many sections each asking the database can slow a page. The phone app's API must not
change. Phones: side-by-side rows stack.

## How to rerun the checks
- Unit tests, fmt, clippy (Docker, from `rust/`): see AGENTS.md "Before you commit".
- Before every commit: gitleaks and trufflehog on the new commits, semgrep on the changed
  files. After any change: `git add` first, then `python tools/release/make_fingerprints.py`
  (it reads the staged files) and commit `CODE-FINGERPRINTS.txt`.
- Side-by-side lab: `docs/rust-pages/` with `main` as base and this branch as candidate,
  then `round` and `shots` (runs on a test machine, never on the live site).
