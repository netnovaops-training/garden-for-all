# WHAT THIS FILE DOES (plain English)
# Keeps the catalogue's words tidy. It learns the words each column already
# uses (for example the sun_exposure values) and matches new values onto
# them, so "part sun" and "Part Sun" do not become two different things.
"""Normalise incoming values onto the words a catalogue ALREADY uses.

THE PROBLEM THIS EXISTS FOR. Pull plant facts off the web and you get "part
sun", "Partial Sun", "sun/part shade", "Sun to Part Shade" and a dozen more
for one idea. Write them straight into a column and you have not gathered
data, you have destroyed a column: every chart, filter and count built on it
now splits one real category across sixteen spellings. That is unrecoverable
in practice, because by the time anybody notices there is no way to tell
which of the sixteen a given row MEANT.

THE VOCABULARY IS LEARNED, NOT WRITTEN DOWN. An earlier draft of this hard-
coded Jun's six sun values, read out of his database. That works for Jun and
is worthless as a product: the next gardener's catalogue has its own words,
its own categories, and possibly another language. So the canonical set is
whatever the target column already contains, with counts -- read at run time,
per field, per install. Point this at an empty database and it learns
nothing and proposes everything, which is correct.

THREE WAYS A VALUE CAN RESOLVE, cheapest first:

  1. EXACT     it already matches a value in the column.
  2. ALIAS     a human has previously said "part sun" means "Partial Shade"
               here, and that answer was remembered.
  3. NEAR      it is close enough to one existing value, and only one, to be
               offered as a suggestion -- never applied on its own.

Anything else is a PROPOSAL for a new canonical value, which a person
accepts or maps onto an existing one. Whatever they choose is written to the
alias table, so the same wording is never asked about twice. That is the
whole self-maintaining trick: the system gets quieter the longer it runs,
and the work per new gardener falls to nearly nothing after the first weeks.

NOTHING HERE WRITES TO THE CATALOGUE. It reads columns and the alias table
and returns decisions. Applying them is somebody else's job, after a person
has said yes.
"""
from __future__ import annotations

import difflib
import re
from dataclasses import dataclass, field as dc_field

# Fields worth policing. Open-ended text -- notes, names, descriptions -- is
# deliberately absent: there is nothing to normalise "care_notes" ONTO, and
# forcing free prose into a vocabulary is how real information is lost.
CONTROLLED = [
    "sun_exposure", "water_needs", "plant_type", "deer_resistance",
    "toxicity", "native_status", "growth_rate", "soil_type", "soil_ph",
    "season", "grow_season", "is_evergreen", "invasive_local",
    "fruit_edibility", "reproduction_type", "root_system", "leaf_type",
    "leaf_shape", "leaf_margin", "flower_shape", "flower_symmetry",
    "fruit_type", "life_expectancy",
]

# Below this, a near-match is not offered at all. Tuned to be cautious:
# a wrong suggestion that a tired person clicks through is worse than no
# suggestion, because it enters the data looking deliberate.
NEAR_ENOUGH = 0.86
# If two existing values are both close, offer NEITHER. Ambiguity is a
# question for a person, not a coin toss.
AMBIGUOUS_GAP = 0.06


def canon_key(s: str) -> str:
    """The form two spellings of one thing share.

    Case, punctuation, whitespace and the words that carry no meaning here
    are removed, so "Full Sun - Partial Shade", "full sun/partial shade" and
    "Full sun to partial shade" collapse to one key. This is what catches
    the overwhelming majority of duplicates before anything cleverer runs.
    """
    s = (s or "").lower().strip()
    s = re.sub(r"[‐-―]", "-", s)          # every kind of dash
    s = re.sub(r"\b(to|and|or|with|the|a|an)\b", " ", s)
    s = re.sub(r"[^a-z0-9]+", " ", s)
    return " ".join(sorted(s.split()))              # order-insensitive


@dataclass
class Decision:
    field: str
    raw: str
    value: str | None = None      # the canonical value, when resolved
    how: str = "proposed"         # exact | alias | near | proposed
    confidence: float = 0.0
    alternatives: list = dc_field(default_factory=list)
    note: str = ""

    @property
    def needs_person(self) -> bool:
        """Only an exact match or a remembered alias applies unattended."""
        return self.how not in ("exact", "alias")


