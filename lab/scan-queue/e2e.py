# ruff: noqa: S603, S607, S608, S108
# (A lab script: it runs docker and builds SQL only from its own made-up values, on a throw-away database.)
"""The scan queue, end to end, in the lab: a fake phone -> the site's API -> the desktop helper's
worker (real tag reader in Docker, real matching) -> a best match -> Jun taps -> plant and bed.

    python lab/scan-queue/e2e.py [--keep] [--live-web]

What is real: the Rust website (built from this folder) and its MariaDB, in a throw-away Docker
project on 127.0.0.1:18030 with made-up plants; the tag reader container (garden-ocr, started here
if it is not running); the helper's real code (plantai.scans). What is faked: the phone (httpx), the
vision model (off: the GPU belongs to another run) and, unless --live-web, the internet look-up
(a stand-in page). Nothing touches the real garden, the NUC or junsgarden.zapto.org.

Needs: Docker, the image gfa-scan-lab/site (docker build --build-arg APP=junsgarden -t
gfa-scan-lab/site rust), the image garden-ocr (docker build -t garden-ocr plant-ai/ocr), and in
Python: httpx, Pillow, argon2-cffi. Prints PASS or FAIL per step; exits 1 if any step failed.
"""
import argparse
import base64
import hashlib
import json
import os
import secrets
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import httpx

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "plant-ai"))
BASE = "http://127.0.0.1:18030"
COMPOSE = ROOT / "lab" / "scan-queue" / "docker-compose.yml"
DB = "gfa-scan-lab-db"
FAILED: list[str] = []


def check(name: str, ok: bool, detail: str = "") -> bool:
    print(("PASS " if ok else "FAIL ") + name + (f"  [{detail}]" if detail and not ok else ""))
    if not ok:
        FAILED.append(name)
    return ok


def run(*cmd, env=None, input=None, check_rc=True) -> str:
    r = subprocess.run(cmd, capture_output=True, text=True, env={**os.environ, **(env or {})}, input=input)
    if check_rc and r.returncode:
        raise RuntimeError(f"{cmd[0]} {cmd[1] if len(cmd) > 1 else ''} failed: {r.stderr[-400:]}")
    return r.stdout


class Lab:
    def __init__(self, work: Path):
        self.work = work
        self.pw = secrets.token_hex(12)          # a random database password, never printed
        self.env = {"LAB_DIR": str(work), "LAB_DB_PASSWORD": self.pw}

    def compose(self, *args):
        return run("docker", "compose", "-f", str(COMPOSE), *args, env=self.env)

    def sql(self, q: str) -> list[list[str]]:
        out = run("docker", "exec", "-i", "-e", f"MYSQL_PWD={self.pw}", DB, "mariadb", "-uroot", "-N", "-B",
                  "juns_garden_db", input=q)
        return [line.split("\t") for line in out.splitlines() if line.strip()]

    def up(self) -> None:
        shutil.copytree(ROOT / "site", self.work / "site", ignore=shutil.ignore_patterns("images", "data", "cashe"))
        for d in ("images", "data", "cashe"):
            (self.work / d).mkdir()
            (self.work / d).chmod(0o777)
        self.compose("up", "-d")
        for _ in range(90):
            try:
                if httpx.get(f"{BASE}/app_api.php?action=ping", timeout=3).status_code == 200:
                    return
            except httpx.HTTPError:
                pass
            time.sleep(2)
        raise RuntimeError("the lab site did not start: " + run("docker", "logs", "--tail", "20", "gfa-scan-lab-site", check_rc=False))

    def down(self) -> None:
        subprocess.run(["docker", "compose", "-f", str(COMPOSE), "down", "-v", "--remove-orphans"], env={**os.environ, **self.env},
                       capture_output=True)


def login(lab: Lab, user: str) -> str:
    """Give `user` a random password and sign in with it; gives the app key."""
    from argon2 import PasswordHasher
    pw = secrets.token_urlsafe(12)
    lab.sql(f"UPDATE garden_users SET pass_hash='{PasswordHasher().hash(pw)}' WHERE username='{user}'")
    r = httpx.post(f"{BASE}/app_api.php?action=login", data={"username": user, "password": pw}, timeout=20)
    return r.json()["token"]


