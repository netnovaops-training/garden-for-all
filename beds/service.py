# WHAT THIS FILE DOES (plain English)
# This is the "beds" tools service for Jun's Garden: a small web server
# (Python's built-in http.server) that serves the garden tool pages
# (/plan, /checkup, /diagnose, /tag, /spend, /wiki-queue, /review) and the
# /api/... routes they use, plus the routes the phone app's camera tools
# reach through the Rust website. It listens on port 8025.
#
# WHO MAY USE IT: the sign-in gate in front of it (Authelia behind Caddy)
# puts the signed-in person's email in the X-Auth-Email header. Only emails
# listed in the BEDS_ALLOWED_EMAILS setting get in; everyone else gets 403.
# /healthz is the one open route (it only says "the service is up").
"""The beds tools service: identify, read a tag, look things up, propose.

WHAT IT IS FOR. A photograph taken in the garden becomes a set of proposed
field values, already spoken in this catalogue's own vocabulary, waiting for
a person to say yes. Nothing reaches `data_source` until somebody does.

THE WHOLE PATH, and every step is separate on purpose:

    photograph -> read (vision)        loose text, the tag's own words
               -> look up (DDG)        loose text, with source URLs
               -> gate (garden_ingest) this catalogue's words, or a question
               -> propose              a row in vocab_proposal, unapplied
               -> a person approves    yes/no, per field  (the /review page)
               -> apply                the only step that writes

AI IS FREE CLOUD AI ONLY (freeai.py): a Gemini free key first, then
OpenRouter's free models. With no key the service still starts; the tools
that need AI answer "AI is not set up yet" instead of failing.

NO PORT IS OPEN TO THE INTERNET. Caddy reaches it on 127.0.0.1 and the
Rust site reaches it on the compose network. The X-Auth-Email header is
trusted, which is only safe while nothing else can reach this port.
"""
from __future__ import annotations

import base64
import binascii
import json
import os
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

import pymysql

# These four are imported by their short names because service.py runs as a
# script from inside the package folder (see the Dockerfile CMD).
import dedupe
import freeai
import gemini
import tagphotos
import tagqueue

import garden_ingest as gi
from garden_ingest import lookup as lk
from garden_ingest import identify as ident
from garden_ingest import semantic, vocab
from garden_ingest import beds as bd
from garden_ingest import checkup as cu
from garden_ingest import wikidraft as wdft
from garden_ingest import purchases as pur
from garden_ingest import wikiapply
from garden_ingest import wikijs
from garden_ingest.review_page import REVIEW_HTML
from garden_ingest.plan_page import PLAN_HTML
from garden_ingest.checkup_page import CHECKUP_HTML
from garden_ingest.spend_page import SPEND_HTML
from garden_ingest.diagnose_page import DIAGNOSE_HTML
from garden_ingest.tagscan_page import TAGSCAN_HTML
from garden_ingest.queue_page import QUEUE_HTML

PORT = int(os.environ.get("PORT", "8025"))
TABLE = os.environ.get("GARDEN_TABLE", "data_source")

# WHO MAY USE THIS SERVICE. A comma-separated list of email addresses, for
# example "jun@example.com,friend@example.com". Compared in lower case.
# Empty means nobody gets in (safe default), and the 403 message says why.
ALLOWED = {e.strip().lower() for e in
           os.environ.get("BEDS_ALLOWED_EMAILS", "").split(",") if e.strip()}

# Where the pages live for a person's browser. Only used to build the "where
# to look" link in a receipt answer. "/beds" matches the Caddy mount.
PUBLIC_URL = os.environ.get("BEDS_PUBLIC_URL", "/beds").rstrip("/")

# The answer any AI tool gives when no free AI key is set.
AI_NOT_SET_UP = freeai.NOT_SET_UP

WIKI_NOT_SET_UP = ("Your wiki is not connected yet (WIKIJS_API_URL and "
                   "WIKIJS_API_KEY are not set). The draft is still waiting "
                   "here; nothing was lost.")


def db():
    """Open one connection to the garden database (settings come from env)."""
    return pymysql.connect(
        host=os.environ.get("DB_HOST", "db"),
        user=os.environ.get("DB_USER", "dbadmin"),
        password=os.environ.get("DB_PASS", ""),
        database=os.environ.get("DB_NAME", "juns_garden_db"),
        charset="utf8mb4", autocommit=True, connect_timeout=10,
        cursorclass=pymysql.cursors.DictCursor)


def llm_call(prompt: str) -> str:
    """One text answer from a FREE cloud model (freeai.py). Empty string
    when nothing answered: the text steps (matching a word to the
    catalogue's vocabulary, pulling fields out of search results) are
    helpers, and a tag still gets proposed without them."""
    return freeai.chat([{"role": "user", "content": prompt}], raise_busy=False)


def describe_image(data_url: str, prompt: str) -> str:
    """A FREE cloud vision model reads a photograph (freeai.py).

    Raises freeai.Busy when every free model is at its limit: reading the
    photo is the one step nothing else can stand in for, so the caller
    decides (a tag is queued and read later; identify/receipt say "later")."""
    return freeai.chat([{"role": "user", "content": [
        {"type": "text", "text": prompt},
        {"type": "image_url", "image_url": {"url": data_url}},
    ]}], raise_busy=True)


# ---------------------------------------------------------------------------
#  The pipeline
# ---------------------------------------------------------------------------
def build_proposal(cur, raw_fields: dict, plant_id=None, source=None) -> dict:
    """Loose fields -> what would be written, and what must be asked."""
    batch = gi.review(cur, raw_fields, TABLE, source)

    # The model gets a look at whatever string matching could not place, and
    # picks ONLY from words the column already contains.
    vocabs = vocab.build(cur, TABLE)
    still = {a["field"]: a for a in batch["asks"]}
    if still:
        decisions = vocab.resolve_all(vocabs, {f: a["raw"] for f, a in still.items()})
        picks = semantic.enrich(decisions["decisions"], vocabs)
        for f, (choice, conf) in picks.items():
            still[f]["suggested"] = choice
            still[f]["confidence"] = round(conf, 3)
            still[f]["how"] = "model"
    return batch


def save_proposals(cur, batch: dict, plant_id=None) -> list:
    """Write each open question as a row in vocab_proposal (status pending)."""
    ids = []
    for a in batch["asks"]:
        cur.execute(
            "INSERT INTO vocab_proposal "
            "(plant_id, field_name, raw_value, suggested, confidence, how, source) "
            "VALUES (%s,%s,%s,%s,%s,%s,%s)",
            (plant_id, a["field"], a["raw"][:500], a.get("suggested"),
             a.get("confidence") or 0, a.get("how") or "proposed",
             (a.get("source") or "")[:500] or None))
        ids.append(cur.lastrowid)
    return ids


class BadJSON(Exception):
    """The request body was not valid JSON."""


