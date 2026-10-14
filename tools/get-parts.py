#!/usr/bin/env python3
"""Get a download that comes in small parts, from several places at once, and check it.

One program serves the photos add-on (about 32 parts of 25 MB, encrypted) and the two
PDF guide zips (one part each; tools/get-pdfs.py uses this file). Needs only Python 3
(and, for the photos, openssl, which Ubuntu has). No git, no extra programs.

How it works
  - The list of parts is in the program: PHOTO-PARTS.json (name, size, SHA-256 of every
    part and which hosts hold it; the SHA-256 and size of the finished file; the id of
    the key). The hosts and their addresses are in DOWNLOAD-HOSTS.txt.
  - One connection per host. Each connection takes the next part nobody has taken yet
    that its host holds, so fast hosts take more parts and, when the list is empty,
    everyone finishes within one part of each other.
  - A part can be preferred on one host ("prefer" in the list; it is fetched first from there and the
    others take it only if that host does not answer: 3 failures in a row or GARDEN_PREFER_TIMEOUT seconds
    of silence, default 60) or pinned (only that host, no fallback).
  - A host that is down or answers wrong (an error page, the wrong size, a part whose
    fingerprint does not match) hands its parts to the others; a bad part is fetched
    again alone, from another host. A cut-off part carries on where it stopped (HTTP Range).
  - Every part is checked against its SHA-256 BEFORE anything else is done with it.
  - Once per version: a download that is already in and verified is never fetched again
    (a small record in .garden-downloads/ says which version came in).
  - A part that is not public has a private GitHub repository as its backup host: it is read with the
    access file photos-access.token (private data package, like photos.key), sent only in a header to that
    host and never printed. When the whole set is in and checked, the access file is overwritten and removed.
  - The photos are encrypted (they are Jun's own garden photos). The key is in the private
    data package (photos.key), never on a public host. The parts are decrypted as a
    stream (openssl, AES-256-CTR) into a temporary file; only when the finished zip has
    the SHA-256 in the list is it moved into place. A wrong or missing key writes nothing.

    python3 tools/get-parts.py photos [--dest FOLDER] [--again] [--is-current]
        --key FILE            the key file (default: photos.key in the garden folder)
        --manifest FILE       another list (a test)
        --hosts FILE          another DOWNLOAD-HOSTS.txt (a test)
        --token FILE          the access file for the private backup host (default: photos-access.token in the garden folder)
        --host ID=ADDRESS     one host's address, overriding the file (a test; may repeat;
                              {tag} {name} {id} are filled in; file:// works)

Exit codes: 0 done or already up to date; 1 failed (nothing was written); 2 with
--is-current: not up to date; 4 key missing or wrong; 5 openssl missing.
"""
import hashlib, json, os, shutil, subprocess, sys, tempfile, threading, time, urllib.parse, urllib.request

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # the garden folder
STATE_DIR = ".garden-downloads"
CHUNK = 64 * 1024
MAX_HOST_FAILS = 3              # in a row; then the host is left out for this run
TIMEOUT = float(os.environ.get("GARDEN_GET_TIMEOUT", "30"))
PREFER_TIMEOUT = float(os.environ.get("GARDEN_PREFER_TIMEOUT", "60"))   # seconds a preferred host may stay silent before others take its part
PINNED_PATIENCE = float(os.environ.get("GARDEN_PINNED_PATIENCE", "300"))   # seconds a pinned part's host may stay silent per run
HOST_NAMES = {"drive": "Google Drive", "github": "GitHub", "author": "the author's server", "codeberg": "Codeberg", "ghprivate": "the backup"}
HOST_NAMES_KO = {"drive": "구글 드라이브", "github": "GitHub", "author": "제작자의 서버", "codeberg": "Codeberg", "ghprivate": "백업"}
TOKEN_FILE = "photos-access.token"    # read-only access to the private backup repo; it comes in the private data package, like photos.key


