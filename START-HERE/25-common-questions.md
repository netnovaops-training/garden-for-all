# 25. Common questions

## Quick version
*Short answers to the questions people ask most. Each one says which menu keys to press.*

**Every day**
- Does the garden start by itself after a power cut or a restart?
- Can I switch the computer off?
- What happens when I close the laptop lid?
- How do I know it is working?
- Where are my web addresses?
- On Windows: does the PC have to stay on?
- Can I switch the garden off for a while?

**Passwords and people**
- I forgot my website password.
- I forgot my Ubuntu password.
- Can a friend edit, or only look?
- Who can see my garden?
- Someone should not have access any more.
- Where are my passwords and keys kept?
- Should the website have its own Gmail?
- I pasted a password or key somewhere by mistake.

**When something is wrong**
- I made a mistake. How do I undo it?
- My photos are missing.
- The AI boxes do not answer.
- My web name stopped working.
- Is the disk filling up?
- Where are the logs?
- How do I ask for help without sharing passwords?

**Changes and updates**
- How do I update the computer?
- How do I update the garden's own programs?
- How does the phone app update?

**Moving or removing**
- How do I move the garden to a new computer?
- Can I give folders and pages my own names?
- How do I uninstall it?

**Windows and Docker**
- I already use Docker Desktop. Is that a problem?
- I have an old Windows 7 PC. Can I use it?
- Can someone else start their own garden with this?

---

## Every day

### Does the garden start by itself after a power cut or a restart?
Yes, once Ubuntu is running again (if the computer itself went off, press its
power button). Docker then starts the garden by itself, even before anyone
logs in, and the front door (Caddy) and the No-IP updater start too. One
exception: if you stopped the garden with menu `3`, it stays stopped until you
start it again.

**Keys:** menu `2` starts it. Menu `5` checks it.
**More:** [guide 19](19-every-menu-option.md) (menu 2 and 3)

### Can I switch the computer off?
Yes, nothing is lost. While it is off, nobody can open your garden, and the
2:30 am nightly backup does not happen. Switch it off the normal way: Ubuntu's
top-right menu, then **Power Off**.

**More:** [guide 13](13-backups-and-folders.md) (the nightly backup)

### What happens when I close the laptop lid?
Normally Ubuntu goes to sleep, and the garden stops. Guide 01, steps 24 and 25,
makes the laptop stay awake with the lid closed. A closed laptop gets hot:
leave the lid open a little.

**More:** [guide 01](01-ubuntu-laptop.md) (Part G)

### How do I know it is working?
Run the checklist. Each line says **PASS** (good), **WARN** (fix soon) or
**FAIL** (fix first), with the fix written under it. The last line counts
them: `0 fail` is what you want.

**Keys:** menu `5` (or `bash garden.sh check`). It is also saved in `CHECKLIST.txt`.
**More:** [guide 09](09-troubleshooting.md)

### Where are my web addresses?
Menu `A` lists every address of your garden, and which ones need signing in.
It also saves them in `MY-ADDRESSES.txt` in the garden folder.

**Keys:** menu `A`
**More:** [guide 23](23-full-setup.md) (Part 15)

### On Windows: does the PC have to stay on?
Yes. On Windows the garden runs inside Windows' own Ubuntu (WSL), so it stops
when the PC is off or asleep. Set **Settings, System, Power, Sleep** to
**Never**. After a restart, sign in to Windows: if you said yes to the
"Jun's Garden keep running" task during the install, it starts the garden
again; if not, use the Windows menu `2`.

**More:** [guide 00b](00b-before-you-start.md)

---

### Can I switch the garden off for a while?
Yes. Menu `M`, then `14`, then `1` (Everything OFF). It stops every program and the
front door, and deletes nothing: your plants, photos, data, cache, backups and
settings stay in their folders. Use it when you do not want to host the site for a
while, or when the traffic is too much. Menu `M`, `14`, then `2` turns it all on
again, even months later, and checks the website. While it is off, keep confirming
your free No-IP name every 30 days.

**Keys:** menu `M`, then `14`
**More:** [guide 26](26-rename-move-and-switch.md) (The on / off switch)

## Passwords and people

### I forgot my website password.
On the login page, click **Forgot password? / First time?** and the website
emails you a link. That only works when email is set up (menu `8`, then `3`).
Or set a new password on the garden computer: menu `U`, then `5`. Nothing
shows while you type; that is normal.

**More:** [guide 07](07-sso-gate.md)

### I forgot my Ubuntu password.
Ubuntu cannot email you a new one. Start the computer in recovery mode, as
Ubuntu's official page explains: <https://wiki.ubuntu.com/RecoveryMode>
(hold `Shift`, or press `Esc`, while it starts; choose **Advanced options**,
then the line ending in **(recovery mode)**, then **Drop to root shell prompt**).
Then type these one at a time, with your own user name instead of `jun`:

```
mount -o remount,rw /
passwd jun
reboot
```

**More:** [guide 01](01-ubuntu-laptop.md) (your account), [guide 22](22-bash-walkthrough.md) (sudo and your password)

### Can a friend edit, or only look?
Anyone can look at your garden once it is online; they need no account. To
let a friend **edit**, use menu `U`, then `2`, and tell them their first
password in person. A **guest** (menu `U`, then `3`) can only sign in on the
sign-in page: no editing, no garden tools. Guests need the front door first
(menu `M`, then `1`).

**More:** [guide 07](07-sso-gate.md), [guide 23](23-full-setup.md) (Part 12)

### Who can see my garden?
Before the front door, only the garden computer itself. Once it is online,
everyone can see the website: the plant catalogue, the gallery, the blog.
Only people on your user list can edit, and the garden tools (`/beds`) open
only after signing in.

**Keys:** menu `U`, then `1` shows who can sign in.
**More:** [guide 07](07-sso-gate.md)

### Someone should not have access any more.
Menu `U`, then `4` removes every login that person has. Menu `U`, then `8`
(LOCK DOWN) leaves only you: new passwords, and everyone is signed out.

**More:** [guide 19](19-every-menu-option.md) (Users and access)

### Should the website have its own Gmail?
It is a good idea, and free: <https://accounts.google.com/signup>. Use it for the
website's email (menu `M`, `3`), the Google Drive backups, the AI key, and to sign in
to Tailscale, DuckDNS and No-IP with "Sign in with Google". Your personal inbox stays
apart, and handing the website on one day means handing on one account. Turn on its
2-Step Verification. The garden's own sign-in stays your garden password: no outside
account can open the garden.

One Gmail can also give each service its own address (`name+noip@gmail.com`), so you see who
sends what: guide 00b, "Plan your website's name and email".

**More:** [guide 00b](00b-before-you-start.md)

### Where are my passwords and keys kept?
In one private file, `.env`, in the garden folder. Only you can read it. The
menu never shows a password or key; it only says "set" or "not set". Never
paste them into a chat, an email or an AI chat.

**Keys:** menu `K` for keys, menu `U`, then `5` for your password.
**More:** [guide 00](00-READ-ME-FIRST.md) (rule 2)

### I pasted a password or key somewhere by mistake.
Change it now. Your password: menu `U`, then `5`. A key: menu `K`, choose
the key, then `1` to paste a new one; then delete the old key on that
service's own website.

**More:** [guide 09](09-troubleshooting.md) (Safety problems)

---

## When something is wrong

### I made a mistake. How do I undo it?
Put a backup back: menu `7`, then pick one (the newest is at the top). Before
it changes anything, it saves what is there now as a `before-restore` backup.
Everything goes back to the time of that backup. Changed your mind? Menu `7`
again, and pick the `before-restore` one.

**Keys:** menu `7`
**More:** [guide 13](13-backups-and-folders.md)

### My photos are missing.
The photos are a separate download: `junsgarden-photos-addon.zip`. Leave it
zipped in your Home, Downloads or Desktop folder, then choose menu `6`, then
`11`. Photos you already have are kept. Photos you added yourself and then
lost come back with a backup (menu `7`).

**Keys:** menu `6`, then `11`
**More:** [guide 13](13-backups-and-folders.md)

### The AI boxes do not answer.
Test your key: menu `K`, then `1` (Gemini), then `2` (Test). If it says the
key is wrong, choose `1` and paste it again. If it says the free limit is
used up, it resets tomorrow. A free OpenRouter key is a backup for busy times:
menu `K`, then `2`, then `1`.

**More:** [guide 16](16-ai-studio.md)

### My web name stopped working.
A free No-IP name must be confirmed every 30 days: No-IP emails you a link,
click **Confirm** (look in spam too). Then run the checklist (menu `5`): it
shows whether the No-IP updater is running and whether the name points at
your home. If the updater is not running: menu `K`, then `N`.

**More:** [guide 09](09-troubleshooting.md) (The internet side), [guide 04](04-go-public-no-ip-caddy.md)

### Is the disk filling up?
The checklist's last check says "Disk is ...% full": PASS under 85%, FAIL from
85%. Nightly backups are deleted by themselves after 7 days. Backups you make
yourself are never deleted for you, and each one is about 800 MB. To free
space, copy your old ones to a USB stick, delete them from `backups/`, then use
the clean-up: menu `M`, `14`, then `5`. It removes only Docker's leftovers (old
versions of the garden's programs, older than 7 days) and asks first.

**Keys:** menu `5`, and menu `M`, then `14`
**More:** [guide 13](13-backups-and-folders.md), [guide 09](09-troubleshooting.md), [guide 26](26-rename-move-and-switch.md)