class Handler(BaseHTTPRequestHandler):
    """Answers one HTTP request. do_GET and do_POST pick the route."""

    # ---- small helpers for answering ---------------------------------------
    def _send(self, code, payload):
        """Answer with JSON."""
        body = json.dumps(payload, default=str).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _html(self, page: str):
        """Answer with one of the HTML pages."""
        body = page.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        # A queue page has no business in a shared cache, and a stale one
        # would show decisions that have already been made.
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _text(self, code, message: str):
        """Answer with a short plain-text message (used for pages)."""
        body = (message + "\n").encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _who(self):
        """The signed-in person's email, from the gate's X-Auth-Email header."""
        return (self.headers.get("X-Auth-Email") or "").strip().lower()

    def _allowed(self):
        """Is the signed-in person on the BEDS_ALLOWED_EMAILS list?"""
        who = self._who()
        return bool(who) and who in ALLOWED

    def _deny(self, path):
        """403 with a short plain message: JSON for /api, text for pages."""
        if not ALLOWED:
            msg = ("Nobody is allowed in yet. Put your email in the "
                   "BEDS_ALLOWED_EMAILS setting and restart the service.")
        elif not self._who():
            msg = "Please sign in first."
        else:
            msg = "Sorry, this email is not allowed to use the garden tools."
        if path.startswith("/api/"):
            return self._send(403, {"error": msg})
        return self._text(403, msg)

    def _json(self):
        """The request body as a dict. Raises BadJSON if it is not JSON."""
        n = int(self.headers.get("Content-Length") or 0)
        try:
            out = json.loads(self.rfile.read(n) or b"{}")
        except ValueError:
            raise BadJSON()
        if not isinstance(out, dict):
            raise BadJSON()
        return out

    def _no_ai(self):
        """True (and a friendly 503 already sent) when no free AI key is set."""
        if freeai.configured():
            return False
        self._send(503, {"error": AI_NOT_SET_UP, "ai_not_set_up": True})
        return True

    # ---- GET ---------------------------------------------------------------
    def do_GET(self):
        p = urlparse(self.path)
        if p.path == "/healthz":
            return self._healthz()
        if not self._allowed():
            return self._deny(p.path)
        try:
            return self._get(p)
        except Exception as e:  # noqa: BLE001
            # One line in the log, a plain message to the person. No
            # traceback is sent to the browser.
            print(f"GET {p.path} failed: {type(e).__name__}: {e}", flush=True)
            return self._send(500, {"error": "Something went wrong inside the "
                                    "beds service. Its log says what."})

    def _healthz(self):
        """Is the service up, and can it reach the database?"""
        try:
            with db() as cx, cx.cursor() as cur:
                gi.ensure(cur)
                cur.execute("SELECT COUNT(*) c FROM vocab_proposal WHERE status='pending'")
                pending = cur.fetchone()["c"]
        except Exception as e:  # noqa: BLE001
            print(f"healthz: database not reachable: {type(e).__name__}", flush=True)
            return self._send(503, {"ok": False, "error": "database not reachable"})
        return self._send(200, {"ok": True, "pending": pending,
                                "ai_set_up": freeai.configured(),
                                "wiki_set_up": not wikijs.check()})

    def _get(self, p):
        """All GET routes except /healthz. The person is already allowed."""
        # ---- the pages --------------------------------------------------
        pages = {"/plan": PLAN_HTML, "/checkup": CHECKUP_HTML,
                 "/spend": SPEND_HTML, "/diagnose": DIAGNOSE_HTML,
                 "/tag": TAGSCAN_HTML, "/wiki-queue": QUEUE_HTML,
                 "/review": REVIEW_HTML}
        page = pages.get(p.path.rstrip("/") or "/")
        if page:
            return self._html(page)

        # ---- the approval queue (used by /review) -----------------------
        if p.path == "/api/pending":
            with db() as cx, cx.cursor() as cur:
                gi.ensure(cur)
                cur.execute(
                    "SELECT * FROM vocab_proposal WHERE status='pending' "
                    "ORDER BY created_at DESC LIMIT 100")
                return self._send(200, {"pending": cur.fetchall()})

        if p.path == "/api/approved":
            with db() as cx, cx.cursor() as cur:
                gi.ensure(cur)
                cur.execute(
                    "SELECT * FROM vocab_proposal WHERE status='approved' "
                    "ORDER BY decided_at DESC LIMIT 200")
                return self._send(200, {"approved": cur.fetchall()})

        if p.path == "/api/review":
            return self._review(self._who())

        # ---- the bed planner (used by /plan and /checkup) ---------------
        if p.path == "/api/beds":
            with db() as cx, cx.cursor() as cur:
                bd.ensure(cur)
                cur.execute(
                    "SELECT b.*, "
                    "  (SELECT COUNT(*) FROM garden_planting p "
                    "     WHERE p.bed_id = b.id) AS planted "
                    "FROM garden_bed b ORDER BY b.name")
                rows = cur.fetchall()
            for r in rows:
                r["created_at"] = str(r.get("created_at") or "")
                r["updated_at"] = str(r.get("updated_at") or "")
            return self._send(200, {"beds": rows})

        if p.path == "/api/tray":
            # The plants waiting to be put somewhere, plus everything
            # already placed. One call, because the planner needs both to
            # draw a bed and nobody benefits from two round trips.
            q = (parse_qs(p.query).get("q") or [""])[0].strip()
            with db() as cx, cx.cursor() as cur:
                bd.ensure(cur)
                sql = (
                    "SELECT id, plant_name, botanical_name, plant_type, "
                    "       picture, flower_bed, bloom_color, foliage_color, "
                    "       bloom_months, grow_months, sun_exposure, "
                    "       plant_length_max_in, plant_width_max_in, "
                    "       spacing_max_in "
                    "FROM " + TABLE + " WHERE COALESCE(flower_bed,'') <> "
                    "'Dead or Removed'")
                args = []
                if q:
                    sql += (" AND (plant_name LIKE %s OR botanical_name "
                            "LIKE %s)")
                    args += ["%" + q + "%"] * 2
                sql += " ORDER BY plant_name LIMIT 600"
                cur.execute(sql, args)
                plants = cur.fetchall()
                cur.execute(
                    "SELECT p.*, d.plant_name FROM garden_planting p "
                    "LEFT JOIN " + TABLE + " d ON d.id = p.plant_id")
                placed = cur.fetchall()
            for r in plants:
                # The one number the planner draws with, decided here so
                # the page never has to guess which column to believe.
                r["footprint_in"] = bd.footprint(r)
                for k in ("plant_length_max_in", "plant_width_max_in",
                          "spacing_max_in"):
                    r[k] = float(r[k]) if r[k] is not None else None
                r["unsorted"] = ((r.get("flower_bed") or "")
                                 in ("", "To Be Sorted"))
            for r in placed:
                r["x"], r["y"] = float(r["x"]), float(r["y"])
                r["placed_at"] = str(r.get("placed_at") or "")
            return self._send(200, {"plants": plants, "placed": placed})

        if p.path == "/api/checkups":
            pid = (parse_qs(p.query).get("plant_id") or [""])[0]
            if not pid.isdigit():
                return self._send(400, {"error": "which plant?"})
            with db() as cx, cx.cursor() as cur:
                cu.ensure(cur)
                cur.execute(
                    "SELECT id, looked_at, consistent, confirm, asked_by, "
                    "asked_at FROM plant_checkup WHERE plant_id=%s "
                    "ORDER BY asked_at DESC LIMIT 20", (int(pid),))
                rows = cur.fetchall()
            for r in rows:
                r["asked_at"] = str(r["asked_at"])
            return self._send(200, {"checkups": rows})

        # ---- wiki drafts (used by /wiki-queue) ---------------------------
        if p.path == "/api/wiki-drafts":
            with db() as cx, cx.cursor() as cur:
                wdft.ensure(cur)
                rows = wdft.pending(cur)
                counts = wdft.counts(cur)
            for r in rows:
                r["created_at"] = str(r["created_at"])
                r["decided_at"] = str(r["decided_at"] or "")
            return self._send(200, {"drafts": rows, "counts": counts,
                                    "wiki_set_up": not wikijs.check()})

        # ---- the tag queue and its photographs --------------------------
        if p.path == "/api/tag-queue":
            # How the pile is going. The app polls this rather than
            # holding a request open.
            with db() as cx, cx.cursor() as cur:
                out = tagqueue.summary(cur)
            out["ai_set_up"] = freeai.configured()
            return self._send(200, out)

        if p.path == "/api/tag-photos":
            # The album for one plant, and the image itself. Served from
            # here (not the public web folder) so only allowed people see it.
            q = parse_qs(p.query)
            want = (q.get("file") or [""])[0]
            if want:
                # The path is built from the parts, never from the caller's
                # string: "../../etc/passwd" is then a filename that does
                # not exist rather than a traversal.
                parts = [x for x in want.split("/") if x not in ("", ".", "..")]
                if len(parts) != 2:
                    return self._send(404, {"error": "no such photograph"})
                # Second check, on the real location: follow any symbolic
                # link, then make sure the file is still inside the tags
                # folder and is a picture (these are only ever written as
                # images). Anything else is "no such photograph".
                root = tagphotos.ROOT.resolve()
                target = (tagphotos.ROOT / parts[0] / parts[1]).resolve()
                if (not target.is_relative_to(root)
                        or target.suffix.lower() not in (".jpg", ".jpeg", ".png", ".webp")
                        or not target.is_file()):
                    return self._send(404, {"error": "no such photograph"})
                data = target.read_bytes()
                self.send_response(200)
                self.send_header("Content-Type", "image/jpeg")
                self.send_header("Content-Length", str(len(data)))
                self.send_header("Cache-Control", "private, max-age=86400")
                self.end_headers()
                return self.wfile.write(data)

            try:
                pid = int((q.get("plant_id") or ["0"])[0])
            except ValueError:
                pid = 0
            if not pid:
                return self._send(400, {"error": "which plant?"})
            with db() as cx, cx.cursor() as cur:
                rows = tagphotos.for_plant(cur, pid)
            for r in rows:
                r["url"] = "api/tag-photos?file=" + r["filename"]
            return self._send(200, {"plant_id": pid, "photos": rows})

        # ---- receipts and spending (used by /spend) ---------------------
        if p.path == "/api/plant-names":
            with db() as cx, cx.cursor() as cur:
                cur.execute("SELECT plant_name FROM " + TABLE + " WHERE "
                            "plant_name IS NOT NULL AND plant_name<>'' "
                            "ORDER BY plant_name")
                return self._send(200, {"names": [r["plant_name"]
                                                  for r in cur.fetchall()]})

        if p.path == "/api/unmatched":
            with db() as cx, cx.cursor() as cur:
                return self._send(200, {"unmatched": pur.unmatched(cur)})

        if p.path == "/api/spend":
            with db() as cx, cx.cursor() as cur:
                return self._send(200, pur.summary(cur))

        return self._send(404, {"error": "no such route"})

    # ---- POST --------------------------------------------------------------
    def do_POST(self):
        p = urlparse(self.path)
        if not self._allowed():
            return self._deny(p.path)
        who = self._who()

        routes = {
            "/api/tag-queue": self._tag_queue_add,
            "/api/tag": self._tag,
            "/api/lookup": self._lookup,
            "/api/identify": self._identify,
            "/api/decide": self._decide,
            "/api/confirm": self._confirm,
            "/api/correct": self._correct,
            "/api/revert": self._revert,
            "/api/bed": self._bed_save,
            "/api/bed-delete": self._bed_delete,
            "/api/place": self._place,
            "/api/unplace": self._unplace,
            "/api/checkup": self._checkup,
            "/api/diagnose": self._diagnose,
            "/api/receipt": self._receipt,
            "/api/purchase-name": self._purchase_name,
            "/api/wiki-decide": self._wiki_decide,
        }
        handler = routes.get(p.path)
        if not handler:
            return self._send(404, {"error": "no such route"})
        try:
            return handler(who)
        except BadJSON:
            return self._send(400, {"error": "that request was not valid JSON"})
        except freeai.Busy as b:
            # Identify, receipt, diagnose: nothing to queue, so say it
            # plainly. (A tag never gets here: _tag queues it.)
            return self._send(503, {"error": str(b), "busy": True})
        except Exception as e:  # noqa: BLE001
            print(f"POST {p.path} failed: {type(e).__name__}: {e}", flush=True)
            return self._send(500, {"error": f"{type(e).__name__}: {e}"})

    # ---- the tag queue -----------------------------------------------------
    def _tag_queue_add(self, who):
        """MANY TAGS IN, A RECEIPT BACK, WORK AFTERWARDS.

        The only thing done while the phone waits is writing the
        photographs to disk. One tag takes about twenty seconds to read; a
        pile of twenty is minutes, and no phone holds a request open that
        long. The worker below reads them afterwards.
        """
        req = self._json()
        # [{"front": dataurl, "back": dataurl|null}, ...]
        items = req.get("tags") or []
        if not isinstance(items, list) or not items:
            return self._send(400, {
                "error": "send tags: [{front, back}] -- front and "
                         "back of each tag, back optional"})
        if len(items) > 100:
            return self._send(400, {
                "error": "100 tags at a time is the limit"})

        queued, dups, failed = [], 0, 0
        with db() as cx, cx.cursor() as cur:
            tagqueue.ensure(cur)
            for item in items:
                front = (item or {}).get("front")
                back = (item or {}).get("back")
                if not front:
                    failed += 1
                    continue
                h = tagqueue.image_hash(front)
                try:
                    fpath, _, _ = tagphotos.save(front, None, "front")
                    bpath = None
                    if back:
                        bpath, _, _ = tagphotos.save(back, None, "back")
                except Exception:  # noqa: BLE001
                    failed += 1
                    continue
                qid, status, note = tagqueue.add(
                    cur, fpath, bpath, h, who)
                if status == "duplicate":
                    dups += 1
                queued.append({"id": qid, "status": status,
                               "note": note})
            state = tagqueue.summary(cur)

        out = {"queued": len(queued), "duplicates": dups,
               "unreadable": failed, "items": queued,
               "waiting": state["waiting"]}
        if freeai.configured():
            # The worker is woken rather than waited for.
            threading.Thread(target=self._drain_tag_queue,
                             daemon=True).start()
        else:
            out["note"] = (AI_NOT_SET_UP + " The tag photos are saved and "
                           "will be read after AI is set up.")
        return self._send(200, out)

    def _drain_tag_queue(self):
        """Work through queued tags, one at a time, until none are left.

        ONE AT A TIME ON PURPOSE: running them in parallel would put a big
        load on a small machine. Slow and finishing beats fast and falling
        over. A FAILURE MARKS ITS OWN ROW AND MOVES ON, so one unreadable
        photograph does not stop the rest.
        """
        # ONE WORKER AT A TIME. Every upload wakes a worker and the retry
        # timer (retry_loop) wakes one too; a second one simply leaves.
        if not _DRAIN_LOCK.acquire(blocking=False):
            return
        try:
            self._drain_locked()
        finally:
            _DRAIN_LOCK.release()

    def _drain_locked(self):
        """The loop inside _drain_tag_queue (runs with the lock held)."""
        while True:
            if not freeai.configured() or freeai.busy():
                return          # the retry timer comes back later
            try:
                with db() as cx, cx.cursor() as cur:
                    item = tagqueue.claim(cur)
                if not item:
                    return
            except Exception:  # noqa: BLE001
                return

            qid = item["id"]
            try:
                shots = []
                for col in ("front_file", "back_file"):
                    rel = item.get(col)
                    if not rel:
                        continue
                    path = tagphotos.ROOT / rel
                    if path.is_file():
                        shots.append(
                            "data:image/jpeg;base64," + base64.b64encode(
                                path.read_bytes()).decode())
                if not shots:
                    with db() as cx, cx.cursor() as cur:
                        tagqueue.finish(cur, qid, "failed",
                                        note="the photographs are gone")
                    continue

                result = self._read_one_tag(shots, item.get("added_by") or "")
                with db() as cx, cx.cursor() as cur:
                    tagqueue.finish(
                        cur, qid,
                        result.get("status", "done"),
                        result.get("plant_id"),
                        result.get("note", ""))
            except freeai.Busy as b:
                # NOT A FAILURE: back in line, untouched, for the next try.
                try:
                    with db() as cx, cx.cursor() as cur:
                        cur.execute("UPDATE tag_queue SET status='waiting', note=%s "
                                    "WHERE id=%s", (str(b)[:500], qid))
                except Exception:  # noqa: BLE001
                    pass
                return
            except Exception as e:  # noqa: BLE001
                try:
                    with db() as cx, cx.cursor() as cur:
                        tagqueue.finish(cur, qid, "failed",
                                        note=str(e)[:500])
                except Exception:  # noqa: BLE001
                    pass

    def _read_one_tag(self, shots, who):
        """Read a tag and do everything that follows. Returns a summary.

        SPLIT OUT SO THE QUEUE AND THE LIVE ENDPOINT CANNOT DIVERGE: the
        worker draining the queue does exactly what one tap on the phone
        does.
        """
        tag = lk.read_tag(describe_image, shots)
        if not tag:
            return {"status": "failed", "plant_id": None,
                    "note": "could not read that as a plant tag"}
        tag.pop("_conflicts", None)
        name = tag.get("plant_name") or tag.get("botanical_name") or ""
        if not name:
            return {"status": "failed", "plant_id": None,
                    "note": "read the tag but found no name"}

        enriched = {}
        try:
            enriched = gemini.enrich({
                "plant_name": tag.get("plant_name") or name,
                "botanical_name": tag.get("botanical_name") or "",
                "company": tag.get("company") or ""})
        except Exception:  # noqa: BLE001
            pass
        merged = lk.merge(tag, {"fields": enriched, "sources": ["Gemini"]})

        with db() as cx, cx.cursor() as cur:
            gi.ensure(cur)
            seen = dedupe.find(cur, name, TABLE)
            if seen["status"] == "already_have":
                pid = (seen.get("match") or {}).get("id")
                batch = build_proposal(cur, merged, pid, "tag queue")
                save_proposals(cur, batch, pid)
                return {"status": "duplicate", "plant_id": pid,
                        "note": "already in the garden as %r (%.0f%% match)"
                                % ((seen.get("match") or {}).get("plant_name"),
                                   seen["score"] * 100)}
            cur.execute(
                "INSERT INTO " + TABLE + " (plant_name, needs_review) "
                "VALUES (%s, 1)", (name,))
            pid = cur.lastrowid
            batch = build_proposal(cur, merged, pid, "tag queue")
            ids = save_proposals(cur, batch, pid)
        return {"status": "done", "plant_id": pid,
                "note": "added %r, %d field(s) waiting for approval"
                        % (name, len(ids))}

    # ---- the camera tools --------------------------------------------------
    def _tag(self, who):
        """A photographed nursery tag -> a plant waiting to be approved.

        THE ROW IS CREATED HERE, flagged `needs_review = 1`, with only the
        name. Every other field arrives as a proposal against that plant,
        so each one is approved on its own on the /review page.

        BOTH SIDES OF THE TAG. `images` is a list, front first; `image`
        still works alone. The back of a nursery tag is where the spacing,
        the zones, the sun and water and the grower are printed.
        """
        req = self._json()
        # `images` (a list, front first) or `image` (one, as before).
        shots = req.get("images")
        if not shots:
            one = req.get("image")
            shots = [one] if one else []
        shots = [i for i in shots if i]
        if not shots:
            return self._send(400, {"error": "no image"})

        # NO AI SET UP -> the photo is kept in the tag queue, not lost, and
        # read once a key is added (the retry timer picks it up).
        if not freeai.configured():
            return self._tag_enqueue(
                shots, who, AI_NOT_SET_UP + " Your tag photo is saved in the "
                "tag queue and will be read after AI is set up.")

        # FREE AI BUSY -> THE TAG WAITS IN LINE, IT IS NOT LOST. The
        # photographs go into the same queue the bulk upload uses, and the
        # worker reads them when a free tier has room.
        try:
            return self._tag_read(req, shots, who)
        except freeai.Busy as b:
            return self._tag_enqueue(
                shots, who, "Saved in the tag queue: " + str(b) + " It is "
                "read automatically when a free AI has room, and the plant "
                "then waits for approval like any other tag.")

    def _tag_enqueue(self, shots, who, note):
        """Put one tag (front, and back if given) in the tag queue."""
        front = shots[0]
        back = shots[1] if len(shots) > 1 else None
        with db() as cx, cx.cursor() as cur:
            tagqueue.ensure(cur)
            fpath, _, _ = tagphotos.save(front, None, "front")
            bpath = tagphotos.save(back, None, "back")[0] if back else None
            qid, status, _ = tagqueue.add(
                cur, fpath, bpath, tagqueue.image_hash(front), who)
            state = tagqueue.summary(cur)
        return self._send(200, {
            "queued": True, "id": qid, "status": status,
            "waiting": state["waiting"], "read": {}, "asks": [],
            "sides_read": 0, "ai_set_up": freeai.configured(),
            "note": note})

    def _tag_read(self, req, shots, who):
        """The rest of _tag: read the tag and propose the plant."""
        tag = lk.read_tag(describe_image, shots)
        if not tag:
            return self._send(200, {
                "read": {}, "asks": [], "sides_read": len(shots),
                "note": "Could not read that as a plant tag."})

        conflicts = tag.pop("_conflicts", None)
        name = tag.get("plant_name") or tag.get("botanical_name") or ""

        # WHAT THE PLANT IS, asked separately from what the tag SAYS.
        # Reading a tag and knowing a plant are different skills, so the
        # name goes to Gemini (gemini.py) for the rest of the record.
        # GEMINI FIRST, THE TAG ON TOP: lk.merge prefers the tag, so
        # anything printed on the card beats anything the model inferred.
        enriched, gem_note = {}, ""
        if req.get("enrich", True) and name:
            seed = {"plant_name": tag.get("plant_name") or name,
                    "botanical_name": tag.get("botanical_name") or "",
                    "company": tag.get("company") or ""}
            try:
                enriched = gemini.enrich(seed)
            except Exception as e:  # noqa: BLE001
                # An enrichment that fails costs the extra fields, never
                # the scan: the tag has already been read.
                gem_note = str(e)[:200]

        web = ({"fields": {}, "sources": []}
               if enriched or not (req.get("enrich", True) and name)
               else lk.lookup(name, llm_call))
        merged = lk.merge(tag, {"fields": enriched, "sources": ["Gemini"]}
                          if enriched else web)

        plant_id, created = None, False
        with db() as cx, cx.cursor() as cur:
            gi.ensure(cur)
            if name:
                # ALREADY HERE? dedupe.py compares normalised words, so
                # "Hosta 'Blue Angel'" and "Blue Angel Hosta" are one plant.
                seen = dedupe.find(cur, name, TABLE)
                dup = {"status": seen["status"], "score": seen["score"],
                       "match": (seen.get("match") or {}).get("plant_name")}
                plant_id = ((seen.get("match") or {}).get("id")
                            if seen["status"] == "already_have" else None)

                if plant_id is None:
                    # NAME AND THE REVIEW FLAG ONLY. Everything else arrives
                    # as a proposal against this id.
                    cur.execute(
                        "INSERT INTO " + TABLE + " (plant_name, needs_review) "
                        "VALUES (%s, 1)", (name,))
                    plant_id = cur.lastrowid
                    created = True
            else:
                dup = {"status": "new", "score": 0.0, "match": None}

            # THE EVIDENCE IS KEPT, filed under the plant it produced, so a
            # field that looks wrong later can be checked against the tag.
            # A failure here must NOT lose the tag.
            album = []
            for idx, shot in enumerate(shots):
                side = "front" if idx == 0 else "back"
                try:
                    rel, w, h = tagphotos.save(shot, plant_id, side)
                    tagphotos.record(cur, plant_id, side, rel, w, h, who)
                    album.append({"side": side, "file": rel})
                except Exception as e:  # noqa: BLE001
                    album.append({"side": side, "error": str(e)[:120]})

            batch = build_proposal(cur, merged, plant_id,
                                   ", ".join(web.get("sources") or [])[:500])
            ids = save_proposals(cur, batch, plant_id)

        self._send(200, {
            "read": tag, "sources": web.get("sources"),
            "ready": batch["ready"], "asks": batch["asks"],
            "passthrough": batch["passthrough"], "proposal_ids": ids,
            "plant_id": plant_id, "created": created,
            "duplicate_check": dup,
            "sides_read": len(shots), "album": album,
            "enriched_fields": len(enriched),
            "enrich_error": gem_note,
            # Kept rather than resolved: where the two sides of one tag
            # printed different things, somebody should look.
            "conflicts": conflicts or {},
            "note": ("Added as \"{}\" and waiting for approval.".format(name)
                     if created else
                     ("Updated {} -- it was already in the garden.".format(name)
                      if plant_id else
                      "Read the tag but found no name, so nothing was added.")),
        })

    def _identify(self, who):
        """What plant is this? The garden is asked before the world is.

        TAKES A PHOTOGRAPH OR A NAME. A name works without AI (it is
        matched against the catalogue); a photograph needs the free AI.

        WRITES NOTHING, PROPOSES NOTHING. Identifying a plant somebody is
        holding is a question, not an import. /api/lookup is the next call
        if the answer is worth recording.
        """
        req = self._json()
        image, name = req.get("image"), (req.get("name") or "").strip()
        if not image and not name:
            return self._send(400, {"error": "send an image or a name"})
        if image and self._no_ai():
            return

        seen = ident.look(describe_image, image) if image else {"names": [name]}
        if not seen or not seen.get("names"):
            return self._send(200, {
                "candidates": [], "in_catalogue": False,
                "note": "I could not make a plant out of that photograph."})

        with db() as cx, cx.cursor() as cur:
            out = ident.identify(cur, llm_call, seen, TABLE)

        # OUTWARD ONLY IF THE GARDEN DID NOT KNOW (and only with AI, which
        # is what turns search results into fields).
        if req.get("enrich", True) and out.get("search_outward"):
            if freeai.configured():
                web = lk.lookup(out["search_outward"], llm_call)
                out["outward"] = {"fields": web["fields"], "sources": web["sources"]}
            else:
                out["ai_note"] = AI_NOT_SET_UP
        self._send(200, out)

    def _lookup(self, who):
        """A name -> proposals, no photograph needed (needs the free AI)."""
        req = self._json()
        name = (req.get("name") or "").strip()
        if not name:
            return self._send(400, {"error": "no name"})
        if self._no_ai():
            return
        web = lk.lookup(name, llm_call)
        with db() as cx, cx.cursor() as cur:
            gi.ensure(cur)
            batch = build_proposal(cur, web["fields"], req.get("plant_id"),
                                   ", ".join(web["sources"])[:500])
            ids = save_proposals(cur, batch, req.get("plant_id"))
        self._send(200, {"sources": web["sources"], "ready": batch["ready"],
                         "asks": batch["asks"], "proposal_ids": ids})

    # ---- beds --------------------------------------------------------------
    def _bed_save(self, who):
        """Create or reshape one bed.

        THE SHAPE IS VALIDATED HERE, not only in the page: a polygon with
        two corners or a coordinate half a mile away draws fine and breaks
        everything downstream.
        """
        req = self._json()
        name = (req.get("name") or "").strip()[:120]
        if not name:
            return self._send(400, {"error": "a bed needs a name"})
        shape, why = bd.clean_shape(req.get("shape"))
        if shape is None:
            return self._send(400, {"error": why})

        fields = (json.dumps(shape), (req.get("sun") or "").strip()[:60],
                  (req.get("soil") or "").strip()[:120],
                  (req.get("notes") or "").strip()[:500])
        with db() as cx, cx.cursor() as cur:
            bd.ensure(cur)
            bid = req.get("id")
            if bid:
                # A RENAME MUST CARRY THE PLANTS WITH IT. Placing a plant
                # writes this bed's NAME into data_source.flower_bed, so
                # renaming the bed without touching them would leave the
                # catalogue pointing at a bed that no longer exists.
                cur.execute("SELECT name FROM garden_bed WHERE id=%s",
                            (int(bid),))
                was = cur.fetchone()
                if not was:
                    return self._send(404, {"error": "no such bed"})
                try:
                    cur.execute(
                        "UPDATE garden_bed SET name=%s, shape=%s, sun=%s, "
                        "soil=%s, notes=%s WHERE id=%s",
                        (name,) + fields + (int(bid),))
                except pymysql.err.IntegrityError:
                    return self._send(409, {"error":
                                            f"there is already a bed called {name}"})
                moved = 0
                if was["name"] != name:
                    cur.execute(
                        "UPDATE " + TABLE + " SET flower_bed=%s "
                        "WHERE flower_bed=%s", (name, was["name"]))
                    moved = cur.rowcount
                    print(f"{who} renamed bed {bid} "
                          f"{was['name']!r} -> {name!r}; {moved} plant(s) "
                          f"followed", flush=True)
            else:
                try:
                    cur.execute(
                        "INSERT INTO garden_bed (name, shape, sun, soil, "
                        "notes, created_by) VALUES (%s,%s,%s,%s,%s,%s)",
                        (name,) + fields + (who,))
                    bid = cur.lastrowid
                except pymysql.err.IntegrityError:
                    return self._send(409, {"error":
                                            f"there is already a bed called {name}"})
        print(f"{who} saved bed {bid}: {name} "
              f"({bd.area_sqft(shape):.0f} sq ft)", flush=True)
        self._send(200, {"id": bid, "area_sqft": round(bd.area_sqft(shape), 1)})

    def _bed_delete(self, who):
        """Remove a bed. Its plantings go; NO PLANT RECORD IS TOUCHED."""
        bid = self._json().get("id")
        if not bid:
            return self._send(400, {"error": "which bed?"})
        with db() as cx, cx.cursor() as cur:
            bd.ensure(cur)
            cur.execute("SELECT name FROM garden_bed WHERE id=%s", (int(bid),))
            row = cur.fetchone()
            if not row:
                return self._send(404, {"error": "no such bed"})
            cur.execute("SELECT COUNT(*) c FROM garden_planting WHERE bed_id=%s",
                        (int(bid),))
            n = cur.fetchone()["c"]
            cur.execute("DELETE FROM garden_bed WHERE id=%s", (int(bid),))
        print(f"{who} deleted bed {bid} ({row['name']}), "
              f"{n} planting(s) unplaced; no plant records touched", flush=True)
        self._send(200, {"ok": True, "unplaced": n})

    def _place(self, who):
        """Put one plant in one bed, at a point inside it.

        PLACING ALSO FILES IT: dragging a plant into a real bed writes
        flower_bed at the same time as the planting row, so the "To Be
        Sorted" backlog clears as a side effect of using the planner.
        THE POINT MUST BE INSIDE THE BED.
        """
        req = self._json()
        try:
            bid, pid = int(req["bed_id"]), int(req["plant_id"])
            x, y = float(req["x"]), float(req["y"])
        except (KeyError, TypeError, ValueError):
            return self._send(400, {"error": "need bed_id, plant_id, x and y"})
        qty = max(1, min(999, int(req.get("qty") or 1)))

        with db() as cx, cx.cursor() as cur:
            bd.ensure(cur)
            cur.execute("SELECT name, shape FROM garden_bed WHERE id=%s", (bid,))
            bed = cur.fetchone()
            if not bed:
                return self._send(404, {"error": "no such bed"})
            shape, why = bd.clean_shape(bed["shape"])
            if shape is None:
                return self._send(500, {"error": "that bed's shape is broken: "
                                                 + str(why)})
            if not bd.inside(shape, x, y):
                return self._send(400, {"error": "that spot is outside the bed"})

            cur.execute("SELECT plant_name FROM " + TABLE + " WHERE id=%s", (pid,))
            plant = cur.fetchone()
            if not plant:
                return self._send(404, {"error": "no such plant"})

            cur.execute(
                "INSERT INTO garden_planting (bed_id, plant_id, x, y, qty, "
                "placed_by) VALUES (%s,%s,%s,%s,%s,%s)",
                (bid, pid, round(x, 2), round(y, 2), qty, who))
            planting_id = cur.lastrowid
            cur.execute("UPDATE " + TABLE + " SET flower_bed=%s WHERE id=%s",
                        (bed["name"], pid))
        print(f"{who} placed {plant['plant_name']} in {bed['name']} "
              f"at {x:.0f},{y:.0f}", flush=True)
        self._send(200, {"id": planting_id, "bed": bed["name"]})

    def _unplace(self, who):
        """Take one planting off the plan.

        flower_bed is LEFT ALONE unless this was the plant's last planting:
        a plant can be in a bed more than once.
        """
        pid = self._json().get("id")
        if not pid:
            return self._send(400, {"error": "which planting?"})
        with db() as cx, cx.cursor() as cur:
            bd.ensure(cur)
            cur.execute("SELECT plant_id FROM garden_planting WHERE id=%s",
                        (int(pid),))
            row = cur.fetchone()
            if not row:
                return self._send(404, {"error": "no such planting"})
            cur.execute("DELETE FROM garden_planting WHERE id=%s", (int(pid),))
            cur.execute("SELECT COUNT(*) c FROM garden_planting WHERE plant_id=%s",
                        (row["plant_id"],))
            left = cur.fetchone()["c"]
            if left == 0:
                cur.execute("UPDATE " + TABLE + " SET flower_bed='To Be Sorted' "
                            "WHERE id=%s", (row["plant_id"],))
        print(f"{who} unplaced planting {pid}; {left} left for that plant",
              flush=True)
        self._send(200, {"ok": True, "still_placed": left})

    # ---- check-ups ---------------------------------------------------------
    def _checkup(self, who):
        """Look at a photograph of one plant and write down what is visible.

        THE PHOTOGRAPH IS NOT KEPT. Only the reading is.
        """
        req = self._json()
        try:
            pid = int(req["plant_id"])
            raw = base64.b64decode((req.get("image") or "").split(",")[-1])
        except (KeyError, TypeError, ValueError, binascii.Error):
            return self._send(400, {"error": "need plant_id and a photograph"})
        if not raw:
            return self._send(400, {"error": "that photograph is empty"})
        if self._no_ai():
            return

        with db() as cx, cx.cursor() as cur:
            cu.ensure(cur)
            cur.execute("SELECT * FROM " + TABLE + " WHERE id=%s", (pid,))
            plant = cur.fetchone()
            if not plant:
                return self._send(404, {"error": "no such plant"})
            cur.execute(
                "SELECT looked_at, asked_at FROM plant_checkup "
                "WHERE plant_id=%s ORDER BY asked_at DESC LIMIT 1", (pid,))
            prev = cur.fetchone()
            if prev:
                prev["asked_at"] = str(prev["asked_at"])[:10]

        try:
            out = cu.ask(raw, plant, prev)
        except ValueError as e:
            return self._send(400, {"error": str(e)})
        except freeai.Busy:
            raise
        except Exception as e:  # noqa: BLE001
            # Say so plainly rather than returning an empty reading that
            # looks like a clean bill of health.
            return self._send(502, {"error":
                                    "the AI did not answer: %s" % e})
        if not (out.get("raw") or "").strip():
            return self._send(502, {"error": "the AI did not answer. Try "
                                    "again in a minute."})

        with db() as cx, cx.cursor() as cur:
            cur.execute(
                "INSERT INTO plant_checkup (plant_id, looked_at, consistent, "
                "confirm, raw, asked_by) VALUES (%s,%s,%s,%s,%s,%s)",
                (pid, out["looked_at"], out["consistent"], out["confirm"],
                 out["raw"], who))
            out["id"] = cur.lastrowid
        print(f"{who} checked {plant['plant_name']} "
              f"({len(raw)} bytes of photograph)", flush=True)
        out["plant_name"] = plant["plant_name"]
        self._send(200, out)

    # ---- receipts ----------------------------------------------------------
    READ_PHOTO = ("This is a photograph of a receipt. Type out every line of "
                  "it exactly as printed, in order, including prices. Do not "
                  "summarise, do not tidy, do not leave anything out. If a "
                  "character is unreadable write a question mark rather than "
                  "guessing at it. Plain text only.")

    def _receipt(self, who):
        """A receipt in (a photograph, or its text), a row per plant out.

        Everything is stored in the garden database (table plant_purchase),
        so the /spend page can show what the garden has cost.
        """
        req = self._json()

        if req.get("file") and not req.get("image") and not req.get("text"):
            # PDF receipts needed a separate document reader that is not
            # part of this copy.
            return self._send(400, {"error": "PDF receipts are not supported "
                                    "here. Send a photo of the receipt, or "
                                    "paste its text."})
        if not req.get("image") and not (req.get("text") or "").strip():
            return self._send(400, {"error": "send a photograph or the text "
                                    "of a receipt"})
        if self._no_ai():
            return

        if req.get("image"):
            text = describe_image(req["image"], self.READ_PHOTO)
            if not text.strip():
                return self._send(400, {"error": "the AI could not read that "
                                        "photograph"})
        else:
            text = (req.get("text") or "").strip()

        out = pur.read_receipt(text, llm_call)
        if not out["lines"]:
            return self._send(200, {
                "kept": 0, "lines": [],
                "message": "I read that receipt but found no plants on it. "
                           "Nothing has been recorded."})

        # The date on the slip, or the one Jun typed, or nothing -- never
        # today's date, which would make the year totals wrong for ever.
        bought = out["date"] or pur._date(req.get("date"))
        store = req.get("store") or out["store"]
        ref = str(req.get("ref") or "")[:200]

        with db() as cx, cx.cursor() as cur:
            pur.ensure(cur)
            cur.execute("SELECT id, plant_name FROM " + TABLE +
                        " WHERE plant_name IS NOT NULL AND plant_name<>''")
            rows = cur.fetchall()
            by_name = {r["plant_name"]: r["id"] for r in rows}
            names = sorted(by_name)

            kept = []
            for ln in out["lines"]:
                name, conf = pur.match_line(ln["text"], names, llm_call)
                pid = by_name.get(name)
                pur.record(cur, ln, pid, name, store, bought, ref, who, conf)
                if pid:
                    pur.refresh_cost(cur, pid, TABLE)
                kept.append({"line": ln["text"], "quantity": ln["quantity"],
                             "unit_price": ln["unit_price"],
                             "line_total": ln["line_total"],
                             "matched": name, "plant_id": pid,
                             "confidence": conf})

        matched = sum(1 for k in kept if k["plant_id"])
        spent = round(sum(float(k["line_total"] or 0) for k in kept), 2)
        print(f"{who} filed a receipt: {len(kept)} plant lines, "
              f"{matched} matched, ${spent:.2f}", flush=True)
        unmatched = len(kept) - matched
        msg = (f"{len(kept)} plant line{'' if len(kept) == 1 else 's'} "
               f"recorded, ${spent:.2f} in total.")
        if unmatched:
            # NAMED, NOT HIDDEN. An unmatched line is a plant the catalogue
            # does not know about yet. The money is counted either way.
            verb = "is" if unmatched == 1 else "are"
            msg += (f" {unmatched} {verb} not in the catalogue yet and "
                    "waiting for you to name.")
        self._send(200, {"kept": len(kept), "matched": matched,
                         "store": store, "date": bought, "spent": spent,
                         "lines": kept, "message": msg,
                         "where": PUBLIC_URL + "/spend"})

    def _purchase_name(self, who):
        """Say which plant an unmatched receipt line was.

        THE NAME IS CHECKED AGAINST THE CATALOGUE, so a typo cannot attach
        the price to nothing and look as though it had worked.
        """
        req = self._json()
        name = str(req.get("name") or "").strip()
        try:
            rid = int(req["purchase_id"])
        except (KeyError, TypeError, ValueError):
            return self._send(400, {"error": "need a purchase to name"})
        if not name:
            return self._send(400, {"error": "need a plant name"})

        with db() as cx, cx.cursor() as cur:
            cur.execute("SELECT id, plant_name FROM " + TABLE +
                        " WHERE plant_name=%s LIMIT 1", (name,))
            row = cur.fetchone()
            if not row:
                return self._send(404, {
                    "error": f'"{name}" is not a plant in the catalogue. '
                             "Add it there first, then name this line."})
            price = pur.name_it(cur, rid, row["id"], row["plant_name"], TABLE)
        print(f"{who} named purchase {rid} as {row['plant_name']}", flush=True)
        self._send(200, {"plant_id": row["id"], "name": row["plant_name"],
                         "average_cost": None if price is None else float(price)})

    def _diagnose(self, who):
        """One photograph: what plant is this, and is anything wrong with it.

        ONE DOOR FOR BOTH QUESTIONS. Naming the plant first means the
        check-up runs with that plant's own record as context, so normal
        silver foliage is not reported as a symptom.

        IT DOES NOT NEED THE PLANT TO BE IN THE CATALOGUE. If the garden
        knows it, the reading is filed against it; if not, nothing is filed.
        """
        req = self._json()
        image = req.get("image") or ""
        if not image.startswith("data:image/"):
            return self._send(400, {"error": "send a photograph"})
        try:
            raw = base64.b64decode(image.split(",")[-1])
        except (binascii.Error, ValueError):
            return self._send(400, {"error": "that photograph could not be read"})
        if not raw:
            return self._send(400, {"error": "that photograph is empty"})
        if self._no_ai():
            return

        # --- what is it ---------------------------------------------------
        seen = ident.look(describe_image, image)
        if not seen or not seen.get("names"):
            return self._send(200, {
                "identified": False, "candidates": [],
                "note": "I could not make a plant out of that photograph."})

        with db() as cx, cx.cursor() as cur:
            ident_out = ident.identify(cur, llm_call, seen, TABLE)

        # THE CATALOGUE MATCH IS ONLY TAKEN WHEN IT IS CONFIDENT. A wrong
        # record would feed the health reading facts about another plant.
        best = (ident_out.get("candidates") or [None])[0]
        plant, prev = None, None
        if best and (best.get("confidence") or 0) >= 0.7 and best.get("id"):
            with db() as cx, cx.cursor() as cur:
                cur.execute("SELECT * FROM " + TABLE + " WHERE id=%s",
                            (best["id"],))
                plant = cur.fetchone()
                if plant:
                    cu.ensure(cur)
                    cur.execute(
                        "SELECT looked_at, asked_at FROM plant_checkup "
                        "WHERE plant_id=%s ORDER BY asked_at DESC LIMIT 1",
                        (plant["id"],))
                    prev = cur.fetchone()
                    if prev:
                        prev["asked_at"] = str(prev["asked_at"])[:10]

        # A plant the garden does not have still gets a reading, from the
        # name the vision pass produced rather than from a database row.
        record = plant or {"plant_name": (seen["names"] or [""])[0],
                           "botanical_name": seen.get("botanical") or ""}

        # --- is anything wrong with it ------------------------------------
        try:
            out = cu.ask(raw, record, prev)
        except ValueError as e:
            return self._send(400, {"error": str(e)})
        except freeai.Busy:
            raise
        except Exception as e:  # noqa: BLE001
            return self._send(502, {
                "error": "the AI did not answer: %s" % e})

        if plant:
            with db() as cx, cx.cursor() as cur:
                cur.execute(
                    "INSERT INTO plant_checkup (plant_id, looked_at, "
                    "consistent, confirm, raw, asked_by) "
                    "VALUES (%s,%s,%s,%s,%s,%s)",
                    (plant["id"], out["looked_at"], out["consistent"],
                     out["confirm"], out["raw"], who))
                out["id"] = cur.lastrowid

        out.update({
            "identified": True,
            "name": record.get("plant_name") or "",
            "botanical_name": record.get("botanical_name") or "",
            "in_catalogue": bool(plant),
            "plant_id": plant["id"] if plant else None,
            "confidence": (best or {}).get("confidence"),
            "candidates": ident_out.get("candidates") or [],
            "filed": bool(plant),
        })
        print(f"{who} diagnosed {out['name'] or 'an unknown plant'} "
              f"({'in catalogue' if plant else 'not in catalogue'})",
              flush=True)
        self._send(200, out)

    # ---- wiki drafts -------------------------------------------------------
    def _wiki_decide(self, who):
        """Keep a draft (write it to the wiki) or throw it out.

        THE ONLY DOOR BETWEEN A SUGGESTION AND A WIKI PAGE. When the wiki is
        not connected (no WIKIJS_API_URL / WIKIJS_API_KEY), "keep" answers
        with a friendly message and the draft stays waiting.

        A FAILED WRITE IS RECORDED AS FAILED AND THE DRAFT CAN BE TRIED
        AGAIN, so a wiki that was briefly down costs a retry, not a tip.
        """
        req = self._json()
        try:
            did = int(req["id"])
        except (KeyError, TypeError, ValueError):
            return self._send(400, {"error": "which draft?"})
        keep = bool(req.get("keep"))
        if keep and wikijs.check():
            return self._send(503, {"ok": False, "error": WIKI_NOT_SET_UP,
                                    "wiki_not_set_up": True})

        with db() as cx, cx.cursor() as cur:
            wdft.ensure(cur)
            out = wdft.decide(cur, did, keep, who,
                              apply=wikiapply.apply_draft if keep else None)
        print(f"{who} {'kept' if keep else 'removed'} draft {did}: "
              f"{out.get('status')}", flush=True)
        code = 200 if out.get("ok") or not keep else 502
        return self._send(code, out)

    # ---- the approval queue ------------------------------------------------
    def _decide(self, who):
        """Yes or no on one proposal. Approving teaches the alias."""
        req = self._json()
        pid, ok = req.get("id"), bool(req.get("approve"))
        with db() as cx, cx.cursor() as cur:
            cur.execute("SELECT * FROM vocab_proposal WHERE id=%s", (pid,))
            row = cur.fetchone()
            if not row:
                return self._send(404, {"error": "no such proposal"})
            if not ok:
                # The second look's signature is cleared, not kept: deciding
                # again starts the row's life over.
                cur.execute("UPDATE vocab_proposal SET status='rejected', "
                            "decided_by=%s, decided_at=NOW(), reviewed_by=NULL, "
                            "reviewed_at=NULL, review_note=NULL WHERE id=%s",
                            (who, pid))
                return self._send(200, {"status": "rejected"})
            if not row["suggested"]:
                return self._send(400, {"error": "nothing suggested to approve"})

            # The decision is remembered FIRST. If the write below fails,
            # the lesson is still learned and the same question is not asked
            # again; the other order loses it.
            gi.remember(cur, row["field_name"], row["raw_value"],
                        row["suggested"], who)
            written, before = 0, None
            if row["plant_id"]:
                # READ BEFORE WRITING, so "revert" can put the old value back.
                cur.execute(
                    f"SELECT `{row['field_name']}` AS v FROM `{TABLE}` WHERE id=%s",
                    (row["plant_id"],))
                got = cur.fetchone()
                before = None if not got else got["v"]
                written = gi.apply_approved(cur, row["plant_id"],
                                            {row["field_name"]: row["suggested"]},
                                            who, TABLE)
            cur.execute("UPDATE vocab_proposal SET status='approved', "
                        "previous_value=%s, decided_by=%s, decided_at=NOW(), "
                        "reviewed_by=NULL, reviewed_at=NULL, review_note=NULL "
                        "WHERE id=%s",
                        (None if before is None else str(before)[:500], who, pid))
        self._send(200, {"status": "approved", "rows_written": written,
                         "remembered": True})

    # ---- the second look ---------------------------------------------------
    # THREE VERBS, AND THE SPLIT IS THE POINT. "Keep" (confirm) cannot change
    # a value by mistake; "correct" and "revert" are separate buttons.
    # Every one of them signs the row with who did it.

    def _review(self, who):
        """What has been approved, and what the second look has checked."""
        q = parse_qs(urlparse(self.path).query)
        scope = (q.get("show") or ["open"])[0]
        where = ("status='approved'" if scope == "open"
                 else "status IN ('approved','confirmed','reverted','rejected')")
        with db() as cx, cx.cursor() as cur:
            gi.ensure(cur)
            cur.execute(f"SELECT * FROM vocab_proposal WHERE {where} "
                        "ORDER BY (status='approved') DESC, "
                        "COALESCE(reviewed_at, decided_at) DESC LIMIT 300")
            rows = cur.fetchall()
            # The plant's name, so a row reads as "the hydrangea" and not as
            # "plant 412".
            ids = sorted({r["plant_id"] for r in rows if r["plant_id"]})
            names = {}
            if ids:
                marks = ",".join(["%s"] * len(ids))
                cur.execute(f"SELECT id, plant_name FROM `{TABLE}` "
                            f"WHERE id IN ({marks})", tuple(ids))
                names = {r["id"]: r["plant_name"] for r in cur.fetchall()}
            vocabs = vocab.learn(cur, TABLE)
        for r in rows:
            r["plant_name"] = names.get(r["plant_id"])
        counts = {}
        for r in rows:
            counts[r["status"]] = counts.get(r["status"], 0) + 1
        return self._send(200, {"rows": rows, "counts": counts,
                                "vocabulary": vocabs})

    def _confirm(self, who):
        """Looked at it; it is right. Writes nothing to the catalogue."""
        pid = self._json().get("id")
        with db() as cx, cx.cursor() as cur:
            cur.execute("UPDATE vocab_proposal SET status='confirmed', "
                        "reviewed_by=%s, reviewed_at=NOW() "
                        "WHERE id=%s AND status='approved'", (who, pid))
            if not cur.rowcount:
                return self._send(404, {"error": "not awaiting review"})
        return self._send(200, {"status": "confirmed"})

    def _correct(self, who):
        """It was approved as the wrong word. Fix the word and the lesson."""
        req = self._json()
        pid, value = req.get("id"), (req.get("value") or "").strip()
        if not value:
            return self._send(400, {"error": "no value"})
        with db() as cx, cx.cursor() as cur:
            cur.execute("SELECT * FROM vocab_proposal WHERE id=%s", (pid,))
            row = cur.fetchone()
            if not row or row["status"] not in ("approved", "confirmed"):
                return self._send(404, {"error": "not awaiting review"})

            # THE SAME GUARD THE MODEL GETS: only a word this field already
            # uses, so a new spelling cannot sneak in by hand.
            allowed = vocab.learn(cur, TABLE).get(row["field_name"]) or {}
            if allowed and value not in allowed:
                return self._send(400, {
                    "error": "not a word this field uses",
                    "field": row["field_name"], "options": sorted(allowed)})

            # The alias is CORRECTED, not added to.
            gi.remember(cur, row["field_name"], row["raw_value"], value, who)
            written = 0
            if row["plant_id"]:
                written = gi.apply_approved(
                    cur, row["plant_id"], {row["field_name"]: value}, who, TABLE)
            cur.execute(
                "UPDATE vocab_proposal SET suggested=%s, status='confirmed', "
                "reviewed_by=%s, reviewed_at=NOW(), review_note=%s WHERE id=%s",
                (value, who, "corrected from {!r}".format(row["suggested"])[:500],
                 pid))
        return self._send(200, {"status": "confirmed", "value": value,
                                "rows_written": written})

    def _revert(self, who):
        """Undo the approval: the field, and the lesson it taught."""
        pid = self._json().get("id")
        with db() as cx, cx.cursor() as cur:
            cur.execute("SELECT * FROM vocab_proposal WHERE id=%s", (pid,))
            row = cur.fetchone()
            if not row or row["status"] not in ("approved", "confirmed"):
                return self._send(404, {"error": "nothing to revert"})

            # THE ALIAS GOES FIRST, mirroring _decide for the same reason.
            cur.execute("DELETE FROM vocab_alias WHERE field_name=%s AND raw_key=%s",
                        (row["field_name"], vocab.canon_key(row["raw_value"])))
            forgotten = cur.rowcount

            restored = 0
            if row["plant_id"]:
                # previous_value NULL means the field was empty before it was
                # approved, and empty is what it goes back to.
                restored = gi.apply_approved(
                    cur, row["plant_id"],
                    {row["field_name"]: row["previous_value"]}, who, TABLE)
            cur.execute(
                "UPDATE vocab_proposal SET status='reverted', reviewed_by=%s, "
                "reviewed_at=NOW(), review_note=%s WHERE id=%s",
                (who, "reverted to {!r}".format(row["previous_value"])[:500], pid))
        return self._send(200, {"status": "reverted", "alias_forgotten": forgotten,
                                "rows_restored": restored,
                                "restored_to": row["previous_value"]})

    def log_message(self, *a):
        """Silence the default one-line-per-request log (it is noisy)."""
        pass


