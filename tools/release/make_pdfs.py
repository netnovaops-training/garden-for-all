"""Make the "Setup documentation" PDFs: every guide twice, a QUICK one (its
"Quick version" / "짧은 버전" only) and a FULL one, in English and Korean,
sorted into numbered step folders.

    python tools/release/make_pdfs.py [OUTPUT-FOLDER]

Default output: "Setup documentation" next to this package folder.
Every page gets a small lavender step label in its top right corner (English
or Korean). If the environment variable GARDEN_LOGO (or GARDEN_WATERMARK) names an image file,
that image is printed very faintly in the bottom right corner of every page
(it is read from that file only; nothing personal is kept in this repository). Needs Google
Chrome (or Microsoft Edge) to print the pages, and python-markdown:
    pip install markdown
Nothing here is needed to RUN the garden; it only makes the PDF copies.
"""
import html, os, re, shutil, subprocess, sys, tempfile, pathlib
import markdown

PKG = pathlib.Path(__file__).resolve().parents[2]
OUT = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else PKG.parent / "Setup documentation"

# The step folders, and which guides go in each (file names without .md).
STEPS = [
    ("0 - Start here", "0 - 여기서 시작", ["00-OVERVIEW", "00-READ-ME-FIRST", "00b-before-you-start", "23-full-setup", "02a-what-is-docker"]),
    ("1 - Ubuntu on the computer", "1 - 컴퓨터에 Ubuntu 설치", ["01-ubuntu-laptop"]),
    ("2 - Docker and the install", "2 - Docker와 설치", ["02-docker", "03-run-your-garden"]),
    ("3 - Going online", "3 - 인터넷에 올리기", ["04-go-public-no-ip-caddy", "04b-take-over-junsgarden-zapto", "07-sso-gate", "14-reserve-your-ip", "20-ssh-and-remote-desktop"]),
    ("4 - Email, keys and AI", "4 - 이메일, 키, AI", ["16-ai-studio", "05-gemini-prompts", "08-gemini-setup", "15-local-ai", "17-android-app-and-ai-coding"]),
    ("5 - Backups and your data", "5 - 백업과 내 데이터", ["13-backups-and-folders", "12-your-database"]),
    ("6 - Everyday use and help", "6 - 평소에 쓰기와 도움", ["19-every-menu-option", "25-common-questions", "26-rename-move-and-switch", "09-troubleshooting", "11-maintain-your-site", "10-the-plan", "18-review-the-code", "06-shopping-list"]),
    # The terminal way: harder than Garden Bridge, but not tough, with the author's walkthrough. English only.
    ("6b - The terminal way (bash), harder but not tough", "6b - 터미널로 하기(bash), 조금 어렵지만 힘들지 않아요", ["22-bash-walkthrough", "24-optional-tools"]),
]
REFERENCE = ("7 - Reference", "7 - 참고", [("HANDOVER-SUMMARY", "HANDOVER-SUMMARY"), ("WHAT-GETS-INSTALLED", "WHAT-GETS-INSTALLED.ko"), ("WHY-THESE-PROGRAMS", "WHY-THESE-PROGRAMS.ko"),
                                           ("JOURNAL", "JOURNAL.ko"), ("CHANGES", None)])
PROJECTS = ("8 - Projects (optional, for fun)", "8 - 프로젝트(선택, 재미로)")

# One soft colour per step (a gentle rainbow), the same in English and Korean:
# (heading colour, soft background, line colour).
STEP_COLOURS = {
    "0": ("#9c3d54", "#fbeef1", "#efc9d3"),   # rose
    "1": ("#9a4f1c", "#fdf1e6", "#f3d3b8"),   # peach
    "2": ("#7d6200", "#fdf8e1", "#eedf9f"),   # warm yellow
    "3": ("#2f6b3a", "#edf6ea", "#cfe3c8"),   # green
    "4": ("#136a6a", "#e8f6f5", "#bfe3e0"),   # teal
    "5": ("#2a5a9a", "#ebf2fb", "#c7d8f0"),   # blue
    "6": ("#45459a", "#efeffb", "#cfcff0"),   # indigo
    "6b": ("#3b4a5e", "#eef1f5", "#cdd6e1"),  # slate: the terminal way
    "7": ("#6a3f8f", "#f4eefa", "#ddcdee"),   # violet
    "8": ("#7a4a68", "#f7eff4", "#e6cfdd"),   # plum
}

