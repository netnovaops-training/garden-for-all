# Jun's Garden: the Rust port

> **History note for Jun:** this page records how the site was converted, on
> the old server, in October 2026. The `tools/` scripts it mentions ran on that
> server and are not part of your copy. Your own copy is installed and looked
> after with `garden.sh` (see `START-HERE/`).

This folder documents the conversion of the garden site from PHP to Rust.
It is written for Jun, who is taking the project over. No passwords, keys or
tokens are in it, and none should ever be added.

## What this is, in plain words

The garden website has been rebuilt page by page in Rust. The goal is
**the same site**: same look, same behaviour, same web addresses (including
the `.php` ones the Android app and the printed QR tags use). Nothing is
redesigned. The reasons are speed, fewer places for a security mistake to
hide, and one small program instead of a PHP server. The one big change
since then is the sign-in: on 2026-10-06 the garden got its own password
login again (see "The sign-in" below and DEV-14).

Branch: `rust-faithful`. Code: `rust/junsgarden/`. Remote: the private Gitea only.

## Where things stand

See `STATUS.md`. Short version: every page and endpoint is ported and
verified, and **junsgarden.zapto.org has run on it since 2026-10-06 01:37 UTC**
(container `junsgarden-rs-live`, switched with `tools/golive.sh`). The PHP
container `nuc-homelab-junsgarden-php-1` was stopped the same night (not
removed): `tools/undo-golive.sh` starts it again and switches back. The
Android app 2.9 was published the same night.

**Why the addresses still end in `.php`.** On purpose. Printed QR codes on
the plant tags, the Android app, bookmarks and search engines all use those
exact addresses; changing them would break every one. The `.php` is only a
name now: no PHP runs anywhere.

## How it works

- **Pages** are Rust code (`rust/junsgarden/src/pages/`) with HTML templates
  (`rust/junsgarden/templates/`). The templates are copied from the PHP
  pages, so the markup is the same.
- **Shared frame** (head, header, hero banner, footer): `layout.rs` plus
  `templates/layout.html`, `header.html`, `footer.html`.
- **Files** (CSS, JavaScript, images, PDFs) are NOT rebuilt. The PHP site's
  own folder is mounted into the Rust container at `/www` and served as it
  is, so `style.css` and every `js/*.js` file are the exact bytes the PHP
  site serves, and no JavaScript needed converting: it calls the same URLs,
  and Rust answers them. Uploaded photos go into the same folder.
- **Only safe file types are served** (`servable()` in `main.rs`): no PHP
  source, no `connection.php`, no `.bak` copies, no `data/` or `cashe/`.
- **Database**: the same MariaDB, same tables. The site makes three tables
  of its own for the sign-in: `garden_sessions` (who is logged in),
  `garden_users` (the two accounts) and `login_attempts` (failed logins).
- **Headers**: the same security headers the PHP pages send. The one
  deliberate omission is the `X-Powered-By: PHP/8.3` version banner.

## The sign-in (the garden's own login, since 2026-10-06)

Until 2026-10-06 the site used the household sign-in (SSO). Jason asked for
the garden to have its own login instead, so now it does. No SSO.

1. Visitors see everything. Nothing asks them to log in. Logging in is only
   for editing.
2. The accounts come from the setting
   `JUNSGARDEN_USERS="Name=email,Name=email"` (GARDEN_USERS in `.env`, written
   by the install with the name and email the owner types). There is no
   built-in account: without that setting nobody can log in. Nobody can
   create an account from the website.
3. The login box (Username, Password, Login) is on the home page and the
   catalogue page. It sends to `/login.php`.
4. `/login.php` also has **"Forgot password? / First time?"**. You type your
   name or your email, and a one-time link is emailed to the address on
   file. The link works for 30 minutes, and only once. That same link is how
   each person sets their **first** password: there are no passwords until
   they do. A password must be at least 10 characters.
5. Passwords are kept only as Argon2 hashes (a one-way scramble) in
   `garden_users`. Reset links are kept only as SHA-256 hashes. So the
   database never holds a password or a working link in plain form.
6. Failed logins are counted in `login_attempts`. Five failures from one
   address, or ten against one name, within 15 minutes, and login is locked
   for 15 minutes.
7. Logging in gives a **new** session id (the `PHPSESSID` cookie). Setting a
   new password logs that person out everywhere.
8. Logout is the same button, and forgets the session.

The reset emails need mail settings (`JUNSGARDEN_SMTP_*`, see
`SETUP-ON-A-NEW-MACHINE.md`). Without them the email is simply not sent, and
the log says "no outbox configured".

**The Android app** logs in with `POST app_api.php?action=login`, with the
username and password in the body (never in the address). It gets back a
64-character key that is good for 90 days, and sends it with every request
as `Authorization: Bearer <key>`. `action=logout` throws the key away;
`action=whoami` checks it. Guests use the app without logging in.

**Caddy**: the garden's live block no longer needs `forward_auth`,
`/sso-login` or an `/auth/` mount. The `/sso-login` address still exists in
the Rust router, only so that old bookmarks land on the home page. Keeping
`request_header -X-Auth-Email` in the block is harmless but no longer
required: the site does not trust that header any more.

Who may upload plant photos is a second list (`JUNSGARDEN_UPLOADERS`).
Its default is now `Jason,Joshua,Jun`, so Jun can upload photos.

## Settings

All by environment variable; the full list with defaults is at the top of
`rust/junsgarden/src/main.rs` and in `SETUP-ON-A-NEW-MACHINE.md`.

