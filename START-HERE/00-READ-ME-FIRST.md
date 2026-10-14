# Jun's Garden: the guides

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- These guides explain what the menu installed, how to look after your garden, and what to build next. The install itself is done by the menu (`00-READ-ME-FIRST.html`, one folder up).
- Rule 1: change things through the menu: Settings (menu `8`), users (menu `U`), parts (menu `9`), the front door (menu `M`, `1`). Back up (menu `6`) before any bigger change.
- Rule 2: passwords and keys live only in `.env`. Never put them in a chat, a screenshot, an email or an AI chat.
- Rule 3: for watering projects, use only low-voltage parts (5 V or 12 V). Never cut or wire anything that plugs into a wall outlet.
- The projects go from easy to hard. Start with Project 1 (Uptime Kuma, 20 minutes).
- AI is sometimes wrong in a confident voice. Check its command against the guide before you run it.
- Stuck? Run the checklist (menu `5`). Menu `H` makes a help file with no passwords in it.

---

This folder holds the guides for **your** garden website and for building your
own home and garden projects. The install itself is done by the menu (see
`00-READ-ME-FIRST.html` one folder up). These guides explain what the menu did,
how to look after the garden, and what to build next.

Everything here is made to be done one small step at a time. You can stop at
any step and come back.

---

## The 3 rules

1. **Change things through the menu.** Settings (menu `8`), users (menu `U`),
   parts (menu `9`) and the front door (menu `M`, `1`) keep everything in step.
   Back up (menu `6`) before any bigger change.
2. **Secrets stay secret.** Passwords, app passwords, the No-IP key and API
   keys live only in `.env` (the menu puts them there, hidden). They never go in
   a chat, a screenshot, an email, or a file you send to someone. This includes AI chats.
3. **Safe power only.** For watering projects, use low-voltage parts (5 V or
   12 V) from a kit, and never cut or wire anything that plugs into a wall
   outlet. The projects say "STOP SIGN" where this matters.

---

## What you have

| Part | What it is |
|---|---|
| The website | Your plant catalogue, gallery, blog, data page, guestbook and plant admin, written in Rust and commented line by line |
| Garden tools (`/beds`) | Bed planner, checkup, diagnose, tag scan, spend, wiki queue (optional part) |
| Wiki | Your garden encyclopedia (optional part) |
| `garden.sh` | The menu that installs and looks after everything |
| `START-HERE/` | These guides |
| `lab/` | Practice scripts that do single jobs, for learning (`lab/README.md`) |
| `docker/` | How each program is built, plus ready-made files for Uptime Kuma, Home Assistant, ESPHome and Apache |
| `AGENTS.md`, `GEMINI.md` | Rules that AI helpers read, so they help you safely |
| `WHAT-GETS-INSTALLED.md`, `WHY-THESE-PROGRAMS.md` | Every program, where it comes from, and why it was chosen |

## Searching the guides
The guides are many, so the handbook has a search box at the top of every page.
Open `00-WELCOME.html` (in the garden folder; on Windows it opens after the install).

1. Click the search box at the top (or press `/`). With nothing typed, it shows
   common topics: click one, for example **router**, **backup** or **rename**.
2. Or type any word, for example `password` or `NAS`. Several words find the places
   that have all of them: `backup Google Drive`.
3. Next to the box, choose the language: **English + 한국어**, **English** or **한국어**.
   The menu remembers your choice.
4. Click a result: the guide opens at that very section. `Enter` opens the first
   result; `Esc` clears the box.

It searches every guide here, in both languages, and the other guides too (the n8n
and project guides, what gets installed, the summary, the journal). It works on
this computer, with no internet, and nothing you type is sent anywhere.

## Looking after your garden

