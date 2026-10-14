# WHAT THIS FILE DOES (plain English)
# The /diagnose page (HTML and JavaScript in one Python string). One photo:
# the AI says what plant it is and whether anything looks wrong.
"""Point the camera at a plant: what is it, and is anything wrong with it.

THE AUTHOR, 2026-09-24: "I want him to be able to identify a plant as well as
identify if it's unwell or dying or has rust or something like that."

ONE PHOTOGRAPH, BOTH ANSWERS, because standing in front of the plant they
are one question. Asking somebody to identify it first and then ask about
its health is asking them to do the join -- and it makes the health reading
worse, because naming the plant first is what lets the check-up run against
that plant's own record and say "silver foliage, normal for this cultivar"
instead of reporting it as a symptom.

A WEB PAGE, NOT A NATIVE SCREEN, and that is deliberate. Jun's app is a
window onto this site with the assistant one tap away; the chat app reaches
the same service through its own tools. A page is written once and both
apps have it, the phone and the desktop get the same thing, and there is no
release to ship when the wording changes. The app adds the button, not the
feature.

THE THREE SECTIONS ARE STYLED DIFFERENTLY ON PURPOSE -- the same rule as the
check-up page. What is VISIBLE is fact and is set as plain text. What it is
CONSISTENT WITH is a possibility and is marked as one. What WOULD CONFIRM IT
is the only part that asks for action, so it is the only part that looks
like an instruction. Flattening them is how "consistent with iron
deficiency" gets read as "has iron deficiency", and then somebody feeds a
plant that is drowning.

WHAT IT SAYS WHEN IT DOES NOT KNOW. A plant the catalogue has never heard of
is reported as exactly that, with the reading still given. The alternative
-- quietly attaching it to the nearest catalogue entry -- would file a
reading against the wrong plant and corrupt that plant's history.
"""

DIAGNOSE_HTML = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Identify a plant</title>
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
.who{font-size:1.5rem;font-weight:700;line-height:1.2;margin:0}
.bot-name{font-style:italic;color:var(--ink-2);margin:2px 0 0}
.where{margin:10px 0 0;font-size:.9rem}
.where.known{color:var(--leaf)}
.where.unknown{color:var(--care)}

/* VISIBLE: fact. Plain text, nothing added. */
.sec-visible{margin:0 0 4px}
/* CONSISTENT WITH: a possibility, and it must not read as a finding. */
.sec-maybe{border-left:3px solid var(--care);padding-left:12px;
  color:var(--ink-2);font-style:italic}
/* WOULD CONFIRM IT: the only part that asks for an action. */
.sec-do{background:rgba(62,107,67,.08);border:1px solid var(--line);
  border-radius:8px;padding:12px}
.lab{font-size:.72rem;letter-spacing:.14em;text-transform:uppercase;
  color:var(--ink-2);margin:18px 0 6px;font-weight:600}
.bot{color:var(--ink-2);font-size:.82rem;font-style:italic;margin-top:20px}
a{color:var(--leaf)}
</style>
</head>
<body>
<div class="wrap">
  <h1>Identify a plant</h1>
  <p class="sub">One photograph: what it is, and whether anything looks
  wrong with it.</p>

  <div class="card">
    <input type="file" id="pick" accept="image/*" capture="environment" hidden>
    <button class="big" id="take">Photograph a plant</button>
    <button class="quiet" id="pickfile">Choose a photo instead</button>
    <img id="shot" hidden alt="">
    <div id="msg" hidden class="msg"></div>
  </div>

  <div class="card" id="result" hidden>
    <h2>What it is</h2>
    <p class="who" id="name">—</p>
    <p class="bot-name" id="botname"></p>
    <p class="where" id="where"></p>

    <p class="lab">Visible in the photograph</p>
    <p class="sec-visible" id="visible"></p>

    <p class="lab">Consistent with — possibilities, not findings</p>
    <p class="sec-maybe" id="maybe"></p>

    <p class="lab">What would confirm it</p>
    <div class="sec-do" id="confirm"></div>

    <p class="bot">A photograph cannot tell a nitrogen shortage from
    waterlogging — both yellow a leaf. The middle section lists what a
    symptom is <em>associated with</em>, not what the plant has.</p>
  </div>

  <p class="bot"><a href="spend">Receipts</a> ·
  <a href="plan">The beds</a> · <a href="checkup">Check-up history</a></p>
</div>

<script>
const BASE = location.pathname.replace(/\/diagnose\/?$/, "");
const $ = s => document.querySelector(s);

function say(t, bad) {
  const m = $("#msg");
  m.textContent = t;
  m.className = "msg" + (bad ? " bad" : "");
  m.hidden = false;
}

// SHRUNK BEFORE IT IS SENT. A phone photograph is several megabytes and the
// model gains nothing past about 1,400px on the long edge; the rest is
// upload time over whatever signal exists at the bottom of a garden.
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
  $("#result").hidden = true;
  say("Looking at that photograph…");

  let data;
  try {
    data = await shrink(file, 1400);
    const s = $("#shot"); s.src = data; s.hidden = false;
  } catch (err) {
    $("#take").disabled = false;
    say(err.message, true);
    return;
  }

  try {
    const r = await fetch(BASE + "/api/diagnose", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ image: data })
    });
    const out = await r.json();
    if (!r.ok) {
      say(out.error || "that did not work", true);
    } else if (!out.identified) {
      say(out.note || "I could not make a plant out of that photograph.", true);
    } else {
      show(out);
      $("#msg").hidden = true;
    }
  } catch (err) {
    say("could not reach the garden: " + err.message, true);
  }
  $("#take").disabled = false;
  $("#pick").value = "";
};

function show(o) {
  $("#name").textContent = o.name || "Not sure";
  $("#botname").textContent = o.botanical_name || "";
  const w = $("#where");
  if (o.in_catalogue) {
    w.textContent = "In Jun's catalogue. This reading has been saved to its "
                  + "history.";
    w.className = "where known";
  } else {
    // SAID PLAINLY. Somebody who does not know the reading was made without
    // the plant's record will trust it more than they should.
    w.textContent = "Not in Jun's catalogue yet, so nothing has been saved "
                  + "and this reading was made from the name alone.";
    w.className = "where unknown";
  }
  $("#visible").textContent = o.looked_at || "—";
  $("#maybe").textContent = o.consistent || "Nothing visible.";
  $("#confirm").textContent = o.confirm || "—";
  $("#result").hidden = false;
  $("#result").scrollIntoView({ behavior: "smooth", block: "start" });
}
</script>
</body>
</html>
"""
