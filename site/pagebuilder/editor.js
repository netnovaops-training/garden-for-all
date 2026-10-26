// The page builder's editor (pagebuilder.php). Owner only; the server checks every request.
//
// HOW TO READ THIS FILE. `S` holds the editor's state: the sections (from the registry), the
// page's layout (rows of 1-4 cells), and the saved version it started from. Every change goes
// through `changed()`, which redraws the canvas and asks the server for a fresh preview.
// Text from the server is only ever put on the page with textContent (never as HTML), except
// the preview, which is the server's own page drawn in a sandboxed frame.
// Design: docs/page-builder/DESIGN.md. Guide: docs/page-builder/GUIDE.md.
(function () {
    'use strict';

    var app = document.getElementById('pb-app');
    if (!app) return;
    var TOKEN = app.getAttribute('data-token') || '';
    var MAX_COLS = 4, MAX_ROWS = 40;

    // ---------------------------------------------------------------- words (EN / KO)
    var WORDS = {
        en: {
            title: 'Page builder', page: 'Page', newPage: 'New page', undo: 'Undo',
            reset: 'Original page', save: 'Save', view: 'View page', sections: 'Sections',
            panelHint: 'Drag a section onto the page, or press Add.', canvas: 'Page',
            plusHint: 'Drop a section here for a new row.', history: 'Saved versions',
            preview: 'Live preview (as visitors see it)', later: 'Coming with the database update',
            add: 'Add', addRow: 'New row with the chosen section', isNew: 'new', admin: 'signed-in only',
            full: 'already on this page', up: 'Move up', down: 'Move down', left: 'Move left',
            right: 'Move right', remove: 'Remove', empty: 'This page is empty. Press + or drag a section here.',
            saved: 'Saved (version {v}).', unsaved: 'Not saved yet.', loading: 'Loading…',
            pickFirst: 'Choose a section on the left first (click its name).',
            confirmLeave: 'You have changes that are not saved. Leave them?',
            confirmReset: 'Put this page back to how it was originally? (It can be undone.)',
            confirmUndo: 'Go back to the version before this one?',
            discard: 'Unsaved changes were dropped.',
            newName: 'Name of the new page (a-z, 0-9 and -):', badName: 'Use only a-z, 0-9 and - (up to 40).',
            restore: 'Bring back', default: 'original', seenOk: 'OK',
            newHere: 'New sections arrived with the last update:', nextUpdate: 'comes with the next garden update',
            updatesOff: 'Checking for new sections is not set up yet.', updatesFail: 'Could not check for new sections right now.',
            rowFull: 'A row holds at most 4 sections.', tooMany: 'At most 40 rows.', settings: 'Settings',
            version: 'version', note_save: 'saved', note_undo: 'undo', note_reset: 'original page',
            note_restore: 'brought back'
        },
        ko: {
            title: '페이지 편집기', page: '페이지', newPage: '새 페이지', undo: '되돌리기',
            reset: '처음 페이지로', save: '저장', view: '페이지 보기', sections: '섹션',
            panelHint: '섹션을 페이지로 끌어다 놓거나 추가를 누르세요.', canvas: '페이지',
            plusHint: '여기에 섹션을 놓으면 새 줄이 생겨요.', history: '저장된 버전',
            preview: '미리보기 (방문자가 보는 모습)', later: '데이터베이스 업데이트와 함께 올 섹션',
            add: '추가', addRow: '고른 섹션으로 새 줄 만들기', isNew: '새로움', admin: '로그인한 사람만',
            full: '이미 이 페이지에 있어요', up: '위로', down: '아래로', left: '왼쪽으로',
            right: '오른쪽으로', remove: '빼기', empty: '빈 페이지예요. +를 누르거나 섹션을 끌어다 놓으세요.',
            saved: '저장했어요 (버전 {v}).', unsaved: '아직 저장하지 않았어요.', loading: '불러오는 중…',
            pickFirst: '먼저 왼쪽에서 섹션을 고르세요 (이름을 누르세요).',
            confirmLeave: '저장하지 않은 바뀐 내용이 있어요. 버릴까요?',
            confirmReset: '이 페이지를 처음 모습으로 되돌릴까요? (다시 되돌릴 수 있어요.)',
            confirmUndo: '바로 앞 버전으로 되돌릴까요?',
            discard: '저장하지 않은 내용을 버렸어요.',
            newName: '새 페이지 이름 (a-z, 0-9, - 만):', badName: 'a-z, 0-9, - 만 쓸 수 있어요 (40자까지).',
            restore: '되살리기', default: '처음 모습', seenOk: '확인',
            newHere: '지난 업데이트로 새 섹션이 왔어요:', nextUpdate: '다음 정원 업데이트와 함께 와요',
            updatesOff: '새 섹션 확인이 아직 설정되지 않았어요.', updatesFail: '지금은 새 섹션을 확인할 수 없어요.',
            rowFull: '한 줄에는 섹션이 4개까지 들어가요.', tooMany: '줄은 40개까지예요.', settings: '설정',
            version: '버전', note_save: '저장', note_undo: '되돌리기', note_reset: '처음 모습',
            note_restore: '되살리기'
        }
    };
    var lang = 'en';
    try { lang = localStorage.getItem('pb_lang') || (navigator.language || '').slice(0, 2); } catch (e) { /* private window */ }
    if (lang !== 'ko') lang = 'en';
    function t(k, v) { var s = (WORDS[lang][k] || WORDS.en[k] || k); return v === undefined ? s : s.replace('{v}', v); }
    function nm(o, base) { return o[base + '_' + lang] || o[base + '_en'] || ''; }

    // ---------------------------------------------------------------- state
    var S = { page: app.getAttribute('data-page') || 'home', reg: [], pending: [], announced: [],
              rows: [], version: 0, dirty: false, selected: null, pages: [], history: [] };
    function def(id) { for (var i = 0; i < S.reg.length; i++) if (S.reg[i].id === id) return S.reg[i]; return null; }
    function count(id) { var n = 0; S.rows.forEach(function (r) { r.cols.forEach(function (c) { if (c.section === id) n++; }); }); return n; }
    function canAdd(id) { var d = def(id); return d && count(id) < d.max_per_page; }

    // ---------------------------------------------------------------- small helpers
    function el(tag, cls, text) { var e = document.createElement(tag); if (cls) e.className = cls; if (text !== undefined) e.textContent = text; return e; }
    function btn(text, title, fn, cls) {
        var b = el('button', cls || '', text); b.type = 'button';
        if (title) { b.title = title; b.setAttribute('aria-label', title); }
        b.addEventListener('click', fn); return b;
    }
    function status(msg, bad) { var s = document.getElementById('pb-status'); s.textContent = msg || ''; s.className = 'pb-status' + (bad ? ' pb-bad' : ''); }
    function errText(d) { return d && (lang === 'ko' ? (d.error_ko || d.error) : d.error) || 'Error · 오류'; }

    function api(path, body) {
        var opt = { credentials: 'same-origin', headers: { 'Accept': 'application/json' } };
        if (body !== undefined) {
            opt.method = 'POST';
            opt.headers['Content-Type'] = 'application/json';
            opt.headers['X-PB-Token'] = TOKEN;
            opt.body = JSON.stringify(body);
        }
        return fetch('pagebuilder/api/' + path, opt).then(function (r) {
            return r.json().catch(function () { return {}; }).then(function (d) { return { ok: r.ok, status: r.status, data: d }; });
        }).catch(function () { return { ok: false, status: 0, data: { error: 'No connection.', error_ko: '연결이 안 돼요.' } }; });
    }

    function layoutJson() {
        return { schema: 1, page: S.page, rows: S.rows.map(function (r) {
            return { cols: r.cols.map(function (c) { return { section: c.section, settings: c.settings || {} }; }) };
        }) };
    }

    // ---------------------------------------------------------------- words on the page
    function applyWords() {
        document.documentElement.lang = lang;
        document.querySelectorAll('[data-t]').forEach(function (e) { e.textContent = t(e.getAttribute('data-t')); });
        var lb = document.getElementById('pb-lang');
        lb.textContent = lang === 'ko' ? 'English' : '한국어';
        lb.lang = lang === 'ko' ? 'en' : 'ko';
        document.getElementById('pb-add').title = t('addRow');
        document.getElementById('pb-add').setAttribute('aria-label', t('addRow'));
    }

    // ---------------------------------------------------------------- the left panel
    function drawPanel() {
        var list = document.getElementById('pb-list');
        list.textContent = '';
        S.reg.forEach(function (d) {
            var li = el('li', 'pb-item' + (S.selected === d.id ? ' pb-selected' : ''));
            li.draggable = canAdd(d.id);
            li.addEventListener('dragstart', function (e) { e.dataTransfer.setData('text/x-pb-section', d.id); e.dataTransfer.effectAllowed = 'copy'; });
            var head = el('div', 'pb-item-head');
            var name = btn(nm(d, 'name'), nm(d, 'desc'), function () { S.selected = d.id; drawPanel(); }, 'pb-item-name');
            head.appendChild(name);
            if (d.new) head.appendChild(el('span', 'pb-badge pb-new', t('isNew')));
            if (d.perm === 'admin') head.appendChild(el('span', 'pb-badge pb-lock', '🔒 ' + t('admin')));
            li.appendChild(head);
            li.appendChild(el('p', 'pb-desc', nm(d, 'desc')));
            var add = btn(t('add'), t('addRow'), function () { addRow(d.id); }, 'pb-addbtn');
            if (!canAdd(d.id)) { add.disabled = true; add.title = t('full'); }
            li.appendChild(add);
            list.appendChild(li);
        });
        S.announced.forEach(function (a) {
            var li = el('li', 'pb-item pb-announced');
            var head = el('div', 'pb-item-head');
            head.appendChild(el('span', 'pb-item-name', nm(a, 'name')));
            head.appendChild(el('span', 'pb-badge pb-new', t('isNew')));
            li.appendChild(head);
            li.appendChild(el('p', 'pb-desc', nm(a, 'desc')));
            li.appendChild(el('p', 'pb-desc pb-soon', t('nextUpdate')));
            list.appendChild(li);
        });
        var pend = document.getElementById('pb-pending');
        pend.textContent = '';
        S.pending.forEach(function (id) { pend.appendChild(el('li', 'pb-desc', id)); });
        var us = { not_set_up: 'updatesOff', failed: 'updatesFail' }[S.updateState];
        document.getElementById('pb-update-state').textContent = us ? t(us) : '';
        drawNewNote();
    }

    function drawNewNote() {
        var box = document.getElementById('pb-updates');
        var fresh = S.reg.filter(function (d) { return d.new; });
        box.textContent = '';
        if (!fresh.length) return;
        box.appendChild(el('p', '', t('newHere') + ' ' + fresh.map(function (d) { return nm(d, 'name'); }).join(', ')));
        box.appendChild(btn(t('seenOk'), '', function () {
            api('seen', { ids: fresh.map(function (d) { return d.id; }) }).then(function (r) {
                if (r.ok) { fresh.forEach(function (d) { d.new = false; }); drawPanel(); }
            });
        }));
    }

    // ---------------------------------------------------------------- the canvas
    function drawCanvas() {
        var box = document.getElementById('pb-rows');
        box.textContent = '';
        if (!S.rows.length) box.appendChild(el('p', 'pb-hint pb-empty', t('empty')));
        S.rows.forEach(function (row, ri) {
            var r = el('div', 'pb-row');
            r.style.gridTemplateColumns = 'repeat(' + row.cols.length + ', minmax(0, 1fr))';
            dropTarget(r, function (id, from) { dropInRow(ri, id, from); });
            row.cols.forEach(function (cell, ci) { r.appendChild(drawCell(cell, ri, ci)); });
            box.appendChild(r);
        });
    }

    function drawCell(cell, ri, ci) {
        var d = def(cell.section) || { name_en: cell.section, settings: [] };
        var c = el('div', 'pb-cell' + (d.perm === 'admin' ? ' pb-cell-admin' : ''));
        c.draggable = true;
        c.addEventListener('dragstart', function (e) { e.dataTransfer.setData('text/x-pb-cell', ri + ',' + ci); e.dataTransfer.effectAllowed = 'move'; });
        var top = el('div', 'pb-cell-top');
        top.appendChild(el('strong', '', (d.perm === 'admin' ? '🔒 ' : '') + nm(d, 'name')));
        top.appendChild(btn('−', t('remove'), function () { removeCell(ri, ci); }, 'pb-minus'));
        c.appendChild(top);
        var moves = el('div', 'pb-moves');
        moves.appendChild(btn('↑', t('up'), function () { moveV(ri, ci, -1); }));
        moves.appendChild(btn('↓', t('down'), function () { moveV(ri, ci, 1); }));
        moves.appendChild(btn('←', t('left'), function () { moveH(ri, ci, -1); }));
        moves.appendChild(btn('→', t('right'), function () { moveH(ri, ci, 1); }));
        c.appendChild(moves);
        (d.settings || []).forEach(function (st) { c.appendChild(settingInput(cell, st)); });
        return c;
    }

    // One setting as a form field. Its limits come from the registry; the server checks again.
    function settingInput(cell, st) {
        var lab = el('label', 'pb-setting');
        lab.appendChild(el('span', '', nm(st, 'label')));
        var inp = el('input');
        cell.settings = cell.settings || {};
        var cur = cell.settings[st.key] !== undefined ? cell.settings[st.key] : st.default;
        if (st.type === 'int') {
            inp.type = 'number'; inp.min = st.min; inp.max = st.max; inp.step = 1;
            inp.value = cur;
            inp.addEventListener('change', function () {
                var n = parseInt(inp.value, 10);
                if (isNaN(n)) n = st.default;
                n = Math.max(st.min, Math.min(st.max, n));
                inp.value = n; cell.settings[st.key] = n; changed(false);
            });
        } else {
            inp.type = 'text'; inp.maxLength = st.max_len; inp.value = cur;
            if (st.charset === 'digits') inp.inputMode = 'numeric';
            inp.addEventListener('change', function () { cell.settings[st.key] = inp.value; changed(false); });
        }
        lab.appendChild(inp);
        return lab;
    }

    // ---------------------------------------------------------------- changes
    function newCell(id) { return { section: id, settings: {} }; }

    function addRow(id, at) {
        if (!id) { status(t('pickFirst'), true); return; }
        if (!canAdd(id)) { status(t('full'), true); return; }
        if (S.rows.length >= MAX_ROWS) { status(t('tooMany'), true); return; }
        var row = { cols: [newCell(id)] };
        if (at === undefined) S.rows.push(row); else S.rows.splice(at, 0, row);
        changed(true);
    }

    function takeCell(ri, ci) {
        var cell = S.rows[ri].cols.splice(ci, 1)[0];
        if (!S.rows[ri].cols.length) S.rows.splice(ri, 1);
        return cell;
    }

    function removeCell(ri, ci) { takeCell(ri, ci); changed(true); }

    // Left/right: swap with the neighbour in the row.
    function moveH(ri, ci, step) {
        var cols = S.rows[ri].cols, to = ci + step;
        if (to < 0 || to >= cols.length) return;
        var x = cols[ci]; cols[ci] = cols[to]; cols[to] = x;
        changed(true);
    }

    // Up/down: a section alone in its row swaps rows with its neighbour; a section sharing a
    // row steps out into a new row of its own above or below.
    function moveV(ri, ci, step) {
        var row = S.rows[ri];
        if (row.cols.length === 1) {
            var to = ri + step;
            if (to < 0 || to >= S.rows.length) return;
            S.rows[ri] = S.rows[to]; S.rows[to] = row;
        } else {
            if (S.rows.length >= MAX_ROWS) { status(t('tooMany'), true); return; }
            var cell = row.cols.splice(ci, 1)[0];
            S.rows.splice(step < 0 ? ri : ri + 1, 0, { cols: [cell] });
        }
        changed(true);
    }

    // A drop on a row: a section from the panel joins it; a cell from the canvas moves into it.
    function dropInRow(ri, id, from) {
        var row = S.rows[ri];
        if (from) {
            var p = from.split(',').map(Number);
            if (p[0] === ri) return;
            if (row.cols.length >= MAX_COLS) { status(t('rowFull'), true); return; }
            var target = row;
            var cell = takeCell(p[0], p[1]);
            target.cols.push(cell);
        } else {
            if (!canAdd(id)) { status(t('full'), true); return; }
            if (row.cols.length >= MAX_COLS) { status(t('rowFull'), true); return; }
            row.cols.push(newCell(id));
        }
        changed(true);
    }

    function dropTarget(node, fn) {
        node.addEventListener('dragover', function (e) {
            var types = Array.prototype.slice.call(e.dataTransfer.types || []);
            if (types.indexOf('text/x-pb-section') >= 0 || types.indexOf('text/x-pb-cell') >= 0) {
                e.preventDefault(); node.classList.add('pb-over');
            }
        });
        node.addEventListener('dragleave', function () { node.classList.remove('pb-over'); });
        node.addEventListener('drop', function (e) {
            e.preventDefault(); e.stopPropagation(); node.classList.remove('pb-over');
            var id = e.dataTransfer.getData('text/x-pb-section');
            var from = e.dataTransfer.getData('text/x-pb-cell');
            if (id || from) fn(id, from);
        });
    }

    function changed(redraw) {
        S.dirty = true;
        status(t('unsaved'));
        if (redraw) { drawCanvas(); drawPanel(); }
        schedulePreview();
    }

    // ---------------------------------------------------------------- preview
    var previewTimer = null, previewSeq = 0;
    function schedulePreview() { clearTimeout(previewTimer); previewTimer = setTimeout(preview, 500); }
    function preview() {
        var seq = ++previewSeq;
        api('preview', { page: S.page, layout: layoutJson() }).then(function (r) {
            if (seq !== previewSeq) return;
            if (!r.ok) { status(errText(r.data), true); return; }
            document.getElementById('pb-frame').srcdoc = r.data.html || '';
        });
    }
    function fitFrame() {
        var box = document.getElementById('pb-frame-box'), f = document.getElementById('pb-frame');
        var scale = Math.min(1, box.clientWidth / 1280);
        f.style.transform = 'scale(' + scale + ')';
        f.style.height = Math.round(box.clientHeight / scale) + 'px';
    }

    // ---------------------------------------------------------------- load and save
    function loadLayout(page) {
        status(t('loading'));
        return api('layout?page=' + encodeURIComponent(page)).then(function (r) {
            if (!r.ok) { status(errText(r.data), true); return; }
            var d = r.data;
            S.page = d.page; S.version = d.version; S.dirty = false;
            S.rows = (d.layout && d.layout.rows) || [];
            S.pages = d.pages || []; S.history = d.history || [];
            if (S.pages.indexOf(S.page) < 0) S.pages.push(S.page);
            document.getElementById('pb-view').href = d.view || 'index.php';
            try { history.replaceState(null, '', 'pagebuilder.php?page=' + encodeURIComponent(S.page)); } catch (e) { /* old browser */ }
            drawPages(); drawHistory(); drawCanvas(); drawPanel(); preview();
            status(S.version ? t('saved', S.version) : '');
        });
    }

    function pageLabel(p) {
        var names = { home: ['Home', '처음 화면'], blog: ['Garden archive', '정원 기록 보관소'], gallery: ['Gallery', '갤러리'] };
        if (names[p]) return names[p][lang === 'ko' ? 1 : 0];
        return p.replace(/^p-/, '');
    }

    function drawPages() {
        var sel = document.getElementById('pb-page');
        sel.textContent = '';
        S.pages.forEach(function (p) {
            var o = el('option', '', pageLabel(p)); o.value = p; if (p === S.page) o.selected = true; sel.appendChild(o);
        });
    }

    function drawHistory() {
        var ol = document.getElementById('pb-history-list');
        ol.textContent = '';
        S.history.forEach(function (h) {
            var li = el('li', '', t('version') + ' ' + h.version + ' · ' + t('note_' + h.note) + ' · ' + h.at + ' ');
            if (h.version !== S.version) {
                li.appendChild(btn(t('restore'), '', function () { restore('version', h.version); }));
            }
            ol.appendChild(li);
        });
        var li0 = el('li', '', t('default') + ' ');
        li0.appendChild(btn(t('restore'), '', function () { restore('reset'); }));
        ol.appendChild(li0);
    }

    function save() {
        api('save', { page: S.page, base: S.version, layout: layoutJson() }).then(function (r) {
            if (!r.ok) { status(errText(r.data), true); return; }
            S.dirty = false;
            loadLayout(S.page).then(function () { status(t('saved', r.data.version)); });
        });
    }

    function restore(action, version) {
        if (S.dirty && !window.confirm(t('confirmLeave'))) return;
        api('restore', { page: S.page, base: S.version, action: action, version: version || 0 }).then(function (r) {
            if (!r.ok) { status(errText(r.data), true); return; }
            S.dirty = false;
            loadLayout(S.page).then(function () { status(t('saved', r.data.version)); });
        });
    }

    // ---------------------------------------------------------------- buttons
    document.getElementById('pb-save').addEventListener('click', save);
    document.getElementById('pb-undo').addEventListener('click', function () {
        if (S.dirty) { if (window.confirm(t('confirmLeave'))) { loadLayout(S.page).then(function () { status(t('discard')); }); } return; }
        if (window.confirm(t('confirmUndo'))) restore('undo');
    });
    document.getElementById('pb-reset').addEventListener('click', function () { if (window.confirm(t('confirmReset'))) restore('reset'); });
    document.getElementById('pb-add').addEventListener('click', function () { addRow(S.selected); });
    dropTarget(document.getElementById('pb-add'), function (id, from) {
        if (from) {
            var p = from.split(',').map(Number);
            if (S.rows[p[0]].cols.length === 1) return;
            S.rows.push({ cols: [takeCell(p[0], p[1])] }); changed(true);
        } else { addRow(id); }
    });
    document.getElementById('pb-page').addEventListener('change', function (e) {
        var to = e.target.value;
        if (S.dirty && !window.confirm(t('confirmLeave'))) { e.target.value = S.page; return; }
        loadLayout(to);
    });
    document.getElementById('pb-newpage').addEventListener('click', function () {
        var name = (window.prompt(t('newName')) || '').trim().toLowerCase();
        if (!name) return;
        if (!/^[a-z0-9-]{1,40}$/.test(name)) { status(t('badName'), true); return; }
        if (S.dirty && !window.confirm(t('confirmLeave'))) return;
        loadLayout('p-' + name);
    });
    document.getElementById('pb-lang').addEventListener('click', function () {
        lang = lang === 'ko' ? 'en' : 'ko';
        try { localStorage.setItem('pb_lang', lang); } catch (e) { /* private window */ }
        applyWords(); drawPages(); drawHistory(); drawCanvas(); drawPanel();
        status(S.dirty ? t('unsaved') : (S.version ? t('saved', S.version) : ''));
    });
    window.addEventListener('beforeunload', function (e) { if (S.dirty) { e.preventDefault(); e.returnValue = ''; } });
    window.addEventListener('resize', fitFrame);

    // ---------------------------------------------------------------- start
    applyWords();
    fitFrame();
    api('registry').then(function (r) {
        if (!r.ok) { status(errText(r.data), true); return; }
        S.reg = r.data.sections || [];
        S.pending = r.data.pending || [];
        loadLayout(S.page);
        api('updates').then(function (u) {
            if (!u.ok) return;
            S.announced = u.data.announced || [];
            S.updateState = u.data.status;
            drawPanel();
        });
    });
})();
