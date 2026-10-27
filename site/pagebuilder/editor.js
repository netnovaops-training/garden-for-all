// The page builder's editor (pagebuilder.php). Owner only; the server checks every request.
//
// HOW TO READ THIS FILE. `S` holds the editor's state: the sections (from the registry), the
// page's layout (rows of 1-4 cells), and the published version it started from. Every change
// goes through `apply()` (a change to the rows, checked by logic.js) or `edit()` (any other
// change); both remember the old state for Undo, redraw and ask the server for a fresh preview.
// Text from the server is only ever put on the page with textContent (never as HTML), except
// the preview, which is the server's own page drawn in a sandboxed frame.
// The page tab is editor-pages.js, the features tab is editor-packs.js; they use `window.PB`.
// Design: docs/page-builder/DESIGN.md. Guide: docs/page-builder/GUIDE.md.
(function () {
    'use strict';

    var app = document.getElementById('pb-app');
    var L = window.PBLogic;
    if (!app || !L) return;
    var TOKEN = app.getAttribute('data-token') || '';

    // ---------------------------------------------------------------- words (EN / KO)
    var WORDS = {
        en: {
            title: 'Page builder', page: 'Page', newPage: 'New page', undoEdit: 'Undo', redoEdit: 'Redo',
            saveDraft: 'Save draft', publish: 'Publish', view: 'View page', panelHint: 'Drag a section onto the page, or press Add.',
            canvas: 'Page', plusHint: 'Drop a section here for a new row.', later: 'Coming with the database update',
            preview: 'Live preview (as visitors see it)',
            tabSections: 'Sections', tabPage: 'Page', tabFeatures: 'Features',
            add: 'Add', addRow: 'New row with the chosen section', isNew: 'new', admin: 'signed-in only',
            full: 'already on this page', up: 'Move up', down: 'Move down', left: 'Move left',
            right: 'Move right', remove: 'Remove', empty: 'This page is empty. Press + or drag a section here.',
            published: 'Published (version {v}).', unsaved: 'Not published yet.', loading: 'Loading…',
            draftSaved: 'Draft saved. The page itself has not changed.', draftOpened: 'Your unpublished draft was opened. Publish it to show it to visitors.',
            draftThrow: 'Throw the draft away', draftGone: 'The draft was thrown away.',
            confirmLeave: 'You have changes that are not published. Leave them?',
            confirmDraftThrow: 'Throw the draft away and go back to the published page?',
            pickFirst: 'Choose a section on the left first (click its name).',
            rowFull: 'A row holds at most 4 sections.', tooMany: 'At most 40 rows.',
            newHere: 'New sections arrived with the last update:', nextUpdate: 'comes with the next garden update',
            updatesOff: 'Checking for new sections is not set up yet.', updatesFail: 'Could not check for new sections right now.',
            seenOk: 'OK', fromPack: 'from a feature pack',
            cellHelp: 'Section {n}. Alt and the arrow keys move it.', newName: 'Name of the new page (a-z, 0-9 and -):',
            badName: 'Use only a-z, 0-9 and - (up to 40).', nothingToUndo: 'Nothing to undo.', nothingToRedo: 'Nothing to redo.'
        },
        ko: {
            title: '페이지 편집기', page: '페이지', newPage: '새 페이지', undoEdit: '되돌리기', redoEdit: '다시 하기',
            saveDraft: '초안 저장', publish: '게시', view: '페이지 보기', panelHint: '섹션을 페이지로 끌어다 놓거나 추가를 누르세요.',
            canvas: '페이지', plusHint: '여기에 섹션을 놓으면 새 줄이 생겨요.', later: '데이터베이스 업데이트와 함께 올 섹션',
            preview: '미리보기 (방문자가 보는 모습)',
            tabSections: '섹션', tabPage: '페이지', tabFeatures: '기능',
            add: '추가', addRow: '고른 섹션으로 새 줄 만들기', isNew: '새로움', admin: '로그인한 사람만',
            full: '이미 이 페이지에 있어요', up: '위로', down: '아래로', left: '왼쪽으로',
            right: '오른쪽으로', remove: '빼기', empty: '빈 페이지예요. +를 누르거나 섹션을 끌어다 놓으세요.',
            published: '게시했어요 (버전 {v}).', unsaved: '아직 게시하지 않았어요.', loading: '불러오는 중…',
            draftSaved: '초안을 저장했어요. 페이지는 아직 그대로예요.', draftOpened: '게시하지 않은 초안을 열었어요. 방문자에게 보이려면 게시하세요.',
            draftThrow: '초안 버리기', draftGone: '초안을 버렸어요.',
            confirmLeave: '게시하지 않은 바뀐 내용이 있어요. 버릴까요?',
            confirmDraftThrow: '초안을 버리고 게시된 페이지로 돌아갈까요?',
            pickFirst: '먼저 왼쪽에서 섹션을 고르세요 (이름을 누르세요).',
            rowFull: '한 줄에는 섹션이 4개까지 들어가요.', tooMany: '줄은 40개까지예요.',
            newHere: '지난 업데이트로 새 섹션이 왔어요:', nextUpdate: '다음 정원 업데이트와 함께 와요',
            updatesOff: '새 섹션 확인이 아직 설정되지 않았어요.', updatesFail: '지금은 새 섹션을 확인할 수 없어요.',
            seenOk: '확인', fromPack: '기능 묶음에서 온 섹션',
            cellHelp: '섹션 {n}. Alt와 화살표 키로 옮겨요.', newName: '새 페이지 이름 (a-z, 0-9, - 만):',
            badName: 'a-z, 0-9, - 만 쓸 수 있어요 (40자까지).', nothingToUndo: '되돌릴 것이 없어요.', nothingToRedo: '다시 할 것이 없어요.'
        }
    };
    var lang = 'en';
    try { lang = localStorage.getItem('pb_lang') || (navigator.language || '').slice(0, 2); } catch (e) { /* private window */ }
    if (lang !== 'ko') lang = 'en';
    function t(k, v) { var s = (WORDS[lang][k] || WORDS.en[k] || k); return v === undefined ? s : s.replace('{v}', v).replace('{n}', v); }
    function nm(o, base) { return o[base + '_' + lang] || o[base + '_en'] || ''; }
    // The other editor files add their words here, then the labels are written again.
    function addWords(en, ko) { Object.assign(WORDS.en, en); Object.assign(WORDS.ko, ko); applyWords(); }

    // ---------------------------------------------------------------- state
    var S = { page: app.getAttribute('data-page') || 'home', reg: [], pending: [], announced: [], rows: [],
              meta: {}, version: 0, dirty: false, selected: null, pages: [], history: [], removed: [],
              focusId: null, simple: false };
    var H = L.History();
    function def(id) { for (var i = 0; i < S.reg.length; i++) if (S.reg[i].id === id) return S.reg[i]; return null; }
    function maxOf(id) { var d = def(id); return d ? d.max_per_page : 0; }
    function canAdd(id) { return !!def(id) && L.count(S.rows, id) < maxOf(id); }

    // ---------------------------------------------------------------- small helpers
    function el(tag, cls, text) { var e = document.createElement(tag); if (cls) e.className = cls; if (text !== undefined) e.textContent = text; return e; }
    function btn(text, title, fn, cls) {
        var b = el('button', cls || '', text); b.type = 'button';
        if (title) { b.title = title; b.setAttribute('aria-label', title); }
        b.addEventListener('click', fn); return b;
    }
    function $(id) { return document.getElementById(id); }
    function status(msg, bad) { var s = $('pb-status'); s.textContent = msg || ''; s.className = 'pb-status' + (bad ? ' pb-bad' : ''); }
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

    // The layout as the server wants it: sections, and the page's own words when they are set.
    function layoutJson() {
        var m = S.meta, o = { schema: 1, page: S.page, rows: S.rows.map(function (r) {
            return { cols: r.cols.map(function (c) { return { section: c.section, settings: c.settings || {} }; }) };
        }) };
        if (m.title_en) o.title_en = m.title_en;
        if (m.title_ko) o.title_ko = m.title_ko;
        if (typeof m.menu === 'boolean') o.menu = m.menu;
        if (typeof m.hero === 'boolean') o.hero = m.hero;
        if (m.removed) o.removed = true;
        return o;
    }
    function metaOf(l) { return { title_en: l.title_en || '', title_ko: l.title_ko || '', menu: l.menu, hero: l.hero, removed: !!l.removed }; }
    function snapshot() { return L.clone({ rows: S.rows, meta: S.meta }); }

    // ---------------------------------------------------------------- changes
    function afterChange(redraw) {
        S.dirty = true;
        status(t('unsaved'));
        if (redraw) { drawCanvas(); drawPanel(); }
        drawUndo();
        schedulePreview(); scheduleDraft();
        if (PB.onChange) PB.onChange();
    }
    // A change to the rows: `result` is what a logic.js function gave back.
    function apply(result) {
        if (L.isError(result)) { if (result.error !== 'none') status(t(result.error), true); return false; }
        H.push(snapshot()); S.rows = result; afterChange(true); return true;
    }
    // Any other change: run `fn`, which changes S.meta or a cell's settings.
    function edit(fn, redraw) { H.push(snapshot()); fn(); afterChange(!!redraw); }

    function undo() {
        var prev = H.undo(snapshot());
        if (!prev) { status(t('nothingToUndo')); return; }
        S.rows = prev.rows; S.meta = prev.meta; afterChange(true); if (PB.onLoad) PB.onLoad();
    }
    function redo() {
        var next = H.redo(snapshot());
        if (!next) { status(t('nothingToRedo')); return; }
        S.rows = next.rows; S.meta = next.meta; afterChange(true); if (PB.onLoad) PB.onLoad();
    }
    function drawUndo() { $('pb-undo').disabled = !H.canUndo(); $('pb-redo').disabled = !H.canRedo(); }

    // ---------------------------------------------------------------- words on the page
    function applyWords() {
        document.documentElement.lang = lang;
        document.querySelectorAll('[data-t]').forEach(function (e) { e.textContent = t(e.getAttribute('data-t')); });
        var lb = $('pb-lang');
        lb.textContent = lang === 'ko' ? 'English' : '한국어';
        lb.lang = lang === 'ko' ? 'en' : 'ko';
        $('pb-add').title = t('addRow');
        $('pb-add').setAttribute('aria-label', t('addRow'));
    }

    // ---------------------------------------------------------------- the sections list
    function drawPanel() {
        var list = $('pb-list');
        list.textContent = '';
        S.reg.forEach(function (d) {
            var li = el('li', 'pb-item' + (S.selected === d.id ? ' pb-selected' : ''));
            li.draggable = canAdd(d.id) && !S.simple;
            li.addEventListener('dragstart', function (e) { e.dataTransfer.setData('text/x-pb-section', d.id); e.dataTransfer.effectAllowed = 'copy'; });
            var head = el('div', 'pb-item-head');
            head.appendChild(btn(nm(d, 'name'), nm(d, 'desc'), function () { S.selected = d.id; drawPanel(); }, 'pb-item-name'));
            if (d.new) head.appendChild(el('span', 'pb-badge pb-new', t('isNew')));
            if (d.perm === 'admin') head.appendChild(el('span', 'pb-badge pb-lock', '🔒 ' + t('admin')));
            if (d.pack) head.appendChild(el('span', 'pb-badge pb-pack', t('fromPack')));
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
        var pend = $('pb-pending');
        pend.textContent = '';
        S.pending.forEach(function (id) { pend.appendChild(el('li', 'pb-desc', id)); });
        var us = { not_set_up: 'updatesOff', failed: 'updatesFail' }[S.updateState];
        $('pb-update-state').textContent = us ? t(us) : '';
        drawNewNote();
    }

    function drawNewNote() {
        var box = $('pb-updates');
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

    // ---------------------------------------------------------------- the page (canvas)
    function addRow(id, at) {
        if (!id) { status(t('pickFirst'), true); return; }
        S.focusId = id;
        apply(L.addRow(S.rows, id, maxOf(id), at));
    }

    function cellAt(from) { var p = from.split(',').map(Number); return { p: p, section: S.rows[p[0]] && S.rows[p[0]].cols[p[1]] ? S.rows[p[0]].cols[p[1]].section : null }; }

    function drawCanvas() {
        var box = $('pb-rows');
        box.textContent = '';
        if (!S.rows.length) box.appendChild(el('p', 'pb-hint pb-empty', t('empty')));
        box.appendChild(gap(0));
        S.rows.forEach(function (row, ri) {
            var r = el('div', 'pb-row');
            r.style.gridTemplateColumns = 'repeat(' + row.cols.length + ', minmax(0, 1fr))';
            dropTarget(r, function (id, from) {
                var c = from ? cellAt(from) : null;
                S.focusId = id || (c && c.section);
                apply(L.dropInRow(S.rows, ri, id, maxOf(id), c && c.p));
            });
            row.cols.forEach(function (cell, ci) { r.appendChild(drawCell(cell, ri, ci)); });
            box.appendChild(r);
            box.appendChild(gap(ri + 1));
        });
        if (S.focusId) {
            var all = box.querySelectorAll('[data-section]');
            for (var i = 0; i < all.length; i++) if (all[i].getAttribute('data-section') === S.focusId) { all[i].focus(); break; }
            S.focusId = null;
        }
    }

    // A thin drop zone between rows: a dropped section gets a row of its own there.
    function gap(at) {
        var g = el('div', 'pb-gap');
        g.setAttribute('aria-hidden', 'true');
        dropTarget(g, function (id, from) {
            var c = from ? cellAt(from) : null;
            S.focusId = id || (c && c.section);
            apply(L.dropAtGap(S.rows, at, id, maxOf(id), c && c.p));
        });
        return g;
    }

    function drawCell(cell, ri, ci) {
        var d = def(cell.section) || { name_en: cell.section, settings: [] };
        var c = el('div', 'pb-cell' + (d.perm === 'admin' ? ' pb-cell-admin' : ''));
        c.draggable = !S.simple;
        c.tabIndex = 0;
        c.setAttribute('data-section', cell.section);
        c.setAttribute('role', 'group');
        c.setAttribute('aria-label', nm(d, 'name') + '. ' + t('cellHelp', ri + 1));
        c.addEventListener('dragstart', function (e) { e.dataTransfer.setData('text/x-pb-cell', ri + ',' + ci); e.dataTransfer.effectAllowed = 'move'; });
        // Keyboard: Alt + an arrow moves the section (the buttons below do the same).
        c.addEventListener('keydown', function (e) {
            if (!e.altKey || e.target !== c) return;
            var k = { ArrowUp: [moveV, -1], ArrowDown: [moveV, 1], ArrowLeft: [moveH, -1], ArrowRight: [moveH, 1] }[e.key];
            if (k) { e.preventDefault(); k[0](ri, ci, k[1]); }
        });
        var top = el('div', 'pb-cell-top');
        top.appendChild(el('strong', '', (d.perm === 'admin' ? '🔒 ' : '') + nm(d, 'name')));
        top.appendChild(btn('−', t('remove'), function () { apply(L.remove(S.rows, ri, ci)); }, 'pb-minus'));
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
    function moveV(ri, ci, step) { S.focusId = S.rows[ri].cols[ci].section; apply(L.moveV(S.rows, ri, ci, step)); }
    function moveH(ri, ci, step) { S.focusId = S.rows[ri].cols[ci].section; apply(L.moveH(S.rows, ri, ci, step)); }

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
                inp.value = n; edit(function () { cell.settings[st.key] = n; });
            });
        } else {
            inp.type = 'text'; inp.maxLength = st.max_len; inp.value = cur;
            if (st.charset === 'digits') inp.inputMode = 'numeric';
            if (st.charset === 'decimal') inp.inputMode = 'decimal';
            inp.addEventListener('change', function () { edit(function () { cell.settings[st.key] = inp.value; }); });
        }
        lab.appendChild(inp);
        return lab;
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

    // ---------------------------------------------------------------- preview
    var previewTimer = null, previewSeq = 0;
    function schedulePreview() { clearTimeout(previewTimer); previewTimer = setTimeout(preview, 500); }
    function preview() {
        if (S.simple) return;
        var seq = ++previewSeq;
        api('preview', { page: S.page, layout: layoutJson() }).then(function (r) {
            if (seq !== previewSeq) return;
            if (!r.ok) { status(errText(r.data), true); return; }
            $('pb-frame').srcdoc = r.data.html || '';
        });
    }
    function fitFrame() {
        var box = $('pb-frame-box'), f = $('pb-frame');
        var scale = Math.min(1, box.clientWidth / 1280);
        f.style.transform = 'scale(' + scale + ')';
        f.style.height = Math.round(box.clientHeight / scale) + 'px';
    }

    // ---------------------------------------------------------------- drafts and publishing
    var draftTimer = null;
    // Unpublished work is also kept on the server, a little after the last change.
    function scheduleDraft() { clearTimeout(draftTimer); draftTimer = setTimeout(function () { saveDraft(true); }, 15000); }
    function saveDraft(quiet) {
        clearTimeout(draftTimer);
        if (!S.dirty) return Promise.resolve();
        return api('draft', { page: S.page, base: S.version, layout: layoutJson() }).then(function (r) {
            if (!r.ok) { if (!quiet) status(errText(r.data), true); return; }
            if (!quiet) status(t('draftSaved'));
        });
    }
    function publish() {
        clearTimeout(draftTimer);
        return api('save', { page: S.page, base: S.version, layout: layoutJson() }).then(function (r) {
            if (!r.ok) { status(errText(r.data), true); return false; }
            S.dirty = false;
            return loadLayout(S.page).then(function () { status(t('published', r.data.version)); return true; });
        });
    }
    function restore(action, version) {
        if (S.dirty && !window.confirm(t('confirmLeave'))) return;
        api('restore', { page: S.page, base: S.version, action: action, version: version || 0 }).then(function (r) {
            if (!r.ok) { status(errText(r.data), true); return; }
            S.dirty = false;
            loadLayout(S.page).then(function () { status(t('published', r.data.version)); });
        });
    }

    // ---------------------------------------------------------------- load
    function loadLayout(page) {
        status(t('loading'));
        return api('layout?page=' + encodeURIComponent(page)).then(function (r) {
            if (!r.ok) { status(errText(r.data), true); return; }
            var d = r.data;
            S.page = d.page; S.version = d.version; S.dirty = false;
            S.pages = d.pages || []; S.history = d.history || []; S.removed = d.removed || [];
            var l = d.layout || { rows: [] };
            S.draftOpened = false;
            if (d.draft && d.draft.layout) { l = d.draft.layout; S.draftOpened = true; }
            S.rows = l.rows || []; S.meta = metaOf(l);
            H.clear();
            $('pb-view').href = d.view || 'index.php';
            try { history.replaceState(null, '', 'pagebuilder.php?page=' + encodeURIComponent(S.page)); } catch (e) { /* old browser */ }
            drawCanvas(); drawPanel(); drawUndo(); drawDraftNote(); preview();
            if (PB.onLoad) PB.onLoad();
            if (S.draftOpened) { S.dirty = true; status(t('draftOpened')); }
            else status(S.version ? t('published', S.version) : '');
        });
    }

    function drawDraftNote() {
        var n = $('pb-draft-note');
        n.textContent = '';
        n.hidden = !S.draftOpened;
        if (!S.draftOpened) return;
        n.appendChild(document.createTextNode(t('draftOpened') + ' '));
        n.appendChild(btn(t('draftThrow'), '', function () {
            if (!window.confirm(t('confirmDraftThrow'))) return;
            api('discard-draft', { page: S.page }).then(function () { S.dirty = false; loadLayout(S.page).then(function () { status(t('draftGone')); }); });
        }));
    }

    function leaveOk() { return !S.dirty || window.confirm(t('confirmLeave')); }

    // ---------------------------------------------------------------- the shared object
    var PB = window.PB = {
        S: S, t: t, nm: nm, el: el, btn: btn, $: $, api: api, status: status, errText: errText,
        addWords: addWords, edit: edit, apply: apply, layoutJson: layoutJson, load: loadLayout,
        publish: publish, restore: restore, leaveOk: leaveOk, redrawPanel: drawPanel, preview: schedulePreview,
        lang: function () { return lang; }
    };
    // The other editor files listen with PB.on('load' | 'change' | 'lang' | 'tab', fn).
    var HANDLERS = { load: [], change: [], lang: [], tab: [] };
    function fire(name, arg) { HANDLERS[name].forEach(function (f) { f(arg); }); }
    PB.on = function (name, fn) { HANDLERS[name].push(fn); };
    PB.onLoad = function () { fire('load'); };
    PB.onChange = function () { fire('change'); };
    PB.onLang = function () { fire('lang'); };
    PB.onTab = function (n) { fire('tab', n); };

    // ---------------------------------------------------------------- buttons
    $('pb-save').addEventListener('click', publish);
    $('pb-draft').addEventListener('click', function () { saveDraft(false); });
    $('pb-undo').addEventListener('click', undo);
    $('pb-redo').addEventListener('click', redo);
    $('pb-add').addEventListener('click', function () { addRow(S.selected); });
    dropTarget($('pb-add'), function (id, from) {
        var c = from ? cellAt(from) : null;
        S.focusId = id || (c && c.section);
        apply(L.dropAtGap(S.rows, S.rows.length, id, maxOf(id), c && c.p));
    });
    $('pb-page').addEventListener('change', function (e) {
        var to = e.target.value;
        if (!leaveOk()) { e.target.value = S.page; return; }
        loadLayout(to);
    });
    $('pb-newpage').addEventListener('click', function () {
        var name = (window.prompt(t('newName')) || '').trim().toLowerCase();
        if (!name) return;
        if (!/^[a-z0-9-]{1,40}$/.test(name)) { status(t('badName'), true); return; }
        if (!leaveOk()) return;
        loadLayout('p-' + name);
    });
    $('pb-lang').addEventListener('click', function () {
        lang = lang === 'ko' ? 'en' : 'ko';
        try { localStorage.setItem('pb_lang', lang); } catch (e) { /* private window */ }
        applyWords(); drawCanvas(); drawPanel(); drawDraftNote();
        if (PB.onLang) PB.onLang();
        status(S.dirty ? t('unsaved') : (S.version ? t('published', S.version) : ''));
    });
    // Ctrl+Z / Ctrl+Y outside a text box.
    document.addEventListener('keydown', function (e) {
        var tag = (e.target.tagName || '').toLowerCase();
        if (tag === 'input' || tag === 'textarea' || tag === 'select' || !(e.ctrlKey || e.metaKey)) return;
        if (e.key === 'z' && !e.shiftKey) { e.preventDefault(); undo(); }
        else if (e.key === 'y' || (e.key === 'z' && e.shiftKey)) { e.preventDefault(); redo(); }
    });
    window.addEventListener('beforeunload', function (e) { if (S.dirty) { e.preventDefault(); e.returnValue = ''; } });
    window.addEventListener('resize', function () { setMode(); if (!S.simple) fitFrame(); });

    // Tabs of the sidebar.
    var TABS = ['sections', 'page', 'features'];
    function showTab(name) {
        TABS.forEach(function (n) {
            $('pb-pane-' + n).hidden = n !== name;
            $('pb-tab-' + n).setAttribute('aria-selected', n === name ? 'true' : 'false');
        });
        if (PB.onTab) PB.onTab(name);
    }
    PB.showTab = showTab;
    TABS.forEach(function (n, i) {
        var b = $('pb-tab-' + n);
        b.addEventListener('click', function () { showTab(n); });
        b.addEventListener('keydown', function (e) {
            var to = { ArrowRight: i + 1, ArrowLeft: i - 1 }[e.key];
            if (to === undefined) return;
            e.preventDefault(); var nb = $('pb-tab-' + TABS[(to + TABS.length) % TABS.length]); nb.focus(); nb.click();
        });
    });

    // Phones get the simple mode: one column, buttons only, no preview and no dragging.
    function setMode() {
        var simple = window.matchMedia('(max-width: 900px)').matches;
        if (simple === S.simple) return;
        S.simple = simple;
        app.classList.toggle('pb-simple', simple);
        drawCanvas(); drawPanel();
    }

    // ---------------------------------------------------------------- start
    setMode();
    applyWords();
    if (!S.simple) fitFrame();
    drawUndo();
    api('registry').then(function (r) {
        if (!r.ok) { status(errText(r.data), true); return; }
        S.reg = r.data.sections || [];
        S.pending = r.data.pending || [];
        PB.reloadRegistry = function () {
            return api('registry').then(function (x) {
                if (x.ok) { S.reg = x.data.sections || []; S.pending = x.data.pending || []; drawPanel(); drawCanvas(); }
            });
        };
        loadLayout(S.page);
        api('updates').then(function (u) {
            if (!u.ok) return;
            S.announced = u.data.announced || [];
            S.updateState = u.data.status;
            drawPanel();
        });
    });
})();
