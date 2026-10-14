"""Make the garden handbook: a welcome page and one web page per guide.

    python tools/release/make_handbook.py [TARGET_FOLDER]

TARGET_FOLDER is the garden folder to write into (the release build passes its
COPY of the package; the default is this package). It writes:

  00-WELCOME.html          every step, every guide, one click away
  handbook/<guide>.html    each guide as a web page: steps on the left, the
                           guide in a readable column, "On this page" on the
                           right, the Quick and Full PDFs as buttons at the top
  handbook/assets/         the style sheet, and the logo (GARDEN_LOGO, if set)

Everything works from the disk (file://): no server, no internet. The PDF
buttons open "Setup documentation/English/...", which the release build puts
in the download. The logo is Jason's emblem: it is passed in at release time
(GARDEN_LOGO=path) and is not kept in git.
"""
import html, os, pathlib, re, shutil, sys, urllib.parse
import markdown
import make_pdfs as m

PKG = m.PKG
ROOT = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else PKG
SRC = PKG / "START-HERE"
HB = ROOT / "handbook"
LOGO = os.environ.get("GARDEN_LOGO", "")
STEP_NAMES = {}            # guide stem -> (step folder, number, label)
ORDER = []                 # every guide stem, in reading order
for folder, _ko, names in m.STEPS:
    num, label = folder.split(" - ", 1)
    for n in names:
        if (SRC / f"{n}.md").exists():
            STEP_NAMES[n] = (folder, num, label)
            ORDER.append(n)

