# 20. Work on the garden computer from your Windows PC (SSH and Remote Desktop)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- **SSH** lets your Windows PC type commands on the garden computer, through a
  private, encrypted connection. Windows 10 and 11 already have it.
- **First time?** Open PowerShell, check with `ssh -V`, then type
  `ssh jun@192.168.1.50` (your user name and IP address). `exit` brings you back
  to Windows. Step by step below.
- The setup wizard (or the Windows menu, `T`) sets it up for you when the
  garden goes on another computer, including a **key**, so you do not type a password every time.
- A small `config` file gives the computer a short name (`ssh garden`) and keeps
  the connection open while you step away.
- **Copy files** between the Linux computer and Windows with the free program **WinSCP** (drag and drop, step by step below).
- **Remote Desktop** shows the garden computer's screen on your PC (for example
  from upstairs): switch it on in Ubuntu's Settings, then use Windows' own
  **Remote Desktop Connection**.
- **From a phone:** the free app **Termius** opens the garden computer's terminal over SSH (on your home Wi-Fi).
- Every connection asks for four things: **host name (IP address), port, user name, password**. The first section shows where to find each.
- Both stay on your **home network only**. Never forward port 22 or 3389 in the router. From anywhere, only you: **Tailscale** (optional, highly recommended; garden menu M, 17).
- Windows menu, More, 18 puts two shortcuts on your desktop: **Garden terminal**
  and **Garden screen**.

---

## Before you start: the four things every connection asks for

Every program in this guide (PowerShell, WinSCP, Termius, Remote Desktop) asks
for the same four things. Find them once and write them down.

| It asks for | What it is | Where to find it |
|---|---|---|
| **Host name** or **IP address** (the same box) | Which computer to connect to. At home, use the garden computer's IP address, for example `192.168.1.50` | The top line of the garden menu (picture below), or type `hostname -I` in its terminal (the first number) |
| **Port** | The "door number" for this kind of connection. SSH, WinSCP and Termius use **22**. Remote Desktop uses **3389** | Already filled in. Leave it as it is |
| **User name** | Your user name on the garden computer (Ubuntu), for example `jun` | The start of the terminal prompt (`jun@garden-lab`), or type `whoami` |
| **Password** | Your **Ubuntu** password: the one you chose when Ubuntu was installed, and that `sudo` asks for | Your password manager. It is not your Gmail password and not the website's password |

![Where the IP address is: the top line of the garden menu](images/where-ip-address.png)

![Where the user name is: the start of the prompt, or type whoami](images/where-user-name.png)

**Your garden has three different passwords.** They do different jobs, so keep
all three in your password manager:

| Password | Used for |
|---|---|
| **Ubuntu** password | Logging in to the garden computer itself: its screen, SSH, WinSCP, Termius, `sudo` |
| **Website** password | Logging in to your garden website and its sign-in page (menu `U`, then `5` sets it) |
| **Gmail app password** | Only for the garden to send email (guide 03, Part C). Not your Gmail password |

**Can I type my web name instead of the IP address?** Not for these. Your web
name (for example `junsgarden.zapto.org`) leads to your home's router from the
internet, and the router only lets the website through (ports 80 and 443). Ports
22 and 3389 stay closed there on purpose, so nobody on the internet can even try
to log in to your computer. At home, use the IP address (or the computer's name
with `.local`, for example `garden-lab.local`). To reach the computer when you
are away, see **Away from home** near the end of this guide.

## What SSH is
SSH ("secure shell") is a way for one computer to work on another, by typing
commands, as if you sat in front of it. Everything that travels between the two
is **encrypted**: someone watching your network sees only scrambled data, not
your commands, files or passwords. It is the standard way servers are looked
after, all over the world. Official page: <https://www.openssh.com/>

