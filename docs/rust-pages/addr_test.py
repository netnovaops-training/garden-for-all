#!/usr/bin/env python3
"""The garden's own address, end to end (run by `lab.sh address`, inside a Python container on
the lab machine's network). Prints one PASS/FAIL line per check and exits 1 if any failed.

    NEW = http://127.0.0.1:18103/garden   the garden in a folder, behind Caddy (lab-proxy.Caddyfile)
    OLD = http://127.0.0.1:18104          an old address that forwards everything to NEW
"""
import json
import re
import sys
import urllib.error
import urllib.request

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

NEW, OLD = "http://127.0.0.1:18103/garden", "http://127.0.0.1:18104"
results = []


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *a, **k):
        return None


raw = urllib.request.build_opener(NoRedirect)
follow = urllib.request.build_opener()


def get(url, redirects=False):
    try:
        r = (follow if redirects else raw).open(url, timeout=30)
    except urllib.error.HTTPError as e:
        r = e
    return r.status, r.headers, r.read(), getattr(r, "url", url)


def check(name, ok, detail=""):
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'} {name}" + ("" if ok else f"  ({detail})"))


# 1. The home page in a folder: every link of the site's own carries /garden.
st, h, body, _ = get(NEW + "/")
html = body.decode("utf-8", "replace")
own = re.findall(r'(?:href|src)="(/[^"/][^"]*)"', html)
bad = [u for u in own if not u.startswith("/garden/")]
check("home page answers in the folder", st == 200, st)
check("site links carry the folder", own and not bad, bad[:3])
# Every own link (page, script, stylesheet, photo) answers through the proxy.
broken = []
for u in sorted(set(own))[:40]:
    s, *_ = get("http://127.0.0.1:18103" + u, redirects=True)
    if s >= 400:
        broken.append((u, s))
check("every linked file and page answers", not broken, broken[:3])
check("login cookie is scoped to the folder", "path=/garden/" in (h.get("Set-Cookie") or ""), h.get("Set-Cookie"))

# 2. Short label paths.
st, h, *_ = get(NEW + "/q/2")
check("/q/2 goes to the plant", st in (302, 303, 307) and h.get("Location") == "/garden/catalog.php?plant=2", (st, h.get("Location")))
st, h, png, _ = get(NEW + "/qr/plant/2.png")
check("plant QR is a PNG", st == 200 and png[:8] == b"\x89PNG\r\n\x1a\n", st)

# 3. Beds: /labels gives every bed a number (logged-in page; the lab adds beds directly
# below instead), /b/<id> follows the CURRENT name after a rename.
bed_id, old_name, new_name = sys.argv[1], sys.argv[2], sys.argv[3]
st, h, *_ = get(NEW + f"/b/{bed_id}")
check("bed label goes to the renamed bed", h.get("Location", "").endswith("bed=" + urllib.request.quote(new_name).replace("%20", "+")) or
      h.get("Location", "").endswith("bed=" + new_name.replace(" ", "+")), h.get("Location"))
check("bed label is not the old name", not (h.get("Location") or "").endswith("bed=" + old_name.replace(" ", "+")), h.get("Location"))

# 4. The app's address: signed, and the signature checks with the public key.
st, h, body, _ = get(NEW + "/app/config.json")
c = json.loads(body)
check("config names the folder address", c.get("base") == NEW and c.get("api") == NEW + "/app_api.php", c.get("base"))
try:
    Ed25519PublicKey.from_public_bytes(bytes.fromhex(c["key"])).verify(
        bytes.fromhex(c["sig"]), f"junsgarden-config-v1\n{c['base']}\n{c['issued']}".encode())
    signed = True
except (ValueError, KeyError, InvalidSignature) as e:  # report, do not stop
    signed = repr(e)
check("config signature verifies", signed is True, signed)
st, h, *_ = get(NEW + "/qr/app-setup.png")
check("setup QR is a PNG", st == 200, st)

# 5. Old address: a label printed for it (old style and new style) reaches the new garden.
for path in ("/catalog.php?plant=2", "/q/2", "/app/config.json"):
    st, h, body, url = get(OLD + path, redirects=True)
    check(f"old address {path} reaches the new garden", st == 200 and url.startswith(NEW), (st, url))

print(f"\n{sum(results)}/{len(results)} passed")
sys.exit(0 if all(results) else 1)
