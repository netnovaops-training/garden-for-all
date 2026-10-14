# WHAT THIS FILE DOES (plain English)
# The /tag page (HTML and JavaScript in one Python string). Photograph a
# plant tag; the plant is added and its details wait for approval in /review.
"""Photograph a nursery tag instead of typing it in field by field.

JASON, 2026-09-24: "I do want him to be able to in his app to be able to
quickly add information from tags without having to manually add everything
one by one."

THE TYPING GOES AWAY. THE DECIDING DOES NOT. A plant tag is printed in the
nursery's words, not the catalogue's -- "Part sun to shade", "Full Sun/Part
Shade", "Sun or Shade" all mean something the catalogue already has one
phrase for. Anything that matches a phrase the catalogue already uses is
marked ready; anything that does not becomes a question with a suggestion
attached. Nothing is written until a person says so.

WHY NOT JUST WRITE IT ALL IN. Because the wrong wording is worse than no
wording: it splits a column into two spellings of the same thing, and every
search, filter and count downstream quietly disagrees with itself from then
on. One tap to confirm is cheap; unpicking a split vocabulary six months
later is not.

SHOT ON THE SPOT. The tag is attached to the pot in the shop or pushed into
the soil at home, and either way it is legible for about one season before
the sun takes it. The camera opens on the back lens straight into the
shutter for that reason.
"""