def is_private(host):
    """A host that needs the access token (a private GitHub repository, read through the contents API)."""
    return host.startswith("ghprivate")
RETRY_WAIT = float(os.environ.get("GARDEN_GET_RETRY_WAIT", "1"))


def say(en, ko):
    print("   " + en, flush=True)
    print("   " + ko, flush=True)


# ---------------------------------------------------------------- lists

def sha_of_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def load_hosts(path, overrides=()):
    """DOWNLOAD-HOSTS.txt -> {host id: address pattern}. A host whose address is a
    placeholder ("-" or PLACEHOLDER...) is not set up and is left out."""
    hosts = {}
    if path and os.path.isfile(path):
        for line in open(path, encoding="utf-8"):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split(None, 1)
            if len(parts) == 2:
                hosts[parts[0]] = parts[1].strip()
    for o in overrides:
        k, _, v = o.partition("=")
        hosts[k.strip()] = v.strip()
    return {k: v for k, v in hosts.items() if v and v != "-" and "PLACEHOLDER" not in v.upper()}


class Part:
    def __init__(self, name, size, sha, holders, pinned="", priority=0, number=0, prefer=""):
        self.name, self.size, self.sha = name, size, sha
        self.holders = holders          # [(host id, file id or "")]
        self.pinned = pinned            # only this host ever gives this part (no fallback)
        self.priority = priority        # >0: fetched before the others, lowest first
        self.number = number            # "part N of M" in messages
        self.prefer = prefer            # fetched first from this host; the others take it only if this host does not answer


class PartSet:
    def __init__(self, name, tag, outfile, parts, size=0, sha="", enc=None):
        self.name, self.tag, self.outfile, self.parts = name, tag, outfile, parts
        self.size, self.sha, self.enc = size, sha, enc

    @property
    def version(self):
        return f"{self.tag}:{self.sha[:16]}"

    @property
    def total(self):
        return sum(p.size for p in self.parts)


def holders_of(entries):
    out = []
    for e in entries:
        host, _, fid = str(e).partition(":")
        out.append((host, fid))
    return out


def load_manifest(path):
    m = json.load(open(path, encoding="utf-8"))
    if m.get("format") != 1 or not m.get("parts"):
        raise ValueError("damaged list")
    parts = []
    for i, p in enumerate(m["parts"], 1):
        hold = holders_of(p.get("hosts", []))
        pin = p.get("pinned", "")
        if pin:
            hold = [h for h in hold if h[0] == pin]       # a pinned part has no other holder
        prefer = p.get("prefer", "") if p.get("prefer", "") in [h[0] for h in hold] else ""
        parts.append(Part(p["name"], int(p["size"]), p["sha256"], hold, pin, int(p.get("priority", 0) or 0), i, prefer))
    return PartSet(m.get("set", "photos"), m["tag"], m["file"], parts, int(m["size"]), m["sha256"], m.get("encryption"))


def url_for(hosts, host, tag, name, fid):
    pattern = hosts.get(host)
    if not pattern or ("{id}" in pattern and not fid):
        return None
    ref = ""
    if "@" in fid:                       # "owner/repo@ref" (a private repository)
        fid, ref = fid.split("@", 1)
    return (pattern.replace("{tag}", tag).replace("{name}", urllib.parse.quote(name))
            .replace("{id}", urllib.parse.quote(fid, safe="/")).replace("{ref}", urllib.parse.quote(ref or "main")))


