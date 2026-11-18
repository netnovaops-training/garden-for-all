# Adding a plant, with the local plant AI

*Jun's Garden, the plant admin (`/streamlit/`). Every picture is from a test garden with made-up
sample plants. The AI's answers in the pictures are a recorded sample run.*

<div class="note" markdown="1">

**Where the AI runs.** The AI that fills in new plants runs only on a **Windows PC with an NVIDIA
graphics card with 12 GB or more** (like an RTX 4070 Ti). It is set up by the Windows setup
wizard (CUSTOM, "Local plant AI"), or later by the Windows menu, More, 22. The website itself can
run on another computer, for example the laptop: the PC asks the website for work every minute,
over your home network. Nothing new is opened on either computer. If the PC is off, new plants
simply wait in the queue and are filled in when it is on again.

</div>

## 1. Add plant: six boxes

Open the plant admin. The first tab, **🌱 Add plant**, asks for only six things:

- **Genus**: the first word of the Latin name, for example *Salvia*.
- **Species**: the second word, for example *nemorosa* (write *hybrid* for a hybrid; empty if
  you do not know it).
- **Name**: the name as it is on the tag.
- **Bed**, **Quantity** and **Price each**.

Leave **Let the desktop AI fill in the rest** ticked and press **➕ Add plant**. The plant is
saved at once; the AI fills in the rest.

@shot 01-add-form

## 2. Watch the AI fill it in

The page shows the plant in **Plants with the AI**, with a bar, and the same bar under
**Review**. It goes: *waiting for the desktop helper* → *searching the web* → *filling in the
facts (phi4)* → *finding photos* → *checking every value (gemma)* → *ready for your review*. The
page refreshes by itself while the AI works; you can also close it and come back later.

@shot 02-queued

What happens on the PC: the helper finds the grower's page for this plant and up to three
trusted garden pages (university extension, botanical gardens), reads them politely (a few
seconds between pages), fills in every field the pages state, finds up to 5 photos, and then a
second model checks every value against the sentence it came from.

@shot 05-filling

## 3. The review: filled, needs review, still missing

When the answers are in, the same page shows the review:

- **The checker says**: a one-line verdict (*looks right*, or *needs a look*).
- **Source**: the grower's page the facts came from (it is also saved with the plant).
- **Photos found**: up to 5, each saved once as a `.webp` picture. The small line under them
  says where each one came from.

@shot 07-review

- **Filled by the AI**: every value that was written, each with the words on the page that say
  so and a ↗ link to that page. The AI only fills **empty** fields; a value you typed is never
  replaced. A copy of the plant is kept before every change.

@shot 08-filled

- **Needs review**: values the checker could not confirm. They are **not** written. Tick the ones
  you agree with and press **Use the ticked values** (only empty fields can be ticked).

@shot 09-needs-review

## 4. Still missing: ask your own Claude

**Still missing** lists the fields that are still empty. Press **📤 Export what's still missing**:
you get a small file with only those fields, the values each one allows, and a ready prompt (in
English and Korean). Give the file to your own Claude. Save its answer (a JSON list) as a
`.json` file and bring it back with **⚙️ Advanced → 🔄 JSON Pipeline → 📥 Import & Review**
(step 8). You see every value before anything is saved.

@shot 10-missing

The ready prompt, as it is in the file:

```
You are helping fill in a garden plant database. The file below lists plants and, for each,
ONLY the fields that are still empty, with the allowed values and the expected format.
Rules: 1) Answer with ONLY a JSON array (a top-level [ ... ]) ... 2) Include only the empty
fields listed for that plant; never add or change any other field. 3) For a field with
"allowed" values, use one of them exactly ... 5) If you are not sure of a value, leave it out:
an empty field is better than a wrong one.
```

## 5. Approve

At the bottom of the review is **✅ Approve**. Press it when the plant looks right. Only the
**garden owner** (you, Jun) can approve; anyone else signed in sees who can.

@shot 11-approve

After pressing it: *Approved by the garden owner*. **Undo approval** takes it back.

@shot 13-approved

What another account sees (it cannot approve):

@desk 12-not-owner

## 6. Receipt

The **🧾 Receipt** tab keeps the receipt with the plant: choose the plant (the newest is first),
type the store, the date and the price, choose the photo or PDF of the receipt and press
**💾 Save receipt**. Receipts are private: they are never on the public website.

@shot 14-receipt

The receipts kept for the plant are listed under the form, each with an **open** link.

@shot 15-receipt-saved

## 7. Pictures

The **🖼️ Pictures** tab adds your own photos: choose the plant, choose one or many pictures,
press **➕ Add pictures**. Each picture is kept once, as a `.webp` of at most 2000 pixels (no small
copies are made); pictures smaller than 500 pixels are not used. The first picture is the main
one; to remove or reorder pictures use *Manual Image Replace*.

@shot 16-pictures

@shot 17-pictures-added

## 8. Advanced: JSON import and the other tools

The JSON import is still there, under **⚙️ Advanced** (under the tabs). Advanced also holds the
cloud-AI tabs and the full Add Plant form. You only need it for step 4's answer.

@shot 18-advanced

## 9. The desktop helper's key

The PC needs a key to talk to the website. On the Add plant tab, open **🖥️ Desktop helper**, type
a name for the PC and press **🔑 Make a helper key**. The key is shown **once**: paste it into
the Windows setup wizard's Local plant AI page (or the Windows menu, More, 22). The list shows
when each key was last used, and **Switch off** stops a key at once.

@desk 03-helper-box

## Good to know

- The language: the **English · 한국어** links in the sidebar switch the new tabs.
- The AI never changes a value you typed, never writes what the checker did not confirm, and a
  copy of the plant is kept before every change.
- The laptop alone (no Windows PC) still works: plants are saved and wait in the queue; you can
  fill them in by hand in the *Manual Editor*, or with step 4.
- Questions: ask the author.
