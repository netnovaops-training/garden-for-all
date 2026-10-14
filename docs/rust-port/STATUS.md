# Page status

> **History note for Jun:** this page records how the site was converted, on
> the old server, in October 2026. The `tools/` scripts it mentions ran on that
> server and are not part of your copy. Your own copy is installed and looked
> after with `garden.sh` (see `START-HERE/`).

Updated 2026-10-06. Every page and endpoint of the PHP site has a Rust
equivalent, and **junsgarden.zapto.org runs on it** (since 2026-10-06 01:37
UTC). The test site runs the same code on copies of the data. The garden's
own password login (Jun and Jason) replaced the household sign-in the same
day.

## How it was verified

- **134 URLs compared with the live PHP** (`tools/urls/everything.txt`,
  `tools/verify.sh`): 129 identical (same DOM for pages, same parsed JSON for
  endpoints, same bytes for images). The 5 that differ are explained and
  accepted: the Data page's "Updated" time and the app's two timestamps
  (DEV-9, time zone) and the order of tied rows in three cross-tabs (DEV-10).
- **54 editing checks** (`tools/verify_admin.sh`) against the test site,
  logging in with a password (until 2026-10-06 they used a simulated
  household sign-in): all pass. They cover who gets in (and who does
  not), posts with photos, guestbook, plants with photos, gallery upload /
  rotate / delete, quantity log, notes, done list, preferences, photo
  position, approve, default photo, bulk move, delete, the comment bell, the
  app's writes, and logout.
- **30 login checks** (`tools/verify_login.sh`) on the test site, using a
  throwaway account "Tester": all pass. The test site has only fake
  accounts and no outbox, so it never emails anyone.
- 42 unit tests pass.
- Screenshots and after-script DOM comparisons from the first session still
  hold for the visitor pages.

## The pages

| Page | State | Notes |
|---|---|---|
| `index.php`, `/`, `/sso-login` | done, incl. editing | posts, photos, guestbook, delete, login box; `/sso-login` only kept so old bookmarks land here |
| `login.php` | new (2026-10-06) | the garden's own login and "Forgot password? / First time?" (DEV-14) |
| `blog.php` | done, incl. delete | |
| `plant_listing.php` | done | |
| `catalog.php` | done, incl. every admin action | DEV-6, DEV-7 |
| `gallery.php` | done | |
| `data.php`, `widget_data.php` | done | weather from the `weather` table; AI calls fail soft; DEV-8 |
| `comments.php` | done, incl. admin | |
| `plant_actions.php` | done | every action |
| `app_api.php` | done, every action | the app's contract: same keys, same shapes |
| `sections/cards/index.php` | done | byte-identical |
| `privacy.html`, `style.css`, `js/*`, images, PDFs | served as the same files | |

## Not ported, on purpose

- `sections/data.php` and `sections/influxdb_config.php`: dead files (nothing
  included them; the second held the old InfluxDB token). Removed from the
  repository.
- `brain.php`'s `brain_api('/health')` detail: the per-bed accuracy table
  of the retired classifier (the "AI Insights" widget is hidden on the page).
- `fix_blog_images.php`, `normalise_db.php`, `garden_normalise.php` as
  scripts: one-off repair tools, not pages. The vocabulary from
  `garden_normalise.php` lives in `normalise.rs`.

## The Android app

The app talks to `app_api.php` on `junsgarden.zapto.org`, and every action
answers the same way. Guests use the app without logging in. To edit, the
app logs in with `POST app_api.php?action=login` (username and password in
the body, never in the address) and gets a 64-character key, good for 90
days. It sends that key with every request as
`Authorization: Bearer <key>`; writes from the app need it.
`action=logout` throws the key away, and `action=whoami` checks it.
