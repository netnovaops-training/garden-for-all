"""Open a photo and make the small square the model reads. Light module (no torch), so the
decode worker processes start fast.

mode "crop":   shrink so the short side is `size`, cut the middle square.
mode "squash": shrink the whole picture to size x size (keeps the edges, bends the shape).
"""
import io
import os

import numpy as np
from PIL import Image, ImageOps

Image.MAX_IMAGE_PIXELS = 100_000_000  # a phone photo is far below this; a bomb is not


def cores():
    """CPU cores this process may really use: the Docker limit (--cpus) when there is one."""
    n = os.cpu_count() or 4
    try:
        quota, period = open("/sys/fs/cgroup/cpu.max").read().split()
        if quota != "max":
            n = min(n, max(1, int(int(quota) / int(period))))
    except (OSError, ValueError):
        pass
    return n


def load(src, size=224, mode="crop"):
    """src: path or bytes -> uint8 array (size, size, 3), or None if it cannot be read."""
    try:
        with Image.open(io.BytesIO(src) if isinstance(src, (bytes, bytearray)) else src) as im:
            im = ImageOps.exif_transpose(im)  # phone photos carry their rotation in EXIF
            im = im.convert("RGB")
            if mode == "squash":
                im = im.resize((size, size), Image.BICUBIC, reducing_gap=3.0)
            else:
                w, h = im.size
                s = size / min(w, h)
                nw, nh = max(size, round(w * s)), max(size, round(h * s))
                im = im.resize((nw, nh), Image.BICUBIC, reducing_gap=3.0)
                x, y = (nw - size) // 2, (nh - size) // 2
                im = im.crop((x, y, x + size, y + size))
            return np.asarray(im, dtype=np.uint8)
    except Exception:  # unreadable or truncated file: the caller records it as failed
        return None


def load_path(args):
    """Pool entry point: (path, size, mode) -> (path, array or None)."""
    path, size, mode = args
    return path, load(path, size, mode)
