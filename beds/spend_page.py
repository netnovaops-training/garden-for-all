# WHAT THIS FILE DOES (plain English)
# The /spend page (HTML and JavaScript in one Python string). Add a receipt
# photo, see what the garden has cost by year and shop, and name receipt
# lines that did not match a plant.
"""What the garden has cost, and the receipt that says so.

THE AUTHOR, 2026-09-24: somewhere for Jun to keep track of all the plants he has
bought over the years, so he has an accurate price for each one.

THE RECEIPT IS PHOTOGRAPHED IN THE CAR PARK, NOT FILED AT HOME. That is the
only moment the slip reliably exists -- it goes in a pocket, through a wash,
or into a drawer with forty others. So the camera button is the biggest
thing on the page and everything else is below it, the same shape as the
check-up page, for the same reason.

WHAT WAS NOT MATCHED IS SHOWN, NOT HIDDEN. A line the catalogue does not
recognise is a plant Jun owns that the catalogue does not know about, and
that is worth seeing. It sits at the top with a box to name it, because an
unmatched line is thirty seconds of work and a wrongly matched one is a
number he would believe.

THE MONEY IS COUNTED EITHER WAY. An unmatched line still carries its price
into the year and shop totals -- it is a plant he bought, whatever it turns
out to be called.
"""

SPEND_HTML = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Plant spending</title>
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
  font-weight:500;margin-top:8px}
.card{border:1px solid var(--line);border-radius:10px;padding:16px;margin:16px 0}
.card h2{font-size:.72rem;letter-spacing:.14em;text-transform:uppercase;
  margin:0 0 10px;color:var(--ink-2)}
.big-n{font-size:2.3rem;line-height:1.1;font-weight:700;
  font-variant-numeric:tabular-nums}
.big-n small{display:block;font-size:.8rem;font-weight:400;color:var(--ink-2);
  letter-spacing:.04em}
table{width:100%;border-collapse:collapse;font-variant-numeric:tabular-nums}
th{text-align:left;font-size:.72rem;letter-spacing:.1em;text-transform:uppercase;
  color:var(--ink-2);font-weight:600;padding:0 0 6px}
td{padding:8px 0;border-top:1px solid var(--line);vertical-align:top}
td.n,th.n{text-align:right;white-space:nowrap}
.scroll{overflow-x:auto}
.name{font-weight:600}
.where{color:var(--ink-2);font-size:.84rem}
.unk{border-left:3px solid var(--care);padding-left:10px;margin:10px 0}
.unk .line{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;
  font-size:.82rem;color:var(--ink-2);white-space:pre-wrap;word-break:break-word}
.bot{color:var(--ink-2);font-size:.82rem;font-style:italic}
.msg{margin:12px 0 0;padding:12px;border-radius:8px;border:1px solid var(--line)}
.msg.bad{border-color:var(--warn);color:var(--warn)}
#shot{max-width:100%;border-radius:10px;display:block;margin:12px 0 0}
a{color:var(--leaf)}
</style>
</head>
<body>
<div class="wrap">
  <h1>Plant spending</h1>
  <p class="sub">Every plant bought, what it cost, and where it came from.</p>

  <div class="card">
    <h2>Add a receipt</h2>
    <!-- capture="environment" opens the back camera straight into the
         shutter rather than the photo library, which is the difference
         between one tap and four while standing in a car park. -->
    <input type="file" id="pick" accept="image/*"
           capture="environment" hidden>
    <button class="big" id="take">Photograph a receipt</button>
    <button class="quiet" id="pickfile">Choose a photo instead</button>
    <img id="shot" hidden alt="">
    <div id="msg" hidden class="msg"></div>
  </div>

  <div class="card" id="totals" hidden>
    <h2>Altogether</h2>
    <div class="big-n" id="total">—<small id="count"></small></div>
  </div>

  <div class="card" id="unmatched-card" hidden>
    <h2>Not in the catalogue yet</h2>
    <div id="unmatched"></div>
  </div>

  <div class="card" id="years-card" hidden>
    <h2>By year</h2>
    <div class="scroll"><table id="years"></table></div>
  </div>

  <div class="card" id="stores-card" hidden>
    <h2>Where they came from</h2>
    <div class="scroll"><table id="stores"></table></div>
  </div>

  <p class="bot">Prices come from the receipts themselves. A line that could
  not be read is left blank rather than counted as nothing.</p>
</div>

<script>
// The page is mounted under a prefix (/beds/spend) and derives it from
// its own URL, so it moves without a code change -- same as the check-up
// and planner pages.
const BASE = location.pathname.replace(/\/spend\/?$/, "");
const $ = s => document.querySelector(s);
const money = v => v == null ? "" : "$" + Number(v).toFixed(2);

function say(text, bad) {
  const m = $("#msg");
  m.textContent = text;
  m.className = "msg" + (bad ? " bad" : "");
  m.hidden = false;
}

