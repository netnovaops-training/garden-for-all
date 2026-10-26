"""Command line.

  python -m plantai check                      is Ollama here, are both models pulled, what the card has
  python -m plantai worker --settings worker.json [--once]
                                               the desktop helper for the garden website (worker.py)
  python -m plantai run --in plants.json [--set new] [--work DIR] [--out DIR] [--photos-dir DIR]
                                               a batch from a JSON array of plant rows (our own data run)
  python -m plantai report [--work DIR] [--out DIR]
                                               rebuild the batch output files only
"""
import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

from . import batch, config, db, llm, report


def gpu() -> str:
    """The graphics card and its memory, from nvidia-smi ('' when there is none)."""
    exe = shutil.which("nvidia-smi")
    if not exe:
        return ""
    try:
        out = subprocess.run([exe, "--query-gpu=name,memory.total", "--format=csv,noheader"],
                             capture_output=True, text=True, timeout=20, check=False).stdout
    except (OSError, subprocess.SubprocessError):
        return ""
    return out.strip().splitlines()[0] if out.strip() else ""


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(prog="plantai")
    ap.add_argument("cmd", choices=["check", "worker", "run", "report"])
    ap.add_argument("--settings", type=Path)
    ap.add_argument("--once", action="store_true")
    ap.add_argument("--in", dest="inp", type=Path)
    ap.add_argument("--set", default="new")
    ap.add_argument("--work", type=Path, default=Path("plant-ai-work"))
    ap.add_argument("--out", type=Path, default=Path("plant-ai-out"))
    ap.add_argument("--photos-dir", type=Path)
    ap.add_argument("--reference-photos", type=Path, help="a folder with up to 3 of your own good plant photos")
    a = ap.parse_args(argv)
    if a.cmd == "worker":
        from . import worker
        if not a.settings:
            ap.error("worker needs --settings worker.json")
        s = worker.load_settings(a.settings)
        here = a.settings.parent
        config.use(here / s.get("work", "work"), here / s.get("out", "out"))
        if s.get("reference_photos"):
            config.CFG.reference_photos = sorted(Path(here / s["reference_photos"]).glob("*.*"))[:3]
        site = worker.Site(s["site"], worker.read_key(here / s.get("key_file", "worker-key.txt")))
        if a.once:
            print(f"done {worker.run_once(site)}")
            return 0
        worker.run_forever(site, float(s.get("poll_seconds", 60)))
        return 0
    config.use(a.work, a.out)
    if a.photos_dir:
        config.CFG.photos_dir = a.photos_dir
    if a.reference_photos:
        config.CFG.reference_photos = sorted(a.reference_photos.glob("*.*"))[:3]
    if a.cmd == "check":
        ok, why = llm.ready()
        print(("PASS " if ok else "FAIL ") + why)
        print("card: " + (gpu() or "no NVIDIA card found"))
        return 0 if ok else 1
    if a.cmd == "run":
        if not a.inp:
            ap.error("run needs --in plants.json")
        rows = json.loads(a.inp.read_text(encoding="utf-8"))
        if not isinstance(rows, list):
            ap.error("the input must be a JSON array of plant rows")
        db.load(a.set, rows)
        batch.run([(a.set, pid, rec) for pid, rec in db.plants(a.set)])
    report.build()
    print(f"PASS report in {config.CFG.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