Windows 10 and 11 include SSH already (Microsoft's OpenSSH):
<https://learn.microsoft.com/windows-server/administration/openssh/openssh_overview>

## PowerShell basics, then SSH into the Linux computer
**PowerShell** is Windows' own terminal: you type a command, press Enter, and
read the answer. These steps take you from PowerShell to the Linux computer and
back.

**Step 1. Open PowerShell.** Start, type `powershell`, and open **Windows
PowerShell**. (Or open **Terminal**: Windows 11 has it, and it opens PowerShell.)
You see a line like this:

```
PS C:\Users\Jun>
```

This is the **prompt**. `PS` means PowerShell. `C:\Users\Jun` is the folder you
are in (your Windows home folder). After `>` you type.

**Step 2. A few basics.** Try them. None of them changes anything.

| Type or press | It does |
|---|---|
| `pwd` | Shows where you are |
| `ls` or `dir` | Lists what is in this folder |
| `cd Desktop` | Goes into the Desktop folder |
| `cd ..` | Goes up one folder |
| `cls` | Clears the screen |
| **Up arrow** | Brings back the last command |
| **Tab** | Finishes a name for you |
| **Ctrl+C** | Stops what is running |

Microsoft's page about these short commands:
<https://learn.microsoft.com/powershell/scripting/learn/shell/using-aliases>

**Step 3. Check that SSH is there.**

```
ssh -V
```

It prints OpenSSH's version, for example `OpenSSH_for_Windows_9.5p2, LibreSSL 3.8.2`.
Windows 10 and 11 have it built in (Microsoft's page is linked above).

**Step 4. Connect.** Type `ssh`, your user name, `@`, and the IP address (see
"the four things" at the top of this guide):

```
ssh jun@192.168.1.50
```

At home, `ssh jun@garden-lab.local` may also work. With Tailscale (end of this
guide): `ssh jun@garden-lab`.

- The first time it asks "Are you sure you want to continue connecting": type
  `yes` and press Enter. It remembers the computer, so nobody can pretend to be
  it later.
- Then type your Ubuntu password. **Nothing shows while you type.** Press Enter.

**Step 5. You are on the Linux computer.** The prompt changes:

```
jun@garden-lab:~$
```

Everything you type now runs **there**, not on Windows. Guide 22 has the
commands. To open the garden menu:

```
cd ~/junsgarden-for-jun
bash garden.sh
```

**Step 6. Leave.** Type `exit`. The prompt is `PS C:\Users\Jun>` again: you are
back on your Windows PC.

**If it does not work:**

| You see | What to do |
|---|---|
| `Connection timed out` | The computer is off, the IP address is wrong, or you are not on the home Wi-Fi. Check each one. |
| `Connection refused` | The computer is on, but its SSH server is not running. On that computer, type `sudo systemctl status ssh`. |
| `Permission denied` | Wrong user name or password. Small and capital letters matter: `jun` is not `Jun`. |
| `WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!` | The computer looks different from last time. If you reinstalled it, type `ssh-keygen -R 192.168.1.50` and connect again. If you did not, stop and ask someone you trust. |

## Keys: no password every time
A **key** is a pair of files made on your PC: a **private** half that never leaves
your PC, and a **public** half you give to the garden computer. When you connect,
the two halves prove who you are without sending any password.

- **ed25519** is the modern kind of key, short and strong, and what OpenSSH
  recommends. The garden menu uses it.
- **RSA** is the older kind. It still works (use 4096 bits) if some old program
  insists on it; otherwise choose ed25519.

**The setup wizard does this for you** when the garden goes on another computer
(its "Where it runs" page; in the Windows menu, `T` then `1`): it makes the key if there is none
(`C:\Users\YOU\.ssh\id_ed25519`) and puts the public half on the garden computer.

**By hand, in three steps** (in PowerShell):

**1. Make the key** (once per PC). Press Enter at each question (or choose a
passphrase: a password for the key itself, even safer).

```
ssh-keygen -t ed25519
```

**2. Give the public half to the garden computer** (once). It asks your Ubuntu
password one last time.

```
type $env:USERPROFILE\.ssh\id_ed25519.pub | ssh jun@192.168.1.50 "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"
```

**3. Connect.** `ssh jun@192.168.1.50` now lets you straight in.

**The same key works for the Pi and any other Linux computer.** Do step 2 once
for each one, with that computer's user name and address.

See or remove the computers that have a key: garden menu `U`, then `7` (on the
computer that runs the garden).

**On your phone (Termius):**

1. **Make a key.** Open the **Keychain**, tap **+**, then **Generate Key** (on
   Android: **New SSH Key**, then **Generate new SSH Key**). Choose **Ed25519**
   and save.
2. **Send it to the garden computer.** Tap the key (on Android, then
   **Export to…**), pick the garden computer, and tap **Export**.
3. **Use it.** If the host does not use the key yet, edit the host and choose
   the key there.

No Export button? Copy the key's public text in Termius. Then, on the garden
computer, type `nano ~/.ssh/authorized_keys`, paste it on a new line (one line
per key), and press **Ctrl+O**, **Enter**, **Ctrl+X**.

Termius's page: <https://docs.termius.com/keychain/ssh-keys-and-certificates>

The private half never leaves your phone or PC. If a phone or PC is lost,
remove its key: garden menu `U`, then `7`.

## Keep the connection alive (and a short name)
A small file on your PC, `C:\Users\YOU\.ssh\config`, gives the garden computer a
short name and keeps the connection open while you step away.

**1. Make the file and open it** (in PowerShell):

```
New-Item -ItemType File $HOME\.ssh\config
notepad $HOME\.ssh\config
```

The first line makes an empty file with no extension. If it says the file
already exists, that is fine. (If it says the folder is missing, make a key
first: step 1 above makes the `.ssh` folder.) The second line opens it in Notepad.

**2. Type this** (with your own IP address and user name), then save with
**Ctrl+S** and close Notepad:

```
Host garden
    HostName 192.168.1.50
    User jun
    ServerAliveInterval 60
```

**3. Connect** with the short name:

```
ssh garden
```

`ServerAliveInterval 60` sends a small "still here" every 60 seconds, so the
connection does not close while you are away from the keyboard.

For a Pi or another Linux computer, add a second block below the first, with its
own name and address (for example `Host pi` and `HostName 192.168.1.60`).

The official manual: <https://man.openbsd.org/ssh_config>

**In Termius:** use the host's Keep-Alive setting, if your Termius version shows
it. Termius's documents: <https://docs.termius.com/>.

## Copy files, securely
The same encrypted connection carries files.

- **One file, from PowerShell:**
  `scp C:\Users\YOU\Pictures\rose.jpg jun@192.168.1.50:~/`
- **A whole folder:** add `-r`:
  `scp -r C:\Users\YOU\Pictures\garden jun@192.168.1.50:~/`
- **Back from the garden computer:**
  `scp jun@192.168.1.50:~/junsgarden-for-jun/backups/NAME.tar C:\Users\YOU\Desktop\`
- **With windows and drag-and-drop:** WinSCP, the next section.

The garden menu's "Copy the newest backup to THIS PC" (More, 10) uses exactly this.

## Copy files between Linux and Windows: WinSCP, step by step

**WinSCP** is a free, well-known Windows program that shows your Windows PC on
the left and the garden computer on the right. You drag files from one side to
the other. It uses the same encrypted SSH connection, so nothing new is opened.

**Step 1. Install it (once).**
Open the official download page: <https://winscp.net/eng/download.php>. Click the
green **Download WinSCP** button, open the file, and click **Next** through the
installer (the suggested choices are fine).

**Step 2. Open it.**
Start menu, type `WinSCP`, open it. The **Login** window appears.

**Step 3. Fill in the Login window** (the four things from the start of this guide):

| Box | What to choose or type |
|---|---|
| **File protocol** | `SFTP` (the encrypted one; it is usually already chosen) |
| **Host name** | The garden computer's IP address, for example `192.168.1.50` |
| **Port number** | `22` (already filled in) |
| **User name** | Your Ubuntu user name, for example `jun` |
| **Password** | Your Ubuntu password |

**Step 4. Save it, so next time is one click.**
Click **Save**, call it `Garden computer`, and click **OK**. (Leave "Save password"
unticked unless the PC is only yours.)

**Step 5. Click Login.**
The first time only, WinSCP shows the garden computer's **key fingerprint** and
asks whether you trust it. On your home network, connecting to your own computer:
click **Yes**. (This is how it makes sure it is the same computer every time.)

**Step 6. Copy files.**
- **Left side:** your Windows PC. **Right side:** the garden computer (it opens
  in your home folder, `/home/jun`).
- **Drag** a file or folder from one side to the other. That copies it.
- Your garden is in `junsgarden-for-jun` on the right; its backups are in
  `junsgarden-for-jun/backups`.

**Step 7. Close it** when you are done (the X, or **Session**, **Disconnect**).

WinSCP's own guide to this window, with pictures:
<https://winscp.net/eng/docs/guide_connect>.

**Good habits:** copy, do not move, files out of `junsgarden-for-jun` (the garden
needs them), and never edit `.env` there by hand (menu `8` does it safely).

## Remote Desktop: see and use the garden computer's screen
To use the garden laptop's desktop from another room, with its windows and mouse:

**On the garden computer (Ubuntu 24.04), once:**
1. **Settings**, **System**, **Remote Desktop**.
2. Choose **Remote Login** (works when nobody is logged in at the laptop: log
   out on the laptop first, or use Desktop Sharing) or **Desktop Sharing**
   (shares the screen of the person logged in). Switch it on.
3. Set the **user name** and **password** it shows under "Login Details" (these
   are only for Remote Desktop; write them down).
4. Let only your home network in (one line in a terminal; the menu's firewall
   step already blocks everything else):
   ```
   sudo ufw allow from 192.168.1.0/24 to any port 3389 proto tcp comment 'remote desktop, home only'
   ```
   (Use your own network's first three numbers: the start of the garden
   computer's IP address, then `.0/24`.)

**On your Windows PC:** Start, type **Remote Desktop Connection**, Computer = the
garden computer's IP address, **Connect**, then the login details from step 3.

The help page (Ubuntu's desktop is GNOME): <https://help.gnome.org/gnome-help/sharing-desktop.html>.
On a Raspberry Pi, use **Raspberry Pi Connect** instead:
<https://www.raspberrypi.com/software/connect/>.

To switch it off: Settings, System, Remote Desktop, off. And
`sudo ufw delete allow from 192.168.1.0/24 to any port 3389 proto tcp`.

## A Linux terminal on your phone (SSH)
Your phone can open the garden computer's terminal, the same one as in guide 22,
from anywhere in your home on your Wi-Fi. You can run the garden menu from it,
check on things, or simply play with Linux from the sofa.

**1. Install a terminal app.** **Termius** is free for this, well known, and the
same on iPhone and Android (the free plan is enough; you can ignore the offers):
- iPhone and iPad: <https://apps.apple.com/app/termius-ssh-shell-console-terminal/id549039908>
- Android: <https://play.google.com/store/apps/details?id=com.server.auditor.ssh.client>
- The maker's own site: <https://termius.com>

**2. Add the garden computer** (Termius calls a computer a "host"). Tap **+** and
add a new host:

| Box | What to type |
|---|---|
| Address (or Hostname) | The garden computer's IP address, for example `192.168.1.50` (the garden menu shows it at the top) |
| Port | `22` (usually already filled in) |
| Username | Your Ubuntu user name, for example `jun` |
| Password | Your Ubuntu password (the one `sudo` asks for) |

Save it, then tap it to connect.

**3. The first time only,** the app shows the computer's **fingerprint** and asks
if you trust it. On your home Wi-Fi, connecting to your own garden computer: yes.
(The fingerprint is how SSH makes sure it is the same computer every time.)

**4. You are in.** You see the same prompt as on the computer itself:

```
jun@garden-lab:~$
```

Now type what you would type there. For the garden menu:

```
cd ~/junsgarden-for-jun
bash garden.sh
```

**Typing on a small screen:** the app adds a row of extra keys above the phone's
keyboard (**Ctrl**, **Tab**, **Esc** and the arrows). **Tab** finishes a long name
for you, **Ctrl** then **C** stops what is running, and `exit` closes the
connection when you are done.

It works on your home Wi-Fi only, on purpose: port 22 is never opened to the
internet. (Not tried on a phone in the package's tests. If it does not connect:
is the phone on the same Wi-Fi, and is the address right? Menu `5`, the checklist,
shows "22 SSH: listening".)

**Want to see the whole desktop instead?** Microsoft's free **Windows App**
(iPhone: <https://apps.apple.com/us/app/windows-app-mobile/id714464092>, Android:
<https://play.google.com/store/apps/details?id=com.microsoft.rdc.androidx>) opens
the Ubuntu desktop over Remote Desktop, once it is switched on (the steps above).
Tap **+**, then **PC**, type the IP address, then the Remote Desktop login.
Microsoft's page: <https://learn.microsoft.com/windows-app/get-started-connect-devices-desktops-apps>.

## Two shortcuts on your desktop
The Windows garden menu, **More, 18**, makes:
- **Garden terminal**: opens SSH to the garden computer (or Windows' own Ubuntu).
- **Garden screen**: opens Remote Desktop Connection to it (for a separate garden
  computer, once Remote Desktop is on).

## Away from home (optional, highly recommended): Tailscale

Everything above works on your home Wi-Fi only, on purpose. **Tailscale** lets you
reach the garden computer from anywhere (at work, on holiday, on your phone's
mobile data), and only you.

**Why it is worth it:**
- **Only your devices get in.** Tailscale links your own devices (the garden
  computer, your PC, your phone) into a small private network. Nobody else can
  join it, or even see that the computer is there.
- **No doors opened in the router.** Ports 22 and 3389 stay closed; Tailscale
  needs no port forwarding at all.
- **One name that works everywhere.** The garden computer keeps one name (for
  example `garden-lab`) at home and away. You use it in Termius, WinSCP and
  Remote Desktop instead of the IP address.
- **Free for personal use** (up to 6 users, any number of devices):
  <https://tailscale.com/pricing>.
- **Encrypted all the way**, and you can see and remove your devices at any time:
  <https://login.tailscale.com/admin/machines>.

**How easy it is: three steps, about 10 minutes, nothing to type but a "y".**

**Step 1. Your account and your devices** (in a browser and on your phone).
Open <https://tailscale.com/download>, sign up by signing in with an account you
already have (for example Google), and install the Tailscale app on your **phone**
(App Store or Google Play) and your **PC**. Sign in on each with the same account.

**Step 2. The garden computer joins** (the garden menu does it).
Garden menu: `M`, then `17` (on the Windows menu: `M`, then `19`). Press `y`. It
installs Tailscale with Tailscale's own installer, shows every command, and then
shows a **sign-in link**. Open that link in a browser (on any device), sign in with
the same account, and the garden computer joins. The menu then lets SSH and Remote
Desktop in through Tailscale only, and tells you the name to use.

![The menu's Tailscale step: press y, then open the link it shows and sign in](images/screen-24-tailscale.png)

**Step 3. Use it.** In Termius, WinSCP or Remote Desktop, type the garden
computer's name (for example `garden-lab`) as the **Host name** instead of the IP
address. Everything else stays the same. With the Tailscale app switched on, it
works at home and away.

**To switch it off:** the same menu item again (`M`, `17`). It disconnects and
closes the firewall line.

Tailscale's own pages: <https://tailscale.com/kb/1017/install> (installing),
<https://tailscale.com/kb/1081/magicdns> (the device names),
<https://tailscale.com/kb/1077/secure-server-ubuntu> (with the firewall).

**Never** forward port 22 or 3389 in your router instead. Tailscale does the same
job without letting the whole internet knock on your computer's door.

## Keep it safe
- **Never forward port 22 (SSH) or 3389 (Remote Desktop) in the router.** Only
  80 and 443 go to the internet, for the website.
- Keys are better than passwords. Keep your PC's `.ssh` folder private, and never
  give anyone the file without `.pub`.
- If a PC is lost or given away, remove its key: menu `U`, then `7`.
