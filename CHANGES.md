# Changes: what is different, and why

This file lists every change made to Jun's copy, so anyone can see what was
changed, why, and what it means for the program and the install. Each entry
matches a commit in the git history (`git log` shows them; `git show COMMIT`
shows the exact lines).

## Part 1. How this copy differs from the old live site

These are deliberate. They make the copy Jun's own and remove everything that
belonged to the old household server.

| What | Changed how | Why | Effect on the program / install |
|---|---|---|---|
| Old PHP pages | Not included. Only the live Rust site and the files it actually serves (`site/`) | The PHP version was replaced by Rust on 2026-10-06; nothing runs it | Smaller, nothing unused. The website looks and works the same |
| `.bak` and test copies of pages and app files | Not included | Clutter, and some old copies held old passwords | None |
| Built-in accounts in the code | Now Jun only (`accounts.rs` fallback, uploader list) | Nobody else should have a built-in way in | Only the people in Jun's settings can log in |
| Logins and sessions in the garden data | Left out of the starter backup | Jun sets his own password; nobody else's login comes along | The install asks Jun for his password |
| Garden tools (`/beds`) | Garden-only version: Paperless, NAS video, MeTube, the household wiki and other household links removed | They belonged to the household server | Same garden pages; they open after Jun's own sign-in page |
| Wiki | Pages, tags and menu copied; the old admin login, API keys, sign-in settings and secrets left out | The wiki becomes Jun's, with his own admin account | The install sets up a fresh wiki and puts all 933 pages in |
| Android app | Dead household code removed (old household sign-in, chat, recipes, tools; all pointed at the household server) | Unused since app 3.0, and it pointed at a server that is not Jun's | Same app, same features; nothing reaches the household server |
| Sign-in for the garden tools | The household sign-in service is replaced by Authelia (open source) on Jun's computer | Jun's own sign-in page, well documented | Set up by the front door step (menu M, 1) |
| AI keys | None of Jason's keys are included | Jun uses his own (free) keys | Jun pastes his own keys during the install or with menu K |
| Server deploy scripts (`docs/rust-port/tools`, `docs/streamlit-port/tools`) | Not included | They were for the old server | History notes in `docs/` say so |
| Web name | `junsgarden.zapto.org` was released from Jason's No-IP account | Jun registers it in his own No-IP account | Until then: front door "home only" (menu M, 1, choice 1) |

## Part 2. Fixes found by testing

Each fix: what was wrong, what changed, and the effect.

| # | Fix | What was wrong | Effect on program / install |
|---|---|---|---|
| 1 | `sudo -v` changed to `sudo true` in the system check | On some setups `sudo -v` asks for a password even when sudo itself does not, so the install waited forever | The install no longer hangs at step 1 |
| 2 | Docker DNS check (install step 6 and 12) | Some networks give containers a DNS address they cannot reach, so the build could not download anything | The install notices, sets Docker to use public DNS (1.1.1.1, 8.8.8.8), and says so |
| 3 | Build and download include every switched-on part | Docker tried to download the garden tools image instead of building it, printing a scary "pull access denied" | Clean output; parts are built, never downloaded from a stranger |
| 4 | Wiki setup includes every switched-on part | The wiki helper could not find the wiki, so the first wiki setup did not run | The wiki pages go in during the install |
| 5 | Sign-in page user list written in one go, and the sign-in page restarted after any user change | A new guest could not sign in until the sign-in page re-read its list | New people and new passwords work straight away |
| 6 | Caddy output and checks | Caddy printed many harmless warnings that looked like errors | Only real problems are shown; settings are tidied and checked first |
| 7 | Ports 80 and 443 checked before Caddy starts | If another program held them, Caddy failed with a confusing message | A clear message saying which program holds the port |
| 8 | LiteLLM port | If port 4000 was taken, LiteLLM could not start | It picks the next free port and says which |
| 9 | Language switch: blank lines | Korean mode printed "bad array subscript" errors on blank lines | Clean Korean and "both" screens |
| 10 | Windows: scp copy folder | A trailing backslash made Windows write the backup to the wrong place | The weekly copy lands in the right folder; the copy is checked |
| 11 | Windows: WSL user check | If Windows' Ubuntu was installed but its user was never made, the install skipped making it | The user is always made before installing |
| 12 | Front door: web name saved again | A line that saves the web name had been lost while editing | Going online remembers the name (found before the final test) |
| 13 | Wrong API key message | Google answers a wrong key with code 400, which was shown as "no clear answer" | Says plainly that the key is wrong |
| 14 | Google Drive download step removed | The garden data now travels inside the package | No Google Drive code or link in the program |

