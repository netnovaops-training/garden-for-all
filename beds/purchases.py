# WHAT THIS FILE DOES (plain English)
# Receipts -> spending records. The free AI reads the plant lines off a
# receipt, each line is matched to a plant in the catalogue, and the result
# is saved in the plant_purchase table. The /spend page shows the totals.
# Everything here uses only the garden database.
"""What every plant cost, from the receipts.

The goal: track every plant bought over the years and get an accurate
price for each one.

THE CATALOGUE ALREADY HAS A PRICE FIELD AND THAT IS THE PROBLEM. There is
one `average_cost` per plant, so a hosta bought in 2019 for $8 and again in
2024 for $22 has one number, and whichever was written last wins. The
question Jun actually asks -- "what have I spent on hostas", "what did this
bed cost", "are they getting dearer" -- cannot be answered from a single
column at all. Purchases are events and events go in rows.

average_cost IS STILL KEPT UP TO DATE, because the website, the Android app
and the printed QR codes all read it. It is set from the MOST RECENT
purchase rather than the mean: somebody asking what a plant costs means
what it costs now, and a mean over five years answers a question nobody
asked.

EVERY LINE IS MATCHED AGAINST REAL CATALOGUE NAMES, and the match is
checked afterwards. A receipt says "HOSTA BLUE ANGEL 1G" and the catalogue
says "Blue Angel Hosta"; a model is good at that and terrible at admitting
when nothing fits. So it picks from a list, and a name it invented is
discarded and the line is left unmatched for a person -- an unmatched line
is visible and fixable, a wrongly matched one silently corrupts what Jun
believes he spent.

NOTHING IS GUESSED FROM A BLURRY RECEIPT. A price that cannot be read is
recorded as null, not as zero. Zero is a number and it will be summed.
"""
from __future__ import annotations

import json
import re

SCHEMA = """
CREATE TABLE IF NOT EXISTS plant_purchase (
  id            INT AUTO_INCREMENT PRIMARY KEY,
  plant_id      INT           NULL,
  line_text     VARCHAR(300)  NOT NULL,
  matched_name  VARCHAR(220)  DEFAULT '',
  quantity      INT           NOT NULL DEFAULT 1,
  unit_price    DECIMAL(10,2) NULL,
  line_total    DECIMAL(10,2) NULL,
  store         VARCHAR(200)  DEFAULT '',
  bought_on     DATE          NULL,
  receipt_ref   VARCHAR(200)  DEFAULT '',
  confidence    DECIMAL(4,3)  NULL,
  added_by      VARCHAR(190)  DEFAULT '',
  added_at      TIMESTAMP     DEFAULT CURRENT_TIMESTAMP,
  KEY k_plant (plant_id),
  KEY k_when (bought_on),
  KEY k_receipt (receipt_ref)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
"""


def ensure(cur):
    cur.execute(SCHEMA)


READ_RECEIPT = """Below is the text of a receipt from a garden centre or
nursery.

Pull out ONLY the lines that are plants. Ignore pots, soil, mulch, tools,
fertiliser, gift cards, tax, subtotals and the total.

ONLY USE WHAT IS PRINTED HERE. Do not work out a price that is not shown,
and do not guess at a smudged one -- use null. A wrong number here becomes
what Jun believes he spent.

RECEIPT:
---
{text}
---

Reply with JSON only:
{{
  "store":     the shop's name, or ""
  "date":      YYYY-MM-DD if a date is printed, else ""
  "lines": [
    {{ "text": the line as printed,
       "name": just the plant name, tidied,
       "quantity": a whole number, 1 if not shown,
       "unit_price": a number or null,
       "line_total": a number or null }}
  ]
}}

No markdown, no preamble. JSON only."""


MATCH_LINE = """A garden catalogue contains these plants:

{names}

A receipt line reads: "{line}"

Which catalogue plant is it? Answer JSON only:
  "name"       the EXACT name from the list, character for character, or ""
  "confidence" 0.0 to 1.0
  "why"        a few words

You may NOT invent a name. If nothing in the list is clearly the same
plant, answer "" -- an unmatched line is fixed in a second by a person, a
wrongly matched one quietly corrupts the record."""


def read_receipt(text, ask):
    """Store, date and plant lines from receipt text."""
    if not (text or "").strip():
        return {"store": "", "date": "", "lines": []}
    try:
        raw = ask(READ_RECEIPT.format(text=text[:14000]))
    except Exception:
        return {"store": "", "date": "", "lines": []}
    m = re.search(r"\{.*\}", raw, re.S)
    if not m:
        return {"store": "", "date": "", "lines": []}
    try:
        out = json.loads(m.group(0))
    except ValueError:
        return {"store": "", "date": "", "lines": []}

    lines = []
    for ln in out.get("lines") or []:
        if not isinstance(ln, dict):
            continue
        name = str(ln.get("name") or "").strip()
        if not name:
            continue
        lines.append({
            "text": str(ln.get("text") or name)[:300],
            "name": name[:220],
            "quantity": _int(ln.get("quantity"), 1),
            "unit_price": _money(ln.get("unit_price")),
            "line_total": _money(ln.get("line_total")),
        })
    return {"store": str(out.get("store") or "")[:200],
            "date": _date(out.get("date")),
            "lines": lines}


def _int(v, default=1):
    try:
        n = int(float(v))
        return n if 1 <= n <= 9999 else default
    except (TypeError, ValueError):
        return default


def _money(v):
    """A price, or None. NEVER zero for 'could not read'."""
    if v is None or v == "":
        return None
    try:
        n = round(float(str(v).replace("$", "").replace(",", "")), 2)
    except (TypeError, ValueError):
        return None
    return n if 0 <= n <= 100000 else None


