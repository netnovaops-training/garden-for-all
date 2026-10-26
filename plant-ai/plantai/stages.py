"""The four stages for one plant, each saved in state.db so a stopped run carries on:

  discover  find the grower's page and the species pages (web, polite)
  fill      phi4 reads every page, judges disagreements, normalises (never overwrites a value)
  images    up to 5 photos from those pages (only for plants that have none yet)
  check     gemma4:12b re-checks every value against its quote, the plant's identity, the photos

A "set" (pset) is a group of plants: "new" (and "future", our own wishlist run) get photos;
"current" plants already have the garden's own photos, which are only checked.
"""
from pathlib import Path

from . import check, db, discover, fill, images, norm, pages
from .config import CFG

PHOTO_SETS = {"new", "future"}


def empty(v) -> bool:
    return v is None or str(v).strip() == ""


def title_ok(rec: dict, page: dict | None) -> bool:
    if not page:
        return False
    hay = set(discover.toks(page["title"] + " " + page["url"].replace("/", " ").replace("-", " ")))
    cv = set(discover.cultivar_tokens(rec))
    g, sp = discover.genus_species(rec)
    if cv:
        return cv <= hay
    return bool(g) and g.lower() in hay and (not sp or sp in hay)


def do_discover(pset: str, pid: int, rec: dict) -> dict:
    d = db.get(pset, pid, "discover")
    if d is not None:
        return d
    vendor = discover.vendor_page(rec)
    vpage = pages.read(vendor) if vendor else None
    key, sp = discover.species_pages(rec)
    product = vendor if vpage and title_ok(rec, vpage) else None
    if not product:
        for u in sp:
            if title_ok(rec, pages.read(u)):
                product = u
                break
    d = {"vendor": vendor if vpage else None, "vendor_title": vpage["title"] if vpage else "",
         "species_key": key, "species": sp, "product_url": product,
         "product_kind": ("vendor" if product == vendor else "botanical") if product else None}
    db.put(pset, pid, "discover", d)
    return d


def do_fill(pset: str, pid: int, rec: dict) -> dict:
    f = db.get(pset, pid, "fill")
    if f is not None:
        return f
    d = do_discover(pset, pid, rec)
    named_cultivar = bool(discover.cultivar_tokens(rec))
    cands: dict[str, list] = {}
    unnorm, sources = [], []
    for url, kind in [(d["vendor"], "vendor")] + [(u, "species") for u in d["species"]]:
        if not url:
            continue
        x = fill.extract(url)
        if not x or x.get("dead"):
            continue
        sources.append({"url": x["url"], "title": x["title"], "kind": kind})
        for it in x["items"]:
            if kind != "vendor" and named_cultivar and it["f"] in fill.CULTIVAR_ONLY:
                continue
            if it["v"] is None:
                unnorm.append({"f": it["f"], "raw": it["raw"], "url": x["url"], "why": it["note"]})
                continue
            cands.setdefault(it["f"], []).append({"v": it["v"], "quote": it["quote"], "url": x["url"],
                                                  "host": x.get("host", ""), "kind": kind})
    judged = fill.judge(rec, cands)
    fill.derive(judged)
    if named_cultivar and "native_status" not in judged and empty(rec.get("native_status")):
        judged["native_status"] = {"v": "Cultivar", "conf": "medium", "sources": [],
                                   "quotes": [f"record name: {rec.get('plant_name')} / {rec.get('botanical_name')}"],
                                   "why": "derived: named cultivar (DB convention)"}
    fills, contra, confirmed = {}, [], []
    for fld, j in judged.items():
        if fld in norm.READ_ONLY:
            continue
        ev = rec.get(fld)
        if empty(ev):
            fills[fld] = j
            continue
        existing, _ = norm.norm(fld, ev)
        if fld in norm.FREE:
            continue  # free text: not compared word-for-word
        if existing == j["v"]:
            confirmed.append(fld)
        else:
            contra.append({"f": fld, "existing": ev, "source_value": j["v"], "sources": j["sources"],
                           "quote": (j["quotes"] or [""])[0], "conf": j["conf"]})
    f = {"fills": fills, "contradictions": contra, "confirmed": confirmed, "unnormalized": unnorm,
         "sources": sources, "product_url": d["product_url"], "product_kind": d["product_kind"]}
    db.put(pset, pid, "fill", f)
    return f


