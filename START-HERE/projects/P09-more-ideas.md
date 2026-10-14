# Project 9. More ideas (pick what looks fun)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- A list of next projects, easier to harder. Each one builds on something you already made.
- Easier: plant QR labels, a garden lights scene, a water-use diary, a dashboard on an old tablet.
- Medium: a frost reminder light, compost bin temperature, a greenhouse fan, Grafana charts, a Telegram bot.
- Harder: a time-lapse camera, a weather station, a practice copy of your website, adding your own page.
- Pick the one that makes you smile. Write one sentence: what will exist when it is done?
- Build the smallest version that works, tick it off in MY-WINS.md, then make it better.

---

Each idea builds on something you already made. Ask Gemini for a plan using the
"design me a Home Assistant automation" prompt, or write one in this same
shape: build it, check it works, then make it safer.

## Easier
- **Plant name labels.** Print QR codes that open a plant's page on your site.
  The catalogue already supports plant links. Print and laminate a few for the
  beds. (Your website works at `http://localhost:8030` only on your laptop; QR
  codes pointing to it will only work on your home Wi-Fi unless you finish guide 04.)
- **Garden lights scene.** One button: all Govee lights to a dim warm color.
- **Water-use diary.** Home Assistant counts how long the valve ran; show it as
  a number card. Compare weeks.
- **Dashboard on an old tablet.** Home Assistant's dashboard in full screen, on
  a tablet by the back door.

## Medium
- **Frost cover reminder with a light.** When frost is forecast, a light by the
  door turns blue until you tap "Done" on your phone.
- **Compost bin temperature.** An ESP32 with a waterproof DS18B20 probe.
- **Greenhouse fan.** A 5 V or 12 V fan on a relay (same wiring as Project 7),
  starting when the temperature sensor reads 80 °F / 27 °C.
- **Grafana.** A drawing program for sensor data. Home Assistant can send data to
  InfluxDB, and Grafana draws it. Its makers' documentation: <https://grafana.com/docs/>.
- **Telegram bot.** Send a photo of the bed on a schedule. The website's own
  camera tools do something like this on Jason's server.

## Harder
- **Time-lapse camera.** An old phone or a Pi camera taking a picture every hour,
  then `ffmpeg` joins them into a movie of a plant growing.
- **Weather station.** An outdoor BME280 and rain gauge on an ESP32 powered by a
  solar panel and battery. Compare it with the forecast from Project 5.
- **Compare with the real thing.** Install a second copy of your website in a
  clean folder, break it on purpose, and fix it. This is how people learn
  disaster recovery.
- **Add a page.** Follow `rust/junsgarden/src/main.rs` ("To add a page: write a
  function in src/pages/, add one `.route` line"). Make a page called
  `/hello` that says your name. Then a page that lists your last 10 sensor
  readings.
- **Your own sign-in gate.** [07 SSO gate](../07-sso-gate.md) in front of all your tools.

## How to choose
Pick the one that makes you smile. Then do it in this order:
1. Write one sentence: what will exist when it is done?
2. List the parts. Do you own them?
3. Build the smallest version that works.
4. Tick it off in MY-WINS.md.
5. Make it better, or choose the next one.

Back to [the ladder](../00-READ-ME-FIRST.md).