class _KeepTokenHome(urllib.request.HTTPRedirectHandler):
    """The access token goes only to the host it was meant for: a redirect elsewhere drops it."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        new = super().redirect_request(req, fp, code, msg, headers, newurl)
        if new is not None and urllib.parse.urlparse(newurl).netloc != urllib.parse.urlparse(req.full_url).netloc:
            for h in ("Authorization", "authorization"):
                new.headers.pop(h, None)
                new.unredirected_hdrs.pop(h, None)
        return new


# ---------------------------------------------------------------- the download

class HostWrong(Exception):
    """This host cannot give this part right now (down, error page, wrong size, bad fingerprint).
    retry=True: the connection broke (the same host may try again and carry on); False: it gave
    something wrong, so another host takes this part."""

    def __init__(self, why, retry=False):
        super().__init__(why)
        self.retry = retry


def open_url(url, start, token=""):
    """(response-like, offset the bytes really start at). file:// is read from the disk (tests)."""
    if url.startswith("file://"):
        f = open(urllib.request.url2pathname(urllib.parse.urlparse(url).path), "rb")
        f.seek(start)
        return f, start
    headers = {"User-Agent": "garden-get-parts"}
    if start:
        headers["Range"] = f"bytes={start}-"
    if token:                            # a private repository: the file itself, with the token in a header (never in the address)
        headers["Authorization"] = "Bearer " + token
        headers["Accept"] = "application/vnd.github.raw"
        r = urllib.request.build_opener(_KeepTokenHome).open(urllib.request.Request(url, headers=headers), timeout=TIMEOUT)
    else:
        r = urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=TIMEOUT)
    if "text/html" in (r.headers.get("Content-Type") or "").lower():
        r.close()
        raise HostWrong("it answered with a web page, not the file")
    if start and r.status == 206:
        if not (r.headers.get("Content-Range") or "").startswith(f"bytes {start}-"):
            r.close()
            raise HostWrong("it answered a different range")
        return r, start
    return r, 0                      # 200: the host ignores Range, so the whole part comes again


class Progress:
    """One shared progress line for all connections."""

    def __init__(self, label, total, nparts):
        self.label, self.total, self.nparts = label, total, nparts
        self.lock = threading.Lock()
        self.done_bytes, self.inflight, self.parts_done = 0, {}, 0
        self.by_host = {}
        self.last, self.tty = 0.0, sys.stdout.isatty()

    def add_inflight(self, name, n):
        with self.lock:
            self.inflight[name] = self.inflight.get(name, 0) + n
        self.show()

    def reset(self, name):
        with self.lock:
            self.inflight.pop(name, None)

    def finished(self, name, size, host):
        with self.lock:
            self.inflight.pop(name, None)
            self.done_bytes += size
            self.parts_done += 1
            self.by_host[host] = self.by_host.get(host, 0) + 1
        self.show(force=True)

    def show(self, force=False):
        now = time.time()
        with self.lock:
            if not force and now - self.last < (0.5 if self.tty else 5):
                return
            self.last = now
            got = (self.done_bytes + sum(self.inflight.values())) / 1e6
            line = (f"   {self.label} {self.parts_done}/{self.nparts} parts, {got:.0f} of {self.total / 1e6:.0f} MB"
                    f" / {self.label_ko} {self.parts_done}/{self.nparts} 조각, {got:.0f}/{self.total / 1e6:.0f} MB")
            print("\r" + line if self.tty else line, end="" if self.tty else "\n", flush=True)

    label_ko = ""

    def end(self):
        if self.tty:
            print()


