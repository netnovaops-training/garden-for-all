# 14. Give the garden computer a fixed address, and open only two ports

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Give the garden computer a fixed address in your router (a "DHCP reservation"), so it keeps the same address after a restart.
- Step 1: menu `M`, then `2` (Network) shows the IP address and the MAC address.
- Step 2: on the router's page, add a reservation with those two. Restart, run menu `M`, `2` again, and check the IP is the same.
- Step 3, only when you put the site online (after menu `M`, `1`): forward TCP ports 80 and 443 to that address.
- Nothing else. Not 22 (SSH), not 8030, not 3306.
- Step 4: test from a phone with Wi-Fi off at `https://junsgarden.zapto.org` (or the web name you chose).
- New router? Do steps 2 and 3 again.
- Garden inside Windows' own Ubuntu (WSL)? This guide is not for that: see "Windows PC" below.

---

## Why
Your router gives each device at home an address like `192.168.1.50`. Usually
it can hand out a *different* address after a restart. Two things depend on the
garden computer keeping the same one:
- **SSH** from another computer (you type its address).
- **Port forwarding**: the router sends internet visitors to ONE address. If the
  garden computer moves, visitors go nowhere.

A **DHCP reservation** tells the router "always give this computer this
address". It is done in the router, not on the computer, so nothing on the
computer can break it.

**Windows PC running the garden in Windows' own Ubuntu (WSL)?** This guide
does not work for that. Windows' own Ubuntu has its own internal address
inside the PC, and the router and your phones cannot reach it, so forwarding
ports to the PC does not reach the garden. On that Windows PC the garden is for
using on the PC itself. To put the garden online, or to show it to other
devices at home, use a separate garden computer (an Ubuntu laptop or a
Raspberry Pi) and follow this guide for that computer. Microsoft explains how
WSL's networking works, for those who want to learn more (reading only; the
garden does not set this up): <https://learn.microsoft.com/windows/wsl/networking>

## Step 1. Find the computer's address and MAC
On the garden computer: menu `M`, then `2` (Network). It shows:
- **IP address**, for example `192.168.1.50`
- **MAC address**, for example `a4:5e:60:12:34:56`: the network card's fixed ID
- **Router address**, for example `192.168.1.1`

## Step 2. Reserve it in the router
1. Open the router's page in a browser at the router address (or use its phone
   app). The admin password is often on a sticker on the router.
2. Look for one of these names: **DHCP reservation**, **Address reservation**,
   **Static IP**, **Fixed IP**, **IP & MAC binding**.
3. Add the computer: its MAC address and the IP address from step 1. Save.
4. Restart the garden computer. Run menu `M`, `2` again: the IP must be the same.
   Answer **y** when it asks if you reserved it. The checklist will now warn you
   if the address ever changes.

Where it is on common routers (menus change, so the official help page is the
one to trust):

| Router | Where | Official help |
|---|---|---|
| Google Nest Wifi / Google Wifi | Google Home app: Wi-Fi, then network settings, look for "DHCP IP reservations" | <https://support.google.com/googlenest> (search "DHCP IP reservation") |
| TP-Link Deco | Deco app: More, Advanced, Address Reservation | <https://www.tp-link.com/support/> (search "Deco address reservation") |
| Netgear | Advanced, Setup, LAN Setup, Address Reservation | <https://www.netgear.com/support/> (search "address reservation") |
| Spectrum / Verizon / Xfinity routers | Their app or the router page: "Reserved IP" or "DHCP reservation" | your internet company's help site |

## Step 3. Open exactly two ports (only when you put the site online)
Do this after the front door step (menu `M`, `1`), not before.

In the router, find **Port Forwarding** (sometimes "Virtual Servers" or "NAT").
Add two rules, both to the garden computer's reserved address:

| Name | Protocol | Outside port | Inside address | Inside port |
|---|---|---|---|---|
| garden-http | TCP | 80 | your reserved IP | 80 |
| garden-https | TCP | 443 | your reserved IP | 443 |

**Nothing else.** Not 22 (SSH), not 8030, not 3306. Caddy is the only program
the internet may reach. It decides what each visit may see.

## Step 4. Test from outside
Use a phone with **Wi-Fi switched off** (so it uses the mobile network) and open
`https://junsgarden.zapto.org` (or the web name you chose). You should see your garden with a padlock.
On the garden computer, the checklist (menu `5`) checks that the name points
at your home.

## If you change routers
Do steps 2 and 3 again on the new router. The garden computer itself needs nothing.

## Read more
- What port forwarding is: <https://en.wikipedia.org/wiki/Port_forwarding>
- Caddy and ports 80/443: <https://caddyserver.com/docs/automatic-https#activation>
