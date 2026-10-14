# WHAT THIS FILE DOES (plain English)
# Talks to YOUR garden wiki (a Wiki.js site) so an approved draft from the
# /wiki-queue page can be written onto a wiki page. It needs two optional
# settings: WIKIJS_API_URL (the wiki's address) and WIKIJS_API_KEY (a key
# made in the wiki's admin area). Without them, check() says "not set up"
# and nothing is sent anywhere.
"""Talk to Wiki.js over its GraphQL API.

WIKI.JS HAS NO REST API WORTH THE NAME -- everything is one GraphQL endpoint
at /graphql, authenticated with a bearer token made in the admin UI. That is
the whole protocol, so this file is small on purpose.

THE KEY IS NEVER PRINTED, by this file or anything that imports it. It is
read from the environment, used, and not echoed -- not in an error message,
not in a debug dump, not in a traceback. `check()` reports its LENGTH when
something is misconfigured, which is enough to tell "empty" from "wrong"
without putting the value on a screen or in a log.

ERRORS COME BACK AS DATA, NOT EXCEPTIONS, because GraphQL answers HTTP 200
with an `errors` array when it refuses something. A caller that only checks
the status code sees every failure as a success.
"""
from __future__ import annotations

import json
import os
import urllib.error
import urllib.request

# THE ENV VAR ALREADY ENDS IN /graphql, so the suffix is stripped here
# rather than assumed away. Wiki.js happens to answer /graphql/graphql
# too, which is exactly the kind of accident that works until it does
# not -- normalise to the site root and build the path in one place.
# Empty by default: no wiki is assumed until you set one.
API_URL = (os.environ.get("WIKIJS_API_URL", "").strip().rstrip("/")
           .removesuffix("/graphql").rstrip("/"))
API_KEY = os.environ.get("WIKIJS_API_KEY", "").strip()
TIMEOUT = 30


def check():
    """Is the wiki configured at all? Returns a complaint, or ''."""
    if not API_URL:
        return "no wiki address set (WIKIJS_API_URL)"
    if not API_KEY:
        return ("no API key for the wiki -- make one in its admin "
                "and put it in WIKIJS_API_KEY")
    if len(API_KEY) < 100:
        # A Wiki.js key is a JWT and is several hundred characters. A short
        # one is a truncated paste, which fails with a bare "Unauthorized"
        # that looks like a permissions problem instead of a typo.
        return f"WIKIJS_API_KEY looks truncated ({len(API_KEY)} characters)"
    return ""


def gql(query, variables=None):
    """One GraphQL call against the wiki. Returns (data, error_message)."""
    bad = check()
    if bad:
        return None, bad
    body = json.dumps({"query": query,
                       "variables": variables or {}}).encode("utf-8")
    req = urllib.request.Request(
        API_URL + "/graphql", data=body, method="POST",
        headers={"Content-Type": "application/json",
                 "Authorization": "Bearer " + API_KEY})
    try:
        with urllib.request.urlopen(req, timeout=TIMEOUT) as r:
            out = json.loads(r.read())
    except urllib.error.HTTPError as e:
        detail = e.read().decode("utf-8", "replace")[:300]
        return None, f"HTTP {e.code}: {detail}"
    except Exception as e:  # noqa: BLE001
        return None, f"{type(e).__name__}: {e}"

    # HTTP 200 WITH AN ERRORS ARRAY is how GraphQL refuses. Reading only the
    # status code here would report every refusal as a success.
    if out.get("errors"):
        msgs = "; ".join(str(e.get("message", e))[:200]
                         for e in out["errors"])
        return None, msgs
    return out.get("data"), ""


CREATE_PAGE = """
mutation ($content:String!, $description:String!, $editor:String!,
          $isPublished:Boolean!, $isPrivate:Boolean!, $locale:String!,
          $path:String!, $tags:[String]!, $title:String!) {
  pages { create(content:$content, description:$description, editor:$editor,
                 isPublished:$isPublished, isPrivate:$isPrivate,
                 locale:$locale, path:$path, tags:$tags, title:$title) {
    responseResult { succeeded errorCode message }
    page { id path } } } }
"""

UPDATE_PAGE = """
mutation ($id:Int!, $content:String!, $description:String!,
          $isPublished:Boolean!, $tags:[String]!, $title:String!) {
  pages { update(id:$id, content:$content, description:$description,
                 isPublished:$isPublished, tags:$tags, title:$title) {
    responseResult { succeeded errorCode message }
    page { id path } } } }
"""

PAGE_ID = """
query ($path:String!, $locale:String!) {
  pages { singleByPath(path:$path, locale:$locale) { id } } }
"""


def page_id(path, locale="en"):
    """The id of one page, or (None, "") when there is no such page.

    WIKI.JS REPORTS A MISSING PAGE AS AN ERROR, not as an empty result --
    `singleByPath` answers "This page does not exist." in the GraphQL errors
    array. That is the ordinary case of a page not written yet, so it is
    not treated as a failure.
    """
    data, err = gql(PAGE_ID, {"path": path.strip("/"), "locale": locale})
    if err:
        if "does not exist" in err.lower() or "not found" in err.lower():
            return None, ""
        return None, err
    node = ((data or {}).get("pages") or {}).get("singleByPath")
    return (node or {}).get("id"), ""


def put_page(path, title, content, description="", tags=None,
             locale="en", published=True, existing=None):
    """Create the page, or update it if it is already there.

    `existing` False means "I already know it is new" and skips the lookup.
    """
    path = path.strip("/")
    tags = tags or []
    if existing is None or existing:
        pid, err = page_id(path, locale)
    else:
        pid, err = None, ""
    if err:
        return False, err

    if pid:
        data, err = gql(UPDATE_PAGE, {
            "id": pid, "content": content,
            "description": description, "isPublished": published,
            "tags": tags, "title": title})
        node = (data or {}).get("pages", {}).get("update", {})
        verb = "updated"
    else:
        data, err = gql(CREATE_PAGE, {
            "content": content, "description": description,
            "editor": "markdown", "isPublished": published,
            "isPrivate": False, "locale": locale, "path": path,
            "tags": tags, "title": title})
        node = (data or {}).get("pages", {}).get("create", {})
        verb = "created"
    if err:
        return False, err

    res = node.get("responseResult") or {}
    if not res.get("succeeded"):
        return False, res.get("message") or "Wiki.js refused it"
    return True, verb
