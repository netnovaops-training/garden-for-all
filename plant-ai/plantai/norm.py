"""Field normaliser + validator: turns a source value into the exact form the garden's DB
already uses (the DB's existing values win over the CSV / code lists).

norm(field, value) -> (value or None, note). None means "could not be made valid".
valid(field, value) -> True when the value is already in DB form (used by the audit).
"""
import re

MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August",
          "September", "October", "November", "December"]
MLET = "JFMAMJJASOND"

# One-of-list fields. First the forms the living-garden DB uses; code-list values the DB
# has not used yet are still allowed (they are in opts.rs). "Unknown" is never written:
# no source -> null.
ENUM = {
    "plant_type": ["Perennial", "Annual", "Biennial", "Shrub", "Tree", "Succulent", "Vine"],
    "native_status": ["Cultivar", "Native", "Non-native", "Introduced"],
    "growth_rate": ["Slow", "Moderate", "Fast"],
    "sun_exposure": ["Full Sun", "Partial Shade", "Full Sun - Partial Shade", "Full Shade",
                     "Full Sun - Full Shade", "Partial Shade - Full Shade"],
    "deer_resistance": ["Deer Resistant", "Rabbit Resistant", "Both", "Not Resistant"],
    "toxicity": ["Non-toxic", "Toxic", "Toxic to Dogs/Cats", "Toxic to Humans",
                 "Toxic to Humans & Pets"],
    "leaf_shape": ["Linear (Long, narrow)", "Lanceolate (Long tapering tip)", "Ovate (Egg-shaped)",
                   "Elliptic (Oval tapering ends)", "Cordate (Heart-shaped)",
                   "Palmate (Hand-like lobes)", "Deltoid (Triangular)"],
    "leaf_type": ["Simple (Single undivided blade)", "Pinnately Compound (Feather-like leaflets)",
                  "Palmately Compound (Radiating leaflets)"],
    "leaf_margin": ["Entire (Smooth, no teeth)", "Serrated (Saw-toothed edges)",
                    "Shallowly Serrate", "Lobed (Deep indentations)"],
    "flower_symmetry": ["Actinomorphic (Radial symmetry)", "Zygomorphic (Bilateral symmetry)", "None"],
    "flower_shape": ["Rotate (Flat wheel-shaped)", "Stellate (Star-shaped)", "Stellate (Chionodoxa)",
                     "Cruciate (Cross-shaped)", "Campanulate (Bell-shaped)",
                     "Infundibuliform (Funnel-shaped)", "Tubular (Cylindrical tube)",
                     "Salverform (Tube opening flat)", "Bilabiate (Two-lipped)",
                     "Urceolate (Urn-shaped)", "Trumpet/Cup-shaped (Daffodil)", "High-centered",
                     "Double Blossoms", "None"],
    "flower_size": ["Tiny (<1 in)", "Small (1-2 in)", "Small to Large (1-5 in)", "Medium (2-4 in)",
                    "Large (4-6 in)", "Massive (>6 in)", "None"],
    "reproduction_type": ["Hermaphrodite", "Monoecious", "Dioecious", "Polygamous"],
    "fruit_edibility": ["Edible", "Inedible", "Toxic", "Wildlife Only", "None"],
    "root_system": ["Fibrous/Adventitious Systems", "Taproot Systems", "Specialized Systems"],
    "root_morphology": ["Fine Roots (< 2 mm)", "Coarse Roots (> 2 mm)", "Mixed Fine & Coarse"],
    "root_function": ["Absorptive Roots (Lower Orders)", "Transport Roots (Higher Orders)", "Mixed Orders"],
    "invasive_local": ["No", "Yes", "Watchlist"],
    "is_evergreen": ["Deciduous", "Herbaceous", "Evergreen", "Semi-evergreen", "Summer Dormant",
                     "Spring Ephemeral", "Deciduous Conifer"],
    "water_needs": ["Low", "Low to Average", "Average", "Average to High", "High"],
    "month_start": MONTHS, "month_end": MONTHS,
    "planting_month_start": MONTHS, "planting_month_end": MONTHS,
}
# Comma lists drawn from a fixed vocabulary, written in this order.
VOCAB = {
    "attracts": ["Bees", "Butterflies", "Hummingbirds", "Birds", "Beneficial Insects", "Pollinators"],
    "soil_type": ["Loamy", "Sandy", "Clay", "Rocky", "Well-draining", "Moist", "Wet", "Rich",
                  "Average fertility", "Poor", "Adaptable", "Acidic", "Alkaline"],
    "fertilizer": ["Plant-tone", "Holly-tone", "Rose-tone", "Tree-tone", "Flower-tone", "Evergreen-tone",
                   "Bulb-tone", "Garden-tone", "Azalea-tone", "Bloom Booster", "Berry-tone", "Bio-tone",
                   "Citrus-tone", "Iron-tone", "Palm-tone", "Tomato-tone", "None"],
}
SEASONS = ["Spring", "Summer", "Fall", "Winter"]
FREE = {"care_notes", "maintenance_notes", "cleanup_notes", "disease_susceptibility", "extra_details",
        "cuttings", "propagation_method", "planting_depth", "aromatics", "bloom_color", "foliage_color",
        "stem_color", "fruit_type", "fruit_shape", "fruit_color", "fruit_size", "companion_plants",
        "family", "genus", "species", "botanical_name", "life_expectancy", "chilling_hours"}
