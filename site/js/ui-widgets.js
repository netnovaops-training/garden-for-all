// js/ui-widgets.js
const config = window.GARDEN_CONFIG || {};

export const gardenIcons = ['🌿', '🌸', '🪴', '🌻', '🌵', '🌷', '🌲', '🌳', '🌺', '🌾', '🍄', '🌱', '🌼'];

// Inline SVG cardinal — male Northern Cardinal facing right
const CARDINAL_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 36" width="26" height="24" style="vertical-align:middle;display:inline-block;">
  <!-- tail feathers -->
  <path d="M5 27 Q1 34 8 31 Q3 28 9 24" fill="#C62828"/>
  <!-- body -->
  <ellipse cx="17" cy="23" rx="8.5" ry="4.5" fill="#E53935" transform="rotate(-20 17 23)"/>
  <!-- wing (darker red) -->
  <ellipse cx="13" cy="19" rx="9" ry="2.8" fill="#B71C1C" opacity="0.9" transform="rotate(-42 13 19)"/>
  <!-- neck bridge -->
  <ellipse cx="23" cy="19" rx="3.5" ry="3.2" fill="#E53935"/>
  <!-- head -->
  <circle cx="25" cy="15" r="5.2" fill="#E53935"/>
  <!-- crest (pointed, upright) -->
  <path d="M23 11 Q25 5 28 3 Q24.5 8 27 7 Q23.5 11 23 11Z" fill="#C62828"/>
  <!-- black mask — throat -->
  <ellipse cx="26" cy="18" rx="4.2" ry="2.4" fill="#1a1a1a"/>
  <!-- black mask — face around beak -->
  <ellipse cx="26.5" cy="14" rx="3" ry="2.8" fill="#1a1a1a"/>
  <!-- eye -->
  <circle cx="27.5" cy="13" r="1.4" fill="#1a1a1a"/>
  <circle cx="27.9" cy="12.6" r="0.55" fill="#fff"/>
  <!-- beak (conical, orange-red) -->
  <path d="M29 14.5 L38 11.5 L29 17.5Z" fill="#FF8F00"/>
  <line x1="29" y1="16" x2="38" y2="13.5" stroke="#E65100" stroke-width="0.6"/>
</svg>`;

// Inline SVG squirrel — facing right, fluffy tail curled over back
const SQUIRREL_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 44 40" width="26" height="24" style="vertical-align:middle;display:inline-block;">
  <!-- fluffy tail (behind body, curling up) -->
  <path d="M9 36 Q2 24 5 13 Q9 3 19 7 Q15 13 13 22 Q11 30 15 36Z" fill="#8D6E63"/>
  <path d="M10 34 Q4 23 7 14 Q10 6 17 9 Q14 14 12 22 Q11 29 14 35Z" fill="#D7CCC8" opacity="0.55"/>
  <!-- body -->
  <ellipse cx="24" cy="29" rx="9.5" ry="7" fill="#795548" transform="rotate(-12 24 29)"/>
  <!-- head -->
  <circle cx="33" cy="20" r="7.5" fill="#795548"/>
  <!-- ear -->
  <path d="M28 13 Q27 7 31 7 Q33 11 30 15Z" fill="#795548"/>
  <path d="M28.5 13 Q28 8.5 30.5 8.5 Q31.8 11 29.5 14.5Z" fill="#FFCCBC" opacity="0.75"/>
  <!-- cheek pouch -->
  <ellipse cx="36" cy="23" rx="3.2" ry="2.6" fill="#8D6E63" opacity="0.55"/>
  <!-- eye -->
  <circle cx="36" cy="18.5" r="2" fill="#1a1a1a"/>
  <circle cx="36.6" cy="17.9" r="0.7" fill="#fff"/>
  <!-- nose -->
  <ellipse cx="40" cy="22" rx="1.3" ry="1" fill="#5D4037"/>
  <!-- front paws (tiny) -->
  <ellipse cx="29" cy="35.5" rx="3.2" ry="1.6" fill="#6D4C41" transform="rotate(-15 29 35.5)"/>
  <ellipse cx="23" cy="37" rx="2.8" ry="1.4" fill="#6D4C41" transform="rotate(5 23 37)"/>
</svg>`;

// Inline SVG hummingbird — used wherever no dedicated Unicode emoji exists
const HUMMINGBIRD_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 36" width="20" height="20" style="vertical-align:middle;display:inline-block;">
  <!-- tail -->
  <path d="M6 26 Q3 32 9 29 Q5 27 10 23" fill="#388E3C"/>
  <!-- body -->
  <ellipse cx="17" cy="21" rx="7" ry="3.5" fill="#43A047" transform="rotate(-25 17 21)"/>
  <!-- wing -->
  <ellipse cx="14" cy="15" rx="9" ry="2.8" fill="#A5D6A7" opacity="0.95" transform="rotate(-55 14 15)"/>
  <!-- head -->
  <circle cx="23" cy="15" r="4" fill="#2E7D32"/>
  <!-- ruby throat patch -->
  <ellipse cx="22" cy="19" rx="2.5" ry="1.5" fill="#E53935" transform="rotate(-25 22 19)"/>
  <!-- eye -->
  <circle cx="24.5" cy="14" r="1.1" fill="#fff"/>
  <circle cx="24.8" cy="14" r="0.55" fill="#111"/>
  <!-- beak -->
  <path d="M27 13.5 L36 11" stroke="#5D4037" stroke-width="1.6" stroke-linecap="round" fill="none"/>
