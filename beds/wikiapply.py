# WHAT THIS FILE DOES (plain English)
# When you press "keep" on a draft in /wiki-queue, apply_draft() writes it
# onto the right page of your garden wiki (through wikijs.py). An existing
# page gets the new text added at the bottom under a dated heading; it is
# never replaced.
"""Write an approved draft into the wiki.

APPROVING IS THE ONLY THING THAT WRITES. Everything upstream of this
produces drafts and stops. This
function is the single door between "a model suggested it" and "it is on the
page", which is why it is small enough to read in one sitting.

AN EXISTING PAGE IS APPENDED TO, NEVER REPLACED. The draft is one tip from
one video; the page may be 400 lines of Jun's own notes. Overwriting it with
the tip would be a data loss that looks like a successful approval. So the
page is read, the new material is added under a dated heading, and the whole
thing is written back.

THE SOURCE GOES ON THE PAGE, NOT IN THE COMMIT. A tip from a video carries a
link to the exact second it was said. Anyone reading the page later can check
it in ten seconds, which is the difference between a citation and a claim.
"""
from __future__ import annotations

import datetime

from garden_ingest import wikijs


def _stamp():
    """Today's date as YYYY-MM-DD, for the "Added ..." heading."""
    return datetime.date.today().isoformat()


def block_for(row):
    """The markdown one approved draft adds to a page."""
    body = (row.get("proposed") or "").strip()
    src = (row.get("source_url") or "").strip()
    name = (row.get("source_name") or "").strip()
    section = (row.get("section") or "").strip()

    out = []
    if section:
        out.append(f"### {section}")
    out.append(body)
    if src:
        # Named link, so the page does not fill up with raw URLs.
        label = name or "source"
        out.append("")
        out.append(f"*Source: [{label}]({src})*")
    return chr(10).join(out)


def apply_draft(row):
    """Put one approved draft on its page. Raises on failure.

    RAISES RATHER THAN RETURNING FALSE, deliberately: `wikidraft.decide`
    catches an exception and records the draft as `failed` instead of
    `applied`. Swallowing the error here would mark it applied and nobody
    would look for it again.
    """
    # Only drafts meant for the garden wiki are written. (Old drafts in a
    # restored database may name another destination; those are skipped.)
    where = (row.get("destination") or "garden").strip() or "garden"
    if where != "garden":
        raise RuntimeError("this draft was not meant for the garden wiki")
    bad = wikijs.check()
    if bad:
        raise RuntimeError(f"the wiki is not usable: {bad}")

    path = (row.get("page_path") or "").strip().strip("/")
    title = (row.get("title") or "Untitled").strip()
    if not path:
        raise ValueError("this draft has no page to go on")

    addition = block_for(row)
    if not addition.strip():
        raise ValueError("this draft has nothing in it")

    existing, err = wikijs.page_id(path)
    if err:
        raise RuntimeError(f"could not look that page up: {err}")

    if existing:
        # READ, APPEND, WRITE BACK. See the header: replacing the page would
        # be a silent data loss dressed up as a successful approval.
        got, err = wikijs.gql(
            """query ($p:String!, $l:String!) {
                 pages { singleByPath(path:$p, locale:$l) { content } } }""",
            {"p": path, "l": "en"})
        if err:
            raise RuntimeError(f"could not read that page: {err}")
        current = (((got or {}).get("pages") or {}).get("singleByPath") or {}
                   ).get("content") or ""
        body = current.rstrip() + chr(10) * 2 + f"## Added {_stamp()}" \
            + chr(10) * 2 + addition + chr(10)
    else:
        body = f"# {title}" + chr(10) * 2 + f"## Added {_stamp()}" \
            + chr(10) * 2 + addition + chr(10)

    ok, what = wikijs.put_page(path, title, body,
                               description=title[:250], tags=["plant"],
                               existing=bool(existing))
    if not ok:
        raise RuntimeError(what)
    return f"{what} on the wiki: /{path}"