SENTENCE = {"care_notes", "maintenance_notes", "cleanup_notes", "extra_details"}
# Fields the import must never write (read-only in the JSON Pipeline) — never filled here.
READ_ONLY = {"id", "plant_name", "quantity", "year_added", "garden_beds", "picture", "image_search_term",
             "approved_jason", "approved_jun", "needs_review", "image_verified", "date_added"}
# Every plant-info field this job fills.
FIELDS = ["botanical_name", "family", "genus", "species", "plant_type", "native_status", "plant_length",
          "plant_width", "spacing", "zones", "life_expectancy", "growth_rate", "sun_exposure",
          "water_needs", "soil_ph", "soil_type", "planting_depth", "chilling_hours", "foliage_color",
          "stem_color", "is_evergreen", "leaf_shape", "leaf_type", "leaf_margin", "bloom_color",
          "aromatics", "flower_symmetry", "flower_shape", "flower_size", "reproduction_type",
          "fruit_type", "fruit_shape", "fruit_color", "fruit_size", "fruit_edibility", "root_system",
          "root_morphology", "root_function", "attracts", "deer_resistance", "toxicity",
          "invasive_local", "propagation_method", "cuttings", "companion_plants", "fertilizer",
          "grow_months", "bloom_months", "care_notes", "maintenance_notes", "cleanup_notes",
          "disease_susceptibility", "extra_details", "season", "grow_season", "month_start",
          "month_end", "planting_month_start", "planting_month_end"]

ALIASES = {
    "sun_exposure": {"partial": "Partial Shade", "part sun": "Partial Shade", "part shade": "Partial Shade",
                     "partial sun": "Partial Shade", "shade": "Full Shade", "sun": "Full Sun",
                     "full sun - partial": "Full Sun - Partial Shade", "full sun to part shade":
                     "Full Sun - Partial Shade", "sun to part shade": "Full Sun - Partial Shade",
                     "full shade - partial shade": "Partial Shade - Full Shade",
                     "part shade to full shade": "Partial Shade - Full Shade"},
    "water_needs": {"moderate": "Average", "medium": "Average", "dry": "Low", "low to moderate":
                    "Low to Average", "dry to medium": "Low to Average", "medium to wet": "Average to High",
                    "moderate to high": "Average to High", "wet": "High"},
    "deer_resistance": {"deer": "Deer Resistant", "rabbit": "Rabbit Resistant",
                        "deer and rabbit": "Both", "deer, rabbits": "Both"},
    "toxicity": {"toxic to pets": "Toxic to Dogs/Cats", "toxic to dogs and cats": "Toxic to Dogs/Cats",
                 "nontoxic": "Non-toxic", "non toxic": "Non-toxic"},
    "is_evergreen": {"semi evergreen": "Semi-evergreen", "semi-evergreen": "Semi-evergreen"},
    "growth_rate": {"medium": "Moderate", "rapid": "Fast"},
    "native_status": {"cultivar": "Cultivar", "not native": "Non-native"},
}


def _clean(v) -> str:
    return re.sub(r"\s+", " ", str(v)).strip().strip(".;,").strip()


def _num(s: str) -> str:
    f = float(s)
    return str(int(f)) if f == int(f) else str(round(f, 2)).rstrip("0").rstrip(".")


def _range(lo: str, hi: str | None) -> str:
    return _num(lo) if not hi or float(hi) == float(lo) else f"{_num(lo)}-{_num(hi)}"


def months_code(v) -> str | None:
    """'....MJJAS...' from a 12-char code, a list of month numbers, or month names/ranges."""
    if isinstance(v, list):
        nums = {int(x) for x in v if str(x).isdigit() and 1 <= int(x) <= 12}
    else:
        s = re.sub(r"\s+", " ", str(v)).strip()   # keep the dots: they are part of the code
        if re.fullmatch(r"[JFMASOND.]{12}", s):
            return s if all(c in (".", MLET[i]) for i, c in enumerate(s)) else None
        names = [m[:3].lower() for m in MONTHS]
        found = [names.index(w[:3].lower()) + 1 for w in re.findall(r"[A-Za-z]{3,}", s)
                 if w[:3].lower() in names]
        if not found:
            return None
        nums = set(found)
        if len(found) == 2 and re.search(r"[-–]|\bto\b|through", s):
            a, b = found
            nums = set(range(a, b + 1)) if a <= b else set(range(a, 13)) | set(range(1, b + 1))
    if not nums:
        return None
    return "".join(MLET[i] if i + 1 in nums else "." for i in range(12))


