"""Build the batch output files from the SQLite state (safe to run any time; partial runs give partial files).

<out>/<set>-plants.json   JSON Pipeline import: top-level array, gemma-supported fills only, never overwrites
<out>/sources.csv         every filled value with confidence, source URL(s) and quote
<out>/discrepancies.csv   everything for the human check
<out>/REVIEW.csv + .md    one row per plant: the model's decision + reason, a blank column for a person
<out>/report.md           counts, fills per field, still missing, photos
"""
import csv
from collections import Counter

from . import db, norm, result
from .config import CFG

DROP = {"qr_code", "barcode_image"}
INTS = {"id", "qty", "died", "new_total", "quantity", "image_verified", "approved_jason", "approved_jun",
        "needs_review"}


def _empty(v) -> bool:
    return v is None or str(v).strip() == ""


def _cast(r: dict) -> None:
    for k in INTS:
        if k in r and not _empty(r[k]):
            try:
                r[k] = int(float(r[k]))
            except (TypeError, ValueError):
                pass
    if not _empty(r.get("average_cost")):
        try:
            r["average_cost"] = float(r["average_cost"])
        except (TypeError, ValueError):
            pass


def sets() -> list[str]:
    return [r[0] for r in db.con().execute("SELECT DISTINCT pset FROM plants ORDER BY pset")]


def build() -> None:
    import json
    out = CFG.out
    out.mkdir(parents=True, exist_ok=True)
    disc_rows, review, src_rows = [], [], []
    stats, per_field = {}, {}
    for pset in sets():
        stats[pset], per_field[pset] = Counter(), Counter()
        recs = []
        for pid, rec in db.plants(pset):
            res = result.plant_result(pset, pid, rec)
            if not res:
                continue
            name = rec.get("plant_name") or ""
            stats[pset]["processed"] += 1
            stats[pset][res["decision"]] += 1
            r = {k: v for k, v in rec.items() if k not in DROP}
            r.update(res["fields"])
            for fld, v in res["fields"].items():
                per_field[pset][fld] += 1
                src_rows.append([pset, pid, name, fld, v, res["confidence"][fld], "yes",
                                 " | ".join(res["sources"][fld]), res["quotes"][fld], ""])
            for fld, x in res["needs_review"].items():
                where = x["quote"] + "  <" + " | ".join(x["sources"]) + ">"
                disc_rows.append([pset, pid, name, fld, x["v"], x["conf"], where, x["why"]])
            for x in res["contradictions"]:
                disc_rows.append([pset, pid, name, x["f"], f"existing={x['existing']!r} source={x['source_value']!r}",
                                  x["conf"], x["quote"][:300], "source contradicts existing value (not overwritten)"])
            for u in res["unnormalized"]:
                disc_rows.append([pset, pid, name, u["f"], u["raw"], "", u["url"], f"value not normalisable: {u['why']}"])
            if res["identity"] and res["identity"] != "yes":
                disc_rows.append([pset, pid, name, "(identity)", rec.get("botanical_name"), res["identity"], "",
                                  f"identity doubtful: {res['identity_why']}"])
            r["product_url"] = res["product_url"]
            r["_enriched_fields"] = sorted(res["fields"])
            r["_missing_fields"] = res["missing"]
            r["_confidence"] = res["confidence"]
            r["_sources"] = res["sources"]
            if res["needs_review"]:
                r["_needs_review"] = {k: x["v"] for k, x in res["needs_review"].items()}
            if res["photos"]:
                r["_photos"] = res["photos"]
            _cast(r)
            recs.append(r)
            review.append([pset, pid, name, rec.get("botanical_name") or "", res["decision"], res["reason"],
                           ", ".join(k for k, v in res["confidence"].items() if v == "low"),
                           ", ".join(res["needs_review"]), res["product_url"] or "",
                           ", ".join(sorted({fl for p in res["photos"] for fl in p["flags"]})), ""])
        (out / f"{pset}-plants.json").write_text(json.dumps(recs, ensure_ascii=False, indent=1), encoding="utf-8")
    _csv("sources.csv", ["set", "plant_id", "plant_name", "field", "value", "confidence", "gemma", "source_urls",
                         "quote", "note"], src_rows)
    _csv("discrepancies.csv", ["set", "plant_id", "plant_name", "field", "value", "confidence", "quote_source_url",
                               "reason"], disc_rows)
    _csv("REVIEW.csv", ["set", "plant_id", "plant_name", "botanical_name", "decision (LLM)", "reason (LLM)",
                        "low_confidence_fields", "needs_review_fields", "source_link", "photo_flags",
                        "Checked by a person"], review)
    lines = ["# Plant fill and verify report", ""]
    for pset, s in stats.items():
        counts = (f"- processed: {s['processed']}; decisions: Accept {s['Accept']}, Needs review "
                  f"{s['Needs review']}, Reject {s['Reject']}")
        lines += [f"## {pset}: {len(db.plants(pset))} plants", "", counts, "", "| field | filled |", "|---|---:|"]
        lines += [f"| {f} | {per_field[pset][f]} |" for f in norm.FIELDS]
        lines.append("")
    lines.append(f"Discrepancies: {len(disc_rows)} rows in discrepancies.csv")
    (out / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (out / "REVIEW.md").write_text(
        "# REVIEW\n\nOne row per plant in REVIEW.csv. The decision column was filled by the local model "
        "(gemma4:12b) from its own checks; the last column is blank for a person's double-check. Nothing here "
        "is imported until that check is done.\n", encoding="utf-8")


def _csv(name: str, head: list, rows: list) -> None:
    with (CFG.out / name).open("w", newline="", encoding="utf-8-sig") as fh:
        w = csv.writer(fh)
        w.writerow(head)
        w.writerows(rows)
