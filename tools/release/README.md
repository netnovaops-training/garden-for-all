# tools/release: how this package is built and tested

None of this is needed to RUN the garden. These are the scripts the author used to
build the downloads, make the PDFs and test every menu option on throwaway
machines, kept here so anyone can see (and repeat) how the package was made.

## Building

| Script | What it makes |
|---|---|
| `make_zip.py` | The three downloads: `junsgarden-for-jun.zip` (program, website, plants database), `junsgarden-photos-addon.zip`, `junsgarden-wiki-addon.zip`. Fixes line endings, marks `.sh` files runnable, writes `FILES-FROM-AUTHOR.txt` (a fingerprint of every file) and the add-ons' fingerprints. `python tools/release/make_zip.py` |
| `make_pdfs.py` | The "Setup documentation" PDFs: every guide as QUICK and FULL, English and Korean, in numbered step folders. Needs Chrome or Edge, and `pip install markdown`. `python tools/release/make_pdfs.py` |
| `make_small_test.py` | A SMALL copy of the garden data for fast tests (all plants, about 40 photos per folder, 40 wiki pages). `python tools/release/make_small_test.py` then `SMALL_DIR=<that folder> python tools/release/make_zip.py <out>/junsgarden-for-jun-TEST-small.zip` |
| `make_docker_svgs.py` | The two pictures in "What is Docker?" (English and Korean) |
| `shoot-wizard.ps1` | Pictures of each page of the Windows guided installer, drawn by the real installer (nothing is installed) |
| `mark_pictures.py` | The orange rings and notes ("click Next") on the installer pictures, always drawn on a clean copy |
| `term_shot.py` | A picture of a terminal window from real program output, with its colours and orange rings |
| `capture-menus.sh`, `make_menu_pictures.py` | The newest menu pictures (rename, switch, clean-up): the real output on a test garden, with sample names, then drawn |
| `build_release.py` | Everything above in one go, from a copy of only the files git tracks: the downloads, the PDFs, the web handbook |

## Where the data download and the Drive files came from

| Script | What it does |
|---|---|
| `export-from-server.sh` | How the garden's data was taken from the old live server: the plants database, photos, data and AI cache, in the garden's own backup format. The logins (password hashes, sessions) were left out on purpose: the new owner sets his own |
| `upload-to-drive.sh` | How the three downloads and the PDFs go to the Google Drive folder, how each file is checked there (Drive's fingerprint must match), and the SHA-256 fingerprints for `DOWNLOADS-SHA256.txt`. The Drive account, the folder's link and any keys are given when it runs; none of them are in the package |

`PDF guides/` in the package holds every guide as a PDF (`English/`, `Korean/`), made with
`make_pdfs.py` (`GARDEN_PDF_LANG=en` or `ko`) and refreshed at each release.

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
| `test-rename-and-switch.sh` | The rename / move tool and the on / off switch (menu M, 18 and 14): a photo folder, a plant photo named in the database, a page, the redo after a restore, refusals, a big folder, the whole garden folder and back, the clean-up, the check-up, off and on |
| `test-windows-functions.ps1` | The new parts of the Windows menu on their own: the WSL path, Docker Desktop's checks, line endings, the copy that follows a rename, and finding the garden computer again after its address changed |

Every problem these tests found is listed in `CHANGES.md`, with what was wrong
and what changed.