class Queue:
    def __init__(self, parts, usable):
        self.lock = threading.Lock()
        # pinned/priority parts first (lowest priority number first), then the list's order
        self.parts = sorted(parts, key=lambda p: (0, p.priority) if p.priority > 0 else (1, 0))
        self.usable = usable            # {host id}
        self.state = {p.name: "todo" for p in parts}
        self.tried = {p.name: set() for p in parts}
        self.fails = {}                 # host -> failures in a row
        self.dead = set()
        self.notes = []
        self.pfails = {}
        self.first_try = {}             # part -> when its preferred host first took it
        self.opened = set()             # preferred parts the other hosts may now take
        self.opened_now = ""

    def is_open(self, p):
        """May hosts other than the preferred one take this part? (call with the lock held)"""
        if not p.prefer or p.name in self.opened:
            return True
        return p.prefer not in self.usable or p.prefer in self.dead

    def mark_done(self, part):
        with self.lock:
            self.state[part.name] = "done"

    def claim(self, host):
        """A part for this host; "wait" if only parts others hold right now are left; None if finished."""
        with self.lock:
            waiting = False
            for p in self.parts:
                s = self.state[p.name]
                if s == "done" or host in self.tried[p.name] or host not in {h for h, _ in p.holders}:
                    continue
                if host in self.dead and not p.pinned:
                    continue            # a left-out host only keeps trying the part that nobody else may give
                if p.prefer and host != p.prefer and not self.is_open(p):
                    waiting = True      # the preferred host is trying first
                    continue
                if s == "todo":
                    self.state[p.name] = "claimed"
                    if host == p.prefer:
                        self.first_try.setdefault(p.name, time.time())
                    return p
                waiting = True
            return "wait" if waiting else None

    def failed(self, host, part, why, retry=False):
        """The part goes back in the list. True if this host is out for this run."""
        with self.lock:
            self.state[part.name] = "todo"
            if not retry:
                self.tried[part.name].add(host)
            if part.pinned:
                self.tried[part.name].discard(host)         # no one else may give it: this host just tries again
            self.fails[host] = self.fails.get(host, 0) + 1
            self.notes.append(f"{host}: {part.name}: {why}")
            if part.prefer == host and part.name not in self.opened:
                self.pfails[part.name] = self.pfails.get(part.name, 0) + 1
                if (self.pfails[part.name] >= MAX_HOST_FAILS or host in self.tried[part.name]
                        or time.time() - self.first_try.get(part.name, time.time()) >= PREFER_TIMEOUT):
                    self.opened.add(part.name)       # the preferred host did not answer: the others may take it
                    self.opened_now = part.name
            if self.fails[host] >= MAX_HOST_FAILS and not part.pinned:
                self.dead.add(host)
                return True
            return False

    def succeeded(self, host):
        with self.lock:
            self.fails[host] = 0


def fetch_one(url, part, cache, progress, token=""):
    """Fetch one part into cache/<name>, resuming a cut-off one. Raises HostWrong."""
    try:
        _fetch_one(url, part, cache, progress, token)
    except HostWrong as e:
        if not e.retry and os.path.isfile(os.path.join(cache, part.name) + ".part"):
            os.remove(os.path.join(cache, part.name) + ".part")      # wrong bytes: start that part over elsewhere
        raise


def _fetch_one(url, part, cache, progress, token=""):
    final = os.path.join(cache, part.name)
    partial = final + ".part"
    start = os.path.getsize(partial) if os.path.isfile(partial) else 0
    if start >= part.size:
        os.remove(partial)
        start = 0
    try:
        resp, offset = open_url(url, start, token)
    except HostWrong:
        raise
    except Exception as e:      # only the kind of error and the HTTP code are kept: never the address or the headers
        raise HostWrong(f"{e.__class__.__name__}" + (f" HTTP {e.code}" if hasattr(e, "code") else ""), retry=not hasattr(e, "code")) from None
    try:
        mode = "ab" if offset else "wb"
        progress.reset(part.name)
        progress.add_inflight(part.name, offset)
        got = offset
        with open(partial, mode) as out:
            while got < part.size:
                try:
                    block = resp.read(min(CHUNK, part.size - got))
                except Exception as e:
                    raise HostWrong(f"cut off ({e.__class__.__name__})", retry=True) from None
                if not block:
                    break
                out.write(block)
                got += len(block)
                progress.add_inflight(part.name, len(block))
        if got == part.size and resp.read(1):
            os.remove(partial)
            raise HostWrong("the part is too long (not the file in the list)")
    finally:
        resp.close()
    if got < part.size:
        raise HostWrong("cut off (the part is kept and carries on)", retry=True)
    if sha_of_file(partial) != part.sha:
        os.remove(partial)
        raise HostWrong("the part's fingerprint does not match")
    os.replace(partial, final)


