"""phi4 stage: read each source page into field candidates, judge across sources, normalise.

Per page the model returns {items:[{f, v, l}]}: field, value, and the numbered page lines that
state it. Lines become the quotes; code normalises every value to the garden DB's forms and drops
anything that cannot be made valid. Species pages are read once and shared by every cultivar.
"""
from . import db, llm, norm, pages

GUIDE = """botanical_name: Latin name, cultivar in single quotes, e.g. Berberis thunbergii 'Koren'
family: plant family (ends in -aceae)
genus: Latin genus | species: species epithet only
plant_type: Perennial|Annual|Biennial|Shrub|Tree|Succulent|Vine
native_status: Native|Non-native|Introduced (to North America), only if stated
plant_length: mature height, e.g. 24-36 in or 3-4 ft
plant_width: mature width/spread | spacing: planting distance
zones: USDA hardiness zones, e.g. 4-8
life_expectancy: e.g. 10+ years
growth_rate: Slow|Moderate|Fast
sun_exposure: Full Sun|Partial Shade|Full Sun - Partial Shade|Full Shade|Partial Shade - Full Shade|Full Sun - Full Shade
water_needs: Low|Low to Average|Average|Average to High|High
soil_ph: e.g. 6.0-7.0
soil_type: words from Loamy, Sandy, Clay, Rocky, Well-draining, Moist, Wet, Rich, Average fertility, Poor, Adaptable, Acidic, Alkaline
planting_depth: e.g. Crown at soil level | chilling_hours: hours of cold needed
bloom_color / foliage_color / stem_color: colours, e.g. Pink, White
is_evergreen: Deciduous|Herbaceous|Evergreen|Semi-evergreen|Summer Dormant|Spring Ephemeral|Deciduous Conifer
leaf_shape: Linear|Lanceolate|Ovate|Elliptic|Cordate|Palmate|Deltoid
leaf_type: Simple|Pinnately Compound|Palmately Compound
leaf_margin: Entire|Serrated|Shallowly Serrate|Lobed
flower_symmetry: Actinomorphic|Zygomorphic|None
flower_shape: Rotate|Stellate|Cruciate|Campanulate|Infundibuliform|Tubular|Salverform|Bilabiate|Urceolate|Trumpet/Cup-shaped|High-centered|Double Blossoms|None
flower_size: Tiny (<1 in)|Small (1-2 in)|Medium (2-4 in)|Large (4-6 in)|Massive (>6 in)|None
reproduction_type: Hermaphrodite|Monoecious|Dioecious|Polygamous
fruit_type / fruit_shape / fruit_color / fruit_size: short words, e.g. Capsule / Oval / Brown / < 1 in
fruit_edibility: Edible|Inedible|Toxic|Wildlife Only|None
root_system: Taproot Systems|Fibrous/Adventitious Systems|Specialized Systems
root_morphology: Fine Roots (< 2 mm)|Coarse Roots (> 2 mm)|Mixed Fine & Coarse
root_function: Absorptive Roots (Lower Orders)|Transport Roots (Higher Orders)|Mixed Orders
aromatics: fragrance, e.g. Sweet floral, or None
attracts: words from Bees, Butterflies, Hummingbirds, Birds, Beneficial Insects, Pollinators
deer_resistance: Deer Resistant|Rabbit Resistant|Both|Not Resistant
toxicity: Non-toxic|Toxic|Toxic to Dogs/Cats|Toxic to Humans|Toxic to Humans & Pets
invasive_local: No|Yes|Watchlist
propagation_method: e.g. Stem cuttings, division | cuttings: how to take cuttings
companion_plants: plants the page says to grow with it
fertilizer: only an Espoma product the page names (Plant-tone, Holly-tone, Rose-tone...)
grow_months: months of active growth, e.g. April-October
bloom_months: months in flower, e.g. June-August
planting_month_start / planting_month_end: planting time, if stated
care_notes: watering/general care | maintenance_notes: pruning/upkeep | cleanup_notes: fall/spring cleanup
disease_susceptibility: pests/diseases named | extra_details: other notable uses/features"""

SYSTEM = ("You copy plant facts out of ONE web page. Use only the numbered page lines given; "
          "never add knowledge of your own. For each field the page states, output f (field name), "
          "v (value in the format shown) and l (the line numbers that state it). Skip fields the page "
          "does not state: never output an item with an empty value. Notes fields: one short "
          "sentence, at most 120 characters, from the page.")
LLM_FIELDS = [f for f in norm.FIELDS if f not in ("season", "grow_season", "month_start", "month_end")]
SCHEMA = {"type": "object", "required": ["items"], "properties": {"items": {"type": "array", "items": {
    "type": "object", "required": ["f", "v", "l"], "properties": {
        "f": {"type": "string", "enum": LLM_FIELDS}, "v": {"type": "string"},
        "l": {"type": "array", "items": {"type": "integer"}}}}}}}
