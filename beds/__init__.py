# WHAT THIS FILE DOES (plain English)
# This makes the beds folder a Python "package" called garden_ingest (the
# Dockerfile copies it to /app/garden_ingest). It holds the small set of
# functions every write to the plant catalogue goes through: review()
# (look, write nothing), remember() (save a person's decision) and
# apply_approved() (write only what a person approved). ensure() creates
# the two approval tables if a fresh database does not have them yet.
"""garden_ingest -- the one door into a garden catalogue.

EVERY WRITE GOES THROUGH HERE. Not "should": the point of this package is
that a second app, written next year by somebody else, cannot invent its own
idea of what "part sun" means, because it never has to. It imports this,
calls `review()`, and gets back a decision per field that is already spoken
in the vocabulary the target database uses.

    from garden_ingest import review, apply_approved

    batch = review(cursor, scraped_dict)      # nothing written yet
    ...                                       # a person approves
    apply_approved(cursor, plant_id, batch, approved_by="jun@...")

WHY A LIBRARY AND NOT A SERVICE. A service is a thing to run, monitor,
version and keep reachable; a library is a file that either imports or does
not. This has to survive being copied to another gardener's machine by
somebody who will not be maintaining it, which rules out anything with a
uptime requirement.

WHAT IT DELIBERATELY WILL NOT DO:

  * It will not write on its own. `review()` only reads. Applying is a
    separate call that takes an approver's address, because an audit trail
    with nobody's name in it is decoration.
  * It will not invent a category. An unrecognised value becomes a proposal
    a person answers once; the answer is remembered as an alias and never
    asked again.
  * It will not touch free text. Notes and names have nothing to normalise
    onto and forcing them into a vocabulary loses the only thing they carry.
"""
from __future__ import annotations

from .vocab import (CONTROLLED, Decision, Vocabulary, build, canon_key,
                    resolve_all)

import re

__all__ = ["review", "apply_approved", "remember", "ensure", "CONTROLLED",
           "Decision", "Vocabulary", "canon_key", "build", "resolve_all",
           "SCHEMA"]

SCHEMA = """
-- What a person has already decided a loose wording means, here.
-- The whole self-maintaining trick lives in this table: each row is a
-- question that will never be asked again.
CREATE TABLE IF NOT EXISTS vocab_alias (
  id          INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  field_name  VARCHAR(64)  NOT NULL,
  -- The NORMALISED form of what arrived, not the raw string: "Part Sun",
  -- "part sun" and "part  sun" are one question, not three.
  raw_key     VARCHAR(191) NOT NULL,
  canonical   VARCHAR(191) NOT NULL,
  approved_by VARCHAR(190) NOT NULL,
  created_at  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uniq_field_raw (field_name, raw_key),
  KEY idx_field (field_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Proposals waiting on a person. A row here has NOT been applied.
CREATE TABLE IF NOT EXISTS vocab_proposal (
  id          INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  plant_id    INT          NULL,
  field_name  VARCHAR(64)  NOT NULL,
  raw_value   VARCHAR(500) NOT NULL,
  suggested   VARCHAR(191) NULL,
  -- WHAT THE FIELD SAID BEFORE THE APPROVAL OVERWROTE IT. Captured at the
  -- moment of writing, because that is the only moment it still exists.
  -- Without it "revert" can only mean "clear the field", which is worse
  -- than the wrong value it would be undoing.
  previous_value VARCHAR(500) NULL,
  confidence  FLOAT        NOT NULL DEFAULT 0,
  how         VARCHAR(16)  NOT NULL,
  source      VARCHAR(500) NULL,
  -- FIVE STATES, NOT THREE. 'approved' is Jun's yes; 'confirmed' is the
  -- second pass having looked. Collapsing them would make a checked row
  -- indistinguishable from an unchecked one the moment it left the queue.
  status      ENUM('pending','approved','rejected','confirmed','reverted')
              NOT NULL DEFAULT 'pending',
  decided_by  VARCHAR(190) NULL,
  decided_at  TIMESTAMP    NULL,
  -- The second signature, kept BESIDE the first rather than over it.
  reviewed_by VARCHAR(190) NULL,
  reviewed_at TIMESTAMP    NULL,
  review_note VARCHAR(500) NULL,
  created_at  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  KEY idx_status (status),
  KEY idx_plant (plant_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
"""


_ENSURED = False


def ensure(cursor):
    """Create vocab_alias and vocab_proposal if they are missing (once per run).

    A restored backup already has them; a brand-new database does not. The
    split is on ";" at the END of a line only, because one comment above
    has a ";" in the middle of a sentence.
    """
    global _ENSURED
    if _ENSURED:
        return
    for stmt in re.split(r";[ \t]*\n", SCHEMA):
        if stmt.strip():
            cursor.execute(stmt)
    _ENSURED = True


def review(cursor, incoming: dict, table: str = "data_source",
           source: str | None = None) -> dict:
    """Look at a scraped record. Write nothing.

    Returns what would happen, field by field: which values are already
    spoken here, which are near-misses worth offering, and which are
    genuinely new and need a person. `source` travels with the proposals so
    a fact can be traced back to the page it came from.
    """
    vocabs = build(cursor, table)
    out = resolve_all(vocabs, incoming)

    ready, asks = {}, []
    for f, d in out["decisions"].items():
        if d.needs_person:
            asks.append({
                "field": f, "raw": d.raw, "suggested": d.value,
                "confidence": round(d.confidence, 3), "how": d.how,
                "alternatives": d.alternatives, "note": d.note,
                "source": source,
            })
        else:
            ready[f] = d.value

    return {
        # Safe to write without asking: an exact match or a remembered alias.
        "ready": ready,
        # Everything a person has to answer, once.
        "asks": asks,
        # Fields with no controlled vocabulary, passed through as written.
        "passthrough": out["passthrough"],
        "source": source,
    }


def remember(cursor, field: str, raw: str, canonical: str, approved_by: str):
    """Record a person's decision so it is never asked again."""
    cursor.execute(
        "INSERT INTO vocab_alias (field_name, raw_key, canonical, approved_by) "
        "VALUES (%s, %s, %s, %s) "
        "ON DUPLICATE KEY UPDATE canonical = VALUES(canonical), "
        "approved_by = VALUES(approved_by)",
        (field, canon_key(raw), canonical, approved_by),
    )


def apply_approved(cursor, plant_id: int, fields: dict, approved_by: str,
                   table: str = "data_source") -> int:
    """Write the fields a person has approved, and nothing else.

    TAKES A DICT THE CALLER BUILT FROM AN APPROVAL, not the batch from
    `review()`. That is deliberate: it makes it impossible to pass the
    proposals straight through by accident, because the shapes differ.
    """
    if not fields:
        return 0
    cols = ", ".join(f"`{f}` = %s" for f in fields)
    cursor.execute(
        f"UPDATE `{table}` SET {cols} WHERE id = %s",
        (*fields.values(), plant_id),
    )
    return cursor.rowcount
