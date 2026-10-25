#!/usr/bin/env python3
"""Side by side: ask the old build (127.0.0.1:18101) and the new one (:18102) the same
questions and compare the answers. Run ON THE NUC from ~/garden-rust-pages.

    python3 sbs.py urls.txt [--only catalog] [--self]

urls.txt, one request per line ('#' starts a comment):
    catalog.php?ajax_load=1&page=2           GET, not logged in
    @login catalog.php                       GET, logged in as the lab's Tester account
    @app app_api.php?action=whoami           as the Android app: its login key as Authorization: Bearer
    POST plant_actions.php | action=x&id=1   form POST (add @login in front to be logged in)
    POST comments.php | a=1 | Origin: https://evil.example    extra request headers, ';'-separated
    expect: <text>    after a request: the NEW side is expected to differ (a listed fix);
                      the line is reported as "fix" instead of DIFF
    expect: maybe ... the new side may or may not differ (the old side is random there)
--self compares the OLD build with itself (two requests), to show what changes on every load.

For each request: same status, same Location, same Content-Type, same security headers, and
the same body (HTML compared as the browser parses it, JSON as data; see compare_pages.py).
Results go to out/<n>.base / out/<n>.cand and one line per request on screen.
"""
import http.cookiejar
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request

BASE, CAND = "http://127.0.0.1:18101/", "http://127.0.0.1:18102/"
HEADERS = ("content-type", "location", "content-security-policy", "x-frame-options",
           "x-content-type-options", "referrer-policy", "cache-control")
HERE = os.path.dirname(os.path.abspath(__file__))


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *a, **k):
        return None


def opener(jar):
    return urllib.request.build_opener(NoRedirect, urllib.request.HTTPCookieProcessor(jar))


def fetch(op, url, body=None, extra=None):
    headers = {"X-Forwarded-For": "10.9.8.7", "User-Agent": "garden-sbs", **(extra or {})}
    req = urllib.request.Request(url, data=body.encode() if body is not None else None, headers=headers)
    if body is not None:
        req.add_header("Content-Type", "application/x-www-form-urlencoded")
    try:
        r = op.open(req, timeout=60)
    except urllib.error.HTTPError as e:
        r = e
    return r.status, {k.lower(): v for k, v in r.headers.items()}, r.read()


def login(root):
    jar = http.cookiejar.CookieJar()
    op = opener(jar)
    pw = dict(line.strip().split("=", 1) for line in open(os.path.join(HERE, "test-login.env")) if "=" in line)["TEST_PW"]
    fetch(op, root + "login.php")
    st, h, _ = fetch(op, root + "login.php", urllib.parse.urlencode({"admin_user": "Tester", "admin_password": pw, "admin_login": "", "back": ""}))
    if st not in (302, 303) or not any(c.name for c in jar):
        sys.exit(f"login on {root} failed ({st}); run `sh lab.sh tester` first")
    return op


