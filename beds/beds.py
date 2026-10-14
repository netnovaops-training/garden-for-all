# WHAT THIS FILE DOES (plain English)
# The bed planner's data: two tables, garden_bed (each bed's name and
# outline, in inches) and garden_planting (which plant sits where in a bed).
# Also the maths the planner needs: is a point inside a bed, how big is it.
"""Where the beds are, and what is planted in them.

STEP TWO OF THE PLANNER. The catalogue knows 483 plants in detail and now
knows how big each one gets (dimensions.py). What nothing anywhere knows is
what a BED is: `flower_bed` is a string. "Front Garden Beds" has no size,
no shape and no position, so nothing can draw it and nothing can say
whether what Jun wants to plant will fit.

TWO TABLES, AND THE CATALOGUE IS NOT RESHAPED. data_source keeps its 90
columns and its `flower_bed` string exactly as the site and the Android app
expect. Beds and plantings live beside it. A planner that required
migrating the plant table would be a planner that could break the garden
site, and it does not need to.

INCHES, FROM ONE ORIGIN. Every coordinate here is inches from the garden's
top-left corner, the same unit dimensions.py wrote. One unit throughout is
the difference between a planner and a bug hunt -- and inches, not feet,
because plant spacing is written in inches and a bed is measured in feet,
so something has to give and the smaller unit loses nothing.

A BED IS A POLYGON, NOT A RECTANGLE. Real beds curve around a lawn or a
driveway. A rectangle-only planner is one Jun would have to fight, and a
list of points costs nothing more to store.

PLACING A PLANT ALSO FILES IT. 431 of 483 plants sit in a bed called "To
Be Sorted". Dragging one into a real bed writes `flower_bed` at the same
time as the planting row -- so clearing that backlog is a side effect of
using the tool rather than a separate afternoon of data entry. That write
is the whole reason `bed_name` is duplicated onto the planting row: the
catalogue's string and the planner's foreign key must not disagree, and
this is the one place that keeps them together.
"""
from __future__ import annotations

import json

# A polygon with fewer than three corners is a line, and a bed with a
# thousand is somebody's mouse stuttering.
MIN_POINTS, MAX_POINTS = 3, 200

# 500 feet. Bigger than any domestic garden, small enough that a decimal
# slip lands outside it.
MAX_INCHES = 500 * 12

SCHEMA = [
    """
    CREATE TABLE IF NOT EXISTS garden_bed (
      id          INT AUTO_INCREMENT PRIMARY KEY,
      name        VARCHAR(120)  NOT NULL,
      shape       TEXT          NOT NULL,
      sun         VARCHAR(60)   DEFAULT '',
      soil        VARCHAR(120)  DEFAULT '',
      notes       VARCHAR(500)  DEFAULT '',
      created_by  VARCHAR(190)  DEFAULT '',
      created_at  TIMESTAMP     DEFAULT CURRENT_TIMESTAMP,
      updated_at  TIMESTAMP     DEFAULT CURRENT_TIMESTAMP
                                ON UPDATE CURRENT_TIMESTAMP,
      UNIQUE KEY uq_bed_name (name)
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
    """,
    """
    CREATE TABLE IF NOT EXISTS garden_planting (
      id          INT AUTO_INCREMENT PRIMARY KEY,
      bed_id      INT           NOT NULL,
      plant_id    INT           NOT NULL,
      x           DECIMAL(9,2)  NOT NULL,
      y           DECIMAL(9,2)  NOT NULL,
      qty         INT           NOT NULL DEFAULT 1,
      placed_by   VARCHAR(190)  DEFAULT '',
      placed_at   TIMESTAMP     DEFAULT CURRENT_TIMESTAMP,
      KEY k_bed (bed_id),
      KEY k_plant (plant_id),
      CONSTRAINT fk_planting_bed FOREIGN KEY (bed_id)
        REFERENCES garden_bed(id) ON DELETE CASCADE
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
    """,
]


def ensure(cur):
    """Create the tables if they are not there. Safe to call every start."""
    for stmt in SCHEMA:
        cur.execute(stmt)


# ---------------------------------------------------------------------------
#  Validation. Everything below is called with whatever a browser sent.
# ---------------------------------------------------------------------------
def clean_shape(raw):
    """A list of [x, y] inch pairs, or a reason it is not one."""
    if isinstance(raw, str):
        try:
            raw = json.loads(raw)
        except ValueError:
            return None, "shape is not valid JSON"
    if not isinstance(raw, list):
        return None, "shape must be a list of points"
    if not (MIN_POINTS <= len(raw) <= MAX_POINTS):
        return None, f"a bed needs {MIN_POINTS} to {MAX_POINTS} corners"
    out = []
    for p in raw:
        if not (isinstance(p, (list, tuple)) and len(p) == 2):
            return None, "each corner must be [x, y]"
        try:
            x, y = float(p[0]), float(p[1])
        except (TypeError, ValueError):
            return None, "corners must be numbers"
        if not (0 <= x <= MAX_INCHES and 0 <= y <= MAX_INCHES):
            return None, "a corner is outside the garden"
        out.append([round(x, 2), round(y, 2)])
    return out, None


def area_sqft(shape):
    """The shoelace formula, in square feet.

    Worth having because it is the first number that tells somebody their
    bed is wrong: a 4x12 bed is 48 sq ft, and if the tool says 4,800 the
    scale is out by a factor of ten.
    """
    n = len(shape)
    total = 0.0
    for i in range(n):
        x1, y1 = shape[i]
        x2, y2 = shape[(i + 1) % n]
        total += x1 * y2 - x2 * y1
    return abs(total) / 2.0 / 144.0


def bounds(shape):
    xs = [p[0] for p in shape]
    ys = [p[1] for p in shape]
    return min(xs), min(ys), max(xs), max(ys)


def inside(shape, x, y):
    """Ray casting: is this point in the polygon?

    Used to refuse a planting placed outside its own bed -- which is easy
    to do by half a pixel at the edge, and which would then draw a plant
    floating on the lawn.
    """
    hit = False
    n = len(shape)
    for i in range(n):
        x1, y1 = shape[i]
        x2, y2 = shape[(i + 1) % n]
        if (y1 > y) != (y2 > y):
            xx = (x2 - x1) * (y - y1) / (y2 - y1) + x1
            if x < xx:
                hit = not hit
    return hit


def footprint(plant):
    """How wide this plant gets, in inches, from the numeric columns.

    WIDTH, NOT LENGTH. dimensions.py wrote both; on a top-down plan the
    width is the circle the plant occupies and the length is how tall it
    stands. Reaching for the wrong one draws a three-foot shrub as a
    nine-foot blob.

    THE MAXIMUM, NOT THE MINIMUM. A bed planted to the minimum of every
    range is a bed that closes up in three years. Spacing beats width
    where the catalogue gives one, because spacing is somebody's advice
    about what this plant needs and width is only what it becomes.
    """
    for key in ("spacing_max_in", "plant_width_max_in"):
        v = plant.get(key)
        if v:
            return float(v)
    return 18.0          # a guess, and flagged as one by the caller