# Traits that differ between cultivars: only the cultivar's own (vendor) page may give these
# when the plant is a named cultivar.
CULTIVAR_ONLY = {"plant_length", "plant_width", "spacing", "bloom_color", "foliage_color", "stem_color",
                 "flower_size", "bloom_months", "aromatics", "growth_rate", "botanical_name"}


def extract(url: str) -> dict | None:
    """Candidates from one page: {title, url, items:[{f, raw, v, note, quote}]}; cached per URL."""
    k = "x:" + url
    hit = db.shared_get(k)
    if hit is not None:
        return hit
    p = pages.read(url)
    if not p or not p["lines"]:
        db.shared_put(k, {"title": "", "url": url, "items": [], "dead": True})
        return None
    out = llm.ask(llm.FILL, SYSTEM, f"FIELDS:\n{GUIDE}\n\nPAGE: {p['title']}\n{pages.numbered(p)}", SCHEMA)
    items = []
    for it in (out or {}).get("items", []):
        f = it.get("f")
        if f not in LLM_FIELDS or not str(it.get("v") or "").strip():
            continue
        lines = [p["lines"][i - 1] for i in it.get("l", []) if isinstance(i, int) and 0 < i <= len(p["lines"])]
        v, note = norm.norm(f, it.get("v"))
        items.append({"f": f, "raw": it.get("v"), "v": v, "note": note, "quote": " … ".join(lines)[:500]})
    res = {"title": p["title"], "url": p["url"], "host": p["host"], "items": items}
    db.shared_put(k, res)
    return res


JUDGE_SYS = ("You pick the best-supported value for plant fields where web sources disagree. "
             "Prefer the grower's own page for cultivar traits (size, colours, bloom time, zones, sun); "
             "prefer university, extension and botanical-garden pages for species traits (family, "
             "leaves, fruit, roots, toxicity). Pick one option number per field, or 0 if none is "
             "supported. Give a short reason.")
JUDGE_SCHEMA = {"type": "object", "required": ["picks"], "properties": {"picks": {"type": "array", "items": {
    "type": "object", "required": ["f", "pick", "why"], "properties": {
        "f": {"type": "string"}, "pick": {"type": "integer"}, "why": {"type": "string"}}}}}}


def judge(rec: dict, cands: dict[str, list[dict]]) -> dict:
    """{field: {v, conf, sources, quotes, why}} from all candidates (already normalised)."""
    out, conflicts = {}, {}
    for f, cs in cands.items():
        vals = {}
        for c in cs:
            vals.setdefault(c["v"], []).append(c)
        if len(vals) == 1:
            v, cs2 = next(iter(vals.items()))
            hosts = {c["host"] for c in cs2}
            out[f] = {"v": v, "conf": "high" if len(hosts) >= 2 else "medium",
                      "sources": sorted({c["url"] for c in cs2}), "quotes": [c["quote"] for c in cs2][:3], "why": ""}
        else:
            conflicts[f] = list(vals.items())
    if conflicts:
        txt = []
        for f, opts in conflicts.items():
            txt.append(f"FIELD {f}:")
            for i, (v, cs2) in enumerate(opts, 1):
                src = ", ".join(sorted({c['host'] + (' (grower)' if c['kind'] == 'vendor' else '') for c in cs2}))
                txt.append(f"  {i}. {v}   [{src}] quote: {cs2[0]['quote'][:200]}")
        res = llm.ask(llm.FILL, JUDGE_SYS, f"PLANT: {rec.get('plant_name')} ({rec.get('botanical_name')})\n" +
                      "\n".join(txt), JUDGE_SCHEMA) or {}
        picks = {p["f"]: p for p in res.get("picks", []) if isinstance(p, dict)}
        for f, opts in conflicts.items():
            p = picks.get(f)
            if not p or not (1 <= int(p.get("pick", 0)) <= len(opts)):
                continue
            v, cs2 = opts[int(p["pick"]) - 1]
            out[f] = {"v": v, "conf": "low", "sources": sorted({c["url"] for c in cs2}),
                      "quotes": [c["quote"] for c in cs2][:3],
                      "why": f"judged: {p.get('why', '')[:200]}; other values: " +
                             "; ".join(f"{ov} ({', '.join(sorted({c['host'] for c in oc}))})"
                                       for ov, oc in opts if ov != v)}
    return out


def derive(fields: dict) -> None:
    """Season/month fields worked out from the sourced month codes (same sources, marked derived)."""
    def add(f, v, base):
        if v and f not in fields:
            b = fields[base]
            fields[f] = {"v": v, "conf": b["conf"], "sources": b["sources"], "quotes": b["quotes"],
                         "why": f"derived from {base}"}
    if "bloom_months" in fields:
        code = fields["bloom_months"]["v"]
        add("season", norm.seasons_from_months(code), "bloom_months")
        on = [i for i, c in enumerate(code) if c != "."]
        if on and on == list(range(on[0], on[-1] + 1)):
            add("month_start", norm.MONTHS[on[0]], "bloom_months")
            add("month_end", norm.MONTHS[on[-1]], "bloom_months")
    if "grow_months" in fields:
        add("grow_season", norm.seasons_from_months(fields["grow_months"]["v"]), "grow_months")
