# 21. Every program, why it is here, and where it comes from

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- This page lists **every** program the garden can use, necessary and optional,
  in one or two sentences each, with a link to its maker's own documentation.
- Each program comes from its **official** source: Ubuntu's own store, the
  maker's own package, or the maker's official Docker image.
- The version the garden uses is shown, so you can read the right documents.
- HTTPS (the padlock) comes from **Let's Encrypt**, for free and by itself;
  how that works is explained below.
- Nothing has to be installed by hand except Ubuntu itself (guide 01).

---

**How to read the versions.** `mariadb:11` means "the newest MariaDB 11". When
the menu updates a program (it downloads the newest image for that version), it
stays within that version, so the documents linked here keep matching.

## Necessary: on the garden computer (Ubuntu)

| Program | Version | Why the garden needs it | Official documents |
|---|---|---|---|
| **Ubuntu** | 24.04 LTS | The operating system the garden runs on. "LTS" means free safety updates for years | <https://ubuntu.com/server/docs> |
| **Raspberry Pi OS** (instead of Ubuntu, on a Pi) | 64-bit | The Pi's own system, for a Raspberry Pi 5 | <https://www.raspberrypi.com/documentation/computers/os.html> |
| **apt** updates, `git`, `curl`, `openssl`, `rsync`, `unzip`, `ca-certificates` | from Ubuntu's store | Basic tools: updates, the change history, downloads, making strong random passwords, copying, unzipping the add-ons | <https://ubuntu.com/server/docs/package-management> |
| **OpenSSH server** | from Ubuntu's store | Lets your Windows PC work on the garden computer, encrypted (guide 20) | <https://www.openssh.com/manual.html> |
| **ufw** (firewall) | from Ubuntu's store | Blocks everything coming in, except what the garden allows | <https://ubuntu.com/server/docs/how-to/security/firewalls/> |
| **Docker Engine** | Docker's own Ubuntu package | Runs each program in its own small box (guide 02a) | <https://docs.docker.com/engine/install/ubuntu/> |
| **cron** (already in Ubuntu) | | Runs the nightly backup at 2:30 am | <https://manpages.ubuntu.com/manpages/noble/man5/crontab.5.html> |

## Necessary: the garden's own boxes (Docker images)

| Program | Version | Why | Official documents |
|---|---|---|---|
| **MariaDB** (the database; speaks the same language as **MySQL**) | `mariadb:11` | Holds your plants, comments and logins. MariaDB was started by MySQL's original makers and works the same way | <https://mariadb.com/kb/en/documentation/> and the image: <https://hub.docker.com/_/mariadb> |
| **Your website** (built on your computer from `rust/`) | Rust 1.95, on Debian 12 "bookworm" | The garden website itself. Built from its code here, not downloaded | <https://www.rust-lang.org/learn>, <https://hub.docker.com/_/rust>, <https://hub.docker.com/_/debian> |
| **The toolbox** (built here) | Alpine Linux 3.20 | Backups, restores and passwords, the same on every computer | <https://alpinelinux.org/about/>, <https://hub.docker.com/_/alpine> |

## Optional parts (switched on in the menu)

| Program | Version | Why | Official documents |
|---|---|---|---|
| **Garden tools** (`/beds`, built here) | Python 3.12 | Bed planner, checkup, diagnose, tag scan, spend list, wiki queue | <https://docs.python.org/3.12/>, <https://hub.docker.com/_/python> |
| **Wiki.js** | 2 | Your garden encyclopedia | <https://docs.requarks.io> |
| The wiki's one-time helper (built here) | Python 3.12, Alpine | Sets up the wiki and adds its pages, then is thrown away | <https://hub.docker.com/_/python> |
| **n8n** | the maker's current image | "When this happens, do that" automations | <https://docs.n8n.io> |
| **LiteLLM** | `main-stable` | One private address for AI, with a monthly spending limit | <https://docs.litellm.ai> |
| **Adminer** | latest | A one-page database viewer, only while you use it | <https://www.adminer.org> |

## Going online: the front door

| Program | Version | Why | Official documents |
|---|---|---|---|
| **Caddy** | Caddy's own Ubuntu package (version 2) | The front door: takes visitors on ports 80 and 443, gets the padlock (below) | <https://caddyserver.com/docs/> |
| **Let's Encrypt** (a service, nothing installed) | | The free, nonprofit authority that issues the padlock certificate | <https://letsencrypt.org/how-it-works/> |
| **Authelia** (your sign-in page) | `4.39` | The login page in front of the garden tools | <https://www.authelia.com/integration/proxies/caddy/> |
| **No-IP DUC** (the updater) | from noip.com | Keeps your web name pointing at your home | <https://www.noip.com/support/knowledgebase/install-linux-3-x-dynamic-update-client-duc> |
| **DuckDNS** (a service; one cron line) | | Free extra names, for subdomains | <https://www.duckdns.org/install.jsp> |
| **Avahi** (already in Ubuntu) | | Lets home devices find `garden-lab.local` | <https://avahi.org> |

