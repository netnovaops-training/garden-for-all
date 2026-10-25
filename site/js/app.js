// js/app.js
import { openModal, closeModal, initModalListeners } from './modal.js';
import { initHolographicTilt, launchFirework } from './effects.js';
import { startBoogie } from './boogie.js';
import {
    getBannerState,
    generateStarHTML,
    getRandomTheme,
    getFavoriteTheme,
    renderSeasonalScaleJS,
    renderEvergreenMonthBar,
    renderFlowerlessTag,
    getAttractsIcon
} from './ui-widgets.js';
import { initUserNotes, interceptCardClick, openNoteEditor, refreshNoteBadges, hasNote } from './user-notes.js';

const config = window.GARDEN_CONFIG || {};
window.plantData = window.plantData || {};
let isBulkMode = false;

// ── Per-user "Done" tracking (Tags view) ───────────────────────────────────
//   _myDone     = the logged-in user's own done plants (drives the checkbox)
//   _doneSet    = the set currently HIGHLIGHTED on the grid (defaults to _myDone,
//                 but an admin can switch it to view another user via setDoneView)
//   _doneViewUser = whose marks are shown (null = my own)
window._myDone = window._myDone || new Set();
window._doneSet = window._doneSet || new Set();
window._doneViewUser = window._doneViewUser || null;

function _doneLabel() {
    const u = window._doneViewUser;
    if (!u) return '✓ Done';
    return '✓ ' + (u === 'Joshua' ? 'Jun' : u);
}
function refreshDoneBadges() {
    const label = _doneLabel();
    document.querySelectorAll('.sv-grid-card[data-id]').forEach(card => {
        const done = window._doneSet.has(String(card.getAttribute('data-id')));
        card.classList.toggle('is-done', done);
        let badge = card.querySelector('.sv-done-badge');
        if (done) {
            if (!badge) { badge = document.createElement('div'); badge.className = 'sv-done-badge'; (card.querySelector('.svd-card') || card).appendChild(badge); }
            badge.textContent = label;
        } else if (badge) { badge.remove(); }
    });
}
function loadDoneSet() {
    if (!config.isAdmin) { window._doneReady = Promise.resolve(); return; }
    window._doneReady = fetch('plant_actions.php?action=list_done')
        .then(r => r.json())
        .then(d => {
            window._myDone = new Set((d.done || []).map(String));
            if (!window._doneViewUser) { window._doneSet = new Set(window._myDone); refreshDoneBadges(); }
        }).catch(() => {});
}
// Admin: switch which user's done-marks are highlighted on the grid (read-only view)
window.setDoneView = function (user) {
    window._doneViewUser = (user && user !== config.currentUser) ? user : null;
    if (!window._doneViewUser) { window._doneSet = new Set(window._myDone); refreshDoneBadges(); return; }
    fetch('plant_actions.php?action=list_done&user=' + encodeURIComponent(window._doneViewUser))
        .then(r => r.json())
        .then(d => { window._doneSet = new Set((d.done || []).map(String)); refreshDoneBadges(); })
        .catch(() => {});
};
document.addEventListener('DOMContentLoaded', loadDoneSet);

// Store original openModal so notes overlay can call it after dismissing
window._originalOpenModal = openModal;
window.openNoteEditor = openNoteEditor;
window.hasNote = hasNote;
let currentPage = 1, isLoading = false, hasMore = true, lastBedName = "";
let renderedBeds = new Set(); // tracks which bed headers have already been created
const feedContainer = document.getElementById('plant-feed');
const searchBox = document.getElementById('search-box');
const isMobile = () => window.innerWidth <= 768;

// ── Simple/Detailed view mode ──────────────────────────────────────────────
// Simple is the DEFAULT (when the visitor has no stored preference).
function isSimpleView() {
    if (typeof window.SIMPLE_VIEW !== 'undefined') return !!window.SIMPLE_VIEW;
    try {
        const v = localStorage.getItem('gardenSimpleView');
        return v === null ? true : v === '1';
    } catch (e) { return true; }
}
// Re-render the whole feed (used when the Simple/Detailed slider flips)
window.reloadFeed = function () { loadPlants(true); };

// 12-char month bitmap (J F M A M J J A S O N D; '.' = absent) → [start,end] names.
// Falls back to explicit start/end month-name fields.
const SV_MONTHS = ['January','February','March','April','May','June',
                   'July','August','September','October','November','December'];
function svMonthRange(bitmap, startName, endName) {
    if (bitmap) {
        const s = String(bitmap);
        let first = -1, last = -1;
        for (let i = 0; i < s.length && i < 12; i++) {
            const ch = s[i];
            if (ch && ch !== '.' && ch !== ' ') { if (first < 0) first = i; last = i; }
        }
        if (first >= 0) return [SV_MONTHS[first], SV_MONTHS[last]];
    }
    if (startName && String(startName).trim() && endName && String(endName).trim())
        return [String(startName).trim(), String(endName).trim()];
    return null;
}

// Build a 12-month boolean mask from the bitmap (handles gaps), or from a
// contiguous start/end month-name range as a fallback.
function svMonthMask(bitmap, startName, endName) {
    if (bitmap) {
        const s = String(bitmap);
        const mask = [];
        for (let i = 0; i < 12; i++) { const ch = s[i]; mask.push(!!(ch && ch !== '.' && ch !== ' ')); }
        if (mask.some(Boolean)) return mask;
    }
    const idx = n => SV_MONTHS.findIndex(m => m.slice(0, 3).toLowerCase() === String(n || '').trim().slice(0, 3).toLowerCase());
    const a = idx(startName), b = idx(endName);
    if (a >= 0 && b >= 0) { const mask = Array(12).fill(false); let i = a; while (true) { mask[i] = true; if (i === b) break; i = (i + 1) % 12; } return mask; }
    return null;
}

// Turn a month mask into readable grouped ranges, e.g. "Apr–Jun, Sep–Oct"
// (merges a December→January wrap so winter seasons read naturally).
function svSeasonText(mask) {
    if (!mask || !mask.some(Boolean)) return '';
    const M = SV_MONTHS; // full month names
    let runs = [], i = 0;
    while (i < 12) { if (mask[i]) { let j = i; while (j + 1 < 12 && mask[j + 1]) j++; runs.push([i, j]); i = j + 1; } else i++; }
    if (mask[0] && mask[11] && runs.length > 1) { const first = runs.shift(); const last = runs.pop(); runs.unshift([last[0], first[1]]); }
    return runs.map(([a, b]) => a === b ? M[a] : M[a] + '–' + M[b]).join(' and ');
}

// Small 12-cell month strip with active months highlighted (green grow / pink bloom).
function svMonthStrip(mask, type) {
    if (!mask) return '';
    const L = ['J','F','M','A','M','J','J','A','S','O','N','D'];
    const on = type === 'bloom' ? 'background:#ec407a;color:#fff;font-weight:700;' : 'background:#43a047;color:#fff;font-weight:700;';
    return '<div class="ms-strip">' + mask.map((m, i) => `<span class="ms-cell" style="${m ? on : ''}">${L[i]}</span>`).join('') + '</div>';
}