| Guide | What it covers |
|---|---|
| [03 Use your garden at home](03-run-your-garden.md) | Your password, email, the AI boxes, your first edit, the everyday menu |
| [23 The full setup](23-full-setup.md) | Everything, the way the garden runs today, in order: choose your level (Easy, Recommended or Full), then one part at a time |
| [11 How your website works](11-maintain-your-site.md) | Where each file lives, and the safe way to change things |
| [12 Your database](12-your-database.md) | The tables, looking inside, safe habits |
| [26 Rename, move, switch off and tidy up](26-rename-move-and-switch.md) | Your own names for folders and pages, moving to a bigger disk, the on / off switch, Docker on Windows |
| [13 Backups and folder names](13-backups-and-folders.md) | Backup names, restoring, moving and renaming folders |
| [14 Reserve your IP, open two ports](14-reserve-your-ip.md) | The router side |
| [04 Put it on the internet](04-go-public-no-ip-caddy.md) | Your No-IP account, the front door, practising first |
| [04b Moving junsgarden.zapto.org](04b-take-over-junsgarden-zapto.md) | What is already yours, and what to check |
| [07 Your sign-in page](07-sso-gate.md) | How the sign-in page and its gates work, guests, two-factor |
| [09 Troubleshooting](09-troubleshooting.md) | Fixes by symptom |
| [25 Common questions](25-common-questions.md) | Short answers with the menu keys: restarts, passwords, friends, undo, photos, updates, moving |

## Learning and AI

| Guide | What it covers |
|---|---|
| [16 Google AI Studio and Gemini](16-ai-studio.md) | Searching the web with sources, coding help, learning about your computer, a free key |
| [08 Gemini setup](08-gemini-setup.md) | Gemini CLI on the laptop |
| [05 Gemini prompts](05-gemini-prompts.md) | Copy-and-paste starters |
| [15 Local AI](15-local-ai.md) | AI on a strong home computer (optional, advanced) |
| [17 Your Android app, and coding it with AI](17-android-app-and-ai-coding.md) | Android Studio, the app on your phone, Gemini in Android Studio, local AI, the safe way to change code |
| [00b Before you start](00b-before-you-start.md) | Everything you need, necessary and optional, with how long each part takes. Read this first |
| [20 SSH and Remote Desktop](20-ssh-and-remote-desktop.md) | Work on the garden computer from your Windows PC; keys; copying files; seeing its screen |
| [22 Bash, step by step](22-bash-walkthrough.md) | The terminal way, harder but not tough: a first walkthrough of the terminal, the prompt, your first commands, running the garden program |
| [24 Tools for learning and programming](24-optional-tools.md) | Very optional: editors, Python, Git and other tools for Windows, Ubuntu and the Raspberry Pi, and whether each one is already there |
| [21 Every program and where it comes from](21-software-and-sources.md) | Each program, necessary and optional, its version, why, and its maker's own documents; how Let's Encrypt gives the padlock |
| [19 Every menu option, explained](19-every-menu-option.md) | Each choice in every menu: what it does, when to use it, what it changes, how to undo it |
| [18 Review the code yourself](18-review-the-code.md) | A map of the files, checking nothing changed, git history, the secret scanner, asking Gemini or a friend to review it |

## Projects (the ladder, easy to hard)

| Step | Guide | You will have | Time | Level |
|---|---|---|---|---|
| 1 | [Project 1: Uptime Kuma](projects/P01-uptime-kuma.md) | A dashboard that tells you if your site is up | 20 min | easy |
| 2 | [Project 2: Backups](projects/P02-backups.md) | Copies of your backups off the laptop | 30 min | easy |
| 3 | [Project 3: Home Assistant](projects/P03-home-assistant.md) | Your own smart-home hub | 45 min | medium |
| 4 | [Project 4: Govee lights](projects/P04-govee-lights.md) | Lights that follow sunset and your rules | 30 min | easy once step 3 works |
| 5 | [Project 5: Weather and rain](projects/P05-weather-and-rain.md) | Rain and frost alerts, and new weather readings | 30 min | medium |
| 6 | [Project 6: Soil sensor](projects/P06-soil-sensor.md) | A sensor that tells you when a bed is dry | 2 h | medium |
| 7 | [Project 7: Automatic watering](projects/P07-automatic-watering.md) | A valve or pump that waters by itself, safely | 3 h | medium to hard |
| 8 | [Project 8: Readings on your website](projects/P08-readings-on-your-website.md) | Your sensor numbers on your own Data page | 3 h | hard |
| 9 | [Project 9: More ideas](projects/P09-more-ideas.md) | A list of next projects to pick from | | choose |

