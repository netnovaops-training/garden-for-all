"""The scan queue, on the PC: take waiting scans from the website, read the tag, find the plant, send
back a best match and likely matches. Run by the desktop helper (worker.py) between plant jobs.

For each scan (the pictures are fetched from the website; nothing is kept on this PC):
  1. READ    tagocr.read_tag: lines of text from the OCR service, fields from the lines; the vision
             model only when the reading is weak.
  2. SEARCH  at the same time, in threads: the garden's own plants (catalog), the image index (a photo
             of the plant, or a tag too weak to read), the internet (webfind). Each one that is not
             there is skipped and a note says so; the scan still gets an answer.
  3. MATCH   matcher.assemble: a best match and 2 to 4 likely ones, each with a confidence.
The website decides if it is "matched" (one tap) or "needs you" (judge in scan_queue.rs).
"""
from concurrent.futures import ThreadPoolExecutor

import httpx

from . import batch, catalog, imgindex, matcher, tagocr, webfind


def _images(photos: list[dict], kind: str, weak: bool, client: httpx.Client | None,
            hints: str = "") -> tuple[list[dict], list[str]]:
    """Image index rows for the plant pictures (always for a plant photo; for a tag only when the
    reading was weak). `hints` is what the tag did say (a name), to help the index re-rank. The
    notes say when the index is not running."""
    use = [p for p in photos if p["side"] == "plant"] or ([] if kind == "plant" else photos[:1] if weak else [])
    if not use:
        return [], []
    if imgindex.status(client) is None:
        return [], ["the image index is not running: photo matching was skipped"]
    rows: list[dict] = []
    for p in use:
        rows += imgindex.search(p["bytes"], client, hints=hints)
    best: dict[tuple, dict] = {}
    for r in rows:
        k = (r.get("set"), r.get("plant_id"), r.get("botanical_name"))
        if k not in best or r["score"] > best[k]["score"]:
            best[k] = r
    return sorted(best.values(), key=lambda r: -r["score"])[:5], []


def process(item: dict, photos: list[dict], cat: catalog.Catalog, say=lambda _s: None,
            client: httpx.Client | None = None, use_gemma: bool = True) -> dict:
    """The result for one scan. `photos` are {side, bytes}; `item` has kind, barcode, qty.
    Returns {ocr, candidates, notes}."""
    kind = item.get("kind", "tag")
    notes: list[str] = []
    tag_photos = [p for p in photos if p["side"] in ("front", "back")]
    ocr = {"fields": {}, "lines": [], "engine": "", "conf": 0.0, "weak": True, "barcodes": [], "notes": []}
    if tag_photos:
        say("reading tag")
        ocr = tagocr.read_tag(tag_photos, hint=item.get("barcode", ""), client=client, use_gemma=use_gemma)
        notes += ocr["notes"]
    elif item.get("barcode"):
        ocr["fields"] = {"barcode": item["barcode"]}
    fields = ocr["fields"]
    say("looking up")
    garden = cat.lookup(fields.get("name", ""), fields.get("botanical", ""), fields.get("barcode", ""))
    known = {(p["set"], p["id"]): p for p in cat.plants}
    top_local = garden[0]["confidence"] if garden else 0.0
    web_wanted = bool(fields.get("name") or fields.get("botanical")) and top_local < 0.9
    img_wanted = bool([p for p in photos if p["side"] == "plant"]) or (tag_photos and ocr["weak"])
    with ThreadPoolExecutor(2, thread_name_prefix="scan") as pool:
        hints = " ".join(x for x in (fields.get("name", ""), fields.get("botanical", "")) if x)
        f_img = pool.submit(_images, photos, kind, ocr["weak"], client, hints) if img_wanted else None
        f_web = pool.submit(webfind.lookup, fields) if web_wanted else None
        if f_img:
            say("matching photo")
        images, n1 = f_img.result() if f_img else ([], [])
        web = f_web.result() if f_web else {"candidates": [], "confirms": False, "botanical": "", "url": "", "notes": []}
    notes += n1 + web.get("notes", [])
    cands = matcher.assemble(fields, garden, images, web, known)
    if not cands:
        notes.append("nothing matched: type the plant in by hand")
    return {"ocr": {**fields, "engine": ocr["engine"], "lines": ocr["lines"][:40]}, "candidates": cands, "notes": notes}


def run_once(site, take: int = 5) -> int:
    """Take up to `take` waiting scans, process each, send each result. Gives how many were done."""
    items = site.take_scans(take)
    if not items:
        return 0
    cat = catalog.Catalog(site.catalog())
    done = 0
    for it in items:
        jid = int(it["id"])

        def say(step: str, _j: int = jid) -> None:
            try:
                site.post_scan(_j, "progress", {"step": step})
            except httpx.HTTPError:
                pass   # progress is only for the screen
        try:
            photos = [{"side": p["side"], "bytes": site.scan_photo(p["id"])} for p in it.get("photos", [])]
            res = process(it, photos, cat, say)
            site.post_scan(jid, "result", {"result": res})
            done += 1
        except httpx.HTTPError as e:
            batch.log(f"scan {jid}: site problem {type(e).__name__}")
            _fail(site, jid, f"could not reach the website while reading ({type(e).__name__})")
        except Exception as e:  # noqa: BLE001 -- one bad scan never stops the others
            batch.log(f"scan {jid}: {type(e).__name__}: {e}")
            _fail(site, jid, f"the helper could not read this scan ({type(e).__name__})")
    return done


def _fail(site, jid: int, why: str) -> None:
    try:
        site.post_scan(jid, "fail", {"error": why})
    except httpx.HTTPError:
        pass


def run_all(site, take: int = 5, limit: int = 200) -> int:
    """Keep taking scans until none are waiting (new ones may arrive meanwhile). Gives the total."""
    total = 0
    while total < limit:
        n = run_once(site, take)
        if n == 0:
            break
        total += n
    return total
