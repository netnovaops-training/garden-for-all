# 14. Give the garden computer a fixed address, and open only two ports

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Give the garden computer a fixed address in your router (a "DHCP reservation"), so it keeps the same address after a restart.
- Step 1: menu `M`, then `2` (Network) shows the IP address and the MAC address.
- Step 2: on the router's page (how to log in: "Before step 2", with the common routers), add a reservation with those two. Restart, run menu `M`, `2` again, and check the IP is the same.
- Step 3, only when you put the site online (after menu `M`, `1`): forward TCP ports 80 and 443 to that address.
- Nothing else. Not 22 (SSH), not 8030, not 3306.
- Step 4: test from a phone with Wi-Fi off at `https://junsgarden.zapto.org` (or the web name you chose).
- New router? Do steps 2 and 3 again.
- Router cannot reserve? The Windows menu finds the garden computer again by its network card when its address changes (More `21`). To go online, give it a fixed address on the computer itself: "If your router cannot reserve an address" below.
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

## Before step 2: log in to your router (a walkthrough)
The router is the box your internet company gave you (or one you bought). Its
settings page lives inside your home: only a device on your home Wi-Fi (or on a
cable) can open it.

1. **Be at home**, on your home Wi-Fi.
2. **Find the router's address.** On the garden computer: menu `M`, then `2`
   ("Router address"). On a Windows PC: open PowerShell, type `ipconfig`, and read
   "Default Gateway". It is usually `192.168.1.1`, `192.168.0.1`, `192.168.50.1`,
   `10.0.0.1` or `192.168.1.254`.
3. **Type it in the browser's address bar** (not in the search box), like
   `http://192.168.1.1`, and press Enter. Many routers have a name for it too (the
   table below). A "Not secure" warning is normal for a router page at home:
   **Advanced**, then **Proceed**. Some routers have only a phone app instead.
4. **Sign in.** The user name and password are usually on a **label on the router**
   (often underneath), or they are the ones set when the router was installed.
   Routers from the internet company are often managed in its app, with your
   account there.
5. **Still the label's password? Change it** to your own and keep it in your
   password manager. Anyone on your Wi-Fi could otherwise change your router.
6. **Forgot it?** The router's own reset button (held about 10 seconds) brings back
   the label's password, but also the label's Wi-Fi name and password: every device
   at home must join again. Use it only as the last step.
7. Then step 2 below. Leave **Remote management** (managing the router from the
   internet) **off**.

Where to sign in on common home routers (menus change; the maker's own page is
the one to trust):

| Router | Address or app | User name and password | The maker's own steps |
|---|---|---|---|
| Asus | `www.asusrouter.com` or `192.168.50.1` | on the router's label (some use `admin`) | <https://www.asus.com/us/support/faq/1005263/> |
| AT&T | `192.168.1.254` | the Device Access Code on the gateway's label | <https://www.att.com/support/article/u-verse-high-speed-internet/KM1206322/> |
| eero | the eero app | your eero account | <https://eero.com/support/articles/how-do-i-set-up-port-forwarding> |
| Google Nest Wifi / Google Wifi | the Google Home app | your Google account | <https://support.google.com/googlehome/> |
| Linksys | `myrouter.local` or `192.168.1.1` | the router password (`admin` until you change it) | <https://support.linksys.com/kb/article/378-en/> |
| Netgear | `routerlogin.net` or `192.168.1.1`, or the Nighthawk app | `admin`, and the password set at setup | <https://kb.netgear.com/980/How-do-I-log-in-to-my-NETGEAR-router> |
| Spectrum | the My Spectrum app | your Spectrum account | <https://www.spectrum.net/support/internet/managing-advanced-settings-my-spectrum-app> |
| TP-Link Deco | the Deco app | your TP-Link account | <https://www.tp-link.com/support/> |
| TP-Link (routers) | `tplinkwifi.net` or `192.168.0.1` | the first time you make one; some older ones use `admin` and `admin` | <https://www.tp-link.com/us/support/faq/87/> |
| Verizon | `mynetworksettings.com` or `192.168.1.1` | `admin`, and the password on the label | <https://www.verizon.com/support/knowledge-base-308501/> |
| Xfinity | the Xfinity app, or `10.0.0.1` | `admin`, and the password you make when you allow the Admin Tool in the app | <https://www.xfinity.com/support/articles/admin-tool-access> |

Not in the list? Search the maker's website for "log in to router" with the model
name from the label. Use the maker's own pages, not sites that list default passwords.

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
| Google Nest Wifi / Google Wifi | Google Home app: Wi-Fi, then network settings, look for "DHCP IP reservations" | <https://support.google.com/googlehome/> (search "DHCP IP reservation") |
| Netgear | Advanced, Setup, LAN Setup, Address Reservation | <https://www.netgear.com/support/> (search "address reservation") |
| Spectrum / Verizon / Xfinity routers | Their app or the router page: "Reserved IP" or "DHCP reservation" | your internet company's help site |
| TP-Link Deco | Deco app: More, Advanced, Address Reservation | <https://www.tp-link.com/support/> (search "Deco address reservation") |

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

## If your router cannot reserve an address
Some routers (often ones from the internet company) have no reservation page, or
it is locked. Then the garden computer's address may change after a restart.

**Your Windows PC keeps finding it anyway.** The guided installer's "Find it for
me" button lists the computers at home with the SSH door open (port 22), so you
can pick it. The Windows menu then remembers the garden computer by its network
card's fixed ID (the MAC address) and its name. Whenever nothing answers at the
saved address, it looks at home for the same network card and saves the new
address, before every job and before the weekly backup copy. To do it by hand:
Windows menu, `M`, then `21` (Find the garden computer again).

**To put the site online you still need a fixed address**, because the router
forwards ports 80 and 443 to one address. Give it to the computer itself:
1. On the router's page, find the range of addresses it hands out (often
   `192.168.1.100` to `192.168.1.200`). Pick an address outside it, for example
   `192.168.1.20`, that no other device uses.
2. On the garden computer: Settings, Network (or Wi-Fi), the gear next to your
   connection, IPv4, **Manual**. Address: the one you picked. Netmask:
   `255.255.255.0`. Gateway and DNS: the router's address (menu `M`, `2` shows it).
   Apply, then switch the connection off and on.
3. Run menu `M`, `2` again: the IP is the new one. Then do step 3 with it.

On an Ubuntu without a screen, the same is done with netplan:
<https://ubuntu.com/server/docs/configuring-networks>

## Read more
- What port forwarding is: <https://en.wikipedia.org/wiki/Port_forwarding>
- Caddy and ports 80/443: <https://caddyserver.com/docs/automatic-https#activation>
