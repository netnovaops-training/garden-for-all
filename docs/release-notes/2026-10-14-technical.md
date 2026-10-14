# Release 2026-10-14: technical notes

The plain-language summary is in `CHANGES.md` (Part 6) and `JOURNAL.md`. This page
is the engineering view: how each part works, the design choices and their
trade-offs, and how each was verified. Stack: Bash (`garden.sh`), PowerShell 5.1
(`windows/`), Docker Compose, MariaDB 11, a Rust (axum) website, Python build tools.

## 1. Rename / move engine (`garden.sh`: `rename_item`, `move_garden`, `do_move_folder`)

**Problem.** A path such as `local/images/Gallery` is referenced in four layers:
compiled Rust constants and Askama templates, browser JavaScript, generated data
files (`tags_cache.csv` stores `images/Gallery/Thumbnails/...`), and database rows
(plant photos are stored as bare file names; Wiki.js pages embed full URLs in
`pages.content` and `pages.render`). Renaming the folder alone breaks the site.

**Resolution of a path to its tokens.** The path is mapped through `realpath` to
its storage area (`IMAGES_DIR`, `SITE_DIR`, `DATA_DIR` from `.env`, which may live
outside the package) and to its web path (`images/…`, site root, `data/…`):

| Area | Tokens searched | Pattern |
|---|---|---|
| Images | `images/<rel>`, `images}/<rel>` (Compose's `${IMAGES_DIR}/plan:` mount) | loose |
| Site, data | `<rel>` (e.g. `privacy.html`, `js/app.js`) | strict |
| Database | `images/<rel>`, the bare file name for photos, `/<rel>` (URL form) | loose |

**Matching rules** (one Perl program, used in `count`, `show` and `apply` modes):
- loose: `(?<![A-Za-z0-9_.-])TOKEN(?![A-Za-z0-9_-]|\.[A-Za-z0-9])`: whole names
  only, so `Gallery` ≠ `Gallery2` and `app.js` ≠ `app.js.map`;
- strict (short names): additionally the token must follow `/ " ' ( = `` ` ``, so the
  same word in prose is never rewritten;
- case-exact, mirroring the Linux file system;
- files are rewritten in place (`>:raw`), preserving inode and permissions.

**Database.** Text columns are enumerated from `information_schema` (base tables
only) for both schemas; one generated SQL batch counts matches per column with
MariaDB's PCRE `REGEXP` (prefixed `(?-i)`, since the default collation is
case-insensitive), and the update uses `REGEXP_REPLACE` with the same pattern.
Affected tables are snapshotted first with `mariadb-dump --single-transaction`
into `logs/rename_<ts>/` (mode 0700). Wiki.js caches rendered pages in its
container, so the wiki is force-recreated when its rows change.

**Ordering and failure handling.** Preview → optional full backup → stop every
service except `db` (with a readiness wait, since a restarted MariaDB refuses
connections for a few seconds; the first test found exactly that) → snapshot →
`mv` (sudo fallback) → file rewrite → SQL → `.env` (`IMAGE_FOLDERS`) → selective
rebuild (`site` if `rust/` changed, `beds` if `beds/` changed) → health wait →
HTTP probe of a file under the new name → git commit of tracked files. Any
failure before the `mv` leaves nothing changed.

**Refusals.** The programs themselves; folders the programs address by name
(`plan/web`, `plan/tags`, `Database/.thumbs`, `site/apps`, which deployed Android
clients poll); names outside `[A-Za-z0-9._/-]`; cross-folder photo moves; photo
names that collide across folders (the database stores bare names, so they would
be ambiguous).

**Restore replay.** `logs/renames.tsv` is an append-only journal. After a restore
or an add-on reload, `redo_renames` folds each entry through later entries
(A→B, B→C ⇒ C; A→B, B→A ⇒ no-op) and re-applies only when the old name exists and
the final one does not. A first version replayed entry by entry and re-applied a
rename that had been undone; the chain resolution fixes that.

