// js/modal.js
import { applyScrollFadeEffects } from './effects.js';
import { generateLeafSVG, generateFlowerSVG } from './plant-morphology-svg.js';
import {
    getBannerState,
    generateStarHTML,
    getRandomTheme,
    getFavoriteTheme,
    renderSeasonalScaleJS,
    generateSunWidgetHTML,
    generateWaterCupHTML,
    generateZoneWidgetHTML,
    generateRulerWidgetHTML,
    generateFertilizerWidgetHTML,
    generateAttractsWidgetHTML,
    generateAromaticsWidgetHTML,
    generatePhWidgetHTML,
    generateCostWidgetHTML,
    generateQuantityWidgetHTML,
    generateToxicityWidgetHTML,
    generateDeerRabbitWidgetHTML,
    generateEdibilityWidgetHTML,
    generateGrowthRateWidgetHTML,
    generateChillingHoursWidgetHTML,
    generateLifeExpectancyWidgetHTML,
    generateFruitSVG,
    generateSoilTypeWidgetHTML,
    generatePlantTypeHTML,
    getFamilyDisplayName,
    generateInvasiveHTML,
    generateNativeStatusHTML,
    generatePlantingDepthHTML,
    renderEvergreenMonthBar,
    renderFlowerlessTag
} from './ui-widgets.js';

let isModalOpen = false;
let modalThemeIndex = 0;

// ── Lightbox photo-cycling state ──────────────────────────────────────────
let _lbImgs = [];
let _lbIdx = 0;
function lbGoto(delta) {
    if (_lbImgs.length < 2) return;
    _lbIdx = (_lbIdx + delta + _lbImgs.length) % _lbImgs.length;
    const img = document.getElementById('img-lightbox-img');
    if (img) img.src = _lbImgs[_lbIdx];
}

// ── Lightbox — injected once, reused for every image click ───────────────
(function initLightbox() {
    if (document.getElementById('img-lightbox')) return;
    const lb = document.createElement('div');
    lb.id = 'img-lightbox';
    lb.innerHTML = `
        <span id="img-lightbox-close" title="Close">✕</span>
        <div id="img-lightbox-prev" class="img-lb-nav" title="Previous photo" style="display:none;position:fixed;top:50%;left:16px;transform:translateY(-50%);z-index:100053;width:54px;height:54px;border-radius:50%;background:rgba(0,0,0,0.45);color:#fff;font-size:2.4rem;line-height:50px;text-align:center;cursor:pointer;user-select:none;backdrop-filter:blur(4px);">‹</div>
        <div id="img-lightbox-next" class="img-lb-nav" title="Next photo" style="display:none;position:fixed;top:50%;right:16px;transform:translateY(-50%);z-index:100053;width:54px;height:54px;border-radius:50%;background:rgba(0,0,0,0.45);color:#fff;font-size:2.4rem;line-height:50px;text-align:center;cursor:pointer;user-select:none;backdrop-filter:blur(4px);">›</div>
        <img id="img-lightbox-img" src="" alt="">
        <div id="img-lightbox-footer" style="position:fixed;bottom:0;left:0;right:0;display:flex;flex-direction:column;align-items:center;gap:8px;padding:12px 16px;background:rgba(0,0,0,0.65);backdrop-filter:blur(6px);z-index:100001;pointer-events:auto;" onclick="event.stopPropagation()">
            <div style="display:flex;gap:10px;align-items:center;width:100%;max-width:520px;">
                <button id="img-lightbox-like" class="cartoony-btn thumb-btn" style="font-size:1.2rem;padding:8px 18px;flex-shrink:0;">
                    👍 Like <span id="img-lightbox-count">0</span>
                </button>
                <input id="lb-comment-name" placeholder="Name" maxlength="30"
                       style="flex:0 0 110px;padding:7px 10px;border-radius:20px;border:2px solid #aaa;font-size:0.82rem;background:rgba(255,255,255,0.9);">
                <input id="lb-comment-msg" placeholder="Leave a comment…" maxlength="200"
                       style="flex:1;padding:7px 10px;border-radius:20px;border:2px solid #aaa;font-size:0.82rem;background:rgba(255,255,255,0.9);">
                <button id="lb-comment-btn" style="padding:7px 14px;border-radius:20px;background:#2e7d32;color:#fff;border:none;font-weight:700;cursor:pointer;font-size:0.85rem;flex-shrink:0;">Post</button>
            </div>
            <div id="lb-comments-list" style="width:100%;max-width:520px;max-height:80px;overflow-y:auto;font-size:0.78rem;color:#eee;text-align:left;"></div>
        </div>`;
    document.body.appendChild(lb);
    const close = () => { lb.classList.remove('open'); document.getElementById('img-lightbox-img').src = ''; };
    lb.addEventListener('click', e => { if (e.target === lb || e.target.id === 'img-lightbox-close') close(); });
    document.getElementById('img-lightbox-prev').addEventListener('click', e => { e.stopPropagation(); lbGoto(-1); });
    document.getElementById('img-lightbox-next').addEventListener('click', e => { e.stopPropagation(); lbGoto(1); });
    document.addEventListener('keydown', e => {
        if (!lb.classList.contains('open')) return;
        if (e.key === 'Escape') close();
        else if (e.key === 'ArrowLeft') lbGoto(-1);
        else if (e.key === 'ArrowRight') lbGoto(1);
    });
})();

