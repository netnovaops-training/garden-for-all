# WHAT THIS FILE DOES (plain English)
# The /review page (HTML and JavaScript in one Python string). Approve or
# refuse the field values a tag scan or lookup proposed, then look again at
# what was approved (keep, correct, or undo).
"""The second pass, as one page.

WHY IT IS A PAGE AND NOT A REPORT. Everything Jun approves is already
visible in `/api/approved`, and reading it proves nothing -- the queue only
does its job if the person reading it can act without opening a database
client. Three buttons is the whole feature; the rest is making a row
readable enough to decide about in two seconds.

THE ONE DESIGN DECISION WORTH DEFENDING: the canonical value is always set
in a monospace face and the loose wording never is. These two strings look
alike and mean completely different things -- one is free text off a tag,
the other must match a column's existing value character for character or
the whole vocabulary gate was pointless. Giving them different faces makes
that difference visible before it is read.

SERVED BY THE SERVICE THAT OWNS THE DATA. One container to deploy, and the
page cannot drift out of step with the endpoints it calls.
"""

REVIEW_HTML = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Garden Second Pass</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Instrument+Serif:ital@0;1&family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&display=swap">
<style>
:root{
  --ground:#f2f4f0;
  --surface:#ffffff;
  --ink:#17211c;
  --ink-2:#4d5a52;
  --ink-3:#7c887f;
  --line:#d6ddd6;
  --accent:#2f6b4f;
  --accent-soft:#e4efe8;
  --undo:#a4432b;
  --undo-soft:#f6e7e2;
}
@media (prefers-color-scheme: dark){
  :root:not([data-theme="light"]){
    --ground:#11150f;
    --surface:#191e18;
    --ink:#e6ece4;
    --ink-2:#a8b4a6;
    --ink-3:#778375;
    --line:#2b332a;
    --accent:#7fc39a;
    --accent-soft:#1d2a22;
    --undo:#e0917a;
    --undo-soft:#2d1f1a;
  }
}
:root[data-theme="dark"]{
  --ground:#11150f; --surface:#191e18; --ink:#e6ece4; --ink-2:#a8b4a6;
  --ink-3:#778375; --line:#2b332a; --accent:#7fc39a; --accent-soft:#1d2a22;
  --undo:#e0917a; --undo-soft:#2d1f1a;
}
*{box-sizing:border-box}
body{
  margin:0; background:var(--ground); color:var(--ink);
  font:400 15px/1.5 "IBM Plex Sans",system-ui,-apple-system,sans-serif;
  -webkit-font-smoothing:antialiased;
}
.wrap{max-width:860px; margin:0 auto; padding:40px 16px 96px}

header{display:flex; flex-wrap:wrap; align-items:baseline; gap:16px; margin-bottom:6px}
h1{
  font:400 40px/1.05 "Instrument Serif",Georgia,serif;
  margin:0; letter-spacing:-.01em; text-wrap:balance;
}
.sub{color:var(--ink-2); margin:10px 0 28px; max-width:62ch}

.bar{display:flex; flex-wrap:wrap; gap:8px; align-items:center; margin-bottom:22px}
button{font:inherit; cursor:pointer; border-radius:7px; border:1px solid var(--line);
  background:var(--surface); color:var(--ink); padding:7px 13px}
button:focus-visible{outline:2px solid var(--accent); outline-offset:2px}
button[aria-pressed="true"]{background:var(--accent-soft); border-color:var(--accent); color:var(--ink)}
.count{font-family:"IBM Plex Mono",monospace; font-size:13px; color:var(--ink-3);
  font-variant-numeric:tabular-nums; margin-left:auto}

.card{background:var(--surface); border:1px solid var(--line); border-radius:11px;
  padding:16px 18px; margin-bottom:12px}
