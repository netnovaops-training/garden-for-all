# Beds: the garden tools service

## What it is

A small Python web service that runs next to your garden website in Docker
(container name `junsgarden-ingest`, port 8025). It has two jobs:

1. It serves the garden tool pages under `/beds/...` on your site (the bed
   planner, plant check-ups, receipts and so on).
2. It does the work behind the phone app's camera tools. The app talks to
   the Rust website, and the website passes the photo on to this service.

It uses the same database as the website (`juns_garden_db`). AI is free
cloud AI only (a Gemini free key, then OpenRouter free models). It works
without any AI key; the AI tools then say "AI is not set up yet, see guide
03 part D".

## The pages

Your browser reaches them at `https://<your site>/beds/<page>`.

| Page          | What it does |
|---------------|--------------|
| `/plan`       | The bed planner. Draw a bed, then drag plants from the tray into it. Placing a plant also sets its `flower_bed`. |
| `/checkup`    | Pick a plant, take a photo, and the AI writes what it can see, what that could mean, and what to check. Old check-ups are listed. |
| `/diagnose`   | One photo: the AI says what plant it is and whether anything looks wrong. If the plant is in your catalogue, the check-up is saved to it. |
| `/tag`        | Photograph a nursery tag. The plant is added (name only) and every other detail waits for your yes on `/review`. |
| `/spend`      | Photograph a receipt. Plant lines are matched to your catalogue and saved; totals by year and shop are shown. |
| `/wiki-queue` | Wiki drafts waiting. "Keep" writes one to your own Wiki.js (only if the wiki settings are set); "remove" drops it. |
| `/review`     | The approval queue: say yes or no to each proposed field value, then look again at what was approved (keep, correct or undo). |
| `/healthz`    | Not a page: answers `{"ok": true}` when the service can reach the database. Open to everyone; it shows no garden data. |

### The camera-tool routes (used by the phone app through the website)

The website forwards exactly six actions, as `POST /api/<action>`:
`identify`, `diagnose`, `tag`, `receipt`, `lookup`, `tag-queue`
(see `rust/junsgarden/src/pages/garden_camera.rs`).

## Who may use it

The sign-in gate (Authelia behind Caddy) puts the signed-in person's email
in the `X-Auth-Email` header. Only emails listed in `BEDS_ALLOWED_EMAILS`
get in. Everybody else gets **403** with a short message. `/healthz` is the
only open route.

This is only safe because nobody can reach port 8025 directly: it is
published on `127.0.0.1` only. In Caddy, always remove any `X-Auth-Email`
the visitor sent before copying Authelia's answer in, for example:

```
handle_path /beds* {
    request_header -X-Auth-Email
    forward_auth 127.0.0.1:9091 {
        uri /api/authz/forward-auth
        copy_headers Remote-Email>X-Auth-Email
    }
    reverse_proxy 127.0.0.1:8025
}
redir /beds /beds/plan 308
```

## Settings (environment variables)