</svg>`;

export function getAttractsIcon(term) {
    term = term.toLowerCase();
    if (term.includes('bee')) return '🐝';
    if (term.includes('butterfly') || term.includes('butterflies')) return '🦋';
    if (term.includes('humming') || term.includes('hummingbird')) return HUMMINGBIRD_SVG;
    if (term.includes('squirrel')) return SQUIRREL_SVG;
    if (term.includes('bird')) return CARDINAL_SVG;
    if (term.includes('moth')) return '🦇';
    if (term.includes('pollinator')) return '🌸';
    if (term.includes('rabbit')) return '🐇';
    if (term.includes('deer')) return '🦌';
    return '✨';
}

export const soothingColors = ['#f4fcf6', '#fffaf0', '#fdf5f5', '#f0f8ff', '#fcf4fc', '#fdfdf0', '#f5f7fa'];

export function getRandomTheme(isGiant = false) {
    return {
        bg: soothingColors[Math.floor(Math.random() * soothingColors.length)],
        icon: gardenIcons[Math.floor(Math.random() * gardenIcons.length)],
        size: isGiant ? (Math.floor(Math.random() * 8) + 12) : (Math.floor(Math.random() * 4) + 6),
        x: Math.floor(Math.random() * 80) + 10,
        y: Math.floor(Math.random() * 70) + 15
    };
}

export function getBannerState(p) {
    let text = "", show = false, isHolo = false;
    const favs = p.favorited_by || [];
    const currentUser = (typeof config !== 'undefined' && config.currentUser) ? config.currentUser : '';
    
    if (favs.includes('Kona') && favs.includes('Jason') && favs.includes('Joshua')) {
        text = "Clifton Fam Fav"; show = true; isHolo = true;
    } else if (favs.includes('Jason') && favs.includes('Joshua')) {
        text = "Staff Pick!"; show = true; isHolo = true;
    } else if (favs.includes('Jason')) {
        text = "Jason's Pick!"; show = true; isHolo = true;
    } else if (favs.includes('Joshua')) {
        text = "Jun's Pick!"; show = true; isHolo = true;
    } else if (favs.includes('Kona')) {
        text = "Kona's Pick 🐾!"; show = true; isHolo = true;
    } else if (p.is_favorited && currentUser === '') {
        text = "Community Fav!"; show = true; isHolo = true;
    } else if (p.fav_count > 0) {
        text = "Favorited!"; show = true; isHolo = true;
    } else {
        text = "Favorite This Plant"; show = true; isHolo = false;
    }
    return { text, show, isHolo };
}

// ── Per-user & combination favorite card themes ───────────────────────────
// Returns { bg, border, accent, name } or null if not yet favorited.
export function getFavoriteTheme(p) {
    const favs = p.favorited_by || [];
    const hasJoshua    = favs.includes('Joshua');
    const hasKona      = favs.includes('Kona');
    const hasJason     = favs.includes('Jason');
    const hasCommunity = (p.is_favorited || (p.fav_count > 0)) && !hasJoshua && !hasKona && !hasJason;

    // Three-way combinations
    if (hasJoshua && hasKona && hasJason)
        return { bg: '#FFF0F5', border: '#F48FB1', accent: '#AD1457', name: 'Rose Blossom'    };
    // Two-way combinations
    if (hasJoshua && hasKona)
        return { bg: '#F3E5F5', border: '#CE93D8', accent: '#7B1FA2', name: 'Orchid Purple'   };
    if (hasJoshua && hasJason)
        return { bg: '#E0F7FA', border: '#4DD0E1', accent: '#00838F', name: 'Ocean Teal'      };
    if (hasKona && hasJason)
        return { bg: '#FFF8E1', border: '#FFD54F', accent: '#F57F17', name: 'Honey Peach'     };
    // Single named users
    if (hasJoshua)
        return { bg: '#E3F2FD', border: '#90CAF9', accent: '#1565C0', name: 'Soothing Blue'   };
    if (hasKona)
        return { bg: '#FCE4EC', border: '#F48FB1', accent: '#C2185B', name: 'Soothing Pink'   };
    if (hasJason)
        return { bg: '#FFF3E0', border: '#FFCC80', accent: '#E65100', name: 'Soothing Orange' };
    // Community / guest favorites
    if (hasCommunity)
        return { bg: '#EDE7F6', border: '#CE93D8', accent: '#7E57C2', name: 'Lavender'        };
    return null;
}

export function generateStarHTML(p, bannerInfo) {
    return `
        <div class="star-wrapper" style="display:flex; flex-direction:row; align-items:center; justify-content:space-between; width:100%; flex-shrink:0; padding:0 20px;">
            <div id="feed-like-btn-${p.id}" class="like-count-display" onclick="event.stopPropagation(); window.toggleLike(${p.id});" style="cursor:pointer; opacity:${p.is_liked ? '1' : '0.6'}; font-size:0.95rem; background:rgba(255,255,255,0.95); color:#333; font-weight:bold; padding:8px 18px; border-radius:14px; box-shadow:0 2px 4px rgba(0,0,0,0.2); text-shadow:none; white-space:nowrap; transition: all 0.2s;">
                👍 Like <span id="feed-like-count-${p.id}">${p.like_count || 0}</span>
            </div>
            <div style="position:relative; display:inline-block;">
                <span class="fav-star ${p.is_favorited ? 'favorited' : ''}" onclick="window.toggleFavorite(event, ${p.id}, this)">
                    ${p.is_favorited ? '⭐' : '☆'}
                </span>
                <div class="fav-banner ${bannerInfo.show ? 'show' : ''}" ${bannerInfo.text === 'Favorite This Plant' ? `style="background: linear-gradient(135deg, var(--svd-accent, #4CAF50), var(--svd-row-label, #2E7D32));"` : ''}>
                     ${bannerInfo.text}
                </div>
            </div>
        </div>
    `;
}

export function renderSeasonalScaleJS(monthRangeStr, type) {
    const NOT_SPECIFIED = `<div style="flex: 1; text-align: center; color: rgba(255,255,255,0.4); font-size: 0.85rem; font-style: italic; background: rgba(0,0,0,0.1); border-radius: 20px; padding: 2px;">Not Specified</div>`;

    if (!monthRangeStr || monthRangeStr.trim() === '' || monthRangeStr === '[]') return NOT_SPECIFIED;

    // Convert to string and trim
    monthRangeStr = String(monthRangeStr).trim();

    // Reject PHP/SQL NULL stored as a literal string
    if (monthRangeStr === 'NULL' || monthRangeStr === 'null' || monthRangeStr === 'undefined') return NOT_SPECIFIED;

    // Reject 12-dot "no data" placeholder (all dots = nothing recorded)
    if (/^\.+$/.test(monthRangeStr)) return NOT_SPECIFIED;

    const letters = ['J', 'F', 'M', 'A', 'M', 'J', 'J', 'A', 'S', 'O', 'N', 'D'];
    const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    let activeMonths = new Set();

    // ── FORMAT 1: 12-char bitmap  e.g. "....MJJ....." or "JFMAMJJASOND" ──────
    // Exactly 12 characters containing only letters and/or dots.
    // The old check (monthRangeStr.includes('.')) broke plants whose bitmap has
    // NO dots — i.e. every month is active — so we match on length + charset only.
    if (/^[A-Za-z.]{12}$/.test(monthRangeStr)) {
        for (let i = 0; i < 12; i++) {
            if (monthRangeStr[i] !== '.') activeMonths.add(i);
        }

    // ── FORMAT 2: JSON numeric array  e.g. "[6,7,8]" ──────────────────────────
    } else if (monthRangeStr.startsWith('[')) {
        try {
            const arr = JSON.parse(monthRangeStr);
            if (Array.isArray(arr)) {
                arr.forEach(n => {
                    const idx = parseInt(n, 10) - 1;   // DB months are 1-based
                    if (idx >= 0 && idx < 12) activeMonths.add(idx);
                });
            }
        } catch (_) { /* malformed JSON — fall through with empty set */ }

    // ── FORMAT 3: "Jun-Aug" range (start–end, fills every month between) ──────
    } else if (/^[A-Za-z]{3}-[A-Za-z]{3}$/.test(monthRangeStr)) {
        const [startPart, endPart] = monthRangeStr.split('-');
        const startIdx = monthNames.findIndex(m => m.toLowerCase() === startPart.toLowerCase());
        const endIdx   = monthNames.findIndex(m => m.toLowerCase() === endPart.toLowerCase());
        if (startIdx !== -1 && endIdx !== -1) {
            if (endIdx >= startIdx) {
                for (let i = startIdx; i <= endIdx; i++) activeMonths.add(i);
            } else {
                // Wraps around year-end (e.g. Nov-Mar)
                for (let i = startIdx; i < 12; i++) activeMonths.add(i);
                for (let i = 0; i <= endIdx; i++) activeMonths.add(i);
            }
        }

    // ── FORMAT 4: Space/comma list  e.g. "Apr May Jun" or "Apr, May, Jun" ─────
    } else {
        const parts = monthRangeStr.replace(/[^a-zA-Z]/g, ' ').split(/\s+/);
        parts.forEach(p => {
            if (p.length >= 3) {
                const idx = monthNames.findIndex(m => m.toLowerCase() === p.substring(0, 3).toLowerCase());
                if (idx !== -1) activeMonths.add(idx);
            }
        });
    }

    // If grow bar has all 12 months active, use evergreen green color
    const allActive = type === 'grow' && activeMonths.size === 12;
    let activeClass = allActive ? 'active-evergreen' : (type === 'grow' ? 'active-grow' : 'active-bloom');
    let html = `<div class="timeline-pill" style="flex: 1;">`;
    for (let i = 0; i < 12; i++) {
        let isActive = activeMonths.has(i);
        html += `<div class="month-dot ${isActive ? activeClass : ''}">${letters[i]}</div>`;
    }
    html += `</div>`;
    return html;
}

export function renderEvergreenMonthBar() {
    // Generate random sparkle positions across the full pill
    const sparkles = Array.from({length: 10}, () => {
        const left = (Math.random() * 92 + 2).toFixed(1);
        const top  = (Math.random() * 12 + 2).toFixed(1);
        const dur  = (1.5 + Math.random() * 2.5).toFixed(2);
        const del  = (Math.random() * 2).toFixed(2);
        return `<div class="eg-dot" style="left:${left}%;top:${top}px;--d:${dur}s;--dl:${del}s;"></div>`;
    }).join('');
    return `<div style="position:relative;flex:1;overflow:hidden;border-radius:20px;
        background:#1a4d1a;height:26px;border:1px solid #4caf50;
        display:flex;align-items:center;justify-content:center;">
        <div style="position:absolute;top:0;bottom:0;left:0;width:30%;
            background:linear-gradient(90deg,transparent 0%,rgba(144,238,144,0.4) 50%,transparent 100%);
            animation:egSweep 3.5s ease-in-out infinite;
            z-index:1;pointer-events:none;"></div>
        <span style="position:relative;z-index:2;font-size:0.78rem;font-weight:900;
            letter-spacing:3px;color:#e8f5e9;
            text-shadow:0 0 8px #a5d6a7,0 0 18px #4caf50;">EVERGREEN</span>
        ${sparkles}
    </div>`;
}

// ── EVERGREEN / FLOWERLESS tags ───────────────────────────────────────────────
(function injectNoBloomCSS() {
    if (document.getElementById('evergreen-css')) return;
    const s = document.createElement('style');
    s.id = 'evergreen-css';
    s.textContent = `
        @keyframes egSweep {
            0%   { transform: translateX(-120%); }
            100% { transform: translateX(400%); }
        }
        @keyframes egSparkle {
            0%,100% { opacity:0; transform:scale(0.4); }
            50%     { opacity:1; transform:scale(1.3); }
        }
        /* EVERGREEN — lighter green */
        .eg-bar {
            position:relative; overflow:hidden; flex:1;
            background:#1a4d1a;
            border-radius:20px; height:24px;
            display:flex; align-items:center; justify-content:center;
            border:1px solid #4caf50;
        }
        .eg-sweep {
            position:absolute; top:0; bottom:0; left:0; width:30%;
            background:linear-gradient(90deg,transparent 0%,rgba(144,238,144,0.45) 50%,transparent 100%);
            animation:egSweep 3.5s ease-in-out infinite;
            z-index:1; pointer-events:none;
        }
        .eg-label {
            position:relative; z-index:2;
            font-size:0.68rem; font-weight:900; letter-spacing:3px;
            color:#a5d6a7;
            text-shadow:0 0 6px #66bb6a, 0 0 14px #2e7d32;
        }
        .eg-dot {
            position:absolute; border-radius:50%;
            width:3px; height:3px; background:#c8e6c9;
            z-index:15; pointer-events:none;
            animation:egSparkle var(--d,2s) var(--dl,0s) ease-in-out infinite;
        }
        /* FLOWERLESS — soft lavender */
        .fl-bar {
            position:relative; overflow:hidden; flex:1;
            background:#2a1f3d;
            border-radius:20px; height:24px;
            display:flex; align-items:center; justify-content:center;
            border:1px solid #9575cd;
        }
        .fl-sweep {
            position:absolute; top:0; bottom:0; left:0; width:30%;
            background:linear-gradient(90deg,transparent 0%,rgba(206,147,216,0.35) 50%,transparent 100%);
            animation:egSweep 4s ease-in-out infinite;
            z-index:1; pointer-events:none;
        }
        .fl-label {
            position:relative; z-index:2;
            font-size:0.78rem; font-weight:900; letter-spacing:3px;
            color:#f3e5f5;
            text-shadow:0 0 8px #ce93d8, 0 0 18px #9c27b0;
        }
        .fl-dot {
            position:absolute; border-radius:50%;
            width:3px; height:3px; background:#e1bee7;
            z-index:3; pointer-events:none;
            animation:egSparkle var(--d,2s) var(--dl,0s) ease-in-out infinite;
        }
    `;
    document.head.appendChild(s);
})();

const _BLOOM_DOTS = [
    {l:8,  t:4,  d:'2.1s', dl:'0s'},
    {l:22, t:14, d:'1.8s', dl:'0.5s'},
    {l:38, t:5,  d:'2.4s', dl:'1.0s'},
    {l:55, t:16, d:'1.9s', dl:'0.3s'},
    {l:70, t:4,  d:'2.2s', dl:'0.8s'},
    {l:85, t:14, d:'2.0s', dl:'1.3s'},
];

export function renderEvergreenTag() {
    const dotsHTML = _BLOOM_DOTS.map(d =>
        `<div class="eg-dot" style="left:${d.l}%;top:${d.t}px;--d:${d.d};--dl:${d.dl};"></div>`
    ).join('');
    return `<div class="eg-bar" title="Evergreen — foliage persists year-round, no bloom cycle">
        <div class="eg-sweep"></div>
        ${dotsHTML}
        <span class="eg-label">✦ EVERGREEN ✦</span>
    </div>`;
}

export function renderFlowerlessTag() {
    const dotsHTML = _BLOOM_DOTS.map(d =>
        `<div class="fl-dot" style="left:${d.l}%;top:${d.t}px;--d:${d.d};--dl:${d.dl};"></div>`
    ).join('');
    return `<div class="fl-bar" title="This plant does not produce flowers">
        <div class="fl-sweep"></div>
        ${dotsHTML}
        <span class="fl-label">FLOWERLESS</span>
    </div>`;
}

export function generateWaterCupHTML(waterText) {
    if (!waterText) return '';
    let darkHeight = 0, lightHeight = 0;
    const lowerText = waterText.toLowerCase();
    if (lowerText.includes('low') && lowerText.includes('average')) { darkHeight = 30; lightHeight = 30; } 
    else if (lowerText.includes('average') && lowerText.includes('high')) { darkHeight = 60; lightHeight = 30; } 
    else if (lowerText.includes('low') && lowerText.includes('high')) { darkHeight = 30; lightHeight = 60; } 
    else if (lowerText.includes('high') || lowerText.includes('wet')) { darkHeight = 90; } 
    else if (lowerText.includes('average') || lowerText.includes('medium')) { darkHeight = 60; } 
    else if (lowerText.includes('low') || lowerText.includes('dry')) { darkHeight = 30; } 
    else { lightHeight = 50; }
    return `
        <div class="glass-widget">
            <div class="water-scene">
                <div class="watering-can">🚿</div><div class="water-stream"></div>
                <div class="water-bucket">
                    <div class="water-layer water-dark" style="--final-height: ${darkHeight}%;"></div>
                    <div class="water-layer water-light" style="--final-height: ${lightHeight}%; bottom: ${darkHeight}%;"></div>
                </div>
            </div>
            <div class="widget-info">
                <span class="widget-label">WATER NEEDS</span>
                <span class="widget-value">${waterText}</span>
            </div>
        </div>`;
}

export function generateSunWidgetHTML(sunText) {
    if (!sunText) return '';
    const lower = sunText.toLowerCase();
    let skyClass = 'sky-full', cloudClass = 'cloud-hidden', cloudClass2 = 'cloud-hidden';
    if (lower.includes('full sun')) skyClass = 'sky-full';
    else if (lower.includes('partial')) { skyClass = 'sky-part'; cloudClass = 'cloud-partial'; }
    else if (lower.includes('shade')) { skyClass = 'sky-shade'; cloudClass = 'cloud-full'; cloudClass2 = 'cloud-second'; }
    return `
        <div class="glass-widget">
            <div class="sun-scene ${skyClass}">
                <div class="sun-orb"></div><div class="cloud-puff ${cloudClass}"></div>
                ${cloudClass2 !== 'cloud-hidden' ? `<div class="cloud-second ${cloudClass2}"></div>` : ''}
            </div>
            <div class="widget-info">
                <span class="widget-label">SUN EXPOSURE</span>
                <span class="widget-value">${sunText}</span>
            </div>
        </div>`;
}

export function generateZoneWidgetHTML(zoneText) {
    if (!zoneText) return '';
    const numbers = zoneText.match(/\d+/g);
    if (!numbers) return '';
    let minZone = parseInt(numbers[0]), maxZone = numbers.length > 1 ? parseInt(numbers[1]) : minZone;
    minZone = Math.max(1, Math.min(13, minZone)); maxZone = Math.max(1, Math.min(13, maxZone));
    if (minZone === maxZone) minZone = Math.max(1, maxZone - 2);
    const getHue = (z) => Math.max(0, 240 - ((z - 1) * 21.8));
    const minColor = `hsl(${getHue(minZone)}, 100%, 50%)`, maxColor = `hsl(${getHue(maxZone)}, 100%, 50%)`;
    const minH = Math.max(10, (minZone / 13) * 100), maxH = Math.min(100, Math.max(20, (maxZone / 13) * 100));
    return `
        <div class="glass-widget">
            <div class="therm-scene">
                <div class="therm-glass"><div class="therm-liquid therm-loop" style="--min-h: ${minH}%; --max-h: ${maxH}%; --c-min: ${minColor}; --c-max: ${maxColor};"></div></div>
                <div class="therm-bulb"><div class="therm-bulb-liquid therm-bulb-loop" style="--c-min: ${minColor}; --c-max: ${maxColor};"></div></div>
            </div>
            <div class="widget-info">
                <span class="widget-label">HARDINESS ZONE</span>
                <span class="widget-value">${zoneText}</span>
            </div>
        </div>`;
}

export function generateRulerWidgetHTML(text, label) {
    if (!text) return '';
    
    // 1. Extract the highest number from the text (supports decimals)
    const numbers = text.match(/\d+(\.\d+)?/g);
    if (!numbers) return '';
    const maxVal = Math.max(...numbers.map(Number));
    
    // 2. Determine Unit & Convert everything to a unified "Inches" scale
    const lowerText = text.toLowerCase();
    const isFeet = lowerText.includes('ft') || lowerText.includes('foot') || lowerText.includes("'");
    
    let totalInches = maxVal;
    if (isFeet) {
        totalInches = maxVal * 12; // Convert feet to inches
    }

    // 3. SCALING: We set 60 inches (5 feet) as the "100% full bar" mark.
    // This makes standard 3-4 ft plants look huge, while 6-12 inch plants stay small.
    let widthPct = (totalInches / 60) * 100;

    // 4. Clamp the limits. (Min 8% so you can always see it, Max 100% so it doesn't break the box)
    widthPct = Math.max(8, Math.min(100, widthPct));
    
    // 5. Set --ruler-width as a CSS custom property so the growRuler keyframe in style.css can read it.
    //    No inline <style> block needed — injected @keyframes via innerHTML are unreliable.
    return `
        <div class="glass-widget">
            <div class="ruler-scene">
                <div class="ruler-bar" style="--ruler-width: ${widthPct}%;"></div>
            </div>
            <div class="widget-info">
                <span class="widget-label">${label.toUpperCase()}</span>
                <span class="widget-value">${text}</span>
            </div>
        </div>`;
}

// ── pH Tube Widget ────────────────────────────────────────────────────────────
// Color scale: pH 6.0 = blue (hsl 240), pH 8.0 = red (hsl 0).
// Ranges produce a gradient (bottom = lower pH color, top = higher pH color).
// Liquid is set via inline style so it always renders regardless of CSS cascade.
export function generatePhWidgetHTML(phText) {
    if (!phText) return '';

    const nums = phText.match(/\d+(\.\d+)?/g);
    if (!nums) return '';

    const phVals = nums.map(Number);
    const minPh  = Math.min(...phVals);
    const maxPh  = Math.max(...phVals);

    // Map a pH value → hue: 6→240° (blue), 7→120° (green), 8→0° (red)
    // Clamp to the display range [6, 8] so values outside still get an end-color.
    function phHue(ph) {
        const t = Math.max(0, Math.min(1, (ph - 6) / 2)); // 0 at pH6, 1 at pH8
        return Math.round(240 - t * 240);                  // 240→0
    }

    function phColor(ph) {
        return `hsl(${phHue(ph)}, 82%, 48%)`;
    }

    const isRange   = phVals.length > 1 && minPh !== maxPh;
    const bottomCol = phColor(minPh);  // lower pH = more blue = bottom of liquid
    const topCol    = phColor(maxPh);  // higher pH = more red = top of liquid

    // Inline background so there's no CSS-variable dependency
    const liquidBg  = isRange
        ? `linear-gradient(to top, ${bottomCol}, ${topCol})`
        : bottomCol;

    const bubbles = [
        { x: '15%', s: '5px', d: '1.8s', de: '0s'   },
        { x: '55%', s: '4px', d: '2.4s', de: '0.6s' },
        { x: '75%', s: '6px', d: '2.0s', de: '1.2s' },
        { x: '35%', s: '3px', d: '2.9s', de: '1.8s' },
        { x: '48%', s: '4px', d: '1.6s', de: '0.9s' },
    ].map(b => `<div class="ph-b" style="--bx:${b.x};--bs:${b.s};--bd:${b.d};--bde:${b.de};"></div>`).join('');

    return `
        <div class="glass-widget">
            <div class="ph-tube-scene">
                <div class="ph-tube-neck"></div>
                <div class="ph-tube-body">
                    <div class="ph-liquid" style="background:${liquidBg};">
                        <div class="ph-bubbles">${bubbles}</div>
                    </div>
                </div>
            </div>
            <div class="widget-info">
                <span class="widget-label">SOIL PH</span>
                <span class="widget-value">${phText}</span>
            </div>
        </div>`;
}

// ── Edibility Widget ─────────────────────────────────────────────────────────
// fruit_edibility values : "Edible" | "Inedible" | "Toxic" | "Wildlife Only" | "None"
// toxicity values        : "Non-toxic" | "Toxic" | "Toxic to Dogs/Cats" |
//                          "Toxic to Humans" | "Toxic to Humans & Pets" | "Unknown"
// extra_details          : free text — "Culinary herb" etc. boosts toward Fully Edible
export function generateEdibilityWidgetHTML(fruitEdibility, toxicity, flowerBed, extraDetails) {
    const fe    = (fruitEdibility || '').trim();
    const feL   = fe.toLowerCase();
    const tox   = (toxicity       || '').trim();
    const toxL  = tox.toLowerCase();
    const bedL  = (flowerBed      || '').toLowerCase();
    const extL  = (extraDetails   || '').toLowerCase();

    // Skip when there is genuinely no edibility information
    if (!fe || feL === 'none') return '';

    const culinary = extL.includes('culinary') || extL.includes('edible') ||
                     extL.includes('herb') || extL.includes('vegetable') ||
                     extL.includes('fruit') || extL.includes('berry');

    let sceneClass, inner, valueDisplay;

    // ── Wildlife Only ────────────────────────────────────────────────────────
    if (feL === 'wildlife only' || bedL.includes('wildlife only')) {
        sceneClass   = 'edib-wild-scene';
        valueDisplay = 'Wildlife Only';
        inner = `<span class="edib-icon-t">🚫</span>
                 <span class="edib-icon-l">🦜</span>
                 <span class="edib-icon-r">🐿️</span>`;

    // ── Fully Edible ─────────────────────────────────────────────────────────
    // fruit_edibility = Edible  AND  toxicity = Non-toxic
    // also catches culinary herbs with Non-toxic status
    } else if (
        (feL === 'edible' && toxL === 'non-toxic') ||
        (toxL === 'non-toxic' && culinary) ||
        bedL.includes('fully edible')
    ) {
        sceneClass   = 'edib-full-scene';
        valueDisplay = 'Fully Edible';
        inner = `<span class="edib-icon-t">✅</span>
                 <span class="edib-icon-l">🧑</span>
                 <span class="edib-icon-c">🧒</span>
                 <span class="edib-icon-r">🐾</span>`;

    // ── Semi Edible ───────────────────────────────────────────────────────────
    // Edible fruit but plant has some toxicity risk,
    // OR plant is toxic to only one group (dogs/cats OR humans, not both)
    } else if (
        bedL.includes('semi edible') ||
        (feL === 'edible' && toxL !== 'non-toxic') ||
        toxL === 'toxic to dogs/cats' ||
        toxL === 'toxic to humans'
    ) {
        sceneClass   = 'edib-semi-scene';
        valueDisplay = 'Semi Edible';
        inner = `<span class="edib-icon-t">⚠️</span>
                 <span class="edib-icon-l">🧑</span>
                 <span class="edib-icon-r">🐾</span>`;

    // ── Completely Toxic ──────────────────────────────────────────────────────
    // fruit_edibility = Toxic, OR toxic to everyone, OR Inedible + dangerous toxicity
    } else if (
        feL === 'toxic' ||
        toxL === 'toxic to humans & pets' ||
        toxL === 'toxic' ||
        (feL === 'inedible' && toxL.includes('toxic')) ||
        bedL.includes('completely toxic')
    ) {
        sceneClass   = 'edib-toxic-scene';
        valueDisplay = 'Completely Toxic';
        inner = `<span class="edib-main">☠️</span>
                 <span class="edib-badge">🚫</span>`;

    } else {
        // Inedible but non-toxic (ornamental) — no widget needed
        return '';
    }

    return `
        <div class="glass-widget">
            <div class="edib-scene ${sceneClass}">${inner}</div>
            <div class="widget-info">
                <span class="widget-label">EDIBILITY</span>
                <span class="widget-value">${valueDisplay}</span>
            </div>
        </div>`;
}

// ── Deer & Rabbit Resistance Widget ──────────────────────────────────────────
// Values: Deer Resistant | Rabbit Resistant | Deer & Rabbit Resistant |
//         Not Resistant | Somewhat Resistant | Semi-Resistant
export function generateDeerRabbitWidgetHTML(resistanceText) {
    if (!resistanceText) return '';
    const t = resistanceText.toLowerCase();

    const hasDeer   = t.includes('deer');
    const hasRabbit = t.includes('rabbit');
    const isBoth    = t === 'both' || (t.includes('both') && !t.includes('not'));
    const notRes    = t.includes('not resistant');
    const somewhat  = t.includes('somewhat') || t.includes('semi');

    let sceneClass, inner;

    if (notRes) {
        // Both animals shown with red X
        sceneClass = 'dr-not';
        inner = `<span class="dr-left">🦌</span>
                 <span class="dr-right">🐇</span>
                 <span class="dr-mark">❌</span>`;

    } else if (somewhat) {
        // Both animals shown with yellow caution
        sceneClass = 'dr-some';
        inner = `<span class="dr-left">🦌</span>
                 <span class="dr-right">🐇</span>
                 <span class="dr-mark">⚠️</span>`;

    } else if (hasDeer && hasRabbit || isBoth) {
        // Both resistant (or value is literally 'Both') — both animals with green check
        sceneClass = 'dr-both-yes';
        inner = `<span class="dr-left">🦌</span>
                 <span class="dr-right">🐇</span>
                 <span class="dr-mark">✅</span>`;

    } else if (hasDeer) {
        // Deer only
        sceneClass = 'dr-deer-yes';
        inner = `<span class="dr-solo">🦌</span>
                 <span class="dr-mark">✅</span>`;

    } else if (hasRabbit) {
        // Rabbit only
        sceneClass = 'dr-rabbit-yes';
        inner = `<span class="dr-solo">🐇</span>
                 <span class="dr-mark">✅</span>`;

    } else {
        // Fallback — show both with caution
        sceneClass = 'dr-some';
        inner = `<span class="dr-left">🦌</span>
                 <span class="dr-right">🐇</span>
                 <span class="dr-mark">❓</span>`;
    }

    return `
        <div class="glass-widget">
            <div class="dr-scene ${sceneClass}">${inner}</div>
            <div class="widget-info">
                <span class="widget-label">DEER &amp; RABBIT</span>
                <span class="widget-value">${resistanceText}</span>
            </div>
        </div>`;
}

// ── Toxicity Widget ───────────────────────────────────────────────────────────
// Detects the toxicity category and renders a unique animated scene for each.
// Values: Non-toxic | Toxic | Toxic to Dogs/Cats | Toxic to Humans |
//         Toxic to Humans & Pets | Unknown
export function generateToxicityWidgetHTML(toxicityText) {
    if (!toxicityText) return '';
    const t = toxicityText.toLowerCase().trim();

    let sceneClass, inner;

    if (t.includes('non')) {
        // ── Non-toxic ──────────────────────────────────────────────────────
        sceneClass = 'tox-safe';
        inner = `<span class="tox-main">🛡️</span><span class="tox-badge">✅</span>`;

    } else if ((t.includes('human') || t.includes('people')) &&
               (t.includes('pet') || t.includes('dog') || t.includes('cat'))) {
        // ── Toxic to Humans & Pets ─────────────────────────────────────────
        sceneClass = 'tox-all';
        inner = `<span class="tox-icon-l">🧑</span>
                 <span class="tox-icon-r">🐾</span>
                 <span class="tox-center">☠️</span>`;

    } else if (t.includes('human') || t.includes('people')) {
        // ── Toxic to Humans ────────────────────────────────────────────────
        sceneClass = 'tox-humans';
        inner = `<span class="tox-main">🧑</span><span class="tox-badge">⚠️</span>`;

    } else if (t.includes('dog') || t.includes('cat') || t.includes('pet')) {
        // ── Toxic to Dogs/Cats ─────────────────────────────────────────────
        sceneClass = 'tox-pets';
        inner = `<span class="tox-icon-l">🐕</span>
                 <span class="tox-icon-r">🐈</span>
                 <span class="tox-center">⚠️</span>`;

    } else if (t.includes('unknown')) {
        // ── Unknown ────────────────────────────────────────────────────────
        sceneClass = 'tox-unknown';
        inner = `<span class="tox-main">❓</span>`;

    } else if (t.includes('toxic')) {
        // ── Generic Toxic ──────────────────────────────────────────────────
        sceneClass = 'tox-generic';
        inner = `<span class="tox-main">☠️</span><span class="tox-badge">⚠️</span>`;

    } else {
        // Fallback — treat as unknown
        sceneClass = 'tox-unknown';
        inner = `<span class="tox-main">❓</span>`;
    }

    return `
        <div class="glass-widget">
            <div class="tox-scene ${sceneClass}">${inner}</div>
            <div class="widget-info">
                <span class="widget-label">TOXICITY</span>
                <span class="widget-value">${toxicityText}</span>
            </div>
        </div>`;
}

// ── Quantity Garden Widget ────────────────────────────────────────────────────
// Shows 1 large flower for qty=1, scaling up to a packed forest for large numbers.
// Positions are pre-computed so dense layouts fill the scene naturally.
export function generateQuantityWidgetHTML(quantityText) {
    if (!quantityText) return '';
    // qty can be a comma/plus-separated string like "2,1" — sum all parts
    const qty = String(quantityText).split(/[,+]/).reduce((sum, part) => {
        const n = parseInt(part.trim(), 10);
        return sum + (isNaN(n) ? 0 : n);
    }, 0);
    if (qty <= 0) return '';

    // Tier: how many flower icons to show and at what font-size
    let count, size;
    if      (qty === 1) { count = 1;  size = 26; }
    else if (qty <= 3)  { count = qty; size = 22; }
    else if (qty <= 6)  { count = 5;  size = 18; }
    else if (qty <= 10) { count = 7;  size = 15; }
    else if (qty <= 20) { count = 11; size = 12; }
    else if (qty <= 50) { count = 18; size = 10; }
    else                { count = 25; size = 8;  }

    // 25 hand-placed positions [left%, top%] inside the scene (above the soil line)
    const positions = [
        [50, 44], [28, 52], [72, 52], [40, 32], [60, 32],
        [16, 44], [84, 44], [50, 62], [33, 62], [67, 62],
        [22, 22], [78, 22], [50, 18], [38, 74], [62, 74],
        [10, 56], [90, 56], [25, 38], [75, 38], [44, 48],
        [56, 48], [12, 28], [88, 28], [36, 22], [64, 22],
    ];

    const flowerPool = ['\u{1F338}', '\u{1F33A}', '\u{1F33B}', '\u{1F337}', '\u{1F33C}', '\u{1F339}'];

    const flowers = positions.slice(0, count).map(([lx, ly], i) => {
        const icon  = flowerPool[i % flowerPool.length];
        const delay = (i * 0.28).toFixed(2);
        return `<span class="qty-flower" style="left:${lx}%;top:${ly}%;font-size:${size}px;animation-delay:${delay}s;">${icon}</span>`;
    }).join('');

    return `
        <div class="glass-widget">
            <div class="qty-scene">
                ${flowers}
                <div class="qty-ground"></div>
            </div>
            <div class="widget-info">
                <span class="widget-label">QUANTITY</span>
                <span class="widget-value">${quantityText}</span>
            </div>
        </div>`;
}

// -- Cost Widget ---------------------------------------------------------------
export function generateCostWidgetHTML(costText) {
    if (!costText || costText.trim() === '$0.00' || costText.trim() === '0') return '';
    return `
        <div class="glass-widget">
            <div class="cost-scene">
                <div class="cost-coin">
                    <div class="cost-coin-face cost-coin-heads">$</div>
                    <div class="cost-coin-face cost-coin-tails">¢</div>
                </div>
            </div>
            <div class="widget-info">
                <span class="widget-label">COST</span>
                <span class="widget-value">${costText}</span>
            </div>
        </div>`;
}

// -- Attracts Widget -----------------------------------------------------------
// Returns one dash-pill per detected creature so each has its own icon + label.
export function generateAttractsWidgetHTML(attracts) {
    if (!attracts || attracts === 'NULL' || attracts === 'null' || attracts === 'undefined') return '';

    // Order matters: more-specific keys first (hummingbird before bird)
    const CREATURE_MAP = [
        { keys: ['hummingbird'],             icon: HUMMINGBIRD_SVG, label: 'Hummingbirds', animCls: 'att-hummingbird' },
        { keys: ['butterfly', 'butterfl'],   icon: '🦋', label: 'Butterflies',  animCls: 'att-butterfly'  },
        { keys: ['bee', 'bumble'],           icon: '🐝', label: 'Bees',         animCls: 'att-bee'        },
        { keys: ['moth'],                    icon: '🦋', label: 'Moths',         animCls: 'att-moth'       },
        { keys: ['dragonfly'],               icon: '🪲', label: 'Dragonflies',   animCls: 'att-butterfly'  },
        { keys: ['songbird', 'bird'],        icon: CARDINAL_SVG,  label: 'Birds',     animCls: 'att-bird-anim'    },
        { keys: ['squirrel'],                icon: SQUIRREL_SVG,  label: 'Squirrels', animCls: 'att-squirrel'     },
        { keys: ['pollinator','pllinator'],  icon: '🌸', label: 'Pollinators',   animCls: ''               },
        { keys: ['beetle'],                  icon: '🐞', label: 'Beetles',       animCls: ''               },
        { keys: ['wasp'],                    icon: '🐝', label: 'Wasps',         animCls: 'att-bee'        },
        { keys: ['fly', 'flies'],            icon: '🪰', label: 'Flies',         animCls: ''               },
    ];

    const lower = attracts.toLowerCase();
    const seenLabels = new Set();
    const pills = [];

    for (const { keys, icon, label, animCls } of CREATURE_MAP) {
        if (keys.some(k => lower.includes(k)) && !seenLabels.has(label)) {
            seenLabels.add(label);
            pills.push(`<div class="dash-pill att-creature-pill">
                <span class="pill-label">ATTRACTS</span>
                <div class="att-creature-row">
                    <span class="att-icon ${animCls}">${icon}</span>
                    <span class="att-creature-name">${label}</span>
                </div>
            </div>`);
        }
    }

    // Fallback: unrecognised value — show raw text with a flower
    if (!pills.length) {
        pills.push(`<div class="dash-pill att-creature-pill">
            <span class="pill-label">ATTRACTS</span>
            <div class="att-creature-row">
                <span class="att-icon">🌼</span>
                <span class="att-creature-name">${attracts}</span>
            </div>
        </div>`);
    }

    return pills.join('\n');
}

// -- Aromatics Widget ----------------------------------------------------------
// Detects scent profile from free-text and shows matching emoji(s) + description.
export function generateAromaticsWidgetHTML(aromatics) {
    if (!aromatics || aromatics === 'NULL' || aromatics === 'null' || aromatics === 'undefined') return '';
    if (aromatics.trim().length < 3) return '';

    const SCENT_MAP = [
        { keys: ['honey','honeysuckle','sweet'],                                            icon: '🍯', profile: 'Sweet & Honey'  },
        { keys: ['floral','rose','jasmine','lilac','bloom','blossom','lavend','violet'],    icon: '🌸', profile: 'Floral'         },
        { keys: ['mint','minty','spearmint','peppermint','menthol'],                        icon: '🌿', profile: 'Minty'          },
        { keys: ['citrus','lemon','lime','orange peel','lemony'],                           icon: '🍋', profile: 'Citrus'         },
        { keys: ['spice','spicy','pepper','cinnamon','clove','anise','licorice'],           icon: '🌶️', profile: 'Spicy & Warm'   },
        { keys: ['vanilla','cream','custard'],                                              icon: '🍦', profile: 'Vanilla'        },
        { keys: ['fruit','berry','strawberry','apricot','peach','apple','pineapple'],      icon: '🍓', profile: 'Fruity'         },
        { keys: ['earthy','earth','soil','musty','fungal'],                                 icon: '🍄', profile: 'Earthy'         },
        { keys: ['pine','resin','conifer','evergreen','cedar','wood','woody','forest'],     icon: '🌲', profile: 'Woody & Pine'   },
        { keys: ['herb','herbal','sage','thyme','rosemary','basil'],                        icon: '🌱', profile: 'Herbal'         },
        { keys: ['musk','musky','animal','pungent'],                                        icon: '🌫️', profile: 'Musky'          },
        { keys: ['fragrant','scented','aromatic','perfume','scent'],                        icon: '💐', profile: 'Fragrant'       },
        { keys: ['faint','subtle','mild','light','slight'],                                 icon: '✨', profile: 'Subtle'         },
        { keys: ['strong','intense','powerful','heady'],                                    icon: '💥', profile: 'Intense'        },
    ];

    const lower = aromatics.toLowerCase();
    const detected = [];
    const seenProfiles = new Set();

    for (const { keys, icon, profile } of SCENT_MAP) {
        if (keys.some(k => lower.includes(k)) && !seenProfiles.has(profile)) {
            seenProfiles.add(profile);
            detected.push({ icon, profile });
        }
    }

    if (!detected.length) detected.push({ icon: '💐', profile: 'Aromatic' });

    const iconRow = detected.slice(0, 4).map(d =>
        `<span class="arom-icon-badge" title="${d.profile}">${d.icon}</span>`
    ).join('');

    const profileLabel = detected.slice(0, 3).map(d => d.profile).join(' · ');

    return `<div class="dash-pill arom-pill full-width">
        <span class="pill-label">AROMATICS</span>
        <div class="arom-body">
            <div class="arom-icons">${iconRow}</div>
            <div class="arom-text">
                <span class="arom-profile">${profileLabel}</span>
                <span class="arom-desc">${aromatics}</span>
            </div>
        </div>
    </div>`;
}

// -- Fertilizer Widget ---------------------------------------------------------
// Espoma product bag images — pulled from espoma.com product pages.
const FERT_IMG = {
    'holly-tone':     'https://www.espoma.com/wp-content/uploads/2023/07/Holly-tone_1667_square_trans-300x300.png',
    'rose-tone':      'https://www.espoma.com/wp-content/uploads/2023/07/RoseTone2023_square-300x300.png',
    'azalea-tone':    'https://www.espoma.com/wp-content/uploads/2023/07/Azalea-tone_1667_square_trans-300x300.png',
    'berry-tone':     'https://www.espoma.com/wp-content/uploads/2023/07/Berry-tone_1667_square_trans-300x300.png',
    'bio-tone':       'https://www.espoma.com/wp-content/uploads/2026/03/bio-tone-starter-plus-BTSP4-150x150.png',
    'bloom-booster':  'https://www.espoma.com/wp-content/uploads/2023/07/Flower-tone_1667_square_trans-300x300.png',
    'bulb-tone':      'https://www.espoma.com/wp-content/uploads/2023/07/Bulb-tone_1667_square_trans-300x300.png',
    'citrus-tone':    'https://www.espoma.com/wp-content/uploads/2026/03/citrus-tone-CT8-150x150.png',
    'evergreen-tone': 'https://www.espoma.com/wp-content/uploads/2026/03/evergreen-tone-ET8-150x150.png',
    'flower-tone':    'https://www.espoma.com/wp-content/uploads/2023/07/Flower-tone_1667_square_trans-300x300.png',
    'garden-tone':    'https://www.espoma.com/wp-content/uploads/2023/07/Garden-tone_1667_square_trans-300x300.png',
    'iron-tone':      'https://www.espoma.com/wp-content/uploads/2026/03/iron-tone-IT5-150x150.png',
    'palm-tone':      'https://www.espoma.com/wp-content/uploads/2023/07/Palm-Tone-Updated-1-150x150.png',
    'plant-tone':     'https://www.espoma.com/wp-content/uploads/2026/03/plant-tone-PT8-150x150.png',
    'tomato-tone':    'https://www.espoma.com/wp-content/uploads/2023/07/TomatoTone2023_square-300x300.png',
    'tree-tone':      'https://www.espoma.com/wp-content/uploads/2026/03/tree-tone-TR4-150x150.png',
    'generic':        'https://www.espoma.com/wp-content/uploads/2026/03/plant-tone-PT8-150x150.png',
};

// DEAD_FERT_SVG_START — the old hand-drawn SVG block has been replaced above.
// Keeping this comment as a tombstone so diffs are traceable.
const FERT_SVG = {

    'holly-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- holly leaves -->
        <path d="M10 20 Q6 14 12 10 Q10 16 16 14 Q12 18 18 18 Q12 22 16 26 Q10 24 12 28 Q6 24 10 20Z" fill="#2E7D32" stroke="#1B5E20" stroke-width="0.8"/>
        <path d="M22 20 Q26 14 24 10 Q22 16 18 14 Q22 18 16 18 Q22 22 18 26 Q24 24 24 28 Q28 24 22 20Z" fill="#388E3C" stroke="#1B5E20" stroke-width="0.8"/>
        <!-- berries -->
        <circle cx="18" cy="17" r="3.2" fill="#C62828"/>
        <circle cx="14" cy="19" r="2.5" fill="#D32F2F"/>
        <circle cx="22" cy="19" r="2.5" fill="#D32F2F"/>
        <circle cx="18" cy="22" r="2.2" fill="#B71C1C"/>
        <!-- berry shine -->
        <circle cx="16.8" cy="15.8" r="0.8" fill="rgba(255,255,255,0.5)"/>
    </svg>`,

    'rose-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- stem -->
        <path d="M18 32 Q16 26 18 20" stroke="#388E3C" stroke-width="2" fill="none" stroke-linecap="round"/>
        <!-- leaves -->
        <path d="M18 26 Q12 24 11 20 Q16 22 18 26Z" fill="#43A047"/>
        <path d="M18 26 Q24 24 25 20 Q20 22 18 26Z" fill="#43A047"/>
        <!-- outer petals -->
        <ellipse cx="18" cy="15" rx="5" ry="7" fill="#E53935" transform="rotate(0 18 15)"/>
        <ellipse cx="18" cy="15" rx="5" ry="7" fill="#E53935" transform="rotate(72 18 15)"/>
        <ellipse cx="18" cy="15" rx="5" ry="7" fill="#EF5350" transform="rotate(144 18 15)"/>
        <ellipse cx="18" cy="15" rx="5" ry="7" fill="#E53935" transform="rotate(216 18 15)"/>
        <ellipse cx="18" cy="15" rx="5" ry="7" fill="#EF5350" transform="rotate(288 18 15)"/>
        <!-- centre -->
        <circle cx="18" cy="15" r="4" fill="#B71C1C"/>
        <circle cx="17" cy="14" r="1.2" fill="rgba(255,255,255,0.3)"/>
    </svg>`,

    'azalea-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- stem -->
        <line x1="18" y1="34" x2="18" y2="24" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
        <!-- 5 trumpet petals -->
        <ellipse cx="18" cy="14" rx="4.5" ry="7" fill="#F06292" transform="rotate(0 18 18)"/>
        <ellipse cx="18" cy="14" rx="4.5" ry="7" fill="#EC407A" transform="rotate(72 18 18)"/>
        <ellipse cx="18" cy="14" rx="4.5" ry="7" fill="#F06292" transform="rotate(144 18 18)"/>
        <ellipse cx="18" cy="14" rx="4.5" ry="7" fill="#EC407A" transform="rotate(216 18 18)"/>
        <ellipse cx="18" cy="14" rx="4.5" ry="7" fill="#F06292" transform="rotate(288 18 18)"/>
        <!-- throat -->
        <circle cx="18" cy="18" r="4" fill="#AD1457"/>
        <!-- stamens -->
        <line x1="18" y1="18" x2="14" y2="11" stroke="#fff" stroke-width="0.8"/>
        <line x1="18" y1="18" x2="22" y2="11" stroke="#fff" stroke-width="0.8"/>
        <line x1="18" y1="18" x2="18" y2="10" stroke="#fff" stroke-width="0.8"/>
        <circle cx="14" cy="11" r="0.9" fill="#FFF176"/>
        <circle cx="22" cy="11" r="0.9" fill="#FFF176"/>
        <circle cx="18" cy="10" r="0.9" fill="#FFF176"/>
    </svg>`,

    'berry-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- branch -->
        <path d="M8 30 Q18 20 28 14" stroke="#5D4037" stroke-width="2" fill="none" stroke-linecap="round"/>
        <path d="M18 24 Q22 18 20 12" stroke="#5D4037" stroke-width="1.5" fill="none"/>
        <!-- berries cluster -->
        <circle cx="12" cy="26" r="4" fill="#6A1B9A"/>
        <circle cx="20" cy="20" r="4" fill="#7B1FA2"/>
        <circle cx="27" cy="14" r="4" fill="#8E24AA"/>
        <circle cx="20" cy="13" r="3.5" fill="#9C27B0"/>
        <circle cx="28" cy="22" r="3.5" fill="#6A1B9A"/>
        <!-- shine -->
        <circle cx="10.5" cy="24.5" r="1.2" fill="rgba(255,255,255,0.45)"/>
        <circle cx="18.5" cy="18.5" r="1.2" fill="rgba(255,255,255,0.45)"/>
        <circle cx="25.5" cy="12.5" r="1.2" fill="rgba(255,255,255,0.45)"/>
        <!-- leaf -->
        <path d="M18 24 Q14 18 18 14 Q22 18 18 24Z" fill="#2E7D32"/>
    </svg>`,

    'bio-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- soil mound -->
        <ellipse cx="18" cy="28" rx="13" ry="5" fill="#795548"/>
        <!-- organic matter dots -->
        <circle cx="11" cy="27" r="1.5" fill="#5D4037"/>
        <circle cx="18" cy="25" r="1.5" fill="#5D4037"/>
        <circle cx="25" cy="27" r="1.5" fill="#5D4037"/>
        <!-- main leaf -->
        <path d="M18 24 Q8 18 10 8 Q18 12 18 24Z" fill="#43A047"/>
        <path d="M18 24 Q28 18 26 8 Q18 12 18 24Z" fill="#66BB6A"/>
        <!-- center vein -->
        <line x1="18" y1="24" x2="18" y2="9" stroke="#2E7D32" stroke-width="1" stroke-linecap="round"/>
        <!-- side veins -->
        <line x1="18" y1="16" x2="13" y2="13" stroke="#2E7D32" stroke-width="0.7"/>
        <line x1="18" y1="16" x2="23" y2="13" stroke="#2E7D32" stroke-width="0.7"/>

        <line x1="18" y1="16" x2="23" y2="13" stroke="#2E7D32" stroke-width="0.7"/>
        <!-- organic circle - worm casting indicator -->
        <circle cx="25" cy="27" r="2" fill="#4E342E" opacity="0.6"/>
    </svg>`,

    'bloom-booster': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- stem -->
        <line x1="18" y1="34" x2="18" y2="22" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
        <!-- 6 bright petals -->
        <ellipse cx="18" cy="14" rx="4" ry="7" fill="#FF6F00" transform="rotate(0 18 18)"/>
        <ellipse cx="18" cy="14" rx="4" ry="7" fill="#FFB300" transform="rotate(60 18 18)"/>
        <ellipse cx="18" cy="14" rx="4" ry="7" fill="#FF6F00" transform="rotate(120 18 18)"/>
        <ellipse cx="18" cy="14" rx="4" ry="7" fill="#FFB300" transform="rotate(180 18 18)"/>
        <ellipse cx="18" cy="14" rx="4" ry="7" fill="#FF6F00" transform="rotate(240 18 18)"/>
        <ellipse cx="18" cy="14" rx="4" ry="7" fill="#FFB300" transform="rotate(300 18 18)"/>
        <circle cx="18" cy="18" r="5.5" fill="#FDD835"/>
        <circle cx="18" cy="18" r="2.5" fill="#F57F17"/>
    </svg>`,

    'bulb-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- bulb body -->
        <path d="M10 22 Q10 14 18 12 Q26 14 26 22 Q26 30 18 32 Q10 30 10 22Z" fill="#8D6E63"/>
        <!-- tunic lines -->
        <path d="M12 20 Q18 18 24 20" stroke="#6D4C41" stroke-width="0.8" fill="none"/>
        <path d="M11 24 Q18 22 25 24" stroke="#6D4C41" stroke-width="0.8" fill="none"/>
        <!-- roots -->
        <line x1="15" y1="32" x2="13" y2="36" stroke="#5D4037" stroke-width="1.2" stroke-linecap="round"/>
        <line x1="18" y1="33" x2="18" y2="36" stroke="#5D4037" stroke-width="1.2" stroke-linecap="round"/>
        <line x1="21" y1="32" x2="23" y2="36" stroke="#5D4037" stroke-width="1.2" stroke-linecap="round"/>
        <!-- green shoot -->
        <path d="M18 12 Q17 7 18 3" stroke="#43A047" stroke-width="2.5" fill="none" stroke-linecap="round"/>
        <path d="M18 8 Q14 5 12 2" stroke="#66BB6A" stroke-width="1.5" fill="none" stroke-linecap="round"/>
        <path d="M18 8 Q22 5 24 2" stroke="#66BB6A" stroke-width="1.5" fill="none" stroke-linecap="round"/>
    </svg>`,

    'citrus-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- orange fruit -->
        <circle cx="18" cy="20" r="13" fill="#FF8F00"/>
        <circle cx="18" cy="20" r="11" fill="#FFA000"/>
        <!-- slice segments -->
        <line x1="18" y1="9" x2="18" y2="31" stroke="#FF6F00" stroke-width="1" opacity="0.5"/>
        <line x1="8" y1="14" x2="28" y2="26" stroke="#FF6F00" stroke-width="1" opacity="0.5"/>
        <line x1="8" y1="26" x2="28" y2="14" stroke="#FF6F00" stroke-width="1" opacity="0.5"/>
        <!-- center pip -->
        <circle cx="18" cy="20" r="3" fill="#FFE082"/>
        <!-- leaf -->
        <path d="M18 9 Q14 4 16 1 Q20 4 18 9Z" fill="#388E3C"/>
    </svg>`,

    'evergreen-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- trunk -->
        <rect x="16" y="28" width="4" height="7" rx="1" fill="#5D4037"/>
        <!-- bottom tier -->
        <polygon points="18,16 6,30 30,30" fill="#2E7D32"/>
        <!-- middle tier -->
        <polygon points="18,9 8,22 28,22" fill="#388E3C"/>
        <!-- top tier -->
        <polygon points="18,3 11,15 25,15" fill="#43A047"/>
        <!-- snow tips -->
        <circle cx="18" cy="3" r="1.5" fill="white" opacity="0.7"/>
        <circle cx="11" cy="15" r="1" fill="white" opacity="0.5"/>
        <circle cx="25" cy="15" r="1" fill="white" opacity="0.5"/>
    </svg>`,

    'flower-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- stem with leaf -->
        <line x1="18" y1="34" x2="18" y2="20" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
        <path d="M18 27 Q12 24 11 20 Q16 22 18 27Z" fill="#43A047"/>
        <!-- 5 petals -->
        <ellipse cx="18" cy="12" rx="4" ry="7" fill="#CE93D8" transform="rotate(0 18 18)"/>
        <ellipse cx="18" cy="12" rx="4" ry="7" fill="#BA68C8" transform="rotate(72 18 18)"/>
        <ellipse cx="18" cy="12" rx="4" ry="7" fill="#CE93D8" transform="rotate(144 18 18)"/>
        <ellipse cx="18" cy="12" rx="4" ry="7" fill="#BA68C8" transform="rotate(216 18 18)"/>
        <ellipse cx="18" cy="12" rx="4" ry="7" fill="#CE93D8" transform="rotate(288 18 18)"/>
        <!-- center -->
        <circle cx="18" cy="18" r="5" fill="#FDD835"/>
        <circle cx="18" cy="18" r="2.5" fill="#F57F17"/>
    </svg>`,

    'garden-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- soil line -->
        <rect x="3" y="27" width="30" height="7" rx="2" fill="#795548"/>
        <!-- 3 sprouts -->
        <line x1="10" y1="27" x2="10" y2="18" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
        <line x1="18" y1="27" x2="18" y2="13" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
        <line x1="26" y1="27" x2="26" y2="18" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
        <!-- leaves -->
        <path d="M10 22 Q6 18 8 14 Q12 18 10 22Z" fill="#43A047"/>
        <path d="M10 22 Q14 18 12 14 Q8 18 10 22Z" fill="#66BB6A"/>
        <path d="M18 17 Q13 13 15 8 Q20 12 18 17Z" fill="#43A047"/>
        <path d="M18 17 Q23 13 21 8 Q16 12 18 17Z" fill="#66BB6A"/>
        <path d="M26 22 Q22 18 24 14 Q28 18 26 22Z" fill="#43A047"/>
        <path d="M26 22 Q30 18 28 14 Q24 18 26 22Z" fill="#66BB6A"/>
    </svg>`,

    'iron-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- leaf showing iron deficiency recovery -->
        <!-- yellowed/green leaf -->
        <path d="M18 32 Q8 24 8 14 Q18 10 28 14 Q28 24 18 32Z" fill="#C5E1A5"/>
        <!-- green veins (healthy) -->
        <line x1="18" y1="32" x2="18" y2="12" stroke="#388E3C" stroke-width="1.5" stroke-linecap="round"/>
        <line x1="18" y1="22" x2="12" y2="17" stroke="#388E3C" stroke-width="1" stroke-linecap="round"/>
        <line x1="18" y1="22" x2="24" y2="17" stroke="#388E3C" stroke-width="1" stroke-linecap="round"/>
        <line x1="18" y1="27" x2="13" y2="23" stroke="#388E3C" stroke-width="0.8"/>
        <line x1="18" y1="27" x2="23" y2="23" stroke="#388E3C" stroke-width="0.8"/>
        <!-- Fe symbol -->
        <text x="14" y="20" font-family="sans-serif" font-size="7" font-weight="bold" fill="#1565C0">Fe</text>
    </svg>`,

    'palm-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- trunk -->
        <path d="M16 34 Q15 20 17 8" stroke="#8D6E63" stroke-width="3" fill="none" stroke-linecap="round"/>
        <path d="M20 34 Q19 20 17 8" stroke="#795548" stroke-width="2" fill="none" stroke-linecap="round"/>
        <!-- fronds -->
        <path d="M17 8 Q4 4 2 10" stroke="#388E3C" stroke-width="2.5" fill="none" stroke-linecap="round"/>
        <path d="M17 8 Q8 2 10 -2" stroke="#43A047" stroke-width="2" fill="none" stroke-linecap="round"/>
        <path d="M17 8 Q17 1 18 -2" stroke="#388E3C" stroke-width="2.5" fill="none" stroke-linecap="round"/>
        <path d="M17 8 Q26 2 28 -2" stroke="#43A047" stroke-width="2" fill="none" stroke-linecap="round"/>
        <path d="M17 8 Q30 4 34 10" stroke="#388E3C" stroke-width="2.5" fill="none" stroke-linecap="round"/>
        <!-- coconuts -->
        <circle cx="15" cy="10" r="2.5" fill="#795548"/>
        <circle cx="19" cy="9" r="2.5" fill="#5D4037"/>
    </svg>`,

    'plant-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- pot -->
        <path d="M10 28 L12 35 L24 35 L26 28 Z" fill="#EF6C00"/>
        <rect x="9" y="25" width="18" height="4" rx="2" fill="#E65100"/>
        <!-- stem -->
        <line x1="18" y1="25" x2="18" y2="14" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
        <!-- leaves -->
        <path d="M18 20 Q10 16 9 10 Q16 13 18 20Z" fill="#43A047"/>
        <path d="M18 20 Q26 16 27 10 Q20 13 18 20Z" fill="#66BB6A"/>
        <path d="M18 14 Q14 8 15 4 Q20 8 18 14Z" fill="#43A047"/>
    </svg>`,

    'tomato-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- tomato body -->
        <circle cx="18" cy="22" r="12" fill="#E53935"/>
        <circle cx="18" cy="22" r="10" fill="#EF5350"/>
        <!-- tomato segments -->
        <path d="M18 10 Q18 22 18 34" stroke="#C62828" stroke-width="0.8" fill="none" opacity="0.4"/>
        <path d="M6 22 Q18 22 30 22" stroke="#C62828" stroke-width="0.8" fill="none" opacity="0.4"/>
        <!-- stem + calyx -->
        <line x1="18" y1="10" x2="18" y2="6" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
        <path d="M18 10 Q14 6 12 8" stroke="#43A047" stroke-width="1.5" fill="none" stroke-linecap="round"/>
        <path d="M18 10 Q22 6 24 8" stroke="#43A047" stroke-width="1.5" fill="none" stroke-linecap="round"/>
        <path d="M18 10 Q16 5 18 3" stroke="#43A047" stroke-width="1.5" fill="none" stroke-linecap="round"/>
        <!-- shine -->
        <ellipse cx="14" cy="17" rx="3" ry="2" fill="rgba(255,255,255,0.25)" transform="rotate(-20 14 17)"/>
    </svg>`,

    'tree-tone': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- trunk -->
        <rect x="15" y="26" width="6" height="9" rx="2" fill="#795548"/>
        <!-- canopy: 3 overlapping circles -->
        <circle cx="12" cy="20" r="9" fill="#388E3C"/>
        <circle cx="22" cy="20" r="9" fill="#43A047"/>
        <circle cx="17" cy="14" r="9" fill="#2E7D32"/>
        <!-- highlight -->
        <circle cx="14" cy="11" r="4" fill="#66BB6A" opacity="0.35"/>
    </svg>`,

    'generic': `<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
        <!-- soil -->
        <ellipse cx="18" cy="31" rx="11" ry="4" fill="#795548"/>
        <!-- stem -->
        <line x1="18" y1="31" x2="18" y2="16" stroke="#388E3C" stroke-width="2.5" stroke-linecap="round"/>
        <!-- leaves -->
        <path d="M18 22 Q9 18 8 11 Q16 15 18 22Z" fill="#43A047"/>
        <path d="M18 22 Q27 18 28 11 Q20 15 18 22Z" fill="#66BB6A"/>
        <!-- seedling tip -->
        <circle cx="18" cy="14" r="3" fill="#A5D6A7"/>
    </svg>`,

};

/**
 * Map fertilizer text → Espoma product bag image URL.
 */
function getFertImage(f) {
    if (f.includes('holly'))                          return FERT_IMG['holly-tone'];
    if (f.includes('rose'))                           return FERT_IMG['rose-tone'];
    if (f.includes('azalea'))                         return FERT_IMG['azalea-tone'];
    if (f.includes('berry'))                          return FERT_IMG['berry-tone'];
    if (f.includes('bio'))                            return FERT_IMG['bio-tone'];
    if (f.includes('bloom') || f.includes('booster')) return FERT_IMG['bloom-booster'];
    if (f.includes('bulb'))                           return FERT_IMG['bulb-tone'];
    if (f.includes('citrus'))                         return FERT_IMG['citrus-tone'];
    if (f.includes('evergreen'))                      return FERT_IMG['evergreen-tone'];
    if (f.includes('flower'))                         return FERT_IMG['flower-tone'];
    if (f.includes('garden'))                         return FERT_IMG['garden-tone'];
    if (f.includes('iron'))                           return FERT_IMG['iron-tone'];
    if (f.includes('palm'))                           return FERT_IMG['palm-tone'];
    if (f.includes('plant'))                          return FERT_IMG['plant-tone'];
    if (f.includes('tomato'))                         return FERT_IMG['tomato-tone'];
    if (f.includes('tree'))                           return FERT_IMG['tree-tone'];
    return FERT_IMG['generic'];
}

// -- Fertilizer Widget ---------------------------------------------------------
export function generateFertilizerWidgetHTML(fertText) {
    if (!fertText) return '';
    const url = getFertImage(fertText.toLowerCase());
    return `
        <div class="glass-widget">
            <div class="fert-scene">
                <img src="${url}" alt="${fertText}" width="40" height="40"
                     style="object-fit:contain;display:block;"
                     onerror="this.style.opacity='0'"/>
            </div>
            <div class="widget-info">
                <span class="widget-label">FERTILIZER</span>
                <span class="widget-value">${fertText}</span>
            </div>
        </div>`;
}

// -- Growth Rate Widget --------------------------------------------------------
// Fast = 3 arrows, Moderate = 2, Slow = 1
export function generateGrowthRateWidgetHTML(growthRate) {
    if (!growthRate) return '';
    const g = growthRate.toLowerCase();
    let cls, arrowCount, speed;
    if      (g.includes('fast') || g.includes('rapid'))
        { cls = 'growth-fast'; arrowCount = 3; speed = '0.7s'; }
    else if (g.includes('slow'))
        { cls = 'growth-slow'; arrowCount = 1; speed = '2.2s'; }
    else
        { cls = 'growth-mod';  arrowCount = 2; speed = '1.3s'; }

    const arrows = Array.from({ length: arrowCount }, (_, i) => {
        const delay = (i * parseFloat(speed) * 0.4).toFixed(2) + 's';
        return `<div class="growth-arrow" style="--arrow-delay:${delay};--arrow-speed:${speed};">&#9650;</div>`;
    }).join('');

    return `
        <div class="glass-widget">
            <div class="growth-scene ${cls}">
                <div class="growth-arrows">${arrows}</div>
            </div>
            <div class="widget-info">
                <span class="widget-label">GROWTH RATE</span>
                <span class="widget-value">${growthRate}</span>
            </div>
        </div>`;
}

