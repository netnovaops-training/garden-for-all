#!/usr/bin/env python3
"""Test-only: tools/get-code.py and tools/get-code.ps1 against a tiny local server (127.0.0.1, closed at the end).

    python tools/release/tests/test_get_code.py [--no-powershell]

The "GitHub" here is a local test server that hands out archives made from this repository's own staged
files (`git archive`, the way GitHub makes them), changed on purpose. Nothing real is touched: every run
happens in a new temporary garden folder (named with spaces and brackets), and the folder is compared
before and after, so "nothing was written" is checked, not assumed.

Cases: main mode ALL MATCH; one tampered file; one extra file; one missing file; a stale list; a path with
spaces, brackets and Korean; the old fixed-commit mode (a real old commit if this repository has one, and
a made-up one); a fixed commit that has the list; a damaged CODE-SOURCE.txt; and the same for PowerShell.
"""
import hashlib, http.server, io, os, shutil, subprocess, sys, tarfile, tempfile, threading, zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
sys.path.insert(0, os.path.join(ROOT, "tools", "release"))
import make_fingerprints as mf          # noqa: E402

REPO = "testowner/testrepo"
LIST = mf.LIST_NAME
FAKE_COMMIT = "0123456789abcdef0123456789abcdef01234567"
RESULTS = []


def ok(name, cond, detail=""):
    RESULTS.append(bool(cond))
    print(("PASS  " if cond else "FAIL  ") + name + ("" if cond else "   " + str(detail)[-500:]))


def git_tree_files(tree):
    return mf.tree_files(tree)


def make_tar(files, top="testrepo-main"):
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as t:
        d = tarfile.TarInfo(top); d.type = tarfile.DIRTYPE; d.mode = 0o755; t.addfile(d)
        for rel, data in sorted(files.items()):
            ti = tarfile.TarInfo(f"{top}/{rel}"); ti.size = len(data); ti.mode = 0o644
            t.addfile(ti, io.BytesIO(data))
    return buf.getvalue()


def make_zip(files, top="testrepo-main"):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        for rel, data in sorted(files.items()):
            zi = zipfile.ZipInfo(f"{top}/{rel}", date_time=(2026, 10, 14, 0, 0, 0))
            zi.create_system = 3; zi.external_attr = (0o100644) << 16
            z.writestr(zi, data)
    return buf.getvalue()


class Server:
    def __init__(self):
        self.routes = {}
        outer = self

        class H(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                body = outer.routes.get(self.path)
                if body is None:
                    self.send_response(404); self.end_headers(); return
                self.send_response(200); self.send_header("Content-Length", str(len(body))); self.end_headers()
                self.wfile.write(body)

            def log_message(self, *a):
                pass

        self.httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)   # local only
        self.base = f"http://127.0.0.1:{self.httpd.server_address[1]}"
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()

    def serve(self, kind, ref, files):
        self.routes.clear()
        self.routes[f"/{REPO}/tar.gz/{ref}"] = make_tar(files)
        self.routes[f"/{REPO}/zip/{ref}"] = make_zip(files)

    def close(self):
        self.httpd.shutdown(); self.httpd.server_close()


def sha(b):
    return hashlib.sha256(b).hexdigest()


def snapshot(d):
    snap = {}
    for root, _dirs, names in os.walk(d):
        for n in names:
            p = os.path.join(root, n)
            snap[os.path.relpath(p, d)] = sha(open(p, "rb").read())
    return snap


STARTERS = ["Start-Here-Ubuntu.sh", "Start-Here-Windows.cmd", "Garden-Menu-Windows.cmd", "tools/get-code.py", "tools/get-code.ps1"]


