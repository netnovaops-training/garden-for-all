"""Tests for the scan queue's work on the PC: reading a tag's text, matching plants, the image-index
client, the web look-up and a whole scan from pictures to matches. No network, no Ollama, no OCR
service: those are stood in for by small fakes. Run:

    python -m pytest plant-ai/tests -q
"""
import io
import json
import sys
from pathlib import Path

import httpx
import pytest
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from plantai import catalog, config, imgindex, matcher, ocr_eval, scans, synth, tagocr, tagparse, webfind


def L(text, h=60, conf=90, box=None):
    """One OCR line as the service gives it."""
    d = {"text": text, "conf": conf, "h": h}
    if box:
        d["box"] = box
    return d


# ---- reading the text of a tag ----------------------------------------------------------------

def test_the_tallest_line_is_the_name_and_the_facts_are_not():
    lines = [L("Hollow Creek Nursery", 50), L("May Night Salvia", 110), L("Salvia nemorosa 'May Night'", 60),
             L("Full sun", 70), L("#2", 90), L("0417776317063", 30)]
    f = tagparse.parse(lines)
    assert f["name"] == "May Night Salvia"
    assert f["botanical"] == "Salvia nemorosa 'May Night'"
    assert f["vendor"] == "Hollow Creek Nursery"
    assert f["size"] == "#2"
    assert f["barcode"] == "0417776317063"


def test_run_together_words_and_a_dropped_quote_are_repaired():
    lines = [L("EndlessSummerHydrangea", 100), L("Hydrangeamacrophylla'EndlessSummer", 60),
             L("GrownbyMapleRidgePerennials", 50), L("4 in pot", 70)]
    f = tagparse.parse(lines)
    assert f["name"] == "Endless Summer Hydrangea"
    assert f["botanical"] == "Hydrangea macrophylla 'Endless Summer'"
    assert f["vendor"] == "Maple Ridge Perennials"
    assert f["size"] == "4 in pot"


def test_a_hybrid_mark_and_a_bare_inch_height():
    f = tagparse.parse([L("Calamagrostisx acutiflora 'Karl Foerster'", 60), L("Height: 18-24 in", 60), L("Spread: 18 in", 60)])
    assert f["botanical"] == "Calamagrostis x acutiflora 'Karl Foerster'"
    assert f["size"] == ""        # an inch figure without "pot" is a height, not a pot size


def test_a_wrapped_name_joins_only_close_lines_of_like_height():
    lines = [L("Stella de", 90, box=[10, 100, 500, 190]), L("Oro Daylily", 88, box=[10, 200, 500, 288]),
             L("Sun: full", 90, box=[10, 900, 400, 990])]
    assert tagparse.parse(lines)["name"] == "Stella de Oro Daylily"
    far = [L("Stella de", 90, box=[10, 100, 500, 190]), L("Oro Daylily", 88, box=[10, 900, 500, 988])]
    assert tagparse.parse(far)["name"] in ("Stella de", "Oro Daylily")     # one of the two, never both


def test_weak_lines_are_ignored_and_back_facts_never_become_the_name():
    lines = [L("MONUM", 140, conf=10), L("Blue Angel Hosta", 80)]
    assert tagparse.parse(lines)["name"] == "Blue Angel Hosta"
    assert tagparse.parse([L("Plant care", 90), L("Sun: Full sun", 70)])["name"] == ""


def test_barcode_check_digit_and_the_phones_own_reading():
    assert tagparse.ean_ok("0417776317063") and not tagparse.ean_ok("0417776317064")
    assert tagparse.ean_ok("036000291452")                       # a 12-digit UPC-A is checked on its own digits
    assert tagparse.find_barcode([L("0 417776 317063")]) == "0417776317063"
    assert tagparse.find_barcode([L("no digits")], hint="https://x.example/p/1") == "https://x.example/p/1"


def test_best_botanical_keeps_the_fuller_name():
    assert tagparse.best_botanical("Echinacea purpurea", "Echinacea purpurea 'Magnus'") == "Echinacea purpurea 'Magnus'"
    assert tagparse.best_botanical("Salvia nemorosa", "Hosta 'Blue Angel'") == "Salvia nemorosa"
    assert tagparse.best_botanical("", "Rosa 'Radrazz'") == "Rosa 'Radrazz'"