window.openLightbox = function(src, plantId) {
    const lb    = document.getElementById('img-lightbox');
    const img   = document.getElementById('img-lightbox-img');
    const likeBtn = document.getElementById('img-lightbox-like');
    const likeCount = document.getElementById('img-lightbox-count');
    if (!lb || !img) return;
    img.src = src;

    // Build this plant's photo list so the lightbox can cycle through them
    const _cfg  = window.GARDEN_CONFIG || {};
    const _base = _cfg.imgBase || '/images/Database/';
    const _pd   = window.plantData && window.plantData[plantId];
    let _list = [];
    if (_pd) {
        _list = [
            ...(_pd.picture ? _pd.picture.split(',') : []),
            ...(_pd.additional_pictures ? _pd.additional_pictures.split(',') : [])
        ].map(s => s && s.trim()).filter(Boolean).map(f => _base + f);
        _list = [...new Set(_list)];
    }
    if (!_list.length) _list = [src];
    _lbImgs = _list;
    _lbIdx  = Math.max(0, _list.indexOf(src));
    const _showNav = _list.length > 1;
    const _pv = document.getElementById('img-lightbox-prev');
    const _nx = document.getElementById('img-lightbox-next');
    if (_pv) _pv.style.display = _showNav ? 'block' : 'none';
    if (_nx) _nx.style.display = _showNav ? 'block' : 'none';

    // Sync like button state + count from the modal's existing like button
    const modalLikeBtn = document.getElementById('like-btn-' + plantId);
    const modalCount   = document.getElementById('like-count-' + plantId);
    if (likeCount && modalCount) likeCount.textContent = modalCount.textContent;
    if (likeBtn && modalLikeBtn) likeBtn.style.opacity = modalLikeBtn.style.opacity;

    // Wire up — calls the same toggleLike used in the modal
    if (likeBtn) {
        likeBtn.onclick = e => {
            e.stopPropagation();
            likeBtn.classList.add('pop-anim');
            setTimeout(() => likeBtn.classList.remove('pop-anim'), 300);
            if (window.toggleLike) window.toggleLike(plantId);
            // Keep lightbox count in sync after toggle
            setTimeout(() => {
                const mc = document.getElementById('like-count-' + plantId);
                const mb = document.getElementById('like-btn-' + plantId);
                if (mc && likeCount) likeCount.textContent = mc.textContent;
                if (mb && likeBtn)   likeBtn.style.opacity = mb.style.opacity;
            }, 400);
        };
    }

    lb.classList.add('open');

    // Load comments for this plant
    const lbList = document.getElementById('lb-comments-list');
    if (lbList) {
        lbList.innerHTML = '<em style="color:#aaa;font-size:0.75rem;">Loading…</em>';
        fetch('comments.php?get_comments=1&type=plant&key=' + encodeURIComponent(plantId))
            .then(r => r.json())
            .then(d => {
                const rows = d.comments || [];
                lbList.innerHTML = rows.length
                    ? rows.map(c =>
                        '<div style="padding:2px 0;border-bottom:1px solid rgba(255,255,255,0.1);">' +
                        '<strong style="color:#aed581;">' + c.name + '</strong>' +
                        '<span style="color:#aaa;font-size:0.72rem;margin:0 5px;">' + c.date + '</span>' +
                        c.message + '</div>'
                      ).join('')
                    : '<em style="color:#aaa;font-size:0.75rem;">No comments yet — be the first!</em>';
            }).catch(()=>{});
    }

    // Post button — per plant_id
    const postBtn = document.getElementById('lb-comment-btn');
    if (postBtn) {
        postBtn.onclick = async () => {
            const name = (document.getElementById('lb-comment-name')?.value || '').trim();
            const msg  = (document.getElementById('lb-comment-msg')?.value  || '').trim();
            if (!msg) return;
            const fd = new FormData();
            fd.append('post_comment', '1');
            fd.append('type',    'plant');
            fd.append('key',     plantId);
            fd.append('name',    name || 'Guest');
            fd.append('message', msg);
            const res  = await fetch('comments.php', { method:'POST', body: fd });
            const data = await res.json();
            if (data.status === 'ok' && lbList) {
                document.getElementById('lb-comment-msg').value = '';
                const e = data.entry;
                const div = document.createElement('div');
                div.style.cssText = 'padding:2px 0;border-bottom:1px solid rgba(255,255,255,0.1);';
                div.innerHTML = '<strong style="color:#aed581;">' + e.name + '</strong>' +
                    '<span style="color:#aaa;font-size:0.72rem;margin:0 5px;">' + e.date + '</span>' + e.message;
                lbList.prepend(div);
            }
        };
        document.getElementById('lb-comment-msg')?.addEventListener('keydown', e => {
            if (e.key === 'Enter') { e.preventDefault(); postBtn.click(); }
        });
    }
};

// Builds <option> tags for the week picker — 172 weeks back from today (≈ March 2023)
function buildWeekOptions() {
    const opts = [];
    const fmt  = d => d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    const now  = new Date();
    const dow  = now.getDay() || 7;
    const thisMonday = new Date(now);
    thisMonday.setDate(now.getDate() - dow + 1);
    thisMonday.setHours(0, 0, 0, 0);
    for (let w = 0; w < 172; w++) {
        const mon = new Date(thisMonday);
        mon.setDate(thisMonday.getDate() - w * 7);
        const sun = new Date(mon);
        sun.setDate(mon.getDate() + 6);
        const val   = mon.toISOString().split('T')[0];
        const range = fmt(mon) + ' – ' + fmt(sun);
        const label = w === 0 ? 'This week  (' + range + ')'
                    : w === 1 ? 'Last week  (' + range + ')'
                    : range;
        opts.push('<option value="' + val + '">' + label + '</option>');
    }
    return opts.join('');
}

// ==========================================
// Global Gallery Management Functions
// ==========================================

// 1. Rotation Logic
window.rotateImage = async function(filename, imgElementId) {
    if (!confirm("Rotate this image 90 degrees?")) return;
    
    try {
        const res = await fetch(`catalog.php?rotate_image=1&filename=${encodeURIComponent(filename)}`);
        const data = await res.json();
        
        if (data.status === 'success') {
            const imgEl = document.getElementById(imgElementId);
            if (imgEl) {
                const currentSrc = imgEl.src.split('?')[0]; 
                imgEl.src = `${currentSrc}?t=${new Date().getTime()}`;
            }
        } else {
            alert("Error rotating image: " + data.message);
        }
    } catch (err) {
        console.error("Rotation failed:", err);
        alert("Server error during rotation.");
    }
}

// 2. Upload Logic (Async without reload)
window.uploadGalleryImages = async function(event, plantId) {
    const files = event.target.files;
    if (!files || files.length === 0) return;

    const formData = new FormData();
    formData.append('plant_id', plantId);
    for (let i = 0; i < files.length; i++) {
        formData.append('gallery_images[]', files[i]);
    }

    const statusEl = document.getElementById(`gallery-upload-status-${plantId}`);
    if (statusEl) statusEl.innerText = 'Uploading... ⏳';

    try {
        const res = await fetch('catalog.php?upload_gallery=1', {
            method: 'POST',
            body: formData
        });
        const data = await res.json();

        if (data.status === 'success') {
            const p = window.plantData[plantId];
            let existing = p.additional_pictures ? p.additional_pictures.split(',').filter(x=>x.trim()) : [];
            
            // If primary is missing, make the first upload the primary
            if (!p.picture && data.new_images.length > 0) {
                p.picture = data.new_images.shift(); 
            }
            
            p.additional_pictures = existing.concat(data.new_images).join(',');
            openModal(p); // Instantly re-render the modal with the new images
        } else {
            alert('Upload failed: ' + data.message);
            if (statusEl) statusEl.innerText = 'Upload failed ❌';
        }
    } catch (err) {
        console.error(err);
        if (statusEl) statusEl.innerText = 'Server error ❌';
    }
};

// ===========================================================================
// SIMPLE VIEW (card) — shown instead of the detailed modal when the
// "Simple Card" slider on the catalog is toggled on. Persisted in
// localStorage('gardenSimpleView') and mirrored to window.SIMPLE_VIEW.
// ===========================================================================
const SV_MONTHS = ['January','February','March','April','May','June',
                   'July','August','September','October','November','December'];

