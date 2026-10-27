// The page builder's editing rules, with no page in sight (so a test can run them in Node).
// Rows are [{cols: [{section, settings}]}]. Every function returns a NEW rows list, or
// {error: 'rowFull' | 'tooMany' | 'full' | 'none'} when the change is not allowed. The editor
// (editor.js) draws what comes back. Test: tools/release/tests/test-pagebuilder-logic.js.
(function (root, factory) {
    if (typeof module === 'object' && module.exports) module.exports = factory();
    else root.PBLogic = factory();
}(this, function () {
    'use strict';
    var MAX_COLS = 4, MAX_ROWS = 40, UNDO_LIMIT = 100;

    function clone(x) { return JSON.parse(JSON.stringify(x)); }
    function fail(code) { return { error: code }; }
    function isError(r) { return !!(r && r.error); }

    function count(rows, id) {
        var n = 0;
        rows.forEach(function (r) { r.cols.forEach(function (c) { if (c.section === id) n++; }); });
        return n;
    }
    function newCell(id) { return { section: id, settings: {} }; }

    // `max` is the section's max_per_page (a pack section: 3).
    function addRow(rows, id, max, at) {
        if (count(rows, id) >= max) return fail('full');
        if (rows.length >= MAX_ROWS) return fail('tooMany');
        var out = clone(rows), row = { cols: [newCell(id)] };
        if (at === undefined) out.push(row); else out.splice(at, 0, row);
        return out;
    }

    function takeCell(out, ri, ci) {
        var cell = out[ri].cols.splice(ci, 1)[0];
        if (!out[ri].cols.length) out.splice(ri, 1);
        return cell;
    }

    function remove(rows, ri, ci) {
        var out = clone(rows);
        takeCell(out, ri, ci);
        return out;
    }

    // Left/right: swap with the neighbour in the row.
    function moveH(rows, ri, ci, step) {
        var to = ci + step;
        if (to < 0 || to >= rows[ri].cols.length) return fail('none');
        var out = clone(rows), cols = out[ri].cols, x = cols[ci];
        cols[ci] = cols[to]; cols[to] = x;
        return out;
    }

    // Up/down: a section alone in its row swaps rows with its neighbour; a section sharing a
    // row steps out into a new row of its own above or below.
    function moveV(rows, ri, ci, step) {
        var out = clone(rows), row = out[ri];
        if (row.cols.length === 1) {
            var to = ri + step;
            if (to < 0 || to >= out.length) return fail('none');
            out[ri] = out[to]; out[to] = row;
            return out;
        }
        if (out.length >= MAX_ROWS) return fail('tooMany');
        var cell = row.cols.splice(ci, 1)[0];
        out.splice(step < 0 ? ri : ri + 1, 0, { cols: [cell] });
        return out;
    }

    // A drop on a row: a section from the list joins it (`id`, `max`); a cell from the page
    // (`from` = [ri, ci]) moves into it.
    function dropInRow(rows, ri, id, max, from) {
        var out = clone(rows);
        if (from) {
            if (from[0] === ri) return fail('none');
            if (out[ri].cols.length >= MAX_COLS) return fail('rowFull');
            var target = out[ri];
            target.cols.push(takeCell(out, from[0], from[1]));
            return out;
        }
        if (count(rows, id) >= max) return fail('full');
        if (out[ri].cols.length >= MAX_COLS) return fail('rowFull');
        out[ri].cols.push(newCell(id));
        return out;
    }

    // A drop on a gap between rows (`at` = the row number it goes before; rows.length = the
    // end, which is also the big +): a section from the list or a cell from the page becomes
    // a row of its own there.
    function dropAtGap(rows, at, id, max, from) {
        var out = clone(rows);
        if (from) {
            if (from[0] === at && out[at].cols.length === 1) return fail('none');
            if (from[0] + 1 === at && out[from[0]].cols.length === 1) return fail('none');
            var alone = out[from[0]].cols.length === 1;
            var cell = takeCell(out, from[0], from[1]);
            if (!alone && out.length >= MAX_ROWS) return fail('tooMany');
            if (alone && from[0] < at) at--;
            out.splice(at, 0, { cols: [cell] });
            return out;
        }
        if (count(rows, id) >= max) return fail('full');
        if (out.length >= MAX_ROWS) return fail('tooMany');
        out.splice(at, 0, { cols: [newCell(id)] });
        return out;
    }

    // Undo and redo for edits that are not published yet. `snapshot` is anything JSON.
    function History() {
        var past = [], future = [];
        return {
            // Call BEFORE a change, with the state as it is now.
            push: function (snapshot) {
                past.push(JSON.stringify(snapshot));
                if (past.length > UNDO_LIMIT) past.shift();
                future = [];
            },
            // `current` goes to the redo list; returns the state before, or null.
            undo: function (current) {
                if (!past.length) return null;
                future.push(JSON.stringify(current));
                return JSON.parse(past.pop());
            },
            redo: function (current) {
                if (!future.length) return null;
                past.push(JSON.stringify(current));
                return JSON.parse(future.pop());
            },
            canUndo: function () { return past.length > 0; },
            canRedo: function () { return future.length > 0; },
            clear: function () { past = []; future = []; }
        };
    }

    return {
        MAX_COLS: MAX_COLS, MAX_ROWS: MAX_ROWS, clone: clone, isError: isError, count: count,
        addRow: addRow, remove: remove, moveH: moveH, moveV: moveV, dropInRow: dropInRow,
        dropAtGap: dropAtGap, History: History
    };
}));
