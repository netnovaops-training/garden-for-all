# 04b. Moving junsgarden.zapto.org to your computer

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- This guide moves `junsgarden.zapto.org` to your computer. Jason released the name, so you claim it in your own No-IP account.
- Your data (plants, photos, comments, wiki pages) was already loaded by the install.
- Not carried over: the weather collector (Project 5 builds one) and passwords (set yours with menu `U`, `5`).
- The steps: practise with a `.test` name (guide 04, Part C), claim the name and make a DDNS key (Part A), run the front door step with the real name (Part D), then forward ports 80 and 443 (guide 14).
- Check from a phone with Wi-Fi off: the padlock, plants and photos, logging in, `/beds` asking you to sign in, and the phone app.
- Your backup from the zip (in `backups/`) is your safety net. `sudo systemctl stop caddy` takes the site off the internet while you fix something.

---

`junsgarden.zapto.org` used to be served from Jason's server. Jason has released
the name from his No-IP account, so you can claim it in **your own** No-IP
account (guide 04, Part A). From then on the name, the website, the data and
every login are yours.

## What is already done for you
- **Your data:** the install loaded the whole garden (485 plants, 7,723 photo
  files, comments, the AI cache, wiki pages) from the backup in the zip.
- **The garden tools** (`/beds`) and the phone app's camera tools run on your
  computer (they need a free AI key for the AI parts: Settings, `4`).
- **Logins:** only the people in your settings can sign in. Jason has no
  account on your copy. Menu `U` shows exactly who can sign in.

## What is NOT carried over
- **The weather collector** ran on the old server. New weather readings stop
  until you add your own ([Project 5](projects/P05-weather-and-rain.md) builds one).
  The old readings are all in your database.
- **Passwords:** nobody's old password came with the data. You set yours (menu `U`, `5`).

## The steps
1. Practise with a `.test` name (guide 04, Part C) until the front door works.
2. Claim the name in your No-IP account and make a DDNS key (guide 04, Part A).
3. Run the front door step with the real name (guide 04, Part D). Your updater
   points the name at your home within minutes.
4. Forward ports 80 and 443 to your computer (guide 14).
5. Check from a phone with Wi-Fi off:
   - `https://junsgarden.zapto.org` opens your garden with the padlock.
   - The catalogue shows the plants and photos.
   - You can log in and change a plant.
   - `/beds` asks you to sign in, then opens.
   - The phone app signs in and shows plants.

## The old server
Two programs on the old server may still try to answer for the name: its No-IP
updater and its web front door. Because the name is no longer in that No-IP
account, its updater can no longer change where the name points, so it cannot
take the name back. If you ever see the old site instead of yours, check
`getent hosts junsgarden.zapto.org` against `curl -s https://api.ipify.org` on
your computer: they must show the same address.

## The way back
Your backup from the zip (in `backups/`) is your safety net. Nothing in these
steps deletes it. If the front door misbehaves, `sudo systemctl stop caddy`
takes the site off the internet while you fix it; the garden keeps running at
home.

## If something goes differently
| What you see | What to do |
|---|---|
| The name shows a different address | Wait 10 minutes. `sudo systemctl status noip-duc`. Check the hostname is in YOUR No-IP account. |
| Padlock error | Ports 80 and 443 must reach your computer from outside (guide 14). `sudo journalctl -u caddy -n 50` explains. |
| Plants appear but no photos | Run the checklist (menu `5`), section 4: the images folder must have files. Guide 13 explains folder names. |
| Password email never arrives | Settings, `3` (email), and send the test email. |

Back to [the ladder](00-READ-ME-FIRST.md).