CSS = """
@page { size: Letter; margin: 0.95in 0.75in 1.0in; }
body { font-family: "Segoe UI", "Malgun Gothic", "Noto Sans KR", Arial, sans-serif; font-size: 11.5pt; line-height: 1.55; color: #1f2a1f; }
h1 { color: var(--a); font-size: 21pt; margin: 0 0 6pt; }
h2 { color: var(--a); font-size: 15pt; border-bottom: 1.5px solid var(--l); padding-bottom: 2pt; margin-top: 18pt; }
h3 { color: var(--a); font-size: 12.5pt; margin-top: 14pt; }
code { font-family: Consolas, "Cascadia Mono", monospace; background: #f1f2f4; padding: 0 3px; border-radius: 3px; font-size: 10pt; }
pre { background: #f1f2f4; padding: 8pt 10pt; border-radius: 6px; white-space: pre-wrap; word-break: break-word; font-size: 9.5pt; }
pre code { background: none; padding: 0; }
table { border-collapse: collapse; width: 100%; margin: 8pt 0; font-size: 10pt; }
th, td { border: 1px solid var(--l); padding: 4pt 6pt; vertical-align: top; text-align: left; }
th { background: var(--s); }
tr { page-break-inside: avoid; }
blockquote { border-left: 4px solid #8a5a00; background: #fff4dc; margin: 8pt 0; padding: 6pt 10pt; }
img { max-width: 100%; }
a { color: var(--a); }
.quick { background: var(--s); border-left: 4px solid var(--a); border-radius: 6px; padding: 2pt 12pt 6pt; margin: 8pt 0 14pt; }
.quick h2 { border-bottom: 0; margin-top: 8pt; }
img { border-radius: 6px; max-height: 8.4in; width: auto; max-width: 100%; display: block; margin: 6pt auto; break-inside: avoid; page-break-inside: avoid; }
* { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
.stick { display: flex; align-items: center; gap: 8pt; margin-bottom: 8pt; }
.stick .dot { width: 26pt; height: 26pt; border-radius: 50%; background: var(--a); color: #fff; font-weight: 700; font-size: 13pt; display: flex; align-items: center; justify-content: center; box-shadow: 0 0 0 3pt var(--s); }
.stick .name { color: var(--a); font-size: 10pt; font-weight: 600; letter-spacing: .02em; }
.badge { display: inline-block; background: var(--a); color: #fff; border-radius: 4px; padding: 1pt 6pt; font-size: 9pt; margin-bottom: 6pt; }
.foot { color: #5b6b5b; font-size: 9pt; margin-top: 24pt; border-top: 1px solid var(--l); padding-top: 4pt; }

/* English: a calmer, more open page (easier on the eyes). Korean keeps its own spacing. */
html[lang=en] body { font-size: 12pt; line-height: 1.72; letter-spacing: .005em; color: #26302a; }
html[lang=en] p { margin: 0 0 10pt; }
html[lang=en] h1 { font-size: 24pt; font-weight: 600; letter-spacing: -.005em; margin: 2pt 0 12pt; }
html[lang=en] h2 { font-size: 16pt; font-weight: 600; margin: 28pt 0 10pt; padding-bottom: 4pt; }
html[lang=en] h3 { font-size: 13pt; font-weight: 600; margin: 20pt 0 6pt; }
html[lang=en] ul, html[lang=en] ol { padding-left: 20pt; margin: 4pt 0 12pt; }
html[lang=en] li { margin: 0 0 5pt; }
html[lang=en] li > ul, html[lang=en] li > ol { margin: 4pt 0 2pt; }
html[lang=en] table { font-size: 10.5pt; line-height: 1.5; margin: 10pt 0 14pt; }
html[lang=en] th, html[lang=en] td { padding: 6pt 9pt; }
html[lang=en] tbody tr:nth-child(even) td { background: #f8faf8; }
html[lang=en] pre { font-size: 10pt; line-height: 1.5; padding: 10pt 13pt; margin: 8pt 0 14pt; border-radius: 8px; }
html[lang=en] blockquote { border-radius: 0 8px 8px 0; padding: 8pt 14pt; margin: 12pt 0 14pt; }
html[lang=en] blockquote p:last-child { margin-bottom: 0; }
html[lang=en] .quick { padding: 4pt 16pt 10pt; margin: 10pt 0 20pt; }
html[lang=en] img { margin: 10pt auto 14pt; max-height: 7.2in; }
h1, h2, h3 { line-height: 1.25; break-after: avoid; page-break-after: avoid; }
/* A heading never ends a page alone: it travels with what follows it. */
h2 + p, h3 + p, h2 + img, h3 + img, h3 + p:has(img), h2 + p:has(img) { break-before: avoid; page-break-before: avoid; }
"""


