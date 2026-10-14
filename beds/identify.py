# WHAT THIS FILE DOES (plain English)
# "What plant is this?" The AI describes the photo, then this looks for
# matching plants in YOUR catalogue first, and only suggests looking on the
# web when nothing in the garden fits. It never writes anything.
"""What plant is this? Ask the garden before asking the world.

THE ORDER IS THE WHOLE IDEA. A photograph taken in Jun's garden is, far
more often than not, a plant Jun already owns -- 483 of them, catalogued,
with the cultivar name he actually bought and the bed it went into. A
general plant identifier cannot know any of that. It will happily answer
"Hydrangea paniculata" when the catalogue says *Little Lime Punch*, and
the second answer is the one worth having.

So: look in the catalogue first, on the names AND on what the plant looks
like, and only go outward when nothing here fits.

NEVER ONE NAME. Every answer is a list with a confidence and a reason,
because identification from a photograph is a guess and presenting a guess
as a fact is how a catalogue fills up with confidently wrong records. The
caller decides; this only narrows.

NOTHING HERE WRITES. Same rule as the rest of the ingest path.
"""
from __future__ import annotations

import difflib
import json
import re

from garden_ingest.vocab import canon_key

# What the vision model is asked for. Deliberately NOT "name this plant":
# a description can be matched against a catalogue, whereas a name can only
# be matched against itself -- and if the model's name is wrong, a name-only
# answer takes the search down with it.
LOOK = """Look at this photograph of a plant and describe what you can actually
see. This is for matching against a specific garden's catalogue, so describe
rather than identify.

Answer with JSON only:
{
  "names": ["most likely common or botanical names, best first, up to 4"],
  "plant_type": "tree | shrub | perennial | annual | vine | grass | bulb, if clear",
  "leaf": "shape, arrangement, edge, colour -- in plain words",
  "flower": "colour and form if any are visible, otherwise null",
  "habit": "upright, mounded, trailing, climbing...",
  "notable": "anything distinctive a gardener would recognise it by",
  "confidence": 0.0
}

Rules:
- If it is not a plant, answer {}.
- "confidence" is your honest confidence in the FIRST name, 0 to 1. A
  photograph without flowers rarely deserves more than 0.6.
- Do not invent a cultivar name. Cultivars are not identifiable from a
  photograph, and the catalogue is where they come from.
"""

# Which columns a name might be hiding in. `plant_name` holds the cultivar
# as bought, which is what a person actually says out loud.
NAME_COLS = ["plant_name", "botanical_name", "genus", "species", "family"]

STOP = re.compile(r"\b(the|a|an|of|and|or|var|subsp|cv|f|x)\b")


def _key(s: str) -> str:
    return canon_key(STOP.sub(" ", (s or "").lower()))


def search_catalogue(cursor, names: list, table: str = "data_source",
                     limit: int = 6) -> list:
    """The 483 first. Returns candidates with a score and a reason.

    MATCHES ON EVERY NAME COLUMN, not just plant_name -- somebody saying
    "the redbud" is naming a species, "Cercis" a genus, and "The Rising Sun"
    a cultivar, and all three have to find the same row.
    """
    wanted = [n for n in (names or []) if (n or "").strip()]
    if not wanted:
        return []

    cols = ", ".join("`{}`".format(c) for c in NAME_COLS)
    cursor.execute(
        # `picture`, NOT `photo`. The read-only API renames it on the way
        # out, which is why the wrong name looked right.
        "SELECT id, {}, flower_bed, bloom_color, plant_type, died, picture "
        "FROM `{}`".format(cols, table))
    rows = cursor.fetchall()

    scored = {}
    for want in wanted:
        wk = _key(want)
        if not wk:
            continue
        for r in rows:
            for col in NAME_COLS:
                have = (r.get(col) or "").strip()
                if not have:
                    continue
                hk = _key(have)
                if not hk:
                    continue
                if hk == wk:
                    s, why = 1.0, "{} is exactly {!r}".format(col, have)
                elif wk in hk or hk in wk:
                    # A CONTAINMENT MATCH IS STRONG HERE and weak elsewhere.
                    # "redbud" inside "The Rising Sun Redbud" is the normal
                    # way a person refers to a plant they own.
                    s = 0.88
                    why = "{} contains it: {!r}".format(col, have)
                else:
                    s = difflib.SequenceMatcher(None, wk, hk).ratio()
                    if s < 0.82:
                        continue
                    why = "{} is close: {!r}".format(col, have)
                # Earlier names from the model are likelier, so a match on
                # the second guess is worth slightly less than on the first.
                s *= 1.0 - 0.06 * wanted.index(want)
                prev = scored.get(r["id"])
                if not prev or s > prev["score"]:
                    scored[r["id"]] = {
                        "id": r["id"], "score": round(s, 3), "why": why,
                        "matched": want,
                        "plant_name": r.get("plant_name"),
                        "botanical_name": r.get("botanical_name"),
                        "plant_type": r.get("plant_type"),
                        "flower_bed": r.get("flower_bed"),
                        "bloom_color": r.get("bloom_color"),
                        "photo": r.get("picture"),
                        # `died` is a NEGATIVE COUNT, not a flag. Said
                        # plainly, because offering a plant that is gone as
                        # the answer without saying so is worse than not
                        # offering it.
                        "in_garden": str(r.get("died") or "0") in ("0", "", "None"),
                    }

    out = sorted(scored.values(), key=lambda c: -c["score"])
    return out[:limit]


