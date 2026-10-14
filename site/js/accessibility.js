// js/accessibility.js — Accessibility settings panel
// Runs as a plain IIFE (no module) so it loads on every page via layout_open.php.
// Settings are persisted in localStorage and applied to <body> before first paint.

(function () {
    'use strict';

    // ── 1. Settings definition ────────────────────────────────────────────────
    const SETTINGS = [
        {
            id:    'no-motion',
            cls:   'acc-no-motion',
            label: 'Reduce Motion',
            desc:  'Stops all animations and transitions across the site',
            icon:  '🎞️',
            key:   'acc_no_motion',
        },
        {
            id:    'dark',
            cls:   'acc-dark',
            label: 'Grey Scale',
            desc:  'Converts the entire page to greyscale tones',
            icon:  '🩶',
            key:   'acc_dark',
        },
        {
            id:    'contrast',
            cls:   'acc-contrast',
            label: 'High Contrast',
            desc:  'Maximum black-and-white contrast for low vision users',
            icon:  '◑',
            key:   'acc_contrast',
        },
        {
            id:    'large-text',
            cls:   'acc-large-text',
            label: 'Large Text',
            desc:  'Increases all text to 125% of its normal size',
            icon:  'A+',
            key:   'acc_large_text',
        },
        {
            id:    'focus',
            cls:   'acc-focus',
            label: 'Enhanced Focus',
            desc:  'Shows bright focus rings for keyboard and switch navigation',
            icon:  '🎯',
            key:   'acc_focus',
        },
    ];

    // ── 2. Apply saved settings immediately (before DOMContentLoaded) ─────────
    const saved = {};
    SETTINGS.forEach(s => {
        saved[s.id] = localStorage.getItem(s.key) === '1';
        if (saved[s.id]) document.documentElement.classList.add(s.cls);
    });

    // ── 3. Build the panel after DOM is ready ────────────────────────────────
    document.addEventListener('DOMContentLoaded', function () {
        injectARIALandmarks();
        buildPanel();
        buildButton();
        applyAll();          // sync button states to saved values
    });

    // ── 4. ARIA / landmark improvements ──────────────────────────────────────
    function injectARIALandmarks() {
        // Give <main> a role if not already set
        const main = document.querySelector('main, #main-content, .catalog-wrapper, .page-content');
        if (main && !main.getAttribute('role')) main.setAttribute('role', 'main');

        // Live region for announcements (screen-reader feedback)
        if (!document.getElementById('acc-live')) {
            const live = document.createElement('div');
            live.id = 'acc-live';
            live.setAttribute('role', 'status');
            live.setAttribute('aria-live', 'polite');
            live.setAttribute('aria-atomic', 'true');
            live.className = 'sr-only';
            document.body.appendChild(live);
        }
    }

    function announce(msg) {
        const live = document.getElementById('acc-live');
        if (!live) return;
        live.textContent = '';
        requestAnimationFrame(() => { live.textContent = msg; });
    }

    // ── 5. Floating toggle button ─────────────────────────────────────────────
    function buildButton() {
        const btn = document.createElement('button');
        btn.id = 'acc-open-btn';
        btn.setAttribute('aria-label', 'Open accessibility settings');
        btn.setAttribute('aria-haspopup', 'dialog');
        btn.setAttribute('aria-expanded', 'false');
        btn.setAttribute('title', 'Accessibility Options');
        btn.innerHTML = '<span aria-hidden="true">♿</span>';
        btn.addEventListener('click', togglePanel);
        document.body.appendChild(btn);

        // Keyboard shortcut: Alt + A
        document.addEventListener('keydown', function (e) {
            if (e.altKey && (e.key === 'a' || e.key === 'A')) {
                e.preventDefault();
                togglePanel();
            }
        });
    }

    // ── 6. Settings panel ─────────────────────────────────────────────────────
    function buildPanel() {
        const panel = document.createElement('div');
        panel.id = 'acc-panel';
        panel.setAttribute('role', 'dialog');
        panel.setAttribute('aria-modal', 'true');
        panel.setAttribute('aria-label', 'Accessibility settings');
        panel.setAttribute('aria-hidden', 'true');
        panel.setAttribute('hidden', '');

        panel.innerHTML = `
            <div id="acc-panel-inner">
                <div id="acc-panel-header">
                    <span id="acc-panel-title">
                        <span aria-hidden="true">♿</span> Accessibility
                    </span>
                    <button id="acc-close-btn" aria-label="Close accessibility settings">✕</button>
                </div>

                <p id="acc-panel-hint">
                    Customise your experience. Settings are saved automatically.
                    Keyboard shortcut: <kbd>Alt</kbd> + <kbd>A</kbd>
                </p>

                <ul id="acc-toggles" role="list">
                    ${SETTINGS.map(s => `
                    <li class="acc-toggle-row" id="acc-row-${s.id}">
                        <div class="acc-toggle-info">
                            <span class="acc-toggle-icon" aria-hidden="true">${s.icon}</span>
                            <div class="acc-toggle-labels">
                                <span class="acc-toggle-name">${s.label}</span>
                                <span class="acc-toggle-desc">${s.desc}</span>
                            </div>
                        </div>
                        <button
                            class="acc-switch"
                            id="acc-btn-${s.id}"
                            role="switch"
                            aria-checked="false"
                            aria-label="${s.label}"
                            data-acc-id="${s.id}"
                        >
                            <span class="acc-switch-thumb" aria-hidden="true"></span>
                        </button>
                    </li>`).join('')}
                </ul>

                <div id="acc-reset-row">
                    <button id="acc-reset-btn">Reset all to defaults</button>
                </div>

                <div id="acc-info-section" aria-label="Additional accessibility information">
                    <h3>Tips for using this site</h3>
                    <ul>
                        <li>Use <kbd>Tab</kbd> to move between interactive elements</li>
                        <li>Press <kbd>Enter</kbd> or <kbd>Space</kbd> to activate buttons</li>
                        <li>Press <kbd>Escape</kbd> to close modals and menus</li>
                        <li>Plant cards include full details in their expanded view</li>
                        <li>Search results update as you type</li>
                    </ul>
                </div>
            </div>
        `;

        document.body.appendChild(panel);

        // Wire up switches
        panel.querySelectorAll('.acc-switch').forEach(btn => {
            btn.addEventListener('click', function () {
                const id = this.dataset.accId;
                toggle(id);
            });
        });

        // Close button
        document.getElementById('acc-close-btn').addEventListener('click', closePanel);

        // Reset button
        document.getElementById('acc-reset-btn').addEventListener('click', resetAll);

        // Close on backdrop click
        panel.addEventListener('click', function (e) {
            if (e.target === panel) closePanel();
        });

        // Escape key closes
        document.addEventListener('keydown', function (e) {
            if (e.key === 'Escape' && !panel.hasAttribute('hidden')) closePanel();
        });

        // Focus trap inside panel
        panel.addEventListener('keydown', trapFocus);
    }

    // ── 7. Toggle logic ───────────────────────────────────────────────────────
    function toggle(id) {
        const s = SETTINGS.find(x => x.id === id);
        if (!s) return;
        saved[id] = !saved[id];
        localStorage.setItem(s.key, saved[id] ? '1' : '0');
        applyOne(s);
        announce(s.label + (saved[id] ? ' enabled' : ' disabled'));
    }

    function applyOne(s) {
        // High contrast always wins over dark mode — if both on, add both classes
        document.documentElement.classList.toggle(s.cls, saved[s.id]);

        const btn = document.getElementById('acc-btn-' + s.id);
        if (btn) btn.setAttribute('aria-checked', saved[s.id] ? 'true' : 'false');

        const row = document.getElementById('acc-row-' + s.id);
        if (row) row.classList.toggle('acc-active', saved[s.id]);
    }

    function applyAll() {
        SETTINGS.forEach(s => applyOne(s));
    }

    function resetAll() {
        SETTINGS.forEach(s => {
            saved[s.id] = false;
            localStorage.removeItem(s.key);
            document.documentElement.classList.remove(s.cls);
        });
        applyAll();
        announce('All accessibility settings have been reset');
    }

    // ── 8. Panel open / close ─────────────────────────────────────────────────
    let lastFocus = null;

    function togglePanel() {
        const panel = document.getElementById('acc-panel');
        if (!panel) return;
        if (panel.hasAttribute('hidden')) openPanel();
        else closePanel();
    }

    function openPanel() {
        const panel = document.getElementById('acc-panel');
        const btn   = document.getElementById('acc-open-btn');
        if (!panel) return;

        lastFocus = document.activeElement;
        panel.removeAttribute('hidden');
        panel.setAttribute('aria-hidden', 'false');
        if (btn) btn.setAttribute('aria-expanded', 'true');

        // Focus first switch after animation frame
        requestAnimationFrame(() => {
            const first = panel.querySelector('.acc-switch, #acc-close-btn');
            if (first) first.focus();
        });
    }

    function closePanel() {
        const panel = document.getElementById('acc-panel');
        const btn   = document.getElementById('acc-open-btn');
        if (!panel) return;

        panel.setAttribute('hidden', '');
        panel.setAttribute('aria-hidden', 'true');
        if (btn) {
            btn.setAttribute('aria-expanded', 'false');
            btn.focus();
        }
        if (lastFocus) lastFocus.focus();
    }

    // ── 9. Focus trap ─────────────────────────────────────────────────────────
    function trapFocus(e) {
        if (e.key !== 'Tab') return;
        const panel = document.getElementById('acc-panel');
        if (!panel) return;
        const focusable = Array.from(panel.querySelectorAll(
            'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        )).filter(el => !el.disabled && el.offsetParent !== null);
        if (!focusable.length) return;
        const first = focusable[0];
        const last  = focusable[focusable.length - 1];
        if (e.shiftKey && document.activeElement === first) {
            e.preventDefault(); last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
            e.preventDefault(); first.focus();
        }
    }

})();
