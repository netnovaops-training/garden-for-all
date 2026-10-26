"""Find source pages for a plant.

vendor page  : the grower's own product page (cultivar facts, photos, product_url).
               PW sitemap match first; else a DuckDuckGo search kept to vendor sites.
species pages: NC State Plant Toolbox + up to 2 trusted .edu/.gov/botanical-garden pages,
               shared by every cultivar of that species (or genus, for hybrids).
DuckDuckGo searches are serialised with >= 3 s between them and cached in SQLite.
"""
import re
import threading
import time
import unicodedata
from urllib.parse import urlsplit

from . import db, fetch, pages

# Vendors (grower/nursery own sites), US/English sites. Monrovia answers 403 to honest crawlers.
VENDORS = ["provenwinners.com", "waltersgardens.com", "bloomineasy.com", "davidaustinroses.com",
           "monrovia.com", "firsteditionsplants.com", "spring-meadow.com", "starrosesandplants.com",
           "southernlivingplants.com", "endlesssummerblooms.com", "terranovanurseries.com",
           "plantsnouveau.com", "intrinsicperennialgardens.com", "skagitgardens.com"]
# Species sources: universities/extension, government, botanical gardens / plant societies.
TRUSTED = [".edu", ".gov", "rhs.org.uk", "wildflower.org", "mortonarb.org", "chicagobotanic.org",
           "nybg.org", "powo.science.kew.org", "gobotany.nativeplanttrust.org", "plants.usda.gov",
           "aspca.org", "lewisginter.org", "bbg.org", "brooklynbotanic.org", "longwoodgardens.org",
           "plantdelights.com"]
NOT_ENGLISH = re.compile(r"\.(de|fr|nl|it|es|jp|cn|kr|pl|se|dk|be|ch|at)(/|$)|/(de|fr|nl|it|es|ja|zh|ko)(-[a-z]{2})?/")
_ddg_lock = threading.Lock()
_ddg_last = [0.0]
_pw_index: list[str] = []
_pw_lock = threading.Lock()


def _ascii(s: str) -> str:
    return unicodedata.normalize("NFKD", s).encode("ascii", "ignore").decode()


def toks(s: str | None) -> list[str]:
    s = _ascii(s or "").lower().replace("&", " and ")
    s = re.sub(r"\b(us)?pp\s*#?\s*[\d,]+|ppaf|®|™|\btm\b", " ", s)
    return [w for w in re.findall(r"[a-z0-9]+", s) if w not in {"x", "the", "and", "a", "of", "plant", "hybrid", "sp", "spp"}]


def search(q: str, n: int = 10) -> list[dict]:
    """DuckDuckGo text search (ddgs), cached; >= 3 s between live searches."""
    k = "ddg:" + q
    hit = db.shared_get(k)
    if hit is not None:
        return hit
    from ddgs import DDGS
    with _ddg_lock:
        res = None
        for attempt in range(4):
            wait = 3.0 - (time.monotonic() - _ddg_last[0])
            if wait > 0:
                time.sleep(wait)
            try:
                res = [{"href": r.get("href", ""), "title": r.get("title", "")}
                       for r in DDGS().text(q, max_results=n, backend="duckduckgo")]
                break
            except Exception:  # noqa: BLE001 -- the search site said no: wait longer, try again
                time.sleep(20 * (attempt + 1))
            finally:
                _ddg_last[0] = time.monotonic()
    if res is None:
        return []
    db.shared_put(k, res)
    return res


def pw_index() -> list[str]:
    with _pw_lock:
        if not _pw_index:
            for p in (1, 2):
                _pw_index.extend(u for u in re.findall(r"<loc>([^<]+)</loc>",
                                 fetch.text(f"https://www.provenwinners.com/sitemap.xml?page={p}"))
                                 if "/plants/" in u)
    return _pw_index


def genus_species(rec: dict) -> tuple[str | None, str | None]:
    b = _ascii(rec.get("botanical_name") or "")
    m = re.match(r"\s*([A-Z][a-z]+)\s+(?:x\s+|×\s*)?([a-z][a-z-]+)", b)
    first = re.match(r"\s*([A-Z][a-z]+)", b)
    g = (rec.get("genus") or "").strip() or (first.group(1) if first else None)
    sp = m.group(2) if m and m.group(2) not in ("hybrid", "sp", "spp", "var", "cv") else None
    if not sp and (rec.get("species") or "").strip():
        s = rec["species"].strip().split()[-1].lower()
        sp = s if s not in ("hybrid", "hybrida", "sp", "spp", "x") and re.fullmatch(r"[a-z-]+", s) else None
    return (g.capitalize() if g else None), sp


