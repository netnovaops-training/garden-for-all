#!/usr/bin/env python3
"""guide-en.md / guide-ko.md -> HTML -> PDF (headless Chromium), the same way as the main
walkthrough (walkthrough/source/build.py).   python build.py SHOTS_DIR OUT_DIR

  @shot NAME   that step's picture in the guide's language: desktop width, phone beside it
  @desk NAME   the desktop picture only (full width)
"""
import html
import pathlib
import re
import sys

import markdown
from playwright.sync_api import sync_playwright

HERE = pathlib.Path(__file__).resolve().parent
SHOTS, OUT = (pathlib.Path(a).resolve() for a in sys.argv[1:3])
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
.desk-only img { width: 100%; border: 1px solid #b7c9b7; border-radius: 6px; margin: 8px 0 14px; }
code { background: #eef5ea; padding: 1px 4px; border-radius: 3px; }
pre { background: #eef5ea; padding: 8px 10px; border-radius: 6px; white-space: pre-wrap; font-size: 9.5pt; }
em { color: #555; }
.note { border-left: 4px solid #9ccc65; background: #f4f9f0; padding: 6px 12px; margin: 10px 0; }
"""


def expand(md, lang):
    def shot(m):
        n = m.group(1)
        d, p = SHOTS / f"{n}-{lang}-desktop.png", SHOTS / f"{n}-{lang}-phone.png"
        if not p.exists():
            return f'\n<div class="desk-only"><img src="{d.as_uri()}"></div>\n'
        return f'\n<div class="shot"><img class="desk" src="{d.as_uri()}"><img class="phone" src="{p.as_uri()}"></div>\n'

    def desk(m):
        return f'\n<div class="desk-only"><img src="{(SHOTS / f"{m.group(1)}-{lang}-desktop.png").as_uri()}"></div>\n'

    md = re.sub(r"^\s*@shot (\S+)\s*$", shot, md, flags=re.MULTILINE)
    return re.sub(r"^\s*@desk (\S+)\s*$", desk, md, flags=re.MULTILINE)


with sync_playwright() as p:
    browser = p.chromium.launch()
    for lang in ("en", "ko"):
        text = expand((HERE / f"guide-{lang}.md").read_text(encoding="utf-8"), lang)
        body = markdown.markdown(text, extensions=["extra"])
        title = html.escape(text.splitlines()[0].lstrip("# "))
        page_html = (f'<!doctype html><html lang="{lang}"><head><meta charset="utf-8"><title>{title}</title>'
                     f"<style>{CSS}</style></head><body>{body}</body></html>")
        src = OUT / f"guide-{lang}.html"
        src.write_text(page_html, encoding="utf-8")
        page = browser.new_page()
        page.goto(src.as_uri(), wait_until="load")
        name = "JunsGarden-Streamlit-guide-English.pdf" if lang == "en" else "JunsGarden-Streamlit-guide-Korean.pdf"
        page.pdf(path=str(OUT / name), format="A4", print_background=True,
                 margin={"top": "14mm", "bottom": "14mm", "left": "12mm", "right": "12mm"})
        page.close()
        print(f"PASS {name}")
    browser.close()