def fetch_parts(pset, hosts, cache, log_label=None, label_ko="", token=""):
    """Get every part of pset into cache/ (each checked). Returns True when all are in."""
    os.makedirs(cache, exist_ok=True)
    todo = []
    for p in pset.parts:                 # parts that are already in and right are never fetched again
        f = os.path.join(cache, p.name)
        if os.path.isfile(f) and os.path.getsize(f) == p.size and sha_of_file(f) == p.sha:
            continue
        if os.path.isfile(f):
            os.remove(f)
        todo.append(p)
    progress = Progress(log_label or pset.name, pset.total, len(pset.parts))
    progress.label_ko = label_ko or progress.label
    progress.done_bytes = pset.total - sum(p.size for p in todo)
    progress.parts_done = len(pset.parts) - len(todo)
    if not todo:
        return True
    if not token and any(is_private(h) for h in hosts):     # no access file: the private backup host is left out
        hosts = {k: v for k, v in hosts.items() if not is_private(k)}
        if any(is_private(h) for p in todo for h, _ in p.holders):
            say("The backup host needs the access file photos-access.token (it comes in your private data package) and it is not here, so the backup is skipped.",
                "백업 호스트는 접근 파일 photos-access.token이 필요해요(개인 데이터 꾸러미에 들어 있어요). 파일이 없어서 백업은 건너뛰어요.")
    usable = {h for p in todo for h, fid in p.holders if url_for(hosts, h, pset.tag, p.name, fid)}
    q = Queue(pset.parts, usable)
    for p in pset.parts:
        if p not in todo:
            q.state[p.name] = "done"
        else:       # a host that has no address (not set up) cannot hold the part
            q.tried[p.name] |= {h for h, fid in p.holders if h not in usable or not url_for(hosts, h, pset.tag, p.name, fid)}

    announced = set()

    def worker(host):
        pinned_since, pinned_tries = 0.0, 0
        while True:
            p = q.claim(host)
            if p is None:
                return
            if p == "wait":
                time.sleep(0.05)
                continue
            fid = dict(p.holders)[host]
            try:
                fetch_one(url_for(hosts, host, pset.tag, p.name, fid), p, cache, progress, token if is_private(host) else "")
            except HostWrong as e:
                progress.reset(p.name)
                if is_private(host) and any(f"HTTP {c}" in str(e) for c in (401, 403, 404)) and host not in announced:
                    announced.add(host)
                    say("The backup did not accept the access (the token may have expired or been removed), so it cannot be used.",
                        "백업이 접근을 받아 주지 않았어요(토큰이 만료됐거나 지워졌을 수 있어요). 그래서 백업은 쓸 수 없어요.")
                q.failed(host, p, str(e), e.retry)
                if q.opened_now == p.name and p.prefer == host:
                    q.opened_now = ""
                    other = next((h for h, _ in p.holders if h != host and h in usable), host)
                    say(f"{HOST_NAMES.get(host, host)} did not answer, using {HOST_NAMES.get(other, other)} (part {p.number} of {len(pset.parts)}).",
                        f"{HOST_NAMES_KO.get(host, host)} 응답이 없어요. 대신 {HOST_NAMES_KO.get(other, other)}에서 받아요 (조각 {p.number}/{len(pset.parts)}).")
                if p.pinned:               # nobody else may give this part: wait for this host, with backoff
                    pinned_since = pinned_since or time.time()
                    name = HOST_NAMES.get(host, host)
                    say(f"Still waiting for {name} (part {p.number} of {len(pset.parts)}); the other parts carry on.",
                        f"{HOST_NAMES_KO.get(host, host)} 응답을 아직 기다리고 있어요 (조각 {p.number}/{len(pset.parts)}). 나머지 조각은 계속 받아요.")
                    if time.time() - pinned_since > PINNED_PATIENCE:
                        return
                    time.sleep(min(RETRY_WAIT * (2 ** pinned_tries), 30))
                    pinned_tries += 1
                    continue
                time.sleep(RETRY_WAIT)
                continue
            except OSError as e:           # the disk, not the host
                progress.reset(p.name)
                q.failed(host, p, f"disk: {e.__class__.__name__}")
                continue
            q.mark_done(p)
            q.succeeded(host)
            if p.pinned:
                pinned_since, pinned_tries = 0.0, 0
            progress.finished(p.name, p.size, host)

    threads = [threading.Thread(target=worker, args=(h,), daemon=True) for h in sorted(usable)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    progress.end()
    ok = all(s == "done" for s in q.state.values())
    pset.report = {"by_host": dict(progress.by_host), "notes": q.notes, "dead": sorted(q.dead)}
    if progress.by_host:
        say("Parts per host: " + ", ".join(f"{h} {n}" for h, n in sorted(progress.by_host.items())),
            "호스트별 조각 수: " + ", ".join(f"{h} {n}" for h, n in sorted(progress.by_host.items())))
    for h in sorted(q.dead):
        say(f"The host \"{h}\" kept failing, so its parts went to the others.",
            f"\"{h}\" 호스트가 계속 실패해서 그 조각은 다른 곳에서 받았어요.")
    return ok


# ---------------------------------------------------------------- "once per version" record

def record_file(dest, setname):
    return os.path.join(dest, STATE_DIR, setname + ".json")


def read_record(dest, setname):
    try:
        return json.load(open(record_file(dest, setname), encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def write_record(dest, setname, version, **extra):
    os.makedirs(os.path.join(dest, STATE_DIR), exist_ok=True)
    tmp = record_file(dest, setname) + ".tmp"
    with open(tmp, "w", encoding="utf-8", newline="\n") as f:
        json.dump({"version": version, "time": time.strftime("%Y-%m-%d %H:%M:%S"), **extra}, f)
    os.replace(tmp, record_file(dest, setname))


# ---------------------------------------------------------------- the photos

def read_key(path):
    """(key file ok?, key id) without ever showing the key. Line 1 is the key, line 2 is id=<8 hex>."""
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError:
        return False, ""
    if len(lines) < 1 or len(lines[0].strip()) < 32:
        return False, ""
    kid = lines[1].strip()[3:] if len(lines) > 1 and lines[1].startswith("id=") else ""
    return True, kid


def decrypt_to(parts_in_order, cache, enc, keyfile, tmp_out):
    """Stream the parts through openssl into tmp_out. Returns (ok, message)."""
    cmd = ["openssl", "enc", "-d", f"-{enc.get('cipher', 'aes-256-ctr')}", "-pbkdf2", "-md", "sha256",
           "-iter", str(enc.get("iter", 10000)), "-pass", "file:" + keyfile]
    try:
        with open(tmp_out, "wb") as out:
            proc = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=out, stderr=subprocess.PIPE)
            err = []
            t = threading.Thread(target=lambda: err.append(proc.stderr.read()), daemon=True)
            t.start()
            try:
                for p in parts_in_order:
                    with open(os.path.join(cache, p.name), "rb") as f:
                        for block in iter(lambda: f.read(1 << 20), b""):
                            proc.stdin.write(block)
                proc.stdin.close()
            except BrokenPipeError:
                pass
            rc = proc.wait()
            t.join()
    except FileNotFoundError:
        return False, "openssl"
    return rc == 0, ("" if rc == 0 else "openssl stopped")


def remove_token(path):
    """Overwrite the access file with zeros, then delete it (if the overwrite fails, just delete it)."""
    if not os.path.isfile(path):
        return False
    try:
        size = os.path.getsize(path)
        with open(path, "r+b") as f:
            f.write(bytes(size))
            f.flush()
            os.fsync(f.fileno())
    except OSError:
        pass
    try:
        os.remove(path)
    except OSError:
        return False
    return True


def photos_main(opts):
    dest = opts.get("--dest") or HERE
    manifest = opts.get("--manifest") or os.path.join(HERE, "PHOTO-PARTS.json")
    try:
        pset = load_manifest(manifest)
    except Exception:
        say("PHOTO-PARTS.json is damaged or missing. Nothing was written.",
            "PHOTO-PARTS.json 파일이 손상됐거나 없어요. 아무것도 쓰지 않았어요.")
        return 1
    out = os.path.join(dest, pset.outfile)
    again = "--again" in opts
    rec = read_record(dest, pset.name)
    current = (not again and rec.get("version") == pset.version and os.path.isfile(out)
               and os.path.getsize(out) == pset.size)
    if "--is-current" in opts:
        return 0 if current else 2
    if current:
        say("The photos download is already up to date; nothing was fetched.",
            "사진 파일은 이미 최신이에요. 아무것도 받지 않았어요.")
        return 0

    # The key first: without it nothing is downloaded or written.
    keyfile = opts.get("--key") or os.path.join(HERE, "photos.key")
    have, kid = read_key(keyfile)
    key_id = (pset.enc or {}).get("key_id", "")
    if not have:
        say("The photos cannot be unlocked: the key file (photos.key) is missing. Nothing was downloaded or written.",
            "사진을 열 수 없어요. 열쇠 파일(photos.key)이 없어요. 아무것도 받거나 쓰지 않았어요.")
        say("The key is in your private data package (junsgarden-for-jun.zip, on your Google Drive): put photos.key in the garden folder, then use menu 6, then 11.",
            "열쇠는 개인 데이터 꾸러미(구글 드라이브의 junsgarden-for-jun.zip)에 있어요. photos.key를 정원 폴더에 두고 메뉴 6, 그다음 11을 쓰세요.")
        say("The garden works without photos until then.", "그때까지 사진 없이도 정원은 잘 돌아가요.")
        return 4
    if key_id and kid != key_id:
        say("This key file (photos.key) is not the one for this photos download. Nothing was downloaded or written.",
            "이 열쇠 파일(photos.key)은 이 사진 파일의 열쇠가 아니에요. 아무것도 받거나 쓰지 않았어요.")
        say("Use the photos.key from your private data package (junsgarden-for-jun.zip).",
            "개인 데이터 꾸러미(junsgarden-for-jun.zip)의 photos.key를 쓰세요.")
        return 4
    if shutil.which("openssl") is None:
        say("The program openssl is missing, which unlocks the photos. On Ubuntu: sudo apt install openssl. Nothing was written.",
            "사진을 여는 openssl 프로그램이 없어요. Ubuntu에서: sudo apt install openssl. 아무것도 쓰지 않았어요.")
        return 5

    hosts = load_hosts(opts.get("--hosts") or os.path.join(HERE, "DOWNLOAD-HOSTS.txt"), opts.get("--host", []))
    cache = os.path.join(dest, STATE_DIR, pset.name)
    token = ""                      # the access file for the private backup host; never printed, never copied anywhere
    token_path = opts.get("--token") or os.path.join(HERE, TOKEN_FILE)
    try:
        token = open(token_path, encoding="utf-8").readline().strip()
    except OSError:
        pass
    mb = pset.total // 1000000
    say(f"The photos are arriving: {len(pset.parts)} parts, about {mb} MB, from {len(hosts)} host(s) at once.",
        f"사진을 받고 있어요: 조각 {len(pset.parts)}개, 약 {mb} MB, 호스트 {len(hosts)}곳에서 동시에.")
    if not fetch_parts(pset, hosts, cache, "Photos", "사진", token):
        left = [p for p in pset.parts if not (os.path.isfile(os.path.join(cache, p.name)))]
        if left and all(p.prefer for p in left):
            h = left[0].prefer
            say(f"Everything is in except part {left[0].number} of {len(pset.parts)}: {HOST_NAMES.get(h, h)} did not answer and the backup is not available. "
                "Nothing was installed. The parts already in are kept; menu 6, then 11 (or the next install run) tries again.",
                f"조각 {left[0].number}/{len(pset.parts)}만 빼고 모두 받았어요. {HOST_NAMES_KO.get(h, h)} 응답이 없고 백업도 쓸 수 없어요. "
                "아무것도 설치하지 않았어요. 받은 조각은 그대로 두었어요. 메뉴 6, 그다음 11(또는 다음 설치)로 다시 해 볼 수 있어요.")
            return 1
        if left and all(p.pinned for p in left):
            h = left[0].pinned
            say(f"Everything is in except part {left[0].number} of {len(pset.parts)}: still waiting for {HOST_NAMES.get(h, h)}. "
                "Nothing was installed. The parts already in are kept; menu 6, then 11 (or the next install run) completes it.",
                f"조각 {left[0].number}/{len(pset.parts)}만 빼고 모두 받았어요. {HOST_NAMES_KO.get(h, h)} 응답을 아직 기다리고 있어요. "
                "아무것도 설치하지 않았어요. 받은 조각은 그대로 두었어요. 메뉴 6, 그다음 11(또는 다음 설치)로 마저 받을 수 있어요.")
            return 1
        say("Could not get every part of the photos (no host could give some). Nothing was written. "
            "The garden works without photos; menu 6, then 11 tries again and carries on where it stopped.",
            "사진 조각을 모두 받지 못했어요(일부는 어느 호스트도 주지 못했어요). 아무것도 쓰지 않았어요. "
            "사진 없이도 정원은 잘 돌아가요. 메뉴 6, 그다음 11로 다시 하면 멈춘 곳에서 이어서 받아요.")
        return 1

    say("All parts are in and checked. Unlocking the photos...", "모든 조각을 받고 확인했어요. 사진을 열고 있어요...")
    os.makedirs(dest, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=".photos-", suffix=".partial", dir=dest)
    os.close(fd)
    try:
        ok, why = decrypt_to(pset.parts, cache, pset.enc or {}, keyfile, tmp)
        good = ok and os.path.getsize(tmp) == pset.size and sha_of_file(tmp) == pset.sha
        if not good:
            say("The photos did not unlock correctly (the key is wrong, or a part is damaged). Nothing was installed. "
                "The downloaded parts are kept; put the right photos.key in the garden folder and use menu 6, then 11.",
                "사진이 제대로 열리지 않았어요(열쇠가 틀렸거나 조각이 손상됐어요). 아무것도 설치하지 않았어요. "
                "받은 조각은 그대로 두었어요. 올바른 photos.key를 정원 폴더에 두고 메뉴 6, 그다음 11을 쓰세요.")
            return 4 if why != "openssl" else 5
        os.chmod(tmp, 0o644)
        os.replace(tmp, out)
        tmp = ""
    finally:
        if tmp and os.path.exists(tmp):
            os.remove(tmp)
    write_record(dest, pset.name, pset.version, file=pset.outfile, size=pset.size, parts=len(pset.parts))
    shutil.rmtree(cache, ignore_errors=True)
    if remove_token(token_path):    # the whole set is in and checked: the access file is not needed any more
        say("The temporary access file was removed.", "임시 접근 파일을 지웠어요.")
    say(f"Done: {pset.outfile} ({pset.size // 1000000} MB), every part and the whole file checked.",
        f"끝: {pset.outfile} ({pset.size // 1000000} MB), 모든 조각과 전체 파일을 확인했어요.")
    return 0


def main(argv):
    try:
        sys.stdout.reconfigure(errors="replace")
    except AttributeError:
        pass
    args, opts = [], {}
    it = iter(argv)
    for a in it:
        if a in ("--dest", "--key", "--manifest", "--hosts", "--token"):
            opts[a] = next(it, "")
        elif a == "--host":
            opts.setdefault("--host", []).append(next(it, ""))
        elif a in ("--again", "--is-current"):
            opts[a] = True
        else:
            args.append(a)
    if args[:1] == ["photos"]:
        return photos_main(opts)
    print(__doc__)
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
