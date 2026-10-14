# WHAT THIS FILE DOES (plain English)
# After a tag is read, this asks Google Gemini (with the same free
# GEMINI_API_KEY) to fill in the rest of the plant's record from its name,
# using the long prompt in _gemini_prompt.json. With no key it quietly
# returns nothing, and the tag scan carries on without the extra fields.
"""Fill a plant record out properly, using the garden's own Gemini prompt.

The idea: take the name, genus and other names from the tag, ask Gemini
for a full JSON record, and offer every field back for approval -- so
adding a plant is not field-by-field typing.

WHY THIS IS THE RIGHT SPLIT, and not just a better model. The tag path
asked ONE vision pass to do two different jobs at once: read what is
printed on a piece of card, and know things about the plant. Run against a
real Proven Winners tag on 2026-09-26 it answered `zones: 10-11` where the
tag prints "Hardy in zones 3-8 to -40F", invented a genus of "Campanula &
Digitalis", and turned the PAIR WITH line -- Kniphofia, Red Hot Poker --
into a bloom colour of "Red Hot Pink". In Clifton Park the zones error
alone is the difference between a plant that overwinters and one that does
not.

Reading a tag and knowing a plant are separate skills. Vision does the
first from the photograph; this does the second from the NAME, which is
all it needs and all it should be trusted with.

THE PROMPT IS KEPT VERBATIM in `_gemini_prompt.json` beside this file --
the garden's location context with its real hardiness zone and chill
hours, the complete field schema with its closed enum lists, and the five
pipeline steps. A schema that is subtly different from the database's is
worse than none, so edit that file with care.

ITS BEST RULE, KEPT: "If you are not certain about a value, leave the field
as an empty string rather than fabricating data." That is the same
instruction the rest of this stack runs on and the one the vision pass was
ignoring.

NOTHING IS WRITTEN FROM HERE. Every field comes back as a proposal for
approval, exactly as a tag read does. Gemini is a better-informed source,
not a trusted one.
"""
from __future__ import annotations

import json
import os
import pathlib
import time
import urllib.error
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent
PROMPT_FILE = HERE / "_gemini_prompt.json"

API = ("https://generativelanguage.googleapis.com/v1beta/models/"
       "{model}:generateContent?key={key}")
# gemini-2.5-flash is retired for new keys -- the API says so in the body
# of a 404, which is why this module surfaces that body rather than "HTTP
# 404". Overridable, because this name will age too.
MODEL = os.environ.get("GEMINI_MODEL", "gemini-3.8-flash")
TIMEOUT = int(os.environ.get("GEMINI_TIMEOUT", "180"))

# Gemini must never change these, whatever it thinks. They are the
# garden's own facts -- who planted it, where, how many -- and no amount of
# botanical knowledge is evidence about them. (These are column names.)
READ_ONLY = {
    "id", "plant_name", "quantity", "qty", "year_added", "year_planted",
    "garden_beds", "flower_bed", "picture", "additional_pictures",
    "image_search_term", "approved_jason", "approved_jun",
    "needs_review", "image_verified",
}


# WHERE THE KEY CAN COME FROM: the GEMINI_API_KEY setting (normal), or a
# file (GEMINI_KEY_FILE, default /app/.gemini-key) as a fallback. The
# setting always wins when both exist.
KEY_FILE = pathlib.Path(os.environ.get("GEMINI_KEY_FILE", "/app/.gemini-key"))


def api_key() -> str:
    """The Gemini key, or "" when none is set."""
    key = os.environ.get("GEMINI_API_KEY", "").strip()
    if key:
        return key
    try:
        return KEY_FILE.read_text(encoding="utf-8").strip()
    except OSError:
        return ""


def available() -> bool:
    """Is there a key and a prompt to use it with?"""
    return bool(api_key()) and PROMPT_FILE.is_file()


def _prompt(records: list) -> str:
    """The full prompt text: the saved parts, then the plant(s) as JSON."""
    parts = json.loads(PROMPT_FILE.read_text(encoding="utf-8"))
    return (parts["HEADER"] + parts["CONTEXT"] + parts["SECTION1"]
            + parts["SECTION2"] + chr(10)
            + json.dumps(records, indent=2, ensure_ascii=False,
                         default=str)
            + chr(10))


