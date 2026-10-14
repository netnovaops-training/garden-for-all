# WHAT THIS FILE DOES (plain English)
# The /wiki-queue page (HTML and JavaScript in one Python string). Shows
# wiki drafts waiting; "keep" writes one to your wiki, "remove" drops it.
"""The page where you approve or remove wiki drafts.

ONE TAP EACH WAY, AND THE SOURCE IS ON THE CARD. A tip is only worth
approving if you can check it, so the card carries a link to where it came
from. Deciding without that is just agreeing with a model.

APPROVING WRITES TO THE WIKI IMMEDIATELY, which is what was asked for. If the
write fails the draft is marked `failed` rather than `applied` and says so on
the card -- telling somebody their page went up when the wiki was down is the
most useless thing this could do, because they would never look again.

OLDEST FIRST. The draft that has waited longest is the one most likely to be
forgotten, and a queue nobody finishes is worse than no queue because it
looks handled.
"""

QUEUE_HTML = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Waiting for approval</title>
<style>
:root{--paper:#F7F6F3;--ink:#1B2019;--ink-2:#4A544A;--line:#D8D9D2;
  --leaf:#3E6B43;--leaf-lit:#5FA463;--warn:#B4472E;--care:#8A6A4B}
@media (prefers-color-scheme:dark){
  :root{--paper:#14170F;--ink:#E8EAE2;--ink-2:#A8B0A4;--line:#2A2F26;
        --leaf:#5FA463;--care:#A8865F}}
*{box-sizing:border-box}
body{margin:0;background:var(--paper);color:var(--ink);
  font:16px/1.6 ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif}
.wrap{max-width:680px;margin:0 auto;padding:20px 16px 70px}
h1{font-size:1.4rem;margin:0 0 2px}
.sub{color:var(--ink-2);margin:0 0 18px;font-size:.92rem}
.card{border:1px solid var(--line);border-radius:10px;padding:16px;margin:14px 0}
.card.gone{opacity:.45}
.kind{font-size:.7rem;letter-spacing:.14em;text-transform:uppercase;
  color:var(--ink-2);font-weight:600}
.kind.new{color:var(--leaf)}
.kind.conflict{color:var(--warn)}
.page{font-weight:700;margin:2px 0 8px}
.body{white-space:pre-wrap;margin:0 0 10px}
.src{font-size:.85rem;margin:0 0 12px}
.src a{color:var(--leaf)}
.row{display:flex;gap:8px}
button{font:inherit;flex:1;padding:12px;border-radius:8px;cursor:pointer;
  font-weight:600;border:1px solid var(--line);background:transparent;
  color:var(--ink)}
button.keep{background:var(--leaf);border-color:var(--leaf);color:#fff}
button.drop{color:var(--warn);border-color:var(--warn)}
button:disabled{opacity:.4}
.outcome{margin:10px 0 0;font-size:.88rem}
.outcome.ok{color:var(--leaf)}
.outcome.bad{color:var(--warn)}
.empty{color:var(--ink-2);font-style:italic;padding:30px 0;text-align:center}
.bot{color:var(--ink-2);font-size:.82rem;font-style:italic;margin-top:22px}
a{color:var(--leaf)}
</style>
</head>
<body>
<div class="wrap">
  <h1>Waiting for approval</h1>
  <p class="sub" id="count">Loading…</p>
  <div id="list"></div>
  <p class="bot">Nothing here is on the wiki yet. Approving puts it on the
  page straight away; removing throws it out. Every item links back to where
  it came from — check it before you keep it.</p>
  <p class="bot"><a href="diagnose">Identify a plant</a> ·
  <a href="tag">Scan a tag</a> · <a href="spend">Receipts</a></p>
</div>

<script>
const BASE = location.pathname.replace(/\/wiki-queue\/?$/, "");
const $ = s => document.querySelector(s);

function esc(s) {
  return String(s == null ? "" : s)
    .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

async function load() {
  let data;
  try {
    const r = await fetch(BASE + "/api/wiki-drafts");
    if (!r.ok) {
      $("#count").textContent = r.status === 403
        ? "You are not one of the garden's approvers."
        : "Could not load the queue.";
      return;
    }
    data = await r.json();
  } catch (e) {
    $("#count").textContent = "Could not reach the garden.";
    return;
  }

  const rows = data.drafts || data.pending || [];
  $("#count").textContent = rows.length
    ? rows.length + (rows.length === 1 ? " thing waiting" : " things waiting")
    : "";
  if (!rows.length) {
    $("#list").innerHTML =
      '<p class="empty">Nothing is waiting. Everything sent in has been decided.</p>';
    return;
  }

  $("#list").innerHTML = rows.map(d => `
    <div class="card" data-id="${d.id}">
      <div class="kind ${esc(d.kind)}">${esc(d.kind)}</div>
      <div class="page">${esc(d.title || d.page_path)}</div>
      <p class="body">${esc(d.proposed)}</p>
      ${d.source_url ? `<p class="src">From
        <a href="${esc(d.source_url)}" target="_blank" rel="noopener">
        ${esc(d.source_name || "the source")}</a> — check it before keeping it.</p>` : ""}
      <div class="row">
        <button class="keep">Add to the wiki</button>
        <button class="drop">Remove</button>
      </div>
      <p class="outcome" hidden></p>
    </div>`).join("");

  document.querySelectorAll(".card").forEach(card => {
    const id = +card.dataset.id;
    const out = card.querySelector(".outcome");
    const buttons = card.querySelectorAll("button");
    const decide = async keep => {
      buttons.forEach(b => b.disabled = true);
      out.hidden = false;
      out.className = "outcome";
      out.textContent = keep ? "Adding it to the wiki…" : "Removing…";
      try {
        const r = await fetch(BASE + "/api/wiki-decide", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ id, keep })
        });
        const o = await r.json();
        if (!r.ok || o.ok === false) {
          // A FAILED WRITE SAYS SO. The draft stays in the queue on a
          // failure, so a wiki that was briefly down costs nothing.
          out.className = "outcome bad";
          out.textContent = o.error || o.result || "That did not work.";
          buttons.forEach(b => b.disabled = false);
          return;
        }
        out.className = "outcome ok";
        out.textContent = keep
          ? "Added." + (o.result ? " " + o.result : "")
          : "Removed.";
        card.classList.add("gone");
      } catch (e) {
        out.className = "outcome bad";
        out.textContent = "Could not reach the garden.";
        buttons.forEach(b => b.disabled = false);
      }
    };
    card.querySelector(".keep").onclick = () => decide(true);
    card.querySelector(".drop").onclick = () => decide(false);
  });
}

load();
</script>
</body>
</html>
"""