// -- Chilling Hours Widget -----------------------------------------------------
// Clifton Park NY: ~52 chilling hours per week of winter.
export function generateChillingHoursWidgetHTML(hours) {
    if (!hours && hours !== 0) return '';
    const h = parseInt(hours, 10);
    if (isNaN(h) || h <= 0) return '';

    const HOURS_PER_WEEK = 52;
    const weeks   = Math.round(h / HOURS_PER_WEEK);
    const barPct  = Math.min(100, (h / 1200) * 100).toFixed(1);

    // Sub-week special case: h < 52 rounds to 0 weeks — show a single faint flake
    if (weeks === 0) {
        return `
        <div class="glass-widget">
            <div class="chill-scene">
                <div class="chill-flakes" style="opacity:0.4; justify-content:center;">
                    <span class="chill-flake" style="--flake-delay:0s;--flake-scale:0.65; font-size:1.1rem;">&#10052;</span>
                </div>
                <div class="chill-bar">
                    <div class="chill-bar-fill" style="width:${barPct}%; min-width:4px;"></div>
                </div>
            </div>
            <div class="widget-info">
                <span class="widget-label">CHILLING HOURS</span>
                <span class="widget-value">${h} hrs ≈ &lt;1 week in Clifton Park</span>
                <span class="le-stage-label" style="font-size:0.7rem; opacity:0.65;">Minimal chill required</span>
            </div>
        </div>`;
    }

    const displayWeeks = Math.min(weeks, 20);
    const isMore       = weeks > 20;

    const flakes = Array.from({ length: displayWeeks }, (_, i) => {
        const delay = ((i * 0.18) % 2.4).toFixed(2) + 's';
        const scale = 0.8 + (i % 4) * 0.1;
        return `<span class="chill-flake" style="--flake-delay:${delay};--flake-scale:${scale};">&#10052;</span>`;
    }).join('');
    const moreSpan  = isMore ? `<span class="chill-more">+</span>` : '';
    const weekLabel = weeks === 1 ? 'week' : 'weeks';

    return `
        <div class="glass-widget">
            <div class="chill-scene">
                <div class="chill-flakes">${flakes}${moreSpan}</div>
                <div class="chill-bar">
                    <div class="chill-bar-fill" style="width:${barPct}%"></div>
                </div>
            </div>
            <div class="widget-info">
                <span class="widget-label">CHILLING HOURS</span>
                <span class="widget-value">${h} hrs ≈ ${weeks} ${weekLabel} in Clifton Park</span>
            </div>
        </div>`;
}
// -- Life Expectancy Widget ----------------------------------------------------
// Maps plant lifespan text to a human life-stage emoji for quick visual context.
function _leCard(emoji, stage, rawText) {
    return `
        <div class="glass-widget le-widget">
            <div class="le-scene">
                <span class="le-emoji">${emoji}</span>
            </div>
            <div class="widget-info">
                <span class="widget-label">LIFE EXPECTANCY</span>
                <span class="widget-value">${rawText}</span>
                <span class="le-stage-label">${stage}</span>
            </div>
        </div>`;
}

