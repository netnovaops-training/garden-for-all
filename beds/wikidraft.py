# WHAT THIS FILE DOES (plain English)
# Keeps the list of wiki drafts (table wiki_draft): text a tool suggested
# for a wiki page, waiting for a person to say "keep" or "remove" on the
# /wiki-queue page. Nothing here writes to the wiki itself; wikiapply.py
# does that, and only after "keep".
"""Wiki pages and merges waiting for a person to say yes.

NOTHING REACHES THE WIKI WITHOUT A YES. Same rule the vocabulary has had
since September, for the same reason: the model is drafting text that will
be read as fact by anybody who finds the site, and a confident mistake in
an encyclopedia is far more expensive to remove than to refuse.

FOUR KINDS OF DRAFT, and the difference between them is the whole design:

    new         a plant with no page yet
    addition    a section this page does not have
    detail      more about a section it does have
    conflict    the source disagrees with what is already there

The first three can be approved with a glance. CONFLICTS CANNOT AND ARE
NEVER AUTO-APPLIED -- they are the case where something true is about to be
replaced by something else that is also plausible. The draft carries both
texts so a person sees them side by side rather than being asked to trust
a summary of the difference.

JUN'S RECORD WINS ABOUT JUN'S GARDEN. An article says what a species does
generally; if Jun has grown it in part shade for six years then his
catalogue is the authority for HIS garden and the article is context. So a
conflict against a household value is presented as "the article disagrees",
never as "your record is wrong".

THE DRAFT HOLDS THE FINISHED TEXT. By the time Jun opens an email on
Sunday the conversation that produced this is days gone -- so approving is
a write, not a re-derivation, and the wiki cannot change under it.
"""
from __future__ import annotations


KINDS = ("new", "addition", "detail", "conflict")
STATUS = ("pending", "applied", "declined", "failed")

# Conflicts are never applied without a person, whatever a caller asks for.
NEVER_AUTO = ("conflict",)

SCHEMA = """
CREATE TABLE IF NOT EXISTS wiki_draft (
  id          INT AUTO_INCREMENT PRIMARY KEY,
  plant_id    INT           NULL,
  kind        VARCHAR(16)   NOT NULL,
  page_path   VARCHAR(220)  NOT NULL,
  title       VARCHAR(220)  NOT NULL,
  section     VARCHAR(120)  NOT NULL DEFAULT '',
  proposed    MEDIUMTEXT    NOT NULL,
  existing    MEDIUMTEXT    NULL,
  source_url  VARCHAR(500)  DEFAULT '',
  source_name VARCHAR(220)  DEFAULT '',
  images      MEDIUMTEXT    NULL,
  status      VARCHAR(16)   NOT NULL DEFAULT 'pending',
  decided_by  VARCHAR(190)  DEFAULT '',
  decided_at  TIMESTAMP     NULL,
  result      VARCHAR(600)  DEFAULT '',
  created_at  TIMESTAMP     DEFAULT CURRENT_TIMESTAMP,
  KEY k_status (status, created_at),
  KEY k_plant (plant_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
"""


def ensure(cur):
    """Create the wiki_draft table if missing, then add `destination`."""
    # Table first: asking a missing table about its columns is an error.
    cur.execute(SCHEMA)
    _ensure_destination(cur)


def _ensure_destination(cur):
    """Add `destination` to an existing table, once.

    WHICH WIKI A DRAFT IS FOR could not be recorded before: every draft went
    to Jun's garden because that was the only wiki. ALTER on a table that
    already has the column is an error rather than a no-op, so this asks
    first. Existing rows default to 'garden', which is what they all were.
    """
    cur.execute("SHOW COLUMNS FROM wiki_draft LIKE 'destination'")
    if cur.fetchone():
        return
    cur.execute("ALTER TABLE wiki_draft ADD COLUMN destination "
                "VARCHAR(16) NOT NULL DEFAULT 'garden' AFTER kind")


def pending(cur, limit=200):
    """What is waiting, OLDEST FIRST.

    Newest-first would bury the draft that has been waiting longest, which
    is the one most likely to be forgotten -- and a queue nobody finishes
    is worse than no queue, because it looks handled.
    """
    # Only drafts for the garden wiki are shown; a restored database may
    # hold old drafts meant for some other wiki, and those are left alone.
    cur.execute(
        "SELECT d.*, p.plant_name FROM wiki_draft d "
        "LEFT JOIN data_source p ON p.id = d.plant_id "
        "WHERE d.status='pending' AND d.destination='garden' "
        "ORDER BY d.created_at ASC LIMIT %s",
        (limit,))
    return cur.fetchall()


def counts(cur):
    """How many garden drafts are waiting, by kind."""
    cur.execute("SELECT kind, COUNT(*) n FROM wiki_draft "
                "WHERE status='pending' AND destination='garden' "
                "GROUP BY kind")
    return {r["kind"]: r["n"] for r in cur.fetchall()}


def decide(cur, draft_id, keep, who, apply=None):
    """Yes or no on one draft.

    A FAILED WRITE IS RECORDED AS FAILED, never as applied. Telling Jun his
    page went up when the wiki was down is the most useless thing this
    could do -- he would never look again.
    """
    cur.execute("SELECT * FROM wiki_draft WHERE id=%s", (int(draft_id),))
    row = cur.fetchone()
    if not row:
        return {"ok": False, "error": "no such draft"}
    # A FAILED DRAFT IS RETRYABLE; a decided one is not.
    #
    # `failed` means the wiki refused the write -- it was down, the API key
    # had gone stale, the page lookup broke. None of that is a decision
    # anybody made, and locking the draft because of it strands the tip for
    # good. `applied` and `declined` ARE decisions and stay final.
    if row["status"] not in ("pending", "failed"):
        return {"ok": False, "error": f"already {row['status']}"}

    if not keep:
        cur.execute("UPDATE wiki_draft SET status='declined', decided_by=%s, "
                    "decided_at=NOW() WHERE id=%s", (who, int(draft_id)))
        return {"ok": True, "status": "declined"}

    status, result = "applied", ""
    if apply:
        try:
            result = str(apply(row))[:600]
        except Exception as e:  # noqa: BLE001
            status, result = "failed", f"{type(e).__name__}: {e}"[:600]
    cur.execute("UPDATE wiki_draft SET status=%s, decided_by=%s, "
                "decided_at=NOW(), result=%s WHERE id=%s",
                (status, who, result, int(draft_id)))
    return {"ok": status == "applied", "status": status, "result": result}
