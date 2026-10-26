"""One plant's finished result, from its saved stages: what to write, what needs a person, what is
still missing, and the photos. Used by the site worker (worker.py) and the batch report (report.py),
so both make the same decisions.

Rules (the same as the batch run):
  - a value is kept only when gemma found it in its quote and the decision is not Reject;
  - everything else phi4 found goes to "needs review" with its quote and source (never written);
  - photos that failed the vision check are dropped; watermarked ones only stay when nothing clean exists.
"""
from . import db, norm


def _empty(v) -> bool:
    return v is None or str(v).strip() == ""


def plant_result(pset: str, pid: int, rec: dict) -> dict | None:
    f, c = db.get(pset, pid, "fill"), db.get(pset, pid, "check")
    if not f:
        return None
    ims = db.get(pset, pid, "images") or []
    claims = {(x["f"], x["kind"]): x for x in (c or {}).get("claims", [])}
    rejected = (c or {}).get("decision") == "Reject"
    fields, review, conf, sources, quotes = {}, {}, {}, {}, {}
    for fld, j in f["fills"].items():
        ck = claims.get((fld, "fill"))
        ok = bool(ck) and ck["gemma"] == "yes"
        quote = (j["quotes"] or [""])[0][:300]
        if ok and not rejected:
            fields[fld] = j["v"]
            conf[fld] = j["conf"]
            sources[fld] = j["sources"]
            quotes[fld] = quote
        else:
            why = ("rejected as a whole: " + (c or {}).get("reason", "")) if rejected else \
                  ("the checker did not find it in the quote: " + (ck or {}).get("gemma_why", "") if ck
                   else "not checked yet")
            review[fld] = {"v": j["v"], "why": why[:300], "quote": quote, "sources": j["sources"],
                           "conf": j["conf"]}
    vis = {p.get("src"): p for p in (c or {}).get("photos", [])}
    good = [x for x in ims if not vis.get(x["src"], {}).get("reject")]
    # Watermarks are never edited: drop watermarked photos when clean ones exist,
    # keep (flagged) only when a watermarked one is all there is.
    clean = [x for x in good if not vis.get(x["src"], {}).get("watermark")]
    good = clean if clean else good
    photos = []
    for x in good:
        flags = list(x.get("flags", []))
        if vis.get(x["src"], {}).get("watermark") and "watermarked" not in flags:
            flags.append("watermarked")
        photos.append({"file": x["file"], "source": x["src"], "page": x.get("page"), "credit": x.get("credit"),
                       "source_px": f"{x['src_w']}x{x['src_h']}", "out_px": f"{x['out_w']}x{x['out_h']}",
                       "flags": flags})
    after = dict(rec)
    after.update(fields)
    missing = [x for x in norm.FIELDS if _empty(after.get(x))]
    return {"fields": fields, "needs_review": review, "missing": missing, "confidence": conf,
            "sources": sources, "quotes": quotes, "product_url": f.get("product_url"),
            "product_kind": f.get("product_kind"), "photos": photos,
            "contradictions": f.get("contradictions", []), "unnormalized": f.get("unnormalized", []),
            "decision": (c or {}).get("decision", "not checked yet"), "reason": (c or {}).get("reason", ""),
            "identity": (c or {}).get("identity", ""), "identity_why": (c or {}).get("identity_why", "")}
