# 04. Put your garden on the internet (your web name, HTTPS, sign-in page)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- This guide puts your garden on the internet at `https://junsgarden.zapto.org`, with the padlock and your sign-in page. The menu does almost all of it: menu `M`, then `1`.
- It takes about 1 hour. Free.
- It needs a separate garden computer (an Ubuntu laptop or a Pi). The garden inside Windows' own Ubuntu (WSL) is for using on that PC only (see the note under "The three choices").
- Part A: make your own free No-IP account, create the name, and make a DDNS key.
- Part B: check you are not behind CGNAT (one address shared by several homes). If you are, hosting from home cannot work: ask your internet company for a public IP address.
- Part C: practise first with choice `2` and a made-up `.test` name.
- Part D: choice `3` for the real name, then forward ports 80 and 443 in the router (guide 14). Test from a phone with Wi-Fi off.
- Warning: forward only ports 80 and 443. Never 22, 8030, 3306, 8123 or anything else.
- A free No-IP name must be confirmed every 30 days: click **Confirm** in No-IP's email, or the name stops working.

---

## What you will build
Your garden at **https://junsgarden.zapto.org**, with the padlock, the same
gates as before (the garden tools open only after signing in), and only two
ports open on your router. The menu does almost all of it: menu `M`, then `1`
(**The front door**). This guide explains each piece and the few things only
you can do.

| Piece | Job | Official page |
|---|---|---|
| **No-IP + its updater (DUC)** | Keeps your web name pointing at your home's internet address, even when your internet company changes it | <https://www.noip.com/support/knowledgebase/> |
| **Caddy** | The front door. Takes visitors on ports 80 and 443, gets the HTTPS padlock by itself, passes each visit to the right program | <https://caddyserver.com/docs/> |
| **Authelia** | Your sign-in page at `/auth`, in front of the garden tools | <https://www.authelia.com> |
| **Your router** | Reserves the garden computer's address, and forwards ports 80 and 443 to it, and nothing else | guide [14](14-reserve-your-ip.md) |

## Time and cost
About 1 hour. Free. A free No-IP name must be confirmed every 30 days: No-IP
emails you a link, and you click **Confirm**.

## Read this first
> **This opens your home to the internet.** Robots will knock within hours.
> That is normal, and it is why the setup is careful:
> - Only ports **80** and **443** go to the garden computer. Never forward 22
>   (SSH), 8030, 3306, 8123 (Home Assistant) or anything else.
> - Everything except Caddy listens only on the garden computer itself.
> - Keep Ubuntu updated. The checklist (menu `5`) tells you when updates are waiting.
> - If your internet company puts several homes behind one address ("CGNAT"),
>   hosting from home cannot work. Part B shows how to check.

## The three choices in the front door step (menu `M`, then `1`)

