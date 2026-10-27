// The editor's Page tab (editor.js loads first): the page's names, the top-menu and banner
// switches, going back to an older published version, removing a page and bringing it back,
// and the page picker at the top. All changes go through PB.edit, so Undo covers them.
(function () {
    'use strict';
    var PB = window.PB;
    if (!PB) return;
    var S = PB.S, t = PB.t, $ = PB.$, el = PB.el, btn = PB.btn;

    PB.addWords({
        pageSettings: 'This page', titleEn: 'Name (English)', titleKo: 'Name (Korean)',
        inMenu: 'Show in the top menu', banner: 'Banner (the photo strip at the top)',
        bannerDefault: 'As the page normally has it', bannerOn: 'Show it', bannerOff: 'Hide it',
        prevVersion: 'Go back to the previous published version', reset: 'Original page',
        removePage: 'Remove this page', history: 'Published versions', removedPages: 'Removed pages',
        version: 'version', note_save: 'published', note_undo: 'previous version', note_reset: 'original page',
        note_restore: 'brought back', restore: 'Bring back', original: 'original', noRemoved: 'None.',
        confirmPrev: 'Go back to the version before this one?', confirmReset: 'Put this page back to how it was originally? (It can be undone.)',
        confirmRemove: 'Remove this page? Visitors will no longer find it. You can bring it back later.',
        siteOwn: 'The site\'s own pages cannot be removed.', removed: 'The page was removed.', broughtBack: 'The page is back.'
    }, {
        pageSettings: '이 페이지', titleEn: '이름 (영어)', titleKo: '이름 (한국어)',
        inMenu: '맨 위 메뉴에 보이기', banner: '배너 (맨 위의 사진 띠)',
        bannerDefault: '페이지 기본대로', bannerOn: '보이기', bannerOff: '숨기기',
        prevVersion: '바로 앞에 게시한 버전으로 돌아가기', reset: '처음 페이지로',
        removePage: '이 페이지 지우기', history: '게시한 버전', removedPages: '지운 페이지',
        version: '버전', note_save: '게시', note_undo: '앞 버전', note_reset: '처음 모습',
        note_restore: '되살리기', restore: '되살리기', original: '처음 모습', noRemoved: '없어요.',
        confirmPrev: '바로 앞 버전으로 되돌릴까요?', confirmReset: '이 페이지를 처음 모습으로 되돌릴까요? (다시 되돌릴 수 있어요.)',
        confirmRemove: '이 페이지를 지울까요? 방문자는 더 이상 찾을 수 없어요. 나중에 되살릴 수 있어요.',
        siteOwn: '사이트 기본 페이지는 지울 수 없어요.', removed: '페이지를 지웠어요.', broughtBack: '페이지가 돌아왔어요.'
    });

    function entry(id) { for (var i = 0; i < S.pages.length; i++) if (S.pages[i].id === id) return S.pages[i]; return null; }
    function isOwn(id) { return /^p-/.test(id); }
    function label(p) { return PB.lang() === 'ko' ? p.name_ko : p.name_en; }

    // The page picker at the top.
    function drawPicker() {
        var sel = $('pb-page');
        sel.textContent = '';
        var seen = false;
        S.pages.forEach(function (p) {
            var o = el('option', '', label(p)); o.value = p.id;
            if (p.id === S.page) { o.selected = true; seen = true; }
            sel.appendChild(o);
        });
        if (!seen) { var n = el('option', '', S.page.replace(/^p-/, '')); n.value = S.page; n.selected = true; sel.appendChild(n); }
    }

    function drawFields() {
        var m = S.meta, e = entry(S.page);
        $('pb-title-en').value = m.title_en || '';
        $('pb-title-ko').value = m.title_ko || '';
        $('pb-in-menu').checked = typeof m.menu === 'boolean' ? m.menu : (e ? e.in_menu : true);
        $('pb-hero').value = typeof m.hero === 'boolean' ? (m.hero ? '1' : '0') : '';
        $('pb-remove-page').hidden = !isOwn(S.page);
    }

    function drawHistory() {
        var ol = $('pb-history-list');
        ol.textContent = '';
        S.history.forEach(function (h) {
            var li = el('li', '', t('version') + ' ' + h.version + ' · ' + t('note_' + h.note) + ' · ' + h.at + ' ');
            if (h.version !== S.version) li.appendChild(btn(t('restore'), '', function () { PB.restore('version', h.version); }));
            ol.appendChild(li);
        });
        var li0 = el('li', '', t('original') + ' ');
        li0.appendChild(btn(t('restore'), '', function () { PB.restore('reset'); }));
        ol.appendChild(li0);
    }

    function drawRemoved() {
        var ul = $('pb-removed-list');
        ul.textContent = '';
        if (!S.removed.length) ul.appendChild(el('li', 'pb-desc', t('noRemoved')));
        S.removed.forEach(function (id) {
            var li = el('li', '', id.replace(/^p-/, '') + ' ');
            li.appendChild(btn(t('restore'), '', function () { bringBack(id); }));
            ul.appendChild(li);
        });
    }

    // A removed page comes back as a new published version without the "removed" mark.
    function bringBack(id) {
        if (!PB.leaveOk()) return;
        PB.api('layout?page=' + encodeURIComponent(id)).then(function (r) {
            if (!r.ok) { PB.status(PB.errText(r.data), true); return; }
            var l = r.data.layout;
            delete l.removed;
            PB.api('save', { page: id, base: r.data.version, layout: l }).then(function (s) {
                if (!s.ok) { PB.status(PB.errText(s.data), true); return; }
                S.dirty = false;
                PB.load(id).then(function () { PB.status(t('broughtBack')); });
            });
        });
    }

    function removePage() {
        if (!isOwn(S.page)) { PB.status(t('siteOwn'), true); return; }
        if (!window.confirm(t('confirmRemove'))) return;
        if (!S.version) { S.dirty = false; PB.load('home'); return; }
        PB.edit(function () { S.meta.removed = true; });
        PB.publish().then(function (ok) { if (ok) { S.dirty = false; PB.load('home').then(function () { PB.status(t('removed')); }); } });
    }

    function draw() { drawPicker(); drawFields(); drawHistory(); drawRemoved(); }
    PB.on('load', draw);
    PB.on('lang', draw);

    $('pb-title-en').addEventListener('change', function (e) { PB.edit(function () { S.meta.title_en = e.target.value.trim(); }); drawPicker(); });
    $('pb-title-ko').addEventListener('change', function (e) { PB.edit(function () { S.meta.title_ko = e.target.value.trim(); }); });
    $('pb-in-menu').addEventListener('change', function (e) { PB.edit(function () { S.meta.menu = e.target.checked; }); });
    $('pb-hero').addEventListener('change', function (e) {
        PB.edit(function () { S.meta.hero = e.target.value === '' ? undefined : e.target.value === '1'; });
    });
    $('pb-prev').addEventListener('click', function () {
        if (S.dirty && !window.confirm(t('confirmLeave'))) return;
        if (window.confirm(t('confirmPrev'))) PB.restore('undo');
    });
    $('pb-reset').addEventListener('click', function () { if (window.confirm(t('confirmReset'))) PB.restore('reset'); });
    $('pb-remove-page').addEventListener('click', removePage);
    draw();
})();
