# 22. Bash, step by step: a first walkthrough of the terminal

> The terminal way is harder than Garden Bridge, but not tough: you have my
> walkthrough, one step at a time. (Jason)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- The **terminal** is a window where you type commands instead of clicking. **Bash** is the program inside it that reads what you type and runs it.
- Open it on Ubuntu with **Ctrl+Alt+T**. On Windows, use the **Garden terminal** shortcut (Windows menu, More, 18).
- Type a command, press **Enter**, read what it prints. Nothing happens until you press Enter.
- Five commands take you a long way: `pwd` (where am I?), `ls` (what is here?), `cd` (go to a folder), `less` (read a file), `clear` (clean the screen).
- Run the garden program with `cd ~/junsgarden-for-jun` and then `bash garden.sh`.
- When the computer asks for your password, **nothing shows while you type**. That is normal: type it and press Enter.
- **Ctrl+C** stops what is running. In a terminal, copy is **Ctrl+Shift+C** and paste is **Ctrl+Shift+V**.
- `rm` deletes for good (there is no Recycle Bin). Never paste commands from people you do not know.
- **Shortcuts:** Up arrow brings back the last command, Tab finishes names, Ctrl+R searches what you typed before.
- **Make and edit a file:** `touch notes.txt` makes it, `nano notes.txt` edits it (Ctrl+O, Enter saves; Ctrl+X leaves).
- **Seems stuck?** Ctrl+C, then `q`, then close the window and open a new one. The garden's install carries on where it stopped.
- **Updates:** `sudo apt update`, then `sudo apt upgrade`, about once a week.
- **Your password** is also the `sudo` password: it proves you may do administrator work. Keep it in a password manager.

---

## What the terminal and bash are

Most of the time you use a computer by clicking: windows, icons, menus. A
**terminal** is a different way in. It is a plain window with text, where you
type an instruction (a **command**) and the computer answers in text.

The program that reads your command and carries it out is called a **shell**.
Ubuntu's shell is **bash** (short for "Bourne Again SHell"). When people say
"type this in bash" or "in the terminal", they mean the same thing: open the
terminal window and type it there.

Why learn it at all? The garden's menu already does every job for you. But the
terminal shows you exactly what is happening, it works the same on every Linux
computer, and it is how you look after the garden from another room (guide 20).
You do not need to learn much of it. This page covers what you need.

## Opening a terminal

**On Ubuntu:** press **Ctrl+Alt+T** (hold Ctrl and Alt, then press T). Or click
**Show Apps** (the grid of dots at the bottom left), type `terminal`, and click
**Terminal**.

**On Windows:** the garden runs on Ubuntu (a separate computer, or Windows' own
Ubuntu inside the PC). You open a terminal *on that Ubuntu*:

- **The Garden terminal shortcut.** Windows menu (`Garden-Menu-Windows.cmd`),
  `M`, then `18` puts a **Garden terminal** shortcut on your desktop.
  Double-click it. For a separate garden computer it connects with SSH (guide
  20); for Windows' own Ubuntu it opens that.
- **From the Windows menu:** `M`, then `5` (Open a terminal on the garden
  computer) opens one straight in the garden folder.
