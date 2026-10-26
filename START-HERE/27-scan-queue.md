# 27. Scan plants: the scan queue (optional, needs the plant AI helper)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- In the Android app, tap **Scan plants**. Pick the **bed** once, then tap **Scan a tag** (front, then back) or **Photo of a plant**. Go straight on to the next one.
- Every scan is kept **on the phone** until the garden website has it, so no signal and app restarts lose nothing. The website keeps it until the **computer at home** picks it up.
- The computer at home (the one with the graphics card, guide 15) reads the tag, looks in your garden and on the internet, and sends back a **best match** and a few **likely matches**, each with how sure it is.
- You tap **This one** and the plant is saved in the bed you picked. **Only the garden owner can tap it** (Jun, not the author). Nothing is added by itself.
- The beds are kept by **number**: if you rename a bed, scans already waiting still go to it.
- Nothing is found? Tap **None of these: type it instead** (genus, species, name). **Type it instead** on the main screen adds a plant without a photo (genus, species, name, how many, price, bed).
- The same page is on the website, tab **Scan queue** (on a phone it opens the camera).
- No cloud AI reads your tags. The pictures stay on your phone, your garden website and your own PC.

---

<a id="help-scan-queue"></a>

## What it is
Walking through a nursery or a garden with a tray of plants, you do not want to stop and type
each one. The scan queue is a line (a queue) of plants to add. You photograph a tag, pick the
bed, and photograph the next one. Meanwhile the computer at home works through the line.

```
1. phone: scan, scan, scan (kept in a folder on the phone)
      |  sent when there is signal
      v
2. garden website: keeps the line and the pictures (private folder)
      ^  the PC at home asks about every minute, and sends its answers back
      v
3. PC at home (graphics card): reads the tag (OCR), searches the garden
   and the internet, finds the best match

You tap "This one" (on the phone or the website): the plant and its bed are saved.
```

The phone does **not** talk to the PC. The PC only makes ordinary outgoing requests to the
website (like a browser), so no new port is opened anywhere. That also means a scan taken far
from home is uploaded as soon as the phone has signal, and read when the PC next asks (about
every minute while it is on).

## What you need
- The garden website and the Android app (the **Scan plants** button is at the top of the
  bottom bar). The app must be signed in as you.
- For the reading: the plant AI helper on the Windows PC (guide 15, "Local plant AI"). The
  Windows setup wizard also starts the **tag reader** there: a small Docker container
  (Tesseract and RapidOCR, on the processor) on `127.0.0.1:8788`, this PC only. Without Docker
  the helper tries the vision model only, which is slower and less exact.
- Optional: the **plant photo index** (a separate helper on the same PC). With it, a photo of
  a plant (or a tag with a picture) finds look-alike plants in your own photos. Without it the
  scan still works from the tag text; a note says the photo matching was skipped.

## On the phone, step by step
1. Open the app, signed in. Tap **Scan plants**.
2. **Bed: ...** is the big button at the top. Tap it and pick the bed. It stays picked for the
   next scans, so you only change it when you move to another bed. (The first scan asks
   by itself.)
3. **How many** has **−** and **+**. It goes back to 1 after every scan.
4. Tap **Scan a tag**. The phone's camera app opens: take the **front**. The app asks for the
   **back** (the back has the sun, water and zones): take it, or tap **Just the front**.
5. The scan is saved at once, and the app is ready for the next one: tap **Scan a tag** again.
   For a plant with no tag, tap **Photo of a plant** instead.
6. Each scan shows in the list with its picture, bed and status:

| Status | What it means |
|---|---|
| on the phone, waiting for signal | saved here; it is sent as soon as the phone is online (also with the app closed) |
| waiting for the computer | the website has it; the PC has not picked it up yet (is it on?) |
| being read | the PC is reading the tag or looking it up |
| found: tap to check | there is a clear best match: one tap |
| needs you | no clear match: pick one of the likely ones, or type it |
| added | the plant is in the garden |
| failed | the PC could not read it (the reason is shown); remove it or type the plant in |