// Does this plant actually flower? Treats "Not ornamental", "None", blank, etc.
// as non-flowering (conifers/ferns), matching the detailed-modal logic.
function svNoFlower(v) {
    return !v || ['', 'none', 'n/a', 'null', 'unknown', 'not ornamental',
                  'insignificant', 'non-flowering', 'inconspicuous'].includes(String(v).trim().toLowerCase());
}
function svFlowerless(p) {
    const bc = (p.bloom_color || '').trim().toLowerCase();
    // Explicitly non-flowering (conifers/ferns tagged "Not ornamental", "None", etc.)
    if (['not ornamental', 'none', 'insignificant', 'non-flowering', 'inconspicuous', 'n/a'].includes(bc)) return true;
    // No bloom months recorded at all → nothing to show, treat as flowerless
    if (!p.bloom_months || !/[^.\s]/.test(p.bloom_months)) return true;
    return false;
}
// True only for full evergreens (not "semi-evergreen", which keeps its month bars).
function svEvergreen(p) {
    const e = (p.is_evergreen || '').trim().toLowerCase();
    return e === 'evergreen' || (e.startsWith('evergreen') && !e.includes('semi'));
}
function svEscHtml(s) {
    return (s == null ? '' : String(s)).replace(/[&<>"']/g,
        c => ({ '&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;' }[c]));
}
// <sv-guest-thumbs>
// Photo strip for visitors who are not signed in. The file name is only ever written
// into an attribute (data-guest-file) and read back with a click listener: it is never
// placed inside a piece of script text, so no character in a file name can break out.
function svGuestThumbHtml(file, isDefault, imgBase) {
    return `<img class="tg-thumb${isDefault ? ' is-default' : ''}" src="${imgBase + svEscHtml(file)}" alt=""
                          title="View photo" data-guest-file="${svEscHtml(file)}">`;
}
function svOpenGuestThumb(imgEl, imgBase, plantId) {
    const file = imgEl && imgEl.dataset ? imgEl.dataset.guestFile : null;
    if (!file) return;
    const full = imgBase + file;
    const main = document.getElementById('tg-main-img');
    if (main) main.src = full;
    if (window.openLightbox) window.openLightbox(full, plantId);
}
function svBindGuestThumbs(root, imgBase, plantId) {
    root.querySelectorAll('.tg-thumb[data-guest-file]').forEach(t =>
        t.addEventListener('click', () => svOpenGuestThumb(t, imgBase, plantId)));
}
// </sv-guest-thumbs>
// Animated "Coming Soon" placeholder SVG (spinner rotates, text stays still)
function _comingSoonSVG() {
    const pills = [];
    for (let i = 0; i < 12; i++) {
        const op = (1 - i * 0.065).toFixed(2);
        const colors = ['#1B5E20','#2E7D32','#388E3C','#43A047','#4CAF50','#66BB6A','#81C784','#A5D6A7','#B0BEC5','#B0BEC5','#90A4AE','#78909C'];
        pills.push(`<rect x="-7" y="-48" width="14" height="32" rx="7" fill="${colors[i]}" opacity="${op}" transform="rotate(${i*30})"/>`);
    }
    return `<div class="coming-soon-placeholder"><svg viewBox="0 0 140 170" width="140" height="170" xmlns="http://www.w3.org/2000/svg"><g transform="translate(70,78)"><g class="cs-spin">${pills.join('')}</g></g><text x="70" y="145" text-anchor="middle" font-family="Arial,sans-serif" font-size="14" font-weight="bold" fill="#444" letter-spacing="1.5">COMING SOON</text></svg></div>`;
}

window._comingSoonSVG = _comingSoonSVG;

// Inject simple-card styles once (shadow borrowed from the detailed modal).
(function injectSimpleCardStyles() {
    if (document.getElementById('sv-grid-styles')) return;
    const st = document.createElement('style');
    st.id = 'sv-grid-styles';
    st.textContent = `
    /* Simple view (Jun's Simple Tags): airy responsive grid of photo-left cards */
    body.simple-view-active #plant-feed{display:grid;grid-template-columns:repeat(auto-fill,minmax(460px,1fr));gap:24px;align-items:start;}
    body.simple-view-active .mobile-card-row{display:contents;}
    .plant-card.sv-grid-card{background:var(--svd-body,#fedbb7);border-radius:18px;overflow:hidden;border:2px solid transparent;
        box-shadow:0 10px 40px rgba(0,0,0,.15);transition:transform .18s ease, box-shadow .18s ease, border-color .3s;}
    .plant-card.sv-grid-card:hover{box-shadow:0 16px 52px rgba(0,0,0,.24);}
    .sv-grid-card .svd-card{position:relative;display:flex;height:100%;cursor:pointer;}
    .sv-grid-card .svd-photo{flex:0 0 210px;background:var(--svd-photo-bg,#dcedc8);}
    .sv-grid-card .svd-photo img{width:100%;height:100%;min-height:330px;object-fit:cover;display:block;}
    .sv-grid-card .svd-admin{margin-top:auto;padding-top:16px;display:flex;gap:8px;align-items:center;}
    .sv-grid-card .svd-admin .btn{padding:7px 14px;font-size:.85rem;}
    .sv-grid-card .svd-body{flex:1;min-width:0;display:flex;flex-direction:column;}
    .sv-grid-card .svd-header-green{background:var(--svd-header,#d3ebd4);padding:18px 22px 18px;
        border-radius:0 18px 50% 50% / 0 18px 10px 10px;}
    .sv-grid-card .svd-overview{flex:1;background:var(--svd-body,#fedbb7);padding:16px 22px 20px;border-radius:0 0 18px 0;}
    .sv-grid-card .svd-common{margin:0;font-size:1.4rem;color:var(--svd-header-text,#2e5a30);font-weight:900;line-height:1.18;}
    .sv-grid-card .svd-bot{font-style:italic;color:var(--svd-header-text,#2e5a30);opacity:0.75;font-size:1.05rem;margin-top:4px;}
    .sv-grid-card .svd-by{color:var(--svd-header-text,#2e5a30);opacity:0.85;font-weight:700;margin-top:10px;}
    .sv-grid-card .svd-tax{color:var(--svd-header-text,#2e5a30);opacity:0.7;font-size:.9rem;margin-top:12px;padding-bottom:14px;border-bottom:1px solid rgba(0,0,0,0.1);}
    .sv-grid-card .svd-chips{display:flex;flex-wrap:wrap;gap:8px;margin-bottom:14px;}
    .sv-grid-card .svd-chip{background:var(--svd-chip-bg,rgba(74,124,89,0.12));border-radius:20px;padding:5px 13px;font-size:.86rem;color:var(--svd-body-text,#3a4a36);font-weight:600;}
    .sv-grid-card .svd-rows{display:flex;flex-direction:column;gap:0;}
    .sv-grid-card .svd-row{display:flex;justify-content:space-between;gap:18px;font-size:.95rem;line-height:1.3;padding:7px 2px;border-bottom:1px dotted var(--svd-row-border,#c5b860);}
    .sv-grid-card .svd-row:last-child{border-bottom:none;}
    .sv-grid-card .svd-row span{color:var(--svd-row-label,#7a8c6f);font-weight:600;white-space:nowrap;}
    .sv-grid-card .svd-row b{color:var(--svd-body-text,#33402f);text-align:right;font-weight:700;}
    .sv-grid-card .svd-months{margin-top:14px;display:flex;flex-direction:column;gap:10px;}
    .sv-grid-card .sv-row{font-size:.92rem;color:var(--svd-body-text,#33402f);background:var(--svd-tint,rgba(74,124,89,0.1));border-radius:10px;padding:10px 14px;}
    .sv-grid-card .sv-row b{color:var(--svd-accent,#2e7d32);}
    .sv-grid-card .sv-bloom{background:#fce4ec !important;}
    .sv-grid-card .sv-bloom b{color:#c2185b !important;}
    .sv-grid-card .sv-ico{margin-right:6px;}
    .ms-strip{display:flex;gap:2px;margin-top:7px;}
    .ms-cell{flex:1;min-width:0;height:15px;border-radius:3px;font-size:.58rem;line-height:15px;text-align:center;background:rgba(0,0,0,.06);color:#9bb09b;}
    .sv-grid-card .sv-seasonbar{display:flex;align-items:center;gap:10px;}
    .sv-grid-card .sv-seasonbar > span{font-weight:700;color:var(--svd-body-text,#33402f);font-size:.92rem;white-space:nowrap;min-width:64px;}
    .sv-grid-card .sv-barwrap{flex:1;display:flex;}
    .sv-grid-card.is-done{outline:3px solid var(--svd-accent,#2e7d32);outline-offset:-3px;}
    .sv-grid-card .sv-done-badge{position:absolute;top:10px;left:10px;z-index:11;background:var(--svd-accent,#2e7d32);color:#fff;
        font-weight:800;font-size:.72rem;padding:3px 10px;border-radius:12px;box-shadow:0 1px 3px rgba(0,0,0,.3);}
    .sv-grid-card .svd-desc{margin-top:14px;font-size:.85rem;color:#52614c;line-height:1.55;
        background:var(--svd-tint,rgba(74,124,89,0.1));padding:11px 13px;border-radius:8px;}
    .sv-grid-card .sv-dead{position:absolute;top:10px;left:10px;background:#ff9a9e;color:#fff;font-weight:900;
        padding:4px 10px;border-radius:6px;z-index:10;font-size:.85rem;box-shadow:2px 2px 0 rgba(0,0,0,.1);}
    .sv-grid-card .sv-future-badge{position:absolute;top:10px;left:10px;background:linear-gradient(135deg,#7c4dff,#448aff);color:#fff;
        font-weight:900;font-size:.78rem;padding:4px 12px;border-radius:6px;z-index:10;box-shadow:2px 2px 0 rgba(0,0,0,.1);letter-spacing:.5px;}
    .sv-grid-card.future-plant{border-color:#7c4dff;border-width:2px;}
    .sv-grid-card.future-plant:hover{box-shadow:0 16px 52px rgba(124,77,255,.3);}
    /* ── Photo reposition button (admin, overview cards) ────── */
    .svd-photo{position:relative;}
    .svd-repo-btn{position:absolute;bottom:8px;left:8px;z-index:12;background:rgba(255,255,255,.9);border:2px solid var(--svd-accent,#2e7d32);
        color:var(--svd-accent,#2e7d32);border-radius:8px;padding:3px 8px;font-size:.75rem;font-weight:700;cursor:pointer;
        opacity:0;transition:opacity .2s;box-shadow:0 2px 6px rgba(0,0,0,.2);}
    .sv-grid-card:hover .svd-repo-btn{opacity:1;}
    .svd-photo.repo-active{outline:3px dashed var(--svd-accent,#2e7d32);outline-offset:-3px;}
    .svd-photo.repo-active img{cursor:grab;}
    .svd-photo.repo-active img.dragging{cursor:grabbing;}
    .svd-repo-save-bar{position:absolute;bottom:8px;left:8px;right:8px;z-index:12;display:flex;gap:6px;justify-content:center;}
    .svd-repo-save-bar button{font-size:.72rem;font-weight:700;border-radius:6px;padding:4px 10px;cursor:pointer;border:none;box-shadow:0 1px 4px rgba(0,0,0,.2);}
    .svd-repo-save-bar .repo-flip{background:#e3f2fd;color:#1565c0;}
    .svd-repo-save-bar .repo-save{background:var(--svd-accent,#2e7d32);color:#fff;}
    .svd-repo-save-bar .repo-cancel{background:#fff;color:#666;}
    /* ── Grid layout overrides (desktop only) ──────────────── */
    body.simple-view-active #plant-feed.grid-1x{grid-template-columns:1fr !important;}
    body.simple-view-active #plant-feed.grid-2x{grid-template-columns:repeat(2,1fr) !important;}
    body.simple-view-active #plant-feed.grid-3x{grid-template-columns:repeat(3,1fr) !important;}
    body.simple-view-active #plant-feed.grid-4x{grid-template-columns:repeat(4,1fr) !important;}
    /* Widen the page container for 3x/4x so columns stay the same card width as 2x */
    body.grid-layout-3 .catalog-container{max-width:1800px !important;}
    body.grid-layout-4 .catalog-container{max-width:2400px !important;}
    /* 3x/4x: same photo-left layout as 2x, clip only notes */
    #plant-feed.grid-3x .sv-grid-card .svd-desc,
    #plant-feed.grid-4x .sv-grid-card .svd-desc{display:-webkit-box;-webkit-line-clamp:5;-webkit-box-orient:vertical;overflow:hidden;}
    /* ── Layout picker ───────────────────────────────────────── */
    .svd-layout-bar{display:flex;justify-content:center;gap:10px;padding:6px 0 2px;flex-wrap:wrap;}
    .svd-layout-btn{width:34px;height:26px;border-radius:6px;border:2px solid rgba(0,0,0,.2);background:#fff;
        cursor:pointer;display:flex;gap:2px;padding:3px;align-items:stretch;transition:border-color .15s,transform .15s;box-shadow:0 1px 4px rgba(0,0,0,.15);}
    .svd-layout-btn:hover{transform:scale(1.1);}
    .svd-layout-btn.active{border-color:#222;transform:scale(1.1);box-shadow:0 2px 6px rgba(0,0,0,.3);}
    .svd-layout-col{flex:1;background:rgba(100,160,80,.45);border-radius:2px;}
    /* ── Theme picker bar ────────────────────────────────────── */
    .svd-theme-bar{display:flex;justify-content:center;gap:22px;padding:14px 0 8px;flex-wrap:wrap;}
    .svd-theme-item{display:flex;flex-direction:column;align-items:center;gap:4px;cursor:pointer;min-width:52px;}
    .svd-theme-item:hover .svd-theme-swatch{transform:scale(1.15);}
    .svd-theme-name{font-size:.84rem;font-weight:900;color:#333;white-space:nowrap;line-height:1.1;}
    .svd-theme-sub{font-size:.7rem;font-weight:800;color:#666;white-space:nowrap;margin-top:-1px;}
    .svd-theme-swatch{width:42px;height:34px;border-radius:50%;border:3px solid rgba(0,0,0,.25);
        transition:transform .15s,border-color .15s;box-shadow:0 2px 8px rgba(0,0,0,.3);position:relative;overflow:hidden;}
    .svd-theme-swatch.active{border-color:#222;transform:scale(1.1);box-shadow:0 3px 10px rgba(0,0,0,.4);}
    .svd-theme-swatch .swatch-half{position:absolute;top:0;width:50%;height:100%;}
    .svd-theme-swatch .swatch-left{left:0;border-radius:42px 0 0 42px;}
    .svd-theme-swatch .swatch-right{right:0;border-radius:0 42px 42px 0;}
    @media(max-width:560px){.sv-grid-card .svd-card{flex-direction:row;}.sv-grid-card .svd-photo{flex:0 0 38%;max-width:38%;}
        .sv-grid-card .svd-photo img{min-height:unset;height:100%;object-fit:cover;}body.simple-view-active #plant-feed{grid-template-columns:1fr;}
        .sv-grid-card .svd-header-green{border-radius:0;padding:10px 12px 8px;}
        .sv-grid-card .svd-header-green h3{font-size:.95rem;}
        .sv-grid-card .svd-overview{border-radius:0 0 18px 0;padding:10px 12px 14px;font-size:.82rem;}
        .svd-theme-bar{gap:14px;}.svd-theme-name{font-size:.72rem;}.svd-theme-sub{font-size:.58rem;}
        .svd-layout-wrap{display:none !important;}
        .tg-fav-row{padding:8px 12px;}}
    /* ── Coming Soon animated placeholder ──── */
    .coming-soon-placeholder{width:100%;height:100%;min-height:330px;display:flex;align-items:center;justify-content:center;background:var(--svd-photo-bg,#e8f5e9);}
    @media(max-width:560px){.coming-soon-placeholder{min-height:unset;height:100%;}}
    .tg-photo .coming-soon-placeholder{min-height:unset;max-height:430px;aspect-ratio:3/4;}
    @keyframes comingSoonSpin{to{transform:rotate(360deg)}}
    .cs-spin{animation:comingSoonSpin 1.8s linear infinite;transform-origin:0 0;}`;
    document.head.appendChild(st);
})();

// ── Garden Tag-View Theme System ─────────────────────────────────────────
// ── Theme definitions ────────────────────────────────────────────────
// Light headers ≈ same lightness as #d4ecd5 in each hue family.
// Dark headers = deeper, richer versions. Body stays same per family.
const SVD_THEMES = {
    // ── Green (default) ─────────────────────────────────────────────
    spring_d:   { label:'Spring',    sub:'Dark',   header:'#4A7c59', body:'#fffff0', photoBg:'#c8e6c9', accent:'#2e7d32', headerText:'#ffffff', chipBg:'rgba(46,125,50,0.10)', tint:'rgba(46,125,50,0.06)', rowBorder:'#c0d8a5', rowLabel:'#5a7a50', bodyText:'#33402f', modalBg:'#4A7c59' },
    spring_l:   { label:'Spring',    sub:'Light',  header:'#d4ecd5', body:'#fffff0', photoBg:'#e8f5e9', accent:'#2e7d32', headerText:'#2e5a30', chipBg:'rgba(46,125,50,0.10)', tint:'rgba(46,125,50,0.06)', rowBorder:'#c0d8a5', rowLabel:'#5a7a50', bodyText:'#33402f', modalBg:'#6aad7a' },
    // ── Red ─────────────────────────────────────────────────────────
    zenith:     { label:'Zenith',    sub:'',       header:'#e53935', body:'#fff5f4', photoBg:'#ffcdd2', accent:'#d32f2f', headerText:'#ffffff', chipBg:'rgba(211,47,47,0.08)',tint:'rgba(211,47,47,0.05)',rowBorder:'#e0a0a0', rowLabel:'#a04040', bodyText:'#3a1210', modalBg:'#b81c1c' },
    summer:     { label:'Cherry', sub:'Blossom', header:'#FF91A4', body:'#fff5f5', photoBg:'#ffe0e6', accent:'#D84356', headerText:'#5d1a24', chipBg:'rgba(216,67,86,0.08)', tint:'rgba(216,67,86,0.05)', rowBorder:'#f0b0b8', rowLabel:'#b05060', bodyText:'#3E2723', modalBg:'#e87088' },
    // ── Orange ──────────────────────────────────────────────────────
    sunset:     { label:'Sunset',    sub:'',       header:'#e8a060', body:'#fff8f0', photoBg:'#ffe0b8', accent:'#c05a10', headerText:'#4a2008', chipBg:'rgba(192,90,16,0.08)', tint:'rgba(192,90,16,0.05)', rowBorder:'#d0a070', rowLabel:'#905020', bodyText:'#4a2810', modalBg:'#c07030' },
    dirt:       { label:'Dirt',      sub:'',       header:'#d5c4ad', body:'#faf5ee', photoBg:'#f0e0d0', accent:'#7a5a3a', headerText:'#4a3520', chipBg:'rgba(155,118,83,0.08)',tint:'rgba(155,118,83,0.05)',rowBorder:'#c8a888', rowLabel:'#7a6040', bodyText:'#3a2a18', modalBg:'#a08060' },
    // ── Yellow ──────────────────────────────────────────────────────
    sunrise:    { label:'Sunrise',   sub:'',       header:'#f5ecc0', body:'#fdfce8', photoBg:'#fff8d0', accent:'#c0a020', headerText:'#5a4a10', chipBg:'rgba(192,160,32,0.08)',tint:'rgba(192,160,32,0.05)',rowBorder:'#d8cc80', rowLabel:'#8a7a30', bodyText:'#3a3010', modalBg:'#c0a040' },
    // ── Blue ────────────────────────────────────────────────────────
    jun:        { label:'Jun',       sub:'',       header:'#d0def5', body:'#f0f5ff', photoBg:'#dce8f5', accent:'#3366CC', headerText:'#1a3366', chipBg:'rgba(51,102,204,0.10)',tint:'rgba(51,102,204,0.06)',rowBorder:'#a0b8d8', rowLabel:'#4a6a90', bodyText:'#1a2a4a', modalBg:'#5a88bb' },
    // ── Indigo ──────────────────────────────────────────────────────
    winter_d:   { label:'Winter',    sub:'Dark',   header:'#1a5276', body:'#f0faff', photoBg:'#bbdefb', accent:'#1976D2', headerText:'#ffffff', chipBg:'rgba(25,118,210,0.08)',tint:'rgba(25,118,210,0.05)',rowBorder:'#90c0d8', rowLabel:'#3a7090', bodyText:'#0D47A1', modalBg:'#1a5276' },
    // ── Violet ──────────────────────────────────────────────────────
    lavender_d: { label:'Kona',      sub:'',       header:'#7B1FA2', body:'#faf0f5', photoBg:'#e1bee7', accent:'#8E24AA', headerText:'#ffffff', chipBg:'rgba(142,36,170,0.08)',tint:'rgba(142,36,170,0.05)',rowBorder:'#d0a8c8', rowLabel:'#8a5080', bodyText:'#4A148C', modalBg:'#7B1FA2' },
    lavender_l: { label:'Lavender',  sub:'Light',  header:'#e4d4ec', body:'#faf0f5', photoBg:'#f3e5f5', accent:'#8E24AA', headerText:'#5a2070', chipBg:'rgba(142,36,170,0.08)',tint:'rgba(142,36,170,0.05)',rowBorder:'#d0a8c8', rowLabel:'#8a5080', bodyText:'#4A148C', modalBg:'#9b70b0' },
    // ── Neutral ─────────────────────────────────────────────────────
    jason:      { label:'Author',     sub:'',       header:'#010203', body:'#f5f6f8', photoBg:'#e4e6ea', accent:'#2a2d32', headerText:'#ffffff', chipBg:'rgba(1,2,3,0.08)',    tint:'rgba(1,2,3,0.04)',    rowBorder:'#c0c4ca', rowLabel:'#606468', bodyText:'#1a1c20', modalBg:'#3a3e44' },
};
const SVD_THEME_KEY = 'garden_tag_theme';

function applySvdTheme(name) {
    // Migration: old keys renamed
    if (name === 'fall_l' || name === 'fall_d') name = 'fall';
    if (name === 'emerald_l') name = 'spring_l';
    if (name === 'fall') name = 'spring_d';
    if (name === 'emerald_d') name = 'spring_d';
    if (name === 'winter_l') name = 'winter_d';
    if (name === 'honey') name = 'sunrise';
    const t = SVD_THEMES[name] || SVD_THEMES.spring_d;
    const r = document.documentElement.style;
    r.setProperty('--svd-header',      t.header);
    r.setProperty('--svd-header-text', t.headerText);
    r.setProperty('--svd-body',        t.body);
    r.setProperty('--svd-photo-bg',    t.photoBg);
    r.setProperty('--svd-accent',      t.accent);
    r.setProperty('--svd-chip-bg',     t.chipBg);
    r.setProperty('--svd-tint',        t.tint);
    r.setProperty('--svd-row-border',  t.rowBorder);
    r.setProperty('--svd-row-label',   t.rowLabel);
    r.setProperty('--svd-body-text',   t.bodyText);
    r.setProperty('--modal-accent',    t.accent);
    // Store modal bg so openModal() can use it
    window._gardenThemeModalBg = t.modalBg;
    try { localStorage.setItem(SVD_THEME_KEY, name); } catch(e){}
    // Persist server-side for logged-in users (survives device/browser changes)
    if (window.GARDEN_CONFIG && window.GARDEN_CONFIG.currentUser) {
        fetch('plant_actions.php', {
            method: 'POST', headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: new URLSearchParams({ action: 'save_pref', key: 'tag_theme', value: name })
        }).catch(() => {});
    }
    // Update active swatch
    document.querySelectorAll('.svd-theme-swatch').forEach(s => {
        s.classList.toggle('active', s.dataset.theme === name);
    });
}

function buildThemePicker() {
    const bar = document.createElement('div');
    bar.className = 'svd-theme-bar';
    bar.innerHTML = Object.entries(SVD_THEMES).map(([key, t]) =>
        `<div class="svd-theme-item" onclick="applySvdTheme('${key}')">
            <div class="svd-theme-swatch" data-theme="${key}" title="${t.label} ${t.sub}">
                <div class="swatch-half swatch-left" style="background:${t.header}"></div>
                <div class="swatch-half swatch-right" style="background:${t.body}"></div>
            </div>
            <span class="svd-theme-name">${t.label}</span>
            ${t.sub ? `<span class="svd-theme-sub">${t.sub}</span>` : ''}
        </div>`
    ).join('');
    return bar;
}

// ── Grid layout picker ──────────────────────────────────────────────────
const GRID_KEY = 'garden_grid_layout';
function applyGridLayout(n) {
    const feed = document.getElementById('plant-feed');
    if (!feed) return;
    feed.classList.remove('grid-1x','grid-2x','grid-3x','grid-4x');
    if (n >= 1 && n <= 4) feed.classList.add('grid-' + n + 'x');
    document.body.classList.remove('grid-wide','grid-layout-1','grid-layout-2','grid-layout-3','grid-layout-4');
    document.body.classList.add('grid-layout-' + n);
    // Highlight active button
    document.querySelectorAll('.svd-layout-btn').forEach(b => b.classList.toggle('active', b.dataset.cols === String(n)));
    localStorage.setItem(GRID_KEY, n);
    // Server-side persistence for logged-in users
    if (window.GARDEN_CONFIG && window.GARDEN_CONFIG.currentUser) {
        fetch('plant_actions.php', { method:'POST', headers:{'Content-Type':'application/x-www-form-urlencoded'},
            body: new URLSearchParams({ action:'save_pref', key:'grid_layout', value: String(n) }) }).catch(()=>{});
    }
}
function buildLayoutPicker() {
    const wrap = document.createElement('div');
    wrap.className = 'svd-layout-wrap';
    wrap.style.cssText = 'margin-top:8px;border-top:1px solid var(--site-row-border, rgba(120,170,100,0.35));padding-top:8px;';
    const lbl = document.createElement('div');
    lbl.style.cssText = 'font-size:.78rem;font-weight:900;color:var(--site-accent, #3a5a30);letter-spacing:.6px;text-transform:uppercase;margin-bottom:4px;';
    lbl.textContent = 'Layout';
    wrap.appendChild(lbl);
    const bar = document.createElement('div');
    bar.className = 'svd-layout-bar';
    for (let n = 1; n <= 4; n++) {
        const btn = document.createElement('div');
        btn.className = 'svd-layout-btn';
        btn.dataset.cols = String(n);
        btn.title = n + ' column' + (n > 1 ? 's' : '');
        btn.onclick = () => applyGridLayout(n);
        for (let c = 0; c < n; c++) {
            const col = document.createElement('div');
            col.className = 'svd-layout-col';
            btn.appendChild(col);
        }
        bar.appendChild(btn);
    }
    wrap.appendChild(bar);
    return wrap;
}
window.applyGridLayout = applyGridLayout;

// ── Card Effects (animated creatures on hover) ───────────────────────
const EFFECT_KEY = 'garden_card_effect';
let _activeEffect = 'none';
let _effectWrap = null;
let _particleTimer = null;
let _effectPos = {x:0,y:0};
let _effectDir = 1;
let _prevEX = 0;
let _currentFxCard = null;

// Inject effect styles
(function(){
    const s = document.createElement('style');
    s.id = 'garden-effect-css';
    s.textContent = `
    .garden-effect-wrap{position:fixed;pointer-events:none;z-index:9999;transition:opacity .15s;}
    .garden-effect-wrap.hidden{opacity:0;}
    .garden-effect-wrap svg{display:block;}
    .effect-creature .deer-head{animation:deerEat 2s ease-in-out infinite;transform-origin:33px 16px;}
    @keyframes deerEat{0%,55%{transform:rotate(0)}65%{transform:rotate(18deg)}75%{transform:rotate(4deg)}85%{transform:rotate(14deg)}100%{transform:rotate(0)}}
    .effect-creature .rabbit-head{animation:rabNib 1.6s ease-in-out infinite;transform-origin:28px 20px;}
    @keyframes rabNib{0%,45%{transform:rotate(0)}55%{transform:rotate(12deg)}62%{transform:rotate(2deg)}70%{transform:rotate(10deg)}78%{transform:rotate(0)}100%{transform:rotate(0)}}
    .effect-creature .bee-wings{animation:wingBzz .1s ease-in-out infinite alternate;transform-origin:15px 8px;}
    @keyframes wingBzz{0%{transform:scaleY(1)}100%{transform:scaleY(.35)}}
    .effect-creature .bfly-wing-l{animation:bflyL .9s ease-in-out infinite;transform-origin:21px 16px;}
    .effect-creature .bfly-wing-r{animation:bflyR .9s ease-in-out infinite;transform-origin:21px 16px;}
    @keyframes bflyL{0%,100%{transform:scaleX(1)}50%{transform:scaleX(.25)}}
    @keyframes bflyR{0%,100%{transform:scaleX(1)}50%{transform:scaleX(.25)}}
    .garden-effect-wrap.fx-hover{animation:fxBob 2.5s ease-in-out infinite;}
    @keyframes fxBob{0%,100%{margin-top:0}50%{margin-top:-5px}}
    .effect-creature .fire-flame{animation:fireFlick .3s ease-in-out infinite alternate;transform-origin:15px 32px;}
    @keyframes fireFlick{0%{transform:scaleX(1) scaleY(1)}100%{transform:scaleX(.82) scaleY(1.06)}}
    .effect-creature .flood-rain{animation:rainFall .5s linear infinite;}
    @keyframes rainFall{0%{transform:translateY(0);opacity:.9}100%{transform:translateY(5px);opacity:.2}}
    .effect-creature .drought-pulse{animation:droughtPulse 2s ease-in-out infinite;transform-origin:18px 18px;}
    @keyframes droughtPulse{0%,100%{transform:scale(1);opacity:.85}50%{transform:scale(1.12);opacity:1}}
    .effect-creature .leaves-sway{animation:leavesSway 2.8s ease-in-out infinite;transform-origin:18px 18px;}
    @keyframes leavesSway{0%,100%{transform:rotate(0)}25%{transform:rotate(8deg)}50%{transform:rotate(-5deg)}75%{transform:rotate(6deg)}}
    .effect-creature .fw-burst{animation:fwBurst .8s ease-in-out infinite;transform-origin:20px 20px;}
    @keyframes fwBurst{0%{transform:scale(.7);opacity:.6}50%{transform:scale(1.15);opacity:1}100%{transform:scale(.7);opacity:.6}}
    .fx-particle{position:fixed;pointer-events:none;z-index:9998;animation:fxDrift var(--dur,1s) ease-out forwards;}
    .fx-leaf{width:6px;height:8px;background:#6aad40;border-radius:50% 50% 50% 0;opacity:.85;}
    .fx-pollen{width:5px;height:5px;border-radius:50%;background:#f0c830;box-shadow:0 0 3px rgba(240,200,48,.4);opacity:.9;}
    .fx-spark{width:4px;height:4px;border-radius:50%;background:#FF6B00;box-shadow:0 0 4px rgba(255,107,0,.6);opacity:.9;}
    .fx-droplet{width:4px;height:7px;background:#4A90B8;border-radius:50% 50% 50% 50%/60% 60% 40% 40%;opacity:.8;}
    .fx-crack{width:5px;height:5px;background:#A08060;border-radius:30%;opacity:.7;}
    .fx-flower{width:10px;height:10px;opacity:.9;}
    .fx-fwspark{width:4px;height:4px;border-radius:50%;opacity:1;}
    .fx-res-flower{position:absolute;width:22px;height:22px;pointer-events:none;z-index:3;opacity:0;transition:opacity .4s;overflow:visible;}
    .fx-res-flower.show{opacity:.85;}
    .fx-res-star{position:absolute;width:18px;height:18px;pointer-events:none;z-index:3;opacity:0;transition:opacity .4s;overflow:visible;}
    .fx-res-star svg{width:100%;height:100%;}
    .fx-res-star.show{opacity:.8;}
    @keyframes fxDrift{0%{opacity:var(--po,.85);transform:translate(0,0) scale(1) rotate(0deg)}100%{opacity:0;transform:translate(var(--pdx,10px),var(--pdy,-30px)) scale(.15) rotate(var(--pr,90deg))}}
    .svd-effect-bar{display:flex;gap:6px;justify-content:center;flex-wrap:wrap;}
    .svd-effect-btn{width:36px;height:36px;border-radius:10px;background:rgba(255,255,255,.5);border:2px solid var(--site-row-border, rgba(120,170,100,.3));cursor:pointer;display:flex;align-items:center;justify-content:center;transition:all .15s;}
    .svd-effect-btn:hover{background:rgba(255,255,255,.8);border-color:var(--site-accent, rgba(80,140,60,.5));}
    .svd-effect-btn.active{background:rgba(255,255,255,.85);border-color:var(--site-accent, #5a9a40);box-shadow:0 0 8px rgba(90,154,64,.25);}
    .svd-effect-btn svg{width:24px;height:24px;}
    /* ── Card residue marks ──────────────────────────────── */
    .fx-residue{position:absolute;pointer-events:none;z-index:20;opacity:0;transition:opacity .4s ease-in;}
    .fx-residue.show{opacity:1;transition:opacity .15s ease-out;}
    .fx-res-burn{width:22px;height:18px;border-radius:50%;background:radial-gradient(ellipse,rgba(30,10,0,.55) 0%,rgba(60,30,5,.3) 40%,rgba(80,40,10,.1) 70%,transparent 100%);filter:blur(.8px);}
    .fx-res-drip{width:10px;height:22px;background:linear-gradient(180deg,rgba(74,144,184,.45) 0%,rgba(74,144,184,.2) 60%,rgba(74,144,184,.05) 90%,transparent);border-radius:50% 50% 50% 50%/30% 30% 70% 70%;filter:blur(.4px);}
    .fx-res-pollen{width:10px;height:10px;border-radius:50%;background:radial-gradient(circle,rgba(240,200,48,.6) 0%,rgba(240,200,48,.2) 60%,transparent);box-shadow:0 0 4px rgba(240,200,48,.25);filter:blur(.3px);}
    .fx-res-bite{width:38px;height:32px;overflow:visible;position:absolute;}
    .fx-res-bite svg{width:100%;height:100%;}
    .fx-res-leaf{width:24px;height:24px;overflow:visible;position:absolute;}
    .fx-res-leaf svg{width:100%;height:100%;}
    .fx-res-crack{width:28px;height:28px;overflow:visible;position:absolute;}
    .fx-res-crack svg{width:100%;height:100%;}
    `;
    document.head.appendChild(s);
})();

// Creature SVGs (follow cursor on cards)
const _FX_CREATURES = {
    deer: `<svg viewBox="0 0 48 36" width="48" height="36" class="effect-creature"><path d="M30 6l3-5M31 3l3-1M30 6l4-3" stroke="#8B6914" stroke-width="1" fill="none" stroke-linecap="round"/><ellipse cx="18" cy="22" rx="13" ry="8" fill="#C4884D"/><g class="deer-head"><ellipse cx="33" cy="14" rx="7" ry="5" fill="#D4985A"/><ellipse cx="28" cy="9" rx="2" ry="3.5" fill="#D4985A" transform="rotate(-10 28 9)"/><circle cx="36" cy="12" r="1.3" fill="#2D1B00"/><circle cx="36.4" cy="11.6" r=".4" fill="#fff"/><ellipse cx="40" cy="15" rx="1.8" ry="1.2" fill="#6B3A1F"/></g><g stroke="#A07040" stroke-width="2" stroke-linecap="round"><line x1="9" y1="28" x2="8" y2="35"/><line x1="15" y1="29" x2="14" y2="35"/><line x1="23" y1="29" x2="24" y2="35"/><line x1="27" y1="28" x2="28" y2="35"/></g><ellipse cx="5" cy="18" rx="2.5" ry="1.5" fill="#E0CDB8"/></svg>`,
    rabbit: `<svg viewBox="0 0 42 34" width="42" height="34" class="effect-creature"><ellipse cx="22" cy="5" rx="2.5" ry="7" fill="#D4C4B0" transform="rotate(5 22 5)"/><ellipse cx="22.3" cy="5" rx="1.2" ry="5" fill="#F0C4C4" transform="rotate(5 22 5)"/><ellipse cx="27" cy="6" rx="2.3" ry="6.5" fill="#D4C4B0" transform="rotate(15 27 6)"/><ellipse cx="27.3" cy="6" rx="1" ry="4.5" fill="#F0C4C4" transform="rotate(15 27 6)"/><ellipse cx="15" cy="22" rx="12" ry="9" fill="#D4C4B0"/><g class="rabbit-head"><circle cx="30" cy="18" r="6.5" fill="#E0D4C4"/><circle cx="33" cy="16" r="1.5" fill="#2D1B00"/><circle cx="33.4" cy="15.6" r=".5" fill="#fff"/><ellipse cx="36" cy="18.5" rx="1.5" ry="1" fill="#FFB0B0"/><g stroke="#B0A090" stroke-width=".4"><line x1="35" y1="19.5" x2="41" y2="18"/><line x1="35" y1="20.5" x2="41" y2="20.5"/></g></g><ellipse cx="25" cy="29" rx="3" ry="1.5" fill="#E0D4C4"/><ellipse cx="7" cy="30" rx="3.5" ry="2" fill="#D4C4B0"/><circle cx="3" cy="19" r="3" fill="#F5F0EB"/></svg>`,
    bees: `<svg viewBox="0 0 34 26" width="34" height="26" class="effect-creature"><g class="bee-wings"><ellipse cx="12" cy="6" rx="6.5" ry="4" fill="rgba(200,220,255,.45)" stroke="rgba(150,180,220,.35)" stroke-width=".5" transform="rotate(-10 12 6)"/><ellipse cx="18" cy="5.5" rx="6" ry="3.5" fill="rgba(200,220,255,.45)" stroke="rgba(150,180,220,.35)" stroke-width=".5" transform="rotate(10 18 5.5)"/></g><ellipse cx="14" cy="14" rx="9" ry="6" fill="#F5C518"/><path d="M9 11.5h10M7.5 14h13M9 16.5h10" stroke="#2D2000" stroke-width="1.8" stroke-linecap="round"/><circle cx="24" cy="13" r="4" fill="#2D2000"/><circle cx="26" cy="11.5" r="1.5" fill="white"/><circle cx="26.3" cy="11.3" r=".7" fill="#111"/><path d="M25 9Q27 5 29 4M26 9Q29 6 31 6" stroke="#2D2000" stroke-width=".7" fill="none"/><circle cx="29" cy="4" r=".8" fill="#2D2000"/><circle cx="31" cy="6" r=".8" fill="#2D2000"/><path d="M5 14L2 14.5" stroke="#2D2000" stroke-width="1" stroke-linecap="round"/></svg>`,
    butterflies: `<svg viewBox="0 0 42 32" width="42" height="32" class="effect-creature"><ellipse cx="21" cy="16" rx="1.5" ry="8" fill="#4A3520"/><path d="M20 8Q17 3 14 2M22 8Q25 3 28 2" stroke="#4A3520" stroke-width=".7" fill="none"/><circle cx="14" cy="2" r="1" fill="#4A3520"/><circle cx="28" cy="2" r="1" fill="#4A3520"/><g class="bfly-wing-l"><ellipse cx="12" cy="11" rx="9" ry="7" fill="#FF7BA8" opacity=".82"/><ellipse cx="11" cy="11" rx="5" ry="4" fill="#FFB8D0" opacity=".6"/><ellipse cx="13" cy="22" rx="6" ry="4.5" fill="#E86898" opacity=".78"/></g><g class="bfly-wing-r"><ellipse cx="30" cy="11" rx="9" ry="7" fill="#FF7BA8" opacity=".82"/><ellipse cx="31" cy="11" rx="5" ry="4" fill="#FFB8D0" opacity=".6"/><ellipse cx="29" cy="22" rx="6" ry="4.5" fill="#E86898" opacity=".78"/></g></svg>`,
    drought: `<svg viewBox="0 0 36 36" width="36" height="36" class="effect-creature"><g class="drought-pulse"><g stroke="#E8A020" stroke-width="2" stroke-linecap="round" opacity=".7"><line x1="18" y1="2" x2="18" y2="7"/><line x1="18" y1="29" x2="18" y2="34"/><line x1="2" y1="18" x2="7" y2="18"/><line x1="29" y1="18" x2="34" y2="18"/><line x1="6.3" y1="6.3" x2="9.8" y2="9.8"/><line x1="26.2" y1="26.2" x2="29.7" y2="29.7"/><line x1="29.7" y1="6.3" x2="26.2" y2="9.8"/><line x1="6.3" y1="29.7" x2="9.8" y2="26.2"/></g><circle cx="18" cy="18" r="9" fill="#F0A830"/><circle cx="18" cy="18" r="6" fill="#F5C040" opacity=".6"/></g></svg>`,
    fire: `<svg viewBox="0 0 30 40" width="30" height="40" class="effect-creature"><g class="fire-flame"><path d="M15 2Q22 12 20 22Q18 30 15 32Q12 30 10 22Q8 12 15 2" fill="#FF6B00" opacity=".9"/><path d="M15 8Q19 14 18 22Q16 28 15 30Q14 28 12 22Q11 14 15 8" fill="#FFA000" opacity=".9"/><path d="M15 14Q17 18 16.5 24Q15.5 28 15 29Q14.5 28 13.5 24Q13 18 15 14" fill="#FFD54F"/></g><ellipse cx="10" cy="36" rx="8" ry="2.5" fill="#8B5E3C" transform="rotate(-10 10 36)"/><ellipse cx="20" cy="36" rx="8" ry="2.5" fill="#6D4C2A" transform="rotate(10 20 36)"/></svg>`,
    flood: `<svg viewBox="0 0 40 28" width="40" height="28" class="effect-creature"><ellipse cx="20" cy="9" rx="14" ry="7" fill="#7EAAC0"/><ellipse cx="11" cy="11" rx="8" ry="5.5" fill="#8EBCD0"/><ellipse cx="29" cy="11" rx="8" ry="5.5" fill="#8EBCD0"/><g class="flood-rain" stroke="#4A90B8" stroke-width="1.5" stroke-linecap="round"><line x1="11" y1="17" x2="9" y2="23"/><line x1="17" y1="16" x2="15" y2="22"/><line x1="23" y1="16" x2="21" y2="22"/><line x1="29" y1="17" x2="27" y2="23"/><line x1="14" y1="19" x2="12" y2="25"/><line x1="26" y1="19" x2="24" y2="25"/></g></svg>`,
    leaves: `<svg viewBox="0 0 44 36" width="44" height="36" class="effect-creature"><g class="leaves-sway"><path d="M14 8 Q22 2, 26 8 Q28 14, 22 16 Q18 12, 14 8 Z" fill="#6aad40" stroke="#4a8a28" stroke-width=".8"/><path d="M14 8 L22 14" stroke="#4a8a28" stroke-width=".5" opacity=".6"/><path d="M28 6 Q34 2, 36 8 Q36 14, 30 14 Q30 10, 28 6 Z" fill="#8bc34a" stroke="#5a9a30" stroke-width=".8" transform="rotate(15 32 10)"/><path d="M28 6 L32 12" stroke="#5a9a30" stroke-width=".5" opacity=".6"/><path d="M8 18 Q14 14, 18 18 Q20 24, 14 26 Q10 22, 8 18 Z" fill="#c0a020" stroke="#9a8018" stroke-width=".8"/><path d="M8 18 L15 24" stroke="#9a8018" stroke-width=".5" opacity=".6"/><path d="M22 20 Q28 16, 32 20 Q34 26, 28 28 Q24 24, 22 20 Z" fill="#e07830" stroke="#b06020" stroke-width=".8"/><path d="M22 20 L29 26" stroke="#b06020" stroke-width=".5" opacity=".6"/><path d="M18 28 Q22 24, 26 28 Q28 32, 24 34 Q20 32, 18 28 Z" fill="#c05030" stroke="#903820" stroke-width=".8"/><path d="M18 28 L24 32" stroke="#903820" stroke-width=".5" opacity=".6"/></g></svg>`,
    fireworks: `<svg viewBox="0 0 40 40" width="40" height="40" class="effect-creature"><g class="fw-burst"><line x1="20" y1="20" x2="20" y2="4" stroke="#FF3030" stroke-width="1.5" stroke-linecap="round"/><line x1="20" y1="20" x2="31" y2="8" stroke="#FFD700" stroke-width="1.5" stroke-linecap="round"/><line x1="20" y1="20" x2="36" y2="20" stroke="#00BFFF" stroke-width="1.5" stroke-linecap="round"/><line x1="20" y1="20" x2="31" y2="32" stroke="#FF69B4" stroke-width="1.5" stroke-linecap="round"/><line x1="20" y1="20" x2="20" y2="36" stroke="#7CFC00" stroke-width="1.5" stroke-linecap="round"/><line x1="20" y1="20" x2="9" y2="32" stroke="#FF8C00" stroke-width="1.5" stroke-linecap="round"/><line x1="20" y1="20" x2="4" y2="20" stroke="#BA55D3" stroke-width="1.5" stroke-linecap="round"/><line x1="20" y1="20" x2="9" y2="8" stroke="#00FF7F" stroke-width="1.5" stroke-linecap="round"/><circle cx="20" cy="4" r="2" fill="#FF3030"/><circle cx="31" cy="8" r="2" fill="#FFD700"/><circle cx="36" cy="20" r="2" fill="#00BFFF"/><circle cx="31" cy="32" r="2" fill="#FF69B4"/><circle cx="20" cy="36" r="2" fill="#7CFC00"/><circle cx="9" cy="32" r="2" fill="#FF8C00"/><circle cx="4" cy="20" r="2" fill="#BA55D3"/><circle cx="9" cy="8" r="2" fill="#00FF7F"/><circle cx="20" cy="20" r="3" fill="#fff" opacity=".9"/></g></svg>`
};

// Picker icon SVGs (static thumbnails)
const _FX_ICONS = {
    none: `<svg viewBox="0 0 20 20"><circle cx="10" cy="10" r="7" fill="none" stroke="#999" stroke-width="1.5"/><line x1="5" y1="15" x2="15" y2="5" stroke="#999" stroke-width="1.5"/></svg>`,
    deer: `<svg viewBox="0 0 20 20"><path d="M13 4l2-3.5M13 4l3-2" stroke="#8B6914" stroke-width=".8" fill="none" stroke-linecap="round"/><ellipse cx="8" cy="12" rx="6" ry="4" fill="#C4884D"/><circle cx="14" cy="7.5" r="3" fill="#D4985A"/><circle cx="15.5" cy="6.5" r=".8" fill="#2D1B00"/><g stroke="#A07040" stroke-width="1.2" stroke-linecap="round"><line x1="4" y1="15" x2="3.5" y2="19"/><line x1="7" y1="15.5" x2="6.5" y2="19"/><line x1="10" y1="15.5" x2="10.5" y2="19"/><line x1="12" y1="15" x2="12.5" y2="19"/></g></svg>`,
    rabbit: `<svg viewBox="0 0 20 20"><ellipse cx="11" cy="4" rx="1.5" ry="4" fill="#D4C4B0" transform="rotate(5 11 4)"/><ellipse cx="14" cy="4.5" rx="1.3" ry="3.5" fill="#D4C4B0" transform="rotate(15 14 4.5)"/><ellipse cx="8" cy="13" rx="5.5" ry="5" fill="#D4C4B0"/><circle cx="14" cy="10.5" r="3.2" fill="#E0D4C4"/><circle cx="15.5" cy="9.5" r=".8" fill="#2D1B00"/><circle cx="3" cy="12" r="2" fill="#F5F0EB"/></svg>`,
    bees: `<svg viewBox="0 0 20 20"><ellipse cx="7" cy="5" rx="4" ry="2.5" fill="rgba(200,220,255,.5)"/><ellipse cx="11" cy="4.5" rx="3.5" ry="2" fill="rgba(200,220,255,.5)"/><ellipse cx="9" cy="11" rx="5.5" ry="4" fill="#F5C518"/><path d="M6 9.5h6.5M5 11.5h8.5M6 13.5h6.5" stroke="#2D2000" stroke-width="1" stroke-linecap="round"/><circle cx="15.5" cy="10.5" r="2.5" fill="#2D2000"/><circle cx="17" cy="9.5" r="1" fill="white"/></svg>`,
    butterflies: `<svg viewBox="0 0 20 20"><ellipse cx="10" cy="10" rx="1" ry="5" fill="#4A3520"/><ellipse cx="5" cy="8" rx="4.5" ry="4" fill="#FF7BA8" opacity=".8"/><ellipse cx="15" cy="8" rx="4.5" ry="4" fill="#FF7BA8" opacity=".8"/><ellipse cx="6" cy="14" rx="3" ry="2.5" fill="#E86898" opacity=".75"/><ellipse cx="14" cy="14" rx="3" ry="2.5" fill="#E86898" opacity=".75"/><path d="M9 5Q7 2 5 1M11 5Q13 2 15 1" stroke="#4A3520" stroke-width=".6" fill="none"/></svg>`,
    drought: `<svg viewBox="0 0 20 20"><g stroke="#E8A020" stroke-width="1.5" stroke-linecap="round" opacity=".7"><line x1="10" y1="1" x2="10" y2="4"/><line x1="10" y1="16" x2="10" y2="19"/><line x1="1" y1="10" x2="4" y2="10"/><line x1="16" y1="10" x2="19" y2="10"/><line x1="3.5" y1="3.5" x2="5.5" y2="5.5"/><line x1="14.5" y1="14.5" x2="16.5" y2="16.5"/><line x1="16.5" y1="3.5" x2="14.5" y2="5.5"/><line x1="3.5" y1="16.5" x2="5.5" y2="14.5"/></g><circle cx="10" cy="10" r="5" fill="#F0A830"/></svg>`,
    fire: `<svg viewBox="0 0 20 20"><path d="M10 2Q14 7 13 13Q11 17 10 18Q9 17 7 13Q6 7 10 2" fill="#FF6B00" opacity=".9"/><path d="M10 6Q12 9 11.5 13Q10.5 16 10 17Q9.5 16 8.5 13Q8 9 10 6" fill="#FFA000"/><path d="M10 9Q11 11 10.5 14Q10 16 10 16.5Q10 16 9.5 14Q9 11 10 9" fill="#FFD54F"/></svg>`,
    flood: `<svg viewBox="0 0 20 20"><ellipse cx="10" cy="6" rx="7" ry="4" fill="#7EAAC0"/><ellipse cx="6" cy="7" rx="4" ry="3" fill="#8EBCD0"/><ellipse cx="14" cy="7" rx="4" ry="3" fill="#8EBCD0"/><g stroke="#4A90B8" stroke-width="1" stroke-linecap="round"><line x1="6" y1="11" x2="5" y2="14"/><line x1="10" y1="10" x2="9" y2="13"/><line x1="14" y1="11" x2="13" y2="14"/><line x1="8" y1="13" x2="7" y2="16"/><line x1="12" y1="13" x2="11" y2="16"/></g></svg>`,
    leaves: `<svg viewBox="0 0 20 20"><path d="M6 5 Q10 2, 13 5 Q14 9, 10 10 Q8 8, 6 5 Z" fill="#6aad40" stroke="#4a8a28" stroke-width=".6"/><path d="M10 8 Q14 6, 16 9 Q16 13, 12 13 Q12 10, 10 8 Z" fill="#c0a020" stroke="#9a8018" stroke-width=".6"/><path d="M5 12 Q9 10, 11 13 Q12 16, 8 17 Q6 14, 5 12 Z" fill="#e07830" stroke="#b06020" stroke-width=".6"/><path d="M12 14 Q15 12, 16 15 Q16 17, 14 18 Q12 16, 12 14 Z" fill="#c05030" stroke="#903820" stroke-width=".6"/></svg>`,
    fireworks: `<svg viewBox="0 0 20 20"><line x1="10" y1="10" x2="10" y2="2" stroke="#FF3030" stroke-width="1" stroke-linecap="round"/><line x1="10" y1="10" x2="16" y2="4" stroke="#FFD700" stroke-width="1" stroke-linecap="round"/><line x1="10" y1="10" x2="18" y2="10" stroke="#00BFFF" stroke-width="1" stroke-linecap="round"/><line x1="10" y1="10" x2="16" y2="16" stroke="#FF69B4" stroke-width="1" stroke-linecap="round"/><line x1="10" y1="10" x2="10" y2="18" stroke="#7CFC00" stroke-width="1" stroke-linecap="round"/><line x1="10" y1="10" x2="4" y2="16" stroke="#FF8C00" stroke-width="1" stroke-linecap="round"/><line x1="10" y1="10" x2="2" y2="10" stroke="#BA55D3" stroke-width="1" stroke-linecap="round"/><line x1="10" y1="10" x2="4" y2="4" stroke="#00FF7F" stroke-width="1" stroke-linecap="round"/><circle cx="10" cy="2" r="1.2" fill="#FF3030"/><circle cx="16" cy="4" r="1.2" fill="#FFD700"/><circle cx="18" cy="10" r="1.2" fill="#00BFFF"/><circle cx="10" cy="10" r="1.5" fill="#fff"/></svg>`
};

function _spawnFxParticle(x, y, type) {
    const p = document.createElement('div');
    const classMap = {leaf:'fx-leaf', pollen:'fx-pollen', spark:'fx-spark', droplet:'fx-droplet', crack:'fx-crack', flower:'fx-flower', fwspark:'fx-fwspark'};
    p.className = 'fx-particle ' + (classMap[type] || 'fx-pollen');
    let dx, dy;
    if (type === 'fwspark') { const a = Math.random()*Math.PI*2, r = 30+Math.random()*50; dx = Math.cos(a)*r; dy = Math.sin(a)*r; }
    else if (type === 'spark') { dx = (Math.random()-.5)*40; dy = -20-Math.random()*40; }
    else if (type === 'droplet') { dx = (Math.random()-.5)*30; dy = 10+Math.random()*40; }
    else if (type === 'crack') { dx = (Math.random()-.5)*35; dy = 5+Math.random()*25; }
    else if (type === 'leaf') { dx = (Math.random()-.5)*50; dy = -15-Math.random()*35; }
    else if (type === 'flower') { dx = (Math.random()-.5)*45; dy = -8-Math.random()*25; }
    else { dx = (Math.random()-.5)*50; dy = -10-Math.random()*30; }
    const rot = (Math.random()*360)|0, dur = type === 'fwspark' ? (.5+Math.random()*.5).toFixed(2) : (.7+Math.random()*.6).toFixed(2);
    const op = type === 'spark' ? .95 : type === 'droplet' ? .8 : type === 'crack' ? .7 : type === 'leaf' ? .85 : type === 'fwspark' ? 1 : .9;
    p.style.cssText = `left:${x}px;top:${y}px;--pdx:${dx}px;--pdy:${dy}px;--pr:${rot}deg;--dur:${dur}s;--po:${op}`;
    if (type === 'flower') {
        const cols = [['#FFB6C1','#FF69B4'],['#FFD700','#DAA520'],['#9370DB','#6A0DAD'],['#FF6347','#DC143C'],['#87CEEB','#4169E1'],['#FFA500','#E08000']];
        const c = cols[(Math.random()*cols.length)|0];
        p.innerHTML = `<svg viewBox="0 0 12 12" width="10" height="10">${[0,72,144,216,288].map(d=>`<ellipse cx="6" cy="2.5" rx="1.8" ry="3" fill="${c[0]}" transform="rotate(${d},6,6)"/>`).join('')}<circle cx="6" cy="6" r="1.5" fill="${c[1]}"/></svg>`;
    }
    if (type === 'fwspark') {
        const fwc = ['#FF3030','#FFD700','#00BFFF','#FF69B4','#7CFC00','#FF8C00','#BA55D3','#00FF7F','#fff'][(Math.random()*9)|0];
        p.style.background = fwc;
        p.style.boxShadow = `0 0 6px ${fwc}, 0 0 12px ${fwc}`;
    }
    document.body.appendChild(p);
    p.addEventListener('animationend', () => p.remove(), {once:true});
}

// ── Card residue marks (persist 20s then fade) ─────────────────────
let _lastResidueTime = 0;
const _RESIDUE_LIFETIME = 20000; // 20 seconds
const _residueTimers = new WeakMap(); // card → timeout id
const _RESIDUE_CFG = {
    fire:        { cls:'fx-res-burn',   interval:100, max:35, jitter:16 },
    flood:       { cls:'fx-res-drip',   interval:80,  max:40, jitter:12 },
    bees:        { cls:'fx-res-pollen', interval:130, max:28, jitter:14 },
    butterflies: { cls:'fx-res-flower', interval:160, max:24, jitter:16 },
    deer:        { cls:'fx-res-bite',   interval:180, max:24, jitter:22 },
    rabbit:      { cls:'fx-res-bite',   interval:200, max:22, jitter:20 },
    drought:     { cls:'fx-res-crack',  interval:250, max:18, jitter:16 },
    leaves:      { cls:'fx-res-leaf',   interval:120, max:35, jitter:18 },
    fireworks:   { cls:'fx-res-star',   interval:90,  max:40, jitter:20 }
};

// Random SVG crack path for drought residue
function _crackSVG() {
    const paths = [
        'M2 14 L8 8 L6 4 L10 1 M8 8 L14 10 L18 6 M8 8 L10 14 L14 18',
        'M1 10 L6 6 L12 8 L16 3 M6 6 L4 2 M12 8 L15 14 L18 16',
        'M3 16 L7 10 L5 6 L9 2 M7 10 L13 12 L17 8 L20 10 M13 12 L12 17',
        'M1 12 L5 7 L3 3 M5 7 L10 5 L14 8 L18 4 M10 5 L9 12 L12 16'
    ];
    const d = paths[(Math.random() * paths.length) | 0];
    return `<svg viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path d="${d}" stroke="rgba(90,60,25,.55)" stroke-width="1.6" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>`;
}

// Random SVG bite marks for deer/rabbit — big chunky chomps
function _biteSVG() {
    const bites = [
        // Half-circle bite taken out of edge
        `<path d="M4 2 C8 0, 14 0, 18 2 L20 8 C22 14, 22 20, 18 26 L14 28 C10 24, 8 18, 10 12 C12 8, 8 6, 4 4 Z" fill="rgba(60,40,15,.18)" stroke="rgba(60,40,15,.45)" stroke-width="1.2"/>
         <path d="M10 12 C12 8, 8 6, 4 4" stroke="rgba(80,50,20,.35)" stroke-width=".8" fill="none" stroke-dasharray="2 2"/>`,
        // Jagged tear
        `<path d="M2 4 L8 2 L12 6 L18 3 L22 8 L20 14 L22 20 L18 24 L12 20 L8 24 L4 18 L6 12 Z" fill="rgba(60,40,15,.15)" stroke="rgba(60,40,15,.4)" stroke-width="1.1" stroke-linejoin="round"/>
         <path d="M8 10 L12 8 L14 12 M10 16 L14 14" stroke="rgba(80,50,20,.3)" stroke-width=".7" fill="none"/>`,
        // Crescent chomp
        `<path d="M6 2 Q16 0, 22 6 Q26 14, 22 22 Q16 28, 6 26 Q10 20, 10 14 Q10 8, 6 2 Z" fill="rgba(60,40,15,.16)" stroke="rgba(60,40,15,.42)" stroke-width="1.2"/>
         <path d="M10 8 Q12 12, 10 18" stroke="rgba(90,60,25,.3)" stroke-width=".8" fill="none" stroke-dasharray="1.5 2.5"/>`,
        // Torn chunk with teeth marks
        `<path d="M3 6 L10 2 L16 4 L20 2 L24 8 L22 16 L24 22 L20 26 L14 22 L8 26 L4 20 L6 14 Z" fill="rgba(60,40,15,.14)" stroke="rgba(60,40,15,.38)" stroke-width="1"/>
         <circle cx="10" cy="10" r="1.2" fill="rgba(60,40,15,.25)"/><circle cx="16" cy="8" r="1" fill="rgba(60,40,15,.25)"/><circle cx="14" cy="16" r="1.1" fill="rgba(60,40,15,.25)"/>`
    ];
    const b = bites[(Math.random() * bites.length) | 0];
    return `<svg viewBox="0 0 28 28" xmlns="http://www.w3.org/2000/svg">${b}</svg>`;
}

// Random SVG leaves for leaves effect
function _leafSVG() {
    const colors = [
        { fill: 'rgba(106,173,64,.7)',  stroke: 'rgba(60,120,30,.5)' },   // green
        { fill: 'rgba(180,200,60,.65)', stroke: 'rgba(120,140,30,.45)' },  // yellow-green
        { fill: 'rgba(200,160,40,.6)',  stroke: 'rgba(150,110,20,.45)' },  // golden
        { fill: 'rgba(190,80,30,.55)',  stroke: 'rgba(140,50,15,.45)' },   // orange
        { fill: 'rgba(160,50,30,.5)',   stroke: 'rgba(120,30,15,.4)' },    // red-brown
        { fill: 'rgba(100,160,60,.65)', stroke: 'rgba(60,110,30,.5)' }     // forest
    ];
    const c = colors[(Math.random() * colors.length) | 0];
    const shapes = [
        // Simple oval leaf
        `<path d="M12 2 Q20 8, 18 16 Q14 22, 12 22 Q10 22, 6 16 Q4 8, 12 2 Z" fill="${c.fill}" stroke="${c.stroke}" stroke-width=".8"/>
         <path d="M12 4 L12 20 M9 8 L12 10 M15 8 L12 10 M8 14 L12 16 M16 14 L12 16" stroke="${c.stroke}" stroke-width=".5" fill="none" opacity=".6"/>`,
        // Maple-ish leaf
        `<path d="M12 1 L14 6 L19 4 L16 9 L22 10 L16 13 L18 18 L13 15 L12 22 L11 15 L6 18 L8 13 L2 10 L8 9 L5 4 L10 6 Z" fill="${c.fill}" stroke="${c.stroke}" stroke-width=".7"/>
         <path d="M12 4 L12 19" stroke="${c.stroke}" stroke-width=".5" fill="none" opacity=".5"/>`,
        // Rounded leaf
        `<path d="M12 3 Q18 6, 19 12 Q20 18, 14 21 L12 22 L10 21 Q4 18, 5 12 Q6 6, 12 3 Z" fill="${c.fill}" stroke="${c.stroke}" stroke-width=".8"/>
         <path d="M12 5 L12 20 M8 10 L12 12 M16 10 L12 12" stroke="${c.stroke}" stroke-width=".5" fill="none" opacity=".5"/>`,
        // Elongated willow leaf
        `<path d="M12 1 Q16 6, 16 12 Q16 18, 12 23 Q8 18, 8 12 Q8 6, 12 1 Z" fill="${c.fill}" stroke="${c.stroke}" stroke-width=".7"/>
         <path d="M12 3 L12 21" stroke="${c.stroke}" stroke-width=".5" fill="none" opacity=".6"/>`
    ];
    const s = shapes[(Math.random() * shapes.length) | 0];
    return `<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">${s}</svg>`;
}

// Random tiny flower SVG for butterfly residue
function _flowerSVG() {
    const flowers = [
        // Daisy — white petals, yellow center
        () => {
            const a = '#fff', b = '#FFD700';
            return `<g>${[0,45,90,135,180,225,270,315].map(d=>`<ellipse cx="12" cy="5" rx="2.8" ry="5" fill="${a}" stroke="#ddd" stroke-width=".3" transform="rotate(${d},12,12)"/>`).join('')}<circle cx="12" cy="12" r="3" fill="${b}" stroke="#DAA520" stroke-width=".4"/></g>`;
        },
        // Rose — pink petals spiral
        () => {
            const p = ['#FFB6C1','#FF69B4','#FF1493'];
            return `<circle cx="12" cy="12" r="9" fill="${p[0]}" stroke="${p[2]}" stroke-width=".5"/><circle cx="12" cy="12" r="6" fill="${p[1]}" stroke="${p[2]}" stroke-width=".4"/><circle cx="12" cy="12" r="3" fill="${p[2]}"/>`;
        },
        // Sunflower — orange petals, brown center
        () => {
            return `<g>${[0,30,60,90,120,150,180,210,240,270,300,330].map(d=>`<ellipse cx="12" cy="4" rx="2" ry="4.5" fill="#FFA500" stroke="#E08000" stroke-width=".3" transform="rotate(${d},12,12)"/>`).join('')}<circle cx="12" cy="12" r="3.5" fill="#8B4513" stroke="#654321" stroke-width=".4"/></g>`;
        },
        // Violet — purple 5-petal
        () => {
            const c = '#9370DB';
            return `<g>${[0,72,144,216,288].map(d=>`<ellipse cx="12" cy="5" rx="3" ry="5.5" fill="${c}" stroke="#6A0DAD" stroke-width=".4" transform="rotate(${d},12,12)"/>`).join('')}<circle cx="12" cy="12" r="2.5" fill="#FFE4B5" stroke="#DAA520" stroke-width=".3"/></g>`;
        },
        // Tulip — red cup shape
        () => {
            return `<path d="M12 3 Q18 6, 17 14 Q15 18, 12 18 Q9 18, 7 14 Q6 6, 12 3 Z" fill="#DC143C" stroke="#8B0000" stroke-width=".5"/><path d="M10 5 Q12 8, 14 5" fill="#FF6347" opacity=".6"/><line x1="12" y1="18" x2="12" y2="23" stroke="#228B22" stroke-width="1.2"/>`;
        },
        // Bluebell
        () => {
            return `<path d="M8 6 Q12 2, 16 6 Q18 10, 16 14 Q14 17, 12 17 Q10 17, 8 14 Q6 10, 8 6 Z" fill="#6495ED" stroke="#4169E1" stroke-width=".5"/><path d="M10 8 Q12 6, 14 8" fill="#87CEEB" opacity=".5"/><line x1="12" y1="17" x2="12" y2="23" stroke="#228B22" stroke-width="1"/>`;
        }
    ];
    const gen = flowers[(Math.random() * flowers.length) | 0];
    return `<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">${gen()}</svg>`;
}

// Random sparkle/star SVG for fireworks residue
function _starSVG() {
    const cols = ['#FF3030','#FFD700','#00BFFF','#FF69B4','#7CFC00','#FF8C00','#BA55D3','#00FF7F'];
    const c = cols[(Math.random()*cols.length)|0];
    const pts = 4 + ((Math.random()*3)|0); // 4–6 points
    let d = '';
    for (let i = 0; i < pts*2; i++) {
        const a = (Math.PI*2/(pts*2))*i - Math.PI/2;
        const r = i%2===0 ? 10 : 4;
        d += (i?'L':'M') + (12+Math.cos(a)*r).toFixed(1) + ' ' + (12+Math.sin(a)*r).toFixed(1);
    }
    return `<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="${d}Z" fill="${c}" opacity=".7" stroke="${c}" stroke-width=".3"/></svg>`;
}

function _spawnResidue(x, y) {
    const cfg = _RESIDUE_CFG[_activeEffect];
    if (!cfg) return;
    const now = performance.now();
    if (now - _lastResidueTime < cfg.interval) return;
    _lastResidueTime = now;
    const el = document.elementFromPoint(x, y);
    const card = el && el.closest('.sv-grid-card');
    if (!card) return;
    const inner = card.querySelector('.svd-card') || card.querySelector('[data-card-bg]') || card;
    const rect = inner.getBoundingClientRect();
    const rx = x - rect.left + (Math.random() - .5) * cfg.jitter;
    const ry = y - rect.top  + (Math.random() - .5) * cfg.jitter;
    // Cap residue count per card
    const existing = inner.querySelectorAll('.fx-residue');
    if (existing.length >= cfg.max) existing[0].remove();
    const mark = document.createElement('div');
    mark.className = 'fx-residue ' + cfg.cls;
    if (cfg.cls === 'fx-res-crack') mark.innerHTML = _crackSVG();
    if (cfg.cls === 'fx-res-bite') mark.innerHTML = _biteSVG();
    if (cfg.cls === 'fx-res-leaf') mark.innerHTML = _leafSVG();
    if (cfg.cls === 'fx-res-flower') mark.innerHTML = _flowerSVG();
    if (cfg.cls === 'fx-res-star') mark.innerHTML = _starSVG();
    const rot = (Math.random() * 60 - 30) | 0;
    const sc  = (.7 + Math.random() * .6).toFixed(2);
    mark.style.cssText = `left:${rx}px;top:${ry}px;transform:translate(-50%,-50%) rotate(${rot}deg) scale(${sc});`;
    inner.appendChild(mark);
    requestAnimationFrame(() => mark.classList.add('show'));
    // Reset the 20s expiry timer for this card (resets each new mark)
    _resetResidueTimer(card);
}

function _resetResidueTimer(card) {
    const prev = _residueTimers.get(card);
    if (prev) clearTimeout(prev);
    _residueTimers.set(card, setTimeout(() => {
        _fadeResidueFrom(card);
        _residueTimers.delete(card);
    }, _RESIDUE_LIFETIME));
}

function _fadeResidueFrom(card) {
    if (!card) return;
    card.querySelectorAll('.fx-residue').forEach(m => {
        m.classList.remove('show');
        setTimeout(() => m.remove(), 400);
    });
}

function _clearAllResidue() {
    document.querySelectorAll('.fx-residue').forEach(m => {
        m.classList.remove('show');
        setTimeout(() => m.remove(), 400);
    });
}

function _showFx(x, y) {
    if (!_effectWrap || _activeEffect === 'none') return;
    _effectWrap.classList.remove('hidden');
    // Offset creature up-left/up-right of finger so it's visible
    const offX = _effectDir < 0 ? -48 : 8;
    const offY = -52;
    _effectWrap.style.left = (x + offX) + 'px';
    _effectWrap.style.top  = (y + offY) + 'px';
    const svg = _effectWrap.firstElementChild;
    if (svg) svg.style.transform = _effectDir < 0 ? 'scaleX(-1)' : '';
    _effectPos = {x, y};
    // Leave residue mark on the card
    _spawnResidue(x, y);
}

function _hideFx() {
    if (_effectWrap) _effectWrap.classList.add('hidden');
    if (_particleTimer) { clearInterval(_particleTimer); _particleTimer = null; }
    // Don't clear residue on leave — it persists for 20s on its own timer
    _currentFxCard = null;
}

function _startFxParticles() {
    if (_particleTimer) return;
    const cfg = {deer:{type:'leaf',n:1,ms:400}, rabbit:{type:'leaf',n:1,ms:400},
        bees:{type:'pollen',n:2,ms:250}, butterflies:{type:'flower',n:2,ms:300},
        fire:{type:'spark',n:2,ms:200}, flood:{type:'droplet',n:2,ms:180},
        drought:{type:'crack',n:1,ms:500}, leaves:{type:'leaf',n:2,ms:300},
        fireworks:{type:'fwspark',n:4,ms:120}}[_activeEffect];
    if (!cfg) return;
    _particleTimer = setInterval(() => {
        if (_effectWrap && !_effectWrap.classList.contains('hidden')) {
            for (let i = 0; i < cfg.n; i++)
                _spawnFxParticle(_effectPos.x+(Math.random()-.5)*10, _effectPos.y+(Math.random()-.5)*10, cfg.type);
        }
    }, cfg.ms);
}

function applyCardEffect(name) {
    _activeEffect = name || 'none';
    if (!_effectWrap) {
        _effectWrap = document.createElement('div');
        _effectWrap.className = 'garden-effect-wrap hidden';
        document.body.appendChild(_effectWrap);
    }
    _hideFx();
    _clearAllResidue(); // wipe old residue when switching effects
    _effectWrap.innerHTML = name !== 'none' ? (_FX_CREATURES[name] || '') : '';
    _effectWrap.classList.toggle('fx-hover', name === 'bees' || name === 'butterflies' || name === 'drought' || name === 'fire' || name === 'flood' || name === 'leaves' || name === 'fireworks');
    document.querySelectorAll('.svd-effect-btn').forEach(b =>
        b.classList.toggle('active', b.dataset.effect === name));
    try { localStorage.setItem(EFFECT_KEY, name); } catch(e){}
    if (window.GARDEN_CONFIG && window.GARDEN_CONFIG.currentUser) {
        fetch('plant_actions.php', { method:'POST', headers:{'Content-Type':'application/x-www-form-urlencoded'},
            body: new URLSearchParams({ action:'save_pref', key:'card_effect', value: name }) }).catch(()=>{});
    }
}

// Mouse tracking (delegated)
document.addEventListener('mousemove', function(e) {
    if (_activeEffect === 'none') return;
    const card = e.target.closest('.sv-grid-card');
    if (card) {
        // Switched to a different card — clear old residue
        if (_currentFxCard && _currentFxCard !== card) _clearResidueFrom(_currentFxCard);
        _currentFxCard = card;
        if (Math.abs(e.clientX - _prevEX) > 3) _effectDir = e.clientX > _prevEX ? 1 : -1;
        _prevEX = e.clientX;
        _showFx(e.clientX, e.clientY);
        _startFxParticles();
    } else { _hideFx(); }
});
document.addEventListener('mouseleave', _hideFx);

// Touch tracking
document.addEventListener('touchmove', function(e) {
    if (_activeEffect === 'none') return;
    const t = e.touches[0]; if (!t) return;
    const el = document.elementFromPoint(t.clientX, t.clientY);
    const card = el && el.closest('.sv-grid-card');
    if (card) {
        if (_currentFxCard && _currentFxCard !== card) _clearResidueFrom(_currentFxCard);
        _currentFxCard = card;
        if (Math.abs(t.clientX - _prevEX) > 3) _effectDir = t.clientX > _prevEX ? 1 : -1;
        _prevEX = t.clientX;
        _showFx(t.clientX, t.clientY);
        _startFxParticles();
    } else { _hideFx(); }
}, {passive:true});
document.addEventListener('touchend', _hideFx);
document.addEventListener('touchcancel', _hideFx);

function buildEffectPicker() {
    const wrap = document.createElement('div');
    wrap.style.cssText = 'margin-top:8px;border-top:1px solid var(--site-row-border, rgba(120,170,100,0.35));padding-top:8px;';
    const lbl = document.createElement('div');
    lbl.style.cssText = 'font-size:.78rem;font-weight:900;color:var(--site-accent, #3a5a30);letter-spacing:.6px;text-transform:uppercase;margin-bottom:4px;';
    lbl.textContent = 'Effect';
    wrap.appendChild(lbl);
    const bar = document.createElement('div');
    bar.className = 'svd-effect-bar';
    ['none','deer','rabbit','bees','butterflies','drought','fire','flood','leaves','fireworks'].forEach(key => {
        const btn = document.createElement('div');
        btn.className = 'svd-effect-btn';
        btn.dataset.effect = key;
        btn.title = key.charAt(0).toUpperCase() + key.slice(1);
        btn.innerHTML = _FX_ICONS[key];
        btn.onclick = () => applyCardEffect(key);
        bar.appendChild(btn);
    });
    wrap.appendChild(bar);
    return wrap;
}
window.applyCardEffect = applyCardEffect;

// Inject picker when simple view is active (called from view toggle)
function injectThemePicker() {
    if (document.getElementById('svd-theme-picker-wrap')) return;
    const feed = document.getElementById('plant-feed');
    if (!feed) return;
    const wrap = document.createElement('div');
    wrap.id = 'svd-theme-picker-wrap';
    wrap.style.cssText = 'padding:0 4px 12px;text-align:center;';
    // Labeled box
    const box = document.createElement('div');
    box.style.cssText = 'display:inline-block;background:var(--site-tint, rgba(180,210,160,0.55));border:2px solid var(--site-row-border, rgba(120,170,100,0.6));border-radius:16px;padding:8px 24px 12px;margin:0 auto;';
    const lbl = document.createElement('div');
    lbl.style.cssText = 'font-size:.85rem;font-weight:900;color:var(--site-accent, #3a5a30);letter-spacing:.8px;text-transform:uppercase;margin-bottom:6px;';
    lbl.textContent = 'Card Theme';
    box.appendChild(lbl);
    box.appendChild(buildThemePicker());
    box.appendChild(buildLayoutPicker());
    box.appendChild(buildEffectPicker());
    wrap.appendChild(box);
    feed.parentNode.insertBefore(wrap, feed);
    // Load theme + layout + effect: try server-side pref first, then localStorage
    const localTheme = localStorage.getItem(SVD_THEME_KEY) || 'spring_d';
    applySvdTheme(localTheme); // apply immediately to avoid flash
    const localGrid = parseInt(localStorage.getItem(GRID_KEY), 10);
    if (localGrid >= 1 && localGrid <= 4) applyGridLayout(localGrid);
    applyCardEffect(localStorage.getItem(EFFECT_KEY) || 'none');
    if (window.GARDEN_CONFIG && window.GARDEN_CONFIG.currentUser) {
        fetch('plant_actions.php?action=load_prefs')
            .then(r => r.json())
            .then(d => {
                if (d.prefs) {
                    if (d.prefs.tag_theme && SVD_THEMES[d.prefs.tag_theme]) applySvdTheme(d.prefs.tag_theme);
                    const sg = parseInt(d.prefs.grid_layout, 10);
                    if (sg >= 1 && sg <= 4) applyGridLayout(sg);
                    if (d.prefs.card_effect) applyCardEffect(d.prefs.card_effect);
                }
            }).catch(() => {});
    }
}
function removeThemePicker() {
    const el = document.getElementById('svd-theme-picker-wrap');
    if (el) el.remove();
}
window.applySvdTheme = applySvdTheme;
window.injectThemePicker = injectThemePicker;
window.removeThemePicker = removeThemePicker;
try { applySvdTheme(localStorage.getItem(SVD_THEME_KEY) || 'spring_d'); } catch(e){}

// Equalize Simple Tags cards to the tallest one (pads shorter cards with space,
// never clips). Skipped on narrow phones where cards stack in a single column.
let _svEqTimer;
function equalizeSimpleCards() {
    if (!feedContainer || !isSimpleView()) return;
    const cards = feedContainer.querySelectorAll('.sv-grid-card');
    if (!cards.length) return;
    cards.forEach(c => { c.style.height = 'auto'; });
    if (window.innerWidth <= 560) return;            // stacked layout: natural heights
    let max = 0;
    cards.forEach(c => { const h = c.offsetHeight; if (h > max) max = h; });
    if (max > 0) cards.forEach(c => { c.style.height = max + 'px'; });
}
window.addEventListener('resize', () => {
    clearTimeout(_svEqTimer);
    _svEqTimer = setTimeout(equalizeSimpleCards, 150);
});

// ── Planting-guide TAG (garden green) — opens from a Tags-view card ─────────
function injectTagGuideStyles() {
    if (document.getElementById('tag-guide-styles')) return;
    const st = document.createElement('style');
    st.id = 'tag-guide-styles';
    st.textContent = `
    .tg-back{position:fixed;inset:0;background:rgba(20,40,25,.5);backdrop-filter:blur(3px);
        display:flex;align-items:center;justify-content:center;z-index:100000;padding:20px;}
    .tg-card{position:relative;background:var(--svd-body,#f1f8e9);border:3px solid var(--svd-row-border,#a5d6a7);border-radius:22px;width:96vw;
        max-width:1080px;max-height:92vh;overflow-y:auto;overflow-x:hidden;box-shadow:0 24px 70px rgba(20,50,25,.45);}
    .tg-x{position:absolute;top:12px;right:14px;background:var(--svd-header,rgba(27,94,32,.6));color:var(--svd-header-text,#fff);border:none;
        width:34px;height:34px;border-radius:50%;font-size:1.3rem;line-height:1;cursor:pointer;z-index:6;}
    .tg-header{position:relative;background:var(--svd-header,linear-gradient(135deg,#e8f5e9,#c8e6c9));padding:26px 26px 24px;
        border-radius:19px 19px 50% 50% / 19px 19px 12px 12px;border-bottom:none;}
    .tg-fav-row{position:sticky;bottom:0;display:flex;align-items:center;justify-content:center;z-index:4;padding:14px 20px 20px;background:var(--svd-body,#f1f8e9);border-top:2px dashed var(--svd-row-border,#c8e6c9);border-radius:0 0 19px 19px;}
    .tg-hole{position:absolute;top:9px;left:50%;transform:translateX(-50%);width:38px;height:12px;background:#fff;border:2px solid var(--svd-row-border,#a5d6a7);border-radius:8px;}
    .tg-title h2{margin:4px 0 0;color:var(--svd-header-text,#1b5e20);font-size:1.7rem;font-weight:900;line-height:1.12;}
    .tg-bot{font-style:italic;color:var(--svd-header-text,#2e7d32);font-size:1.1rem;margin-top:4px;opacity:.85;}
    .tg-by{color:var(--svd-header-text,#6d4c41);font-weight:700;margin-top:6px;opacity:.8;}
    .tg-tax{color:var(--svd-header-text,#558b2f);font-size:.9rem;margin-top:6px;opacity:.8;}
    .tg-edit{position:absolute;top:16px;right:62px;background:var(--svd-accent,#2e7d32);color:#fff;border:none;border-radius:8px;padding:7px 14px;font-weight:700;cursor:pointer;}
    .tg-body{display:grid;grid-template-columns:minmax(280px,2fr) 3fr;}
    .tg-left{padding:20px 22px;background:transparent;border-right:2px dashed var(--svd-row-border,#c8e6c9);}
    .tg-photo{border-radius:14px;overflow:hidden;border:3px solid #fff;box-shadow:0 6px 18px rgba(20,50,25,.18);background:var(--svd-photo-bg,#dcedc8);}
    .tg-photo img{width:100%;display:block;max-height:430px;object-fit:cover;cursor:zoom-in;}
    .tg-chips{display:flex;flex-wrap:wrap;gap:7px;margin-top:16px;}
    .tg-chip{background:#fff;color:var(--svd-accent,#2e7d32);border-radius:18px;padding:5px 12px;font-size:.85rem;font-weight:600;box-shadow:0 1px 3px rgba(20,50,25,.12);}
    .tg-timing{margin-top:14px;display:flex;flex-direction:column;gap:8px;}
    .tg-time{border-radius:10px;padding:10px 14px;font-size:.95rem;}
    .tg-plant{background:var(--svd-tint,#e8f5e9);color:var(--svd-accent,#2e7d32);}
    .tg-bloom{background:#fce4ec;color:#c2185b;}
    .tg-time-head{display:flex;flex-wrap:wrap;gap:6px;align-items:baseline;}
    .tg-time-head span{font-weight:600;}
    .tg-time .ms-strip{display:flex;gap:3px;margin-top:8px;}
    .tg-time .sv-barwrap{flex:1;display:flex;margin-top:8px;}
    .tg-admin-actions{position:absolute;top:14px;right:54px;display:flex;align-items:center;gap:10px;z-index:7;}
    .tg-admin-actions .tg-edit{position:static;top:auto;right:auto;}
    .tg-done{display:inline-flex;align-items:center;gap:5px;font-weight:700;color:var(--svd-accent,#1b5e20);cursor:pointer;
        font-size:.88rem;background:#fff;border:2px solid var(--svd-row-border,#a5d6a7);border-radius:8px;padding:5px 10px;}
    .tg-setdef{margin-top:10px;}
    .tg-setdef-lbl{font-size:.82rem;color:var(--svd-row-label,#558b2f);font-weight:700;}
    .tg-thumbs{display:flex;flex-wrap:wrap;gap:6px;margin-top:6px;}
    .tg-thumb{width:46px;height:46px;object-fit:cover;border-radius:8px;border:3px solid transparent;cursor:pointer;box-shadow:0 1px 3px rgba(13,40,71,.2);}
    .tg-thumb:hover{border-color:var(--svd-row-border,#a5d6a7);}
    .tg-thumb.is-default{border-color:var(--svd-accent,#2e7d32);box-shadow:0 0 0 2px var(--svd-accent,#2e7d32);}
    .tg-time .ms-cell{flex:1;min-width:0;height:18px;border-radius:3px;font-size:.64rem;line-height:18px;text-align:center;background:rgba(0,0,0,.07);color:#8a9a8a;}
    .tg-left .tg-sec{margin-top:18px;}
    .tg-right{padding:22px 24px;display:flex;flex-direction:column;gap:18px;}
    .tg-sec h4{margin:0 0 10px;color:var(--svd-header-text,#1b5e20);font-size:1.02rem;background:var(--svd-header,#c8e6c9);border-radius:8px;padding:7px 12px;}
    .tg-row{display:flex;justify-content:space-between;gap:16px;font-size:.92rem;padding:6px 4px;border-bottom:1px dotted var(--svd-row-border,#9fc7a2);}
    .tg-row span{color:var(--svd-row-label,#7a8c7a);font-weight:600;white-space:nowrap;}
    .tg-row b{color:var(--svd-body-text,#33402f);text-align:right;}
    .tg-note{margin:8px 0;}
    .tg-note h5{margin:0 0 3px;color:var(--svd-accent,#2e7d32);font-size:.85rem;}
    .tg-note p{margin:0;font-size:.86rem;color:var(--svd-body-text,#4a5d4a);line-height:1.5;}
    /* ── Favorite glow border pulse (uses current tag theme accent) ── */
    @keyframes favGlow{0%,100%{box-shadow:0 0 28px 10px var(--fav-glow,transparent);}50%{box-shadow:0 0 52px 22px var(--fav-glow,transparent);}}
    .tg-card.fav-glow{animation:favGlow 3s ease-in-out infinite;}
    .sv-grid-card.fav-glow{animation:favGlow 3s ease-in-out infinite;}
    .sv-grid-card{position:relative;}
    /* ── Favorite info bar ─────────────────────────────── */
    .tg-fav-info{display:flex;flex-wrap:wrap;gap:6px;align-items:center;justify-content:center;padding:6px 16px;font-size:.82rem;color:var(--svd-row-label,#666);border-bottom:1px dotted var(--svd-row-border,#ccc);}
    .tg-fav-info .fav-who{font-weight:700;color:var(--svd-accent,#2e7d32);}
    .tg-fav-info .fav-guests{opacity:.7;}
    /* ── Approval stamp ────────────────────────────────── */
    .tg-stamp{position:absolute;top:50%;left:50%;transform:translate(-50%,-50%) rotate(-18deg);z-index:0;pointer-events:none;
        width:280px;height:280px;border-radius:50%;border:6px solid rgba(139,119,90,.20);
        display:flex;align-items:center;justify-content:center;flex-direction:column;
        font-family:'Georgia',serif;color:rgba(139,119,90,.20);font-weight:900;text-transform:uppercase;letter-spacing:3px;}
    .tg-stamp .stamp-name{font-size:2.2rem;line-height:1;}
    .tg-stamp .stamp-label{font-size:.9rem;letter-spacing:5px;margin-top:4px;}
    .tg-stamp .stamp-ring{position:absolute;inset:8px;border-radius:50%;border:3px solid rgba(139,119,90,.15);}
    @media(max-width:760px){
        .tg-body{grid-template-columns:1.05fr 1.35fr;}
        .tg-left{padding:14px 14px;}
        .tg-right{padding:16px 16px;gap:14px;}
        .tg-header{padding:24px 16px 16px;}
        .tg-fav-row{padding:8px 14px;}
        .tg-title h2{font-size:1.25rem;}
        .tg-bot{font-size:.95rem;}
        .tg-sec h4{font-size:.92rem;padding:6px 10px;}
        .tg-row{font-size:.8rem;gap:8px;padding:5px 3px;}
        .tg-chip{font-size:.76rem;padding:4px 9px;}
        .tg-time{font-size:.82rem;padding:8px 11px;}
        .tg-note p{font-size:.8rem;}
        .tg-stamp{width:180px;height:180px;border-width:4px;}
        .tg-stamp .stamp-name{font-size:1.4rem;}
        .tg-stamp .stamp-label{font-size:.65rem;}
        .tg-setdef{display:none !important;}
    }`;
    document.head.appendChild(st);
}

// ── Update tag guide popup favorite UI (glow, info bar, stamp) ────────────
function updateTagGuideFavUI(plantId) {
    const p = window.plantData && window.plantData[plantId];
    if (!p) return;
    const card = document.querySelector('#tag-guide-back .tg-card');
    if (!card) return;
    const favTheme = getFavoriteTheme(p);
    const favs = p.favorited_by || [];

    // ── Glow border (uses current tag theme accent) ──
    const themeAccent = getComputedStyle(document.documentElement).getPropertyValue('--svd-accent').trim() || '#2e7d32';
    if (favs.length > 0) {
        card.style.setProperty('--fav-glow', themeAccent + '90');
        card.style.borderColor = themeAccent;
        card.classList.add('fav-glow');
    } else {
        card.style.removeProperty('--fav-glow');
        card.style.borderColor = '';
        card.classList.remove('fav-glow');
    }

    // ── Star + banner inside popup ──
    const star = card.querySelector('.fav-star');
    if (star) {
        star.innerText = p.is_favorited ? '⭐' : '☆';
        p.is_favorited ? star.classList.add('favorited') : star.classList.remove('favorited');
    }
    const banner = card.querySelector('.fav-banner');
    if (banner) {
        const bi = getBannerState(p);
        banner.innerText = bi.text;
        bi.show ? banner.classList.add('show') : banner.classList.remove('show');
        banner.style.background = (bi.text === 'Favorite This Plant')
            ? 'linear-gradient(135deg, var(--svd-accent, #4CAF50), var(--svd-row-label, #2E7D32))' : '';
    }

    // ── Fav info bar ──
    let infoEl = card.querySelector('.tg-fav-info');
    const named = [];
    const displayName = u => u === 'Joshua' ? 'Jun' : u;
    if (favs.includes('Joshua')) named.push('Jun');
    if (favs.includes('Jason')) named.push('Jason');
    if (favs.includes('Kona')) named.push('Kona');
    const guestCount = favs.filter(u => !['Joshua','Jason','Kona'].includes(u)).length;
    const totalCount = favs.length;
    if (totalCount > 0) {
        const parts = [];
        if (named.length) parts.push(`<span class="fav-who">⭐ ${named.join(', ')}</span>`);
        if (guestCount > 0) parts.push(`<span class="fav-guests">+${guestCount} visitor${guestCount > 1 ? 's' : ''}</span>`);
        parts.push(`(${totalCount} total)`);
        if (!infoEl) {
            infoEl = document.createElement('div');
            infoEl.className = 'tg-fav-info';
            const header = card.querySelector('.tg-header');
            if (header) header.after(infoEl);
        }
        infoEl.innerHTML = parts.join(' ');
    } else if (infoEl) {
        infoEl.remove();
    }

    // ── Approval stamp ──
    let stamp = card.querySelector('.tg-stamp');
    // Show stamp if any named user favorited
    if (named.length > 0) {
        if (!stamp) {
            stamp = document.createElement('div');
            stamp.className = 'tg-stamp';
            const body = card.querySelector('.tg-body');
            if (body) { body.style.position = 'relative'; body.appendChild(stamp); }
        }
        stamp.innerHTML = `<div class="stamp-ring"></div><span class="stamp-name">${named.join(' & ')}</span><span class="stamp-label">Approved</span>`;
    } else if (stamp) {
        stamp.remove();
    }
}

window.openTagGuide = function (p) {
    if (!p) return;
    injectTagGuideStyles();
    const old = document.getElementById('tag-guide-back'); if (old) old.remove();
    const E = svEscHtml;
    const imgBase = config.imgBase || '/images/Database/';
    const imgs = [
        ...(p.picture ? p.picture.split(',') : []),
        ...(p.additional_pictures ? p.additional_pictures.split(',') : [])
    ].map(s => s && s.trim()).filter(Boolean);
    const _hasImg = imgs.length > 0;
    const src = _hasImg ? imgBase + imgs[0] : '';
    // Thumbnail gallery: admin can set default, guests can browse & open lightbox
    let thumbsHTML = '';
    if (imgs.length > 1) {
        const isAdmin = !!config.currentUser;   // any signed-in garden account, as the server checks
        const label = isAdmin ? '⭐ Default photo — click to set:' : '📷 Photos:';
        thumbsHTML = `<div class="tg-setdef"><span class="tg-setdef-lbl">${label}</span>
             <div class="tg-thumbs">${imgs.map(f => {
                const fullSrc = imgBase + E(f);
                if (isAdmin) {
                    return `<img class="tg-thumb${f === imgs[0] ? ' is-default' : ''}" src="${fullSrc}" alt=""
                          title="Set as default" data-file="${E(f)}" onclick="window.setDefaultPhoto(${p.id}, this)">`;
                } else {
                    return svGuestThumbHtml(f, f === imgs[0], imgBase);
                }
             }).join('')}</div></div>`;
    }
    const sl   = v => (v || '').split('(')[0].trim();
    const few  = v => (v || '').split(';')[0].trim().split(/\s+/).slice(0, 4).join(' ');
    const trunc = (s, n) => { s = (s == null ? '' : String(s)).trim(); return s.length > n ? E(s.slice(0, n - 1).trim()) + '…' : E(s); };
    const gMask = svMonthMask(p.grow_months, p.planting_month_start, p.planting_month_end);
    const bMask = svMonthMask(p.bloom_months, p.month_start, p.month_end);
    const gText = svSeasonText(gMask);
    const bText = svSeasonText(bMask);
    const evergreen  = svEvergreen(p);
    const flowerless = svFlowerless(p);
    const row  = (l, v) => (v && String(v).trim()) ? `<div class="tg-row"><span>${l}</span><b>${E(v)}</b></div>` : '';
    const chip = (ic, v) => (v && String(v).trim()) ? `<span class="tg-chip">${ic} ${E(v)}</span>` : '';
    const note = (l, v, n = 240) => (v && String(v).trim()) ? `<div class="tg-note"><h5>${l}</h5><p>${trunc(v, n)}</p></div>` : '';
    const section = (title, inner) => inner.trim() ? `<div class="tg-sec"><h4>${title}</h4>${inner}</div>` : '';

    const leaf  = [p.leaf_type, p.leaf_shape, p.leaf_margin].filter(Boolean).map(sl).filter(Boolean).join(' · ');
    const bloom = [p.bloom_color, p.flower_shape].filter(Boolean).join(' · ');
    const fruit = [p.fruit_type, p.fruit_color, p.fruit_edibility].filter(Boolean).join(' · ');
    const size  = [p.plant_length || p.size, p.plant_width].filter(Boolean).join(' × ');
    const tax   = `🌿 ${E(p.family || '')}` + (p.genus ? ` › <b>${E(p.genus)}</b>` : '') + (p.species ? ` › <i>${E(p.species)}</i>` : '');
    const _qty  = (p.qty != null && String(p.qty).trim() !== '') ? 'Qty ' + p.qty : '';
    const _cost = (p.average_cost && parseFloat(p.average_cost) > 0) ? '$' + parseFloat(p.average_cost).toFixed(2) : '';
    const _year = (p.year_planted || p.year_added || '').toString().trim();
    const chips = chip('🪴', _qty) + chip('📅', _year ? 'Planted ' + _year : '') + chip('🌿', p.plant_type) + chip('📈', p.growth_rate) + chip('🌡️', p.zones) + chip('☀️', p.sun_exposure) + chip('💧', few(p.water_needs)) + chip('🌎', p.native_status) + chip('🛒', p.local_store);

    const notesSection = section('📝 Notes',
        note('Care', p.care_notes) +
        note('Maintenance', p.maintenance_notes) +
        note('Cleanup', p.cleanup_notes) +
        note('Propagation', p.propagation_method) +
        note('Companion plants', p.companion_plants) +
        note('Pests & disease', p.disease_susceptibility) +
        note('More', p.extra_details));

    const bannerInfo = getBannerState(p);
    const starHTML   = generateStarHTML(p, bannerInfo);

    const back = document.createElement('div');
    back.className = 'tg-back';
    back.id = 'tag-guide-back';
    back.innerHTML = `
        <div class="tg-card" onclick="event.stopPropagation()">
            <button class="tg-x" title="Close" onclick="document.getElementById('tag-guide-back').remove()">×</button>
            <div class="tg-header">
                <div class="tg-hole"></div>
                <div class="tg-title">
                    <h2>${E(p.plant_name || p.botanical_name || 'Unknown')}</h2>
                    ${p.botanical_name ? `<div class="tg-bot">${E(p.botanical_name)}</div>` : ''}
                    ${p.company ? `<div class="tg-by">By ${E(p.company)}</div>` : ''}
                    ${(p.family || p.genus || p.species) ? `<div class="tg-tax">${tax}</div>` : ''}
                </div>
                ${config.isAdmin ? `<div class="tg-admin-actions">
                    <label class="tg-done"><input type="checkbox" ${window._myDone.has(String(p.id)) ? 'checked' : ''} onchange="window.toggleTagDone(${p.id}, this.checked)"> ✅ ${E(config.currentUser === 'Joshua' ? 'Jun' : (config.currentUser || ''))} Done</label>
                    <label class="tg-done tg-img-approved"><input type="checkbox" ${parseInt(p.image_approved,10)===1 ? 'checked' : ''} onchange="window.toggleImageApproved(${p.id}, this.checked)"> 📸 Image Approved</label>
                    <button class="tg-edit" onclick="document.getElementById('tag-guide-back').remove(); window.editPlant(${p.id}${p.source_table === 'future_plants' ? ", 'future'" : ''})">✏️ Edit</button>
                </div>` : ''}
            </div>
            <div class="tg-body">
                <div class="tg-left">
                    <div class="tg-photo" id="tg-photo-wrap">${_hasImg ? `<img id="tg-main-img" draggable="false" src="${src}" alt="${E(p.plant_name || '')}"
                        style="object-position:${E(p.photo_position || '50% 50%')}${parseInt(p.photo_flip,10)===1?';transform:scaleX(-1)':''}"
                        onclick="window.openLightbox && window.openLightbox(document.getElementById('tg-main-img').src, ${p.id})"
                        onerror="this.onerror=null;this.insertAdjacentHTML('afterend',window._comingSoonSVG());this.remove();">` : _comingSoonSVG()}</div>
                    ${thumbsHTML}
                    <div class="tg-chips">${chips}</div>
                    <div class="tg-timing">
                        ${evergreen
                            ? `<div class="tg-time tg-plant"><div class="tg-time-head">🌲 <b>Foliage</b></div><div class="sv-barwrap">${renderEvergreenMonthBar()}</div></div>`
                            : (gText ? `<div class="tg-time tg-plant"><div class="tg-time-head">🌱 <b>Plant / Grow</b> <span>${gText}</span></div></div>` : '')}
                        ${flowerless
                            ? `<div class="tg-time tg-bloom"><div class="tg-time-head">🌸 <b>Bloom</b></div><div class="sv-barwrap">${renderFlowerlessTag()}</div></div>`
                            : (bText ? `<div class="tg-time tg-bloom"><div class="tg-time-head">🌸 <b>Bloom</b> <span>${bText}</span></div></div>` : '')}
                    </div>
                    ${notesSection}
                </div>
                <div class="tg-right">
                    ${section('🌱 Planting Guide',
                        row('Grow season', evergreen ? 'Evergreen (year-round)' : gText) +
                        row('Bloom season', flowerless ? 'Does not flower' : bText) +
                        row('Spacing', p.spacing || p.size_spacing) +
                        row('Planting depth', p.planting_depth) +
                        row('Mature size', size) +
                        row('Chilling hours', p.chilling_hours) +
                        row('Life expectancy', p.life_expectancy) +
                        row('Avg. cost', _cost) +
                        row('Store', p.local_store))}
                    ${section('🪵 Root System',
                        row('System', p.root_system) +
                        row('Morphology', p.root_morphology) +
                        row('Function', p.root_function))}
                    ${section('💧 Growing Conditions',
                        row('Sun', p.sun_exposure) +
                        row('Water', p.water_needs) +
                        row('Soil type', p.soil_type) +
                        row('Soil pH', p.soil_ph) +
                        row('Zones', p.zones) +
                        row('Native status', p.native_status))}
                    ${section('🌸 Appearance',
                        row('Type', p.plant_type) +
                        row('Leaf', leaf) +
                        row('Foliage color', p.foliage_color) +
                        row('Bloom', bloom) +
                        row('Fruit', fruit))}
                    ${section('🐝 Wildlife & Safety',
                        row('Attracts', p.attracts) +
                        row('Deer', p.deer_resistance) +
                        row('Toxicity', p.toxicity) +
                        row('Edibility', p.fruit_edibility))}
                    ${config.isAdmin ? `<div style="text-align:center;margin-top:14px;padding:10px 0;border-top:1px solid rgba(0,0,0,.08);">
                        <img src="qr/plant/${p.id}.png" alt="QR Code" style="width:120px;height:120px;image-rendering:pixelated;" onerror="this.parentElement.style.display='none';">
                        <div style="font-size:.75rem;color:var(--svd-row-label,#558b2f);margin-top:4px;">Scan to open this plant</div>
                    </div>` : ''}
                </div>
            </div>
            <div class="tg-fav-row">${starHTML}</div>
        </div>`;
    back.addEventListener('click', e => { if (e.target === back) back.remove(); });
    document.body.appendChild(back);
    svBindGuestThumbs(back, imgBase, p.id);
    // Apply favorite glow, info bar, and stamp
    updateTagGuideFavUI(p.id);
    function onKey(ev) { if (ev.key === 'Escape') { back.remove(); document.removeEventListener('keydown', onKey); } }
    document.addEventListener('keydown', onKey);
};

// ── Photo reposition (drag to adjust crop — on overview cards) ──────────
window._startReposition = function(plantId, photoDiv) {
    if (!photoDiv) return;
    const img = photoDiv.querySelector('img');
    if (!img || photoDiv.classList.contains('repo-active')) return;

    // Parse current object-position
    const cs = img.style.objectPosition || getComputedStyle(img).objectPosition || '50% 50%';
    const parts = cs.split(/\s+/);
    let posX = parseFloat(parts[0]) || 50;
    let posY = parseFloat(parts[1]) || 50;
    const origX = posX, origY = posY;

    // Track flip state
    const pData = window.plantData && window.plantData[plantId];
    let flipped = pData ? parseInt(pData.photo_flip, 10) === 1 : img.style.transform === 'scaleX(-1)';
    const origFlip = flipped;

    // Activate repo mode
    photoDiv.classList.add('repo-active');
    // Hide the 📐 button, show save/cancel bar
    const repoBtn = photoDiv.querySelector('.svd-repo-btn');
    if (repoBtn) repoBtn.style.display = 'none';
    const saveBar = document.createElement('div');
    saveBar.className = 'svd-repo-save-bar';
    saveBar.innerHTML = `<button class="repo-flip" title="Flip horizontal">↔️ Flip</button><button class="repo-save">💾 Save</button><button class="repo-cancel">✖ Cancel</button>`;
    photoDiv.appendChild(saveBar);

    // Flip toggle handler
    saveBar.querySelector('.repo-flip').onclick = function(e) {
        e.stopPropagation();
        flipped = !flipped;
        img.style.transform = flipped ? 'scaleX(-1)' : '';
    };

    // Block native drag + card click
    const block = e => { e.preventDefault(); e.stopPropagation(); };
    img.addEventListener('dragstart', block);
    img.addEventListener('selectstart', block);
    // Prevent the card's onclick from opening the popup while repositioning
    // but allow clicks on save/cancel buttons through
    const cardEl = photoDiv.closest('.svd-card');
    const blockCardClick = e => {
        if (e.target.closest('.svd-repo-save-bar')) return;
        e.stopImmediatePropagation();
    };
    if (cardEl) cardEl.addEventListener('click', blockCardClick, true);

    let dragging = false, startX = 0, startY = 0, startPosX = 50, startPosY = 50;
    function getXY(e) { const t = e.touches ? e.touches[0] : e; return [t.clientX, t.clientY]; }

    function onDown(e) {
        // Don't start drag if clicking save/cancel buttons
        if (e.target.closest('.svd-repo-save-bar')) return;
        e.preventDefault(); e.stopPropagation();
        dragging = true;
        img.classList.add('dragging');
        [startX, startY] = getXY(e);
        startPosX = posX; startPosY = posY;
    }
    function onMove(e) {
        if (!dragging) return;
        e.preventDefault();
        const [cx, cy] = getXY(e);
        const imgW = img.offsetWidth || 200;
        const imgH = img.offsetHeight || 300;
        const flipDir = flipped ? 1 : -1;
        posX = Math.max(0, Math.min(100, startPosX + flipDir * ((cx - startX) / imgW) * 100));
        posY = Math.max(0, Math.min(100, startPosY - ((cy - startY) / imgH) * 100));
        img.style.objectPosition = posX.toFixed(1) + '% ' + posY.toFixed(1) + '%';
    }
    function onUp() { dragging = false; img.classList.remove('dragging'); }

    photoDiv.addEventListener('mousedown', onDown);
    photoDiv.addEventListener('touchstart', onDown, {passive:false});
    document.addEventListener('mousemove', onMove);
    document.addEventListener('touchmove', onMove, {passive:false});
    document.addEventListener('mouseup', onUp);
    document.addEventListener('touchend', onUp);

    function cleanup() {
        photoDiv.classList.remove('repo-active');
        photoDiv.removeEventListener('mousedown', onDown);
        photoDiv.removeEventListener('touchstart', onDown);
        document.removeEventListener('mousemove', onMove);
        document.removeEventListener('touchmove', onMove);
        document.removeEventListener('mouseup', onUp);
        document.removeEventListener('touchend', onUp);
        img.removeEventListener('dragstart', block);
        img.removeEventListener('selectstart', block);
        if (cardEl) cardEl.removeEventListener('click', blockCardClick, true);
        saveBar.remove();
        if (repoBtn) repoBtn.style.display = '';
    }

    saveBar.querySelector('.repo-cancel').onclick = function(e) {
        e.stopPropagation();
        posX = origX; posY = origY;
        flipped = origFlip;
        img.style.objectPosition = origX.toFixed(1) + '% ' + origY.toFixed(1) + '%';
        img.style.transform = origFlip ? 'scaleX(-1)' : '';
        cleanup();
    };

    saveBar.querySelector('.repo-save').onclick = function(e) {
        e.stopPropagation();
        const pos = posX.toFixed(1) + '% ' + posY.toFixed(1) + '%';
        fetch('plant_actions.php', {
            method: 'POST', headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: new URLSearchParams({ action: 'save_photo_position', plant_id: plantId, position: pos, flip: flipped ? '1' : '0' })
        }).then(r => r.json()).then(d => {
            if (d.status === 'ok') {
                if (window.plantData && window.plantData[plantId]) {
                    window.plantData[plantId].photo_position = pos;
                    window.plantData[plantId].photo_flip = flipped ? '1' : '0';
                }
                // Also update the popup photo if it's open
                const popupImg = document.getElementById('tg-main-img');
                if (popupImg) {
                    popupImg.style.objectPosition = pos;
                    popupImg.style.transform = flipped ? 'scaleX(-1)' : '';
                }
            }
            cleanup();
        }).catch(() => cleanup());
    };
};

// Signed-in garden accounts: set a plant's default (primary) photo
window.setDefaultPhoto = function (plantId, imgEl) {
    const file = imgEl && imgEl.dataset ? imgEl.dataset.file : null;
    if (!file) return;
    const base = config.imgBase || '/images/Database/';
    fetch('plant_actions.php', {
        method: 'POST', headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({ action: 'set_default', plant_id: plantId, filename: file })
    }).then(r => r.json()).then(d => {
        if (d.status === 'ok') {
            if (window.plantData && window.plantData[plantId]) {
                window.plantData[plantId].picture = d.picture;
                window.plantData[plantId].additional_pictures = d.additional_pictures;
            }
            const main = document.getElementById('tg-main-img'); if (main) main.src = base + file;
            document.querySelectorAll('.tg-thumb').forEach(t => t.classList.toggle('is-default', t.dataset.file === file));
            const cardImg = document.querySelector('.sv-grid-card[data-id="' + plantId + '"] .svd-photo img');
            if (cardImg) cardImg.src = base + file;
        } else { alert(d.error || 'Could not set default photo'); }
    }).catch(() => alert('Network error setting default photo'));
};

// Admin: mark a plant reviewed/done for THIS user (persists in user_done)
window.toggleTagDone = function (plantId, checked) {
    fetch('plant_actions.php', {
        method: 'POST', headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({ action: 'toggle_done', plant_id: plantId, done: checked ? 1 : 0 })
    }).then(r => r.json()).then(d => {
        if (d.status !== 'ok') { alert(d.error || 'Could not update'); return; }
        if (d.done) window._myDone.add(String(plantId)); else window._myDone.delete(String(plantId));
        if (!window._doneViewUser) window._doneSet = new Set(window._myDone);
        refreshDoneBadges();
    }).catch(() => alert('Network error updating done status'));
};

// Admin: toggle global image_approved flag on a plant
window.toggleImageApproved = function (plantId, checked) {
    fetch('plant_actions.php', {
        method: 'POST', headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({ action: 'toggle_image_approved', plant_id: plantId, approved: checked ? 1 : 0 })
    }).then(r => r.json()).then(d => {
        if (d.status !== 'ok') { alert(d.error || 'Could not update'); return; }
        // Update the overview card badge if visible
        const card = document.querySelector('.sv-grid-card[data-id="' + plantId + '"]');
        if (card) {
            card.dataset.imageApproved = d.image_approved;
            let badge = card.querySelector('.img-approved-badge');
            if (d.image_approved && !badge) {
                badge = document.createElement('span');
                badge.className = 'img-approved-badge';
                badge.textContent = '📸';
                badge.style.cssText = 'position:absolute;top:6px;left:6px;font-size:1.1rem;z-index:3;';
                card.querySelector('.svd-photo').appendChild(badge);
            } else if (!d.image_approved && badge) {
                badge.remove();
            }
        }
    }).catch(() => alert('Network error updating image approval'));
};

// =======================================================
// 1. GLOBAL WINDOW REGISTRATIONS (HOISTED TO TOP)
// =======================================================
// By placing these here, we guarantee the HTML buttons 
// can find them immediately, bypassing any render errors.

window.closeModal = closeModal;

// Route all window.openModal calls through notes interceptor
window.openModal = function(plantData) {
    interceptCardClick(plantData, window._originalOpenModal);
};

// toggleBedGroup and bedSearch now live in js/search.js

window.toggleLike = async function(plantId) {
    try {
        const response = await fetch(`catalog.php?toggle_like=${plantId}`);
        const result = await response.json();
        
        // Grab elements for both the Modal and the Main Feed
        const modalLikeBtn = document.getElementById(`like-btn-${plantId}`);
        const modalLikeCount = document.getElementById(`like-count-${plantId}`);
        const feedLikeBtn = document.getElementById(`feed-like-btn-${plantId}`);
        const feedLikeCount = document.getElementById(`feed-like-count-${plantId}`);

        let currentCount = 0;
        if (window.plantData[plantId]) {
            currentCount = parseInt(window.plantData[plantId].like_count) || 0;
        }

        if (result.status === 'added') {
            currentCount++;
            if (modalLikeBtn) modalLikeBtn.style.opacity = '1'; 
            if (feedLikeBtn) feedLikeBtn.style.opacity = '1';
        } else if (result.status === 'removed') {
            currentCount = Math.max(0, currentCount - 1);
            if (modalLikeBtn) modalLikeBtn.style.opacity = '0.5'; 
            if (feedLikeBtn) feedLikeBtn.style.opacity = '0.6';
        }
        
        if (modalLikeCount) modalLikeCount.innerText = currentCount;
        if (feedLikeCount) feedLikeCount.innerText = currentCount;
        
        if (window.plantData[plantId]) {
            window.plantData[plantId].is_liked = (result.status === 'added');
            window.plantData[plantId].like_count = currentCount;
        }
    } catch (error) {
        console.error("Error toggling like:", error);
    }
};

window.toggleFavorite = async function(e, plantId, starElement) {
    e.stopPropagation(); 
    starElement.classList.remove('animate-pop');
    void starElement.offsetWidth; 
    starElement.classList.add('animate-pop');

    try {
        const res = await fetch(`catalog.php?toggle_favorite=${plantId}`);
        // Never let one bad page kill the whole catalog.
        //
        // This threw "Unexpected end of JSON input" for months of plants:
        // catalog.php returned HTTP 200 with an EMPTY body whenever
        // json_encode() hit non-UTF-8 data (the binary qr_code column), and
        // `echo false` prints nothing. The throw landed in the catch below,
        // which meant currentPage++ never ran, so the infinite scroll retried
        // the same broken page forever and the list froze after 9 plants.
        //
        // The server side is fixed (see json_out() in catalog.php), but this
        // stays as a seatbelt: if a page ever comes back unreadable again,
        // skip past it and keep loading instead of freezing the catalog.
        const _raw = await res.text();
        let data;
        try {
            data = JSON.parse(_raw);
        } catch (e) {
            console.error('Bad JSON on page ' + currentPage +
                          ' (' + _raw.length + ' bytes) — skipping it.', e);
            currentPage++;          // the step the old code never reached
            return;                 // finally{} clears isLoading, so scrolling still works
        }
        
        const p = window.plantData[plantId];
        p.favorited_by = p.favorited_by || [];
        const localUser = config.currentUser || 'Guest';

        if (data.status === 'added') {
            p.is_favorited = true;
            if (!p.favorited_by.includes(localUser)) p.favorited_by.push(localUser);
            p.fav_count = p.favorited_by.length;
            launchFirework(e.clientX, e.clientY);
        } else if (data.status === 'removed') {
            p.is_favorited = false;
            p.favorited_by = p.favorited_by.filter(u => u !== localUser);
            p.fav_count = p.favorited_by.length;
        }

        const bannerInfo = getBannerState(p);
        const modalFoil = starElement.closest('.plant-modal-content');
        const gridCard = document.querySelector(`.plant-card[data-id="${plantId}"]`);

        const updatedFavTheme = getFavoriteTheme(p);

        const updateUI = (container) => {
            if (!container) return;
            const star        = container.querySelector('.fav-star');
            const banner      = container.querySelector('.fav-banner');
            const countDisplay= container.querySelector('.fav-count-display');
            const bgEl        = container.querySelector('[data-card-bg]');

            if (star) {
                star.innerText = p.is_favorited ? '⭐' : '☆';
                p.is_favorited ? star.classList.add('favorited') : star.classList.remove('favorited');
            }
            if (countDisplay) countDisplay.innerText = `⭐ ${p.fav_count || 0}`;

            // ── Feed card: bgEl is the inner data-card-bg div ─────────────────────
            if (bgEl) {
                // Restore to the original random theme color stored at render time, not blank
                const defaultBg = bgEl.dataset.defaultBg || '';
                bgEl.style.backgroundColor = updatedFavTheme ? updatedFavTheme.bg : defaultBg;
                if (updatedFavTheme) {
                    container.style.border    = `2px solid ${updatedFavTheme.border}`;
                    container.style.boxShadow = `0 0 0 1px ${updatedFavTheme.border}40, 0 4px 16px ${updatedFavTheme.accent}25`;
                } else {
                    container.style.border    = '';
                    container.style.boxShadow = '';
                }
            }

            // ── Modal card: foilCard itself is the container (no data-card-bg child) ──
            if (!bgEl && container.classList.contains('plant-modal-content')) {
                if (updatedFavTheme) {
                    container.style.backgroundColor = updatedFavTheme.bg;
                    container.style.borderColor     = updatedFavTheme.border;
                    container.style.boxShadow       = `0 0 0 2px ${updatedFavTheme.border}, 0 20px 60px ${updatedFavTheme.accent}30`;
                } else {
                    // Clear inline style so cycled bg-theme-X CSS class colour shows through again
                    container.style.backgroundColor = '';
                    container.style.borderColor     = '';
                    container.style.boxShadow       = '';
                }
            }

            if (!bannerInfo.isHolo || data.status === 'removed') {
                container.classList.remove('holographic-active');
                container.style.transform = `perspective(1000px) rotateX(0deg) rotateY(0deg) scale3d(1, 1, 1)`;
                container.style.setProperty('--mouse-x', '50%');
                container.style.setProperty('--mouse-y', '50%');

                const pContainer = container.querySelector('#modal-parallax');
                if (pContainer) {
                    pContainer.style.setProperty('--move-x', '0px');
                    pContainer.style.setProperty('--move-y', '0px');
                }
            } else {
                container.classList.add('holographic-active');
            }

            if (banner) {
                banner.innerText = bannerInfo.text;
                bannerInfo.show ? banner.classList.add('show') : banner.classList.remove('show');
                banner.style.background = (bannerInfo.text === 'Favorite This Plant')
                    ? `linear-gradient(135deg, var(--svd-accent, #4CAF50), var(--svd-row-label, #2E7D32))`
                    : '';
            }
        };

        updateUI(modalFoil);
        updateUI(gridCard);

        // ── Update overview card glow ──
        if (gridCard && gridCard.classList.contains('sv-grid-card')) {
            const accent = getComputedStyle(document.documentElement).getPropertyValue('--svd-accent').trim() || '#2e7d32';
            if (p.favorited_by && p.favorited_by.length > 0) {
                gridCard.style.borderColor = accent;
                gridCard.style.setProperty('--fav-glow', accent + '90');
                gridCard.classList.add('fav-glow');
            } else {
                gridCard.style.borderColor = 'transparent';
                gridCard.style.removeProperty('--fav-glow');
                gridCard.classList.remove('fav-glow');
            }
        }

        // Also update the tag guide popup (glow, info, stamp)
        updateTagGuideFavUI(plantId);

    } catch (err) { console.error("Error toggling favorite", err); }
};

window.runSearch = function(term) {
    if (searchBox) {
        searchBox.value = term;
        sessionStorage.setItem('garden_search_persist', term);
        loadPlants(true);
    }
};

window.checkClick = function(e, id, source) {
    if (!e.target.closest('button, select, input, textarea, .bulk-checkbox, .fav-star, .like-count-display')) {
        // For future plants, fetch from correct table if not cached
        if (source === 'future' && !window.plantData[id]) {
            fetch(`catalog.php?ajax_load=1&plant_id=${id}&source=future`)
                .then(r => r.json())
                .then(d => {
                    if (d.results && d.results[0]) {
                        window.plantData[id] = d.results[0];
                        if (isSimpleView() && window.openTagGuide) window.openTagGuide(d.results[0]);
                        else interceptCardClick(d.results[0], openModal);
                    }
                });
            return;
        }
        // Simple/Tags view → custom baby-blue planting-guide tag.
        // Full Cards view → the standard detailed modal.
        if (isSimpleView() && window.openTagGuide) window.openTagGuide(window.plantData[id]);
        else interceptCardClick(window.plantData[id], openModal);
    }
};

// ── Promote a future plant to data_source ─────────────────────────
window.promoteFuture = async function(id) {
    if (!confirm('Move this plant from Wishlist into your garden database?')) return;
    try {
        const res = await fetch('plant_actions.php', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ action: 'promote_future', id })
        });
        const data = await res.json();
        if (data.status === 'ok') {
            alert(`${data.plant_name} has been added to your garden! (New ID: ${data.new_id})`);
            // Remove the card from the grid
            const card = document.querySelector(`.plant-card[data-id="${id}"][data-source="future"]`);
            if (card) card.remove();
        } else {
            alert('Error: ' + (data.error || 'Unknown'));
        }
    } catch (err) {
        alert('Network error: ' + err.message);
    }
};

