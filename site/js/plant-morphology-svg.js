// js/plant-morphology-svg.js
// Generates inline SVG botanical illustrations for plant morphology data.
// Exported: generateLeafSVG(), generateFlowerSVG()

// ─────────────────────────────────────────────────────────────────────────────
// COLOR UTILITIES
// ─────────────────────────────────────────────────────────────────────────────

// ── Bloom (flower) color lookup ──────────────────────────────────────────────
const BLOOM_COLORS = [
    // Multi-word / compound (sorted longest-first by parseColor, listed here for readability)
    ['pale pink',       '#fce4ec'], ['light pink',    '#f8bbd9'], ['soft pink',    '#f8bbd9'],
    ['deep pink',       '#e91e63'], ['hot pink',      '#e91e63'], ['bright pink',  '#f06292'],
    ['pale yellow',     '#fff9c4'], ['light yellow',  '#fffde7'], ['deep yellow',  '#f9a825'],
    ['pale blue',       '#e3f2fd'], ['light blue',    '#90caf9'], ['deep blue',    '#1565c0'],
    ['pale purple',     '#ede7f6'], ['light purple',  '#ce93d8'], ['deep purple',  '#6a1b9a'],
    ['pale red',        '#ffcdd2'], ['deep red',      '#b71c1c'], ['dark red',     '#b71c1c'],
    ['pale orange',     '#ffe0b2'], ['pale green',    '#c8e6c9'], ['light green',  '#a5d6a7'],
    ['pale lavender',   '#ede7f6'], ['pale lilac',    '#e8d5f5'],
    ['pale white',      '#fafafa'], ['pure white',    '#ffffff'], ['creamy white', '#fffde7'],
    ['pale cream',      '#fffff0'], ['off white',     '#fafaf0'],
    ['gold yellow',     '#ffb300'], ['warm white',    '#fffff0'],
    ['rose pink',       '#f48fb1'], ['blush pink',    '#fce4ec'],
    // Single-word descriptors
    ['white',       '#f9f9f6'], ['cream',       '#fffde7'], ['ivory',      '#fffff0'],
    ['yellow',      '#fdd835'], ['gold',        '#ffb300'], ['butter',     '#fff9c4'],
    ['orange',      '#ef6c00'], ['apricot',     '#ffcc80'], ['peach',      '#ffccbc'],
    ['red',         '#c62828'], ['crimson',     '#b71c1c'], ['scarlet',    '#c62828'],
    ['burgundy',    '#880e4f'], ['maroon',      '#6d0019'], ['ruby',       '#c62828'],
    ['rose',        '#e91e63'], ['coral',       '#ff7043'], ['salmon',     '#ef9a9a'],
    ['pink',        '#f48fb1'],
    ['purple',      '#7b1fa2'], ['violet',      '#6a1b9a'], ['plum',       '#6a1b9a'],
    ['lavender',    '#b39ddb'], ['lilac',       '#ce93d8'], ['mauve',      '#ce93d8'],
    ['magenta',     '#ad1457'], ['fuchsia',     '#ad1457'],
    ['blue',        '#1565c0'], ['sky',         '#039be5'], ['navy',       '#0d47a1'],
    ['indigo',      '#1a237e'], ['periwinkle',  '#7986cb'],
    ['green',       '#388e3c'], ['teal',        '#00695c'],
    ['brown',       '#795548'], ['bronze',      '#6d4c41'], ['copper',     '#8d6e63'],
];

// ── Foliage color lookup ─────────────────────────────────────────────────────
const FOLIAGE_COLORS = [
    // Compound (multi-word) — checked before single-word thanks to length sort
    ['silver green',    '#9db5a0'], // muted sage-silver-green
    ['blue green',      '#00838f'], ['gray green',     '#78909c'],
    ['olive green',     '#689f38'], ['dark green',     '#1b5e20'],
    ['deep green',      '#1b5e20'], ['forest green',   '#2e7d32'],
    ['bright green',    '#43a047'], ['light green',    '#81c784'],
    ['pale green',      '#a5d6a7'], ['medium green',   '#388e3c'],
    ['yellow green',    '#aed581'], ['lime green',     '#c6ef4f'],
    ['blue gray',       '#78909c'], ['gray blue',      '#78909c'],
    ['gold yellow',     '#f9a825'], ['dark red',       '#b71c1c'],
    ['red purple',      '#880e4f'], ['blue purple',    '#5c35a0'],
    ['deep red',        '#b71c1c'], ['dark purple',    '#4a148c'],
    ['dark brown',      '#3e2723'], ['reddish brown',  '#6d4c41'],
    ['greenish yellow', '#c6ef4f'],
    // Single-word descriptors
    ['silver',      '#b0c4b1'], // silver w/ green tint — avoids returning plain grey
    ['gray',        '#9e9e9e'], ['grey',        '#9e9e9e'],
    ['yellow',      '#f9a825'], ['gold',        '#f57f17'],
    ['reddish',     '#b71c1c'], ['red',         '#c62828'],
    ['burgundy',    '#880e4f'], ['maroon',      '#6d0019'],
    ['purple',      '#6a1b9a'], ['violet',      '#6a1b9a'],
    ['bronze',      '#6d4c41'], ['copper',      '#8d6e63'], ['brown',     '#795548'],
    ['variegated',  '#66bb6a'], ['mottled',     '#66bb6a'], ['spotted',   '#66bb6a'],
    ['white',       '#f5f5f5'], ['cream',       '#fffde7'], ['ivory',     '#fffff0'],
    ['blue',        '#1565c0'], ['teal',        '#00695c'],
    ['green',       '#388e3c'],
];

