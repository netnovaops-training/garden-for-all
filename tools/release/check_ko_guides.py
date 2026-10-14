"""Do the Korean guides say the same as the English ones?

    python tools/release/check_ko_guides.py

For every guide that has a Korean version (START-HERE/ko/), compares what must
never differ between languages: the web links, the commands and code in
`backticks` and code blocks, and the pictures. Lists what one side has and the
other does not. A Korean guide that is only a short version (it says so and
links to the English) is checked on its links and pictures only.
"""
import pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parents[2] / "START-HERE"


# Differences that are right by design: the Korean side points to the Korean
# version of a file or picture, and translates placeholder words and comments.
SAME = [(r"\.ko\.md\b", ".md"), (r"-KO\.html\b", ".html"), (r"-ko\.svg\b", "-en.svg"),
        ("날짜", "DATE"), ("노트북이름", "LAPTOP-NAME")]


# Checked by hand and right by design, for these files only (2026-10-13):
KNOWN = {("04-go-public-no-ip-caddy.md", "link only in English: http://LAPTOP-NAME.local"),    # Korean: http://노트북이름.local
         ("13-backups-and-folders.md", "code only in English: date       time label (optional)"),  # a labelled diagram
         ("23-full-setup.md", "`00-READ-ME-FIRST.html` only in English")}  # Korean links 00-READ-ME-FIRST-KO.html


def normal(x):
    for a, b in SAME:
        x = re.sub(a, b, x)
    return x


def facts(text):
    text = re.sub(r"\*\*", "", text)                      # bold marks are not part of a link
    urls = {normal(u.rstrip(".,);:")) for u in re.findall(r"https?://[A-Za-z0-9._~:/?#\[\]@!$&'*+,;=%-]+", text)}
    blocks = re.findall(r"```.*?\n(.*?)```", text, re.S)
    code = set()
    for b in blocks:
        for l in b.split("\n"):
            l = re.sub(r"\s+#\s.*$", "", l).strip()       # a comment after the command may be translated
            if l and re.search(r"[A-Za-z0-9]", l) and not re.search(r"[가-힣]", l):
                code.add(normal(l))
    inline = {normal(c) for c in re.findall(r"`([^`\n]+)`", re.sub(r"```.*?```", "", text, flags=re.S))}
    pics = {normal(p) for p in re.findall(r"images/([\w.-]+\.(?:png|svg|jpg))", text)}
    return urls, code, inline, pics


def short_version(text):
    return "영어 원문(전체 안내서)" in text or "전체 내용은 영어 안내서에 있어요" in text


bad = 0
for en in sorted(ROOT.glob("*.md")):
    ko = ROOT / "ko" / en.name
    if not ko.exists():
        continue
    te, tk = en.read_text(encoding="utf-8"), ko.read_text(encoding="utf-8")
    eu, ec, ei, ep = facts(te)
    ku, kc, ki, kp = facts(tk)
    problems = []
    short = short_version(tk)
    if not short:
        problems += [f"link only in English: {u}" for u in sorted(eu - ku)]
        problems += [f"code only in English: {c}" for c in sorted(ec - kc)]
        problems += [f"`{c}` only in English" for c in sorted(ei - ki) if not re.fullmatch(r"[0-9A-Z]", c)]
        problems += [f"picture only in English: {p}" for p in sorted(ep - kp)]
    problems += [f"link only in Korean: {u}" for u in sorted(ku - eu)]
    problems += [f"code only in Korean: {c}" for c in sorted(kc - ec)]
    problems += [f"picture only in Korean: {p}" for p in sorted(kp - ep)]
    problems = [p for p in problems if (en.name, p) not in KNOWN]
    status = "short version" if short else "full"
    if problems:
        bad += 1
        print(f"\n== {en.name} ({status}): {len(problems)} difference(s)")
        for p in problems[:25]:
            print("   ", p)
    else:
        print(f"OK  {en.name} ({status})")
print(f"\n{bad} guide(s) with differences")
sys.exit(1 if bad else 0)
