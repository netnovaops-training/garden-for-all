# Project 6. A soil moisture sensor that tells you when a bed is dry

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- A small ESP32 board with a soil sensor sends the moisture to Home Assistant every minute. Your phone gets a message when the bed is dry.
- Medium, about 2 hours, about $15. Do Project 3 first.
- You need an ESP32 board, a capacitive soil sensor, a USB cable that carries data, and 2.4 GHz Wi-Fi.
- Start ESPHome: `docker compose -f docker/extras/esphome.yml up -d`, then open <http://localhost:6052>.
- Wire the sensor to 3V3, GND and GPIO34, with the board unplugged. Only the long thin part goes in the soil.
- Measure your own dry and wet numbers (Part E). The ESP32 must never get wet.
- STOP SIGN: the sensor and the ESP32 run on 3.3 V from USB. Nothing here touches wall power.

---

**Level:** medium. **Time:** about 2 hours. **Cost:** about $15 (see the
[shopping list](../06-shopping-list.md)).

## What you will build
A small ESP32 board with a soil sensor stuck in a plant pot or bed. Every minute
it sends the moisture number over Wi-Fi to Home Assistant. You see it on a
dashboard and get a phone message when the soil is dry.

How the pieces talk:

```
soil sensor --wire--> ESP32 board --Wi-Fi--> Home Assistant --> your phone
                      (runs ESPHome)
```

You do not write the program for the ESP32 by hand. You write a short settings
file ("YAML") and a tool called **ESPHome** builds the program for you.

## What you need first
- [Project 3](P03-home-assistant.md) done.
- ESP32 board, capacitive soil sensor, and a USB cable that carries data.
- Wi-Fi name and password. Use the 2.4 GHz network (an ESP32 cannot use 5 GHz).

> **STOP SIGN:** the sensor and the ESP32 run on 3.3 V from USB. Nothing here
> touches wall power or water pumps yet.

## Steps

### Part A. Start ESPHome
1. From the `junsgarden-for-jun` folder:

```
docker compose -f docker/extras/esphome.yml up -d
```

2. Open **http://localhost:6052**.
   *You should see:* the ESPHome Dashboard, empty.

### Part B. Wire the sensor
3. The capacitive sensor has three wires. Connect (board unplugged):

   | Sensor wire | ESP32 pin |
   |---|---|
   | VCC (red) | 3V3 |
   | GND (black) | GND |
   | AOUT (yellow) | GPIO34 |

   GPIO34 is an input that can read a smooth number (not just on or off).
   Do not put the sensor's electronics (the top part with the chip) in water.
   Only the long thin part with the line goes in the soil.

### Part C. Make the ESPHome settings file
4. In the dashboard click **New Device**, name it `garden-bed-1`, choose
   **ESP32**, and skip the install for now. ESPHome shows an **encryption
   key** (a long string) for the device. Keep it private. It stays in the
   settings file on your laptop.
5. Click **Edit** on the new device. Under the part ESPHome made for you, add
   this to the end. Do not delete the `api:`, `ota:` and `wifi:` blocks it
   wrote:

```yaml
sensor:
  - platform: adc
    pin: GPIO34
    name: "Bed 1 moisture raw"
    id: bed1_raw
    update_interval: 60s
    attenuation: 12db        # lets the board read up to ~3.1 V
    filters:
      - median:
          window_size: 5
          send_every: 1
  - platform: template
    name: "Bed 1 moisture percent"
    unit_of_measurement: "%"
    accuracy_decimals: 0
    update_interval: 60s
    lambda: |-
      // Two numbers you measure yourself in Part E.
      const float dry = 2.80;   // volts with the sensor dry in the air
      const float wet = 1.30;   // volts with the sensor in a glass of water
      float v = id(bed1_raw).state;
      float pct = (dry - v) / (dry - wet) * 100.0;
      if (pct < 0) pct = 0;
      if (pct > 100) pct = 100;
      return pct;
```

   Save.
6. Read the code once, slowly. `adc` reads the voltage. `median` ignores one
   wild number. The `lambda` turns a voltage into a percent with a little
   arithmetic: the dry number is 0 percent and the wet number is 100 percent.
   Ask Gemini to explain any line you want.

### Part D. Put the program on the board
7. Plug the ESP32 into the **laptop** (not a phone). In the dashboard click
   **Install, Plug into this computer**. If the browser asks for a serial port,
   choose the ESP32. *The first compile takes several minutes.*
   *You should see:* "Successfully uploaded".
8. Watch the log. *You should see:* lines with `Bed 1 moisture raw` and a number.

### Part E. Calibrate (measure your own dry and wet numbers)
9. Hold the sensor in the air. Write down the raw number it shows in the log.
10. Put the sensor's line end in a glass of water, up to the line (not over it).
    Write down the number.
11. Put those into `dry` and `wet` in the file. Install again (wireless this
    time: **Install, Wirelessly**).

### Part F. Add it to Home Assistant
12. In HA a notification "New device discovered" appears. Click **Configure**.
    If not: **Settings, Devices & Services, Add Integration, ESPHome**, then
    enter the board's address from the ESPHome log.
13. Put the ESP32 on a USB power adapter near the bed. Use a short outdoor-safe
    extension cord or put the board in a sealed box with only the sensor wire out.
    The ESP32 must never get wet.

### Part G. A "dry soil" message
14. **Create Automation.**
    **Trigger:** Numeric state, `sensor.garden_bed_1_moisture_percent`, below
    `30`, **for** `00:30:00` (30 minutes, so one dip does not send a message).
    **Action:** notify your phone `Bed 1 is dry`.
    Add a condition that the helper `Rain expected today` from
    [Project 5](P05-weather-and-rain.md) is **off**.

## How you know it worked
- The number changes when you move the sensor from air to water.
- It shows on the HA dashboard.
- Your phone buzzes when you leave the sensor in the air for 30 minutes.

## If something goes differently
| What you see | What to do |
|---|---|
| Laptop does not see the board | Try another cable (many are charge-only). Hold the **BOOT** button on the board while it starts connecting. |
| Wi-Fi never connects | 2.4 GHz only; check name and password in `secrets.yaml` (ESPHome dashboard, Secrets). |
| Numbers jump around | Wires are loose, or the sensor shares a pin with Wi-Fi. GPIO34 is correct on ESP32; do not switch to GPIO2 or GPIO0. |
| The percent is always 0 or 100 | `dry` and `wet` are swapped or not measured; repeat Part E. |

## Make it yours
- Add a second sensor on GPIO35 for a second bed.
- Add a BME280 for air temperature and humidity.
- Show the moisture as a gauge card on your HA dashboard.
- Name the beds the same as the ones in your garden catalogue.

**Next:** [Project 7: Automatic watering](P07-automatic-watering.md)