/**
 * Normalize a color description string for matching.
 */
function normalizeColorStr(str) {
    return str.toLowerCase()
        .replace(/[-_]/g, ' ')
        .replace(/\bsilvery\b/g,   'silver')
        .replace(/\bgolden\b/g,    'gold')
        .replace(/\bgreenish\b/g,  'green')
        .replace(/\breddish\b/g,   'red')
        .replace(/\bbluish\b/g,    'blue')
        .replace(/\byellowish\b/g, 'yellow')
        .replace(/\bpurplish\b/g,  'purple')
        .replace(/\bgreyish\b/g,   'gray')
        .replace(/\bgrayish\b/g,   'gray')
        .replace(/\bpinkish\b/g,   'pink')
        .replace(/\bbrownish\b/g,  'brown')
        .replace(/\borangish\b/g,  'orange')
        .replace(/\bcreamy\b/g,    'cream')
        .replace(/\brosé\b/g,      'rose')
        .replace(/\bolivish\b/g,   'olive');
}

/**
 * Parse a plain-language color description into a hex value.
 *
 * Strategy: find ALL matching color keywords in the string and return the
 * one that appears EARLIEST (position wins). Ties broken by longest key
 * (more specific wins). This means "Slightly blue to dark purple with golden
 * yellow centers" returns blue (pos 9) rather than gold (pos ~35).
 */
function parseColor(str, map, def) {
    if (!str) return def;
    const s = normalizeColorStr(str);

    let bestVal = null;
    let bestPos = Infinity;
    let bestLen = 0;

    for (const [key, val] of map) {
        const pos = s.indexOf(key);
        if (pos < 0) continue;
        // Earlier position wins; tie-break by longer (more specific) key
        if (pos < bestPos || (pos === bestPos && key.length > bestLen)) {
            bestVal = val;
            bestPos = pos;
            bestLen = key.length;
        }
    }
    return bestVal !== null ? bestVal : def;
}

/**
 * Parse a secondary / center color from a bloom description.
 * Looks for color words near "center", "eye", "stamen", "throat", or after "with".
 * Falls back to golden yellow (botanically accurate — most stamens are yellow).
 */
function parseCenterColor(bloomStr) {
    if (!bloomStr) return '#ffd54f';
    const s = normalizeColorStr(bloomStr);

    // Check for color near anatomical center keywords
    const centerWords = ['center', 'centre', 'eye', 'stamen', 'pistil', 'throat', 'core', 'disk'];
    for (const kw of centerWords) {
        const idx = s.indexOf(kw);
        if (idx < 0) continue;
        const ctx = s.slice(Math.max(0, idx - 24), idx + 24);
        const c = parseColor(ctx, BLOOM_COLORS, null);
        if (c) return c;
    }

    // After "with …" secondary color clause
    const withIdx = s.search(/\bwith\b/);
    if (withIdx >= 0) {
        const afterWith = s.slice(withIdx + 5, withIdx + 50);
        const c = parseColor(afterWith, BLOOM_COLORS, null);
        if (c && c !== parseColor(s, BLOOM_COLORS, null)) return c;
    }

    return '#ffd54f'; // golden yellow — typical stamen/pistil color
}

/** Lighten (amt>0) or darken (amt<0) a hex color. */
function adj(hex, amt) {
    if (!hex || hex.length < 7) return hex || '#888';
    const n = parseInt(hex.slice(1), 16);
    const clamp = v => Math.max(0, Math.min(255, v));
    const r = clamp(((n >> 16) & 0xff) + amt);
    const g = clamp(((n >> 8)  & 0xff) + amt);
    const b = clamp(( n        & 0xff) + amt);
    return `#${[r, g, b].map(v => v.toString(16).padStart(2, '0')).join('')}`;
}

// Unique ID counter so gradient IDs never collide in the same document
let _uid = 0;
const uid = () => `msvg_${++_uid}`;