def seasons_from_months(code: str | None) -> str | None:
    """'Spring - Summer' from a month code (Mar-May Spring, Jun-Aug Summer, Sep-Nov Fall, Dec-Feb Winter)."""
    if not code:
        return None
    on = {i + 1 for i, c in enumerate(code) if c != "."}
    if len(on) == 12:
        return "Year-Round"
    groups = [("Spring", {3, 4, 5}), ("Summer", {6, 7, 8}), ("Fall", {9, 10, 11}), ("Winter", {12, 1, 2})]
    names = [n for n, ms in groups if on & ms]
    return " - ".join(names) or None


def _vocab(field: str, s: str) -> str | None:
    words = VOCAB[field]
    low = s.lower()
    syn = {"attracts": {"pollinator": "Pollinators", "hummingbird": "Hummingbirds", "butterfl": "Butterflies",
                        "bee": "Bees", "bird": "Birds", "beneficial": "Beneficial Insects"},
           "soil_type": {"loam": "Loamy", "sand": "Sandy", "clay": "Clay", "rock": "Rocky", "gravel": "Rocky",
                         "drain": "Well-draining", "moist": "Moist", "wet": "Wet", "rich": "Rich",
                         "fertile": "Rich", "organic": "Rich", "average": "Average fertility",
                         "poor": "Poor", "adaptable": "Adaptable", "tolerat": "Adaptable",
                         "acid": "Acidic", "alkaline": "Alkaline"},
           "fertilizer": {w.lower(): w for w in words}}[field]
    if field == "attracts" and low in ("none", "no"):
        return "None"
    got = {v for k, v in syn.items() if k in low and k != "bird"}
    if field == "attracts" and re.search(r"(?<!humming)\bbirds?\b", low):
        got.add("Birds")
    if field == "fertilizer":
        return ", ".join(sorted(got)) or None   # the DB lists fertilizers alphabetically
    if field == "soil_type" and "well-drain" in low or "well drain" in low:
        got.add("Well-draining")
    if field == "soil_type" and "poorly drain" in low:
        got.discard("Well-draining")
        got.discard("Poor")
        got.add("Wet")
    out = [w for w in words if w in got]
    return ", ".join(out) or None


def _size(s: str, apart: bool) -> str | None:
    t = s.lower().replace("–", "-").replace("—", "-").replace("to", "-")
    t = re.sub(r"(\d)\s*'\s*(\d+)\s*\"", lambda m: m.group(0), t)
    m = re.search(r"(\d+(?:\.\d+)?)\s*(?:'|ft|feet|foot)?\s*(?:-\s*(\d+(?:\.\d+)?))?\s*('|ft|feet|foot|\"|in|inch|inches|cm|m\b)", t)
    if not m:
        return None
    lo, hi, unit = m.group(1), m.group(2), m.group(3)
    if unit in ("cm", "m"):
        return None  # metric only: let the imperial line on the page supply it
    unit = "ft" if unit in ("'", "ft", "feet", "foot") else "in"
    out = f"{_range(lo, hi)} {unit}"
    return out + " apart" if apart else out


