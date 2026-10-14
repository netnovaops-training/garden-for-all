// Test-only: the plant guide's photo strip for visitors who are not signed in (site/js/app.js).
// The photo's file name must never be written inside a piece of script text: a name such as
//     x');alert(1);//.jpg      or      a</script><script>alert(1)</script>.jpg
// must stay plain text in an attribute, and the click must hand the exact name to the
// big-picture viewer. The helper functions are cut out of the real file and run with a
// tiny made-up page (no browser, no network). Nothing real is touched.
//
//   node tools/release/tests/test-guest-thumbs.js
//
// Before the fix the name sits inside onclick="...'NAME'..." (FAIL); after it, it sits in
// data-guest-file and a click listener reads it (PASS).
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const FILE = process.env.APP_JS ||
    path.join(__dirname, '..', '..', '..', 'site', 'js', 'app.js');
const SRC = fs.readFileSync(FILE, 'utf8');
let passed = 0, failed = 0;
function ok(name, cond, extra) {
    if (cond) { passed++; console.log('PASS ', name); }
    else { failed++; console.log('FAIL ', name, extra || ''); }
}

// 1. the guest thumbnail line in the guide must not carry an inline click handler
const viewLines = SRC.split('\n').filter(l => l.includes('title="View photo"'));
ok('the guest photo markup has no inline onclick', viewLines.length > 0 && viewLines.every(l => !/onclick/i.test(l)),
    viewLines.map(l => l.trim().slice(0, 100)).join(' | '));

// 2. cut the real helpers out of the file
function grab(re) { const m = SRC.match(re); return m ? m[0] : ''; }
const esc = grab(/function svEscHtml\(s\) \{[\s\S]*?\n\}\n/);
const block = grab(/\/\/ <sv-guest-thumbs>[\s\S]*?\/\/ <\/sv-guest-thumbs>/);
ok('the guest photo helpers exist in app.js', esc && block);
if (!block) { console.log(`\n${passed} passed, ${failed} failed`); process.exit(1); }

const decode = s => s.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'").replace(/&amp;/g, '&');   // what a browser does to an attribute value

function load(document, window) {
    return new Function('document', 'window',
        esc + block + '\nreturn { svEscHtml, svGuestThumbHtml, svOpenGuestThumb, svBindGuestThumbs };')(document, window);
}

const NAMES = [
    "x');alert(1);//.jpg",
    'a</script><script>alert(1)</script>.jpg',
    'q"onmouseover="alert(1).jpg',
    "it's a & b <i>.jpg",
    'rose-1.jpg',
];
const BASE = '/images/Database/';

for (const name of NAMES) {
    // a made-up page for this one name
    const main = { src: '' };
    const lightbox = [];
    const document = { getElementById: id => (id === 'tg-main-img' ? main : null) };
    const window = { openLightbox: (src, id) => lightbox.push([src, id]) };
    const h = load(document, window);

    const html = h.svGuestThumbHtml(name, false, BASE);
    const attr = (html.match(/data-guest-file="([^"]*)"/) || [])[1];
    ok(`[${name}] name sits in an attribute, escaped`, attr !== undefined && !/["'<>]/.test(attr) && decode(attr) === name, html);
    ok(`[${name}] no script, no inline handler, one tag`, !/onclick|<script|javascript:/i.test(html) && (html.match(/</g) || []).length === 1, html);

    // the listener: a stand-in for the browser's click on the picture
    const img = { dataset: { guestFile: decode(attr || '') }, handlers: {}, addEventListener(t, f) { this.handlers[t] = f; } };
    const root = { querySelectorAll: sel => (sel.includes('data-guest-file') ? [img] : []) };
    h.svBindGuestThumbs(root, BASE, 42);
    ok(`[${name}] a click listener was added`, typeof img.handlers.click === 'function');
    if (img.handlers.click) img.handlers.click();
    ok(`[${name}] click: big picture and viewer get the exact name`,
        main.src === BASE + name && lightbox.length === 1 && lightbox[0][0] === BASE + name && lightbox[0][1] === 42,
        JSON.stringify({ main: main.src, lightbox }));
}

// 3. the general escape also covers the single quote now
const h0 = load({ getElementById: () => null }, {});
ok("svEscHtml escapes ' as well as & < > \"", h0.svEscHtml(`'&<>"`) === '&#39;&amp;&lt;&gt;&quot;', h0.svEscHtml(`'&<>"`));

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
