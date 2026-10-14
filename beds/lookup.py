# WHAT THIS FILE DOES (plain English)
# Two ways to get plant facts: read them off a nursery tag photo (with the
# free AI), or search the web (DuckDuckGo, no key needed) and let the free AI
# pull out the facts. Nothing here writes; it only gathers.
"""Find facts about a plant on the web, and read them off a tag.

TWO WAYS IN, ONE WAY OUT. A nursery tag photographed in the garden and a
search of the open web both end up as the same loose dict, which then goes
through the vocabulary gate like anything else. Nothing here writes, and
nothing here decides -- it gathers.

NO API KEY. DuckDuckGo needs none, which is why `plant_orchestrator` already
uses it (duckduckgo_search 8.1.1, DDGS().text and DDGS().images). Reusing
that rather than adding a paid search key keeps the whole thing copyable to
somebody else's machine, which is the point of the product version.

THE SOURCE TRAVELS WITH THE FACT. Every field carries the URL it came from,
so a wrong value can be traced to the page that claimed it rather than
becoming an anonymous entry in a catalogue nobody can audit.

GROWERS FIRST. The catalogue's own `company` column says who supplies this
garden -- Proven Winners 250, Monrovia 40 -- so their pages are searched
before the open web. A grower's page about their own cultivar beats a
gardening blog quoting it second hand.
"""
from __future__ import annotations

import json
import os
import re

TRUSTED = ["provenwinners.com", "monrovia.com", "davidaustinroses.com",
           "missouribotanicalgarden.org", "extension.edu"]

PERENUAL_KEY = os.environ.get("PERENUAL_KEY", "")

# The fields worth asking a model to pull out of prose. Deliberately the
# controlled ones plus a few plain facts -- not everything the schema has,
# because a model asked for ninety fields invents most of them.
WANTED = ["botanical_name", "genus", "species", "family", "plant_type",
          "sun_exposure", "water_needs", "soil_type", "soil_ph", "zones",
          "spacing", "size", "growth_rate", "bloom_color", "bloom_months",
          "foliage_color", "deer_resistance", "toxicity", "native_status",
          "attracts", "companion_plants", "care_notes"]


# WORDS THAT NAME A PLANT, and the genus each one means.
#
# WHY A HINT IS NEEDED AT ALL. Searching the bare trade name "Berry Box"
# returned a page about xPyracomeles and the extractor duly reported that
# genus -- rejected by the two-source check, but a wasted lookup and a
# plausible-looking wrong answer. Searching "Berry Box Ilex" returns the
# Ilex glabra pages on the first result.
#
# The hint is already sitting in the nursery name. "Alpine Carpet JUNIPER",
# "Big Tuna Mugo PINE", "Danica ARBORVITAE" -- growers put the plant last,
# because that is the part the customer is shopping for. Taking it turns an
# ambiguous two-word query into a specific one.
#
# READ FROM HIS OWN CATALOGUE FIRST, so this list is not a second vocabulary
# to maintain: any common word that already maps to a genus in data_source
# is used before these. This table is only the fallback for words the
# catalogue has not seen yet.
COMMON_GENUS = {
    "juniper": "Juniperus", "pine": "Pinus", "arborvitae": "Thuja",
    "holly": "Ilex", "inkberry": "Ilex", "winterberry": "Ilex",
    "hosta": "Hosta", "rose": "Rosa", "hydrangea": "Hydrangea",
    "spirea": "Spiraea", "spiraea": "Spiraea", "viburnum": "Viburnum",
    "weigela": "Weigela", "boxwood": "Buxus", "maple": "Acer",
    "dogwood": "Cornus", "magnolia": "Magnolia", "lilac": "Syringa",
    "azalea": "Rhododendron", "rhododendron": "Rhododendron",
    "coneflower": "Echinacea", "echinacea": "Echinacea",
    "daylily": "Hemerocallis", "sedum": "Sedum", "salvia": "Salvia",
    "coreopsis": "Coreopsis", "heuchera": "Heuchera", "astilbe": "Astilbe",
    "clematis": "Clematis", "peony": "Paeonia", "iris": "Iris",
    "phlox": "Phlox", "yarrow": "Achillea", "catmint": "Nepeta",
    "lavender": "Lavandula", "cypress": "Chamaecyparis", "fir": "Abies",
    "spruce": "Picea", "yew": "Taxus", "elderberry": "Sambucus",
    "ninebark": "Physocarpus", "potentilla": "Potentilla",
    "barberry": "Berberis", "forsythia": "Forsythia", "lilyturf": "Liriope",
    "butterfly bush": "Buddleja", "buddleia": "Buddleja",
    "hummingbird mint": "Agastache", "agastache": "Agastache",
}


