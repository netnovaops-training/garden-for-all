"""Made-up plant tags, to test the tag reader before there are real tag photos.

  python -m plantai synth --out DIR [--count 40] [--seed 1] [--level easy|normal|hard]

Each tag gets a front and a back picture (JPEG, like a phone photo of a tag held in the hand: a bit
rotated, a bit blurred, uneven light, noise, JPEG damage) and a `truth.json` that says what is
printed on it (name, botanical name, size, grower, barcode). The plants and growers are common
garden plants and invented nurseries: none of it is anyone's data. Only Pillow is needed; a QR code
is drawn when the `qrcode` package happens to be installed.

HOW REAL IS IT? It is not a real tag. It tests that the reading and parsing code works across
fonts, layouts, rotation, blur and noise. The accuracy printed from it is a floor for "does the
pipeline work", not a promise about Jun's tags: real photos need to be added when they exist.
"""
import json
import math
import random
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

# (common name, botanical name, genus, species) -- widely grown garden plants.
PLANTS = [
    ("May Night Salvia", "Salvia nemorosa 'May Night'", "Salvia", "nemorosa"),
    ("Blue Angel Hosta", "Hosta 'Blue Angel'", "Hosta", ""),
    ("Magnus Coneflower", "Echinacea purpurea 'Magnus'", "Echinacea", "purpurea"),
    ("Moonbeam Tickseed", "Coreopsis verticillata 'Moonbeam'", "Coreopsis", "verticillata"),
    ("Stella de Oro Daylily", "Hemerocallis 'Stella de Oro'", "Hemerocallis", ""),
    ("Autumn Joy Stonecrop", "Sedum 'Autumn Joy'", "Sedum", ""),
    ("Limelight Hydrangea", "Hydrangea paniculata 'Limelight'", "Hydrangea", "paniculata"),
    ("Endless Summer Hydrangea", "Hydrangea macrophylla 'Endless Summer'", "Hydrangea", "macrophylla"),
    ("Knock Out Rose", "Rosa 'Radrazz'", "Rosa", ""),
    ("Black-Eyed Susan", "Rudbeckia fulgida 'Goldsturm'", "Rudbeckia", "fulgida"),
    ("Karl Foerster Feather Reed Grass", "Calamagrostis x acutiflora 'Karl Foerster'", "Calamagrostis", "acutiflora"),
    ("Purple Smoke Bush", "Cotinus coggygria 'Royal Purple'", "Cotinus", "coggygria"),
    ("Dwarf Fountain Grass", "Pennisetum alopecuroides 'Hameln'", "Pennisetum", "alopecuroides"),
    ("Lavender Hidcote", "Lavandula angustifolia 'Hidcote'", "Lavandula", "angustifolia"),
    ("Russian Sage", "Perovskia atriplicifolia", "Perovskia", "atriplicifolia"),
    ("Catmint Walker's Low", "Nepeta racemosa 'Walker's Low'", "Nepeta", "racemosa"),
    ("Bee Balm Jacob Cline", "Monarda didyma 'Jacob Cline'", "Monarda", "didyma"),
    ("Siberian Iris Caesar's Brother", "Iris sibirica 'Caesar's Brother'", "Iris", "sibirica"),
    ("Peony Sarah Bernhardt", "Paeonia lactiflora 'Sarah Bernhardt'", "Paeonia", "lactiflora"),
    ("Japanese Forest Grass", "Hakonechloa macra 'Aureola'", "Hakonechloa", "macra"),
    ("Coral Bells Caramel", "Heuchera villosa 'Caramel'", "Heuchera", "villosa"),
    ("Creeping Phlox", "Phlox subulata", "Phlox", "subulata"),
    ("Oakleaf Hydrangea", "Hydrangea quercifolia", "Hydrangea", "quercifolia"),
    ("Boxwood Green Velvet", "Buxus 'Green Velvet'", "Buxus", ""),
    ("Winterberry Holly", "Ilex verticillata 'Winter Red'", "Ilex", "verticillata"),
    ("Spirea Goldflame", "Spiraea japonica 'Goldflame'", "Spiraea", "japonica"),
    ("Butterfly Bush Pugster Blue", "Buddleja davidii 'Pugster Blue'", "Buddleja", "davidii"),
    ("Daylily Happy Returns", "Hemerocallis 'Happy Returns'", "Hemerocallis", ""),
    ("Astilbe Fanal", "Astilbe x arendsii 'Fanal'", "Astilbe", "arendsii"),
    ("Bleeding Heart", "Lamprocapnos spectabilis", "Lamprocapnos", "spectabilis"),
    ("Lamb's Ear Helene von Stein", "Stachys byzantina 'Helene von Stein'", "Stachys", "byzantina"),
    ("Yarrow Moonshine", "Achillea 'Moonshine'", "Achillea", ""),
    ("Gaura Whirling Butterflies", "Gaura lindheimeri", "Gaura", "lindheimeri"),
    ("Switchgrass Shenandoah", "Panicum virgatum 'Shenandoah'", "Panicum", "virgatum"),
    ("Clematis Jackmanii", "Clematis x jackmanii", "Clematis", "jackmanii"),
    ("Blueberry Bluecrop", "Vaccinium corymbosum 'Bluecrop'", "Vaccinium", "corymbosum"),
    ("Creeping Thyme", "Thymus serpyllum", "Thymus", "serpyllum"),
    ("Foxglove Camelot", "Digitalis purpurea 'Camelot'", "Digitalis", "purpurea"),
    ("Columbine Origami", "Aquilegia caerulea 'Origami'", "Aquilegia", "caerulea"),
    ("Veronica Royal Candles", "Veronica spicata 'Royal Candles'", "Veronica", "spicata"),
]
VENDORS = ["Hollow Creek Nursery", "Bluebird Hill Gardens", "North Fork Growers", "Maple Ridge Perennials",
           "Quiet Meadow Farms", "Fernwood Greenhouses", "Stonebrook Nursery", "Sunny Acre Plants Co.",
           "Willow Bend Nurseries", "Pine Hollow Gardens"]
