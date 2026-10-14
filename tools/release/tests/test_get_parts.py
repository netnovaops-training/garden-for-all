#!/usr/bin/env python3
"""Test-only: tools/get-parts.py (the multi-host, small-parts, once-per-version download) and
tools/release/make_photo_parts.py. Everything is made up and local: a small synthetic photos zip
(about 30 MB, parts of 1 MB), fake hosts on localhost (python http.server with Range support, plus
a slow one, a dead one, one that serves a corrupted part, one that cuts a part off), a throwaway
key outside every repo. Nothing real is touched, nothing goes on the internet.

    python tools/release/tests/test_get_parts.py
"""
import hashlib, http.server, json, os, shutil, socket, subprocess, sys, tempfile, threading, time, zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
PY = sys.executable
GET = os.path.join(ROOT, "tools", "get-parts.py")
GETPDF = os.path.join(ROOT, "tools", "get-pdfs.py")
MAKE = os.path.join(ROOT, "tools", "release", "make_photo_parts.py")
ENV = {**os.environ, "PYTHONIOENCODING": "utf-8", "GARDEN_GET_RETRY_WAIT": "0.05", "GARDEN_GET_TIMEOUT": "5",
       "GARDEN_PINNED_PATIENCE": "2"}
passed = failed = 0


def ok(name, cond, extra=""):
    global passed, failed
    if cond:
        passed += 1
        print("PASS ", name)
    else:
        failed += 1
        print("FAIL ", name, extra)