def _date(v):
    m = re.match(r"(\d{4})-(\d{2})-(\d{2})", str(v or ""))
    return m.group(0) if m else None


def match_line(line_text, catalogue_names, ask):
    """Which plant this line is -- chosen from the list, then verified."""
    if not catalogue_names:
        return "", 0.0
    # The model sees names, not ids, and a long list buries the answer.
    listed = chr(10).join("  - " + n for n in catalogue_names[:400])
    try:
        raw = ask(MATCH_LINE.format(names=listed, line=line_text))
    except Exception:
        return "", 0.0
    m = re.search(r"\{.*\}", raw, re.S)
    if not m:
        return "", 0.0
    try:
        out = json.loads(m.group(0))
    except ValueError:
        return "", 0.0
    name = str(out.get("name") or "").strip()
    # CHECKED AGAINST THE REAL LIST. An invented name becomes no match.
    if name and name not in catalogue_names:
        return "", 0.0
    try:
        conf = max(0.0, min(1.0, float(out.get("confidence") or 0)))
    except (TypeError, ValueError):
        conf = 0.0
    return name, round(conf, 3)


def record(cur, line, plant_id, matched_name, store, bought_on,
           receipt_ref, who, confidence=None):
    """Save one plant line from a receipt as a row in plant_purchase."""
    ensure(cur)
    cur.execute(
        "INSERT INTO plant_purchase (plant_id, line_text, matched_name, "
        "quantity, unit_price, line_total, store, bought_on, receipt_ref, "
        "confidence, added_by) "
        "VALUES (%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s)",
        (plant_id, line["text"], matched_name, line["quantity"],
         line["unit_price"], line["line_total"], store, bought_on,
         receipt_ref, confidence, who))
    return cur.lastrowid


def refresh_cost(cur, plant_id, table="data_source"):
    """Set average_cost from the MOST RECENT purchase, not the mean.

    Somebody asking what a plant costs means what it costs now. A mean
    across five years of prices answers a question nobody asked, and gets
    further from the truth every time something is bought.
    """
    cur.execute(
        "SELECT unit_price FROM plant_purchase WHERE plant_id=%s "
        "AND unit_price IS NOT NULL ORDER BY bought_on DESC, id DESC LIMIT 1",
        (plant_id,))
    row = cur.fetchone()
    if not row:
        return None
    price = row["unit_price"]
    cur.execute(f"UPDATE {table} SET average_cost=%s WHERE id=%s",
                (price, plant_id))
    return price


def spent_on(cur, plant_id):
    """Every purchase of one plant, and the totals."""
    ensure(cur)
    cur.execute(
        "SELECT bought_on, store, quantity, unit_price, line_total "
        "FROM plant_purchase WHERE plant_id=%s ORDER BY bought_on DESC",
        (plant_id,))
    rows = cur.fetchall()
    total = sum(float(r["line_total"] or 0) for r in rows)
    count = sum(int(r["quantity"] or 0) for r in rows)
    return {"purchases": rows, "total_spent": round(total, 2),
            "plants_bought": count}


def summary(cur):
    """What the garden has cost, by year and by shop."""
    ensure(cur)
    # NOT `lines`. LINES is a reserved word in MariaDB (LOAD DATA ... LINES
    # TERMINATED BY) and an unquoted alias by that name is a syntax error,
    # which is how this returned a 500 the first time it was called.
    cur.execute(
        "SELECT YEAR(bought_on) y, COUNT(*) line_count, SUM(quantity) plants, "
        "ROUND(SUM(line_total),2) spent FROM plant_purchase "
        "WHERE bought_on IS NOT NULL GROUP BY y ORDER BY y DESC")
    by_year = cur.fetchall()
    cur.execute(
        "SELECT store, COUNT(*) line_count, ROUND(SUM(line_total),2) spent "
        "FROM plant_purchase WHERE store<>'' GROUP BY store "
        "ORDER BY spent DESC LIMIT 20")
    by_store = cur.fetchall()
    cur.execute("SELECT ROUND(SUM(line_total),2) t, SUM(quantity) n, "
                "SUM(plant_id IS NULL) unmatched FROM plant_purchase")
    overall = cur.fetchone() or {}
    return {"by_year": by_year, "by_store": by_store,
            "total_spent": overall.get("t"),
            "plants_bought": overall.get("n"),
            "unmatched_lines": overall.get("unmatched")}


def unmatched(cur, limit=40):
    """Lines that are a plant but not a plant the catalogue knows.

    SHOWN, NOT SWEPT UP. Each one is either a plant missing from the
    catalogue or a name the matcher was right to refuse, and both are worth
    a person's thirty seconds. Their money is already in the totals.
    """
    ensure(cur)
    cur.execute(
        "SELECT id, line_text, matched_name, quantity, unit_price, "
        "line_total, store, bought_on FROM plant_purchase "
        "WHERE plant_id IS NULL ORDER BY id DESC LIMIT %s", (int(limit),))
    rows = cur.fetchall()
    for r in rows:
        r["bought_on"] = str(r["bought_on"] or "")[:10]
        for k in ("unit_price", "line_total"):
            r[k] = None if r[k] is None else float(r[k])
    return rows


def name_it(cur, purchase_id, plant_id, plant_name, table="data_source"):
    """A person says which plant a line was. Their answer is taken as final."""
    ensure(cur)
    cur.execute("UPDATE plant_purchase SET plant_id=%s, matched_name=%s, "
                "confidence=1.000 WHERE id=%s",
                (plant_id, plant_name, int(purchase_id)))
    return refresh_cost(cur, plant_id, table)
