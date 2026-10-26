"""Polite web fetcher: robots.txt, per-host delay (>= 3 s, or the site's Crawl-delay), disk cache.

Every page or image is fetched at most once; later calls read the cache in <work>/cache/.
Thread-safe: a lock per host keeps that host's requests spaced, so different hosts can run
in parallel threads without breaking any one site's limit.
"""
import hashlib
import json
import ssl
import threading
import time
import urllib.robotparser
from pathlib import Path
from urllib.parse import urlsplit

import certifi
import httpx

from .config import CFG

UA = ("JunsGarden-research/1.0 (private garden notes; polite crawler; "
      "+https://github.com/netnovaops-training/garden-for-all)")

_client: httpx.Client | None = None
_robots: dict[str, urllib.robotparser.RobotFileParser | None] = {}
_last: dict[str, float] = {}
_locks: dict[str, threading.Lock] = {}
_glock = threading.Lock()


def client() -> httpx.Client:
    """One shared client. Normal roots plus one intermediate (GeoTrust TLS RSA CA G1) that
    missouribotanicalgarden.org forgets to send; it still has to chain to a real root."""
    global _client
    with _glock:
        if _client is None:
            ctx = ssl.create_default_context(cafile=certifi.where())
            ctx.load_verify_locations(cafile=str(Path(__file__).with_name("ca") / "extra.pem"))
            _client = httpx.Client(headers={"User-Agent": UA}, follow_redirects=True, timeout=40, verify=ctx)
    return _client


def _cache() -> Path:
    d = CFG.work / "cache"
    d.mkdir(parents=True, exist_ok=True)
    return d


def _key(url: str) -> Path:
    # A file name for the cache, not a security check.
    return _cache() / hashlib.sha1(url.encode(), usedforsecurity=False).hexdigest()


def cached(url: str) -> dict | None:
    """The cached result for `url` ({status, type, url, path}) or None."""
    m = _key(url).with_suffix(".json")
    return json.loads(m.read_text()) if m.exists() else None


def _lock(host: str) -> threading.Lock:
    with _glock:
        return _locks.setdefault(host, threading.Lock())


def _wait(host: str, delay: float) -> None:
    gap = time.monotonic() - _last.get(host, 0.0)
    if gap < delay:
        time.sleep(delay - gap)
    _last[host] = time.monotonic()


def _rules(scheme: str, host: str) -> urllib.robotparser.RobotFileParser | None:
    """robots.txt for a host (fetched once, itself rate-limited). None = no robots file."""
    if host in _robots:
        return _robots[host]
    rp = urllib.robotparser.RobotFileParser()
    _wait(host, CFG.min_delay)
    try:
        r = client().get(f"{scheme}://{host}/robots.txt")
        rp.parse(r.text.splitlines() if r.status_code == 200 else [])
    except httpx.HTTPError:
        rp.parse([])
    _robots[host] = rp
    return rp


def delay_for(host: str) -> float:
    rp = _robots.get(host)
    cd = rp.crawl_delay(UA) if rp else None
    return max(CFG.min_delay, float(cd or 0))


def get(url: str, max_bytes: int = 15_000_000) -> dict:
    """Fetch (or read from cache) one URL. Gives {status, type, url, path}; status 0 = error,
    -1 = blocked by robots.txt. The body is in `path` (bytes on disk)."""
    hit = cached(url)
    if hit:
        return hit
    p = urlsplit(url)
    if p.scheme not in ("http", "https") or not p.netloc:
        return {"status": 0, "type": "", "url": url, "path": "", "error": "not a web address"}
    host = p.netloc.lower()
    with _lock(host):
        rp = _rules(p.scheme, host)
        if rp and not rp.can_fetch(UA, url):
            res = {"status": -1, "type": "", "url": url, "path": ""}
        else:
            _wait(host, delay_for(host))
            try:
                r = client().get(url)
                body = r.content[:max_bytes]
                path = _key(url).with_suffix(".bin")
                path.write_bytes(body)
                res = {"status": r.status_code, "type": r.headers.get("content-type", ""),
                       "url": str(r.url), "path": str(path)}
            except httpx.HTTPError as e:
                res = {"status": 0, "type": "", "url": url, "path": "", "error": type(e).__name__}
    # Network errors are not cached, so a later run retries them.
    if res["status"] != 0:
        _key(url).with_suffix(".json").write_text(json.dumps(res))
    return res


def text(url: str) -> str:
    """Body of a cached/fetched URL as text ('' when not a 200)."""
    r = get(url)
    if r["status"] != 200 or not r["path"]:
        return ""
    return Path(r["path"]).read_bytes().decode("utf-8", "replace")
