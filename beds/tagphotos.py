# WHAT THIS FILE DOES (plain English)
# Saves the photos of plant tags under /photos/tags/<plant id>/ so you can
# check later what the tag really said. Table plant_tag_photo lists them.
"""Keep the photographs a tag was read from, filed under the plant.

JASON, 2026-09-26: "For the tagging, can we add a folder album to be added
to it" -- after asking for "the picture from the tag to compare against
online information so that an informed decision can be made".

WHY KEEPING THEM MATTERS MORE HERE THAN IT LOOKS. A tag scan produces a
plant record built by a vision model reading small print in a garden, then
enriched from the web. Every one of those fields is a claim, and until now
the only evidence for any of them was discarded the moment it was read. A
value that looks wrong six months later could not be checked against
anything. With the photograph kept, "the tag says 18-24 inches" is a thing
somebody can go and look at.

A FOLDER PER PLANT, NOT ONE FLAT PILE. garden_photo writes every image into
a single directory under a uuid, which is right for bed photographs that
are looked up by database row and wrong for these: the point of an album is
that a person can open it. /photos/tags/435/ is Aphrodite's tags, and it
reads as that from a file manager, a backup, or a shell -- no database
needed to make sense of it.

UNASSIGNED IS A REAL FOLDER, NOT AN ERROR. A tag whose name could not be
read produces no plant, and the photograph is the only evidence of what was
scanned. Throwing it away because the read failed discards exactly the
case somebody would want to look at.

THE SIDE IS IN THE FILENAME. front/back rather than 1/2, because in six
months the question is "what did the back say", and a directory listing
should answer it without opening anything.
"""
from __future__ import annotations

import base64
import binascii
import io
import os
import pathlib
import time
import uuid

ROOT = pathlib.Path(os.environ.get("GARDEN_PHOTO_DIR", "/photos")) / "tags"
MAX_BYTES = 24 * 1024 * 1024
LONG_EDGE = 2000          # the print on a nursery tag is small; keep detail

SCHEMA = [
    """
    CREATE TABLE IF NOT EXISTS plant_tag_photo (
      id          INT AUTO_INCREMENT PRIMARY KEY,
      plant_id    INT           NULL,
      side        VARCHAR(10)   NOT NULL DEFAULT 'front',
      filename    VARCHAR(200)  NOT NULL,
      width       INT           NOT NULL,
      height      INT           NOT NULL,
      taken_by    VARCHAR(190)  DEFAULT '',
      taken_at    TIMESTAMP     DEFAULT CURRENT_TIMESTAMP,
      KEY k_plant (plant_id)
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
    """,
]


def ensure(cur):
    for sql in SCHEMA:
        try:
            cur.execute(sql)
        except Exception:
            # Already there. Same tolerance the other modules here use --
            # a migration that cannot be run twice is a migration that gets
            # run by hand once and then forgotten.
            pass


def _folder(plant_id) -> pathlib.Path:
    """Where this plant's tags live. Named for the plant, not a hash."""
    return ROOT / (str(plant_id) if plant_id else "unassigned")


def save(data_url: str, plant_id=None, side: str = "front") -> tuple:
    """Decode, straighten, shrink, write. Returns (relative_path, w, h).

    Raises ValueError with something a person can read, because these
    failures reach the phone.
    """
    from PIL import Image, ImageOps

    side = (side or "front").lower()
    if side not in ("front", "back"):
        side = "front"

    try:
        raw = base64.b64decode((data_url or "").split(",")[-1])
    except (binascii.Error, ValueError):
        raise ValueError("that is not an image")
    if not raw:
        raise ValueError("that photograph is empty")
    if len(raw) > MAX_BYTES:
        raise ValueError("that photograph is too large")

    try:
        im = Image.open(io.BytesIO(raw))
        # EXIF rotation baked in. A phone records orientation as metadata
        # and something downstream always ignores it -- then the tag is
        # sideways and unreadable in the very album meant to verify it.
        im = ImageOps.exif_transpose(im).convert("RGB")
    except Exception:
        raise ValueError("could not read that as a photograph")

    im.thumbnail((LONG_EDGE, LONG_EDGE), Image.LANCZOS)
    folder = _folder(plant_id)
    folder.mkdir(parents=True, exist_ok=True)

    # Sortable, and says what it is without a database. The uuid tail keeps
    # two scans in the same second from colliding.
    name = "{}-{}-{}.jpg".format(
        time.strftime("%Y%m%d-%H%M%S"), side, uuid.uuid4().hex[:8])
    im.save(folder / name, "JPEG", quality=88, optimize=True)
    return "{}/{}".format(folder.name, name), im.width, im.height


def record(cur, plant_id, side, rel_path, width, height, who):
    """Note the photograph against the plant."""
    ensure(cur)
    cur.execute(
        "INSERT INTO plant_tag_photo "
        "(plant_id, side, filename, width, height, taken_by) "
        "VALUES (%s,%s,%s,%s,%s,%s)",
        (plant_id, side, rel_path, width, height, (who or "")[:190]))
    return cur.lastrowid


def for_plant(cur, plant_id) -> list:
    """Every tag photograph kept for one plant, newest first."""
    ensure(cur)
    cur.execute(
        "SELECT id, side, filename, width, height, taken_by, taken_at "
        "FROM plant_tag_photo WHERE plant_id = %s "
        "ORDER BY taken_at DESC, id DESC", (plant_id,))
    rows = cur.fetchall()
    for r in rows:
        r["taken_at"] = str(r["taken_at"])
    return rows
