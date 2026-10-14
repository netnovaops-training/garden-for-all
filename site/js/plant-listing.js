// =============================================================================
// js/plant-listing.js — Quick Search: compact icon grid, 15 per page, lazy load
//
// Uses catalog.php?ajax_load (same search backend as the Full Catalog).
// =============================================================================
import { startBoogie } from './boogie.js';

(function () {
    'use strict';

    const cfg      = window.PLANT_LISTING_CONFIG || {};
    const imgBase  = cfg.imgBase || '';
    const searchEl = document.getElementById('list-search-box');
    const grid     = document.getElementById('qs-grid');
    const trigger  = document.getElementById('qs-trigger');

    if (!searchEl || !grid) return;

    let currentPage = 1;
    let isLoading   = false;
    let hasMore     = true;
    const PER_PAGE  = 15;

    // ── Build a compact icon-row card ────────────────────────────────────────
    function buildCard(p) {
        const isFuture = p.source_table === 'future_plants';
        const _allImgs = [
            ...(p.picture            ? p.picture.split(',')             : []),
            ...(p.additional_pictures ? p.additional_pictures.split(',') : [])
        ].map(s => s && s.trim()).filter(Boolean);
        const src = _allImgs.length
            ? imgBase + _allImgs[Math.floor(Math.random() * _allImgs.length)]
            : imgBase + 'default.webp';

        const name    = p.plant_name || 'Unknown';
        const qty     = isFuture ? '—' : (p.qty || 0);
        const company = p.company || '—';
        const year    = isFuture ? '—' : (p.year_planted || '—');
        const cost    = p.average_cost ? '$' + parseFloat(p.average_cost).toFixed(2) : '—';
        const notes   = p.admin_todo || '';
        const badge   = isFuture
            ? '<span style="background:linear-gradient(135deg,#7c4dff,#448aff);color:#fff;font-weight:700;font-size:.7rem;padding:2px 8px;border-radius:4px;margin-right:6px;vertical-align:middle;">Wish List</span>'
            : '';

        return `
        <div class="qs-row${isFuture ? ' qs-future' : ''}" style="${isFuture ? 'border-left:3px solid #7c4dff;' : ''}">
            <img class="qs-thumb" src="${src}" loading="lazy" alt="${name}">
            <div class="qs-name" title="${name}">${badge}${name}</div>
            <div class="qs-cell qs-qty">${qty}</div>
            <div class="qs-cell qs-company" title="${company}">${company}</div>
            <div class="qs-cell qs-year">${year}</div>
            <div class="qs-cell qs-cost">${cost}</div>
            ${notes ? `<div class="qs-cell qs-notes" title="${notes.replace(/"/g, '&quot;')}">📋</div>` : `<div class="qs-cell qs-notes"></div>`}
        </div>`;
    }

    // ── Fetch one page ───────────────────────────────────────────────────────
    async function loadPage(reset) {
        if (isLoading) return;
        if (!reset && !hasMore) return;
        isLoading = true;

        if (reset) {
            currentPage = 1;
            hasMore = true;
            grid.innerHTML = '';
        }

        const term = (searchEl.value || '').trim();

        try {
            // TWO CATALOG PAGES PER STEP, BOTH KEPT IN FULL.
            //
            // There used to be a `.slice(0, PER_PAGE)` here, on the assumption
            // that catalog.php returns 9 rows a page. It does not: a SEARCH
            // runs two queries there -- the garden and the wish list -- each
            // LIMIT 9, so a page holds up to 18. Fetching two pages gave up to
            // 36 and the slice threw 18 of them away. They were never shown,
            // because the next step asked for the two pages after those.
            //
            // Nothing is trimmed now. The page size is the server's business
            // and this code no longer needs to know it, which is what stops
            // this breaking again the next time that limit moves.
            const startPage = (currentPage - 1) * 2 + 1;
            const [d1, d2] = await Promise.all([
                fetch('catalog.php?ajax_load=1&page=' + startPage + '&search=' + encodeURIComponent(term) + '&_t=' + Date.now()).then(r => r.json()),
                fetch('catalog.php?ajax_load=1&page=' + (startPage + 1) + '&search=' + encodeURIComponent(term) + '&_t=' + Date.now()).then(r => r.json())
            ]);

            const first = d1.results || [];
            const second = d2.results || [];
            const results = [...first, ...second];

            // THE END IS WHEN THE SERVER RUNS OUT, not when a count falls short
            // of a number this file made up. The second page being empty means
            // there is nothing after it.
            const atEnd = second.length === 0;

            if (results.length === 0) {
                hasMore = false;
                if (currentPage === 1) {
                    grid.innerHTML = '<div class="qs-empty">No plants found 🌱</div>';
                }
                if (trigger) { trigger.innerHTML = `<p style="font-size:1.6rem;font-weight:900;letter-spacing:1px;color:#555;">🪴 End of garden. <span id="boogie-trigger" style="cursor:pointer;text-decoration:underline dotted;color:#d81b60;font-weight:900;font-size:1.8rem;" title="Click me...">Boogie? 🕺</span></p>`; document.getElementById('boogie-trigger')?.addEventListener('click', () => startBoogie(cfg.imgBase || '/images/Database/')); }
                return;
            }

            grid.innerHTML += results.map(buildCard).join('');
            currentPage++;

            if (atEnd) {
                hasMore = false;
                if (trigger) { trigger.innerHTML = `<p style="font-size:1.6rem;font-weight:900;letter-spacing:1px;color:#555;">🪴 End of garden. <span id="boogie-trigger" style="cursor:pointer;text-decoration:underline dotted;color:#d81b60;font-weight:900;font-size:1.8rem;" title="Click me...">Boogie? 🕺</span></p>`; document.getElementById('boogie-trigger')?.addEventListener('click', () => startBoogie(cfg.imgBase || '/images/Database/')); }
            }
        } catch (e) {
            console.error('Quick search error:', e);
        } finally {
            isLoading = false;
        }
    }

    // ── Infinite scroll ──────────────────────────────────────────────────────
    if (trigger) {
        new IntersectionObserver(function (entries) {
            if (entries[0].isIntersecting && hasMore && !isLoading) loadPage(false);
        }, { rootMargin: '300px' }).observe(trigger);
    }

    // ── Debounced search ─────────────────────────────────────────────────────
    let timer;
    searchEl.addEventListener('input', function () {
        clearTimeout(timer);
        timer = setTimeout(function () { loadPage(true); }, 300);
    });

    // ── Initial load ─────────────────────────────────────────────────────────
    loadPage(true);
}());