// ─────────────────────────────────────────────────────────────────────────────
// LEAF SVG HELPERS
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Return n [x,y] points for the RIGHT side of a simple leaf outline.
 * s=0 → tip (top), s=1 → base (bottom). Leaf is 80px tall, centered at x=60.
 */
function leafRightPoints(shape, n = 60) {
    const pts = [];
    const cx = 60, tipY = 18, H = 80;
    for (let i = 0; i <= n; i++) {
        const s = i / n;
        const y = tipY + s * H;
        let w;
        switch (shape) {
            case 'linear':
                w = 4.5;
                break;
            case 'lanceolate':
                // Widest ~30% from tip, tapers sharply at both ends
                w = 22 * Math.pow(s, 0.28) * Math.pow(1 - s, 0.38);
                break;
            case 'ovate':
                // Widest near base side (~60%)
                w = 32 * Math.pow(s, 0.38) * Math.pow(1 - s * 0.52, 1.25);
                break;
            case 'elliptic':
                // Widest at midpoint
                w = 30 * Math.sin(Math.PI * s);
                break;
            case 'deltoid':
                // Wide at base, linear taper to tip
                w = 36 * (1 - Math.pow(s * 0.9, 0.85));
                break;
            default:
                w = 26 * Math.sin(Math.PI * s);
        }
        pts.push([cx + w, y]);
    }
    return pts;
}

/**
 * Apply margin treatment to an array of [x,y] points.
 * Works for both sides — determines direction by whether x > 60.
 */
function applyMargin(pts, margin) {
    const m = (margin || '').toLowerCase();
    if (!m || m.includes('entire')) return pts;

    if (m.includes('lobed')) {
        // Deep sinusoidal waves into the leaf body
        return pts.map(([x, y], i) => {
            const s = i / pts.length;
            if (s < 0.06 || s > 0.94) return [x, y];
            const isRight = x > 60;
            const sign    = isRight ? 1 : -1;
            const radius  = Math.abs(x - 60);
            const wave    = Math.sin(s * Math.PI * 3.5) * 0.38 * Math.sin(Math.PI * s);
            return [60 + sign * Math.max(1, radius * (1 + wave)), y];
        });
    }

    // Serrated / Shallowly Serrate — outward-only sawtooth teeth
    const amp  = m.includes('shallowly') ? 1.8 : 3.5;
    const freq = m.includes('shallowly') ? 6   : 10;
    return pts.map(([x, y], i) => {
        const s = i / (pts.length - 1);
        if (s < 0.07 || s > 0.93) return [x, y];
        const taper = Math.sin(Math.PI * s);
        const tooth = Math.max(0, Math.sin(i * freq * 0.22)) * amp * taper;
        return x > 60 ? [x + tooth, y] : [x - tooth, y];
    });
}

