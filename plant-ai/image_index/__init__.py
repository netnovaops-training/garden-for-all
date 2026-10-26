"""Image index for Jun's Garden: find a plant from a photo, using the photos already in the garden.

    from image_index import build, search, status
    build(photo_dirs, db_path, out_dir, workers)   # new or changed photos only, resumable
    search(image_bytes_or_path, k=5)               # [{plant_id, plant_name, botanical_name,
                                                   #   score, photo, set}, ...]

The heavy parts (torch, the model) load only when they are needed.
"""


def build(photo_dirs, db_path, out_dir, workers=None, **kw):
    from .indexer import build as _build
    return _build(photo_dirs, db_path, out_dir, workers, **kw)


def search(image, k=5, **kw):
    from .finder import search as _search
    return _search(image, k, **kw)


def status(out_dir=None):
    from .finder import status as _status
    return _status(out_dir)


__all__ = ["build", "search", "status"]
