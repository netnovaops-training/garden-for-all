# WHAT THIS FILE DOES (plain English)
# Every AI question in the beds service comes through chat() here. It asks
# free cloud AI only: Google Gemini with a free key first, then OpenRouter's
# free models. No key at all is fine: configured() says False and the
# service shows NOT_SET_UP instead of calling anything. Never a local GPU.
"""The garden's AI, from FREE cloud tiers only. Never a local GPU.

The rule: free APIs only, and when the free limit is used up, queue the
work for the next time the API has room (a tag waits in the tag queue).

So every model call in the beds service comes through `chat()`, which
tries, in order (each list is a setting, comma-separated):

  1. Google Gemini, FREE key         GARDEN_GEMINI_MODELS
     (GEMINI_API_KEY, or the /app/.gemini-key file gemini.py also reads)
  2. OpenRouter's ":free" models     GARDEN_OPENROUTER_MODELS
     (OPENROUTER_API_KEY)

All of them speak the same OpenAI-style chat format, photographs included
(as data: URLs), so one request shape serves every provider.

BUSY IS NOT FAILURE. Free tiers answer 429 (limit reached) and 503 ("high
demand") a lot. When every model says so, `chat()` raises `Busy` instead of
returning nothing, and remembers it for a while (`_busy_until`) so the next
call fails fast instead of knocking on every door again. Callers decide
what busy means: a tag goes into the tag queue and is read later
(service.py), identify and receipt tell the person to try again later.

A local model on a graphics card is NOT a fallback, on purpose.
"""
from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.request

GEMINI_URL = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"

GEMINI_MODELS = [m.strip() for m in os.environ.get(
    "GARDEN_GEMINI_MODELS", "gemini-flash-latest,gemini-flash-lite-latest").split(",") if m.strip()]
OPENROUTER_MODELS = [m.strip() for m in os.environ.get(
    "GARDEN_OPENROUTER_MODELS",
    "google/gemma-4-31b-it:free,google/gemma-4-26b-a4b-it:free").split(",") if m.strip()]

# How long to stop trying after everything said busy. A daily quota resets
# on Google's clock, a "high demand" spike passes in minutes; 15 minutes is
# a fair middle and the queue simply tries again.
COOLDOWN = int(os.environ.get("GARDEN_FREEAI_COOLDOWN", "900"))
_busy_until = 0.0
_last_reason = ""

# One model that just said busy (or hung) is skipped for a few minutes, so
# the next photo goes straight to one that has room instead of waiting on
# the same overloaded door again. Measured 2026-10-07: without this, one
# Identify took 2 minutes of waiting on busy models before one answered.
MODEL_COOLDOWN = int(os.environ.get("GARDEN_FREEAI_MODEL_COOLDOWN", "300"))
# Per attempt. A free model that has room answers a photo in seconds; one
# that has not answered in 45 is overloaded, and the next one is tried.
ATTEMPT_TIMEOUT = int(os.environ.get("GARDEN_FREEAI_ATTEMPT_TIMEOUT", "45"))
_model_rest = {}


# What a person sees when no AI key is set. Guide 03 part D explains how
# to get a free key and where to put it.
NOT_SET_UP = ("AI is not set up yet, see guide 03 part D "
              "(START-HERE/03-run-your-garden.md).")


class Busy(Exception):
    """Every free model is at its limit or overloaded right now."""


def configured() -> bool:
    """Is at least one free AI key set? (Says yes/no, never the key.)"""
    return bool(_gemini_key() or os.environ.get("OPENROUTER_API_KEY", "").strip())


def busy() -> bool:
    """True while we are waiting out a 'everything is busy' cool-down."""
    return time.time() < _busy_until


def busy_message() -> str:
    mins = max(1, int((_busy_until - time.time()) // 60) + 1)
    return ("The free AI readers are all busy or at today's free limit "
            f"({_last_reason}). Trying again in about {mins} minutes.")


def _gemini_key() -> str:
    """The Gemini key from the environment, or from the key file."""
    key = os.environ.get("GEMINI_API_KEY", "").strip()
    if key:
        return key
    try:
        with open(os.environ.get("GEMINI_KEY_FILE", "/app/.gemini-key"), encoding="utf-8") as f:
            return f.read().strip()
    except OSError:
        return ""


def _providers():
    """Every (provider, url, key, model) to try, in order."""
    g = _gemini_key()
    if g:
        for m in GEMINI_MODELS:
            yield "gemini", GEMINI_URL, g, m
    o = os.environ.get("OPENROUTER_API_KEY", "").strip()
    if o:
        for m in OPENROUTER_MODELS:
            yield "openrouter", OPENROUTER_URL, o, m


def chat(messages: list, max_tokens: int = 900, timeout: int = 120, raise_busy: bool = True) -> str:
    """One answer from the first free model that has room. "" if a model
    answered with nothing usable; raises Busy if every one is at its limit
    (or returns "" when raise_busy is False)."""
    global _busy_until, _last_reason
    if not configured():
        return ""          # no key at all: nothing to ask, nothing to log
    if busy():
        if raise_busy:
            raise Busy(busy_message())
        return ""
    reasons = []
    for name, url, key, model in _providers():
        if _model_rest.get(model, 0) > time.time():
            reasons.append(f"{name} {model}: resting")
            continue
        started = time.time()
        body = json.dumps({"model": model, "temperature": 0,
                           "max_tokens": max_tokens, "messages": messages}).encode()
        req = urllib.request.Request(url, data=body, method="POST")
        req.add_header("Content-Type", "application/json")
        req.add_header("Authorization", f"Bearer {key}")
        if name == "openrouter":
            req.add_header("HTTP-Referer", "https://junsgarden.zapto.org")
            req.add_header("X-Title", "Jun's Garden")
        try:
            with urllib.request.urlopen(req, timeout=min(timeout, ATTEMPT_TIMEOUT)) as r:
                data = json.loads(r.read())
            text = ((data.get("choices") or [{}])[0].get("message") or {}).get("content") or ""
            if isinstance(text, list):  # some providers return content parts
                text = "".join(p.get("text", "") for p in text if isinstance(p, dict))
            if text.strip():
                print(f"[freeai] {name} {model} answered in {time.time() - started:.0f}s", flush=True)
                return text
            reasons.append(f"{name} {model}: empty answer")
        except urllib.error.HTTPError as e:
            reasons.append(f"{name} {model}: {e.code}")
            if e.code in (429, 500, 502, 503, 504):
                _model_rest[model] = time.time() + MODEL_COOLDOWN
            continue               # busy, limited, or refused: try the next one
        except Exception as e:     # noqa: BLE001  timeouts, DNS, resets
            reasons.append(f"{name} {model}: {e.__class__.__name__}")
            _model_rest[model] = time.time() + MODEL_COOLDOWN
            continue
    print(f"[freeai] nothing answered: {'; '.join(reasons)[:300]}", flush=True)
    # Nothing answered. Busy only if at least one said it was busy/limited;
    # anything else (no keys at all) is a configuration problem.
    # A resting model and a connection that timed out count as busy too:
    # queueing a tag that could have waited is cheap, losing one is not.
    if any(r.endswith((": 429", ": 503", ": 502", ": 504", ": 500", ": resting",
                       "TimeoutError", "timeout", "URLError"))
           for r in reasons):
        _busy_until = time.time() + COOLDOWN
        _last_reason = "; ".join(reasons)[:300]
        if raise_busy:
            raise Busy(busy_message())
    return ""
