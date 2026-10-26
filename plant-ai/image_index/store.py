"""The index on disk: one SQLite file (`index.db`) in the out folder.

    meta    key/value: model, dim, preprocess, generation (bumped on every change), last_build
    plants  one row per (set, plant): name, botanical name
    photos  one row per photo: where it is, which plant, size + mtime (to spot changes), and the
            unit vector as float16 bytes (NULL + ok=0 when the file could not be read)

SQLite in WAL mode: a search can run while a build is writing, and an interrupted build loses
nothing that was committed (the build commits every few hundred photos).
"""
import json
import sqlite3
import time
from pathlib import Path

import numpy as np

SCHEMA = """
CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE IF NOT EXISTS plants(set_ TEXT, plant_id INTEGER, name TEXT, botanical TEXT,
                                  PRIMARY KEY(set_, plant_id));
CREATE TABLE IF NOT EXISTS photos(key TEXT PRIMARY KEY, path TEXT, set_ TEXT, plant_id INTEGER,
                                  rel TEXT, size INTEGER, mtime_ns INTEGER, ok INTEGER, emb BLOB);
"""


class Store:
    def __init__(self, out_dir):
        self.dir = Path(out_dir)
        self.dir.mkdir(parents=True, exist_ok=True)
        self.db = sqlite3.connect(self.dir / "index.db", timeout=30, check_same_thread=False)
        self.db.execute("PRAGMA journal_mode=WAL")
        self.db.executescript(SCHEMA)

    def meta(self, key, default=None):
        row = self.db.execute("SELECT value FROM meta WHERE key=?", (key,)).fetchone()
        return json.loads(row[0]) if row else default

    def set_meta(self, key, value):
        self.db.execute("INSERT OR REPLACE INTO meta VALUES(?,?)", (key, json.dumps(value)))

    def bump(self):
        self.set_meta("generation", (self.meta("generation", 0) or 0) + 1)

    def known(self):
        """{key: (size, mtime_ns, ok, path)} for every photo already handled."""
        return {k: (s, m, ok, p) for k, s, m, ok, p in
                self.db.execute("SELECT key, size, mtime_ns, ok, path FROM photos")}

    def put_photo(self, p, size, mtime_ns, vec):
        self.db.execute(
            "INSERT OR REPLACE INTO photos VALUES(?,?,?,?,?,?,?,?,?)",
            (p.key, str(p.path), p.set, p.plant_id, p.rel, size, mtime_ns,
             0 if vec is None else 1, None if vec is None else vec.astype(np.float16).tobytes()))

    def drop(self, keys):
        self.db.executemany("DELETE FROM photos WHERE key=?", [(k,) for k in keys])

    def replace_plants(self, plants):
        self.db.execute("DELETE FROM plants")
        self.db.executemany(
            "INSERT INTO plants VALUES(?,?,?,?)",
            [(s, pid, p["name"], p["botanical"])
             for s, d in plants.items() for pid, p in d.items()])

    def retarget(self, photos):
        """The photo-to-plant link can change in the database; keep it without re-embedding."""
        self.db.executemany("UPDATE photos SET set_=?, plant_id=?, rel=?, path=? WHERE key=?",
                            [(p.set, p.plant_id, p.rel, str(p.path), p.key) for p in photos])

    def commit(self):
        self.db.commit()

    def load_matrix(self):
        """(vectors float32 (n, dim), [(set, plant_id, rel)]) of every readable photo."""
        rows = self.db.execute(
            "SELECT set_, plant_id, rel, emb FROM photos WHERE ok=1 ORDER BY key").fetchall()
        if not rows:
            return np.zeros((0, 1), np.float32), []
        dim = len(rows[0][3]) // 2  # float16 = 2 bytes
        mat = np.frombuffer(b"".join(r[3] for r in rows), dtype=np.float16).reshape(len(rows), dim)
        return mat.astype(np.float32), [r[:3] for r in rows]

    def plants(self):
        return {(s, i): {"name": n, "botanical": b} for s, i, n, b in
                self.db.execute("SELECT set_, plant_id, name, botanical FROM plants")}

    def counts(self):
        q = self.db.execute("SELECT set_, ok, COUNT(*), COUNT(DISTINCT plant_id) FROM photos "
                            "GROUP BY set_, ok").fetchall()
        out = {"current": {"photos": 0, "failed": 0, "plants": 0},
               "future": {"photos": 0, "failed": 0, "plants": 0}}
        for s, ok, n, plants in q:
            out[s]["photos" if ok else "failed"] = n
            if ok:
                out[s]["plants"] = plants
        return out

    def stamp_build(self, info):
        info["finished"] = time.strftime("%Y-%m-%d %H:%M:%S")
        self.set_meta("last_build", info)
        self.bump()
        self.commit()
