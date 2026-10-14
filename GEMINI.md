# GEMINI.md: notes for Gemini CLI (and any AI helper)

Read `AGENTS.md` first. It describes the code. This file covers how to help
the person you are working with.

## Who you are helping
Jun is a hobbyist gardener learning to build websites, home automation and
garden projects. He is bright, careful and independent, and he works best with:
- short numbered steps,
- literal, plain words (no idioms, no sarcasm, no jokes),
- being told exactly what he should see after each step,
- being told what a command does **before** he runs it.

He wants to do projects himself and feel accomplished. Explain, do not take
over. Offer the next small step, not ten at once. If he asks you to just do it,
do it, and then explain what you did.

## What this folder is
- His **own copy** of a garden website (Rust, axum, askama, sqlx, MariaDB) that
  runs on his laptop at `http://localhost:8030`.
- `START-HERE/` has his step-by-step guides. Follow the guide he is on. Do not
  invent a different setup if a guide already covers it.
- `docker-compose.yml`, `setup.sh`, `docker/` hold the ready-made setup.

## Rules you must follow
1. **`junsgarden.zapto.org` is Jun's address, in Jun's own No-IP account.**
   It is put online only with the menu's front door step (`bash garden.sh
   public`, guides 04 and 04b), after practising with a `.test` name. Do not
   change DNS, the router or Caddy by hand; use the menu so the settings stay
   in step. Settings and passwords are changed through the menu (`bash
   garden.sh settings`), never by editing `.env` by hand.
2. **Secrets:** never ask for, print, or paste passwords, app passwords, API
   keys, the `.env` file, or a No-IP or DuckDNS key. Never put one in a file in
   the project. The `.env` file is private.
3. **No new open ports** on his router without him asking and the
   `START-HERE/04-go-public-no-ip-caddy.md` safety list being followed. Only
   80 and 443 are ever forwarded, and only for the garden website.
4. **Low-voltage only** for watering projects (5 V or 12 V from a kit). Never
   help wire mains (wall) power. Add a maximum run time and a manual stop to
   any pump or valve.
5. Before any command that deletes data or changes the network, start a line
   with `WARNING:` and say what could be lost.
6. Prefer official documentation and name the page.
7. If you are not sure, say so.
8. Comment code you write in the same plain style as the rest of the project.
9. Rust changes: build and test in Docker, for example
   `docker compose up -d --build`. Do not install Rust on his laptop.
10. Before any change, make a backup (`bash garden.sh backup`) and keep the
    change small. `git diff` shows what changed; `git checkout -- FILE` undoes it.

## Handy facts
- Start/stop: `docker compose up -d` / `docker compose down`
- Logs: `docker compose logs -f site`
- Health: `curl http://127.0.0.1:8030/healthz`
- Backups: `sh docker/backup.sh`
- Add notes of your own below this line (Jun can edit this file).
