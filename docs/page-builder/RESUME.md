# Page builder: where the work stopped (resume notes)

Branch `page-builder`, draft PR #37 (do not merge; the author approves it himself).

## Done
- Phase 1: `DESIGN.md` (inventory, layout JSON, permissions, signed updates, test plan, open
  questions). The database area is "handled separately": its blocks are pending sections only.
- Phase 2 MVP: registry, default layouts equal to today's pages, editor (add, remove, move,
  side by side, live preview, save, undo, original page, history, owner pages), owner-only
  edit, admin-section protection, layouts versioned in the database with a backup copy
  (table `page_layout_backups`) before each save.
- Phase 3: the signed (Ed25519) sections list and "new" badges (not set up until the author
  adds a key).
- Phase 4: `GUIDE.md` and `GUIDE.ko.md` with screenshots in `images/`.
- Tests: `cargo test` 127 pass; fmt and clippy `-D warnings` clean. Lab (main vs branch,
  sample data): HTML 12/12 identical, screenshots 8/10 (the other 2 are the gallery, which
  drifts with time), browser end to end 11/11.

## Open steps
1. Check CI on the newest push (`gh pr checks 37`). The CodeQL check failed twice on
   "path from user input" in the old file backups; backups now go to the database (commit
   `7624cbc`). Confirm that CodeQL is green now. If it still flags something, read the
   annotations on the CodeQL check run.
2. When CI is green, send the final report: the PR link, what works, test results, and the
   open questions (DESIGN.md section 9).

## How to rerun the checks
- Unit tests, fmt, clippy (Docker, from `rust/`): see AGENTS.md "Before you commit".
- Before every push: gitleaks and trufflehog on the new commits, semgrep on the changed files.
  After any change, `python tools/release/make_fingerprints.py` and commit
  `CODE-FINGERPRINTS.txt` (the `fingerprints-in-sync` check needs it).
- Side-by-side lab: the scripts were kept outside the repo. To rebuild it, use the lab in
  `docs/rust-pages/` with `main` as base and this branch as candidate, then `round` and `shots`.