.card[data-status="confirmed"]{opacity:.62}
.card[data-status="reverted"]{opacity:.62}
.top{display:flex; flex-wrap:wrap; gap:10px; align-items:baseline; margin-bottom:12px}
.field{font-weight:600; letter-spacing:.02em}
.plant{color:var(--ink-2)}
.pill{font-family:"IBM Plex Mono",monospace; font-size:11px; letter-spacing:.06em;
  text-transform:uppercase; padding:3px 8px; border-radius:99px;
  border:1px solid var(--line); color:var(--ink-2)}
.pill.open{border-color:var(--accent); color:var(--accent); background:var(--accent-soft)}
.pill.undone{border-color:var(--undo); color:var(--undo); background:var(--undo-soft)}

/* THE SIGNATURE: loose wording in prose, the catalogue's own word in mono.
   They are different kinds of string and they get different faces. */
.move{display:flex; flex-wrap:wrap; align-items:center; gap:10px; margin-bottom:12px}
.raw{font-style:italic; font-family:"Instrument Serif",Georgia,serif; font-size:21px;
  color:var(--ink-2)}
.arrow{color:var(--ink-3)}
.canon{font-family:"IBM Plex Mono",monospace; font-size:16px; font-weight:500;
  background:var(--accent-soft); color:var(--ink); border-radius:5px; padding:3px 8px}
.was{font-family:"IBM Plex Mono",monospace; font-size:12px; color:var(--ink-3)}

.meta{font-size:13px; color:var(--ink-3); display:flex; flex-wrap:wrap; gap:14px;
  font-variant-numeric:tabular-nums}
.meta a{color:var(--ink-3)}
.acts{display:flex; flex-wrap:wrap; gap:8px; margin-top:14px}
.keep{border-color:var(--accent); color:var(--accent)}
.undo{border-color:var(--undo); color:var(--undo)}
select{font:inherit; font-family:"IBM Plex Mono",monospace; font-size:13px;
  padding:7px 9px; border-radius:7px; border:1px solid var(--line);
  background:var(--surface); color:var(--ink); max-width:100%; min-width:15ch}
.note{margin-top:10px; font-size:13px; color:var(--ink-3)}
.empty{border:1px dashed var(--line); border-radius:11px; padding:44px 20px;
  text-align:center; color:var(--ink-2)}
.empty strong{display:block; font:400 26px/1.2 "Instrument Serif",Georgia,serif;
  color:var(--ink); margin-bottom:6px}
footer{margin-top:40px; font-size:13px; color:var(--ink-3); max-width:62ch}
@media (prefers-reduced-motion:no-preference){
  .card{transition:opacity .2s ease}
}
</style>
</head>
<body>
<div class="wrap">
  <header>
    <h1>Second pass</h1>
  </header>
  <p class="sub"><strong>Not decided yet</strong> is the first pass: a wording
  the importer found, and what this catalogue would call it. Yes teaches the
  alias; No sends it back to be asked about again. The other two tabs are the
  second pass — everything already approved, before it settles.</p>

  <div class="bar">
    <button id="tab-new"   aria-pressed="false">Not decided yet</button>
    <button id="tab-open"  aria-pressed="true">Waiting on you</button>
    <button id="tab-all"   aria-pressed="false">Everything</button>
    <span class="count" id="counts"></span>
  </div>

  <div id="list"></div>

  <footer>Every decision here is signed with your email and dated, in the
  same table the rest of the garden reads.</footer>
</div>

<script>
// Works whether this is reached through Caddy at /garden-review/review or
// straight at the service on the LAN. Deriving the mount from the URL beats
// baking it in, because the mount is a Caddyfile line somebody may change.
const BASE = location.pathname.replace(/\/review\/?$/, "");
let SCOPE = "open", VOCAB = {};

const esc = s => String(s == null ? "" : s).replace(/[&<>"']/g,
  c => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));

async function post(path, body){
  const r = await fetch(BASE + path, {
    method:"POST", headers:{"Content-Type":"application/json"},
    body: JSON.stringify(body)});
  const d = await r.json().catch(() => ({}));
  if(!r.ok) throw new Error(d.error || ("HTTP " + r.status));
  return d;
}

