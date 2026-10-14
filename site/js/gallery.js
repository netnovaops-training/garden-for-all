// =============================================================================
// js/gallery.js — Infinite 45° brick-scroll gallery
// Dual-canvas crossfade | thumbnail preloader | lightbox | tag search
//
// PHP passes data via a <script> tag:
//   <script id="gallery-data"
//           data-photos="<?= htmlspecialchars(json_encode($photos)) ?>"
//           src="js/gallery.js?v=1"></script>
// =============================================================================

(function () {
    'use strict';

    // ── Photo data injected by PHP ────────────────────────────────
    const dataEl    = document.getElementById('gallery-data');
    const ALL_PHOTOS = JSON.parse(dataEl ? dataEl.dataset.photos : '[]');

    // ── Layout constants ──────────────────────────────────────────
    const GAP      = 6;
    const VW        = window.innerWidth;
    // Column width — tuned per breakpoint
    const COL_W     = VW < 380  ? 140   // small phones: 2 cols
                    : VW < 540  ? 185   // large phones (S24 Ultra): 2 cols
                    : VW < 768  ? 200   // iPad mini / portrait tablet
                    : VW < 1025 ? 220   // iPad Air / Pro portrait & landscape
                    : 240;              // desktop

    // Three-way device class — drives which thumbnail tier to use
    // Mobile  ≤ 600px  → thumbMob    (400px)
    // Tablet  601–1024 → thumbTablet (600px)
    // Desktop > 1024px → thumbWeb    (800px)
    const IS_MOBILE  = VW <= 600;
    const IS_TABLET  = VW > 600 && VW <= 1024;
    // IS_DESKTOP = !IS_MOBILE && !IS_TABLET (implicit)
    const SPEED     = 20;    // px/s drift
    const MIN_POOL  = 80;    // repeat photos until pool reaches this size
    const FADE_MS   = 1000;  // crossfade duration (ms)

    // ── State ─────────────────────────────────────────────────────
    let currentPhotos = [...ALL_PHOTOS];
    let animId        = null;
    let lbIdx         = 0;
    let tileW = 0, tileH = 0, tx = 0, ty = 0, lastTs = 0;
    let videoObserver = null;
    let activeCanvas  = null;
    let standbyCanvas = null;
    let isCrossfading = false;

    // ── Pool helper ───────────────────────────────────────────────
    function expandPool(photos) {
        if (!photos.length) return [];
        let pool = [...photos];
        while (pool.length < MIN_POOL) pool = [...pool, ...photos];
        return pool;
    }

    // ── Layout builder ────────────────────────────────────────────
    function buildLayout(photos) {
        const numCols = Math.ceil(window.innerWidth / (COL_W + GAP)) + 4;
        const heights = new Array(numCols).fill(0);
        const items   = [];
        const pool    = expandPool(photos);

        pool.forEach(photo => {
            let ar = (photo.w && photo.h) ? (photo.w / photo.h) : 1.778;
            if (!isFinite(ar) || ar <= 0) ar = 1.778;
            const imgH = Math.round(COL_W / ar);

            let col = 0;
            for (let c = 1; c < numCols; c++) {
                if (heights[c] < heights[col]) col = c;
            }

            items.push({ photo, x: col * (COL_W + GAP), y: heights[col], w: COL_W, h: imgH });
            heights[col] += imgH + GAP;
        });

        const tw = numCols * (COL_W + GAP);
        const th = Math.max(Math.max(...heights), window.innerHeight * 2);
        return { items, tileW: tw, tileH: th };
    }

    // ── Canvas builder ────────────────────────────────────────────
    function buildCanvasEl(photos, el, eagerLoad) {
        el.textContent = '';

        const emptyEl = document.getElementById('gallery-empty');
        if (!photos.length) {
            emptyEl.classList.add('on');
            stopAnim();
            return null;
        }
        emptyEl.classList.remove('on');

        const { items, tileW: tw, tileH: th } = buildLayout(photos);
        el.style.width  = (3 * tw) + 'px';
        el.style.height = (3 * th) + 'px';

        const frag = document.createDocumentFragment();

        for (let row = 0; row < 3; row++) {
            for (let col = 0; col < 3; col++) {
                const ox       = col * tw;
                const oy       = row * th;
                const isCentre = (row === 1 && col === 1);

                items.forEach(item => {
                    const div      = document.createElement('div');
                    div.className  = 'photo-item';
                    div.style.left   = (item.x + ox) + 'px';
                    div.style.top    = (item.y + oy) + 'px';
                    div.style.width  = item.w + 'px';
                    div.style.height = item.h + 'px';

                    if (item.photo.type === 'video') {
                        const thumb = IS_MOBILE
                            ? (item.photo.thumbMob    || item.photo.thumbTablet || item.photo.thumbWeb)
                            : IS_TABLET
                            ? (item.photo.thumbTablet || item.photo.thumbWeb    || item.photo.thumbMob)
                            : (item.photo.thumbWeb    || item.photo.thumbTablet || item.photo.thumbMob);

                        if (thumb) {
                            const img    = document.createElement('img');
                            img.src      = thumb;
                            img.alt      = item.photo.name || '';
                            img.loading  = eagerLoad ? 'eager' : 'lazy';
                            img.decoding = 'async';
                            div.appendChild(img);
                        } else {
                            const vid       = document.createElement('video');
                            vid.src         = item.photo.src;
                            vid.muted       = true;
                            vid.loop        = true;
                            vid.playsInline = true;
                            vid.preload     = isCentre ? 'auto' : 'none';
                            div.appendChild(vid);
                        }

                        const badge       = document.createElement('div');
                        badge.className   = 'video-badge';
                        badge.textContent = '▶';
                        div.appendChild(badge);

                    } else {
                        // GIFs: full src (animates); images: thumbnail
                        const thumb = item.photo.type === 'gif'
                            ? item.photo.src
                            : (IS_MOBILE
                                ? (item.photo.thumbMob || item.photo.thumbWeb || item.photo.src)
                                : (item.photo.thumbWeb || item.photo.thumbMob || item.photo.src));

                        const img    = document.createElement('img');
                        img.src      = thumb;
                        img.alt      = item.photo.name || '';
                        img.loading  = eagerLoad ? 'eager' : 'lazy';
                        img.decoding = 'async';
                        div.appendChild(img);
                    }

                    const photoSrc = item.photo.src;
                    div.addEventListener('click', () => {
                        const idx = currentPhotos.findIndex(p => p.src === photoSrc);
                        if (idx >= 0) lbOpen(idx);
                    });

                    frag.appendChild(div);
                });
            }
        }

        el.appendChild(frag);
        return { tileW: tw, tileH: th };
    }

    // ── Video observer ────────────────────────────────────────────
    function setupVideoObserver() {
        if (videoObserver) videoObserver.disconnect();
        const stage = document.getElementById('gallery-stage');
        videoObserver = new IntersectionObserver(entries => {
            entries.forEach(e => {
                if (e.isIntersecting) e.target.play().catch(() => {});
                else                  e.target.pause();
            });
        }, { root: stage, rootMargin: '80px', threshold: 0.01 });
        activeCanvas.querySelectorAll('video').forEach(v => videoObserver.observe(v));
    }

    // ── Animation loop ────────────────────────────────────────────
    function tick(ts) {
        if (!lastTs) lastTs = ts;
        const dt = Math.min((ts - lastTs) / 1000, 0.05);
        lastTs = ts;

        tx += SPEED * dt;
        ty -= SPEED * dt;

        if (tx >= 0) { crossfadeToNext(); animId = requestAnimationFrame(tick); return; }
        if (ty <= -2 * tileH) ty += tileH;

        activeCanvas.style.transform = `translate3d(${tx}px,${ty}px,0)`;
        animId = requestAnimationFrame(tick);
    }

    function startAnim() {
        if (animId) return;
        lastTs = 0;
        animId = requestAnimationFrame(tick);
    }

    function stopAnim() {
        if (animId) { cancelAnimationFrame(animId); animId = null; }
    }

    // ── Crossfade between loops ───────────────────────────────────
    function crossfadeToNext() {
        if (isCrossfading) {
            tx -= tileW;
            if (ty <= -2 * tileH) ty += tileH;
            activeCanvas.style.transform = `translate3d(${tx}px,${ty}px,0)`;
            return;
        }
        isCrossfading = true;

        tx -= tileW;
        if (ty <= -2 * tileH) ty += tileH;
        activeCanvas.style.transform = `translate3d(${tx}px,${ty}px,0)`;

        // Shuffle for next pass
        for (let i = currentPhotos.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [currentPhotos[i], currentPhotos[j]] = [currentPhotos[j], currentPhotos[i]];
        }

        if (videoObserver) videoObserver.disconnect();
        const result = buildCanvasEl(currentPhotos, standbyCanvas, true);
        if (!result) { isCrossfading = false; return; }

        const ntx = -result.tileW;
        const nty = -result.tileH;

        standbyCanvas.style.transition = 'none';
        standbyCanvas.style.opacity    = '0';
        standbyCanvas.style.zIndex     = '2';
        standbyCanvas.style.transform  = `translate3d(${ntx}px,${nty}px,0)`;

        requestAnimationFrame(() => {
            activeCanvas.style.transition  = `opacity ${FADE_MS}ms ease`;
            standbyCanvas.style.transition = `opacity ${FADE_MS}ms ease`;
            activeCanvas.style.opacity     = '0';
            standbyCanvas.style.opacity    = '1';

            setTimeout(() => {
                const tmp     = activeCanvas;
                activeCanvas  = standbyCanvas;
                standbyCanvas = tmp;

                tileW = result.tileW;
                tileH = result.tileH;
                tx    = ntx;
                ty    = nty;

                standbyCanvas.textContent      = '';
                standbyCanvas.style.opacity    = '';
                standbyCanvas.style.transition = '';
                standbyCanvas.style.zIndex     = '';
                activeCanvas.style.transition  = '';
                activeCanvas.style.opacity     = '';
                activeCanvas.style.zIndex      = '';

                isCrossfading = false;
                setupVideoObserver();
            }, FADE_MS + 60);
        });
    }

    // ── Start / restart gallery ───────────────────────────────────
    function startGallery(photos) {
        stopAnim();
        isCrossfading = false;
        if (videoObserver) { videoObserver.disconnect(); videoObserver = null; }
        lastTs = 0;

        const result = buildCanvasEl(photos, activeCanvas, false);
        standbyCanvas.textContent      = '';
        standbyCanvas.style.opacity    = '';
        standbyCanvas.style.transition = '';
        standbyCanvas.style.zIndex     = '';
        activeCanvas.style.opacity     = '';
        activeCanvas.style.transition  = '';
        activeCanvas.style.zIndex      = '';

        if (!result) return;

        tileW = result.tileW;
        tileH = result.tileH;
        tx    = -tileW;
        ty    = -tileH;

        activeCanvas.style.transform = `translate3d(${tx}px,${ty}px,0)`;

        setupVideoObserver();
        startAnim();

        // Dismiss loader
        const loader = document.getElementById('gal-loader');
        if (loader) {
            loader.classList.add('fade-out');
            setTimeout(() => { loader.style.display = 'none'; }, 520);
        }
        // Begin background full-res preload after loader fades
        setTimeout(() => scheduleFullResPreload(photos), 600);
    }

    // ── Thumbnail preloader with progress bar ─────────────────────
    function preloadAllThumbs(photos, onComplete) {
        const fillEl  = document.getElementById('gal-progress-fill');
        const labelEl = document.getElementById('gal-loader-label');

        const urls = [];
        const seen = new Set();
        photos.forEach(p => {
            const url = p.type === 'gif'
                ? p.src
                : IS_MOBILE
                ? (p.thumbMob    || p.thumbTablet || p.thumbWeb || p.src)
                : IS_TABLET
                ? (p.thumbTablet || p.thumbWeb    || p.thumbMob || p.src)
                : (p.thumbWeb    || p.thumbTablet || p.thumbMob || p.src);
            if (url && !seen.has(url)) { seen.add(url); urls.push(url); }
        });

        const total = urls.length;
        if (total === 0) { onComplete(); return; }

        let loaded = 0;

        function onLoad() {
            loaded++;
            const pct = Math.round((loaded / total) * 100);
            if (fillEl)  fillEl.style.width  = pct + '%';
            if (labelEl) labelEl.textContent = loaded + ' / ' + total;
            if (loaded >= total) onComplete();
        }

        const BATCH = 6;
        let   next  = 0;

        function loadNext() {
            while (next < total && (next - loaded) < BATCH) {
                const url = urls[next++];
                const img = new Image();
                img.onload = img.onerror = () => { onLoad(); loadNext(); };
                img.src = url;
            }
        }
        loadNext();
    }

    // ── Background full-res preloader ─────────────────────────────
    const _preloadCache = {};
    let   _preloadQueue  = [];
    let   _preloadActive = 0;
    const PRELOAD_CONCURRENCY = 3;

    function scheduleFullResPreload(photos) {
        _preloadQueue = photos
            .filter(p => p.type === 'image' && p.src && !_preloadCache[p.src])
            .map(p => p.src);
        _drainPreloadQueue();
    }

    function _drainPreloadQueue() {
        while (_preloadActive < PRELOAD_CONCURRENCY && _preloadQueue.length) {
            const src = _preloadQueue.shift();
            if (_preloadCache[src]) { _drainPreloadQueue(); return; }
            _preloadCache[src] = true;
            _preloadActive++;
            const img = new Image();
            img.onload = img.onerror = () => { _preloadActive--; _drainPreloadQueue(); };
            img.src = src;
        }
    }

    // ── Search ────────────────────────────────────────────────────
    const searchInput = document.getElementById('search-input');
    const searchClear = document.getElementById('search-clear');
    const resultEl    = document.getElementById('search-result');
    let   searchTimer = null;

    function applySearch(q) {
        q = q.trim().toLowerCase();
        const filtered = q === ''
            ? [...ALL_PHOTOS]
            : ALL_PHOTOS.filter(p => (p.tags || []).some(t => t.toLowerCase().includes(q)));
        currentPhotos = filtered;

        if (q) {
            resultEl.textContent   = filtered.length + ' photo' + (filtered.length === 1 ? '' : 's') + ' found';
            resultEl.style.display = 'block';
        } else {
            resultEl.style.display = 'none';
        }
        startGallery(filtered);
    }

    searchInput.addEventListener('input', () => {
        searchClear.style.display = searchInput.value ? 'block' : 'none';
        clearTimeout(searchTimer);
        searchTimer = setTimeout(() => applySearch(searchInput.value), 280);
    });

    searchClear.addEventListener('click', () => {
        searchInput.value = '';
        searchClear.style.display = 'none';
        applySearch('');
        searchInput.focus();
    });

    // ── Lightbox ──────────────────────────────────────────────────
    const PRELOAD_AHEAD = 3;
    const preloaded     = {};
    const lbEl  = document.getElementById('lb');
    const lbImg = document.getElementById('lb-img');
    const lbVid = document.getElementById('lb-video');

    function lbOpen(idx) {
        lbIdx = idx;
        lbShow();
        lbEl.classList.add('on');
        document.body.style.overflow = 'hidden';
        stopAnim();
    }

    // Exposed globally so inline onclick attributes in the HTML still work
    window.lbClose = function () {
        lbVid.pause();
        lbVid.src          = '';
        lbVid.style.display = 'none';
        lbImg.style.display = '';
        lbEl.classList.remove('on');
        document.body.style.overflow = '';
        lastTs = 0;
        startAnim();
    };

    window.lbMove = function (d) {
        lbIdx = (lbIdx + d + currentPhotos.length) % currentPhotos.length;
        lbShow();
    };

    function lbShow() {
        const p = currentPhotos[lbIdx];

        if (p.type === 'video') {
            lbImg.style.display  = 'none';
            lbVid.style.display  = 'block';
            lbVid.src = p.src;
            lbVid.load();
            lbVid.play().catch(() => {});
        } else {
            lbVid.pause();
            lbVid.style.display  = 'none';
            lbImg.style.display  = '';
            lbImg.style.opacity  = '0';
            lbImg.src = p.src;
            lbImg.alt = p.name || '';
            lbImg.onload = () => { lbImg.style.opacity = '1'; };
        }

        document.getElementById('lb-title').textContent = p.title || p.name || '';
        document.getElementById('lb-desc').textContent  = p.desc  || '';
        document.getElementById('lb-name').textContent  = p.name  || '';

        const tagsEl = document.getElementById('lb-tags');
        tagsEl.innerHTML = '';
        (p.tags || []).forEach(t => {
            const s = document.createElement('span');
            s.className   = 'lb-tag';
            s.textContent = t;
            tagsEl.appendChild(s);
        });

        document.getElementById('lb-ctr').textContent = (lbIdx + 1) + ' / ' + currentPhotos.length;

        const dlEl = document.getElementById('lb-download');
        if (p.downloadUrl) {
            dlEl.href = p.downloadUrl;
            dlEl.setAttribute('download', (p.name || 'download') + '.' + p.downloadUrl.split('.').pop());
            dlEl.classList.add('visible');
        } else {
            dlEl.classList.remove('visible');
        }

        // Preload upcoming images
        for (let i = 1; i <= PRELOAD_AHEAD; i++) {
            const n = (lbIdx + i) % currentPhotos.length;
            const m = currentPhotos[n];
            if (m.type === 'image' && !preloaded[m.src]) {
                const x = new Image(); x.src = m.src; preloaded[m.src] = x;
            }
        }
    }

    // Keyboard navigation
    document.addEventListener('keydown', e => {
        if (!lbEl.classList.contains('on')) return;
        if (e.key === 'ArrowLeft'  || e.key === 'ArrowUp')    window.lbMove(-1);
        if (e.key === 'ArrowRight' || e.key === 'ArrowDown')   window.lbMove(1);
        if (e.key === 'Escape')                                window.lbClose();
    });

    lbEl.addEventListener('click', e => { if (e.target === lbEl) window.lbClose(); });

    // Touch swipe
    let _touchX = null;
    lbEl.addEventListener('touchstart', e => { _touchX = e.touches[0].clientX; }, { passive: true });
    lbEl.addEventListener('touchend', e => {
        if (_touchX === null) return;
        if (Math.abs(e.changedTouches[0].clientX - _touchX) > 40)
            window.lbMove(e.changedTouches[0].clientX < _touchX ? 1 : -1);
        _touchX = null;
    }, { passive: true });

    // ── Boot ──────────────────────────────────────────────────────
    activeCanvas  = document.getElementById('gallery-canvas-a');
    standbyCanvas = document.getElementById('gallery-canvas-b');

    if (ALL_PHOTOS.length === 0) {
        document.getElementById('gallery-empty').classList.add('on');
        document.getElementById('empty-msg').textContent =
            'Add photos or videos to images/Gallery/ to get started.';
        const loader = document.getElementById('gal-loader');
        if (loader) { loader.classList.add('fade-out'); setTimeout(() => { loader.style.display = 'none'; }, 520); }
    } else {
        preloadAllThumbs(ALL_PHOTOS, () => startGallery(ALL_PHOTOS));
    }

})();
