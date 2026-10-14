# WHAT THIS FILE DOES (plain English)
# Before a tag adds a new plant, this checks whether the garden already has
# it ("Blue Angel Hosta" and "Hosta 'Blue Angel'" count as the same plant),
# and also looks in the wishlist table future_plants.
"""Has this plant already been added?

JASON, 2026-09-24: "I don't want to add duplicates so if there are some
don't add them. Just keep adding new ones ... there's about 200 new plants
to add."

TWO HUNDRED TAGS IS WHERE DUPLICATES COME FROM. One at a time, a person
remembers what they just did. Across an afternoon and a couple of sessions
they do not, and the catalogue ends up with "Blue Angel Hosta" twice with
different spellings -- which then splits every count, every price total and
every wiki page that groups by name.

MATCHED ON A NORMALISED NAME, NOT ON THE RAW STRING. Nursery tags write the
same plant as "Hosta 'Blue Angel'", "Blue Angel Hosta" and "HOSTA BLUE
ANGEL". Comparing raw strings finds none of those; comparing sorted
significant words finds all three.

IT CHECKS THE FUTURE LIST TOO. `future_plants` holds 6,534 plants Jason has
already catalogued but Jun does not own yet. A tag that matches one of those
is not a duplicate -- it is that row finally arriving, and saying so lets it
be promoted with the details it already has instead of being retyped.

IT NEVER DECIDES ALONE. It reports what it found and how sure it is; adding
or skipping is the caller's choice, and an uncertain match is shown to a
person rather than silently dropped. A wrongly skipped plant is invisible --
it simply never appears - which is the worst failure available here.
"""
from __future__ import annotations

import re

# Words that carry no identity. "Hosta 'Blue Angel' 1 Gallon Perennial" and
# "Blue Angel Hosta" are the same plant; the size and the habit are noise.
NOISE = {
    "gallon", "gal", "quart", "qt", "inch", "in", "pot", "potted", "plant",
    "plants", "perennial", "annual", "shrub", "tree", "the", "and", "a",
    "of", "new", "improved", "series", "collection", "live", "hardy",
    "dwarf",
}


def key(name):
    """A comparable fingerprint for a plant name."""
    t = re.sub(r"[^a-z0-9 ]", " ", str(name or "").lower())
    words = [w for w in t.split() if w and w not in NOISE and len(w) > 1]
    return " ".join(sorted(set(words)))


def _overlap(a, b):
    sa, sb = set(a.split()), set(b.split())
    if not sa or not sb:
        return 0.0
    return len(sa & sb) / max(len(sa), len(sb))


def find(cur, name, table="data_source"):
    """Where this plant already exists, if anywhere.

    Returns {"status": one of already_have / in_future_list / maybe / new,
             "match": row or None, "score": 0..1}
    """
    k = key(name)
    if not k:
        return {"status": "new", "match": None, "score": 0.0}

    cur.execute(f"SELECT id, plant_name, botanical_name FROM {table} "
                "WHERE plant_name IS NOT NULL AND plant_name<>''")
    live = cur.fetchall()

    best, score = None, 0.0
    for r in live:
        for candidate in (r["plant_name"], r.get("botanical_name")):
            s = _overlap(k, key(candidate))
            if s > score:
                best, score = r, s

    if score >= 0.85:
        return {"status": "already_have", "match": best, "score": round(score, 2)}

    # Only now is the much larger future list consulted -- 6,534 rows, and
    # the live catalogue is the answer that matters more.
    cur.execute("SELECT id, plant_name, botanical_name, company, quantity "
                "FROM future_plants WHERE plant_name IS NOT NULL "
                "AND plant_name<>''")
    fbest, fscore = None, 0.0
    for r in cur.fetchall():
        for candidate in (r["plant_name"], r.get("botanical_name")):
            s = _overlap(k, key(candidate))
            if s > fscore:
                fbest, fscore = r, s

    if fscore >= 0.85:
        return {"status": "in_future_list", "match": fbest,
                "score": round(fscore, 2)}
    if max(score, fscore) >= 0.6:
        # CLOSE BUT NOT CERTAIN GOES TO A PERSON. Skipping on a 0.6 would
        # quietly lose a real plant; adding on one would create the
        # duplicate this file exists to stop.
        return {"status": "maybe",
                "match": best if score >= fscore else fbest,
                "score": round(max(score, fscore), 2)}
    return {"status": "new", "match": None, "score": round(max(score, fscore), 2)}
