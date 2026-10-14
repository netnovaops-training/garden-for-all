# Project 4. Govee lights that follow the sun

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- A Govee light that Home Assistant controls, and that turns on by itself at sunset, without the internet.
- 30 minutes. A Govee light with LAN control costs about $15 to $40. Do Project 3 first.
- Not every Govee model supports LAN control. Check yours before you buy.
- In the Govee Home app: tap the light, then the gear, and switch **LAN Control** on.
- In Home Assistant: **Settings, Devices & Services, Add Integration**, then **Govee lights local**.
- Make an automation: trigger **Sun, Sunset**, action **Light, Turn on**. Test it with **Run actions**.
- "No devices found"? LAN Control is off, or the light and Home Assistant are on different Wi-Fi networks.

---

**Level:** easy once Home Assistant works. **Time:** 30 minutes.
**Cost:** a Govee light that supports LAN control, about $15 to $40, if you do
not have one yet.

## What you will build
A Govee light that you control from Home Assistant and that turns on by itself
when the sun goes down. It works over your home network, without the internet.

## What you need first
- [Project 3](P03-home-assistant.md) done.
- A Govee Wi-Fi light **that supports LAN control**. Not every Govee model does.
  Check the Govee Home app (device settings) or Govee's support page for the
  model. The Home Assistant page that lists this is
  <https://www.home-assistant.io/integrations/govee_light_local>.
- The Govee Home app on your phone and the light already set up in it.

## Steps

### Part A. Turn on LAN control in the Govee app
1. Open the **Govee Home** app. Tap your light.
2. Tap the settings gear (top right).
3. Find **LAN Control** and switch it **On**.
4. Do this for every light you want in Home Assistant.

### Part B. Add it to Home Assistant
5. In HA open **Settings, Devices & Services, Add Integration**.
6. Search for **Govee lights local**. Choose it.
7. Follow the prompts. It scans your network and finds the lights by itself.
   *You should see:* your light listed with a name.
8. Click the light's name, then use the brightness slider and color picker.
   *You should see:* the real light change.

### Part C. Your first automation
9. **Settings, Automations & Scenes, Create Automation, Create new automation.**
10. **Add trigger, Sun, Sunset.** (Offset: `-00:30:00` makes it come on 30
    minutes before sunset.)
11. **Add action, Light, Turn on.** Pick your Govee light. Pick a warm color
    and a brightness you like.
12. **Save.** Name it `Garden glow at sunset`.
13. Test without waiting: three-dots menu on your automation, **Run actions**.

## How you know it worked
- The light turns on from the HA button.
- The next evening it turns on by itself.

## If something goes differently
| What you see | What to do |
|---|---|
| "No devices found" | LAN Control is off for that light, or HA and the light are on different Wi-Fi networks (for example 2.4 GHz vs a guest network). Put them on the same one. |
| It worked, then stopped | The light's address changed. In your router, give the light a fixed address (address reservation). |
| The light is not in the list at all | That model may not support LAN control. Ask Gemini with the exact model name. The cloud-based Govee integration is a different, less reliable way. |

## Make it yours
- Turn off at 11 pm: copy the automation and use **Time** as the trigger.
- Use a color for status: green when the website is up, red when it is down (a
  strong tie to Project 1; ask Gemini how HA can read Uptime Kuma).
- A "garden party" scene: three lights, three colors, one button.

**Next:** [Project 5: Weather and rain](P05-weather-and-rain.md)
