# WHAT THIS FILE DOES (plain English)
# When a tag says "Medium" and the catalogue says "Average", plain spelling
# checks cannot see they mean the same. This asks the free AI to pick which
# EXISTING catalogue word a value means (or none). It may never invent a
# new word. With no AI, it simply has no opinion.
"""Ask a model which existing word a value means. It may not invent one.

WHY THIS EXISTS. String distance is the wrong tool for the last mile.
Measured against Jun's live catalogue:

    "Medium"          vs "Average"    -> 0.22
    "Moderate grower" vs "Moderate"   -> 0.70
    "Non toxic to pets" vs "Non-toxic"-> 0.78

Every one of those is obvious to a gardener and invisible to a matcher,
because the letters do not overlap. A model closes exactly that gap, and
nothing else in the pipeline can.

THE MODEL PICKS FROM A LIST. It is handed the values the column already
contains and must answer with one of them, or with NONE. It is never asked
"what should this be" -- only "which of these is it". That single constraint
is what makes a language model safe here: it cannot produce a seventeenth
spelling of "Partial Shade", because it is choosing, not writing.

ITS ANSWER IS STILL A SUGGESTION. A match from here goes in front of a
person like any other. The value is that the person is confirming an obvious
mapping instead of being asked an open question -- one tap rather than a
decision -- and once confirmed it becomes an alias and is never asked again.

THE MODEL IS ALLOWED TO BE UNREACHABLE. Every failure path returns "no
opinion", and the caller falls back to the string result. A garden catalogue
must not stop accepting data because an AI service is busy.
"""
from __future__ import annotations

import json
import os

# How long to wait for the free AI to pick a word (seconds).
TIMEOUT = int(os.environ.get("GARDEN_LLM_TIMEOUT", "25"))

PROMPT = """You map plant-catalogue values onto a fixed list.

FIELD: {field}
THE ONLY ALLOWED ANSWERS: {options}
VALUE TO MAP: {value}

Answer with JSON and nothing else:
{{"choice": "<exactly one of the allowed answers, copied character for character>", "confidence": <0.0-1.0>}}

Rules:
- If the value clearly means one of the allowed answers, choose it. "Medium"
  means "Average" if that is in the list; "Moderate grower" means "Moderate".
- If the value spans two of them (a range like "sun to part shade"), choose
  the allowed answer that expresses the range, if one exists.
- If nothing in the list means it, answer {{"choice": null, "confidence": 0}}.
- NEVER invent a value that is not in the allowed answers.
- A negation flips the meaning: "not deer resistant" is not "Deer Resistant".
"""


def match(field: str, value: str, options: list[str]) -> tuple[str | None, float]:
    """Which allowed option `value` means, and how sure. (None, 0.0) if unsure."""
    if not value or not options:
        return None, 0.0

    # FREE CLOUD MODELS ONLY (freeai.py), since 2026-10-07 (no GPU).
    # Temperature is 0 there: this is a lookup, and the same value must map
    # the same way every time or the catalogue drifts one row at a time.
    # Busy or empty = no opinion; the caller still has its string match.
    import freeai
    text = freeai.chat([{"role": "user", "content": PROMPT.format(
        field=field,
        options=json.dumps(options, ensure_ascii=False),
        value=json.dumps(value, ensure_ascii=False),
    )}], max_tokens=120, timeout=TIMEOUT, raise_busy=False).strip()
    if not text:
        return None, 0.0

    # Models fence JSON in markdown however often you ask them not to.
    if text.startswith("```"):
        text = text.strip("`")
        text = text.split("\n", 1)[-1] if "\n" in text else text
        text = text.rsplit("```", 1)[0]
    try:
        out = json.loads(text[text.index("{"):text.rindex("}") + 1])
    except (ValueError, IndexError):
        return None, 0.0

    choice = out.get("choice")
    try:
        conf = float(out.get("confidence") or 0)
    except (TypeError, ValueError):
        conf = 0.0

    # THE GUARD THAT MAKES THIS SAFE. Whatever came back must be one of the
    # options, character for character. A model that paraphrased, corrected
    # the capitalisation or helpfully improved the wording is rejected --
    # that is precisely the seventeenth-spelling problem this exists to stop.
    if choice not in options:
        return None, 0.0
    return choice, max(0.0, min(conf, 1.0))


def enrich(decisions: dict, vocabs: dict) -> dict:
    """Give the model a look at everything the string matcher could not place.

    Mutates nothing: returns a new dict of field -> (choice, confidence) for
    the caller to fold into its proposals.
    """
    out = {}
    for field, d in decisions.items():
        if not d.needs_person:
            continue
        options = list(vocabs[field].values) if field in vocabs else []
        choice, conf = match(field, d.raw, options)
        if choice:
            out[field] = (choice, conf)
    return out