- **Windows Terminal** (Windows' own terminal program, already on Windows 11):
  open it from the Start menu, click the small down arrow next to the tab, and
  choose **JunsGarden** (the name of the garden's Ubuntu on this PC). For a
  separate garden computer, type `ssh jun@192.168.1.50` there instead (with your
  user name and address; guide 20).

A new terminal window starts in your **home folder** (more about that below).

## Reading the prompt

Before you type, the terminal shows a line that ends in `$`. This is the
**prompt**: it means "ready, type a command". On the garden computer it looks
like this:

```
jun@garden-lab:~/junsgarden-for-jun$
```

| Part | What it means |
|---|---|
| `jun` | your user name on this computer |
| `@` | "at" |
| `garden-lab` | the computer's name (its host name) |
| `:` | a separator |
| `~/junsgarden-for-jun` | the folder you are in now. `~` is your home folder |
| `$` | the end of the prompt. You type after it |

So the line reads: "user jun, at the computer garden-lab, in the folder
junsgarden-for-jun inside the home folder, ready."

Your own prompt shows your own names. If it ends in `#` instead of `$`, you are
working as the administrator (root). The garden never needs that: type `exit`
to go back.

In this guide, commands are shown without the prompt. You type only the
command, then press **Enter**.

## Your first five commands

Try each one. None of them changes anything.

**1. `pwd`: where am I?**

```
pwd
```

| You type | You see | It means |
|---|---|---|
| `pwd` | `/home/jun` | You are in the folder `/home/jun`, your home folder. `pwd` stands for "print working directory" (directory is another word for folder). |

**2. `ls`: what is in this folder?**

```
ls
```

| You type | You see | It means |
|---|---|---|
| `ls` | `Desktop  Documents  Downloads  junsgarden-for-jun  ...` | The folders and files here. `ls` stands for "list". |
| `ls -l` | one line per item, with its size and date | `-l` is an **option**: a small extra word that changes how a command works. Here it means "long list". |

**3. `cd`: go into a folder**

```
cd junsgarden-for-jun
```

| You type | You see | It means |
|---|---|---|
| `cd junsgarden-for-jun` | nothing, but the prompt now ends in `~/junsgarden-for-jun$` | You are now inside the garden folder. `cd` stands for "change directory". No message means it worked. |
| `cd ..` | the prompt ends in `~$` again | `..` means "the folder above this one". |
| `cd` | the prompt ends in `~$` | `cd` on its own always takes you home. |

**4. `cat` and `less`: read a file**

Go back into the garden folder first (`cd junsgarden-for-jun`), then:

```
less CHANGES.md
```

| You type | You see | It means |
|---|---|---|
| `cat CHECKLIST.txt` | the whole file, printed at once | Good for short files. `cat` prints a file to the screen. (`CHECKLIST.txt` is there once the checklist has run.) |
| `less CHANGES.md` | the first screenful of the file | Good for long files. Move with the arrow keys or the space bar. Press **q** to leave. |

`cat` and `less` only read. They never change the file.

**5. `clear`: clean the screen**

```
clear
```

| You type | You see | It means |
|---|---|---|
| `clear` | an empty window with a new prompt at the top | Only the screen is cleaned. Nothing is deleted. **Ctrl+L** does the same. |

## Paths and folders

A **path** is the address of a file or folder: the folder names on the way to
it, with `/` between them. For example `/home/jun/junsgarden-for-jun/garden.sh`.

| Sign | Means | Example |
|---|---|---|
| `/` at the start | the very top of the computer's folders | `/home/jun` |
| `~` | your home folder | `~` is `/home/jun` for the user jun |
| `.` | this folder | `ls .` lists this folder |
| `..` | the folder above this one | `cd ..` goes up one |

**Absolute and relative paths.** A path that starts with `/` (or `~`) is
**absolute**: it works from anywhere, like a full postal address.
A path without it is **relative**: it starts from the folder you are in, like
"the second door on the left". These two do the same thing, from your home folder:

```
cd /home/jun/junsgarden-for-jun
cd junsgarden-for-jun
```

The second one only works when you are in your home folder. When in doubt, use
the absolute one.

**Two keys that save typing:**

- **Tab** finishes a name for you. Type `cd juns` and press **Tab**: bash
  completes it to `cd junsgarden-for-jun/`. If nothing happens, press Tab twice
  to see the choices. This also prevents typing mistakes.
- **Arrow up** brings back the commands you typed before, newest first. Press
  it, change the command if you need to, and press Enter. `history` lists them all.

Upper and lower case matter: `Backups` and `backups` are two different names.
A name with a space in it needs quotes: `cd "My Photos"`.

## Shortcuts that save typing

Linux is built for doing things fast by typing. These keys mean you rarely type
anything twice.

| Press or type | It does |
|---|---|
| **Up arrow** | Brings back the last command. Keep pressing for older ones |
| **Down arrow** | Goes back to newer ones |
| **Tab** | Finishes a name for you. Press it twice to see the choices |
| **Ctrl+R** | Searches what you typed before. Type a few letters; Enter runs what it finds |
| **Ctrl+A** / **Ctrl+E** | Jumps to the start / end of the line |
| **Ctrl+U** | Clears the line (from the cursor back to the start) |
| **Ctrl+L** | Clears the screen |
| **Ctrl+C** | Stops what is running |
| `!!` | Runs the last command again |
| `sudo !!` | Runs the last command again, as administrator |

The official list (the GNU Bash manual):
<https://www.gnu.org/software/bash/manual/html_node/Bindable-Readline-Commands.html>

## Make a file, and edit it with nano

**Make a file and look at it:**

| You type | It does |
|---|---|
| `touch notes.txt` | Makes an empty file called `notes.txt` (or updates the time on one that is already there) |
| `ls -l` | Lists the files, so you can see it |
| `mkdir seeds` | Makes a folder called `seeds` |
| `cat notes.txt` | Shows what is inside the file |

**Edit it with nano.** `nano` is a simple editor inside the terminal:

```
nano notes.txt
```

Type your text. The two lines at the bottom of nano's screen show its keys. `^`
means **Ctrl**:

| Key | nano calls it | It does |
|---|---|---|
| **Ctrl+O**, then **Enter** | Write Out | Saves the file |
| **Ctrl+X** | Exit | Leaves nano. If you did not save, it asks: **Y** saves, **N** throws the changes away |
| **Ctrl+K** | Cut | Cuts the whole line |
| **Ctrl+U** | Paste | Pastes the line you cut |
| **Ctrl+W** | Where Is | Searches the file |
| **Ctrl+G** | Help | Shows nano's help |

**Practise in your playground** (a folder of your own, see "Play with Linux"
below):

```
mkdir ~/playground
cd ~/playground
touch notes.txt
nano notes.txt
```

Type one line, for example `My first note`. Press **Ctrl+O**, then **Enter**
(saved). Press **Ctrl+X** (back at the prompt). Then look:

```
cat notes.txt
```

If `mkdir` says the folder already exists, that is fine.

**Never edit the garden's `.env` or `garden.sh` with nano.** The garden menu
changes its settings safely (Settings, menu `8`).

nano's own manual: <https://www.nano-editor.org/dist/latest/nano.html>

## Running the garden program

The garden's program is one file, `garden.sh`, in the garden folder. Go there
first, then start it:

```
cd ~/junsgarden-for-jun
bash garden.sh
```

`bash garden.sh` means "bash, please run the file garden.sh". The menu opens:

![The garden menu, opened with bash garden.sh](images/screen-01-main-menu.png)

Type a number or letter and press Enter, the same as when you open it from the
desktop icon. `0` quits and brings you back to the prompt.

**Command words.** You can also jump straight to one job by adding a word after
`garden.sh`. For example:

```
bash garden.sh check
```

These are the everyday ones (from the program's own list):

| You type | It does | Changes anything? |
|---|---|---|
| `bash garden.sh check` | the checklist: is everything OK? (menu 5) | No (writes `CHECKLIST.txt`) |
| `bash garden.sh start` | start the garden (menu 2) | Starts it |
| `bash garden.sh stop` | stop the garden (menu 3). Nothing is deleted | Stops it |
| `bash garden.sh open` | open the website in the browser (menu 4) | No |
| `bash garden.sh logs` | the programs' recent messages (More 6) | No |
| `bash garden.sh backup` | make a backup now (Backups 1) | Adds one backup file |
| `bash garden.sh verify` | check the newest backup's fingerprints (Backups 3) | No |
| `bash garden.sh backups` | the Backups menu (menu 6) | Only what you choose there |
| `bash garden.sh restore` | put a backup back; it asks which (menu 7) | Yes, after asking |
| `bash garden.sh settings` | the Settings menu (menu 8) | Only what you choose there |
| `bash garden.sh addresses` | every web address of your garden (menu A) | No (writes `MY-ADDRESSES.txt`) |
| `bash garden.sh helpme` | a help file with no passwords in it (menu H) | No (writes a file in `help/`) |
| `bash garden.sh install` | install, carry on, or repair (menu 1) | Yes |

The other words are `public`, `remote`, `nas`, `cloud`, `password`, `email`,
`network`, `parts`, `ai`, `wiki`, `rebuild`, `dbweb`, `dbweb-stop`, `db`,
`icon`, `uninstall`, `files`, `keys`, `subdomains`, `offline`, `llm`,
`language`, `users`, `lockdown`, `webname`, `photos` and `test`. Guide 19
explains each one ("Every command word"). A word the program does not know
shows its short help, so `bash garden.sh help` is a safe way to see the list.

## sudo and your password

Some commands change the computer itself (install a program, change a system
setting). Ubuntu protects those. To run one, you put `sudo` in front of it
("do this as the administrator"):

```
sudo apt update
```

The first time, it asks:

```
[sudo] password for jun:
```

Type **your own Ubuntu password** (the one you log in with) and press Enter.
**Nothing shows while you type**: no letters, no dots, no stars. That is on
purpose, so nobody looking over your shoulder can count the letters. Your typing
still arrives. If you make a mistake, it says so and asks again.

After that, sudo remembers you for about 15 minutes in that window, so it does
not ask every time. The garden menu uses sudo itself when a step needs it, and
always shows the command first.

Use sudo only for commands you understand or that a guide gives you. It can
change anything on the computer. Official page:
<https://manpages.ubuntu.com/manpages/noble/man8/sudo.8.html>

## Your password and sudo: why it mattered at setup

When Ubuntu (or Raspberry Pi OS) was installed, you chose a password. (For the
garden on Windows' own Ubuntu, the Windows menu asked you to choose it.) That is
your own login password, and it is also what `sudo` asks for.

- **It proves you may do administrator work:** installing programs, changing the
  firewall, and the garden's install steps.
- **Ubuntu keeps the separate "root" account locked on purpose.** It has no
  password you could use. `sudo` is the safe way in: one command at a time, with
  your own password, and Ubuntu keeps a record of each use.
- **Nothing shows while you type it.** Type it and press Enter.
- **Keep it in a password manager.** If it is lost, it can be reset from
  Ubuntu's **recovery mode**, at the computer itself. That is a careful job:
  ask someone you trust to sit with you. Ubuntu's page:
  <https://wiki.ubuntu.com/RecoveryMode>

Ubuntu's official page about root and sudo:
<https://ubuntu.com/server/docs/how-to/security/user-management/>

## Keeping the computer up to date from the terminal

Two lines, one after the other:

```
sudo apt update
sudo apt upgrade
```

| Line | What it does |
|---|---|
| `sudo apt update` | Fetches the list of new versions. It changes nothing yet |
| `sudo apt upgrade` | Installs them. It shows what it will change and asks first: type `y` and press Enter |

**How often:** about once a week, or when Ubuntu says updates are waiting. If it
asks you to restart afterwards, type `sudo reboot`.

**The garden's own programs are different.** They run in Docker boxes, not
through apt. The install (menu `1`) downloads fresh copies of them when it builds
the garden; it skips that step when the garden is already built and working.
Docker itself and Caddy (the front door, once you set it up) came from their
makers' own lists, so `sudo apt upgrade` keeps those up to date too.

Ubuntu's official page: <https://ubuntu.com/server/docs/how-to/software/package-management/>

## Stopping things safely

| To stop | Press or type | What happens |
|---|---|---|
| A command that is running | **Ctrl+C** | It stops and the prompt comes back. In the garden menu, if a step was half done, choose `1` again later: it carries on. |
| Reading a file in `less` | **q** | You are back at the prompt. |
| A program waiting for you to type | **Ctrl+C**, or its own way out (`0` in the garden menu, `exit` in the database command line) | You are back at the prompt. |
| The terminal itself | `exit` | The window closes (or, over SSH, the connection to the garden computer ends). |

**Ctrl+C in a terminal does not copy.** It stops things. Copying is below.

## When the terminal seems stuck

Try these, in this order:

| What you see | What to do |
|---|---|
| A program is running and nothing changes | **Ctrl+C** stops it and gives the prompt back |
| A question, waiting for you | Type the answer (for example `y`) and press Enter |
| A page of text, with `:` or `(END)` at the bottom | **q** leaves the page viewer (`less`) |
| An empty line waiting for input | **Ctrl+D** ends the input. On an empty prompt it closes the terminal |
| Still frozen | Close the window and open a new one |

If the garden's install was running in that window, choose `1` again later: it
carries on where it stopped.

## Copy and paste in a terminal

| To | On Ubuntu's Terminal | In Windows Terminal |
|---|---|---|
| Copy selected text | **Ctrl+Shift+C** | **Ctrl+Shift+C** (or **Ctrl+C** while text is selected) |
| Paste | **Ctrl+Shift+V** | **Ctrl+Shift+V** (or **Ctrl+V**) |

Select text by dragging over it with the mouse. Right-clicking also opens a menu
with **Copy** and **Paste**.

After you paste a command, read it once before you press Enter.

## Reading the messages

The garden program speaks in a few fixed ways, so you can tell at a glance what
happened:

| You see | It means |
|---|---|
| `== Some title` (green) | A new step starts. |
| A blue box with **What:**, **Why:** and **Read:** | The step explains itself before it changes anything, with its official page. |
| `$ some command` (light blue) | The exact command it is about to run. |
| `WARNING: ...` (yellow) | Something to know or fix soon. The step carries on. |
| `STOPPED: ...` (red) | The step could not finish. The words after it say why and what to do. Fix that, then run the same choice again: it carries on. |

With the language set to Korean (menu `L`), these words appear in Korean; the
colors stay the same.

**The checklist** (menu 5, or `bash garden.sh check`) marks every line:

![The checklist: PASS, WARN or FAIL, with what to do](images/screen-19-checklist.png)

- **PASS**: good.
- **WARN**: fix soon. The line under it, starting with `->`, says how.
- **FAIL**: fix first. The fix is under it too.
- **INFO**: just information.

The last line counts them: `RESULT: 28 pass, 1 warn, 0 fail`. The whole list is
saved in `CHECKLIST.txt`.

**The log files.** You do not have to catch everything as it scrolls by. The
install and the front door step save everything they show in the `logs/`
folder, one file per run, named with the job and the date (for example
`logs/install-2026-10-13_1415.log`; the first lines on screen tell you the exact
name). The copies to your NAS and Google Drive are listed in `logs/offsite.log`.
To read the newest lines of a file:

```
tail -20 logs/offsite.log
```

(`tail` shows the end of a file; `-20` means the last 20 lines.)

## Safe practice

**Commands that only look** (try them any time):
`pwd`, `ls`, `cd`, `cat`, `less`, `tail`, `history`, `clear`,
`bash garden.sh check`, `bash garden.sh logs`, `crontab -l` (shows the
schedule), `docker ps` (shows the running garden programs).

**Commands that change things** (only when a guide or the menu tells you):

- `rm` deletes files. **There is no Recycle Bin**: what `rm` deletes is gone.
  `rm -r` deletes whole folders. Check twice, or use the Files app instead.
- `mv` moves or renames, and `cp` copies. Both can overwrite a file that
  already has that name, without asking.
- Anything that starts with `sudo`.
- `nano FILE` opens a file to edit it. Leave without saving with **Ctrl+X**,
  then **N**.
- `bash garden.sh` jobs such as `install`, `restore` and `uninstall` (they explain
  themselves and ask before they change anything).

**Habits that keep you safe:**

- **Never paste commands from strangers** (a forum, a video, a message). A
  single line can delete files or give someone a way in. If you are unsure what
  a command does, ask Gemini to explain it first (guide 16), and check it against the guide.
- Read a command before pressing Enter, especially a pasted one.
- Use the garden menu for garden jobs. It does the same commands, in the right
  order, with checks.
- Make a backup before a big change: `bash garden.sh backup`.
- Never type a password into a command line where it shows on the screen. The
  garden asks for passwords in a way that hides them.

## Play with Linux: no boot camp needed

You do not need a course to enjoy Linux. A little play, often, teaches more than
a long class. Here is a safe way to start, from the garden computer itself or from
your PC or phone through Remote Desktop (guide 20).

**Make a playground.** A folder of your own where nothing can hurt the garden:

```
mkdir ~/playground
cd ~/playground
```

Everything you try in `~/playground` stays there. Leave `~/junsgarden-for-jun` to
the garden menu.

**Ten minutes of play:**

| Try this | What it does |
|---|---|
| `echo "hello garden" > note.txt` | Makes a small text file called `note.txt` |
| `ls -l` | Lists the files here, with their size and date |
| `cat note.txt` | Shows what is inside the file |
| `cp note.txt copy.txt` | Makes a copy |
| `mkdir seeds` then `mv copy.txt seeds/` | Makes a folder and moves the copy into it |
| `ls seeds` | Looks inside the new folder |
| `date` and `cal` | Today's date, and this month's calendar (`cal` may need `sudo apt install ncal` first) |
| `df -h ~` | How much disk space is free |
| `history` | Everything you have typed so far |

**Try programs, the easy way.** Ubuntu's **App Center** (Show Apps, App Center) is
Ubuntu's own app store: browse, click Install, and try things. Nothing there
touches the garden.

**When you want more,** Ubuntu's own beginner tutorial is a good next step (about
an hour, in small pieces): <https://ubuntu.com/tutorials/command-line-for-beginners>.

## A short glossary

| Word | Meaning |
|---|---|
| terminal | the window where you type commands |
| bash | the shell Ubuntu uses: it reads your command and runs it |
| shell | the program that reads and runs commands |
| command | one instruction, such as `ls` |
| option | a small extra word that changes a command, such as `-l` in `ls -l` |
| prompt | the line ending in `$`: ready for your command |
| directory | another word for folder |
| path | the address of a file or folder, such as `/home/jun/junsgarden-for-jun` |
| home folder (`~`) | your own folder, `/home/jun` |
| sudo | "do this as the administrator"; asks your password |
| root | the administrator account; the prompt then ends in `#` |
| script | a file of commands that runs as one program, such as `garden.sh` |
| log | a file where a program writes what it did |
| SSH | a secure way to use the terminal of another computer (guide 20) |

## Read more (official)

- Ubuntu's own tutorial, "The Linux command line for beginners":
  <https://ubuntu.com/tutorials/command-line-for-beginners>
- The GNU Bash manual (bash's makers):
  <https://www.gnu.org/software/bash/manual/>
- The GNU Coreutils manual (`ls`, `cat`, `pwd`, `rm`, `mv`, `cp` and more):
  <https://www.gnu.org/software/coreutils/manual/>
- Microsoft's overview of Windows Terminal:
  <https://learn.microsoft.com/windows/terminal/>

Next: [19 Every menu option, explained](19-every-menu-option.md)