// SHRUNK IN THE BROWSER, BEFORE IT IS SENT. A phone photograph of a receipt
// is several megabytes; the text stops getting more legible somewhere around
// 1,600px on the long edge, and the rest is upload time over whatever signal
// exists outside a garden centre.
function shrink(file, edge) {
  return new Promise((resolve, reject) => {
    const fr = new FileReader();
    fr.onerror = () => reject(new Error("could not read that file"));
    fr.onload = () => {
      if (file.type === "application/pdf") { resolve(fr.result); return; }
      const img = new Image();
      img.onerror = () => reject(new Error("could not open that picture"));
      img.onload = () => {
        const s = Math.min(1, edge / Math.max(img.width, img.height));
        const c = document.createElement("canvas");
        c.width = Math.round(img.width * s);
        c.height = Math.round(img.height * s);
        c.getContext("2d").drawImage(img, 0, 0, c.width, c.height);
        resolve(c.toDataURL("image/jpeg", 0.85));
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
  say("Reading that receipt…");
  let body;
  try {
    const data = await shrink(file, 1600);
    if (file.type === "application/pdf") {
      body = { file: data, filename: file.name };
    } else {
      body = { image: data };
      const s = $("#shot"); s.src = data; s.hidden = false;
    }
  } catch (err) {
    $("#take").disabled = false;
    say(err.message, true);
    return;
  }
  try {
    const r = await fetch(BASE + "/api/receipt", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const out = await r.json();
    if (!r.ok) { say(out.error || "that did not work", true); }
    else { say(out.message); load(); }
  } catch (err) {
    say("could not reach the garden: " + err.message, true);
  }
  $("#take").disabled = false;
  $("#pick").value = "";
};

function rows(table, head, data, cells) {
  table.innerHTML = "<tr>" + head.map((h, i) =>
    `<th${i ? ' class="n"' : ""}>${h}</th>`).join("") + "</tr>" +
    data.map(d => "<tr>" + cells(d).map((c, i) =>
      `<td${i ? ' class="n"' : ""}>${c}</td>`).join("") + "</tr>").join("");
}

async function load() {
  let s;
  try {
    const r = await fetch(BASE + "/api/spend");
    if (!r.ok) return;
    s = await r.json();
  } catch (e) { return; }

  if (s.total_spent != null) {
    $("#total").firstChild.nodeValue = money(s.total_spent);
    $("#count").textContent = (s.plants_bought || 0) + " plants";
    $("#totals").hidden = false;
  }
  if ((s.by_year || []).length) {
    rows($("#years"), ["Year", "Plants", "Spent"], s.by_year,
         d => [d.y, d.plants, money(d.spent)]);
    $("#years-card").hidden = false;
  }
  if ((s.by_store || []).length) {
    rows($("#stores"), ["Shop", "Lines", "Spent"], s.by_store,
         d => [d.store, d.line_count, money(d.spent)]);
    $("#stores-card").hidden = false;
  }
  loadUnmatched();
}

async function loadUnmatched() {
  let list;
  try {
    const r = await fetch(BASE + "/api/unmatched");
    if (!r.ok) return;
    list = (await r.json()).unmatched || [];
  } catch (e) { return; }
  if (!list.length) { $("#unmatched-card").hidden = true; return; }

  $("#unmatched").innerHTML = list.map(u => `
    <div class="unk" data-id="${u.id}">
      <div class="name">${u.matched_name || "(no name read)"} ${money(u.line_total)}</div>
      <div class="line">${u.line_text.replace(/</g, "&lt;")}</div>
      <div class="where">${u.store || ""} ${u.bought_on || ""}</div>
      <input list="plants" placeholder="Which plant is this?" class="pick-plant">
      <button class="quiet save">Save</button>
    </div>`).join("");
  $("#unmatched-card").hidden = false;

  document.querySelectorAll(".unk .save").forEach(b => {
    b.onclick = async () => {
      const box = b.closest(".unk");
      const name = box.querySelector(".pick-plant").value.trim();
      if (!name) return;
      b.disabled = true;
      const r = await fetch(BASE + "/api/purchase-name", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ purchase_id: +box.dataset.id, name })
      });
      const out = await r.json();
      if (r.ok) { load(); } else { say(out.error || "that did not work", true); b.disabled = false; }
    };
  });
}

// The catalogue names, so naming an unmatched line is a pick and not a typo.
(async () => {
  try {
    const r = await fetch(BASE + "/api/plant-names");
    if (!r.ok) return;
    const dl = document.createElement("datalist");
    dl.id = "plants";
    dl.innerHTML = (await r.json()).names
      .map(n => `<option value="${n.replace(/"/g, "&quot;")}">`).join("");
    document.body.appendChild(dl);
  } catch (e) { /* the box still takes free text */ }
})();

load();
</script>
</body>
</html>
"""
