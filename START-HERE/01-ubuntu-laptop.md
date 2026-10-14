# 01. Turn the old Windows 7 laptop into an Ubuntu computer (from a USB stick)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- This is the one thing the garden does not do for you: putting Ubuntu on the computer. About 2 to 3 hours, free.
- Copy your files off the laptop first: **this erases Windows**.
- On a Windows computer, download Ubuntu 24.04 LTS (`.iso`) and **Rufus** (rufus.ie). Rufus puts Ubuntu on a 16 GB or larger USB stick (Part B).
- Start the laptop from the stick (tap the boot key, Part C), choose **Erase disk and install Ubuntu**.
- Choose a **user name** (for example `jun`), a **computer name** (for example `garden-lab`) and a **password**. Write them down. That password is also the one Ubuntu asks for when you change something (`sudo`): there is no separate "root password".
- Write down the laptop's **IP address** (Settings, Wi-Fi, the gear next to your network).
- Prefer not to change a laptop? A **Raspberry Pi 5** kit works too (guide 06).

---

## What you will build
A laptop that used to run Windows 7 and now runs **Ubuntu Desktop 24.04 LTS**
(free Linux). It becomes your home-lab computer for every other guide.

**Or a Raspberry Pi 5 instead.** A small new computer, made for this kind of
job, with no old laptop to change. A Pi 5 with 8 GB of memory, its official power
supply and a 128 GB or bigger storage card or SSD (guide 06, shopping list). Its
own Imager program puts Raspberry Pi OS (64-bit) on it in about 30 minutes:
<https://www.raspberrypi.com/software/>. Next: [00-READ-ME-FIRST.html](../00-READ-ME-FIRST.html),
step 1 (download the garden).

> **Good to know about a Pi (measured on a real Raspberry Pi 5, 4 GB, on 2026-10-13).**
> - **It works.** The garden's build-and-start part of the install finished in
>   **8 minutes**, the checklist ended with **0 fail**, and the website showed all
>   485 plants. The whole install, with the system updates first, takes longer:
>   plan on up to an hour. A laptop or PC does the same work faster.
> - **It gets hot while it builds.** Without a fan it reached **84.5 °C** and slowed
>   itself down to cool off (the Pi 5 does this between 80 and 85 °C), then cooled
>   back to 60 °C once the build was done. It is not broken, only slower. Use
>   Raspberry Pi's official **Active Cooler** or its **Case** with the fan (guide 06).
>   Raspberry Pi's page on this: <https://www.raspberrypi.com/documentation/computers/raspberry-pi.html>
>   (the "Cooling" and "Frequency management and thermal control" parts).
> - **A memory card wears out.** The garden writes to its database and logs every
>   day, and SD cards are not made for that. An **SSD** (with Raspberry Pi's M.2
>   HAT+, or on a USB 3 port) is faster and lasts much longer. Either way, keep the
>   nightly backups going to a second place (guide 13).

Why this is a good idea: Windows 7 stopped getting safety updates in 2020, so it
is not safe to put on the internet. Ubuntu LTS gets free safety updates for
years.

