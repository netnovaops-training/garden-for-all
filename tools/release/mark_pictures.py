"""Draw an orange ring (and a short note) on the window pictures, around what to click.

    python tools/release/mark_pictures.py

The clean pictures are kept in tools/release/clean-pictures/ (the first run copies
them there), so marks are always drawn on a clean copy and never pile up. Run it
again after shoot-wizard.ps1 retakes the pictures: it refreshes the clean copy
first when the picture is newer.
"""
import json, pathlib, shutil
from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parents[2]
IMG = ROOT / "START-HERE" / "images"
CLEAN = pathlib.Path(__file__).resolve().parent / "clean-pictures"
ORANGE = (255, 140, 26)

# picture: [(box (left, top, right, bottom), note, where the note goes: "left"/"right"/"above"/"below" or (x, y))]
MARKS = {
    "wizard-0-onedrive.png": [((28, 203, 616, 231), "click this one", "above")],
}


def font(size):
    for f in ("C:/Windows/Fonts/segoeuib.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"):
        if pathlib.Path(f).exists():
            return ImageFont.truetype(f, size)
    return ImageFont.load_default()


def mark(name, marks):
    src, clean = IMG / name, CLEAN / name
    if not src.exists():
        return False
    if not clean.exists() or src.stat().st_mtime > clean.stat().st_mtime + 2:
        shutil.copy2(src, clean)          # a fresh (unmarked) picture: keep it as the clean copy
    im = Image.open(clean).convert("RGBA")
    layer = Image.new("RGBA", im.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    f = font(15)
    for (l, t, r, b), note, side in marks:
        d.rounded_rectangle((l - 6, t - 6, r + 6, b + 6), radius=12, outline=ORANGE + (255,), width=4)
        w = d.textlength(note, font=f)
        if isinstance(side, tuple):
            x, y = side
        elif side == "left":
            x, y = l - 16 - w - 22, (t + b) / 2 - 11
        elif side == "right":
            x, y = r + 18, (t + b) / 2 - 11
        elif side == "above":
            x, y = (l + r) / 2 - (w + 22) / 2, t - 40
        else:
            x, y = (l + r) / 2 - (w + 22) / 2, b + 14
        d.rounded_rectangle((x, y, x + w + 22, y + 24), radius=12, fill=ORANGE + (255,))
        d.text((x + 11, y + 3), note, font=f, fill=(255, 255, 255, 255))
    Image.alpha_composite(im, layer).convert("RGB").save(src)
    shutil.copystat(clean, src)            # same time as the clean copy, so it is not taken for a new picture
    return True


# The setup wizard's pictures: shoot-wizard.ps1 writes where the buttons are
# (clean-pictures/wizard-marks.json), so the rings follow the window's layout.
WIZ = CLEAN / "wizard-marks.json"
if WIZ.exists():
    for name, rings in json.loads(WIZ.read_text(encoding="utf-8")).items():
        MARKS[name] = [((l, t, r, b), note, side) for l, t, r, b, note, side in rings]


if __name__ == "__main__":
    CLEAN.mkdir(exist_ok=True)
    done = [n for n, mk in MARKS.items() if mark(n, mk)]
    print("marked:", len(done), "pictures")
