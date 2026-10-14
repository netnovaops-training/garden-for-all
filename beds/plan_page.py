# WHAT THIS FILE DOES (plain English)
# The /plan page, the bed planner (HTML and JavaScript in one Python
# string). Draw beds, then drag plants into them.
"""The page where Jun draws his beds.

WHY DRAWING AND NOT A FORM. He knows his garden by walking it, not by its
coordinates. A form asking for "x offset, y offset, width, depth" is a form
he would have to translate his own garden into, and every translation is a
chance to be wrong in a way nobody can see afterwards. Dragging a shape on
a grid and typing "12 feet" on its edge is the same knowledge with nothing
in between.

THE GRID IS IN FEET AND THE DATA IS IN INCHES. Gardens are talked about in
feet; plant spacing is written in inches. Storing inches and labelling feet
means neither has to be converted by a person.

CLICK TO PLACE CORNERS, not drag-a-rectangle, because real beds curve
around a lawn and follow a fence. The area in square feet updates as it is
drawn, which is the number that catches a wrong scale immediately: a 4x12
bed is 48 square feet, and if the readout says 4,800 the garden size is out
by a factor of ten.

IT WORKS ON A TABLET, because the person who knows where the beds are will
be standing in them. Pointer events, not mouse events, and nothing depends
on hover.

SAME SIGN-IN AS EVERY OTHER PANEL, and the service checks the address on
its own side too -- being signed in to the house is not the same as being
allowed to redraw somebody's garden.
"""