## Time and cost
2 to 3 hours. Free. You need a USB stick of 16 GB or larger (it will be erased; Ubuntu's own guide asks for at least 12 GB, and 16 GB is the common size).
A cheap SSD (about $25 to $40) makes an old laptop feel much faster. It is
optional.

## What you need first
- The old laptop and its charger (keep it plugged in the whole time).
- A **second computer with internet** to make the install USB. If you only have
  the Windows 7 laptop, see Part B, option 2.
- A USB stick, 16 GB or larger.
- A cable to the router if the laptop has a network port (more reliable than
  Wi-Fi during install).

> **STOP SIGN: this erases Windows 7.** The steps below delete everything on
> the laptop. Copy your files off it first (Part A, step 3). You cannot undo it.

## Steps

### Part A. Check the laptop is able to do this
1. **Is the processor 64-bit?** Ubuntu 24.04 and Docker only work on 64-bit.
   On the laptop in Windows 7: click **Start**, right-click **Computer**,
   choose **Properties**. Find **System type**.
   - `64-bit Operating System` : good, continue.
   - `32-bit Operating System` : the processor *may* still be 64-bit. Download
     **CPU-Z** (cpuid.com) and look at **Instructions**: if it lists `EM64T`
     or `AMD64` or `x86-64`, it is 64-bit and you can continue. If it does not,
     this laptop cannot run Docker. Use it for something else (a simple
     Linux learning computer) and find another laptop for the garden.
2. **How much memory (RAM) and disk?** In the same Properties window look at
   **Installed memory (RAM)**. Then open **Computer** and look at the size of
   the C: drive.

   | Memory | What to do |
   |---|---|
   | 8 GB or more | Ubuntu Desktop is fine |
   | 4 GB | Works, but install **Xubuntu 24.04** instead (a lighter look, same everything else, from xubuntu.org). The steps below are the same. |
   | 2 GB or less | Too small to build the website. Add memory first (laptop memory is cheap), then continue. |

   You need about 60 GB of free disk space on the Ubuntu laptop for Docker,
   the garden and its nightly backups. Erasing Windows frees the whole disk,
   so look at the C: drive's total size: Ubuntu itself needs about 25 GB, so a
   disk of about 90 GB or more is enough.
3. **Save anything you want to keep.** Copy your pictures, documents and
   browser bookmarks to a different USB stick or a different computer. Do this
   now. Write down your Wi-Fi password.
4. **Windows 7 only:** if you want to keep the Windows product key sticker, take
   a photo of the sticker on the bottom of the laptop. You do not need it for
   Ubuntu.

### Part B. Make the install USB
**Option 1 (recommended): on a newer Windows computer, with Rufus.**
Rufus is a small, free, well-known program that puts an `.iso` on a USB stick
(<https://rufus.ie>, by its maker Pete Batard). Nothing to install: it just runs.
5. Open <https://releases.ubuntu.com/24.04/> and download **Ubuntu 24.04 LTS**,
   the "Desktop image" (a file ending in `.iso`, about 6 GB). For a 4 GB laptop
   download Xubuntu from <https://xubuntu.org/download> instead.
   *Why 24.04, when Ubuntu's main page now offers 26.04?* The garden was built and
   tested on 24.04, which gets free safety updates until 2029. 26.04 is expected
   to work too (Docker supports it), but 24.04 is the tested one.
6. Download **Rufus** from <https://rufus.ie> (the top link, "Rufus 4.x").
   Open it. Windows asks "allow this app to make changes?": **Yes** (it needs
   that to write the stick).
7. Put the USB stick in. In Rufus: **Device** = your USB stick (check the size,
   so you do not pick the wrong drive). **Boot selection**: click **SELECT** and
   pick the `.iso`. Leave everything else as Rufus chose. **START**. If it asks
   about **ISO image mode** or **DD image mode**, choose **ISO**. It warns the
   stick will be erased: **OK**. Wait for **READY** (5 to 15 minutes).

*On a Mac or Linux computer instead:* **balenaEtcher** (<https://etcher.balena.io>)
does the same: **Flash from file**, pick the `.iso`, **Select target**, pick the
stick, **Flash!**

**Option 2: only the Windows 7 laptop is available.**
5. Etcher does not run on Windows 7. Use **Rufus version 3.22**, the last
   version that supports Windows 7. Get it from the official download page
   (<https://rufus.ie>, look under "Other versions" or "Previous versions").
6. Download the Ubuntu or Xubuntu `.iso` (see option 1, step 5).
7. In Rufus choose your USB stick, click **SELECT** and pick the `.iso`, leave
   the other settings as Rufus chose (it picks MBR for older BIOS laptops), then
   **START**. If it asks to download extra files (Syslinux), say Yes. If it asks
   for **ISO image mode** vs **DD mode**, choose **ISO** first. If the laptop
   does not boot from the stick later, redo it with **DD**.

### Part C. Start the laptop from the USB stick
8. Plug the stick into the laptop. Turn the laptop **off** (not restart).
9. Turn it on and **tap the boot menu key** straight away, again and again.
   It depends on the brand. The screen often shows it for one second.

   | Brand | Boot menu key |
   |---|---|
   | Dell | F12 |
   | HP | Esc, then F9 |
   | Lenovo | F12 (or the Novo button) |
   | Acer | F12 |
   | ASUS | Esc or F8 |
   | Toshiba | F12 |
   | Samsung | Esc or F10 |

10. In the menu pick the USB stick. It may be called `USB HDD`, `USB-HDD`,
    `Removable Device` or the name of your stick. If there are two entries,
    one with `UEFI:` in front, either can work. Try the one without `UEFI`
    first on a Windows 7 laptop (it is likely a "legacy" BIOS laptop).
    *You should see:* a screen that says **Try or Install Ubuntu**.

If the laptop skips the USB and starts Windows: turn it off and try again, or
enter the BIOS setup (usually `F2` or `Del`) and move **USB** to the top of the
**Boot** order.

### Part D. Install
These are the screens of the Ubuntu 24.04 installer, in order, as Ubuntu's
own tutorial shows them: <https://ubuntu.com/tutorials/install-ubuntu-desktop>.
You move on from each screen with its **Next** button.
11. In the black start menu, choose **Try or Install Ubuntu** and press Enter.
    Wait up to 5 minutes on an old laptop.
12. The first screens, one at a time:
    - **Language**: choose yours, **Next**.
    - **Accessibility**: nothing is needed here, **Next**.
    - **Keyboard layout**: the one on your keyboard, **Next**.
    - **Internet** ("Connect to your network"): with a cable, choose the wired
      connection; otherwise choose your Wi-Fi and type its password. **Next**.
      (Ubuntu uses the internet to get updates and drivers while it installs.)
    - **Try or install**: choose **Install Ubuntu**, **Next**.
13. **Type of installation**: choose **Interactive installation**, **Next**.
    Then **Applications**: choose **Default selection** (the lighter one;
    Extended selection adds office programs and more), **Next**.
14. **Third-party software**: tick the box to install third-party software for
    graphics and Wi-Fi hardware. This matters because old Wi-Fi chips often
    need it. The box for extra media formats is your choice. **Next**.
15. **Disk setup**: choose **Erase disk and install Ubuntu** (see the stop sign).
    Leave the encryption choice off: with it, the laptop asks for a key every
    time it starts, so it cannot start by itself after a power cut. **Next**.
16. **Create your account**. These are three names (explained in
    [START-HERE/00-READ-ME-FIRST.md, section "Three names for one computer"](00-READ-ME-FIRST.md#three-names-for-one-computer-people-mix-these-up-all-the-time)):
    - **Your name**: anything, for example `Jun`. Only shown on the login screen.
    - **Computer name** (also called host name): `garden-lab`. Other computers
      at home see the laptop by this name (`garden-lab.local`).
    - **Username**: lowercase, no spaces, easy to type, for example `jun`. You
      type it to log in, and SSH uses it (guide 20).
    - **Password** (typed twice): long and one you will remember. Save it in a
      password manager. **Ubuntu has no separate "root password".** When Ubuntu
      (or the garden menu) asks for a password to change something, it is this
      one: `sudo` means "do this as the administrator", and your account is allowed to.
    - Leave **Require my password to log in** switched **on** (do not choose to
      log in automatically). **Next**.
17. **Time zone**: click your place on the map, **Next**. Then the **Review**
    page shows your choices: check it says the disk will be erased, then click
    **Install**. Wait. An old hard disk can take 30 to 60 minutes. Do not close
    the lid and do not unplug the charger.
18. When it says **Restart Now**, click it. Take the USB stick out when asked
    and press Enter.

*You should see:* a login screen with your name. Log in.

### Part E. First updates (this is the safety part)
19. Press the Windows key (Super), type `terminal`, press Enter.
20. Update everything. Type this, press Enter, and type your password
    (nothing shows as you type, which is normal):

```
sudo apt update && sudo apt upgrade -y
```

   *You should see:* lots of text, then the `$` line again. It can take 10 to 40
   minutes the first time. Then restart: `sudo reboot`.

### Part F. Tools and your address
(The garden install, menu 1, also does the SSH server, the firewall and the swap
of Part G for you. It switches the firewall on with an SSH rule for your home
network only, so do not set up the firewall by hand here.)
21. After the restart, in the terminal:

```
sudo apt install -y git curl unzip htop openssh-server
```

   *You should see:* lines about the programs being installed, then the `$`
   line again.
22. Find the laptop's address on your home network:

```
hostname -I
```

   *You should see:* something like `192.168.1.50`. Write it down.

   **Or without the terminal:** open **Settings**, then **Wi-Fi** (or **Network**
   with a cable), and click the **gear** next to your connection. The **Details**
   page shows **IPv4 Address**: that is it. The computer name is in **Settings,
   System, About** ("Device Name").

### Part G. Make an old laptop work well as a server
23. **Add swap** (extra "memory" on the disk). The garden website's first build
    needs more memory than a small laptop has, and swap stops it from stopping
    part-way. This adds 4 GB:

```
sudo fallocate -l 4G /swapfile2
sudo chmod 600 /swapfile2
sudo mkswap /swapfile2
sudo swapon /swapfile2
echo '/swapfile2 none swap sw 0 0' | sudo tee -a /etc/fstab
free -h
```

   *You should see:* a `Swap:` line with 4 GB or more. Expect the first build of
   the website to take 15 to 45 minutes (up to an hour on a very old laptop or a
   Pi). That is normal.
24. **Keep it awake with the lid closed:**

```
sudo nano /etc/systemd/logind.conf
```

   Find `#HandleLidSwitch=suspend`. Remove the `#` and change it to
   `HandleLidSwitch=ignore`. Do the same for `#HandleLidSwitchExternalPower`.
   Save with `Ctrl+O`, `Enter`, leave with `Ctrl+X`. Then restart the laptop
   (`sudo reboot`, or Restart from the power menu), so the change takes effect.
   (Restarting the whole laptop is safer than restarting only the login
   service, which can log you out in the middle of your work.)
25. **Settings, Power:** set **Automatic Suspend** to **Off**.
26. **Heat:** an old laptop with the lid shut gets very hot. Leave the lid open
    a little or prop the back up so air can flow. Watch for fans running hard
    all day.
27. **Battery:** if the battery is old and swollen (the laptop sits crooked, or
    the trackpad pushes up), take it out and run on the charger only. A swollen
    battery is a fire risk.

## How you know it worked
- You can log in and open a terminal.
- `sudo apt update` ends with "All packages are up to date" (or lists updates).
- `free -h` shows swap.
- You wrote down the laptop's address.

## If something goes differently
| What you see | What to do |
|---|---|
| Laptop starts Windows, not the USB | Try the boot key again, or set USB first in the BIOS. |
| USB is not in the boot menu | Remake the stick with Rufus **DD mode**, or try a different USB port (use a USB 2.0 port if it has one). |
| `This kernel requires an x86-64 CPU` | The processor is 32-bit. See Part A, step 1. |
| Screen is black after "Install" | At the boot screen press `e`, add `nomodeset` at the end of the line starting `linux`, press `F10`. After install ask Gemini "how to install the right graphics driver on Ubuntu for my laptop's graphics chip" and paste the "VGA" line from `lspci`. |
| No Wi-Fi after install | Plug in a cable, run `sudo apt update`, then search the menu for **Additional Drivers** and install the Broadcom or other driver it offers. |
| Very slow | Use Xubuntu, close the browser when idle, and consider an SSD. Ask Gemini: "How do I make Ubuntu faster on an old laptop with 4 GB RAM?" |
| Installer freezes at the end | Wait 10 minutes (old disks are slow). Only then hold the power button. If it will not boot afterwards, run the install again. |

## Make it yours
- Pick a wallpaper that is a photo from your own garden.
- Install a password manager on it (Bitwarden or Proton Pass have Linux apps).
- Learn three commands today: `ls` (list files), `cd` (go to a folder),
  `pwd` (where am I?).

**Next:** [00-READ-ME-FIRST.html](../00-READ-ME-FIRST.html), step 1 (download the garden).
(The install, menu 1, installs Docker for you; [02 Docker](02-docker.md) only explains it.)