# ---- matching against the garden ----------------------------------------------------------------

PLANTS = [
    {"id": 1, "set": "current", "name": "May Night Salvia", "botanical": "Salvia nemorosa 'May Night'", "genus": "Salvia",
     "species": "nemorosa", "company": "", "barcodes": ["0417776317063"], "photo": "a.webp", "died": False},
    {"id": 2, "set": "current", "name": "Blue Angel Hosta", "botanical": "Hosta 'Blue Angel'", "genus": "Hosta",
     "species": "", "company": "", "barcodes": [], "photo": "", "died": False},
    {"id": 7, "set": "future", "name": "Karl Foerster Feather Reed Grass", "botanical": "Calamagrostis x acutiflora 'Karl Foerster'",
     "genus": "Calamagrostis", "species": "acutiflora", "company": "", "barcodes": [], "photo": "", "died": False},
]


def test_names_alike_and_not():
    assert catalog.sim("Endless SummerHydrangea", "Endless Summer Hydrangea") >= 0.85
    assert catalog.sim("May Night Salvia", "Salvia May Night") >= 0.85
    assert catalog.sim("Blue Angel Hosta", "Magnus Coneflower") < 0.4
    assert catalog.botanical_sim("Salvia nemorosa 'May Night'", "Salvia nemorosa 'Caradonna'") < 0.6
    assert catalog.botanical_sim("Salvia nemorosa 'May Night'", "Salvia nemorosa 'May Night'") > 0.95


def test_the_barcode_alone_finds_the_plant_even_without_the_leading_zero():
    c = catalog.Catalog(PLANTS)
    hit = c.lookup(barcode="417776317063")[0]
    assert hit["plant"]["id"] == 1 and hit["confidence"] >= 0.95


def test_two_clues_beat_one_and_a_wrong_cultivar_is_not_the_same_plant():
    c = catalog.Catalog(PLANTS)
    both = c.lookup("May Night Salvia", "Salvia nemorosa 'May Night'")[0]
    name_only = c.lookup("May Night Salvia")[0]
    assert both["confidence"] > name_only["confidence"] > 0.5
    other = c.lookup("Caradonna Salvia", "Salvia nemorosa 'Caradonna'")
    assert not other or other[0]["confidence"] < 0.5


def test_combine_never_reaches_certainty():
    assert catalog.combine([0.5, 0.5]) == 0.75
    assert catalog.combine([0.99, 0.99, 0.99]) <= 0.99


def test_image_scores_become_capped_confidences():
    assert matcher.image_confidence(0.3) == 0
    assert matcher.image_confidence(1.0) == matcher.IMAGE_CAP
    assert 0 < matcher.image_confidence(0.7) < matcher.image_confidence(0.85)


def test_assemble_has_a_best_match_and_likely_ones_with_their_confidence():
    c = catalog.Catalog(PLANTS)
    fields = {"name": "May Night Salvia", "botanical": "Salvia nemorosa 'May Night'", "vendor": "X Nursery",
              "barcode": "0417776317063", "size": "#1"}
    garden = c.lookup(fields["name"], fields["botanical"], fields["barcode"])
    images = [{"plant_id": 2, "set": "current", "plant_name": "Blue Angel Hosta", "botanical_name": "Hosta 'Blue Angel'",
               "score": 0.75, "photo": "b.webp"}]
    out = matcher.assemble(fields, garden, images, {"candidates": [], "confirms": False}, {("current", 2): PLANTS[1]})
    assert out[0]["plant_id"] == 1 and out[0]["source"] == "garden" and out[0]["confidence"] > 0.9
    assert any(x["plant_id"] == 2 and x["source"] == "garden" for x in out)       # the photo's guess, as a likely match
    assert out == sorted(out, key=lambda x: -x["confidence"])
    # The tag itself is not offered as a NEW plant when a garden plant already is that plant.
    assert not any(x["source"] == "tag" for x in out)


