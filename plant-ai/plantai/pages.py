"""Turn a cached page into (title, numbered text lines, image URLs) for the LLM.

Lines are leaf text blocks (a paragraph, a list item, one spec row such as "Plant Type: Annual");
menus, footers, reviews and related plants are cut so prompts stay short. Lines are what the
model cites as its quotes. Image URLs are listed biggest-first (full-size links, og:image, the
largest srcset entry) so the photo stage starts from originals, never thumbnails.
"""
import re
from urllib.parse import urljoin, urlsplit

from selectolax.parser import HTMLParser

from . import fetch

LEAF = "p, li, h1, h2, h3, h4, dt, dd, td, th, div.field-item, div.field-label, span.label"
MAX_CHARS = 6000
IMG_EXT = re.compile(r"\.(jpe?g|png|webp)(\?|$)", re.IGNORECASE)


def on_site(host: str, domain: str) -> bool:
    """True for `domain` itself and its sub-names (www.domain, plants.domain)."""
    return host == domain or host.endswith("." + domain)


def _text(n) -> str:
    return re.sub(r"\s+", " ", n.text(separator=" ")).strip()


def _lines(root) -> list[str]:
    out, seen = [], set()
    for n in root.css(LEAF):
        if n.css_first(LEAF):   # has a smaller block inside: use the smaller ones instead
            continue
        t = _text(n)
        if len(t) < 3 or t in seen:
            continue
        seen.add(t)
        out.append(t[:600])
    return out


def _largest_srcset(s: str) -> str | None:
    best, bw = None, -1
    for part in s.split(","):
        bits = part.strip().split()
        if not bits:
            continue
        w = int(bits[1][:-1]) if len(bits) > 1 and bits[1].endswith("w") and bits[1][:-1].isdigit() else 0
        if w > bw:
            best, bw = bits[0], w
    return best


def read(url: str) -> dict | None:
    """{title, lines, images, host, url} for a 200 HTML page, else None."""
    r = fetch.get(url)
    if r["status"] != 200 or "html" not in r.get("type", ""):
        return None
    t = HTMLParser(fetch.text(url))
    h1 = t.css_first("h1")
    title = (h1.text(separator=" ", strip=True) if h1 else "") or \
            (t.css_first("title").text(strip=True) if t.css_first("title") else "")
    title = re.sub(r"\s+", " ", title)
    base = r["url"]
    host = (urlsplit(base).hostname or "").lower()
    # The site's own name, or one of its sub-names (never just "contains": evil-site.com/x.ncsu.edu).
    pw = on_site(host, "provenwinners.com")
    imgs: list[str] = []
    if pw:
        # The plant's own gallery ("View larger image" links); other plants only appear as <img>.
        imgs = [a.attributes.get("href") for a in t.css("a")
                if "/ifa_upload/" in (a.attributes.get("href") or "")]
    else:
        for a in t.css("a"):
            href = a.attributes.get("href") or ""
            if IMG_EXT.search(href):
                imgs.append(urljoin(base, href))
        for m in t.css('meta[property="og:image"]'):
            imgs.append(urljoin(base, m.attributes.get("content") or ""))
        for i in t.css("img, source"):
            ss = i.attributes.get("srcset") or i.attributes.get("data-srcset")
            big = _largest_srcset(ss) if ss else None
            src = big or i.attributes.get("data-src") or i.attributes.get("src") or ""
            if src and not src.startswith("data:"):
                imgs.append(urljoin(base, src))
    for n in t.css("script, style, noscript, svg, nav, header, footer, form, iframe"):
        n.decompose()
    if pw:
        parts = t.css("div.fieldgroup")
        lines = [title] + [ln for p in parts for ln in _lines(p)]
    else:
        root = t.css_first("main") or t.css_first("article") or t.body
        lines = _lines(root) if root else []
        if on_site(host, "ces.ncsu.edu"):
            s = next((i for i, ln in enumerate(lines) if ln.startswith("Description")), 0)
            lines = lines[s:]
    size, out = 0, []
    for ln in lines:
        if size + len(ln) > MAX_CHARS:
            break
        out.append(ln)
        size += len(ln)
    seen, uniq = set(), []
    for i in imgs:
        if i and i not in seen:
            seen.add(i)
            uniq.append(i)
    return {"title": title, "lines": out, "images": uniq, "host": host, "url": base}


def numbered(page: dict) -> str:
    return "\n".join(f"{i + 1}| {ln}" for i, ln in enumerate(page["lines"]))
