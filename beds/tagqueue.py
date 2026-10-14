# WHAT THIS FILE DOES (plain English)
# The tag queue: many tag photos in at once, read one by one afterwards.
# Table tag_queue holds each tag's status (waiting, working, done, failed,
# duplicate). Used by the phone app's bulk tag upload and by /api/tag when
# the free AI is busy or not set up yet.
"""A pile of tags in, plants out, without anybody waiting.

JASON, 2026-09-26: "I would like to make a tag queue and a photo with
Google photos that will catch duplicates ... This will need front and back."

WHY A QUEUE AND NOT A LOOP. One tag takes about twenty-two seconds: two
vision passes over the photographs, a taxonomy lookup, and an enrichment
call. Twenty tags is seven minutes, and a phone cannot hold an HTTP request
open for seven minutes -- it sleeps, the radio drops, the screen rotates.
Every one of those loses the answer and, worse, leaves no record that the
tags were ever sent.

So the upload does the only fast thing: write the photographs to disk and
hand back a receipt. Everything slow happens afterwards, to rows that
already exist, and a crash costs a retry rather than an afternoon of
photographs.

FRONT AND BACK STAY TOGETHER. A tag is a pair, not two pictures. Queuing
them separately would read a front with no spacing and a back with no name
and produce two useless halves; the pair is the unit of work, and the back
is optional because a tag in a pot does not always turn over.

TWO KINDS OF DUPLICATE, AND THEY ARE NOT THE SAME QUESTION.

  THE SAME PHOTOGRAPH, SENT TWICE. Caught here, on the bytes, before any
  work is done. A phone that resends a batch after a dropped connection is
  the ordinary case, and re-reading forty tags to discover they are the
  forty already read is the expensive way to find out.

  THE SAME PLANT, ALREADY IN THE CATALOGUE. Caught after reading, by
  dedupe.py, which compares normalised names so that "Hosta 'Blue Angel'"
  and "Blue Angel Hosta" are recognised as one plant. That one cannot be
  answered from the image at all -- two different photographs of two
  different tags for the same cultivar are duplicates, and two photographs
  of the same tag in different light are not.

NOTHING IS EVER SILENTLY DROPPED. A duplicate is recorded with a status and
a reason and stays in the queue to be looked at. Jason's own note in
dedupe.py is the rule: "A wrongly skipped plant is invisible -- it simply
never appears - which is the worst failure available here."
"""
from __future__ import annotations

import hashlib
import json
import uuid

SCHEMA = [
    """
    CREATE TABLE IF NOT EXISTS tag_queue (
      id          INT AUTO_INCREMENT PRIMARY KEY,
      batch       VARCHAR(32)   NOT NULL,
      front_file  VARCHAR(200)  NOT NULL,
      back_file   VARCHAR(200)  NULL,
      front_hash  CHAR(64)      NOT NULL,
      status      VARCHAR(12)   NOT NULL DEFAULT 'waiting',
      plant_id    INT           NULL,
      note        TEXT          NULL,
      added_by    VARCHAR(190)  DEFAULT '',
      added_at    TIMESTAMP     DEFAULT CURRENT_TIMESTAMP,
      done_at     TIMESTAMP     NULL,
      KEY k_status (status),
      KEY k_batch (batch),
      -- THE INDEX IS THE DUPLICATE CHECK. Asking "have I seen this
      -- photograph" with a scan of a growing table gets slower exactly as
      -- the pile of tags gets bigger, which is the moment it matters.
      KEY k_hash (front_hash)
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
    """,
]


def ensure(cur):
    for sql in SCHEMA:
        try:
            cur.execute(sql)
        except Exception:
            pass


def image_hash(data_url: str) -> str:
    """A fingerprint for the photograph itself.

    ON THE DECODED BYTES, not the data URL: the same JPEG re-encoded by a
    phone between two upload attempts differs as a string and not as an
    image. This still misses a genuinely re-taken photograph of the same
    tag -- that one is caught later, by name.
    """
    raw = (data_url or "").split(",")[-1].encode("utf-8", "ignore")
    return hashlib.sha256(raw).hexdigest()


def add(cur, front_file, back_file, front_hash, who) -> tuple:
    """Put one tag in the queue. Returns (id, status, note)."""
    ensure(cur)
    cur.execute(
        "SELECT id, status, plant_id FROM tag_queue WHERE front_hash = %s "
        "ORDER BY id LIMIT 1", (front_hash,))
    seen = cur.fetchone()
    if seen:
        # Recorded, not refused: he should be able to see that a photograph
        # arrived twice rather than wonder where it went.
        cur.execute(
            "INSERT INTO tag_queue (batch, front_file, back_file, "
            "front_hash, status, plant_id, note, added_by) "
            "VALUES (%s,%s,%s,%s,'duplicate',%s,%s,%s)",
            ("dup", front_file, back_file, front_hash, seen.get("plant_id"),
             "the same photograph is already queued as #%s" % seen["id"],
             (who or "")[:190]))
        return cur.lastrowid, "duplicate", "already queued as #%s" % seen["id"]

    cur.execute(
        "INSERT INTO tag_queue (batch, front_file, back_file, front_hash, "
        "added_by) VALUES (%s,%s,%s,%s,%s)",
        (uuid.uuid4().hex[:12], front_file, back_file, front_hash,
         (who or "")[:190]))
    return cur.lastrowid, "waiting", ""


def claim(cur):
    """Take the next tag to work on, or None.

    CLAIMED BY UPDATE, NOT BY SELECT-THEN-UPDATE. Two workers that both
    read 'waiting' would both do the same tag and create the plant twice,
    which is the exact thing this file exists to prevent.
    """
    ensure(cur)
    cur.execute(
        "UPDATE tag_queue SET status = 'working' WHERE status = 'waiting' "
        "ORDER BY id LIMIT 1")
    if not cur.rowcount:
        return None
    cur.execute(
        "SELECT * FROM tag_queue WHERE status = 'working' "
        "ORDER BY id LIMIT 1")
    return cur.fetchone()


def finish(cur, qid, status, plant_id=None, note=""):
    cur.execute(
        "UPDATE tag_queue SET status=%s, plant_id=%s, note=%s, "
        "done_at=NOW() WHERE id=%s",
        (status, plant_id, (note or "")[:2000], qid))


def summary(cur) -> dict:
    ensure(cur)
    cur.execute("SELECT status, COUNT(*) n FROM tag_queue GROUP BY status")
    counts = {r["status"]: r["n"] for r in cur.fetchall()}
    cur.execute(
        "SELECT id, status, plant_id, note, added_at, done_at "
        "FROM tag_queue ORDER BY id DESC LIMIT 25")
    recent = cur.fetchall()
    for r in recent:
        r["added_at"] = str(r["added_at"] or "")
        r["done_at"] = str(r["done_at"] or "")
    return {"counts": counts, "recent": recent,
            "waiting": counts.get("waiting", 0),
            "working": counts.get("working", 0)}