def app_key(root):
    """Log in the way the Android app does (POST app_api.php?action=login) and return its key."""
    import json
    pw = dict(line.strip().split("=", 1) for line in open(os.path.join(HERE, "test-login.env")) if "=" in line)["TEST_PW"]
    st, _, body = fetch(opener(http.cookiejar.CookieJar()), root + "app_api.php?action=login",
                        urllib.parse.urlencode({"username": "Tester", "password": pw}))
    token = json.loads(body or b"{}").get("token") if st == 200 else None
    if not token:
        sys.exit(f"app login on {root} failed ({st})")
    return token


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    only = sys.argv[sys.argv.index("--only") + 1] if "--only" in sys.argv else None
    cand_root = BASE if "--self" in sys.argv else CAND
    raw = [l.rstrip("\n") for l in open(args[0], encoding="utf-8")]
    raw = [l.strip() for l in raw if l.strip() and not l.lstrip().startswith("#")]
    # Pair each request with the "expect:" line that may follow it.
    lines = []
    for l in raw:
        if l.startswith("expect:") and lines:
            lines[-1] = (lines[-1][0], l[7:].strip())
        else:
            lines.append((l, ""))
    lines = [x for x in lines if not only or only in x[0]]
    os.makedirs(os.path.join(HERE, "out"), exist_ok=True)
    anon = {r: opener(http.cookiejar.CookieJar()) for r in (BASE, cand_root)}
    logged = {}
    same = diff = fixed = 0
    keys = {}
    for n, (line, expect) in enumerate(lines, 1):
        auth = line.startswith("@login ")
        app = line.startswith("@app ")
        line = line.removeprefix("@login ").removeprefix("@app ").strip()
        body, extra = None, {}
        if line.startswith("POST "):
            parts = [p.strip() for p in line[5:].split("|")]
            line, body = parts[0], parts[1] if len(parts) > 1 else ""
            if len(parts) > 2:
                extra = dict(h.split(":", 1) for h in parts[2].split(";") if ":" in h)
                extra = {k.strip(): v.strip() for k, v in extra.items()}
        res = {}
        for root in (BASE, cand_root):
            if auth and root not in logged:
                logged[root] = login(root)
            hdrs = dict(extra)
            if app:
                if root not in keys:
                    keys[root] = app_key(root)
                hdrs["Authorization"] = "Bearer " + keys[root]
            res[root] = fetch((logged if auth else anon)[root], root + line.lstrip("/"), body, hdrs)
        (sa, ha, old), (sb, hb, new) = res[BASE], res[cand_root]
        problems = []
        if sa != sb:
            problems.append(f"status {sa} vs {sb}")
        for k in HEADERS:
            va, vb = ha.get(k, ""), hb.get(k, "")
            if k == "location":
                va, vb = va.replace(BASE, "/"), vb.replace(cand_root, "/")
            if va != vb:
                problems.append(f"{k}: {va[:60]!r} vs {vb[:60]!r}")
        fa, fb = (os.path.join(HERE, "out", f"{n}.{s}") for s in ("base", "cand"))
        if "json" in ha.get("content-type", "") and "json" in hb.get("content-type", ""):
            old, new = untimed(old), untimed(new)
        if "html" in ha.get("content-type", ""):
            # The data page prints the time it was made, to the second.
            stamp = re.compile(rb"Updated: \d{4}-\d\d-\d\d \d\d:\d\d:\d\d")
            old, new = stamp.sub(b"Updated: T", old), stamp.sub(b"Updated: T", new)
        open(fa, "wb").write(old)
        open(fb, "wb").write(new)
        if old != new:
            kind = ["--json"] if "json" in ha.get("content-type", "") else []
            if "html" in ha.get("content-type", "") or kind:
                out = subprocess.run([sys.executable, os.path.join(HERE, "compare_pages.py"), fa, fb, *kind],
                                     capture_output=True, text=True)
                if out.returncode:
                    problems.append("body: " + " | ".join(out.stdout.strip().splitlines()[:4]))
            else:
                problems.append(f"body bytes differ ({len(old)} vs {len(new)})")
        tag = ("@login " if auth else "@app " if app else "") + ("POST " if body is not None else "") + line
        if problems and expect:
            fixed += 1
            print(f"fix  {n:3} {tag[:90]}\n       expected: {expect}\n       " + "\n       ".join(problems))
        elif problems:
            diff += 1
            print(f"DIFF {n:3} {tag[:90]}\n       " + "\n       ".join(problems))
        elif expect.startswith("maybe"):
            same += 1
            print(f"same {n:3} {tag[:90]}  ({expect})")
        elif expect:
            diff += 1
            print(f"DIFF {n:3} {tag[:90]}\n       expected a change ({expect}) but both answered the same")
        else:
            same += 1
            print(f"same {n:3} {tag[:90]}")
    print(f"\n{same} same, {fixed} changed as expected (fixes), {diff} different")
    sys.exit(1 if diff else 0)


def untimed(raw):
    """JSON made comparable: every "ts" (a comment's Unix time) set to 0, because the two sides
    post a second apart; and a cross-table (rowLabels/colLabels/matrix) turned into
    {row: {col: count}} so only its contents count, not the order of rows with equal totals
    (the old build's order for ties is random; the new one sorts ties by name)."""
    import json

    def walk(v):
        if isinstance(v, dict):
            if {"rowLabels", "colLabels", "matrix"} <= v.keys():
                v = dict(v)
                rows, cols, m = v.pop("rowLabels"), v.pop("colLabels"), v.pop("matrix")
                v.pop("rowTotals", None), v.pop("colTotals", None)
                v["cells"] = {str(r): {str(c): m[i][j] for j, c in enumerate(cols)} for i, r in enumerate(rows)}
            return {k: (0 if k in ("ts", "generated_at") else walk(x)) for k, x in v.items()}
        if isinstance(v, list):
            return [walk(x) for x in v]
        return v
    try:
        return json.dumps(walk(json.loads(raw)), ensure_ascii=False, sort_keys=True).encode()
    except ValueError:
        return raw


if __name__ == "__main__":
    main()
