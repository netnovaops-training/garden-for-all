# WHAT THIS FILE DOES (plain English)
# A plant check-up: send one photo of a plant, and the free AI writes what
# it can SEE, what that could mean, and what to check next. The answer is
# saved in the plant_checkup table (the photo itself is not kept). Used by
# the /checkup and /diagnose pages.
"""Photograph a plant, and get back what can be SEEN about it.

The wish: use a photo to help spot a plant that needs more nutrients or
less water.

IT DOES NOT NEED TO BE 3D. A flat photograph already carries everything a
plant assessment uses: leaf colour, the pattern relative to the veins,
spotting, scorch, wilt, new growth against old. Rotating the plant adds
nothing to any of those. Tested 2026-09-24 against a real catalogue photo:
the model correctly read reddish new growth on a Sherwood Abelia as a
CULTIVAR TRAIT rather than a deficiency, which is the exact mistake that
would have made this worse than useless.

IT DOES NOT DIAGNOSE, AND THE SHAPE OF THE ANSWER ENFORCES THAT. Three
fixed sections: what is VISIBLE, what that is CONSISTENT WITH, and what
would CONFIRM it. A photograph cannot tell nitrogen from waterlogging --
both yellow a leaf -- and a tool that says "nitrogen deficiency" will have
somebody feeding a plant that is actually drowning. Soil tests diagnose.
This narrows what to test.

THE CATALOGUE'S OWN FACTS GO IN THE PROMPT. Jun's record already knows this
plant's water needs, sun, soil pH, fertiliser and known disease
susceptibility. A generic "what is wrong with this plant" is a question
anything answers badly; the same question with "its record says Partial
Shade and pH 5.0-7.5" is one worth asking, and it is the reason to run this
here rather than paste the photo into a chatbot.

THE MODEL IS CHECKED FOR ACTUALLY LOOKING. Given the same photograph and
told it was a Blue Angel Hosta, it answered NO and described the shrub in
front of it. A model that agrees with whatever the prompt asserts is one
that will confirm any fear somebody brings to it.

EVERY CHECK-UP IS KEPT. One photograph is a moment; four over a season is
the only thing that can show a plant declining, and that trend is worth
more than any single reading.
"""
from __future__ import annotations

import base64
import json
import os

# How long to wait for the free AI to look at a photograph (seconds).
TIMEOUT = int(os.environ.get("GARDEN_VISION_TIMEOUT", "180"))

# A phone photograph is several megabytes and the model gains nothing from
# the sensor's full output. Enforced on the way in, not trusted from the
# page: this endpoint accepts an upload and an unbounded one is a hole.
MAX_UPLOAD = 12 * 1024 * 1024

SCHEMA = """
CREATE TABLE IF NOT EXISTS plant_checkup (
  id         INT AUTO_INCREMENT PRIMARY KEY,
  plant_id   INT           NOT NULL,
  looked_at  TEXT          NOT NULL,
  consistent TEXT          NOT NULL,
  confirm    TEXT          NOT NULL,
  raw        MEDIUMTEXT,
  photo      MEDIUMTEXT,
  asked_by   VARCHAR(190)  DEFAULT '',
  asked_at   TIMESTAMP     DEFAULT CURRENT_TIMESTAMP,
  KEY k_plant (plant_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
"""

# The record columns worth telling the model about. Not everything -- 90
# columns of context buries the question -- just the ones that change what
# a symptom means.
CONTEXT = [
    ("water_needs", "water needs"), ("sun_exposure", "sun"),
    ("soil_ph", "soil pH"), ("soil_type", "soil"),
    ("fertilizer", "fertiliser"), ("zones", "zones"),
    ("disease_susceptibility", "known to suffer from"),
    ("is_evergreen", "evergreen"), ("plant_type", "type"),
    ("bloom_months", "blooms (12-char mask, Jan first)"),
]

PROMPT = """You are looking at a photograph of one plant in a home garden.

WHAT THE GARDEN'S RECORD SAYS ABOUT THIS PLANT:
{facts}

Answer in exactly three sections, with these headings and nothing else:

VISIBLE:
What you can actually see of this plant's foliage, form and flowers. Only
what is in the photograph.

CONSISTENT WITH:
Any symptom you can see, and what it is typically associated with. If a
colour or form is normal for this species or cultivar, say so plainly --
many plants have red new growth or silver foliage by nature. If you see
nothing wrong, write "Nothing visible."

WOULD CONFIRM IT:
What a person should check, test or look at next. Be specific and cheap
where possible.

Rules. Do not diagnose. Do not name a deficiency or disease as fact -- a
photograph cannot separate nitrogen shortage from waterlogging, because
both yellow a leaf. Do not comment on anything the photograph does not
show. If the plant in the photograph does not match the record above, say
so first.{history}"""


def ensure(cur):
    cur.execute(SCHEMA)


def facts_for(plant):
    """The record, as lines the model can use. Empty columns are omitted
    rather than sent as blanks -- "soil pH: " invites it to fill one in."""
    out = [f"- name: {plant.get('plant_name') or 'unknown'}"]
    bot = plant.get("botanical_name")
    if bot:
        out.append(f"- botanical name: {bot}")
    for col, label in CONTEXT:
        v = (plant.get(col) or "").strip() if isinstance(plant.get(col), str) \
            else plant.get(col)
        if v:
            out.append(f"- {label}: {v}")
    return "\n".join(out)


def split_sections(text):
    """Pull the three headings apart, and never lose the answer.

    A model that ignores the format must not produce an empty record: if a
    heading is missing, everything goes in `looked_at` so the page shows
    the real answer rather than three blank boxes.
    """
    keys = {"VISIBLE": "looked_at", "CONSISTENT WITH": "consistent",
            "WOULD CONFIRM IT": "confirm"}
    found, current = {}, None
    for line in (text or "").splitlines():
        head = line.strip().rstrip(":").upper()
        if head in keys:
            current = keys[head]
            found[current] = []
            continue
        if current:
            found[current].append(line)
    if not found:
        return {"looked_at": (text or "").strip(), "consistent": "", "confirm": ""}
    return {k: "\n".join(found.get(k, [])).strip()
            for k in ("looked_at", "consistent", "confirm")}


def ask(image_bytes, plant, previous=None):
    """One check-up. Returns the three sections plus the raw answer."""
    if len(image_bytes) > MAX_UPLOAD:
        raise ValueError("that photograph is too large")

    hist = ""
    if previous:
        # THE TREND IS THE POINT. Told what was seen last time, the model
        # can say "worse than before", which one photograph never can.
        hist = ("\n\nWHAT WAS SEEN HERE PREVIOUSLY, on "
                f"{previous['asked_at']}:\n{previous['looked_at']}\n"
                "If anything has clearly changed since then, say so in "
                "VISIBLE.")

    # FREE CLOUD MODELS ONLY (freeai.py), never a local GPU model.
    # Raises freeai.Busy when every free model is at its limit; the
    # service turns that into "try again later".
    import freeai
    text = freeai.chat([{"role": "user", "content": [
        {"type": "text",
         "text": PROMPT.format(facts=facts_for(plant), history=hist)},
        {"type": "image_url", "image_url": {
            "url": "data:image/jpeg;base64,"
                   + base64.b64encode(image_bytes).decode()}},
    ]}], max_tokens=600, timeout=TIMEOUT)
    out = split_sections(text)
    out["raw"] = text
    return out