def norm(field: str, value) -> tuple[str | None, str]:
    """Normalise one value for one field. Gives (value, note); value None = rejected."""
    if value is None or (isinstance(value, str) and not value.strip()):
        return None, "empty"
    if field in READ_ONLY:
        return None, "read-only field"
    if field in ("grow_months", "bloom_months"):
        c = months_code(value)
        return (c, "") if c and c != "............" else (None, "not a month list")
    s = _clean(value if not isinstance(value, list) else ", ".join(map(str, value)))
    if field in ENUM:
        opts = ENUM[field]
        for o in opts:
            if s.lower() == o.lower():
                return o, ""
        a = ALIASES.get(field, {}).get(s.lower())
        if a:
            return a, f"alias {s!r}"
        if field in ("month_start", "month_end", "planting_month_start", "planting_month_end"):
            c = months_code(s)
            if c and c.count(".") == 11:
                return MONTHS[next(i for i, x in enumerate(c) if x != ".")], ""
        # Prefix match on the code list's short name, e.g. "Ovate" -> "Ovate (Egg-shaped)".
        hits = [o for o in opts if o.lower().split(" (")[0] == s.lower().split(" (")[0]]
        if len(hits) == 1:
            return hits[0], f"short form {s!r}"
        return None, f"not in allowed list: {s!r}"
    if field in VOCAB:
        v = _vocab(field, s)
        return (v, "") if v else (None, f"no known words: {s!r}")
    if field in ("season", "grow_season"):
        if "year" in s.lower() and "round" in s.lower():
            return "Year-Round", ""
        got = [x for x in SEASONS if x.lower() in s.lower() or (x == "Fall" and "autumn" in s.lower())]
        if not got:
            c = months_code(s)
            v = seasons_from_months(c)
            return (v, "from months") if v else (None, f"no season words: {s!r}")
        return " - ".join(got), ""
    if field == "zones":
        s = re.sub(r"\(.*?\)", " ", s)   # "10-11 (annual in zones 1-9)": the note is not the range
        nums = [int(n) for n in re.findall(r"\b(\d{1,2})[ab]?\b", s) if 1 <= int(n) <= 13]
        if not nums:
            return None, f"no zone numbers: {s!r}"
        lo, hi = min(nums), max(nums)
        return (f"{lo}-{hi}" if hi != lo else str(lo)), ""
    if field == "soil_ph":
        nums = [float(n) for n in re.findall(r"\d+(?:\.\d+)?", s) if 3 <= float(n) <= 10]
        if not nums:
            return None, f"no pH numbers: {s!r}"
        lo, hi = min(nums), max(nums)
        return (f"{lo:.1f}-{hi:.1f}" if hi != lo else f"{lo:.1f}"), ""
    if field in ("plant_length", "plant_width", "spacing"):
        v = _size(s, field == "spacing")
        return (v, "") if v else (None, f"no size: {s!r}")
    if field == "genus":
        w = s.split()[0] if s.split() else ""
        return (w.capitalize(), "") if re.fullmatch(r"[A-Za-z-]+", w) else (None, f"bad genus {s!r}")
    if field == "species":
        w = s.split()[-1].lower() if s.split() else ""
        if w in ("hybrid", "hybrida"):
            return "hybrid", ""   # the DB writes hybrids as species "hybrid"
        if w in ("x", "sp", "spp") or not re.fullmatch(r"[a-z-]+", w):
            return None, f"no species epithet in {s!r}"
        return w, ""
    if field == "family":
        return (s.capitalize(), "") if re.fullmatch(r"[A-Za-z]+aceae|Compositae|Leguminosae|Gramineae|Labiatae|Umbelliferae|Cruciferae|Palmae|Guttiferae", s, re.IGNORECASE) else (None, f"not a family name {s!r}")
    if field == "botanical_name":
        t = re.sub(r"\b(US)?PP\s*#?\s*[\d,]+|\bPPAF\b|\bCPBR\s*#?\s*[\d,]+|[®™]", "", s)
        t = re.sub(r"[‘’`]", "'", t)
        t = re.sub(r"\s+", " ", t).strip()
        return (t, "") if re.match(r"^[A-Z][a-z-]+(\s|$)", t) else (None, f"not a botanical name {s!r}")
    if field == "chilling_hours":
        m = re.search(r"(\d[\d,]*)\s*(?:-\s*(\d[\d,]*))?\s*(\+)?", s)
        if not m:
            return None, f"no hours {s!r}"
        lo, hi = m.group(1).replace(",", ""), (m.group(2) or "").replace(",", "")
        return (_range(lo, hi or None) + (m.group(3) or "")), ""
    if field == "life_expectancy":
        m = re.search(r"(\d+)\s*(?:-\s*(\d+))?\s*(\+)?\s*(year|yr)", s.lower())
        if not m:
            return None, f"no years {s!r}"
        r = _range(m.group(1), m.group(2)) + (m.group(3) or "")
        return (r + (" year (annual)" if r == "1" else " years")), ""
    if field in FREE:
        if len(s) > 240:
            s = s[:240].rsplit(" ", 1)[0]
        if s[:1].isalpha() and field != "fruit_size":
            s = s[0].upper() + s[1:]
        # The DB ends these notes with a period; every other text field without one.
        return (s + "." if field in SENTENCE else s), ""
    return None, f"unknown field {field}"


def valid(field: str, value) -> bool:
    """True when the value is already exactly in DB form (the audit test)."""
    if value is None or str(value).strip() == "":
        return True
    v, _ = norm(field, value)
    return v is not None and v == str(value).strip()


def cultivar_of(botanical: str | None, plant_name: str | None) -> str | None:
    """The cultivar epithet ('Koren') from a botanical name or plant name, if any."""
    for s in (botanical or "", plant_name or ""):
        m = re.search(r"['‘’]([^'‘’]{2,})['‘’]", s)
        if m:
            return m.group(1).strip()
    return None
