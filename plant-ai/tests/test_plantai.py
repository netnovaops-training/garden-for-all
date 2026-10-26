"""Tests that need no network, no Ollama and no garden data: the value rules, the result rules,
the photo rules and the helper's conversation with the website (a stand-in website in memory).

    python -m pytest plant-ai/tests -q
"""
import base64
import json
import sys
from pathlib import Path

import httpx
import pytest
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from plantai import config, db, images, norm, result, worker


@pytest.fixture
def work(tmp_path):
    config.use(tmp_path / "work", tmp_path / "out")
    return tmp_path


def test_norm_enums_and_formats():
    assert norm.norm("sun_exposure", "part shade") == ("Partial Shade", "alias 'part shade'")
    assert norm.norm("plant_type", "perennial")[0] == "Perennial"
    assert norm.norm("leaf_shape", "Ovate")[0] == "Ovate (Egg-shaped)"
    assert norm.norm("zones", "4-8 (annual in 1-3)")[0] == "4-8"
    assert norm.norm("soil_ph", "6 to 7.5")[0] == "6.0-7.5"
    assert norm.norm("plant_length", "24-36 inches")[0] == "24-36 in"
    assert norm.norm("spacing", "18 in")[0] == "18 in apart"
    assert norm.norm("bloom_months", "June-August")[0] == ".....JJA...."
    assert norm.norm("attracts", "bees and hummingbirds")[0] == "Bees, Hummingbirds"
    # Never written by the AI: the owner's own fields.
    assert norm.norm("approved_jun", "1")[0] is None
    assert norm.norm("quantity", "3")[0] is None
    # Not in the allowed list: rejected, not guessed.
    assert norm.norm("growth_rate", "very speedy")[0] is None


def _stage_plant(pset="new", pid=7):
    rec = {"id": pid, "plant_name": "Sample Sage", "genus": "Salvia", "species": "nemorosa",
           "sun_exposure": "Full Sun"}
    db.load(pset, [rec])
    fills = {
        "zones": {"v": "4-8", "conf": "high", "sources": ["https://a.example/p"], "quotes": ["Zones 4-8"], "why": ""},
        "bloom_color": {"v": "Violet", "conf": "medium", "sources": ["https://a.example/p"], "quotes": ["violet"],
                        "why": ""},
        "water_needs": {"v": "Low", "conf": "low", "sources": ["https://b.example/q"], "quotes": ["dry"], "why": ""},
    }
    db.put(pset, pid, "fill", {"fills": fills, "contradictions": [], "confirmed": [], "unnormalized": [],
                               "sources": [], "product_url": "https://a.example/p", "product_kind": "vendor"})
    claims = [{"f": "zones", "v": "4-8", "kind": "fill", "gemma": "yes", "gemma_why": "stated"},
              {"f": "bloom_color", "v": "Violet", "kind": "fill", "gemma": "yes", "gemma_why": "stated"},
              {"f": "water_needs", "v": "Low", "kind": "fill", "gemma": "no", "gemma_why": "page says moist"}]
    db.put(pset, pid, "check", {"identity": "yes", "identity_why": "same", "claims": claims, "photos": [],
                                "decision": "Needs review", "reason": "one value unsupported"})
    return rec


def test_result_keeps_only_supported_values(work):
    rec = _stage_plant()
    res = result.plant_result("new", 7, rec)
    assert res["fields"] == {"zones": "4-8", "bloom_color": "Violet"}
    assert set(res["needs_review"]) == {"water_needs"}
    assert "sun_exposure" not in res["missing"]          # already filled by the owner
    assert "zones" not in res["missing"] and "family" in res["missing"]
    assert res["product_url"] == "https://a.example/p"


def test_result_rejected_writes_nothing(work):
    rec = _stage_plant()
    c = db.get("new", 7, "check")
    c["decision"] = "Reject"
    db.put("new", 7, "check", c)
    res = result.plant_result("new", 7, rec)
    assert res["fields"] == {}
    assert {"zones", "bloom_color", "water_needs"} <= set(res["needs_review"])