SIZES = ["#1", "#2", "1 gal", "2 gal", "4 in pot", "1 qt", "#3", "3.5 in pot"]
SUN = ["Full sun", "Full sun to part shade", "Part shade", "Shade to part shade"]
WATER = ["Low", "Average", "Medium", "Moist, well-drained soil"]
BLOOM = ["Late spring to summer", "Midsummer to frost", "Early spring", "Summer"]

FONT_DIRS = [Path("C:/Windows/Fonts"), Path("/usr/share/fonts/truetype/dejavu"),
             Path("/usr/share/fonts/truetype/liberation"), Path("/usr/share/fonts/dejavu"),
             Path("/usr/share/fonts/TTF")]
# (regular, bold, italic) candidates per family; the first family that exists is used.
FAMILIES = [("arial.ttf", "arialbd.ttf", "ariali.ttf"), ("georgia.ttf", "georgiab.ttf", "georgiai.ttf"),
            ("verdana.ttf", "verdanab.ttf", "verdanai.ttf"), ("times.ttf", "timesbd.ttf", "timesi.ttf"),
            ("calibri.ttf", "calibrib.ttf", "calibrii.ttf"), ("tahoma.ttf", "tahomabd.ttf", "tahoma.ttf"),
            ("DejaVuSans.ttf", "DejaVuSans-Bold.ttf", "DejaVuSans-Oblique.ttf"),
            ("DejaVuSerif.ttf", "DejaVuSerif-Bold.ttf", "DejaVuSerif-Italic.ttf"),
            ("LiberationSans-Regular.ttf", "LiberationSans-Bold.ttf", "LiberationSans-Italic.ttf")]


def _find(name: str) -> Path | None:
    for d in FONT_DIRS:
        if (d / name).is_file():
            return d / name
    return None


def families() -> list[tuple[Path, Path, Path]]:
    """The font families this computer has (regular, bold, italic files)."""
    out = []
    for r, b, i in FAMILIES:
        fr, fb, fi = _find(r), _find(b), _find(i)
        if fr and fb and fi:
            out.append((fr, fb, fi))
    return out


def ean13(rng: random.Random) -> str:
    """A 13-digit number with a correct check digit (prefix 0 = a UPC-A code)."""
    body = "0" + "".join(str(rng.randrange(10)) for _ in range(11))
    total = sum(int(c) * (3 if i % 2 else 1) for i, c in enumerate(body))
    return body + str((10 - total % 10) % 10)


