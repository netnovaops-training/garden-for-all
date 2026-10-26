"""How well does the tag reader read the made-up tags (synth.py)? Used to pick the OCR engine.

  python -m plantai ocr-eval --tags DIR [--engine tesseract|rapid|both|all]

For each tag it reads the front and back pictures the way the scan queue does (the OCR service,
then tagparse; no vision model) and compares the five fields with `truth.json`. Two scores per
field: EXACT (the same after case, punctuation and spaces are ignored) and CLOSE (at least 85%
alike, so one wrong letter still counts). `item` is the share of tags with all five fields close.
Barcodes and sizes must match exactly. It runs on made-up tags: it shows the engines work, not how
they will do on real photos.
"""
import json
import re
from pathlib import Path

from . import catalog, tagocr, tagparse

FIELDS = ("name", "botanical", "size", "vendor", "barcode")


def canon_size(s: str) -> str:
    """A pot size in one form: '#1' and '1 gal' both give '1g'; '4 in pot' gives '4in'."""
    t = (s or "").lower().replace("gallon", "gal").replace("inch", "in").replace('"', "in")
    m = re.search(r"#\s?(\d+)", t)
    if m:
        return m.group(1) + "g"
    m = re.search(r"(\d+(?:\.\d+)?)\s?(gal|qt|in)", t)
    return (m.group(1) + m.group(2).replace("gal", "g")) if m else re.sub(r"\W", "", t)


def same(field: str, got: str, want: str) -> tuple[bool, bool]:
    """(exact, close) for one field."""
    if field == "barcode":
        # A UPC-A (12 digits) is the same code as the EAN-13 with a 0 in front.
        ok = bool(got) and re.sub(r"\D", "", got).lstrip("0") == re.sub(r"\D", "", want).lstrip("0")
        return ok, ok
    if field == "size":
        ok = bool(got) and canon_size(got) == canon_size(want)
        return ok, ok
    a, b = catalog.norm(got), catalog.norm(want)
    exact = bool(a) and a.replace(" ", "") == b.replace(" ", "")
    if field == "botanical":
        return exact, exact or (bool(got) and catalog.botanical_sim(got, want) >= 0.85)
    return exact, exact or (bool(a) and catalog.sim(got, want) >= 0.85)


def read_tag(folder: Path, engine: str) -> tuple[dict, int]:
    """The five fields read from one tag folder (front.jpg, back.jpg), and the milliseconds taken."""
    front, back, codes, ms = [], [], [], 0
    for side, into in (("front", front), ("back", back)):
        f = folder / f"{side}.jpg"
        # The engine's raw answer is kept next to the picture, so changing only the parsing rules
        # can be re-scored in seconds. Delete the .ocr-<engine>.json files to read everything again.
        cache = folder / f"{side}.ocr-{engine}.json"
        if cache.is_file():
            r = json.loads(cache.read_text(encoding="utf-8"))
        else:
            r = tagocr.from_service(f.read_bytes(), engine=engine) if f.is_file() else None
            if r:
                cache.write_text(json.dumps(r), encoding="utf-8")
        if r:
            ms += r.get("ms", 0)
            into.extend(r.get("lines", []))
            codes += [b["text"] for b in r.get("barcodes", []) if b.get("text")]
    hint = next((c for c in codes if c.isdigit()), "")
    ff, fb = tagparse.parse(front, hint), tagparse.parse(back, hint)
    fields = {k: ff.get(k) or fb.get(k) or "" for k in ("size", "vendor", "barcode")}
    fields["botanical"] = tagparse.best_botanical(ff.get("botanical", ""), fb.get("botanical", ""))
    fields["name"] = ff.get("name", "")
    return fields, ms


def evaluate(tags: Path, engine: str) -> dict:
    """Score one engine over every tag in `tags`: {n, exact, close (percent per field), item, ms_per_tag, misses}."""
    truth = json.loads((tags / "truth.json").read_text(encoding="utf-8"))
    ex, cl = dict.fromkeys(FIELDS, 0), dict.fromkeys(FIELDS, 0)
    full, ms, misses = 0, 0, []
    for t in truth:
        got, took = read_tag(tags / t["dir"], engine)
        ms += took
        all_close = True
        for f in FIELDS:
            e, c = same(f, got.get(f, ""), t[f])
            ex[f] += e
            cl[f] += c
            all_close &= c
            if not c:
                misses.append((t["dir"], f, got.get(f, ""), t[f]))
        full += all_close
    n = max(1, len(truth))
    return {"engine": engine, "n": len(truth),
            "exact": {k: round(100 * v / n, 1) for k, v in ex.items()},
            "close": {k: round(100 * v / n, 1) for k, v in cl.items()},
            "item": round(100 * full / n, 1), "ms_per_tag": ms // n, "misses": misses}


def report(tags: Path, engines: list[str]) -> str:
    """The engines side by side, then the first mistakes of each."""
    rows = [evaluate(tags, e) for e in engines]
    out = [f"{rows[0]['n']} made-up tags (front + back each)",
           f"{'engine':<10}" + "".join(f"{f:>12}" for f in FIELDS) + f"{'item':>8}{'ms/tag':>9}"]
    for r in rows:
        out.append(f"{r['engine']:<10}" + "".join(f"{r['close'][f]:>6.1f}/{r['exact'][f]:<5.1f}" for f in FIELDS)
                   + f"{r['item']:>8.1f}{r['ms_per_tag']:>9}")
    out.append("(each field: close% / exact%; item = all five fields close)")
    for r in rows:
        out.append(f"-- first mistakes, {r['engine']}")
        out += [f"   {d} {f}: read {g!r}, printed {w!r}" for d, f, g, w in r["misses"][:10]]
    return "\n".join(out)
