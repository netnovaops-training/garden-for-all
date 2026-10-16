# What gets installed on your computer

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Every program the garden setup can install, why it is there, and its official page. Nothing else is installed.
- Three groups: **Necessary** (the install does them), **Optional** (the install asks yes or no) and **Advanced optional** (later, from menu `M`).
- Necessary includes Ubuntu updates, the SSH server, the ufw firewall, Docker, the MariaDB database, your website and the automatic backups.
- Nothing gives anybody else access to your computer. The setup only downloads programs from the official places listed.
- Only ports 80 and 443 (Caddy, after the front door step) can be reached from the internet. SSH (22) is for your home network only. The rest are for this computer only.
- The checklist (menu `5`) checks all of this every time you run it.
- To remove it: menu `M`, then `12` (Uninstall). It keeps your photos, database and backups.

---

Every program the garden setup can install is on this page: what it is, why it
is there, and the official page it comes from, so you can read more. Nothing
else is installed. The setup also shows each of these on screen, with its
commands, before it runs them, and saves everything it prints in `logs/`.

Three groups:
- **Necessary.** The website cannot run without these. The install does them.
- **Optional.** These are recommended. The install asks yes or no for each one.
- **Advanced optional.** Do these later, from the menu (`M` for More).

Nothing gives anybody else access to your computer. Every password is made on
your computer or typed by you. The setup sends nothing anywhere: it only
downloads programs from the official places listed below.

---

## Necessary