PLAN_HTML = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Garden beds</title>
<style>
:root{
  --paper:#F7F6F3; --ink:#1B2019; --ink-2:#4A544A; --line:#D8D9D2;
  --leaf:#3E6B43; --leaf-lit:#5FA463; --soil:#8A6A4B; --warn:#B4472E;
}
@media (prefers-color-scheme:dark){
  :root{--paper:#14170F; --ink:#E8EAE2; --ink-2:#A8B0A4; --line:#2A2F26;
        --leaf:#5FA463; --soil:#A8865F;}
}
*{box-sizing:border-box}
body{margin:0;background:var(--paper);color:var(--ink);
  font:15px/1.55 ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif}
.wrap{max-width:1180px;margin:0 auto;padding:22px 18px 60px}
h1{font-size:1.5rem;margin:0 0 2px;letter-spacing:-.01em}
.sub{color:var(--ink-2);margin:0 0 20px;max-width:64ch}
.cols{display:grid;grid-template-columns:1fr 310px;gap:20px;align-items:start}
@media (max-width:860px){.cols{grid-template-columns:1fr}}

.board{position:relative;background:var(--paper);border:1px solid var(--line);
  border-radius:10px;overflow:hidden;touch-action:none}
canvas{display:block;width:100%;height:auto;cursor:crosshair}
.hud{position:absolute;left:10px;top:10px;font:600 12px/1.5 ui-monospace,monospace;
  background:rgba(0,0,0,.62);color:#fff;padding:5px 9px;border-radius:5px;
  letter-spacing:.04em;pointer-events:none}
.hint{position:absolute;right:10px;bottom:10px;font:12px/1.4 ui-monospace,monospace;
  background:rgba(0,0,0,.62);color:#fff;padding:5px 9px;border-radius:5px;
  pointer-events:none;max-width:60%}

.panel{border:1px solid var(--line);border-radius:10px;padding:16px;margin:0 0 16px}
.panel h2{font-size:.76rem;letter-spacing:.14em;text-transform:uppercase;
  color:var(--ink-2);margin:0 0 12px;font-weight:600}
label{display:block;font-size:.78rem;color:var(--ink-2);margin:10px 0 3px}
input,select,textarea,button{font:inherit;width:100%;padding:8px 10px;
  border:1px solid var(--line);border-radius:6px;background:var(--paper);
  color:var(--ink)}
textarea{min-height:56px;resize:vertical}
button{cursor:pointer;font-weight:600;background:var(--leaf);color:#fff;
  border-color:var(--leaf);margin-top:12px}
button.quiet{background:transparent;color:var(--ink-2);border-color:var(--line);
  font-weight:500}
button:disabled{opacity:.45;cursor:default}
.row{display:flex;gap:8px}
.row button{margin-top:0}

ul.beds{list-style:none;margin:0;padding:0}
ul.beds li{border-top:1px solid var(--line);padding:10px 0;display:flex;
  gap:10px;align-items:baseline;cursor:pointer}
ul.beds li:first-child{border-top:0}
ul.beds li:hover .nm{color:var(--leaf)}
ul.beds li.on .nm{color:var(--leaf);font-weight:700}
.nm{flex:1;font-weight:600}
.sz{font:12px ui-monospace,monospace;color:var(--ink-2)}
.empty{color:var(--ink-2);font-style:italic}
.note{font-size:.8rem;color:var(--warn);margin-top:8px;min-height:1.2em}
.ok{color:var(--leaf)}
:focus-visible{outline:2px solid var(--leaf);outline-offset:2px}
</style>
</head>
<body>
<div class="wrap">
  <h1>Garden beds</h1>
  <p class="sub">Click along the edge of a bed to put its corners down, then
  close it and give it a name. The grid is one foot per square &mdash; check
  the area reads about what you'd expect before saving.</p>

  <div class="cols">
    <div>
      <div class="board">
        <canvas id="c" width="1600" height="1100"></canvas>
        <div class="hud" id="hud">&mdash;</div>
        <div class="hint" id="hint">Click to start a bed</div>
      </div>
      <div class="row" style="margin-top:12px">
        <button class="quiet" id="mode-beds" aria-pressed="true">Draw beds</button>
        <button class="quiet" id="mode-plant" aria-pressed="false">Plant</button>
      </div>

      <div id="bedtools" class="row" style="margin-top:10px">
        <button class="quiet" id="undo">Undo corner</button>
        <button class="quiet" id="cancel">Cancel</button>
        <button id="close" disabled>Close the shape</button>
      </div>

      <div id="planttools" hidden style="margin-top:10px">
        <label>Month &mdash; <b id="monthName">all year</b></label>
        <input id="month" type="range" min="0" max="12" step="1" value="0">
        <p class="sub" style="margin:6px 0 0;font-size:.82rem">
          Slide to a month and anything not in bloom then goes grey. That is
          how you find the gap before you buy for it.</p>
      </div>

      <label>Garden size across (feet)</label>
      <input id="scale" type="number" min="10" max="400" step="5" value="80">
    </div>

    <div>
      <div class="panel">
        <h2 id="formTitle">New bed</h2>
        <label>Name</label>
        <input id="b-name" placeholder="Front Garden Beds">
        <label>Sun</label>
        <select id="b-sun">
          <option value="">&mdash; not set &mdash;</option>
          <option>Full sun</option><option>Part sun</option>
          <option>Part shade</option><option>Full shade</option>
        </select>
        <label>Soil</label>
        <input id="b-soil" placeholder="clay, amended">
        <label>Notes</label>
        <textarea id="b-notes" placeholder="Anything worth remembering"></textarea>
        <button id="save" disabled>Save bed</button>
        <div class="note" id="msg"></div>
      </div>

      <div class="panel" id="traypanel" hidden>
        <h2>Plants <span class="sz" id="traycount"></span></h2>
        <input id="q" placeholder="Search by name">
        <label style="margin-top:10px">
          <input type="checkbox" id="onlyUnsorted" checked
                 style="width:auto;margin-right:6px">Only unfiled
        </label>
        <ul class="beds" id="tray" style="max-height:340px;overflow:auto">
          <li class="empty">Loading&hellip;</li>
        </ul>
        <div class="note" id="traymsg"></div>
      </div>

      <div class="panel">
        <h2>Beds <span class="sz" id="count"></span></h2>
        <ul class="beds" id="list"><li class="empty">None yet.</li></ul>
        <button class="quiet" id="del" disabled>Delete selected bed</button>
      </div>
    </div>
  </div>
</div>

<script>
const $ = s => document.querySelector(s);
const BASE = location.pathname.replace(/\/plan\/?$/, "");
const c = $("#c"), g = c.getContext("2d");

let FEET = 80;               // how many feet the board is across
let beds = [], draft = [], selected = null;
let MODE = "beds";            // "beds" or "plant"
let plants = [], placed = [], chosen = null, MONTH = 0;
const MONTHS = ["all year","January","February","March","April","May","June",
                "July","August","September","October","November","December"];

// bloom_months is a twelve-character mask -- ".....JJASO.." is June to
// October -- so "is this in bloom in month n" is one character lookup.
// Read off the live data rather than assumed; see the note in beds.py.
const bloomsIn = (mask, m) =>
  !m || !mask || mask.length < 12 ? true : mask[m-1] !== ".";

// A plant's own colour, for the dot on the plan. bloom_color is free text
// written by a person -- "pale pink to white." -- so this matches the
// first word it recognises and falls back to leaf green rather than
// pretending to understand the sentence.
const NAMED = {red:"#B4472E", pink:"#D97A8E", rose:"#C9557B", purple:"#7D5BA6",
  violet:"#7D5BA6", blue:"#4A6FA5", lavender:"#9B8EC4", white:"#E8E4D9",
  cream:"#E6DCC3", yellow:"#D9B23F", gold:"#C9992E", orange:"#D2802F",
  peach:"#E0A878", burgundy:"#7A2E3C", magenta:"#B5478F", green:"#5FA463",
  silver:"#AFB6B0", coral:"#E07856"};
function colourOf(p){
  const t = (p.bloom_color || p.foliage_color || "").toLowerCase();
  for (const k in NAMED) if (t.includes(k)) return NAMED[k];
  return "#5FA463";
}

const esc = s => String(s == null ? "" : s).replace(/[&<>"']/g,
  ch => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[ch]));

// Inches per canvas pixel. Everything stored is inches; the canvas is just
// a window onto it, so changing the garden size never changes the data.
const perPx = () => (FEET * 12) / c.width;
const toIn = (px, py) => [px * perPx(), py * perPx()];
const toPx = (x, y) => [x / perPx(), y / perPx()];

function areaSqFt(shape){
  let t = 0;
  for (let i = 0; i < shape.length; i++){
    const [x1,y1] = shape[i], [x2,y2] = shape[(i+1) % shape.length];
    t += x1*y2 - x2*y1;
  }
  return Math.abs(t) / 2 / 144;
}

function draw(){
  const css = getComputedStyle(document.documentElement);
  g.clearRect(0,0,c.width,c.height);
  g.fillStyle = css.getPropertyValue("--paper").trim() || "#F7F6F3";
  g.fillRect(0,0,c.width,c.height);

  // One line per foot, a heavier one every ten. Without the ten-foot rule
  // it is impossible to judge scale at a glance.
  const step = 12 / perPx();
  for (let i = 0; i * step < Math.max(c.width, c.height); i++){
    const p = i * step;
    g.strokeStyle = (i % 10 === 0) ? "rgba(128,128,128,.40)" : "rgba(128,128,128,.14)";
    g.lineWidth = (i % 10 === 0) ? 1.6 : 1;
    g.beginPath(); g.moveTo(p,0); g.lineTo(p,c.height); g.stroke();
    g.beginPath(); g.moveTo(0,p); g.lineTo(c.width,p); g.stroke();
  }

  beds.forEach(b => paint(b.shape, b === selected, b.name));
  if (draft.length) paint(draft, true, null, true);
  if (MODE === "plant") paintPlantings();
}

function paintPlantings(){
  // Drawn at the plant's real footprint, so two things too close for each
  // other visibly overlap. That overlap IS the spacing warning -- a number
  // in a panel is something you read; a collision is something you see.
  const clash = new Set();
  for (let i = 0; i < placed.length; i++){
    for (let j = i+1; j < placed.length; j++){
      const a = placed[i], b = placed[j];
      const need = (foot(a) + foot(b)) / 2;
      if (Math.hypot(a.x-b.x, a.y-b.y) < need * 0.82){ clash.add(i); clash.add(j); }
    }
  }
  placed.forEach((pl, i) => {
    const info = plants.find(x => x.id === pl.plant_id) || {};
    const r = (foot(pl) / 2) / perPx();
    const [px,py] = toPx(pl.x, pl.y);
    const lit = bloomsIn(info.bloom_months, MONTH);

    g.beginPath(); g.arc(px,py,Math.max(6,r),0,Math.PI*2);
    g.fillStyle = lit ? colourOf(info) + "cc" : "rgba(140,140,140,.28)";
    g.fill();
    g.lineWidth = clash.has(i) ? 4 : 2;
    g.strokeStyle = clash.has(i) ? "#B4472E" : (lit ? "#1B2019aa" : "#8888");
    g.stroke();

    g.fillStyle = lit ? "#1B2019" : "#8A8F8A";
    g.font = "600 19px ui-sans-serif,system-ui,sans-serif";
    g.textAlign = "center";
    g.fillText((info.plant_name || "?").slice(0,22), px, py + Math.max(6,r) + 22);
  });
  $("#hud").textContent = placed.length
    ? `${placed.length} planted` + (clash.size ? ` · ${clash.size} too close` : "")
      + (MONTH ? ` · ${MONTHS[MONTH]}` : "")
    : "Nothing planted yet";
}

// The footprint the plan draws with. The server already decided which
// column to believe; this only has to find it again for a placed row.
function foot(pl){
  const info = plants.find(x => x.id === pl.plant_id);
  return (info && info.footprint_in) || 18;
}

function paint(shape, active, name, isDraft){
  if (!shape.length) return;
  g.beginPath();
  shape.forEach(([x,y], i) => {
    const [px,py] = toPx(x,y);
    i ? g.lineTo(px,py) : g.moveTo(px,py);
  });
  if (!isDraft) g.closePath();
  g.fillStyle = active ? "rgba(95,164,99,.30)" : "rgba(138,106,75,.22)";
  if (!isDraft) g.fill();
  g.strokeStyle = active ? "#5FA463" : "#8A6A4B";
  g.lineWidth = 3; g.stroke();

  shape.forEach(([x,y]) => {
    const [px,py] = toPx(x,y);
    g.beginPath(); g.arc(px,py,7,0,Math.PI*2);
    g.fillStyle = active ? "#5FA463" : "#8A6A4B"; g.fill();
  });

  if (name){
    const xs = shape.map(p => p[0]), ys = shape.map(p => p[1]);
    const [px,py] = toPx((Math.min(...xs)+Math.max(...xs))/2,
                         (Math.min(...ys)+Math.max(...ys))/2);
    g.fillStyle = getComputedStyle(document.documentElement)
                    .getPropertyValue("--ink").trim() || "#1B2019";
    g.font = "600 26px ui-sans-serif,system-ui,sans-serif";
    g.textAlign = "center";
    g.fillText(name, px, py);
  }
}

function hud(){
  if (draft.length >= 2){
    const a = areaSqFt(draft);
    const [lx,ly] = draft[draft.length-2], [x,y] = draft[draft.length-1];
    const edge = Math.hypot(x-lx, y-ly) / 12;
    $("#hud").textContent =
      `${draft.length} corners · last edge ${edge.toFixed(1)} ft · ` +
      (draft.length >= 3 ? `${a.toFixed(0)} sq ft` : "—");
  } else if (draft.length === 1){
    $("#hud").textContent = "1 corner · keep clicking";
  } else if (selected){
    $("#hud").textContent =
      `${selected.name} · ${areaSqFt(selected.shape).toFixed(0)} sq ft`;
  } else {
    $("#hud").textContent = "—";
  }
  $("#close").disabled = draft.length < 3;
  $("#save").disabled = !(draft.length >= 3 || selected);
  $("#del").disabled = !selected;
}

c.addEventListener("pointerdown", e => {
  const r = c.getBoundingClientRect();
  const px = (e.clientX - r.left) * (c.width / r.width);
  const py = (e.clientY - r.top) * (c.height / r.height);

  const [ix,iy] = toIn(px,py);

  if (MODE === "plant"){
    // Tap a planted dot to lift it; tap empty ground in a bed to put the
    // chosen plant there. Lifting first, because a mis-tap that plants a
    // second copy is more annoying to undo than one that removes nothing.
    const hitPl = placed.find(pl =>
      Math.hypot(pl.x-ix, pl.y-iy) < Math.max(foot(pl)/2, 9 * perPx()));
    if (hitPl){ lift(hitPl); return; }
    const bed = beds.find(b => pointIn(b.shape, ix, iy));
    if (!bed){ sayTray("That spot is not in a bed.", false); return; }
    if (!chosen){ sayTray("Pick a plant from the list first.", false); return; }
    plant(bed, ix, iy);
    return;
  }

  // Clicking an existing bed selects it -- but only when not mid-draw,
  // because a corner landing on top of another bed is ordinary.
  if (!draft.length){
    const hit = beds.find(b => pointIn(b.shape, ix, iy));
    if (hit){ select(hit); return; }
  }
  draft.push(toIn(px,py));
  $("#hint").textContent = draft.length >= 3
    ? "Close the shape, or keep going" : "Keep clicking the corners";
  draw(); hud();
});

function pointIn(shape, x, y){
  let hit = false;
  for (let i = 0; i < shape.length; i++){
    const [x1,y1] = shape[i], [x2,y2] = shape[(i+1) % shape.length];
    if ((y1 > y) !== (y2 > y)){
      if (x < (x2-x1)*(y-y1)/(y2-y1)+x1) hit = !hit;
    }
  }
  return hit;
}

function select(b){
  selected = b; draft = [];
  $("#formTitle").textContent = "Edit bed";
  $("#b-name").value = b.name; $("#b-sun").value = b.sun || "";
  $("#b-soil").value = b.soil || ""; $("#b-notes").value = b.notes || "";
  $("#hint").textContent = "Editing " + b.name;
  draw(); hud();
}

function clearForm(){
  selected = null; draft = [];
  $("#formTitle").textContent = "New bed";
  for (const id of ["#b-name","#b-soil","#b-notes"]) $(id).value = "";
  $("#b-sun").value = "";
  $("#hint").textContent = "Click to start a bed";
  draw(); hud();
}

$("#undo").onclick = () => { draft.pop(); draw(); hud(); };
$("#cancel").onclick = clearForm;
$("#close").onclick = () => { $("#hint").textContent = "Name it, then save";
                              $("#b-name").focus(); };
$("#scale").onchange = e => {
  FEET = Math.max(10, Math.min(400, +e.target.value || 80));
  draw(); hud();
};

async function post(path, body){
  const r = await fetch(BASE + path, {method:"POST",
    headers:{"Content-Type":"application/json"}, body: JSON.stringify(body)});
  const d = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(d.error || ("HTTP " + r.status));
  return d;
}

$("#save").onclick = async () => {
  const name = $("#b-name").value.trim();
  if (!name){ say("A bed needs a name.", false); return; }
  const shape = draft.length >= 3 ? draft : (selected ? selected.shape : null);
  if (!shape){ say("Draw the bed first.", false); return; }
  try {
    await post("/api/bed", {
      id: selected ? selected.id : null, name, shape,
      sun: $("#b-sun").value, soil: $("#b-soil").value,
      notes: $("#b-notes").value});
    say("Saved.", true);
    clearForm(); await load();
  } catch (e){ say(e.message, false); }
};

$("#del").onclick = async () => {
  if (!selected) return;
  if (!confirm(`Delete "${selected.name}"? Anything planted in it is `
             + `unplaced, but no plant record is deleted.`)) return;
  try { await post("/api/bed-delete", {id: selected.id});
        clearForm(); await load(); say("Deleted.", true); }
  catch (e){ say(e.message, false); }
};

function say(text, good){
  const n = $("#msg"); n.textContent = text;
  n.className = "note" + (good ? " ok" : "");
  setTimeout(() => { if (n.textContent === text) n.textContent = ""; }, 4000);
}

async function load(){
  const r = await fetch(BASE + "/api/beds", {cache:"no-store"});
  if (r.status === 403){
    document.querySelector(".cols").innerHTML =
      '<p class="empty">This page is for the garden&rsquo;s keepers.</p>';
    return;
  }
  const d = await r.json();
  beds = (d.beds || []).map(b => ({...b,
    shape: typeof b.shape === "string" ? JSON.parse(b.shape) : b.shape}));
  $("#count").textContent = beds.length ? `(${beds.length})` : "";
  const ul = $("#list");
  ul.innerHTML = beds.length ? "" : '<li class="empty">None yet.</li>';
  beds.forEach(b => {
    const li = document.createElement("li");
    if (b === selected) li.className = "on";
    li.innerHTML = `<span class="nm">${esc(b.name)}</span>`
      + `<span class="sz">${areaSqFt(b.shape).toFixed(0)} sq ft`
      + (b.planted ? ` · ${b.planted} planted` : "") + `</span>`;
    li.onclick = () => select(b);
    ul.appendChild(li);
  });
  draw(); hud();
}

// ---- plant mode ---------------------------------------------------------
function setMode(m){
  MODE = m; draft = [];
  $("#mode-beds").setAttribute("aria-pressed", m === "beds");
  $("#mode-plant").setAttribute("aria-pressed", m === "plant");
  $("#bedtools").hidden = m !== "beds";
  $("#planttools").hidden = m !== "plant";
  $("#traypanel").hidden = m !== "plant";
  $("#hint").textContent = m === "plant"
    ? (beds.length ? "Pick a plant, then tap where it goes"
                   : "Draw a bed first")
    : "Click to start a bed";
  draw(); hud();
}
$("#mode-beds").onclick = () => setMode("beds");
$("#mode-plant").onclick = () => setMode("plant");

$("#month").oninput = e => {
  MONTH = +e.target.value;
  $("#monthName").textContent = MONTHS[MONTH];
  draw();
};

$("#q").oninput = drawTray;
$("#onlyUnsorted").onchange = drawTray;

function drawTray(){
  const q = $("#q").value.trim().toLowerCase();
  const onlyU = $("#onlyUnsorted").checked;
  const list = plants.filter(p =>
    (!onlyU || p.unsorted) &&
    (!q || (p.plant_name || "").toLowerCase().includes(q)
        || (p.botanical_name || "").toLowerCase().includes(q)));
  $("#traycount").textContent = `(${list.length})`;
  const ul = $("#tray");
  ul.innerHTML = list.length ? "" : '<li class="empty">Nothing matches.</li>';
  list.slice(0, 250).forEach(p => {
    const li = document.createElement("li");
    if (chosen && chosen.id === p.id) li.className = "on";
    const inches = p.footprint_in;
    li.innerHTML = `<span class="nm">${esc(p.plant_name)}</span>`
      + `<span class="sz">${(inches/12).toFixed(1)}'</span>`;
    li.onclick = () => {
      chosen = p; drawTray();
      $("#hint").textContent = "Tap where " + p.plant_name + " goes";
    };
    ul.appendChild(li);
  });
  if (list.length > 250)
    ul.insertAdjacentHTML("beforeend",
      `<li class="empty">${list.length - 250} more &mdash; search to narrow.</li>`);
}

async function plant(bed, x, y){
  try {
    await post("/api/place", {bed_id: bed.id, plant_id: chosen.id, x, y});
    sayTray(`${chosen.plant_name} filed into ${bed.name}.`, true);
    await loadTray(); await load();
  } catch (e){ sayTray(e.message, false); }
}

async function lift(pl){
  const info = plants.find(x => x.id === pl.plant_id) || {};
  if (!confirm(`Take ${info.plant_name || "this"} off the plan?`)) return;
  try {
    const d = await post("/api/unplace", {id: pl.id});
    sayTray(d.still_placed
      ? "Removed one; others remain."
      : "Removed, and put back on the unfiled list.", true);
    await loadTray(); await load();
  } catch (e){ sayTray(e.message, false); }
}

function sayTray(text, good){
  const n = $("#traymsg"); n.textContent = text;
  n.className = "note" + (good ? " ok" : "");
  setTimeout(() => { if (n.textContent === text) n.textContent = ""; }, 4000);
}

async function loadTray(){
  const r = await fetch(BASE + "/api/tray", {cache:"no-store"});
  if (!r.ok) return;
  const d = await r.json();
  plants = d.plants || [];
  placed = (d.placed || []).map(p => ({...p, x:+p.x, y:+p.y}));
  drawTray();
}

loadTray();

load();
</script>
</body>
</html>
"""