| Name | Meaning | Needed? |
|------|---------|---------|
| `DB_HOST` | Database host. Default `db`. | optional |
| `DB_NAME` | Database name. Default `juns_garden_db`. | optional |
| `DB_USER` | Database user. Default `dbadmin`. | optional |
| `DB_PASS` | Database password (from `DB_PASSWORD` in `.env`). | **required** |
| `BEDS_ALLOWED_EMAILS` | Comma-separated emails allowed in, e.g. `jun@example.com,friend@example.com`. Empty = nobody. | **required** |
| `GEMINI_API_KEY` | Free Google Gemini key. | optional |
| `OPENROUTER_API_KEY` | Free OpenRouter key (used when Gemini is busy). | optional |
| `WIKIJS_API_URL` | Your Wiki.js address, e.g. `http://garden-wiki:3000`. | optional |
| `WIKIJS_API_KEY` | A Wiki.js API key (made in the wiki's admin area). | optional |
| `BEDS_PUBLIC_URL` | Where the pages live, for links in answers. Default `/beds`. | optional |
| `PERENUAL_KEY` | A free perenual.com plant-data key, used to fill gaps in web lookups. | optional |
| `PORT` | Port inside the container. Default `8025`. | optional |
| `TZ` | Time zone, e.g. `America/New_York`. | optional |

Fine-tuning (rarely needed): `GARDEN_GEMINI_MODELS`,
`GARDEN_OPENROUTER_MODELS`, `GARDEN_FREEAI_COOLDOWN`,
`GARDEN_FREEAI_MODEL_COOLDOWN`, `GARDEN_FREEAI_ATTEMPT_TIMEOUT`,
`GEMINI_MODEL`, `GEMINI_TIMEOUT`, `GEMINI_RETRIES`, `GEMINI_KEY_FILE`,
`GARDEN_LLM_TIMEOUT`, `GARDEN_VISION_TIMEOUT`, `GARDEN_QUEUE_RETRY`,
`GARDEN_PHOTO_DIR` (default `/photos`), `GARDEN_TABLE` (default `data_source`).

The start-up log says whether AI and the wiki are set up. It never prints a
key or a password.

## Folders

`/photos` inside the container is the `plan` folder of your images folder
(`${IMAGES_DIR}/plan`). Tag photos are saved in `/photos/tags/<plant id>/`
(`unassigned/` while a tag waits in the queue). The service runs as user id
1000; if your images folder belongs to another user, set `user:` in the
compose file to that user's `id -u`:`id -g`.

## How to change a page and rebuild

1. Each page is one Python file holding the HTML: for example the planner
   is `plan_page.py` (`/plan`), the tag page is `tagscan_page.py`.
2. Edit the HTML/JavaScript inside the `"""..."""` string. Links between
   pages are relative (`href="spend"`), so they work under `/beds/`.
3. Rebuild and restart just this service:
   ```
   docker compose up -d --build beds
   ```
4. Check it: `curl -s http://127.0.0.1:8025/healthz` and
   `docker logs junsgarden-ingest`.

The routes themselves are in `service.py` (`_get` for GET, the `routes`
table in `do_POST` for POST).

## Files

| File | What it is |
|------|-----------|
| `service.py` | The web server and every route. |
| `__init__.py`, `vocab.py`, `semantic.py` | The approval gate: new values are matched onto words the catalogue already uses. |
| `freeai.py`, `gemini.py`, `_gemini_prompt.json` | Free cloud AI. |
| `lookup.py`, `identify.py`, `dedupe.py` | Reading tags, web lookups, "what plant is this", "do we have it already". |
| `tagqueue.py`, `tagphotos.py` | The tag queue and the saved tag photos. |
| `beds.py`, `checkup.py`, `purchases.py` | Beds and plantings, check-ups, receipts. |
| `wikidraft.py`, `wikiapply.py`, `wikijs.py` | Wiki drafts and writing them to your Wiki.js. |
| `*_page.py` | The seven pages. |

## What was removed from the household version, and why

This copy is garden-only. Removed (one line each):

- **Paperless** document archive and all its per-person tokens: a household service, not part of your garden.
- **NAS videos / MeTube / YouTube video intake** (`video.py`, `video_intake.py`, `metube.py`): needed the household NAS and downloader.
- **GPG**: household secrets tooling, not used by the garden.
- **The second (household) wiki** (`JASON_WIKI_*`): drafts go only to your own wiki now.
- **Per-person tokens for other household members**: the only identity setting is `BEDS_ALLOWED_EMAILS`.
- **PDF documents and PDF receipts** (`intake.py`, `classify.py`, `destination.py`): needed the household Tika reader; receipts still work from a photo or text.
- **The local GPU model gateway**: free cloud AI only.
- **Garden photo upload, panorama stitching, garden objects** (`photos.py`, `objects.py`): no page here uses them; dropping them also removed OpenCV and NumPy from the image.
- **Bulk backfill, nightly wiki rebuild, weekly e-mail digest and one-off fix-up scripts** (`backfill.py`, `wikisync.py`, `seed-plant-pages.py`, `digest.py`, and others): maintenance jobs with no page, some tied to household mail and models.
- **Read-only search APIs for the household assistant** (`/api/filter`, `/api/vocabulary`, `/api/purchases`): no page here uses them.
