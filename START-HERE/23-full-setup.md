# 23. The full setup: everything, the way the garden runs today

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- The EASY install gives you the garden at home. This guide is the other route: everything, in order, the way the garden runs today. Each part can be done on a different day.
- Choose your level first: Easy (beer), Recommended (wine) or Full (champagne). Every part is marked with its level.
- Part 1: install with menu `1`. Full level: choose `2` (CUSTOM): garden tools yes, wiki yes (with the wiki download), photos now. Easy and Recommended: choose `1` (EASY).
- Parts 2 to 5 come at the end of the install (EASY asks 2 to 4): your password, your AI keys, email for password links, and a reserved address in the router.
- Parts 6 to 8, backups in three places: your NAS and more network places (menu `6`, then `4` and `13`), Google Drive (menu `6`, then `5`), and a weekly copy on your Windows PC (Windows menu `M`, then `13`).
- Part 9: your own free No-IP account and web name (guide 04, Part A), a check for CGNAT, and a practice run.
- Part 10: online with HTTPS and your sign-in page: menu `M`, `1`, choice `3`, then ports 80 and 443 in the router.
- Part 11: subdomains with DuckDNS (menu `M`, then `13`; Windows: More, `14`), for example your wiki.
- Part 12: friends and guests (menu `U`).
- Part 13: the phone and remote tools: Termius, WinSCP, and Tailscale if you want it (guide 20).
- Part 14: the Android app on your phone.
- Part 15: the final check: the checklist (menu `5`) ends with 0 fail, and menu `A` lists every address.

---

## Two routes: EASY, or this one

**EASY** (install choice `1`) puts the garden on one computer for your home,
with the garden tools, and asks only for your name, your email and your
passwords. Everything else waits until you want it.

**This guide is the other route.** It takes you from a fresh install to the
complete garden, the way it runs today: the website, the garden tools, the wiki
and the photos; your password; email for "forgot password" links; a fixed
address in the router; your AI keys; backups in three places; your own web name
with HTTPS and your sign-in page; subdomains; friends; the phone and remote
tools; and the Android app.

**Every part can be done on a different day.** Do them in order: each part
needs the ones before it (the front door in Part 10 needs your web name from
Part 9, and the subdomains in Part 11 need the front door). Stop after any
part: the menu remembers where you are, and nothing is lost.
[Guide 00b](00b-before-you-start.md) lists the same things with their times,
and [guide 10](10-the-plan.md) is a slower plan with checkpoints. Each part
below links to the guide with the full details.

## Choose your level

This table is only here to help you choose. You can start small and add more
any time, one part at a time, and every part has its own guide. The numbers
are the chance of a frustrating moment on the way (a step that needs a second
try, or a page that looks different from the guide), not a test of you. When
one comes, it is normal: stop, take a break, and the menu carries on where you were.

| Level | What it includes | Parts below | Time in all | Chance of a frustrating moment |
|---|---|---|---|---|
| **Easy (beer)** | The EASY install only: the garden at home with the garden tools, your plants and photos, and your password | 1 (with EASY) and 2 | about 1 to 2 hours, mostly waiting | 3 / 10 |
| **Recommended (wine)** | Easy, then in order: email for password links, a fixed address (reserve it), the free AI key, backups to a second place (NAS and/or Google Drive, and the Windows weekly copy), the phone and PC at home (Termius, WinSCP), the checklist | 1 to 8, 13 (without Tailscale), 15 | about 2 to 4 hours, over several days | 5 / 10 |
| **Full (champagne)** | Everything: Recommended, plus the wiki, going online with your own web name (No-IP, router, HTTPS, sign-in page), subdomains, friends and guests, the Android app, Tailscale for away from home | all, 1 to 15 | about 3 to 5 hours, over several days | 8 / 10 |

**Recommended (wine) is the one to aim for.** It gives you a garden that is
backed up in more than one place, sends password emails, uses AI, and is easy
to look after from your phone and PC at home, without opening your home to the
internet. Full (champagne) can come later, part by part, whenever you feel like it.

*How the times are added up:* Easy is guide 00b's time for downloading and
installing with EASY ("A realistic plan", sitting 2). Each level above it adds
the **Time** lines of its parts below (they come from guide 00b and from the
install itself). Not counted:
putting Ubuntu on the laptop (2 to 3 hours, guide 01), the parts the guides give
no time for (the Windows weekly copy, friends, Tailscale, the final check), and
building the app yourself (about 3 hours, guide 17).

