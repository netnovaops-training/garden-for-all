# 09. Troubleshooting: when something does not work

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- How to fix problems: a method first, then common problems with the check and the fix for each.
- Stay calm and change nothing yet. Write down what you were doing.
- Run the checklist (menu `5`). Fix the first FAIL line, then run it again.
- If the install stopped, menu `1` again carries on from there.
- Change one thing, test, and write down what you changed.
- Still stuck after 20 minutes? Menu `H` makes a help file (no passwords in it). Your backups in `backups/` mean you can always go back.
- Stop and ask someone you trust if a battery looks swollen, anything smells hot or burnt, or water has touched anything electrical.

---

Problems are normal. This page gives you a **method**, then a list of common
problems with the exact check and fix for each.

## The method (follow it in order)
1. **Stay calm. Change nothing yet.** Write down what you were doing and what
   you expected.
2. **Run the checklist:** menu `5` (or `bash garden.sh check`). It lists what is
   working and what is not, and what to do. Fix the **first FAIL** line, then run it again.
   If the INSTALL stopped: `logs/install-progress.txt` shows the step it stopped at;
   menu `1` again carries on from there.
3. **Read the error.** The last 5 lines usually say the cause.
4. **Look at the logs:**
   - Website: `docker compose logs --tail 40 site`
   - Database: `docker compose logs --tail 40 db`
   - Caddy: `sudo journalctl -u caddy -n 40 --no-pager`
   - No-IP: `sudo journalctl -u noip-duc -n 20 --no-pager`
5. **Change one thing**, test, and write down what you changed.
6. **Still stuck after 20 minutes?** Menu `H` makes a help file in `help/`
   (no passwords in it). Read it, then paste it into Gemini (see
   [16](16-ai-studio.md)) or show it to someone you trust.
7. **Stuck and worried you broke something?** Your backups are in `backups/`.
   You can always go back.

## Quick fixes you can always try
| Try | Command |
|---|---|
| Start everything that should be running | menu `2` (or `docker compose up -d`) |
| The install stopped half way | menu `1` again: it shows the steps done and carries on |
| The build "hangs" for minutes | On an old laptop the Rust build is slow. If text still appears now and then, it is working. |
| "Cannot connect to the Docker daemon" right after install | Log out and back in once (the docker group). The menu uses sudo until then. |
| Restart just the website | `docker compose restart site` |
| Restart the laptop (fixes many things) | `sudo reboot` |
| See what is using the disk | `docker system df` and `df -h /` |
| Free space safely | `docker system prune` (asks first; does not delete your data folders) |

---

## Ubuntu and the laptop
| Problem | Check | Fix |
|---|---|---|
| Laptop will not start from the USB | Boot menu key (guide 01, Part C) | Try another USB port; remake the stick (Rufus DD mode); set USB first in the BIOS |
| Black screen after install | Did it get past the login? | Boot with `nomodeset` (guide 01 table), then install the right graphics driver |
| No Wi-Fi | `ip a` shows a Wi-Fi line? | Plug in a cable; search the menu for **Additional Drivers** |
| Everything is very slow | `htop` (press `q` to leave) | Close the browser; check `free -h` for swap; consider Xubuntu and an SSD |
| Laptop is very hot | Is the lid shut? | Open it a little; put it on a stand; clean the vents |
| Screen goes to sleep or laptop suspends | `systemctl status sleep.target` | Run `lab/01-prepare-ubuntu.sh` again and restart |
| `sudo` says "not in the sudoers file" | The first user you made should be allowed | Ask someone you trust who knows Linux; do not try to fix this alone |

## Docker
| Problem | Check | Fix |
|---|---|---|
| `permission denied ... docker.sock` | `groups` shows `docker`? | Log out and in again (or reboot) |
| `docker: command not found` | `docker --version` | Run `lab/02-install-docker.sh` |
| Build stops: "killed" or "out of memory" | `free -h`, then `dmesg --level=err` | Add swap (`lab/01-prepare-ubuntu.sh`), close other programs, build again |
| Build is very slow | Normal on old laptops | Wait; the first build takes 15 to 45 minutes (up to an hour on a very old laptop or a Pi) |
| Port is already in use | `sudo ss -ltnp` (look for :8030) | Stop the other program, or change the port in `docker-compose.yml` |
| Disk full | `df -h /` | `docker system prune`; delete old files in `backups/` after copying them |

