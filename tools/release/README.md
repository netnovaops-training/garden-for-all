# tools/release: how this package is built and tested

None of this is needed to RUN the garden. These are the scripts the author used to
build the downloads, make the PDFs and test every menu option on throwaway
machines, kept here so anyone can see (and repeat) how the package was made.

## Building

| Script | What it makes |
|---|---|
| `make_zip.py` | The three downloads: `junsgarden-for-jun.zip` (only what is not on GitHub: the plants database, the music, the handbook, the start files, `CODE-SOURCE.txt` and `FILES-FROM-AUTHOR.txt`), `junsgarden-photos-addon.zip`, `junsgarden-wiki-addon.zip`. Fixes line endings, marks `.sh` files runnable, writes `FILES-FROM-AUTHOR.txt` (a fingerprint of every file) and the add-ons' fingerprints. `python tools/release/make_zip.py` |
| `make_pdf_zips.py` | The two PDF guide downloads (one per language) from the folders `make_pdfs.py` built, with their SHA-256; `--list-file PDF-GUIDES-SHA256.txt` writes the list the installer checks. They are attached to a GitHub release (tag in the list's `url=` line), not kept in git and not on Drive. `python tools/release/make_pdf_zips.py OUT --english DIR --korean DIR --list-file PDF-GUIDES-SHA256.txt` |
| `make_photo_parts.py` | The photos add-on for the public hosts: encrypts the zip (openssl AES-256-CTR, one stream), cuts it into parts (`--part-mb 25`) with a SHA-256 each, writes `PHOTO-PARTS.json` (`--pin N=HOST` pins a part to one host, `--author-every N` puts every Nth part on the author's server too) and makes the key once into a folder OUTSIDE every repo (`--key-dir`; it refuses a folder in a repo, and never prints the key). `--set-ids LIST IDS.txt --host drive` writes each part's Drive file id into the list afterwards. The key goes only into the main data zip: `GARDEN_PHOTOS_KEY=<key folder>/photos.key python make_zip.py ...` (or `build_release.py --photos-key`) |
| `make_pdfs.py` | The PDFs in `PDF guides/`: every guide as QUICK and FULL, English and Korean, in numbered step folders. Needs Chrome or Edge, and `pip install markdown`. `python tools/release/make_pdfs.py` |
| `make_small_test.py` | A SMALL copy of the garden data for fast tests (all plants, about 40 photos per folder, 40 wiki pages). `python tools/release/make_small_test.py` then `SMALL_DIR=<that folder> python tools/release/make_zip.py <out>/junsgarden-for-jun-TEST-small.zip` |
| `make_docker_svgs.py` | The two pictures in "What is Docker?" (English and Korean) |
| `shoot-wizard.ps1` | Pictures of each page of the Windows setup wizard, drawn by the real wizard (nothing is installed); it also writes where the buttons are (`clean-pictures/wizard-marks.json`) for `mark_pictures.py` |
| `mark_pictures.py` | The orange rings and notes ("click Next") on the installer pictures, always drawn on a clean copy |
| `term_shot.py` | A picture of a terminal window from real program output, with its colours and orange rings |
| `capture-menus.sh`, `make_menu_pictures.py` | The newest menu pictures (rename, switch, clean-up): the real output on a test garden, with sample names, then drawn |
| `build_release.py` | A release, built from exactly what Jun will get: the public code at one GitHub commit (`--commit`), checked to be the same as this package, plus what is not on GitHub, the web handbook and the three downloads; then the main zip is checked the way Jun's computer will check it (`tools/get-code.py --check`) |
| `../get-code.py`, `../get-code.ps1` | The first start (Ubuntu, Windows): get the program from GitHub at the commit in `CODE-SOURCE.txt`, check every file against `FILES-FROM-AUTHOR.txt`, and write nothing if one does not match. Only Python 3 or PowerShell, no git |

## Release steps (the order matters)

1. **Photos:** `make_photo_parts.py ZIP PARTS_FOLDER --key-dir <a folder outside every repo> --hosts github,drive --private-part 1 --private-repo OWNER/REPO@main`
   makes the key once (an existing key is reused), the encrypted parts and `PHOTO-PARTS.json`. Part 1 is not public: its file goes to `private-repo/parts/` next to PARTS_FOLDER, never into PARTS_FOLDER. Back the key folder up safely: without `photos.key` the public photo copies can never be opened.
2. **Upload the parts** to each host: a GitHub release (tag in `PHOTO-PARTS.json`; every part except part 1), the private GitHub repository (the files in `private-repo/parts/`, as `parts/<name>` on the branch named in the list; read-only token, 180-day expiry, only that repo, Contents read-only), the Google Drive public copy (all parts) (one file per part;
   direct link `https://drive.google.com/uc?export=download&id=<FILEID>`), optionally the author's server (a subset) or a mirror.
   Put each host's address in `DOWNLOAD-HOSTS.txt` (that file is public code), and the Drive file ids into the list:
   `make_photo_parts.py --set-ids PHOTO-PARTS.json IDS.txt --host drive`.
3. **PDF guides:** `GARDEN_LOGO=<emblem png> GARDEN_PDF_LANG=en python make_pdfs.py <new folder>` (then `ko`), then
   `make_pdf_zips.py OUT --english <en>/English --korean <ko>/Korean --tag <new release tag> --list-file PDF-GUIDES-SHA256.txt`; attach the two zips to a GitHub release with that tag.
4. **Public code:** commit and push the program (this repo; `PDF-GUIDES-SHA256.txt` and `DOWNLOAD-HOSTS.txt` are in it). The new commit is the PIN.
5. **Build the downloads:** `build_release.py OUT EMBLEM.png --commit <the pin> --photos-key <key folder>/photos.key --photos-token <folder>/photos-access.token --photo-parts PARTS_FOLDER/PHOTO-PARTS.json`
   (or `GARDEN_PHOTOS_KEY` / `GARDEN_PHOTOS_TOKEN` for `make_zip.py`).
   `PHOTO-PARTS.json`, `photos.key` and `photos-access.token` go ONLY into the main data zip (they are not in git: `.gitignore` lists them, and the build
   stops if one is found in the public code or if the key does not belong to the list). The list therefore never changes the pin: adding or fixing
   a host's file ids is a new main zip, not a new code commit.
6. Put the main zip on Drive (replace the old one), update `DOWNLOADS-SHA256.txt`, and check the fingerprints.

## Where the data download and the Drive files came from

| Script | What it does |
|---|---|
| `export-from-server.sh` | How the garden's data was taken from the old live server: the plants database, photos, data and AI cache, in the garden's own backup format. The logins (password hashes, sessions) were left out on purpose: the new owner sets his own |
| `upload-to-drive.sh` | How the three downloads go to the Google Drive folder (the PDFs come with the program now), how each file is checked there (Drive's fingerprint must match), and the SHA-256 fingerprints for `DOWNLOADS-SHA256.txt`. The Drive account, the folder's link and any keys are given when it runs; none of them are in the package |

`PDF guides/` (every guide as a PDF, `English/`, `Korean/`) is made with `make_pdfs.py`
(`GARDEN_PDF_LANG=en` or `ko`) and refreshed at each release, but it is not kept in git: the
installer fetches the two zips from the GitHub release while it installs (`tools/get-pdfs.py`,
`tools/get-pdfs.ps1`, checked against `PDF-GUIDES-SHA256.txt`). `tests/test-get-pdfs.sh` checks that. Both PDFs and the photos are fetched by one program, `tools/get-parts.py` (several hosts from `DOWNLOAD-HOSTS.txt`, small parts, once per version); `tests/test-get-parts.sh` checks it with fake hosts on localhost.

The Korean words of the menu are checked with `python lang/check_ko.py`, and the
code for secrets with `sh tools/check-secrets.sh`.

## Testing (in `tests/`)

The tests ran on fresh, throwaway Ubuntu machines (Windows' WSL), never on a real
garden. Two of them set test-only conveniences that you would never use on a real
computer: a fixed test password, and `sudo` without a password. They say so at
the top.

| Script | What it checks |
|---|---|
| `drive-cycle.ps1` | Builds the small downloads, puts them on Google Drive, downloads them again like Jun would, checks each fingerprint, unzips, and (with `-Install`) installs through the Windows menu |
| `test-linux-from-drive.sh` | The Ubuntu path: downloads the three zips from the Drive links into Home, checks them, unzips, installs (Easy) |
| `check-after-install.sh` | What an install left behind: all 14 steps, the plants, the photos, the wiki off, the web name; then switches the wiki on from its add-on |
| `test-all-options.sh` | Every menu option in turn: password and login, checklist, re-install skipping working steps, backup/verify/restore, keys, moving folders, parts on/off, LiteLLM, the front door (home only, practice, sign-in, guests, editors), subdomains, NAS (a Samba stand-in) and changing NAS, help file, Korean menu, uninstall level 1 and 2 |
| `test-webname.sh` | The Website name swap on a throwaway copy |
| `test-smart-backups.sh` | The automatic backup schedule (menu 6, 7): quiet, medium and busy with test blog posts, the thinning of old automatic backups, the weekly backup, nothing while switched off; removes its test posts again |
| `test-wizard-flow.ps1` | The whole setup wizard in a real window, with nothing started: EASY, CUSTOM through every page, a reopen; every page's Help anchor in the built handbook (English and Korean); the private file of passwords and keys (content, permissions); no secret in `garden-setup.json` or on the command line |
| `test-install-no-questions.sh` | With the wizard, the install does not ask the database password, and the SSH server step finishes when port 22 is already taken |
| `check-site-links.sh` | Every file the website's pages link to (a PDF, a page, the app) is in `site/` (no "not found" for visitors) |
| `test-rename-and-switch.sh` | The rename / move tool and the on / off switch (menu M, 18 and 14): a photo folder, a plant photo named in the database, a page, the redo after a restore, refusals, a big folder, the whole garden folder and back, the clean-up, the check-up, off and on |
| `test-windows-functions.ps1` | The new parts of the Windows menu on their own: the WSL path, Docker Desktop's checks, line endings, the copy that follows a rename, and finding the garden computer again after its address changed |

Every problem these tests found is listed in `CHANGES.md`, with what was wrong
and what changed.

The stable help ids (`START-HERE/help-ids.tsv`: the sections the setup wizard's Help
button and #hot words open) are checked with `python tools/release/check_help_ids.py`
(add `--handbook FOLDER` to check a built handbook too).
