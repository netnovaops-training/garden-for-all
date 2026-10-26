"""Look on the internet for the plant a tag names, so a plant that is not in the garden yet can still
be matched, and a Latin name the tag did not print can be found.

It reuses the polite search and page reader the fill pipeline already has (discover.py, fetch.py,
pages.py): DuckDuckGo with at least 3 seconds between searches, robots.txt respected, every page
fetched once. The pages are only read for who/what the plant is. Filling in its facts happens
later, after Jun confirms, by the normal fill-and-check helper.
"""
import re
from urllib.parse import urlsplit

from . import catalog, discover, pages, tagparse

# Hosts whose pages are about one named plant (growers and plant references): worth more.
GOOD = tuple(discover.VENDORS) + tuple(discover.TRUSTED)


def _host(u: str) -> str:
    return (urlsplit(u).hostname or "").lower()


def _good(host: str) -> bool:
    return any(host.endswith(t) if t.startswith(".") else (host == t or host.endswith("." + t)) for t in GOOD)


def lookup(fields: dict, max_pages: int = 3) -> dict:
    """Search for the plant in `fields` (name, botanical, barcode as read from the tag).
    Gives {"candidates": [...], "confirms": bool, "botanical": str, "url": str, "notes": [...]}.
    `confirms` is True when a page about this very plant states the name and the Latin name the tag
    printed (or the Latin name, when the tag printed none)."""
    name, bot = fields.get("name", ""), fields.get("botanical", "")
    out = {"candidates": [], "confirms": False, "botanical": "", "url": "", "notes": []}
    q = " ".join(x for x in (name, bot) if x).strip()
    if not q:
        q = (fields.get("barcode") or "").strip()
    if not q:
        return out
    try:
        hits = discover.search(f"{q} plant")
    except Exception as e:  # noqa: BLE001 -- search is optional: say so, go on without it
        out["notes"].append(f"web search not available ({type(e).__name__})")
        return out
    want = set(catalog.tokens(f"{name} {bot}"))
    if not want:
        return out
    # Pages from growers and plant references first, social sites never.
    ranked = sorted((h for h in hits if h.get("href") and not discover.NOT_ENGLISH.search(h["href"])
                     and not any(_host(h["href"]).endswith(s) for s in discover.SOCIAL_HOSTS)),
                    key=lambda h: (not _good(_host(h["href"])),))
    seen_keys = set()
    for h in ranked[:max_pages]:
        try:
            page = pages.read(h["href"])
        except Exception:  # noqa: BLE001 -- one bad page never stops the others
            page = None
        if not page:
            continue
        text_lines = [page["title"]] + page["lines"][:60]
        have = set(catalog.tokens(" ".join(text_lines)))
        share = len(want & have) / len(want)
        page_bot = tagparse.find_botanical([{"text": t} for t in text_lines])
        good = _good(page["host"])
        agree = catalog.botanical_sim(bot, page_bot) if bot and page_bot else 0.0
        if share < 0.6:
            continue
        # How sure: the share of the tag's words the page has, more for a grower or reference page,
        # more when its Latin name agrees with the tag's.
        conf = 0.35 * share + (0.15 if good else 0.0) + (0.2 * agree if agree >= 0.7 else 0.0)
        # The name Jun knows is the one on the tag; a page title is only a fallback, without the
        # site's name after a "|" or " - ".
        title = re.split(r"\s+[|–—-]\s+", re.sub(r"\s+", " ", page["title"]).strip())[0]
        nm = name or (title if len(title) <= 80 else "")
        cand_bot = page_bot or bot
        key = catalog.norm(cand_bot or nm)
        if key in seen_keys:
            continue
        seen_keys.add(key)
        g, s, _ = catalog.split_botanical(cand_bot)
        out["candidates"].append({
            "name": nm or cand_bot, "botanical": cand_bot, "genus": g.capitalize(), "species": s,
            "confidence": round(min(0.7, conf), 3), "source": "web", "set": "", "plant_id": None,
            "url": h["href"], "why": f"a page on {page['host']} is about it",
        })
        if share >= 0.8 and good and (agree >= 0.7 or not bot):
            out["confirms"] = True
            out["botanical"] = out["botanical"] or page_bot
            out["url"] = out["url"] or h["href"]
    return out
