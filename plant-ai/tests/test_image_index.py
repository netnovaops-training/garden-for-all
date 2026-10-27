"""Image index tests: no model, no network, no garden data. A stand-in "model" turns a picture
into its colour layout, which is enough to check the plumbing: placing photos on plants, the
incremental build, search, the text hook and the local-only service.

    python -m pytest plant-ai/tests/test_image_index.py -q      (needs numpy + Pillow)
"""
import io
import json
import sys
import threading
import types
import urllib.request
from pathlib import Path

import numpy as np
import pytest
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from image_index import catalog, finder, indexer, models, server  # noqa: E402
from image_index.store import Store  # noqa: E402

COLORS = {"red": (220, 30, 30), "green": (30, 200, 40), "blue": (30, 40, 220),
          "yellow": (230, 220, 30), "white": (240, 240, 240)}


class FakeEmbedder:
    """Colour layout of the picture as the vector (4x4 grid of mean colours)."""
    def logit_scale(self):  # a BioCLIP-style model has one; here a fixed number
        return 50.0

    def __init__(self, name, threads=None, feature=None):
        self.spec = models.MODELS[name]

    def text_embed(self, prompts):
        """A name 'looks like' its colour: the same vector a picture of that colour gets."""
        word_color = {"rosa": "red", "hosta": "green", "iris": "blue", "rudbeckia": "yellow"}
        out = []
        for p in prompts:
            c = next((COLORS[v] for k, v in word_color.items() if k in p.lower()), COLORS["white"])
            img = np.zeros((1, 224, 224, 3), np.uint8) + np.array(c, np.uint8)
            out.append(self.embed(img)[0])
        return np.stack(out)

    def embed(self, batch):
        n, s = batch.shape[0], batch.shape[1] // 4
        v = batch.reshape(n, 4, s, 4, s, 3).mean((2, 4)).reshape(n, -1).astype(np.float32)
        v = v - 128.0
        return v / np.maximum(np.linalg.norm(v, axis=1, keepdims=True), 1e-9)


@pytest.fixture(autouse=True)
def fake_model(monkeypatch):
    mod = types.ModuleType("image_index.embed")
    mod.Embedder = FakeEmbedder
    monkeypatch.setitem(sys.modules, "image_index.embed", mod)
    finder._cache.clear()


def _jpg(path, color, noise=0):
    rng = np.random.default_rng(noise)
    arr = np.zeros((64, 48, 3), np.uint8) + np.array(color, np.uint8)
    jitter = rng.integers(-noise, noise + 1, arr.shape)
    arr = np.clip(arr.astype(int) + jitter, 0, 255).astype(np.uint8)
    Image.fromarray(arr).save(path)


@pytest.fixture
def garden(tmp_path):
    """Two current plants (red rose, green hosta), one future plant (blue iris)."""
    db = tmp_path / "db"
    db.mkdir()
    cur = [{"id": 1, "plant_name": "Red Rose", "botanical_name": "Rosa x hybrida",
            "picture": "rose_1_manual_1.webp", "additional_pictures": ""},
           {"id": 2, "plant_name": "Green Hosta", "botanical_name": "Hosta sieboldiana",
            "picture": "hosta.webp", "additional_pictures": "hosta2.webp"}]
    fut = [{"id": 7, "plant_name": "Blue Flag", "botanical_name": "Iris versicolor"},
           {"id": 8, "plant_name": "Yellow Daisy", "botanical_name": "Rudbeckia hirta"}]
    (db / "current.json").write_text(json.dumps(cur))
    (db / "future.json").write_text(json.dumps(fut))
    cdir, fdir = tmp_path / "cur", tmp_path / "fut"
    (fdir / "7").mkdir(parents=True)
    (fdir / "8").mkdir()
    cdir.mkdir()
    _jpg(cdir / "rose_1_manual_1.webp", COLORS["red"])
    _jpg(cdir / "hosta.webp", COLORS["green"], 3)
    _jpg(cdir / "hosta2.webp", COLORS["green"], 9)
    _jpg(cdir / "Rose_extra_1_manual_5.webp", COLORS["red"], 5)  # placed by the id in its name
    _jpg(cdir / "stray.webp", COLORS["white"])  # no plant: reported, not indexed
    _jpg(fdir / "7" / "1.webp", COLORS["blue"])
    _jpg(fdir / "8" / "1.webp", COLORS["yellow"])
    return types.SimpleNamespace(db=db, dirs=[str(cdir), str(fdir)], out=tmp_path / "idx",
                                 tmp=tmp_path)