## Before Part 1

- **A separate garden computer** that can stay on: an Ubuntu laptop (guide
  [01](01-ubuntu-laptop.md): 2 to 3 hours the first time) or a Raspberry Pi.
  The garden inside Windows' own Ubuntu (WSL) is for using on that PC only: it
  cannot go on the internet ([guide 00b](00b-before-you-start.md), "On a Windows PC").
- **The three downloads** from the Google Drive folder:
  `junsgarden-for-jun.zip` (unzip this one), `junsgarden-photos-addon.zip` and
  `junsgarden-wiki-addon.zip` (leave these two zipped). The full setup needs
  all three. Where to put them: `00-READ-ME-FIRST.html` (one folder up), step 1.
- **A password manager.** This route makes many passwords and keys. Each one
  goes in it first. Guide 00b, "Write these down", lists them.

## Part 1. Install the garden (beer)

**What you get:** the website with all your plants and photos and the garden
tools (`/beds`), running at home; on the Full level also the wiki with all its
pages. The automatic backup schedule and the desktop icon come with it.

**Easy or Recommended level?** Choose `1` (EASY) at "How do you want to
install?". EASY skips the questions about the parts, the photos and the wiki,
and ends with your password, your AI keys and email (Parts 2 to 4). The table
below is for the **Full** level, with `2` (CUSTOM). (Started with EASY and want
the wiki later? Menu `9`, then `2`, then `A`, with the wiki download in place.)

**Time:** 30 to 90 minutes, mostly waiting. The longest step, building the
website, takes 15 to 45 minutes (up to an hour on a very old laptop or a Pi)
(guide 00b). The wiki adds about 30 minutes while its pages go in (the install
says so on screen). Each step shows its usual time before it starts.

**Before you start:** put `junsgarden-wiki-addon.zip` (still zipped) in your
Home folder, Downloads, the Desktop or the garden folder. Without it the wiki
starts empty, and the install tells you so. (Later, with the zip in place,
`bash garden.sh wiki` adds the pages.)

**Start it on the garden computer:** open the `junsgarden-for-jun` folder,
right-click `Start-Here-Ubuntu.sh` and choose **Run as a Program** (or type
`bash garden.sh` in a terminal there). Type `1` (INSTALL) and press Enter.

![The install asks EASY or CUSTOM: for the full setup, type 2](images/screen-03-easy-or-custom.png)

**The questions, and what to answer for the full setup** (in the order they come):

| The window asks | Type |
|---|---|
| Where should your garden run? `Type 1, 2 or 0` | `1` (THIS computer) |
| How do you want to install? `Type 1, 2 or 0` | `2` (CUSTOM) for the Full level (`1`, EASY, for Easy and Recommended) |
| Your web name: `Type 1 or 2` | `1`: keep the name it has. You add it to your own No-IP account in Part 9. (A new name: `2`, or later menu `M`, then `16`; guide 04.) |
| `Start at which step? Type a number, or press Enter for step 1 (0 = cancel)` | `Enter` |
| Your Ubuntu password (for `sudo`) | your Ubuntu password. Nothing shows while you type |
| `Is this the right place for it? (Y/n)` | `Enter` (yes) |
| `Your first name for the login (letters only) [Jun]` | `Enter` (or your first name) |
| `Your email address (password links go here)` | your Gmail address |
| `Database password (Enter = make a strong one for me)` | `Enter`. A strong one is made for you; you never type it |
| `Install the garden tools? (Y/n)` | `y` |
| `Install the wiki? (y/N)` | `y` |
| `Install n8n now? (y/N)` | your choice. `n` is fine: menu `9` adds it any time |
| `Load the photos now (a few minutes)? Or later: menu 6, then 11 (Y/n)` | `y` |
| `Wiki admin password (nothing shows as you type)`, then `Type it again` | a new password for your wiki, 10 characters or more. It can differ from your garden password |

Capital letters show what `Enter` means: `(Y/n)` is yes, `(y/N)` is no.