TAGSCAN_HTML = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Scan a plant tag</title>
<style>
:root{--paper:#F7F6F3;--ink:#1B2019;--ink-2:#4A544A;--line:#D8D9D2;
  --leaf:#3E6B43;--leaf-lit:#5FA463;--warn:#B4472E;--care:#8A6A4B}
@media (prefers-color-scheme:dark){
  :root{--paper:#14170F;--ink:#E8EAE2;--ink-2:#A8B0A4;--line:#2A2F26;
        --leaf:#5FA463;--care:#A8865F}}
*{box-sizing:border-box}
body{margin:0;background:var(--paper);color:var(--ink);
  font:16px/1.6 ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif}
.wrap{max-width:660px;margin:0 auto;padding:20px 16px 70px}
h1{font-size:1.4rem;margin:0 0 2px}
.sub{color:var(--ink-2);margin:0 0 18px;font-size:.92rem}
button{font:inherit;width:100%;padding:12px;border-radius:8px;cursor:pointer;
  font-weight:600;background:var(--leaf);color:#fff;border:1px solid var(--leaf);
  margin-top:12px}
button:disabled{opacity:.45}
button.big{padding:20px;font-size:1.06rem}
button.quiet{background:transparent;color:var(--ink-2);border-color:var(--line);
  font-weight:500;margin-top:8px}
.card{border:1px solid var(--line);border-radius:10px;padding:16px;margin:16px 0}
.card h2{font-size:.72rem;letter-spacing:.14em;text-transform:uppercase;
  margin:0 0 10px;color:var(--ink-2)}
#shot{max-width:100%;border-radius:10px;display:block;margin:12px 0 0}
.msg{margin:12px 0 0;padding:12px;border-radius:8px;border:1px solid var(--line)}
.msg.bad{border-color:var(--warn);color:var(--warn)}
table{width:100%;border-collapse:collapse}
td{padding:8px 0;border-top:1px solid var(--line);vertical-align:top}
td.k{color:var(--ink-2);width:40%;font-size:.88rem}
.tally{display:flex;gap:10px;margin:14px 0 0;flex-wrap:wrap}
.pill{border:1px solid var(--line);border-radius:999px;padding:5px 12px;
  font-size:.84rem}
.pill.ok{border-color:var(--leaf);color:var(--leaf)}
.pill.ask{border-color:var(--care);color:var(--care)}
.bot{color:var(--ink-2);font-size:.82rem;font-style:italic;margin-top:18px}
a{color:var(--leaf)}
.src{font-size:.8rem;color:var(--ink-2);word-break:break-all}
</style>
</head>
<body>
<div class="wrap">
  <h1>Scan a plant tag</h1>
  <p class="sub">Photograph the label instead of typing it. Nothing is saved
  until you approve it.</p>

  <div class="card">
    <input type="file" id="pick" accept="image/*" capture="environment" hidden>
    <button class="big" id="take">Photograph a tag</button>
    <button class="quiet" id="pickfile">Choose a photo instead</button>
    <img id="shot" hidden alt="">
    <div id="msg" hidden class="msg"></div>
  </div>

  <div class="card" id="result" hidden>
    <h2>Read from the tag</h2>
    <table id="read"></table>
    <div class="tally" id="tally"></div>
    <div id="sources"></div>
    <button id="review" hidden>Go and approve these</button>
    <p class="bot">Nothing above has been written to the catalogue. The
    fields that matched wording the catalogue already uses are ready; the
    rest are waiting with a suggestion for you to accept or refuse.</p>
  </div>

  <p class="bot"><a href="diagnose">Identify a plant</a> ·
  <a href="spend">Receipts</a> · <a href="plan">The beds</a></p>
</div>

<script>
const BASE = location.pathname.replace(/\/tag\/?$/, "");
const $ = s => document.querySelector(s);

function say(t, bad) {
  const m = $("#msg");
  m.textContent = t;
  m.className = "msg" + (bad ? " bad" : "");
  m.hidden = false;
}

// A TAG IS SMALL PRINT, so this keeps more pixels than the plant page does.
// 1,400px is plenty for a leaf and marginal for six-point type on a label.
function shrink(file, edge) {
  return new Promise((resolve, reject) => {
    const fr = new FileReader();
    fr.onerror = () => reject(new Error("could not read that file"));
    fr.onload = () => {
      const img = new Image();
      img.onerror = () => reject(new Error("could not open that picture"));
      img.onload = () => {
        const s = Math.min(1, edge / Math.max(img.width, img.height));
        const c = document.createElement("canvas");
        c.width = Math.round(img.width * s);
        c.height = Math.round(img.height * s);
        c.getContext("2d").drawImage(img, 0, 0, c.width, c.height);
        resolve(c.toDataURL("image/jpeg", 0.92));
      };
      img.src = fr.result;
    };
    fr.readAsDataURL(file);
  });
}

$("#take").onclick = () => $("#pick").click();
$("#pickfile").onclick = () => {
  const p = $("#pick");
  p.removeAttribute("capture");
  p.click();
};

$("#pick").onchange = async e => {
  const file = e.target.files[0];
  if (!file) return;
  $("#take").disabled = true;
  $("#result").hidden = true;
  say("Reading that tag…");

  let data;
  try {
    data = await shrink(file, 1800);
    const s = $("#shot"); s.src = data; s.hidden = false;
  } catch (err) {
    $("#take").disabled = false;
    say(err.message, true);
    return;
  }

  try {
    const r = await fetch(BASE + "/api/tag", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ image: data, enrich: true })
    });
    const out = await r.json();
    if (!r.ok) say(out.error || "that did not work", true);
    else if (!out.read || !Object.keys(out.read).length)
      say(out.note || "That did not read as a plant tag.", true);
    else { show(out); $("#msg").hidden = true; }
  } catch (err) {
    say("could not reach the garden: " + err.message, true);
  }
  $("#take").disabled = false;
  $("#pick").value = "";
};

function esc(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;");
}

function show(o) {
  const read = o.read || {};
  $("#read").innerHTML = Object.entries(read)
    .filter(([, v]) => v)
    .map(([k, v]) => `<tr><td class="k">${esc(k.replace(/_/g, " "))}</td>`
                   + `<td>${esc(v)}</td></tr>`).join("");

  const ready = Object.keys(o.ready || {}).length;
  const asks = (o.asks || []).length;
  const pills = [];
  if (ready) pills.push(`<span class="pill ok">${ready} ready</span>`);
  if (asks) pills.push(`<span class="pill ask">${asks} need a decision</span>`);
  if (!ready && !asks) pills.push('<span class="pill">nothing new</span>');
  $("#tally").innerHTML = pills.join("");

  $("#sources").innerHTML = (o.sources || []).length
    ? '<p class="src">Also looked up: ' + o.sources.slice(0, 4).map(esc).join(", ") + "</p>"
    : "";

  const b = $("#review");
  b.hidden = !asks;
  b.onclick = () => { location.href = BASE + "/review"; };

  $("#result").hidden = false;
  $("#result").scrollIntoView({ behavior: "smooth", block: "start" });
}
</script>
</body>
</html>
"""