def _answer_text(body: dict) -> str:
    """The model's text, whatever shape the response came back in."""
    for cand in body.get("candidates") or []:
        for part in (cand.get("content") or {}).get("parts") or []:
            if part.get("text"):
                return part["text"]
    return ""



# HOW LONG TO KEEP TRYING, AND WHY THIS IS NOT OPTIONAL.
#
# Gemini often answers 503 "currently experiencing high demand" and then
# works a few seconds later, so a few retries with a growing wait are worth
# it. Only the errors worth retrying are retried: 429 (rate limited), 500,
# 502, 503 and 504 are the service saying "not now". A 400 or a 403 is the
# service saying "not ever", and repeating it wastes time and quota.
RETRY_ON = (429, 500, 502, 503, 504)
RETRIES = int(os.environ.get("GEMINI_RETRIES", "4"))


def _post(body: bytes, model: str | None = None) -> dict:
    """One call, with backoff on the failures that pass."""
    url = API.format(model=(model or MODEL), key=api_key())
    wait = 3.0
    last = ""
    for attempt in range(RETRIES):
        req = urllib.request.Request(url, data=body, method="POST")
        req.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(req, timeout=TIMEOUT) as r:
                return json.loads(r.read())
        except urllib.error.HTTPError as e:
            last = e.read()[:240].decode("utf-8", "replace")
            if e.code not in RETRY_ON:
                raise GeminiUnavailable(
                    "Gemini said {}: {}".format(e.code, last))
            code = e.code
        except Exception as e:  # noqa: BLE001
            last, code = str(e)[:200], 0
        if attempt < RETRIES - 1:
            time.sleep(wait)
            wait *= 2
    raise GeminiUnavailable(
        "Gemini unavailable after {} tries: {}".format(RETRIES, last))


def enrich(record: dict, model: str | None = None) -> dict:
    """A sparse plant record -> a filled one. Returns {} on any failure.

    NEVER RAISES, and that is deliberate: this runs inside a tag scan that
    has already read a tag and created a plant. An enrichment that fails
    should cost the extra fields, never the scan.
    """
    key = api_key()
    if not key or not PROMPT_FILE.is_file():
        return {}

    body = json.dumps({
        "contents": [{"parts": [{"text": _prompt([record])}]}],
        "generationConfig": {
            # Zero, for the same reason everything else here is zero:
            # variety in a factual record is called an error.
            "temperature": 0,
            "maxOutputTokens": 8192,
            "responseMimeType": "application/json",
        },
    }).encode()

    answer = _post(body, model)

    text = _answer_text(answer).strip()
    if not text:
        raise GeminiUnavailable("Gemini answered with nothing.")
    if text.startswith("```"):
        text = text.strip("`").split("\n", 1)[-1].rsplit("```", 1)[0]

    try:
        got = json.loads(text[text.index("["):text.rindex("]") + 1]
                         if "[" in text else text)
    except (ValueError, IndexError):
        raise GeminiUnavailable("Gemini's answer was not readable JSON.")
    if isinstance(got, dict):
        got = [got]
    if not got:
        return {}

    out = dict(got[0]) if isinstance(got[0], dict) else {}
    # THE GUARD, ENFORCED HERE AS WELL AS ASKED FOR IN THE PROMPT. A rule
    # a model is told to follow is a hope; a key that is deleted on the way
    # back is a rule.
    for k in list(out):
        if k in READ_ONLY:
            out.pop(k, None)
    # Empty strings mean "not confident" and must not overwrite anything.
    # And anything underscored is the model talking about its own answer --
    # `_enriched_fields`, `_missing_fields` -- which is useful to read and
    # is not a column. A backfill dry run showed both being offered as
    # plant data.
    return {k: v for k, v in out.items()
            if v not in (None, "", [], {}) and not k.startswith("_")}


class GeminiUnavailable(Exception):
    """Gemini could not answer. The scan continues without it."""

