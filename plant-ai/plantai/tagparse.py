"""Turn the text an OCR engine read from a plant tag into fields: plant name, botanical name, size,
vendor, barcode. Pure text work (no images, no network), so it is easy to test.

The input is a list of lines, each {"text": str, "conf": 0-100, "h": text height in px or 0}. Both
OCR engines in plant-ai/ocr give that shape. Nursery tags put the common name in the LARGEST type
on the front, the Latin name in italics under it, the grower's name at the bottom and the facts
(sun, water, zones, size) on the back, so the tallest line that is not a fact or a grower is the
name.
"""
import re

# Words that start a care fact or are boilerplate: never a plant name.
FACT = re.compile(
    r"^\W*(full\s*sun|part(ial)?\s*(sun|shade)|shade|sun|water|zones?|hardy|height|spread|width|"
    r"bloom(s)?|blooms?|plant\s*(spacing|in|after|care)|spacing|space|soil|deer|attracts|fertili[sz]e|"
    r"care|notes?|light|exposure|mature|grows?|type|perennial|annual|shrub|tree|vine|"
    r"keep|do not|please|warning|caution|wholesale|retail|patent|pp\s*#|pat\.|made in|www\.|http|"
    r"price|\$|upc|sku|item|lot|qty|net)\b",
    re.I,
)
# A grower's name: ends in or holds one of these words.
VENDOR_WORD = re.compile(
    r"\b(nursery|nurseries|gardens?|growers?|farms?|plants?\s*(co|inc|llc)|greenhouses?|"
    r"perennials|propagators?|botanicals?|horticulture|co\.|inc\.?|llc|ltd)\b",
    re.I,
)
VENDOR_LEAD = re.compile(r"\b(grown|produced|distributed|packed|propagated|brought to you)\s*by\b[:\s]*(.*)", re.I)
# A pot size. Inches only count when "pot" follows ("4 in pot", '4" pot'): a bare "18 in" is far
# more often a height or spread from the care facts.
SIZE = re.compile(
    r"(#\s?\d{1,2}\b|\b\d+(\.\d+)?\s?(gal(lon)?s?|qt|quarts?|liters?|litres?|l)\b|"
    r"\b\d+(\.\d+)?\s?(\"|''|in(ch(es)?)?)\s*pots?\b|\b\d+\s?-?\s?(pack|pk|cell)\b|"
    r"\b(quart|gallon|bare\s*root|plug|liner)\b)",
    re.I,
)
# Popular garden genera: used to put the space back in a Latin name an engine ran together
# ('Echinaceapurpurea' -> 'Echinacea purpurea'). The garden's own genera are added at run time.
GENERA = set("""Acer Achillea Aconitum Actaea Agastache Ajuga Alchemilla Allium Amelanchier Amsonia Anemone Aquilegia Armeria
Artemisia Aruncus Asclepias Aster Astilbe Astrantia Baptisia Begonia Bergenia Berberis Betula Brunnera Buddleja Buxus Calamagrostis
Callicarpa Campanula Carex Caryopteris Ceanothus Centranthus Cercis Chaenomeles Chionanthus Chrysanthemum Cimicifuga Clematis Cleome
Clethra Convallaria Coreopsis Cornus Cotinus Cotoneaster Crocosmia Crocus Cyclamen Dahlia Delphinium Dianthus Dicentra Digitalis
Echinacea Echinops Epimedium Erigeron Eryngium Euonymus Eupatorium Euphorbia Fagus Festuca Filipendula Forsythia Fothergilla
Gaillardia Galium Gaura Gentiana Geranium Geum Gillenia Gypsophila Hakonechloa Hamamelis Helenium Helianthus Helleborus Hemerocallis
Heuchera Hibiscus Hosta Hydrangea Hypericum Iberis Ilex Iris Itea Juniperus Kalmia Knautia Kniphofia Lamium Lamprocapnos Lavandula
Leucanthemum Liatris Ligularia Lilium Liriope Lobelia Lonicera Lupinus Lysimachia Magnolia Mahonia Malva Miscanthus Monarda Muscari
Myosotis Narcissus Nepeta Nigella Oenothera Origanum Osmanthus Paeonia Panicum Papaver Penstemon Perovskia Persicaria Phlox Physocarpus
Physostegia Picea Pieris Pinus Platycodon Polemonium Polygonatum Potentilla Primula Prunus Pulmonaria Pulsatilla Quercus Ratibida Rhododendron
Rodgersia Rosa Rosmarinus Rudbeckia Salvia Sambucus Sanguisorba Santolina Saxifraga Scabiosa Sedum Sempervivum Solidago Spigelia Spiraea
Stachys Stokesia Symphyotrichum Syringa Tanacetum Thalictrum Thuja Thymus Tiarella Tradescantia Tricyrtis Trillium Tulipa Vaccinium
Verbena Verbascum Veronica Veronicastrum Viburnum Vinca Viola Weigela Wisteria Yucca Zinnia""".split())
EXTRA_GENERA: set[str] = set()
# A Latin name: Genus species (with an optional x/hybrid mark and cultivar in quotes), or Genus 'Cultivar'.
QUOTE = r"[\'‘’“”\"`]"
QUOTE_RE = re.compile(QUOTE)
BOT_FULL = re.compile(
    rf"\b([A-Z][a-z]{{3,}})\s+([x×]\s+)?([a-z]{{3,}})(?:\s+(?:var\.|subsp\.|ssp\.)\s*[a-z]+)?(\s*{QUOTE}[A-Za-z][^\'‘’“”\"`]{{1,40}}(?:{QUOTE}|$))?"
)
# The closing quote may be missing: engines often drop it at the end of a line.
BOT_CULTIVAR = re.compile(rf"\b([A-Z][a-z]{{3,}})\s*({QUOTE}[A-Za-z][^\'‘’“”\"`]{{1,40}}(?:{QUOTE}|$))")
DIGITS = re.compile(r"(?<!\d)(\d[\d \-]{10,16}\d)(?!\d)")
URL = re.compile(r"(https?://[^\s]+|www\.[^\s]+)", re.I)
# Words that look Latin but are not a genus.
NOT_GENUS = {"Full", "Part", "Plant", "Water", "Zones", "Bloom", "Blooms", "Height", "Spread", "Shade", "Grown",
             "Produced", "Please", "Keep", "Notes", "Deer", "Attracts", "Hardy", "Spacing", "Perennial", "Annual",
             "Garden", "Gardens", "Nursery", "Sunny", "Moist", "Well", "Average", "Medium", "Light", "Spring",
             "Summer", "Autumn", "Winter", "Early", "Late", "Mature", "Grows", "Wholesale", "Distributed"}