### How Let's Encrypt turns `http` into `https`
1. A web address starting with `https://` needs a **certificate**: a small file
   that proves "this really is junsgarden.zapto.org". Browsers show the padlock
   when it is valid.
2. **Let's Encrypt** gives these certificates away, free, to anyone who can
   prove the name points at their computer.
3. **Caddy asks for one by itself** the first time someone visits your name.
   Let's Encrypt checks by connecting to your name (that is why ports 80 and
   443 must reach the garden computer). When it answers correctly, the
   certificate arrives, usually within a minute.
4. From then on, Caddy serves your site over `https`, and sends anyone who
   types `http://` to the `https://` address.
5. Certificates last 90 days. **Caddy renews them by itself** well before that.
   You never do anything.

Caddy's page about this: <https://caddyserver.com/docs/automatic-https>.
In practice mode (`.test` names) Caddy makes its own practice certificate
instead, which is why the browser warns there.

## Backups

| Program | Version | Why | Official documents |
|---|---|---|---|
| **cifs-utils**, **smbclient** | from Ubuntu's store | Connect your NAS (SMB 3.1.1) and find it on the network | <https://wiki.samba.org/index.php/Mounting_samba_shares_from_a_unix_client> |
| **rclone** | from Ubuntu's store | The weekly copy to Google Drive | <https://rclone.org/drive/> |

## On your Windows PC (only if you use it)

| Program | Why | Official documents |
|---|---|---|
| **WSL** (Windows' own Ubuntu, by Microsoft) | Runs the garden on this PC | <https://learn.microsoft.com/windows/wsl/about> |
| **PowerShell** (built into Windows) | The Windows menu and guided installer are PowerShell scripts | <https://learn.microsoft.com/powershell/> |
| **OpenSSH client** (built into Windows 10 and 11) | SSH and copying files to the garden computer | <https://learn.microsoft.com/windows-server/administration/openssh/openssh_overview> |
| **Rufus** | Makes the Ubuntu USB stick | <https://rufus.ie> |
| **Remote Desktop Connection** (built into Windows) | Shows the garden computer's screen (guide 20) | <https://learn.microsoft.com/windows-server/remote/remote-desktop-services/clients/remote-desktop-clients> |
| **WinSCP** (optional) | Copy files between the Linux computer and Windows with drag and drop, over SSH (guide 20, step by step) | Download: <https://winscp.net/eng/download.php>, guide: <https://winscp.net/eng/docs/guide_connect> |

## On your phone or tablet (optional)

| Program | Why | Official pages |
|---|---|---|
| **Termius** (free plan) | A Linux terminal on your phone: SSH to the garden computer, run the menu (guide 20) | <https://termius.com>; iPhone: <https://apps.apple.com/app/termius-ssh-shell-console-terminal/id549039908>; Android: <https://play.google.com/store/apps/details?id=com.server.auditor.ssh.client> |
| **Windows App** (by Microsoft, free) | See and use the garden computer's whole desktop from the phone, over Remote Desktop (guide 20) | <https://learn.microsoft.com/windows-app/get-started-connect-devices-desktops-apps>; iPhone: <https://apps.apple.com/us/app/windows-app-mobile/id714464092>; Android: <https://play.google.com/store/apps/details?id=com.microsoft.rdc.androidx> |

## Away from home (optional)

| Program | Why | Official pages |
|---|---|---|
| **Tailscale** (free for personal use) | A private network of only your devices, so SSH, WinSCP and Remote Desktop work from anywhere without opening any port in the router (guide 20) | <https://tailscale.com/download>, install: <https://tailscale.com/kb/1017/install>, with the firewall: <https://tailscale.com/kb/1077/secure-server-ubuntu>, price: <https://tailscale.com/pricing> |

## Your accounts and keys (services, nothing installed)

| Service | Why | Official page |
|---|---|---|
| **Google AI Studio** (Gemini) | The AI boxes (your own free key) | <https://ai.google.dev/gemini-api/docs> |
| **OpenRouter** | A backup when Gemini is busy (your own key) | <https://openrouter.ai/docs> |
| **Perenual** | Plant facts (your own key) | <https://perenual.com/docs/api> |
| **Gmail app password** | Sends your password-reset emails | <https://support.google.com/accounts/answer/185833> |
| **Google Cloud client ID** (for rclone) | Your own sign-in for the Drive backups | <https://rclone.org/drive/#making-your-own-client-id> |

## Extras for later projects (only if you build them)

| Program | Version | Why | Official documents |
|---|---|---|---|
| **Home Assistant** | `stable` | Smart-home dashboard (projects) | <https://www.home-assistant.io/installation/> |
| **Mosquitto** | 2 | Messages between sensors and Home Assistant | <https://mosquitto.org/documentation/> |
| **ESPHome** | current | Programs small sensor boards | <https://esphome.io> |
| **Uptime Kuma** | 1 | A status page that watches your site | <https://github.com/louislam/uptime-kuma/wiki> |
| **Ollama** (on a desktop with a graphics card) | current | Local AI (guide 15) | <https://ollama.com> |
| **Android Studio** | current | Building the phone app (guide 17) | <https://developer.android.com/studio> |
