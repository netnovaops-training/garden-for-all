"""Check the stable help ids: START-HERE/help-ids.tsv and the anchors in the guides.

    python tools/release/check_help_ids.py                 # the registry and the guides
    python tools/release/check_help_ids.py --handbook DIR  # also a built handbook (DIR/handbook/)

Fails (exit 1) when:
 - the registry is not plain UTF-8 (no BOM, no Windows line endings), its columns
   are not the documented ones, or an id is repeated or not short-and-plain;
 - an id has no  <a id="help-ID"></a>  in its English AND its Korean guide, or the
   line after it is not a heading, or the heading's own anchor is no longer the one
   written in the registry (a reworded heading: run this, then fix the row);
 - a guide has a help anchor that is not in the registry;
 - tags are not 2 to 4 words starting with #, or a brief is missing;
 - with --handbook: a page lacks #help-ID or #help-ID-ko.
"""
import pathlib, re, sys
import markdown
from markdown.extensions.toc import slugify_unicode

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = ROOT / "START-HERE"
REG = SRC / "help-ids.tsv"
COLS = ["id", "guide", "anchor_en", "anchor_ko", "tags_en", "tags_ko", "brief_en", "brief_ko", "wizard"]
ANCHOR = re.compile(r'^<a id="help-([a-z0-9-]+)"></a>\s*$')


def read_registry(problems):
    raw = REG.read_bytes()
    if raw.startswith(b"\xef\xbb\xbf"):
        problems.append("help-ids.tsv starts with a BOM")
    if b"\r" in raw:
        problems.append("help-ids.tsv has Windows line endings")
    lines = [l for l in raw.decode("utf-8").split("\n") if l and not l.startswith("#")]
    if not lines or lines[0].split("\t") != COLS:
        problems.append("help-ids.tsv: the column names must be: " + " ".join(COLS))
        return []
    rows = []
    for n, l in enumerate(lines[1:], 2):
        f = l.split("\t")
        if len(f) != len(COLS):
            problems.append(f"help-ids.tsv row {n}: {len(f)} columns, not {len(COLS)}")
            continue
        rows.append(dict(zip(COLS, f)))
    return rows


def heading_slug(heading, ko):
    md = markdown.Markdown(extensions=["toc"], extension_configs={"toc": {"slugify": slugify_unicode}} if ko else {})
    m = re.search(r'id="([^"]+)"', md.convert(heading))
    return m.group(1) if m else ""


def anchors_in(path):
    """help id -> the heading line after it, for every help anchor in one guide."""
    out, lines = {}, path.read_text(encoding="utf-8").splitlines()
    for i, l in enumerate(lines):
        m = ANCHOR.match(l)
        if m:
            nxt = [x for x in lines[i + 1:i + 4] if x.strip()]
            out.setdefault(m.group(1), []).append(nxt[0] if nxt else "")
    return out


def main():
    problems = []
    rows = read_registry(problems)
    seen = set()
    for r in rows:
        i = r["id"]
        if not re.fullmatch(r"[a-z0-9]+(-[a-z0-9]+)*", i):
            problems.append(f"{i}: an id is small letters, numbers and - only")
        if i in seen:
            problems.append(f"{i}: listed twice")
        seen.add(i)
        for lang, sub, ko in (("en", "", False), ("ko", "ko/", True)):
            g = SRC / sub / (r["guide"] + ".md")
            if not g.exists():
                problems.append(f"{i}: no guide {sub}{r['guide']}.md"); continue
            found = anchors_in(g).get(i, [])
            if len(found) != 1:
                problems.append(f"{i}: {sub}{r['guide']}.md has {len(found)} <a id=\"help-{i}\"></a> lines (needs exactly 1)"); continue
            if not found[0].startswith("#"):
                problems.append(f"{i}: in {sub}{r['guide']}.md the line after the anchor is not a heading"); continue
            now = heading_slug(found[0], ko)
            if now != r["anchor_" + lang]:
                problems.append(f"{i}: the {lang} heading's anchor is now '{now}', the registry says '{r['anchor_' + lang]}'")
        for lang in ("en", "ko"):
            tags = r["tags_" + lang].split()
            if not 2 <= len(tags) <= 4 or not all(re.fullmatch(r"#\S+", t) for t in tags):
                problems.append(f"{i}: tags_{lang} must be 2 to 4 words, each starting with #")
            if not r["brief_" + lang].strip():
                problems.append(f"{i}: brief_{lang} is empty")
    # Every help anchor in the guides must be in the registry, for that guide.
    where = {r["id"]: r["guide"] for r in rows}
    for g in sorted(SRC.glob("*.md")) + sorted((SRC / "ko").glob("*.md")):
        for i in anchors_in(g):
            if i not in where:
                problems.append(f"{g.relative_to(SRC)}: help-{i} is not in help-ids.tsv")
            elif where[i] != g.stem:
                problems.append(f"{g.relative_to(SRC)}: help-{i} belongs to {where[i]}.md in the registry")
    if "--handbook" in sys.argv:
        hb = pathlib.Path(sys.argv[sys.argv.index("--handbook") + 1]) / "handbook"
        for r in rows:
            page = hb / (r["guide"] + ".html")
            text = page.read_text(encoding="utf-8") if page.exists() else ""
            for a in (f'id="help-{r["id"]}"', f'id="help-{r["id"]}-ko"'):
                if a not in text:
                    problems.append(f"{r['id']}: handbook/{page.name} has no {a}")
    for p in problems:
        print("PROBLEM:", p)
    print(f"RESULT: {len(rows)} help ids, {len(problems)} problem(s)" + ("" if problems else ", OK"))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
