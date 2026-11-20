#!/usr/bin/env python3
"""The walkthrough's pictures: each step's page, at desktop (1366) and phone (390) width, with the
thing to look at outlined. Taken from the guide garden (guide-site.sh: GitHub main + sample data).

    python shots.py BASE OUTDIR        (Jun's password for the copy in env GUIDE_PW;
                                        SHOTS_ONLY="20 3" takes only those pictures again)
Prints one PASS/FAIL line per picture.
"""
import os
import sys

from playwright.sync_api import sync_playwright

BASE, OUT = sys.argv[1].rstrip("/"), sys.argv[2]
PW = os.environ["GUIDE_PW"]
WIDTHS = {"desktop": (1366, 900), "phone": (390, 844)}
os.makedirs(OUT, exist_ok=True)


def search(text):
    def act(page):
        box = page.locator("input[type=search], input[name=term], input[name=search], input[type=text]").first
        box.fill(text)
        box.press("Enter")
        page.wait_for_timeout(1500)
    return act


def fill(*pairs):
    def act(page):
        for sel, v in pairs:
            page.locator(sel).first.fill(v)
    return act


def click(sel, wait=2000):
    def act(page):
        page.locator(sel).first.click()
        page.wait_for_timeout(wait)
    return act


def scroll_to(sel):
    def act(page):
        page.locator(sel).first.scroll_into_view_if_needed()
        page.wait_for_timeout(500)
    return act


def wait(ms):
    return lambda page: page.wait_for_timeout(ms)


def js(code, ms=2500):
    """Run the page's own function (what its button would call), then wait."""
    def act(page):
        page.evaluate(code)
        page.wait_for_timeout(ms)
    return act


def center(sel, ms=700):
    """Scroll so the thing sits in the middle of the screen (the full manual's pictures)."""
    def act(page):
        page.evaluate("sel => { const e = document.querySelector(sel); if (e) e.scrollIntoView({block: 'center'}); }", sel)
        page.wait_for_timeout(ms)
    return act


def top(sel, ms=700):
    """Scroll so the thing sits at the top of the screen."""
    def act(page):
        page.evaluate("sel => { const e = document.querySelector(sel); if (e) e.scrollIntoView({block: 'start'}); }", sel)
        page.wait_for_timeout(ms)
    return act


def typed(sel, text, ms=1500):
    """Type into a box key by key (so suggestions open), without pressing Enter."""
    def act(page):
        page.locator(sel).first.click()
        page.locator(sel).first.press_sequentially(text, delay=60)
        page.wait_for_timeout(ms)
    return act


def pick(sel, label, ms=1200):
    """Choose an item of a drop-down by its words (as a person would), then tell the page."""
    def act(page):
        page.evaluate(
            """([sel, label]) => { const s = document.querySelector(sel);
                 const o = [...s.options].find(o => o.textContent.trim() === label);
                 s.value = o.value; s.dispatchEvent(new Event('change', {bubbles: true})); }""",
            [sel, label])
        page.wait_for_timeout(ms)
    return act


# The photo-position button only shows when the pointer is over a card; phones have no pointer.
SHOW_REPO = js("document.querySelectorAll('.svd-repo-btn').forEach(b => b.style.opacity = '1')", 200)