export function generateLifeExpectancyWidgetHTML(lifeText) {
    if (!lifeText || lifeText === 'NULL' || lifeText === 'null' || lifeText === 'undefined') return '';
    const txt = String(lifeText).trim().toLowerCase();
    if (!txt || txt.length < 1) return '';

    // Perennial / indefinite / permanent
    if (txt.includes('perennial') || txt.includes('indefinit') || txt.includes('permanent') || txt.includes('forever')) {
        return _leCard('\u{1F333}', 'Perennial', lifeText);
    }

    // Parse years: handles "5", "5-10", "10+", "2-3 years", "5 to 10"
    let years = 0;
    const rangeMatch  = txt.match(/(\d+)\s*[-\u2013to]\s*(\d+)/);
    const plusMatch   = txt.match(/(\d+)\s*\+/);
    const singleMatch = txt.match(/(\d+)/);

    if (rangeMatch) {
        years = (parseInt(rangeMatch[1]) + parseInt(rangeMatch[2])) / 2;
    } else if (plusMatch) {
        years = parseInt(plusMatch[1]) * 1.4;
    } else if (singleMatch) {
        years = parseInt(singleMatch[1]);
    }

    if (years <= 0) return '';

    let emoji, stage;
    if      (years <= 1)  { emoji = '\u{1F331}'; stage = 'Annual';      } // seedling
    else if (years <= 3)  { emoji = '\u{1F476}'; stage = 'Short-Lived'; } // baby
    else if (years <= 7)  { emoji = '\u{1F9D2}'; stage = 'Young';       } // child
    else if (years <= 15) { emoji = '\u{1F9D1}'; stage = 'Established'; } // teen/adult
    else if (years <= 30) { emoji = '\u{1F468}'; stage = 'Mature';      } // adult
    else if (years <= 75) { emoji = '\u{1F9D3}'; stage = 'Long-Lived';  } // elder
    else                  { emoji = '\u{1F333}'; stage = 'Ancient';     } // tree

    return _leCard(emoji, stage, lifeText);
}

