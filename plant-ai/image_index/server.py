"""A tiny HTTP service so other programs on this computer can ask the index.

    POST /search?k=5&hints=text   body = the picture's bytes   ->  {"results": [...], "ms": 12}
    GET  /status                                               ->  counts, model, last build

No password: it only answers this computer. The bind address must be a loopback address
(127.0.0.1 or ::1). One exception: inside Docker the program must listen on 0.0.0.0 so the
published port can reach it; set IMAGE_INDEX_IN_DOCKER=1 there AND publish only as
`-p 127.0.0.1:8099:8099` (README). As a second guard it refuses a request whose Host is not a
local name (DNS rebinding) or that carries an Origin header (a web page in a browser).
"""
import ipaddress
import json
import os
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

from . import finder

MAX_BODY = 25 * 1024 * 1024
LOCAL_HOSTS = {"localhost", "127.0.0.1", "[::1]", "::1"}


def _host_ok(header):
    """Host header names this computer, or a name in IMAGE_INDEX_ALLOWED_HOSTS (a container
    reached by its Docker network name, e.g. plant-image-index)."""
    extra = {h.strip().lower() for h in os.environ.get("IMAGE_INDEX_ALLOWED_HOSTS", "").split(",")}
    host = (header or "").strip().lower()
    if host.startswith("["):
        host = host.split("]")[0] + "]"
    else:
        host = host.split(":")[0]
    return host in LOCAL_HOSTS or (host in extra and host != "")


class Handler(BaseHTTPRequestHandler):
    out_dir = None
    server_version = "image-index"

    def _send(self, code, obj):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _guard(self):
        if not _host_ok(self.headers.get("Host")):
            self._send(403, {"error": "local requests only (Host)"})
            return False
        if self.headers.get("Origin"):
            self._send(403, {"error": "browser requests are refused"})
            return False
        return True

    def do_GET(self):
        if not self._guard():
            return
        if urlparse(self.path).path == "/status":
            self._send(200, finder.status(self.out_dir))
        else:
            self._send(404, {"error": "use GET /status or POST /search"})

    def do_POST(self):
        if not self._guard():
            return
        url = urlparse(self.path)
        if url.path != "/search":
            return self._send(404, {"error": "use GET /status or POST /search"})
        q = parse_qs(url.query)
        try:
            k = max(1, min(50, int(q.get("k", ["5"])[0])))
            n = int(self.headers.get("Content-Length") or 0)
        except ValueError:
            return self._send(400, {"error": "k and Content-Length must be numbers"})
        if not 0 < n <= MAX_BODY:
            return self._send(413 if n else 400,
                              {"error": f"send the picture bytes (up to {MAX_BODY >> 20} MB)"})
        body = self.rfile.read(n)
        t0 = time.time()
        try:
            res = finder.search(body, k, out_dir=self.out_dir, hints=q.get("hints") or None)
        except ValueError as e:  # not a picture
            return self._send(400, {"error": str(e)})
        except Exception as e:  # noqa: BLE001  keep serving; say what failed
            return self._send(500, {"error": type(e).__name__})
        self._send(200, {"results": res, "ms": round((time.time() - t0) * 1000)})

    def log_message(self, fmt, *args):  # one short line per request, no picture data
        sys.stderr.write(f"{self.command} {urlparse(self.path).path}\n")


def serve(bind, port, out_dir):
    ip = ipaddress.ip_address(bind.strip("[]"))
    in_docker = os.environ.get("IMAGE_INDEX_IN_DOCKER") == "1"
    # not a bind to everything: this REFUSES everything but loopback (and 0.0.0.0 in Docker)
    if not ip.is_loopback and not (in_docker and str(ip) == "0.0.0.0"):  # nosec B104
        sys.exit(f"refusing to listen on {bind}: this service is for this computer only "
                 "(127.0.0.1). Inside Docker set IMAGE_INDEX_IN_DOCKER=1 and publish "
                 "-p 127.0.0.1:PORT:PORT.")
    Handler.out_dir = out_dir
    idx = finder.get_index(out_dir)  # load vectors now
    if len(idx.keys):
        idx.warm()  # load the model now so the first search is not slow
    httpd = ThreadingHTTPServer((str(ip), port), Handler)
    print(f"image index listening on {ip}:{port} ({len(idx.keys)} plants)", flush=True)
    httpd.serve_forever()
