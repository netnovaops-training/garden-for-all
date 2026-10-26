"""Ollama calls: JSON-schema `format`, temperature 0, results cached in SQLite (db.llm_cache).

Ollama runs on this computer only (127.0.0.1); nothing is sent to a cloud model.
"""
import base64
import hashlib
import json
import time

import httpx

from . import db
from .config import CFG

_client = httpx.Client(timeout=900)
STATS = {"calls": 0, "cached": 0, "secs": 0.0, "out_tokens": 0}


def __getattr__(name: str) -> str:
    """llm.FILL and llm.CHECK: the two model names, read from the settings when used."""
    if name == "FILL":
        return CFG.fill_model
    if name == "CHECK":
        return CFG.check_model
    raise AttributeError(name)


def ask(model: str, system: str, user: str, schema: dict, images: list[bytes] | None = None,
        num_ctx: int = 8192, tag: str = "") -> dict | None:
    """One chat turn; gives the parsed JSON object, or None if the model never produced valid JSON."""
    key = hashlib.sha256(json.dumps([model, system, user, schema, tag,
                                     [hashlib.sha1(i).hexdigest() for i in images or []]]).encode()).hexdigest()
    hit = db.cache_get(key)
    if hit is not None:
        STATS["cached"] += 1
        return hit
    msg = {"role": "user", "content": user}
    if images:
        msg["images"] = [base64.b64encode(i).decode() for i in images]
    body = {"model": model, "stream": False, "format": schema, "keep_alive": "30m",
            "messages": [{"role": "system", "content": system}, msg],
            "options": {"temperature": 0, "num_ctx": num_ctx, "num_predict": 2048}}
    if model.startswith("gemma"):
        body["think"] = False
    out = None
    for attempt in range(3):
        t = time.monotonic()
        try:
            r = _client.post(CFG.ollama.rstrip("/") + "/api/chat", json=body).json()
        except (httpx.HTTPError, ValueError):
            time.sleep(5 * (attempt + 1))
            continue
        STATS["calls"] += 1
        STATS["secs"] += time.monotonic() - t
        STATS["out_tokens"] += r.get("eval_count", 0)
        try:
            out = json.loads(r["message"]["content"])
            break
        except (KeyError, ValueError, TypeError):
            continue
    if out is not None:
        db.cache_put(key, out)
    return out


def ready() -> tuple[bool, str]:
    """Is Ollama answering here, and are both models pulled? (ok, plain words)."""
    try:
        tags = _client.get(CFG.ollama.rstrip("/") + "/api/tags", timeout=10).json()
    except (httpx.HTTPError, ValueError):
        return False, f"Ollama does not answer at {CFG.ollama}"
    have = {m.get("name", "") for m in tags.get("models", [])}
    have |= {n.split(":")[0] for n in have if n.endswith(":latest")}
    missing = [m for m in (CFG.fill_model, CFG.check_model) if m not in have]
    if missing:
        return False, "models not pulled yet: " + ", ".join(missing)
    return True, "Ollama and both models are ready"