def unglue(s: str) -> str:
    """Words an engine ran together ('EndlessSummerHydrangea') get their spaces back, at each
    small-to-capital letter step, but only inside long words (so 'McIntosh' is left alone)."""
    return " ".join(re.sub(r"(?<=[a-z])(?=[A-Z])", " ", w) if len(w) >= 9 else w for w in s.split(" "))


def split_latin(s: str) -> str:
    """'Echinaceapurpurea' becomes 'Echinacea purpurea' and 'Calamagrostisx acutiflora' becomes
    'Calamagrostis x acutiflora', when the word starts with a known genus (the longest one wins)."""
    def one(m: re.Match) -> str:
        w = m.group(0)
        for g in sorted(GENERA | EXTRA_GENERA, key=len, reverse=True):
            if w.startswith(g) and len(w) > len(g):
                rest = w[len(g):]
                if rest == "x" or (len(rest) >= 4 and rest.islower()):
                    return f"{g} {rest}"
        return w
    return re.sub(r"\b[A-Z][a-z]{6,}x?\b", one, s)


def clean(s: str) -> str:
    """One line of OCR text with the usual damage undone: odd spaces, glued words, stray symbols
    at the ends."""
    s = (s or "").replace(" ", " ")
    s = split_latin(unglue(re.sub(r"\s+", " ", s).strip()))
    return s.strip("|_~`^*<>[]{}\\/ \t-.,:;")


def ean_ok(code: str) -> bool:
    """Does an 8-, 12- or 13-digit number pass the barcode check digit?"""
    d = [int(c) for c in code]
    if len(d) not in (8, 12, 13):
        return False
    body, chk = d[:-1], d[-1]
    # Weights alternate 3,1 from the right-most body digit.
    total = sum(v * (3 if i % 2 == 0 else 1) for i, v in enumerate(reversed(body)))
    return (10 - total % 10) % 10 == chk


def find_barcode(lines: list[dict], hint: str = "") -> str:
    """The barcode number (or the QR/URL text) in the lines: a number with a good check digit wins;
    else a long number; else the phone's own reading (`hint`)."""
    best = ""
    for ln in lines:
        for m in DIGITS.finditer(ln.get("text", "")):
            raw = re.sub(r"\D", "", m.group(1))
            if ean_ok(raw):
                return raw
            if len(raw) in (8, 12, 13) and not best:
                best = raw
    return hint.strip() or best


