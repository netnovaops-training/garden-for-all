"""Honest accuracy check, using only what is in the index (no new photos needed).

Every photo of a CURRENT plant is a test photo. It is searched against all other photos (the
other current photos AND every future-plant photo as distractors). A hit is the right plant in
the top 1 / top 5 plants. Two protocols:

* leave-one-photo-out: remove only the test photo. Optimistic: the garden owner often takes
  several shots in a row, and a near-identical neighbour makes it easy.
* leave-one-visit-out: remove every photo of that plant taken within 30 minutes of the test
  photo (file-name time stamp, else file time). The test photo must be matched from photos of
  the same plant taken at another time: closer to a phone photo taken next week.

Only plants with a photo left to match against count: a plant with a single photo has nothing
to be matched with, so it is not tested by either protocol.
"""
import re

import numpy as np

from .finder import Index
from .store import Store

GAP = 30 * 60  # seconds between two visits
_TS = re.compile(r"_(1[5-9]\d{8})(?=\.|_)")


def _stamp(rel, mtime_ns):
    m = _TS.search(rel)
    return int(m.group(1)) if m else mtime_ns // 10**9


def run(out_dir, write_temperature=False):
    store = Store(out_dir)
    rows = store.db.execute("SELECT set_, plant_id, rel, mtime_ns, emb FROM photos WHERE ok=1 "
                            "ORDER BY set_, plant_id, rel").fetchall()
    if not rows:
        return {"error": "the index is empty"}
    dim = len(rows[0][4]) // 2
    V = np.frombuffer(b"".join(r[4] for r in rows), np.float16).reshape(len(rows), dim)
    V = V.astype(np.float32)
    grp = np.array([f"{r[0]}:{r[1]}" for r in rows])
    ug, gi = np.unique(grp, return_inverse=True)
    order = np.argsort(gi, kind="stable")
    V, gi = V[order], gi[order]
    rows = [rows[i] for i in order]
    ts = np.array([_stamp(r[2], r[3]) for r in rows])
    starts = np.concatenate([[0], np.flatnonzero(np.diff(gi)) + 1])
    cur = np.array([r[0] == "current" for r in rows])
    out = {"photos": len(rows), "current_test_photos": int(cur.sum())}
    names = store.plants()
    genus, species = [], []  # per plant group: first word / first two words of the botanical name
    for g in ug:
        s, pid = g.split(":")
        b = names.get((s, int(pid)), {}).get("botanical", "").lower().split()
        genus.append(b[0] if b else "?")
        species.append(" ".join(b[:2]) if len(b) > 1 else "?")
    genus, species = np.array(genus), np.array(species)
    sims_all = {}
    for name in ("one_photo", "one_visit"):
        hits1 = hits5 = n = g1 = s1 = 0
        per_plant = {}
        nll_inputs = []
        for q in np.flatnonzero(cur):
            same = gi == gi[q]
            drop = same & (np.arange(len(gi)) == q) if name == "one_photo" else \
                same & (np.abs(ts - ts[q]) <= GAP)
            if same.sum() - drop.sum() < 1:
                continue
            s = V @ V[q]
            s[drop] = -2.0
            best = np.maximum.reduceat(s, starts)
            true = gi[q]
            rank = int((best > best[true]).sum())  # 0 = best
            n += 1
            hits1 += rank == 0
            hits5 += rank < 5
            top = int(np.argmax(best))
            g1 += genus[top] == genus[true] != "?"
            s1 += species[top] == species[true] != "?"
            per_plant.setdefault(true, []).append((rank == 0, rank < 5))
            nll_inputs.append((best, true))
        mac = np.array([np.mean(v, axis=0) for v in per_plant.values()])
        out[name] = {"test_photos": n, "plants": len(per_plant),
                     "top1": round(hits1 / max(n, 1), 3), "top5": round(hits5 / max(n, 1), 3),
                     "top1_same_genus": round(g1 / max(n, 1), 3),
                     "top1_same_species": round(s1 / max(n, 1), 3),
                     "per_plant_top1": round(float(mac[:, 0].mean()), 3) if n else 0,
                     "per_plant_top5": round(float(mac[:, 1].mean()), 3) if n else 0}
        sims_all[name] = nll_inputs
    out["temperature"] = _fit_temperature(sims_all["one_visit"] or sims_all["one_photo"],
                                          store, write_temperature)
    out["hints"] = _hint_test(out_dir)
    return out


def _fit_temperature(items, store, write):
    """Pick the softmax temperature that makes `confidence` honest on the visit-out test."""
    best_t, best_nll = None, 1e9
    for t in (0.005, 0.01, 0.015, 0.02, 0.03, 0.04, 0.05, 0.07, 0.1):
        nll = 0.0
        for best, true in items:
            z = (best - best.max()) / t
            nll -= z[true] - np.log(np.exp(z).sum())
        nll /= max(len(items), 1)
        if nll < best_nll:
            best_t, best_nll = t, nll
    if write and best_t:
        store.set_meta("temperature", best_t)
        store.bump()
        store.commit()
    return best_t


def _hint_test(out_dir, n_max=300):
    """Does the text hook help with the right tag text (the botanical name with its last letter
    missing, like a bad OCR read), and how much does a wrong one (another plant's name) hurt?
    Photos: the first photo of up to 300 current plants that have another photo."""
    idx = Index(out_dir)
    if not len(idx.keys):
        return {}
    rng = np.random.default_rng(0)
    cur = [i for i, (s, _) in enumerate(idx.keys) if s == "current" and idx.info[i]["botanical"]]
    cur = rng.permutation(cur)[:n_max]
    res = {"no_hint_top1": 0, "no_hint_top5": 0, "right_hint_top1": 0,
           "wrong_hint_top5": 0, "n": len(cur)}
    for gi in cur:
        lo = idx.starts[gi]
        hi = idx.starts[gi + 1] if gi + 1 < len(idx.starts) else len(idx.mat)
        q = idx.mat[lo]
        # the test photo is the plant's own first photo: take the best OTHER plant as competition
        # by hiding it (so this tests only the re-ranking, not the image match).
        saved = idx.mat[lo].copy()
        idx.mat[lo] = -q / max(np.linalg.norm(q), 1e-9)
        if hi - lo == 1:  # only photo: its plant has no look-alike left, skip
            idx.mat[lo] = saved
            res["n"] -= 1
            continue
        pid = idx.keys[gi][1]
        name = idx.info[gi]["botanical"]
        typo = name[:-1] if len(name) > 6 else name
        def hit(found, upto):
            return any((x["set"], x["plant_id"]) == ("current", pid) for x in found[:upto])

        base = idx.search_vector(saved, 5)
        res["no_hint_top1"] += hit(base, 1)
        res["no_hint_top5"] += hit(base, 5)
        res["right_hint_top1"] += hit(idx.search_vector(saved, 5, hints=typo), 1)
        other = idx.info[int(rng.integers(len(idx.keys)))]["botanical"]
        res["wrong_hint_top5"] += hit(idx.search_vector(saved, 5, hints=other), 5)
        idx.mat[lo] = saved
    n = max(res["n"], 1)
    for k in ("no_hint_top1", "no_hint_top5", "right_hint_top1", "wrong_hint_top5"):
        res[k] = round(res[k] / n, 3)
    return res
