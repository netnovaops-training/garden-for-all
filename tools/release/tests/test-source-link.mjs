// Test-only: site/js/source-link.js, the "Source" link the plant cards and the plant
// window draw from product_url. Made-up addresses only; nothing real is touched.
//
//   node tools/release/tests/test-source-link.mjs
//
// Good https addresses become one link with the safe attributes and the text "Source"
// (never the address as text); anything else draws nothing.
import { safeSourceUrl, sourceLinkHTML } from '../../../site/js/source-link.js';

let passed = 0, failed = 0;
function ok(name, cond, extra) {
    if (cond) { passed++; console.log('PASS ', name); }
    else { failed++; console.log('FAIL ', name, extra || ''); }
}

const good = 'https://www.example.com/plants/x?a=1&b=2';
const h = sourceLinkHTML(good, 'svd-source');
ok('good address: href escaped', h.includes('href="https://www.example.com/plants/x?a=1&amp;b=2"'), h);
ok('good address: target _blank', h.includes('target="_blank"'), h);
ok('good address: rel', h.includes('rel="noopener noreferrer nofollow"'), h);
ok('good address: text is Source', h.endsWith('>Source</a>'), h);
ok('good address: url not shown as text', h.split('example.com').length === 2, h);
ok('trimmed', safeSourceUrl('  https://x.y/p  ') === 'https://x.y/p');
ok('port ok', safeSourceUrl('https://example.com:8443/x') === 'https://example.com:8443/x');

const bad = [
    'http://example.com/x', 'javascript:alert(1)', 'data:text/html,<b>x</b>', '/plants/x',
    'example.com/x', 'https://', 'https:///x', 'https://exa mple.com/', 'https://example.com/a b',
    'https://example.com/\tx', 'https://user:pw@example.com/', 'https://localhost/',
    'https://example.com/"onmouseover="x', 'https://example.com/<script>', '//example.com/x',
    '', '   ', null, undefined, 42, 'https://example.com/' + 'a'.repeat(481),
];
for (const b of bad) ok('refused: ' + String(JSON.stringify(b)).slice(0, 40), sourceLinkHTML(b) === '');
ok('500 characters is fine', safeSourceUrl('https://example.com/' + 'a'.repeat(480)) !== '');

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