## The garden website
| Problem | Check | Fix |
|---|---|---|
| `localhost:8030` does not load | `docker ps` lists `garden-site`? | `docker compose up -d`; read `docker compose logs site` |
| `{"status":"error"}` on healthz | Database up? `docker compose ps` | `docker compose restart db` then `restart site` |
| Cannot log in | Did you set a password? | Menu `U`, then `5` |
| "Too many tries" | 5 wrong tries lock you out | Wait 15 minutes |
| Photos will not upload | `ls -ld local/images` | `chmod -R a+rwX local/images` |
| No password email arrives | Is email set up? | Menu `8` Settings, `3`, and send the test email; check spam |
| Page is empty | You have no plants yet | Add some in `/streamlit/`, or restore a backup |
| Gallery error in the log about `tags_cache.csv` | Do you have gallery photos? | Normal with no photos |
| Site is down after a reboot | `docker ps` | Docker starts it again by itself (it is set to). If you stopped it with menu `3`, it stays stopped: menu `2` starts it |

## The internet side (Caddy, Apache, No-IP, router)
| Problem | Check | Fix |
|---|---|---|
| Name does not resolve | `getent hosts YOUR-NAME.ddns.net` | `sudo systemctl status noip-duc`; check the key; confirm the free name on noip.com (No-IP emails you a link every 30 days: click **Confirm**) |
| Name shows the wrong address | Compare with `curl -s https://api.ipify.org` | Wait 5 to 10 minutes; make sure only ONE updater is running for that name |
| Works at home, not on a phone | Test with Wi-Fi off | Router port forwarding for 80 and 443 (guide 04, Part D); some routers cannot reach their own name from inside, which is normal |
| Padlock or certificate error | `sudo journalctl -u caddy -n 50` | Ports 80 and 443 must reach the laptop from outside; the name must resolve first |
| 502 Bad Gateway | `curl http://127.0.0.1:8030/healthz` | The website is down: `docker compose up -d` |
| Caddy will not start | `sudo caddy validate --config /etc/caddy/Caddyfile` | Fix the line it names |
| "Address already in use" for Caddy or Apache | `sudo ss -ltnp` (look for :80 and :443) | Only Caddy may use 80 and 443. Apache must be on 127.0.0.1:8081 (`lab/06-install-apache-learn.sh`) |
| Router says CGNAT or a `100.64.x.x` address | Compare router WAN address with `ipify` | Hosting from home is not possible; use Tailscale for private access |

### An old or wrong padlock certificate

**What you see:** the browser says "Your connection is not private", or that the
certificate has expired or is not valid for this name.

**Why:** Caddy (the front door) gets a new certificate by itself, about every two
months. It can only do that while your web name points to your home and ports 80
and 443 reach the garden computer. If one of those stops, the old certificate runs out.

**Fix it, in this order:**
1. Run the checklist: menu `5`.
2. Is your web name pointing home? `sudo systemctl status noip-duc` should say
   **active (running)**. If not: menu `K`, then `N`, and paste your No-IP key again.
3. Is the router still sending ports 80 and 443 to the garden computer's address?
   (Guide 04, Part D. If the address changed: guide 14.)
4. Ask Caddy why: `sudo journalctl -u caddy -n 50`. Read the lines with "error".
5. Set the front door again: menu `M`, then `1`, then `3` (your real web name). It
   rewrites the settings, checks them and restarts Caddy, which then gets a new
   certificate (up to a minute).
6. In the browser, reload the page with **Ctrl+F5**.

A practice name (ending in `.test`) always shows a warning. That one is expected.
Caddy's own page: <https://caddyserver.com/docs/automatic-https>.

## Home Assistant, Govee, n8n, ESPHome
| Problem | Check | Fix |
|---|---|---|
| Home Assistant page is blank | `docker logs --tail 30 homeassistant` | Wait a few minutes after first start |
| Govee light not found | Is **LAN Control** on in the Govee app? Same Wi-Fi network? | Turn it on; put the light and the laptop on the same network |
| n8n email test fails | Credential test message | App password wrong, or 2-Step Verification off; try port 587 |
| ESP32 not seen by the laptop | `lsusb` or the ESPHome device list | Use a data USB cable; hold BOOT while connecting |
| Sensor values jump around | Loose wire | Re-seat wires; use GPIO34 |

## Safety problems: stop and ask
Stop and ask someone you trust before you go on if:
- A battery looks swollen, or anything smells hot or burnt.
- Water has touched anything electrical.
- You see logins or strange requests in the Caddy log that you do not recognise,
  repeated for hours (a few bots are normal).
- You pasted a password somewhere by mistake. Change that password first (menu
  `U`, `5` for your garden password; menu `K` for a key), then delete what you
  pasted, if you can.

**Next:** [10 The plan](10-the-plan.md)