// Derive a [startMonth, endMonth] pair. Primary source is the 12-char
// month bitmap (J F M A M J J A S O N D; '.' = month absent) — the first
// and last filled positions give the range. Falls back to explicit
// start/end month-name columns when no bitmap is stored.
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

// Pick a random photo filename from picture + additional_pictures.
function svRandomImage(p, imgBase) {
    const parts = []
        .concat(p.picture ? p.picture.split(',') : [])
        .concat(p.additional_pictures ? p.additional_pictures.split(',') : [])
        .map(s => s.trim()).filter(Boolean);
    const f = parts.length ? parts[Math.floor(Math.random() * parts.length)] : 'default.webp';
    return imgBase + f;
}

function svEsc(s) {
    return (s == null ? '' : String(s)).replace(/[&<>"]/g,
        c => ({ '&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;' }[c]));
}

function svInjectStyles() {
    if (document.getElementById('sv-card-styles')) return;
    const st = document.createElement('style');
    st.id = 'sv-card-styles';
    st.textContent = `
    #modal-body .sv-card{position:relative;display:grid;grid-template-columns:300px 1fr;background:#fff;border-radius:18px;overflow:hidden;min-height:340px;box-shadow:0 10px 40px rgba(0,0,0,.15)}
    #modal-body .sv-back{position:absolute;top:12px;left:12px;z-index:5;background:rgba(0,0,0,.5);color:#fff;border:none;border-radius:50px;padding:8px 18px;font-weight:900;cursor:pointer;font-size:.95rem;backdrop-filter:blur(4px);text-shadow:-1px -1px 0 #000,1px -1px 0 #000,-1px 1px 0 #000,1px 1px 0 #000}
    #modal-body .sv-photo{background:var(--svd-photo-bg,#dcedc8);display:flex;align-items:center;justify-content:center}
    #modal-body .sv-photo img{width:100%;height:100%;object-fit:cover;min-height:340px;cursor:zoom-in}
    #modal-body .sv-info{padding:34px 30px;display:flex;flex-direction:column;justify-content:center}
    #modal-body .sv-common{margin:0;font-size:2rem;color:var(--svd-accent,#1b5e20);font-weight:900;line-height:1.12}
    #modal-body .sv-botanical{margin:6px 0 0;font-style:italic;color:var(--svd-row-label,#33691e);font-size:1.2rem}
    #modal-body .sv-by{margin:12px 0 0;color:#6d4c41;font-weight:700;font-size:1.02rem}
    #modal-body .sv-meta{margin:12px 0 0;color:var(--svd-body-text,#33402f);font-size:1rem;background:var(--svd-tint,#eef6e7);border-radius:10px;padding:9px 14px;display:inline-block}
    #modal-body .sv-meta b{color:var(--svd-accent,#2e7d32)}
    #modal-body .sv-months{margin-top:24px;display:flex;flex-direction:column;gap:10px}
    #modal-body .sv-row{font-size:1.05rem;color:var(--svd-body-text,#33402f);background:var(--svd-tint,#f1f8e9);border-radius:10px;padding:11px 15px}
    #modal-body .sv-row b{color:var(--svd-accent,#2e7d32)}
    #modal-body .sv-bloom{background:#fce4ec}
    #modal-body .sv-bloom b{color:#c2185b}
    #modal-body .sv-ico{margin-right:6px}
    @media(max-width:600px){#modal-body .sv-card{grid-template-columns:1fr}#modal-body .sv-photo img{min-height:220px}}`;
    document.head.appendChild(st);
}

function renderSimpleCard(p, config, modal, body, foilCard) {
    svInjectStyles();
    // Clear any cycled background-theme styling; use garden theme modalBg if set
    foilCard.className = 'plant-modal-content';
    foilCard.style.backgroundColor = window._gardenThemeModalBg || '';
    foilCard.style.borderColor = '';
    foilCard.style.boxShadow = '';

    const imgBase    = (config && config.imgBase) || '/images/Database/';
    const imgSrc     = svRandomImage(p, imgBase);
    const plantRange = svMonthRange(p.grow_months,  p.planting_month_start, p.planting_month_end);
    const bloomRange = svMonthRange(p.bloom_months, p.month_start,          p.month_end);

    const common    = svEsc(p.plant_name || p.botanical_name || 'Unknown');
    const botanical = svEsc(p.botanical_name || '');
    const company   = svEsc(p.company || '');
    const qty       = (p.qty != null && String(p.qty).trim() !== '') ? svEsc(p.qty) : '';
    const bed       = svEsc(p.flower_bed || p.garden_beds || '');

    // Like / Favorite buttons for the tag-view popup
    const bannerInfo = getBannerState(p);
    const starHTML   = generateStarHTML(p, bannerInfo);
    const likeOpacity = p.is_liked ? '1' : '0.5';
    const likeCount   = p.like_count || 0;

    body.innerHTML = `
        <div class="sv-card">
            <button class="sv-back" onclick="window.closeModal()" title="Back to catalog">&#8592; Back</button>
            <div class="sv-photo">
                <img src="${imgSrc}" alt="${common}"
                     onclick="window.openLightbox && window.openLightbox('${imgSrc}', ${p.id})"
                     onerror="this.onerror=null;this.src='${imgBase}default.webp';">
            </div>
            <div class="sv-info">
                <h2 class="sv-common">${common}</h2>
                ${botanical ? `<p class="sv-botanical">${botanical}</p>` : ''}
                ${company   ? `<p class="sv-by">By ${company}</p>` : ''}
                ${(qty || bed) ? `<p class="sv-meta">${qty ? `🪴 Qty: <b>${qty}</b>` : ''}${(qty && bed) ? ' &nbsp;·&nbsp; ' : ''}${bed ? `🗺️ <b>${bed}</b>` : ''}</p>` : ''}
                <div class="sv-months">
                    ${plantRange ? `<div class="sv-row sv-plant"><span class="sv-ico">🌱</span>Plant between <b>${plantRange[0]}</b> &ndash; <b>${plantRange[1]}</b></div>` : ''}
                    ${bloomRange ? `<div class="sv-row sv-bloom"><span class="sv-ico">🌸</span>Bloom between <b>${bloomRange[0]}</b> &ndash; <b>${bloomRange[1]}</b></div>` : ''}
                </div>
                <div class="sv-actions" style="margin-top:18px;display:flex;align-items:center;gap:12px;">
                    ${starHTML}
                </div>
            </div>
        </div>`;

    body.scrollTop = 0;
    modal.style.display = 'flex';
    document.body.style.overflow = 'hidden';
    if (!isModalOpen) { history.pushState({ modal: true }, null, ''); isModalOpen = true; }
}

// 3. Main Modal Generation
export function openModal(p) {
    const config = window.GARDEN_CONFIG;
    const modal = document.getElementById("plant-modal");
    const body = document.getElementById("modal-body");
    const foilCard = document.querySelector('.plant-modal-content');

    // window.openModal is set in app.js to route through notes interceptor
    // NOTE: the modal always shows the FULL detailed view — even in Simple Tags
    // mode, clicking a card opens the complete details here.

    // 1. Cycle background themes (unless tag-view theme overrides)
    foilCard.className = 'plant-modal-content';
    foilCard.classList.add(`bg-theme-${modalThemeIndex}`);
    modalThemeIndex = (modalThemeIndex + 1) % 6;
    // If user is in Simple/Tag view with a garden theme, override the bg-theme pastel
    const _isSimple = document.body.classList.contains('simple-view-active');
    const _themeModalBg = window._gardenThemeModalBg;


    // 2. Determine holographic state
    const bannerInfo = getBannerState(p);
    if (bannerInfo.isHolo) foilCard.classList.add('holographic-active');

    // 3. Gallery Logic
    // picture may contain multiple comma-separated filenames (e.g. plants uploaded in bulk)
    const _picParts  = p.picture            ? p.picture.split(',').map(s => s.trim()).filter(Boolean) : [];
    const _addParts  = p.additional_pictures ? p.additional_pictures.split(',').map(s => s.trim()).filter(Boolean) : [];
    // Deduplicate while preserving order
    const _seen = new Set();
    let imageFilenames = [..._picParts, ..._addParts].filter(f => { if (_seen.has(f)) return false; _seen.add(f); return true; });
    if (!imageFilenames.length) imageFilenames = ['default.webp'];
    let images = imageFilenames.map(f => config.imgBase + f);

    let slidesHTML = images.map((src, index) => `
        <div class="gallery-slide" style="position:relative;">
            <img id="gal-main-img-${index}" src="${src}" alt="Plant Image"
                 onclick="window.openLightbox('${src}',${p.id})"
                 style="${p.died == 1 ? 'filter:grayscale(100%); opacity:0.6;' : ''}">
        </div>
    `).join('');

    const likeOpacity = p.is_liked ? '1' : '0.5';
    const likeCount = p.like_count || 0;

    // Admin Controls 
    const adminControls = config.isAdmin ? `
        <button class="cartoony-btn rotate-gal-btn" id="rotate-current-img" title="Rotate current photo" style="margin-right:5px;"><i class="garden-icon icon-rotate"></i></button>
        <button class="cartoony-btn delete-gal-btn" id="delete-current-img" title="Delete current photo"><i class="garden-icon icon-delete"></i></button>
        <input type="file" id="gallery-upload-${p.id}" multiple accept="image/*" style="display:none;" onchange="window.uploadGalleryImages(event, ${p.id})">
        <button class="cartoony-btn" onclick="document.getElementById('gallery-upload-${p.id}').click()" title="Add Photos"><i class="garden-icon icon-camera"></i></button>
    ` : '';

    const galleryHTML = `
        <div style="text-align:center; margin-bottom:15px;">
            <div class="plant-gallery-carousel">
                <div class="cartoony-nav prev-arrow disabled" id="gal-prev"><i class="garden-icon icon-left"></i></div>
                <div class="cartoony-nav next-arrow ${images.length <= 1 ? 'disabled' : ''}" id="gal-next"><i class="garden-icon icon-right"></i></div>
                <div class="gallery-slides-container" id="gallery-container">
                    ${slidesHTML}
                </div>
                <div class="gallery-controls-overlay">
                    ${adminControls}
                    <button id="like-btn-${p.id}" class="cartoony-btn thumb-btn" style="opacity:${likeOpacity};" onclick="this.classList.add('pop-anim'); setTimeout(() => this.classList.remove('pop-anim'), 300); window.toggleLike(${p.id})">👍 Like <span id="like-count-${p.id}">${likeCount}</span></button>
                </div>
            </div>
            <div id="gallery-upload-status-${p.id}" style="font-size: 0.8rem; margin-top: 5px; text-align:center;"></div>
        </div>
    `;

    // 4. Content Setup
    const starHTML = generateStarHTML(p, bannerInfo); 
    const theme = getRandomTheme(true); 
    const pill = (label, data, iconClass) => data && data !== '0.00' && data !== '0.0' ? `<div class="dash-pill"><span class="pill-label"><i class="garden-icon ${iconClass}"></i> ${label}</span><span class="pill-value">${data}</span></div>` : '';
    const note = (iconClass, label, data, type) => data && data.trim() !== '' ? `<div class="dash-note note-${type}"><div class="note-header"><i class="garden-icon ${iconClass}"></i> ${label}</div><div class="note-body">${data}</div></div>` : '';

    // Grouping Helper Function
    const renderGroup = (title, content) => {
        if (!content.trim()) return '';
        return `
            <div class="dash-pill-group" style="margin-bottom: 25px; padding: 15px; background: rgba(255,255,255,0.6); border-radius: 12px; box-shadow: 0 2px 10px rgba(0,0,0,0.03);">
                <h4 style="margin: 0 0 15px 0; font-size: 1.05rem; color: var(--modal-accent, #2e7d32); border-bottom: 2px solid rgba(255,255,255,0.15); padding-bottom: 5px; font-weight: bold;">${title}</h4>
                <div class="dash-pills-grid">
                    ${content}
                </div>
            </div>
        `;
    };

    // Morphology Card — SVG illustration + compact tagged pills (2-col grid) beside it
    // mpill: like pill() but adds a title tooltip for truncated values
    const mpill = (label, value, icon) => {
        if (!value || value === '0.00' || value === '0.0' || String(value).trim() === '') return '';
        const safe = String(value).replace(/"/g, '&quot;');
        return `<div class="dash-pill morpho-pill" title="${label}: ${safe}">
            <span class="pill-label"><i class="garden-icon ${icon}"></i> ${label}</span>
            <span class="pill-value">${value}</span>
        </div>`;
    };

    const renderMorphologyGroup = (title, svgHTML, fields) => {
        const pillsHTML = fields.map(([label, value, icon]) => mpill(label, value, icon)).join('');
        if (!pillsHTML.trim() && !svgHTML) return '';
        return `
            <div class="dash-pill-group" style="margin-bottom: 25px; padding: 15px; background: rgba(255,255,255,0.6); border-radius: 12px; box-shadow: 0 2px 10px rgba(0,0,0,0.03);">
                <h4 style="margin: 0 0 14px 0; font-size: 1.05rem; color: #2e7d32; border-bottom: 2px solid rgba(46,125,50,0.1); padding-bottom: 5px; font-weight: bold;">${title}</h4>
                <div class="morpho-card">
                    ${svgHTML ? `<div class="morpho-svg-wrap">${svgHTML}</div>` : ''}
                    <div class="morpho-pills-flow">${pillsHTML}</div>
                </div>
            </div>`;
    };

    let parallaxHTML = '<div class="parallax-container" id="modal-parallax">';
    const favs = p.favorited_by || [];
    if (favs.includes('Joshua')) parallaxHTML += `<div class="parallax-layer parallax-user" style="--speed: 1.5; top: 12%; left: 5%;">Jun's Pick 👍</div>`;
    if (favs.includes('Jason')) parallaxHTML += `<div class="parallax-layer parallax-user" style="--speed: -1.2; top: 25%; right: 5%;">Author's pick 👍</div>`;
    if (favs.includes('Kona')) parallaxHTML += `<div class="parallax-layer parallax-user" style="--speed: 0.8; top: 18%; right: 15%; background: rgba(255, 150, 50, 0.4);">Kona's Pick 👍</div>`;
    
    for (let i = 0; i < 15; i++) {
        const spd = (Math.random() * 3 - 1.5).toFixed(2); 
        parallaxHTML += `<div class="parallax-layer parallax-star" style="--speed: ${spd}; top: ${Math.random() * 90}%; left: ${Math.random() * 90}%; font-size: ${Math.random() * 1.5 + 0.5}rem;">?</div>`;
    }
    // Use per-user favorite theme if this plant has been favorited.
    // When no favTheme, clear any leftover inline style so the cycled bg-theme-X CSS class shows.
    const favTheme = getFavoriteTheme(p);
    if (favTheme) {
        foilCard.style.backgroundColor = favTheme.bg;
        foilCard.style.borderColor     = favTheme.border;
        foilCard.style.boxShadow       = `0 0 0 2px ${favTheme.border}, 0 20px 60px ${favTheme.accent}30`;
    } else if (_isSimple && _themeModalBg) {
        // In tag view, apply the garden theme's modal background
        foilCard.style.backgroundColor = _themeModalBg;
        foilCard.style.borderColor     = '';
        foilCard.style.boxShadow       = '';
    } else {
        foilCard.style.backgroundColor = '';   // let bg-theme-X class take over
        foilCard.style.borderColor     = '';
        foilCard.style.boxShadow       = '';
    }
    parallaxHTML += `<div style="position: absolute; top: ${theme.y}%; left: ${theme.x}%; transform: translate(-50%, -50%); font-size: ${theme.size}rem; opacity: 0.05; z-index: -1; pointer-events: none; user-select: none;">${theme.icon}</div></div>`;

    // 5. Inject Final HTML 
    let growStr = p.grow_months || "";
    // Fallback: derive from planting start/end month names if no bitmap stored.
    // Format as "Mon-Mon" so renderSeasonalScaleJS fills all months in between.
    if (!growStr && p.planting_month_start && p.planting_month_end) {
        growStr = String(p.planting_month_start).substring(0, 3) + '-' + String(p.planting_month_end).substring(0, 3);
    }
    let bloomStr = p.bloom_months || "";
    // Fallback: derive from bloom start/end month names if no bitmap stored.
    if (!bloomStr && p.month_start && p.month_end) {
        bloomStr = String(p.month_start).substring(0, 3) + '-' + String(p.month_end).substring(0, 3);
    }
    // Determine bloom bar type based on flower field data
    const _noFlower = v => !v || ['', 'none', 'n/a', 'null', 'unknown'].includes(String(v).trim().toLowerCase());
    const _emptyFlowerFields = [p.bloom_color, p.flower_shape, p.flower_symmetry, p.flower_size].filter(_noFlower).length;
    const isFlowerless = _emptyFlowerFields >= 2; // applies to ALL plants
    const isEvergreen  = p.is_evergreen && isFlowerless;

    // Notes button — only visible to logged-in users
    const _noteExists = window.hasNote && window.hasNote(p.id);
    const _noteUser   = config.currentUser || 'My';
    const notesBtnHTML = config.currentUser ? `
        <button onclick="window.openNoteEditor && window.openNoteEditor(window.plantData[${p.id}])" title="${_noteUser}'s Notes" style="
            display:block; margin:0 auto 8px auto; z-index:10;
            background:${_noteExists ? 'rgba(255,193,7,0.85)' : 'rgba(0,0,0,0.5)'};
            color:${_noteExists ? '#333' : '#fff'}; border:none; border-radius:50px;
            padding:8px 22px; cursor:pointer; font-size:0.95rem; font-weight:900;
            letter-spacing:0.5px; backdrop-filter:blur(4px);
            text-shadow:${_noteExists ? 'none' : '-1px -1px 0 #000,1px -1px 0 #000,-1px 1px 0 #000,1px 1px 0 #000'};
            transition:background 0.2s;">
            📝 ${_noteExists ? `${_noteUser}'s Notes` : 'Add Note'}
        </button>` : '';

    body.innerHTML = parallaxHTML + `
        <div class="dash-header">
            <button onclick="window.closeModal()" title="Back to catalog" style="
                display:block; margin:0 auto 12px auto; z-index:10;
                background:rgba(0,0,0,0.5); color:#fff; border:none; border-radius:50px;
                padding:8px 22px; cursor:pointer; font-size:1rem; font-weight:900;
                letter-spacing:0.5px; backdrop-filter:blur(4px);
                text-shadow:-1px -1px 0 #000,1px -1px 0 #000,-1px 1px 0 #000,1px 1px 0 #000;
                transition:background 0.2s;">&#8592; Back to Catalog</button>
            ${notesBtnHTML}
            <div style="display:flex; justify-content:space-between; align-items:flex-start; gap:10px; width:100%; text-align:left; flex-wrap:nowrap;">
                <h2 style="margin:0; flex:1; word-wrap:break-word;">${p.plant_name}${p.new_name ? `<br><span style="color:#FFD700 !important; font-size:0.6em; text-shadow: 1px 1px 3px rgba(0,0,0,0.8);">Also Appears in "${p.new_name}"</span>` : ''}</h2>
                ${starHTML}
            </div>
        </div>
        <div class="dash-scroller">
            ${galleryHTML} 
            
            <div class="dash-note" style="background: rgba(255,255,255,0.7); border-radius: 15px; padding: 15px; margin-bottom: 20px; box-shadow: 0 4px 15px rgba(0,0,0,0.05);">
                <div class="note-header" style="font-size: 1.1rem; border-bottom: 2px solid rgba(0,0,0,0.1); padding-bottom: 5px; margin-bottom: 10px;"><i class="garden-icon icon-schedule"></i> Seasonal Schedule</div>
                <div style="display:flex; align-items:center; gap:15px; margin-bottom:10px;">
                    <div style="min-width: 80px; font-weight:bold; color:var(--modal-accent, #2e7d32);"><i class="garden-icon icon-grow"></i> Grow</div>
                    ${isEvergreen ? renderEvergreenMonthBar() : renderSeasonalScaleJS(growStr, 'grow')}
                </div>
                <div style="display:flex; align-items:center; gap:15px;">
                    <div style="min-width: 80px; font-weight:bold; color:${isFlowerless ? '#ce93d8' : '#d81b60'};"><i class="garden-icon icon-bloom"></i> ${isFlowerless ? 'None' : 'Bloom'}</div>
                    ${isFlowerless ? renderFlowerlessTag() : renderSeasonalScaleJS(bloomStr, 'bloom')}
                </div>
            </div>

            <div class="dash-grouped-sections">
                ${renderGroup('🧬 Taxonomy & Identity', `
                    ${pill('Location', p.flower_bed, 'icon-location')}
                    ${pill('Company/Brand', p.company, 'icon-botanical')}
                    ${config.isAdmin && p.died_year ? `<div class="dash-pill" style="background:#fce4ec;border:2px solid #e91e63;"><span class="pill-label">💀 Died Year</span><span class="pill-value">${p.died_year}</span></div>` : ''}
                    ${generatePlantTypeHTML(p.plant_type)}
                    ${pill('Family', getFamilyDisplayName(p.family), 'icon-family')}
                    ${pill('Genus', p.genus, 'icon-botanical')}
                    ${pill('Botanical Name', p.botanical_name, 'icon-botanical')}
                    ${pill('Species', p.species, 'icon-species')}
                    ${generateNativeStatusHTML(p.native_status)}
                    ${generateInvasiveHTML(p.invasive_local)}
                    ${generateToxicityWidgetHTML(p.toxicity)}
                    ${generateEdibilityWidgetHTML(p.fruit_edibility, p.toxicity, p.flower_bed, p.extra_details)}
                `)}

                ${renderGroup('📦 Acquisition & Inventory', `
                    ${generateQuantityWidgetHTML(p.qty)}
                    ${pill('Year Added', p.year_added, 'icon-schedule')}
                    ${generateCostWidgetHTML((p.average_cost && parseFloat(p.average_cost) > 0) ? '$' + parseFloat(p.average_cost).toFixed(2) : '')}
                `)}

                ${renderGroup('🌿 Fertilizer & Wildlife Attractants', `
                    ${generateDeerRabbitWidgetHTML(p.deer_resistance)}
                    ${generateFertilizerWidgetHTML(p.fertilizer)}
                    ${generateAttractsWidgetHTML(p.attracts)}
                `)}

                ${renderGroup('📏 Growth Metrics & Spacing', `
                    ${generateGrowthRateWidgetHTML(p.growth_rate)}
                    ${generateLifeExpectancyWidgetHTML(p.life_expectancy)}
                    ${generateChillingHoursWidgetHTML(p.chilling_hours)}
                    ${generateZoneWidgetHTML(p.zones)}
                    ${generateRulerWidgetHTML(p.plant_length || p.size, 'Length / Height')}
                    ${generateRulerWidgetHTML(p.plant_width, 'Width / Spread')}
                    ${generateRulerWidgetHTML(p.spacing, 'Spacing')}
                `)}

                ${renderGroup('🌦️ Soil & Climate Requirements', `
                    ${generateSoilTypeWidgetHTML(p.soil_type)}
                    ${generatePhWidgetHTML(p.soil_ph)}
                    ${generatePlantingDepthHTML(p.planting_depth)}
                    ${generateSunWidgetHTML(p.sun_exposure)}
                    ${generateWaterCupHTML(p.water_needs)}
                `)}

                ${renderMorphologyGroup('🌿 Foliage & Stem Morphologies',
                    generateLeafSVG(p.leaf_shape, p.leaf_type, p.leaf_margin, p.foliage_color),
                    [
                        ['Leaf Shape',    p.leaf_shape,    'icon-foliage'],
                        ['Leaf Type',     p.leaf_type,     'icon-foliage'],
                        ['Leaf Margin',   p.leaf_margin,   'icon-foliage'],
                        ['Foliage Color', p.foliage_color, 'icon-color'],
                        ['Evergreen',     p.is_evergreen,  'icon-foliage'],
                        ['Stem Color',    p.stem_color,    'icon-color'],
                    ]
                )}

                ${renderMorphologyGroup('🌸 Flower & Scent Morphologies',
                    generateFlowerSVG(p.flower_shape, p.flower_size, p.flower_symmetry, p.bloom_color),
                    [
                        ['Flower Shape',    p.flower_shape,    'icon-botanical'],
                        ['Flower Size',     p.flower_size,     'icon-size'],
                        ['Flower Symmetry', p.flower_symmetry, 'icon-botanical'],
                        ['Bloom Color',     p.bloom_color,     'icon-color'],
                    ]
                )}

                \${generateAromaticsWidgetHTML(p.aromatics)}

                ${renderMorphologyGroup('🍒 Fruit & Berry Yields',
                    generateFruitSVG(p.fruit_type, p.fruit_shape, p.fruit_color, p.fruit_edibility),
                    [
                        ['Fruit/Berry Type', p.fruit_type,     'icon-fruit'    ],
                        ['Fruit Shape',      p.fruit_shape,    'icon-fruit'    ],
                        ['Fruit Color',      p.fruit_color,    'icon-color'    ],
                        ['Fruit Size',       p.fruit_size,     'icon-size'     ],
                        ['Fruit Edibility',  p.fruit_edibility,'icon-toxicity' ],
                    ]
                )}

                ${renderGroup('🪵 Propagation & Cuttings', `
                    ${pill('Propagation Method', p.propagation_method, 'icon-prop')}
                    ${pill('Cuttings Instructions', p.cuttings, 'icon-cut')}
                `)}
            </div>

            <!-- Quantity History — ADMIN ONLY, hidden from public -->
            ${config.isAdmin ? `
            <div class="dash-note" style="background:rgba(255,255,255,0.7);border-radius:15px;padding:15px;margin-bottom:20px;" id="qty-history-section">
                <div class="note-header" style="font-size:1.1rem;border-bottom:2px solid rgba(0,0,0,0.1);padding-bottom:5px;margin-bottom:10px;">
                    📊 Quantity History
                    <span style="font-size:0.8rem;font-weight:normal;color:#888;margin-left:6px;">current: <strong>${p.qty ?? 0}</strong></span>
                </div>
                <div id="qty-history-body-${p.id}" style="font-size:0.85rem;color:#555;">Loading…</div>

                <details style="margin-top:12px;">
                    <summary style="cursor:pointer;font-weight:700;font-size:0.85rem;color:#2e7d32;padding:6px 0;">➕ Log New Event</summary>
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-top:10px;">
                        <select id="qty-type-${p.id}" style="padding:7px;border-radius:8px;border:2px solid #ccc;font-size:0.85rem;">
                            <option value="purchased">🛒 Purchased / Added</option>
                            <option value="died">💀 Died</option>
                            <option value="removed">🗑️ Removed</option>
                            <option value="other">📝 Other</option>
                        </select>
                        <input type="number" id="qty-amount-${p.id}" min="1" value="1" placeholder="# of plants"
                               style="padding:7px;border-radius:8px;border:2px solid #ccc;font-size:0.85rem;">
                        <div style="grid-column:1/-1;">
                            <label style="font-size:0.75rem;font-weight:700;color:#555;display:block;margin-bottom:4px;">📅 Week</label>
                            <select id="qty-date-${p.id}" style="width:100%;padding:8px;border-radius:8px;border:2px solid #aed581;font-size:0.85rem;box-sizing:border-box;">
                                ${buildWeekOptions()}
                            </select>
                        </div>
                        <input type="text" id="qty-notes-${p.id}" placeholder="Notes (optional)"
                               style="grid-column:1/-1;padding:7px;border-radius:8px;border:2px solid #ccc;font-size:0.85rem;">
                        <button onclick="window.logQtyEvent(${p.id})"
                                style="grid-column:1/-1;padding:9px;background:#2e7d32;color:#fff;border:none;border-radius:10px;font-weight:700;cursor:pointer;font-size:0.9rem;">
                            💾 Save Event
                        </button>
                    </div>

                    <div style="margin-top:12px;border-top:1px dashed #ccc;padding-top:10px;">
                        <label style="font-size:0.82rem;font-weight:700;color:#c62828;">💀 Set Died Year</label>
                        <div style="display:flex;gap:8px;margin-top:6px;">
                            <input type="number" id="died-year-input-${p.id}" value="${p.died_year||''}" min="2000" max="2099"
                                   placeholder="e.g. 2024" style="padding:7px;border-radius:8px;border:2px solid #e57373;flex:1;font-size:0.85rem;">
                            <button onclick="window.saveDiedYear(${p.id})"
                                    style="padding:7px 14px;background:#c62828;color:#fff;border:none;border-radius:10px;font-weight:700;cursor:pointer;font-size:0.85rem;">
                                Save
                            </button>
                        </div>
                    </div>
                </details>
            </div>` : ''}

            <div class="dash-notes-stack">
                ${note('icon-botanical', 'Companion Plants', p.companion_plants, 'comp')}
                ${note('icon-care', 'Extra Details / Edibility', p.extra_details, 'extra')}
                ${note('icon-disease', 'Disease & Pests', p.disease_susceptibility, 'disease')}
                ${note('icon-maint', 'Maintenance', p.maintenance_notes, 'maint')}
                ${note('icon-care', 'Care Notes', p.care_notes, 'care')}
                ${note('icon-cleanup', 'Cleanup', p.cleanup_notes, 'clean')}
                ${note('icon-admin', 'Admin Tasks', p.admin_todo, 'admin')}
            </div>

            ${p.source_table !== 'future_plants' ? `
            <!-- ── Scan Codes ─────────────────────────────────────────── -->
            <div id="scan-section-${p.id}" style="
                    background:rgba(255,255,255,0.7); border-radius:15px;
                    padding:20px; margin-bottom:20px; box-shadow:0 4px 15px rgba(0,0,0,0.05);">
                <div class="note-header" style="
                        font-size:1.1rem; font-weight:700;
                        border-bottom:2px solid rgba(0,0,0,0.1);
                        padding-bottom:6px; margin-bottom:16px;">
                    🔍 Scan Codes
                </div>
                <div style="display:flex; gap:28px; flex-wrap:wrap; justify-content:center; align-items:flex-start;">

                    <div id="barcode-block-${p.id}" style="display:flex; flex-direction:column; align-items:center; gap:6px;">
                        <span style="font-size:0.75rem; font-weight:700; color:#666; text-transform:uppercase; letter-spacing:0.6px;">Barcode</span>
                        <img src="plant_actions.php?action=barcode&plant_id=${p.id}"
                             alt="Product Barcode"
                             style="max-width:230px; height:auto; border-radius:8px; border:1px solid rgba(0,0,0,0.08); background:#fff; padding:6px;"
                             onerror="document.getElementById('barcode-block-${p.id}').style.display='none'; window._checkScanSection(${p.id});">
                    </div>

                    <div id="qr-block-${p.id}" style="display:flex; flex-direction:column; align-items:center; gap:6px;">
                        <span style="font-size:0.75rem; font-weight:700; color:#666; text-transform:uppercase; letter-spacing:0.6px;">QR Code</span>
                        <img src="qr/plant/${p.id}.png"
                             alt="QR Code"
                             style="max-width:140px; height:auto; border-radius:8px; border:1px solid rgba(0,0,0,0.08); background:#fff; padding:6px;"
                             onerror="document.getElementById('qr-block-${p.id}').style.display='none'; window._checkScanSection(${p.id});">
                    </div>

                </div>
            </div>` : ''}
        </div>
    `;

    // Hide the whole scan-codes section if both images 404
    window._checkScanSection = function(id) {
        const bBlock = document.getElementById('barcode-block-' + id);
        const qBlock = document.getElementById('qr-block-'    + id);
        if (bBlock?.style.display === 'none' && qBlock?.style.display === 'none') {
            const sec = document.getElementById('scan-section-' + id);
            if (sec) sec.style.display = 'none';
        }
    };

    // 6. Post-Injection Gallery Slider, Rotate, & Delete Logic
    let currentGalleryIndex = 0;
    const totalGallerySlides = images.length;
    const prevBtn = document.getElementById('gal-prev');
    const nextBtn = document.getElementById('gal-next');
    const galContainer = document.getElementById('gallery-container');

    function updateGallery() {
        if (!galContainer) return;
        galContainer.style.transform = `translateX(-${currentGalleryIndex * 100}%)`;
        
        if (prevBtn) prevBtn.classList.toggle('disabled', currentGalleryIndex === 0);
        if (nextBtn) nextBtn.classList.toggle('disabled', currentGalleryIndex === totalGallerySlides - 1);
    }
    
    if (prevBtn) prevBtn.addEventListener('click', () => { if (currentGalleryIndex > 0) { currentGalleryIndex--; updateGallery(); } });
    if (nextBtn) nextBtn.addEventListener('click', () => { if (currentGalleryIndex < totalGallerySlides - 1) { currentGalleryIndex++; updateGallery(); } });

    // ROTATE BUTTON LOGIC
    const rotateBtn = document.getElementById('rotate-current-img');
    if (rotateBtn) {
        rotateBtn.addEventListener('click', () => {
            const currentFilename = imageFilenames[currentGalleryIndex];
            if (!currentFilename || currentFilename === 'default.webp') {
                alert("Cannot rotate the default placeholder image.");
                return;
            }
            window.rotateImage(currentFilename, `gal-main-img-${currentGalleryIndex}`);
        });
    }

    // DELETE BUTTON LOGIC
    const deleteBtn = document.getElementById('delete-current-img');
    if (deleteBtn) {
        deleteBtn.addEventListener('click', async () => {
            const currentFilename = imageFilenames[currentGalleryIndex];
            if (!currentFilename || currentFilename === 'default.webp') {
                alert("Cannot delete the default image placeholder.");
                return;
            }
            if (!confirm("Are you sure you want to permanently delete this photo?")) return;

            try {
                const res = await fetch(`catalog.php?delete_image=1&plant_id=${p.id}&filename=${encodeURIComponent(currentFilename)}`);
                const data = await res.json();
                
                if(data.status === 'success') {
                    if (p.picture === currentFilename) {
                        p.picture = null; 
                        if (p.additional_pictures) {
                            let pics = p.additional_pictures.split(',').filter(x=>x.trim());
                            if(pics.length > 0) {
                                p.picture = pics.shift();
                                p.additional_pictures = pics.join(',');
                            }
                        }
                    } else {
                        let pics = p.additional_pictures.split(',').filter(x => x.trim() && x.trim() !== currentFilename);
                        p.additional_pictures = pics.join(',');
                    }
                    openModal(p); 
                } else {
                    alert('Error: ' + data.message);
                }
            } catch (err) {
                console.error(err);
                alert('Server error during deletion.');
            }
        });
    }

    // 7. Initialize View State
    const scroller = body.querySelector('.dash-scroller');
    if (scroller) scroller.scrollTop = 0; 
    
    applyScrollFadeEffects('.dash-scroller', '.dash-note');

    // Load quantity history asynchronously — admin only
    if (config.isAdmin) loadQtyHistory(p.id);

    modal.style.display = "flex";
    document.body.style.overflow = "hidden"; 
    if (!isModalOpen) { history.pushState({modal: true}, null, ""); isModalOpen = true; }
}

export function closeModal() { 
    if (isModalOpen) history.back(); 
    else { 
        document.getElementById("plant-modal").style.display = "none"; 
        document.body.style.overflow = "auto"; 
    } 
}

// ── Quantity history loader (admin only) ──────────────────────────────────
export async function loadQtyHistory(plantId) {
    const el = document.getElementById(`qty-history-body-${plantId}`);
    if (!el) return;
    try {
        const res  = await fetch(`catalog.php?qty_history=${plantId}`);
        const rows = await res.json();
        if (!rows.length) { el.innerHTML = '<em style="color:#aaa">No events recorded yet.</em>'; return; }

        const ICONS = { purchased:'🛒', died:'💀', removed:'🗑️', other:'📝' };
        const fmtWeek = dateStr => {
            if (!dateStr) return '—';
            const d = new Date(dateStr + 'T00:00:00');
            return 'Wk of ' + d.toLocaleDateString('en-US', { month:'short', day:'numeric', year:'numeric' });
        };
        el.innerHTML = `<table style="width:100%;border-collapse:collapse;font-size:0.82rem;">
            <thead><tr style="border-bottom:2px solid #eee;background:#f9f9f9;">
                <th style="text-align:left;padding:5px 6px;">Event</th>
                <th style="text-align:center;padding:5px 6px;">Qty</th>
                <th style="text-align:left;padding:5px 6px;">Week</th>
                <th style="text-align:left;padding:5px 6px;">Notes</th>
            </tr></thead>
            <tbody>
            ${rows.map(r => `<tr style="border-bottom:1px solid #f0f0f0;">
                <td style="padding:5px 6px;">${ICONS[r.event_type]||'•'} ${r.event_type}</td>
                <td style="text-align:center;padding:5px 6px;font-weight:700;color:${r.quantity>0?'#2e7d32':'#c62828'};">${r.quantity>0?'+':''}${r.quantity}</td>
                <td style="padding:5px 6px;white-space:nowrap;font-size:0.78rem;">${fmtWeek(r.event_date)}</td>
                <td style="padding:5px 6px;color:#888;font-size:0.78rem;">${r.notes||''}</td>
            </tr>`).join('')}
            </tbody></table>`;
    } catch(e) { el.innerHTML = '<em style="color:#e00">Failed to load history.</em>'; }
}

// ── Log a new quantity event ───────────────────────────────────────────────
window.logQtyEvent = async function(plantId) {
    const type  = document.getElementById(`qty-type-${plantId}`)?.value;
    const qty   = parseInt(document.getElementById(`qty-amount-${plantId}`)?.value || '0');
    const date  = document.getElementById(`qty-date-${plantId}`)?.value || '';
    const notes = document.getElementById(`qty-notes-${plantId}`)?.value || '';
    if (!qty) { alert('Enter a quantity.'); return; }

    const fd = new FormData();
    fd.append('log_qty_event', '1');
    fd.append('plant_id',   plantId);
    fd.append('event_type', type);
    fd.append('quantity',   qty);
    fd.append('event_date', date);
    fd.append('notes',      notes);

    const res  = await fetch('catalog.php', { method:'POST', body:fd });
    const data = await res.json();
    if (data.status === 'ok') {
        await loadQtyHistory(plantId);
        const badge = document.querySelector('#qty-history-section .note-header strong');
        if (badge) badge.textContent = data.new_qty;
        if (window.plantData?.[plantId]) window.plantData[plantId].qty = data.new_qty;
    } else { alert('Error: ' + data.message); }
};

// ── Save died year ─────────────────────────────────────────────────────────
window.saveDiedYear = async function(plantId) {
    const year = document.getElementById(`died-year-input-${plantId}`)?.value || '';
    const fd   = new FormData();
    fd.append('set_died_year', '1');
    fd.append('plant_id',  plantId);
    fd.append('died_year', year);
    const res  = await fetch('catalog.php', { method:'POST', body:fd });
    const data = await res.json();
    if (data.status === 'ok') {
        if (window.plantData?.[plantId]) window.plantData[plantId].died_year = data.died_year;
        alert(data.died_year ? `Died year set to ${data.died_year}` : 'Died year cleared.');
    }
};

export function initModalListeners() {
    // Back button (browser or Android hardware) closes the modal
    window.onpopstate = function() {
        document.getElementById("plant-modal").style.display = "none";
        document.body.style.overflow = "auto";
        isModalOpen = false;
    };

    // Tap/click the dark backdrop (outside the card) to close
    const modal = document.getElementById("plant-modal");
    if (modal) {
        modal.addEventListener("click", function(e) {
            if (e.target === modal) window.closeModal();
        });
        // Touch: also handle touchend so it feels snappy on mobile
        modal.addEventListener("touchend", function(e) {
            if (e.target === modal) {
                e.preventDefault();
                window.closeModal();
            }
        }, { passive: false });
    }
}