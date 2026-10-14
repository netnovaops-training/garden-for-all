# WHAT THIS FILE DOES (plain English)
# The /checkup page (HTML and JavaScript in one Python string). Pick a plant,
# take a photo, and see the AI's check-up plus earlier check-ups.
"""The page Jun uses standing in front of the plant.

BUILT FOR A PHONE HELD IN ONE HAND, because that is where it will be used.
The camera button is the biggest thing on the screen, the search box
remembers nothing, and there is no step that needs two hands or a mouse.
`capture="environment"` opens the back camera straight into the shutter
rather than the photo library.

THE PHOTOGRAPH IS SHRUNK IN THE BROWSER, BEFORE IT IS SENT. A modern phone
takes 4-12MP and the model gains nothing past about 1,400px; sending the
original is several megabytes over whatever signal exists at the bottom of
a garden, for no better answer.

THE THREE SECTIONS ARE NOT STYLED THE SAME, on purpose. What is VISIBLE is
fact and is set as plain text. What it is CONSISTENT WITH is a possibility
and is marked as one. What WOULD CONFIRM IT is the only part that asks for
action, so it is the part that looks like an instruction. Flattening them
into one paragraph is how "consistent with iron deficiency" gets read as
"has iron deficiency", and then somebody feeds a plant that is drowning.

THE HISTORY IS SHOWN UNDER IT because one photograph is a moment and the
useful thing is the trend.
"""

CHECKUP_HTML = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Plant check-up</title>
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
input,button,select{font:inherit;width:100%;padding:12px;border-radius:8px;
  border:1px solid var(--line);background:var(--paper);color:var(--ink)}
button{cursor:pointer;font-weight:600;background:var(--leaf);color:#fff;
  border-color:var(--leaf);margin-top:12px}
button:disabled{opacity:.45}
button.big{padding:20px;font-size:1.06rem}
button.quiet{background:transparent;color:var(--ink-2);border-color:var(--line);
  font-weight:500}
ul{list-style:none;margin:8px 0 0;padding:0;max-height:260px;overflow:auto}
li{border-top:1px solid var(--line);padding:11px 2px;cursor:pointer}
li:first-child{border-top:0}
li.on{color:var(--leaf);font-weight:700}
.bot{color:var(--ink-2);font-size:.82rem;font-style:italic}
.card{border:1px solid var(--line);border-radius:10px;padding:16px;margin:16px 0}
.card h2{font-size:.72rem;letter-spacing:.14em;text-transform:uppercase;
  margin:0 0 8px;color:var(--ink-2)}
#shot{max-width:100%;border-radius:10px;display:block;margin:12px 0 0}
.sec{margin:0 0 18px}
.sec h3{font-size:.72rem;letter-spacing:.13em;text-transform:uppercase;
  margin:0 0 5px}
.visible h3{color:var(--ink-2)}
.maybe h3{color:var(--care)}
.maybe p{border-left:3px solid var(--care);padding-left:11px;margin:0;
  color:var(--ink)}
.todo h3{color:var(--leaf)}
.todo p{background:rgba(95,164,99,.12);border-radius:8px;padding:11px 13px;
  margin:0}
.sec p{white-space:pre-wrap;margin:0}
.warn{color:var(--warn);font-size:.9rem;min-height:1.3em;margin-top:10px}
.foot{color:var(--ink-2);font-size:.8rem;border-top:1px solid var(--line);
  margin-top:26px;padding-top:12px}
.hist{font-size:.88rem;color:var(--ink-2);border-top:1px solid var(--line);
  padding:10px 0}
.hist b{color:var(--ink)}
:focus-visible{outline:2px solid var(--leaf);outline-offset:2px}
</style>
</head>
<body>
<div class="wrap">
  <h1>Plant check-up</h1>
  <p class="sub">Find the plant, photograph it, and see what can be read off
  the picture. It reports what is <em>visible</em> &mdash; it does not
  diagnose, and it will tell you what to check instead.</p>

  <div class="card">
    <h2>Which plant</h2>
    <input id="q" placeholder="Start typing a name" autocomplete="off">
    <ul id="hits"></ul>
  </div>

  <div class="card" id="shoot" hidden>
    <h2 id="chosenName">&mdash;</h2>
    <input id="file" type="file" accept="image/*" capture="environment" hidden>
    <button class="big" id="take">Take a photograph</button>
    <img id="shot" hidden alt="">
    <button id="go" hidden>Look at it</button>
    <p class="warn" id="msg"></p>
  </div>

  <div id="out"></div>
  <div id="history"></div>

  <p class="foot">A photograph cannot tell a nutrient shortage from too much
  water &mdash; both yellow a leaf. This narrows down what to test; a soil
  test is what settles it.</p>
</div>

<script>
const $ = s => document.querySelector(s);
const BASE = location.pathname.replace(/\/checkup\/?$/, "");
let plants = [], chosen = null, shot = null;

