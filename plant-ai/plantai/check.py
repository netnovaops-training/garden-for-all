"""gemma4:12b stage: independent re-check of phi4's work, photo check, and the decision.

verify(): is each value supported by its quoted source lines? Is the source about this plant?
vision(): does each photo show this plant (not a logo/banner/other plant)?
decide(): Accept / Reject / Needs review with a reason, from the summary of both checks.
"""
from . import llm
from .config import CFG

V_SYS = ("You audit plant data using ONLY the text given, never your own knowledge. For each numbered "
         "claim, answer ok=yes only if the quoted source text "
         "states that value (wording may differ, meaning must match); otherwise ok=no. Also say whether "
         "the source pages describe the same plant as the record (same genus, species and cultivar; "
         "watch for similar-named cultivars or a different species). Short reasons.")
V_SCHEMA = {"type": "object", "required": ["same_plant", "identity_why", "checks"], "properties": {
    "same_plant": {"type": "string", "enum": ["yes", "no", "unsure"]},
    "identity_why": {"type": "string"},
    "checks": {"type": "array", "items": {"type": "object", "required": ["k", "ok", "why"], "properties": {
        "k": {"type": "integer"}, "ok": {"type": "string", "enum": ["yes", "no"]}, "why": {"type": "string"}}}}}}

P_SYS = ("You check garden photos. For each image (in order, numbered from 1) say if it is a real "
         "photo of a plant (plant_photo=false for a logo, banner, text graphic, chart, map, any picture "
         "showing a person's face, a name or username burned into the image, or a heavily "
         "blurred/compressed image), whether it carries a watermark or text overlay "
         "(watermark=true), and whether "
         "it plausibly shows the named plant: right kind of plant, flower colour and form, leaves. "
         "matches=unsure when it is a plant photo but you cannot tell. Also judge whether the named "
         "plant is the clear main subject: is_target_main_subject=false when it fills only a small part "
         "of the frame (under about 30%), when trees or landscape dominate, or in a mixed bed or "
         "container combo where it is not obviously the focus. One other plant in view is fine if the "
         "target is still obviously the one shown. Close-ups of its flowers or leaves count as main "
         "subject. approx_frame_share = percent of the frame it fills; distinct_plant_count = how many "
         "different kinds of plant are visible; viewer_can_tell_which_is_target = whether a viewer "
         "could tell which plant is meant; distractions = short list of what competes for attention.")
P_SCHEMA = {"type": "object", "required": ["photos"], "properties": {"photos": {"type": "array", "items": {
    "type": "object", "required": ["i", "plant_photo", "watermark", "matches", "is_target_main_subject",
                                   "approx_frame_share", "distinct_plant_count",
                                   "viewer_can_tell_which_is_target", "distractions", "why"], "properties": {
        "i": {"type": "integer"}, "plant_photo": {"type": "boolean"}, "watermark": {"type": "boolean"},
        "is_target_main_subject": {"type": "boolean"},
        "approx_frame_share": {"type": "integer", "minimum": 0, "maximum": 100},
        "distinct_plant_count": {"type": "integer", "minimum": 0},
        "viewer_can_tell_which_is_target": {"type": "boolean"},
        "distractions": {"type": "array", "items": {"type": "string"}},
        "matches": {"type": "string", "enum": ["yes", "no", "unsure"]}, "why": {"type": "string"}}}}}}

D_SYS = ("You decide whether a plant record's new data can be accepted, using ONLY the check results "
         "given (never your own knowledge of the plant). Accept: the sources are the "
         "same plant, (nearly) all values are supported, no photo is a wrong plant. Reject: the sources "
         "or photos are a different plant. Needs review: anything doubtful (unsupported values, "
         "low-confidence conflicts, unclear identity, failed photos, contradictions with existing "
         "values). One short reason.")
D_SCHEMA = {"type": "object", "required": ["decision", "reason"], "properties": {
    "decision": {"type": "string", "enum": ["Accept", "Reject", "Needs review"]},
    "reason": {"type": "string"}}}


