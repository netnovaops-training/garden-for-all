"""Read a plant tag photo: text lines from the OCR service (Tesseract + PaddleOCR models, in Docker),
fields from the lines (tagparse.py), and, for a hard tag only, a second reading by the local vision
model (gemma4:12b in Ollama). Everything stays on this PC; no cloud model is ever called.

Order, cheapest first:
  1. the OCR service at 127.0.0.1:8788 (plant-ai/ocr). Both engines, lines merged.
  2. when that service is not running: the `tesseract` program, if it is installed here.
  3. when the result is weak (no name, or the reading was poor): gemma reads the front picture
     as a picture. It is only asked then, because it uses the graphics card.
"""
import os
import shutil
import statistics
import subprocess
import tempfile

import httpx

from . import tagparse

OCR_URL = os.environ.get("PLANTAI_OCR", "http://127.0.0.1:8788").rstrip("/")
# tesseract | rapid | both. RapidOCR (PaddleOCR models) measured far better on tags than Tesseract
# (see plant-ai/README.md, "How the tag reader was chosen"), so it is the default.
ENGINE = os.environ.get("PLANTAI_OCR_ENGINE", "rapid")

SCHEMA = {
    "type": "object",
    "properties": {k: {"type": "string"} for k in ("name", "botanical", "size", "vendor", "barcode")},
    "required": ["name", "botanical", "size", "vendor", "barcode"],
}
SYSTEM = ("You read photographs of plant tags. Copy the text exactly as it is printed. Use an empty string "
          "for anything that is not printed on the tag. Never guess and never add facts from memory.")
USER = ("This is a plant tag. Give: name (the plant's common name, the biggest text), botanical (the Latin "
        "name, usually in italics), size (the pot size such as #1 or 1 gal), vendor (the grower), barcode "
        "(the digits under the bars).")


def service_up(client: httpx.Client | None = None) -> bool:
    """Is the OCR service answering on this PC?"""
    try:
        return (client or httpx).get(f"{OCR_URL}/health", timeout=3).status_code == 200
    except httpx.HTTPError:
        return False


def from_service(img: bytes, engine: str = ENGINE, client: httpx.Client | None = None) -> dict | None:
    """The service's reading of one picture ({lines, barcodes, ...}), or None when it is not there."""
    try:
        r = (client or httpx).post(f"{OCR_URL}/ocr", params={"engine": engine}, content=img, timeout=180,
                                   headers={"Content-Type": "application/octet-stream"})
        if r.status_code != 200:
            return None
        return r.json()
    except (httpx.HTTPError, ValueError):
        return None


def from_tesseract(img: bytes) -> dict | None:
    """A reading with the `tesseract` program on this PC, if there is one (a plain fallback)."""
    exe = shutil.which("tesseract")
    if not exe:
        return None
    with tempfile.NamedTemporaryFile(suffix=".jpg", delete=False) as f:
        f.write(img)
        name = f.name
    try:
        # The program is found with shutil.which and the file is our own temporary copy: no input
        # from a scan reaches the command line.
        tsv = subprocess.run([exe, name, "-", "--psm", "11", "tsv"], capture_output=True, text=True,  # noqa: S603
                             timeout=120, check=False).stdout
    except (OSError, subprocess.SubprocessError):
        return None
    finally:
        os.unlink(name)
    lines: dict[tuple, list[dict]] = {}
    for row in tsv.splitlines()[1:]:
        c = row.split("\t")
        if len(c) < 12 or not c[11].strip() or float(c[10]) < 0:
            continue
        lines.setdefault((c[2], c[3], c[4]), []).append({"t": c[11], "conf": float(c[10]), "h": int(c[9])})
    out = [{"text": " ".join(w["t"] for w in ws), "conf": statistics.fmean(w["conf"] for w in ws),
            "h": int(statistics.median(w["h"] for w in ws))} for ws in lines.values()]
    return {"engine": "tesseract", "lines": out, "barcodes": []}


def gemma_read(images: list[bytes]) -> dict | None:
    """The vision model's reading of the tag picture(s), or None when it is not ready. Needs Ollama
    and the check model; used only for a hard tag."""
    from . import llm
    from .config import CFG
    ok, _ = llm.ready()
    if not ok:
        return None
    r = llm.ask(CFG.check_model, SYSTEM, USER, SCHEMA, images=images[:2], tag="tagread")
    return {k: str(r.get(k, "") or "").strip() for k in SCHEMA["properties"]} if isinstance(r, dict) else None


def _mean_conf(lines: list[dict]) -> float:
    """Average OCR confidence, each line weighted by its letters."""
    w = [(ln.get("conf", 0.0), sum(c.isalnum() for c in ln.get("text", ""))) for ln in lines]
    n = sum(x[1] for x in w)
    return sum(c * k for c, k in w) / n if n else 0.0


def read_tag(photos: list[dict], hint: str = "", client: httpx.Client | None = None, use_gemma: bool = True) -> dict:
    """Read the tag photos ({side, bytes}, front first). Gives
    {fields, lines, engine, conf, weak, barcodes, notes}. Fields come from the front for the name
    and from front-then-back for the rest, so the care facts on the back are never taken for a name."""
    notes, engines = [], []
    front_lines, back_lines, codes = [], [], []
    for p in photos:
        r = from_service(p["bytes"], client=client) or from_tesseract(p["bytes"])
        if not r:
            notes.append("no tag reader is running (start the garden-ocr container)")
            continue
        engines.append(r.get("engine", "?"))
        (back_lines if p.get("side") == "back" else front_lines).extend(r.get("lines", []))
        codes += [b["text"] for b in r.get("barcodes", []) if b.get("text")]
    lines = front_lines + back_lines
    # A barcode zbar decoded exactly beats digits read from the printed numbers.
    hint = next((c for c in codes if c.isdigit()), "") or hint or next(iter(codes), "")
    f_front = tagparse.parse(front_lines, hint) if front_lines else {}
    f_back = tagparse.parse(back_lines, hint) if back_lines else {}
    fields = {k: (f_front.get(k) or f_back.get(k) or "") for k in ("botanical", "size", "vendor", "barcode")}
    fields["botanical"] = tagparse.best_botanical(f_front.get("botanical", ""), f_back.get("botanical", ""))
    fields["name"] = f_front.get("name") or ("" if front_lines else f_back.get("name", ""))
    if hint and not fields["barcode"]:
        fields["barcode"] = hint
    conf = _mean_conf(lines)
    weak = (not fields["name"]) or conf < 45 or (not fields["botanical"] and conf < 60)
    if weak and use_gemma and photos:
        g = gemma_read([p["bytes"] for p in photos if p.get("side") != "back"] or [photos[0]["bytes"]])
        if g:
            engines.append("gemma")
            for k, v in g.items():
                if v and not fields.get(k):
                    fields[k] = v
        elif lines or not engines:
            notes.append("the tag reading is weak and the vision model is not available")
    return {"fields": fields, "lines": [ln["text"] for ln in lines], "engine": "+".join(dict.fromkeys(engines)),
            "conf": round(conf, 1), "weak": weak, "barcodes": codes, "notes": notes}