/** Convert separate right / left point arrays into a closed SVG path string. */
function ptsToPath(rightPts, leftPts) {
    const f = ([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`;
    let d = `M ${f(rightPts[0])}`;
    for (let i = 1; i < rightPts.length; i++) d += ` L ${f(rightPts[i])}`;
    for (let i = leftPts.length - 1; i >= 0; i--) d += ` L ${f(leftPts[i])}`;
    return d + ' Z';
}

/** Draw midrib + lateral veins for a simple leaf. */
function simpleLeafVeins(shape, veinColor) {
    const cx = 60, tipY = 18, baseY = 98;
    let v = `<line x1="${cx}" y1="${tipY+2}" x2="${cx}" y2="${baseY-2}" stroke="${veinColor}" stroke-width="1.1" stroke-linecap="round" opacity="0.45"/>`;
    const numV = shape === 'linear' ? 0 : shape === 'lanceolate' ? 5 : 6;
    for (let i = 1; i <= numV; i++) {
        const s = i / (numV + 1);
        const y = tipY + s * (baseY - tipY);
        const reach = (shape === 'ovate' ? 22 : shape === 'elliptic' ? 20 : shape === 'deltoid' ? 28 * (1-s) : 17)
                      * Math.sin(Math.PI * s) * 0.9;
        v += `<line x1="${cx}" y1="${y}" x2="${cx + reach}" y2="${y + 4}" stroke="${veinColor}" stroke-width="0.75" stroke-linecap="round" opacity="0.35"/>`;
        v += `<line x1="${cx}" y1="${y}" x2="${cx - reach}" y2="${y + 4}" stroke="${veinColor}" stroke-width="0.75" stroke-linecap="round" opacity="0.35"/>`;
    }
    return v;
}


// ─────────────────────────────────────────────────────────────────────────────
// LEAF SVG — MAIN EXPORT
// ─────────────────────────────────────────────────────────────────────────────

export function generateLeafSVG(leafShape = '', leafType = '', leafMargin = '', foliageColor = '') {
    const shapeRaw = (leafShape  || '').toLowerCase();
    const typeRaw  = (leafType   || '').toLowerCase();
    const fill     = parseColor(foliageColor, FOLIAGE_COLORS, '#4caf50');
    const light    = adj(fill,  48);
    const dark     = adj(fill, -38);
    const vein     = adj(fill, -60);
    const gid      = uid();

    let inner = '';

    // ── PINNATELY COMPOUND ─────────────────────────────────────────────────
    if (typeRaw.includes('pinnately')) {
        const rx = 60, ry0 = 18, ryN = 98;
        inner = `<line x1="${rx}" y1="${ry0}" x2="${rx}" y2="${ryN}" stroke="${dark}" stroke-width="1.8" stroke-linecap="round"/>`;
        // Terminal leaflet
        inner += `<ellipse cx="${rx}" cy="${ry0 + 6}" rx="9" ry="13" fill="${fill}" stroke="${dark}" stroke-width="0.9" opacity="0.9"/>`;
        // 4 pairs of lateral leaflets
        [[0.22, 1], [0.38, -1], [0.56, 1], [0.72, -1]].forEach(([t, side]) => {
            const y   = ry0 + t * (ryN - ry0);
            const lx  = rx + side * 7;
            const rot = side * -28;
            inner += `<line x1="${rx}" y1="${y}" x2="${lx}" y2="${y}" stroke="${dark}" stroke-width="0.9" opacity="0.6"/>`;
            inner += `<ellipse cx="${lx + side * 14}" cy="${y}" rx="15" ry="7" fill="${fill}" stroke="${dark}" stroke-width="0.8" opacity="0.88" transform="rotate(${rot},${lx},${y})"/>`;
        });
        inner += `<line x1="${rx}" y1="${ryN}" x2="${rx}" y2="108" stroke="${dark}" stroke-width="1.5" stroke-linecap="round"/>`;

    // ── PALMATELY COMPOUND ─────────────────────────────────────────────────
    } else if (typeRaw.includes('palmately')) {
        const bx = 60, by = 88;
        const angles = [-62, -32, 0, 32, 62];
        inner = '';
        angles.forEach(deg => {
            const rad  = (deg - 90) * Math.PI / 180;
            const len  = 36;
            const tx   = bx + len * Math.cos(rad);
            const ty   = by + len * Math.sin(rad);
            const mx   = bx + len * 0.52 * Math.cos(rad);
            const my   = by + len * 0.52 * Math.sin(rad);
            const px   = Math.sin(rad) * 11;
            const py   = -Math.cos(rad) * 11;
            inner += `<path d="M ${bx},${by} C ${mx+px},${my+py} ${tx+px*0.3},${ty+py*0.3} ${tx},${ty} C ${tx-px*0.3},${ty-py*0.3} ${mx-px},${my-py} ${bx},${by}" fill="${fill}" stroke="${dark}" stroke-width="0.9" opacity="0.9"/>`;
            inner += `<line x1="${bx}" y1="${by}" x2="${tx}" y2="${ty}" stroke="${vein}" stroke-width="0.7" opacity="0.35"/>`;
        });
        inner += `<line x1="${bx}" y1="${by}" x2="${bx}" y2="106" stroke="${dark}" stroke-width="1.5" stroke-linecap="round"/>`;

    // ── PALMATE LEAF SHAPE (single hand-shaped blade) ───────────────────────
    } else if (shapeRaw.includes('palmate')) {
        const cx = 60, by = 90;
        inner = '';
        [-55, -27, 0, 27, 55].forEach(deg => {
            const rad = (deg - 90) * Math.PI / 180;
            const len = 38;
            const tx  = cx + len * Math.cos(rad);
            const ty  = by + len * Math.sin(rad);
            const mx  = cx + len * 0.6 * Math.cos(rad);
            const my  = by + len * 0.6 * Math.sin(rad);
            const px  = Math.sin(rad) * 13;
            const py  = -Math.cos(rad) * 13;
            inner += `<path d="M ${cx},${by} C ${mx+px},${my+py} ${tx+px*0.3},${ty+py*0.3} ${tx},${ty} C ${tx-px*0.3},${ty-py*0.3} ${mx-px},${my-py} ${cx},${by}" fill="${fill}" stroke="${dark}" stroke-width="1" opacity="0.9"/>`;
            inner += `<line x1="${cx}" y1="${by}" x2="${tx}" y2="${ty}" stroke="${vein}" stroke-width="0.7" opacity="0.35"/>`;
        });
        inner += `<line x1="${cx}" y1="${by}" x2="${cx}" y2="106" stroke="${dark}" stroke-width="1.5" stroke-linecap="round"/>`;

    // ── CORDATE (Heart-shaped) ─────────────────────────────────────────────
    } else if (shapeRaw.includes('cordate') || shapeRaw.includes('heart')) {
        const cx = 60;
        inner = `
            <defs>
                <linearGradient id="${gid}" x1="25%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="${light}"/>
                    <stop offset="100%" stop-color="${fill}"/>
                </linearGradient>
            </defs>
            <path d="M ${cx},22 C 88,12 104,46 84,74 Q 70,95 ${cx},100 Q 50,95 36,74 C 16,46 32,12 ${cx},22 Z"
                  fill="url(#${gid})" stroke="${dark}" stroke-width="1.2"/>
            <path d="M 52,96 Q 60,86 68,96" fill="none" stroke="${dark}" stroke-width="0.9" opacity="0.45"/>
            ${simpleLeafVeins('cordate', vein)}
            <line x1="${cx}" y1="100" x2="${cx}" y2="108" stroke="${dark}" stroke-width="1.5" stroke-linecap="round"/>`;

    // ── SIMPLE LEAVES: linear, lanceolate, ovate, elliptic, deltoid ─────────
    } else {
        const known = ['linear', 'lanceolate', 'ovate', 'elliptic', 'deltoid'];
        const key   = known.find(k => shapeRaw.includes(k)) || 'elliptic';

        let rPts = leafRightPoints(key, 55);
        let lPts = rPts.map(([x, y]) => [120 - x, y]); // mirror about x=60
        rPts = applyMargin(rPts, leafMargin);
        lPts = applyMargin(lPts, leafMargin);

        const pathD = ptsToPath(rPts, lPts);
        inner = `
            <defs>
                <linearGradient id="${gid}" x1="28%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="${light}"/>
                    <stop offset="100%" stop-color="${fill}"/>
                </linearGradient>
            </defs>
            <path d="${pathD}" fill="url(#${gid})" stroke="${dark}" stroke-width="1.2"/>
            ${simpleLeafVeins(key, vein)}
            <line x1="60" y1="97" x2="60" y2="108" stroke="${dark}" stroke-width="1.5" stroke-linecap="round"/>`;
    }

    return `<svg viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">${inner}</svg>`;
}


// ─────────────────────────────────────────────────────────────────────────────
// FLOWER SVG HELPERS
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Render n copies of a petal shape rotated evenly around (cx, cy).
 * petalFn(cx, cy) → SVG path d-string for one petal pointing UP.
 */
function petals(n, cx, cy, petalFn, fill, stroke, opacity = 1) {
    let out = '';
    for (let i = 0; i < n; i++) {
        const angle = (360 / n) * i;
        out += `<path d="${petalFn(cx, cy)}" fill="${fill}" stroke="${stroke}" stroke-width="0.9"
                      fill-opacity="${opacity}" transform="rotate(${angle},${cx},${cy})"/>`;
    }
    return out;
}

/**
 * Smooth rounded petal pointing UP from (cx,cy).
 * r = radius to tip, w = width factor (0.15 narrow … 0.60 wide).
 */
function rPetal(cx, cy, r, w = 0.38) {
    const tip = cy - r;
    const c   = w * r;
    return `M ${cx},${cy} C ${cx+c},${cy-r*0.33} ${cx+c*0.65},${tip+r*0.22} ${cx},${tip} C ${cx-c*0.65},${tip+r*0.22} ${cx-c},${cy-r*0.33} ${cx},${cy}`;
}


// ─────────────────────────────────────────────────────────────────────────────
// FLOWER SVG — MAIN EXPORT
// ─────────────────────────────────────────────────────────────────────────────

export function generateFlowerSVG(flowerShape = '', flowerSize = '', flowerSymmetry = '', bloomColor = '') {
    const color  = parseColor(bloomColor,  BLOOM_COLORS,  '#e91e63');
    const light  = adj(color,  55);
    const dark   = adj(color, -45);
    const center = parseCenterColor(bloomColor);  // dynamic — from bloom description
    const cx = 60, cy = 60;
    const shape = (flowerShape    || '').toLowerCase();
    const sym   = (flowerSymmetry || '').toLowerCase();
    const gid   = uid();

    // ── Size scale factor ──────────────────────────────────────────────────
    const sizeL = (flowerSize || '').toLowerCase();
    const sz = sizeL.includes('large') || sizeL.includes('4+')  ? 1.28
             : sizeL.includes('medium')                          ? 1.08
             : sizeL.includes('tiny')  || sizeL.includes('<1')  ? 0.75
             : 1.0; // small / default

    let inner = '';

    // ── CAMPANULATE (Bell-shaped) ──────────────────────────────────────────
    // Fused-corolla cup viewed face-on: 5 scalloped lobes, dark-center gradient
    // shows throat depth — visually opposite of flat rotate/wheel flowers.
    if (shape.includes('campanulate') || shape.includes('bell')) {
        const campLR = 40 * sz;   // lobe tip radius
        const campVR = 16 * sz;   // valley (sinus) radius — deep scallop gives bell look
        const campLA = [0,72,144,216,288].map(a => (a - 90) * Math.PI / 180);
        const campVA = [36,108,180,252,324].map(a => (a - 90) * Math.PI / 180);
        let campPath = `M ${(cx + campLR*Math.cos(campLA[0])).toFixed(1)},${(cy + campLR*Math.sin(campLA[0])).toFixed(1)}`;
        for (let ci = 0; ci < 5; ci++) {
            campPath += ` Q ${(cx + campVR*Math.cos(campVA[ci])).toFixed(1)},${(cy + campVR*Math.sin(campVA[ci])).toFixed(1)}`
                      + ` ${(cx + campLR*Math.cos(campLA[(ci+1)%5])).toFixed(1)},${(cy + campLR*Math.sin(campLA[(ci+1)%5])).toFixed(1)}`;
        }
        campPath += ' Z';
        inner = `
            <defs><radialGradient id="${gid}" cx="50%" cy="50%" r="78%">
                <stop offset="0%"   stop-color="${dark}"/>
                <stop offset="42%"  stop-color="${color}"/>
                <stop offset="100%" stop-color="${light}"/>
            </radialGradient></defs>
            <path d="${campPath}" fill="url(#${gid})" stroke="${dark}" stroke-width="1.3"/>
            <circle cx="${cx}" cy="${cy}" r="${Math.round(19*sz)}" fill="${dark}" opacity="0.28"/>
            <circle cx="${cx}" cy="${cy}" r="${Math.round(7*sz)}"  fill="${center}" stroke="${adj(center,-25)}" stroke-width="0.9"/>
            <circle cx="${cx}" cy="${cy}" r="${Math.round(3.5*sz)}" fill="${adj(center,-35)}" opacity="0.75"/>`;

    // ── STELLATE (Star-shaped) ─────────────────────────────────────────────
    } else if (shape.includes('stellate') || shape.includes('star')) {
        inner = `
            ${petals(6, cx, cy, (cx,cy) => rPetal(cx, cy, 42*sz, 0.17), color, dark, 0.93)}
            <circle cx="${cx}" cy="${cy}" r="${9*sz}"   fill="${center}" stroke="${dark}" stroke-width="1.1"/>
            <circle cx="${cx}" cy="${cy}" r="${4.5*sz}" fill="${dark}"   opacity="0.4"/>`;

    // ── ROTATE (Flat wheel-shaped) ─────────────────────────────────────────
    } else if (shape.includes('rotate') || shape.includes('wheel')) {
        inner = `
            <defs><radialGradient id="${gid}" cx="50%" cy="60%" r="72%">
                <stop offset="0%" stop-color="${light}"/>
                <stop offset="100%" stop-color="${color}"/>
            </radialGradient></defs>
            ${petals(6, cx, cy, (cx,cy) => rPetal(cx, cy, 43*sz, 0.44), `url(#${gid})`, dark, 0.93)}
            <circle cx="${cx}" cy="${cy}" r="${10*sz}" fill="${center}" stroke="${dark}" stroke-width="1.4"/>
            <circle cx="${cx}" cy="${cy}" r="${5*sz}"  fill="${dark}"   opacity="0.4"/>`;

    // ── CRUCIATE (Cross-shaped) ────────────────────────────────────────────
    } else if (shape.includes('cruciate') || shape.includes('cross')) {
        inner = `
            ${petals(4, cx, cy, (cx,cy) => rPetal(cx, cy, 38*sz, 0.38), color, dark, 0.93)}
            <circle cx="${cx}" cy="${cy}" r="${8*sz}" fill="${center}" stroke="${dark}" stroke-width="1"/>
            <circle cx="${cx}" cy="${cy}" r="${4*sz}" fill="${dark}"   opacity="0.4"/>`;

    // ── TUBULAR ────────────────────────────────────────────────────────────
    } else if (shape.includes('tubular') || shape.includes('cylindrical')) {
        // Compact centered tube: opening at top, stem below
        const tw   = Math.round(8 * sz);   // half-width of tube
        const tTop = Math.round(cy - 16);  // tube opening  y≈44
        const tBot = Math.round(cy + 18);  // tube base     y≈78
        const pr   = Math.round(13 * sz);  // petal radius at opening
        inner = `
            <defs><linearGradient id="${gid}" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%"   stop-color="${dark}"/>
                <stop offset="44%"  stop-color="${light}"/>
                <stop offset="100%" stop-color="${dark}"/>
            </linearGradient></defs>
            <line x1="${cx}" y1="${tBot}" x2="${cx}" y2="${tBot+18}"
                  stroke="#5a8a3c" stroke-width="2.5" stroke-linecap="round"/>
            <path d="M ${cx-tw},${tBot}
                     C ${cx-tw-1},${tBot-10} ${cx-tw},${tTop+12} ${cx-tw+1},${tTop}
                     Q ${cx},${tTop-7} ${cx+tw-1},${tTop}
                     C ${cx+tw},${tTop+12} ${cx+tw+1},${tBot-10} ${cx+tw},${tBot}
                     Q ${cx},${tBot+7} ${cx-tw},${tBot} Z"
                  fill="url(#${gid})" stroke="${dark}" stroke-width="1.3"/>
            ${petals(5, cx, tTop, (cx,cy) => rPetal(cx, cy, pr, 0.46), light, dark, 0.85)}
            <circle cx="${cx}" cy="${tTop}" r="${Math.round(4.5*sz)}"
                    fill="${center}" stroke="${dark}" stroke-width="0.9"/>`;

    // ── INFUNDIBULIFORM (Funnel-shaped) ────────────────────────────────────
    } else if (shape.includes('infundibuliform') || shape.includes('funnel')) {
        inner = `
            <defs><linearGradient id="${gid}" x1="0%" y1="100%" x2="0%" y2="0%">
                <stop offset="0%"   stop-color="${dark}"/>
                <stop offset="100%" stop-color="${light}"/>
            </linearGradient></defs>
            <path d="M 53,88 C 50,78 46,62 43,44 Q 60,28 77,44 C 74,62 70,78 67,88 Q 60,94 53,88 Z"
                  fill="url(#${gid})" stroke="${dark}" stroke-width="1.4"/>
            ${petals(5, cx, 43, (cx,cy) => rPetal(cx, cy, 20, 0.48), color, dark, 0.72)}
            <circle cx="${cx}" cy="43" r="5.5" fill="${center}" stroke="${dark}" stroke-width="0.9"/>`;

    // ── SALVERFORM ─────────────────────────────────────────────────────────
    } else if (shape.includes('salverform')) {
        inner = `
            <rect x="54.5" y="60" width="11" height="28" rx="5.5" fill="${dark}" stroke="${dark}" stroke-width="0.5"/>
            ${petals(5, cx, 55, (cx,cy) => rPetal(cx, cy, 36, 0.44), color, dark, 0.93)}
            <circle cx="${cx}" cy="55" r="9"   fill="${center}" stroke="${dark}" stroke-width="1.4"/>
            <circle cx="${cx}" cy="55" r="4.5" fill="${dark}"   opacity="0.4"/>`;

    // ── BILABIATE (Two-lipped) ─────────────────────────────────────────────
    } else if (shape.includes('bilabiate') || shape.includes('two-lipped')) {
        inner = `
            <path d="M ${cx},${cy} C ${cx+10},${cy-6} ${cx+24},${cy-22} ${cx+16},${cy-43}
                                   C ${cx+6},${cy-31} ${cx},${cy-27}     ${cx},${cy}"
                  fill="${color}" stroke="${dark}" stroke-width="1.1" opacity="0.93"/>
            <path d="M ${cx},${cy} C ${cx-10},${cy-6} ${cx-24},${cy-22} ${cx-16},${cy-43}
                                   C ${cx-6},${cy-31} ${cx},${cy-27}     ${cx},${cy}"
                  fill="${color}" stroke="${dark}" stroke-width="1.1" opacity="0.93"/>
            <path d="M ${cx},${cy+4} C ${cx+18},${cy+8} ${cx+34},${cy+27} ${cx+18},${cy+44}
                                      Q ${cx+8},${cy+35} ${cx},${cy+4}"
                  fill="${light}" stroke="${dark}" stroke-width="1.1" opacity="0.88"/>
            <path d="M ${cx},${cy+4} C ${cx},${cy+16} ${cx},${cy+32} ${cx},${cy+48}
                                      Q ${cx},${cy+32} ${cx},${cy+4}"
                  fill="${color}" stroke="${dark}" stroke-width="0.9" opacity="0.82"/>
            <path d="M ${cx},${cy+4} C ${cx-18},${cy+8} ${cx-34},${cy+27} ${cx-18},${cy+44}
                                      Q ${cx-8},${cy+35} ${cx},${cy+4}"
                  fill="${light}" stroke="${dark}" stroke-width="1.1" opacity="0.88"/>
            <ellipse cx="${cx}" cy="${cy+4}" rx="6" ry="4" fill="${center}" stroke="${dark}" stroke-width="0.8"/>`;

    // ── URCEOLATE (Urn-shaped) ─────────────────────────────────────────────
    } else if (shape.includes('urceolate') || shape.includes('urn')) {
        inner = `
            <defs><linearGradient id="${gid}" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%"   stop-color="${dark}"/>
                <stop offset="48%"  stop-color="${light}"/>
                <stop offset="100%" stop-color="${dark}"/>
            </linearGradient></defs>
            <path d="M 50,76 C 38,68 36,52 40,40 C 44,30 52,26 60,26 C 68,26 76,30 80,40
                              C 84,52 82,68 70,76 Q 65,88 60,88 Q 55,88 50,76 Z"
                  fill="url(#${gid})" stroke="${dark}" stroke-width="1.4"/>
            <ellipse cx="${cx}" cy="28" rx="10" ry="4" fill="${color}" stroke="${dark}" stroke-width="1"/>
            ${[0,72,144,216,288].map(a =>
                `<path d="M ${cx},26 L ${(cx + 9*Math.sin(a*Math.PI/180)).toFixed(1)},${(22 - 7*Math.cos(a*Math.PI/180)).toFixed(1)}"
                       stroke="${dark}" stroke-width="1.4" fill="none" stroke-linecap="round"/>`
            ).join('')}`;

    // ── TRUMPET / CUP-SHAPED ───────────────────────────────────────────────
    } else if (shape.includes('trumpet') || shape.includes('cup')) {
        const t = cy + 10;
        inner = `
            <defs><linearGradient id="${gid}" x1="0%" y1="100%" x2="0%" y2="0%">
                <stop offset="0%"   stop-color="${dark}"/>
                <stop offset="100%" stop-color="${light}"/>
            </linearGradient></defs>
            ${petals(6, cx, t, (cx,cy) => rPetal(cx, cy, 36, 0.38), adj(color, 30), dark, 0.68)}
            <path d="M 43,${t} C 41,${t-16} 45,${t-31} ${cx},${t-37}
                              C ${2*cx-45},${t-31} ${2*cx-43},${t-16} ${2*cx-43},${t}
                              Q ${cx},${t+12} 43,${t} Z"
                  fill="url(#${gid})" stroke="${dark}" stroke-width="1.4"/>
            <ellipse cx="${cx}" cy="${t}" rx="17" ry="6" fill="none" stroke="${light}" stroke-width="1.5"/>`;

    // ── HIGH-CENTERED (Rose-like) ──────────────────────────────────────────
    } else if (shape.includes('high-centered') || shape.includes('high centered')) {
        inner = `
            ${petals(5, cx, cy, (cx,cy) => rPetal(cx, cy, 44, 0.52), light,            dark, 0.55)}
            ${petals(5, cx, cy, (cx,cy) => rPetal(cx, cy, 34, 0.48), color,            dark, 0.74)}
            ${petals(5, cx, cy, (cx,cy) => rPetal(cx, cy, 23, 0.42), adj(color,-14),   dark, 0.88)}
            <circle cx="${cx}" cy="${cy}" r="5" fill="${adj(color,-26)}"/>`;

    // ── DOUBLE BLOSSOMS ────────────────────────────────────────────────────
    } else if (shape.includes('double')) {
        inner = `
            ${petals(8, cx, cy, (cx,cy) => rPetal(cx, cy, 43, 0.46), light,          dark, 0.52)}
            ${petals(7, cx, cy, (cx,cy) => rPetal(cx, cy, 34, 0.43), color,          dark, 0.68)}
            ${petals(6, cx, cy, (cx,cy) => rPetal(cx, cy, 26, 0.40), adj(color,-12), dark, 0.82)}
            ${petals(5, cx, cy, (cx,cy) => rPetal(cx, cy, 18, 0.36), adj(color,-22), dark, 0.93)}
            <circle cx="${cx}" cy="${cy}" r="4" fill="${center}"/>`;

    // ── DEFAULT: 5-petal classic ──────────────────────────────────────────
    } else {
        inner = `
            <defs><radialGradient id="${gid}" cx="50%" cy="60%" r="72%">
                <stop offset="0%" stop-color="${light}"/>
                <stop offset="100%" stop-color="${color}"/>
            </radialGradient></defs>
            ${petals(5, cx, cy, (cx,cy) => rPetal(cx, cy, 42*sz, 0.44), `url(#${gid})`, dark, 0.93)}
            <circle cx="${cx}" cy="${cy}" r="${10*sz}" fill="${center}" stroke="${dark}" stroke-width="1.4"/>
            <circle cx="${cx}" cy="${cy}" r="${5*sz}"  fill="${dark}"   opacity="0.4"/>`;
    }

    // Zygomorphic axis indicator (subtle dashed line)
    const symLine = sym.includes('zygomorphic')
        ? `<line x1="${cx}" y1="8" x2="${cx}" y2="112" stroke="rgba(0,0,0,0.11)"
                 stroke-width="1" stroke-dasharray="3,3"/>`
        : '';

    return `<svg viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg" style="overflow:visible;">${symLine}${inner}</svg>`;
}
