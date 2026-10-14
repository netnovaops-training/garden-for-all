// js/garden-chronicles.js
// Fetches all Garden Chronicles cards from /sections/cards/index.php,
// shuffles them with Fisher-Yates, and renders them into
// #garden-chronicles-container on index.php.
// To add a new card: drop a card-XX-name.json file into /sections/cards/
// — no code changes needed.

(function () {
    'use strict';

    // ── Fisher-Yates shuffle ───────────────────────────────────────────────────
    function shuffle(arr) {
        const a = arr.slice();
        for (let i = a.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [a[i], a[j]] = [a[j], a[i]];
        }
        return a;
    }

    // ── Build one card's HTML ──────────────────────────────────────────────────
    function buildCard(card) {
        const themeClass = 'bg-theme-' + (Number(card.theme) % 6);

        const tagsHTML = (card.tags || []).map(t =>
            `<span class="blog-cartoony-text chronicles-tag">
                ${t.emoji} ${t.label}
            </span>`
        ).join('');

        return `
            <div class="plant-card cartoony-card holographic-active ${themeClass} chronicles-card">
                <div class="chronicles-img-box">
                    <span class="chronicles-emoji">${card.emoji}</span>
                </div>
                <div class="plant-details blog-cartoony-text chronicles-body">
                    <h3 class="blog-cartoony-text chronicles-title">${card.title}</h3>
                    <div class="chronicles-tags">${tagsHTML}</div>
                    <div class="blog-cartoony-text chronicles-text">${card.body}</div>
                </div>
            </div>`;
    }

    // ── Main ──────────────────────────────────────────────────────────────────
    function init() {
        const container = document.getElementById('garden-chronicles-container');
        if (!container) return;

        fetch('sections/cards/index.php')
            .then(function (res) {
                if (!res.ok) throw new Error('HTTP ' + res.status);
                return res.json();
            })
            .then(function (cards) {
                if (!Array.isArray(cards) || !cards.length) {
                    container.innerHTML = '<p style="text-align:center;width:100%;">No cards found.</p>';
                    return;
                }
                const shuffled = shuffle(cards).slice(0, 3);
                container.innerHTML = shuffled.map(buildCard).join('');
            })
            .catch(function (err) {
                console.error('Garden Chronicles failed to load:', err);
                container.innerHTML = '<p style="text-align:center;width:100%;color:#c00;">Could not load cards.</p>';
            });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