CSS = r"""
:root {
  --ink:#1f3b2d; --paper:#f2f6f1; --sheet:#ffffff; --moss:#3e7a57; --lav:#6b58b0; --petal:#ece6f7;
  --rule:#d3ddd3; --muted:#5d7065; --code:#eef2ee;
  --read:"Sitka Text","Iowan Old Style",Georgia,"Noto Serif","Malgun Gothic",serif;
  --head:"Sitka Heading","Sitka Text",Georgia,"Noto Serif","Malgun Gothic",serif;
  --ui:"Segoe UI Variable Text","Segoe UI",Ubuntu,"Noto Sans","Malgun Gothic",sans-serif;
  --mono:"Cascadia Mono",Consolas,"Ubuntu Mono","DejaVu Sans Mono",monospace;
}
@media (prefers-color-scheme: dark) {
  :root { --ink:#e3ebe5; --paper:#141c17; --sheet:#1b251f; --moss:#8fc9a5; --lav:#b9a8ef; --petal:#2a2540;
          --rule:#2f3d34; --muted:#9fb1a5; --code:#202b24; }
}
* { box-sizing:border-box; }
html { scroll-behavior:smooth; }
@media (prefers-reduced-motion: reduce) { html { scroll-behavior:auto; } *, *::before { animation:none !important; transition:none !important; } }
body { margin:0; background:var(--paper); color:var(--ink); font:18px/1.7 var(--read); }
a { color:var(--moss); text-underline-offset:3px; }
a:focus-visible, summary:focus-visible { outline:3px solid var(--lav); outline-offset:3px; border-radius:4px; }
img { max-width:100%; height:auto; }

/* Top bar: the emblem and the handbook's name */
.top { position:sticky; top:0; z-index:5; display:flex; align-items:center; gap:14px; padding:10px 24px;
       background:color-mix(in srgb, var(--paper) 88%, transparent); backdrop-filter:blur(8px); border-bottom:1px solid var(--rule); }
.brand { display:flex; align-items:center; gap:12px; text-decoration:none; color:var(--ink); }
.brand img { width:74px; height:auto; }
.brand b { font:600 20px/1.1 var(--head); display:block; }
.brand span { font:13px var(--ui); color:var(--muted); }
.top .home { margin-left:auto; font:14px var(--ui); }

/* ---------- the welcome page ---------- */
.hero { max-width:1060px; margin:0 auto; padding:56px 24px 18px; display:grid; grid-template-columns:minmax(0,1fr) 300px; gap:32px; align-items:center; }
.hero h1 { font:600 clamp(36px,5vw,58px)/1.05 var(--head); letter-spacing:-.01em; margin:0 0 18px; }
.hero p { font-size:20px; max-width:34em; margin:0 0 12px; }
.hero .quiet { font:15px/1.6 var(--ui); color:var(--muted); }
.hero .mark { width:100%; opacity:.95; }
.begin { display:inline-flex; gap:10px; flex-wrap:wrap; margin-top:14px; }
.btn { display:inline-block; font:600 15px var(--ui); text-decoration:none; padding:10px 18px; border-radius:999px;
       border:1.5px solid var(--lav); color:var(--lav); background:transparent; }
.btn.solid { background:var(--lav); color:#fff; }
.btn:hover { filter:brightness(1.08); }

.path { max-width:1060px; margin:0 auto; padding:10px 24px 70px; position:relative; }
.path::before { content:""; position:absolute; left:47px; top:84px; bottom:80px; width:3px; border-radius:3px;
                background:linear-gradient(var(--moss), var(--lav)); transform-origin:top; animation:grow 1.4s ease-out both; }
@keyframes grow { from { transform:scaleY(0); } to { transform:scaleY(1); } }
.step { position:relative; padding:0 0 26px 76px; }
.leaf { position:absolute; left:28px; top:4px; width:42px; height:42px; border-radius:0 60% 0 60%; transform:rotate(-12deg);
        background:var(--a); color:#fff; display:grid; place-items:center; font:700 15px var(--ui); box-shadow:0 0 0 6px var(--paper); }
.leaf span { transform:rotate(12deg); }
.step h2 { font:600 26px/1.25 var(--head); color:var(--a); margin:6px 0 4px; }
@media (prefers-color-scheme: dark) { .step h2 { color:color-mix(in srgb, var(--a) 55%, white); } }
.step > p { font:15px var(--ui); color:var(--muted); margin:0 0 10px; }
.guides { list-style:none; margin:0; padding:0; border-top:1px solid var(--rule); max-width:860px; }
.guides li { display:grid; grid-template-columns:minmax(0,1fr) auto; gap:6px 18px; align-items:baseline; padding:12px 2px; border-bottom:1px solid var(--rule); }
.guides a.title { font:19px/1.35 var(--read); color:var(--ink); text-decoration:none; }
.guides a.title:hover { color:var(--moss); text-decoration:underline; }
.pdf { font:13px var(--ui); white-space:nowrap; }
.pdf a { color:var(--muted); margin-left:12px; }
.pdf a:hover { color:var(--lav); }
.foot { max-width:1060px; margin:0 auto; padding:0 24px 40px; font:14px var(--ui); color:var(--lav); }

/* Two ways, and what the install does */
.ways, .does { max-width:1060px; margin:0 auto; padding:26px 24px 10px; }
.ways h2, .does h2, .pathhead { font:600 30px/1.2 var(--head); margin:0 0 14px; }
.pathhead { margin:0 0 22px; }
.two { display:grid; grid-template-columns:1fr 1fr; gap:22px; }
.calm { font-size:18px; max-width:44em; margin:0 0 20px; }
.three { display:grid; grid-template-columns:repeat(3, minmax(0,1fr)); gap:20px; align-items:start; }
.lvl { background:var(--sheet); border:1px solid var(--rule); border-radius:16px; padding:20px 22px 22px; --c:var(--moss); }
.lvl.wine { --c:var(--lav); border:2px solid var(--lav); }
.lvl.champagne { --c:#a07a2c; }
@media (prefers-color-scheme: dark) { .lvl.champagne { --c:#e2c27a; } }
.glass svg { width:44px; height:44px; fill:none; stroke:var(--c); stroke-width:1.8; stroke-linecap:round; stroke-linejoin:round; }
.lvl h3 { font:600 28px/1.15 var(--head); margin:6px 0 10px; }
.lvl h3 span { font:400 18px var(--read); color:var(--muted); }
.meter { display:flex; flex-direction:column; gap:6px; margin:0 0 10px; }
.dots { display:flex; gap:5px; }
.dots i { width:14px; height:14px; border-radius:50%; border:1.5px solid var(--c); }
.dots i.on { background:var(--c); }
.meter .lbl { font:14px var(--ui); color:var(--muted); }
.lvl .time { font:15px var(--ui); margin:0 0 8px; }
.lvl ul { font:15.5px/1.5 var(--ui); margin:0 0 16px; padding-left:18px; }
.lvl li { margin:0 0 5px; }
.lvl li:first-child { list-style:none; margin-left:-18px; font-weight:600; }
.lvl.beer li:first-child { list-style:disc; margin-left:0; font-weight:400; }
.way { background:var(--sheet); border:1px solid var(--rule); border-radius:16px; padding:20px 24px 22px; }
.way.easy { border-top:5px solid var(--lav); }
.way.full { border-top:5px solid var(--moss); }
.way h3 { font:600 26px/1.2 var(--head); margin:0; }
.way .rec { font:14px var(--ui); color:var(--muted); margin:2px 0 10px; }
.way p { font-size:17px; margin:0 0 12px; }
.does .sub { font:16px/1.6 var(--ui); color:var(--muted); max-width:46em; margin:0 0 16px; }
.install { counter-reset:s; list-style:none; margin:0 0 26px; padding:0; columns:2 380px; column-gap:36px; font:16px/1.4 var(--ui); }
.install li { counter-increment:s; break-inside:avoid; display:flex; gap:12px; align-items:baseline; padding:9px 0; border-bottom:1px solid var(--rule); }
.install li::before { content:counter(s); flex:none; width:28px; height:28px; border-radius:50%; background:var(--petal); color:var(--lav);
                      font:700 13px var(--ui); display:grid; place-items:center; }
.install span { flex:1; }
.install small { color:var(--muted); font-size:13px; white-space:nowrap; }
.plan { display:grid; grid-template-columns:repeat(3, minmax(0,1fr)); gap:22px; font:15px/1.5 var(--ui); }
.plan h3 { font:600 18px var(--head); margin:0 0 8px; color:var(--moss); }
.plan ul { margin:0; padding-left:18px; }
.plan li { margin:0 0 6px; }
.never { font:15px/1.6 var(--ui); color:var(--muted); background:var(--petal); border-radius:12px; padding:12px 16px; margin:20px 0 0; }

/* The three views of a guide: In brief, 한국어, Full */
.views { display:inline-flex; gap:4px; padding:4px; margin:4px 0 22px; background:var(--sheet); border:1px solid var(--rule); border-radius:999px; }
.views button { font:600 15px var(--ui); color:var(--muted); background:transparent; border:0; border-radius:999px; padding:8px 18px; cursor:pointer; }
.views button[aria-selected="true"] { background:var(--a); color:#fff; }
.views button:focus-visible { outline:3px solid var(--lav); outline-offset:2px; }
.view.brief { font-size:19px; }
.view.brief ul { padding-left:22px; }
.view.brief li { margin:0 0 10px; }
.view .more { margin:22px 0 0; }
body[data-view="brief"] .toc, body[data-view="ko"] .toc { visibility:hidden; }
.read .pdfs { margin:40px 0 0; }

/* The tree: step > guide > topic (left column and the welcome page's map) */
.tree details { margin:0; }
.tree summary { list-style:none; cursor:pointer; display:flex; gap:6px; align-items:baseline; padding:4px 8px 4px 0; }
.tree summary::-webkit-details-marker { display:none; }
.tree summary::before { content:""; flex:none; width:7px; height:7px; margin:0 4px 1px 2px; border-right:1.5px solid var(--muted);
                        border-bottom:1.5px solid var(--muted); transform:rotate(-45deg); transition:transform .15s; }
.tree details[open] > summary::before { transform:rotate(45deg); }
.tree summary a { color:inherit; text-decoration:none; }
.tree summary a:hover { color:var(--moss); text-decoration:underline; }
.tree ul { list-style:none; margin:2px 0 8px 9px; padding:0 0 0 12px; border-left:1px solid var(--rule); }
.tree ul a { display:block; padding:3px 0; color:var(--muted); text-decoration:none; font-size:.94em; }
.tree ul a:hover { color:var(--moss); }
.map { max-width:1060px; margin:0 auto; padding:10px 24px 40px; }
.map h2 { font:600 30px/1.2 var(--head); margin:0 0 6px; }
.map > p { font:15px/1.6 var(--ui); color:var(--muted); margin:0 0 18px; max-width:44em; }
.map .cols { columns:2 420px; column-gap:40px; font:16px/1.45 var(--ui); }
.map .stepblock { break-inside:avoid; margin:0 0 18px; }
.map h3 { font:600 17px var(--ui); color:var(--a); margin:0 0 4px; display:flex; gap:8px; align-items:center; }
.map h3 i { width:12px; height:12px; border-radius:0 60% 0 60%; background:var(--a); display:inline-block; }
.map summary { color:var(--ink); }

/* ---------- a guide page ---------- */
.shell { display:grid; grid-template-columns:270px minmax(0,1fr) 220px; gap:40px; max-width:1360px; margin:0 auto; padding:28px 24px 60px; }
.steps { position:sticky; top:96px; align-self:start; max-height:calc(100vh - 110px); overflow:auto; font:14px/1.45 var(--ui); padding-right:6px; }
.steps h3 { font:600 14px var(--ui); margin:16px 0 6px; display:flex; gap:8px; align-items:center; color:var(--ink); }
.steps h3 i { width:12px; height:12px; border-radius:0 60% 0 60%; background:var(--a); display:inline-block; }
.steps > a, .steps .g > summary { color:var(--muted); }
.steps .g[open] > summary { color:var(--ink); font-weight:600; }
.steps a:hover { color:var(--ink); }
.steps .g.here > summary { background:var(--petal); border-radius:6px; color:var(--ink); }
.read { min-width:0; max-width:46em; }
.read h1 { font:600 clamp(30px,4vw,44px)/1.12 var(--head); margin:4px 0 14px; color:var(--ink); }
.where { font:14px var(--ui); color:var(--a); display:flex; align-items:center; gap:8px; margin:0; }
.where i { width:14px; height:14px; border-radius:0 60% 0 60%; background:var(--a); display:inline-block; }
.pdfs { display:flex; gap:10px; flex-wrap:wrap; margin:0 0 26px; }
.read h2 { font:600 27px/1.25 var(--head); margin:46px 0 12px; padding-top:6px; border-top:1px solid var(--rule); scroll-margin-top:90px; }
.read h3 { font:600 21px/1.3 var(--head); margin:32px 0 8px; scroll-margin-top:90px; }
.read p, .read li { max-width:40em; }
.read li { margin:0 0 6px; }
.read code { font:15px var(--mono); background:var(--code); padding:1px 5px; border-radius:4px; }
.read pre { background:var(--code); border:1px solid var(--rule); border-radius:10px; padding:14px 16px; overflow:auto; font:15px/1.55 var(--mono); }
.read pre code { background:none; padding:0; }
.read table { border-collapse:collapse; width:100%; margin:14px 0 22px; font:15px/1.5 var(--ui); display:block; overflow-x:auto; }
.read th, .read td { border-bottom:1px solid var(--rule); padding:9px 12px; text-align:left; vertical-align:top; }
.read th { color:var(--muted); font-weight:600; border-bottom-width:2px; }
.read blockquote { margin:20px 0; padding:12px 18px; background:var(--petal); border-left:4px solid var(--lav); border-radius:0 10px 10px 0; }
.read img { display:block; margin:18px 0 8px; border-radius:10px; border:1px solid var(--rule); }
.quick { background:var(--sheet); border:1px solid var(--rule); border-left:5px solid var(--a); border-radius:0 12px 12px 0; padding:4px 22px 10px; margin:0 0 30px; }
.quick h2 { border:0; margin:14px 0 4px; font-size:22px; }
.pager { display:flex; justify-content:space-between; gap:16px; margin-top:56px; padding-top:18px; border-top:1px solid var(--rule); font:15px var(--ui); }
.pager a { text-decoration:none; max-width:48%; }
.pager small { display:block; color:var(--muted); }
.toc { position:sticky; top:96px; align-self:start; font:13.5px/1.45 var(--ui); }
.toc b { display:block; margin:0 0 8px; color:var(--muted); font-weight:600; }
.toc a { display:block; color:var(--muted); text-decoration:none; padding:3px 0; }
.toc a:hover { color:var(--moss); }
.allguides { display:none; }

@media (max-width:1180px) { .shell { grid-template-columns:240px minmax(0,1fr); } .toc { display:none; } .three { grid-template-columns:1fr; max-width:560px; } }
@media (max-width:820px) {
  body { font-size:17px; }
  .hero { grid-template-columns:1fr; padding-top:30px; }
  .hero .mark { max-width:220px; order:-1; }
  .shell { grid-template-columns:1fr; padding-top:16px; }
  .steps { display:none; }
  .allguides { display:block; margin:0 0 18px; font:15px var(--ui); }
  .allguides summary { cursor:pointer; color:var(--moss); }
  .guides li { grid-template-columns:1fr; }
  .two, .plan, .three { grid-template-columns:1fr; }
  .pdf a { margin:0 12px 0 0; }
  .path::before { left:35px; }
  .step { padding-left:62px; }
  .leaf { left:16px; }
  .top { padding:8px 16px; }
  .brand img { width:46px; }
}
@media print { .top, .steps, .toc, .pager, .allguides { display:none; } .shell { display:block; } }
"""


