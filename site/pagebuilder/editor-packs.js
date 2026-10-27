// The editor's Features tab (editor.js loads first). Feature packs are data only: art, styles,
// text sections. The order the owner sees: look at the signed list (nothing downloaded yet) ->
// Download (every file checked, PASS or FAIL) -> Review what it changes -> Install (one step, the
// replaced version is kept for Roll back) -> turn parts on (Apply, with Undo) -> Verify my install.
// Everything from the server goes on the page with textContent; art is shown as an image from a
// data address, after the server's strict reader has re-written it.
(function () {
    'use strict';
    var PB = window.PB;
    if (!PB) return;
    var t = PB.t, el = PB.el, btn = PB.btn, $ = PB.$;

    PB.addWords({
        featTitle: 'Features', featHint: 'New looks and parts for the site. They are only pictures, styles and text: nothing runs.',
        lookNow: 'Check for new features', lookWizard: 'Open what the wizard downloaded', looking: 'Looking…',
        lookFrom: 'Read from', signedYes: 'signed by the author', signedNo: 'NOT signed (no key set up)',
        listFrom: 'Features on offer', newer: 'new', changed: 'changed', same: 'you have this', older: 'older than yours',
        removedFromList: 'No longer on the list', size: 'size', files: 'files', showFull: 'show full', hideFull: 'hide',
        download: 'Download and check', downloading: 'Downloading…', changes: 'What it changes',
        pass: 'PASS', fail: 'FAIL', allPass: 'Every file passed. Nothing is turned on yet.',
        someFail: 'A file did not pass, so nothing was kept. Nothing on your site changed.',
        review: 'Review', installNow: 'Install', throwAway: 'Throw it away', acceptUnsigned: 'I understand this is not signed by the author and I want it anyway.',
        reviewTitle: 'Before you install', sigOk: 'The list is signed by the author.', sigNone: 'Not signed: nobody can prove who made it.',
        sigBad: 'The signature does not match.', installed: 'Installed features', none: 'Nothing installed yet.',
        rollback: 'Roll back', rollbackDone: 'Rolled back.', remove: 'Remove', removeAsk: 'Remove this feature?', removed: 'Removed.',
        installedOk: 'Installed. To use it, turn it on below.', previousKept: 'earlier version {v} is kept',
        turnOn: 'On', siteLook: 'Turn on and choose', header: 'Header art', catalogLayout: 'Catalog layout', theme: 'Colours',
        defaultWaves: 'The site\'s own waves', defaultCards: 'The site\'s own cards', defaultColours: 'The site\'s own colours',
        apply: 'Apply', applied: 'Saved (version {v}). Reload a page to see it.', undoSite: 'Undo the last change', undone: 'Undone.',
        verify: 'Verify my install', verifying: 'Checking…', verifyOk: 'Everything kept still matches the signed list.',
        verifyBad: 'Something does not match. Do not use the feature that says FAIL.',
        sources: 'Where to look', sourcesHint: 'The first one that answers is used. Addresses must start with https://.',
        saveSources: 'Save the places', sourcesSaved: 'Saved.', useDefaults: 'Use the defaults',
        gitea: 'the author\'s Gitea', drive: 'the author\'s Google Drive file', notSet: 'not set up',
        bundleWaiting: 'The wizard has a file ready.', tryLater: 'Could not do that right now.',
        part_header: 'header art', part_catalog_layout: 'catalog layout', part_theme: 'colours', part_section: 'section',
        previewArt: 'Drawing', checks: 'Checks', fingerprint: 'fingerprint', lookFirst: 'Check for new features first.',
        unsignedTick: 'Tick the box first.', byUser: 'by {v}', undoTitle: 'Undo'
    }, {
        featTitle: '기능', featHint: '사이트의 새 모습과 새 부분이에요. 그림, 스타일, 글뿐이라서 실행되는 것은 없어요.',
        lookNow: '새 기능 찾아보기', lookWizard: '마법사가 내려받은 것 열기', looking: '찾는 중…',
        lookFrom: '읽은 곳', signedYes: '제작자가 서명했어요', signedNo: '서명 없음 (키가 설정되지 않았어요)',
        listFrom: '받을 수 있는 기능', newer: '새로움', changed: '바뀜', same: '이미 있어요', older: '가진 것보다 옛 버전',
        removedFromList: '목록에서 빠진 기능', size: '크기', files: '파일', showFull: '전체 보기', hideFull: '접기',
        download: '내려받고 검사하기', downloading: '내려받는 중…', changes: '무엇이 바뀌나요',
        pass: '통과', fail: '실패', allPass: '모든 파일이 통과했어요. 아직 아무것도 켜지 않았어요.',
        someFail: '통과하지 못한 파일이 있어서 아무것도 보관하지 않았어요. 사이트는 그대로예요.',
        review: '살펴보기', installNow: '설치', throwAway: '버리기', acceptUnsigned: '제작자의 서명이 없다는 것을 알고, 그래도 쓰고 싶어요.',
        reviewTitle: '설치하기 전에', sigOk: '목록에 제작자의 서명이 있어요.', sigNone: '서명 없음: 누가 만들었는지 증명할 수 없어요.',
        sigBad: '서명이 맞지 않아요.', installed: '설치된 기능', none: '아직 설치된 것이 없어요.',
        rollback: '되돌리기', rollbackDone: '되돌렸어요.', remove: '지우기', removeAsk: '이 기능을 지울까요?', removed: '지웠어요.',
        installedOk: '설치했어요. 쓰려면 아래에서 켜세요.', previousKept: '이전 버전 {v}을(를) 보관 중',
        turnOn: '켜기', siteLook: '켜고 고르기', header: '머리글 그림', catalogLayout: '카탈로그 배치', theme: '색',
        defaultWaves: '사이트 기본 물결', defaultCards: '사이트 기본 카드', defaultColours: '사이트 기본 색',
        apply: '적용', applied: '저장했어요 (버전 {v}). 페이지를 새로 고치면 보여요.', undoSite: '마지막 변경 되돌리기', undone: '되돌렸어요.',
        verify: '설치한 것 검사하기', verifying: '검사하는 중…', verifyOk: '보관한 것이 모두 서명된 목록과 같아요.',
        verifyBad: '맞지 않는 것이 있어요. 실패라고 나온 기능은 쓰지 마세요.',
        sources: '찾아볼 곳', sourcesHint: '대답하는 첫 곳을 써요. 주소는 https:// 로 시작해야 해요.',
        saveSources: '찾을 곳 저장', sourcesSaved: '저장했어요.', useDefaults: '기본값 쓰기',
        gitea: '제작자의 Gitea', drive: '제작자의 구글 드라이브 파일', notSet: '설정 안 됨',
        bundleWaiting: '마법사가 준비해 둔 파일이 있어요.', tryLater: '지금은 할 수 없어요.',
        part_header: '머리글 그림', part_catalog_layout: '카탈로그 배치', part_theme: '색', part_section: '섹션',
        previewArt: '그림', checks: '검사', fingerprint: '지문', lookFirst: '먼저 새 기능을 찾아보세요.',
        unsignedTick: '먼저 칸에 표시하세요.', byUser: '{v}', undoTitle: '되돌리기'
    });

    var pane = $('pb-pane-features');
    var F = { data: null, list: null, dl: {}, review: null, verify: null, busy: false, msg: '', bad: false, sourcesOpen: false };

    function say(msg, bad) { F.msg = msg || ''; F.bad = !!bad; PB.status(msg, bad); }
    function tw(o, base) { return PB.nm(o, base); }
    function words(en, ko) { return PB.lang() === 'ko' ? (ko || en) : en; }
    function load() {
        return PB.api('features').then(function (r) {
            if (r.ok) { F.data = r.data; F.site = null; draw(); } else say(PB.errText(r.data), true);
        });
    }
    function call(path, body, busy) {
        F.busy = true; if (busy) say(t(busy)); draw();
        return PB.api(path, body).then(function (r) { F.busy = false; return r; });
    }
    function fail(r) { say(PB.errText(r.data), true); draw(); }

    function chip(text, cls) { return el('span', 'pb-badge ' + (cls || ''), text); }
    function short(sha) { return sha.slice(0, 12) + '…'; }

    // sha256 with a "show full" switch
    function shaSpan(sha) {
        var s = el('span'), full = false, code = el('code', '', short(sha));
        var b = btn(t('showFull'), '', function () {
            full = !full; code.textContent = full ? sha : short(sha); b.textContent = t(full ? 'hideFull' : 'showFull');
        });
        s.appendChild(code); s.appendChild(document.createTextNode(' ')); s.appendChild(b);
        return s;
    }

    function partLine(p) {
        var d = el('div', 'pb-desc');
        d.appendChild(el('strong', '', tw(p, 'name') + ' '));
        d.appendChild(chip(t('part_' + p.kind)));
        d.appendChild(el('div', '', words(p.what_en, p.what_ko)));
        if (p.art) p.art.forEach(function (a) {
            if (a.svg) {
                var img = el('img', 'pb-art');
                img.alt = t('previewArt'); img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(a.svg);
                img.style.height = '38px'; img.style.width = '100%'; d.appendChild(img);
            } else d.appendChild(el('div', 'pb-fail', words(a.error, a.error_ko)));
        });
        return d;
    }

    function filesTable(files, report) {
        var dl = el('dl');
        files.forEach(function (f) {
            var rep = report && report.filter(function (r) { return r.path === f.path; })[0];
            var dt = el('dt', '', f.path + ' · ' + f.bytes + ' B');
            if (rep) { dt.appendChild(document.createTextNode(' ')); dt.appendChild(el('span', rep.ok ? 'pb-pass' : 'pb-fail', t(rep.ok ? 'pass' : 'fail'))); }
            var dd = el('dd'); dd.appendChild(shaSpan(f.sha256));
            if (rep && !rep.ok) dd.appendChild(el('div', 'pb-fail', words(rep.why_en, rep.why_ko)));
            dl.appendChild(dt); dl.appendChild(dd);
        });
        return dl;
    }

    // ------------------------------------------------------------ the list of offers
    function drawOffer(p) {
        var box = el('div', 'pb-feat');
        var h = el('h3', '', tw(p, 'name') + ' ' + p.version + ' ');
        h.appendChild(chip(t({ 'new': 'newer', changed: 'changed', same: 'same', older: 'older' }[p.status]), p.status === 'new' ? 'pb-new' : ''));
        box.appendChild(h);
        box.appendChild(el('p', 'pb-desc', tw(p, 'desc')));
        box.appendChild(el('div', 'pb-desc', t('size') + ': ' + p.bytes + ' B · ' + t('files') + ': ' + p.files.length));
        var ch = el('div'); ch.appendChild(el('strong', '', t('changes') + ':'));
        p.parts.forEach(function (x) { ch.appendChild(partLine(x)); });
        box.appendChild(ch);
        var dtl = el('details'); dtl.appendChild(el('summary', '', t('files') + ' (' + p.files.length + ')'));
        dtl.appendChild(filesTable(p.files, F.dl[p.id] && F.dl[p.id].report));
        box.appendChild(dtl);
        var d = F.dl[p.id];
        if (d) {
            box.appendChild(el('p', d.ok ? 'pb-pass' : 'pb-fail', t(d.ok ? 'allPass' : 'someFail')));
            // The files that failed, in plain words, where they cannot be missed.
            d.report.filter(function (f) { return !f.ok; }).forEach(function (f) {
                box.appendChild(el('div', 'pb-fail', t('fail') + ' ' + f.path + ': ' + words(f.why_en, f.why_ko)));
            });
            if (d.ok) box.appendChild(btn(t('review'), '', function () { openReview(p.id); }, 'pb-primary'));
        }
        if (p.status !== 'same' && p.status !== 'older') {
            var b = btn(t('download'), '', function () { download(p.id); });
            b.disabled = F.busy; box.appendChild(b);
        }
        return box;
    }

    function drawList(root) {
        var L = F.list;
        if (!L) return;
        root.appendChild(el('h3', '', t('listFrom')));
        var info = el('p', 'pb-desc', t('lookFrom') + ': ' + L.source + ' · ' + L.released + ' · ');
        info.appendChild(el('span', L.signed ? 'pb-pass' : 'pb-fail', t(L.signed ? 'signedYes' : 'signedNo')));
        root.appendChild(info);
        (L.notes || []).forEach(function (n) { root.appendChild(el('p', 'pb-desc', n.source + ': ' + words(n.why, n.why_ko))); });
        L.packs.forEach(function (p) { root.appendChild(drawOffer(p)); });
        if (L.removed && L.removed.length) {
            root.appendChild(el('p', 'pb-desc', t('removedFromList') + ': ' + L.removed.map(function (r) { return tw(r, 'name') + ' ' + r.version; }).join(', ')));
        }
    }

    function afterList(r) {
        if (!r.ok) { fail(r); return; }
        F.list = r.data; F.dl = {}; say(''); draw();
    }
    function look() { call('features/check', {}, 'looking').then(afterList); }
    function lookWizard() { call('features/import', {}, 'looking').then(afterList); }

    function download(id) {
        call('features/download', { id: id }, 'downloading').then(function (r) {
            if (!r.ok) { fail(r); return; }
            F.dl[id] = r.data; say(t(r.data.ok ? 'allPass' : 'someFail'), !r.data.ok); load();
        });
    }

    // ------------------------------------------------------------ review and install
    function openReview(id) {
        PB.api('features/review?id=' + encodeURIComponent(id)).then(function (r) {
            if (!r.ok) { fail(r); return; }
            F.review = r.data; F.review.accept = false; draw();
        });
    }

    function drawReview(root) {
        var R = F.review;
        if (!R) return;
        var box = el('div', 'pb-feat');
        box.appendChild(el('h3', '', t('reviewTitle') + ': ' + tw(R, 'name') + ' ' + R.version));
        box.appendChild(el('p', 'pb-desc', tw(R, 'desc')));
        var sig = el('p', R.signature === 'signed' ? 'pb-pass' : 'pb-warn',
            t({ signed: 'sigOk', unsigned: 'sigNone', fail: 'sigBad' }[R.signature]) + (R.signature === 'fail' ? ' ' + words(R.signature_why, R.signature_why_ko) : ''));
        box.appendChild(sig);
        box.appendChild(el('strong', '', t('changes') + ':'));
        R.parts.forEach(function (x) { box.appendChild(partLine(x)); });
        box.appendChild(el('strong', '', t('checks') + ':'));
        var dl = el('dl');
        R.report.forEach(function (f) {
            var dt = el('dt', '', f.path + ' · ' + f.bytes + ' B ');
            dt.appendChild(el('span', f.ok ? 'pb-pass' : 'pb-fail', t(f.ok ? 'pass' : 'fail')));
            var dd = el('dd'); dd.appendChild(shaSpan(f.sha256)); dl.appendChild(dt); dl.appendChild(dd);
        });
        box.appendChild(dl);
        var allOk = R.report.every(function (f) { return f.ok; }) && R.signature !== 'fail';
        var tickBox = null;
        if (R.signature === 'unsigned') {
            var lab = el('label', 'pb-check'); tickBox = el('input'); tickBox.type = 'checkbox';
            lab.appendChild(tickBox); lab.appendChild(el('span', '', t('acceptUnsigned'))); box.appendChild(lab);
        }
        var inst = btn(t('installNow'), '', function () {
            if (tickBox && !tickBox.checked) { say(t('unsignedTick'), true); return; }
            call('features/install', { id: R.id, accept_unsigned: !!(tickBox && tickBox.checked) }).then(function (r) {
                if (!r.ok) { fail(r); return; }
                F.review = null; delete F.dl[R.id]; say(t('installedOk')); load(); if (PB.reloadRegistry) PB.reloadRegistry();
            });
        }, 'pb-primary');
        inst.disabled = !allOk || F.busy;
        box.appendChild(inst);
        box.appendChild(btn(t('throwAway'), '', function () {
            call('features/discard', { id: R.id }).then(function () { F.review = null; delete F.dl[R.id]; load(); });
        }));
        root.appendChild(box);
    }

    // ------------------------------------------------------------ installed, turn on, undo
    function parts(kind) {
        var out = [], on = F.site.enabled;
        F.data.installed.forEach(function (p) {
            if (on.indexOf(p.id) < 0) return;
            p.parts.forEach(function (x) { if (x.kind === kind) out.push({ value: p.id + '/' + x.id, label: tw(x, 'name') + ' (' + tw(p, 'name') + ')' }); });
        });
        return out;
    }

    function select(labelKey, kind, defKey) {
        var lab = el('label', 'pb-field'); lab.appendChild(el('span', '', t(labelKey)));
        var s = el('select'), o = el('option', '', t(defKey)); o.value = ''; s.appendChild(o);
        var cur = F.site[kind] || '';
        parts(kind === 'header' ? 'header' : kind === 'catalog_layout' ? 'catalog_layout' : 'theme').forEach(function (p) {
            var op = el('option', '', p.label); op.value = p.value; if (p.value === cur) op.selected = true; s.appendChild(op);
        });
        s.addEventListener('change', function () { F.site[kind] = s.value || null; });
        lab.appendChild(s);
        return lab;
    }

    function drawInstalled(root) {
        root.appendChild(el('h3', '', t('installed')));
        if (!F.data.installed.length) root.appendChild(el('p', 'pb-desc', t('none')));
        F.data.installed.forEach(function (p) {
            var box = el('div', 'pb-feat');
            var h = el('h3', '', tw(p, 'name') + ' ' + p.version + ' ');
            h.appendChild(chip(t(p.signed ? 'signedYes' : 'signedNo'), p.signed ? '' : 'pb-lock'));
            box.appendChild(h);
            box.appendChild(el('p', 'pb-desc', tw(p, 'desc')));
            p.parts.forEach(function (x) { var d = el('div', 'pb-desc', tw(x, 'name') + ' · ' + t('part_' + x.kind)); box.appendChild(d); });
            var lab = el('label', 'pb-check'), cb = el('input'); cb.type = 'checkbox'; cb.checked = F.site.enabled.indexOf(p.id) >= 0;
            cb.addEventListener('change', function () {
                var i = F.site.enabled.indexOf(p.id);
                if (cb.checked && i < 0) F.site.enabled.push(p.id);
                if (!cb.checked && i >= 0) {
                    F.site.enabled.splice(i, 1);
                    ['header', 'catalog_layout', 'theme'].forEach(function (k) { if (F.site[k] && F.site[k].split('/')[0] === p.id) F.site[k] = null; });
                }
                draw();
            });
            lab.appendChild(cb); lab.appendChild(el('span', '', t('turnOn'))); box.appendChild(lab);
            var prev = F.data.previous.filter(function (x) { return x.id === p.id; })[0];
            if (prev) {
                box.appendChild(el('div', 'pb-desc', t('previousKept', prev.version)));
                box.appendChild(btn(t('rollback'), '', function () {
                    call('features/rollback', { id: p.id }).then(function (r) { if (!r.ok) fail(r); else { say(t('rollbackDone')); load(); if (PB.reloadRegistry) PB.reloadRegistry(); } });
                }));
            }
            var rm = btn(t('remove'), '', function () {
                if (!window.confirm(t('removeAsk'))) return;
                call('features/remove', { id: p.id }).then(function (r) { if (!r.ok) fail(r); else { say(t('removed')); load(); } });
            });
            rm.disabled = F.data.site.state.enabled.indexOf(p.id) >= 0; box.appendChild(rm);
            root.appendChild(box);
        });
        if (!F.data.installed.length) return;
        var box = el('div', 'pb-feat');
        box.appendChild(el('h3', '', t('siteLook')));
        box.appendChild(select('header', 'header', 'defaultWaves'));
        box.appendChild(select('catalogLayout', 'catalog_layout', 'defaultCards'));
        box.appendChild(select('theme', 'theme', 'defaultColours'));
        box.appendChild(btn(t('apply'), '', apply, 'pb-primary'));
        var u = btn(t('undoSite'), t('undoTitle'), undoSite); u.disabled = F.data.site.version < 1; box.appendChild(u);
        root.appendChild(box);
    }

    function apply() {
        call('site/save', { base: F.data.site.version, state: F.site }).then(function (r) {
            if (!r.ok) { fail(r); return; }
            say(t('applied', r.data.version)); load(); if (PB.reloadRegistry) PB.reloadRegistry(); PB.preview();
        });
    }
    function undoSite() {
        call('site/undo', { base: F.data.site.version }).then(function (r) {
            if (!r.ok) { fail(r); return; }
            say(t('undone')); load(); if (PB.reloadRegistry) PB.reloadRegistry(); PB.preview();
        });
    }

    // ------------------------------------------------------------ verify, sources
    function drawVerify(root) {
        var box = el('div', 'pb-feat');
        box.appendChild(btn(t('verify'), '', function () {
            call('features/verify', {}, 'verifying').then(function (r) { if (!r.ok) fail(r); else { F.verify = r.data; say(t(r.data.ok ? 'verifyOk' : 'verifyBad'), !r.data.ok); draw(); } });
        }));
        if (F.verify) {
            F.verify.packs.forEach(function (p) {
                var d = el('div', 'pb-desc');
                d.appendChild(el('strong', '', tw(F.data.installed.filter(function (x) { return x.id === p.id; })[0] || { name_en: p.id }, 'name') + ' ' + p.version + ' '));
                d.appendChild(el('span', p.ok ? 'pb-pass' : 'pb-fail', t(p.ok ? 'pass' : 'fail')));
                p.files.filter(function (f) { return !f.ok; }).forEach(function (f) { d.appendChild(el('div', 'pb-fail', f.path + ': ' + words(f.why_en, f.why_ko))); });
                if (p.signature === 'fail') d.appendChild(el('div', 'pb-fail', words(p.signature_why, p.signature_why_ko)));
                box.appendChild(d);
            });
        }
        root.appendChild(box);
    }

    function drawSources(root) {
        var dtl = el('details'); dtl.open = F.sourcesOpen;
        dtl.addEventListener('toggle', function () { F.sourcesOpen = dtl.open; });
        dtl.appendChild(el('summary', '', t('sources')));
        dtl.appendChild(el('p', 'pb-hint', t('sourcesHint')));
        var inputs = [];
        F.data.sources.forEach(function (s) {
            var lab = el('label', 'pb-field'); lab.appendChild(el('span', '', s.name + ' (' + s.kind + ')'));
            var i = el('input'); i.type = 'url'; i.value = s.url; i.placeholder = t('notSet'); i.maxLength = 400; lab.appendChild(i);
            inputs.push([s, i]); dtl.appendChild(lab);
        });
        dtl.appendChild(btn(t('saveSources'), '', function () {
            var list = inputs.map(function (p) { return { name: p[0].name, kind: p[0].kind, url: p[1].value.trim() }; });
            call('features/sources', { sources: list }).then(function (r) { if (!r.ok) fail(r); else { say(t('sourcesSaved')); load(); } });
        }));
        dtl.appendChild(btn(t('useDefaults'), '', function () {
            call('features/sources', { sources: [] }).then(function (r) { if (!r.ok) fail(r); else { say(t('sourcesSaved')); load(); } });
        }));
        root.appendChild(dtl);
    }

    // ------------------------------------------------------------ the pane
    function draw() {
        pane.textContent = '';
        pane.appendChild(el('h2', '', t('featTitle')));
        pane.appendChild(el('p', 'pb-hint', t('featHint')));
        if (!F.data) { pane.appendChild(el('p', 'pb-hint', t('looking'))); return; }
        if (!F.site) F.site = JSON.parse(JSON.stringify(F.data.site.state));
        var row = el('p', 'pb-actions');
        var b = btn(t('lookNow'), '', look, 'pb-primary'); b.disabled = F.busy; row.appendChild(b);
        if (F.data.bundle_waiting) { var w = btn(t('lookWizard'), t('bundleWaiting'), lookWizard); w.disabled = F.busy; row.appendChild(w); }
        pane.appendChild(row);
        pane.appendChild(el('p', F.bad ? 'pb-fail' : 'pb-hint', F.msg));
        drawReview(pane);
        drawList(pane);
        drawInstalled(pane);
        drawVerify(pane);
        drawSources(pane);
    }

    PB.on('tab', function (n) { if (n === 'features' && !F.data) load(); });
    PB.on('lang', function () { draw(); });
    draw();
}());