class Host:
    """A fake host. files: {url path -> file path}. Options change how it answers."""

    def __init__(self, files, delay=0.0, corrupt=(), cutoff=(), down=False, by_id=None, silent=False, token=None):
        self.files, self.delay, self.corrupt, self.cutoff = files, delay, set(corrupt), set(cutoff)
        self.down, self.by_id, self.silent, self.token = down, by_id or {}, silent, token
        self.auth = []                  # 200 / 401 per request of a host that wants a token (the token itself is never kept)
        self.log, self.cut_done, self.lock = [], set(), threading.Lock()
        host = self

        class H(http.server.BaseHTTPRequestHandler):
            def log_message(self, *a):
                pass

            def do_GET(self):
                if host.down:
                    with host.lock:
                        host.log.append((host.by_id.get(self.path.split("id=")[-1], "") if "id=" in self.path else self.path.lstrip("/"), "down"))
                    self.connection.close()
                    return
                if host.token is not None:        # a private repository: Bearer token and the raw media type are required
                    good = (self.headers.get("Authorization") == "Bearer " + host.token
                            and self.headers.get("Accept") == "application/vnd.github.raw")
                    with host.lock:
                        host.auth.append(200 if good else 401)
                    if not good:
                        self.send_error(401)
                        return
                if host.silent:                       # accepts the request and never answers
                    with host.lock:
                        host.log.append((host.by_id.get(self.path.split("id=")[-1], "") if "id=" in self.path else self.path.lstrip("/"), "silent"))
                    for _ in range(150):
                        time.sleep(0.1)
                    return
                path = self.path.split("?")[0].lstrip("/")
                if "id=" in self.path:
                    path = host.by_id.get(self.path.split("id=")[1], "")
                f = host.files.get(path)
                rng = self.headers.get("Range")
                with host.lock:
                    host.log.append((path, rng))
                if not f:
                    self.send_error(404)
                    return
                data = open(f, "rb").read()
                if path in host.corrupt:
                    data = bytes([data[0] ^ 255]) + data[1:]
                start = int(rng.split("=")[1].rstrip("-")) if rng else 0
                body = data[start:]
                self.send_response(206 if rng else 200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Length", str(len(body)))
                if rng:
                    self.send_header("Content-Range", f"bytes {start}-{len(data) - 1}/{len(data)}")
                self.end_headers()
                if path in host.cutoff and path not in host.cut_done:
                    host.cut_done.add(path)
                    self.wfile.write(body[: len(body) // 2])
                    self.wfile.flush()
                    self.connection.close()
                    return
                for i in range(0, len(body), 16384):
                    if host.delay:
                        time.sleep(host.delay)
                    try:
                        self.wfile.write(body[i:i + 16384])
                    except OSError:
                        return

        self.srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)
        self.srv.daemon_threads = True
        self.port = self.srv.server_address[1]
        threading.Thread(target=self.srv.serve_forever, daemon=True).start()

    def url(self, pattern="{name}"):
        return f"http://127.0.0.1:{self.port}/" + pattern

    def names(self):
        return [p for p, _ in self.log]

    def stop(self):
        self.srv.shutdown()
        self.srv.server_close()


def closed_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def run(args, env=None, **kw):
    r = subprocess.run([PY, "-I", "-X", "utf8"] + args, capture_output=True, text=True, encoding="utf-8", env={**ENV, **(env or {})}, **kw)
    return r.returncode, r.stdout + r.stderr


def main():
    T = tempfile.mkdtemp(prefix="garden-parts-test-")
    try:
        return body(T)
    finally:
        shutil.rmtree(T, ignore_errors=True)


def body(T):
    # ---- a small synthetic photos zip (about 30 MB of random bytes) and its encrypted 1 MB parts
    zdir, keydir, outdir = (os.path.join(T, d) for d in ("zip", "key", "parts"))
    os.makedirs(zdir)
    zpath = os.path.join(zdir, "junsgarden-photos-addon.zip")
    with zipfile.ZipFile(zpath, "w", zipfile.ZIP_STORED) as z:
        z.writestr("images.tar", os.urandom(30 * (1 << 20)))
    zsha = hashlib.sha256(open(zpath, "rb").read()).hexdigest()
    rc, out = run([MAKE, zpath, outdir, "--key-dir", keydir, "--part-mb", "1", "--tag", "photos-test",
                   "--author-every", "10", "--private-part", "1", "--private-repo", "testowner/testrepo@main"])
    ok("make_photo_parts: builds the parts and the list", rc == 0 and os.path.isfile(os.path.join(outdir, "PHOTO-PARTS.json")), out)
    key = open(os.path.join(keydir, "photos.key")).read()
    ok("make_photo_parts: never prints the key", key.splitlines()[0] not in out)
    rc, out2 = run([MAKE, zpath, os.path.join(T, "x"), "--key-dir", ROOT])
    ok("make_photo_parts: refuses a key folder inside a git repo", rc != 0 and "STOP" in out2)
    man = json.load(open(os.path.join(outdir, "PHOTO-PARTS.json")))
    parts = man["parts"]
    names = [p["name"] for p in parts]
    ok("the list: about 30 parts, each with its own sha, hosts and the whole-file sha",
       28 <= len(parts) <= 33 and all(len(p["sha256"]) == 64 for p in parts) and man["sha256"] == zsha
       and man["encryption"]["key_id"] == key.splitlines()[1][3:])
    ok("the list: encrypted bytes differ from the zip (the parts are not the plain zip)",
       open(os.path.join(outdir, names[1]), "rb").read(64) != open(zpath, "rb").read(64))
    first = parts[0]
    privdir = os.path.join(T, "private-repo", "parts")
    ok("the list: part 1 lists drive and the private repository only (not github), prefers drive, priority 1",
       first["hosts"] == ["drive", "ghprivate:testowner/testrepo@main"] and first.get("prefer") == "drive" and first.get("priority") == 1 and "pinned" not in first)
    ok("the layout: part 1 is only in private-repo/parts, never in the public folder; the others are public",
       os.listdir(privdir) == [first["name"]] and not os.path.exists(os.path.join(outdir, first["name"]))
       and all(os.path.isfile(os.path.join(outdir, n)) for n in names[1:]))
    # Give every Drive entry a file id the way the release does.
    ids = {n: f"ID{i:03d}" for i, n in enumerate(names)}
    idsfile = os.path.join(T, "ids.txt")
    open(idsfile, "w").write("".join(f"{n} {i}\n" for n, i in ids.items()))
    manp = os.path.join(T, "PHOTO-PARTS.json")
    shutil.copy(os.path.join(outdir, "PHOTO-PARTS.json"), manp)
    run([MAKE, "--set-ids", manp, idsfile, "--host", "drive"])
    man = json.load(open(manp))
    manp_pin = os.path.join(T, "PHOTO-PARTS-pinned.json")      # the same parts, part 1 held by Drive only (the generic "pinned" feature)
    mp = json.load(open(manp))
    mp["parts"][0]["hosts"] = [h for h in mp["parts"][0]["hosts"] if h.startswith("drive")]
    mp["parts"][0]["pinned"] = "drive"
    del mp["parts"][0]["prefer"]
    json.dump(mp, open(manp_pin, "w"))
    author_parts = {p["name"] for p in man["parts"] if "author" in p["hosts"]}
    ok("set-ids: every Drive entry names its file id", all(any(h.startswith("drive:ID") for h in p["hosts"]) for p in man["parts"]))
    files = {n: os.path.join(outdir if i else privdir, n) for i, n in enumerate(names)}
    TOKEN = "ghp_TESTTOKEN_" + "x" * 20
    PRIV_PATH = "repos/testowner/testrepo/contents/parts/"
    priv_files = {PRIV_PATH + names[0]: files[names[0]]}
    PRIV = "repos/{id}/contents/parts/{name}?ref={ref}"

    def tokfile(name, text=None):
        d = os.path.join(T, "tok-" + name)
        os.makedirs(d, exist_ok=True)
        path = os.path.join(d, "photos-access.token")
        if text is not False:
            open(path, "w").write((text or TOKEN) + "\n")
        return path
    by_id = {ids[n]: n for n in names}
    nodrive = os.path.join(T, "no-hosts.txt")
    open(nodrive, "w").write("")

    def go(dest, hosts, key_path=None, extra=(), manifest=None, env=None, token=None):
        a = [GET, "photos", "--dest", dest, "--key", key_path or os.path.join(keydir, "photos.key"),
             "--manifest", manifest or manp, "--hosts", nodrive, "--token", token or os.path.join(T, "no-token-here")]
        for k, v in hosts.items():
            a += ["--host", f"{k}={v}"]
        return run(a + list(extra), env=env)

    def fresh(name):
        d = os.path.join(T, name)
        os.makedirs(d)
        return d

    def installed(d):
        f = os.path.join(d, "junsgarden-photos-addon.zip")
        return os.path.isfile(f) and hashlib.sha256(open(f, "rb").read()).hexdigest() == zsha

    def nothing_written(d):
        return not [f for f in os.listdir(d) if f.endswith(".zip") or f.endswith(".partial")]

    def hostset(github, drive, author=None, **kw):
        h = {"github": github.url("{name}"), "drive": drive.url("uc?export=download&id={id}")}
        if author:
            h["author"] = author.url("{name}")
        return h

    # ---- 1. a good run uses all hosts; pinned part first and only from Drive
    gh = Host(files, delay=0.002)
    dr = Host(files, delay=0.002, by_id=by_id)
    au = Host({n: files[n] for n in author_parts}, delay=0.002)
    # the Drive host answers by id: map "uc" requests through by_id (files keyed by part name)
    d1 = fresh("d1")
    rc, out = go(d1, hostset(gh, dr, au))
    ok("good run: exit 0 and the finished zip is exactly the original (whole-file sha)", rc == 0 and installed(d1), out)
    ok("good run: every host served parts (github, drive, author)", gh.log and dr.log and au.log, f"{len(gh.log)} {len(dr.log)} {len(au.log)}")
    ok("good run: the author's host got only the parts it holds", set(au.names()) <= author_parts)
    ok("good run: the pinned part came from Drive only, never from github or author",
       names[0] not in gh.names() and names[0] not in au.names() and names[0] in dr.names())
    ok("good run: Drive's very first request was the pinned part (fetched first)", dr.names()[0] == names[0])
    ok("good run: the parts folder is gone, the key is not in the output", not os.path.exists(os.path.join(d1, ".garden-downloads", "photos")) and key.splitlines()[0] not in out)
    ok("good run: one shared progress line shows parts and MB", "parts," in out and "MB" in out)

    # ---- 2. second run downloads nothing
    n_before = (len(gh.log), len(dr.log), len(au.log))
    rc, out = go(d1, hostset(gh, dr, au))
    ok("second run: 'already up to date' and no request at all", rc == 0 and "already up to date" in out and (len(gh.log), len(dr.log), len(au.log)) == n_before, out)
    rc, _ = go(d1, hostset(gh, dr, au), extra=["--is-current"])
    ok("--is-current says 0 when current", rc == 0)
    rc, out = go(d1, hostset(gh, dr, au), extra=["--again"])
    ok("--again fetches again and still ends right", rc == 0 and installed(d1) and len(gh.log) > n_before[0])

    # ---- 3. a dead host: its parts go to the others
    d3 = fresh("d3")
    dead = f"http://127.0.0.1:{closed_port()}/{{name}}"
    rc, out = go(d3, {"github": dead, "drive": dr.url("uc?export=download&id={id}"), "author": au.url("{name}")})
    ok("dead host (github): the others finish it, exit 0, file right", rc == 0 and installed(d3), out)
    ok("dead host: it is reported as left out", "kept failing" in out or "github" in out)

    # ---- 4. a corrupted part is re-fetched from another host; nothing bad is written
    bad_name = names[4]
    gh_bad = Host(files, corrupt=[bad_name])
    dr2 = Host(files, by_id=by_id, delay=0.02)
    d4 = fresh("d4")
    rc, out = go(d4, {"github": gh_bad.url("{name}"), "drive": dr2.url("uc?export=download&id={id}")})
    ok("corrupt part: re-fetched from another host, final file right", rc == 0 and installed(d4), out)
    ok("corrupt part: github served the bad bytes once, Drive supplied the good part", bad_name in gh_bad.names() and bad_name in dr2.names())

    # ---- 5. a slow host takes fewer parts than a fast one
    slow = Host(files, delay=0.03)
    fast = Host(files, by_id=by_id)
    d5 = fresh("d5")
    rc, out = go(d5, {"github": slow.url("{name}"), "drive": fast.url("uc?export=download&id={id}")})
    served_slow = len({p for p in slow.names()})
    served_fast = len(set(fast.names()))
    ok("slow host: the fast host took more parts", rc == 0 and installed(d5) and served_fast > served_slow, f"{served_fast} vs {served_slow}")

    # ---- 6. a cut-off part carries on with Range
    cutter = Host(files, cutoff=names[6:8])
    d6 = fresh("d6")
    rc, out = go(d6, {"github": cutter.url("{name}"), "drive": Host(files, by_id=by_id, delay=0.05).url("uc?export=download&id={id}")})
    reqs = [r for p, r in cutter.log if p in names[6:8]]
    ok("cut-off: the part resumed with a Range request and the file is right", rc == 0 and installed(d6) and any(r for r in reqs), f"{reqs} {out[-300:]}")

    # ---- 7. missing key / 8. wrong key
    d7 = fresh("d7")
    g7 = Host(files)
    rc, out = go(d7, {"github": g7.url("{name}")}, key_path=os.path.join(T, "nokey", "photos.key"))
    ok("missing key: exit 4, nothing downloaded, nothing written", rc == 4 and not g7.log and nothing_written(d7) and not os.path.exists(os.path.join(d7, ".garden-downloads")), out)
    ok("missing key: says so in plain words, English and Korean, and where the key is",
       "key file (photos.key) is missing" in out and "열쇠" in out and "junsgarden-for-jun.zip" in out)
    d8 = fresh("d8")
    os.makedirs(os.path.join(T, "k2"))
    open(os.path.join(T, "k2", "photos.key"), "w").write("0" * 64 + "\nid=deadbeef\n")
    rc, out = go(d8, {"github": g7.url("{name}")}, key_path=os.path.join(T, "k2", "photos.key"))
    ok("key of another download (id differs): refused before downloading", rc == 4 and not g7.log and nothing_written(d8))
    open(os.path.join(T, "k2", "photos.key"), "w").write("0" * 64 + "\nid=" + man["encryption"]["key_id"] + "\n")
    g8 = Host(files, by_id=by_id)
    rc, out = go(d8, {"github": g8.url("{name}"), "drive": g8.url("uc?export=download&id={id}")}, key_path=os.path.join(T, "k2", "photos.key"))
    ok("wrong key (right id): decrypt gives a wrong file, nothing installed, exit 4", rc == 4 and nothing_written(d8) and "Nothing was installed" in out, out)
    n = len(g8.log)
    rc, out = go(d8, {"github": g8.url("{name}"), "drive": g8.url("uc?export=download&id={id}")})
    ok("then the right key: the kept parts are used (no new downloads) and it installs", rc == 0 and installed(d8) and len(g8.log) == n, f"{len(g8.log)} {n}")

    # ---- 9. only a subset host is left: fails cleanly
    d9 = fresh("d9")
    au9 = Host({n: files[n] for n in author_parts})
    rc, out = go(d9, {"author": au9.url("{name}")})
    ok("only the subset host: the run fails, writes nothing, says so", rc == 1 and nothing_written(d9), out)

    # ---- 10. pinned part: Drive slow or dead; never asked of other hosts; rest carries on; Drive back completes
    dr_dead = Host(files, by_id=by_id, down=True)
    gh10 = Host(files)
    d10 = fresh("d10")
    t0 = time.time()
    rc, out = go(d10, {"github": gh10.url("{name}"), "drive": dr_dead.url("uc?export=download&id={id}")}, manifest=manp_pin)
    ok("pinned, Drive dead: the run fails, nothing is installed", rc == 1 and nothing_written(d10), out[-400:])
    ok("pinned, Drive dead: says 'still waiting for Google Drive' and 'part 1 of N' (and in Korean)",
       "still waiting for google drive" in out.lower() and f"part 1 of {len(names)}" in out and "구글 드라이브" in out, out[-500:])
    ok("pinned, Drive dead: part 1 was never asked of github", names[0] not in gh10.names())
    cache = os.path.join(d10, ".garden-downloads", "photos")
    ok("pinned, Drive dead: the other parts are in the kept folder", len([f for f in os.listdir(cache) if not f.endswith(".part")]) == len(names) - 1)
    dr_dead.down = False
    before = len(gh10.log)
    rc, out = go(d10, {"github": gh10.url("{name}"), "drive": dr_dead.url("uc?export=download&id={id}")}, manifest=manp_pin)
    ok("pinned, Drive back: only the missing part is fetched, the file completes", rc == 0 and installed(d10) and len(gh10.log) == before, out[-300:])
    ok("pinned, Drive back: part 1 came from Drive", names[0] in dr_dead.names())
    # slow Drive: github is never asked for part 1 either
    dslow = Host(files, by_id=by_id, delay=0.02)
    g11 = Host(files)
    d11 = fresh("d11")
    rc, out = go(d11, {"github": g11.url("{name}"), "drive": dslow.url("uc?export=download&id={id}")}, manifest=manp_pin)
    ok("pinned, Drive slow: github still never gets part 1, the file is right", rc == 0 and installed(d11) and names[0] not in g11.names())
    ok("pinned: the other parts still use the queue (github took many)", len(set(g11.names())) > 5)

    # ---- 10b. part 1 is not public: Drive first and only while it answers; if not, the private repository (with the token)
    def with_priv(**kw):
        h = {"github": kw["gh"].url("{name}"), "drive": kw["dr"].url("uc?export=download&id={id}")}
        if kw.get("priv") is not None:
            h["ghprivate"] = kw["priv"].url(PRIV)
        return h

    dr_ok, gh_ok, pv_ok = Host(files, by_id=by_id), Host(files), Host(priv_files, token=TOKEN)
    d12, t12 = fresh("d12"), tokfile("12")
    rc, out = go(d12, with_priv(gh=gh_ok, dr=dr_ok, priv=pv_ok), token=t12)
    ok("part 1, Drive healthy: from Drive first and only; the private repository and github are never asked; no fallback message",
       rc == 0 and installed(d12) and dr_ok.names()[0] == names[0] and names[0] not in gh_ok.names() and not pv_ok.log and "did not answer" not in out, out[-300:])
    ok("token: the access file is gone after a successful run (and the log says so neutrally, in English and Korean)",
       not os.path.exists(t12) and "The temporary access file was removed." in out and "임시 접근 파일을 지웠어요." in out, out[-300:])
    dr_down, gh_b, pv_b = Host(files, by_id=by_id, down=True), Host(files), Host(priv_files, token=TOKEN)
    d13, t13 = fresh("d13"), tokfile("13")
    rc, out = go(d13, with_priv(gh=gh_b, dr=dr_down, priv=pv_b), token=t13)
    ok("part 1, Drive dead: Drive tried first, then the private repository gave it (with the token); github never; the file is right",
       rc == 0 and installed(d13) and names[0] in dr_down.names() and names[0] not in gh_b.names() and pv_b.auth == [200], f"{pv_b.auth} " + out[-400:])
    ok("part 1, Drive dead: says 'Google Drive did not answer, using the backup (part 1 of N)' in English and Korean, nothing about content",
       f"Google Drive did not answer, using the backup (part 1 of {len(names)})" in out and "구글 드라이브 응답이 없어요" in out and out.count("part 1 of") <= 2, out[-500:])
    seen = [TOKEN in out]
    for root, _d, fs in os.walk(d13):
        for f in fs:
            seen.append(TOKEN.encode() in open(os.path.join(root, f), "rb").read())
    ok("token: never in the output and never written into the garden folder or its parts", not any(seen))
    ok("token: removed after the success (zeros then deleted)", not os.path.exists(t13))
    dr_back, pv_back = Host(files, by_id=by_id), Host(priv_files, token=TOKEN)
    n_gh = len(gh_b.log)
    rc, out = go(d13, with_priv(gh=gh_b, dr=dr_back, priv=pv_back))
    ok("once-only: Drive coming back later is not needed; the second run fetches nothing ('already up to date'), even without the access file",
       rc == 0 and "already up to date" in out and not dr_back.log and not pv_back.log and len(gh_b.log) == n_gh, out[-200:])
    dr_sil, gh_c, pv_c = Host(files, by_id=by_id, silent=True), Host(files), Host(priv_files, token=TOKEN)
    d14, t14 = fresh("d14"), tokfile("14")
    t0 = time.time()
    rc, out = go(d14, with_priv(gh=gh_c, dr=dr_sil, priv=pv_c), token=t14,
                 env={"GARDEN_GET_TIMEOUT": "2", "GARDEN_PREFER_TIMEOUT": "2"})
    ok("part 1, Drive silent: after the time limit the private repository gives it; the run ends right and soon",
       rc == 0 and installed(d14) and pv_c.auth == [200] and time.time() - t0 < 60, f"{time.time() - t0:.0f}s " + out[-300:])
    # the access file is missing: plain message, no hang, nothing written
    dr_dead2, pv_d = Host(files, by_id=by_id, down=True), Host(priv_files, token=TOKEN)
    d15 = fresh("d15")
    t0 = time.time()
    rc, out = go(d15, with_priv(gh=Host(files), dr=dr_dead2, priv=pv_d), token=tokfile("none", False))
    ok("access file missing and Drive dead: says so (English and Korean), exit 1, no hang, nothing installed, the private host never asked",
       rc == 1 and "photos-access.token" in out and "백업" in out and nothing_written(d15) and not pv_d.log and time.time() - t0 < 60, f"{time.time() - t0:.0f}s " + out[-500:])
    # wrong token: 401 -> unavailable, said plainly; the access file STAYS after a failed run; then the right one completes it
    pv_w = Host(priv_files, token=TOKEN)
    d16, t16 = fresh("d16"), tokfile("16", "ghp_WRONG_TOKEN")
    rc, out = go(d16, with_priv(gh=Host(files), dr=Host(files, by_id=by_id, down=True), priv=pv_w), token=t16)
    ok("wrong token: the private host answers 401, it is called unavailable in plain words, exit 1, nothing installed, the access file stays",
       rc == 1 and nothing_written(d16) and set(pv_w.auth) == {401} and "did not accept the access" in out and os.path.exists(t16) and "ghp_WRONG_TOKEN" not in out, out[-500:])
    open(t16, "w").write(TOKEN + "\n")
    rc, out = go(d16, with_priv(gh=Host(files), dr=Host(files, by_id=by_id), priv=pv_w), token=t16)
    ok("after the failed run the verified parts were kept: a second run with Drive back and the right token completes, then the access file is gone",
       rc == 0 and installed(d16) and not os.path.exists(t16), out[-300:])
    pv_404 = Host({}, token=TOKEN)
    d17, t17 = fresh("d17"), tokfile("17")
    rc, out = go(d17, with_priv(gh=Host(files), dr=Host(files, by_id=by_id, down=True), priv=pv_404), token=t17)
    ok("private host answers 404: unavailable, exit 1, access file stays, no hang",
       rc == 1 and os.path.exists(t17) and "did not accept the access" in out and nothing_written(d17), out[-300:])
    d18, t18 = fresh("d18"), tokfile("18")
    t0 = time.time()
    rc, out = go(d18, {"github": Host(files).url("{name}"), "drive": Host(files, by_id=by_id, down=True).url("uc?export=download&id={id}"),
                       "ghprivate": f"http://127.0.0.1:{closed_port()}/" + PRIV}, token=t18)
    ok("Drive and the backup both dead: gives up in bounded time (no loop), exit 1, nothing installed, access file stays",
       rc == 1 and nothing_written(d18) and os.path.exists(t18) and time.time() - t0 < 60, f"{time.time() - t0:.0f}s " + out[-300:])

    # ---- 11. PDFs through the same code: two hosts, the first dead; second run up to date; --again
    g = os.path.join(T, "garden")
    os.makedirs(os.path.join(g, "tools"))
    for f in ("get-pdfs.py", "get-parts.py"):
        shutil.copy(os.path.join(ROOT, "tools", f), os.path.join(g, "tools", f))
    serve = os.path.join(T, "serve")
    os.makedirs(serve)
    pdfs = {}
    for code, folder, mem in (("en", "English", ["0 - Start/a.pdf", "1 - Two/b.pdf"]), ("ko", "Korean", ["0 - 시작/a.pdf"])):
        zn = f"junsgarden-pdf-guides-{'english' if code == 'en' else 'korean'}.zip"
        with zipfile.ZipFile(os.path.join(serve, zn), "w") as z:
            for m in mem:
                z.writestr(f"{folder}/{m}", b"%PDF-1.4 test " + m.encode())
        b = open(os.path.join(serve, zn), "rb").read()
        pdfs[code] = f"{hashlib.sha256(b).hexdigest()}  {zn} {len(b)} {code} github,author"
    open(os.path.join(g, "PDF-GUIDES-SHA256.txt"), "w").write(
        f"url=http://127.0.0.1:{closed_port()}/x\n" + pdfs["en"] + "\n" + pdfs["ko"] + "\n")
    pdf_host = Host({f: os.path.join(serve, f) for f in os.listdir(serve)})
    open(os.path.join(g, "DOWNLOAD-HOSTS.txt"), "w").write(f"author  {pdf_host.url('{name}')}\n")
    dd = os.path.join(T, "pd")
    os.makedirs(dd)
    rc, out = run([os.path.join(g, "tools", "get-pdfs.py"), "both", "--dest", dd])
    n_pdf = sum(f.endswith(".pdf") for _r, _d, fs in os.walk(dd) for f in fs)
    ok("PDFs: github dead, the author's host serves them (same code), 3 PDFs in", rc == 0 and n_pdf == 3, out)
    k = len(pdf_host.log)
    rc, out = run([os.path.join(g, "tools", "get-pdfs.py"), "both", "--dest", dd])
    ok("PDFs: second run is 'already up to date' and fetches nothing", rc == 0 and "already up to date" in out and len(pdf_host.log) == k, out)
    rc, _ = run([os.path.join(g, "tools", "get-pdfs.py"), "both", "--dest", dd, "--is-current"])
    ok("PDFs: --is-current is 0", rc == 0)
    rc, out = run([os.path.join(g, "tools", "get-pdfs.py"), "en", "--dest", dd, "--again"])
    ok("PDFs: --again fetches again", rc == 0 and len(pdf_host.log) > k)
    ok("PDFs: the key/parts leave nothing behind but the record", not os.path.exists(os.path.join(dd, ".garden-downloads", "pdf-en")))

    # ---- 12. the glue in garden.sh: photos_start (background) and photos_wait (one progress line), the real functions
    bash = shutil.which("bash")
    if bash:
        src = open(os.path.join(ROOT, "garden.sh"), encoding="utf-8").read()
        glue = src[src.index("PHOTOS_PID=\n"):src.index("# Menu 6, 11 (or: bash garden.sh photos)")]
        gd = os.path.join(T, "gardenfolder")
        os.makedirs(os.path.join(gd, "tools"))
        shutil.copy(GET, os.path.join(gd, "tools", "get-parts.py"))
        shutil.copy(manp, os.path.join(gd, "PHOTO-PARTS.json"))
        shutil.copy(os.path.join(keydir, "photos.key"), os.path.join(gd, "photos.key"))
        gg, gdr = Host(files), Host(files, by_id=by_id)
        open(os.path.join(gd, "DOWNLOAD-HOSTS.txt"), "w").write(
            "github " + gg.url("{name}") + "\ndrive " + gdr.url("uc?export=download&id={id}") + "\n")
        drv = os.path.join(T, "glue.sh")
        pyx = PY.replace("\\", "/")
        open(drv, "w", newline="\n").write(
            'python3() { "' + pyx + '" "$@"; }\nnote() { echo "NOTE $*"; }\nfind_download() { return 1; }\nhave_photos() { return 1; }\n'
            'HERE="$PWD"\n' + glue + '\nphotos_start\nphotos_wait\nls\n')
        r = subprocess.run([bash, drv], cwd=gd, capture_output=True, text=True, encoding="utf-8", env=ENV)
        ok("garden.sh glue: photos_start/photos_wait fetch the photos in the background, zip lands in the garden folder",
           "junsgarden-photos-addon.zip" in r.stdout and "arriving in the background" in r.stdout and installed(gd), r.stdout[-400:] + r.stderr[-300:])
        shutil.rmtree(os.path.join(gd, ".garden-downloads"), ignore_errors=True)
        os.remove(os.path.join(gd, "junsgarden-photos-addon.zip"))
        os.remove(os.path.join(gd, "photos.key"))
        r = subprocess.run([bash, drv], cwd=gd, capture_output=True, text=True, encoding="utf-8", env=ENV)
        ok("garden.sh glue: with no key the script carries on, says so in plain words, writes no zip",
           r.returncode == 0 and "photos.key" in r.stdout and not installed(gd) and not os.path.exists(os.path.join(gd, "junsgarden-photos-addon.zip")), r.stdout[-500:])

    print(f"RESULT: {passed} passed, {failed} failed")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
