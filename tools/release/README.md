# tools/release: how this package is built and tested

None of this is needed to RUN the garden. These are the scripts the author used to
build the downloads, make the PDFs and test every menu option on throwaway
machines, kept here so anyone can see (and repeat) how the package was made.

## Building

| Script | What it makes |
|---|---|
| `make_zip.py` | The three downloads: `junsgarden-for-jun.zip` (only what is not on GitHub: the plants database, the music, the handbook, the start files, `CODE-SOURCE.txt` and `FILES-FROM-AUTHOR.txt`), `junsgarden-photos-addon.zip`, `junsgarden-wiki-addon.zip`. Fixes line endings, marks `.sh` files runnable, marks `.sh` files runnable, writes `FILES-FROM-AUTHOR.txt` (with `GARDEN_CODE_REF=main`: a fingerprint of each file the zip carries; with `GARDEN_CODE_COMMIT`: of every file) and the add-ons' fingerprints. `python tools/release/make_zip.py` |
| `make_pdf_zips.py` | The two PDF guide downloads (one per language) from the folders `make_pdfs.py` built, with their SHA-256; `--list-file PDF-GUIDES-SHA256.txt` writes the list the installer checks. They are attached to a GitHub release (tag in the list's `url=` line), not kept in git and not on Drive. `python tools/release/make_pdf_zips.py OUT --english DIR --korean DIR --list-file PDF-GUIDES-SHA256.txt` |
| `make_photo_parts.py` | The photos add-on for the public hosts: encrypts the zip (openssl AES-256-CTR, one stream), cuts it into parts (`--part-mb 25`) with a SHA-256 each, writes `PHOTO-PARTS.json` (`--pin N=HOST` pins a part to one host, `--author-every N` puts every Nth part on the author's server too) and makes the key once into a folder OUTSIDE every repo (`--key-dir`; it refuses a folder in a repo, and never prints the key). `--set-ids LIST IDS.txt --host drive` writes each part's Drive file id into the list afterwards. The key goes only into the main data zip: `GARDEN_PHOTOS_KEY=<key folder>/photos.key python make_zip.py ...` (or `build_release.py --photos-key`) |
| `make_pdfs.py` | The PDFs in `PDF guides/`: every guide as QUICK and FULL, English and Korean, in numbered step folders. Needs Chrome or Edge, and `pip install markdown`. `python tools/release/make_pdfs.py` |
| `make_small_test.py` | A SMALL copy of the garden data for fast tests (all plants, about 40 photos per folder, 40 wiki pages). `python tools/release/make_small_test.py` then `SMALL_DIR=<that folder> python tools/release/make_zip.py <out>/junsgarden-for-jun-TEST-small.zip` |
| `make_docker_svgs.py` | The two pictures in "What is Docker?" (English and Korean) |
| `shoot-wizard.ps1` | Pictures of each page of the Windows setup wizard, drawn by the real wizard (nothing is installed); it also writes where the buttons are (`clean-pictures/wizard-marks.json`) for `mark_pictures.py` |
| `mark_pictures.py` | The orange rings and notes ("click Next") on the installer pictures, always drawn on a clean copy |
| `term_shot.py` | A picture of a terminal window from real program output, with its colours and orange rings |
| `capture-menus.sh`, `make_menu_pictures.py` | The newest menu pictures (rename, switch, clean-up): the real output on a test garden, with sample names, then drawn |
| `build_release.py` | A release, built from exactly what Jun will get: the public code (`--ref main`: main is looked up and resolved to its commit; or `--commit <40 hex>`), checked to be the same as this package, plus what is not on GitHub, the web handbook and the three downloads; then the main zip is checked the way Jun's computer will check it (`tools/get-code.py --check`). `--ref main` writes `ref=main` into `CODE-SOURCE.txt` and stops if main moves during the build |
| `../get-code.py`, `../get-code.ps1` | The first start (Ubuntu, Windows): get the program from GitHub as `CODE-SOURCE.txt` says (`ref=main`, or a 40-hex commit to stay on one version) and check every file against the public list `CODE-FINGERPRINTS.txt` (a version from before the list existed: against `FILES-FROM-AUTHOR.txt`). A file that differs, is missing or is extra stops everything and nothing is written. Only Python 3 or PowerShell, no git |
| `make_fingerprints.py`, `hooks/pre-commit`, `install-hooks.sh` | `CODE-FINGERPRINTS.txt`: the fingerprint of every tracked program file (not the list itself, not the files kept off GitHub: `README.md`, `site/Boogie.mp3`, `FILES-FROM-AUTHOR.txt`, `DOWNLOADS-SHA256.txt`, the key and token, `PHOTO-PARTS.json`). `python tools/release/make_fingerprints.py` writes it from the staged files, `--check` only checks. The pre-commit hook (`sh tools/release/install-hooks.sh` installs it) makes the list again at every commit and adds it to the commit |

## Release steps (the order matters)

1. **Photos:** `make_photo_parts.py ZIP PARTS_FOLDER --key-dir <a folder outside every repo> --hosts github,drive --private-part 1 --private-repo OWNER/REPO@main`
   makes the key once (an existing key is reused), the encrypted parts and `PHOTO-PARTS.json`. Part 1 is not public: its file goes to `private-repo/parts/` next to PARTS_FOLDER, never into PARTS_FOLDER. Back the key folder up safely: without `photos.key` the public photo copies can never be opened.
2. **Upload the parts** to each host: a GitHub release (tag in `PHOTO-PARTS.json`; every part except part 1), the private GitHub repository (the files in `private-repo/parts/`, as `parts/<name>` on the branch named in the list; read-only token, 180-day expiry, only that repo, Contents read-only), the Google Drive public copy (all parts) (one file per part;
   direct link `https://drive.google.com/uc?export=download&id=<FILEID>`), optionally the author's server (a subset) or a mirror.
   Put each host's address in `DOWNLOAD-HOSTS.txt` (that file is public code), and the Drive file ids into the list:
   `make_photo_parts.py --set-ids PHOTO-PARTS.json IDS.txt --host drive`.
3. **PDF guides:** `GARDEN_LOGO=<emblem png> GARDEN_PDF_LANG=en python make_pdfs.py <new folder>` (then `ko`), then
   `make_pdf_zips.py OUT --english <en>/English --korean <ko>/Korean --tag <new release tag> --list-file PDF-GUIDES-SHA256.txt`; attach the two zips to a GitHub release with that tag.
4. **Public code:** commit and push the program (this repo; `PDF-GUIDES-SHA256.txt` and `DOWNLOAD-HOSTS.txt` are in it) to `main`, following the merge procedure below, so that `CODE-FINGERPRINTS.txt` matches. Jun's installs follow `main`, so there is no pin to choose.
5. **Build the downloads:** `build_release.py OUT EMBLEM.png --ref main --photos-key <key folder>/photos.key --photos-token <folder>/photos-access.token --photo-parts PARTS_FOLDER/PHOTO-PARTS.json`
   (`--commit <40 hex>` instead of `--ref main` makes a zip fixed to one version; or `GARDEN_PHOTOS_KEY` / `GARDEN_PHOTOS_TOKEN` for `make_zip.py`).
   `PHOTO-PARTS.json`, `photos.key` and `photos-access.token` go ONLY into the main data zip (they are not in git: `.gitignore` lists them, and the build
   stops if one is found in the public code or if the key does not belong to the list). The list therefore never changes the public code: adding or fixing
   a host's file ids is a new main zip, not a new code commit.
6. Put the main zip on Drive (replace the old one), update `DOWNLOADS-SHA256.txt`, and check the fingerprints.

## CODE-FINGERPRINTS.txt and how `main` is kept right

Jun's installs follow `main` of the public page. What makes that safe is the public list
`CODE-FINGERPRINTS.txt` in the repository: the first start refuses (and writes nothing) if the download
holds a file that differs from the list, a listed file that is missing, or a file that is not on the list.
So **`main` must only ever hold commits where the list matches the files.** Three things keep it so:

1. `tools/release/tests/test-code-fingerprints.sh` fails when the list is stale or a tracked file is missing from it
   (it also proves the check catches a changed, a new and a removed file).
2. The pre-commit hook makes the list again at every commit. Install it once per clone: `sh tools/release/install-hooks.sh`.
3. Merge into `main` like this (the same in the package repository and in the public one):

       git checkout main
       git merge --no-ff --no-commit <branch>        # the merge result is staged, not yet committed
       python tools/release/make_fingerprints.py     # the list again, from the staged merge result
       git add CODE-FINGERPRINTS.txt
       python tools/release/make_fingerprints.py --check
       sh tools/release/tests/test-code-fingerprints.sh
       git commit                                    # the merge commit holds the new list

   Do not use a fast-forward or an automatic merge commit for `main`: they would carry the branch's older list.
   Never push a `main` commit for which `--check` fails.

**If branch protection is switched on for `main`** (GitHub: Settings, Branches, a rule for `main` that requires a pull
request), nobody can push to `main` directly any more, and the steps change:

- Make the branch with the list already right: on the branch, merge `main` into it (`git merge main`), run
  `python tools/release/make_fingerprints.py`, `git add CODE-FINGERPRINTS.txt`, commit, push the branch.
- Open a pull request into `main`. While the branch is up to date with `main` and its own list is right, any merge button gives the right
  result (the merge result is then exactly the branch's files, so the list in it matches).
- If `main` moved while the pull request was open, GitHub asks to update the branch: do it, regenerate the list on the branch
  again, push, then merge.
- A required status check that runs `sh tools/release/tests/test-code-fingerprints.sh` makes GitHub refuse a pull request whose
  list is stale. (This repository has no such workflow yet; adding one is a small `.github/workflows` file.)

**When GitHub itself changes `main`** (merging a Dependabot pull request or a code-scanning fix on the website) the pre-commit hook
does not run, so the list is stale from that moment and every first start refuses ("being updated, try again") until it is
made again. This happened with Dependabot's rustls bump (`rust/Cargo.lock`). Make it again by hand:
`git pull`, `python tools/release/make_fingerprints.py`, `git add CODE-FINGERPRINTS.txt`, commit, push; and give the package repository the same
change. To do it automatically, a workflow like this one (`.github/workflows/fingerprints.yml`; it needs permission to push to `main`, so with branch
protection on, the rule must let this bot through) can be added:

    name: fingerprints
    on:
      push:
        branches: [main]
        paths-ignore: [CODE-FINGERPRINTS.txt]
    permissions:
      contents: write
    jobs:
      refresh:
        runs-on: ubuntu-latest
        steps:
          - uses: actions/checkout@v4
          - run: python3 tools/release/make_fingerprints.py
          - run: |
              git config user.name "github-actions[bot]"
              git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
              git add CODE-FINGERPRINTS.txt
              git diff --cached --quiet || (git commit -m "CODE-FINGERPRINTS.txt: made again for the new main" && git push)

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
| `test-code-fingerprints.sh` | The public list `CODE-FINGERPRINTS.txt`: up to date for the staged files; a changed file, a new tracked file and a removed file are each caught; the pre-commit hook makes the list by itself in a throwaway repository |
| `test_get_code.py` | `tools/get-code.py` and `tools/get-code.ps1` against a tiny local server (127.0.0.1): following `main` (every file right; one tampered file, one extra file, one missing file, a stale list, no list: each refused with nothing written), a path with spaces, brackets and Korean, the old fixed-commit way (also on a real old commit), a fixed commit that has the list, a damaged `CODE-SOURCE.txt`, no answer from GitHub. `--no-powershell` skips the PowerShell half |

Every problem these tests found is listed in `CHANGES.md`, with what was wrong
and what changed.

The stable help ids (`START-HERE/help-ids.tsv`: the sections the setup wizard's Help
button and #hot words open) are checked with `python tools/release/check_help_ids.py`
(add `--handbook FOLDER` to check a built handbook too).
