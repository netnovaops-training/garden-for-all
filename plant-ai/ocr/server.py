"""The tag reader service: a small web server that reads the text and barcodes in one picture.

It runs in Docker (plant-ai/ocr/Dockerfile), published ONLY on this PC (127.0.0.1:8788). The desktop
helper (plantai/tagocr.py) sends it a tag photo and gets back lines of text. Nothing is sent to the
internet, and no cloud model is used: the two engines are Tesseract and RapidOCR (the PaddleOCR
models, run with ONNX on the processor).

  GET  /health                          {"ok": true, "engines": [...]}
  POST /ocr?engine=tesseract|rapid|both the picture as the body ->
       {"engine": ..., "lines": [{"text", "conf" 0-100, "h" (text height px), "box": [x0,y0,x1,y1]}],
        "barcodes": [{"type", "text"}], "rotation": 0|90|180|270, "ms": n}
"""
import io
import json
import os
import statistics
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

from PIL import Image, ImageOps

MAX_BODY = 30_000_000
_rapid = None


def load(data: bytes) -> Image.Image:
    """The picture, upright (EXIF), as RGB, at a size that reads well: small ones are enlarged."""
    img = ImageOps.exif_transpose(Image.open(io.BytesIO(data))).convert("RGB")
    long = max(img.size)
    if long < 1100:
        f = 1500 / long
        img = img.resize((int(img.width * f), int(img.height * f)), Image.LANCZOS)
    elif long > 2600:
        f = 2600 / long
        img = img.resize((int(img.width * f), int(img.height * f)), Image.LANCZOS)
    return img


def tesseract(img: Image.Image) -> list[dict]:
    """Lines from Tesseract (sparse-text mode finds text scattered over a tag)."""
    import pytesseract
    d = pytesseract.image_to_data(img, lang="eng", config="--psm 11 --oem 1", output_type=pytesseract.Output.DICT)
    lines: dict[tuple, list[int]] = {}
    for i, t in enumerate(d["text"]):
        if t.strip() and float(d["conf"][i]) >= 0:
            lines.setdefault((d["block_num"][i], d["par_num"][i], d["line_num"][i]), []).append(i)
    out = []
    for idx in lines.values():
        words = sorted(idx, key=lambda i: d["left"][i])
        x0 = min(d["left"][i] for i in words)
        y0 = min(d["top"][i] for i in words)
        x1 = max(d["left"][i] + d["width"][i] for i in words)
        y1 = max(d["top"][i] + d["height"][i] for i in words)
        out.append({"text": " ".join(d["text"][i] for i in words),
                    "conf": round(statistics.fmean(float(d["conf"][i]) for i in words), 1),
                    "h": int(statistics.median(d["height"][i] for i in words)),
                    "box": [x0, y0, x1, y1]})
    return sorted(out, key=lambda ln: (ln["box"][1], ln["box"][0]))


def rapid(img: Image.Image) -> list[dict]:
    """Lines from RapidOCR (PaddleOCR detection and recognition models)."""
    global _rapid
    import numpy as np
    if _rapid is None:
        from rapidocr_onnxruntime import RapidOCR
        _rapid = RapidOCR()
    res, _ = _rapid(np.asarray(img)[:, :, ::-1])
    out = []
    for box, text, score in res or []:
        xs, ys = [p[0] for p in box], [p[1] for p in box]
        out.append({"text": text, "conf": round(float(score) * 100, 1),
                    "h": int(max(ys) - min(ys)), "box": [int(min(xs)), int(min(ys)), int(max(xs)), int(max(ys))]})
    return sorted(out, key=lambda ln: (ln["box"][1], ln["box"][0]))


def score(lines: list[dict]) -> float:
    """How readable a result looks: confidence times the number of letters (to pick a rotation)."""
    return sum(ln["conf"] * sum(c.isalnum() for c in ln["text"]) for ln in lines)


def passes(img: Image.Image, engine: str) -> list[dict]:
    """One engine over several looks at the same picture, lines merged. Tesseract reads very big
    letters badly, so it also gets a half-size look; dark bands with white letters are read from
    the inverted picture. (RapidOCR copes with size, so it only gets the inverted look.)"""
    fn = rapid if engine == "rapid" else tesseract
    looks = [(False, 1.0), (True, 1.0)] if engine == "rapid" else [(False, 1.0), (False, 0.5), (True, 0.5)]
    out: list[dict] = []
    for invert, scale in looks:
        im = ImageOps.invert(img) if invert else img
        if scale != 1.0:
            im = im.resize((int(im.width * scale), int(im.height * scale)), Image.LANCZOS)
        lines = [ln for ln in fn(im) if ln["conf"] >= 30]
        for ln in lines:
            if scale != 1.0:
                ln["h"] = int(ln["h"] / scale)
                ln["box"] = [int(v / scale) for v in ln["box"]]
        out = merge(out, lines)
    return out


