/* ================================================================
   site-theme.js  —  Site-Wide Theme System for Jun's & Kona's Garden
   ================================================================
   Loaded on every page via layout_open.php.  Provides a floating
   theme picker (bottom-left) and applies the chosen palette to
   global CSS variables consumed by header, footer, body, buttons,
   waves, data-page widgets, etc.

   Default theme: spring_d (Spring Dark).
   Persists in localStorage; also syncs server-side for logged-in users.
   On catalog.php the catalog theme picker is hidden and this one
   takes over — it also sets the --svd-* card variables.
   ================================================================ */
(function () {
    'use strict';

    /* ── Theme definitions ──────────────────────────────────────── */
    const SITE_THEMES = {
        // ── Green (default) ─────────────────────────────────────────
        spring_d: {
            label: 'Spring', sub: '',
            header: '#4A7c59', headerText: '#ffffff',
            body: '#fffff0', bodyText: '#33402f',
            footer: '#4A7c59', footerText: '#fff',
            accent: '#2e7d32', accentHover: '#256d29',
            waveBack: '#3a6a48', waveFront: '#4A7c59',
            modalBg: '#4A7c59',
            chipBg: 'rgba(46,125,50,0.10)', tint: 'rgba(46,125,50,0.06)',
            rowBorder: '#c0d8a5', rowLabel: '#5a7a50',
            photoBg: '#c8e6c9',
            cardHeader: '#4A7c59', cardBody: '#fffff0'
        },
        // ── Red ─────────────────────────────────────────────────────
        zenith: {
            label: 'Zenith', sub: '',
            header: '#b81c1c', headerText: '#ffffff',
            body: '#fff5f4', bodyText: '#3a1210',
            footer: '#b81c1c', footerText: '#fff',
            accent: '#d32f2f', accentHover: '#b71c1c',
            waveBack: '#a01818', waveFront: '#b81c1c',
            modalBg: '#b81c1c',
            chipBg: 'rgba(211,47,47,0.08)', tint: 'rgba(211,47,47,0.05)',
            rowBorder: '#e0a0a0', rowLabel: '#a04040',
            photoBg: '#ffcdd2',
            cardHeader: '#e53935', cardBody: '#fff5f4'
        },
        summer: {
            label: 'Rosa', sub: '',
            header: '#d8506a', headerText: '#ffffff',
            body: '#fff5f5', bodyText: '#3E2723',
            footer: '#e87088', footerText: '#fff',
            accent: '#D84356', accentHover: '#c03848',
            waveBack: '#c84860', waveFront: '#d8506a',
            modalBg: '#e87088',
            chipBg: 'rgba(216,67,86,0.08)', tint: 'rgba(216,67,86,0.05)',
            rowBorder: '#f0b0b8', rowLabel: '#b05060',
            photoBg: '#ffe0e6',
            cardHeader: '#FF91A4', cardBody: '#fff5f5'
        },
        // ── Orange ──────────────────────────────────────────────────
        sunset: {
            label: 'Sunset', sub: '',
            header: '#c07030', headerText: '#ffffff',
            body: '#fff8f0', bodyText: '#4a2810',
            footer: '#c07030', footerText: '#fff',
            accent: '#c05a10', accentHover: '#a04a08',
            waveBack: '#a86028', waveFront: '#c07030',
            modalBg: '#c07030',
            chipBg: 'rgba(192,90,16,0.08)', tint: 'rgba(192,90,16,0.05)',
            rowBorder: '#d0a070', rowLabel: '#905020',
            photoBg: '#ffe0b8',
            cardHeader: '#e8a060', cardBody: '#fff8f0'
        },
        fall: {
            label: 'Fall', sub: '',
            header: '#b08060', headerText: '#ffffff',
            body: '#fff5ee', bodyText: '#3E2723',
            footer: '#b08060', footerText: '#fff',
            accent: '#BF5B00', accentHover: '#a04e00',
            waveBack: '#9a7050', waveFront: '#b08060',
            modalBg: '#b08060',
            chipBg: 'rgba(153,51,0,0.08)', tint: 'rgba(153,51,0,0.05)',
            rowBorder: '#d4a878', rowLabel: '#8a6030',
            photoBg: '#ffe0b2',
            cardHeader: '#ecded4', cardBody: '#fff5ee'
        },
        dirt: {
            label: 'Dirt', sub: '',
            header: '#8a6a48', headerText: '#ffffff',
            body: '#faf5ee', bodyText: '#3a2a18',
            footer: '#a08060', footerText: '#fff',
            accent: '#7a5a3a', accentHover: '#6a4a2a',
            waveBack: '#7a5a38', waveFront: '#8a6a48',
            modalBg: '#a08060',
            chipBg: 'rgba(155,118,83,0.08)', tint: 'rgba(155,118,83,0.05)',
            rowBorder: '#c8a888', rowLabel: '#7a6040',
            photoBg: '#f0e0d0',
            cardHeader: '#d5c4ad', cardBody: '#faf5ee'
        },
        // ── Yellow ──────────────────────────────────────────────────
        sunrise: {
            label: 'Sunrise', sub: '',
            header: '#c0a040', headerText: '#ffffff',
            body: '#fdfce8', bodyText: '#3a3010',
            footer: '#c0a040', footerText: '#fff',
            accent: '#c0a020', accentHover: '#a08818',
            waveBack: '#b09030', waveFront: '#c0a040',
            modalBg: '#c0a040',
            chipBg: 'rgba(192,160,32,0.08)', tint: 'rgba(192,160,32,0.05)',
            rowBorder: '#d8cc80', rowLabel: '#8a7a30',
            photoBg: '#fff8d0',
            cardHeader: '#f5ecc0', cardBody: '#fdfce8'
        },
        // ── Blue ────────────────────────────────────────────────────
        jun: {
            label: 'Jun', sub: '',
            header: '#5a88bb', headerText: '#ffffff',
            body: '#f0f5ff', bodyText: '#1a2a4a',
            footer: '#5a88bb', footerText: '#fff',
            accent: '#3366CC', accentHover: '#2a56b0',
            waveBack: '#4a78aa', waveFront: '#5a88bb',
            modalBg: '#5a88bb',
            chipBg: 'rgba(51,102,204,0.10)', tint: 'rgba(51,102,204,0.06)',
            rowBorder: '#a0b8d8', rowLabel: '#4a6a90',
            photoBg: '#dce8f5',
            cardHeader: '#d0def5', cardBody: '#f0f5ff'
        },
        // ── Indigo ──────────────────────────────────────────────────
        winter_d: {
            label: 'Winter', sub: '',
            header: '#1a5276', headerText: '#ffffff',
            body: '#f0faff', bodyText: '#0D47A1',
            footer: '#1a5276', footerText: '#fff',
            accent: '#1976D2', accentHover: '#1565C0',
            waveBack: '#12426a', waveFront: '#1a5276',
            modalBg: '#1a5276',
            chipBg: 'rgba(25,118,210,0.08)', tint: 'rgba(25,118,210,0.05)',
            rowBorder: '#90c0d8', rowLabel: '#3a7090',
            photoBg: '#bbdefb',
            cardHeader: '#1a5276', cardBody: '#f0faff'
        },
        // ── Violet ──────────────────────────────────────────────────
        lavender_d: {
            label: 'Kona', sub: '',
            header: '#7B1FA2', headerText: '#ffffff',
            body: '#faf0f5', bodyText: '#4A148C',
            footer: '#7B1FA2', footerText: '#fff',
            accent: '#8E24AA', accentHover: '#6A1B9A',
            waveBack: '#6a1890', waveFront: '#7B1FA2',
            modalBg: '#7B1FA2',
            chipBg: 'rgba(142,36,170,0.08)', tint: 'rgba(142,36,170,0.05)',
            rowBorder: '#d0a8c8', rowLabel: '#8a5080',
            photoBg: '#e1bee7',
            cardHeader: '#7B1FA2', cardBody: '#faf0f5'
        },
        lavender_l: {
            label: 'Lavender', sub: 'Light',
            header: '#9b70b0', headerText: '#ffffff',
            body: '#faf0f5', bodyText: '#4A148C',
            footer: '#9b70b0', footerText: '#fff',
            accent: '#8E24AA', accentHover: '#7B1FA2',
            waveBack: '#8a60a0', waveFront: '#9b70b0',
            modalBg: '#9b70b0',
            chipBg: 'rgba(142,36,170,0.08)', tint: 'rgba(142,36,170,0.05)',
            rowBorder: '#d0a8c8', rowLabel: '#8a5080',
            photoBg: '#f3e5f5',
            cardHeader: '#e4d4ec', cardBody: '#faf0f5'
        },
        // ── Neutral ─────────────────────────────────────────────────
        jason: {
            label: 'Author', sub: '',
            header: '#010203', headerText: '#ffffff',
            body: '#f5f6f8', bodyText: '#1a1c20',
            footer: '#3a3e44', footerText: '#fff',
            accent: '#2a2d32', accentHover: '#1a1d22',
            waveBack: '#2a2e34', waveFront: '#3a3e44',
            modalBg: '#3a3e44',
            chipBg: 'rgba(1,2,3,0.08)', tint: 'rgba(1,2,3,0.04)',
            rowBorder: '#c0c4ca', rowLabel: '#606468',
            photoBg: '#e4e6ea',
            cardHeader: '#010203', cardBody: '#f5f6f8'
        }
    };

    const STORAGE_KEY = 'garden_site_theme';
    const DEFAULT_THEME = 'spring_d';

    /* ── Determine perceived brightness of a hex colour ─────────── */
    function luminance(hex) {
        const c = hex.replace('#', '');
        const r = parseInt(c.substring(0, 2), 16);
        const g = parseInt(c.substring(2, 4), 16);
        const b = parseInt(c.substring(4, 6), 16);
        return (r * 299 + g * 587 + b * 114) / 1000;
    }

    /* ── Apply theme to CSS custom properties ───────────────────── */
    function applySiteTheme(name) {
        const t = SITE_THEMES[name] || SITE_THEMES[DEFAULT_THEME];
        if (!t) return;
        const r = document.documentElement.style;

        // Header
        r.setProperty('--pw-green', t.header);
        r.setProperty('--site-header-text', t.headerText);

        // Body
        r.setProperty('--site-body-bg', t.body);
        r.setProperty('--site-body-text', t.bodyText);

        // Footer
        r.setProperty('--site-footer-bg', t.footer);
        r.setProperty('--site-footer-text', t.footerText);

        // Accent / buttons
        r.setProperty('--primary-green', t.accent);
        r.setProperty('--site-accent', t.accent);
        r.setProperty('--site-accent-hover', t.accentHover);

        // Waves
        r.setProperty('--site-wave-back', t.waveBack);
        r.setProperty('--site-wave-front', t.waveFront);

        // Modal
        r.setProperty('--modal-bg', t.modalBg);
        r.setProperty('--modal-accent', t.accent);

        // Page content background (used by .page-content, plant_listing, etc.)
        r.setProperty('--light-green', t.body);

        // Data page widgets
        r.setProperty('--site-chip-bg', t.chipBg);
        r.setProperty('--site-tint', t.tint);
        r.setProperty('--site-row-border', t.rowBorder);
        r.setProperty('--site-row-label', t.rowLabel);

        // Nav link contrast — auto-detect from header brightness
        const headerLight = luminance(t.header) > 160;
        r.setProperty('--site-nav-link', headerLight ? 'rgba(0,0,0,0.75)' : 'rgba(255,255,255,0.9)');
        r.setProperty('--site-nav-hover-bg', headerLight ? 'rgba(0,0,0,0.10)' : 'rgba(255,255,255,0.2)');
        r.setProperty('--site-nav-hover-text', headerLight ? '#000' : '#fff');
        r.setProperty('--site-logo-color', headerLight ? '#1a1a1a' : '#fff');

        // Expose for modal JS
        window._siteThemeModalBg = t.modalBg;

        // Persist
        try { localStorage.setItem(STORAGE_KEY, name); } catch (e) {}

        // Server-side persistence for logged-in users
        if (window.SITE_CONFIG && window.SITE_CONFIG.currentUser ||
            window.GARDEN_CONFIG && window.GARDEN_CONFIG.currentUser) {
            fetch('plant_actions.php', {
                method: 'POST',
                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                body: new URLSearchParams({ action: 'save_pref', key: 'site_theme', value: name })
            }).catch(() => {});
        }

        // Update picker swatches
        document.querySelectorAll('.st-swatch').forEach(s => {
            s.classList.toggle('active', s.dataset.theme === name);
        });
    }

    /* ── Floating theme picker (bottom-left) ────────────────────── */
    function buildSiteThemePicker() {
        const wrap = document.createElement('div');
        wrap.id = 'site-theme-picker';
        wrap.innerHTML = `
            <button class="st-toggle" title="Change theme" aria-label="Theme picker">🎨</button>
            <div class="st-panel hidden">
                <div class="st-panel-title">Site Theme</div>
                <div class="st-swatches">
                    ${Object.entries(SITE_THEMES).map(([key, t]) => `
                        <div class="st-swatch-item" data-theme="${key}">
                            <div class="st-swatch" data-theme="${key}" title="${t.label}${t.sub ? ' ' + t.sub : ''}">
                                <div class="st-half st-left" style="background:${t.header}"></div>
                                <div class="st-half st-right" style="background:${t.body}"></div>
                            </div>
                            <span class="st-name">${t.label}</span>
                            ${t.sub ? `<span class="st-sub">${t.sub}</span>` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
        document.body.appendChild(wrap);

        // Toggle panel
        const toggle = wrap.querySelector('.st-toggle');
        const panel = wrap.querySelector('.st-panel');
        toggle.addEventListener('click', (e) => {
            e.stopPropagation();
            panel.classList.toggle('hidden');
        });

        // Close on outside click
        document.addEventListener('click', (e) => {
            if (!wrap.contains(e.target)) panel.classList.add('hidden');
        });

        // Swatch clicks
        wrap.querySelectorAll('.st-swatch-item').forEach(item => {
            item.addEventListener('click', () => {
                applySiteTheme(item.dataset.theme);
            });
        });
    }

    /* ── Inject picker styles ───────────────────────────────────── */
    function injectStyles() {
        const style = document.createElement('style');
        style.textContent = `
/* ── Site Theme Picker ────────────────────────────── */
#site-theme-picker {
    position: fixed; bottom: 18px; left: 18px; z-index: 99999;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}
.st-toggle {
    width: 48px; height: 48px; border-radius: 50%;
    border: 3px solid #222; background: #fff;
    font-size: 22px; cursor: pointer;
    box-shadow: 3px 3px 0 rgba(0,0,0,0.5);
    transition: transform 0.15s, box-shadow 0.15s;
    display: flex; align-items: center; justify-content: center;
}
.st-toggle:hover { transform: scale(1.1); box-shadow: 4px 4px 0 rgba(0,0,0,0.6); }
.st-panel {
    position: absolute; bottom: 58px; left: 0;
    background: #fff; border: 3px solid #222; border-radius: 16px;
    box-shadow: 5px 5px 0 rgba(0,0,0,0.5);
    padding: 14px; width: 240px;
    max-height: 70vh; overflow-y: auto;
    animation: stSlideUp 0.2s ease-out;
}
.st-panel.hidden { display: none; }
@keyframes stSlideUp { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
.st-panel-title {
    font-size: 0.82rem; font-weight: 800; text-transform: uppercase;
    letter-spacing: 0.08em; color: #555; margin-bottom: 10px;
    padding-bottom: 6px; border-bottom: 2px solid #eee;
}
.st-swatches {
    display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px;
}
.st-swatch-item {
    display: flex; flex-direction: column; align-items: center;
    cursor: pointer; gap: 3px;
}
.st-swatch-item:hover .st-swatch { transform: scale(1.15); }
.st-swatch {
    width: 40px; height: 40px; border-radius: 50%;
    border: 2.5px solid #222; overflow: hidden;
    display: flex; transition: transform 0.15s, box-shadow 0.15s;
    box-shadow: 2px 2px 0 rgba(0,0,0,0.35);
}
.st-swatch.active { border-color: #FFD700; box-shadow: 0 0 0 2px #FFD700, 2px 2px 0 rgba(0,0,0,0.35); }
.st-half { flex: 1; }
.st-name { font-size: 0.62rem; font-weight: 700; color: #333; line-height: 1.1; text-align: center; }
.st-sub { font-size: 0.52rem; font-weight: 600; color: #888; line-height: 1; margin-top: -2px; }

/* ── Accessibility panel offset — move up when acc panel is open ── */
body.acc-panel-open #site-theme-picker { bottom: 320px; }

@media (max-width: 600px) {
    #site-theme-picker { bottom: 12px; left: 12px; }
    .st-panel { width: 210px; padding: 10px; }
    .st-swatch { width: 34px; height: 34px; }
    .st-toggle { width: 42px; height: 42px; font-size: 19px; }
}
`;
        document.head.appendChild(style);
    }

    /* ── Initialise ─────────────────────────────────────────────── */
    function init() {
        injectStyles();

        // Determine saved theme
        let saved = null;
        try { saved = localStorage.getItem(STORAGE_KEY); } catch (e) {}
        const themeName = (saved && SITE_THEMES[saved]) ? saved : DEFAULT_THEME;

        // Apply immediately (before DOM ready for instant paint)
        applySiteTheme(themeName);

        // Build picker once DOM is ready
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', buildSiteThemePicker);
        } else {
            buildSiteThemePicker();
        }
    }

    // Expose globally so catalog's applySvdTheme can call back
    window.applySiteTheme = applySiteTheme;
    window.SITE_THEMES = SITE_THEMES;

    init();
})();
