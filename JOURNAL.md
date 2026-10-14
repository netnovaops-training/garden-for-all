# Journal: what the author did for this copy, and why

A plain diary, newest day last, so you can follow the thinking behind the
package, not only the result. The exact lines of every change are in git
(`git log`, `git show`), and the full list of changes is in `CHANGES.md`.

## Before 2026-10-13: making the copy yours

- The garden had moved from PHP to Rust on 2026-10-06. Only the live Rust site
  came into this copy; the old PHP pages did not.
- Everything that belonged to the old household server was taken out: its
  sign-in service, household links in the garden tools, old logins, and all of
  The author's own keys and accounts. Nothing in this package gives the author (or
  anyone else) a way into your computer or your garden.
- Your own sign-in page (Authelia) replaced the household one.
- The install was written to explain every step before doing it (what, why,
  the official page) and to save a log of everything.
- `junsgarden.zapto.org` was released from the author's No-IP account so you can
  register it in yours.

## 2026-10-13: the final test, and what it changed

**Testing.** A fresh, throwaway Ubuntu was set up and every menu option was
tried. Each problem found was fixed and recorded (CHANGES.md, Parts 2 and 4).
Among them: the sign-in page needed a few seconds after starting, "remove
everything" did not remove the programs, and a restore could stop new sign-in
settings from saving. The test was then repeated the way you will do it:
downloading the package from Google Drive first, with a small sample of the
photos and wiki so it runs quickly.

**Your web name.** You may want a name other than `junsgarden.zapto.org`.
The **Website name swap** puts a new name into every file that uses it, and the
install asks once, at the start: keep the name, or choose a new one. Free names
come from No-IP (one per free account, confirmed every 30 days). DuckDNS can
add more free addresses later, for example for Home Assistant, each behind your
sign-in page.

**"Not on the internet yet."** The first choice of the front door used to be
called "home only". It now says plainly what it is: your garden on your home
Wi-Fi only, with the next step (No-IP) spelled out.

**Google Drive backups.** rclone's shared Google sign-in is being retired during
2026, so connecting Google Drive now walks you through making your own free
one, with Google's and rclone's official steps.

**The wiki is your choice.** The wiki is no longer switched on by the Easy
install. Its pages are a separate small download, so nobody waits for a
program and pages they may never use.

**The photos are a separate download.** The garden download stays small. The
photos (about 800 MB) come in their own zip, which you can download when your
internet is quiet, or again any time. The install puts the pieces together and
checks each one's fingerprint first.

**Security review.** Before publishing, the whole package and its history were
checked by a separate reviewer for any way in. The answer: someone who reads
the public code, even knowing your name and email, cannot sign in as you or
anyone else. The package holds no passwords, password hashes or keys; yours are
made on your computer during your install. A few improvements it suggested are
listed in CHANGES.md as they are made.

The review's suggestions were then made (CHANGES.md, Part 4). The biggest: a
stranger's link can no longer make your site do anything while you are logged
in, the wiki only goes online once you are its admin, and nobody has a built-in
account. One fix was for you directly: setting a plant's main photo had been
allowed for the author's account only; now any logged-in gardener can.

**Reviewing it yourself.** Guide 18 shows how you, a friend or an AI helper
can check every file, see every change, and run the same secret scanners.

**Docker, in plain words.** A short page with two pictures explains what Docker
is and why the garden runs in small boxes instead of whole pretend computers,
which is what lets it run on an old laptop or a Raspberry Pi. It is there for
your information, not something you need to learn.

**Words used here.** Both read-me pages now start with a short list of the
words the package uses (the menu, the front door, Caddy, add-on, and so on),
one plain line each.

**For later (planned, not in this version).** A tool to rename the website's
folders and sub-folders, so you can rebuild the site around your own ideas
with every file and database entry following the new names.

**The last evening of testing (CHANGES.md, fixes 33 to 39).** Checking the
screenshots and reading the guides as you would found the last things:

- *Your Ubuntu in Korean.* If your Ubuntu is set up in Korean, its folders
  are called 다운로드 and 바탕화면. The install now finds the photos and wiki
  downloads there too.
- *OneDrive.* On many Windows 11 PCs the Desktop lives in OneDrive, which
  uploads everything on it, and its free plan is 5 GB. When the garden
  folder is there, the installer now offers to move it for you (it copies
  and checks every file, then carries on from the new place), and the big
  photos download is no longer copied twice.
- *Backups, wherever you like.* On Windows, the weekly backup copy goes to a
  folder you pick: this PC, a USB drive, Google Drive for desktop, or a NAS.
  On Ubuntu, the nightly backup can go to up to three network places (a NAS,
  a second NAS, a PC's shared folder), and you choose which one to stop.
- *Email for "forgot password".* No mail server is installed: Gmail's own
  server sends the emails, with an app password. The email screen now shows
  the two Google pages you need, right where it asks.
- *Small, careful fixes.* The home network is found the right way once Docker
  is installed; yes/no checks can no longer answer a wrong "no"; the USB
  stick for Ubuntu is 16 GB or larger.

**The terminal way.** A new guide, 22, walks through bash (the terminal) one
step at a time, for the day you want to do things by hand. Harder than the
Garden Bridge, but not tough.

**Screenshots.** Every menu and option now has a picture, taken from the real
program, with sample names and addresses instead of anyone's real ones.

## 2026-10-14: your own names, an off switch, and Docker Desktop

**Your own names.** The folders and pages came with the author's names (Gallery,
HeroBanner, privacy.html...). Menu M, 18 now renames a file or folder of the
website and changes every place that uses the name with it: the code, the
pages, the photo lists and the database. It shows every place first, offers a
backup, and checks the website afterwards. The guides keep the first names; a
small list, `logs/renames.tsv`, says what is called what now. The same menu
moves a big folder, or the whole garden, to a bigger disk.

**An off switch.** Menu M, 14 switches everything off without deleting
anything, for a quiet month or too much traffic, and on again later. A check-up
restarts a program that got stuck (it can run by itself every 6 hours), and a
clean-up removes Docker's old leftovers, never your files and never another
project's.

**Docker Desktop.** On a Windows PC the installer now asks: Docker Desktop (the
easy way, with buttons) or Docker inside Ubuntu (the hands-on way). Guide 26
explains both, and why an old Windows 7 PC is better turned into an Ubuntu
laptop.

All of it is plain scripts, with no AI inside: each does one job, and every line
can be read. Each new option was tried on a real test garden before release.