def read(img: Image.Image, engine: str) -> tuple[list[dict], int]:
    """Read with the chosen engine; when the first pass looks poor, try the picture turned 90, 180
    and 270 degrees too and keep the best. Gives (lines, rotation used)."""
    best, rot = passes(img, engine), 0
    chars = sum(c.isalnum() for ln in best for c in ln["text"])
    mean = statistics.fmean([ln["conf"] for ln in best]) if best else 0
    if chars < 25 or mean < 55:
        for deg in (90, 270, 180):
            alt = passes(img.rotate(deg, expand=True), engine)
            if score(alt) > score(best) * 1.15:
                best, rot = alt, deg
    return best, rot


def merge(a: list[dict], b: list[dict]) -> list[dict]:
    """Both engines' lines, without the near-duplicates (the better-read one is kept)."""
    import difflib
    out = list(a)
    for ln in b:
        key = "".join(c for c in ln["text"].lower() if c.isalnum())
        dup = None
        for i, o in enumerate(out):
            ok = "".join(c for c in o["text"].lower() if c.isalnum())
            if key and ok and difflib.SequenceMatcher(None, key, ok).ratio() > 0.8:
                dup = i
                break
        if dup is None:
            out.append(ln)
        elif ln["conf"] > out[dup]["conf"] + 10:
            out[dup] = ln          # the first list wins ties: it is the main reader
    return sorted(out, key=lambda ln: (ln["box"][1], ln["box"][0]))


def barcodes(img: Image.Image) -> list[dict]:
    """Barcodes and QR codes in the picture (zbar); [] when zbar is not available."""
    try:
        from pyzbar.pyzbar import decode
    except (ImportError, OSError):
        return []
    seen, out = set(), []
    for im in (img.convert("L"), ImageOps.autocontrast(img.convert("L"))):
        for r in decode(im):
            t = r.data.decode("utf-8", "replace")
            if t not in seen:
                seen.add(t)
                out.append({"type": r.type, "text": t})
    return out


def run(data: bytes, engine: str) -> dict:
    t0 = time.monotonic()
    img = load(data)
    if engine == "both":
        a, rot_a = read(img, "tesseract")
        b, rot_b = read(img, "rapid")
        if rot_a == rot_b:
            # RapidOCR is the main reader (it measured far better on tags); Tesseract only adds
            # lines it is quite sure of that RapidOCR did not give.
            lines, rot = merge(b, [ln for ln in a if ln["conf"] >= 70]), rot_a
        else:
            # The engines found the text turned differently: keep the one that read more.
            lines, rot = (a, rot_a) if score(a) >= score(b) else (b, rot_b)
    else:
        lines, rot = read(img, engine)
    return {"engine": engine, "lines": lines, "barcodes": barcodes(img), "rotation": rot,
            "ms": int((time.monotonic() - t0) * 1000)}


class Handler(BaseHTTPRequestHandler):
    server_version = "garden-ocr/1.0"

    def _send(self, code: int, body: dict) -> None:
        raw = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def do_GET(self) -> None:
        if urlsplit(self.path).path == "/health":
            self._send(200, {"ok": True, "engines": ["tesseract", "rapid", "both"]})
        else:
            self._send(404, {"error": "not found"})

    def do_POST(self) -> None:
        u = urlsplit(self.path)
        if u.path != "/ocr":
            return self._send(404, {"error": "not found"})
        n = int(self.headers.get("Content-Length") or 0)
        if not 0 < n <= MAX_BODY:
            return self._send(400, {"error": "send one picture as the body (at most 30 MB)"})
        engine = parse_qs(u.query).get("engine", ["both"])[0]
        if engine not in ("tesseract", "rapid", "both"):
            return self._send(400, {"error": "engine is tesseract, rapid or both"})
        try:
            self._send(200, run(self.rfile.read(n), engine))
        except Exception as e:  # noqa: BLE001 -- a bad picture must not stop the service
            self._send(422, {"error": f"could not read the picture ({type(e).__name__})"})

    def log_message(self, *_a) -> None:
        pass   # no request log: nothing about the pictures is kept


if __name__ == "__main__":
    # All interfaces INSIDE the container (Docker publishes it on 127.0.0.1 only: see the Dockerfile).
    ThreadingHTTPServer(("0.0.0.0", int(os.environ.get("PORT", "8098"))), Handler).serve_forever()  # noqa: S104