def _photo(path: Path, size, sharp=True):
    im = Image.new("RGB", size, (40, 120, 40))
    d = ImageDraw.Draw(im)
    if sharp:
        for x in range(0, size[0], 12):
            d.line([(x, 0), (x, size[1])], fill=(250, 240, 90), width=3)
    im.save(path, "JPEG", quality=95)


def test_one_webp_no_thumbnail_and_size_limits(work, monkeypatch):
    big, small = work / "big.jpg", work / "small.jpg"
    _photo(big, (3000, 2000))
    _photo(small, (400, 300))
    pages = {"https://x.example/big.jpg": big, "https://x.example/small.jpg": small}
    monkeypatch.setattr(images.fetch, "get", lambda u: {"status": 200, "type": "image/jpeg", "url": u,
                                                        "path": str(pages[u])} if u in pages else
                        {"status": 404, "type": "", "url": u, "path": ""})
    got = images.collect(3, list(pages), want=5)
    assert len(got) == 1                                   # the 400 px one is too small
    assert max(got[0]["out_w"], got[0]["out_h"]) == 2000   # shrunk once to 2000
    files = sorted(p.name for p in (work / "out" / "images" / "3").iterdir())
    assert files == ["1.webp", "rejected.txt"]             # one picture, no thumbnail file


class FakeSite:
    """The website's side of the helper's conversation, in memory."""

    def __init__(self, key):
        self.key, self.jobs, self.posts = key, [], []

    def handler(self, req: httpx.Request) -> httpx.Response:
        if req.headers.get("authorization") != f"Worker {self.key}":
            return httpx.Response(403)
        if req.method == "GET" and req.url.path == "/plant-ai/jobs":
            out, self.jobs = self.jobs, []
            return httpx.Response(200, json={"jobs": out})
        self.posts.append((req.url.path, json.loads(req.content)))
        return httpx.Response(200, json={"ok": True})


def test_worker_round_trip(work, monkeypatch):
    site = FakeSite("k" * 40)
    site.jobs = [{"id": 11, "plant": {"id": 7, "plant_name": "Sample Sage", "genus": "Salvia",
                                      "species": "nemorosa"}}]
    monkeypatch.setattr(worker.llm, "ready", lambda: (True, "ok"))

    def fake_run(work_list, progress=None):
        for pset, pid, _ in work_list:
            progress(pset, pid, "filling")
            _stage_plant(pset, pid)
            img = work / "out" / "images" / str(pid)
            img.mkdir(parents=True, exist_ok=True)
            Image.new("RGB", (800, 600), (1, 2, 3)).save(img / "1.webp", "WEBP")
            db.put(pset, pid, "images", [{"file": f"images/{pid}/1.webp", "src": "https://a.example/1.jpg",
                                          "src_w": 800, "src_h": 600, "out_w": 800, "out_h": 600, "flags": []}])

    monkeypatch.setattr(worker.batch, "run", fake_run)
    s = worker.Site("https://garden.example", site.key, httpx.Client(transport=httpx.MockTransport(site.handler)))
    assert worker.run_once(s) == 1
    paths = [p for p, _ in site.posts]
    assert paths == ["/plant-ai/jobs/11/progress", "/plant-ai/jobs/11/result"]
    body = site.posts[-1][1]
    assert body["result"]["fields"]["zones"] == "4-8"
    assert base64.b64decode(body["photos"][0]["data"])[:4] == b"RIFF"   # a .webp file


def test_worker_wrong_key_is_refused(work):
    site = FakeSite("right-key-123456")
    s = worker.Site("https://garden.example", "wrong-key", httpx.Client(transport=httpx.MockTransport(site.handler)))
    with pytest.raises(httpx.HTTPStatusError):
        s.take(1)