def test_catalog_places_photos(garden):
    plants = catalog.load_plants(garden.db)
    photos, skipped = catalog.scan(garden.dirs, plants)
    got = {p.rel: (p.set, p.plant_id) for p in photos}
    assert got["rose_1_manual_1.webp"] == ("current", 1)
    assert got["hosta2.webp"] == ("current", 2)
    assert got["Rose_extra_1_manual_5.webp"] == ("current", 1)
    assert got["7/1.webp"] == ("future", 7)
    assert any(r == "stray.webp" for r, _ in skipped)


def test_hybrid_sign_repaired():
    assert catalog.clean_name("Abelia � grandiflora ") == "Abelia x grandiflora"


def test_build_is_incremental_and_resumable(garden):
    log = []
    info = indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=log.append)
    assert info["new"] == 6 and info["unreadable"] == 0
    again = indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=log.append)
    assert again["new"] == 0  # nothing changed: nothing embedded
    _jpg(Path(garden.dirs[1]) / "7" / "2.webp", COLORS["blue"], 4)  # one new photo
    third = indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=log.append)
    assert third["new"] == 1
    # changed file (new size/time) is embedded again, removed file is dropped
    _jpg(Path(garden.dirs[0]) / "hosta.webp", COLORS["yellow"], 1)
    (Path(garden.dirs[1]) / "8" / "1.webp").unlink()
    fourth = indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=log.append)
    assert fourth["new"] == 1 and fourth["removed"] == 1
    c = Store(garden.out).counts()
    assert c["future"]["photos"] == 2 and c["current"]["photos"] == 4


def test_unreadable_photo_is_remembered(garden):
    (Path(garden.dirs[1]) / "7" / "bad.webp").write_bytes(b"not a picture")
    a = indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=lambda *_: None)
    assert a["unreadable"] == 1
    b = indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=lambda *_: None)
    assert b["unreadable"] == 0 and b["new"] == 0  # not tried again while unchanged


def test_search_finds_the_right_plant(garden):
    indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=lambda *_: None)
    q = garden.tmp / "q.jpg"
    _jpg(q, COLORS["green"], 7)
    res = finder.search(str(q), k=3, out_dir=garden.out)
    assert res[0]["plant_id"] == 2 and res[0]["set"] == "current"
    assert res[0]["plant_name"] == "Green Hosta" and res[0]["botanical_name"].startswith("Hosta")
    assert set(res[0]) >= {"plant_id", "plant_name", "botanical_name", "score", "photo", "set"}
    assert 0 <= res[0]["score"] <= 1 and res[0]["photo"] in ("hosta.webp", "hosta2.webp")
    assert res[0]["score"] >= res[1]["score"]
    by_bytes = finder.search(q.read_bytes(), k=1, out_dir=garden.out)  # bytes work too
    assert by_bytes[0]["plant_id"] == 2
    with pytest.raises(ValueError):
        finder.search(b"junk", out_dir=garden.out)


def test_text_hint_reranks(garden):
    indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=lambda *_: None)
    q = garden.tmp / "q.jpg"
    _jpg(q, (30, 130, 120), 7)  # between green and blue, a little closer to green
    plain = finder.search(str(q), k=4, out_dir=garden.out)
    assert plain[0]["plant_id"] == 2
    # a tag that clearly names the iris (with an OCR typo) lifts it above the look-alike order
    hinted = finder.search(str(q), k=4, out_dir=garden.out, hints="Iris versicolr")
    iris = next(r for r in hinted if r["plant_id"] == 7)
    assert iris["text_match"] > 0
    assert iris["score"] > next(r for r in plain if r["plant_id"] == 7)["score"]
    assert "text_match" in hinted[0] and "text_match" not in plain[0]


