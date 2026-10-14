# Jun's Garden, in Rust

A page-by-page rewrite of the PHP site that lives in the root of this
repository. **The PHP is not modified by anything in here.** It keeps serving
`junsgarden.zapto.org`; this serves `test.junsgarden.duckdns.org` from its own
container on its own port, and only gets promoted a page at a time once each
one matches.

## Why

Three reasons, in the order they actually pay off:

1. **Speed.** PHP re-reads and re-interprets every file on every request. A
   compiled binary does not. `widget_data.php` caches the dashboard for five
   minutes specifically to hide that cost — this does not need the cache.
   Measured 2026-09-24, straight at the app with no TLS or proxy in the way:

   | Route | Time | Size |
   |---|---|---|
   | `/privacy.html` | 1.6 ms | 8.8 KB |
   | `/` | 5.7 ms | 15.9 KB |
   | `/catalog` | 6.8 ms | 192 KB |

2. **Fewer places to get security wrong** — but from specific choices, not
   from the language. `sqlx` checks queries at compile time instead of PDO's
   runtime-only checking; credentials come from the environment instead of
   being written in source; there is one authentication path instead of six
   hand-rolled gates. Rust prevents memory bugs, which PHP did not have. It
   would **not** have prevented the `$USERS` holes found in `app_api.php` on
   2026-09-24 — six write endpoints gated on a variable that had been deleted
   weeks earlier, which failed open into an "undefined variable" warning and
   a broken JSON response. Those compile fine in any language. Logic still
   has to be read.

3. **Types catch a real class of bug here.** `grow_months` and `bloom_months`
   are twelve-character bitmaps, and app-layer bugs have corrupted them
   before — there are one-off repair scripts in this repository because of it.

## What is NOT allowed to change

`app_api.php` is a **contract, not just code**. The companion Android app and
the QR codes physically printed and pushed into the garden beds call it by
exact URL and expect exact JSON. A "tidier" response shape means somebody
scans a tag in the garden and gets nothing.

So it goes last, and the old one keeps serving until the new one matches
response-for-response on every endpoint.

## Order of work

1. Static pages (`/privacy.html` — done)
2. Read-only pages: catalog, plant detail, gallery, blog
3. The JavaScript those pages carry
4. Write paths, behind the household sign-in
5. `app_api.php`, last, verified against the live one

## Running it

Needs a MySQL/MariaDB URL and a port:

```
JUNSGARDEN_DATABASE_URL=mysql://user:pass@host:3306/dbname
APP_PORT=8030
```

Plant images are read from `/images` (mount the site's `images/` directory
there). The app indexes them at startup and says how many it found — if that
number is 0, the mount is wrong.

```
docker build --build-arg APP=junsgarden -t junsgarden/rs:test rust/
```

## Status

Live on `test.junsgarden.duckdns.org`, marked `noindex` so a staging copy of
a real site never outranks the original.

**One thing not yet explained:** `/catalog` returns 192 KB where
`catalog.php` returns 90 KB in about the same time. More content for the same
cost is not automatically a win — it may be rendering rows the PHP version
paginates. The outputs need diffing before that page is called finished.
