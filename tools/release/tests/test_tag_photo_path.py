#!/usr/bin/env python3
"""Test-only: the photo-album route of the garden-tools service, GET /api/tag-photos?file=...
(beds/service.py). A made-up tags folder in a temporary place, a made-up "outside" folder next to
it, and the real request handler run with stand-ins for the database and the other modules.
Nothing real is touched.

    python tools/release/tests/test_tag_photo_path.py

Needs: a Python that can make symbolic links (Linux, or Windows with Developer Mode); the
symbolic-link cases are skipped, and said so, when it cannot. To use the Python of the service:
    docker run --rm -v "<repo>:/r:ro" python:3.12-slim python /r/tools/release/tests/test_tag_photo_path.py
"""
import importlib.abc, importlib.machinery, io, os, shutil, sys, tempfile
from unittest import mock

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
BEDS = os.path.join(ROOT, "beds")
passed = failed = skipped = 0


def ok(name, cond, extra=""):
    global passed, failed
    if cond:
        passed += 1
        print("PASS ", name)
    else:
        failed += 1
        print("FAIL ", name, extra)


def skip(name, why):
    global skipped
    skipped += 1
    print("SKIP ", name, "-", why)


# ---- stand-ins for what service.py imports (not under test) --------------------------
class _Stub(importlib.abc.MetaPathFinder, importlib.abc.Loader):
    NAMES = {"pymysql", "dedupe", "freeai", "gemini", "tagqueue", "garden_ingest"}

    def find_spec(self, name, path=None, target=None):
        if name.split(".")[0] in self.NAMES:
            return importlib.machinery.ModuleSpec(name, self, is_package=True)
        return None

    def create_module(self, spec):
        return mock.MagicMock(__path__=[], __name__=spec.name)

    def exec_module(self, module):
        pass


work = tempfile.mkdtemp(prefix="tagphoto-test-")
photos = os.path.join(work, "photos")
tags = os.path.join(photos, "tags")
outside = os.path.join(work, "outside")
os.makedirs(os.path.join(tags, "1"))
os.makedirs(os.path.join(tags, "unassigned"))
os.makedirs(outside)
JPEG = b"\xff\xd8\xff\xe0FAKE-TAG-PHOTO"
SECRET = b"\xff\xd8\xff\xe0OUTSIDE-SECRET"
for rel in ("1/abc.jpg", "unassigned/good.JPG", "1/pic.png"):
    open(os.path.join(tags, *rel.split("/")), "wb").write(JPEG)
open(os.path.join(tags, "1", "notes.txt"), "wb").write(b"not a picture")
open(os.path.join(tags, "1", "noext"), "wb").write(b"not a picture")
open(os.path.join(outside, "secret.jpg"), "wb").write(SECRET)
open(os.path.join(outside, "passwd"), "wb").write(b"root:x:0:0")

os.environ["GARDEN_PHOTO_DIR"] = photos
os.environ["BEDS_ALLOWED_EMAILS"] = "tester@example.com"
sys.meta_path.insert(0, _Stub())
sys.path.insert(0, BEDS)
import service  # noqa: E402  (the real file)


class Req(service.Handler):
    """The real handler, fed a made-up request and writing to memory."""

    def __init__(self, target):
        self.path = target
        self.headers = {"X-Auth-Email": "tester@example.com"}
        self.wfile = io.BytesIO()
        self.status = None
        self.sent = {}

    def send_response(self, code, message=None):
        self.status = code

    def send_header(self, k, v):
        self.sent[k] = v

    def end_headers(self):
        pass


def get(file_param):
    from urllib.parse import quote
    r = Req("/api/tag-photos?file=" + quote(file_param, safe=""))
    r.do_GET()
    return r.status, r.wfile.getvalue(), r.sent


def refused(label, file_param, why=""):
    status, body, _ = get(file_param)
    ok(label, status == 404 and SECRET not in body and b"root:" not in body and b"not a picture" not in body,
       f"{why} status={status} body={body[:60]!r}")


# ---- 1. a normal tag is still served ------------------------------------------------
status, body, sent = get("1/abc.jpg")
ok("normal tag 1/abc.jpg is served as an image", status == 200 and body == JPEG and sent.get("Content-Type") == "image/jpeg", f"status={status}")
status, body, _ = get("unassigned/good.JPG")
ok("capital .JPG in the unassigned folder is served", status == 200 and body == JPEG, f"status={status}")
status, body, _ = get("1/pic.png")
ok("a .png tag is served", status == 200 and body == JPEG, f"status={status}")

# ---- 2. the old cleaning still holds ------------------------------------------------
refused("..  : ../outside/secret.jpg", "../outside/secret.jpg")
refused("..  : ../../etc/passwd", "../../etc/passwd")
refused("..  : 1/../../outside/secret.jpg", "1/../../outside/secret.jpg")
refused("absolute path: /etc/passwd", "/etc/passwd")
refused("absolute path to the outside folder", os.path.join(outside, "secret.jpg").replace("\\", "/"))
refused("only one piece: abc.jpg", "abc.jpg")
refused("three pieces: 1/2/abc.jpg", "1/2/abc.jpg")
refused("missing file: 1/missing.jpg", "1/missing.jpg")

# ---- 3. only pictures --------------------------------------------------------------
refused("a text file inside the tags folder is not served", "1/notes.txt")
refused("a file with no extension is not served", "1/noext")

# ---- 4. symbolic links that point outside the tags folder -----------------------------
try:
    os.symlink(os.path.join(outside, "secret.jpg"), os.path.join(tags, "1", "evil.jpg"))
    os.symlink(outside, os.path.join(tags, "2"), target_is_directory=True)
    os.symlink(os.path.join(outside, "passwd"), os.path.join(tags, "1", "pw.jpg"))
    can_link = True
except (OSError, NotImplementedError, AttributeError):
    can_link = False
if can_link:
    refused("a file link pointing outside (1/evil.jpg) is not followed", "1/evil.jpg")
    refused("a folder link pointing outside (2/secret.jpg) is not followed", "2/secret.jpg")
    refused("a .jpg link to a non-picture outside (1/pw.jpg) is not followed", "1/pw.jpg")
    # a link that stays inside the tags folder is harmless and still works
    os.symlink(os.path.join(tags, "1", "abc.jpg"), os.path.join(tags, "1", "alias.jpg"))
    status, body, _ = get("1/alias.jpg")
    ok("a link that stays inside the tags folder still works", status == 200 and body == JPEG, f"status={status}")
else:
    for n in ("file link", "folder link", "link to a non-picture", "link inside the folder"):
        skip("symbolic link: " + n, "this Python cannot make symbolic links here")

print(f"\n{passed} passed, {failed} failed, {skipped} skipped")
sys.exit(1 if failed else 0)