def test_a_plant_not_in_the_garden_comes_from_the_tag_and_a_web_page_makes_it_sure():
    fields = {"name": "Magnus Coneflower", "botanical": "Echinacea purpurea 'Magnus'", "vendor": "Y Farms",
              "barcode": "0662585178125", "size": "#1"}
    alone = matcher.assemble(fields, [], [], {"candidates": [], "confirms": False})
    assert alone[0]["source"] == "tag" and 0.5 < alone[0]["confidence"] < 0.75
    web = {"candidates": [], "confirms": True, "botanical": "", "url": "https://example.edu/p"}
    sure = matcher.assemble(fields, [], [], web)
    assert sure[0]["confidence"] >= 0.75 and sure[0]["url"] == "https://example.edu/p"


# ---- the image index (optional) ------------------------------------------------------------------

def _client(handler):
    return httpx.Client(transport=httpx.MockTransport(handler))


def test_image_index_that_is_not_there_gives_nothing():
    def down(_req):
        raise httpx.ConnectError("refused")
    assert imgindex.status(_client(down)) is None
    assert imgindex.search(b"x", _client(down)) == []
    # Something else answering on the port (not the index) is "not running" too.
    assert imgindex.status(_client(lambda r: httpx.Response(404, json={"detail": "Not Found"}))) is None
    assert imgindex.status(_client(lambda r: httpx.Response(200, json={"ready": False}))) is None
    # The real index answers /status with its counts and no "ready" key: that is "running".
    assert imgindex.status(_client(lambda r: httpx.Response(200, json={"model": "x", "photos": {"n": 3}}))) is not None


def test_image_index_answers_are_sorted_and_checked():
    rows = [{"plant_id": 1, "plant_name": "A", "botanical_name": "A a", "score": 0.6, "photo": "", "set": "current"},
            {"plant_id": 2, "plant_name": "B", "botanical_name": "B b", "score": 0.9, "photo": "", "set": "current"},
            {"junk": True}]
    got = imgindex.search(b"x", _client(lambda r: httpx.Response(200, json=rows)))
    assert [r["plant_id"] for r in got] == [2, 1]
    got = imgindex.search(b"x", _client(lambda r: httpx.Response(200, json={"results": rows})))
    assert len(got) == 2


# ---- the tag reader ---------------------------------------------------------------------------------

FRONT = {"lines": [L("MAGNUSCONEFLOWER", 80, box=[0, 0, 1, 1]), L("Echinaceapurpurea'Magnus", 60), L("#1", 50),
                   L("GrownbyHollowCreekNursery", 50)], "barcodes": [{"type": "EAN13", "text": "0662585178125"}], "engine": "rapid"}
BACK = {"lines": [L("Plant care", 90), L("Echinacea purpurea 'Magnus'", 70), L("Sun: Full sun", 70)], "barcodes": [], "engine": "rapid"}


def ocr_client(front=FRONT, back=BACK):
    def handler(req):
        if req.url.path == "/health":
            return httpx.Response(200, json={"ok": True})
        # The first picture sent is the front: the fakes tell them apart by their size.
        return httpx.Response(200, json=front if len(req.content) == 5 else back)
    return _client(handler)


def test_read_tag_merges_front_and_back_and_trusts_the_decoded_barcode():
    photos = [{"side": "front", "bytes": b"front"}, {"side": "back", "bytes": b"back!!"}]
    r = tagocr.read_tag(photos, client=ocr_client(), use_gemma=False)
    f = r["fields"]
    assert f["botanical"] == "Echinacea purpurea 'Magnus'"
    assert f["vendor"] == "Hollow Creek Nursery" and f["size"] == "#1" and f["barcode"] == "0662585178125"
    assert f["name"] == "MAGNUSCONEFLOWER" or f["name"].lower().replace(" ", "") == "magnusconeflower"
    assert not r["weak"] and r["engine"] == "rapid"


def test_a_weak_reading_asks_the_vision_model_and_a_missing_reader_says_so(monkeypatch):
    weak = {"lines": [L("sm", 40, conf=20)], "barcodes": [], "engine": "rapid"}
    seen = {}

    def fake_gemma(images):
        seen["n"] = len(images)
        return {"name": "Moonbeam Tickseed", "botanical": "", "size": "", "vendor": "", "barcode": ""}
    monkeypatch.setattr(tagocr, "gemma_read", fake_gemma)
    r = tagocr.read_tag([{"side": "front", "bytes": b"front"}], client=ocr_client(front=weak))
    assert r["weak"] and r["fields"]["name"] == "Moonbeam Tickseed" and "gemma" in r["engine"] and seen["n"] == 1
    # No reader at all: the scan still gets an answer, with a note.
    def down(_req):
        raise httpx.ConnectError("refused")
    monkeypatch.setattr(tagocr, "from_tesseract", lambda _b: None)
    monkeypatch.setattr(tagocr, "gemma_read", lambda _i: None)
    r = tagocr.read_tag([{"side": "front", "bytes": b"front"}], client=_client(down))
    assert r["fields"]["name"] == "" and any("reader" in n for n in r["notes"])