window.toggleBulkMode = function() {
    isBulkMode = !isBulkMode;
    const bt = document.getElementById('bulk-toolbar');
    if (bt) bt.style.display = isBulkMode ? 'flex' : 'none';
    const pf = document.getElementById('plantForm');
    if (pf) pf.classList.toggle('mobile-open', false); 
    document.body.classList.toggle('bulk-active', isBulkMode);
    const bf = document.getElementById('bulk-floater');
    if (bf) bf.classList.toggle('active', isBulkMode);
    
    if (!isBulkMode) {
        document.querySelectorAll('.plant-card').forEach(c => c.classList.remove('selected-card'));
        document.querySelectorAll('.bulk-checkbox').forEach(cb => cb.checked = false);
        window.updateBulkCount();
    }
};

window.updateBulkCount = function() {
    const checked = document.querySelectorAll('.bulk-checkbox:checked');
    const badge = document.getElementById('count-badge');
    if (badge) badge.innerText = checked.length;
    const bulkIds = document.getElementById('bulk_ids');
    if (bulkIds) bulkIds.value = Array.from(checked).map(cb => cb.value).join(',');
};

// Admin Functions
window.rotateGalleryImage = async function(filename, imgElementId) {
    const imgEl = document.getElementById(imgElementId);
    if (imgEl) imgEl.style.opacity = '0.5';

    try {
        const res = await fetch(`catalog.php?rotate_image=1&filename=${encodeURIComponent(filename)}`);
        const data = await res.json();

        if (data.status === 'success') {
            if (imgEl) {
                const baseUrl = imgEl.src.split('?')[0]; 
                imgEl.src = `${baseUrl}?_t=${Date.now()}`;
                imgEl.style.opacity = '1';
            }
        } else {
            alert('Error rotating image: ' + data.message);
            if (imgEl) imgEl.style.opacity = '1';
        }
    } catch (error) {
        console.error("Rotation error:", error);
        alert("Failed to rotate image. Check console for details.");
        if (imgEl) imgEl.style.opacity = '1';
    }
};