function when(s){
  if(!s) return "";
  const d = new Date(s.replace(" ", "T") + "Z");
  return isNaN(d) ? s : d.toLocaleString(undefined,
    {month:"short", day:"numeric", hour:"numeric", minute:"2-digit"});
}

function card(r){
  const open = r.status === "approved";
  const el = document.createElement("div");
  el.className = "card";
  el.dataset.status = r.status;

  const srcs = (r.source || "").split(",").map(s => s.trim()).filter(Boolean);
  const host = u => { try { return new URL(u).hostname.replace(/^www\./, ""); }
                      catch(e){ return u.slice(0, 28); } };

  const options = Object.keys(VOCAB[r.field_name] || {}).sort();

  el.innerHTML = `
    <div class="top">
      <span class="field">${esc(r.field_name)}</span>
      <span class="plant">${r.plant_name ? esc(r.plant_name)
        : (r.plant_id ? "plant " + r.plant_id : "no plant attached")}</span>
      <span class="pill ${open ? "open" : (r.status === "reverted" ? "undone" : "")}">
        ${open ? "waiting" : esc(r.status)}</span>
    </div>
    <div class="move">
      <span class="raw">${esc(r.raw_value)}</span>
      <span class="arrow">&rarr;</span>
      <span class="canon">${esc(r.suggested)}</span>
      ${r.previous_value != null
        ? `<span class="was">was ${esc(r.previous_value)}</span>` : ""}
    </div>
    <div class="meta">
      <span>${esc(r.how)}${r.confidence ? " &middot; " +
        Math.round(r.confidence * 100) + "%" : ""}</span>
      <span>approved by ${esc((r.decided_by || "").split("@")[0])}
        ${esc(when(r.decided_at))}</span>
      ${r.reviewed_by ? `<span>reviewed ${esc(when(r.reviewed_at))}</span>` : ""}
      ${srcs.slice(0, 3).map(u =>
        `<a href="${esc(u)}" target="_blank" rel="noopener noreferrer">${esc(host(u))}</a>`
      ).join("")}
    </div>
    ${r.review_note ? `<div class="note">${esc(r.review_note)}</div>` : ""}
  `;

  if(open || r.status === "confirmed"){
    const acts = document.createElement("div");
    acts.className = "acts";

    if(open){
      const keep = document.createElement("button");
      keep.className = "keep"; keep.textContent = "Keep";
      keep.onclick = () => act(keep, "/api/confirm", {id:r.id});
      acts.appendChild(keep);
    }

    if(options.length){
      const sel = document.createElement("select");
      sel.innerHTML = `<option value="">Change to&hellip;</option>` +
        options.filter(o => o !== r.suggested)
               .map(o => `<option>${esc(o)}</option>`).join("");
      sel.onchange = () => { if(sel.value)
        act(sel, "/api/correct", {id:r.id, value:sel.value}); };
      acts.appendChild(sel);
    }

    const undo = document.createElement("button");
    undo.className = "undo"; undo.textContent = "Undo";
    undo.onclick = () => act(undo, "/api/revert", {id:r.id});
    acts.appendChild(undo);

    el.appendChild(acts);
  }
  return el;
}

async function act(el, path, body){
  const old = el.textContent;
  el.disabled = true;
  try{
    await post(path, body);
    await (SCOPE === "new" ? loadPending() : load());
  }catch(e){
    el.disabled = false;
    const n = document.createElement("div");
    n.className = "note"; n.textContent = e.message;
    el.parentElement.appendChild(n);
  }
}