def test_the_vision_model_is_asked_in_the_right_shape(monkeypatch, tmp_path):
    """gemma_read against a stand-in Ollama: the picture goes as base64, the answer must fit the schema."""
    from plantai import llm
    config.use(tmp_path / "w", tmp_path / "o")
    sent = {}

    def ollama(req):
        if req.url.path == "/api/tags":
            return httpx.Response(200, json={"models": [{"name": "phi4"}, {"name": "gemma4:12b"}]})
        sent.update(json.loads(req.content))
        reply = {"name": "Moonbeam Tickseed", "botanical": "Coreopsis verticillata 'Moonbeam'", "size": "1 gal",
                 "vendor": "", "barcode": ""}
        return httpx.Response(200, json={"message": {"content": json.dumps(reply)}, "eval_count": 5})
    monkeypatch.setattr(llm, "_client", httpx.Client(transport=httpx.MockTransport(ollama)))
    got = tagocr.gemma_read([b"jpeg bytes"])
    assert got["name"] == "Moonbeam Tickseed" and got["size"] == "1 gal"
    assert sent["model"] == config.CFG.check_model and len(sent["messages"][1]["images"]) == 1
    assert sent["format"]["required"] == ["name", "botanical", "size", "vendor", "barcode"]
    assert sent["options"]["temperature"] == 0
    # Ollama not answering: no reading, no crash.
    monkeypatch.setattr(llm, "_client", httpx.Client(transport=httpx.MockTransport(lambda r: httpx.Response(500))))
    assert tagocr.gemma_read([b"x"]) is None


# ---- the internet look-up ----------------------------------------------------------------------------

def test_web_lookup_finds_a_grower_page_and_confirms(monkeypatch):
    page = {"title": "Magnus Coneflower", "lines": ["Echinacea purpurea 'Magnus' is a classic coneflower."],
            "host": "provenwinners.com", "images": [], "url": "https://www.provenwinners.com/plants/echinacea/magnus"}
    monkeypatch.setattr(webfind.discover, "search", lambda q: [
        {"href": "https://www.instagram.com/p/1", "title": "x"},
        {"href": page["url"], "title": "Magnus"}])
    monkeypatch.setattr(webfind.pages, "read", lambda u: page if u == page["url"] else None)
    out = webfind.lookup({"name": "Magnus Coneflower", "botanical": "Echinacea purpurea 'Magnus'"})
    assert out["confirms"] and out["candidates"][0]["source"] == "web" and out["candidates"][0]["url"] == page["url"]
    assert out["candidates"][0]["botanical"] == "Echinacea purpurea 'Magnus'"


def test_web_lookup_is_optional(monkeypatch):
    def boom(_q):
        raise RuntimeError("no internet")
    monkeypatch.setattr(webfind.discover, "search", boom)
    out = webfind.lookup({"name": "Magnus Coneflower"})
    assert out["candidates"] == [] and out["notes"]
    assert webfind.lookup({})["candidates"] == []


# ---- a whole scan --------------------------------------------------------------------------------------

def test_a_tag_scan_goes_from_pictures_to_a_best_match(monkeypatch):
    cat = catalog.Catalog(PLANTS)
    monkeypatch.setattr(scans.webfind, "lookup", lambda f: {"candidates": [], "confirms": False, "botanical": "", "url": "", "notes": []})
    steps = []
    front = {"lines": [L("May Night Salvia", 100), L("Salvia nemorosa 'May Night'", 60), L("Hollow Creek Nursery", 50)],
             "barcodes": [], "engine": "rapid"}
    res = scans.process({"kind": "tag", "barcode": ""}, [{"side": "front", "bytes": b"front"}], cat, steps.append,
                        client=ocr_client(front=front), use_gemma=False)
    assert res["candidates"][0]["plant_id"] == 1 and res["candidates"][0]["confidence"] > 0.75
    assert res["ocr"]["name"] == "May Night Salvia" and "reading tag" in steps


