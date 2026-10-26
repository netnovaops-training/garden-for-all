"""Search the garden's own plants (current and wish list) for what a tag says.

The website gives the PC a short list of every plant (`GET /plant-ai/catalog`); matching happens
here, so the PC needs no database login. A match is scored from the evidence it has: the barcode
(exact), the common name and the Latin name (both fuzzy: tags are read with small mistakes), and
those pieces of evidence are combined so that two agreeing clues are worth more than one.
"""
import difflib
import re
import unicodedata

STOP = {"the", "a", "an", "of", "and", "plant", "plants", "hybrid", "x", "var", "cv", "sp", "spp", "pp", "ppaf"}


def norm(s: str) -> str:
    """Lower case, no accents, no punctuation, single spaces."""
    s = unicodedata.normalize("NFKD", s or "").encode("ascii", "ignore").decode().lower()
    return re.sub(r"\s+", " ", re.sub(r"[^a-z0-9 ]", " ", s.replace("'", " "))).strip()


def tokens(s: str) -> list[str]:
    return [w for w in norm(s).split() if w not in STOP]


def sim(a: str, b: str) -> float:
    """How alike two names are, 0 to 1: the better of the shared-words share and a letter-by-letter
    comparison of the words in order-free form (so 'Salvia, May Night' ~ 'May Night Salvia')."""
    ta, tb = tokens(a), tokens(b)
    if not ta or not tb:
        return 0.0
    # Words match when equal or nearly equal (one OCR slip).
    used, hit = set(), 0.0
    for w in ta:
        best, bj = 0.0, -1
        for j, v in enumerate(tb):
            if j in used:
                continue
            r = 1.0 if w == v else difflib.SequenceMatcher(None, w, v).ratio()
            if r > best:
                best, bj = r, j
        if best >= 0.8:
            used.add(bj)
            hit += best
    share = hit / max(len(ta), len(tb))
    letters = difflib.SequenceMatcher(None, " ".join(sorted(ta)), " ".join(sorted(tb))).ratio()
    # The same letters in the same order, whatever the spaces ('EndlessSummer' = 'Endless Summer').
    flat = difflib.SequenceMatcher(None, "".join(ta), "".join(tb)).ratio()
    return max(share, letters if letters >= 0.85 else 0.0, flat if flat >= 0.85 else 0.0)


def split_botanical(b: str) -> tuple[str, str, list[str]]:
    """(genus, species, cultivar words) of a Latin name like Salvia nemorosa 'May Night'."""
    cult = re.findall(r"['‘’\"]([^'‘’\"]{2,})['‘’\"]", b or "")
    base = re.sub(r"['‘’\"][^'‘’\"]*['‘’\"]", " ", b or "")
    words = [w for w in norm(base).split() if w not in STOP]
    genus = words[0] if words else ""
    species = words[1] if len(words) > 1 else ""
    return genus, species, tokens(" ".join(cult))


def botanical_sim(a: str, b: str) -> float:
    """How alike two Latin names are. Same genus and species with no cultivar named on either side
    is a full match; a cultivar that differs makes it a different plant."""
    ga, sa, ca = split_botanical(a)
    gb, sb, cb = split_botanical(b)
    if not ga or not gb:
        return 0.0
    g = difflib.SequenceMatcher(None, ga, gb).ratio()
    if g < 0.85:
        return 0.0
    s = difflib.SequenceMatcher(None, sa, sb).ratio() if sa and sb else (1.0 if not sa and not sb else 0.5)
    base = 0.4 * g + 0.6 * (s if s >= 0.85 else 0.0) if (sa or sb) else g
    if ca and cb:
        cs = sim(" ".join(ca), " ".join(cb))
        return base * (0.4 + 0.6 * cs) if cs < 0.8 else base
    if ca or cb:
        return base * 0.7        # one side names a cultivar and the other does not
    return base


def combine(parts: list[float]) -> float:
    """Several clues that each say 'probably' make a stronger 'probably': 1 - product of doubts."""
    p = 1.0
    for e in parts:
        p *= 1.0 - max(0.0, min(0.99, e))
    return min(0.99, 1.0 - p)


class Catalog:
    """The garden's plants, searchable by barcode, name and Latin name."""

    def __init__(self, plants: list[dict]):
        self.plants = [p for p in plants if p.get("name") or p.get("botanical")]
        self.by_code: dict[str, list[dict]] = {}
        for p in self.plants:
            for c in p.get("barcodes", []):
                # A UPC-A (12 digits) is the same code as its EAN-13 with a 0 in front.
                k = re.sub(r"\D", "", c).lstrip("0")
                if k:
                    self.by_code.setdefault(k, []).append(p)

    def lookup(self, name: str = "", botanical: str = "", barcode: str = "", limit: int = 5) -> list[dict]:
        """Candidates from the garden, best first: {plant, confidence, why}. Plants marked died are
        kept (the same plant can come back) but score a little lower."""
        code = re.sub(r"\D", "", barcode or "").lstrip("0")
        out = []
        for p in self.plants:
            parts, why = [], []
            if code and len(code) >= 8 and p in self.by_code.get(code, []):
                parts.append(0.97)
                why.append("barcode matches")
            n = sim(name, p.get("name", "")) if name else 0.0
            if n >= 0.6:
                parts.append(0.9 * n ** 1.5)
                why.append(f"name {round(n * 100)}% alike")
            b = botanical_sim(botanical, p.get("botanical", "")) if botanical else 0.0
            if b >= 0.5:
                parts.append(0.85 * b ** 1.5)
                why.append(f"Latin name {round(b * 100)}% alike")
            if not parts:
                continue
            conf = combine(parts)
            if p.get("died"):
                conf *= 0.9
            if conf >= 0.2:
                out.append({"plant": p, "confidence": round(conf, 3), "why": ", ".join(why)})
        out.sort(key=lambda x: -x["confidence"])
        return out[:limit]
