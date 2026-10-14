# 11. How your website works, and how to change it safely

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- How a visit travels through your garden, where each kind of file lives, and the safe way to change things.
- Files in `site/` (style, scripts) need no rebuild: save, then reload the page with `Ctrl+Shift+R`.
- Rust files and page templates need a rebuild: menu `M`, then `7`.
- The safe way, every time: back up (menu `6`), change one thing, rebuild if needed, test (menu `5`), then keep it or undo it.
- `git diff` shows exactly what you changed. `git checkout -- FILE` undoes it.
- Settings and passwords are in `.env`: change them with menu `8`. Never paste `.env` or a password into Gemini.

---

## What you will learn
How a visit travels through your garden, where each kind of file lives, and the
safe way to change things: back up, change one thing, test, keep it (or undo).

## How a visit travels

```
 browser --> Caddy (front door, ports 80/443)
               |-- /auth ...   --> sign-in page (Authelia)         container garden-auth
               |-- /beds ...   --> garden tools (Python)            container junsgarden-ingest
               |                   only after signing in
               |-- everything else --> the website (Rust)            container garden-site
                                        |-- asks the database        container garden-db
                                        |-- reads files in site/     (style, scripts, app download)
                                        |-- reads photos in local/images
```

Before the front door is set up, you visit the website directly at
`http://localhost:8030` on the garden computer.

## Where things live

| Folder or file | What is in it | Change it how |
|---|---|---|
| `site/` | Style (`style.css`), scripts (`js/`), page cards (`sections/`), the app download (`apps/`) | Edit, save, reload the page with `Ctrl+Shift+R`. No rebuild needed. |
| `rust/junsgarden/templates/` | The HTML of each page. `{{ ... }}` are the holes Rust fills in. | Edit, then rebuild (menu `M`, `7`) |
| `rust/junsgarden/src/pages/` | One Rust file per page. `catalog.rs` makes `/catalog.php`. | Edit, then rebuild |
| `rust/junsgarden/src/main.rs` | The list of web addresses and which function answers each | Edit, then rebuild |
| `local/images/` | Your photos (in `Database/`, `Gallery/`, `post/`, `blog/`, `plan/`, `HeroBanner/`) | Upload through the website. Do not rename them by hand: to rename, use menu `M`, then `18` (guide 26). |
| `local/data/` | Comments, guestbook, the facts on the home page | Through the website |
| `.env` | Settings and passwords | Menu `8` Settings. You never need to open it. |
| `docker-compose.yml` | How every program starts (every line has a comment) | Read it to learn. Change it with care. |

The Rust code has comments on almost every line, written for learning.
`docs/rust-port/CODE-TOUR.md` walks through it in order.

## The safe way to change anything (every time)

1. **Back up:** menu `6`, then `1`. It takes a minute.
2. **Change one thing.** Just one.
3. **Rebuild if it was a Rust or template file:** menu `M`, then `7`. If the build
   fails, the red lines name the file and the line. That is Rust checking your work.
4. **Test:** open the page, and run the checklist (menu `5`).
5. **Keep it, or undo it.** In a terminal in the garden folder:

```
git diff                       # shows exactly what you changed
git checkout -- FILE           # undoes your change to FILE
git add -A && git commit -m "Changed the catalogue heading"   # keeps it
```

## Example 1: change a colour (no rebuild)
1. Open `site/style.css` in the Text Editor (right-click it, then "Open With Text Editor").
2. Search (`Ctrl+F`) for a colour such as `#2f7d32` and change it.
3. Save, then reload the website with `Ctrl+Shift+R`.

## Example 2: change words on a page (rebuild)
1. Find the page's template, for example `rust/junsgarden/templates/catalog.html`.
2. Change a heading's text and save.
3. Menu `M`, then `7` (rebuild). Wait for the health line `{"status":"ok", ...}`.
4. Reload the page.

## Run the tests (optional)
`bash garden.sh test` runs the 64 automatic checks that come with the code. The
first time downloads Rust (about 1.5 GB) and takes a while. After that it is quick.

## Working with Gemini on the code
- Gemini CLI reads `GEMINI.md` in this folder, which holds the rules for helping you
  (guide 08).
- Paste the exact red error lines, and say which file you changed.
- Never paste `.env` or a password.
- Check what it suggests with `git diff` before you keep it.

## Read more (official)
- Rust, the book: <https://doc.rust-lang.org/book/>
- axum (the web server part): <https://docs.rs/axum>
- askama (the page templates): <https://askama.readthedocs.io/>
- sqlx (talking to the database): <https://docs.rs/sqlx>
- git basics: <https://git-scm.com/book/en/v2/Git-Basics-Undoing-Things>
