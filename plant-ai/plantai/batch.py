"""Run the stages for many plants, in two phases so the graphics card never swaps models per plant:

  phase 1 (phi4)   discover + fill for every plant (photos are fetched alongside, in threads)
  phase 2 (gemma)  check for every plant

With OLLAMA_MAX_LOADED_MODELS=1 the card holds one model at a time, so a queue of 10 plants loads
phi4 once and gemma once, instead of 20 swaps. `progress(pset, pid, step)` is told each step.
"""
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from . import llm, stages
from .config import CFG


def log(msg: str) -> None:
    line = f"{time.strftime('%m-%d %H:%M:%S')} {msg}"
    with Path(CFG.work / "run.log").open("a", encoding="utf-8") as f:
        f.write(line + "\n")


def safe(fn, *a):
    """One bad plant never stops the run: the error goes to run.log, the plant gets None."""
    try:
        return fn(*a)
    except Exception as e:  # noqa: BLE001 -- logged, then the next plant
        log(f"FAIL {fn.__name__} {a[0]}/{a[1]}: {type(e).__name__}: {e}")
        return None


def run(work: list[tuple[str, int, dict]], progress=None, discover_threads: int = 4,
        image_threads: int = 3) -> None:
    say = progress or (lambda *_: None)
    log(f"run start: {len(work)} plants")
    t0 = time.monotonic()
    with ThreadPoolExecutor(discover_threads, thread_name_prefix="disc") as pool_d, \
            ThreadPoolExecutor(image_threads, thread_name_prefix="img") as pool_i:
        def disc(item):
            s, pid, rec = item
            say(s, pid, "searching")
            safe(stages.do_discover, s, pid, rec)
            return item

        img_f = []
        # Phase 1: phi4 reads the pages as each plant's search finishes (in work order).
        for n, item in enumerate(pool_d.map(disc, work)):
            s, pid, rec = item
            img_f.append(pool_i.submit(safe, stages.do_images, s, pid, rec))
            say(s, pid, "filling")
            safe(stages.do_fill, s, pid, rec)
            say(s, pid, "photos")
            if (n + 1) % 20 == 0 or n + 1 == len(work):
                el = max(1e-6, time.monotonic() - t0)
                log(f"fill {n + 1}/{len(work)}  {60 * (n + 1) / el:.2f} plants/min  llm {llm.STATS}")
        for fut in img_f:
            fut.result()
    t1 = time.monotonic()
    log(f"phi4 stage done in {(t1 - t0) / 60:.1f} min")
    # Phase 2: gemma checks everything phi4 did.
    for n, (s, pid, rec) in enumerate(work):
        say(s, pid, "checking")
        safe(stages.do_check, s, pid, rec)
        say(s, pid, "done")
        if (n + 1) % 20 == 0 or n + 1 == len(work):
            el = max(1e-6, time.monotonic() - t1)
            log(f"check {n + 1}/{len(work)}  {60 * (n + 1) / el:.2f} plants/min")
    log(f"run done in {(time.monotonic() - t0) / 60:.1f} min")
