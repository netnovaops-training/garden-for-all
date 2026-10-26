"""Put the clues together: a BEST MATCH and a few LIKELY MATCHES, each with a confidence.

Clues come from four places and are combined per plant ("two agreeing clues are worth more than
one", catalog.combine):
  garden   the garden's own plants (current and wish list): barcode, name, Latin name
  image    the image index's photo matches (a score of how alike the pictures are)
  tag      what the tag itself says: a NEW plant from its name and Latin name (the tag is read, not
           matched against anything), stronger when the barcode check digit is right
  web      a page on the internet about this plant, which also confirms what the tag says

The numbers are choices, not measurements: the website decides "matched" (one tap) only for a best
match at 75% or more that is 15 points ahead of the next one; everything else is left to Jun.
"""
import os

from . import catalog, tagparse
from .catalog import combine

# Image-index scores below this say nothing; at 0.9 and above they are strong.
IMAGE_FLOOR = float(os.environ.get("PLANTAI_IMAGE_FLOOR", "0.55"))
IMAGE_SPAN = float(os.environ.get("PLANTAI_IMAGE_SPAN", "0.35"))
IMAGE_CAP = 0.8      # a photo alone is never "sure"


def image_confidence(score: float) -> float:
    """From an index score to a confidence."""
    return round(max(0.0, min(1.0, (score - IMAGE_FLOOR) / IMAGE_SPAN)) * IMAGE_CAP, 3)


def _cand(name, botanical, conf, source, why, plant=None, url="", photo="", set_="", plant_id=None) -> dict:
    g, s, _ = catalog.split_botanical(botanical or "")
    if plant:
        g, s = plant.get("genus") or g, plant.get("species") or s
    return {"name": name or botanical, "botanical": botanical or "", "genus": (g or "").capitalize(),
            "species": s or "", "confidence": round(conf, 3), "source": source, "set": set_,
            "plant_id": plant_id, "photo": photo, "url": url, "why": why}


def tag_confidence(fields: dict, web_confirms: bool) -> float:
    """How sure the tag alone makes us of a NEW plant: Latin name 0.5, name 0.3, grower 0.1, a good
    barcode check digit 0.1, a web page that confirms it 0.5 (combined, not added)."""
    parts = []
    if fields.get("botanical"):
        parts.append(0.5)
    if fields.get("name"):
        parts.append(0.3)
    if fields.get("vendor"):
        parts.append(0.1)
    code = "".join(c for c in fields.get("barcode", "") if c.isdigit())
    if code and tagparse.ean_ok(code):
        parts.append(0.1)
    if web_confirms and parts:
        parts.append(0.5)
    return combine(parts)


def assemble(fields: dict, garden: list[dict], images: list[dict], web: dict, known: dict | None = None) -> list[dict]:
    """The candidates, best first (at most 5). `garden` is catalog.Catalog.lookup output, `images` the
    image index rows, `web` the webfind answer; `known` maps (set, plant_id) to the garden's own row
    (to show an image match with the garden's own words)."""
    found: dict[str, dict] = {}

    def add(key: str, cand: dict) -> None:
        if key in found:
            old = found[key]
            old["confidence"] = round(combine([old["confidence"], cand["confidence"]]), 3)
            old["why"] = ", ".join(x for x in (old["why"], cand["why"]) if x and x not in old["why"])
            old["photo"] = old["photo"] or cand["photo"]
            if old["source"] != cand["source"] and old["source"] != "garden":
                old["source"] = cand["source"] if cand["source"] == "garden" else old["source"]
        else:
            found[key] = cand

    for g in garden:
        p = g["plant"]
        add(f"L:{p['set']}:{p['id']}", _cand(p.get("name"), p.get("botanical"), g["confidence"],
                                             "garden" if p["set"] == "current" else "wishlist", g["why"],
                                             plant=p, photo=p.get("photo", ""), set_=p["set"], plant_id=p["id"]))
    for im in images:
        pid, st = im.get("plant_id"), im.get("set")
        conf = image_confidence(float(im["score"]))
        if conf <= 0:
            continue
        own = (known or {}).get((st, pid)) if pid and st in ("current", "future") else None
        why = f"the photo looks like it ({round(float(im['score']) * 100)}% alike)"
        if own:
            add(f"L:{st}:{pid}", _cand(own.get("name"), own.get("botanical"), conf,
                                       "garden" if st == "current" else "wishlist", why, plant=own,
                                       photo=own.get("photo", ""), set_=st, plant_id=pid))
        else:
            add("N:" + catalog.norm(im.get("botanical_name") or im.get("plant_name") or ""),
                _cand(im.get("plant_name"), im.get("botanical_name"), conf, "image", why, photo=im.get("photo", "")
                      if str(im.get("photo", "")).startswith("http") else ""))
    for w in web.get("candidates", []):
        add("N:" + catalog.norm(w["botanical"] or w["name"]), w)
    # What the tag says, as a new plant, unless a garden plant already is that plant.
    if fields.get("name") or fields.get("botanical"):
        bot = fields.get("botanical") or web.get("botanical") or ""
        c = tag_confidence({**fields, "botanical": bot}, bool(web.get("confirms")))
        same = [k for k, v in found.items() if k.startswith("L:") and (
            (bot and v["botanical"] and catalog.botanical_sim(bot, v["botanical"]) >= 0.85)
            or (fields.get("name") and catalog.sim(fields["name"], v["name"]) >= 0.85))]
        if not same and c > 0:
            why = "read from the tag" + (", and a web page agrees" if web.get("confirms") else "")
            add("N:" + catalog.norm(bot or fields["name"]),
                _cand(fields.get("name"), bot, c, "tag", why, url=web.get("url", "")))
    out = sorted(found.values(), key=lambda c: -c["confidence"])
    return out[:5]