window.deleteGalleryImage = async function(filename, plantId) {
    if (!confirm('⚠️ Are you sure you want to delete this image?')) return;

    try {
        const res = await fetch(`catalog.php?delete_image=1&plant_id=${plantId}&filename=${encodeURIComponent(filename)}`);
        const data = await res.json();

        if (data.status === 'success') {
            if (window.plantData[plantId] && window.plantData[plantId].additional_pictures) {
                let pics = window.plantData[plantId].additional_pictures.split(',');
                pics = pics.filter(p => p.trim() !== filename);
                window.plantData[plantId].additional_pictures = pics.join(',');
            }
            if (window.openModal) window.openModal(window.plantData[plantId]);
        } else {
            alert('Error deleting image: ' + data.message);
        }
    } catch (error) {
        console.error("Deletion error:", error);
        alert("Failed to delete image.");
    }
};

window.toggleReviewStatus = async function(plantId) {
    try {
        const res = await fetch(`catalog.php?toggle_review=${plantId}`);
        const data = await res.json();
        
        if (data.status === 'success') {
            if(window.plantData[plantId]) window.plantData[plantId].needs_review = data.needs_review;
            const btn = document.getElementById(`review-btn-${plantId}`);
            if (btn) {
                if (data.needs_review) {
                    btn.innerHTML = '⚠️ Flagged';
                    btn.style.background = '#d32f2f';
                    btn.style.color = '#fff';
                } else {
                    btn.innerHTML = '✅ Mark Needs Review';
                    btn.style.background = '#e0e0e0';
                    btn.style.color = '#333';
                }
            }
        }
    } catch (err) {
        console.error("Error toggling review status:", err);
    }
};

