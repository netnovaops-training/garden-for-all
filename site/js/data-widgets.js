// =============================================================================
// js/data-widgets.js — Widget picker for the Data / Analytics page
// =============================================================================
(function () {
    'use strict';

    const STORAGE_KEY = 'garden_widget_active_v7';

    // ── Widget definitions with groups ──
    // Groups: 'live' = real data, 'demo' = IoT mock/future
    // Sorted alphabetically within each group at render time.
    const WIDGETS = [
        // ── AI widget ──
        { id: 'ww-ai-insights',   label: 'AI Insights',            icon: '🧠',  group: 'live', alwaysOn: true },
        // ── Live data widgets ──
        { id: 'ww-season',        label: 'Bloom Season',           icon: '🌸',  group: 'live' },
        { id: 'ww-plant-weather',  label: 'Plant Weather Detective', icon: '🔎', group: 'live', alwaysOn: true },
        { id: 'ww-intelligence',   label: 'Garden Intelligence',    icon: '🧠', group: 'live', alwaysOn: true },
        { id: 'ww-iot-current',   label: 'Current Conditions',     icon: '🌡️', group: 'live', alwaysOn: true },
        { id: 'ww-deer',          label: 'Deer Resistance',        icon: '🦌',  group: 'live' },
        { id: 'ww-edibility',     label: 'Edible vs Toxic',        icon: '🍃',  group: 'live' },
        { id: 'ww-evergreen',     label: 'Evergreen Coverage',     icon: '🌲',  group: 'live' },
        { id: 'ww-fertilizer',    label: 'Fertilizer',             icon: '🌱',  group: 'live' },
        { id: 'ww-beds',          label: 'Flower Beds',            icon: '🏡',  group: 'live' },
        { id: 'ww-flower-shapes', label: 'Flower Shapes',          icon: '💮',  group: 'live' },
        { id: 'ww-growth',        label: 'Growth by Year',         icon: '📈',  group: 'live' },
        { id: 'ww-growth-rate',   label: 'Growth Rate',            icon: '🚀',  group: 'live' },
        { id: 'ww-grow-season',   label: 'Grow Season',            icon: '🌿',  group: 'live' },
        { id: 'ww-zones',         label: 'Hardiness Zones',        icon: '🌍',  group: 'live' },
        { id: 'ww-investment',    label: 'Investment ($)',          icon: '💰',  group: 'live' },
        { id: 'ww-store',         label: 'Local Stores',           icon: '🛒',  group: 'live' },
        { id: 'ww-native',        label: 'Native vs Cultivar',     icon: '🗺️',  group: 'live' },
        { id: 'ww-vendors',       label: 'Plant Brands',           icon: '🏢',  group: 'live' },
        { id: 'ww-families',      label: 'Plant Families',         icon: '🌺',  group: 'live' },
        { id: 'ww-losses',        label: 'Plant Losses',           icon: '🥀',  group: 'live' },
        { id: 'ww-roots',         label: 'Root Systems',           icon: '🌳',  group: 'live' },
        { id: 'ww-sun',           label: 'Sun Exposure',           icon: '☀️',  group: 'live' },
        { id: 'ww-survival',      label: 'Survival Rate',          icon: '📉',  group: 'live' },
        { id: 'ww-top-plants',    label: 'Top Plants (Qty)',       icon: '🏆',  group: 'live' },
        { id: 'ww-toxicity',      label: 'Toxicity Safety',        icon: '☠️',  group: 'live' },
        { id: 'ww-water',         label: 'Water Needs',            icon: '💧',  group: 'live' },
        { id: 'ww-iot-moisture',   label: 'Moisture & Precipitation',icon: '🌧️', group: 'demo' },
        { id: 'ww-iot-heatgrid',  label: 'Weather Heatmap',         icon: '🗓️', group: 'live' },
        { id: 'ww-iot-weather',   label: 'Weather Trends',         icon: '🌦️', group: 'live' },
        { id: 'ww-wildlife',      label: 'Wildlife / Pollinators', icon: '🦋',  group: 'live' },
        // ── Demo / Future IoT widgets ──
        { id: 'ww-iot-soil',      label: 'Soil Moisture',          icon: '💧',  group: 'demo' },
        { id: 'ww-iot-water-use', label: 'Water Usage',            icon: '🚿',  group: 'demo' },
    ];

    // All live widgets on by default, demos off
    const DEFAULT_ACTIVE = WIDGETS.filter(w => w.group === 'live').map(w => w.id);

    function loadActive() {
        try {
            const v = localStorage.getItem(STORAGE_KEY);
            return v !== null ? JSON.parse(v) : null;
        } catch { return null; }
    }

    function saveActive(arr) {
        try { localStorage.setItem(STORAGE_KEY, JSON.stringify(arr)); } catch {}
    }

    function getActive() {
        const saved = loadActive();
        return saved !== null ? saved : [...DEFAULT_ACTIVE];
    }

    function applyVisibility(active) {
        const grid = document.getElementById('widget-grid');
        if (!grid) return;

        const wrappers = [];
        WIDGETS.forEach(function (w) {
            const el = document.getElementById(w.id);
            if (el) {
                // alwaysOn widgets are always visible
                el.classList.toggle('ww-visible', w.alwaysOn || active.includes(w.id));
                wrappers.push({ el: el, widget: w });
            }
        });

        // Sort: live first (alphabetical), then demo (alphabetical)
        // But pin 'Current Conditions' and 'Weather Trends' to positions 1-2
        const pinned = ['ww-iot-current', 'ww-intelligence', 'ww-plant-weather', 'ww-iot-heatgrid', 'ww-iot-weather'];
        wrappers.sort(function (a, b) {
            const aPin = pinned.indexOf(a.widget.id);
            const bPin = pinned.indexOf(b.widget.id);
            // Pinned items go first
            if (aPin !== -1 && bPin !== -1) return aPin - bPin;
            if (aPin !== -1) return -1;
            if (bPin !== -1) return 1;
            // Then group: live before demo
            if (a.widget.group !== b.widget.group) return a.widget.group === 'live' ? -1 : 1;
            // Then alphabetical
            return a.widget.label.localeCompare(b.widget.label);
        });

        // Re-append in sorted order
        wrappers.forEach(function (w) { grid.appendChild(w.el); });
    }

    function renderChips(active) {
        const bar = document.getElementById('picker-chips');
        if (!bar) return;
        bar.innerHTML = '';

        // Sort chips alphabetically within groups
        const sorted = WIDGETS.slice().sort(function (a, b) {
            if (a.group !== b.group) return a.group === 'live' ? -1 : 1;
            return a.label.localeCompare(b.label);
        });

        let lastGroup = '';
        sorted.filter(function(w) { return !w.alwaysOn; }).forEach(function (w) {
            // Add group separator for demo
            if (w.group !== lastGroup && w.group === 'demo') {
                const sep = document.createElement('div');
                sep.style.cssText = 'width:100%;font-size:0.7rem;font-weight:700;color:#888;text-transform:uppercase;letter-spacing:0.08em;margin-top:8px;padding-top:8px;border-top:1px dashed #ccc;';
                sep.textContent = '🔌 Demo / Future IoT';
                bar.appendChild(sep);
            }
            lastGroup = w.group;

            const isOn = active.includes(w.id);
            const chip = document.createElement('button');
            chip.type      = 'button';
            chip.className = 'widget-chip' + (isOn ? ' wc-active' : '');
            chip.setAttribute('aria-pressed', String(isOn));
            chip.innerHTML = isOn
                ? '<span class="chip-icon">' + w.icon + '</span>' + w.label + ' <span class="chip-x" aria-hidden="true">✕</span>'
                : '<span class="chip-icon">' + w.icon + '</span>' + w.label;

            chip.addEventListener('click', function () {
                let cur = getActive();
                cur = isOn ? cur.filter(id => id !== w.id) : cur.concat([w.id]);
                saveActive(cur);
                applyVisibility(cur);
                renderChips(cur);
            });

            bar.appendChild(chip);
        });
    }

    function init() {
        const btnAll  = document.getElementById('btn-show-all');
        const btnNone = document.getElementById('btn-hide-all');

        if (btnAll) btnAll.addEventListener('click', function () {
            const all = WIDGETS.map(w => w.id);
            saveActive(all); applyVisibility(all); renderChips(all);
        });
        if (btnNone) btnNone.addEventListener('click', function () {
            saveActive([]); applyVisibility([]); renderChips([]);
        });

        const active = getActive();
        applyVisibility(active);
        renderChips(active);

        // Auto-expand the compare tool
        const compareTool = document.getElementById('compare-tool');
        if (compareTool && !compareTool.open) {
            compareTool.open = true;
        }
    }

    // Expose for widget-loader.js to re-init after AJAX widgets load
    window.initWidgetPicker = function () {
        const active = getActive();
        applyVisibility(active);
        renderChips(active);
    };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
}());
