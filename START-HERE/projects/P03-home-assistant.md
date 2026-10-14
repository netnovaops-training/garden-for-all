# Project 3. Home Assistant: your own smart-home hub

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Home Assistant: a free program that puts your smart devices in one place, runs your rules, and keeps the data in your house.
- Medium, 45 minutes, free. Do guide 02 (Docker) first.
- Start it: `docker compose -f docker/extras/home-assistant.yml up -d homeassistant`.
- Wait 2 to 5 minutes the first time, then open <http://localhost:8123> and create your owner account with a **new** password.
- Set your home's location on the map, so the sun and weather work.
- Install the **Home Assistant Companion** app on your phone, on the same Wi-Fi.
- Phone cannot reach it? Allow only your home network in the firewall (see the table). Never open it on the router.

---

**Level:** medium. **Time:** 45 minutes. **Cost:** free.

## What you will build
Home Assistant (HA) is a free program that puts all your smart devices in one
place, lets you make rules ("automations"), and keeps the data in your house
instead of a company's cloud. Jason uses the same idea for his house.

## What you need first
- Guide 02 (Docker) done.
- The laptop on your home network, ideally with a cable.

## Steps
This follows Home Assistant's own Docker instructions
(<https://www.home-assistant.io/installation/linux#docker-compose>).
1. Start it:

```
docker compose -f docker/extras/home-assistant.yml up -d homeassistant
```

   (Naming `homeassistant` at the end starts only that one. Mosquitto waits for
   Project 6.)
   *You should see:* `Container homeassistant Started`.
2. Wait 2 to 5 minutes the first time. Then open **http://localhost:8123**.
   *You should see:* "Welcome to Home Assistant" and a button to create your
   account.
3. Create your owner account. Use a **new** password and keep it in your
   password manager.
4. Name your home, set your location on the map (this lets the sun and weather
   work), and choose your units. You can skip analytics.
5. Click **Finish**.
   *You should see:* a dashboard. HA will say it found some devices on your
   network (printers, TVs). You can ignore them for now.
6. Install the phone app (**Home Assistant Companion**) and connect it while on
   the same Wi-Fi. It shows your location and battery to HA. That helps later
   ("send me a message when it will rain").

## How you know it worked
- `http://localhost:8123` shows your dashboard.
- After `sudo reboot`, HA starts again by itself. Check with `docker ps`.

## If something goes differently
| What you see | What to do |
|---|---|
| Page does not load right away | HA takes a few minutes the first time. `docker logs -f homeassistant` shows progress. |
| Cannot reach it from your phone | Use the laptop's address from guide 01: `http://192.168.x.x:8123`. If it still fails: `sudo ufw allow from 192.168.0.0/16 to any port 8123` (adjust to your home network's range). Only your home network, never the router. |

## Make it yours
- Open **Settings, Devices & Services**. Add **Sun** if it is not there. It
  tells HA when the sun rises and sets, used in Project 4.
- Make a dashboard card that shows the time and today's weather.
- Ask Gemini: "What is the difference between an integration, a device and an
  entity in Home Assistant?" It explains the three words you will see everywhere.

**Next:** [Project 4: Govee lights](P04-govee-lights.md)