def genus_hint(name: str) -> str:
    """A genus to narrow the search with, or "" when the name gives none."""
    low = " " + (name or "").lower().strip() + " "
    # Longest phrase first: "butterfly bush" must beat "bush" if that is ever
    # added, the same longest-match rule the soil parser needed.
    for word in sorted(COMMON_GENUS, key=len, reverse=True):
        if " " + word + " " in low or low.endswith(" " + word + " "):
            return COMMON_GENUS[word]
        if word in low:
            return COMMON_GENUS[word]
    return ""


def _ddg_text(query: str, max_results: int = 4):
    """Search, preferring the growers this garden actually buys from."""
    # `ddgs` FIRST. The package was renamed: duckduckgo_search 8.1.1 still
    # imports, still runs, and returns ZERO results for everything -- a
    # silent failure, not an error. plant_orchestrator is pinned to that
    # version, so its web enrichment has been quietly returning nothing.
    try:
        from ddgs import DDGS
    except ImportError:
        try:
            from duckduckgo_search import DDGS
        except ImportError:
            return []
    out = []
    try:
        with DDGS() as ddgs:
            for site in TRUSTED[:2]:
                out += list(ddgs.text(f"{query} site:{site}", max_results=2))
            if len(out) < max_results:
                out += list(ddgs.text(f"{query} plant care",
                                      max_results=max_results - len(out)))
    except Exception:
        # A search that fails is a lookup with nothing in it, not an error
        # the person needs to see. The tag they photographed is still there.
        return out
    return out[:max_results + 4]


def _perenual(name: str):
    """The one plant-specific source with a key already configured."""
    if not PERENUAL_KEY:
        return None
    import urllib.parse
    import urllib.request
    url = ("https://perenual.com/api/species-list?key="
           f"{PERENUAL_KEY}&q={urllib.parse.quote(name)}")
    try:
        with urllib.request.urlopen(url, timeout=20) as r:
            data = json.loads(r.read().decode("utf-8"))
        hits = data.get("data") or []
        return hits[0] if hits else None
    except Exception:
        return None


EXTRACT = """Read these search results about one plant and pull out what they
actually say. This is for a garden catalogue, so accuracy matters more than
completeness.

PLANT: {name}

RESULTS:
{blob}

Answer with JSON only: an object whose keys are any of {fields} and whose
values are short strings quoting or closely paraphrasing the source.

Rules:
- OMIT any field the results do not actually state. A missing field is fine;
  an invented one is a wrong fact in somebody's records.
- Do not normalise or tidy the wording. Report what the page says, in the
  page's own terms. Something later maps it onto this catalogue's vocabulary.
- If the results are about a different plant, answer {{}}.
"""


def _extract(name: str, results: list, llm_call) -> dict:
    if not results:
        return {}
    blob = "\n\n".join(
        f"[{r.get('href','')}]\n{r.get('title','')}\n{r.get('body','')}"
        for r in results)[:6000]
    raw = llm_call(EXTRACT.format(
        name=name, blob=blob, fields=json.dumps(WANTED)))
    if not raw:
        return {}
    try:
        text = raw.strip()
        if text.startswith("```"):
            text = text.strip("`").split("\n", 1)[-1].rsplit("```", 1)[0]
        return json.loads(text[text.index("{"):text.rindex("}") + 1])
    except (ValueError, IndexError):
        return {}


TAG = """This is a photograph of a plant nursery tag. Read it.

Answer with JSON only, using any of these keys: {fields}, plus "barcode" if
a barcode number is printed and legible, and "company" for the grower or
brand.

Rules:
- Report ONLY what is printed on the tag. Do not add what you know about the
  plant from elsewhere -- that happens in a separate step, from sources that
  can be cited.
- If the tag is unreadable or is not a plant tag, answer {{}}.
- Keep the tag's own wording. Do not tidy it.
"""


def _read_one(describe_image, image_data_url: str) -> dict:
    """One photograph -> whatever JSON the model could read off it."""
    raw = describe_image(image_data_url, TAG.format(fields=json.dumps(WANTED)))
    if not raw:
        return {}
    try:
        text = raw.strip()
        if text.startswith("```"):
            text = text.strip("`").split(chr(10), 1)[-1].rsplit("```", 1)[0]
        return json.loads(text[text.index("{"):text.rindex("}") + 1])
    except (ValueError, IndexError):
        return {}


