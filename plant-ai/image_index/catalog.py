"""Which photo belongs to which plant.

`db_path` is a folder with the garden's two table exports, `current.json` (data_source, the
plants in the garden) and `future.json` (future_plants, the wish list): the same files the
plant-data pipeline reads. Photos come from `photo_dirs`:

* current set: a file named in a plant's `picture` / `additional_pictures`, or (not listed
  there) a file whose name carries the plant id, like `Name_218_manual_1779500230.webp`;
* future set: a file inside a numbered folder, `<future plant id>/<n>.webp`.

A photo dir can be given as `current=PATH` or `future=PATH` to force the set.
"""
import json
import re
from dataclasses import dataclass
from pathlib import Path

EXTS = {".webp", ".jpg", ".jpeg", ".png"}
_ID_IN_NAME = re.compile(r"_(\d+)(?=_|\.)")
_BAD_CHAR = "�"  # the export lost the x in hybrid names ("Abelia x grandiflora")


@dataclass(frozen=True)
class Photo:
    key: str  # "<set>:<path relative to its photo dir>"; the id of a photo in the index
    path: Path
    set: str
    plant_id: int
    rel: str  # path relative to its photo dir (this is what search() returns)


def clean_name(text):
    """Botanical names from the export: fix the lost hybrid sign, squeeze spaces."""
    return " ".join((text or "").replace(_BAD_CHAR, "x").split())


def load_plants(db_path):
    """{'current': {id: {...}}, 'future': {id: {...}}} from the two json exports."""
    out = {}
    for set_, fname in (("current", "current.json"), ("future", "future.json")):
        f = Path(db_path) / fname
        rows = json.loads(f.read_text(encoding="utf-8")) if f.exists() else []
        out[set_] = {int(r["id"]): {
            "name": (r.get("plant_name") or "").strip(),
            "botanical": clean_name(r.get("botanical_name")),
            "pictures": [p.strip() for p in [r.get("picture") or ""] +
                         (r.get("additional_pictures") or "").split(",") if p.strip()],
        } for r in rows}
    return out


def _split_dirs(photo_dirs):
    """['current=/a', '/b'] -> [(forced set or None, Path), ...]"""
    for d in photo_dirs:
        d = str(d)
        forced = None
        for s in ("current", "future"):
            if d.startswith(s + "="):
                forced, d = s, d[len(s) + 1:]
        yield forced, Path(d)


def scan(photo_dirs, plants):
    """Every photo we can place on a plant. Files we cannot place are listed in `skipped`."""
    listed = {}  # current set: file name -> plant id, from the database columns
    for pid, p in plants["current"].items():
        for name in p["pictures"]:
            listed.setdefault(Path(name).name, pid)
    photos, skipped = [], []
    for forced, root in _split_dirs(photo_dirs):
        if not root.is_dir():
            skipped.append((str(root), "folder not found"))
            continue
        for f in sorted(root.rglob("*")):
            if f.suffix.lower() not in EXTS or not f.is_file():
                continue
            rel = f.relative_to(root).as_posix()
            hit = _place(f, rel, forced, listed, plants)
            if hit:
                photos.append(Photo(f"{hit[0]}:{rel}", f, hit[0], hit[1], rel))
            else:
                skipped.append((rel, "no plant found for this file"))
    return photos, skipped


def _place(f, rel, forced, listed, plants):
    parts = rel.split("/")
    if forced in (None, "current"):
        pid = listed.get(f.name)
        if pid is None:
            ids = [int(x) for x in _ID_IN_NAME.findall(f.name)]
            known = [i for i in ids if i in plants["current"]]
            pid = known[-1] if known else None
        if pid is not None:
            return "current", pid
    if forced in (None, "future") and len(parts) >= 2 and parts[-2].isdigit() \
            and int(parts[-2]) in plants["future"]:
        return "future", int(parts[-2])
    return None

