# 18. Review the code yourself (with friends or an AI)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Everything in this package is plain text you can read. This guide shows how you, a friend or an AI can check it.
- Check nothing changed on the way: menu `M`, then `11`.
- See every change: `git log --oneline`, `git show`, `git diff`. `CHANGES.md` explains the changes made before you got it.
- Scan for secrets and security mistakes (5 minutes): `sh tools/check-secrets.sh`. It prints `CLEAN`, or what it found.
- Ask an AI: give it `AGENTS.md` and `GEMINI.md` first. Never paste `.env` or anything from `local/`.
- Ask a friend: share the code, not your garden. Leave out `.env`, `local/`, `backups/` and `logs/`.
- Something looks wrong? Do not guess: make a help file (menu `H`) and ask someone you trust.

---

Everything in this package is plain text you can read. Nothing is hidden or
compiled ahead of time: the website is built on your computer from the code in
`rust/`, and every program comes from its official home. This guide shows how
**you**, a friend, or an AI helper can check it, as much or as little as you like.

![The Windows guided installer: every page links to what it installs](images/wizard-1-welcome.png)

## 1. What is where (a map)

| Folder or file | What it is |
|---|---|
| `garden.sh` | The one program for Ubuntu: the menu and every install step (heavily commented) |
| `windows/` | The Windows guided installer and the Windows menu (PowerShell, commented) |
| `rust/` | The website's code (Rust). `docs/rust-port/CODE-TOUR.md` walks through it |
| `beds/` | The garden tools' code (Python) |
| `docker/` | The recipes (Dockerfiles) for each program, the Caddy settings template, the wiki helper |
| `docker-compose.yml` | Which programs run, and on which ports (all on this computer only, except Caddy) |
| `android-app/` | The phone app's code |
| `site/` | The pictures, styles and scripts the website serves |
| `CHANGES.md` | Every change made for your copy, and why |
| `FILES-FROM-JASON.txt` | A fingerprint (SHA-256) of every file exactly as it came |
| `WHAT-GETS-INSTALLED.md` | Every program that can be installed, and where it comes from |

## 2. Check that nothing was changed on the way
Menu `M`, then `11` (on Windows: More, `11`). It compares every file with its
fingerprint in `FILES-FROM-JASON.txt` and lists any that differ. (After a
Website name swap, those files differ: that is expected.)

## 3. See every change, line by line
The install keeps a git history of your folder (guide 11). In a terminal in the
garden folder:

```
git log --oneline          # every saved change, newest first
git show                   # the newest change, every line
git diff                   # what you changed since the last save
```

`CHANGES.md` explains each change made before you got it: what was removed or
fixed, why, and how it affects the program.

## 4. Scan for secrets and security mistakes (5 minutes)
Two well-known free scanners, run in Docker so nothing needs installing:
**gitleaks** (passwords, keys and tokens, also in the history) and **semgrep**
(security bugs in the code).

```
sh tools/check-secrets.sh
```

It prints `CLEAN`, or what it found (with any secret value hidden).
Official pages: <https://github.com/gitleaks/gitleaks> and <https://semgrep.dev/docs/>

Your own secrets are never in the code. Your passwords are set by you during the
install and kept only as scrambled hashes on your computer. The other secrets are
made fresh on your computer and kept in `.env`, which git never saves and which
you should never share. Knowing the code does not let anyone sign in as you.

## 5. Every web address the code talks to
This lists every `http`/`https` address in the install programs, so you can see
where anything comes from or goes to:

```
grep -rhoE 'https?://[A-Za-z0-9./_?=&%:-]+' garden.sh windows docker docker-compose.yml beds | sort -u
```

Most are official download pages and documentation. The ones the garden
contacts while running are Google (Gemini, only with YOUR key), OpenRouter
(only with YOUR key), No-IP and DuckDNS (only when you set them up).

## 6. Ask an AI to review it
Gemini (AI Studio, guide 16) can read code. Good questions:

- "Here is garden.sh. Does anything send my data anywhere, or give anyone else
  access to my computer? Quote the lines."
- "Here is docker-compose.yml. Which programs can be reached from other
  computers, and which only from this one?"
- "Here is rust/junsgarden/src/accounts.rs. How are passwords checked and stored?
  Is there any default password or way in without one?"

`AGENTS.md` and `GEMINI.md` in this folder are instructions written for AI
helpers; give them to the AI first. Never paste `.env` or anything from `local/`
into an AI: those hold your passwords and keys.

## 7. Ask a friend to review it
Share the **code**, not your garden: everything except `.env`, `local/`,
`backups/` and `logs/`. If the code is on a public git page, simply send the
link. Your friend can read it, run the scanner in section 4, or install their own
empty copy to try it. That copy is theirs: it has none of your plants, photos,
passwords or keys, and it cannot reach your garden.

## 8. If anything looks wrong
Do not guess: ask. Make a help file (menu `H`; it has no passwords) and show it
to someone you trust, or ask Gemini with the lines that worry you.