## The test site

`test.junsgarden.duckdns.org` runs the Rust build with:

- a **copy** of the database (`juns_garden_db_test`), refreshed from live
  with `tools/refresh_test_db.sh`,
- a **copy** of the images folder (`~/rust-port-work/images-copy`),
- a **copy** of the data folder (guestbook, comments) and of the AI cache.

So nothing done on the test site touches the live site.

**The test site has only fake accounts.** Before the garden had its own
login, nobody could sign in there at all (Jason, 2026-10-05: "I don't want
anybody trying to sign in with my SSO"). Now it runs with
`JUNSGARDEN_USERS=Jason=jason@example.test,Tester=tester@example.test`, so
the real Jun and Jason accounts do not exist on it. It has no outbox (no
mail settings), so it never emails anyone. The test passwords live only in
`~/rust-port-work/test-login.env` on the server (owner-only, mode 600),
never in the repository.

Two scripts test the login on the test site, and both pass:

- `tools/verify_login.sh`: 30 checks of the login itself, using a
  throwaway account, "Tester".
- `tools/verify_admin.sh`: 54 editing checks, logging in with a password.

**Caddy reads a mounted copy of one file.** Saving a new copy of
`Caddyfile.nuc` (git checkout, `sed -i`, most editors) leaves Caddy reading
the old one, and `caddy reload` reloads the old one. Check with
`stat -c %i` of the Caddyfile on the server against
`docker exec caddy-nuc stat -c %i /etc/caddy/Caddyfile`; if they differ,
`docker restart caddy-nuc`. `golive.sh` and `undo-golive.sh` check this.

`tools/deploy_test.sh` on the NUC rebuilds and restarts it; `rollback`
undoes it.

## Switching the live site (done 2026-10-06 with golive.sh; the steps it runs)

1. Make the folders the container writes to writable by it: `images/`,
   `data/`, `cashe/AI/` under the old server's site folder (the
   container runs as user id 999).
2. Run a second container on another LAN port (say 8031) with the LIVE
   mounts: `-v .../www:/www -v .../images:/www/images`, the live database
   URL, `TZ`, and the lists above. No `JUNSGARDEN_DATA_DIR` (so it uses
   `/www/data`) and no `JUNSGARDEN_AI_CACHE_DIR`.
3. In the `junsgarden.zapto.org` block of `Caddyfile.nuc`, change the two
   `reverse_proxy 127.0.0.1:8022` lines to the new port. Keep everything
   else in the block exactly as it is (the `/sso-login` rule, the
   `request_header -X-Auth-Email` line, the `/streamlit` and `/beds`
   routes).
4. `caddy validate`, then `caddy reload`. Rollback is changing the two lines
   back; the PHP container keeps running untouched.
5. Check: home page, sign in, post a test entry, scan a QR tag, open the app.

These are the steps as they were run on the night of the switch, when the
site still used the household sign-in. Since the garden's own login (also
2026-10-06), the `/sso-login` rule and `forward_auth` are no longer needed
in that block, and the `request_header -X-Auth-Email` line is harmless but
not required. See "The sign-in" above.

## How a page was checked

1. `tools/compare_pages.py`: the live PHP page and the Rust page are parsed
   into the same structure and compared tag by tag and text by text. Only
   things PHP randomizes on purpose (hero image order, `bg-theme-N`
   colours, favicon, widget ids) are ignored. Result must be `IDENTICAL`.
2. `tools/compare_images.py`: headless Chrome screenshots of live and Rust
   at desktop and phone width, compared pixel by pixel.
3. Pages whose JavaScript loads data were dumped *after* their scripts ran
   and compared again.
4. JSON endpoints are compared as parsed JSON; images byte for byte.
5. `tools/verify_admin.sh` exercises every editing action against the test
   copies, logging in with a password (54 checks). `tools/verify_login.sh`
   checks the login itself (30 checks).
6. `cargo test -p junsgarden` passes.

`tools/verify.sh -f urls/everything.txt` runs the comparison over 134 URLs.

## Backups taken before any change (on the NUC, `~/backups/junsgarden-rust-port-*`)

Owner-only permissions, never committed to git.

- `database-all*.sql.gz`: every database, integrity-checked
- `www-php-source.tgz`: the PHP site's files
- `images.tgz` (7,738 files), `www-data-dir.tgz`
- `Caddyfile.nuc`, `build-junsgarden-rs.tgz` (the earlier test build)

Restore the database: `zcat database-all.sql.gz | docker exec -i <database container> sh -c 'mariadb -uroot -p"$MARIADB_ROOT_PASSWORD"'`.
A copy of the git history is a bundle on Jason's PC under `AndroidStudioProjects/_backups`.

## Things decided on purpose

See `DEVIATIONS.md`: where the PHP had a bug or a security hole the Rust
page does not copy it, and each such case is written down.

## Known gaps worth knowing about

- **Gallery without its cache file.** The gallery list comes from
  `images/Gallery/tags_cache.csv`, written by `images/Gallery/build_gallery_cache.py`
  (run it after adding photos). The PHP also had a fallback that scanned the
  folder when the file was missing; the Rust page does not, and shows no
  photos (and logs why). On a new machine, run the builder first.
- **The database schema.** A structure-only dump of the current database is
  in `db/schema.sql` for setting up a new machine.
- **The AI and the forecast** depend on things outside this site (the model
  on the desktop PC, files under `/home/jason`). When they are absent the
  boxes say so; the rest of the page is unaffected.