window.uploadGalleryImages = async function(e, plantId) {
    const files = e.target.files;
    if (!files.length) return;
    
    if (!window.plantData[plantId]) {
        window.plantData[plantId] = { additional_pictures: "" };
    }

    const statusDiv = document.getElementById(`gallery-upload-status-${plantId}`);
    if (statusDiv) statusDiv.innerHTML = `<em>Uploading ${files.length} image(s)... Please wait.</em>`;
    
    const formData = new FormData();
    formData.append('plant_id', plantId);
    for(let i=0; i<files.length; i++) {
        formData.append('gallery_images[]', files[i]);
    }
    
    try {
        const res = await fetch('catalog.php?upload_gallery=1', {
            method: 'POST',
            body: formData
        });
        const data = await res.json();
        
        if(data.status === 'success') {
            if (statusDiv) statusDiv.innerHTML = "<span style='color:green;'>✅ Upload successful!</span>";
            
            if (!window.plantData[plantId].additional_pictures) window.plantData[plantId].additional_pictures = "";
            let currentPics = window.plantData[plantId].additional_pictures.split(',').filter(x => x);
            currentPics = currentPics.concat(data.new_images);
            window.plantData[plantId].additional_pictures = currentPics.join(',');
            
            setTimeout(() => { if(window.openModal) window.openModal(window.plantData[plantId]); }, 500);
        } else {
            alert('Error: ' + data.message);
            if (statusDiv) statusDiv.innerHTML = "<span style='color:red;'>❌ Upload failed.</span>";
        }
    } catch(err) {
        console.error(err);
        alert('Upload failed. Check console for details.');
        if (statusDiv) statusDiv.innerHTML = "<span style='color:red;'>⚠️ Server error.</span>";
    }
};


