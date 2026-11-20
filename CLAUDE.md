# CLAUDE.md

Read `AGENTS.md` first; everything about this project is there. In short:

- This is Jun's Garden, a Rust (axum + askama + sqlx) port of a PHP site.
  Only the live Rust site is here; the browser files it serves are in `site/`.
- Visitors see everything; only the people in GARDEN_USERS (Jun), logged in with their own
  passwords (`accounts.rs`, `pages/login.rs`, sessions and app keys in
  `session.rs`), can edit. Never store a password in plain text, never trust
  a header or a `username=` field as proof of identity, never add another
  way in.
- No secrets in the repo. Run `sh tools/check-secrets.sh` before committing.
- Build and test in Docker on Linux:
  `cd rust && docker run --rm -v "$PWD":/build -w /build rust:1.99-trixie cargo test --locked -p junsgarden`
- Jun is learning: explain in plain words and keep changes small.
