"""SQLite state for the run (<work>/state.db, WAL): plants, one row per (plant, stage), and caches.
A stage row present = that stage is done for that plant, so every stage is resumable."""
import json
import sqlite3
import threading

from .config import CFG

_local = threading.local()


def con() -> sqlite3.Connection:
    c = getattr(_local, "c", None)
    path = str(CFG.work / "state.db")
    if c is None or getattr(_local, "path", "") != path:
        c = sqlite3.connect(path, timeout=120, isolation_level=None)
        c.execute("PRAGMA journal_mode=WAL")
        c.execute("PRAGMA synchronous=NORMAL")
        c.executescript("""
            CREATE TABLE IF NOT EXISTS plants(pset TEXT, pid INTEGER, name TEXT, record TEXT,
                PRIMARY KEY(pset, pid));
            CREATE TABLE IF NOT EXISTS stage(pset TEXT, pid INTEGER, stage TEXT, data TEXT,
                PRIMARY KEY(pset, pid, stage));
            CREATE TABLE IF NOT EXISTS shared(k TEXT PRIMARY KEY, data TEXT);
            CREATE TABLE IF NOT EXISTS llm_cache(k TEXT PRIMARY KEY, out TEXT);
        """)
        _local.c, _local.path = c, path
    return c


def put(pset: str, pid: int, stage: str, data) -> None:
    con().execute("INSERT OR REPLACE INTO stage VALUES(?,?,?,?)", (pset, pid, stage, json.dumps(data)))


def get(pset: str, pid: int, stage: str):
    r = con().execute("SELECT data FROM stage WHERE pset=? AND pid=? AND stage=?", (pset, pid, stage)).fetchone()
    return json.loads(r[0]) if r else None


def done(pset: str, stage: str) -> set[int]:
    return {r[0] for r in con().execute("SELECT pid FROM stage WHERE pset=? AND stage=?", (pset, stage))}


def drop(pset: str, pid: int, stages: list[str]) -> None:
    for s in stages:
        con().execute("DELETE FROM stage WHERE pset=? AND pid=? AND stage=?", (pset, pid, s))


def shared_get(k: str):
    r = con().execute("SELECT data FROM shared WHERE k=?", (k,)).fetchone()
    return json.loads(r[0]) if r else None


def shared_put(k: str, data) -> None:
    con().execute("INSERT OR REPLACE INTO shared VALUES(?,?)", (k, json.dumps(data)))


def cache_get(k: str):
    r = con().execute("SELECT out FROM llm_cache WHERE k=?", (k,)).fetchone()
    return json.loads(r[0]) if r else None


def cache_put(k: str, out) -> None:
    con().execute("INSERT OR REPLACE INTO llm_cache VALUES(?,?)", (k, json.dumps(out)))


def plants(pset: str) -> list[tuple[int, dict]]:
    return [(r[0], json.loads(r[1])) for r in
            con().execute("SELECT pid, record FROM plants WHERE pset=? ORDER BY pid", (pset,))]


def load(pset: str, rows: list[dict], replace: bool = False) -> None:
    """Add plants to a set. replace=True refreshes a record that is already there (a queued job
    sent again after an edit); otherwise the first copy wins."""
    c = con()
    c.execute("BEGIN")
    verb = "INSERT OR REPLACE" if replace else "INSERT OR IGNORE"
    for r in rows:
        # `verb` is one of the two fixed words above; every value is a `?`.
        c.execute(f"{verb} INTO plants VALUES(?,?,?,?)",
                  (pset, int(r["id"]), r.get("plant_name") or "", json.dumps(r)))
    c.execute("COMMIT")
