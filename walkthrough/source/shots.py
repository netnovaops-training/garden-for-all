#!/usr/bin/env python3
"""The walkthrough's pictures: each step's page, at desktop (1366) and phone (390) width, with the
thing to look at outlined. Taken from the guide garden (guide-site.sh: GitHub main + sample data).

    python shots.py BASE OUTDIR        (Jun's password for the copy in env GUIDE_PW)
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
