# The lab scripts: how to use them

**The menu does all of this for you:** `bash garden.sh` (or the icon), choice 1.
These scripts are for LEARNING: each one does a single job, so you can read it,
run it, and see exactly what that one job does. They do the setup for you, in order, so your lab works when they are
done. Each one explains itself at the top, is safe to run twice, and tells you
what to do next. Read a script before you run it. That is a good habit, and
the comments make it easy.

Run them from the project folder (`~/junsgarden-for-jun`) with `bash`:

```
cd ~/junsgarden-for-jun
bash lab/01-prepare-ubuntu.sh
```

## The order

| # | Script | What it does | Time | Run it |
|---|---|---|---|---|
| 1 | `01-prepare-ubuntu.sh` | Updates Ubuntu, firewall, swap, keeps the laptop awake, automatic security updates | 10 to 40 min | once, then restart |
| 2 | `02-install-docker.sh` | Installs Docker and Compose the official way | 5 to 10 min | once, then log out and in |
| 3 | `03-start-garden.sh` | Builds and starts your website, adds the nightly backup, runs the health check | 5 to 60 min | once |
| 4 | `04-install-caddy.sh NAME` | Installs Caddy (HTTPS front door) for your web name | 5 min | when you go online |
| 5 | `05-install-noip-duc.sh` | Installs the No-IP updater and saves your DDNS Key privately | 5 min | when you go online |
| 6 | `06-install-apache-learn.sh` | Installs Apache as a private learning server on 127.0.0.1:8081 | 5 min | optional |
| 7 | `07-start-extras.sh WORD` | Starts Uptime Kuma, Home Assistant, ESPHome or n8n | 2 min each | when you reach that project |
| 8 | `08-install-nightly-backup.sh` | Schedules the backup at 2:30 every night | 1 min | once (script 3 does it) |
| 9 | `09-health-check.sh` | Checks your whole lab. Changes nothing | 10 sec | any time, weekly |
| 10 | `10-collect-for-gemini.sh` | Collects facts for Gemini, with secrets removed | 10 sec | when you are stuck |

Also in the project folder: `setup.sh` (called by script 3), `docker/backup.sh`
and `docker/set-password.sh`.

## How to read the results
- `PASS` is good. `WARN` means fine for now but fix it soon. `FAIL` means fix
  this first.
- Every FAIL and WARN line says what to do next.
- If a script stops with an error, read the last 10 lines. Then see
  [09 Troubleshooting](../START-HERE/09-troubleshooting.md).

## Each script: the how-to

### 1. Prepare Ubuntu
**Run:** `bash lab/01-prepare-ubuntu.sh`
**You should see:** lots of update text, `Status: active` for the firewall, a
swap line, and `DONE` with your laptop's address.
**Next:** `sudo reboot`.
**Undo (sleep settings):** the commands are listed at the top of the script.

### 2. Install Docker
**Run:** `bash lab/02-install-docker.sh`
**You should see:** "Hello from Docker!" and `DONE`.
**Next:** log out and back in, then `docker run --rm hello-world` (no `sudo`).

### 3. Start the garden
**Run:** `bash lab/03-start-garden.sh`
It asks your first name and email. It never shows passwords.
**You should see:** `{"status":"ok","plant_count":0}` and the health check.
**Next:** `sh docker/set-password.sh YourName` and open `http://localhost:8030`.

### 4. Caddy (the front door)
**Run:** `bash lab/04-install-caddy.sh YOUR-TEST-NAME.ddns.net`
(add `--with-apache` if you installed Apache and want `/learn/`).
**You should see:** `Valid configuration` and `Status: active` for the firewall.
**Not done until:** your router forwards ports 80 and 443 to the laptop.
**Undo:** `sudo cp /etc/caddy/Caddyfile.bak /etc/caddy/Caddyfile && sudo systemctl reload caddy`

### 5. No-IP updater
**Run:** `bash lab/05-install-noip-duc.sh`
You type the DDNS Key username and password (the password is hidden).
**You should see:** `active (running)`.
**Check:** `getent hosts YOUR-NAME.ddns.net` and `curl -s https://api.ipify.org`
show the same address.
**Undo:** `sudo systemctl disable --now noip-duc`

### 6. Apache for learning
**Run:** `bash lab/06-install-apache-learn.sh`
**You should see:** "Hello from Apache" and `DONE`.
**Undo:** the line at the top of the script.

### 7. Start a helper
**Run:** `bash lab/07-start-extras.sh kuma` (or `ha`, `esphome`, `n8n`)
**See what is running:** `bash lab/07-start-extras.sh status`
**Stop one:** `bash lab/07-start-extras.sh stop n8n`

### 8. Nightly backup
**Run:** `bash lab/08-install-nightly-backup.sh`
**Check:** `crontab -l` shows one line with `docker/backup.sh`.

### 9. Health check
**Run:** `bash lab/09-health-check.sh`
Do this after every change, and once a week.

### 10. Facts for Gemini
**Run:** `bash lab/10-collect-for-gemini.sh`, then read the file it writes
(`gemini-help-DATE.txt`) before pasting it into Gemini.

## What "my lab works" means
After scripts 1, 2 and 3, `bash lab/09-health-check.sh` should show no FAIL
lines. After 4 and 5 and the router, your site opens from a phone on mobile data.

## Safe habits for scripts
- Read a script before you run it. Ask Gemini to explain any line.
- Never run a script someone sends you that you did not read.
- Scripts here never ask you to turn off the firewall or open ports other than
  80 and 443.