## Part 3. Added at Jason's request (2026-10-13)

| What | Why | Effect on the program / install |
|---|---|---|
| The wiki is optional | Jun may not want a wiki | It already was a part: menu M, Parts, number 2 switches it off (its pages are kept) and on again. Tested both ways |
| "Home only" now says **Not on the internet yet (home Wi-Fi only, no www)** and names the next step | "Home only" could confuse Jun | The choice and its finish screen say plainly that nobody outside the house can reach it, and the next step: sign up at No-IP (guide 04, Part A), DuckDNS optional, then choice 3, which installs the No-IP updater (DUC) |
| **Change the web name** (menu M, 16; Windows More, 17; `bash garden.sh webname`) | Jun may want another name than junsgarden.zapto.org (for example one for his mom) | Swaps the name in the 12 files that use it (share buttons, privacy page, the app's update check, Android source), after cleaning `https://`, `http://`, `www.` and paths off what was typed and checking it is a web name. Keeps the old files in `logs/web-name-change_DATE/`, updates `.env`, Caddy and the sign-in page when online, offers a rebuild, and saves the change in Jun's git history. Works before installing too. On Windows, the PC's own copy is changed as well, so a later install does not bring the old name back. Guide 04 explains it (English and Korean) |
| It is called the **Website name swap**, and the install asks first | Jun should decide before setup: keep junsgarden.zapto.org, or a new name | A fresh install asks once: 1) keep the name (recommended) or 2) a new name now. The Windows guided installer asks on its "About you" page (keep / a new name, with links), and the Windows menu asks in its window; Windows puts the name into the PC's copy before copying it over. The swap shows how to get a free web name (No-IP: one free name, confirmed every 30 days; the paid plan has 3) |
| The read-me and guide 04 explain web names | So Jun knows how to start a new name, without being overwhelmed | Read-me (English and Korean): a short "Your web name" section, the "Not on the internet yet" choice and the DUC. Guide 04: the No-IP plan facts, and that free DuckDNS names can give other things (Home Assistant panels, automations) their own address behind the sign-in page, with far less exposed |
| Google Drive backups ask for Jun's own client ID | rclone's shared one stops working during 2026 | See #22 below |
| **The wiki is off unless Jun chooses it** | Jun may never want it; nobody should wait for a program and data they do not use | Easy mode no longer switches the wiki on (Custom asks, default No; the Windows installer's box starts unticked). Its pages are a separate download, `junsgarden-wiki-addon.zip`. Turning the wiki on finds that zip (garden folder, Home, Downloads, Desktop), checks its fingerprint (`docker/wiki/seed/wiki-pages.sha256`) and adds the pages; without it the wiki is set up empty. The pages are no longer in git (only their fingerprint) |
| **The photos are a separate download** | Keep the first download small; let Jun fetch the 800 MB of photos when his internet is quiet, or again later | Three downloads: the garden (`junsgarden-for-jun.zip`), the photos (`junsgarden-photos-addon.zip`) and the wiki pages. The install loads the photos if the zip is there (fingerprint checked against `docker/tools/photos.sha256`); otherwise the garden works without pictures until menu 6, 11 (or `bash garden.sh photos`) loads them. Photos already there are kept |
| Downloads: Home on Ubuntu, Desktop on Windows; a copy on the NAS | So Jun knows where to keep them, and has a spare | The read-me says where to keep the three zips. The menu looks there (and in Downloads). Menu 6, 12 keeps a copy of the downloads on the NAS. The Windows menu brings the photos and wiki zips along when it installs |
| A timer on every install step | Jun likes to know how long things take, and to stop and carry on later | Before each step: "Usually takes ..."; after it: "This step took N min". The progress file (and the Windows guided installer's list) shows both. Every step can be stopped and resumed (menu 1) |
| Custom mode: photos now or later | A busy network or a short evening | Custom asks "Load the photos now? Or later: menu 6, then 11". Easy loads them when the photos download is there |
| A question for the database password | Jun should know his database has its own password, and may choose it | The settings step asks once: Enter makes a strong one (recommended; never typed again, kept in `.env`, menu 8 renews it), or type your own (16+ letters and numbers, hidden). Jun's everyday password stays his login; with Gmail set up, "forgot password" emails go straight to him |
| **JOURNAL.md** | So Jun has a plain diary of what Jason did and why | A dated journal next to this file, kept up to date with each change |
| **Guide 18: review the code yourself** | Jun, his friends or an AI should be able to check everything | A map of the files, the fingerprint check, git history, the secret scanner (`tools/check-secrets.sh`), and safe ways to ask Gemini or a friend. English and Korean |

## Part 4. Security review (before publishing)

A separate reviewer read every file, all of git's history, the wiki pages and
the private starter backup. **Answer to the key question: someone who reads the
public code, even knowing Jun's name and email, cannot sign in as Jun or anyone
else.** There are no passwords, password hashes or keys anywhere; the login
tables in the backup are empty; every program listens only on the garden
computer except Caddy. These improvements were made from its suggestions:

| # | Change | What could have happened | Effect |
|---|---|---|---|
| S1 | The login cookie is `SameSite=Strict`, and a request that another website started is treated as a visitor (`Sec-Fetch-Site`) | A stranger's link, opened while Jun was logged in, could make the site move a plant or delete a photo (no login was given away) | Only Jun's own clicks on his own site can change anything. Side effect: arriving from another site (for example an email link), the first page shows him logged out; the next click on his site shows him logged in |
| S2 | A login only counts while the person is on the user list | Someone removed from GARDEN_USERS by hand kept a working login | Removed means removed, at once |
| S3 | No built-in account | Someone else running the public code without settings got a "Jun" account, and its reset emails went to Jun | Without GARDEN_USERS nobody can log in, and the site says so |
| S4 | The wiki goes online only after its setup is finished | An unfinished wiki on the internet could be claimed by whoever finished its setup first | Its web name or `wiki.` subdomain is left out (with a note) until the wiki has its admin. Home Assistant's subdomain now defaults to behind the sign-in page for the same reason |
| S5 | Caddy throws away identity headers sent by visitors before asking the sign-in page | Defence in depth; nothing was open | `/beds` and every subdomain drop `X-Auth-Email` and `Remote-*` from visitors first |
| S6 | After a restore, the sign-in folder is private again | A restore made it readable by every account on the computer (it holds password hashes and the email password) | `local/authelia` is owner-only after a restore |
| S7 | LiteLLM's key is made at install | Started by hand without the menu, LiteLLM ran with no key | The key is created with the other secrets |
| S8 | Notes on the optional extras | Mosquitto and ESPHome listen on the home network | Their files say so (home only, never forwarded); ESPHome can get a login in `.env` |
| S9 | Personal leftovers removed from the website code | Jason's home-network address and folder paths were defaults in `main.rs` | Neutral defaults |
| F1 | Jun can set a plant's main photo | "Make this the main photo" was allowed for the user "Jason" only | Any logged-in gardener can |

Accepted, with the reason: likes and favourites take the name sent with them
(this only changes card colours, never a plant), and the 15-minute login
lock counts per name (it slows guessing; a determined stranger could keep Jun
waiting, but never get in). The phone app file (`site/apps/junsgarden.apk`) was an older build (3.1) that
still named Jason's other server and his email; it was rebuilt from
`android-app/` as **3.2** on 2026-10-13 (same signing key, so the phones update; neither
the email nor that server is in it).

## Part 5. Changes after the final test

| # | Fix | What was wrong | Effect on program / install |
|---|---|---|---|
| 16 | Wait for the sign-in page after it starts or restarts | Right after the front door was set up, or a person was added, the sign-in page needed a few seconds to start; anyone (or a test) arriving in those seconds got an error | The menu says "done" only once the sign-in page answers; quick clicks work |
| 17 | "Home only" checks port 80 first, and can use another port | If another program held port 80, Caddy failed with an unclear message | A clear message naming the program; CADDY_HTTP_PORT in .env can move it |
| 18 | "Home only" really hides the garden tools | Caddy runs a catch-all `handle` before `respond`, so `/beds` was passed to the site instead of getting the "not yet" page | `/beds` and `/auth` answer 403 at home until the garden is online |
| 19 | "Remove everything" reads its answers correctly | The final backup swallowed the typed answers, so it stopped with "Nothing was deleted" | The questions after the final backup get Jun's answers |
| 20 | Sign-in settings can always be rewritten | After a restore, the sign-in folder belonged to the administrator (root), so new settings (subdomains, email, web name) silently failed to save | The folder is given back to Jun before writing; a failed write now says so |
| 21 | "Remove everything" removes the programs too | It called "remove the programs" in its no-questions mode, which answered that step's "are you sure?" with its default, **no**. So the data was deleted but the containers, Caddy, the No-IP updater and the nightly backup line stayed | Uninstall level 2 now removes everything it lists. (Level 1 always worked.) |
| 22 | Google Drive backups: your own Google client ID | rclone's shared Google client ID "is being retired and will stop working during 2026" (rclone's own notice), so the weekly Drive copy would stop | Connecting Google Drive (menu 6, 5) first walks Jun through making his own free client ID (rclone's official steps), with the secret hidden; Enter skips it for now. Guide 13 explains it (English and Korean) |
| 23 | Korean words for the newest screens | The "home only" choice and the new Drive steps had no Korean yet; 12 old lines were no longer used | Korean is back to 100% (692 of 692) |
| 24 | Windows: no WSL update on every install | Every install from Windows ran `wsl --update`, which waits for Windows' permission prompt; it looked frozen, with no word about the prompt (found by the test from the Drive download) | WSL is updated only when it is too old to report its version, and the menu first says Windows will ask for permission. `wsl --install` already brings an up-to-date WSL |
| 25 | Windows menu install keeps the Easy parts | Installing from the Windows menu (not the guided installer) switched OFF the garden tools and the wiki: PowerShell turns a left-out text option into "", which the menu passed on as "no parts" (found by the test from the Drive download) | The parts are passed only when they were really chosen (the guided installer); otherwise Easy mode picks the garden tools and the wiki as it should |
| 26 | Windows uses its own tar, ssh and scp | If Git for Windows (or another program) put its own versions first on the PATH, packing the garden failed ("Cannot connect to C:") because those read `C:\...` as a network address | The menu calls Windows' own tools by their full path; and when an install stops, the menu now ends with an error code instead of 0 (which hid this in a test) |
| 27 | "Remove everything" only removes the garden's own Docker storage | It ran `docker volume prune`, which deletes unused storage of ANY program on the computer (found while writing guide 19) | Only volumes of the project "junsgarden" are removed |
| 28 | Uninstall level 1 also removes LiteLLM | Its container could stay behind | All garden parts are removed |
| 29 | Front door back on after "off" restarts the No-IP updater | Menu M, 14 switches it off; menu M, 1 again asked "Enter a new DDNS key anyway?" and, on No, left it off: the web name would quietly stop following the house | The updater is switched back on with the saved key |
| 30 | "Front door off" closes the home-only door on the port really used | It always removed the rule for port 80, even with CADDY_HTTP_PORT set | The right rule is removed |
| 31 | Two messages corrected | The LiteLLM key was said to be in Settings (it is menu M, 15, then 6); the Windows Adminer item pointed at the website's address (Adminer opens on the garden computer only) | Correct directions |
| 32 | The firewall keeps SSH open on the port it really uses | The firewall step allowed SSH on port 22 only. On a computer whose SSH listens on another port, switching the firewall on would have cut off SSH | The step reads SSH's real port(s) (`sshd -T`) and allows those, from the home network only; it says which port stays open |
| 33 | The home network is the one that leads to the router | It took the FIRST network with an address. Once Docker is installed that is Docker's own inside network (172.17...), so "home Wi-Fi only" let only Docker in, and a firewall re-run allowed SSH only from Docker (found while checking the screenshots: with Docker there were eight networks) | It uses the network of the connection to the router (the default route). Tested: old way 172.17.0.0/16 (Docker), new way the real network |
| 34 | The downloads are found in a Korean Ubuntu too | `garden.sh` looked for the photos and wiki zips only in `~/Downloads` and `~/Desktop`. In an Ubuntu set up in Korean those folders are `~/다운로드` and `~/바탕화면`, so the photos were not found | It asks Ubuntu for the real folder names (`xdg-user-dir`, the standard tool), then tries the English names. Tested in 7 places, English and Korean: all found |
| 35 | Windows: the photos zip is not copied twice | The installer copied the 808 MB photos zip into the garden folder on the PC, then sent the folder on. That doubled the space it took, and inside OneDrive it was uploaded twice | The photos and wiki zips go straight from where they are (garden folder, Desktop, Downloads) to the garden computer; the bundle of the folder leaves them out. Tested: both arrive whole, settings and private files still stay behind |
| 36 | Windows: the weekly backup copy goes where YOU pick (Jason's request) | The copy (More 13) always went into `backups\from-garden-computer` in the garden folder. On a Desktop inside OneDrive, 8 copies (several GB) would fill OneDrive's free 5 GB | More 13 first asks where: a folder window (this PC, a USB drive, Google Drive for desktop, a NAS under Network), or a typed network path like `\\192.168.1.20\backups`. It checks it can write there, remembers it, and warns if the folder is in OneDrive. Tested: a normal folder, a Korean-named folder, a network path (the file arrived whole), and a folder that cannot be written (stops politely, keeps the old choice). A bug in the new code (a name clash that wiped the saved settings) was found by this test and fixed before release |
| 37 | Yes/no checks can no longer answer a wrong "no" | 16 checks fed a program's output into `grep -q`. With `pipefail` on (as in garden.sh), `grep -q` stops at the first match and the program feeding it can fail on the cut-off pipe, so a true "yes" became "no" when the output was long (tested: 50 of 50 wrong with long output; today's outputs are short, 0 of 50). Found by a newer shellcheck (SC2337) | A small helper, `has`, reads all of the output before answering; the 16 checks use it. Tested: right every time. shellcheck is clean on all 18 shell scripts |
| 38 | Windows: the installer offers to move the garden folder out of OneDrive (Jason's request) | On many Windows 11 PCs the Desktop is inside OneDrive. OneDrive uploads everything there, and its free plan is 5 GB; the garden's downloads (about 900 MB) and backup copies grow past that | When the garden folder is inside OneDrive, Start-Here-Windows.cmd first shows a small window: **Move it to C:\Users\<you>\junsgarden-for-jun (recommended)**, **Choose another place...**, or **Keep it in OneDrive**. Moving copies every file and checks they all arrived, moves the photos and wiki zips off the OneDrive Desktop too, puts a "Garden folder" shortcut on the Desktop, and opens the installer again from the new place. The old folder is never deleted for you. The menu's INSTALL also says so. Tested on a full copy of the package: 440 of 440 files, identical |
| 39 | Ubuntu: up to 3 network places for the backups (Jason's request) | The nightly backup could go to ONE NAS only | Backups menu (6), 13: **Add another network place** (a second NAS, or a PC's shared folder); every nightly backup goes to each place, the newest 14 kept in each. Menu 6, 8 asks which place to stop. Place 1 is unchanged, so an existing NAS keeps working. Tested with two real SMB 3.1.1 shares: both connected through the menus, identical copies in each, login files readable by the administrator only, stopping one left the other |
| 40 | Moving out of OneDrive keeps the automatic jobs working | The Windows scheduled tasks (More 13's weekly copy, "keep running") name the folder they run from. Moved after they were set up, they would still point at the old folder and stop once it was deleted (found by the guide review) | The move points every "Jun's Garden" task at the new folder. Tested with a throwaway task: before, the old folder; after, the new one (and its start folder) |
| 15 | Wiki setup does not wait for a password nobody can type | When the install ran again unattended (no keyboard, or the "yes to everything" mode) and the wiki import was not finished, it waited forever for the wiki admin password | The helper checks first: a wiki that is already set up just says so (no password asked, also when switching the wiki part off and on); a new wiki with nobody at the keyboard says "later run `bash garden.sh wiki`" and the install carries on |
