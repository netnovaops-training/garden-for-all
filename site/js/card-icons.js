// js/card-icons.js
// ─────────────────────────────────────────────────────────────────────────────
// Animated SVG icons for index.php Garden Chronicles + blog cards.
// Each icon is a tiny inline SVG with a CSS animation class.
// ─────────────────────────────────────────────────────────────────────────────

const S = (vb, w, h, cls, inner) =>
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${vb}" width="${w}" height="${h}" class="card-svg ${cls}" aria-hidden="true">${inner}</svg>`;

// ══════════════════════════════════════════════════════════════════════════════
// ICON LIBRARY — each returns an inline SVG string
// ══════════════════════════════════════════════════════════════════════════════

const ICONS = {

// ── Nature / Plants ──────────────────────────────────────────────────────────
'🌿': () => S('0 0 36 36', 28, 28, 'svg-sway', `
    <path d="M18 34 C18 34 18 18 18 18" stroke="#4CAF50" stroke-width="2.5" fill="none" stroke-linecap="round"/>
    <path d="M18 18 Q10 10 6 4 Q14 8 18 18" fill="#66BB6A"/>
    <path d="M18 22 Q26 16 30 8 Q22 14 18 22" fill="#43A047"/>`),

'🌱': () => S('0 0 32 32', 24, 24, 'svg-grow', `
    <path d="M16 28 L16 18" stroke="#795548" stroke-width="2.5" fill="none" stroke-linecap="round"/>
    <path d="M16 18 Q8 12 4 6 Q12 10 16 18" fill="#66BB6A"/>
    <path d="M16 20 Q22 14 26 8 Q20 14 16 20" fill="#81C784"/>`),

'🌸': () => S('0 0 36 36', 26, 26, 'svg-pulse', `
    <circle cx="18" cy="18" r="4" fill="#FFD54F"/>
    <ellipse cx="18" cy="8" rx="4" ry="6" fill="#F48FB1" opacity="0.9"/>
    <ellipse cx="18" cy="28" rx="4" ry="6" fill="#F48FB1" opacity="0.9"/>
    <ellipse cx="8" cy="18" rx="6" ry="4" fill="#F48FB1" opacity="0.9"/>
    <ellipse cx="28" cy="18" rx="6" ry="4" fill="#F48FB1" opacity="0.9"/>
    <ellipse cx="11" cy="11" rx="4" ry="6" fill="#F48FB1" opacity="0.8" transform="rotate(45 11 11)"/>
    <ellipse cx="25" cy="25" rx="4" ry="6" fill="#F48FB1" opacity="0.8" transform="rotate(45 25 25)"/>
    <ellipse cx="25" cy="11" rx="4" ry="6" fill="#F48FB1" opacity="0.8" transform="rotate(-45 25 11)"/>
    <ellipse cx="11" cy="25" rx="4" ry="6" fill="#F48FB1" opacity="0.8" transform="rotate(-45 11 25)"/>`),

'🌻': () => S('0 0 36 36', 26, 26, 'svg-spin-slow', `
    <circle cx="18" cy="18" r="5" fill="#795548"/>
    <g fill="#FDD835">
      <ellipse cx="18" cy="6" rx="3" ry="6"/><ellipse cx="18" cy="30" rx="3" ry="6"/>
      <ellipse cx="6" cy="18" rx="6" ry="3"/><ellipse cx="30" cy="18" rx="6" ry="3"/>
      <ellipse cx="9.5" cy="9.5" rx="3" ry="6" transform="rotate(45 9.5 9.5)"/>
      <ellipse cx="26.5" cy="26.5" rx="3" ry="6" transform="rotate(45 26.5 26.5)"/>
      <ellipse cx="26.5" cy="9.5" rx="3" ry="6" transform="rotate(-45 26.5 9.5)"/>
      <ellipse cx="9.5" cy="26.5" rx="3" ry="6" transform="rotate(-45 9.5 26.5)"/>
    </g>`),

'🌼': () => S('0 0 32 32', 24, 24, 'svg-pulse', `
    <circle cx="16" cy="16" r="4" fill="#FFC107"/>
    <g fill="#FFF9C4" opacity="0.9">
      <ellipse cx="16" cy="6" rx="3.5" ry="5.5"/><ellipse cx="16" cy="26" rx="3.5" ry="5.5"/>
      <ellipse cx="6" cy="16" rx="5.5" ry="3.5"/><ellipse cx="26" cy="16" rx="5.5" ry="3.5"/>
    </g>`),

// ── Animals ──────────────────────────────────────────────────────────────────
'🐕': () => S('0 0 40 36', 30, 27, 'svg-sway', `
    <ellipse cx="22" cy="24" rx="10" ry="7" fill="#C8A06E"/>
    <circle cx="32" cy="16" r="6" fill="#C8A06E"/>
    <ellipse cx="34" cy="12" rx="2.5" ry="4" fill="#A67C52" transform="rotate(15 34 12)"/>
    <ellipse cx="28" cy="12" rx="2.5" ry="4" fill="#A67C52" transform="rotate(-15 28 12)"/>
    <circle cx="34" cy="15" r="1.2" fill="#333"/>
    <circle cx="34.4" cy="14.6" r="0.4" fill="#fff"/>
    <ellipse cx="36" cy="18" rx="1.5" ry="1" fill="#5D4037"/>
    <path d="M8 22 Q3 28 6 32" stroke="#C8A06E" stroke-width="3" fill="none" stroke-linecap="round"/>
    <ellipse cx="16" cy="31" rx="2" ry="1.5" fill="#A67C52"/>
    <ellipse cx="22" cy="31" rx="2" ry="1.5" fill="#A67C52"/>
    <ellipse cx="28" cy="30" rx="2" ry="1.5" fill="#A67C52"/>`),

'🐾': () => S('0 0 32 32', 22, 22, 'svg-bounce', `
    <ellipse cx="16" cy="22" rx="6" ry="5" fill="#8D6E63"/>
    <circle cx="9" cy="13" r="3.5" fill="#8D6E63"/>
    <circle cx="23" cy="13" r="3.5" fill="#8D6E63"/>
    <circle cx="6" cy="20" r="3" fill="#8D6E63"/>
    <circle cx="26" cy="20" r="3" fill="#8D6E63"/>`),

'🐝': () => S('0 0 36 36', 24, 24, 'svg-buzz', `
    <ellipse cx="18" cy="22" rx="6" ry="5" fill="#FDD835"/>
    <rect x="14" y="19" width="8" height="2" rx="1" fill="#333"/>
    <rect x="14" y="23" width="8" height="2" rx="1" fill="#333"/>
    <circle cx="18" cy="15" r="4" fill="#333"/>
    <circle cx="16" cy="14" r="1" fill="#fff"/><circle cx="20" cy="14" r="1" fill="#fff"/>
    <path d="M12 14 Q8 6 14 10" stroke="#B3E5FC" stroke-width="1.5" fill="rgba(179,229,252,0.3)"/>
    <path d="M24 14 Q28 6 22 10" stroke="#B3E5FC" stroke-width="1.5" fill="rgba(179,229,252,0.3)"/>
    <path d="M18 27 L18 30" stroke="#333" stroke-width="1" stroke-linecap="round"/>`),

'🦋': () => S('0 0 36 36', 24, 24, 'svg-flutter', `
    <path d="M18 10 L18 28" stroke="#5D4037" stroke-width="1.5" stroke-linecap="round"/>
    <path d="M18 16 Q6 6 4 16 Q6 24 18 20" fill="#CE93D8" opacity="0.9"/>
    <path d="M18 16 Q30 6 32 16 Q30 24 18 20" fill="#BA68C8" opacity="0.9"/>
    <path d="M18 20 Q10 22 8 28 Q12 26 18 24" fill="#E1BEE7" opacity="0.8"/>
    <path d="M18 20 Q26 22 28 28 Q24 26 18 24" fill="#CE93D8" opacity="0.8"/>
    <circle cx="18" cy="10" r="2" fill="#5D4037"/>
    <path d="M16 8 Q14 4 12 3" stroke="#5D4037" stroke-width="1" fill="none"/>
    <path d="M20 8 Q22 4 24 3" stroke="#5D4037" stroke-width="1" fill="none"/>`),

'🐑': () => S('0 0 36 32', 26, 24, 'svg-sway', `
    <circle cx="12" cy="16" r="5" fill="#E0E0E0"/>
    <circle cx="20" cy="14" r="5.5" fill="#EEEEEE"/>
    <circle cx="16" cy="11" r="5" fill="#F5F5F5"/>
    <circle cx="24" cy="18" r="4.5" fill="#E0E0E0"/>
    <circle cx="26" cy="12" r="5" fill="#EEEEEE"/>
    <circle cx="30" cy="10" r="4" fill="#BDBDBD"/>
    <circle cx="32" cy="8" r="1.2" fill="#333"/>
    <ellipse cx="14" cy="28" rx="1.5" ry="3" fill="#333"/>
    <ellipse cx="22" cy="28" rx="1.5" ry="3" fill="#333"/>`),

'🐩': () => S('0 0 36 36', 26, 26, 'svg-bounce', `
    <ellipse cx="18" cy="22" rx="7" ry="5" fill="#E0E0E0"/>
    <circle cx="18" cy="22" r="5" fill="#F5F5F5"/>
    <circle cx="26" cy="14" r="5" fill="#F5F5F5"/>
    <circle cx="26" cy="14" r="3.5" fill="#E0E0E0"/>
    <circle cx="28" cy="12" r="1" fill="#333"/>
    <ellipse cx="30" cy="15" rx="1.5" ry="1" fill="#E91E63"/>
    <path d="M10 22 Q6 28 8 32" stroke="#9E9E9E" stroke-width="2" fill="none"/>
    <ellipse cx="18" cy="28" rx="1.5" ry="2.5" fill="#9E9E9E"/>`),

// ── Garden / Home ────────────────────────────────────────────────────────────
'🏡': () => S('0 0 36 36', 26, 26, 'svg-pulse', `
    <path d="M18 6 L4 18 H8 V30 H28 V18 H32 Z" fill="#A5D6A7" stroke="#2E7D32" stroke-width="1.5"/>
    <rect x="14" y="20" width="8" height="10" rx="1" fill="#8D6E63"/>
    <circle cx="20" cy="26" r="0.8" fill="#FFD54F"/>
    <rect x="9" y="14" width="5" height="5" rx="0.5" fill="#BBDEFB" stroke="#1565C0" stroke-width="0.5"/>
    <rect x="22" y="14" width="5" height="5" rx="0.5" fill="#BBDEFB" stroke="#1565C0" stroke-width="0.5"/>
    <path d="M24 6 L24 10 H28 L28 12" stroke="#8D6E63" stroke-width="1.5" fill="none"/>`),

'🏘️': () => S('0 0 40 36', 28, 26, 'svg-pulse', `
    <path d="M10 12 L2 20 H5 V30 H15 V20 H18 Z" fill="#C8E6C9" stroke="#388E3C" stroke-width="1"/>
    <path d="M22 8 L14 18 H17 V30 H27 V18 H30 Z" fill="#A5D6A7" stroke="#2E7D32" stroke-width="1"/>
    <path d="M34 14 L28 22 H30 V30 H38 V22 H40 Z" fill="#81C784" stroke="#1B5E20" stroke-width="1"/>
    <rect x="7" y="22" width="4" height="5" fill="#8D6E63"/>`),

// ── Weather / Seasons ────────────────────────────────────────────────────────
'☀️': () => S('0 0 36 36', 24, 24, 'svg-spin-slow', `
    <circle cx="18" cy="18" r="7" fill="#FDD835"/>
    <g stroke="#FFA000" stroke-width="2.5" stroke-linecap="round">
      <line x1="18" y1="2" x2="18" y2="7"/><line x1="18" y1="29" x2="18" y2="34"/>
      <line x1="2" y1="18" x2="7" y2="18"/><line x1="29" y1="18" x2="34" y2="18"/>
      <line x1="6.7" y1="6.7" x2="10.2" y2="10.2"/><line x1="25.8" y1="25.8" x2="29.3" y2="29.3"/>
      <line x1="29.3" y1="6.7" x2="25.8" y2="10.2"/><line x1="10.2" y1="25.8" x2="6.7" y2="29.3"/>
    </g>`),

'❄️': () => S('0 0 32 32', 24, 24, 'svg-spin-slow', `
    <g stroke="#90CAF9" stroke-width="1.8" stroke-linecap="round">
      <line x1="16" y1="2" x2="16" y2="30"/>
      <line x1="4" y1="9" x2="28" y2="23"/>
      <line x1="4" y1="23" x2="28" y2="9"/>
      <line x1="12" y1="4" x2="16" y2="8"/><line x1="20" y1="4" x2="16" y2="8"/>
      <line x1="12" y1="28" x2="16" y2="24"/><line x1="20" y1="28" x2="16" y2="24"/>
    </g>`),

'🌨️': () => S('0 0 36 36', 24, 24, 'svg-sway', `
    <ellipse cx="18" cy="14" rx="10" ry="6" fill="#B0BEC5"/>
    <ellipse cx="12" cy="14" rx="6" ry="5" fill="#CFD8DC"/>
    <ellipse cx="24" cy="14" rx="6" ry="5" fill="#CFD8DC"/>
    <circle cx="12" cy="24" r="2" fill="#E3F2FD"/><circle cx="18" cy="26" r="2" fill="#E3F2FD"/>
    <circle cx="24" cy="23" r="2" fill="#E3F2FD"/><circle cx="15" cy="29" r="1.5" fill="#BBDEFB"/>`),

'🌙': () => S('0 0 32 32', 22, 22, 'svg-glow', `
    <path d="M22 6 A12 12 0 1 0 22 26 A8 8 0 0 1 22 6" fill="#FDD835"/>
    <circle cx="12" cy="10" r="1" fill="#FFF9C4" opacity="0.5"/>
    <circle cx="8" cy="18" r="0.7" fill="#FFF9C4" opacity="0.4"/>`),

'🌕': () => S('0 0 32 32', 22, 22, 'svg-glow', `
    <circle cx="16" cy="16" r="12" fill="#FDD835"/>
    <circle cx="11" cy="12" r="2.5" fill="#FBC02D" opacity="0.5"/>
    <circle cx="19" cy="9" r="1.5" fill="#FBC02D" opacity="0.4"/>
    <circle cx="14" cy="20" r="3" fill="#FBC02D" opacity="0.3"/>`),

'🍁': () => S('0 0 32 32', 24, 24, 'svg-fall', `
    <path d="M16 28 L16 18" stroke="#8D6E63" stroke-width="2" stroke-linecap="round"/>
    <path d="M16 4 L12 10 L6 8 L10 14 L4 16 L10 18 L8 24 L16 18 L24 24 L22 18 L28 16 L22 14 L26 8 L20 10 Z" fill="#FF7043"/>`),

'🍂': () => S('0 0 32 32', 22, 22, 'svg-fall', `
    <path d="M16 4 Q4 12 8 24 Q16 20 16 4" fill="#E65100"/>
    <path d="M16 4 L14 16" stroke="#BF360C" stroke-width="1" fill="none"/>
    <path d="M18 6 Q28 14 24 24 Q18 20 18 6" fill="#FF8F00" opacity="0.8"/>`),

// ── Gardening / Tools ────────────────────────────────────────────────────────
'🪱': () => S('0 0 32 32', 24, 24, 'svg-wiggle', `
    <path d="M6 24 Q10 16 16 20 Q22 24 26 16 Q30 10 28 6" stroke="#E91E63" stroke-width="3.5" fill="none" stroke-linecap="round"/>
    <circle cx="28" cy="6" r="2" fill="#E91E63"/>
    <circle cx="27" cy="5" r="0.7" fill="#fff"/>`),

'🍅': () => S('0 0 32 32', 24, 24, 'svg-bounce', `
    <circle cx="16" cy="18" r="10" fill="#F44336"/>
    <path d="M10 12 Q16 6 22 12" fill="#4CAF50"/>
    <path d="M16 6 L16 10" stroke="#388E3C" stroke-width="1.5" stroke-linecap="round"/>
    <ellipse cx="12" cy="16" rx="3" ry="6" fill="#E53935" opacity="0.4"/>`),

'🥬': () => S('0 0 32 32', 24, 24, 'svg-sway', `
    <ellipse cx="16" cy="18" rx="10" ry="9" fill="#66BB6A"/>
    <ellipse cx="16" cy="16" rx="7" ry="7" fill="#81C784"/>
    <ellipse cx="16" cy="14" rx="4" ry="5" fill="#A5D6A7"/>
    <path d="M16 26 L16 22" stroke="#388E3C" stroke-width="1.5"/>`),

'🥕': () => S('0 0 32 32', 22, 22, 'svg-sway', `
    <path d="M16 6 L10 28 Q16 32 22 28 Z" fill="#FF9800"/>
    <line x1="13" y1="14" x2="16" y2="13" stroke="#F57C00" stroke-width="0.8"/>
    <line x1="13" y1="18" x2="17" y2="17" stroke="#F57C00" stroke-width="0.8"/>
    <path d="M14 8 Q10 2 8 4" stroke="#4CAF50" stroke-width="1.5" fill="none"/>
    <path d="M16 6 Q16 0 18 2" stroke="#4CAF50" stroke-width="1.5" fill="none"/>
    <path d="M18 8 Q22 2 24 4" stroke="#66BB6A" stroke-width="1.5" fill="none"/>`),

// ── Objects / Misc ───────────────────────────────────────────────────────────
'📅': () => S('0 0 32 32', 20, 20, 'svg-pulse', `
    <rect x="4" y="6" width="24" height="22" rx="3" fill="#fff" stroke="#2E7D32" stroke-width="1.5"/>
    <rect x="4" y="6" width="24" height="7" rx="3" fill="#4CAF50"/>
    <line x1="10" y1="3" x2="10" y2="9" stroke="#333" stroke-width="2" stroke-linecap="round"/>
    <line x1="22" y1="3" x2="22" y2="9" stroke="#333" stroke-width="2" stroke-linecap="round"/>
    <g fill="#555" font-size="5" font-weight="bold" text-anchor="middle">
      <text x="10" y="20">12</text><text x="16" y="20">13</text><text x="22" y="20">14</text>
    </g>`),

'📖': () => S('0 0 32 32', 22, 22, 'svg-pulse', `
    <path d="M4 6 Q16 2 16 16 Q16 2 28 6 V28 Q16 24 16 28 Q16 24 4 28 Z" fill="#FFF8E1" stroke="#8D6E63" stroke-width="1.2"/>
    <line x1="16" y1="4" x2="16" y2="28" stroke="#D7CCC8" stroke-width="0.8"/>
    <g stroke="#BDBDBD" stroke-width="0.6"><line x1="7" y1="12" x2="14" y2="11"/><line x1="7" y1="16" x2="14" y2="15"/><line x1="7" y1="20" x2="13" y2="19"/><line x1="18" y1="11" x2="25" y2="12"/><line x1="18" y1="15" x2="25" y2="16"/></g>`),

'📍': () => S('0 0 28 36', 20, 26, 'svg-bounce', `
    <path d="M14 34 Q14 22 6 14 A10 10 0 1 1 22 14 Q14 22 14 34Z" fill="#F44336"/>
    <circle cx="14" cy="12" r="4" fill="#FFCDD2"/>`),

'✍️': () => S('0 0 32 32', 20, 20, 'svg-sway', `
    <path d="M6 26 L22 10 L26 14 L10 30 Z" fill="#FFF9C4" stroke="#8D6E63" stroke-width="1"/>
    <path d="M22 10 L26 6 L30 10 L26 14 Z" fill="#F44336"/>
    <path d="M6 26 L4 30 L10 30" fill="#8D6E63"/>`),

'📘': () => S('0 0 28 32', 20, 24, 'svg-pulse', `
    <rect x="4" y="2" width="20" height="28" rx="2" fill="#1565C0"/>
    <rect x="8" y="2" width="16" height="28" rx="1" fill="#fff"/>
    <g stroke="#BDBDBD" stroke-width="0.6"><line x1="10" y1="8" x2="22" y2="8"/><line x1="10" y1="12" x2="20" y2="12"/><line x1="10" y1="16" x2="22" y2="16"/><line x1="10" y1="20" x2="18" y2="20"/></g>`),

'🛒': () => S('0 0 36 32', 24, 22, 'svg-sway', `
    <path d="M4 4 H8 L14 22 H28 L32 10 H12" stroke="#555" stroke-width="2" fill="none" stroke-linejoin="round"/>
    <circle cx="16" cy="28" r="2.5" fill="#555"/><circle cx="26" cy="28" r="2.5" fill="#555"/>
    <rect x="14" y="12" width="16" height="8" rx="1" fill="#E8F5E9" stroke="#4CAF50" stroke-width="0.8"/>`),

'✂️': () => S('0 0 32 32', 22, 22, 'svg-sway', `
    <circle cx="8" cy="24" r="4" fill="none" stroke="#555" stroke-width="1.5"/>
    <circle cx="24" cy="24" r="4" fill="none" stroke="#555" stroke-width="1.5"/>
    <path d="M10 21 L22 6" stroke="#9E9E9E" stroke-width="2.5" stroke-linecap="round"/>
    <path d="M22 21 L10 6" stroke="#9E9E9E" stroke-width="2.5" stroke-linecap="round"/>`),

'♻️': () => S('0 0 32 32', 22, 22, 'svg-spin-slow', `
    <path d="M16 4 L22 14 L16 12 Z" fill="#4CAF50"/>
    <path d="M26 22 L16 28 L18 22 Z" fill="#66BB6A"/>
    <path d="M6 22 L10 12 L14 18 Z" fill="#81C784"/>
    <path d="M16 4 Q26 4 26 14" stroke="#388E3C" stroke-width="2" fill="none"/>
    <path d="M26 22 Q22 30 12 28" stroke="#388E3C" stroke-width="2" fill="none"/>
    <path d="M6 22 Q4 12 12 8" stroke="#388E3C" stroke-width="2" fill="none"/>`),

'🔄': () => S('0 0 32 32', 20, 20, 'svg-spin-slow', `
    <path d="M16 4 A12 12 0 0 1 28 16" stroke="#4CAF50" stroke-width="2.5" fill="none" stroke-linecap="round"/>
    <path d="M16 28 A12 12 0 0 1 4 16" stroke="#4CAF50" stroke-width="2.5" fill="none" stroke-linecap="round"/>
    <path d="M26 10 L28 16 L22 16" fill="#4CAF50"/>
    <path d="M6 22 L4 16 L10 16" fill="#4CAF50"/>`),

// ── Achievements / Social ────────────────────────────────────────────────────
'🏆': () => S('0 0 32 36', 24, 28, 'svg-glow', `
    <path d="M10 4 H22 L20 18 Q16 22 12 18 Z" fill="#FFD54F" stroke="#F9A825" stroke-width="1"/>
    <rect x="13" y="22" width="6" height="4" fill="#FFA000"/>
    <rect x="10" y="26" width="12" height="3" rx="1" fill="#F9A825"/>
    <path d="M10 8 Q4 8 4 14 Q4 18 10 16" fill="none" stroke="#FFD54F" stroke-width="1.5"/>
    <path d="M22 8 Q28 8 28 14 Q28 18 22 16" fill="none" stroke="#FFD54F" stroke-width="1.5"/>`),

'🎾': () => S('0 0 32 32', 22, 22, 'svg-bounce', `
    <circle cx="16" cy="16" r="11" fill="#C0CA33"/>
    <path d="M6 10 Q16 16 6 24" stroke="#fff" stroke-width="1.8" fill="none"/>
    <path d="M26 10 Q16 16 26 24" stroke="#fff" stroke-width="1.8" fill="none"/>`),

'💪': () => S('0 0 32 32', 22, 22, 'svg-bounce', `
    <path d="M8 22 Q6 14 12 10 Q16 8 18 12 L20 18 Q22 12 26 14 Q28 16 26 20 Q24 26 18 26 L12 26 Q8 26 8 22Z" fill="#FFCC80"/>
    <ellipse cx="16" cy="14" rx="4" ry="5" fill="#FF7043" opacity="0.5"/>`),

'🧠': () => S('0 0 32 32', 22, 22, 'svg-pulse', `
    <path d="M16 4 Q6 4 6 14 Q6 26 16 28 Q26 26 26 14 Q26 4 16 4" fill="#F48FB1"/>
    <path d="M16 6 Q16 28 16 28" stroke="#E91E63" stroke-width="1" fill="none"/>
    <path d="M10 10 Q14 14 10 18" stroke="#E91E63" stroke-width="0.8" fill="none"/>
    <path d="M22 10 Q18 14 22 18" stroke="#E91E63" stroke-width="0.8" fill="none"/>`),

'🎓': () => S('0 0 36 32', 26, 24, 'svg-sway', `
    <path d="M18 4 L2 14 L18 24 L34 14 Z" fill="#333"/>
    <rect x="16" y="14" width="4" height="10" fill="#333"/>
    <rect x="12" y="24" width="12" height="3" rx="1" fill="#333"/>
    <circle cx="32" cy="14" r="1.5" fill="#FDD835"/>
    <line x1="32" y1="15" x2="32" y2="22" stroke="#FDD835" stroke-width="1"/>`),

'💛': () => S('0 0 32 32', 22, 22, 'svg-pulse', `
    <path d="M16 28 Q4 20 4 12 A6 6 0 0 1 16 8 A6 6 0 0 1 28 12 Q28 20 16 28Z" fill="#FFD54F"/>`),

'🤝': () => S('0 0 36 32', 26, 24, 'svg-pulse', `
    <path d="M6 16 L14 16 L18 20 L22 16 L30 16 L26 24 H10 Z" fill="#FFCC80"/>
    <path d="M6 16 Q4 10 8 8 L14 12" stroke="#8D6E63" stroke-width="2" fill="none"/>
    <path d="M30 16 Q32 10 28 8 L22 12" stroke="#8D6E63" stroke-width="2" fill="none"/>`),

'🌀': () => S('0 0 32 32', 22, 22, 'svg-spin-slow', `
    <path d="M16 6 Q24 6 24 14 Q24 20 18 20 Q14 20 14 16 Q14 13 16 13 Q18 13 18 15" stroke="#4FC3F7" stroke-width="2" fill="none" stroke-linecap="round"/>`),

'🌍': () => S('0 0 32 32', 24, 24, 'svg-spin-slow', `
    <circle cx="16" cy="16" r="12" fill="#4FC3F7"/>
    <path d="M8 10 Q12 8 16 12 Q20 16 18 22 Q14 26 10 22 Q6 18 8 10" fill="#66BB6A"/>
    <path d="M20 6 Q24 8 24 12 Q22 14 20 12 Q18 8 20 6" fill="#81C784"/>`),

'✅': () => S('0 0 32 32', 20, 20, 'svg-pulse', `
    <rect x="4" y="4" width="24" height="24" rx="5" fill="#4CAF50"/>
    <path d="M10 16 L14 22 L24 10" stroke="#fff" stroke-width="3" fill="none" stroke-linecap="round" stroke-linejoin="round"/>`),

'🚫': () => S('0 0 32 32', 20, 20, 'svg-pulse', `
    <circle cx="16" cy="16" r="12" fill="none" stroke="#F44336" stroke-width="2.5"/>
    <line x1="7" y1="7" x2="25" y2="25" stroke="#F44336" stroke-width="2.5"/>`),

'✨': () => S('0 0 32 32', 20, 20, 'svg-sparkle', `
    <path d="M16 2 L18 12 L28 14 L18 16 L16 26 L14 16 L4 14 L14 12 Z" fill="#FFD54F"/>
    <path d="M26 4 L27 8 L30 9 L27 10 L26 14 L25 10 L22 9 L25 8 Z" fill="#FFF9C4" opacity="0.7"/>
    <path d="M6 22 L7 25 L10 26 L7 27 L6 30 L5 27 L2 26 L5 25 Z" fill="#FFF9C4" opacity="0.7"/>`),

'📡': () => S('0 0 32 32', 22, 22, 'svg-pulse', `
    <path d="M20 12 Q28 4 28 4" stroke="#4FC3F7" stroke-width="1.5" fill="none" stroke-linecap="round"/>
    <path d="M22 16 A8 8 0 0 0 28 10" stroke="#4FC3F7" stroke-width="1.5" fill="none" opacity="0.7"/>
    <path d="M24 20 A14 14 0 0 0 32 8" stroke="#4FC3F7" stroke-width="1.5" fill="none" opacity="0.4"/>
    <circle cx="18" cy="14" r="3" fill="#1565C0"/>
    <path d="M18 17 L10 28 H16 L14 30 H22 L20 28 H26 Z" fill="#78909C"/>`),

'🐦': () => S('0 0 32 32', 22, 22, 'svg-sway', `
    <ellipse cx="16" cy="18" rx="7" ry="5" fill="#42A5F5"/>
    <circle cx="22" cy="14" r="4" fill="#42A5F5"/>
    <circle cx="24" cy="13" r="1" fill="#333"/>
    <circle cx="24.3" cy="12.7" r="0.4" fill="#fff"/>
    <path d="M26 14 L30 13 L26 16Z" fill="#FF8F00"/>
    <path d="M8 20 Q4 24 6 26" stroke="#42A5F5" stroke-width="2" fill="none"/>
    <path d="M10 16 Q4 10 2 12" stroke="#64B5F6" stroke-width="2" fill="none"/>`),

'📷': () => S('0 0 32 32', 20, 20, 'svg-pulse', `
    <rect x="4" y="8" width="24" height="18" rx="3" fill="#37474F"/>
    <rect x="12" y="4" width="8" height="6" rx="1" fill="#455A64"/>
    <circle cx="16" cy="18" r="5" fill="#263238" stroke="#78909C" stroke-width="1.5"/>
    <circle cx="16" cy="18" r="3" fill="#4FC3F7"/>
    <circle cx="24" cy="12" r="1.5" fill="#FDD835"/>`),

'🧤': () => S('0 0 28 32', 20, 24, 'svg-sway', `
    <path d="M8 12 L8 28 Q8 30 14 30 Q20 30 20 28 L20 14 Q20 6 14 6 Q8 6 8 12" fill="#E53935"/>
    <path d="M8 14 Q4 10 6 6 Q8 8 10 12" fill="#E53935"/>
    <rect x="8" y="12" width="12" height="3" rx="1" fill="#C62828"/>`),

'➕': () => S('0 0 28 28', 18, 18, 'svg-pulse', `
    <circle cx="14" cy="14" r="12" fill="#4CAF50"/>
    <line x1="8" y1="14" x2="20" y2="14" stroke="#fff" stroke-width="3" stroke-linecap="round"/>
    <line x1="14" y1="8" x2="14" y2="20" stroke="#fff" stroke-width="3" stroke-linecap="round"/>`),
};

// ══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ══════════════════════════════════════════════════════════════════════════════

/**
 * Convert an emoji string to its SVG equivalent.
 * Falls back to the original emoji if no SVG exists.
 */
export function emojiToSvg(emoji) {
    const trimmed = (emoji || '').trim();
    const factory = ICONS[trimmed];
    return factory ? factory() : trimmed;
}

/**
 * Returns true if we have an SVG for this emoji.
 */
export function hasSvgIcon(emoji) {
    return ICONS.hasOwnProperty((emoji || '').trim());
}
