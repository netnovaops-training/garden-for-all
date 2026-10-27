"""Does this photo match the plant named in the database?  (verify-name)

Two ways, picked by the model (an index built with another model uses DEFAULT_VERIFY_MODEL):
  bioclip    zero-shot: the photo against the text "a photo of <Genus species>" for every species
             name in the database; match_score = the named species' share (softmax x100 scale).
  plantclef  the PlantCLEF-2024 classifier's share for the named species (7,806 species).

    verify_name(photo, "Salvia nemorosa 'Violet Profusion'", out_dir=INDEX)
    -> {"match_score": 0.31, "genus_score": 0.82, "rank": 1, "of": 4812, "covered": True, ...}

match_score is a share among candidate names, NOT a probability that the label is right: many
look-alike species split it. Use the README's table (right-name vs wrong-name scores) to pick a
cut-off, and treat a high genus_score with a low match_score as "right genus, species unsure".
"""
import hashlib
import os

import numpy as np

from . import decode, finder
from .embed import Embedder
from .models import MODELS
from .taxon import label, taxon

CAN_VERIFY = ("bioclip", "plantclef")
DEFAULT_VERIFY_MODEL = "bioclip-2"
_cache = {}


def species_names(store):
    """Distinct species and genera of all plants in the database: ([(genus, species)], genera)."""
    sp, ge = set(), set()
    for p in store.plants().values():
        g, s = taxon(p["botanical"])
        if g:
            ge.add(g)
        if g and s:
            sp.add((g, s))
    return sorted(sp), sorted(ge)


def plantclef_taxa(embedder):
    """Class index -> (genus, species) for the PlantCLEF classifier."""
    return [taxon(n) for n in embedder.species]


class Verifier:
    def __init__(self, out_dir=None, model=None, embedder=None):
        self.index = finder.get_index(out_dir)
        self.model = model or (self.index.model if MODELS[self.index.model].kind in CAN_VERIFY
                               else DEFAULT_VERIFY_MODEL)
        self.spec = MODELS[self.model]
        if embedder is not None:
            self.emb = embedder
        elif self.model == self.index.model:
            self.index._embedder()
            self.emb = self.index.embedder
        else:
            self.emb = Embedder(self.model)
        self.names, self.genera = species_names(self.index.store)
        if self.spec.kind == "bioclip":
            self._text()
        elif self.spec.kind == "plantclef":
            self.pc_taxa = plantclef_taxa(self.emb)

    # --- text side of BioCLIP, cached next to the index ---
    def _text(self):
        key = hashlib.sha256((self.model + repr(self.names)).encode()).hexdigest()[:16]
        path = os.path.join(str(self.index.store.dir), f"names-{self.model}-{key}.npy")
        if os.path.exists(path):
            self.T = np.load(path)
            return
        prompts = [f"a photo of {label(g, s)}" for g, s in self.names]
        parts = [self.emb.text_embed(prompts[i:i + 256]) for i in range(0, len(prompts), 256)]
        self.T = np.concatenate(parts)
        try:
            np.save(path, self.T)
        except OSError:  # a read-only index folder: just do it again next time
            pass

    def scores(self, arr):
        """One photo array -> (labels [(genus, species)], probabilities over them)."""
        k = self.spec.kind
        if k == "bioclip":
            z = self.emb.logit_scale() * (self.emb.embed(arr[None])[0] @ self.T.T)
            labels = self.names
        elif k == "plantclef":
            z = self.emb.species_logits(arr[None])[0]
            labels = self.pc_taxa
        else:
            raise ValueError(f"{self.model} cannot check names")
        z = z - z.max()
        p = np.exp(z)
        return labels, p / p.sum()

    def verify(self, image, name, top=3):
        g, s = taxon(name)
        if not g:
            raise ValueError("no botanical name to check")
        arr = decode.load(image if isinstance(image, (bytes, bytearray)) else str(image),
                          self.spec.size, self.spec.mode)
        if arr is None:
            raise ValueError("could not read this as a picture")
        labels, p = self.scores(arr)
        out = {"name": name, "genus": g, "species": s, "model": self.model,
               "method": "zero-shot text" if self.spec.kind == "bioclip" else "species classifier"}
        gp = sum(float(p[i]) for i, (lg, _) in enumerate(labels) if lg == g)
        out["genus_score"] = round(gp, 4)
        if s:
            hit = [i for i, t in enumerate(labels) if t == (g, s)]
            out["covered"] = bool(hit)
            sp = float(sum(p[i] for i in hit))
            out["match_score"] = round(sp, 4)
            out["rank"] = int((p > max(p[i] for i in hit)).sum()) + 1 if hit else None
        else:  # genus only (cultivar names): the genus is all there is to check
            out["covered"] = gp > 0
            out["match_score"] = out["genus_score"]
            out["rank"] = None
        out["of"] = len(labels)
        best = np.argsort(-p)[:top]
        out["top"] = [{"name": label(*labels[i]), "score": round(float(p[i]), 4)} for i in best]
        return out


def verify_name(image, name, out_dir=None, model=None, top=3):
    key = (str(out_dir), model)
    if key not in _cache:
        _cache[key] = Verifier(out_dir, model)
    return _cache[key].verify(image, name, top)

