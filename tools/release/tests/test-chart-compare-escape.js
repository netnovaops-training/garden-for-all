// Test-only: site/js/chart-compare.js must not put raw text into the HTML blocks it builds
// (the five weather chart screens and the insights list). The real file is run with a tiny
// made-up page (no browser, no network); the server answers are made up and carry markup
// in every label and value. Nothing real is touched.
//
//   node tools/release/tests/test-chart-compare-escape.js
//
// Before the fix the markup reaches the page as markup (FAIL); after it, it shows as text (PASS).
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const FILE = process.env.CHART_COMPARE_JS ||
    path.join(__dirname, '..', '..', '..', 'site', 'js', 'chart-compare.js');
const SRC = fs.readFileSync(FILE, 'utf8');

const EVIL = '<img src=x onerror=alert(1)>"\'&';
const ESCAPED = '&lt;img src=x onerror=alert(1)&gt;&quot;&#39;&amp;';
let passed = 0, failed = 0;
function ok(name, cond, extra) {
    if (cond) { passed++; console.log('PASS ', name); }
    else { failed++; console.log('FAIL ', name, extra || ''); }
}

// --- the made-up page -------------------------------------------------------------
const IDS = ['compare-tool', 'compare-chart-a', 'compare-chart-b', 'compare-output', 'compare-hint',
    'compare-insights', 'compare-weather-metric', 'compare-weather-year', 'compare-weather-overlay',
    'year-weather-overlay', 'standalone-weather-chart', 'split-plant-chart', 'split-weather-chart',
    'fused-chart', 'fused-content'];
function makeEl(id) {
    return {
        id, innerHTML: '', value: '', checked: false, textContent: '', style: {}, dataset: {}, h: {},
        addEventListener(t, f) { (this.h[t] = this.h[t] || []).push(f); },
        querySelector() { return null; }, querySelectorAll() { return []; },
        getContext() { return {}; },
    };
}
let els, responses;
function freshPage() {
    els = {}; IDS.forEach(id => { els[id] = makeEl(id); });
    responses = {};
    const document = {
        readyState: 'complete',
        getElementById: id => els[id] || null,
        querySelector: () => null, querySelectorAll: () => [],
        addEventListener() {},
    };
    class Chart { constructor() {} destroy() {} }
    const fetch = async url => ({ json: async () => (url.includes('weather_data') ? responses.weather : responses.cross) });
    const ctx = { document, window: {}, fetch, Chart, console, Date, Math, Promise, JSON, Number, String, Array, Object, parseInt, parseFloat, isNaN };
    vm.runInNewContext(SRC, ctx, { filename: 'chart-compare.js' });
}
const fire = async (id, ev) => { for (const f of (els[id].h[ev] || [])) await f(); };

function crossTab(dimA) {
    return {
        dim_a_label: EVIL, dim_b_label: EVIL,
        rowLabels: dimA === 'year' ? ['2022', '2023'] : ['A', EVIL],
        colLabels: ['C', EVIL],
        matrix: [[3, 1], [2, 5]], rowTotals: [4, 7], colTotals: [5, 6],
    };
}
function weather() {
    return {
        labels: [EVIL, 'Feb 2024'], data: [20, 90],
        stats: { mean: EVIL, min: EVIL, max: EVIL },
        coverage: { noaa_fill_months: 3, om_months: EVIL, total_months: EVIL },
        quarterly: { year: [2022, 2023], q: [1, 3], data: [10, 80] },
        yearly: { labels: ['2022', '2023'], data: [50, 60] },
    };
}
function noRawMarkup(html) { return !html.includes('<img src=x') && !html.includes('onerror=alert(1)>'); }

