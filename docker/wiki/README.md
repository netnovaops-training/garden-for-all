# Your garden wiki (Wiki.js)

## What is it?

A **wiki** is a website made of pages that you can edit in the browser, like
Wikipedia. Your garden wiki has one page for every plant, every genus, and
lists like "Toxic to dogs and cats".

It runs on **Wiki.js version 2**, a free wiki program.

- Home page of Wiki.js: <https://js.wiki>
- Its manual (docs): <https://docs.requarks.io>
- How to write pages (Markdown): <https://docs.requarks.io/editors/markdown>
- Its API, which the setup job uses: <https://docs.requarks.io/dev/api>

## How to reach it

- On your own computer: **http://localhost:3000**
- Visitors who are not signed in can read every page. Only you (signed in)
  can edit.
- The `127.0.0.1` in front of the port in `docker-compose.yml` means only
  this computer can open it. To put it on the internet later, give it its
  **own web name** (for example `wiki.` in front of your garden name) in
  Caddy. Wiki.js 2 cannot work under a folder like `yourgarden/wiki`.

## The parts

| Part | What it is |
|---|---|
| `garden-wiki` | The wiki itself (container from `ghcr.io/requarks/wiki:2`). |
| `junsgarden_wiki` | The wiki's own database, inside `garden-db`. Pages, tags, menu, settings and your wiki login are all kept here. |
| `wikijs` | The database login the wiki uses. It may only touch `junsgarden_wiki`. Its password is `WIKI_DB_PASSWORD` in your `.env`. |
| `wiki-db-init` | A tiny job that runs on every start. It makes the database and the `wikijs` login if they are missing. Safe to run any number of times. In `docker compose ps -a` it shows as "Exited (0)": that is normal. File: `wiki-db-init.sh`. |
| `wiki-setup` | The one-time setup helper (below). Files: `Dockerfile`, `wiki_setup.py`. |
| `seed/wiki-pages.json.gz` | The pages copied from the old wiki. It is a zipped JSON text file. |
| `export/export_wiki.py` | The script the author used to make that file. You do not need it. |

## The first-time setup job

Start it with:

```
docker compose run --rm wiki-setup
```

It asks you for one thing: **the password for your wiki admin login**.
Choose 10 characters or more. Nothing shows while you type: that is normal.
Your password is never saved in a file. The wiki only keeps a scrambled
"hash" of it, which cannot be turned back into the password.

What it does, step by step:

1. Waits until the wiki answers.
2. If the wiki is already fully set up, it says "Nothing to do" and stops.
   Running it again is always safe.
3. Fills in the Wiki.js setup wizard for you: your email
   (`WIKI_ADMIN_EMAIL` in `.env`) is your login name, plus your password and
   the web address (`WIKI_SITE_URL` in `.env`). It turns telemetry (sending
   usage data to Wiki.js's makers) off.
4. Signs in as you through the wiki's GraphQL API (`/graphql`).
5. Gives the **Guests** group (people who are not signed in) permission to
   read pages, so the pages are public, like before.
6. Adds every page with its title, description, tags, text and original
   address (for example `/en/plants/elberta-peach-404`). Pages already
   there are skipped. If it stops halfway (power cut, closed window), just
   run it again and it carries on. This is the slow part: about 900
   pages, each one is turned into a web page by the wiki. The whole job
   took about 30 minutes on a test computer. Let it run.
7. Draws every page once more, so the links between pages work (while
   pages were being added, a link to a page that did not exist yet was
   marked "missing"). This is the same as the admin button "Rerender all
   pages". Then it puts back each page's original "created" and "last
   edited" dates.
8. Puts back the left-hand menu and names the site **Jun's Garden Wiki**.
9. Writes a small "done" note (setting `junsgardenSeed`) into the wiki's
   settings, so step 2 knows next time.

Then open http://localhost:3000 and sign in with your email and password.
The admin area is at http://localhost:3000/a.

If you forget the wiki password: the wiki can email a reset link only if
you set up email in the admin area. Otherwise ask Gemini: "How do I reset
the admin password of Wiki.js 2 in MariaDB?"

## Where your data lives

Everything is in the database `junsgarden_wiki` inside the `garden-db`
container. The database files are in the folder `DB_DIR` from your `.env`
(the same folder as your plant database). Never edit inside that folder by
hand.

The pictures in the pages are not in the wiki: they are links to
`https://junsgarden.zapto.org/images/...` (your garden site).

## Backups

The garden backup (`bash garden.sh` menu, or `docker compose run --rm tools
backup`) saves the wiki database too, as `wiki.sql.gz` inside the backup
`.tar` file. Restore puts it back the same way. After a restore, restart the
wiki so it reads its settings again: `docker compose restart wiki`.

Keep backups private: the wiki database holds your wiki login's password
hash and the wiki's own secret keys.

To save only the wiki by hand (this asks for nothing and prints no
password, because it reads the password inside the container):

```
docker exec garden-db sh -c 'MYSQL_PWD="$MARIADB_ROOT_PASSWORD" mariadb-dump -uroot --single-transaction junsgarden_wiki' | gzip > wiki-backup.sql.gz
```

## What was deliberately NOT copied, and why

The old wiki ran on the author's server. Only the **content** came across:
pages, tags, the menu, and the "guests may read" rule.

These were left behind on purpose:

- **the author's logins** (users and their password hashes). Your wiki has only
  your login. A hash is still a secret: it can be attacked to guess the
  password.
- **API keys** and **sign-in settings** (`apiKeys`, `authentication`). They
  are keys to the author's server.
- **The session secret and certificates** (`settings`). Your wiki makes its
  own new ones during setup. Sharing them would let someone fake a
  sign-in.
- **Page history** (more than 18,000 old versions). It is very big and
  not needed: every page arrives in its newest version. New history starts
  when you edit.
- **Comments, analytics, link lists, search index.** There were no
  comments; the rest the wiki rebuilds by itself.