def verify(rec: dict, sources: list[dict], claims: list[dict]) -> dict:
    """claims: [{f, v, quote, url}] -> {same_plant, identity_why, checks:{k: {ok, why}}}."""
    src = "\n".join(f"- {s['url']}  title: {s['title']}  start: {' / '.join(s['lines'][:4])[:300]}"
                    for s in sources)
    cl = "\n".join(f"{k}. {c['f']} = {c['v']}  | source text: {c['quote'][:400]}" for k, c in enumerate(claims, 1))
    out = llm.ask(llm.CHECK, V_SYS, f"RECORD: {rec.get('plant_name')} | botanical: {rec.get('botanical_name')}\n"
                  f"SOURCES:\n{src}\n\nCLAIMS:\n{cl or '(none)'}", V_SCHEMA, num_ctx=8192) or {}
    checks = {c["k"]: c for c in out.get("checks", []) if isinstance(c, dict) and "k" in c}
    return {"same_plant": out.get("same_plant", "unsure"), "identity_why": out.get("identity_why", "no answer"),
            "checks": checks}


_refs: list[bytes] = []


def refs() -> list[bytes]:
    """Optional reference photos: a few of the garden owner's own good photos (small previews),
    shown to the model first as examples. None are built in; with none, the check runs without."""
    if not _refs and CFG.reference_photos:
        from . import images
        for p in CFG.reference_photos[:3]:
            if p.is_file():
                try:
                    _refs.append(images.preview(p, 384))
                except OSError:
                    continue
    return _refs


def vision(rec: dict, imgs: list[bytes], fewshot: bool = True) -> list[dict]:
    if not imgs:
        return []
    want = f"PLANT: {rec.get('plant_name')} ({rec.get('botanical_name')}); flower colour on record: " \
           f"{rec.get('bloom_color') or 'unknown'}; {len(imgs)} images."
    ex = refs() if fewshot else []
    if ex:
        want = (f"The FIRST {len(ex)} images are reference examples of good garden photos from this garden "
                f"(clear main subject). Do not report them. Report only the {len(imgs)} images after them, "
                f"numbered from 1.\n" + want)
    out = llm.ask(llm.CHECK, P_SYS, want, P_SCHEMA, images=ex + imgs, num_ctx=8192) or {}
    photos = [p for p in out.get("photos", []) if isinstance(p, dict) and "i" in p]
    if ex and len(photos) == len(imgs) + len(ex):   # the model numbered the references too
        photos = photos[len(ex):]
        for k, p in enumerate(photos, 1):
            p["i"] = k
    got = {p["i"]: p for p in photos}
    res = [got.get(i, {"i": i, "plant_photo": False, "matches": "unsure", "why": "no answer"})
           for i in range(1, len(imgs) + 1)]
    for p in res:
        p["reject"] = photo_reject(p)
    return res


def photo_reject(p: dict) -> str:
    """'' when the photo is usable, else the reason code (logged in discrepancies.csv)."""
    if not p.get("plant_photo"):
        return "not_a_plant_photo"
    if p.get("matches") == "no":
        return "wrong_plant"
    # Thresholds from the settings (config.CALIB_DEFAULTS, or the garden's own calib.json).
    max_n = int(CFG.calib.get("max_distinct_plants", 4))
    min_share = int(CFG.calib.get("min_frame_share", 30))
    must_tell = bool(CFG.calib.get("require_viewer_can_tell", True))
    if ((must_tell and p.get("viewer_can_tell_which_is_target") is False)
            or int(p.get("distinct_plant_count") or 0) > max_n
            or int(p.get("approx_frame_share") or 0) < min_share):
        return "not_main_subject"
    return ""


def decide(summary: str) -> dict:
    out = llm.ask(llm.CHECK, D_SYS, summary, D_SCHEMA, num_ctx=4096) or {}
    if out.get("decision") not in ("Accept", "Reject", "Needs review"):
        return {"decision": "Needs review", "reason": "model gave no decision"}
    return out