# (picture, needs Jun's login, address, steps before the picture, what to outline)
STEPS = [
    ("01-login", False, "/login.php", [], "form"),
    ("02-forgot", False, "/login.php?forgot=1", [], "form"),
    ("03-home-signed-in", True, "/index.php", [], "nav, header"),
    ("04-admin-tabs", True, "/streamlit/", [], None),
    ("05-add-plant", True, "/streamlit/?tab=1",
     [fill(("input[name=name]", "Hidcote Lavender"), ("input[name=bot]", "Lavandula angustifolia"))], "input[name=name]"),
    ("06-add-plant-next", True, "/streamlit/?tab=1",
     [fill(("input[name=name]", "Hidcote Lavender"), ("input[name=bot]", "Lavandula angustifolia")),
      lambda page: (page.locator("input[name=bot]").press("Enter"), page.wait_for_timeout(2500))],
     "button[name=go]"),
    ("06b-add-plant-saved", True, "/streamlit/?tab=1",
     [fill(("input[name=name]", "Hidcote Lavender"), ("input[name=bot]", "Lavandula angustifolia")),
      lambda page: (page.locator("input[name=bot]").press("Enter"), page.wait_for_timeout(2500)),
      click("button[name=go]", 3000)], None),
    ("07-edit-plant", True, "/streamlit/?tab=4", [], "input[name=botanical_name]"),
    ("08-plant-window", True, "/catalog.php?plant=2", [wait(2500)], None),
    ("09-remove-plant", True, "/catalog.php?plant=2",
     [wait(2500), js("window.editPlant(2)"), scroll_to("#qty-event-type")], "#qty-log-section"),
    ("10-photos-admin", True, "/streamlit/?tab=5", [], None),
    ("11-gallery", False, "/gallery.php", [wait(1500)], None),
    ("12-beds", True, "/streamlit/?tab=12", [], None),
    ("13-labels", True, "/labels", [], ".lbls"),
    ("14-guestbook", False, "/index.php", [scroll_to("textarea[name=user_message], input[name=user_message]")],
     "textarea[name=user_message], input[name=user_message]"),
    ("15-plant-comments", False, "/catalog.php?plant=2",
     [wait(2500), click("[onclick*='openLightbox']", 2500)], "#lb-comments-list"),
    ("16-quick-search", False, "/plant_listing.php", [search("rose")], None),
    ("17-catalog-search", False, "/catalog.php", [wait(1500), search("hosta")], None),
    ("18-app-setup", False, "/app/setup", [], "img"),
    ("19-address", True, "/labels", [scroll_to("form[action='/labels/address']")], "form[action='/labels/address']"),
    # ---- the full manual (v2): every page and its parts ----
    # Getting around
    ("20-nav", True, "/index.php", [], "header nav"),
    ("21-site-theme", False, "/index.php", [click(".st-toggle", 800)], ".st-panel"),
    ("22-home-weather", False, "/index.php", [center("#forecast-container", 2500)], "#forecast-container"),
    ("23-home-posts", True, "/index.php", [top("button[onclick*='blog-modal']", 1500)], "button[onclick*='blog-modal']"),
    ("23b-new-post", True, "/index.php",
     [js("document.getElementById('blog-modal').style.display = 'flex'", 800)], "#blog-modal form"),
    ("24-blog", True, "/blog.php", [], None),
    # The Full Catalog
    ("25-catalog-login", False, "/catalog.php", [wait(1500)], "#admin-login-form"),
    ("26-catalog-admin-box", True, "/catalog.php", [wait(1500)], ".admin-bar"),
    ("27-catalog-bulk", True, "/catalog.php", [wait(1500), js("window.toggleBulkMode()", 800)], "#bulk-toolbar"),
    ("28-catalog-search-box", False, "/catalog.php", [wait(1500), center("#search-box")], "#search-box"),
    ("29-catalog-suggestions", False, "/catalog.php", [wait(1500), top(".stats-pill-container"), typed("#search-box", "Lil", 2000)],
     "#ac-dropdown"),
    ("30-catalog-az-quick", False, "/catalog.php", [wait(2000), top("#az-bar")], ".quick-filters"),
    ("31-catalog-views-tags", True, "/catalog.php", [wait(2000), center(".views-filter-bar")], ".views-filter-bar"),
    ("32-catalog-beds", False, "/catalog.php", [wait(2000), center(".bed-search-grid"), click(".bed-group-btn", 800)],
     ".bed-search-grid"),
    ("33-catalog-card-theme", False, "/catalog.php", [wait(2500), center("#svd-theme-picker-wrap")], "#svd-theme-picker-wrap"),
    ("34-catalog-cards", False, "/catalog.php", [wait(2500), top("#plant-feed")], ".sv-grid-card"),
    ("35-catalog-card-owner", True, "/catalog.php", [wait(2500), top("#plant-feed"), SHOW_REPO], ".svd-repo-btn"),
    ("36-focus-point", True, "/catalog.php",
     [wait(2500), top("#plant-feed"), SHOW_REPO,
      js("document.querySelector('.svd-photo img').parentElement.querySelector('.svd-repo-btn').click()", 800)],
     ".svd-repo-save-bar"),
    ("37-plant-window-owner", True, "/catalog.php?plant=10", [wait(2500)], ".tg-admin-actions"),
    ("38-default-photo", True, "/catalog.php?plant=10", [wait(2500), center(".tg-setdef")], ".tg-setdef"),
    ("39-plant-window-facts", False, "/catalog.php?plant=10", [wait(2500), center(".tg-right")], None),
    ("40-search-results", False, "/catalog.php", [wait(1500), search("Bees"), top("#plant-feed")], None),
    ("41-search-az", False, "/catalog.php", [wait(2000), click("#az-bar .az-btn", 2500), top("#plant-feed")], None),
    # Gallery, Data, Help
    ("42-gallery-tag", False, "/gallery.php", [wait(2500), typed("#search-input-wrap input", "Veg", 1500)], "#search-input-wrap"),
    ("43-data-widgets", False, "/data.php", [wait(2500)], "#picker-chips"),
    ("44-data-crossref", False, "/data.php",
     [wait(2500), js("document.getElementById('compare-tool').open = true", 600),
      pick("#compare-chart-a", "Garden Bed"), pick("#compare-chart-b", "Sun Exposure", 3000), center("#compare-output")],
     "#compare-tool"),
    ("45-data-cards", False, "/data.php", [wait(4000), top("#widget-grid", 2500)], None),
    ("46-help", True, "/help", [], None),
    ("47-help-ko", True, "/help?lang=ko", [], None),
    # The admin page, tab by tab
    ("48-admin-ai-review", True, "/streamlit/?tab=2", [], None),
    ("49-admin-fill-missing", True, "/streamlit/?tab=3", [], None),
    ("50-admin-editor-sections", True, "/streamlit/?tab=4&p=2", [center("h4", 300)], None),
    ("51-admin-image-filter", True, "/streamlit/?tab=5", [], "form"),
    ("52-admin-partials", True, "/streamlit/?tab=6", [], None),
    ("53-admin-awaiting", True, "/streamlit/?tab=7", [], None),
    ("54-admin-approved", True, "/streamlit/?tab=8", [], None),
    ("55-admin-flagged", True, "/streamlit/?tab=9", [], None),
    ("56-admin-ai-missing", True, "/streamlit/?tab=10", [], None),
    ("57-admin-pipeline", True, "/streamlit/?tab=11", [], None),
    ("58-admin-pipeline-import", True, "/streamlit/?tab=11&sub=4", [], None),
    ("59-admin-field-reference", True, "/streamlit/?tab=11&sub=6", [], None),
    ("60-admin-wishlist", True, "/streamlit/?tab=13", [], None),
    ("61-admin-barcodes", True, "/streamlit/?tab=14", [], None),
    ("62-add-wishlist-plant", True, "/streamlit/?tab=11&sub=5", [], None),
]