**From a Windows PC instead** (the guided installer, `Start-Here-Windows.cmd`):
on page 2 choose **Another computer at home** and type its IP address (or click
**Find it for me**) and your user name there. Page 3 asks your name, your email and the web name (keep it).
On page 4 tick **Garden tools** and **Wiki** (n8n is your choice). Then **Start
the install**: in the black window choose `2` (CUSTOM). Your name, your email,
the web name and the parts are already answered, so the black window asks the
rest of the questions above. The photos and wiki zips go along by themselves
when they are in the garden folder, on the Desktop or in Downloads.

Changed your mind about a part later? Menu `9` (Parts) switches each one on or off:

![Menu 9, Parts: each part ON or off, then A to apply](images/screen-09-parts.png)

**At the end, keep the window open.** CUSTOM finishes with "Three short
follow-ups (each can also be done later)". They are Parts 2 to 5 below, in the
same order.

**You know it worked when:** the window says **ALL DONE** with
`Your garden: http://localhost:8030` and `Your wiki: http://localhost:3000`, and
menu `4` shows your plants with photos.

**Full details:** `00-READ-ME-FIRST.html` (one folder up),
[00b, "Easy or Custom?"](00b-before-you-start.md#easy-or-custom),
[19, menu 1 (INSTALL)](19-every-menu-option.md#main-menu).

## Part 2. Your password (beer)

**What you get:** one password for the website login (to edit your garden) and
for your sign-in page.

**Time:** 1 minute (guide 00b).

**Where:** the first follow-up, `1. Set your password now (needed to edit your garden)? (Y/n)`:
type `y`. (EASY asks for it straight away, under "Last thing: your password".)
Later, or to change it: menu `U`, then `5`.

**The questions:** `Login name [Jun]`: press `Enter`. Then
`New password for Jun (10+ characters, nothing shows as you type)` and
`Type it again`. Put it in your password manager.

**You know it worked when:** you can log in at `http://localhost:8030/login.php`
on the garden computer (from Windows: Windows menu `4`, then add `/login.php`).

**Full details:** [03, Part B](03-run-your-garden.md#part-b-your-password).

## Part 3. Your AI keys (wine)

**What you get:** the AI boxes on the website and in the garden tools (tag
reading, identify, diagnose) work with your own free Gemini key, with
OpenRouter as a backup when Gemini is busy.

**Time:** Gemini key 2 minutes, OpenRouter key 5 minutes, Perenual key 5
minutes (guide 00b).

**Where:** right after the password the install asks
`Add your Google Gemini key (AI Studio) now? (Y/n)`: type `y`. It shows the
steps (open <https://aistudio.google.com/apikey>, **Create API key**, copy it).
Paste it at `Paste the key now (or just press Enter to skip)` (hidden) and press
`Enter`. It tests the key. Then `Add your OpenRouter key (a backup when Gemini is busy) now? (y/N)`:
type `y` for the full setup; the steps are on screen (<https://openrouter.ai/keys>).

Later, or to replace one: menu `K` (API keys). `1` Gemini, `2` OpenRouter,
`3` Perenual (plant facts in the plant admin), `5` the wiki's key (Part 11).
Each one then has `1` Paste a new key, `2` Test the key, `3` How to get a key.

**You know it worked when:** after the paste it says
`Works: the service accepted the key.` (Code 429 means the key is right, but
today's free limit is used up.)

**Full details:** [16, section 4](16-ai-studio.md#4-a-free-api-key-for-the-gardens-ai-boxes),
[19, API keys (K)](19-every-menu-option.md#api-keys-k).

## Part 4. Email for 'forgot password' links (wine)

**What you get:** "forgot password" emails for you and for the friends you add
in Part 12, sent through your Gmail.

**Time:** 5 minutes (10 if 2-Step Verification is not on yet) (guide 00b).

**Where:** the second follow-up, `2. Set up email (Gmail) now, for 'forgot password' links? (y/N)`:
type `y`. (`Enter` means no, and it asks once more:
`Are you sure? ... Skip email for now? (Y/n)`. Type `n` to set it up after all.)
(EASY asks the same question, without the `2.`.) Later: menu `M`, then `3`,
or menu `8`, then `3`.

![The email step: two Google pages to open, then your Gmail address and the 16 letters](images/screen-23-email.png)

1. Turn on 2-Step Verification: <https://myaccount.google.com/signinoptions/twosv>.
2. Open <https://myaccount.google.com/apppasswords>, type the name `garden`,
   click **Create**. Google shows 16 letters once: copy them now.
3. `Gmail address [you@gmail.com]`: `Enter` if it is right.
4. `Gmail app password (16 letters, hidden)`: paste, then `Enter`.
5. `Send a test email to ... now? (Y/n)`: `y`.

**You know it worked when:** the test email arrives (check spam the first time).

**Full details:** [03, Part C](03-run-your-garden.md#part-c-email-for-forgot-password-links-optional).

## Part 5. A reserved address in the router (wine)

**What you get:** the garden computer keeps the same address on your home
network, so SSH, the copies in Parts 6 to 8 and, later, the router's two open
ports always find it.

**Time:** guide 00b gives 15 to 30 minutes for the router work in all: this
reservation, plus the two ports in Part 10.

**Where:** the third follow-up, `3. Reserve this laptop's address in your router now? (y/N)`:
type `y`. EASY does not ask this one: use menu `M`, then `2` (Network), which
is also the way to do it later.

1. It shows the garden computer's **IP address**, **MAC address** and the
   **router address**, with the steps.
2. On the router's page (or its phone app), find **DHCP reservation** (or
   Address reservation, Static IP, Fixed IP) and add that MAC with that IP. Save.
3. Back in the window: `Have you reserved 192.168.1.50 for this laptop in the router? (y/N)`
   (with your own address): `y`, once it is saved.
4. Restart the garden computer once and choose menu `M`, `2` again: the IP must be the same.

**You know it worked when:** the checklist (menu `5`) says
`Address ... matches the one reserved in the router`.

**Full details:** [14, Step 2](14-reserve-your-ip.md#step-2-reserve-it-in-the-router).

## Part 6. Backups 1: your NAS, and more network places (wine)

**What you get:** a copy of every nightly backup on your NAS at home (the newest
14 are kept), and, if you like, on up to two more network places (a second NAS,
or a folder a Windows PC shares). Also, if you say yes, a weekly copy of the
whole garden folder on the NAS.

**Time:** 10 to 20 minutes for the NAS (guide 00b). Each extra place asks the
same questions again.

**First, on the NAS:** a shared folder for backups (for example `backups`), and
a NAS user and password that may write to it. Reserve the NAS's address in the
router too (Part 5, the same way).

**Menu `6`, then `4`** (Connect your NAS):

| The window asks | Type |
|---|---|
| `Search your home network for the NAS? (Y/n)` | `Enter`. It lists what it finds |
| `Number of your NAS (or type its address) [1]` | its number in the list |
| `NAS user name`, then the NAS password (hidden) | the NAS user that may write to the folder |
| `Number of the folder for backups (or type its name) [1]` | the number of your backups folder |
| `Folder inside it for the garden [junsgarden]` | `Enter` |
| `Copy the newest backup there now? (Y/n)` | `Enter` (yes) |
| `Also keep a copy of the WHOLE garden folder on the NAS (weekly)? (Y/n)` | `y`. It includes your settings file `.env`, so keep the NAS private |

**More places: menu `6`, then `13`** (Add another network place). The same
questions; the first one you add is place 2, the next place 3. In this example
place 2 is a Windows PC that shares a folder named `GardenCopies`:

![Adding place 2: a PC's shared folder at 192.168.1.30 named GardenCopies](images/screen-21-nas-add.png)

**You know it worked when:** it says `Connected:` and `NAS copy: ...`, the top
of menu `6` shows your NAS copies, and `logs/offsite.log` has a line with `NAS OK`
(`NAS2 OK` for place 2).

**Full details:** [13, Connect your NAS](13-backups-and-folders.md#connect-your-nas-menu-6-then-4)
and [More than one network place](13-backups-and-folders.md#more-than-one-network-place-menu-6-then-13).

## Part 7. Backups 2: Google Drive (wine)

**What you get:** once a week the newest backup goes to a Google Drive folder
named `JunsGarden-backups`, away from home (the newest 4 are kept, about 3.2 GB).

**Time:** 15 minutes (guide 00b). Your own Google "client ID" is part of that:
about 10 minutes, once (the menu says so).

**Menu `6`, then `5`** (Connect Google Drive):

1. The window shows 7 steps for your own Google client ID, at
   <https://console.developers.google.com/>: a project, the **Google Drive API**,
   the consent screen, the scope `https://www.googleapis.com/auth/drive.file`,
   yourself as a user, an **OAuth client** of type **Desktop app**, then
   **PUBLISH APP** (without that, Google asks you to sign in again every week).
2. `Your client ID (ends in .apps.googleusercontent.com; ...)`: paste it.
3. `Your client secret`: paste it (hidden).
4. Google's sign-in page opens in the browser: choose your account, click
   **Allow**. (No browser, because you work over SSH? rclone says what to do:
   run `rclone authorize "drive"` on a computer that has a browser, and paste
   back what it prints.)
5. `Copy the newest backup to Google Drive now? (about 800 MB) (Y/n)`: `Enter` (yes).

**You know it worked when:** Google Drive has a folder `JunsGarden-backups` with
the backup in it, and `logs/offsite.log` has a line with `CLOUD OK`.

**Full details:** [13, Connect Google Drive](13-backups-and-folders.md#connect-google-drive-menu-6-then-5).

## Part 8. Backups 3: the weekly copy on your Windows PC (wine)

**What you get:** every Sunday at 11:00 your Windows PC copies the newest
backup from the garden computer into a folder you pick (the newest 8 are kept).
If it missed a Sunday, it runs as soon as the PC is on.

**Time:** the guides give no time for this one. It is three questions.

**Windows menu** (`Garden-Menu-Windows.cmd`), **`M`, then `13`**. (This is the
Windows menu. In the Ubuntu menu, `M`, `13` is Subdomains.)

1. Where the copies go: `1` pick a folder in a window (this PC, a USB drive,
   Google Drive for desktop, or a NAS under **Network**), `2` type it (for
   example `C:\Backups\weekly`, or `\\192.168.1.20\backups`), `3` keep it as it is.
2. `Add the weekly copy now? (Y/n)`: `y`.
3. `Make the first copy right now? (Y/n)`: `y`.

A NAS needs one thing first: open it once in File Explorer, sign in, and tick
**Remember my credentials**.

![Windows menu, More, 13: the weekly copy, sent to C:\Backups\weekly](images/windows-13-backup-copy.png)

**You know it worked when:** a `junsgarden-backup_...tar` file appears in the
`junsgarden-backups` folder inside the place you chose, and `logs\backup-copy.log`
in the garden folder on the PC says `OK`.

**Full details:** [13, A copy on your Windows PC](13-backups-and-folders.md#a-copy-on-your-windows-pc-automatic-if-you-want-it),
and the whole 3-2-1 picture: [13, 3-2-1](13-backups-and-folders.md#3-2-1-your-nas-and-google-drive).

## Part 9. Your web name at No-IP, and two checks (champagne)

**What you get:** your own free No-IP account with your web name in it
(`junsgarden.zapto.org`, or the name you chose) and its **DDNS key**, which the
front door needs in Part 10.

**Time:** about 10 minutes, plus confirming the email No-IP sends (guide 00b).

1. Make a free account at <https://www.noip.com> with your own email, and
   confirm the email No-IP sends.
2. **Dynamic DNS, No-IP Hostnames, Create Hostname.** Name `junsgarden`,
   domain `zapto.org` (or your own name). `junsgarden.zapto.org` was released so
   you can claim it: [guide 04b](04b-take-over-junsgarden-zapto.md).
3. **Dynamic DNS, DDNS Keys, Add DDNS Key.** Put its **username** and
   **password** in your password manager.

Then two checks before going online:

- **CGNAT** (guide 04, Part B): on the router's page, find its **WAN** (or
  Internet) address. On the garden computer run `curl -s https://api.ipify.org; echo`.
  The same address: fine. Different, starting with `10.`, `100.64.` to
  `100.127.` or `192.168.`: ask your internet company for a "public IP address".
- **Practise** (recommended, guide 04, Part C): at the garden computer's own
  screen, menu `M`, `1`, choice `2`, name `garden.test`. Open
  `https://garden.test` in Firefox there and try `/beds` and signing in.

**You know it worked when:** your web name is listed under No-IP Hostnames, the
DDNS key is in your password manager, and the CGNAT check shows the same address twice.

**Every 30 days** No-IP emails you a link: click **Confirm**, or the free name stops working.

**Full details:** [04, Part A](04-go-public-no-ip-caddy.md#part-a-your-own-no-ip-account-and-the-name-once),
[Part B](04-go-public-no-ip-caddy.md#part-b-check-you-are-not-behind-cgnat),
[Part C](04-go-public-no-ip-caddy.md#part-c-practise-first-recommended).

## Part 10. Online: HTTPS and your sign-in page (champagne)

**What you get:** `https://junsgarden.zapto.org` for everyone, with the padlock;
your sign-in page (Authelia) at `/auth` in front of the garden tools; the No-IP
updater that keeps the name pointing home. Only ports 80 and 443 are opened.

**Time:** the front door step 15 to 30 minutes, the router 15 to 30 minutes
(guide 00b). Guide 04 as a whole: about 1 hour.

**Menu `M`, then `1`** (The front door). The Windows menu has the same number: More, `1`.

![The front door's three choices: for the full setup, 3](images/screen-14-front-door.png)

| The window asks | Type |
|---|---|
| `Type 1, 2, 3 or 0` | `3` (THE REAL WEB NAME) |
| `Web name [junsgarden.zapto.org]` | `Enter` (or your own name) |
| `Is junsgarden.zapto.org in your own No-IP account? (Y/n)` | `Enter` (yes; you did it in Part 9) |
| `Wiki web name (empty = private only)` | `Enter` (empty). The wiki gets its own address in Part 11. Typing one here and adding it again in Part 11 would give it twice |
| Your Ubuntu password (for `sudo`) | your Ubuntu password |
| `DDNS Key username:`, then `DDNS Key password (hidden):` | the DDNS key from Part 9 |
| `New password for Jun (10+ characters, ...)`, then `Type it again` | your garden password from Part 2. One password for the website and the sign-in page |

If Part 5 is not finished yet, it also asks whether you reserved the address.

**Then the router** (only you can do this): forward **TCP 80** and **TCP 443**
to the garden computer's reserved address. Nothing else: never 22, 8030, 3306
or any other port.

**Test** from a phone with **Wi-Fi off**: open `https://junsgarden.zapto.org`.
The first visit can take 30 seconds while Caddy gets the certificate.

**You know it worked when:** the phone shows your garden with the padlock;
`https://junsgarden.zapto.org/beds` sends you to your sign-in page and opens
after you sign in; and the checklist (menu `5`), section 6, says
`No-IP updater is running` and `junsgarden.zapto.org points at this home`.

**Full details:** [04, Part D](04-go-public-no-ip-caddy.md#part-d-the-real-thing),
[14, Step 3](14-reserve-your-ip.md#step-3-open-exactly-two-ports-only-when-you-put-the-site-online),
[07 Your sign-in page](07-sso-gate.md).

## Part 11. Subdomains with DuckDNS (champagne)

**What you get:** a free DuckDNS name (for example `junsgarden.duckdns.org`)
whose subdomains point home: `wiki.junsgarden.duckdns.org` for your wiki, and
`n8n.junsgarden.duckdns.org` if you switched n8n on. Each one gets its own
padlock, through the same two router ports. Your garden stays at `junsgarden.zapto.org`.

**Time:** 5 minutes for DuckDNS (guide 00b).

**Menu `M`, then `13`** (Subdomains). On the Windows menu: More, `14`.

1. Choose `1` (Set up DuckDNS). Sign in at <https://www.duckdns.org>, type a
   name under "domains" and click **add domain**, then copy your **token** at
   the top of the page.
   `Your DuckDNS name (only the first part, e.g. junsgarden)`: type it.
   `Paste your DuckDNS token` (hidden): paste it. It says
   `DuckDNS says OK: junsgarden.duckdns.org points home now.`
2. Choose `2` (Add a subdomain).
   `Subdomain name (letters, numbers, -) [wiki]`: `Enter`.
   `Port of the program on this computer [3000]`: `Enter`.
   `Put wiki behind your sign-in page (recommended unless it has its own login)? (y/N)`:
   `Enter` (no: the wiki has its own login). It writes the settings, reloads
   Caddy and shows your addresses.
3. n8n switched on? Choose `2` again: name `n8n`, port `5678`, and `Enter` (yes)
   to put it behind your sign-in page.
4. `0` goes back.

The wiki goes online only once its own setup has finished (its admin exists).
If it is not finished, the menu leaves it out and says so: run `bash garden.sh wiki`, then step 2 again.

**The wiki's key for the garden tools** (their wiki queue publishes drafts to
your wiki): menu `K`, then `5`, then `1`. Open your wiki (the new address, or
`http://localhost:3000` on the garden computer) and sign in as its admin.
**Administration > API Access**: switch **Enable API** on, **Apply**, then
**New API Key**: name `garden-tools`, expiration 1 year, **Full Access**. Copy
the key (it is shown once) and paste it.

**You know it worked when:** `https://wiki.junsgarden.duckdns.org` (with your
DuckDNS name) opens your wiki with the padlock (give the first visit a minute),
menu `A` lists it, and the wiki key's test says `Works`.

**Full details:** [04, Advanced: subdomains with DuckDNS](04-go-public-no-ip-caddy.md#advanced-optional-subdomains-with-duckdns),
[19, Subdomains](19-every-menu-option.md#subdomains-more-13-windows-more-14).

## Part 12. Friends and guests (champagne)

**What you get:** the people you choose can sign in. A person who may **edit**
gets a website login, the sign-in page and the garden tools. A **guest** gets a
sign-in page account only and sees nothing private.

**Time:** the guides give no time. Each person is a name, an email and a first password.

**Menu `U`** (Users and access):

- `2` Add a person who may EDIT: `Their first name (letters only)`,
  `Their email address`, then a first password for them, twice. Tell them the
  password in person. With email set up (Part 4) they can choose their own with
  "Forgot password?" on the login page.
- `3` Add a GUEST: `Guest's first name (letters only)`,
  `Guest's email (optional)`, then their password. (Guests live on the sign-in
  page, so they need Part 10 first.)
- `1` shows who can sign in. `4` removes a person (every login they have).
  `8` LOCK DOWN leaves only you.

With subdomains, a phone code (two-factor) on the sign-in page is a good idea:
guide 07, "Make it stronger".

**You know it worked when:** menu `U`, `1` lists them, and they can sign in.

**Full details:** [07, Who can sign in](07-sso-gate.md#who-can-sign-in-menu-u-users-and-access),
[19, Users and access (U)](19-every-menu-option.md#users-and-access-u).

## Part 13. The phone and remote tools (wine)

**What you get:** the garden computer's terminal on your phone (Termius), files
copied between Windows and the garden computer by dragging (WinSCP), two
desktop shortcuts on your PC, and, if you want it, a safe way in from away from
home (Tailscale).

**Time:** 10 to 20 minutes for SSH keys and Remote Desktop (guide 00b).

Every one of these asks the same four things: the garden computer's **IP
address** (the one you reserved in Part 5), the **port** `22`, your Ubuntu
**user name**, and your Ubuntu **password**.

- **Termius** on the phone (iPhone or Android, free): tap **+**, add a host
  with those four things, save, tap it, and say yes to the fingerprint the
  first time. Then `cd ~/junsgarden-for-jun` and `bash garden.sh` opens the
  menu. It works on your home Wi-Fi.
- **WinSCP** on Windows (<https://winscp.net/eng/download.php>): File
  protocol `SFTP`, then the four things. **Save** it as `Garden computer`,
  **Login**, and **Yes** to the key the first time. Your PC is on the left,
  the garden computer on the right.
- **Shortcuts:** Windows menu, More, `18`: **Garden terminal** (SSH) and
  **Garden screen** (Remote Desktop; switch it on in Ubuntu's Settings first, guide 20).
- **Tailscale** (champagne; optional, highly recommended: only your own devices can reach the
  garden computer, from anywhere, with no port opened). Make the account and install the app on
  your phone and PC from <https://tailscale.com/download>; then garden menu `M`, `17` (Windows
  menu `M`, `19`), press `y`, open the link it shows and sign in. Use the garden computer's name
  instead of the IP address. Step by step, with a picture: guide 20, "Away from home".

**Never** forward port 22 or 3389 in the router. Tailscale does that job safely.

**You know it worked when:** Termius on your home Wi-Fi shows the prompt
(`jun@garden-lab:~$`), WinSCP shows `junsgarden-for-jun` on the right, and (with
Tailscale) Termius also connects on mobile data, by name.

**Full details:** [20, a terminal on your phone](20-ssh-and-remote-desktop.md#a-linux-terminal-on-your-phone-ssh),
[WinSCP](20-ssh-and-remote-desktop.md#copy-files-between-linux-and-windows-winscp-step-by-step),
[shortcuts](20-ssh-and-remote-desktop.md#two-shortcuts-on-your-desktop),
[Tailscale](20-ssh-and-remote-desktop.md#away-from-home-optional-highly-recommended-tailscale).

## Part 14. The Android app (champagne)

**What you get:** the Jun's Garden app on your Android phone: your plants, and
the camera tools once you sign in.

**Time:** the app is one download of about 45 MB (the size `garden.sh` notes for
it). Building the app yourself in Android Studio is about 3 hours, split over a
few sessions (guide 17); that is only for changing it.

1. On the phone, open your garden (`https://junsgarden.zapto.org`) and tap
   **Android App** at the top, or open `https://junsgarden.zapto.org/apps/junsgarden.apk`
   (menu `A` lists it as "the phone app download").
2. Android asks before it installs an app that is not from the Play Store.
   Allow it for this one file.
3. Open the app. Sign in with your website name and password, or tap
   **Continue as guest**.

Later, the app finds updates by itself on your website (`site/apps/version.json`).
To change the app: guide 17 and `android-app/README.md`. The key that signed
today's app is not in this package, so the README's "Signing and updates",
choice (b) (your own new key), is the one for your own builds.

**You know it worked when:** the app opens and shows your plants, and after you
sign in, the camera tools open.

**Full details:** [17, Part A](17-android-app-and-ai-coding.md#part-a-android-studio-and-the-app-on-your-phone),
`android-app/README.md` (one folder up).

## Part 15. The final check (wine)

**What you get:** proof that everything works together, and a list of every address.

**Time:** the guides give no time. It is two menu choices and a phone test.

1. **Menu `5`** (the checklist). It checks the programs, the ports, the
   folders, the backups, the network and the web name. PASS is good, WARN means
   fix soon, FAIL means fix first; the fix is written under each line.

   ![The checklist: PASS, WARN or FAIL, with what to do](images/screen-19-checklist.png)

2. **Menu `A`** (My addresses): every address of your garden, and which ones
   need signing in. It is also saved in `MY-ADDRESSES.txt`.
3. **Menu `6`, then `1`** (Back up now): watch the copy go to your NAS. Menu
   `6`, then `6` sends one to Google Drive right away too.
4. **Full level: a phone on mobile data:** your garden, `/beds` (sign in), and your wiki address.

**You know it worked when:** the checklist ends with `RESULT: ... 0 fail`, and
every address from menu `A` opens.

**Full details:** [19, menu 5](19-every-menu-option.md#main-menu), [09 Troubleshooting](09-troubleshooting.md).

## When everything is done

On the Full level you now have the garden the way it runs today. (On the
Recommended level, tick the ones your level includes; the rest can come later.)

- [ ] The website, the garden tools and the wiki, with all your plants, photos and pages
- [ ] Your password, and email for "forgot password" links
- [ ] Your own AI keys
- [ ] Backups in three places: the garden computer, your NAS (and more network
      places), Google Drive, plus the weekly copy on your Windows PC
- [ ] `https://junsgarden.zapto.org` with the padlock and your sign-in page
- [ ] Your wiki (and n8n) on DuckDNS subdomains
- [ ] Friends and guests who can sign in
- [ ] Termius, WinSCP (and Tailscale) to look after it from anywhere
- [ ] The app on your phone
- [ ] The checklist (menu `5`) ends with 0 fail

**Your addresses:** menu `A`, any time (also in `MY-ADDRESSES.txt`).

**Keep it healthy: 15 minutes a week** (from [guide 10](10-the-plan.md#after-that-keep-it-healthy-15-minutes-a-week)):

- [ ] `bash lab/09-health-check.sh` and fix any FAIL.
- [ ] `sudo apt update && sudo apt upgrade -y`, then `sudo reboot` if asked.
- [ ] Copy `backups/` to a USB stick.
- [ ] Look at Uptime Kuma (Project 1), if you built it. Is everything green?

**And now and then:**

- Every 30 days: click **Confirm** in No-IP's email.
- Once a month: check your copies (guide 13, "Check your copies once a month"):
  `tail logs/offsite.log`, each NAS folder, and the Drive folder.

**Next:** [19 Every menu option, explained](19-every-menu-option.md).
