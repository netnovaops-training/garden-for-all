# Project 2: Backups you can trust

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Your garden already backs itself up on its own (automatic: more often when it is busy, at least weekly; or every night at 2:30 am if you chose that). Here you keep a copy somewhere else and practise a restore.
- 30 minutes. A USB stick (16 GB or more) if you don't have one.
- Make a backup: menu `6`, then `1`. Check it: `bash garden.sh verify`.
- Copy the newest `.tar` file from the `backups` folder onto the USB stick.
- Practise a restore: menu `7`, pick the backup, say **y**. It saves what is there first (a `before-restore` backup).
- Habit: every Sunday, copy the newest backup to the USB stick. Keep the stick away from the laptop.
- Easier still: connect your NAS (menu `6`, then `4`) and Google Drive (menu `6`, then `5`). Then every backup is copied there by itself.

---

## What you will build
Your garden already backs itself up on its own: the automatic schedule backs up
more often while you are busy editing and at least once a week when it is quiet
(or every night at 2:30 am, if you chose that in menu `6`, then `7`). In this project
you make sure a copy also lives **somewhere else**, and you practise a restore,
so you know it works before you ever need it.

## Time and cost
30 minutes. A USB stick (16 GB or more) if you don't have one.

## What you need first
The install is done. Read [13 Backups and folder names](../13-backups-and-folders.md).

## Steps
1. **Make a backup now:** menu `6`, then `1`.
   *You should see:* `Saved: junsgarden-backup_DATE_TIME.tar` and its size.
2. **Check it:** in a terminal in the garden folder, `bash garden.sh verify`.
   *You should see:* the manifest (what is inside) and "every piece matches its fingerprint".
3. **See the backup schedule:** `crontab -l`.
   *You should see:* one line with `garden.sh backup auto` (or `garden.sh backup nightly`
   if you chose every night).
4. **Copy it off the laptop.** Plug in the USB stick. Open **Files**, go to the
   garden folder, then `backups`. Drag the newest `.tar` file onto the USB
   stick. From Windows, the Windows menu `M`, `10` copies the newest backup to
   the PC for you.
5. **Practise a restore** (safe: it saves what is there first). Menu `7`, pick
   the backup you just made, say **y**.
   *You should see:* "Restored ... plants, ... image files", then the health line.
   A `before-restore` backup now exists too.

## How you know it worked
- A backup from today is on the USB stick (or the PC).
- The restore finished and the site still shows your plants.
- The checklist (menu `5`) shows "Newest backup ... (0 days old)".

## Make it a habit
Every Sunday: copy the newest backup to the USB stick. Keep the stick
somewhere other than next to the laptop.

## If something goes differently
| What you see | What to do |
|---|---|
| "a piece ... is damaged" | That backup file is broken (often a bad copy). Use another one. Check the disk with the checklist. |
| The USB stick is too small | Backups include every photo (about 800 MB). Use a bigger stick, or keep only the newest. |
| The backup schedule line is missing | Menu `1` (install) puts it back. |

## Make it yours
- Connect your NAS (menu `6`, then `4`) and Google Drive (menu `6`, then `5`): after that,
  every backup is copied there by itself. No commands to learn.
- Have a Windows PC? The Windows menu (`M`, then `13`) copies the newest backup to it every week.
- Read about the 3-2-1 rule: <https://www.backblaze.com/blog/the-3-2-1-backup-strategy/>