| What | What it is | Why | Where it runs | Official page |
|---|---|---|---|---|
| Ubuntu updates | Ubuntu's own updates through `apt` (its app store) | Safety fixes come first | Ubuntu | <https://ubuntu.com/server/docs/how-to/software/package-management/> |
| git, curl, openssl, rsync | Small standard tools | git keeps a history of your changes. curl fetches web pages. openssl makes random passwords. rsync is a copying tool other programs use (you never need to run it yourself). | Ubuntu | <https://git-scm.com/doc>, <https://curl.se/docs/> |
| OpenSSH server | Lets you use this computer's terminal from another computer at home | So you can manage the laptop with the lid closed | Ubuntu, port 22, home network only | <https://ubuntu.com/server/docs/openssh-server> |
| ufw firewall | Decides which connections may come in | Blocks everything except SSH from home, and later 80 and 443 | Ubuntu | <https://ubuntu.com/server/docs/how-to/security/firewalls/> |
| Docker Engine and Compose | Runs each program in its own sealed box (a "container") | Programs can't break Ubuntu or each other. The same setup works on any computer. | Ubuntu | <https://docs.docker.com/engine/install/ubuntu/> |
| MariaDB 13 | The database. It speaks the same language as MySQL. | Holds plants, comments, logins and the wiki's pages | Docker, inside only (no port) | <https://mariadb.com/kb/en/documentation/> |
| Jun's Garden website | Your site, written in Rust, built on your computer from `rust/` | It is your garden: catalogue, gallery, blog, data, guestbook, plant admin, the phone app's server | Docker, `127.0.0.1:8030` | <https://www.rust-lang.org/learn>, `docs/rust-port/CODE-TOUR.md` |
| Toolbox | A tiny Alpine Linux container with database and archive tools | Backup, restore and set-password work the same on Ubuntu and Windows | Docker, only while a job runs | <https://alpinelinux.org/about/> |
| Backup schedule | One line in your cron (Ubuntu's alarm clock) | Every hour it counts what changed: a full backup hourly while busy, within 6 hours after a few edits, at least weekly when quiet; thinned to about 8 GB (or one every night at 2:30 am, if you choose) | Ubuntu | <https://manpages.ubuntu.com/manpages/noble/man5/crontab.5.html> |
| Swap file (small laptops only) | Disk space used as extra memory | Building the website needs about 3 GB of memory | Ubuntu | <https://manpages.ubuntu.com/manpages/noble/man8/swapon.8.html> |

## Optional (the install asks)

| What | What it is | Why | Where it runs | Official page |
|---|---|---|---|---|
| Garden tools (`/beds`) | A small Python program: bed planner, checkup, diagnose, tag scan, spend, wiki queue | They were part of the live garden. The phone app's camera tools use them too. | Docker, `127.0.0.1:8025` | `beds/README.md`, <https://docs.python.org/3/> |
| Wiki (off unless you choose it) | Wiki.js 2, a modern wiki. Your garden articles are a separate download, `junsgarden-wiki-addon.zip`; without it the wiki starts empty | Your encyclopedia, yours to edit. Only if you want it: one more program to download, and about 30 minutes to add the pages | Docker, `127.0.0.1:3000` | <https://docs.requarks.io>, <https://js.wiki> |
| AI keys (Gemini, OpenRouter) | Nothing is installed: you paste YOUR own free keys with menu `K`. Gemini (Google AI Studio) reads tag photos and fills plant facts. OpenRouter is only a backup that passes the same request to a free model when Gemini is busy. Both are your own accounts; nobody else can see them, and they never see your computer, only the plant photo or question the garden sends. | So the AI boxes work | Settings only | <https://ai.google.dev/gemini-api/docs>, <https://openrouter.ai/docs>, guide 16 |
| Email (Gmail) | Your Gmail sends the site's emails (password links) | Email sent straight from a home connection lands in spam | Settings only, nothing installed | <https://support.google.com/accounts/answer/185833> |
| Desktop icon | "Jun's Garden" in the app list and on the desktop | Opens the menu with one click | Ubuntu | <https://specifications.freedesktop.org/desktop-entry-spec/latest/> |
| Docker Desktop (Windows only, if you choose it) | Docker Inc.'s Windows app: it runs the garden's programs and shows them with buttons (start, stop, their messages). Installed with winget; free for personal use | The easy way to manage the garden on a Windows PC. The other way, Docker inside Ubuntu, needs nothing more on Windows | Windows | <https://docs.docker.com/desktop/setup/install/windows-install/>, guide 26 |

## Advanced optional (later, from menu `M`)

| What | What it is | Why | Where it runs | Official page |
|---|---|---|---|---|
| Caddy | A web server that is the front door, with automatic HTTPS | One door with the same gates as the old site. It gets the padlock certificate by itself. | Ubuntu, ports **80 and 443**, the only two the router forwards | <https://caddyserver.com/docs/> |
| Authelia | Your sign-in page, at `/auth` | The garden tools open only after you sign in. Wrong passwords lock the name for a while. | Docker, `127.0.0.1:9091` | <https://www.authelia.com/overview/prologue/introduction/> |
| No-IP updater (DUC) | Tells No-IP your home internet address every few minutes | Keeps `junsgarden.zapto.org` pointing at your home | Ubuntu | <https://www.noip.com/support/knowledgebase/install-linux-3-x-dynamic-update-client-duc> |
| Router settings | You do this in your router: reserve the laptop's address, forward 80 and 443 | Visitors from the internet can reach Caddy, and only Caddy | Your router | `START-HERE/14-reserve-your-ip.md` |
| n8n | Drag-and-drop automations ("soil is dry, so email me") | Fun and powerful when you are ready | Docker, `127.0.0.1:5678` | <https://docs.n8n.io> |
| Adminer | A one-page web viewer for the database | Look inside the tables without typing SQL | Docker, `127.0.0.1:8082`, only while you use it | <https://www.adminer.org> |
| LiteLLM (menu `M`, `15`) | One private address for AI, with a monthly spending limit | So a paid AI key can never cost more than you chose | Docker, `127.0.0.1:4000` | <https://docs.litellm.ai> |
| DuckDNS updater (menu `M`, `13`) | One cron line that tells DuckDNS your home address, for subdomains | Free extra names (wiki., n8n., ...) behind your sign-in page | Ubuntu (cron) | <https://www.duckdns.org/install.jsp> |
| Tailscale (menu `M`, `17`) | Tailscale's own program: a private network of only your devices | Reach the garden computer from anywhere, with no port opened in the router | Ubuntu | <https://tailscale.com/kb/1017/install> |
| Local AI (Ollama) | AI models on a strong computer at home (not this laptop) | Private, free AI for the site's AI boxes | That other computer | <https://github.com/ollama/ollama>, `START-HERE/15-local-ai.md` |
| cifs-utils (NAS) | Ubuntu's program for connecting a network drive (SMB), from Ubuntu's store | A second copy of every backup on your NAS | Ubuntu (menu 6, then 4) | <https://manpages.ubuntu.com/manpages/noble/man8/mount.cifs.8.html> |
| rclone (Google Drive) | A well-known free program that copies files to cloud storage, from Ubuntu's store | A weekly copy of your backup away from home (3-2-1) | Ubuntu (menu 6, then 5) | <https://rclone.org/drive/> |
| Android Studio (your choice, not installed by the setup) | Google's program for building Android apps | To work on the garden's phone app in `android-app/` | Your desktop | <https://developer.android.com/studio>, `START-HERE/17-android-app-and-ai-coding.md` |

The wiki's one-time setup helper is built here from `docker/wiki/` on the
official **python:3.12-alpine** image (<https://hub.docker.com/_/python>) and
is thrown away after it runs.

---

## The doors (ports): what can be reached from where

| Port | Program | Who can reach it |
|---|---|---|
| 22 | SSH | Only computers on your home network (firewall rule) |
| 80, 443 | Caddy (after the front door step) | The internet. These are the only two ports the router forwards. |
| 8030, 8025, 3000, 9091, 5678, 8082, 4000 | website, garden tools, wiki, sign-in page, n8n, Adminer, LiteLLM | This computer only (`127.0.0.1`). Caddy passes visits to them. |
| 3306 | database | Nobody outside Docker |

The checklist (menu `5`) checks all of this every time you run it.

## How to look at anything yourself

| You want to see | Type this in a terminal in the garden folder |
|---|---|
| Which programs are running | `docker ps` |
| What a program has been saying | `docker logs --tail 50 garden-site` (or garden-db, junsgarden-ingest, garden-wiki, garden-auth) |
| Every setting (no passwords) | menu `8` Settings |
| How each program is started | `cat docker-compose.yml` (every line has a comment) |
| How each program is built | `cat rust/Dockerfile`, `cat docker/tools/Dockerfile`, `cat beds/Dockerfile` |
| The front door's settings | `cat /etc/caddy/Caddyfile` |
| What the setup did and when | the files in `logs/`, and `logs/install-progress.txt` |
| The firewall rules | `sudo ufw status verbose` |
| Your backup schedule line | `crontab -l` |

## Removing it

Menu `M`, then `12` (Uninstall), stops and removes the garden's programs, the backup
schedule and the icon, and removes Caddy and the No-IP updater too. It keeps your photos,
database and backups. It prints the command that removes Docker, so you decide.