def do_images(pset: str, pid: int, rec: dict) -> list:
    im = db.get(pset, pid, "images")
    if im is not None:
        return im
    if pset not in PHOTO_SETS or not empty(rec.get("picture")):
        im = []
    else:
        d = do_discover(pset, pid, rec)
        im, want = [], 5
        # Priority: the vendor's own page, then botanical/extension pages, then (only if still
        # nothing) public social posts. Each source tops up what the previous one lacked.
        srcs = [("vendor", d["vendor"])] + [("botanical", u) for u in d["species"]]
        for kind, url in srcs:
            if not url or len(im) >= want:
                continue
            p = pages.read(url)
            if not p:
                continue
            if kind == "botanical" and discover.cultivar_tokens(rec) and not title_ok(rec, p):
                continue   # a species page's photos can show another cultivar's colour
            got = images.collect(pid, p["images"], want - len(im), start=len(im) + 1)
            for x in got:
                x["credit"] = f"Photo: {p['host']} ({p['title']})"
                x["page"] = p["url"]
                x["kind"] = kind
            im += got
        if not im:
            for img_url, post in discover.social_images(rec):
                got = images.collect(pid, [img_url], 1, start=len(im) + 1)
                for x in got:
                    x["credit"] = f"Public post: {post}"
                    x["page"] = post
                    x["kind"] = "social"
                im += got
                if len(im) >= want:
                    break
    db.put(pset, pid, "images", im)
    return im


def photo_bytes(pset: str, pid: int, rec: dict) -> tuple[list[bytes], list[str]]:
    if pset in PHOTO_SETS:
        ims = db.get(pset, pid, "images") or []
        return [images.preview(CFG.out / x["file"]) for x in ims], [x["src"] for x in ims]
    root = CFG.photos_dir
    if not root:
        return [], []
    names = [n.strip() for n in f"{rec.get('picture') or ''},{rec.get('additional_pictures') or ''}".split(",")
             if n.strip()]
    got, which = [], []
    for n in names[:6]:
        p = Path(root) / n
        # Only files inside the photos folder (a name with ../ in it is skipped).
        if p.is_file() and p.resolve().is_relative_to(Path(root).resolve()):
            try:
                got.append(images.preview(p))
                which.append(n)
            except OSError:
                pass
    return got, which


def do_check(pset: str, pid: int, rec: dict) -> dict:
    c = db.get(pset, pid, "check")
    if c is not None:
        return c
    f = do_fill(pset, pid, rec)
    claims = [{"f": k, "v": j["v"], "quote": " … ".join(j["quotes"]), "url": (j["sources"] or [""])[0],
               "kind": "fill"} for k, j in f["fills"].items()]
    if pset == "current":
        for x in f["contradictions"]:
            claims.append({"f": x["f"], "v": x["existing"], "quote": x["quote"], "url": (x["sources"] or [""])[0],
                           "kind": "existing"})
    srcs = []
    for s in f["sources"]:
        p = pages.read(s["url"])
        if p:
            srcs.append({"url": s["url"], "title": p["title"], "lines": p["lines"]})
    v = check.verify(rec, srcs, claims) if (claims or srcs) else {"same_plant": "unsure",
                                                                  "identity_why": "no sources found", "checks": {}}
    imgs, which = photo_bytes(pset, pid, rec)
    ph = check.vision(rec, imgs)
    res_claims = []
    for k, cl in enumerate(claims, 1):
        ck = v["checks"].get(k, {"ok": "no", "why": "not answered"})
        res_claims.append({**cl, "gemma": ck.get("ok", "no"), "gemma_why": ck.get("why", "")})
    sup = sum(1 for c in res_claims if c["kind"] == "fill" and c["gemma"] == "yes")
    nfill = sum(1 for c in res_claims if c["kind"] == "fill")
    lows = [k for k, j in f["fills"].items() if j["conf"] == "low"]
    bad_ph = [p for p in ph if p.get("reject")]
    summary = (f"Plant: {rec.get('plant_name')} ({rec.get('botanical_name')}). Identity of sources: "
               f"{v['same_plant']} ({v['identity_why']}). New values: {nfill}, supported by quote: {sup}, "
               f"unsupported: {[c['f'] for c in res_claims if c['kind'] == 'fill' and c['gemma'] != 'yes']}. "
               f"Low-confidence (sources disagreed): {lows}. Contradictions with existing values: "
               f"{len(f['contradictions'])}. Photos: {len(ph)} checked, failed: {len(bad_ph)} "
               f"({'; '.join(p['reject'] + ': ' + p.get('why', '') for p in bad_ph)[:300]}); a dropped photo alone "
               f"is not a reason to reject the data. Values not normalisable: {len(f['unnormalized'])}.")
    dec = check.decide(summary)
    c = {"identity": v["same_plant"], "identity_why": v["identity_why"], "claims": res_claims,
         "photos": [{**p, "src": which[p["i"] - 1] if p["i"] - 1 < len(which) else ""} for p in ph],
         "decision": dec["decision"], "reason": dec["reason"], "summary": summary}
    db.put(pset, pid, "check", c)
    return c