const esc = s => String(s == null ? "" : s).replace(/[&<>"']/g,
  c => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));

fetch(BASE + "/api/tray", {cache:"no-store"})
  .then(r => r.ok ? r.json() : {plants:[]})
  .then(d => { plants = d.plants || []; });

$("#q").oninput = () => {
  const q = $("#q").value.trim().toLowerCase();
  const ul = $("#hits");
  if (q.length < 2){ ul.innerHTML = ""; return; }
  const hits = plants.filter(p =>
    (p.plant_name || "").toLowerCase().includes(q) ||
    (p.botanical_name || "").toLowerCase().includes(q)).slice(0, 12);
  ul.innerHTML = "";
  hits.forEach(p => {
    const li = document.createElement("li");
    li.innerHTML = esc(p.plant_name)
      + (p.botanical_name ? `<br><span class="bot">${esc(p.botanical_name)}</span>` : "");
    li.onclick = () => pick(p);
    ul.appendChild(li);
  });
  if (!hits.length) ul.innerHTML = '<li class="bot">Nothing matches.</li>';
};

function pick(p){
  chosen = p; shot = null;
  $("#q").value = p.plant_name;
  $("#hits").innerHTML = "";
  $("#chosenName").textContent = p.plant_name;
  $("#shoot").hidden = false;
  $("#shot").hidden = true; $("#go").hidden = true;
  $("#out").innerHTML = ""; $("#msg").textContent = "";
  loadHistory(p.id);
}

$("#take").onclick = () => $("#file").click();

$("#file").onchange = async e => {
  const f = e.target.files[0];
  if (!f) return;
  $("#msg").textContent = "";
  shot = await shrink(f);
  $("#shot").src = shot; $("#shot").hidden = false;
  $("#go").hidden = false;
};

// Shrunk here, not on the way in. The model gains nothing past ~1400px and
// a phone photograph is several megabytes over whatever signal exists at
// the bottom of a garden.
function shrink(file, edge = 1400){
  return new Promise((res, rej) => {
    const img = new Image();
    img.onload = () => {
      const k = Math.min(1, edge / Math.max(img.width, img.height));
      const cv = document.createElement("canvas");
      cv.width = Math.round(img.width * k);
      cv.height = Math.round(img.height * k);
      cv.getContext("2d").drawImage(img, 0, 0, cv.width, cv.height);
      res(cv.toDataURL("image/jpeg", 0.85));
      URL.revokeObjectURL(img.src);
    };
    img.onerror = () => rej(new Error("that file is not a photograph"));
    img.src = URL.createObjectURL(file);
  });
}

$("#go").onclick = async () => {
  if (!chosen || !shot) return;
  $("#go").disabled = true;
  $("#go").textContent = "Looking…";
  $("#msg").textContent = "";
  try {
    const r = await fetch(BASE + "/api/checkup", {method:"POST",
      headers:{"Content-Type":"application/json"},
      body: JSON.stringify({plant_id: chosen.id, image: shot})});
    const d = await r.json();
    if (!r.ok) throw new Error(d.error || ("HTTP " + r.status));
    show(d);
    loadHistory(chosen.id);
  } catch (e){
    $("#msg").textContent = e.message;
  } finally {
    $("#go").disabled = false;
    $("#go").textContent = "Look at it";
  }
};

function show(d){
  // Three different treatments on purpose -- see the note at the top of
  // checkup_page.py. Fact, possibility, and the only part that asks for
  // anything to be done.
  $("#out").innerHTML = `
    <div class="card">
      <div class="sec visible"><h3>What is visible</h3>
        <p>${esc(d.looked_at) || "&mdash;"}</p></div>
      ${d.consistent ? `<div class="sec maybe"><h3>Consistent with</h3>
        <p>${esc(d.consistent)}</p></div>` : ""}
      ${d.confirm ? `<div class="sec todo"><h3>What would confirm it</h3>
        <p>${esc(d.confirm)}</p></div>` : ""}
    </div>`;
  $("#out").scrollIntoView({behavior:"smooth", block:"start"});
}

async function loadHistory(pid){
  const r = await fetch(BASE + "/api/checkups?plant_id=" + pid,
                        {cache:"no-store"});
  if (!r.ok) return;
  const d = await r.json();
  const rows = (d.checkups || []).slice(1);   // the newest is already shown
  $("#history").innerHTML = rows.length
    ? `<div class="card"><h2>Seen before</h2>` + rows.map(x =>
        `<div class="hist"><b>${esc(x.asked_at.slice(0,10))}</b> &mdash; `
        + `${esc((x.looked_at || "").slice(0,190))}` +
        ((x.looked_at || "").length > 190 ? "&hellip;" : "") + `</div>`
      ).join("") + `</div>`
    : "";
}
</script>
</body>
</html>
"""