_L = ["0001101", "0011001", "0010011", "0111101", "0100011", "0110001", "0101111", "0111011", "0110111", "0001011"]
_G = ["0100111", "0110011", "0011011", "0100001", "0011101", "0111001", "0000101", "0010001", "0001001", "0010111"]
_R = ["1110010", "1100110", "1101100", "1000010", "1011100", "1001110", "1010000", "1000100", "1001000", "1110100"]
_FIRST = ["LLLLLL", "LLGLGG", "LLGGLG", "LLGGGL", "LGLLGG", "LGGLLG", "LGGGLL", "LGLGLG", "LGLGGL", "LGGLGL"]


def barcode_image(code: str, module: int = 3, height: int = 150, font: ImageFont.FreeTypeFont | None = None) -> Image.Image:
    """An EAN-13 barcode picture with its digits underneath."""
    first, left, right = int(code[0]), code[1:7], code[7:]
    bits = "101" + "".join((_L if p == "L" else _G)[int(d)] for p, d in zip(_FIRST[first], left, strict=True)) + "01010" \
        + "".join(_R[int(d)] for d in right) + "101"
    w = (len(bits) + 18) * module
    img = Image.new("L", (w, height + 48), 255)
    d = ImageDraw.Draw(img)
    x0 = 9 * module
    for i, b in enumerate(bits):
        if b == "1":
            guard = i < 3 or 45 <= i < 50 or i >= len(bits) - 3
            d.rectangle([x0 + i * module, 0, x0 + (i + 1) * module - 1, height + (10 if guard else 0)], fill=0)
    f = font or ImageFont.load_default()
    d.text((x0 - 7 * module, height + 8), code[0], fill=0, font=f)
    d.text((x0 + 4 * module, height + 14), code[1:7], fill=0, font=f)
    d.text((x0 + 50 * module, height + 14), code[7:], fill=0, font=f)
    return img.convert("RGB")


def qr_image(text: str, size: int = 190) -> Image.Image | None:
    """A QR code picture, or None when the `qrcode` package is not installed."""
    try:
        import qrcode
    except ImportError:
        return None
    return qrcode.make(text).convert("RGB").resize((size, size), Image.NEAREST)