7. Tap a scan that is **found** or **needs you**. You see the **best match** first (with its
   picture of the tag, how sure it is, and why) and the **likely matches** under it. Tap
   **This one**. If the plant is already in your garden the button says **Same plant: add to its
   count**: its count and bed list are updated instead of making a second row.
8. After you tap, the website saves the plant in the bed (by the bed's name **now**), and the
   normal fill-and-check helper (guide 15) completes its facts. Then the plant waits for the
   usual **Approve** on the Add plant tab.

**How sure?** The percent is the computer's own estimate from the clues it found (the barcode,
the name and the Latin name read from the tag, a matching photo, a page on the internet). Two
clues that agree count for more than one. 75% or more, and clearly ahead of the next one, is
called **found**; below that it is **needs you**. It is a guide, not a promise: you decide.

## On the website
- Tab **📷 Scan queue** on the plant admin page does the same on a phone's browser: three
  picture boxes (tag front, tag back, or a plant), the bed, the count, and the list with the
  same buttons.
- The **Add plant** tab (typing the six things) now has the same **bed picker** (by number).

## What the computer at home does
For each scan, with no cloud AI at all:
1. **Reads the tag** with the tag reader (OCR) and finds the plant name (the biggest words),
   the Latin name, the pot size, the grower and the barcode. A barcode or QR code is decoded
   exactly. When the reading is poor, the vision model (`gemma4:12b`) reads the picture too.
2. **Looks in your garden**: every current plant and every wish-list plant, by barcode, name and
   Latin name (small reading mistakes are forgiven).
3. **Looks at the photo** (if the plant photo index is running): which of your plants look
   like it.
4. **Looks on the internet** when the plant is not in the garden: a polite search (grower and
   garden-reference pages first, at most one request every 3 seconds, every page fetched once,
   `robots.txt` respected) to confirm the name and find the Latin name. Only plant names
   are searched for, never your pictures.
5. Sends back the **best match** and 2 to 4 **likely matches** with their confidence.

*Engineering note.* The steps run in threads (garden lookup first; photo index and internet in
parallel). The tag reader was chosen by measuring on made-up tags (a generator in
`plant-ai/plantai/synth.py`, no real tags exist yet): on 40 fresh tags, RapidOCR (the PaddleOCR
models) got the name, Latin name, size, grower and barcode all right on 37 (92.5%), Tesseract on
9 (22.5%). So RapidOCR is the main reader. Real tags will be harder: add real photos when you
have them and run `python -m plantai ocr-eval`.

## Where the pictures go
- On the phone: the app's private storage, until the website has them.
- On the website: a private folder (`data/scans/`), **not** a web address and never in git.
- A scan you remove is deleted from both. The reading (text and matches) stays on the scan.
- Nothing about your plants or pictures is sent to a cloud AI.

## If something is wrong
| What you see | What to do |
|---|---|
| "on the phone, waiting for signal" for a long time | The phone cannot reach the website. Check the signal; it retries by itself, a little less often each time. |
| "waiting for the computer" for a long time | The PC is off, or the helper is not running. Switch the PC on; on the Add plant tab the line "The desktop helper is on" says when it last asked. |
| "needs you" with nothing in the list | The tag could not be read and nothing matched. Retake the tag closer and square on, or type it in. |
| The reader is slow or empty | Is Docker Desktop running? Windows menu, More, 22 shows "Tag reader ... answering". |
| "Only the garden owner can confirm a match" | You are signed in as someone who is not the garden owner. Ask Jun. |
| A wrong plant was added | The Add plant tab keeps a copy of every plant before the helper changes it (`plant_ai_backups`); use the Full editor to fix it. |

## Read more (official)
- Tesseract, the text reader: <https://tesseract-ocr.github.io/tessdoc/>
- RapidOCR (the PaddleOCR models used here): <https://rapidai.github.io/RapidOCRDocs/>
- Android WorkManager (how the phone sends scans with the app closed): <https://developer.android.com/develop/background-work/background-tasks/persistent/getting-started>
- Docker: publishing a port to this computer only (`-p 127.0.0.1:...`): <https://docs.docker.com/engine/containers/run/>