// A proposal nobody has answered yet. Deliberately a different shape from
// card(): there is no decided_by, no previous value and nothing to undo --
// there are two buttons and the thing being asked.
function pendingCard(r){
  const el = document.createElement("div");
  el.className = "card";
  el.dataset.status = "pending";
  const srcs = (r.source || "").split(",").map(s => s.trim()).filter(Boolean);
  const host = u => { try { return new URL(u).hostname.replace(/^www\./, ""); }
                      catch(e){ return u.slice(0, 28); } };

  el.innerHTML = `
    <div class="top">
      <span class="field">${esc(r.field_name)}</span>
      <span class="plant">${r.plant_name ? esc(r.plant_name)
        : (r.plant_id ? "plant " + r.plant_id : "no plant attached")}</span>
      <span class="pill open">not decided</span>
    </div>
    <div class="move">
      <span class="raw">${esc(r.raw_value)}</span>
      ${r.suggested
        ? `<span class="arrow">&rarr;</span>
           <span class="canon">${esc(r.suggested)}</span>`
        : `<span class="was">nothing close enough to suggest</span>`}
    </div>
    <div class="meta">
      <span>${esc(r.how || "")}${r.confidence ? " &middot; " +
        Math.round(r.confidence * 100) + "% sure" : ""}</span>
      <span>seen ${esc(when(r.created_at))}</span>
      ${srcs.slice(0, 3).map(u =>
        `<a href="${esc(u)}" target="_blank" rel="noopener noreferrer">${esc(host(u))}</a>`
      ).join("")}
    </div>`;

  const acts = document.createElement("div");
  acts.className = "acts";
  if(r.suggested){
    const yes = document.createElement("button");
    yes.className = "keep"; yes.textContent = "Yes, that's it";
    yes.onclick = () => act(yes, "/api/decide", {id:r.id, approve:true});
    acts.appendChild(yes);
  }
  const no = document.createElement("button");
  no.className = "undo";
  // Honest about what "no" costs: the wording comes back next time rather
  // than being recorded as wrong forever.
  no.textContent = r.suggested ? "No" : "Not a word for this catalogue";
  no.onclick = () => act(no, "/api/decide", {id:r.id, approve:false});
  acts.appendChild(no);
  el.appendChild(acts);
  return el;
}

async function loadPending(){
  const list = document.getElementById("list");
  const r = await fetch(BASE + "/api/pending");
  if(r.status === 403){
    list.innerHTML = `<div class="empty"><strong>Not your queue</strong>
      This email is not on the garden's allowed list.</div>`;
    return;
  }
  const d = await r.json();
  const rows = d.pending || [];
  document.getElementById("counts").textContent =
    rows.length ? rows.length + " waiting to be answered" : "";
  list.textContent = "";
  if(!rows.length){
    list.innerHTML = `<div class="empty"><strong>Nothing waiting</strong>
      Everything imported so far has been answered one way or the other.</div>`;
    return;
  }
  rows.forEach(r => list.appendChild(pendingCard(r)));
}

async function load(){
  const r = await fetch(BASE + "/api/review?show=" + SCOPE);
  if(r.status === 403){
    document.getElementById("list").innerHTML =
      `<div class="empty"><strong>Not your queue</strong>
       This email is not on the garden's allowed list.</div>`;
    return;
  }
  const d = await r.json();
  VOCAB = d.vocabulary || {};
  const list = document.getElementById("list");
  list.textContent = "";

  const c = d.counts || {};
  document.getElementById("counts").textContent =
    Object.entries(c).map(([k, v]) => v + " " + k).join("   ") || "nothing yet";

  if(!(d.rows || []).length){
    list.innerHTML = SCOPE === "open"
      ? `<div class="empty"><strong>Nothing waiting</strong>
         Everything Jun has approved has been through here.</div>`
      : `<div class="empty"><strong>No decisions yet</strong>
         Proposals appear once somebody approves or rejects one.</div>`;
    return;
  }
  d.rows.forEach(r => list.appendChild(card(r)));
}

document.getElementById("tab-new").onclick  = () => setScope("new");
document.getElementById("tab-open").onclick = () => setScope("open");
document.getElementById("tab-all").onclick  = () => setScope("all");
function setScope(s){
  SCOPE = s;
  for(const t of ["new", "open", "all"])
    document.getElementById("tab-" + t).setAttribute("aria-pressed", s === t);
  // The undecided queue is a different endpoint, not a filter on the same
  // one -- /api/review only ever knew about rows that had been decided.
  (s === "new" ? loadPending : load)();
}
load();
</script>
</body>
</html>
"""
