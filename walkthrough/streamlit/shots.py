#!/usr/bin/env python3
"""Pictures for the Add-plant guide, from the lab garden (lab.sh: sample data only), in English and
Korean, at desktop (1366) and phone (390) width. The desktop helper's part is a RECORDED sample run
(recorded-run.json) sent through the helper's real conversation with the website (GET
/plant-ai/jobs, POST progress and result, with a helper key made on the Add plant tab), so no AI
model runs here.

    python shots.py BASE OUTDIR        (the lab's password for Jun and Helper in env GUIDE_PW)
Prints one PASS/FAIL line per step.
"""
import base64
import io
import json
import os
import pathlib
import re
import sys

from PIL import Image, ImageDraw
from playwright.sync_api import sync_playwright

BASE, OUT = sys.argv[1].rstrip("/"), pathlib.Path(sys.argv[2])
PW = os.environ["GUIDE_PW"]
HERE = pathlib.Path(__file__).resolve().parent
RUN = json.loads((HERE / "recorded-run.json").read_text(encoding="utf-8"))
WIDTHS = {"desktop": (1366, 900), "phone": (390, 844)}
OUT.mkdir(parents=True, exist_ok=True)
ok = True


def say(good, what):
    global ok
    ok &= good
    print(("PASS " if good else "FAIL ") + what, flush=True)


def photo(seed: int, w=1400, h=1050) -> bytes:
    """A made-up 'plant' picture: violet flower spikes over green leaves (not a real photo)."""
    im = Image.new("RGB", (w, h), (46, 94, 52))
    d = ImageDraw.Draw(im)
    for i in range(160):
        x = (i * 97 + seed * 31) % w
        y = (i * 53 + seed * 17) % h
        d.ellipse([x, y, x + 60, y + 24], fill=(70 + i % 40, 140 + i % 50, 70))
    for i in range(14):
        x = 90 + i * (w - 180) // 14 + seed * 7
        top = 120 + (i * 37 + seed * 11) % 220
        for k in range(18):
            yy = top + k * 26
            d.ellipse([x - 16, yy, x + 16, yy + 22], fill=(96 + k * 3, 52, 160 + k * 3))
    b = io.BytesIO()
    im.save(b, "JPEG", quality=92)
    return b.getvalue()


def receipt() -> bytes:
    im = Image.new("RGB", (900, 1300), "white")
    d = ImageDraw.Draw(im)
    lines = ["SAMPLE NURSERY", "123 Example Road", "", "MAY NIGHT SALVIA 1G", "  3 x 8.99      26.97",
             "", "TOTAL          26.97", "", "(a made-up receipt for the guide)"]
    for i, t in enumerate(lines):
        d.text((80, 90 + i * 60), t, fill="black")
    b = io.BytesIO()
    im.save(b, "PNG")
    return b.getvalue()


def outline(page, sel):
    """Draw an orange frame round the thing to look at, and scroll it to the middle."""
    if sel:
        loc = page.locator(sel).first
        loc.evaluate("""e => { e.style.outline = '4px solid #e8590c'; e.style.outlineOffset = '3px';
                              e.scrollIntoView({block: 'center'}); }""")
        page.wait_for_timeout(300)


def shoot(ctxs, name, path, sel=None, before=None, full=False):
    """The same address in every language and width; one picture each."""
    for (lang, width), ctx in ctxs.items():
        page = ctx.new_page()
        try:
            page.goto(BASE + path, wait_until="networkidle")
            if before:
                before(page)
            outline(page, sel)
            page.screenshot(path=str(OUT / f"{name}-{lang}-{width}.png"), full_page=full)
            say(True, f"{name} {lang} {width}")
        except Exception as e:  # noqa: BLE001 -- report, then the next picture
            say(False, f"{name} {lang} {width}: {str(e)[:160]}")
        page.close()