_DRAIN_LOCK = threading.Lock()


def retry_loop():
    """Every 15 minutes, read whatever is waiting in the tag queue.

    Tags that arrived while every free model was busy (or before AI was
    set up) are picked up on the next round where one answers.
    `Handler.__new__` gives a handler without a request, which is all the
    queue worker needs (it never answers an HTTP request)."""
    while True:
        time.sleep(int(os.environ.get("GARDEN_QUEUE_RETRY", "900")))
        try:
            if freeai.configured() and not freeai.busy():
                Handler.__new__(Handler)._drain_tag_queue()
        except Exception as e:  # noqa: BLE001
            print(f"tag queue retry: {e}", flush=True)


if __name__ == "__main__":
    # A start-up summary for the log. It says WHETHER things are set, never
    # what the keys or passwords are.
    print(f"beds service on 0.0.0.0:{PORT}", flush=True)
    print(f"  allowed emails: {len(ALLOWED)}"
          + ("" if ALLOWED else "  (set BEDS_ALLOWED_EMAILS, or nobody gets in)"),
          flush=True)
    print(f"  free AI set up: {'yes' if freeai.configured() else 'no'}", flush=True)
    print(f"  wiki connected: {'yes' if not wikijs.check() else 'no'}", flush=True)
    threading.Thread(target=retry_loop, daemon=True).start()
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()