// =======================================================
// 2. UTILITY & RENDER LOGIC
// =======================================================

function getTagClass(text) {
    return text ? 'tag-' + text.toLowerCase().replace(/[']/g, '').replace(/[^a-z0-9]+/g, '-') : '';
}

const getCSSColor = (str, def) => {
    if(!str) return def;
    const c = str.toLowerCase();
    const colors = ['red','blue','green','yellow','purple','orange','pink','white','black','gray','silver','gold','brown','cyan','magenta', 'lime', 'olive', 'maroon', 'navy', 'teal'];
    let matched = colors.find(color => c.includes(color));
    return matched ? matched : def;
};

const generateMorphologyHTML = (p) => {
    let html = '';

    if (p.leaf_shape || p.foliage_color || p.leaf_type || p.leaf_margin) {
        const fColor = getCSSColor(p.foliage_color, '#4CAF50'); 
        const shapeStr = (p.leaf_shape || '').split(' ')[0] || 'Leaf'; 
        const title = `FOLIAGE\nColor: ${p.foliage_color || 'N/A'}\nShape: ${p.leaf_shape || 'N/A'}\nType: ${p.leaf_type || 'N/A'}\nMargin: ${p.leaf_margin || 'N/A'}`;
        
        let svgPath = "M12 2L8 8C4 14 12 22 12 22C12 22 20 14 16 8L12 2Z"; 
        if(shapeStr.includes('Linear')) svgPath = "M10 2L10 22L14 22L14 2Z"; 
        if(shapeStr.includes('Cordate')) svgPath = "M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"; 
        if(shapeStr.includes('Ovate')) svgPath = "M12 2C8 6 6 12 12 22C18 12 16 6 12 2Z"; 

        html += `<span title="${title}" class="tag" style="background:#f1f8e9; border:1px solid #c5e1a5; display:inline-flex; align-items:center; gap:4px; cursor:help; font-size:0.75rem; color:#33691e;">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="${fColor}" stroke="#33691e" stroke-width="1.5"><path d="${svgPath}"/></svg>
            ${shapeStr}
        </span>`;
    }

    if (p.flower_shape || p.bloom_color || p.flower_symmetry || p.flower_size) {
        const bColor = getCSSColor(p.bloom_color, '#e91e63'); 
        const shapeStr = (p.flower_shape || '').split(' ')[0] || 'Bloom'; 
        const title = `BLOOM\nColor: ${p.bloom_color || 'N/A'}\nShape: ${p.flower_shape || 'N/A'}\nSize: ${p.flower_size || 'N/A'}\nSymmetry: ${p.flower_symmetry || 'N/A'}`;
        
        html += `<span title="${title}" class="tag" style="background:#fce4ec; border:1px solid #f48fb1; display:inline-flex; align-items:center; gap:4px; cursor:help; font-size:0.75rem; color:#880e4f;">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="${bColor}" stroke="#880e4f" stroke-width="1.5">
                <path d="M12 22c4.97 0 9-4.03 9-9-4.97 0-9 4.03-9 9zM12 2c-4.97 0-9 4.03-9 9 4.97 0 9-4.03 9-9zM22 12c0-4.97-4.03-9-9-9 0 4.97 4.03 9 9 9zM2 12c0 4.97 4.03 9 9 9 0-4.97-4.03-9-9-9z"/>
                <circle cx="12" cy="12" r="3" fill="#ffd54f"/>
            </svg>
            ${p.bloom_color ? p.bloom_color.split(' ')[0] : shapeStr}
        </span>`;
    }

    if (p.fruit_shape || p.fruit_color || p.fruit_size || p.fruit_edibility) {
        const frColor = getCSSColor(p.fruit_color, '#d32f2f');
        const shapeStr = (p.fruit_shape || 'Fruit').split(' ')[0];
        const title = `FRUIT\nColor: ${p.fruit_color || 'N/A'}\nShape: ${p.fruit_shape || 'N/A'}\nSize: ${p.fruit_size || 'N/A'}\nEdibility: ${p.fruit_edibility || 'N/A'}`;
        
        let svgPath = "M12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22Z"; 
        if(shapeStr.toLowerCase().includes('pod') || shapeStr.toLowerCase().includes('legume')) {
            svgPath = "M4 20C4 20 2 10 10 4C18 -2 22 2 22 2C22 2 24 12 16 18C8 24 4 20 4 20Z"; 
        } else if (shapeStr.toLowerCase().includes('oval') || shapeStr.toLowerCase().includes('drupe')) {
            svgPath = "M12 2C7 2 4 7 4 12C4 17 7 22 12 22C17 22 20 17 20 12C20 7 17 2 12 2Z"; 
        }

        html += `<span title="${title}" class="tag" style="background:#ffebee; border:1px solid #ef9a9a; display:inline-flex; align-items:center; gap:4px; cursor:help; font-size:0.75rem; color:#b71c1c;">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="${frColor}" stroke="#b71c1c" stroke-width="1.5"><path d="${svgPath}"/></svg>
            ${p.fruit_color ? p.fruit_color.split(' ')[0] : shapeStr}
        </span>`;
    }

    if (p.stem_color) {
        const sColor = getCSSColor(p.stem_color, '#795548'); 
        const title = `STEM\nColor: ${p.stem_color}`;
        html += `<span title="${title}" class="tag" style="background:#efebe9; border:1px solid #bcaaa4; display:inline-flex; align-items:center; gap:4px; cursor:help; font-size:0.75rem; color:#4e342e;">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="${sColor}" stroke="#3e2723" stroke-width="1.5"><rect x="10" y="2" width="4" height="20" rx="1"/></svg>
            ${p.stem_color.split(' ')[0]} Stem
        </span>`;
    }

    if (p.aromatics && p.aromatics.toLowerCase() !== 'none' && p.aromatics.trim() !== '') {
        const matchedColor = getCSSColor(p.bloom_color, '#ba68c8');
        const title = `AROMATICS\nSmells like: ${p.aromatics}`;
        html += `<span title="${title}" class="tag" style="background:#fafafa; border:1px solid ${matchedColor}; display:inline-flex; align-items:center; gap:4px; cursor:help; font-size:0.75rem; color:${matchedColor}; font-weight: bold;">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="${matchedColor}" stroke-width="2" stroke-linecap="round"><path d="M6 21c0-4 4-6 4-10s-4-6-4-10M12 21c0-4 4-6 4-10s-4-6-4-10M18 21c0-4 4-6 4-10s-4-6-4-10"/></svg>
            ${p.aromatics}
        </span>`;
    }

    return html ? `<div style="display:flex; flex-wrap:wrap; gap:4px; margin-bottom: 8px;">${html}</div>` : '';
};


// =======================================================
// 3. EVENT LISTENERS & INITIALIZATION
// =======================================================

const savedSearch = sessionStorage.getItem('garden_search_persist');
if(savedSearch && searchBox) searchBox.value = savedSearch;

let timer;
if (searchBox) {
    searchBox.addEventListener('input', () => { 
        sessionStorage.setItem('garden_search_persist', searchBox.value);
        clearTimeout(timer); 
        timer = setTimeout(() => loadPlants(true), 500); 
    });
}

document.addEventListener('change', (e) => {
    if (e.target.classList.contains('bulk-checkbox')) {
        const card = e.target.closest('.plant-card');
        if (e.target.checked) card.classList.add('selected-card');
        else card.classList.remove('selected-card');
        window.updateBulkCount();
    }
});

async function loadPlants(reset = false) {
    if (isLoading || !feedContainer) return; 
    if (!reset && !hasMore) return;
    isLoading = true;
    const _isSV = isSimpleView();
    document.body.classList.toggle('simple-view-active', _isSV);
    if (_isSV) { injectThemePicker(); } else { removeThemePicker(); }
    if (reset) { currentPage = 1; hasMore = true; lastBedName = ""; renderedBeds.clear(); feedContainer.innerHTML = ""; }
    try {
        // Ensure done-set is loaded before first render so badges appear
        if (window._doneReady) await window._doneReady;
        const mobileParam = isMobile() ? '&mobile=1' : '';
        const res = await fetch(`catalog.php?ajax_load=1&page=${currentPage}&search=${encodeURIComponent(searchBox?.value || '')}${mobileParam}&_t=${Date.now()}`);
        const data = await res.json();
        
        if (data.error) { console.error("Server Error:", data.error); return; } 
        if (!data.results || data.results.length === 0) {
            hasMore = false; 
            const trig = document.getElementById('loading-trigger');
            if (trig) trig.innerHTML = `<p style="font-size:1.6rem;font-weight:900;letter-spacing:1px;">🪴 End of Garden. <span id="boogie-trigger" style="cursor:pointer;text-decoration:underline dotted;color:#d81b60;font-weight:900;font-size:1.8rem;" title="Click me...">Boogie? 🕺</span></p>`;
            document.getElementById('boogie-trigger')?.addEventListener('click', () => startBoogie(config.imgBase || '/images/Database/'));
            return;
        }

        let flatList = [];
        const isSearching = (searchBox?.value || '').trim().length > 0;

        data.results.forEach(p => {
            window.plantData[p.id] = p;
            const beds = (p.flower_bed || "To Be Sorted").split(',').map(b => b.trim()).filter(b => b);
            if (beds.length <= 1 || isSearching) {
                let mainEntry = { ...p };
                mainEntry.flower_bed = beds[0];
                mainEntry.also_in = beds.slice(1).join(', ');
                flatList.push(mainEntry);
            } else {
                beds.forEach(bed => {
                    let clone = { ...p };
                    clone.flower_bed = bed; clone.qty = 1; clone.also_in = beds.filter(b => b !== bed).join(', ');
                    flatList.push(clone);
                });
            }
        });

        flatList.sort((a, b) => {
            if (a.flower_bed < b.flower_bed) return -1;
            if (a.flower_bed > b.flower_bed) return 1;
            return a.plant_name.localeCompare(b.plant_name);
        });

        // ── Card builder (shared by desktop + mobile) ─────────────────────────
        const buildCard = (p) => {
            const card = document.createElement('div');
            const bannerInfo = getBannerState(p);
            const favTheme   = getFavoriteTheme(p);

            card.className = 'plant-card';
            card.dataset.id = p.id;
            const isFuture = p.source_table === 'future_plants';
            if (isFuture) {
                card.classList.add('future-plant');
                card.dataset.source = 'future';
            }

            // ── SIMPLE VIEW — Jun's Simple Tags: minimal photo-left card (no popup) ──
            if (isSimpleView()) {
                const _isDead = p.died == 1 || (p.flower_bed && p.flower_bed.includes('Dead'));
                const _imgs = [
                    ...(p.picture ? p.picture.split(',') : []),
                    ...(p.additional_pictures ? p.additional_pictures.split(',') : [])
                ].map(s => s && s.trim()).filter(Boolean);
                // Randomly pick one of the plant's photos (if it has any)
                const _hasPhoto = _imgs.length > 0;
                const _src = _hasPhoto ? config.imgBase + _imgs[0] : '';
                const _photoPos = p.photo_position || '50% 50%';
                const _photoFlip = parseInt(p.photo_flip, 10) === 1;
                const _imgStyle = (_isDead ? 'opacity:.4; filter:grayscale(20%) contrast(70%) brightness(1.2);' : '') + 'object-position:' + _photoPos + ';' + (_photoFlip ? 'transform:scaleX(-1);' : '');
                const _common = svEscHtml(p.plant_name || p.botanical_name || 'Unknown');
                const _bot    = svEscHtml(p.botanical_name || '');
                const _co     = svEscHtml(p.company || '');
                const _qty    = (p.qty != null && String(p.qty).trim() !== '') ? svEscHtml(p.qty) : '';
                const _gm     = svMonthMask(p.grow_months,  p.planting_month_start, p.planting_month_end);
                const _bm     = svMonthMask(p.bloom_months, p.month_start,          p.month_end);
                const _gt     = svSeasonText(_gm);
                const _bt     = svSeasonText(_bm);
                const _sl     = v => (v || '').split('(')[0].trim();                       // strip "(...)" on leaf fields
                const _fewWords = v => (v || '').split(';')[0].trim().split(/\s+/).slice(0, 4).join(' '); // a few words only
                const _row    = (lbl, val) => (val && String(val).trim()) ? `<div class="svd-row"><span>${lbl}</span><b>${svEscHtml(val)}</b></div>` : '';
                const _chip   = (icon, val) => (val && String(val).trim()) ? `<span class="svd-chip">${icon} ${svEscHtml(val)}</span>` : '';
                const _leaf   = [p.leaf_type, p.leaf_shape, p.leaf_margin].filter(Boolean).map(_sl).filter(Boolean).join(' · ');
                const _fruit  = [p.fruit_type, p.fruit_color, p.fruit_edibility].filter(Boolean).join(' · ');
                const _size   = [p.plant_length || p.size, p.plant_width].filter(Boolean).join(' × ');
                const _water  = _fewWords(p.water_needs);
                const _tax    = `🌿 ${svEscHtml(p.family || '')}` + (p.genus ? ` › <b>${svEscHtml(p.genus)}</b>` : '') + (p.species ? ` › <i>${svEscHtml(p.species)}</i>` : '');
                // Compact chip strip: Qty + Growth + Zones + Sun + Water (short)
                const _chips  = _chip('🪴', _qty ? 'Qty ' + _qty : '') + _chip('📈', p.growth_rate) + _chip('🌡️', p.zones) + _chip('☀️', p.sun_exposure) + _chip('💧', _water) + _chip('🛒', p.local_store);
                card.classList.add('sv-grid-card');
                if (window._doneSet.has(String(p.id))) card.classList.add('is-done');
                const _imgApproved = parseInt(p.image_approved, 10) === 1;
                card.dataset.imageApproved = _imgApproved ? '1' : '0';
                const _adminFooter = config.isAdmin ? `
                            <div class="svd-admin" onclick="event.stopPropagation()">
                                <button class="btn btn-blue" onclick="window.editPlant(${p.id}${isFuture ? ", 'future'" : ''})">✏️ Edit</button>
                                ${isFuture ? `<button class="btn" style="background:linear-gradient(135deg,#7c4dff,#448aff);color:white;" onclick="window.promoteFuture(${p.id})">🌱 Add</button>` : ''}
                            </div>` : '';
                card.innerHTML = `
                    <div class="svd-card" onclick="window.checkClick(event, ${p.id}${isFuture ? ", 'future'" : ''})">
                        ${isFuture ? '<div class="sv-future-badge">Wish List</div>' : ''}
                        ${_isDead ? '<div class="sv-dead">DIED</div>' : ''}
                        ${window._doneSet.has(String(p.id)) ? '<div class="sv-done-badge">✓ Done</div>' : ''}
                        ${config.isAdmin ? `<div onclick="event.stopPropagation()" style="position:absolute;top:10px;right:10px;z-index:30;"><input type="checkbox" class="bulk-checkbox" value="${p.id}"></div>` : ''}
                        <div class="svd-photo" data-plant-id="${p.id}">
                            ${_imgApproved ? '<span class="img-approved-badge" style="position:absolute;top:6px;left:6px;font-size:1.1rem;z-index:3;">📸</span>' : ''}
                            ${_hasPhoto ? `<img src="${_src}" loading="lazy" draggable="false" alt="${_common}" style="${_imgStyle}"
                                 onerror="this.onerror=null;this.insertAdjacentHTML('afterend',window._comingSoonSVG());this.remove();">` : _comingSoonSVG()}
                            ${config.isAdmin ? `<button class="svd-repo-btn" onclick="event.stopPropagation(); window._startReposition(${p.id}, this.parentElement)" title="Reposition photo">📐</button>` : ''}
                        </div>
                        <div class="svd-body">
                            <div class="svd-header-green">
                                <h3 class="svd-common">${_common}</h3>
                                ${_bot ? `<div class="svd-bot">${_bot}</div>` : ''}
                                ${_co  ? `<div class="svd-by">By ${_co}</div>` : ''}
                            </div>
                            <div class="svd-overview">
                                ${_chips ? `<div class="svd-chips">${_chips}</div>` : ''}
                                <div class="svd-rows">
                                    ${_row('Type', p.plant_type)}
                                    ${_row('🍃 Leaf', _leaf)}
                                    ${_row('🍒 Fruit', _fruit)}
                                    ${_row('🌸 Bloom', p.bloom_color)}
                                    ${_row('📐 Size', _size)}
                                    ${_row('🌎 Native', p.native_status)}
                                    ${_row('🦋 Attracts', p.attracts)}
                                </div>
                                <div class="svd-months">
                                    ${svEvergreen(p)
                                        ? `<div class="sv-seasonbar"><span>🌲 Foliage</span><div class="sv-barwrap">${renderEvergreenMonthBar()}</div></div>`
                                        : (_gt ? `<div class="sv-row sv-plant"><span class="sv-ico">🌱</span><b>Plant</b> ${_gt}</div>` : '')}
                                    ${svFlowerless(p)
                                        ? `<div class="sv-seasonbar"><span>🌸 Bloom</span><div class="sv-barwrap">${renderFlowerlessTag()}</div></div>`
                                        : (_bt ? `<div class="sv-row sv-bloom"><span class="sv-ico">🌸</span><b>Bloom</b> ${_bt}</div>` : '')}
                                </div>
                                ${p.extra_details ? `<div class="svd-desc">${svEscHtml(p.extra_details)}</div>` : ''}
                                ${_adminFooter}
                            </div>
                        </div>
                    </div>`;
                const _favs = p.favorited_by || [];
                if (_favs.length > 0) {
                    const _accent = getComputedStyle(document.documentElement).getPropertyValue('--svd-accent').trim() || '#2e7d32';
                    card.style.border = `2px solid ${_accent}`;
                    card.style.setProperty('--fav-glow', _accent + '90');
                    card.classList.add('fav-glow');
                }
                return card;
            }

            if (bannerInfo.isHolo) card.classList.add('holographic-active');
            if (favTheme) {
                const _accentOld = getComputedStyle(document.documentElement).getPropertyValue('--svd-accent').trim() || '#2e7d32';
                card.style.border = `2px solid ${_accentOld}`;
                card.style.setProperty('--fav-glow', _accentOld + '90');
                card.classList.add('fav-glow');
            }

            const isDead = p.died == 1 || (p.flower_bed && p.flower_bed.includes('Dead'));
            if (isDead) card.style.opacity = "0.85";

            const imgStyle = isDead ? 'opacity: 0.35; filter: grayscale(20%) contrast(70%) brightness(1.2); object-fit: cover; width: 100%; height: 100%;' : 'object-fit: cover; width: 100%; height: 100%;';
            const diedBadge = isDead ? '<div style="position:absolute; top:10px; left:10px; background:#ff9a9e; color:white; font-weight:900; padding:4px 10px; border-radius:6px; z-index:10; font-size:0.85rem; box-shadow: 2px 2px 0px rgba(0,0,0,0.1); letter-spacing: 0.5px;">DIED</div>' : '';

            const _allImgs = [
                ...(p.picture ? p.picture.split(',') : []),
                ...(p.additional_pictures ? p.additional_pictures.split(',') : [])
            ].map(s => s && s.trim()).filter(Boolean);
            const _detailHasImg = _allImgs.length > 0;
            const imgSrc = _detailHasImg ? config.imgBase + _allImgs[Math.floor(Math.random() * _allImgs.length)] : '';
            const starHTML = generateStarHTML(p, bannerInfo);
            const theme    = getRandomTheme(false);
            const cardBg   = favTheme ? favTheme.bg : theme.bg;

            const renderTags = (text, typeClass, icon = '') => {
                if (!text) return '';
                return text.split(/[,/&+]| and /i).map(part => {
                    const trimmed = part.trim();
                    if (!trimmed) return '';
                    const dataVal = trimmed.replace(/"/g, '&quot;');
                    return `<span class='tag ${typeClass} ${getTagClass(trimmed)}' data-search="${dataVal}" onclick="event.stopPropagation(); window.runSearch(this.dataset.search)">${icon}${trimmed}</span>`;
                }).join('');
            };

            const classificationTags = `${renderTags(p.year_planted, 'tag-year-', '🗓️ ')}${renderTags(p.family, 'tag-family')}${renderTags(p.company, 'tag-company')}${p.local_store ? renderTags(p.local_store, 'tag-store', '🛒 ') : ''}`;
            const attractsHTML = (p.attracts || '').split(/[,/]/).map(a => {
                const trimmed = a.trim();
                if (!trimmed || trimmed === 'None') return '';
                const dataVal = trimmed.replace(/"/g, '&quot;');
                return `<span class='tag' style="background:#fff3e0; color:#e65100; border:1px solid #ffe0b2;" data-search="${dataVal}" onclick="event.stopPropagation(); window.runSearch(this.dataset.search)">${getAttractsIcon(trimmed)} ${trimmed}</span>`;
            }).join('');

            const qtyParts = (p.qty || '0').toString().split(/[,+]/).map(s => s.trim());
            const companyParts = (p.company || '').split(/[,/&+]| and /i).map(s => s.trim()).filter(s => s);
            let qtyHTML = (qtyParts.length > 1 && qtyParts.length === companyParts.length)
                ? qtyParts.map((q, i) => `<div class="data-item badge-qty" title="${companyParts[i]}">📦 ${q} <span style="font-size:0.8em; opacity:0.8;">${companyParts[i].substr(0,3)}..</span></div>`).join('')
                : `<div class="data-item badge-qty">📦 Qty: ${p.qty || 0}</div>`;

            const growStr  = p.grow_months  || (p.planting_month_start ? p.planting_month_start.substring(0,3) + '-' + p.planting_month_end.substring(0,3) : '');
            const bloomStr = p.bloom_months || (p.month_start ? p.month_start.substring(0,3) + '-' + p.month_end.substring(0,3) : '');
            // Determine bloom bar type based on flower field data
            const _noFlower = v => !v || ['', 'none', 'n/a', 'null', 'unknown'].includes(String(v).trim().toLowerCase());
            const _emptyFlowerFields = [p.bloom_color, p.flower_shape, p.flower_symmetry, p.flower_size].filter(_noFlower).length;
            const isFlowerless = _emptyFlowerFields >= 2; // applies to ALL plants
            const isEvergreen  = p.is_evergreen && isFlowerless;

            const futureBadge = isFuture ? '<div style="position:absolute; top:10px; left:10px; background:linear-gradient(135deg,#7c4dff,#448aff); color:white; font-weight:900; padding:4px 12px; border-radius:6px; z-index:10; font-size:0.85rem; box-shadow:2px 2px 0px rgba(0,0,0,0.1); letter-spacing:0.5px;">Wish List</div>' : '';
            card.innerHTML = `
                <div data-card-bg data-default-bg="${cardBg}" onclick="window.checkClick(event, ${p.id}${isFuture ? ", 'future'" : ''})" style="cursor:pointer; position:relative; display:flex; flex-direction:column; height:100%; background-color: ${cardBg}; border-radius: inherit; overflow: hidden; transition: background-color 0.4s ease;">
                    <div style="position: absolute; top: ${theme.y}%; left: ${theme.x}%; transform: translate(-50%, -50%); font-size: ${theme.size}rem; opacity: 0.05; z-index: 1; pointer-events: none; user-select: none;">${theme.icon}</div>
                    ${config.isAdmin ? `<div onclick="event.stopPropagation()" style="position:absolute; top:10px; right:10px; z-index:30;"><input type="checkbox" class="bulk-checkbox" value="${p.id}"></div>` : ''}
                    <div class="plant-img-box" style="position:relative; z-index:2; background:#fff;">
                        ${_detailHasImg ? `<img src="${imgSrc}" loading="lazy" style="${imgStyle}" onerror="this.onerror=null;this.insertAdjacentHTML('afterend',window._comingSoonSVG());this.remove();">` : _comingSoonSVG()}
                        ${diedBadge}
                        ${futureBadge}
                    </div>
                    <div class="plant-details" style="position:relative; z-index:2; display:flex; flex-direction:column; flex-grow:1; padding: 12px;">
                        <div style="display:flex; justify-content:space-between; align-items:flex-start; gap:10px; margin-bottom: 10px; flex-wrap:nowrap;">
                            <h3 style="margin:0; font-size: 1.15rem; line-height: 1.2; flex:1; word-wrap:break-word;">${p.plant_name} ${p.new_name ? `<br><small style="color:var(--accent-gold); font-size:0.8em;">✨ Now: ${p.new_name}</small>` : ''}</h3>
                            ${starHTML}
                        </div>
                        ${classificationTags ? `<div style="display:flex; flex-wrap:wrap; gap:6px; margin-bottom: 8px;">${classificationTags}</div>` : ''}
                        ${attractsHTML ? `<div style="display:flex; flex-wrap:wrap; gap:6px; margin-bottom: 12px; padding-top: 8px; border-top: 1px dashed rgba(0,0,0,0.08);">${attractsHTML}</div>` : ''}
                        <div style="flex-grow: 1;"></div>
                        <div class="data-grid" style="display:flex; flex-wrap: wrap; gap:10px; margin-bottom: 12px; align-items: center;">
                            ${qtyHTML}
                            ${p.new_total ? `<div class="data-item badge-qty" style="background:#e3f2fd; color:#1565c0;">📊 Total: ${p.new_total}</div>` : ''}
                            <div class="data-item badge-sun">☀️ ${p.sun_exposure || 'N/A'}</div>
                        </div>
                        ${p.also_in ? `<div style="font-size:0.75rem; color:#d81b60; margin-top:5px; font-style:italic;"><strong>Also in:</strong> ${p.also_in}</div>` : ''}
                        <div class="timelines-wrapper" style="background: rgba(0,0,0,0.02); border-radius: 8px; padding: 8px; display:flex; flex-direction:column; gap:6px;">
                            <div style="display:flex; align-items:center; gap:10px;"><strong style="font-size:0.8rem; color:#2e7d32; width:50px;">🌱 Grow</strong> ${isEvergreen ? renderEvergreenMonthBar() : renderSeasonalScaleJS(growStr, 'grow')}</div>
                            <div style="display:flex; align-items:center; gap:10px;"><strong style="font-size:0.8rem; color:${isFlowerless ? '#ce93d8' : '#d81b60'}; width:50px;">🌸 ${isFlowerless ? 'None' : 'Bloom'}</strong> ${isFlowerless ? renderFlowerlessTag() : renderSeasonalScaleJS(bloomStr, 'bloom')}</div>
                        </div>
                    </div>
                </div>
                <div style="padding: 10px; border-top:1px solid #eee; background:#f9f9f9; position:relative; z-index:20;">
                    ${config.isAdmin ? (isFuture
                        ? `<div style="display:flex; gap:5px; margin-bottom:10px;">
                            <button class="btn btn-blue" style="flex:1;" onclick='window.editPlant(${p.id}, "future")'>✏️ Edit</button>
                            <button class="btn" style="flex:1; background:linear-gradient(135deg,#7c4dff,#448aff); color:white;" onclick='window.promoteFuture(${p.id})'>🌱 Add to Garden</button>
                           </div>`
                        : `<div style="display:flex; gap:5px; margin-bottom:10px;"><button class="btn btn-blue" style="flex:1;" onclick='window.editPlant(${p.id})'>✏️ Edit</button><form method="POST" onsubmit="return confirm('Delete?');" style="flex:1;"><input type="hidden" name="id" value="${p.id}"><button name="delete_plant" class="btn btn-red" style="width:100%;">🗑️ Delete</button></form></div>`)
                    : ''}
                </div>
            `;
            return card;
        };

        // ── Append to feed: mobile = rows of 3, desktop = flat grid ──────────
        // Simple view always uses the flat-grid path (wide photo-left cards),
        // never the mobile horizontal rows.
        if (isMobile() && !isSimpleView()) {
            // Mobile: each fetch (3 plants) gets its own horizontal scroll row.
            // Bed-change headers sit between rows in the vertical stack.
            let currentRow = null;
            flatList.forEach(p => {
                if (p.flower_bed !== lastBedName) {
                    // Seal the previous row before the heading
                    if (currentRow) { feedContainer.appendChild(currentRow); currentRow = null; }
                    lastBedName = p.flower_bed;
                    if (!renderedBeds.has(lastBedName)) {
                        renderedBeds.add(lastBedName);
                        const header = document.createElement('div');
                        header.className = 'grid-header';
                        if (lastBedName === 'Died' || lastBedName === 'Dead or Removed') header.classList.add('dead-header');
                        header.innerText = lastBedName || "General Collection";
                        feedContainer.appendChild(header);
                    }
                }
                if (!currentRow) {
                    currentRow = document.createElement('div');
                    currentRow.className = 'mobile-card-row';
                }
                currentRow.appendChild(buildCard(p));
            });
            if (currentRow) feedContainer.appendChild(currentRow);
        } else {
            // Desktop: existing flat grid behavior
            flatList.forEach(p => {
                if (p.flower_bed !== lastBedName) {
                    lastBedName = p.flower_bed;
                    if (!renderedBeds.has(lastBedName)) {
                        renderedBeds.add(lastBedName);
                        const header = document.createElement('div');
                        header.className = 'grid-header';
                        if (lastBedName === 'Died' || lastBedName === 'Dead or Removed') header.classList.add('dead-header');
                        header.innerText = lastBedName || "General Collection";
                        feedContainer.appendChild(header);
                    }
                }
                feedContainer.appendChild(buildCard(p));
            });
        }
        currentPage++;
        refreshNoteBadges();
        refreshDoneBadges();
        // Keep every Simple Tags card the same height
        if (isSimpleView()) { requestAnimationFrame(equalizeSimpleCards); setTimeout(equalizeSimpleCards, 300); }
    } catch (error) { console.error(error); } finally { isLoading = false; }
}

/* ---------------------------------------------------------------------
   Temporary diagnostic for the "catalog stops after 9 plants" report.
   Hidden unless the URL has ?scrolldebug=1 -- normal visitors never see it.
   It surfaces the loader state live, and captures the errors that the
   existing try/catch swallows into console.error where nobody looks.
   Delete this block once the cause is confirmed.
   --------------------------------------------------------------------- */
const _sd = new URLSearchParams(location.search).has('scrolldebug');
const _sdErrs = [];
if (_sd) {
    const box = document.createElement('div');
    box.style.cssText = 'position:fixed;left:8px;bottom:8px;z-index:99999;background:rgba(20,30,18,.94);color:#dff5d8;font:12px/1.5 monospace;padding:10px 12px;border-radius:8px;max-width:92vw;white-space:pre';
    document.body.appendChild(box);
    window.addEventListener('error', function (e) { _sdErrs.push('ERR: ' + e.message); });
    window.addEventListener('unhandledrejection', function (e) { _sdErrs.push('REJECT: ' + ((e.reason && e.reason.message) || e.reason)); });
    const _oe = console.error;
    console.error = function () { _sdErrs.push('caught: ' + Array.prototype.map.call(arguments, String).join(' ')); _oe.apply(console, arguments); };
    setInterval(function () {
        const t = document.getElementById('loading-trigger');
        const r = t ? t.getBoundingClientRect() : null;
        const cards = document.querySelectorAll('#plant-feed .plant-card').length;
        const ajax = performance.getEntriesByType('resource').filter(function (x) { return x.name.indexOf('ajax_load') > -1; }).length;
        box.textContent = [
            'cards=' + cards + '  page=' + currentPage + '  hasMore=' + hasMore + '  loading=' + isLoading,
            'ajaxCalls=' + ajax + '  trigTop=' + (r ? Math.round(r.top) : '-') + '  winH=' + window.innerHeight,
            'onScreen=' + (r ? (r.top < window.innerHeight && r.bottom > 0) : '-') + '  scroll=' + Math.round(window.scrollY) + '/' + document.body.scrollHeight,
            (_sdErrs.length ? _sdErrs.slice(-3).join(String.fromCharCode(10)) : 'no errors seen')
        ].join(String.fromCharCode(10));
    }, 500);
}

const trigger = document.getElementById('loading-trigger');
if (trigger) {
    // WHY THIS RE-OBSERVES RATHER THAN JUST CALLING loadPlants ONCE
    // -----------------------------------------------------------------
    // IntersectionObserver fires only when the intersection STATE CHANGES.
    // It does not keep firing while an element stays on screen.
    //
    // The old line was: if (isIntersecting && hasMore) loadPlants().
    // That loads exactly one page. If those cards do not push the trigger
    // below the fold -- which is what happens on a tall screen, where 9
    // cards do not fill the viewport -- the trigger never leaves the
    // screen, the state never changes, no second callback arrives, and the
    // catalog stops after the first handful showing 'Loading garden...'.
    //
    // Fix: after each load, if the trigger is STILL on screen and there is
    // more to fetch, unobserve then re-observe it. Re-observing makes the
    // browser deliver a fresh initial callback, which loads the next page.
    // That repeats until the page is long enough to push the trigger off
    // screen (ordinary infinite scroll takes over) or hasMore turns false.
    let observer;
    let guard = 0;                     // never loop forever on a bad state
    const pump = async () => {
        if (!hasMore || isLoading || guard > 60) return;
        guard++;
        await loadPlants();
        // Measure on the next frame, once layout has settled with the new
        // cards -- measuring immediately would read the old geometry.
        requestAnimationFrame(() => {
            if (!hasMore) return;
            const r = trigger.getBoundingClientRect();
            if (r.top < window.innerHeight + 100 && r.bottom > -100) {
                observer.unobserve(trigger);
                observer.observe(trigger);
            }
        });
    };
    observer = new IntersectionObserver((e) => { if (e[0].isIntersecting) pump(); }, { rootMargin: '100px' });
    observer.observe(trigger);

    // Belt and braces: if the observer never fires at all (some browsers do
    // not deliver an initial callback for an element already on screen),
    // kick the first page off once after load.
    setTimeout(function () {
        if (!isLoading && hasMore && document.querySelectorAll('#plant-feed .plant-card').length === 0) pump();
    }, 800);
}

// ── QR deep link: auto-open plant popup when ?plant=ID is in the URL ──
(function handleDeepLink() {
    const params = new URLSearchParams(window.location.search);
    const plantId = params.get('plant');
    if (!plantId) return;
    // Fetch the specific plant and open its popup
    fetch(`catalog.php?ajax_load=1&plant_id=${encodeURIComponent(plantId)}&_t=${Date.now()}`)
        .then(r => r.json())
        .then(data => {
            if (data.error || !data.results || !data.results.length) return;
            const p = data.results[0];
            window.plantData = window.plantData || {};
            window.plantData[p.id] = p;
            // Wait a tick for the page to finish rendering
            setTimeout(() => {
                if (window.openTagGuide) window.openTagGuide(p);
            }, 400);
        })
        .catch(err => console.error('Deep link fetch error:', err));
})();

// ── QR deep link: auto-search flower bed when ?bed=Name is in the URL ──
(function handleBedDeepLink() {
    const params = new URLSearchParams(window.location.search);
    const bed = params.get('bed');
    if (!bed) return;
    // Wait for the search system to initialize, then trigger bed search
    setTimeout(() => {
        if (window.runSearch) window.runSearch(bed);
    }, 600);
})();

// ── QR Tags grid — browsable grid of QR codes with plant names ──────────
(function initQrTags() {
    let loaded = false;
    const grid = document.getElementById('qr-tags-grid');
    const btn  = document.getElementById('qr-tags-btn');
    if (!grid || !btn) return;

    window.toggleQrTags = function () {
        const visible = grid.style.display !== 'none';
        if (visible) {
            grid.style.display = 'none';
            btn.textContent = '🏷️ Show QR Tags';
            return;
        }
        grid.style.display = '';
        btn.textContent = '🏷️ Hide QR Tags';
        if (loaded) return;
        loaded = true;
        grid.innerHTML = '<p style="text-align:center;color:#888;padding:20px;">Loading tags…</p>';
        fetch('catalog.php?qr_tags=1&_t=' + Date.now())
            .then(r => r.json())
            .then(data => {
                if (!data.tags || !data.tags.length) {
                    grid.innerHTML = '<p style="text-align:center;color:#888;padding:20px;">No QR tags found. Run generate_qr_codes.py on the Pi first.</p>';
                    return;
                }
                let html = '';
                let lastBed = '';
                data.tags.forEach(t => {
                    const bed = t.flower_bed || 'Unsorted';
                    if (bed !== lastBed) {
                        lastBed = bed;
                        html += `<div class="qr-tags-bed-header">${bed}</div>`;
                    }
                    // Show barcode and/or QR code — display both if both exist
                    const hasBarcode = t.has_barcode == 1;
                    const hasQr = t.has_qr == 1;
                    let imgHtml = '';
                    if (hasBarcode) {
                        imgHtml += `<img src="plant_actions.php?action=barcode&plant_id=${t.id}" alt="Barcode for ${t.plant_name}" loading="lazy" class="tag-barcode-img">`;
                    }
                    if (hasQr) {
                        imgHtml += `<img src="qr/plant/${t.id}.png" alt="QR for ${t.plant_name}" loading="lazy" width="120" height="120" style="display:block;margin:6px auto 0;">`;
                    }
                    const upcLabel = t.barcode ? `<div class="qr-tag-upc">${t.barcode}</div>` : '';
                    html += `<div class="qr-tag-card" data-id="${t.id}">` +
                        imgHtml +
                        `<div class="qr-tag-name">${t.plant_name}</div>` +
                        upcLabel +
                    `</div>`;
                });
                grid.innerHTML = html;
                // Click a card → open the plant popup
                grid.addEventListener('click', e => {
                    const card = e.target.closest('.qr-tag-card');
                    if (!card) return;
                    const pid = card.dataset.id;
                    if (window.plantData[pid] && window.openTagGuide) {
                        window.openTagGuide(window.plantData[pid]);
                    } else {
                        fetch(`catalog.php?ajax_load=1&plant_id=${pid}&_t=${Date.now()}`)
                            .then(r => r.json())
                            .then(d => {
                                if (d.results && d.results[0]) {
                                    window.plantData[pid] = d.results[0];
                                    if (window.openTagGuide) window.openTagGuide(d.results[0]);
                                }
                            });
                    }
                });
            })
            .catch(() => {
                grid.innerHTML = '<p style="text-align:center;color:#c62828;padding:20px;">Error loading tags.</p>';
            });
    };
})();

function parseMonthsToSelects(monthData, startId, endId, fallbackStart, fallbackEnd) {
    const startEl = document.getElementById(startId);
    const endEl = document.getElementById(endId);
    if (!startEl || !endEl) return;
    
    startEl.value = ""; endEl.value = "";
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    
    const setVals = (s, e) => {
        const sMatch = months.find(m => m.toLowerCase() === String(s).trim().toLowerCase().substring(0,3));
        const eMatch = months.find(m => m.toLowerCase() === String(e).trim().toLowerCase().substring(0,3));
        if (sMatch) startEl.value = sMatch;
        if (eMatch) endEl.value = eMatch;
    };

    if (!monthData || String(monthData).trim() === '') {
        if (fallbackStart || fallbackEnd) setVals(fallbackStart || '', fallbackEnd || fallbackStart || '');
        return;
    }

    const str = String(monthData).trim();

    // 12-char bitmap: letters and/or dots, exactly 12 chars.
    // Old check used str.includes('.') which broke all-active bitmaps (no dots).
    if (/^[A-Za-z.]{12}$/.test(str)) {
        let first = -1, last = -1;
        for (let i = 0; i < 12; i++) {
            if (str[i] !== '.') {
                if (first === -1) first = i;
                last = i;
            }
        }
        if (first !== -1) {
            setVals(months[first], months[last]);
            return;
        }
    }
    
    if (str.startsWith('[')) {
        try {
            const arr = JSON.parse(str);
            if (Array.isArray(arr) && arr.length > 0) {
                setVals(months[arr[0] - 1], months[arr[arr.length - 1] - 1]);
                return;
            }
        } catch(e) {}
    }

    const cleanStr = str.replace(/[^a-zA-Z]/g, ' ');
    const parts = cleanStr.split(/\s+/).filter(p => p.length >= 3);
    
    if (parts.length > 0) {
        const validParts = parts.filter(p => months.some(m => m.toLowerCase() === p.substring(0,3).toLowerCase()));
        if (validParts.length > 0) {
            setVals(validParts[0], validParts[validParts.length - 1]);
            return;
        }
    }

    if (fallbackStart || fallbackEnd) {
        setVals(fallbackStart || '', fallbackEnd || fallbackStart || '');
    }
}

window.editPlant = function(id, source) {
    const pf = document.getElementById('plantForm');
    if (pf) pf.classList.add('mobile-open');

    // For future plants, fetch data from the future table if not cached
    if (source === 'future' && !window.plantData[id]) {
        fetch(`catalog.php?ajax_load=1&plant_id=${id}&source=future`)
            .then(r => r.json())
            .then(d => {
                if (d.results && d.results[0]) {
                    window.plantData[id] = d.results[0];
                    window.editPlant(id, source);
                }
            });
        return;
    }

    const data = window.plantData[id];
    if(!data) return;

    // Track if editing a future plant (for save routing)
    const editSource = document.getElementById('edit_source');
    if (editSource) editSource.value = (source === 'future' || data.source_table === 'future_plants') ? 'future' : '';
    window._editingFuture = (source === 'future' || data.source_table === 'future_plants');
    
    const ls = document.getElementById('p_local_store');
    if (ls) ls.value = data.local_store || '';
    window.scrollTo({ top: 0, behavior: 'smooth' });
    
    const setters = {
        'edit_id': data.id, 'p_name': data.plant_name, 'p_family': data.family,
        'p_company': data.company, 'p_sun': data.sun_exposure, 'p_qty': data.qty || '1',
        'p_year': data.year_planted, 'p_zones': data.zones, 'p_water': data.water_needs,
        'p_size_new': data.size, 'p_size': data.size_spacing, 'p_deer': data.deer_resistance,
        'p_plant_type': data.plant_type, 'p_genus': data.genus, 'p_species': data.species,
        'p_est_cost': data.average_cost, 'p_stem_color': data.stem_color, 'p_fert': data.fertilizer,
        'p_extra_details': data.extra_details, 'p_native_status': data.native_status,
        'p_fruit_shape': data.fruit_shape, 'p_fruit_color': data.fruit_color, 'p_fruit_size': data.fruit_size,
        'p_fruit_edibility': data.fruit_edibility, 'p_aromatics': data.aromatics, 'p_flower_size': data.flower_size,
        'p_year_added': data.year_added, 'p_qty_edit': data.qty,
        'p_grow_months': data.grow_months, 'p_bloom_months': data.bloom_months, 'p_space': data.spacing,
        'p_maint': data.maintenance_notes, 'p_attracts': data.attracts, 'p_care': data.care_notes,
        'p_cleanup': data.cleanup_notes, 'p_cuttings': data.cuttings, 'p_admin_todo': data.admin_todo,
        'p_split': data.split_amt, 'p_botanical': data.botanical_name, 'p_soil_ph': data.soil_ph,
        'p_soil_type': data.soil_type, 'p_bloom_color': data.bloom_color, 'p_foliage_color': data.foliage_color,
        'p_toxicity': data.toxicity, 'p_prop': data.propagation_method, 'p_disease': data.disease_susceptibility
    };

    for (const [key, value] of Object.entries(setters)) {
        const el = document.getElementById(key);
        if (el) el.value = value || '';
    }

    parseMonthsToSelects(data.grow_months, 'p_grow_start', 'p_grow_end', data.planting_month_start, data.planting_month_end);
    parseMonthsToSelects(data.bloom_months, 'p_bloom_start', 'p_bloom_end', data.month_start, data.month_end);
    
    const bedSelect = document.getElementById('p_bed');
    if (bedSelect) {
        Array.from(bedSelect.options).forEach(o => o.selected = false);
        const beds = (data.flower_bed || '').split(',').map(s => s.trim());
        Array.from(bedSelect.options).forEach(opt => { if (beds.includes(opt.value)) opt.selected = true; });
    }
    
    if (data.picture) {
        const pt = document.getElementById('preview-thumb');
        if (pt) {
            pt.src = config.imgBase + data.picture;
            pt.style.display = 'block';
        }
    }
    
    const ft = document.getElementById('form-title');
    if(ft) ft.innerText = (window._editingFuture ? "Wish List · Edit: " : "✏️ Edit: ") + data.plant_name;
    const sb = document.getElementById('save_btn');
    if(sb) sb.innerText = window._editingFuture ? "Save & Promote to Garden" : "Update Plant";

    // Show the optional qty logger and set the plant id
    const qls = document.getElementById('qty-log-section');
    if (qls) { qls.style.display = 'block'; }
    const qlid = document.getElementById('qty-log-plant-id');
    if (qlid) qlid.value = data.id;
    const qlmsg = document.getElementById('qty-log-msg');
    if (qlmsg) qlmsg.style.display = 'none';
};

window.resetForm = function() {
    const pf = document.getElementById('plantForm');
    if (pf) pf.reset();
    ['p_year_added','p_qty_edit'].forEach(id => { const el = document.getElementById(id); if (el) el.value = ''; });
    const qls = document.getElementById('qty-log-section');
    if (qls) qls.style.display = 'none';
    const ed = document.getElementById('edit_id');
    if (ed) ed.value = "";
    const es = document.getElementById('edit_source');
    if (es) es.value = "";
    window._editingFuture = false;
    const pt = document.getElementById('preview-thumb');
    if (pt) pt.style.display = 'none';
    const ft = document.getElementById('form-title');
    if (ft) ft.innerText = "➕ Add New Plant";
    const sb = document.getElementById('save_btn');
    if (sb) sb.innerText = "Save Plant";
};

// ── Future plant form submit interceptor ─────────────────────────
(function() {
    const pf = document.getElementById('plantForm');
    if (!pf) return;
    pf.addEventListener('submit', function(e) {
        if (!window._editingFuture) return; // let normal submit proceed
        e.preventDefault();

        const id = parseInt(document.getElementById('edit_id').value);
        if (!id) return;

        // Collect all field values from the hidden inputs + visible fields
        const data = { id, action: 'save_future' };
        const fieldMap = {
            p_name: 'plant_name', p_family: 'family', p_genus: 'genus', p_company: 'company',
            p_local_store: 'local_store', p_sun: 'sun_exposure', p_qty: 'qty', p_year: 'year_planted',
            p_zones: 'zones', p_water: 'water_needs', p_size_new: 'size', p_size: 'size_spacing',
            p_deer: 'deer_resistance', p_space: 'spacing', p_maint: 'maintenance_notes',
            p_attracts: 'attracts', p_care: 'care_notes', p_cleanup: 'cleanup_notes',
            p_cuttings: 'cuttings', p_admin_todo: 'admin_todo', p_split: 'split_amt',
            p_botanical: 'botanical_name', p_soil_ph: 'soil_ph', p_soil_type: 'soil_type',
            p_bloom_color: 'bloom_color', p_foliage_color: 'foliage_color', p_stem_color: 'stem_color',
            p_toxicity: 'toxicity', p_prop: 'propagation_method', p_disease: 'disease_susceptibility',
            p_grow_months: 'grow_months', p_bloom_months: 'bloom_months', p_est_cost: 'average_cost',
            p_extra_details: 'extra_details', p_native_status: 'native_status', p_plant_type: 'plant_type',
            p_species: 'species', p_aromatics: 'aromatics', p_flower_size: 'flower_size',
            p_fruit_shape: 'fruit_shape', p_fruit_color: 'fruit_color', p_fruit_size: 'fruit_size',
            p_fruit_edibility: 'fruit_edibility', p_year_added: 'year_added', p_qty_edit: 'qty',
        };
        for (const [elId, col] of Object.entries(fieldMap)) {
            const el = document.getElementById(elId);
            if (el && el.value !== '') data[col] = el.value;
        }
        // Bed select (multi)
        const bedSel = document.getElementById('p_bed');
        if (bedSel) {
            const beds = Array.from(bedSel.selectedOptions).map(o => o.value);
            if (beds.length) { data.flower_bed = beds.join(', '); data.garden_beds = data.flower_bed; }
        }

        const btn = document.getElementById('save_btn');
        if (btn) { btn.disabled = true; btn.innerText = 'Saving & Promoting...'; }

        // Step 1: Save edits to future_plants
        fetch('plant_actions.php', {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        })
        .then(r => r.json())
        .then(res => {
            if (res.error) throw new Error(res.error);
            // Step 2: Promote to data_source
            return fetch('plant_actions.php', {
                method: 'POST', headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action: 'promote_future', id })
            });
        })
        .then(r => r.json())
        .then(res => {
            if (res.error) throw new Error(res.error);
            alert(`${res.plant_name} has been promoted to your garden! (New ID: ${res.new_id})`);
            // Remove the card from view and reset form
            const card = document.querySelector(`.plant-card[data-id="${id}"][data-source="future"]`);
            if (card) card.remove();
            window._editingFuture = false;
            window.resetForm();
        })
        .catch(err => {
            alert('Error: ' + err.message);
            if (btn) { btn.disabled = false; btn.innerText = 'Save & Promote to Garden'; }
        });
    });
})();

// --- DRAG & DROP LOGIC ---
const dz = document.getElementById('drop-zone');
if (dz) {
    const inp = document.getElementById('plant_image_input');
    dz.addEventListener('click', (e) => { if (e.target !== inp) inp.click(); });
    inp.addEventListener('change', () => {
        if (inp.files && inp.files[0]) {
            const reader = new FileReader();
            reader.onload = (ev) => {
                const img = document.getElementById('preview-thumb');
                if (img) {
                    img.src = ev.target.result;
                    img.style.display = 'block';
                }
            };
            reader.readAsDataURL(inp.files[0]);
        }
    });
    dz.addEventListener('dragover', (e) => { e.preventDefault(); dz.classList.add('dragover'); });
    dz.addEventListener('dragleave', () => { dz.classList.remove('dragover'); });
    dz.addEventListener('drop', (e) => { 
        e.preventDefault(); dz.classList.remove('dragover'); 
        if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
            inp.files = e.dataTransfer.files;
            inp.dispatchEvent(new Event('change'));
        }
    });
}

// --- FORM INTERCEPTOR ---
document.addEventListener('submit', function(e) {
    if (e.target && e.target.id === 'plantForm') {
        const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
        const chars = "JFMAMJJASOND";
        
        function to12CharFormat(startId, endId) {
            const startVal = document.getElementById(startId)?.value;
            const endVal = document.getElementById(endId)?.value;
            if(!startVal || !endVal) return "";
            
            const sIdx = months.indexOf(startVal);
            const eIdx = months.indexOf(endVal);
            if(sIdx === -1 || eIdx === -1) return "";
            
            let result = "";
            for (let i = 0; i < 12; i++) {
                let inRange = false;
                if (sIdx <= eIdx) {
                    inRange = (i >= sIdx && i <= eIdx);
                } else {
                    inRange = (i >= sIdx || i <= eIdx); 
                }
                result += inRange ? chars[i] : '.';
            }
            return result;
        }

        const gmInput = document.getElementById('p_grow_months');
        if (gmInput) {
            const val = to12CharFormat('p_grow_start', 'p_grow_end');
            if(val) gmInput.value = val;
        }

        const bmInput = document.getElementById('p_bloom_months');
        if (bmInput) {
            const val = to12CharFormat('p_bloom_start', 'p_bloom_end');
            if(val) bmInput.value = val;
        }
    }
});

initModalListeners();
initHolographicTilt();
initUserNotes();