The n8n guides (email and automation) fit after Project 5. Switch n8n on
first (menu `9`):
[n8n 1: First email](n8n/N1-install-and-first-email.md),
[n8n 2: A dry-soil email](n8n/N2-dry-soil-email.md),
[n8n 3: A weekly garden report](n8n/N3-weekly-garden-report.md).

If you set up a laptop from scratch: [01 Ubuntu on the laptop](01-ubuntu-laptop.md)
and [02 Docker](02-docker.md) explain what the install's first steps do.
Curious what Docker is, without the technical part? [02a What is Docker?](02a-what-is-docker.md) (two pictures, plain words).
[10 The plan](10-the-plan.md) is a careful week-by-week order with checkpoints.

Also useful: [the shopping list](06-shopping-list.md) and [your wins checklist](MY-WINS.md).

## How each guide is laid out

Every guide has the same parts in the same order, so you always know where you
are:

1. **What you will build**
2. **Time and cost**
3. **What you need first**
4. **Steps** (numbered, with what you should see after each one)
5. **How you know it worked**
6. **If something goes differently**
7. **Make it yours** (optional changes to try)

## Using Gemini as your helper

- **Give it the context first.** Open [Gemini prompts](05-gemini-prompts.md),
  copy the "context block", and paste it as your first message in a new chat.
- **Paste the exact error.** Copy the red text from the terminal word for word.
- **Ask it to explain.** "Explain this command in plain words before I run it"
  is always a good question.

AI is sometimes wrong, in a confident voice. **Check the command against the
guide before you run it.** If they disagree, trust the guide first, then ask
Gemini why they differ. Never paste a password or key into any AI chat. If you
do by accident, change it (Settings, menu `8`).

## When you get stuck

1. Run the checklist (menu `5`). It says what to fix first.
2. Read the "If something goes differently" part of the guide.
3. Menu `H` makes a help file (no passwords in it). Paste it into Gemini with
   the "I am stuck" prompt, or show it to someone you trust.
4. Still stuck after 20 minutes? Stop, take a break, and write down what you
   tried. Fresh eyes fix most problems.

Being stuck is normal. It happens to professionals every day. Solving it is
the actual skill.

## Three names for one computer (people mix these up all the time)

| Name | What it is | Example | How to see it (in a terminal on that computer) |
|---|---|---|---|
| **User name** | YOUR name on that computer: the account you log in with. Each person has their own. SSH asks for it: `ssh USERNAME@ADDRESS`. | `jun` | `whoami` |
| **Host name** (device name) | The COMPUTER's own name, for people to recognise it on the network. Not a login. | `garden-lab` | `hostname` |
| **IP address** | The computer's NUMBER on your home network. The router gives it; reserve it so it never changes (guide 14). Programs use this to find the computer. | `192.168.1.50` | `hostname -I` (the first number) |

So to reach the garden laptop from another computer you type
`ssh jun@192.168.1.50`: **user name** `@` **IP address**. The host name
(`garden-lab`) only helps you recognise it.

Official beginner pages: [Change the device name (host name)](https://help.gnome.org/gnome-help/about-hostname.html),
[Find your IP address](https://help.gnome.org/gnome-help/net-findip.html),
[Add a new user account](https://help.gnome.org/gnome-help/user-add.html),
[User management on Ubuntu](https://documentation.ubuntu.com/server/how-to/security/user-management/).

## Words you will see

| Word | Plain meaning |
|---|---|
| Terminal | The window where you type commands |
| Command | One line you type in the terminal, then press Enter |
| `sudo` | "Run this as the administrator". It asks for your password |
| Container | A small, sealed box that holds one program and everything it needs |
| Docker | The tool that makes and runs containers |
| Compose | A file (`docker-compose.yml`) that lists the containers to start together |
| Image | The recipe a container is made from |
| Port | A numbered door on a computer. `8030` is the door your website uses |
| localhost | "This computer". Only you can see addresses that start with it |
| `.env` | The file holding your private settings. The menu changes it for you |
| Reverse proxy | A front door program (Caddy) that sends visitors to the right container |
| SSO / sign-in page | One login page in front of several tools (yours is Authelia) |
| DDNS | A service that keeps a web name pointed at your home internet address |
| HTTPS | The padlock in the browser. It means the connection is private |

Have fun with it. You are building real things.
