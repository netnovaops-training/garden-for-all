// Test-only: the page builder editor's editing rules (site/pagebuilder/logic.js), run in Node
// with no page and no network.
//
//   node tools/release/tests/test-pagebuilder-logic.js
'use strict';
const path = require('path');
const assert = require('assert');
const L = require(path.join(__dirname, '..', '..', '..', 'site', 'pagebuilder', 'logic.js'));

let passed = 0, failed = 0;
function test(name, fn) {
    try { fn(); passed++; console.log('PASS ', name); }
    catch (e) { failed++; console.log('FAIL ', name, e.message); }
}
const row = (...ids) => ({ cols: ids.map(s => ({ section: s, settings: {} })) });
const ids = rows => rows.map(r => r.cols.map(c => c.section).join('+')).join(' | ');

test('adding appends or inserts a row and refuses a section that is on the page', () => {
    const rows = [row('a')];
    assert.strictEqual(ids(L.addRow(rows, 'b', 1)), 'a | b');
    assert.strictEqual(ids(L.addRow(rows, 'b', 1, 0)), 'b | a');
    assert.strictEqual(L.addRow(rows, 'a', 1).error, 'full');
    assert.strictEqual(ids(L.addRow(rows, 'a', 3)), 'a | a');
    assert.strictEqual(ids(rows), 'a', 'the old list is not changed');
});

test('at most 40 rows', () => {
    const rows = Array.from({ length: 40 }, (_, i) => row('s' + i));
    assert.strictEqual(L.addRow(rows, 'x', 1).error, 'tooMany');
});

test('up and down swap whole rows; a shared row steps out', () => {
    const rows = [row('a'), row('b'), row('c', 'd')];
    assert.strictEqual(ids(L.moveV(rows, 1, 0, -1)), 'b | a | c+d');
    assert.strictEqual(ids(L.moveV(rows, 1, 0, 1)), 'a | c+d | b');
    assert.strictEqual(L.moveV(rows, 0, 0, -1).error, 'none');
    assert.strictEqual(ids(L.moveV(rows, 2, 1, -1)), 'a | b | d | c');
    assert.strictEqual(ids(L.moveV(rows, 2, 0, 1)), 'a | b | c+d'.replace('c+d', 'd | c'));
});

test('left and right swap neighbours in a row', () => {
    const rows = [row('a', 'b', 'c')];
    assert.strictEqual(ids(L.moveH(rows, 0, 0, 1)), 'b+a+c');
    assert.strictEqual(L.moveH(rows, 0, 0, -1).error, 'none');
    assert.strictEqual(L.moveH(rows, 0, 2, 1).error, 'none');
});

test('removing the last section of a row removes the row', () => {
    assert.strictEqual(ids(L.remove([row('a'), row('b', 'c')], 0, 0)), 'b+c');
    assert.strictEqual(ids(L.remove([row('a'), row('b', 'c')], 1, 0)), 'a | c');
});

test('dropping a section on a row joins it, up to 4 columns', () => {
    const rows = [row('a'), row('b', 'c', 'd', 'e')];
    assert.strictEqual(ids(L.dropInRow(rows, 0, 'x', 1)), 'a+x | b+c+d+e');
    assert.strictEqual(L.dropInRow(rows, 1, 'x', 1).error, 'rowFull');
    assert.strictEqual(L.dropInRow(rows, 0, 'a', 1).error, 'full');
});

test('dropping a cell moves it; its old row closes if it was alone', () => {
    const rows = [row('a'), row('b'), row('c')];
    assert.strictEqual(ids(L.dropInRow(rows, 2, null, 1, [0, 0])), 'b | c+a');
    assert.strictEqual(L.dropInRow(rows, 1, null, 1, [1, 0]).error, 'none', 'same row');
});

test('dropping on a gap puts the section in a row of its own there', () => {
    const rows = [row('a'), row('b', 'c'), row('d')];
    assert.strictEqual(ids(L.dropAtGap(rows, 0, 'x', 1)), 'x | a | b+c | d');
    assert.strictEqual(ids(L.dropAtGap(rows, 3, 'x', 1)), 'a | b+c | d | x');
    assert.strictEqual(L.dropAtGap(rows, 1, 'a', 1).error, 'full');
    // A cell from a shared row steps out into the gap.
    assert.strictEqual(ids(L.dropAtGap(rows, 0, null, 1, [1, 1])), 'c | a | b | d');
    assert.strictEqual(ids(L.dropAtGap(rows, 3, null, 1, [1, 0])), 'a | c | d | b');
    // A lone row moves, and the gap number follows the row that left.
    assert.strictEqual(ids(L.dropAtGap(rows, 3, null, 1, [0, 0])), 'b+c | d | a');
    assert.strictEqual(ids(L.dropAtGap(rows, 0, null, 1, [2, 0])), 'd | a | b+c');
    // Dropping a lone row where it already is changes nothing.
    assert.strictEqual(L.dropAtGap(rows, 0, null, 1, [0, 0]).error, 'none');
    assert.strictEqual(L.dropAtGap(rows, 1, null, 1, [0, 0]).error, 'none');
    const full = Array.from({ length: 40 }, (_, i) => row('s' + i));
    full[5] = row('p', 'q');
    assert.strictEqual(L.dropAtGap(full, 0, null, 1, [5, 0]).error, 'tooMany');
});

test('undo and redo walk back and forth through unpublished edits', () => {
    const h = L.History();
    let state = [row('a')];
    assert.ok(!h.canUndo() && !h.canRedo());
    h.push(state); state = L.addRow(state, 'b', 1);
    h.push(state); state = L.addRow(state, 'c', 1);
    assert.strictEqual(ids(state), 'a | b | c');
    state = h.undo(state); assert.strictEqual(ids(state), 'a | b');
    state = h.undo(state); assert.strictEqual(ids(state), 'a');
    assert.strictEqual(h.undo(state), null);
    state = h.redo(state); assert.strictEqual(ids(state), 'a | b');
    // A new edit drops the redo list.
    h.push(state); state = L.addRow(state, 'z', 1);
    assert.ok(!h.canRedo());
});

test('the undo list is limited', () => {
    const h = L.History();
    for (let i = 0; i < 300; i++) h.push({ i });
    let n = 0, cur = { i: 300 };
    while ((cur = h.undo(cur)) !== null) n++;
    assert.strictEqual(n, 100);
});

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
