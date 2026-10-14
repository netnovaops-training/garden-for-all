# tools/release: how this package is built and tested

None of this is needed to RUN the garden. These are the scripts Jason used to
build the downloads, make the PDFs and test every menu option on throwaway
machines, kept here so anyone can see (and repeat) how the package was made.

## Building

| Script | What it makes |
|---|---|
| `make_zip.py` | The three downloads: `junsgarden-for-jun.zip` (program, website, plants database), `junsgarden-photos-addon.zip`, `junsgarden-wiki-addon.zip`. Fixes line endings, marks `.sh` files runnable, writes `FILES-FROM-JASON.txt` (a fingerprint of every file) and the add-ons' fingerprints. `python tools/release/make_zip.py` |
| `make_pdfs.py` | The "Setup documentation" PDFs: every guide as QUICK and FULL, English and Korean, in numbered step folders. Needs Chrome or Edge, and `pip install markdown`. `python tools/release/make_pdfs.py` |
| `make_small_test.py` | A SMALL copy of the garden data for fast tests (all plants, about 40 photos per folder, 40 wiki pages). `python tools/release/make_small_test.py` then `SMALL_DIR=<that folder> python tools/release/make_zip.py <out>/junsgarden-for-jun-TEST-small.zip` |
| `make_docker_svgs.py` | The two pictures in "What is Docker?" (English and Korean) |
| `shoot-wizard.ps1` | Pictures of each page of the Windows guided installer, drawn by the real installer (nothing is installed) |

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

Every problem these tests found is listed in `CHANGES.md`, with what was wrong
and what changed.