def outline(page, sel):
    if not sel:
        return
    page.evaluate(
        """sel => { const e = document.querySelector(sel);
                    if (e) { e.style.outline = '4px solid #e8590c'; e.style.outlineOffset = '3px'; } }""",
        sel,
    )


def main():
    ok = True
    with sync_playwright() as p:
        browser = p.chromium.launch()
        for name, (w, h) in WIDTHS.items():
            ctx = browser.new_context(viewport={"width": w, "height": h}, locale="en-US")
            # Jun's login for this copy (the same form a person fills in).
            ctx.request.post(f"{BASE}/login.php", form={
                "admin_user": "Jun", "admin_password": PW, "admin_login": "", "back": "/"})
            guest = browser.new_context(viewport={"width": w, "height": h}, locale="en-US")
            for shot, login, path, steps, sel in STEPS:
                # SHOTS_ONLY="20 3" takes again only the pictures whose names start so.
                only = os.environ.get("SHOTS_ONLY", "").split()
                if only and not any(shot.startswith(o) for o in only):
                    continue
                page = (ctx if login else guest).new_page()
                try:
                    page.goto(BASE + path, wait_until="networkidle")
                    for step in steps:
                        step(page)
                    outline(page, sel)
                    page.screenshot(path=os.path.join(OUT, f"{shot}-{name}.png"), full_page=False)
                    print(f"PASS {shot} {name}", flush=True)
                except Exception as e:  # noqa: BLE001 -- report, then the next picture
                    ok = False
                    print(f"FAIL {shot} {name}: {str(e)[:140]}", flush=True)
                page.close()
            ctx.close()
            guest.close()
        browser.close()
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
