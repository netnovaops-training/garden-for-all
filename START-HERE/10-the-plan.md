# 10. The plan: a careful order, with checkpoints

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- A careful order for the guides, with a checkpoint after each step. A "session" is about 2 to 3 hours.
- Build the foundation first and prove each layer works. Never add a new thing on top of something that is not working.
- Before you begin (15 minutes): read guide 00, start a notes file `lab-notes.txt`, and have a password manager ready.
- The order: Ubuntu, then Docker and your garden, Uptime Kuma and backups, Gemini, Home Assistant, n8n email, sensors and watering, your readings on the website, and last the internet.
- The internet steps (sessions 14 to 16): practise with a test name first (guide 04, Part C), then do it for real with guide 23, Parts 9 to 11.
- Watering: the valve must work dry on the table before it ever touches water. Never skip the dry test.
- Every week (15 minutes): `bash lab/09-health-check.sh`, Ubuntu updates, and a copy of `backups/` on a USB stick.

---

This is the careful, step-by-step plan. It puts the guides in the best order,
says when to stop and check, and keeps risky steps for later. Go at your own
speed. A "session" below is about 2 to 3 hours. You can split one over two days.

**The idea:** build the foundation first, prove each layer works, then add the
next layer. Never add a new thing on top of something that is not working.

## Before you begin (15 minutes)
- [ ] Read [00 Read me first](00-READ-ME-FIRST.md) (the 3 rules).
- [ ] Open a notes file called `lab-notes.txt`. Each session, write the date,
      what you did, and what you saw. This helps you and Gemini.
- [ ] Have a password manager ready. Every new password goes in it first.
- [ ] Charger plugged in. Laptop on a table, not a bed or a couch.

## Session 1. A clean base
Guides: [01 Ubuntu](01-ubuntu-laptop.md), then the first start: unzip the
download, right-click `Start-Here-Ubuntu.sh` and choose **Run as a Program** (or
type `bash Start-Here-Ubuntu.sh` in that folder). The first time, it gets the
program from GitHub and checks every file; that brings `garden.sh`, `lab/` and
the guides. When the menu opens, type `0` to leave it for today: the install is
Session 2. (`lab/` did not exist before this first start, so the commands below
work only after it.)
**Checkpoint:** `bash lab/09-health-check.sh` shows no red FAIL for the laptop
section. The laptop restarts and comes back.
**Why careful:** a good base saves hours later.

## Session 2. Docker and your garden
Read [02 Docker](02-docker.md) to know what Docker is (the install sets it up
for you). Then the install: open the menu again (`Start-Here-Ubuntu.sh`, as in
Session 1) and type `1`. Then [03 Run your garden](03-run-your-garden.md).
**Checkpoint:**
- `curl http://127.0.0.1:8030/healthz` says ok.
- You logged in and added one plant.
- You took a backup (menu `6`, then `1`) and the file exists.
**Stop here for a day.** Look at the site. Read `rust/junsgarden/src/main.rs`.

## Session 3. Watch it and protect it
Guides: [Project 1 Uptime Kuma](projects/P01-uptime-kuma.md),
[Project 2 Backups](projects/P02-backups.md).
**Checkpoint:** you stopped the site on purpose and Uptime Kuma noticed. You
restored a deleted test plant from a backup.
**Why careful:** you should know your safety net works before you rely on it.

## Session 4. Your helper (Gemini)
Guide: [08 Gemini setup](08-gemini-setup.md).
**Checkpoint:** your Gem answers in numbered steps, and you ran
`bash lab/10-collect-for-gemini.sh` and read the file it made.

## Sessions 5 to 7. Home automation
Guides: [Project 3 Home Assistant](projects/P03-home-assistant.md), then
[Project 4 Govee](projects/P04-govee-lights.md), then
[Project 5 Weather](projects/P05-weather-and-rain.md).
**Checkpoint after each:** the thing works twice in a row, after a restart too.
**Stop and enjoy it.** A light that comes on at sunset is a real win. Tick it in
MY-WINS.md.

## Sessions 8 to 9. Email and automation with n8n
Guides: [n8n 1](n8n/N1-install-and-first-email.md),
[n8n 2](n8n/N2-dry-soil-email.md), [n8n 3](n8n/N3-weekly-garden-report.md).
**Checkpoint:** you received emails from your own lab at
`you@gmail.com`.

## Sessions 10 to 12. Sensors and watering (hardware)
Guides: [Project 6 Soil sensor](projects/P06-soil-sensor.md), then
[Project 7 Watering](projects/P07-automatic-watering.md).
Buy parts only when you reach a project ([shopping list](06-shopping-list.md)).
**Checkpoints:** the sensor changes when you move it from air to water. The
valve works **dry on the table** before it ever touches water. The watering stops
by itself at the time limit. Ask a friend who knows electrics to check the wiring before the
first power-on.
**Never skip the dry test.**

## Session 13. Your readings on your website
Guide: [Project 8](projects/P08-readings-on-your-website.md).
**Checkpoint:** a new row in the database every 15 minutes.

## Sessions 14 to 15. The internet: practise first
Guides: [23 The full setup, Part 9](23-full-setup.md) and
[04 Put your garden on the internet](04-go-public-no-ip-caddy.md), Parts A
to C: your own No-IP account and web name, the CGNAT check, then a **test name**
(menu `M`, `1`, choice `2`, for example `garden.test`). The scripts `lab/04`,
`lab/05` and `lab/06` do the same jobs one at a time, if you want to learn each piece.
**Checkpoint:** on the garden computer, `https://garden.test` opens your site
(Firefox warns about the practice certificate: that is expected), and `/beds`
opens after you sign in. Your web name is in your own No-IP account, with its
DDNS key in your password manager.
**Why careful:** the next session is the only step that opens your home network
to the internet. Practise until it feels familiar, and do the real thing on a
calm day with time to spare.

## Session 16. Your web name for real (junsgarden.zapto.org)
Guides: [23 The full setup, Parts 10 and 11](23-full-setup.md),
[04, Part D](04-go-public-no-ip-caddy.md), and [04b](04b-take-over-junsgarden-zapto.md).
Menu `M`, `1`, choice `3`, then forward only ports 80 and 443 in the router (guide 14).
**Checkpoint:** from a phone on mobile data, the real name opens your site with a
padlock, with the real plants, and you can log in. Run
`bash lab/09-health-check.sh`: no FAIL. You know the way back: menu `M`, `14`,
then `3` takes the garden off the internet again (guide 04).

## After that. Keep it healthy (15 minutes a week)
- [ ] `bash lab/09-health-check.sh` and fix any FAIL.
- [ ] `sudo apt update && sudo apt upgrade -y`, then `sudo reboot` if asked.
- [ ] Copy `backups/` to a USB stick.
- [ ] Look at Uptime Kuma. Is everything green?

## Later and optional
- [07 Your sign-in page](07-sso-gate.md): two-step sign-in and more users (the
  front door step in Session 16 already set it up).
- [Project 9 More ideas](projects/P09-more-ideas.md).

## Pace and rest
- Take a break every hour. Stand up, drink water.
- If you feel stuck or tired, stop and come back. Nothing here is a race.
- One new thing at a time. Finishing small things is better than starting
  many.
- Write down what you want to learn next in `lab-notes.txt`. Your list matters.

Back to [00 Read me first](00-READ-ME-FIRST.md).