def find_browser():
    for p in (r"C:\Program Files\Google\Chrome\Application\chrome.exe",
              r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
              r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
              "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"):
        if os.path.exists(p):
            return p
    sys.exit("Google Chrome or Microsoft Edge is needed to print the PDFs.")


def quick_part(text):
    """Only the title and the "Quick version" (or "짧은 버전") section."""
    title = text.split("\n", 1)[0]
    m = re.search(r"^## (Quick version|짧은 버전)\n(.*?)(?=^---\s*$|^## )", text, re.S | re.M)
    return f"{title}\n\n{m.group(0)}" if m else None


# GARDEN_LOGO (the emblem image, also used by make_handbook.py) or GARDEN_WATERMARK (this script only): a path, never tracked in git.
WATERMARK = os.environ.get("GARDEN_WATERMARK") or os.environ.get("GARDEN_LOGO", "")


def faint_image_uri(path, opacity=0.16):
    """The watermark image, made faint: wrapped in a small SVG at low opacity
    (works for any picture type), as a data URI."""
    import base64, mimetypes
    mime = mimetypes.guess_type(path)[0] or "image/png"
    data = base64.b64encode(open(path, "rb").read()).decode()
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 100 100">'
           f'<image width="100" height="100" preserveAspectRatio="xMaxYMax meet" opacity="{opacity}" '
           f'xlink:href="data:{mime};base64,{data}"/></svg>')
    return "data:image/svg+xml;base64," + base64.b64encode(svg.encode()).decode()


def margin_boxes(step):
    """Printed in the page margins of EVERY page, never over the text: the
    lavender step label top right, the faint watermark bottom right."""
    css = ("@page { @top-right { content: \"" + step.replace('"', "'") + "\"; color: #7a63b5; "
           "font-family: 'Segoe UI', 'Malgun Gothic', sans-serif; font-size: 8.5pt; letter-spacing: .02em; "
           "vertical-align: bottom; padding-bottom: 3pt; border-bottom: 1.5pt solid #d9cdf2; margin-bottom: 14pt; } ")
    if WATERMARK and os.path.exists(WATERMARK):
        css += ("@bottom-right { content: ''; width: 0.9in; height: 0.85in; margin-top: 0.08in; "
                f"background: url('{faint_image_uri(WATERMARK)}') no-repeat right bottom / contain; }} ")
    return css + "}"


