"""Screenshot side by side: the same page from the old build (:18101) and the new one (:18102)
at phone (390 px) and desktop (1366 px) width, compared pixel by pixel.

Runs inside the Playwright container (see `lab.sh shots`), on the NUC's own network:
    python shots.py pages.txt out-dir

pages.txt: one path per line; "@login path" to take it logged in. What is random on purpose is
pinned first, so only real differences show: Math.random is fixed, the colour theme class
(bg-theme-N) is set to 0, the hero photos and animations are switched off.
Writes <n>-<width>-{base,cand,diff}.png and prints the share of pixels that differ.
"""
import json
import os
import re
import sys

from PIL import Image, ImageChops
from playwright.sync_api import sync_playwright

BASE, CAND = "http://127.0.0.1:18101/", "http://127.0.0.1:18102/"
WIDTHS = (390, 1366)
PIN_JS = "Math.random = () => 0.42;"
PIN_CSS = """
*, *::before, *::after { animation: none !important; transition: none !important; caret-color: transparent !important; }
.hero-slide, .hero-banner, [class*='hero'] { background-image: none !important; }
video { visibility: hidden !important; }
"""
LOGIN = "login.php"


def pin_html(route):
    """What the server randomises is fixed on both sides: the colour theme class becomes theme 0,
    and the gallery's shuffled photo list is put in file-name order."""
    r = route.fetch()
    body = r.text() if "html" in r.headers.get("content-type", "") else None
    if body is None:
        return route.fulfill(response=r)
    body = re.sub(r"bg-theme-\d", "bg-theme-0", body)
    marker = "const ALL_PHOTOS = "
    at = body.find(marker)
    if at >= 0:
        try:
            photos, end = json.JSONDecoder().raw_decode(body, at + len(marker))
            photos.sort(key=lambda p: p.get("fn", ""))
            body = body[:at] + marker + json.dumps(photos) + body[end:]
        except ValueError:
            pass
    route.fulfill(response=r, body=body)


def log_in(page, root, pw):
    page.goto(root + LOGIN)
    page.fill("input[name=admin_user]", "Tester")
    page.fill("input[name=admin_password]", pw)
    page.click("button[name=admin_login]")
    page.wait_for_load_state("networkidle")


def shot(browser, root, path, width, out, auth, pw):
    ctx = browser.new_context(viewport={"width": width, "height": 900}, device_scale_factor=1,
                              extra_http_headers={"X-Forwarded-For": "10.9.8.8"})
    ctx.add_init_script(PIN_JS)
    page = ctx.new_page()
    page.route("**/*", pin_html)
    if auth:
        log_in(page, root, pw)
    page.goto(root + path, wait_until="networkidle")
    page.add_style_tag(content=PIN_CSS)
    page.wait_for_timeout(800)
    page.screenshot(path=out, full_page=True)
    ctx.close()


def compare(a, b, out):
    ia, ib = Image.open(a).convert("RGB"), Image.open(b).convert("RGB")
    if ia.size != ib.size:
        return f"size {ia.size} vs {ib.size}"
    d = ImageChops.difference(ia, ib)
    box = d.getbbox()
    if not box:
        return "0.000%"
    changed = sum(1 for p in d.getdata() if p != (0, 0, 0))
    d.point(lambda v: 255 if v else 0).save(out)
    return f"{100 * changed / (ia.size[0] * ia.size[1]):.3f}% (area {box})"


def main():
    pages = [l.strip() for l in open(sys.argv[1]) if l.strip() and not l.startswith("#")]
    out = sys.argv[2]
    os.makedirs(out, exist_ok=True)
    pw = dict(l.strip().split("=", 1) for l in open("test-login.env") if "=" in l)["TEST_PW"]
    with sync_playwright() as p:
        browser = p.chromium.launch()
        for n, line in enumerate(pages, 1):
            auth = line.startswith("@login ")
            path = line.removeprefix("@login ").strip()
            for w in WIDTHS:
                files = [os.path.join(out, f"{n}-{w}-{s}.png") for s in ("base", "cand", "diff")]
                try:
                    shot(browser, BASE, path, w, files[0], auth, pw)
                    shot(browser, CAND, path, w, files[1], auth, pw)
                    print(f"{n:3} {w:5} {line[:60]:60} {compare(*files)}", flush=True)
                except Exception as e:  # report and go on with the next page
                    print(f"{n:3} {w:5} {line[:60]:60} ERROR {str(e)[:120]}", flush=True)
        browser.close()


if __name__ == "__main__":
    main()
