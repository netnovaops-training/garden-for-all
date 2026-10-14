# Project 1. Uptime Kuma: know when your site is down

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- A dashboard that checks your garden website every minute and shows a green or red dot.
- Easy, 20 minutes, free. Do guides 01 to 03 first.
- Start it: `docker compose -f docker/extras/uptime-kuma.yml up -d`, then open <http://localhost:3001> and make an admin account.
- **Add New Monitor**: type HTTP(s), URL `http://host.docker.internal:8030/healthz`, interval 60.
- Test the alarm: `docker compose stop site` turns the dot red within about a minute. `docker compose start site` turns it green again.
- If you open port 8030 to your home network (step 4), never forward port 8030 on your router.

---

**Level:** easy. **Time:** 20 minutes. **Cost:** free.

## What you will build
A dashboard that checks your garden website every minute and shows a green or
red dot. Later it can send a message to your phone when something goes down.

## What you need first
- Guides 01 to 03 done.

## Steps
1. From the `junsgarden-for-jun` folder, start it:

```
docker compose -f docker/extras/uptime-kuma.yml up -d
```

   *You should see:* `Container uptime-kuma Started`.
2. Open **http://localhost:3001**. The first visit asks you to make an admin
   username and password. Make them and save the password in your password
   manager.
3. Click **Add New Monitor**.
   - Monitor Type: **HTTP(s)**
   - Friendly Name: `Garden website`
   - URL: `http://host.docker.internal:8030/healthz`
   - Heartbeat Interval: 60
4. If step 3 shows red with "ENOTFOUND host.docker.internal", use the laptop's
   address from guide 01 instead: `http://192.168.x.x:8030/healthz`. (The garden
   only listens on this laptop's own door, so also edit `docker-compose.yml`:
   change `"127.0.0.1:8030:8030"` to `"8030:8030"`, run `docker compose up -d`,
   and know the garden is now visible to your home network. That is fine at
   home; do not forward port 8030 on your router.)
5. Click **Save**.
   *You should see:* a green dot and "Up".

## How you know it worked
Run `docker compose stop site`. Within about a minute the dot turns red. Run
`docker compose start site`. It turns green again. You just tested an alarm.

## If something goes differently
| What you see | What to do |
|---|---|
| Red, "ECONNREFUSED" | The garden container is stopped, or step 4 applies. |
| Cannot open port 3001 | `docker ps` shows `uptime-kuma`? If not, run step 1 again. |

## Make it yours
- Add a monitor for `https://www.google.com` so you can tell "my site is down"
  from "my internet is down".
- In Settings, Notifications, add Telegram or email so your phone buzzes.
- After guide 04 is done, add a monitor for your public name.

**Next:** [Project 2: Backups](P02-backups.md)
