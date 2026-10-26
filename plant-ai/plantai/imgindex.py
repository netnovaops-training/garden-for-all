"""Client for the plant image index (a separate helper service on this PC, 127.0.0.1:8099).

  GET  /status                      is it running and ready
  POST /search   (picture bytes)    a JSON list of {plant_id, plant_name, botanical_name, score, photo, set}

It is optional: when it is not running, or answers anything unexpected, every call here gives
"nothing" and the scan queue goes on without photo matching (a note says so). It only ever talks to
this computer.
"""
import os

import httpx

BASE = os.environ.get("PLANTAI_IMAGE_INDEX", "http://127.0.0.1:8099").rstrip("/")


def status(client: httpx.Client | None = None) -> dict | None:
    """The index's /status answer when it is running and ready, else None."""
    try:
        r = (client or httpx).get(f"{BASE}/status", timeout=3)
        if r.status_code != 200:
            return None
        j = r.json()
    except (httpx.HTTPError, ValueError):
        return None
    if not isinstance(j, dict) or j.get("ready") is False:
        return None
    return j


def search(img: bytes, client: httpx.Client | None = None, k: int = 5, hints: str = "") -> list[dict]:
    """The best matches for one picture, best first: [{plant_id, plant_name, botanical_name, score,
    photo, set}]. `hints` is optional text (a name read from a tag) the index may use to re-rank.
    [] when the index is not there or the picture gives nothing."""
    params = {"k": k, **({"hints": hints} if hints.strip() else {})}
    try:
        r = (client or httpx).post(f"{BASE}/search", params=params, content=img,
                                   headers={"Content-Type": "image/jpeg"}, timeout=60)
        if r.status_code != 200:
            return []
        j = r.json()
    except (httpx.HTTPError, ValueError):
        return []
    rows = j.get("results", []) if isinstance(j, dict) else j
    out = []
    for x in rows if isinstance(rows, list) else []:
        if isinstance(x, dict) and isinstance(x.get("score"), (int, float)):
            out.append(x)
    return sorted(out, key=lambda x: -x["score"])[:k]
