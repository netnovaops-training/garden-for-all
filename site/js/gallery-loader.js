/**
 * gallery-loader.js
 * Loaded synchronously in <head> so window.galleryLoaderDismiss is defined
 * before the gallery JS runs. The overlay HTML is rendered inline by PHP.
 * Call window.galleryLoaderDismiss() when the gallery is ready to reveal.
 */
(function () {
    'use strict';

    var _dismissed = false;
    var _startTime = Date.now();
    var MIN_SHOW_MS = 1400; // minimum time the loader is visible (feels intentional)

    /**
     * Fade out and remove #gal-loader.
     * Respects MIN_SHOW_MS so the animation is always seen.
     */
    window.galleryLoaderDismiss = function () {
        if (_dismissed) return;
        var elapsed = Date.now() - _startTime;
        var wait    = Math.max(0, MIN_SHOW_MS - elapsed);

        setTimeout(function () {
            if (_dismissed) return;
            _dismissed = true;

            var el = document.getElementById('gal-loader');
            if (!el) return;

            el.style.transition = 'opacity 0.7s ease';
            el.style.opacity    = '0';

            setTimeout(function () {
                if (el && el.parentNode) el.parentNode.removeChild(el);
            }, 750);
        }, wait);
    };
}());
