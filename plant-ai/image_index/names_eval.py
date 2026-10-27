"""How well does a model check names?  python -m image_index eval-names

For a fixed random sample of the garden's own photos (same sample for every model: seed 0), the
photo is scored against the species names of the database and we look at:

* top-1 / top-5: is the plant's own species among the best names (zero-shot, the name is hidden);
* genus: is the best name's genus right;
* verify: the score of the TRUE name against a WRONG name for the same photo, two kinds of wrong
  (another species of the same genus when there is one: the hard case; any other species).
  auc = how often the true name scores higher; at the cut-off that lets 5 in 100 wrong names
  through, `accepts_true` of the true names still pass.

PlantCLEF only knows 7,806 south-west-European species: `covered` = share of the sample whose
species it knows; species numbers for it are over those. `covered_only` repeats every model on that
same subset so the models compare fairly.
"""
import random

import numpy as np

from . import decode
from .embed import Embedder, _snapshot
from .models import MODELS
from .store import Store
from .taxon import label, taxon
from .verify import plantclef_taxa, species_names


def _scores(emb, spec, arrays, names):
    """(labels, probability rows (n, labels)) for a list of photo arrays."""
    k = spec.kind
    rows = []
    if k == "bioclip":
        prompts = [f"a photo of {label(g, s)}" for g, s in names]
        T = np.concatenate([emb.text_embed(prompts[i:i + 256])
                            for i in range(0, len(prompts), 256)])
        scale = emb.logit_scale()
        labels = names
        for i in range(0, len(arrays), 16):
            z = scale * (emb.embed(np.stack(arrays[i:i + 16])) @ T.T)
            rows.append(z)
    else:
        labels = plantclef_taxa(emb)
        for i in range(0, len(arrays), 16):
            rows.append(emb.species_logits(np.stack(arrays[i:i + 16])))
    z = np.concatenate(rows)
    z = z - z.max(1, keepdims=True)
    p = np.exp(z)
    return labels, p / p.sum(1, keepdims=True)


def plantclef_set():
    """(genus, species) the PlantCLEF classifier knows (read from its species list)."""
    import os
    path = os.path.join(_snapshot(MODELS["plantclef-b14"]), "species_id_mapping.txt")
    with open(path, encoding="utf-8") as f:
        return {taxon(ln) for ln in f if ln.strip()}


def run(out_dir, model, limit=800, seed=0, workers=6):
    store = Store(out_dir)
    names, _genera = species_names(store)
    db_species = set(names)
    plants = store.plants()
    # the garden's own plants only: that set of files does not change between builds, so every
    # model gets exactly the same sample
    rows = store.db.execute("SELECT set_, plant_id, path, key FROM photos WHERE ok=1 "
                            "AND set_='current' ORDER BY key").fetchall()
    sample = [(r[2], taxon(plants.get((r[0], r[1]), {}).get("botanical", ""))) for r in rows]
    sample = [(p, t) for p, t in sample if t[0] and t[1]]  # species-level names only
    random.Random(seed).shuffle(sample)
    sample = sample[:limit]
    spec = MODELS[model]
    emb = Embedder(model)
    arrays, kept = [], []
    for (_path, t), (_p, arr) in zip(sample, decode.decode_many([s[0] for s in sample],
                                                              spec.size, spec.mode, workers),
                                    strict=True):
        if arr is not None:
            arrays.append(arr)
            kept.append(t)
    labels, P = _scores(emb, spec, arrays, names)
    lab_idx = {}
    for i, t in enumerate(labels):
        lab_idx.setdefault(t, []).append(i)
    # PlantCLEF species list, to say which photos are "covered" (also for BioCLIP runs)
    pc = plantclef_set()
    if spec.kind == "plantclef":  # only names the database has are valid answers for a name check
        keep = [i for i, t in enumerate(labels) if t in db_species]
        mask = np.zeros(len(labels), bool)
        mask[keep] = True
        P = np.where(mask, P, 0.0)
        P = P / P.sum(1, keepdims=True)
    rng = random.Random(seed)
    out = {"model": model, "photos": len(kept), "candidates": len(labels)}
    sets = {"all": list(range(len(kept)))}
    if pc:
        cov = [i for i, t in enumerate(kept) if t in pc]
        out["covered"] = round(len(cov) / max(len(kept), 1), 3)
        sets["covered_only"] = cov
    for name, idxs in sets.items():
        top1 = top5 = g1 = n = 0
        true_s, hard_s, easy_s = [], [], []
        for i in idxs:
            t = kept[i]
            if t not in lab_idx:
                continue  # this model has no such class (PlantCLEF): cannot be scored
            n += 1
            p = P[i]
            order = np.argsort(-p)
            best = [labels[j] for j in order[:5]]
            top1 += best[0] == t
            top5 += t in best
            g1 += best[0][0] == t[0]
            ts = float(p[lab_idx[t]].sum())
            true_s.append(ts)
            sibs = [u for u in lab_idx if u[0] == t[0] and u != t and u in db_species]
            others = [u for u in lab_idx if u != t and u in db_species]
            if sibs:
                hard_s.append((ts, float(p[lab_idx[rng.choice(sibs)]].sum())))
            if others:
                easy_s.append((ts, float(p[lab_idx[rng.choice(others)]].sum())))
        out[name] = {"scored": n, "top1": round(top1 / max(n, 1), 3),
                     "top5": round(top5 / max(n, 1), 3),
                     "top1_same_genus": round(g1 / max(n, 1), 3),
                     "verify_hard": _auc(hard_s, true_s), "verify_any": _auc(easy_s, true_s)}
    return out


def _auc(pairs, true_scores):
    """auc over (true, wrong) pairs, and how many true names pass a cut-off that admits 5% wrong."""
    if not pairs:
        return {}
    t = np.array([a for a, _ in pairs])
    w = np.array([b for _, b in pairs])
    cut = float(np.quantile(w, 0.95))
    return {"pairs": len(pairs), "auc": round(float((t > w).mean() + 0.5 * (t == w).mean()), 3),
            "cut_5pct_wrong": round(cut, 4),
            "accepts_true": round(float((np.array(true_scores) > cut).mean()), 3)}