### Where are the logs?
The garden programs' recent messages: menu `M`, then `6`. The install and the
front door save everything they show in the `logs/` folder
(`logs/install-progress.txt` shows where an install stopped). The nightly
backup writes to `backups/backup.log`.

**Keys:** menu `M`, then `6`
**More:** [guide 09](09-troubleshooting.md) (Look at the logs)

### How do I ask for help without sharing passwords?
Menu `H` makes a help file in the `help/` folder, with passwords and keys
taken out. Read it first (it is plain text), then send it to the person
helping you, or paste it into Gemini.

**Keys:** menu `H`
**More:** [guide 09](09-troubleshooting.md), [guide 05](05-gemini-prompts.md)

---

## Changes and updates

### How do I update the computer?
Once a week, in a terminal:

```
sudo apt update && sudo apt upgrade -y
```

This also updates Docker and Caddy, because they came from their makers'
Ubuntu sources. If Ubuntu asks for a restart, type `sudo reboot`; the garden
starts again by itself.

**More:** [guide 01](01-ubuntu-laptop.md) (Part E), [guide 22](22-bash-walkthrough.md) (sudo)

### How do I update the garden's own programs?
The menu has no update button for them. Menu `1` repairs what is broken, but
it skips everything that already works, so it does not fetch newer versions.
By hand: back up first (menu `6`, then `1`), then in the garden folder type
the line below, which downloads newer copies of the ready-made programs (the
database, the wiki, the sign-in page). Then menu `2` starts the new copies.

```
docker compose pull --ignore-buildable
```

**More:** [guide 02](02-docker.md) (Docker commands)

### How does the phone app update?
Each time it opens, the app asks your website whether there is a newer app
(the file `site/apps/version.json`). If there is, it offers it, and Android
asks you before installing. A newer app only appears there when you build one
yourself.

**More:** [guide 17](17-android-app-and-ai-coding.md)

---

## Moving or removing

### How do I move the garden to a new computer?
Install the garden on the new computer (menu `1`). Copy your newest backup into
its `backups/` folder, then menu `7` and pick it. Your keys and email password
are not in the backup: add them again with menu `K` and menu `8`, then `3`.
To go online from there: menu `M`, then `1`, and point the router at the new
computer (guide 14).

**More:** [guide 13](13-backups-and-folders.md) (Moving to a new computer), [guide 14](14-reserve-your-ip.md)

### Can I give folders and pages my own names?
Yes, with menu `M`, then `18`. Do not rename them by hand: the code and the
database still use the old name, and the page would break. The menu renames the
file or folder AND every place that uses it, shows you everything first, and makes
a backup. One thing to know: the guides keep showing the names the garden came
with. `logs/renames.tsv` lists what is called what now (menu `M`, `18`, then `4`).
Moving the whole garden folder to a bigger disk is in the same menu (`3`).

**Keys:** menu `M`, then `18`
**More:** [guide 26](26-rename-move-and-switch.md)

### How do I uninstall it?
Menu `M`, then `12`. Choice `1` removes the programs and keeps your garden
(photos, database, backups); menu `1` installs it again. Choice `2` removes
everything, but first makes a final backup and asks you to type `DELETE`.

**More:** [guide 19](19-every-menu-option.md) (Uninstall)

## Windows and Docker

### I already use Docker Desktop. Is that a problem?
No. When you install on this PC, the installer asks which Docker to use. Docker
Desktop is the easy way: the garden's programs show in its window, with buttons.
Docker inside Ubuntu is the hands-on way. If you picked the hands-on way, keep
Docker Desktop's WSL integration switched off for `JunsGarden` (Docker Desktop >
Settings > Resources > WSL integration); the checklist warns you if it is on. The
clean-up never touches the images of your other Docker projects.

**More:** [guide 26](26-rename-move-and-switch.md) (On Windows: two ways to run Docker)

### I have an old Windows 7 PC. Can I use it?
Not for the garden. Windows 7 gets no security updates since January 2020, so it is
not safe for a website on the internet, and it has no WSL and no current Docker.
Put Ubuntu on that old laptop instead (copy your files to a USB stick first), or run
the garden on another computer and use the old PC only to look at it.

**More:** [guide 26](26-rename-move-and-switch.md), [guide 01](01-ubuntu-laptop.md)

### Can someone else start their own garden with this?
Yes: the code is public and free to use (MIT license). Without your garden's data
the installer sets up an empty garden. A starter garden for new people, with
free-to-use plants for their area and their own photos, is an idea for a later
version; it is not in this one.

**More:** [guide 18](18-review-the-code.md) (the public code)

---

**Next:** [19 Every menu option, explained](19-every-menu-option.md)
