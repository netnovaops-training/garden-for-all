# 06. Shopping list

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- What to buy for the projects, with rough prices.
- Buy things only when you reach the project that needs them. Projects 4, 6 and 7 need parts.
- The whole ladder costs about $80 to $150, spread over weeks. Prices change: check before you buy.
- For Project 7, the valve must be **normally closed**: water is OFF when the power is off.
- Do not buy anything that says "mains", "110V", "220V" or "AC" for these projects.
- Not sure a part matches the list? Ask Gemini with prompt 6 (Check my plan) before buying.
- Buy from stores that list returns.

---

You need **nothing** to do guides 01 to 03 and Projects 1, 2, 3 and 5 beyond
the laptop. Buy things only when you
reach the project that needs them.

Prices are rough ranges and change. Check the current price before you buy.

| For | Part | Rough price | Notes |
|---|---|---|---|
| Guide 01 | USB stick, 16 GB or larger | $5 to $10 | You may have one. It gets erased. |
| Guide 01, a Pi instead of a laptop | **Raspberry Pi 5**, 8 GB of memory | Check the official shops below | The 4 GB one works too (it was tested), 8 GB gives more room. |
| Same | The **official Raspberry Pi 27 W USB-C power supply** | Same | A weaker phone charger makes a Pi 5 unstable. |
| Same | The official **Active Cooler** (a fan), or the official **Case** with its fan | Same | Needed: without a fan, the Pi slowed itself down at 84.5 °C while building the garden (guide 01). |
| Same | An **SSD** with the **M.2 HAT+**, or a USB 3 SSD; or a 128 GB+ microSD card | Same | An SSD is faster and lasts much longer than a memory card (the garden writes every day). |
| Project 4 | A Govee light that supports **LAN control** | $15 to $40 | Check the model's page or the Govee app for "LAN Control" before buying. Older models only work through the cloud. |
| Project 6 | **ESP32 dev board** (ESP32-WROOM-32, with USB) | $6 to $12 | Buy 2; one is a spare for mistakes. |
| Project 6 | **Capacitive** soil moisture sensor v1.2 (not "resistive") | $2 to $4 each | Capacitive ones do not rust. Buy 2 to 4. |
| Project 6 | USB cable that carries data (not charge-only) | $5 | If the laptop does not see the board, the cable is the usual cause. |
| Project 6 | DHT22 or BME280 air sensor (optional) | $4 to $10 | BME280 gives temperature, humidity and pressure. |
| Project 7 | 12 V **normally-closed** water solenoid valve, 1/2 inch, garden-hose type, **or** a small 5 V/12 V submersible pump with tubing | $10 to $25 | Normally closed means water is OFF when power is off. This is the safe one. |
| Project 7 | **Relay module**, 1 channel, 5 V, opto-isolated | $3 to $6 | It is the switch the ESP32 uses. |
| Project 7 | 12 V DC power adapter, 2 A, with a screw-terminal adapter | $8 to $12 | Low-voltage power for the valve. |
| Project 7 | **Flyback diode** 1N4007 | $1 | Protects the relay from the valve's electrical kick. |
| Project 7 | Waterproof box (IP65), hose fittings, Teflon tape, drip tubing | $15 to $30 | Electronics and water must be kept apart. |
| Project 7 | 2 A fuse and holder for the 12 V line | $3 | A fuse stops a fire if something shorts. |
| Any | Breadboard, jumper wires, small screwdriver set | $10 to $15 | A starter kit has all of this. |

Raspberry Pi's official products and their list of approved shops:
<https://www.raspberrypi.com/products/>.

**Total for the whole ladder:** about $80 to $150, spread over weeks.

## Rules for buying
- Buy from stores that list returns.
- Do not buy anything that says "mains", "110V", "220V" or "AC" for these projects.
- If a part does not match this list, ask Gemini with prompt 6 (Check my plan)
  before buying.
