"""Find the plants whose photos look most like a given photo.

How: the query photo becomes one vector; cosine similarity against every stored photo vector is a
single matrix product (50,000 photos x 384 numbers is a few milliseconds); each plant scores as
its best photo; the best plants come back, best first.

Optional `hints` (text read from a plant tag, a botanical name, anything): plants whose names
share words with the hint get a boost, so a plant whose photos look unlike the query can still win
when the tag clearly names it. Rare words weigh more than common ones ("Echinacea" more than "Red").
"""
import difflib
import math
import os
import re
import threading
import unicodedata
from pathlib import Path

import numpy as np

from . import decode
from .models import DEFAULT_MODEL, MODELS
from .store import Store

BOOST = 0.20  # most a text match can add to a cosine score
TEMPERATURE = 0.03  # default softmax temperature for `confidence`; eval can store a fitted one
FUZZY_POOL = 300  # fuzzy (typo) matching only looks at this many best-looking plants
_STOP = {"the", "and", "for", "with", "plant", "tree", "var", "ssp", "subsp", "hybrid"}


def words(text):
    """Lower-case words of 3+ letters, accents removed, a few filler words dropped."""
    text = unicodedata.normalize("NFKD", text or "").encode("ascii", "ignore").decode().lower()
    return [w for w in re.findall(r"[a-z0-9]+", text) if len(w) >= 3 and w not in _STOP]


class Index:
    """The loaded index. Reloads by itself when a build has changed it."""

    def __init__(self, out_dir):
        self.store = Store(out_dir)
        self.lock = threading.Lock()  # one model forward pass at a time
        self.dlock = threading.RLock()  # the loaded data: a search never sees a half-reload
        self.generation = None
        self.embedder = None
        self.refresh()

    # --- loading ---
    def refresh(self):
        with self.dlock:
            gen = self.store.meta("generation")
            if gen is not None and gen == self.generation:
                return
            self.model = self.store.meta("model") or DEFAULT_MODEL
            mat, rows = self.store.load_matrix()
            names = self.store.plants()
            groups = {}
            for i, (s, pid, _rel) in enumerate(rows):
                groups.setdefault((s, pid), []).append(i)
            order = [i for ids in groups.values() for i in ids]
            self.mat = mat[order] if len(order) else mat
            self.rel = [rows[i][2] for i in order]
            self.keys = list(groups)
            sizes = np.array([len(v) for v in groups.values()], dtype=np.int64)
            self.starts = np.concatenate([[0], np.cumsum(sizes)[:-1]]) if len(sizes) else sizes
            self.info = [names.get(k, {"name": "", "botanical": ""}) for k in self.keys]
            self._text_index()
            self.temperature = self.store.meta("temperature") or TEMPERATURE
            self.generation = gen

    def _text_index(self):
        """word -> plants having it, and each plant's word weights (rare words weigh more)."""
        n = max(1, len(self.keys))
        df = {}
        self.pwords = []
        for inf in self.info:
            ws = set(words(inf["botanical"]) + words(inf["name"]))
            self.pwords.append(ws)
            for w in ws:
                df[w] = df.get(w, 0) + 1
        self.idf = {w: math.log(1 + n / c) for w, c in df.items()}
        self.inv = {}
        for gi, ws in enumerate(self.pwords):
            for w in ws:
                self.inv.setdefault(w, []).append(gi)

    # --- searching ---
    def _embedder(self):
        from .embed import Embedder
        with self.lock:
            if self.embedder is None:
                self.embedder = Embedder(self.model)

    def warm(self):
        self._embedder()
        s = MODELS[self.model].size
        self.embedder.embed(np.zeros((1, s, s, 3), np.uint8))

    def embed(self, image):
        self.refresh()  # the model name can change with a rebuild
        self._embedder()
        spec = MODELS[self.model]
        arr = decode.load(image if isinstance(image, (bytes, bytearray)) else str(image),
                          spec.size, spec.mode)
        if arr is None:
            raise ValueError("could not read this as a picture")
        with self.lock:  # one forward pass at a time: the model already uses every core
            return self.embedder.embed(arr[None])[0]

    def search_vector(self, q, k=5, hints=None):
        with self.dlock:
            self.refresh()
            return self._rank(q, k, hints)

    def _rank(self, q, k, hints):
        if not len(self.keys):
            return []
        sims = self.mat @ q
        best = np.maximum.reduceat(sims, self.starts)  # best photo per plant
        scores = best.copy()
        match = np.zeros(len(best), np.float32)
        if hints:
            match = self._text_match(hints, best)
            scores = scores + BOOST * match
        top = np.argsort(-scores)[:k]
        z = np.exp((scores - scores.max()) / self.temperature)
        conf = z / z.sum()
        out = []
        for gi in top:
            lo = self.starts[gi]
            hi = self.starts[gi + 1] if gi + 1 < len(self.starts) else len(sims)
            photo = self.rel[lo + int(np.argmax(sims[lo:hi]))]
            s, pid = self.keys[gi]
            r = {"plant_id": pid, "plant_name": self.info[gi]["name"],
                 "botanical_name": self.info[gi]["botanical"],
                 "score": round(float(min(1.0, max(0.0, scores[gi]))), 4),
                 "photo": photo, "set": s, "confidence": round(float(conf[gi]), 4)}
            if hints:
                r["text_match"] = round(float(match[gi]), 3)
            out.append(r)
        return out

    def _text_match(self, hints, best):
        """Per plant 0..1: the share of its name-word weight found in the hint text."""
        hint_words = set(words(" ".join(hints) if isinstance(hints, (list, tuple)) else hints))
        match = np.zeros(len(best), np.float32)
        found = {}  # plant -> matched words
        for w in hint_words:
            for gi in self.inv.get(w, ()):
                found.setdefault(gi, set()).add(w)
        # typos (OCR): compare the unmatched hint words with the words of the best-looking plants
        for gi in np.argsort(-best)[:FUZZY_POOL]:
            got = found.get(int(gi), set())
            for pw in self.pwords[gi] - got:
                if len(pw) < 5:
                    continue
                for hw in hint_words:
                    if abs(len(hw) - len(pw)) <= 2 and hw[0] == pw[0] and \
                            difflib.SequenceMatcher(None, hw, pw).ratio() >= 0.85:
                        found.setdefault(int(gi), set()).add(pw)
                        break
        for gi, ws in found.items():
            total = sum(self.idf[w] for w in self.pwords[gi])
            if total:
                match[gi] = sum(self.idf[w] for w in ws) / total
        return match


_cache = {}


def _default_dir():
    d = os.environ.get("IMAGE_INDEX_DIR")
    return d or ("/index" if Path("/index").is_dir() else "./index")


def get_index(out_dir=None):
    out_dir = str(out_dir or _default_dir())
    if out_dir not in _cache:
        _cache[out_dir] = Index(out_dir)
    return _cache[out_dir]


def search(image, k=5, out_dir=None, hints=None):
    """image: bytes or a file path -> [{plant_id, plant_name, botanical_name, score, photo, set,
    confidence}] best first. hints: optional text (str or list) to re-rank with."""
    if isinstance(image, (str, Path)) and not Path(image).is_file():
        raise FileNotFoundError(str(image))
    idx = get_index(out_dir)
    return idx.search_vector(idx.embed(image), k=k, hints=hints)


def status(out_dir=None):
    """Counts and the last build, for people and for GET /status."""
    store = Store(out_dir or _default_dir())
    return {"model": store.meta("model"), "mode": store.meta("mode"),
            "generation": store.meta("generation"), "photos": store.counts(),
            "last_build": store.meta("last_build"), "temperature": store.meta("temperature")}
