"""Check every outside web address in the guides, read-mes and scripts.

    python tools/release/check_links.py

For each address: does the page open? (Some sites refuse automatic visitors
with 403 or 429; those are listed as "check by hand", not as broken.) For the
pages the guides rely on most, it also checks the page still says what the
guide expects (MUST_CONTAIN below). Writes the full list to
link-check-report.txt next to this package folder.
Nothing here is needed to RUN the garden.
"""
import concurrent.futures as cf, pathlib, re, subprocess, sys

PKG = pathlib.Path(__file__).resolve().parents[2]
REPORT = PKG.parent / "link-check-report.txt"
FILES = [p for pat in ("*.md", "*.html", "START-HERE/**/*.md", "garden.sh", "windows/*.ps1", "docker/**/*.yml",
                       "docker/**/Dockerfile", "docker-compose.yml", "beds/*.md", "android-app/README.md", "lang/ko.tsv")
         for p in PKG.glob(pat)]
URL = re.compile(r"https?://[A-Za-z0-9][A-Za-z0-9.-]*\.[A-Za-z]{2,}(?:[/?#][^\s<>\"'`)\]|]*)?")
SKIP = re.compile(r"(localhost|127\.0\.0\.1|192\.168\.|\.test\b|\.local\b|\.example\b|example\.(com|org)|junsgarden\.zapto\.org|"
                  r"momsgarden|testgarden|duckdns\.org/update|junsgarden\.duckdns|yourname|NAME\.|host\.docker\.internal|"
                  r"\$|openrouter\.ai/api/|perenual\.com/api/)")   # $ = an address built by the code, not a page to read
# Pages whose WORDS the guides depend on: the address, and words that must be on it.
MUST_CONTAIN = {
    "https://rclone.org/drive/": ["client_id", "Making your own client_id"],
    "https://www.noip.com/free": ["Hostname"],
    "https://www.duckdns.org/spec.jsp": ["TXT"],
    "https://rufus.ie": ["Rufus"],
    "https://releases.ubuntu.com/24.04/": ["24.04"],
    "https://caddyserver.com/docs/automatic-https": ["Let's Encrypt"],
    "https://docs.docker.com/engine/install/ubuntu/": ["Ubuntu"],
    "https://www.authelia.com/integration/proxies/caddy/": ["forward_auth"],
    "https://aistudio.google.com/apikey": [],
}
UA = "Mozilla/5.0 (link check for Jun's Garden guides)"


def fetch(url):
    """Status and the start of the page, with curl (it follows redirects and uses
    the system's certificates, like a browser)."""
    try:
        r = subprocess.run(["curl", "-sL", "--max-time", "25", "-A", UA, "-H", "Accept-Language: en",
                            "-r", "0-600000", "-w", "\n%{http_code}", url], capture_output=True, timeout=40)
        out = r.stdout.decode("utf-8", "replace")
        body, _, code = out.rpartition("\n")
        code = int(code) if code.strip().isdigit() else 0
        # 206 = "partial content": the page answered with the part we asked for.
        return (200 if code == 206 else code) if code else f"error: curl {r.returncode}", body
    except Exception as e:
        return f"error: {type(e).__name__}", ""


def main():
    found = {}
    for f in FILES:
        for u in URL.findall(f.read_text(encoding="utf-8", errors="replace")):
            u = u.rstrip(".,;:")
            if not SKIP.search(u):
                found.setdefault(u, set()).add(f.relative_to(PKG).as_posix())
    for u in MUST_CONTAIN:
        found.setdefault(u, {"(checked page)"})
    results = {}
    with cf.ThreadPoolExecutor(12) as ex:
        for u, res in zip(found, ex.map(fetch, found)):
            results[u] = res
    broken, by_hand, words_missing, ok = [], [], [], 0
    for u, (status, body) in sorted(results.items()):
        where = ", ".join(sorted(found[u]))[:150]
        if status in (403, 429, 405, 999):
            by_hand.append(f"{status}  {u}   [{where}]")
        elif status != 200:
            broken.append(f"{status}  {u}   [{where}]")
        else:
            ok += 1
            missing = [w for w in MUST_CONTAIN.get(u, []) if w.lower() not in body.lower()]
            if missing:
                words_missing.append(f"{u}  is missing: {missing}   [{where}]")
    lines = [f"Checked {len(results)} addresses: {ok} open fine, {len(broken)} broken, "
             f"{len(by_hand)} to check by hand, {len(words_missing)} with expected words missing.", ""]
    for title, items in (("BROKEN (fix these)", broken), ("EXPECTED WORDS MISSING", words_missing),
                         ("CHECK BY HAND (the site refuses automatic checks)", by_hand)):
        lines += [f"== {title}"] + (items or ["(none)"]) + [""]
    REPORT.write_text("\n".join(lines), encoding="utf-8")
    print("\n".join(lines[:1] + [l for l in lines[2:] if l.startswith("==") or not l.startswith(" ")][:80]))
    print(f"\nFull list: {REPORT}")
    return 1 if broken or words_missing else 0


if __name__ == "__main__":
    sys.exit(main())