def _json_from(raw: str) -> dict:
    if not raw:
        return {}
    text = raw.strip()
    if text.startswith("```"):
        text = text.strip("`").split("\n", 1)[-1].rsplit("```", 1)[0]
    try:
        return json.loads(text[text.index("{"):text.rindex("}") + 1])
    except (ValueError, IndexError):
        return {}


def look(describe_image, image_data_url: str) -> dict:
    """A photograph -> what can be seen in it, and some guesses at a name."""
    return _json_from(describe_image(image_data_url, LOOK))


CONFIRM = """A photograph of a plant was described like this:

{seen}

These plants from the garden's own catalogue are possible matches:

{options}

Which of them, if any, is the plant in the photograph?

Answer with JSON only: {{"id": <the catalogue id, or null>, "confidence": 0.0,
"why": "one short sentence"}}

Rules:
- Answer null unless the description genuinely fits. A wrong match here
  becomes a wrong record, and "not in the catalogue" is a useful answer.
- You may only choose from the ids listed. Do not invent one.
"""


def confirm(seen: dict, candidates: list, llm_call) -> dict:
    """Ask the model to choose between catalogue rows, or to decline.

    THE SAME GUARD AS semantic.py: the answer is rejected unless it is one
    of the ids offered. A model allowed to volunteer an id will eventually
    volunteer one that does not exist.
    """
    if not candidates:
        return {}
    ids = {c["id"] for c in candidates}
    options = "\n".join(
        "  id {}: {} ({}) -- {}, {}, bed {}".format(
            c["id"], c.get("plant_name") or "unnamed",
            c.get("botanical_name") or "no botanical name",
            c.get("plant_type") or "type unrecorded",
            c.get("bloom_color") or "bloom unrecorded",
            c.get("flower_bed") or "unsorted")
        for c in candidates)
    got = _json_from(llm_call(CONFIRM.format(
        seen=json.dumps(seen, indent=1)[:1500], options=options)))
    if not got or got.get("id") not in ids:
        return {}
    try:
        conf = max(0.0, min(float(got.get("confidence") or 0), 1.0))
    except (TypeError, ValueError):
        conf = 0.0
    return {"id": got["id"], "confidence": conf,
            "why": str(got.get("why") or "")[:200]}


def identify(cursor, llm_call, seen: dict, table: str = "data_source") -> dict:
    """Everything known, ordered: this garden first, then the wider world.

    Returns candidates and a verdict. `outward` is left for the caller to
    run -- a web lookup costs seconds and should not happen when the answer
    was in the catalogue all along.
    """
    names = list(seen.get("names") or [])
    local = search_catalogue(cursor, names, table)
    picked = confirm(seen, local[:6], llm_call) if local else {}

    if picked:
        for c in local:
            if c["id"] == picked["id"]:
                c["chosen"] = True
                c["model_confidence"] = picked["confidence"]
                c["model_why"] = picked["why"]

    # THE THRESHOLD IS WHERE THE DECISION LIVES. Below it the answer goes
    # outward; above it the garden already knew. 0.7 because a containment
    # match scores 0.88 and a merely-similar string scores low 0.8s -- the
    # line sits under the first and over nothing accidental.
    best = local[0]["score"] if local else 0.0
    settled = bool(picked and picked["confidence"] >= 0.55) or best >= 0.95

    return {
        "seen": seen,
        "candidates": local,
        "in_catalogue": settled,
        "verdict": picked or None,
        # Said in words, because a caller that only prints the first
        # candidate would otherwise present a guess as a finding.
        "note": ("This looks like a plant already in the garden."
                 if settled else
                 "Nothing in the garden clearly matches. Worth looking "
                 "outward -- and the names below are the model's guesses "
                 "from the photograph, not a catalogue record."),
        "search_outward": None if settled else (names[0] if names else None),
    }