def b64(path: Path) -> str:
    return base64.b64encode(path.read_bytes()).decode()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--keep", action="store_true", help="leave the lab running afterwards")
    ap.add_argument("--live-web", action="store_true", help="really search the internet for the tag that is not in the garden")
    a = ap.parse_args()
    work = Path(tempfile.mkdtemp(prefix="gfa-scan-lab-"))
    lab = Lab(work)
    started_ocr = False
    try:
        # --- the tag reader ---------------------------------------------------------------
        from plantai import config, scans, synth, tagocr, worker
        if not tagocr.service_up():
            run("docker", "run", "-d", "--name", "garden-ocr-lab", "-p", "127.0.0.1:8788:8098", "--read-only", "--tmpfs", "/tmp",
                "--cap-drop", "ALL", "--security-opt", "no-new-privileges", "garden-ocr")
            started_ocr = True
            for _ in range(30):
                if tagocr.service_up():
                    break
                time.sleep(2)
        check("tag reader (OCR container) is answering", tagocr.service_up())

        # --- made-up tags and the lab site -------------------------------------------------
        tags = work / "tags"
        truth = synth.make(tags, count=3, seed=11, level="normal")
        lab.up()
        check("lab website is up", True)
        # Made-up garden: two plants (one with the first tag's barcode) and one wish-list plant.
        lab.sql(f"""
          INSERT INTO data_source (plant_name, genus, species, botanical_name, garden_beds, flower_bed, qty, died, year_added, barcode)
            VALUES ('{truth[0]['name']}', '{truth[0]['genus']}', '{truth[0]['species']}', "{truth[0]['botanical']}", 'Zen Garden', 'Zen Garden', 2, 0, '2026', NULL),
                   ('{truth[1]['name']}', '{truth[1]['genus']}', '', "{truth[1]['botanical']}", 'Korea Garden', 'Korea Garden', 1, 0, '2026', NULL);
          INSERT INTO plant_barcodes (plant_id, source_table, barcode)
            SELECT id, 'data_source', '{truth[0]['barcode']}' FROM data_source WHERE plant_name = '{truth[0]['name']}';
          INSERT INTO future_plants (plant_name, genus, species, botanical_name)
            VALUES ('Karl Foerster Feather Reed Grass', 'Calamagrostis', 'acutiflora', "Calamagrostis x acutiflora 'Karl Foerster'");""")
        # The ids the database gave them (the schema starts counting high), as text for the checks.
        P1 = lab.sql(f"SELECT id FROM data_source WHERE plant_name = '{truth[0]['name']}'")[0][0]
        P2 = lab.sql(f"SELECT id FROM data_source WHERE plant_name = '{truth[1]['name']}'")[0][0]
        jun, author = login(lab, "Jun"), login(lab, "Author")
        hj, ha = {"Authorization": f"Bearer {jun}"}, {"Authorization": f"Bearer {author}"}
        wkey = "gw_" + secrets.token_hex(20)
        lab.sql(f"INSERT INTO plant_ai_keys (label, key_sha256) VALUES ('lab', '{hashlib.sha256(wkey.encode()).hexdigest()}')")
        c = httpx.Client(timeout=60)

        # --- the phone ----------------------------------------------------------------------
        check("a visitor without a key is refused", c.get(f"{BASE}/app/scan/items").status_code == 401)
        beds = c.get(f"{BASE}/app/scan/beds", headers=hj).json()["beds"]
        by_name = {b["name"]: b["id"] for b in beds}
        check("the beds come with numbers (To Be Sorted first, in use ones included)",
              beds[0]["name"] == "To Be Sorted" and "Zen Garden" in by_name and "Korea Garden" in by_name, str(beds))
        zen, korea = by_name["Zen Garden"], by_name["Korea Garden"]

        def post_item(uid, kind, bed, qty, sides, headers=hj):
            return c.post(f"{BASE}/app/scan/items", headers=headers, json={
                "client_uid": uid, "kind": kind, "bed_id": bed, "qty": qty,
                "photos": [{"side": s, "data": b64(p)} for s, p in sides]})

        d = lambda n: tags / truth[n]["dir"]  # noqa: E731
        blank = work / "blank.jpg"
        from PIL import Image
        Image.new("RGB", (900, 1200), (128, 128, 128)).save(blank)
        flower = work / "plant.jpg"
        Image.new("RGB", (1000, 800), (60, 140, 60)).save(flower)
        r = post_item("lab-scan-0000001a", "tag", zen, 2, [("front", d(0) / "front.jpg"), ("back", d(0) / "back.jpg")])
        check("scan A (tag in the garden, two sides) is queued", r.status_code == 201 and r.json()["item"]["status"] == "waiting", r.text[:200])
        post_item("lab-scan-0000002b", "tag", korea, 3, [("front", d(2) / "front.jpg"), ("back", d(2) / "back.jpg")])
        post_item("lab-scan-0000003c", "tag", None, 1, [("front", blank)])
        post_item("lab-scan-0000004d", "plant", korea, 1, [("plant", flower)])
        post_item("lab-scan-0000005e", "tag", korea, 1, [("front", d(1) / "front.jpg")])
        dup = post_item("lab-scan-0000001a", "tag", zen, 2, [("front", d(0) / "front.jpg")])
        check("sending the same scan twice makes one item", dup.json().get("duplicate") is True and
              len(c.get(f"{BASE}/app/scan/items", headers=hj).json()["items"]) == 5)
        check("a bad picture is refused, nothing half-saved", post_item("lab-scan-bad00001", "tag", None, 1, []).status_code == 400
              and c.post(f"{BASE}/app/scan/items", headers=hj, json={"client_uid": "lab-scan-bad00002", "photos": [{"side": "front", "data": "aGVsbG8="}]}).status_code == 400
              and len(lab.sql("SELECT id FROM scan_items")) == 5)
        check("the author's phone may queue scans too", post_item("lab-scan-0000006f", "tag", korea, 1, [("front", d(1) / "front.jpg")], ha).status_code == 201)
        # Jun renames a bed while the scans wait: a scan holds the bed by NUMBER.
        lab.sql("UPDATE garden_bed SET name = 'Zen Garden (renamed)' WHERE name = 'Zen Garden'")

        # --- the desktop helper --------------------------------------------------------------
        config.use(work / "helper-work", work / "helper-out")
        site = worker.Site(BASE, wkey)
        tagocr.gemma_read = lambda _imgs: None        # the graphics card belongs to another run

        if a.live_web:
            print("(live web look-up is on: DuckDuckGo is really asked for tag B)")
        else:
            def fake_web(fields):
                hit = "coneflower" in fields.get("name", "").lower().replace(" ", "")
                return {"candidates": [], "confirms": hit, "botanical": "", "url": "https://example.edu/echinacea-magnus" if hit else "", "notes": []}
            scans.webfind.lookup = fake_web
        t0 = time.monotonic()
        n = scans.run_all(site)
        check(f"the helper worked through the queue ({n} scans, {time.monotonic() - t0:.0f} s)", n == 6, str(n))

        items = {i["client_uid"]: i for i in c.get(f"{BASE}/app/scan/items", headers=hj).json()["items"]}
        A, B = items["lab-scan-0000001a"], items["lab-scan-0000002b"]
        C, D, E = items["lab-scan-0000003c"], items["lab-scan-0000004d"], items["lab-scan-0000005e"]
        check("A: matched, best = the garden plant, confidence over 90%",
              A["status"] == "matched" and str(A["candidates"][0]["plant_id"]) == P1 and A["candidates"][0]["confidence"] > 0.9 and "barcode" in A["candidates"][0]["why"], json.dumps(A["candidates"][:1]))
        check("A: its bed is shown by the NEW name (renamed after scanning)", A["bed_name"] == "Zen Garden (renamed)", A["bed_name"])
        check("B: a plant not in the garden comes from the tag, confirmed by the web page, shown to Jun",
              B["status"] == "matched" and B["candidates"][0]["source"] == "tag" and B["candidates"][0]["botanical"].startswith("Echinacea")
              if not a.live_web else B["status"] in ("matched", "needs_you"), json.dumps(B["candidates"][:2]))
        check("C: an empty photo needs you (type it in)", C["status"] == "needs_you" and not C["candidates"], C["status"])
        check("D: a plant photo with no image index needs you, with a note", D["status"] == "needs_you")
        check("E: a front-only tag still finds the garden plant", E["status"] in ("matched", "needs_you") and any(str(x["plant_id"]) == P2 for x in E["candidates"]), json.dumps(E["candidates"][:2]))
        check("every candidate carries a confidence and a source",
              all(0 <= x["confidence"] <= 1 and x["source"] for i in items.values() for x in i["candidates"]))
        row = lab.sql("SELECT result FROM scan_items WHERE client_uid='lab-scan-0000004d'")[0][0]
        check("D's note says the image index is off", "image index" in row, row[-200:])

        # The website's own page for the same queue: matches with their percent; buttons for Jun only.
        page = c.get(f"{BASE}/streamlit/?tab=18", headers=hj).text
        check("the Scan queue tab shows the matches with their confidence, and Jun's confirm buttons",
              "Scan queue" in page and "Echinacea" in page and "sq_confirm" in page and "99%" in page, page[:200])
        page_a = c.get(f"{BASE}/streamlit/?tab=18", headers=ha).text
        check("the author sees the matches but no confirm button", "Echinacea" in page_a and "sq_confirm" not in page_a)

        # --- one tap: the garden owner only ----------------------------------------------------
        r = c.post(f"{BASE}/app/scan/items/{A['id']}/confirm", headers=ha, json={"pick": "c1"})
        check("the author (not the owner) cannot confirm", r.status_code == 403 and r.json().get("owner_only") is True, r.text[:200])
        r = c.post(f"{BASE}/app/scan/items/{A['id']}/confirm", headers=hj, json={"pick": "c1"})
        check("Jun confirms A: same plant, count added", r.status_code == 200 and r.json()["merged"] is True and str(r.json()["plant_id"]) == P1, r.text[:200])
        row = lab.sql(f"SELECT qty, garden_beds, flower_bed FROM data_source WHERE id={P1}")[0]
        check("A: the count went 2 -> 4 and the bed list has the renamed bed", row[0] == "4" and "Zen Garden (renamed)" in row[1] and "Zen Garden (renamed)" in row[2], str(row))
        check("A: a copy of the plant was kept first", lab.sql(f"SELECT COUNT(*) FROM plant_ai_backups WHERE plant_id={P1}")[0][0] == "1")
        r2 = c.post(f"{BASE}/app/scan/items/{A['id']}/confirm", headers=hj, json={"pick": "c1"})
        check("a second tap changes nothing (no double count)", r2.status_code == 200 and lab.sql(f"SELECT qty FROM data_source WHERE id={P1}")[0][0] == "4")

        r = c.post(f"{BASE}/app/scan/items/{B['id']}/confirm", headers=hj, json={"pick": "c1"})
        pid = r.json().get("plant_id", 0)
        check("Jun confirms B: a new plant", r.status_code == 200 and r.json()["merged"] is False and pid > int(P2), r.text[:200])
        row = lab.sql(f"SELECT plant_name, genus, species, botanical_name, flower_bed, garden_beds, qty FROM data_source WHERE id={pid}")[0]
        check("B: the row has name, genus, species, the bed's name and the count 3",
              row[1] == "Echinacea" and row[2] == "purpurea" and row[4] == "Korea Garden" and row[5] == "Korea Garden" and row[6] == "3", str(row))
        job = lab.sql(f"SELECT status FROM plant_ai_jobs WHERE plant_id={pid}")
        check("B: the normal fill-and-check helper is queued for it", job and job[0][0] == "queued", str(job))

        r = c.post(f"{BASE}/app/scan/items/{C['id']}/confirm", headers=hj, json={"manual": {"genus": "Hosta", "species": "", "name": "Typed By Hand"}})
        check("C: Jun types it in (manual), in the default bed", r.status_code == 200 and lab.sql(
            "SELECT flower_bed FROM data_source WHERE plant_name='Typed By Hand'")[0][0] == "To Be Sorted", r.text[:200])
        r = c.post(f"{BASE}/app/scan/items/{D['id']}/bed", headers=hj, json={"bed_id": zen})
        check("D: the bed of a waiting scan can be changed", r.status_code == 200)
        r = c.post(f"{BASE}/app/scan/items/{D['id']}/remove", headers=hj, json={})
        check("D: a scan can be taken out of the queue (its picture file too)", r.status_code == 200 and not lab.sql(f"SELECT id FROM scan_photos WHERE item_id={D['id']}"))
        r = c.post(f"{BASE}/app/scan/items/{B['id']}/remove", headers=hj, json={})
        check("an added scan cannot be removed", r.status_code == 409)

        # --- manual add, and the website's own page --------------------------------------------
        r = c.post(f"{BASE}/app/scan/manual", headers=hj, json={"genus": "Salvia", "species": "nemorosa", "name": "Lab Salvia", "qty": 2, "price": 7.5, "bed_id": korea})
        got = lab.sql("SELECT flower_bed, qty, average_cost FROM data_source WHERE plant_name='Lab Salvia'")[0]
        check("manual add: genus, species, name, qty, price and bed",
              r.status_code == 201 and got[:2] == ["Korea Garden", "2"] and float(got[2]) == 7.5, f"{r.text[:100]} {got}")
        r = c.post(f"{BASE}/app/scan/manual", headers=hj, json={"genus": "", "name": "x"})
        check("manual add refuses a missing genus", r.status_code == 400)
        page15 = c.get(f"{BASE}/streamlit/?tab=15", headers=hj).text
        check("the Add plant form has the bed picker by number", 'name="bed_id"' in page15 and "Zen Garden (renamed)" in page15)
        r = c.post(f"{BASE}/streamlit/", headers=hj, data={"act": "pa_add", "tab": "15", "genus": "Hosta", "species": "", "name": "Form Hosta", "bed_id": str(zen), "qty": "1", "price": "3"}, follow_redirects=False)
        check("the Add plant form saves with the bed's current name", r.status_code in (302, 303) and lab.sql("SELECT flower_bed FROM data_source WHERE plant_name='Form Hosta'")[0][0] == "Zen Garden (renamed)")
        files = {"front": ("f.jpg", (d(1) / "front.jpg").read_bytes(), "image/jpeg")}
        r = c.post(f"{BASE}/streamlit/", headers=hj, data={"act": "sq_add", "tab": "18", "bed_id": str(korea), "qty": "1"}, files=files, follow_redirects=False)
        check("the website's phone page can add a scan (camera upload)", r.status_code in (302, 303) and len(lab.sql("SELECT id FROM scan_items")) == 6 - 1 + 1)
        # --- the helper's doors are closed to everyone else -------------------------------------
        check("the helper's doors need the helper key", c.get(f"{BASE}/plant-ai/scans", headers=hj).status_code == 401
              and c.get(f"{BASE}/plant-ai/catalog").status_code == 401 and c.get(f"{BASE}/plant-ai/scan-photos/1").status_code == 401)
        check("a picture is handed out only while its scan is being read",
              c.get(f"{BASE}/plant-ai/scan-photos/1", headers={"Authorization": f"Worker {wkey}"}).status_code == 404)
        check("the photos are not on a public web address",
              c.get(f"{BASE}/data/scans/1_1_front.jpg").status_code == 404 and c.get(f"{BASE}/scans/1_1_front.jpg").status_code == 404)
    finally:
        if not a.keep:
            lab.down()
            if started_ocr:
                subprocess.run(["docker", "rm", "-f", "garden-ocr-lab"], capture_output=True)
            shutil.rmtree(work, ignore_errors=True)
    print(("FAIL: " + "; ".join(FAILED)) if FAILED else "PASS all steps")
    return 1 if FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
