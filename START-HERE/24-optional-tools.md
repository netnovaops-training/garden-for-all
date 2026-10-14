# 24. Tools for learning and programming (very optional)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- **Nothing here is needed for the garden.** It runs without any of these tools.
- This page is for later, when you want to explore: an editor, a first
  programming language, a look at what the computer is doing.
- One table each for Windows, Ubuntu 24.04 and Raspberry Pi OS. Each table says
  what is already there.
- On Ubuntu and the Pi, most tools install with one line: `sudo apt install NAME`.
  `sudo apt remove NAME` takes it away again.
- Install from official pages only. One tool at a time. Never paste commands from
  strangers.

---

> **Nothing on this page is needed for the garden.** The garden menu installs
> everything the garden uses. These tools are for when you want to explore,
> learn, or write a little code of your own. Skip this page as long as you like.

## On Windows

| Tool | What it is for | Already there? | How to get it |
|---|---|---|---|
| Windows Terminal | A window for typing commands, with tabs | Yes, in Windows 11 | <https://learn.microsoft.com/windows/terminal/> |
| PowerShell | Windows' own command language (guide 20) | Yes, in Windows | <https://learn.microsoft.com/powershell/> |
| WSL | A real Ubuntu inside Windows | Yes, if the garden runs on this PC (the Windows menu switched it on) | <https://learn.microsoft.com/windows/wsl/install> |
| Visual Studio Code | An editor for code and text files | No | <https://code.visualstudio.com/> |
| Git | Keeps a history of changes to your files | No | <https://git-scm.com/downloads/win> |
| Python | A beginner-friendly programming language | No | <https://www.python.org/downloads/windows/> |
| Thonny | A simple editor for learning Python (Python comes with it) | No | <https://thonny.org/> |
| Geany | A light, fast editor | No | <https://www.geany.org/> |

## On Ubuntu 24.04

| Tool | What it is for | Already there? | How to get it |
|---|---|---|---|
| nano | A simple editor inside the terminal (guide 22) | Yes | <https://www.nano-editor.org/> |
| Git | Keeps a history of changes to your files | Yes, the garden's install added it | <https://git-scm.com/> |
| Python (`python3`) | A beginner-friendly programming language | Yes, part of Ubuntu | <https://www.python.org/> |
| htop | Shows what the computer is doing, live (**q** leaves) | No | `sudo apt install htop` (<https://htop.dev/>) |
| tree | Shows folders as a tree | No | `sudo apt install tree` (<https://packages.ubuntu.com/noble/tree>) |
| Thonny | A simple editor for learning Python | No | `sudo apt install thonny` (<https://thonny.org/>) |
| Geany | A light, fast editor | No | `sudo apt install geany` (<https://www.geany.org/>) |
| Visual Studio Code | An editor for code and text files | No | `sudo snap install --classic code` (<https://code.visualstudio.com/docs/setup/linux>) |

## On Raspberry Pi OS (64-bit, with desktop)

| Tool | What it is for | Already there? | How to get it |
|---|---|---|---|
| nano | A simple editor inside the terminal (guide 22) | Yes | <https://www.nano-editor.org/> |
| Git | Keeps a history of changes to your files | Yes | <https://git-scm.com/> |
| Python (`python3`) | A beginner-friendly programming language | Yes | <https://www.raspberrypi.com/documentation/computers/os.html> |
| Thonny | A simple editor for learning Python | Yes | <https://thonny.org/> |
| Geany | A light, fast editor | Yes | <https://www.geany.org/> |
| htop | Shows what the computer is doing, live (**q** leaves) | Yes | <https://htop.dev/> |
| tree | Shows folders as a tree | No | `sudo apt install tree` (<https://packages.debian.org/trixie/tree>) |
| Visual Studio Code | An editor for code and text files | No | `sudo apt install code` (<https://code.visualstudio.com/docs/setup/raspberry-pi>) |

## For later

| Tool | What it is for | How to get it |
|---|---|---|
| Rust (with rustup) | The language the garden website is written in. The garden builds the website without it (inside Docker) | <https://rustup.rs/> |
| Android Studio | Builds the garden's phone app (guide 17) | Windows and Ubuntu: <https://developer.android.com/studio>. Not on the Raspberry Pi: Google's page says Linux computers with ARM processors are not supported yet |

**Rust on Windows:** download and run `rustup-init.exe` from <https://rustup.rs/>.

**Rust on Ubuntu or the Pi:** the one line from <https://rustup.rs/>:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## How to install a program from the terminal on Ubuntu or the Pi

```
sudo apt install NAME
```

Put the program's name in place of `NAME`, for example `sudo apt install tree`.
It asks for your password (nothing shows while you type). If it asks
"Do you want to continue?", type `y` and press Enter.

To remove it again:

```
sudo apt remove NAME
```

Ubuntu's official page: <https://ubuntu.com/server/docs/how-to/software/package-management/>

## Good habits

- **Install from official pages only**, like the ones on this page.
- **One tool at a time.** Try it, then decide on the next.
- **Never paste commands from strangers** (a forum, a video, a message).
- Try things in your own playground folder (guide 22), not in the garden folder.

Next: [22 Bash, step by step](22-bash-walkthrough.md)