def find_botanical(lines: list[dict]) -> str:
    """The Latin name: 'Salvia nemorosa 'May Night'', 'Hosta 'Blue Angel'' or 'Echinacea purpurea'."""
    for ln in lines:
        t = clean(ln.get("text", ""))
        for m in BOT_FULL.finditer(t):
            if m.group(1) in NOT_GENUS or m.group(3) in {"and", "the", "for", "with", "from", "more", "less"}:
                continue
            out = f"{m.group(1)} {'x ' if m.group(2) else ''}{m.group(3)}"
            if m.group(4):
                out += " " + norm_quotes(m.group(4).strip())
            return out
    for ln in lines:
        t = clean(ln.get("text", ""))
        m = BOT_CULTIVAR.search(t)
        if m and m.group(1) not in NOT_GENUS:
            return f"{m.group(1)} {norm_quotes(m.group(2))}"
    return ""


def norm_quotes(s: str) -> str:
    """Curly and odd quote marks around a cultivar become plain single quotes, and a closing quote
    the engine dropped is put back."""
    s = re.sub(QUOTE, "'", s).strip()
    s = re.sub(r"'\s+", "'", re.sub(r"\s+'", " '", s)) if s else s
    return s + "'" if s.count("'") == 1 else s


def best_botanical(front: str, back: str) -> str:
    """The front's and the back's Latin names are the same plant: keep the fuller one (the one that
    names the cultivar, then the longer). With only one, that one. When they disagree, the front."""
    from . import catalog
    if not front or not back:
        return front or back
    if catalog.botanical_sim(front, back) < 0.5 and catalog.botanical_sim(back, front) < 0.5:
        return front
    key = lambda b: ("'" in b, len(b))  # noqa: E731
    return max((front, back), key=key)


def find_size(lines: list[dict]) -> str:
    """The pot size as printed ('#1', '1 gal', '4 in pot'), or ''."""
    for ln in lines:
        t = clean(ln.get("text", ""))
        m = SIZE.search(t)
        if m and not FACT.match(t.lstrip()):
            return re.sub(r"\s+", " ", m.group(0)).strip()
    return ""


def find_vendor(lines: list[dict]) -> str:
    """The grower: text after 'Grown by', else a line holding Nursery/Gardens/Farms/... ."""
    for ln in lines:
        m = VENDOR_LEAD.search(clean(ln.get("text", "")))
        if m and len(m.group(2).strip()) >= 3:
            return clean(m.group(2))
    for ln in lines:
        t = clean(ln.get("text", ""))
        if VENDOR_WORD.search(t) and len(t) <= 60 and not FACT.match(t):
            return t
    return ""


def find_name(lines: list[dict], botanical: str, vendor: str, size: str) -> str:
    """The common name: the tallest line that is not a fact, the Latin name, the grower, the size or
    a number. Two lines of nearly the same height next to each other are one wrapped name."""
    cands = []
    for i, ln in enumerate(lines):
        t = clean(ln.get("text", ""))
        # An engine sometimes joins the name and the Latin name under it into one line: cut at the
        # Latin name (known genus, or a quoted cultivar) and keep what comes before it.
        m = BOT_FULL.search(t) or BOT_CULTIVAR.search(t)
        if m and (m.group(1) in GENERA | EXTRA_GENERA or QUOTE_RE.search(t[m.start():])):
            t = t[:m.start()].strip()
        letters = sum(c.isalpha() for c in t)
        if letters < 3 or letters < len(t) * 0.55:
            continue
        if FACT.match(t) or (vendor and t == vendor) or (size and t == size):
            continue
        if botanical and (t == botanical or BOT_FULL.fullmatch(t) or BOT_CULTIVAR.fullmatch(t)):
            continue
        if VENDOR_WORD.search(t) and len(t) <= 60:
            continue
        if URL.search(t):
            continue
        cands.append((float(ln.get("h") or 0), -i, i, t))
    if not cands:
        return ""
    top = max(cands)
    name, idx, h = top[3], top[2], top[0]
    # A wrapped name: the line just above or below, of about the same height and close by, joins it.
    if h > 0:
        for j in (idx - 1, idx + 1):
            nb = next((c for c in cands if c[2] == j), None)
            if not nb or abs(nb[0] - h) > 0.35 * h:
                continue
            a, b = lines[idx].get("box"), lines[j].get("box")
            if a and b and max(a[1], b[1]) - min(a[3], b[3]) > 1.0 * h:
                continue
            name = f"{nb[3]} {name}" if j < idx else f"{name} {nb[3]}"
            break
    return name


def parse(lines: list[dict], hint: str = "") -> dict:
    """All fields from the OCR lines of one or more tag photos (front first). Lines the engine itself
    doubts (confidence under 35) are left out: they are mostly pictures and shadows."""
    lines = [ln for ln in lines if ln.get("conf", 100) >= 35]
    bot = find_botanical(lines)
    vendor = find_vendor(lines)
    size = find_size(lines)
    return {
        "name": find_name(lines, bot, vendor, size),
        "botanical": bot,
        "size": size,
        "vendor": vendor,
        "barcode": find_barcode(lines, hint),
    }