def colours(stem_or_num):
    num = STEP_NAMES[stem_or_num][1] if stem_or_num in STEP_NAMES else stem_or_num
    return m.STEP_COLOURS.get(num, m.STEP_COLOURS["3"])


def title_of(text):
    first = text.split("\n", 1)[0].lstrip("# ").strip()
    return re.sub(r"^\d+[a-z]?\.\s+", "", first)


def pdf_href(stem, kind, prefix):
    folder = STEP_NAMES[stem][0]
    return prefix + urllib.parse.quote(f"Setup documentation/English/{folder}/{kind} - {m.pretty(stem)}.pdf")


def fix_links(body, here_dir, base=None):
    """Links and pictures written for START-HERE/*.md (or ko/*.md), made right for handbook/*.html."""
    base = base or SRC
    def one(mo):
        attr, url = mo.group(1), mo.group(2)
        if re.match(r"^(https?:|mailto:|#|data:)", url):
            return mo.group(0)
        path, _, frag = url.partition("#")
        target = (base / urllib.parse.unquote(path)).resolve()
        if target.suffix == ".md" and target.parent in (SRC, SRC / "ko") and target.stem in STEP_NAMES:
            new = f"{target.stem}.html" + (f"#{frag}" if frag else "")
        else:
            new = urllib.parse.quote(os.path.relpath(target, here_dir).replace(os.sep, "/")) + (f"#{frag}" if frag else "")
        return f'{attr}="{new}"'
    return re.sub(r'(href|src)="([^"]+)"', one, body)


