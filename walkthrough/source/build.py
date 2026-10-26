#!/usr/bin/env python3
"""guide-en.md / guide-ko.md -> HTML -> PDF (headless Chromium), so the walkthroughs can be made
again on every release.  python build.py SHOTS_DIR APP_SHOTS_DIR OUT_DIR

  @shot NAME   the step's picture at desktop width, with the phone picture beside it
  @app NAME    a phone screenshot of the app (from the emulator test)
"""
import html
import pathlib
import re
import sys

import markdown
from playwright.sync_api import sync_playwright

HERE = pathlib.Path(__file__).resolve().parent
SHOTS, APPS, OUT = (pathlib.Path(a).resolve() for a in sys.argv[1:4])
OUT.mkdir(parents=True, exist_ok=True)

CSS = """
@page { size: A4; margin: 14mm 12mm; }
body { font-family: 'Noto Sans', 'Noto Sans CJK KR', sans-serif; font-size: 11.5pt; line-height: 1.5; color: #1d2b1d; }
h1 { color: #2e6b2e; border-bottom: 3px solid #9ccc65; padding-bottom: 4px; }
h2 { color: #2e6b2e; margin-top: 26px; page-break-after: avoid; }
li { margin: 6px 0; }
.shot { display: flex; gap: 10px; align-items: flex-start; margin: 8px 0 14px; page-break-inside: avoid; }
.shot img { border: 1px solid #b7c9b7; border-radius: 6px; }
.shot .desk { width: 74%; } .shot .phone { width: 22%; }
.app img { width: 38%; border: 1px solid #b7c9b7; border-radius: 10px; margin: 8px 0 14px; }
code { background: #eef5ea; padding: 1px 4px; border-radius: 3px; }
em { color: #555; }
"""


def expand(md):
    def shot(m):
        n = m.group(1)
        d, p = SHOTS / f"{n}-desktop.png", SHOTS / f"{n}-phone.png"
        return (f'\n<div class="shot"><img class="desk" src="{d.as_uri()}">'
                f'<img class="phone" src="{p.as_uri()}"></div>\n')

    def app(m):
        return f'\n<div class="app"><img src="{(APPS / (m.group(1) + ".png")).as_uri()}"></div>\n'

    md = re.sub(r"^\s*@shot (\S+)\s*$", shot, md, flags=re.M)
    return re.sub(r"^\s*@app (\S+)\s*$", app, md, flags=re.M)


with sync_playwright() as p:
    browser = p.chromium.launch()
    for lang in ("en", "ko"):
        text = expand((HERE / f"guide-{lang}.md").read_text(encoding="utf-8"))
        body = markdown.markdown(text, extensions=["extra"])
        title = html.escape(text.splitlines()[0].lstrip("# "))
        page_html = (f'<!doctype html><html lang="{lang}"><head><meta charset="utf-8"><title>{title}</title>'
                     f"<style>{CSS}</style></head><body>{body}</body></html>")
        src = OUT / f"guide-{lang}.html"
        src.write_text(page_html, encoding="utf-8")
        page = browser.new_page()
        page.goto(src.as_uri(), wait_until="load")
        name = "JunsGarden-guide-English.pdf" if lang == "en" else "JunsGarden-guide-Korean.pdf"
        page.pdf(path=str(OUT / name), format="A4", print_background=True,
                 margin={"top": "14mm", "bottom": "14mm", "left": "12mm", "right": "12mm"})
        page.close()
        print(f"PASS {name}")
    browser.close()
