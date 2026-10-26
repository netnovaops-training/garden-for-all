"""Build or update the index. Safe to run again and again:

* only photos that are new, or whose size / modified time changed, are embedded;
* photos whose file is gone are removed;
* every few hundred photos are committed, so a stopped build resumes where it stopped;
* a different model or picture mode means starting over (the old vectors do not fit).
"""
import os
import time
from multiprocessing import get_context

import numpy as np

from . import catalog, decode
from .models import DEFAULT_MODEL, MODELS, fingerprint
from .store import Store

BATCH = 32
COMMIT_EVERY = 256


def build(photo_dirs, db_path, out_dir, workers=None, model=None, threads=None, limit=None,
          log=print):
    """Index the photos in `photo_dirs` (folders, see catalog.py) for the plants in `db_path`.

    workers: decode processes (default: a third of the cores); threads: model threads (default:
    all cores). Cores = the Docker --cpus limit when there is one. limit: stop after that many
    NEW photos (for tests). Returns a summary dict.
    """
    model = model or DEFAULT_MODEL
    n_cores = decode.cores()
    workers = workers or max(1, n_cores // 3)  # decoding is the lighter half of the work
    mode = MODELS[model].mode
    plants = catalog.load_plants(db_path)
    photos, skipped = catalog.scan(photo_dirs, plants)
    store = Store(out_dir)
    fp = fingerprint(model)
    if store.meta("fingerprint") not in (None, fp):
        log("model or picture settings changed: starting the index over")
        store.db.execute("DELETE FROM photos")
    store.set_meta("model", model)  # set now, so an interrupted build resumes, not restarts
    store.set_meta("fingerprint", fp)
    store.set_meta("mode", mode)
    store.commit()
    known = store.known()
    stats = {p.key: os.stat(p.path) for p in photos}
    todo = [p for p in photos if p.key not in known
            or known[p.key][:2] != (stats[p.key].st_size, stats[p.key].st_mtime_ns)]
    if limit:
        todo = todo[:limit]
    gone = [k for k, v in known.items() if not os.path.exists(v[3])]
    store.drop(gone)
    done_keys = {p.key for p in todo}
    store.retarget([p for p in photos if p.key in known and p.key not in done_keys])
    store.replace_plants(plants)
    store.commit()
    log(f"{len(photos)} photos found, {len(photos) - len(todo)} up to date, {len(todo)} to do, "
        f"{len(gone)} removed, {len(skipped)} not placed on a plant")
    n_ok = n_bad = 0
    t0 = time.time()
    if todo:
        n_ok, n_bad = _embed(todo, stats, store, model, mode, workers, threads or n_cores, log)
    secs = time.time() - t0
    info = {"new": n_ok, "unreadable": n_bad, "removed": len(gone), "seconds": round(secs, 1),
            "images_per_sec": round(n_ok / secs, 1) if secs and n_ok else 0,
            "not_placed": len(skipped), "workers": workers}
    store.stamp_build(info)
    log(f"done: {info}")
    return info


def _embed(todo, stats, store, model, mode, workers, threads, log):
    from .embed import Embedder  # torch only loads when there is something to embed
    emb = Embedder(model, threads=threads)
    size = emb.spec.size
    pool = get_context("spawn").Pool(workers)  # spawn: no fork after torch started threads
    by_path = {str(p.path): p for p in todo}
    n_ok = n_bad = 0
    batch, batch_p = [], []

    def flush():
        nonlocal n_ok
        if not batch:
            return
        for p, v in zip(batch_p, emb.embed(np.stack(batch)), strict=True):
            st = stats[p.key]
            store.put_photo(p, st.st_size, st.st_mtime_ns, v)
        n_ok += len(batch)
        batch.clear()
        batch_p.clear()

    t0 = time.time()
    try:
        for path, arr in pool.imap(decode.load_path, [(str(p.path), size, mode) for p in todo],
                                   chunksize=8):
            p = by_path[path]
            if arr is None:
                st = stats[p.key]
                store.put_photo(p, st.st_size, st.st_mtime_ns, None)  # remember: do not retry
                n_bad += 1
                continue
            batch.append(arr)
            batch_p.append(p)
            if len(batch) == BATCH:
                flush()
                if (n_ok // BATCH) % (COMMIT_EVERY // BATCH) == 0:
                    store.bump()
                    store.commit()
                    log(f"  {n_ok + n_bad}/{len(todo)}  {n_ok / (time.time() - t0):.1f} img/s")
        flush()
    finally:  # also on Ctrl-C: keep what is finished
        pool.terminate()
        store.bump()
        store.commit()
    return n_ok, n_bad