![What you will see: the front door's three choices](images/screen-14-front-door.png)

| Choice | What you get | Needs |
|---|---|---|
| **1 Not on the internet yet** (home Wi-Fi only, no www) | The garden for phones and computers on your home Wi-Fi at `http://LAPTOP-NAME.local` (and the laptop's IP address). Nobody outside the house can reach it; the router is not touched. The garden tools stay private until choice 3. | Nothing. Use it now. **Next step:** sign up at No-IP (Part A), DuckDNS if you want sub-names (optional, see Advanced), then choice 3. |
| **2 Practice** | Everything (HTTPS, your sign-in page, the gates) on the laptop only, with a made-up `.test` name. | Nothing. Good for learning. |
| **3 The real web name** | `https://junsgarden.zapto.org` for everyone, with the padlock and your sign-in page. | Your own No-IP account with the name (Part A), and the router (guide 14). |

> **Garden running inside Windows' own Ubuntu (WSL) on a Windows PC?** Then
> choice 1 (home Wi-Fi only) and choice 3 (the internet) do not work as written
> here. Windows' own Ubuntu has its own internal address inside the PC, and
> the router and your phones cannot reach it. On that Windows PC the garden is
> for using on the PC itself. To put the garden online, or to show it to other
> phones and computers at home, use a separate garden computer: an Ubuntu laptop
> (guide 01) or a Raspberry Pi. Want to learn more about how WSL's networking
> works? Microsoft explains it here (reading only; the garden does not set this
> up): <https://learn.microsoft.com/windows/wsl/networking>

## Part A. Your own No-IP account and the name (once)
1. Make a free account at <https://www.noip.com> (use your own email). Confirm the email No-IP sends.
   The free plan has **one** web name, confirmed every 30 days (No-IP emails a link; unconfirmed
   names are deleted). Their paid Enhanced plan has three (<https://www.noip.com/remote-access>).
   One is enough: more addresses can come free from DuckDNS (see Advanced below).
2. **Dynamic DNS, No-IP Hostnames, Create Hostname.** Name: `junsgarden`,
   domain: `zapto.org`. (If it says the name is taken, wait a day and try
   again, or choose another name. The menu works with any name.)
3. **Dynamic DNS, DDNS Keys, Add DDNS Key.** Choose your hostname. Write down the
   key's **username** and **password** somewhere safe (a password manager).
   A DDNS key can only update your names. It cannot log in to your account,
   so it is the safe thing to keep on the computer.

## Part B. Check you are not behind CGNAT
On the router's page, find its **WAN** or **Internet** address. On the garden
computer run `curl -s https://api.ipify.org; echo`. If the two are the same,
you are fine. If the router shows an address starting with `10.`, `100.64.` to
`100.127.` or `192.168.`, your internet company shares one address between
homes. Ask them for a "public IP address" (sometimes free), or keep the garden
at home only.

## Part C. Practise first (recommended)
Do this at the garden computer's own screen and keyboard (it is not possible
with Windows' own Ubuntu on a Windows PC). Menu `M`, `1`, then choice `2`. Type
a made-up name ending in `.test`, for example `garden.test`. Everything gets set
up (Caddy, the sign-in page, the gates) on the garden computer only, with a
practice certificate. Nothing on the internet changes. Open `https://garden.test`
in Firefox on that computer. Firefox warns about the practice certificate: that
is expected. Click **Advanced…**, then **Accept the Risk and Continue**. Try
`/beds`, and sign in. When you are happy, do Part D.

## Part D. The real thing
1. Reserve the garden computer's address in the router (guide [14](14-reserve-your-ip.md), steps 1 and 2).
2. Menu `M`, `1`, then choice `3`. Web name: `junsgarden.zapto.org` (or the web name you chose). Answer `y` when it asks if the name is in your own No-IP account.
   It installs Caddy, starts your sign-in page, opens ports 80 and 443 in the
   firewall, installs the No-IP updater (type the DDNS key from Part A; the
   password is hidden), and asks you to set your password.
3. In the router, forward TCP **80** and **443** to the garden computer's
   address (guide [14](14-reserve-your-ip.md), step 3). Nothing else.
4. Test from a phone with **Wi-Fi off**: open `https://junsgarden.zapto.org`.
   The first visit can take 30 seconds while Caddy gets the certificate.
5. Run the checklist (menu `5`). Section 6 checks the name points at your home.

## What the front door step changes on the computer
- Installs Caddy from its official source and writes `/etc/caddy/Caddyfile`
  from `docker/caddy/Caddyfile.template`. Read it: `cat /etc/caddy/Caddyfile`.
- Writes the sign-in page's settings to `local/authelia/` (private, only you can read them).
- Firewall: allows 80 and 443.
- Installs the No-IP updater and saves your key in `/etc/default/noip-duc` (only the administrator can read it).
- Sets `PUBLIC_URL` so password emails link to your real address.

## Advanced (optional): subdomains with DuckDNS

![The Subdomains menu (menu M, then 13)](images/screen-15-subdomains.png)

**Why:** a free No-IP name like `junsgarden.zapto.org` cannot have sub-names
(`wiki.junsgarden.zapto.org` does not exist). A free **DuckDNS** name can:
`wiki.junsgarden.duckdns.org`, `n8n.junsgarden.duckdns.org`, `status.junsgarden.duckdns.org`
all point to your home (DuckDNS: <https://www.duckdns.org/spec.jsp>). Separate addresses suit
separate jobs, for example a Home Assistant panel or your automations, each behind your sign-in page:
far less is exposed than opening each program to the internet on its own. That gives extra programs their own neat address, each
with its own padlock, through the same two router ports. Your garden stays at
`junsgarden.zapto.org`.

**How (menu `M`, then `13`, Subdomains):**
1. Choose `1`: sign in at <https://www.duckdns.org> (Google or GitHub login),
   add a name (for example `junsgarden`, if it is free), copy your **token**,
   and paste it in (hidden). The menu checks it with DuckDNS, saves the token in
   `~/.config/garden-duckdns/token` (only you can read it), and adds an updater
   that runs every 5 minutes (one cron line, the way DuckDNS's own install page
   shows: <https://www.duckdns.org/install.jsp>).
2. Choose `2` to add a subdomain: `wiki` (your wiki), `n8n`, `status` (Uptime
   Kuma) or `home` (Home Assistant), or any name and port. It asks whether to put
   it **behind your sign-in page**. Say yes for anything that has no login of its
   own (n8n, Uptime Kuma). Those then sign in at `https://auth.junsgarden.duckdns.org/auth/`.
3. The menu writes Caddy's settings and the sign-in page's rules, checks them,
   and reloads Caddy. If something is wrong it puts the old settings back.
4. Menu `A` (My addresses) lists every address and which ones need signing in.

**Keep it safe:**
- Behind the sign-in page by default. Only open ones with their own login (the wiki, Home Assistant).
- Never forward any port except 80 and 443. Subdomains all come in through those two.
- Two-factor (a phone code) on the sign-in page is a good idea once you have subdomains (guide 07).
- Removing a subdomain (menu `M`, `13`, then `3`) closes it straight away.

AI helpers know Caddy and DuckDNS well: if you want something unusual, paste
`cat /etc/caddy/Caddyfile` (it has no passwords) into Gemini with your question.

## Website name swap: a different web name (optional)

![The Website name swap explains itself first](images/screen-18-website-name.png)

Want another name than `junsgarden.zapto.org` (for example one for your mom's
garden)? Menu `M`, then `16` (on Windows: More, `17`), or `bash garden.sh webname`.
It works right after downloading too, before installing.
1. Add the new name in YOUR No-IP account first (Part A). The No-IP updater (DUC)
   updates every name on your DDNS key, so it needs no change.
2. Type the new name. `https://`, `http://`, `www.` and anything after `/` are left
   out for you, so `https://www.momsgarden.zapto.org/` becomes `momsgarden.zapto.org`.
3. It shows the files it will change (the share buttons, the privacy page, the
   phone app's update check `site/apps/version.json`, the Android app's source),
   keeps a copy of each in `logs/web-name-change_DATE/`, changes them, updates
   your settings and the front door if the garden is online, offers to rebuild
   the website, and saves the change in your git history (`git show` shows every line).
4. Phones that already have the app look at the old name for updates: build the
   app again (`android-app/README.md`) and install it once on each phone.

To undo: copy the files from `logs/web-name-change_DATE/` back, or run it again with the old name.

> **Why use the menu for this, not File Explorer?**
> The web name is written in 12 files (the list is `WEBNAME_FILES` in
> `garden.sh`: the website's code, the privacy page, the phone app's update file,
> the Android app, `docker-compose.yml` and more), and also in your settings
> (`.env`) and the front door's settings (Caddy and the sign-in page). Renaming by
> hand means finding every one; miss one and the share buttons, the phone app's
> updates or the front door quietly keep the old name. The menu changes all of
> them together: it checks the new name looks like a web name, lists the files
> before it changes anything, keeps a copy of each, updates `.env` and the front
> door when they use the old name, and offers to rebuild the website (it then waits
> until the website answers). It can be undone, as described just above.

## Taking the garden off the internet
Menu `M`, then `14`, then `3` (Off the internet only): stops Caddy, closes ports 80 and 443 in the firewall, stops
the No-IP and DuckDNS updaters and the sign-in page. The garden keeps running at
home. Menu `M`, `1` puts it back. Also remove the two forwarding rules in the router.

## Option: Apache (for learning)
Apache runs a lot of the world's websites. `lab/06-install-apache-learn.sh`
installs it as a private learning server on `127.0.0.1:8081`, next to Caddy.
Caddy stays the front door. `docker/extras/Caddyfile.with-apache.example` shows
how Caddy can send `/learn/` to Apache. Do not forward 8081 on the router.

## Safety checks after it is live
- Checklist (menu `5`): firewall shows only 22 (home only), 80 and 443.
- Watch Caddy for a day: `sudo journalctl -u caddy -f`. Requests for
  `/wp-login.php` or `/.env` are robots, and they get a 404. That is the site
  doing its job.
- Automatic security updates: `sudo apt install -y unattended-upgrades`, then
  `sudo dpkg-reconfigure -plow unattended-upgrades` and answer Yes.

## If something goes differently
| What you see | What to do |
|---|---|
| Name does not point home | `sudo systemctl status noip-duc`. Is the hostname confirmed in your No-IP account? Run menu `M`, `1` again to re-enter the key. |
| Works at home, not outside | Port forwarding is wrong, or CGNAT (Part B). Many routers cannot open their own public name from inside. Always test from phone data. |
| Certificate error | Ports 80 and 443 must reach the garden computer from the internet. `sudo journalctl -u caddy -n 50` says why. Too many tries? Wait an hour (Let's Encrypt limits) and practise with `.test`. |
| `/beds` keeps sending you to sign in | Set your password again (menu `U`, `5`). Check the checklist's "sign-in page" line. |
| "502 Bad Gateway" | A program behind Caddy is down: menu `2` (start), then menu `5`. |
| Name stopped working after a month | You missed No-IP's 30-day confirmation email. Log in to No-IP and confirm. |

**Next:** [04b](04b-take-over-junsgarden-zapto.md) (moving the name from the old server) and [07](07-sso-gate.md) (more about your sign-in page).
