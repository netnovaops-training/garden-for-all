# 00. Overview: what is happening, in one screen

<a id="help-overview"></a>
## Quick version
*Read this while the install runs. It takes two minutes.*

- Jun's Garden is a website with your plants, photos and blog, plus garden tools and an optional wiki. The install puts it on a computer you choose; from then on it is yours.
- The install works through the steps below, in order. Each one explains itself first, and every step is safe to run again.
- Nothing needs you to wait at the screen except your password near the end. You can read the guides while it works.

## What the install does, in order

1. **Check this computer** (memory, disk, Ubuntu version).
2. **Confirm the folder** where the garden lives.
3. **Ubuntu updates and tools** (git, curl, openssl, rsync).
4. **SSH server**, so you can reach the computer from another one.
5. **Firewall**, with SSH let through.
6. **Docker**, which runs every program of the garden.
7. **Swap**, extra working room for small laptops.
8. **Settings file** (`.env`), where passwords and keys are kept, hidden.
9. **Choose the optional parts** (EASY chooses for you).
10. **Folders** for photos, data and backups.
11. **Change history (git)**, so you can see what changed.
12. **Build, load your garden, start.** The longest step, 15 to 45 minutes.
13. **Backup schedule**, automatic backups.
14. **Desktop icon**.

After that, a few short questions: your password, and optionally email and keys.

## Easy or Custom

- **EASY** installs everything the garden needs plus the garden tools. You type your name, your email and your password. Extra parts (the wiki, going online, email, NAS, Google Drive, n8n, subdomains) come later from the menu.
- **CUSTOM** lets you tick every optional part, one page each, and start at any step. In the Windows wizard you type all answers there; the install window only installs.

## Where help is

- The **Help** button on every wizard page opens the guide at that question.
- Once the garden runs, **/help** on your own website shows every guide.
- Something wrong: [09. Troubleshooting](09-troubleshooting.md), then [25. Common questions](25-common-questions.md). Menu `H` makes a help file with no passwords in it.
- Next: [Read me first](00-READ-ME-FIRST.md), [Before you start](00b-before-you-start.md) and the [full setup](23-full-setup.md).

## Guides on your own computer

The installer also fetches the guides as PDFs, English and Korean (about 27 MB and 25 MB), while it works. The download does not slow the install. If it fails, nothing is half-written and the garden is not affected.

Open `PDF guides/STATUS.txt` to see where each set is:

| Set | Folder | Status line shows |
|---|---|---|
| English | `PDF guides/English` | `arriving`, `ready (88 PDFs)` or `not downloaded - use menu item P to get them` |
| Korean | `PDF guides/Korean` | the same |

While a set is arriving, its folder holds a small note, `_arriving.txt`; it is replaced when the set is in. Menu item `P` (PDF guides) fetches or refreshes a set at any time. The same guides are always on the website at /help.