def test_a_plant_photo_uses_the_image_index_and_says_so_when_it_is_off(monkeypatch):
    cat = catalog.Catalog(PLANTS)
    photos = [{"side": "plant", "bytes": b"jpeg"}]
    off = scans.process({"kind": "plant", "barcode": ""}, photos, cat, client=_client(lambda r: httpx.Response(404)), use_gemma=False)
    assert off["candidates"] == [] and any("image index" in n for n in off["notes"])

    def on(req):
        if req.url.path == "/status":
            return httpx.Response(200, json={"ready": True})
        return httpx.Response(200, json=[{"plant_id": 2, "set": "current", "plant_name": "Blue Angel Hosta",
                                          "botanical_name": "Hosta 'Blue Angel'", "score": 0.88, "photo": ""}])
    got = scans.process({"kind": "plant", "barcode": ""}, photos, cat, client=_client(on), use_gemma=False)
    assert got["candidates"][0]["plant_id"] == 2 and got["candidates"][0]["why"].startswith("the photo looks like it")


class FakeSite:
    """The website's scan queue, in memory: two scans, one of which breaks the helper."""

    def __init__(self):
        self.waiting = [{"id": 1, "kind": "plant", "qty": 1, "barcode": "", "photos": [{"id": 10, "side": "plant"}]},
                        {"id": 2, "kind": "plant", "qty": 1, "barcode": "", "photos": [{"id": 11, "side": "plant"}]}]
        self.posted = []

    def take_scans(self, n):
        out, self.waiting = self.waiting[:n], self.waiting[n:]
        return out

    def catalog(self):
        return PLANTS

    def scan_photo(self, pid):
        if pid == 11:
            raise ValueError("bad picture")
        return b"jpeg"

    def post_scan(self, jid, what, body):
        self.posted.append((jid, what, body))


def test_one_bad_scan_does_not_stop_the_queue(monkeypatch, tmp_path):
    config.use(tmp_path / "w", tmp_path / "o")
    monkeypatch.setattr(scans, "process", lambda it, ph, cat, say=None, **k: {"ocr": {}, "candidates": [], "notes": []})
    site = FakeSite()
    assert scans.run_all(site, take=1) == 1
    kinds = [(j, w) for j, w, _ in site.posted if w != "progress"]
    assert (1, "result") in kinds and (2, "fail") in kinds


# ---- the made-up tags and the scoring ----------------------------------------------------------------------

def test_made_up_tags_have_pictures_and_the_truth(tmp_path):
    if not synth.families():
        pytest.skip("no fonts on this computer")
    truth = synth.make(tmp_path, count=3, seed=5, level="easy")
    assert len(truth) == 3 and (tmp_path / "truth.json").is_file()
    img = Image.open(tmp_path / truth[0]["dir"] / "front.jpg")
    assert max(img.size) <= 1500 and tagparse.ean_ok(truth[0]["barcode"])
    assert json.loads((tmp_path / "truth.json").read_text())[0]["name"]


def test_the_scoring_ignores_spaces_case_and_a_leading_zero():
    assert ocr_eval.same("name", "MAGNUSCONEFLOWER", "Magnus Coneflower") == (True, True)
    assert ocr_eval.same("name", "Magnus Conefower", "Magnus Coneflower")[1]
    assert ocr_eval.same("barcode", "662585178125", "0662585178125") == (True, True)
    assert ocr_eval.same("size", "1 gal", "#1") == (True, True)
    assert ocr_eval.same("size", "", "#1") == (False, False)
    assert ocr_eval.same("botanical", "Echinacea purpurea", "Echinacea purpurea 'Magnus'")[0] is False


def test_barcode_pictures_are_real_ean13():
    code = synth.ean13(__import__("random").Random(1))
    assert len(code) == 13 and tagparse.ean_ok(code)
    img = synth.barcode_image(code)
    assert isinstance(img, Image.Image) and img.width > 200
    buf = io.BytesIO()
    img.save(buf, "PNG")
    assert buf.tell() > 100
