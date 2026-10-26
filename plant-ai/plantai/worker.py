"""The desktop helper: asks the garden website for queued plants, does the AI work here, sends the
answers back. It only ever makes OUTGOING requests (like a browser does), so no port is opened on
this computer, and the website needs no new open port either: it answers on its usual address,
through Caddy, like every other page.

  GET  /plant-ai/jobs?take=N            the next queued plants (the site marks them "working")
  POST /plant-ai/jobs/<id>/progress     {"step": "searching" | "filling" | "photos" | "checking"}
  POST /plant-ai/jobs/<id>/result       {"result": {...}, "photos": [{"name": "1.webp", "data": base64}]}
  POST /plant-ai/jobs/<id>/fail         {"error": "short reason"}

Every request carries the helper's key:  Authorization: Worker <key>. The key is made on the
website (Add plant tab, "Desktop helper"), shown once, and stored there only as a fingerprint
(SHA-256). Here it lives in a file only this Windows user can read; it is never printed or logged.
"""
import base64
import json
import time
from pathlib import Path

import httpx

from . import batch, db, llm, result
from .config import CFG

SET = "new"


class Site:
    """The garden website, as the helper sees it."""

    def __init__(self, base: str, key: str, client: httpx.Client | None = None):
        self.base = base.rstrip("/")
        self._h = {"Authorization": f"Worker {key}", "User-Agent": "JunsGarden-plant-ai/1.0"}
        self.c = client or httpx.Client(timeout=120)

    def take(self, n: int) -> list[dict]:
        r = self.c.get(f"{self.base}/plant-ai/jobs", params={"take": n}, headers=self._h)
        r.raise_for_status()
        return r.json().get("jobs", [])

    def post(self, job: int, what: str, body: dict) -> None:
        r = self.c.post(f"{self.base}/plant-ai/jobs/{int(job)}/{what}", json=body, headers=self._h)
        r.raise_for_status()


def read_key(path: Path) -> str:
    """The key from its file: the first non-empty line, trimmed."""
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            return line.strip()
    raise ValueError(f"no key in {path}")


def payload(pid: int, rec: dict) -> dict:
    """The result for one plant plus its photos (the .webp files, base64) for the site."""
    res = result.plant_result(SET, pid, rec) or {"fields": {}, "needs_review": {}, "missing": [], "photos": [],
                                                 "decision": "Needs review", "reason": "no result"}
    photos = []
    for i, p in enumerate(res.get("photos", []), 1):
        f = CFG.out / p["file"]
        if f.is_file():
            photos.append({"name": f"{i}.webp", "data": base64.b64encode(f.read_bytes()).decode(),
                           "credit": p.get("credit") or "", "source": p.get("source") or ""})
    return {"result": res, "photos": photos}


def run_once(site: Site, take: int = 10) -> int:
    """One round: take up to `take` plants, do them all (phi4 first, then gemma), send each back.
    Gives how many plants were done."""
    ok, why = llm.ready()
    if not ok:
        batch.log(f"not ready: {why}")
        return 0
    jobs = site.take(take)
    if not jobs:
        return 0
    recs = {}
    for j in jobs:
        rec = dict(j["plant"])
        rec["id"] = int(rec["id"])
        recs[int(j["id"])] = rec
        # A plant sent again (after an edit) is worked out again from its pages (cached, so quick).
        db.drop(SET, rec["id"], ["discover", "fill", "images", "check"])
    db.load(SET, list(recs.values()), replace=True)
    by_pid = {rec["id"]: jid for jid, rec in recs.items()}

    def progress(_pset, pid, step):
        try:
            site.post(by_pid[pid], "progress", {"step": step})
        except httpx.HTTPError:
            pass   # progress is only for the screen; the result still goes back below

    batch.run([(SET, rec["id"], rec) for rec in recs.values()], progress=progress)
    done = 0
    for jid, rec in recs.items():
        try:
            site.post(jid, "result", payload(rec["id"], rec))
            done += 1
        except httpx.HTTPError as e:
            batch.log(f"FAIL sending job {jid}: {type(e).__name__}")
            try:
                site.post(jid, "fail", {"error": f"could not send the result ({type(e).__name__})"})
            except httpx.HTTPError:
                pass
    return done


def run_forever(site: Site, poll: float = 60.0, take: int = 10) -> None:
    """Ask every `poll` seconds while idle; straight away again after a busy round."""
    while True:
        try:
            n = run_once(site, take)
        except (httpx.HTTPError, ValueError, KeyError) as e:
            batch.log(f"site not reachable or odd answer: {type(e).__name__}")
            n = 0
        if n == 0:
            time.sleep(poll)


def load_settings(path: Path) -> dict:
    """worker.json: {"site": "https://...", "key_file": "...", "work": "...", "out": "..."}."""
    return json.loads(path.read_text(encoding="utf-8"))
