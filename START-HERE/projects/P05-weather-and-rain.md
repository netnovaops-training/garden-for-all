# Project 5. Weather and rain alerts for your garden

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Home Assistant sends your phone a message when frost or rain is coming. No hardware.
- Medium, 30 minutes, free. Do Project 3 first, with the phone app connected.
- Add the free weather source **Met.no**: Settings, Devices & Services, Add Integration. It needs no key.
- Frost alert: an automation at `18:00` that sends `Frost tonight. Cover tender plants.` when it is under 3 °C (37 °F).
- Test it: change the condition to `< 99`, press **Run actions**, then put it back.
- Make a Toggle helper called `Rain expected today`. Project 7 uses it to skip watering.

---

**Level:** medium. **Time:** 30 minutes. **Cost:** free. No hardware.

## What you will build
Home Assistant reads the weather forecast and sends a message to your phone
when frost is coming (cover the tender plants) or rain is coming (skip
watering). This is the "brain" of the watering project later.

## What you need first
- [Project 3](P03-home-assistant.md) done, with the phone app connected.

## Steps

### Part A. A weather source
1. **Settings, Devices & Services, Add Integration.** Search **Meteorologisk
   institutt (Met.no)**. Add it with your home's location. It is free and needs
   no key. (HA may already have added one called **Sun** or **Home**; that is
   fine. If Met.no is not offered, choose any free weather integration.)
2. Look at **Settings, Devices & Services, Entities** and search `weather.`
   *You should see:* an entity like `weather.forecast_home`.

### Part B. A frost alert
3. **Create Automation.**
4. **Trigger:** Time, at `18:00`.
5. **Condition:** Template. Paste this (read it first; Gemini can explain it):

```
{{ state_attr('weather.forecast_home', 'temperature') | float(99) < 3 }}
```

   It means "the current temperature is under 3 degrees Celsius". Use
   `Settings, System, General` to see if HA uses Celsius or Fahrenheit and change
   the number to match (3 °C is 37 °F).
6. **Action:** Notifications, **Send a notification via mobile_app_YOUR_PHONE.**
   Message: `Frost tonight. Cover tender plants.`
7. Save as `Frost alert`.
8. Test: edit the condition to `< 99`, press **Run actions**, then put it back.

### Part C. A "rain is coming" flag
9. **Settings, Devices & Services, Helpers, Create Helper, Toggle.** Name it
   `Rain expected today`.
10. Make an automation: **Trigger:** every morning at `06:00`. **Action:** turn
    the helper on if the forecast says `rainy` or `pouring` (a **Choose**
    action with a **State** condition on `weather.forecast_home`), otherwise
    off. Ask Gemini: "Show me how to build this automation in the Home
    Assistant screen, step by step." Paste the entity names you see.

You will use `Rain expected today` in [Project 7](P07-automatic-watering.md)
to skip watering.

## How you know it worked
- You receive the test frost message on your phone.
- The helper turns on when the forecast shows rain.

## If something goes differently
| What you see | What to do |
|---|---|
| No `weather.` entity | Part A did not finish. Add the integration again. |
| Template shows an error | Read the entity name in Entities and use that exactly. |
| No message arrives | Check the phone app is connected: **Settings, Companion app** on the phone. |

## Make it yours
- A heat alert at 95 °F / 35 °C.
- A message that says the 3-day forecast every morning.
- Compare HA's forecast with your own weather on the Data page of your site.

**Next:** [Project 6: Soil sensor](P06-soil-sensor.md)