(async () => {
    // 1. the error message from the server (also the weather one)
    freshPage();
    els['compare-weather-metric'].value = 'temperature';
    responses.weather = { error: EVIL };
    await fire('compare-weather-metric', 'change');
    let html = els['compare-weather-overlay'].innerHTML;
    ok('weather error text is shown as text', noRawMarkup(html) && html.includes(ESCAPED), html.slice(0, 120));

    // 2. network/parse error (e.message)
    freshPage();
    els['compare-weather-metric'].value = 'temperature';
    responses.weather = null;   // res.json() gives null -> reading .error throws a TypeError; not markup
    await fire('compare-weather-metric', 'change');
    html = els['compare-weather-overlay'].innerHTML;
    ok('a failed weather request still shows a plain message', /<p [^>]*>[^<]*<\/p>/.test(html) && noRawMarkup(html), html);

    // 3. a chosen year (#5): year drop-down value and the metric name carry markup
    freshPage();
    els['compare-weather-metric'].value = EVIL;          // unknown metric -> its name is used as the label
    els['compare-weather-year'].value = EVIL;
    responses.weather = weather();
    await fire('compare-weather-metric', 'change');
    html = els['compare-weather-overlay'].innerHTML;
    ok('year view (#5): year, label and numbers shown as text', html.includes('Weather:') && noRawMarkup(html) && html.includes(ESCAPED), html.slice(0, 200));

    // 4. standalone monthly wave (#6)
    freshPage();
    els['compare-weather-metric'].value = EVIL;
    responses.weather = weather();
    await fire('compare-weather-metric', 'change');
    html = els['compare-weather-overlay'].innerHTML;
    ok('standalone view (#6): label, span, numbers shown as text', html.includes('standalone-weather-chart') && noRawMarkup(html) && html.includes(ESCAPED), html.slice(0, 200));

    // 5. side by side with plant data (#7)
    freshPage();
    els['compare-chart-a'].value = 'bed'; els['compare-chart-b'].value = 'sun';
    responses.cross = crossTab('bed');
    await fire('compare-chart-a', 'change');
    els['compare-weather-metric'].value = EVIL;
    responses.weather = weather();
    await fire('compare-weather-metric', 'change');
    html = els['compare-weather-overlay'].innerHTML;
    ok('side-by-side view (#7): labels shown as text', html.includes('split-plant-chart') && noRawMarkup(html) && html.includes(ESCAPED), html.slice(0, 200));

    // 6. fused view (#8) and its chart notes (#9): dimension A = year
    freshPage();
    els['compare-chart-a'].value = 'year'; els['compare-chart-b'].value = 'sun';
    responses.cross = crossTab('year');
    await fire('compare-chart-a', 'change');
    els['compare-weather-metric'].value = EVIL;
    responses.weather = weather();
    await fire('compare-weather-metric', 'change');
    html = els['compare-weather-overlay'].innerHTML;
    ok('fused view (#8): title shown as text', html.includes('Fused View') && noRawMarkup(html) && html.includes(ESCAPED), html.slice(0, 200));
    html = els['fused-content'].innerHTML;
    ok('fused chart notes (#9): labels, trend and coverage shown as text', html.includes('fused-chart') && noRawMarkup(html) && html.includes(ESCAPED), html.slice(0, 200));

    // 7. the insights list under the heatmap
    freshPage();
    els['compare-chart-a'].value = 'bed'; els['compare-chart-b'].value = 'sun';
    responses.cross = crossTab('bed');
    await fire('compare-chart-a', 'change');
    html = els['compare-insights'].innerHTML;
    ok('insights: row and column names shown as text', html.includes('Largest') && noRawMarkup(html) && html.includes(ESCAPED), html.slice(0, 200));

    // 8. a normal owner sees the same words (nothing needed escaping)
    freshPage();
    els['compare-weather-metric'].value = 'precipitation';
    els['compare-weather-year'].value = '2024';
    responses.weather = { labels: ['Jan 2024', 'Feb 2024'], data: [0.5, 3], stats: { mean: 1.8, min: 0.5, max: 3 }, coverage: {} };
    await fire('compare-weather-metric', 'change');
    html = els['compare-weather-overlay'].innerHTML;
    // what the browser would show: entity codes turned back into characters
    const shown = html.replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
    ok('normal values read the same: "2024 Weather: Total Precipitation (in)", avg 1.8"',
        shown.includes('2024 Weather: Total Precipitation (in)') && shown.includes('avg <strong>1.8"</strong>') && shown.includes('Dry months'), shown.slice(0, 300));

    console.log(`\n${passed} passed, ${failed} failed`);
    process.exit(failed ? 1 : 0);
})().catch(e => { console.log('FAIL  test crashed:', e && e.stack || e); process.exit(2); });