def new_garden(tmp, source_lines, zip_list_all=None):
    """A garden folder as the zip leaves it. Returns (folder, the files it holds that are in the list)."""
    d = os.path.join(tmp, "junsgarden-for-jun (1)", "junsgarden-for-jun")        # spaces and brackets on purpose
    os.makedirs(d)
    carried = {}
    for rel in STARTERS:
        carried[rel] = open(os.path.join(ROOT, rel), "rb").read()
    carried["README.md"] = b"the private read-me\n"
    carried["site/Boogie.mp3"] = b"not really music\n"
    carried["00-WELCOME.html"] = b"<html>welcome</html>\n"
    carried["handbook/index.html"] = b"<html>handbook</html>\n"
    carried["backups/junsgarden-backup_test.tar"] = b"tar-ish\n"
    carried["PHOTO-PARTS.json"] = b"{}\n"
    carried["CODE-SOURCE.txt"] = source_lines.encode()
    for rel, data in carried.items():
        p = os.path.join(d, *rel.split("/"))
        os.makedirs(os.path.dirname(p), exist_ok=True)
        open(p, "wb").write(mf.normalized(rel, data))
    # the zip's list: only what the zip carries, as it is on disk. (Old zips list everything: zip_list_all.)
    listed = {rel: mf.normalized(rel, data) for rel, data in carried.items() if rel != "CODE-SOURCE.txt"}   # the setting is not fingerprinted
    if zip_list_all:
        listed.update({k: v for k, v in zip_list_all.items() if k not in listed})   # the zip's own copies stay as they are
    lines = "".join(f"{sha(b)}  ./{rel}\n" for rel, b in sorted(listed.items()))
    open(os.path.join(d, "FILES-FROM-AUTHOR.txt"), "w", encoding="utf-8", newline="\n").write(lines)
    open(os.path.join(d, "photos.key"), "w").write("not a real key\n")           # in the zip, never listed
    return d


def run_py(d, base, *args):
    env = dict(os.environ, GARDEN_CODE_BASE=base, PYTHONIOENCODING="utf-8")
    r = subprocess.run([sys.executable, "-I", os.path.join(d, "tools", "get-code.py"), *args], capture_output=True, env=env, timeout=300)
    return r.returncode, (r.stdout + r.stderr).decode("utf-8", "replace")


def run_ps(d, base):
    env = dict(os.environ, GARDEN_CODE_BASE=base)
    script = os.path.join(d, "tools", "get-code.ps1")
    r = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script], capture_output=True, env=env, timeout=600)
    return r.returncode, (r.stdout + r.stderr).decode("utf-8", "replace")


def expected_files(files, listed_zip):
    """What a good run writes: every file of the download except the zip's own and the public-but-unlisted ones."""
    skip = set(listed_zip) | mf.OFF_GITHUB
    return {rel for rel in files if rel not in skip}