def login(browser, user, lang, width):
    w, h = WIDTHS[width]
    ctx = browser.new_context(viewport={"width": w, "height": h}, locale="ko-KR" if lang == "ko" else "en-US")
    ctx.request.post(f"{BASE}/login.php", form={"admin_user": user, "admin_password": PW, "admin_login": "", "back": "/"})
    ctx.add_cookies([{"name": "st_lang", "value": lang, "url": BASE + "/streamlit/"}])
    return ctx


def fill_add(page):
    p = RUN["plant"]
    page.fill("input[name=genus]", p["genus"])
    page.fill("input[name=species]", p["species"])
    page.fill("input[name=name]", p["name"])
    page.select_option("select[name=bed]", p["bed"])
    page.fill("input[name=qty]", p["qty"])
    page.fill("input[name=price]", p["price"])


def main():
    with sync_playwright() as pw:
        browser = pw.chromium.launch()
        ctxs = {(lang, width): login(browser, "Jun", lang, width) for lang in ("en", "ko") for width in WIDTHS}
        jun = ctxs[("en", "desktop")]
        # 1. The six boxes.
        shoot(ctxs, "01-add-form", "/streamlit/?tab=15", "form:has(input[name=genus])", fill_add)
        # 2. Add the plant (once): the page shows it waiting in the queue.
        page = jun.new_page()
        page.goto(BASE + "/streamlit/?tab=15", wait_until="networkidle")
        fill_add(page)
        page.click("button[name=go][value='1'] >> nth=0")
        page.wait_for_load_state("networkidle")
        m = re.search(r"[?&]p=(\d+)", page.url)
        pid = m.group(1) if m else ""
        say(bool(pid), f"plant added (id {pid})")
        page.close()
        shoot(ctxs, "02-queued", f"/streamlit/?tab=15&p={pid}", "h3.st-h3 + .pa-bar")
        # 3. A helper key (the owner makes it; shown once). The picture shows an example key.
        page = jun.new_page()
        page.goto(BASE + "/streamlit/?tab=15", wait_until="networkidle")
        page.click("details.st-expander summary")
        page.fill("input[name=label]", "Desktop PC")
        page.click("form:has(input[value=pa_key_new]) button")
        page.wait_for_load_state("networkidle")
        key = page.input_value("input.pa-key")
        say(bool(re.fullmatch(r"gw_[0-9a-f]{40}", key)), "helper key made")
        page.close()
        example = "gw_" + "0123456789abcdef" * 2 + "01234567"
        shoot({k: v for k, v in ctxs.items() if k[1] == "desktop"}, "03-helper-box", "/streamlit/?tab=15",
              "details.st-expander", lambda p: p.click("details.st-expander summary"))
        # 4. The helper's side, recorded: take the job, report progress, send the answers.
        api = jun.request
        auth = {"Authorization": f"Worker {key}"}
        r = api.get(f"{BASE}/plant-ai/jobs?take=5", headers=auth)
        jobs = r.json().get("jobs", []) if r.ok else []
        say(len(jobs) == 1 and str(jobs[0]["plant"]["id"]) == pid, "the helper took the job")
        jid = jobs[0]["id"] if jobs else 0
        bad = api.get(f"{BASE}/plant-ai/jobs?take=5", headers={"Authorization": "Worker " + example})
        say(bad.status == 403, "a wrong helper key is refused (403)")
        for step, name in (("searching", "04-searching"), ("filling", "05-filling"), ("checking", "06-checking")):
            api.post(f"{BASE}/plant-ai/jobs/{jid}/progress", headers=auth, data={"step": step})
            if name == "05-filling":
                shoot(ctxs, name, f"/streamlit/?tab=15&p={pid}", "h3.st-h3 + .pa-bar")
        photos = [{"name": f"{i}.webp", "data": base64.b64encode(photo(i)).decode(), "credit": c,
                   "source": f"https://grower.example.org/photos/{i}.jpg"}
                  for i, c in enumerate(RUN["photo_credits"], 1)]
        r = api.post(f"{BASE}/plant-ai/jobs/{jid}/result", headers=auth,
                     data={"result": RUN["result"], "photos": photos})
        say(r.ok and r.json().get("photos") == 3, f"the answers are in ({r.text()[:80]})")
        # 5. The review.
        shoot(ctxs, "07-review", f"/streamlit/?tab=15&p={pid}", ".pa-photos")
        shoot(ctxs, "08-filled", f"/streamlit/?tab=15&p={pid}", "h4 + .st-df")
        shoot(ctxs, "09-needs-review", f"/streamlit/?tab=15&p={pid}", "form:has(input[value=pa_use])")
        shoot(ctxs, "10-missing", f"/streamlit/?tab=15&p={pid}", "a[href*='export=pa_missing']")
        ex = api.get(f"{BASE}/streamlit/?export=pa_missing&id={pid}")
        doc = ex.json() if ex.ok else {}
        (OUT / "still-missing-sample.json").write_text(json.dumps(doc, ensure_ascii=False, indent=2), encoding="utf-8")
        say("prompt_en" in doc and isinstance(doc.get("answer_template"), list), "the still-missing file")
        shoot(ctxs, "11-approve", f"/streamlit/?tab=15&p={pid}", "form:has(input[value=pa_approve])")
        helper = {("en", "desktop"): login(browser, "Helper", "en", "desktop"),
                  ("ko", "desktop"): login(browser, "Helper", "ko", "desktop")}
        shoot(helper, "12-not-owner", f"/streamlit/?tab=15&p={pid}", "h4:has-text('Approve'), h4:has-text('승인')")
        page = jun.new_page()
        page.goto(f"{BASE}/streamlit/?tab=15&p={pid}", wait_until="networkidle")
        page.click("form:has(input[value=pa_approve]) button")
        page.wait_for_load_state("networkidle")
        say("Approved" in page.content(), "approved by the owner")
        page.close()
        shoot(ctxs, "13-approved", f"/streamlit/?tab=15&p={pid}",
              "div.st-alert.st-success:has-text('Approved by the garden owner'), div.st-alert.st-success:has-text('정원 주인이 승인')")
        # 6. Receipt: one sample receipt with the plant.
        rc = OUT / "sample-receipt.png"
        rc.write_bytes(receipt())

        def pick_receipt(p):
            p.fill("input[name=store]", "Sample Nursery")
            p.set_input_files("input[name=receipt]", str(rc))
        shoot(ctxs, "14-receipt", f"/streamlit/?tab=16&p={pid}", "form:has(input[value=pa_receipt])", pick_receipt)
        page = jun.new_page()
        page.goto(f"{BASE}/streamlit/?tab=16&p={pid}", wait_until="networkidle")
        pick_receipt(page)
        page.click("form:has(input[value=pa_receipt]) button")
        page.wait_for_load_state("networkidle")
        page.close()
        shoot(ctxs, "15-receipt-saved", f"/streamlit/?tab=16&p={pid}", ".st-df")
        # 7. Pictures: add two more.
        pics = [OUT / "sample-photo-a.jpg", OUT / "sample-photo-b.jpg"]
        for i, f in enumerate(pics, 7):
            f.write_bytes(photo(i, 1600, 1200))
        shoot(ctxs, "16-pictures", f"/streamlit/?tab=17&p={pid}", "form:has(input[value=pa_pics])",
              lambda p: p.set_input_files("input[type=file]", [str(x) for x in pics]))
        page = jun.new_page()
        page.goto(f"{BASE}/streamlit/?tab=17&p={pid}", wait_until="networkidle")
        page.set_input_files("input[type=file]", [str(x) for x in pics])
        page.click("form:has(input[value=pa_pics]) button")
        page.wait_for_load_state("networkidle")
        page.close()
        shoot(ctxs, "17-pictures-added", f"/streamlit/?tab=17&p={pid}", ".pa-photos")
        # 8. Advanced: the JSON import, now under Advanced.
        shoot(ctxs, "18-advanced", "/streamlit/?tab=11&sub=4", "details.st-advanced")
        for c in list(ctxs.values()) + list(helper.values()):
            c.close()
        browser.close()
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