def flower(d: ImageDraw.ImageDraw, box: tuple[int, int, int, int], rng: random.Random) -> None:
    """A simple drawn flower for the picture on a tag (no real photo)."""
    x0, y0, x1, y1 = box
    cx, cy = (x0 + x1) // 2, (y0 + y1) // 2
    d.rectangle(box, fill=(rng.randrange(200, 250), rng.randrange(225, 250), rng.randrange(200, 245)))
    d.line([cx, y1, cx, cy], fill=(40, 120, 50), width=8)
    col = rng.choice([(150, 60, 170), (230, 90, 120), (240, 190, 30), (80, 110, 210), (240, 120, 40)])
    r = min(x1 - x0, y1 - y0) // 4
    for k in range(9):
        a = k * 2 * math.pi / 9
        px, py = cx + r * 0.9 * math.cos(a), cy + r * 0.9 * math.sin(a)
        d.ellipse([px - r // 2, py - r // 2, px + r // 2, py + r // 2], fill=col)
    d.ellipse([cx - r // 3, cy - r // 3, cx + r // 3, cy + r // 3], fill=(250, 220, 60))


def _font(path: Path, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(path), size)


def _fit(d, text: str, path: Path, start: int, width: int) -> ImageFont.FreeTypeFont:
    """The biggest font (from `start` down) in which `text` is at most `width` px wide."""
    size = start
    while size > 14:
        f = _font(path, size)
        if d.textlength(text, font=f) <= width:
            return f
        size -= 2
    return _font(path, 14)


def front(rng: random.Random, plant, vendor: str, size: str, code: str, fam, layout: int, caps: bool) -> Image.Image:
    """The front of a tag: name big, Latin name in italics, grower, pot size, barcode."""
    reg, bold, ital = fam
    W, H = 900, 1300
    paper = rng.choice([(250, 250, 244), (255, 255, 255), (245, 250, 240), (255, 252, 235)])
    img = Image.new("RGB", (W, H), paper)
    d = ImageDraw.Draw(img)
    name = plant[0].upper() if caps else plant[0]
    bot = plant[1]
    ink = rng.choice([(20, 20, 20), (25, 50, 30), (35, 35, 70)])
    band = rng.choice([(40, 110, 60), (90, 50, 120), (30, 90, 130), (150, 60, 50)])
    y = 40
    if layout == 0:      # grower on top, flower, centred name
        d.rectangle([0, 0, W, 110], fill=band)
        f = _fit(d, vendor, bold, 46, W - 80)
        d.text((W // 2, 55), vendor, font=f, fill=(255, 255, 255), anchor="mm")
        flower(d, (120, 150, W - 120, 560), rng)
        y = 600
        words = name.split()
        lines = [name] if len(name) <= 20 else [" ".join(words[: len(words) // 2 + len(words) % 2]), " ".join(words[len(words) // 2 + len(words) % 2:])]
        for ln in lines:
            f = _fit(d, ln, bold, 100, W - 100)
            d.text((W // 2, y), ln, font=f, fill=ink, anchor="mt")
            y += f.size + 14
        f = _fit(d, bot, ital, 48, W - 100)
        d.text((W // 2, y + 10), bot, font=f, fill=ink, anchor="mt")
        d.text((70, H - 330), size, font=_font(bold, 70), fill=ink)
    elif layout == 1:    # picture left/top, name in a band, grower at the bottom
        flower(d, (0, 0, W, 520), rng)
        d.rectangle([0, 520, W, 800], fill=band)
        words = name.split()
        lines = [name] if len(name) <= 18 else [" ".join(words[: (len(words) + 1) // 2]), " ".join(words[(len(words) + 1) // 2:])]
        yy = 545
        for ln in lines:
            f = _fit(d, ln, bold, 96, W - 80)
            d.text((50, yy), ln, font=f, fill=(255, 255, 255))
            yy += f.size + 10
        f = _fit(d, bot, ital, 44, W - 100)
        d.text((50, 830), bot, font=f, fill=ink)
        d.text((50, 930), size, font=_font(reg, 56), fill=ink)
        f = _fit(d, "Grown by " + vendor, reg, 40, W - 100)
        d.text((50, 1010), "Grown by " + vendor, font=f, fill=ink)
    else:                # plain tag: Latin name first, grower in a footer
        f = _fit(d, name, bold, 110, W - 100)
        d.text((50, 70), name, font=f, fill=ink)
        f = _fit(d, bot, ital, 52, W - 100)
        d.text((50, 70 + f.size * 2 + 20), bot, font=f, fill=ink)
        flower(d, (50, 400, W - 50, 800), rng)
        d.text((50, 830), size, font=_font(bold, 64), fill=ink)
        d.rectangle([0, H - 140, W, H], fill=band)
        f = _fit(d, vendor, bold, 52, W - 80)
        d.text((W // 2, H - 70), vendor, font=f, fill=(255, 255, 255), anchor="mm")
    bc = barcode_image(code, 3, 140, _font(reg, 26))
    # Layout 1 has the grower line at y 1010, so its barcode sits lower.
    img.paste(bc, (W - bc.width - 40, H - (250 if layout == 1 else 330)))
    return img


def back(rng: random.Random, plant, vendor: str, size: str, code: str, fam, with_barcode: bool, qr_text: str) -> Image.Image:
    """The back of a tag: the care facts, the grower and sometimes the barcode or a QR code."""
    reg, bold, ital = fam
    W, H = 900, 1300
    img = Image.new("RGB", (W, H), rng.choice([(252, 252, 248), (255, 255, 255), (246, 250, 244)]))
    d = ImageDraw.Draw(img)
    ink = (25, 25, 25)
    d.text((50, 40), "Plant care", font=_font(bold, 76), fill=ink)
    d.text((50, 140), plant[1], font=_fit(d, plant[1], ital, 44, W - 100), fill=ink)
    h1, h2 = rng.randrange(10, 40), rng.randrange(14, 36)
    rows = [f"Sun: {rng.choice(SUN)}", f"Water: {rng.choice(WATER)}", f"Zones {rng.randrange(3, 6)}-{rng.randrange(7, 10)}",
            f"Height: {h1}-{h1 + 8} in", f"Spread: {h2} in", f"Blooms: {rng.choice(BLOOM)}",
            f"Plant {rng.randrange(12, 36)} in apart", "Attracts butterflies and bees", "Deer resistant"]
    y = 240
    for r in rows:
        d.text((50, y), r, font=_font(reg, 50), fill=ink)
        y += 78
    d.text((50, y + 10), "Grown by " + vendor, font=_fit(d, "Grown by " + vendor, bold, 46, W - 100), fill=ink)
    d.text((50, y + 90), "www." + "".join(c for c in vendor.lower() if c.isalpha())[:14] + ".example", font=_font(reg, 38), fill=(60, 60, 60))
    if size:
        d.text((50, y + 160), size, font=_font(bold, 50), fill=ink)
    if with_barcode:
        bc = barcode_image(code, 3, 140, _font(reg, 26))
        img.paste(bc, (W - bc.width - 40, H - 260))
    q = qr_image(qr_text) if qr_text else None
    if q:
        img.paste(q, (50, H - 260))
    return img


def damage(img: Image.Image, rng: random.Random, level: str) -> Image.Image:
    """What a phone photo does to a tag: tilt, a little perspective, blur, shadow, noise, JPEG."""
    k = {"easy": 0.4, "normal": 1.0, "hard": 1.7}[level]
    bg = (rng.randrange(60, 140), rng.randrange(80, 150), rng.randrange(50, 110))   # soil / bench
    canvas = Image.new("RGB", (img.width + 260, img.height + 260), bg)
    canvas.paste(img, (130, 130))
    canvas = canvas.rotate(rng.uniform(-7, 7) * k, resample=Image.BICUBIC, fillcolor=bg)
    w, h = canvas.size
    j = 0.035 * k
    quad = [rng.uniform(0, w * j), rng.uniform(0, h * j), rng.uniform(0, w * j), h - rng.uniform(0, h * j),
            w - rng.uniform(0, w * j), h - rng.uniform(0, h * j), w - rng.uniform(0, w * j), rng.uniform(0, h * j)]
    canvas = canvas.transform(canvas.size, Image.QUAD, quad, resample=Image.BICUBIC, fillcolor=bg)
    # Uneven light: a diagonal gradient.
    big = int(max(canvas.size) * 1.5)
    grad = Image.linear_gradient("L").resize((big, big)).rotate(rng.uniform(0, 360))
    gx, gy = (big - canvas.width) // 2, (big - canvas.height) // 2
    grad = grad.crop((gx, gy, gx + canvas.width, gy + canvas.height))
    shade = rng.uniform(0.1, 0.35) * k
    canvas = Image.composite(canvas, Image.eval(canvas, lambda v: int(v * (1 - shade))), grad)
    canvas = canvas.filter(ImageFilter.GaussianBlur(rng.uniform(0.3, 1.4) * k))
    noise = Image.effect_noise(canvas.size, 18 * k).convert("RGB")
    canvas = Image.blend(canvas, noise, 0.05 * k)
    canvas.thumbnail((1500, 1500))
    return canvas


def make(out: Path, count: int = 40, seed: int = 1, level: str = "normal") -> list[dict]:
    """Make `count` tags (front.jpg, back.jpg each) in `out`; write and return the truth list."""
    fams = families()
    if not fams:
        raise RuntimeError("no fonts found (Windows Fonts or DejaVu/Liberation)")
    rng = random.Random(seed)  # noqa: S311 -- made-up tags, the same every time for one seed; no secrets
    out.mkdir(parents=True, exist_ok=True)
    truth = []
    for i in range(count):
        plant = PLANTS[i % len(PLANTS)]
        vendor, size, code = rng.choice(VENDORS), rng.choice(SIZES), ean13(rng)
        fam = rng.choice(fams)
        caps = rng.random() < 0.2
        layout = rng.randrange(3)
        with_bc_back = rng.random() < 0.5
        qr_text = f"https://{''.join(c for c in vendor.lower() if c.isalpha())[:12]}.example/p/{code[-5:]}" if rng.random() < 0.3 else ""
        d = out / f"tag{i + 1:03d}"
        d.mkdir(exist_ok=True)
        for side, im in (("front", front(rng, plant, vendor, size, code, fam, layout, caps)),
                         ("back", back(rng, plant, vendor, size, code, fam, with_bc_back, qr_text))):
            damage(im, rng, level).save(d / f"{side}.jpg", quality=rng.randrange(60, 90))
        truth.append({"dir": d.name, "name": plant[0], "botanical": plant[1], "genus": plant[2], "species": plant[3],
                      "size": size, "vendor": vendor, "barcode": code, "qr": qr_text, "layout": layout,
                      "font": fam[0].name, "caps": caps})
    (out / "truth.json").write_text(json.dumps(truth, indent=1), encoding="utf-8")
    return truth