def test_words_ignore_filler_and_accents():
    assert finder.words("The Éclat Tree, Rosa x hybrida") == ["eclat", "rosa", "hybrida"]


def test_taxon_spellings():
    from image_index.taxon import taxon
    assert taxon("Salvia nemorosa 'Violet Profusion'") == ("salvia", "nemorosa")
    assert taxon("Abelia � grandiflora 'Sherwoodii'") == ("abelia", "grandiflora")
    assert taxon("Abelia x grandiflora") == ("abelia", "grandiflora")
    assert taxon("Allium 'Millennium'") == ("allium", None)  # a cultivar is not a species
    assert taxon("Lactuca virosa L.") == ("lactuca", "virosa")
    assert taxon("") == (None, None)


def test_verify_name_zero_shot(garden, monkeypatch):
    """The name check with a stand-in model whose text side knows four colours."""
    monkeypatch.setitem(models.MODELS, "fake-bio", models.Spec(
        "x/y", "", 224, models.CLIP, "bioclip", feature=""))
    indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=lambda *_: None)
    from image_index import verify
    verify._cache.clear()
    q = garden.tmp / "q.jpg"
    _jpg(q, COLORS["green"], 5)
    ok = verify.verify_name(str(q), "Hosta sieboldiana", out_dir=garden.out, model="fake-bio")
    assert ok["covered"] and ok["rank"] == 1 and ok["match_score"] > 0.9
    assert ok["genus_score"] >= ok["match_score"] and ok["top"][0]["name"] == "Hosta sieboldiana"
    bad = verify.verify_name(str(q), "Iris versicolor 'Blue Flag'", out_dir=garden.out,
                             model="fake-bio")
    assert bad["match_score"] < 0.1 and bad["rank"] > 1
    cultivar_only = verify.verify_name(str(q), "Hosta 'Frances Williams'", out_dir=garden.out,
                                       model="fake-bio")
    assert cultivar_only["species"] is None and cultivar_only["match_score"] == \
        cultivar_only["genus_score"]
    with pytest.raises(ValueError):
        verify.verify_name(b"junk", "Hosta sieboldiana", out_dir=garden.out, model="fake-bio")


def test_server_local_only(garden):
    indexer.build(garden.dirs, garden.db, garden.out, workers=2, log=lambda *_: None)
    with pytest.raises(SystemExit):
        server.serve("0.0.0.0", 0, str(garden.out))  # not allowed outside Docker
    with pytest.raises(SystemExit):
        server.serve("192.168.1.5", 0, str(garden.out))
    server.Handler.out_dir = str(garden.out)
    httpd = server.ThreadingHTTPServer(("127.0.0.1", 0), server.Handler)
    port = httpd.server_address[1]
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    try:
        base = f"http://127.0.0.1:{port}"
        st = json.load(urllib.request.urlopen(base + "/status"))
        assert st["photos"]["current"]["photos"] == 4
        q = io.BytesIO()
        Image.new("RGB", (40, 40), COLORS["blue"]).save(q, "PNG")
        req = urllib.request.Request(base + "/search?k=2", data=q.getvalue(), method="POST")
        out = json.load(urllib.request.urlopen(req))
        assert out["results"][0]["plant_id"] == 7 and out["results"][0]["set"] == "future"
        for headers in ({"Host": "evil.example"}, {"Origin": "http://evil.example"}):
            bad = urllib.request.Request(base + "/status", headers=headers)
            with pytest.raises(urllib.error.HTTPError) as e:
                urllib.request.urlopen(bad)
            assert e.value.code == 403
        junk = urllib.request.Request(base + "/search", data=b"junk", method="POST")
        with pytest.raises(urllib.error.HTTPError) as e:
            urllib.request.urlopen(junk)
        assert e.value.code == 400
    finally:
        httpd.shutdown()