class Vocabulary:
    """What one column already says, and what people have taught it."""

    def __init__(self, field: str, values: dict[str, int], aliases: dict[str, str]):
        self.field = field
        self.values = values                      # canonical -> row count
        self.aliases = aliases                    # canon_key(raw) -> canonical
        self._by_key: dict[str, str] = {}
        for v in values:
            # A column that ALREADY contains two spellings of one thing keeps
            # the more common one as canonical. Counts are the only honest
            # tie-breaker available without asking somebody.
            k = canon_key(v)
            if k not in self._by_key or values[v] > values[self._by_key[k]]:
                self._by_key[k] = v

    def resolve(self, raw: str) -> Decision:
        text = (raw or "").strip()
        if not text:
            return Decision(self.field, raw, note="empty")

        if text in self.values:
            return Decision(self.field, raw, text, "exact", 1.0)

        key = canon_key(text)
        if key in self.aliases:
            return Decision(self.field, raw, self.aliases[key], "alias", 1.0)
        if key in self._by_key:
            # Same words, different punctuation or order. Treated as exact,
            # because it is: nobody meant a new category by writing a slash
            # instead of a dash.
            return Decision(self.field, raw, self._by_key[key], "exact", 0.99)

        scored = sorted(
            ((difflib.SequenceMatcher(None, key, k).ratio(), v)
             for k, v in self._by_key.items()),
            reverse=True,
        )
        if not scored:
            return Decision(self.field, raw, None, "proposed", 0.0,
                            note="nothing in this column yet")

        best, runner = scored[0], (scored[1] if len(scored) > 1 else (0.0, None))
        if best[0] >= NEAR_ENOUGH and (best[0] - runner[0]) >= AMBIGUOUS_GAP:
            return Decision(self.field, raw, best[1], "near", best[0],
                            [v for _, v in scored[1:4]])
        if best[0] >= NEAR_ENOUGH:
            return Decision(self.field, raw, None, "proposed", best[0],
                            [v for _, v in scored[:3]],
                            "too close to call between existing values")
        return Decision(self.field, raw, None, "proposed", best[0],
                        [v for _, v in scored[:3]],
                        "no existing value is close")


def learn(cursor, table: str, fields=None, min_count: int = 1) -> dict:
    """Read the vocabulary straight out of the catalogue.

    ONE QUERY PER FIELD rather than one big one: the columns are unrelated
    and a single query would need a UNION per column anyway. Values below
    `min_count` can be excluded, which is how a column that already contains
    a handful of typos stops teaching them to everything that follows.
    """
    out = {}
    for f in (fields or CONTROLLED):
        try:
            # COUNTS ARE OF PLANTS THAT ARE ACTUALLY GROWING.
            #
            # These numbers are read aloud as answers -- "133 are toxic to
            # dogs and cats" -- so they have to describe the same garden
            # every other tool describes. Counting the dead as well made
            # this disagree with /api/filter by exactly the plants that are
            # gone (133 against 120), and two tools answering the same
            # question with different numbers is worse than either being
            # slightly off.
            #
            # The VOCABULARY still comes from every row: a word used only
            # by a plant that has since died is still a word this catalogue
            # uses, and dropping it would let the same spelling be invented
            # a second time. Only the counts are narrowed.
            cursor.execute(
                f"SELECT `{f}` AS v, COUNT(*) AS c FROM `{table}` "
                f"WHERE `{f}` IS NOT NULL AND `{f}` <> '' "
                f"AND COALESCE(died,0) = 0 "
                f"AND COALESCE(flower_bed,'') <> 'Dead or Removed' "
                f"GROUP BY `{f}` HAVING c >= %s ORDER BY c DESC",
                (min_count,),
            )
        except Exception:
            # A column this install does not have is not an error. The
            # product ships one field list and meets many schemas.
            continue
        out[f] = {r["v"]: r["c"] for r in cursor.fetchall()}
    return out


def load_aliases(cursor) -> dict:
    """Everything a person has already decided, by field."""
    aliases: dict[str, dict[str, str]] = {}
    try:
        cursor.execute(
            "SELECT field_name, raw_key, canonical FROM vocab_alias")
    except Exception:
        return aliases          # table not created yet; nothing learned yet
    for r in cursor.fetchall():
        aliases.setdefault(r["field_name"], {})[r["raw_key"]] = r["canonical"]
    return aliases


def build(cursor, table: str = "data_source", fields=None) -> dict:
    """Every field's vocabulary, ready to resolve against."""
    learned = learn(cursor, table, fields)
    aliases = load_aliases(cursor)
    return {f: Vocabulary(f, vals, aliases.get(f, {}))
            for f, vals in learned.items()}


def resolve_all(vocabs: dict, incoming: dict) -> dict:
    """A scraped record in; a decision per controlled field out.

    Fields with no vocabulary are passed through untouched rather than
    dropped -- an unpoliced column is not a forbidden one.
    """
    decisions, passthrough = {}, {}
    for f, raw in (incoming or {}).items():
        if not isinstance(raw, str) or not raw.strip():
            continue
        if f in vocabs:
            decisions[f] = vocabs[f].resolve(raw)
        else:
            passthrough[f] = raw.strip()
    return {"decisions": decisions, "passthrough": passthrough}