def read_tag(describe_image, images) -> dict:
    """A nursery tag -> the fields printed on it. One photo or several.

    BOTH SIDES, BECAUSE THE BACK IS WHERE THE FACTS ARE. The author, 2026-09-26:
    Jun photographed a tag and "it didn't allow him to take the back of the
    tag and it did not add enough information". A nursery tag prints the
    name and a picture on the front and nearly everything worth recording
    -- spacing, zones, sun, water, bloom months, the grower -- on the back.
    One photograph of the front is a name and very little else, which is
    exactly what came back.

    EACH SIDE IS READ SEPARATELY, then merged here. Handing a model two
    images at once invites it to blend them and attribute a fact to
    whichever side it likes; read apart, each answer is about one
    photograph and the merge is ours to control.

    FIRST NON-EMPTY WINS, IN THE ORDER GIVEN, so the front settles the name
    when both sides print it. Where the sides genuinely disagree the
    disagreement is KEPT, in `_conflicts`, rather than quietly resolved --
    a wrong silent merge is the kind of error nobody goes looking for.

    Accepts a bare string too, so every existing caller keeps working.
    """
    if isinstance(images, str):
        images = [images]
    images = [i for i in (images or []) if i]
    if not images:
        return {}

    merged, conflicts = {}, {}
    for shot in images:
        for field, value in (_read_one(describe_image, shot) or {}).items():
            text = str(value).strip() if value is not None else ""
            if not text:
                continue
            if field not in merged:
                merged[field] = text
            elif merged[field].strip().lower() != text.lower():
                conflicts.setdefault(field, [merged[field]]).append(text)
    if conflicts:
        merged["_conflicts"] = conflicts
    return merged


def lookup(name: str, llm_call, company: str = "") -> dict:
    """Everything the web says about this plant, with sources.

    Returns {"fields": {...}, "sources": [...]} where fields are still in
    the SOURCE's wording. Normalising is the gate's job, not this one's --
    keeping them separate is what lets the same lookup serve a catalogue
    with different words.
    """
    name = (name or "").strip()
    if not name:
        return {"fields": {}, "sources": []}

    # THE HINT GOES INTO THE QUERY, NOT INTO THE ANSWER. It narrows what is
    # searched; whatever genus comes back still has to pass verify_genus.
    #
    # THE GROWER IS THE BEST HINT THERE IS, and it costs nothing because the
    # catalogue already records it on every row. "Berry Box" and "Juke Box"
    # contain no plant word at all -- searching them bare returned a page
    # about xPyracomeles -- but they are Proven Winners names, and "Berry Box
    # Proven Winners" goes straight to the grower's own page for the plant.
    # Monrovia and Proven Winners alone account for 5,474 of the 6,534
    # wishlist rows, and both are already in TRUSTED.
    #
    # A plant word in the name is used FIRST when there is one, because
    # "Juniperus" narrows harder than "Monrovia" does.
    hint = genus_hint(name) or (company or "").strip()
    results = _ddg_text((name + " " + hint).strip() if hint else name)
    fields = _extract(name, results, llm_call)
    sources = [r.get("href") for r in results if r.get("href")][:5]

    per = _perenual(name)
    if per:
        # Perenual is structured, so it is mapped by hand rather than read by
        # a model -- and it only FILLS GAPS. A field the growers' own pages
        # already answered is not overwritten by a general database.
        for src, dst in (("cycle", "plant_type"), ("watering", "water_needs"),
                         ("scientific_name", "botanical_name")):
            v = per.get(src)
            if isinstance(v, list):
                v = v[0] if v else None
            if v and not fields.get(dst):
                fields[dst] = str(v)
        sun = per.get("sunlight")
        if sun and not fields.get("sun_exposure"):
            fields["sun_exposure"] = ", ".join(sun) if isinstance(sun, list) else str(sun)
        sources.append("perenual.com")

    return {"fields": fields, "sources": sources}


def merge(tag: dict, web: dict) -> dict:
    """Tag first, web second.

    THE TAG WINS every disagreement. It is the thing that came home in the
    pot: it names the cultivar actually bought, from the grower who grew it.
    A web page about the same cultivar is a good second source and a poor
    first one.
    """
    out = dict(web.get("fields") or {})
    for k, v in (tag or {}).items():
        if isinstance(v, str) and v.strip():
            out[k] = v.strip()
    return out