def main():
    use_ps = "--no-powershell" not in sys.argv and shutil.which("powershell.exe") is not None
    os.chdir(ROOT)
    tree = mf.git("write-tree").decode().strip()
    good = git_tree_files(tree)
    ok("the repository's own list is up to date (needed for this test)", good.get(LIST, b"").decode() == mf.make_list(good))
    # What GitHub's download of the public repository holds: the tracked files, plus the public copies of the
    # three files the zip carries privately (the public README.md and the two lists) which the list does not name.
    public = dict(good)
    public["README.md"] = b"the public read-me\n"
    public["DOWNLOADS-SHA256.txt"] = public.get("DOWNLOADS-SHA256.txt", b"downloads\n")
    public["FILES-FROM-AUTHOR.txt"] = b"the old list\n"
    public.pop("site/Boogie.mp3", None)
    srv = Server()
    tmp = tempfile.mkdtemp(prefix="getcode-test-")
    try:
        runners = [("python", lambda d: run_py(d, srv.base), "tar.gz")]
        if use_ps:
            runners.append(("powershell", lambda d: run_ps(d, srv.base), "zip"))
        for label, run, _kind in runners:
            print(f"--- {label}")
            sub = tempfile.mkdtemp(dir=tmp)

            def attempt(name, files, source="ref=main", ref="main", expect_ok=True, expect_text=None, extra_listed=None):
                srv.serve(None, ref, files)
                d = new_garden(tempfile.mkdtemp(dir=sub), f"repo={REPO}\n{source}\n", extra_listed)
                before = snapshot(d)
                rc, out = run(d)
                after = snapshot(d)
                if expect_ok:
                    ok(f"{label}: {name}: exit 0", rc == 0, out)
                else:
                    ok(f"{label}: {name}: refused (exit 1)", rc == 1, out)
                    ok(f"{label}: {name}: nothing written", before == after, set(after) ^ set(before))
                if expect_text:
                    ok(f"{label}: {name}: says '{expect_text}'", expect_text in out, out)
                return d, before, after, out

            # 1. main mode, everything right
            d, before, after, out = attempt("main, every file right", public)
            written = {w.replace(os.sep, "/") for w in set(after) - set(before)}
            want_new = {r for r in good if r not in mf.OFF_GITHUB and r not in STARTERS}
            ok(f"{label}: main: every program file was written ({len(written)})", written == want_new, (sorted(want_new - written)[:5], sorted(written - want_new)[:5]))
            ok(f"{label}: main: the zip's own files were not replaced",
               all(after[k] == before[k] for k in before), [k for k in before if after.get(k) != before[k]])
            ok(f"{label}: main: the public README, lists and the missing music were not written",
               after["README.md"] == before["README.md"] and after["FILES-FROM-AUTHOR.txt"] == before["FILES-FROM-AUTHOR.txt"])
            ok(f"{label}: main: CODE-FINGERPRINTS.txt is on disk, the same as the download", after.get(LIST) == sha(good[LIST]))
            sample = "tools/get-pdfs.ps1"
            ok(f"{label}: main: a .ps1 got Windows line endings, a .sh kept its own",
               open(os.path.join(d, *sample.split("/")), "rb").read() == mf.normalized(sample, good[sample]) and b"\r\n" in open(os.path.join(d, *sample.split("/")), "rb").read()
               and b"\r" not in open(os.path.join(d, "garden.sh"), "rb").read())
            # 2. one tampered file
            t = dict(public); t["garden.sh"] = public["garden.sh"] + b"# changed\n"
            attempt("one tampered file", t, expect_ok=False, expect_text="different from the list: garden.sh")
            # 3. one extra file
            t = dict(public); t["extra/new-file.txt"] = b"hello\n"
            attempt("one extra file", t, expect_ok=False, expect_text="extra, not on the list: extra/new-file.txt")
            # 4. one missing file
            t = dict(public); del t["rust/README.md"]
            attempt("one missing file", t, expect_ok=False, expect_text="missing: rust/README.md")
            # 5. a stale list: a new file and a changed file, the list not made again
            old_list = good[LIST].decode().splitlines(True)
            stale = "".join(old_list[:-1]).encode()                # the last line gone, as if made before that file existed
            t = dict(public); t[LIST] = stale
            last = old_list[-1].split("  ", 1)[1].strip()
            attempt("a stale list (a file is not on it)", t, expect_ok=False, expect_text=f"extra, not on the list: {last}")
            changed = good[LIST].decode().replace(old_list[0][:64], "0" * 64, 1).encode()
            t = dict(public); t[LIST] = changed
            attempt("a stale list (a wrong fingerprint)", t, expect_ok=False, expect_text="different from the list")
            t = dict(public); del t[LIST]
            attempt("no list at all", t, expect_ok=False, expect_text=f"{LIST} is not in the download")
            # 6. a path with spaces, brackets and Korean (the file and the list agree; the garden folder has spaces too)
            odd_name = "docs/with space (1)/a b - 한글.txt"
            t = dict(public); t[odd_name] = b"odd name\n"
            t[LIST] = (good[LIST].decode() + f"{sha(b'odd name' + chr(10).encode())}  {odd_name}\n").encode("utf-8")
            d, before, after, out = attempt("a path with spaces, brackets and Korean", t)
            ok(f"{label}: odd path: written", os.path.isfile(os.path.join(d, "docs", "with space (1)", "a b - 한글.txt")), out)
            # 7. the old fixed-commit way: a version without the list, the zip listing every file
            old = {k: v for k, v in public.items() if k != LIST}
            all_listed = {rel: mf.normalized(rel, data) for rel, data in old.items() if rel not in mf.OFF_GITHUB}
            d, before, after, out = attempt("fixed commit, old way (no list in the download), every file against the zip's list",
                                            old, source=f"commit={FAKE_COMMIT}", ref=FAKE_COMMIT, extra_listed=all_listed)
            ok(f"{label}: old way: a program file was written", os.path.isfile(os.path.join(d, "garden.sh")), out)
            t = dict(old); t["garden.sh"] = old["garden.sh"] + b"# changed\n"
            attempt("fixed commit, old way, one tampered file", t, source=f"commit={FAKE_COMMIT}", ref=FAKE_COMMIT,
                    expect_ok=False, expect_text="different from the list: garden.sh", extra_listed=all_listed)
            t = dict(old); t["extra/x.txt"] = b"x\n"
            attempt("fixed commit, old way, one extra file", t, source=f"commit={FAKE_COMMIT}", ref=FAKE_COMMIT,
                    expect_ok=False, expect_text="not in the list: extra/x.txt", extra_listed=all_listed)
            # 7b. a REAL old commit of this repository (made before the list existed), checked the old way
            real = next((c for c in ("77b0b3b", "70c479a", "979929f") if subprocess.run(["git", "cat-file", "-e", c + "^{commit}"], capture_output=True).returncode == 0), None)
            if real:
                full = subprocess.run(["git", "rev-parse", real], capture_output=True, text=True).stdout.strip()
                oldreal = {k: v for k, v in git_tree_files(full).items() if k != "site/Boogie.mp3"}
                listed_real = {rel: mf.normalized(rel, data) for rel, data in oldreal.items() if rel not in mf.OFF_GITHUB}
                d, before, after, out = attempt(f"a real old commit ({real}), the old way", oldreal, source=f"commit={full}", ref=full, extra_listed=listed_real)
                ok(f"{label}: real old commit: program files written", os.path.isfile(os.path.join(d, "garden.sh")), out)
            # 7c. an old-style zip whose list also names CODE-SOURCE.txt, which the person then edited: still fine
            stale_source = {"CODE-SOURCE.txt": b"the list names an older CODE-SOURCE.txt"}
            attempt("CODE-SOURCE.txt edited after an old-style list named it", old, source=f"commit={FAKE_COMMIT}", ref=FAKE_COMMIT,
                    extra_listed={**all_listed, **stale_source})
            # 8. a fixed commit whose download has the list: checked against the list
            attempt("fixed commit that has the list", public, source=f"commit={FAKE_COMMIT}", ref=FAKE_COMMIT)
            attempt("fixed commit as ref=<hex>", public, source=f"ref={FAKE_COMMIT}", ref=FAKE_COMMIT)
            # 9. a damaged CODE-SOURCE.txt
            srv.serve(None, "main", public)
            d = new_garden(tempfile.mkdtemp(dir=sub), f"repo={REPO}\nref=main\ncommit={FAKE_COMMIT}\n")
            before = snapshot(d); rc, out = run(d)
            ok(f"{label}: both ref and commit: refused, nothing written", rc != 0 and snapshot(d) == before, out)
            d = new_garden(tempfile.mkdtemp(dir=sub), f"repo={REPO}\nref=../../etc\n")
            before = snapshot(d); rc, out = run(d)
            ok(f"{label}: a ref with '..': refused, nothing written", rc != 0 and snapshot(d) == before, out)
            # 10. no connection
            srv.routes.clear()
            d = new_garden(tempfile.mkdtemp(dir=sub), f"repo={REPO}\nref=main\n")
            before = snapshot(d); rc, out = run(d)
            ok(f"{label}: GitHub not answering (404): refused, nothing written", rc != 0 and snapshot(d) == before, out)
        # --check (the release build's way) against a downloaded archive
        srv.serve(None, "main", public)
        d = new_garden(tempfile.mkdtemp(dir=tmp), f"repo={REPO}\nref=main\n")
        arc = os.path.join(tmp, "good.tar.gz"); open(arc, "wb").write(make_tar(public))
        rc, out = run_py(d, srv.base, "--check", arc)
        ok("python --check: main download: ALL MATCH", rc == 0 and out.strip() == "ALL MATCH", out)
        bad = dict(public); bad["garden.sh"] += b"x"
        arc2 = os.path.join(tmp, "bad.tar.gz"); open(arc2, "wb").write(make_tar(bad))
        rc, out = run_py(d, srv.base, "--check", arc2)
        ok("python --check: tampered download: refused", rc == 1 and "different from the list: garden.sh" in out, out)
    finally:
        srv.close()
        shutil.rmtree(tmp, ignore_errors=True)
    print(f"\n{sum(RESULTS)} of {len(RESULTS)} checks passed")
    sys.exit(0 if all(RESULTS) else 1)


if __name__ == "__main__":
    main()
