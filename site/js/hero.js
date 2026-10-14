// =============================================================================
// js/hero.js — Hero banner: facts rotation + deferred slide loading
//
// Requires window.SITE_CONFIG to be set before this script runs (done by
// the inline <script> block in layout_open.php).
// Only runs when a #hero-banner element exists on the page.
// =============================================================================
(function () {
    'use strict';

    document.addEventListener('DOMContentLoaded', function () {
        const banner  = document.getElementById('hero-banner');
        if (!banner) return;  // not a hero page

        const overlay = document.getElementById('overlay-box');
        const factEl  = document.getElementById('fun-fact-text');
        const cfg     = window.SITE_CONFIG || {};
        const facts   = cfg.allFacts || [];

        // Facts rotation
        function updateFact() {
            if (factEl && facts.length > 0)
                factEl.innerText = facts[Math.floor(Math.random() * facts.length)];
        }
        updateFact();
        setInterval(updateFact, 9000);

        // Slide rotation
        let cur = 0;
        setInterval(function () {
            const slides = banner.querySelectorAll('.hero-slide');
            if (slides.length <= 1) return;
            slides[cur % slides.length].classList.remove('active');
            cur = (cur + 1) % slides.length;
            slides[cur].classList.add('active');
        }, 8000);

        // Drip in deferred images after full page load
        window.addEventListener('load', function () {
            const pool = (cfg.deferredImages || [])
                .sort(() => Math.random() - 0.5)
                .slice(0, 12);

            pool.forEach(function (src, i) {
                setTimeout(function () {
                    const div = document.createElement('div');
                    div.className = 'hero-slide';
                    div.style.backgroundImage = "url('" + src + "')";
                    banner.insertBefore(div, overlay);
                }, i * 300);
            });
        });
    });
}());
