#!/usr/bin/env python3
"""guide-en.md / guide-ko.md -> HTML -> PDF (headless Chromium), so the manual can be made
again on every release.  python build.py SHOTS_DIR APP_SHOTS_DIR OUT_DIR [MENU_DIR]

  @shot NAME   the step's picture at desktop width, with the phone picture beside it
  @app NAME    a phone screenshot of the app (from the emulator test)
  @menu NAME   a picture of the garden computer's menu (START-HERE/images/NAME.png, or MENU_DIR)
  [TOC]        the table of contents (every ## and ### heading, as links)

The PDF gets page numbers at the bottom and bookmarks (the reader's side panel) for the headings.
"""
import html
import pathlib
import re
import sys

import markdown
from playwright.sync_api import sync_playwright

HERE = pathlib.Path(__file__).resolve().parent
SHOTS, APPS, OUT = (pathlib.Path(a).resolve() for a in sys.argv[1:4])
MENUS = pathlib.Path(sys.argv[4]).resolve() if len(sys.argv) > 4 else HERE.parent.parent / "START-HERE" / "images"
OUT.mkdir(parents=True, exist_ok=True)

CSS = """
@page { size: A4; margin: 14mm 12mm; }
body { font-family: 'Noto Sans', 'Noto Sans CJK KR', sans-serif; font-size: 11.5pt; line-height: 1.5; color: #1d2b1d; }
h1 { color: #2e6b2e; border-bottom: 3px solid #9ccc65; padding-bottom: 4px; }
h2 { color: #2e6b2e; margin-top: 26px; page-break-after: avoid; }
h3 { color: #3d7a3d; margin-top: 18px; page-break-after: avoid; }
h2.part { page-break-before: always; border-bottom: 2px solid #9ccc65; }
li { margin: 6px 0; }
.shot { display: flex; gap: 10px; align-items: flex-start; margin: 8px 0 14px; page-break-inside: avoid; }
.shot img { border: 1px solid #b7c9b7; border-radius: 6px; }
.shot .desk { width: 74%; } .shot .phone { width: 22%; }
.app, .menu { display: block; }
.app img { width: 38%; border: 1px solid #b7c9b7; border-radius: 10px; margin: 8px 0 14px; }
.menu img { width: 80%; margin: 8px 0 14px; page-break-inside: avoid; }
code { background: #eef5ea; padding: 1px 4px; border-radius: 3px; }
em { color: #555; }
table { border-collapse: collapse; width: 100%; margin: 8px 0 14px; font-size: 10pt; page-break-inside: auto; }
th, td { border: 1px solid #c8dcc8; padding: 4px 6px; text-align: left; vertical-align: top; }
th { background: #eef5ea; }
tr { page-break-inside: avoid; }
blockquote { margin: 10px 0; padding: 8px 12px; background: #fff8e1; border-left: 5px solid #f0a020; border-radius: 4px; }
.toc { page-break-after: always; }
.toc ul { list-style: none; padding-left: 14px; margin: 2px 0; }
.toc > ul { padding-left: 0; }
.toc li { margin: 2px 0; }
.toc a { color: #1d2b1d; text-decoration: none; }
.toc > ul > li > a { font-weight: 700; color: #2e6b2e; }
"""

FOOTER = ('<div style="font-size:8pt; color:#777; width:100%; text-align:center;">'
          '<span class="pageNumber"></span> / <span class="totalPages"></span></div>')


def expand(md):
    # Each picture stays on its own (indented) line, as part of the step above it, so a numbered
    # list keeps counting 1, 2, 3 across its pictures (a block with blank lines restarted it at 1).
    def shot(m):
        d, p = SHOTS / f"{m.group(2)}-desktop.png", SHOTS / f"{m.group(2)}-phone.png"
        return (f'{m.group(1)}<span class="shot"><img class="desk" src="{d.as_uri()}">'
                f'<img class="phone" src="{p.as_uri()}"></span>')

    def app(m):
        return f'{m.group(1)}<span class="app"><img src="{(APPS / (m.group(2) + ".png")).as_uri()}"></span>'

    def menu(m):
        return f'{m.group(1)}<span class="menu"><img src="{(MENUS / (m.group(2) + ".png")).as_uri()}"></span>'

    md = re.sub(r"^([ \t]*)@shot (\S+)[ \t]*$", shot, md, flags=re.MULTILINE)
    md = re.sub(r"^([ \t]*)@menu (\S+)[ \t]*$", menu, md, flags=re.MULTILINE)
    return re.sub(r"^([ \t]*)@app (\S+)[ \t]*$", app, md, flags=re.MULTILINE)


with sync_playwright() as p:
    browser = p.chromium.launch()
    for lang in ("en", "ko"):
        text = expand((HERE / f"guide-{lang}.md").read_text(encoding="utf-8"))
        conv = markdown.Markdown(extensions=["extra", "toc"],
                                 extension_configs={"toc": {"toc_depth": "2-3", "title": ""}})
        body = conv.convert(text)
        # A heading written "## Title {.part}" starts on a new page.
        title = html.escape(text.splitlines()[0].lstrip("# "))
        page_html = (f'<!doctype html><html lang="{lang}"><head><meta charset="utf-8"><title>{title}</title>'
                     f"<style>{CSS}</style></head><body>{body}</body></html>")
        src = OUT / f"guide-{lang}.html"
        src.write_text(page_html, encoding="utf-8")
        page = browser.new_page()
        page.goto(src.as_uri(), wait_until="load")
        name = "JunsGarden-guide-English.pdf" if lang == "en" else "JunsGarden-guide-Korean.pdf"
        page.pdf(path=str(OUT / name), format="A4", print_background=True, outline=True, tagged=True,
                 display_header_footer=True, header_template="<div></div>", footer_template=FOOTER,
                 margin={"top": "14mm", "bottom": "16mm", "left": "12mm", "right": "12mm"})
        page.close()
        print(f"PASS {name}")
    browser.close()