**Whole-folder move.** Linux-filesystem check (`stat -f`; NTFS, exFAT and
network mounts are refused because they cannot hold MariaDB's ownership), a
free-space check across devices, `docker compose down` (bind mounts are resolved
at `up`, so containers must be recreated), `sudo mv` with a before/after file
count, `.env` absolute-path rewrite, crontab rewrite (`cd "<old>"` →
`cd "<new>"`), desktop entry regeneration, and an optional symlink at the old
path, which keeps the Windows controller (it addresses `~/junsgarden-for-jun`)
working. Moving back onto the signpost is detected and allowed.

## 2. Lifecycle: off / on / check-up / clean-up (`do_power`)

- **Off** is `docker compose down` without `-v` (containers and networks only; all
  state is in bind mounts) plus disabling Caddy, recorded in a `.garden-off` marker
  that also gates the nightly backup cron job.
- **On** is `up -d`; an image lost meanwhile is rebuilt by Compose. It waits on
  `/healthz`, reads the plant count, and offers a backup after a long gap.
- **Check-up** parses `docker compose ps -a` (fields separated by `|`, not tabs:
  bash `read` collapses consecutive tabs, which shifted the exit code into the
  empty health column and restarted a healthy one-shot job; found in testing), then
  recreates services that are exited with an error, unhealthy or restarting, and
  probes `/healthz`. A cron mode runs it quietly every 6 hours.
- **Clean-up** computes the keep-set from `docker compose --profile <all> config
  --images`, normalised to `name:tag`. A candidate image must be unused by any
  container (`--filter ancestor`), be dangling or another tag of a garden
  repository, and be older than 7 days (`docker image inspect .Created`). It is
  removed with `docker rmi` without `-f`, so the daemon refuses anything in use.
  Build cache is pruned with `until=720h`. The confirmation defaults to No. This
  is safe on a Docker Desktop shared with other projects, where
  `docker system prune` (previously recommended in the guides) is not.

## 3. Windows controller (`windows/*.ps1`)

- **Docker Desktop mode.** It detects Docker Desktop in both installation layouts
  (Program Files, and the newer per-user `%LOCALAPPDATA%\Programs\DockerDesktop`),
  installs it with `winget` if needed, waits for the engine, then polls inside the
  distro until `docker info` reports `Docker Desktop` (WSL integration on).
  `garden.sh` reads the mode from `.garden-docker` and skips Engine installation.
  In Engine mode it detects Desktop's integration symlinks in `/usr/bin` and stops
  with instructions, rather than silently running on the wrong engine.
- **Host discovery.** `Garden-Network.ps1` takes the IPv4 interface that has the
  default gateway, issues 254 concurrent `TcpClient.ConnectAsync(x, 22)` calls
  with a single 1.5 s `Task.WaitAll` budget (about 2 s on a real network), and
  then reverse DNS with a timeout per host. Used by the installer's "Find it for
  me" and by the menu.
- **Address drift.** The target's MAC is taken from `Get-NetNeighbor` after a
  port-22 knock. Before each job, and before the weekly backup copy, the saved IP
  is probed; on failure the subnet is swept and the MAC (or the hostname) is
  matched, then saved. Tested with a deliberately wrong IP: corrected in 7 s; an
  unknown MAC is not matched.
- **Line endings.** Files that run in Ubuntu are normalised from CRLF to LF before
  packing, with backups; the skip list is matched on the path relative to the
  package (a first version matched `\Local\` in `%TEMP%` and skipped everything).
- **WSL paths** fall back from `\\wsl.localhost` to `\\wsl$` (older Windows 10).
- **Rename sync.** `garden.sh` writes `logs/last-rename.txt` (`edit`/`move`
  records); the controller mirrors them into its own copy, so a later push from
  the PC does not reintroduce old names.

## 4. Offline handbook search (`tools/release/make_handbook.py`)

The handbook is static HTML opened from `file://`, where `fetch()` of a local
JSON file is blocked. So the index is emitted as a script
(`window.GARDEN_INDEX = [...]`): one record per `h2`/`h3` section of every
handbook page (the short version, the full English text and the Korean, with
Korean anchors from `slugify_unicode`), plus the guides that are not handbook
pages. That is 1,031 records, about 0.9 MB. The query is an AND of
case-insensitive substrings (which also suits Korean, where word splitting is
unreliable), weighted toward title and heading matches. Results deep-link to
the section; the page script selects the right language tab from the anchor.
It also has a language filter, a list of common topics, keyboard shortcuts
(`/`, `Enter`, `Esc`) and `?q=` deep links. Nothing leaves the machine.

## 5. Other fixes

- Two-factor sign-in: `write_authelia` regenerates `configuration.yml` on several
  flows; it now preserves a hand-set `two_factor` policy (tested in isolation with
  stubbed helpers).
- The Compose `container_name` for n8n is now `garden-n8n`, avoiding a
  name-space clash on a shared Docker.
- A read of every guide against the code by an audit agent, cross-checking menu
  numbers, command words, ports, container names, files and counts. It found 6
  errors and about 12 omissions, all fixed, in English and Korean.

## 6. Verification

| Area | How |
|---|---|
| Rename / move / switch | `tools/release/tests/test-rename-and-switch.sh` on an installed WSL test garden: each operation is applied and reverted, then checked over HTTP and SQL |
| Windows functions | `tools/release/tests/test-windows-functions.ps1`: functions loaded from the script's AST and run against a scratch copy, a test distro and a real LAN |
| Website | `cargo test`: 64/64 |
| Shell | shellcheck: 26 scripts clean |
| Localisation | `lang/check_ko.py` (938/938 strings); `check_ko_guides.py` (links, code and pictures equal in both languages) |
| Links | `check_links.py`: 264 URLs (rate-limited hosts checked by hand) |
| Secrets | gitleaks (files and history) and semgrep: clean |

## 7. Supply chain and transparency

Every script used to produce the published artefacts is in `tools/release/`:
the data export from the old server (it omits credential tables), the Drive
upload with server-side MD5 verification, the picture pipeline, and the test
suites. Personal identifiers (accounts, the Drive folder ID, keys, e-mail
addresses) are runtime parameters, never constants. `DOWNLOADS-SHA256.txt`
publishes SHA-256 fingerprints of the downloads.