def to_pdf(md_text, src_path, pdf_path, label, browser, tmp, step=""):
    base = src_path.parent
    body = markdown.markdown(md_text, extensions=["tables", "fenced_code", "sane_lists"])
    # The Quick version sits in a soft box of the step's colour.
    body = re.sub(r'(<h2>(?:Quick version|짧은 버전)</h2>.*?)(<hr\s*/?>|$)', r'<div class="quick">\1</div>\2', body, count=1, flags=re.S)
    a, soft, line = STEP_COLOURS.get(step.split("단계")[0].replace("Step ", "").split(" ")[0].strip(), STEP_COLOURS["3"])
    if "단계" in step and step.split("단계")[0].strip() == "6b":
        a, soft, line = STEP_COLOURS["6b"]
    colours = f":root {{ --a: {a}; --s: {soft}; --l: {line}; }}"
    # The sticker: the step's number in a soft round mark, and its name.
    sticker_num, _, sticker_name = step.partition(" · ")
    sticker_num = sticker_num.replace("Step ", "").replace("단계", "").strip()
    # Pictures: point at the real files, so they show in the PDF.
    body = re.sub(r'src="(?!https?:|file:)([^"]+)"', lambda m: f'src="{(base / m.group(1)).resolve().as_uri()}"', body)
    lang = "ko" if "/ko/" in src_path.as_posix() or src_path.name.endswith(".ko.md") else "en"
    page = (f'<!doctype html><html lang="{lang}"><head><meta charset="utf-8"><title>{html.escape(src_path.stem)}</title>'
            f'<style>{colours}{CSS}{margin_boxes(step)}</style></head><body>'
            + f'<div class="stick"><span class="dot">{html.escape(sticker_num)}</span><span class="name">{html.escape(sticker_name)}</span>'
            + f'<span class="badge" style="margin:0 0 0 auto">{label}</span></div>{body}'
            f'<div class="foot">Jun\'s Garden &middot; {html.escape(src_path.relative_to(PKG).as_posix())}</div></body></html>')
    page_file = tmp / (re.sub(r"[^A-Za-z0-9._-]", "_", str(pdf_path.name)) + ".html")
    page_file.write_text(page, encoding="utf-8")
    pdf_path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run([browser, "--headless=new", "--disable-gpu", "--no-pdf-header-footer", "--virtual-time-budget=4000",
                    f"--print-to-pdf={pdf_path}", page_file.as_uri()], check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def pretty(stem):
    return re.sub(r"^(\d+[a-z]?)-", r"\1 ", stem).replace("-", " ")


def main():
    browser = find_browser()
    if OUT.exists():
        shutil.rmtree(OUT)
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="garden-pdf-"))
    made = 0
    jobs = []
    def labels(folder_en, folder_ko):   # "3 - Going online" -> "Step 3 · Going online" / "3단계 · 인터넷에 올리기"
        num, en = folder_en.split(" - ", 1)
        return f"Step {num} · {en}", f"{num}단계 · {folder_ko.split(' - ', 1)[1]}"
    for en_folder, ko_folder, names in STEPS:
        en_lab, ko_lab = labels(en_folder, ko_folder)
        for n in names:
            jobs.append((PKG / "START-HERE" / f"{n}.md", OUT / "English" / en_folder, "en", en_lab))
            jobs.append((PKG / "START-HERE" / "ko" / f"{n}.md", OUT / "Korean" / ko_folder, "ko", ko_lab))
    en_lab, ko_lab = labels(REFERENCE[0], REFERENCE[1])
    for en_name, ko_name in REFERENCE[2]:
        jobs.append((PKG / f"{en_name}.md", OUT / "English" / REFERENCE[0], "en", en_lab))
        if ko_name:
            jobs.append((PKG / f"{ko_name}.md", OUT / "Korean" / REFERENCE[1], "ko", ko_lab))
    en_lab, _ = labels(PROJECTS[0], PROJECTS[1])
    for sub in ("projects", "n8n"):
        for f in sorted((PKG / "START-HERE" / sub).glob("*.md")):
            jobs.append((f, OUT / "English" / PROJECTS[0] / sub, "en", en_lab))
    only = os.environ.get("GARDEN_PDF_LANG", "")   # "en" or "ko": only that language
    if only:
        jobs = [j for j in jobs if j[2] == only]
    for src, folder, lang, step in jobs:
        if not src.exists():
            continue
        text = src.read_text(encoding="utf-8")
        name = pretty(src.stem.replace(".ko", ""))
        quick = quick_part(text)
        if quick:
            to_pdf(quick, src, folder / f"QUICK - {name}.pdf", "QUICK" if lang == "en" else "짧은 버전", browser, tmp, step)
            made += 1
        to_pdf(text, src, folder / f"FULL - {name}.pdf", "FULL" if lang == "en" else "전체", browser, tmp, step)
        made += 1
    shutil.rmtree(tmp, ignore_errors=True)
    print(f"PDFs made: {made}  in  {OUT}")


if __name__ == "__main__":
    main()