// -- Fruit SVG Widget ----------------------------------------------------------
// Returns an inline SVG illustration of a fruit, adapting shape/color/edibility.
// Intended for use as the svgHTML argument of renderMorphologyGroup.
export function generateFruitSVG(fruitType, fruitShape, fruitColor, fruitEdibility) {
    const colorMap = {
        red:'#E53935', crimson:'#C62828', scarlet:'#D32F2F',
        orange:'#FB8C00', yellow:'#FDD835', gold:'#F9A825',
        green:'#43A047', lime:'#8BC34A', teal:'#00897B',
        blue:'#1E88E5', purple:'#7B1FA2', violet:'#6A1B9A',
        pink:'#E91E63', magenta:'#D81B60', rose:'#EC407A',
        white:'#ECEFF1', cream:'#FFF9C4', ivory:'#FFFDE7',
        brown:'#8D6E63', tan:'#A1887F', beige:'#D7CCC8',
        black:'#212121', dark:'#37474F', gray:'#78909C', grey:'#78909C',
    };
    const rawColor = (fruitColor || '').toLowerCase().trim();
    let fill = '#A5D6A7';
    for (const [key, val] of Object.entries(colorMap)) {
        if (rawColor.includes(key)) { fill = val; break; }
    }

    const shape = ((fruitShape || '') + ' ' + (fruitType || '')).toLowerCase();
    let body;
    if (shape.includes('oval') || shape.includes('ellip')) {
        body = `<ellipse cx="28" cy="30" rx="12" ry="15" fill="${fill}" stroke="${fill}99" stroke-width="1.2"/>`;
    } else if (shape.includes('capsule') || shape.includes('pod')) {
        body = `<rect x="16" y="14" width="14" height="28" rx="7" fill="${fill}" stroke="${fill}99" stroke-width="1.2"/>`;
    } else if (shape.includes('aggregate') || shape.includes('berry') || shape.includes('cluster')) {
        body = `<circle cx="21" cy="33" r="7" fill="${fill}" stroke="${fill}88" stroke-width="1"/>
                <circle cx="33" cy="33" r="7" fill="${fill}" stroke="${fill}88" stroke-width="1"/>
                <circle cx="27" cy="24" r="7" fill="${fill}" stroke="${fill}88" stroke-width="1"/>`;
    } else if (shape.includes('cone') || shape.includes('conical')) {
        body = `<path d="M27 12 L42 44 Q27 48 12 44Z" fill="${fill}" stroke="${fill}99" stroke-width="1.2"/>`;
    } else if (shape.includes('drupe') || shape.includes('stone')) {
        body = `<circle cx="27" cy="30" r="14" fill="${fill}" stroke="${fill}99" stroke-width="1.2"/>
                <path d="M27 18 Q21 22 23 30" stroke="${fill}55" stroke-width="2" fill="none" stroke-linecap="round"/>`;
    } else if (shape.includes('samara') || shape.includes('wing')) {
        body = `<ellipse cx="27" cy="34" rx="6" ry="9" fill="${fill}"/>
                <path d="M27 25 Q38 10 42 20 Q36 24 27 25Z" fill="${fill}cc"/>`;
    } else {
        body = `<circle cx="27" cy="30" r="14" fill="${fill}" stroke="${fill}99" stroke-width="1.2"/>`;
    }

    const stemLeaf = `<path d="M27 16 Q27 10 27 8" stroke="#558B2F" stroke-width="2" stroke-linecap="round" fill="none"/>
        <path d="M27 10 Q33 6 36 9 Q32 13 27 11Z" fill="#66BB6A"/>`;

    const ed = (fruitEdibility || '').toLowerCase();
    let overlay = '';
    if (ed.includes('toxic') || ed.includes('poison')) {
        overlay = `<circle cx="46" cy="11" r="9" fill="#B71C1C" opacity="0.92"/>
            <text x="46" y="15.5" text-anchor="middle" font-size="11" fill="white">&#9760;</text>`;
    } else if (ed.includes('wildlife')) {
        overlay = `<circle cx="46" cy="11" r="9" fill="#1565C0" opacity="0.88"/>
            <text x="46" y="15.5" text-anchor="middle" font-size="11" fill="white">&#128062;</text>`;
    } else if (ed.includes('inedible') || ed.includes('not edible')) {
        overlay = `<circle cx="46" cy="11" r="9" fill="#E65100" opacity="0.88"/>
            <text x="46" y="15.5" text-anchor="middle" font-size="11" fill="white">&#8856;</text>`;
    } else if (ed.includes('edible')) {
        overlay = `<circle cx="46" cy="11" r="9" fill="#2E7D32" opacity="0.88"/>
            <text x="46" y="15.5" text-anchor="middle" font-size="11" fill="white">&#10003;</text>`;
    }

    return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 56 56" width="80" height="80"
        style="display:block;filter:drop-shadow(0 2px 4px rgba(0,0,0,0.18));">
        ${body}
        ${stemLeaf}
        ${overlay}
    </svg>`;
}

// -- Soil Type Widget ----------------------------------------------------------
// Scans the free-text soil description for keywords and emits one emoji badge
// per matched trait (loamy, sandy, clay, moist, well-draining, acidic, etc.).
export function generateSoilTypeWidgetHTML(soilText) {
    if (!soilText || soilText === 'NULL' || soilText === 'null' || String(soilText).trim() === '') return '';
    const t = soilText.toLowerCase();

    const TRAITS = [
        { keys: ['loam','loamy'],                        emoji: '\u{1F331}', label: 'Loamy'         },
        { keys: ['sandy','sand'],                         emoji: '\u{1F3D6}', label: 'Sandy'         },
        { keys: ['clay'],                                 emoji: '\u{1F9F1}', label: 'Clay'          },
        { keys: ['chalk','chalky'],                       emoji: '\u{25AB}',  label: 'Chalky'        },
        { keys: ['peat','peaty'],                         emoji: '\u{1F342}', label: 'Peaty'         },
        { keys: ['rocky','gravel','gravelly','poor soil'],emoji: '\u{1FAA8}', label: 'Rocky/Poor'    },
        { keys: ['moist'],                                emoji: '\u{1F4A7}', label: 'Moist'         },
        { keys: ['well-drain','well drain','drainage'],   emoji: '\u{1F4A7}', label: 'Well-Draining' },
        { keys: ['acidic','acid'],                        emoji: '🧪',        label: 'Acidic'        },
        { keys: ['alkaline'],                             emoji: '\u{1F52C}', label: 'Alkaline'      },
        { keys: ['organic','humus','humusy'],             emoji: '\u{1F343}', label: 'Organic-Rich'  },
        { keys: ['rich','fertile','fertility'],           emoji: '\u{1F49A}', label: 'Fertile'       },
        { keys: ['average'],                              emoji: '\u{1F33F}', label: 'Average'       },
        { keys: ['adapt','various','versatile'],          emoji: '\u{1F504}', label: 'Adaptable'     },
    ];

    const seen = new Set();
    const pills = [];
    for (const { keys, emoji, label } of TRAITS) {
        if (keys.some(k => t.includes(k)) && !seen.has(label)) {
            seen.add(label);
            pills.push(`<div class="glass-widget">
                <span style="font-size:2rem; line-height:1;">${emoji}</span>
                <div class="widget-info">
                    <span class="widget-label">SOIL TYPE</span>
                    <span class="widget-value">${label}</span>
                </div>
            </div>`);
        }
    }

    if (!pills.length) {
        return `<div class="glass-widget">
            <span style="font-size:2rem; line-height:1;">\u{1F30D}</span>
            <div class="widget-info">
                <span class="widget-label">SOIL TYPE</span>
                <span class="widget-value">${soilText}</span>
            </div>
        </div>`;
    }

    return pills.join('\n');
}

// -- Plant Type Widget ---------------------------------------------------------
export function generatePlantTypeHTML(plantType) {
    if (!plantType || String(plantType).trim() === '') return '';
    const TYPE_MAP = {
        'Perennial':  '🌸',
        'Annual':     '🌱',
        'Biennial':   '🌿',
        'Shrub':      '🪴',
        'Tree':       '🌳',
        'Succulent':  '🌵',
        'Vine':       '🍃',
        'Unknown':    '❓',
    };
    const emoji = TYPE_MAP[plantType] || '🌾';
    return `<div class="glass-widget">
        <span style="font-size:2rem; line-height:1;">${emoji}</span>
        <div class="widget-info">
            <span class="widget-label">PLANT TYPE</span>
            <span class="widget-value">${plantType}</span>
        </div>
    </div>`;
}

// -- Family Common Name --------------------------------------------------------
export function getFamilyDisplayName(family) {
    if (!family || String(family).trim() === '') return family;
    const FAMILY_COMMON = {
        'Acanthaceae':      'Acanthus 🌿',
        'Amaranthaceae':    'Amaranth 🌾',
        'Amaryllidaceae':   'Amaryllis 🧅',
        'Apiaceae':         'Carrot 🥕',
        'Apocynaceae':      'Dogbane 🌸',
        'Araceae':          'Arum 🪴',
        'Araliaceae':       'Ginseng 🌿',
        'Asparagaceae':     'Asparagus 🥦',
        'Asteraceae':       'Daisy 🌼',
        'Berberidaceae':    'Barberry 🫐',
        'Boraginaceae':     'Borage 💜',
        'Brassicaceae':     'Mustard 🥬',
        'Calycanthaceae':   'Sweetshrub 🌸',
        'Campanulaceae':    'Bellflower 🔔',
        'Caprifoliaceae':   'Honeysuckle 🍯',
        'Caryophyllaceae':  'Carnation 💐',
        'Celastraceae':     'Bittersweet 🍂',
        'Convolvulaceae':   'Morning Glory 🌅',
        'Cornaceae':        'Dogwood 🌸',
        'Crassulaceae':     'Stonecrop 🪨',
        'Cupressaceae':     'Cypress 🌲',
        'Cyperaceae':       'Sedge 🌾',
        'Ericaceae':        'Heath 🫐',
        'Fabaceae':         'Legume 🫘',
        'Geraniaceae':      'Geranium 🌸',
        'Hydrangeaceae':    'Hydrangea 💐',
        'Iridaceae':        'Iris 🌷',
        'Lamiaceae':        'Mint 🌿',
        'Liliaceae':        'Lily 🌸',
        'Malvaceae':        'Mallow 🌺',
        'Myrtaceae':        'Myrtle 🌿',
        'Oleaceae':         'Olive 🫒',
        'Orchidaceae':      'Orchid 🌸',
        'Oxalidaceae':      'Wood Sorrel 🍀',
        'Papaveraceae':     'Poppy 🌺',
        'Plantaginaceae':   'Plantain 🌿',
        'Poaceae':          'Grass 🌾',
        'Polemoniaceae':    'Phlox 🌸',
        'Primulaceae':      'Primrose 🌼',
        'Ranunculaceae':    'Buttercup 🌼',
        'Rosaceae':         'Rose 🌹',
        'Rubiaceae':        'Coffee ☕',
        'Rutaceae':         'Citrus 🍋',
        'Sapindaceae':      'Japanese Maple 🍁',
        'Saxifragaceae':    'Saxifrage 🪨',
        'Scrophulariaceae': 'Figwort 🌿',
        'Solanaceae':       'Nightshade 🍅',
        'Urticaceae':       'Nettle 🌿',
        'Verbenaceae':      'Verbena 💜',
        'Violaceae':        'Violet 💜',
        'Zingiberaceae':    'Ginger 🫚',
    };
    const common = FAMILY_COMMON[String(family).trim()];
    return common ? `${family} (${common})` : family;
}

// -- Planting Depth Widget -----------------------------------------------------
export function generatePlantingDepthHTML(depthText) {
    if (!depthText || String(depthText).trim() === '') return '';
    const lower = String(depthText).toLowerCase();
    let emoji = '🌱';
    if (lower.includes('surface') || lower.includes('light'))          emoji = '☀️';
    else if (lower.includes('crown') || lower.includes('soil level'))  emoji = '🌿';
    else if (lower.includes('bulb') || lower.includes('tuber'))        emoji = '🧅';
    else if (lower.includes('seed'))                                    emoji = '🫘';
    else if (lower.includes('deep') || lower.includes('6') || lower.includes('8') || lower.includes('12')) emoji = '⛏️';
    return `<div class="glass-widget">
        <span style="font-size:2rem; line-height:1;">${emoji}</span>
        <div class="widget-info">
            <span class="widget-label">PLANTING DEPTH</span>
            <span class="widget-value">${depthText}</span>
        </div>
    </div>`;
}

// -- Invasive Status Widget ----------------------------------------------------
export function generateInvasiveHTML(invasiveLocal) {
    if (!invasiveLocal || String(invasiveLocal).trim() === '') return '';
    const INVASIVE_MAP = {
        'No':        { emoji: '✅', label: 'NOT INVASIVE' },
        'Yes':       { emoji: '🚨', label: 'INVASIVE'     },
        'Watchlist': { emoji: '👁️',  label: 'WATCHLIST'   },
    };
    const key = String(invasiveLocal).trim();
    const entry = INVASIVE_MAP[key] || { emoji: '❓', label: key.toUpperCase() };
    return `<div class="glass-widget">
        <span style="font-size:2rem; line-height:1;">${entry.emoji}</span>
        <div class="widget-info">
            <span class="widget-label">INVASIVE</span>
            <span class="widget-value">${entry.label}</span>
        </div>
    </div>`;
}

// -- Native Status Widget ------------------------------------------------------
export function generateNativeStatusHTML(nativeStatus) {
    if (!nativeStatus || String(nativeStatus).trim() === '') return '';
    const NATIVE_MAP = {
        'Native':      { emoji: '🏡', label: 'NATIVE'      },
        'Non-native':  { emoji: '🌍', label: 'NON-NATIVE'  },
        'Introduced':  { emoji: '🚢', label: 'INTRODUCED'  },
        'Cultivar':    { emoji: '🧬', label: 'CULTIVAR'    },
        'Unknown':     { emoji: '❓', label: 'UNKNOWN'     },
    };
    const key = String(nativeStatus).trim();
    const entry = NATIVE_MAP[key] || { emoji: '🌐', label: key.toUpperCase() };
    return `<div class="glass-widget">
        <span style="font-size:2rem; line-height:1;">${entry.emoji}</span>
        <div class="widget-info">
            <span class="widget-label">NATIVE STATUS</span>
            <span class="widget-value">${entry.label}</span>
        </div>
    </div>`;
}