def cultivar_tokens(rec: dict) -> list[str]:
    """Words that name this cultivar (from quotes in names, else the trade name)."""
    names = f"{rec.get('plant_name') or ''} {rec.get('botanical_name') or ''}"
    q = re.findall(r"['‘’]([^'‘’]{2,})['‘’]", names)
    if q:
        return toks(" ".join(q))
    g, sp = genus_species(rec)
    drop = set(toks(f"{g or ''} {sp or ''}"))
    return [w for w in toks(rec.get("plant_name")) if w not in drop]


def pw_match(rec: dict) -> str | None:
    g, _ = genus_species(rec)
    want = set(toks(rec.get("plant_name")))
    cv = set(cultivar_tokens(rec))
    best, score = None, 0.0
    for u in pw_index():
        parts = u.rstrip("/").split("/")
        if g and parts[-2] != g.lower():
            continue
        slug = set(parts[-1].split("-"))
        if cv and not cv <= slug:
            continue
        s = len(want & slug) / max(1, len(want))
        if s > score:
            best, score = u, s
    return best if score >= 0.6 else None


def _host(u: str) -> str:
    return urlsplit(u).netloc.lower()


def vendor_page(rec: dict) -> str | None:
    """Best vendor product page for this plant, or None."""
    u = pw_match(rec)
    if u:
        return u
    cv = cultivar_tokens(rec)
    if not cv:
        return None
    g, _ = genus_species(rec)
    q = f"{rec.get('plant_name') or ''} {g or ''}".strip()
    for r in search(q):
        h, href = _host(r["href"]), r["href"]
        if NOT_ENGLISH.search(href) or not any(h == v or h.endswith("." + v) for v in VENDORS):
            continue
        hay = set(toks(href.replace("/", " ") + " " + r["title"]))
        if set(cv) <= hay:
            return href
    return None


SOCIAL_HOSTS = ("instagram.com", "facebook.com", "pinterest.com", "reddit.com", "x.com", "twitter.com",
                "tiktok.com", "pinimg.com", "fbcdn.net", "cdninstagram.com", "redd.it", "twimg.com")


def social_images(rec: dict, n: int = 5) -> list[tuple[str, str]]:
    """Last resort: public images on social sites (no login, robots.txt still checked by fetch).
    Gives [(image_url, post_url)]; cached like other searches."""
    q = f"{rec.get('plant_name') or ''} {genus_species(rec)[0] or ''} plant"
    k = "ddgimg:" + q
    res = db.shared_get(k)
    if res is None:
        from ddgs import DDGS
        with _ddg_lock:
            wait = 3.0 - (time.monotonic() - _ddg_last[0])
            if wait > 0:
                time.sleep(wait)
            try:
                res = [{"image": r.get("image", ""), "url": r.get("url", "")}
                       for r in DDGS().images(q, max_results=30, backend="duckduckgo")]
            except Exception:  # noqa: BLE001 -- no image search today: no social photos
                res = []
            finally:
                _ddg_last[0] = time.monotonic()
        db.shared_put(k, res)
    out = []
    for r in res:
        h = _host(r["url"])
        if any(h == s or h.endswith("." + s) for s in SOCIAL_HOSTS) and r["image"]:
            out.append((r["image"], r["url"]))
    return out[:n]


def species_pages(rec: dict) -> tuple[str | None, list[str]]:
    """(key, up to 3 species/genus page URLs). Key is shared by every plant of that species."""
    g, sp = genus_species(rec)
    if not g:
        return None, []
    key = f"{g} {sp}" if sp else g
    hit = db.shared_get("sp:" + key)
    if hit is not None:
        return key, hit
    urls = []
    nc = f"https://plants.ces.ncsu.edu/plants/{g.lower()}{'-' + sp if sp else ''}/"
    if pages.read(nc):
        urls.append(nc)
    for r in search(f'"{key}" plant care' if sp else f'"{g}" genus plant'):
        h, href = _host(r["href"]), r["href"]
        if len(urls) >= 3:
            break
        if href in urls or NOT_ENGLISH.search(href) or href.lower().endswith(".pdf"):
            continue
        if any(h.endswith(t) if t.startswith(".") else (h == t or h.endswith("." + t)) for t in TRUSTED):
            p = pages.read(href)
            if p and key.lower().split()[0] in " ".join(p["lines"][:40] + [p["title"]]).lower():
                urls.append(href)
    db.shared_put("sp:" + key, urls)
    return key, urls