def page(title, body, logo, home):
    logo_tag = f'<img src="{logo}" alt="">' if logo else ""
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>{html.escape(title)}</title><link rel="stylesheet" href="{home}handbook/assets/handbook.css"></head>
<body><header class="top"><a class="brand" href="{home}00-WELCOME.html">{logo_tag}<div><b>Jun's Garden</b><span>Your handbook</span></div></a>
<a class="home" href="{home}00-WELCOME.html">All steps</a></header>
{body}</body></html>
"""


def step_nav(current):
    out = []
    for folder, _ko, names in m.STEPS:
        num, label = folder.split(" - ", 1)
        a = m.STEP_COLOURS.get(num, m.STEP_COLOURS["3"])[0]
        links = "".join(guide_tree(n, f"{n}.html", is_open=(n == current), here=(n == current))
                        for n in names if n in STEP_NAMES)
        if links:
            out.append(f'<h3 style="--a:{a}"><i></i>{html.escape(num)}. {html.escape(label)}</h3>{links}')
    return "".join(out)


TITLES = {n: title_of((SRC / f"{n}.md").read_text(encoding="utf-8")) for n in ORDER}


def topics_of(n):
    text = (SRC / f"{n}.md").read_text(encoding="utf-8")
    md = markdown.Markdown(extensions=["tables", "fenced_code", "sane_lists", "toc"])
    md.convert(text.split("\n", 1)[1] if text.startswith("# ") else text)
    out = []
    for t in md.toc_tokens:
        for h in ([t] if t["level"] == 2 else t.get("children", [])):
            if h["level"] == 2 and h["name"] != "Quick version":
                out.append((h["id"], html.unescape(h["name"])))
    return out


TOPICS = {n: topics_of(n) for n in ORDER}

# ---- the install, in its own words (read from garden.sh, so the page cannot drift) ----
GSH = (PKG / "garden.sh").read_text(encoding="utf-8")


def heredoc_after(marker):
    start = GSH.index(marker)
    body = GSH[GSH.index("<<'EOF'\n", start) + 8:]
    return body[:body.index("\nEOF")]


def mode_text(block, head):
    lines, on = [], False
    for l in block.split("\n"):
        if l.strip().startswith(head):
            on = True
            continue
        if on:
            if not l.strip() or l.strip()[:2] in ("2)", "0)"):
                break
            lines.append(l.strip())
    return " ".join(lines)


_modes = heredoc_after("choose_mode() {")
EASY_TEXT = mode_text(_modes, "1) EASY")
CUSTOM_TEXT = mode_text(_modes, "2) CUSTOM")
_steps_block = GSH[GSH.index("INSTALL_STEPS=("):]
_steps_block = _steps_block[:_steps_block.index("\n)")]
INSTALL = re.findall(r'"(\w+)\|([^"]+)"', _steps_block)
_times_block = GSH[GSH.index("declare -A STEP_TIME=("):]
STEP_TIME = dict(re.findall(r'\[(\w+)\]="([^"]+)"', _times_block[:_times_block.index("\n)")]))
PLAN = {}
_cur = None
for l in heredoc_after("show_plan() {").split("\n"):
    t = l.strip()
    if not t:
        _cur = None
        continue
    for key in ("NECESSARY", "OPTIONAL", "ADVANCED OPTIONAL", "NOT DONE, EVER:", "WHAT COMES FROM JASON:"):
        if t.startswith(key) and (key.endswith(":") or t.startswith(key + " (")):
            _cur = key
            PLAN[key] = [t[len(key):].strip()] if key.endswith(":") else []
            break
    else:
        if _cur and t.startswith("- "):
            PLAN[_cur].append(t[2:])
        elif _cur and PLAN.get(_cur):
            PLAN[_cur][-1] += " " + t


GLASS = {  # small line drawings, in the emblem's lavender line style
    "beer": '<svg viewBox="0 0 48 48" aria-hidden="true"><path d="M12 12h20v28a3 3 0 0 1-3 3H15a3 3 0 0 1-3-3z"/><path d="M32 18h5a4 4 0 0 1 4 4v8a4 4 0 0 1-4 4h-5"/><path d="M12 12c0-4 4-6 7-4 2-3 7-3 9 0 3-1 5 1 4 4"/><path d="M18 20v16M24 20v16"/></svg>',
    "wine": '<svg viewBox="0 0 48 48" aria-hidden="true"><path d="M15 5h18c1 9-1 17-9 19-8-2-10-10-9-19z"/><path d="M16 13h16"/><path d="M24 24v15M16 43h16"/></svg>',
    "champagne": '<svg viewBox="0 0 48 48" aria-hidden="true"><path d="M19 4h10l-1 18a3 3 0 0 1-8 0z"/><path d="M24 25v14M17 43h14"/><circle cx="23" cy="11" r="1"/><circle cx="25.5" cy="16" r="1"/><circle cx="22.5" cy="19" r="0.8"/><path d="M33 6l3-2M35 11h4M13 6l-3-2"/></svg>',
}
LEVELS = [  # (key, name, glass word, risk out of 10, what you get, button text, link)
    ("beer", "Easy", "beer", 3,
     ["The EASY install: the program chooses for you", "Your garden at home: website, garden tools, plants and photos",
      "Your password, and your free AI key"],
     "Start the easy way", None),
    ("wine", "Recommended", "wine", 5,
     ["Easy, plus what is in your best interest:", "Email for \"forgot password\" links", "A fixed address for the garden computer",
      "Backups to a second place (a NAS, Google Drive, your PC)", "Your phone and PC at home (Termius, WinSCP)"],
     "Follow the recommended parts", "23-full-setup.html#choose-your-level"),
    ("champagne", "Full", "champagne", 8,
     ["Recommended, plus everything the garden has today:", "Online with your own web name, HTTPS and your sign-in page",
      "Subdomains, the wiki, friends, the Android app", "Reaching it from anywhere (Tailscale)"],
     "Follow the full setup", "23-full-setup.html"),
]


def level_time(word):
    """The level's time, from guide 23's "Choose your level" table (empty if not there yet)."""
    f = SRC / "23-full-setup.md"
    if not f.exists():
        return ""
    for line in f.read_text(encoding="utf-8").split("\n"):
        if line.startswith("|") and f"({word})" in line:
            m_ = re.search(r"\b[Aa]bout [^|]+", line)
            if m_:
                return m_.group(0).strip()
    return ""


def meter(n):
    dots = "".join(f'<i class="{"on" if k < n else ""}"></i>' for k in range(10))
    return (f'<div class="meter" role="img" aria-label="Chance of a frustrating moment: {n} out of 10">'
            f'<span class="dots">{dots}</span><span class="lbl">Chance of a frustrating moment: <b>{n} of 10</b></span></div>')


def plan_list(key):
    return "".join(f"<li>{html.escape(x)}</li>" for x in PLAN.get(key, []))


def guide_tree(n, href, is_open=False, here=False):
    topics = "".join(f'<li><a href="{href}#{tid}">{html.escape(t)}</a></li>' for tid, t in TOPICS[n])
    return (f'<details class="g{" here" if here else ""}"{" open" if is_open else ""}>'
            f'<summary><a href="{href}"{" aria-current=page" if here else ""}>{html.escape(TITLES[n])}</a></summary>'
            f'<ul>{topics}</ul></details>')


VIEW_JS = """<script>
(function () {
  var views = document.querySelectorAll('.view'), tabs = document.querySelectorAll('.views button');
  if (!views.length) return;
  function show(v) {
    views.forEach(function (s) { s.hidden = s.dataset.view !== v; });
    tabs.forEach(function (b) { b.setAttribute('aria-selected', b.dataset.view === v ? 'true' : 'false'); });
    document.querySelectorAll('a[data-keep]').forEach(function (a) { a.href = a.getAttribute('data-keep') + '#' + v; });
    document.body.setAttribute('data-view', v);
    try { localStorage.setItem('gardenView', v); } catch (e) {}
  }
  function fromHash() {
    var h = decodeURIComponent(location.hash.slice(1));
    if (h === 'brief' || h === 'ko' || h === 'full') { show(h); window.scrollTo(0, 0); return; }
    var t = h && document.getElementById(h);
    if (t) { var p = t.closest('.view'); if (p) { show(p.dataset.view); t.scrollIntoView(); } return; }
    var saved = null; try { saved = localStorage.getItem('gardenView'); } catch (e) {}
    show(saved || 'brief');
  }
  tabs.forEach(function (b) { b.addEventListener('click', function () { location.hash = b.dataset.view; }); });
  window.addEventListener('hashchange', fromHash);
  fromHash();
})();
</script>"""


def build():
    (HB / "assets").mkdir(parents=True, exist_ok=True)
    (HB / "assets" / "handbook.css").write_text(CSS, encoding="utf-8")
    logo = ""
    if LOGO and os.path.exists(LOGO):
        shutil.copyfile(LOGO, HB / "assets" / "logo.png")
        logo = "handbook/assets/logo.png"
    # One page per guide.
    for i, n in enumerate(ORDER):
        text = (SRC / f"{n}.md").read_text(encoding="utf-8")
        md_text = text.split("\n", 1)[1] if text.startswith("# ") else text
        # In brief: the guide's Quick version. Full: the rest of the guide.
        qm = re.search(r"^## (Quick version)\n(.*?)(?=^---\s*$|^## )", md_text, re.S | re.M)
        brief_md = qm.group(2) if qm else ""
        full_md = (md_text[:qm.start()] + md_text[qm.end():]).lstrip().lstrip("-").lstrip() if qm else md_text
        conv = lambda t, slug=None: markdown.Markdown(extensions=["tables", "fenced_code", "sane_lists", "toc"],
                                                     extension_configs={"toc": {"slugify": slug}} if slug else {}).convert(t)
        brief_md = "\n".join(l for l in brief_md.split("\n") if not l.startswith("*No time to read it all?"))
        brief = fix_links(conv(brief_md), HB) if brief_md else "<p>This guide has no short version; the full guide is one step away.</p>"
        body = fix_links(conv(full_md), HB)
        ko_file = SRC / "ko" / f"{n}.md"
        if ko_file.exists():
            kt = ko_file.read_text(encoding="utf-8")
            kt = kt.split("\n", 1)[1] if kt.startswith("# ") else kt
            from markdown.extensions.toc import slugify_unicode
            ko = fix_links(conv(kt, slugify_unicode), HB, SRC / "ko")
        else:
            ko = '<p lang="ko">이 안내서는 영어로만 있어요. <b>Full</b>(전체)을 눌러 영어로 읽어 보세요.</p>'
        heads = re.findall(r'<h2 id="([^"]+)">(.*?)</h2>', body)
        toc = "".join(f'<a href="#{hid}">{re.sub("<[^>]+>", "", t)}</a>' for hid, t in heads)
        folder, num, label = STEP_NAMES[n]
        a, soft, _line = colours(n)
        prev_n, next_n = (ORDER[i - 1] if i else None), (ORDER[i + 1] if i + 1 < len(ORDER) else None)
        no = lambda x: re.match(r"\d+[a-z]?", x).group(0)
        pager = ((f'<a href="{prev_n}.html" data-keep="{prev_n}.html"><small>Previous</small>&larr; {no(prev_n)}. {html.escape(TITLES[prev_n])}</a>') if prev_n else "<span></span>") + \
                ((f'<a href="{next_n}.html" data-keep="{next_n}.html" style="text-align:right"><small>Next</small>{no(next_n)}. {html.escape(TITLES[next_n])} &rarr;</a>') if next_n else "")
        quick_pdf = f'<a class="btn" href="{pdf_href(n, "QUICK", "../")}">In brief (PDF)</a>' if m.quick_part(text) else ""
        views = ('<div class="views" role="tablist" aria-label="How to read this guide">'
                 '<button role="tab" data-view="brief">In brief</button>'
                 '<button role="tab" data-view="ko" lang="ko">한국어</button>'
                 '<button role="tab" data-view="full">Full</button></div>')
        main = (f'<div class="shell"><nav class="steps tree" aria-label="All guides and their topics">{step_nav(n)}</nav>'
                f'<main class="read" style="--a:{a};--petal:{soft}">'
                f'<details class="allguides"><summary>All guides</summary><nav class="steps tree" style="display:block;position:static">{step_nav(n)}</nav></details>'
                f'<p class="where"><i></i>Step {html.escape(num)}: {html.escape(label)}</p>'
                f'<h1>{html.escape(TITLES[n])}</h1>{views}'
                f'<section class="view brief" data-view="brief" id="v-brief">{brief}'
                f'<p class="more"><a class="btn" href="#full">Read the full guide</a></p></section>'
                f'<section class="view" data-view="ko" id="v-ko" lang="ko">{ko}</section>'
                f'<section class="view" data-view="full" id="v-full">{body}</section>'
                f'<div class="pdfs">{quick_pdf}<a class="btn" href="{pdf_href(n, "FULL", "../")}">Full guide (PDF)</a></div>'
                f'<nav class="pager" aria-label="Previous and next guide">{pager}</nav></main>'
                f'<aside class="toc"><b>On this page</b>{toc}</aside></div>{VIEW_JS}')
        (HB / f"{n}.html").write_text(page(TITLES[n], main, ("../" + logo) if logo else "", "../"), encoding="utf-8")
    # The welcome page.
    steps = []
    for folder, _ko, names in m.STEPS:
        num, label = folder.split(" - ", 1)
        a = m.STEP_COLOURS.get(num, m.STEP_COLOURS["3"])[0]
        rows = []
        for n in names:
            if n not in STEP_NAMES:
                continue
            text = (SRC / f"{n}.md").read_text(encoding="utf-8")
            ko_link = f'<a href="handbook/{n}.html#ko" lang="ko">한국어</a>' if (SRC / "ko" / f"{n}.md").exists() else ""
            rows.append(f'<li><a class="title" href="handbook/{n}.html#brief">{html.escape(TITLES[n])}</a>'
                        f'<span class="pdf"><a href="handbook/{n}.html#brief">In brief</a>{ko_link}'
                        f'<a href="handbook/{n}.html#full">Full</a><a href="{pdf_href(n, "FULL", "")}">PDF</a></span></li>')
        if rows:
            steps.append(f'<section class="step" style="--a:{a}" id="step-{num}"><div class="leaf" aria-hidden="true"><span>{html.escape(num)}</span></div>'
                         f'<h2>{html.escape(label)}</h2><ul class="guides">{"".join(rows)}</ul></section>')
    mark = f'<img class="mark" src="{logo}" alt="">' if logo else ""
    blocks = []
    for folder, _ko, names in m.STEPS:
        num, label = folder.split(" - ", 1)
        a = m.STEP_COLOURS.get(num, m.STEP_COLOURS["3"])[0]
        gs = "".join(guide_tree(n, f"handbook/{n}.html") for n in names if n in STEP_NAMES)
        if gs:
            blocks.append(f'<div class="stepblock" style="--a:{a}"><h3><i></i>{html.escape(num)}. {html.escape(label)}</h3>{gs}</div>')
    map_html = "".join(blocks)
    full = "handbook/23-full-setup.html" if "23-full-setup" in STEP_NAMES else "00-HOW-IT-WORKS.html"
    steps_li = "".join(f'<li><span>{html.escape(what)}</span><small>{html.escape(STEP_TIME.get(fn, ""))}</small></li>' for fn, what in INSTALL)
    cards_l = []
    for key, name, word, risk, gets, btn, link in LEVELS:
        if link and not (SRC / "23-full-setup.md").exists():
            link = "00-HOW-IT-WORKS.html"
        href = f"handbook/{link}" if link and link.startswith("23-") else (link or f"handbook/{ORDER[0]}.html")
        t = level_time(word)
        items = "".join(f"<li>{html.escape(x)}</li>" for x in gets)
        cards_l.append(f'<div class="lvl {key}"><div class="glass">{GLASS[key]}</div>'
                       f'<h3>{html.escape(name)} <span>({html.escape(word)})</span></h3>'
                       f'{meter(risk)}'
                       + (f'<p class="time">Time: {html.escape(t)}</p>' if t else "")
                       + f'<ul>{items}</ul><a class="btn{" solid" if key == "wine" else ""}" href="{href}">{html.escape(btn)}</a></div>')
    setup_html = (
        f'<section class="ways" id="ways"><h2>Choose your level</h2>'
        f'<p class="calm">These three levels are only here to help you choose. Each one is a complete, working garden, '
        f'and every part has its own guide, one step at a time. The number is the chance of a frustrating moment along the way, '
        f'not a test. Start with the one that feels right today: you can always add the next level later, from the menu, '
        f'without starting again.</p>'
        f'<div class="three">{"".join(cards_l)}</div></section>'
        f'<section class="does"><h2>What the install does for you</h2>'
        f'<p class="sub">In this order, by itself. Each step explains itself and shows its commands before it runs them. '
        f'You only answer its questions. If it stops, run it again: it carries on where it stopped.</p>'
        f'<ol class="install">{steps_li}</ol>'
        f'<div class="plan"><div><h3>Necessary</h3><ul>{plan_list("NECESSARY")}</ul></div>'
        f'<div><h3>Optional: you choose</h3><ul>{plan_list("OPTIONAL")}</ul></div>'
        f'<div><h3>Later, from the menu</h3><ul>{plan_list("ADVANCED OPTIONAL")}</ul></div></div>'
        f'<p class="never"><b>Never done:</b> {html.escape(" ".join(PLAN.get("NOT DONE, EVER:", [])))}</p></section>')
    first = ORDER[0]
    welcome = (f'<section class="hero"><div><h1>Your garden, one step at a time.</h1>'
               f'<p>Every guide is here, in order. Read it on screen, or open its PDF to keep or print. '
               f'Everything opens from this computer, so you need no internet.</p>'
               f'<p class="quiet">Your garden itself: <a href="http://localhost:8030">http://localhost:8030</a> '
               f'(on Windows, the garden menu, 4). The same PDFs are in your Google Drive folder, "Setup documentation".</p>'
               f'<div class="begin"><a class="btn solid" href="handbook/{first}.html">Start with the first guide</a>'
               f'<a class="btn" href="00-HOW-IT-WORKS.html">See the whole path on one page</a>'
               f'<a class="btn" href="#map">Stuck? See every topic</a></div></div>{mark}</section>'
               f'{setup_html}'
               f'<div class="path" id="steps"><h2 class="pathhead">Every guide, step by step</h2>{"".join(steps)}</div>'
               f'<section class="map tree" id="map"><h2>If you are stuck: every topic, as a map</h2>'
               f'<p>Every step, every guide, every topic in it. Open a guide to see its topics, and click one to go straight there.</p>'
               f'<div class="cols">{map_html}</div></section>'
               f'<p class="foot">Garden Bridge: from Windows to Linux. Imagined by Jason, prototyped by Claude.</p>')
    (ROOT / "00-WELCOME.html").write_text(page("Welcome to your garden", welcome, logo, ""), encoding="utf-8")
    print(f"handbook: 00-WELCOME.html and {len(ORDER)} guide pages in {ROOT}")


if __name__ == "__main__":
    build()
