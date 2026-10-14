# Project 7. Automatic watering, safely

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- A low-voltage valve (or small pump) waters a bed for a set time when the soil is dry, skips watering when rain is coming, and stops with one button.
- Medium to hard, about 3 hours, about $35 to $60. Do Projects 3, 5 and 6 first.
- Low voltage only: a 12 V adapter and 5 V USB. If a part plugs into a wall outlet or says 110 V, 120 V or 230 V, do not use it.
- Use a **normally closed** valve: if power or Wi-Fi fails, the water stops.
- The ESP32, the relay and every wire joint go inside the waterproof box. Only the valve gets wet.
- Do the whole project dry first, with the water OFF. Ask a friend who knows electrics to check the wiring before you first plug it in.
- The board itself closes the valve after 10 minutes, whatever Home Assistant says. Add a `STOP WATER` button to your dashboard too.

---

**Level:** medium to hard. **Time:** about 3 hours. **Cost:** about $35 to $60.

## What you will build
A low-voltage valve (or a small pump) that waters a bed for a set time when the
soil is dry, skips watering when rain is coming, can never run for more than a
fixed number of minutes, and can be stopped with one button.

```
Home Assistant --Wi-Fi--> ESP32 --signal--> relay --switches--> 12 V valve
                                                        |
                                           12 V adapter + fuse
```

The relay is only a switch. The ESP32 is not strong enough to run a valve
itself.

## What you need first
- Projects 3, 5 and 6 done and working.
- The parts from the [shopping list](../06-shopping-list.md): ESP32, 1-channel
  5 V opto-isolated relay module, 12 V **normally closed** solenoid valve, 12 V 2 A
  adapter, 1N4007 diode, 2 A fuse and holder, waterproof box, hose fittings.

## STOP SIGNS (read all four)
1. **Low voltage only.** Everything runs from a 12 V adapter and 5 V USB. If any
   part has a plug for a wall outlet, or says 110 V, 120 V or 230 V, do not
   use it. Never open or rewire an adapter.
2. **Water and electronics stay apart.** The ESP32, relay and every wire joint go
   inside the waterproof box. Only the valve (made for water) is wet. Run the
   box higher than the valve and make wires drip downward before they enter the
   box ("drip loops").
3. **Normally closed valve.** If power or Wi-Fi fails, the valve closes and the
   water stops. Never use a normally-open valve for this.
4. **Test with the water supply OFF first.** Do the whole project dry. You
   should hear the valve click before it ever sees water.

Ask a friend who knows electrics to look at the wiring before the first time you plug it
in. Use prompt 6 ("Check my plan") in [05](../05-gemini-prompts.md) too.

## Steps

### Part A. Dry wiring on the table (no water, not plugged in)
1. Wire the relay module to the ESP32:

   | Relay pin | ESP32 pin |
   |---|---|
   | VCC | 5V (the VIN pin) |
   | GND | GND |
   | IN | GPIO26 |

2. The valve side. The relay has three screw terminals: COM, NO (normally open),
   NC (normally closed). The relay's "NO" means the circuit is open until the
   ESP32 turns it on. Wire the 12 V line like this:

   ```
   12 V adapter (+) --> fuse --> relay COM
   relay NO ----------> valve wire 1
   valve wire 2 ------> 12 V adapter (-)
   ```

3. Put the 1N4007 diode across the valve's two wires, **stripe (band) toward
   the + side** (toward the wire from the relay). It soaks up an electric kick
   that valves make when they switch off. Ask Gemini to show a picture
   description of "flyback diode across a solenoid valve" to check your
   direction.

### Part B. The ESPHome settings
4. In the ESPHome dashboard make a new device `garden-water-1` (or add to the
   existing one if you have a spare GPIO). Add this under the generated parts:

```yaml
switch:
  - platform: gpio
    pin:
      number: GPIO26
      inverted: true            # most relay boards switch ON when the pin goes LOW
    id: valve_relay
    name: "Bed 1 valve"
    restore_mode: ALWAYS_OFF    # after any restart, the valve starts OFF
    # THE SAFETY NET: whatever Home Assistant says, the valve closes after 10 minutes.
    on_turn_on:
      - delay: 10min
      - switch.turn_off: valve_relay
```

   `restore_mode: ALWAYS_OFF` and the 10-minute limit live **on the board
   itself**, so a Home Assistant crash or a lost Wi-Fi signal cannot leave the
   water running.
5. Install to the board by USB or wirelessly. In the log, switch the valve on
   and off from the dashboard.
   *You should see:* you hear a click from the relay, and the valve buzzes or
   clicks.

### Part C. Water for a set time (in Home Assistant)
6. **Settings, Devices & Services, Helpers, Create Helper, Number.** Name it
   `Watering minutes`. Min 1, max 10, default 3.
7. **Create Automation** `Water bed 1 when dry`:
   - **Trigger:** `sensor.garden_bed_1_moisture_percent` below `30`, for 30 minutes.
   - **Condition 1:** helper `Rain expected today` is **off**.
   - **Condition 2:** Time is between `06:00` and `09:00` (watering in the cool
     morning wastes less water).
   - **Condition 3:** the valve was not on in the last 12 hours (Template:
     `{{ (now() - states.switch.bed_1_valve.last_changed).total_seconds() > 43200 }}`).
   - **Actions:** turn the valve **on**, wait `Watering minutes`, turn the valve **off**,
     notify your phone `Watered bed 1 for X minutes`.
8. Add a second automation: if the valve has been on for more than 12 minutes
   (it should not be), turn it off and send a phone message `Valve stuck on`.

### Part D. A big red stop button
9. Add a button card to your dashboard that turns **off** the switch
   `Bed 1 valve`. Name it `STOP WATER`. Try it while the valve is on.

### Part E. First wet test
10. Put the box where it will live. Connect the valve between the garden hose
    tap and the drip line. Use Teflon tape on threaded fittings.
11. Turn the water on slowly. Look for drips for 5 minutes. Fix leaks first.
12. Turn the valve on from Home Assistant for 1 minute. Watch it the whole time.
13. Run the automation for real once, with you standing there.

## How you know it worked
- Click on, water flows; click off, water stops.
- The valve stops by itself after the time you chose.
- Pull the power: the valve closes.
- With `Rain expected today` on, the automation does not water.

## If something goes differently
| What you see | What to do |
|---|---|
| Relay clicks but no water | Check the valve has the right direction arrow, check 12 V reaches it (a multimeter is a good $15 tool), check it is a 12 V valve and not 24 V. |
| Valve is on when the board starts | The relay board is inverted the other way. Remove `inverted: true` and test dry. Do not connect water until the on/off is right. |
| Relay chatters | The 5 V supply is weak. Use a better USB adapter and short wires. |
| Water leaks at joints | Turn the water off. Redo the Teflon tape. Do not tighten with a wrench past snug. |

## Make it yours
- Add a flow-meter sensor and show litres used per week.
- Add a rain sensor or the forecast's rain amount ("skip if 5 mm or more is expected").
- A second valve for a second bed.
- Log each watering in the garden notes of your website (a stepping stone to
  Project 8).

**Next:** [Project 8: Readings on your website](P08-readings-on-your-website.md)
