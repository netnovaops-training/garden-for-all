"""Photos from the vendor page: the largest original the page offers, ONE .webp per picture.

Rules: source long side >= 500 px (else rejected); over 2000 px is downscaled once to 2000 (Lanczos);
in between kept as is (never upscaled). cwebp q90, method 6, sharp_yuv, metadata stripped (when the
cwebp program is not installed, Pillow writes the same quality and method, also without metadata).
Rejected: logos/banners (name, shape), near-duplicates (perceptual hash), blurry (low Laplacian
variance). Watermarks are judged later by the vision check. Source URL, resolution and credit go
in the data. No thumbnails: the site shows the one .webp at whatever size the page needs.
"""
import io
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from urllib.parse import urlsplit

import imagehash
from PIL import Image, ImageFilter, ImageStat

from . import fetch
from .config import CFG

SKIP_NAME = re.compile(r"logo|icon|banner|badge|flag|sprite|button|placeholder|tray|header|footer|"
                       r"social|award|tag[-_]|label|swatch|map|qr", re.IGNORECASE)
SOCIAL = re.compile(r"(^|\.)(instagram\.com|cdninstagram\.com|facebook\.com|fbcdn\.net|fbsbx\.com|"
                    r"pinterest\.[a-z.]+|pinimg\.com|tiktok\.com|tiktokcdn\.com|twitter\.com|x\.com|"
                    r"twimg\.com|reddit\.com|redd\.it|redditmedia\.com)$")


def min_px() -> int:
    return max(500, int(CFG.calib.get("min_px", 500)))


def max_px() -> int:
    return min(2000, int(CFG.calib.get("max_px", 2000)))


def blur_min() -> float:
    """Variance of the Laplacian at 1000 px. Below it = blurry, rejected. Up to 1.5x the floor =
    "soft": used only when nothing sharper exists."""
    return float(CFG.calib.get("blur_min", 20.0))


def social(url: str) -> bool:
    return bool(SOCIAL.search(urlsplit(url).netloc.lower().split(":")[0]))


def sharpness(im: Image.Image) -> float:
    g = im.convert("L")
    if max(g.size) > 1000:
        g.thumbnail((1000, 1000), Image.LANCZOS)
    lap = g.filter(ImageFilter.Kernel((3, 3), [0, 1, 0, 1, -4, 1, 0, 1, 0], scale=1, offset=128))
    return ImageStat.Stat(lap).var[0]


def original(u: str) -> str:
    """The full-size file behind a resized URL where the site's pattern is known."""
    return re.sub(r"/files/imagecache/[^/]+/", "/files/", u)


def to_webp(im: Image.Image, dst: Path) -> None:
    """The one save: q90, method 6, no metadata. cwebp (sharp_yuv) when installed, else Pillow."""
    if max(im.size) > max_px():
        im = im.copy()
        im.thumbnail((max_px(), max_px()), Image.LANCZOS)   # the one and only resize
    cwebp = shutil.which("cwebp")
    if cwebp:
        with tempfile.TemporaryDirectory() as t:
            png = Path(t) / "src.png"
            im.save(png)   # lossless hand-off; carries no EXIF/ICC/XMP
            subprocess.run([cwebp, "-quiet", "-q", "90", "-m", "6", "-sharp_yuv", "-metadata", "none",
                            str(png), "-o", str(dst)], check=True)
    else:
        im.save(dst, "WEBP", quality=90, method=6)   # no exif/icc passed = none written


def collect(pid: int, urls: list[str], want: int = 5, start: int = 1) -> list[dict]:
    """Up to `want` photos for plant `pid`: [{n, src, file, src_w, src_h, out_w, out_h, kb, sharp, flags}].
    Sharp photos first; "soft" ones fill in only when there are not enough sharp ones."""
    cands, hashes, rejected = [], [], []
    d = CFG.out / "images" / str(pid)
    floor = blur_min()
    for u in urls:
        if len([c for c in cands if not c[2]]) >= want:
            break
        if SKIP_NAME.search(u.rsplit("/", 1)[-1]) or u.lower().split("?")[0].endswith((".svg", ".gif")):
            continue
        src = original(u)
        r = fetch.get(src)
        if r["status"] != 200 or "image" not in r.get("type", ""):
            src, r = u, fetch.get(u)
            if r["status"] != 200 or "image" not in r.get("type", ""):
                continue
        try:
            im = Image.open(io.BytesIO(Path(r["path"]).read_bytes()))
            im.load()
            im = im.convert("RGB")
        except Exception as e:  # noqa: BLE001 -- not a picture Pillow can read: noted, then skipped
            rejected.append((src, f"not a readable picture ({type(e).__name__})"))
            continue
        w, h = im.size
        if max(w, h) < min_px():
            rejected.append((src, f"too small {w}x{h}"))
            continue
        if not (0.45 <= w / h <= 2.2):
            rejected.append((src, f"banner-like shape {w}x{h}"))
            continue
        sh = sharpness(im)
        if sh < floor:
            rejected.append((src, f"blurry (sharpness {sh:.0f})"))
            continue
        ph = imagehash.phash(im)
        if any(ph - x <= 8 for x in hashes):
            continue
        hashes.append(ph)
        cands.append((src, im, sh < floor * 1.5, sh, ph, w, h))
    # Sharp ones first, in page order; soft ones only to reach `want`.
    cands = [c for c in cands if not c[2]] + [c for c in cands if c[2]]
    out = []
    for src, im, soft, sh, ph, w, h in cands[:want]:
        d.mkdir(parents=True, exist_ok=True)
        n = start + len(out)
        dst = d / f"{n}.webp"
        to_webp(im, dst)
        with Image.open(dst) as done:
            ow, oh = done.size
        out.append({"n": n, "src": src, "file": f"images/{pid}/{n}.webp", "src_w": w, "src_h": h,
                    "out_w": ow, "out_h": oh, "kb": round(dst.stat().st_size / 1024),
                    "sharp": round(sh), "phash": str(ph),
                    "flags": (["soft"] if soft else []) + (["social"] if social(src) else [])})
    if rejected:
        d.mkdir(parents=True, exist_ok=True)
        (d / "rejected.txt").write_text("\n".join(f"{s}\t{why}" for s, why in rejected) + "\n", encoding="utf-8")
    return out


def preview(path: Path, px: int = 512) -> bytes:
    """Small in-memory JPEG of a photo for the vision check (nothing written to disk)."""
    im = Image.open(path).convert("RGB")
    im.thumbnail((px, px), Image.LANCZOS)
    b = io.BytesIO()
    im.save(b, "JPEG", quality=85)
    return b.getvalue()
