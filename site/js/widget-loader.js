/**
 * widget-loader.js — Lazy-loads plant widgets from widget_data.php
 *
 * Fetches all widget data in ONE request, then renders each chart
 * only when the user scrolls it into view (IntersectionObserver).
 */
(function () {
    'use strict';

    // Color palettes
    const COLORS = {
        green6:  ['rgba(76,175,80,0.75)','rgba(129,199,132,0.75)','rgba(165,214,167,0.75)','rgba(200,230,201,0.75)','rgba(46,125,50,0.75)','rgba(104,159,56,0.75)'],
        green12: ['rgba(76,175,80,0.75)','rgba(129,199,132,0.75)','rgba(165,214,167,0.75)','rgba(200,230,201,0.75)','rgba(46,125,50,0.75)','rgba(104,159,56,0.75)','rgba(174,213,129,0.75)','rgba(220,237,200,0.75)','rgba(100,221,23,0.75)','rgba(56,142,60,0.75)','rgba(27,94,32,0.75)','rgba(51,105,30,0.75)'],
        sun6:    ['rgba(255,235,59,0.7)','rgba(255,183,77,0.7)','rgba(255,138,101,0.7)','rgba(129,199,132,0.7)','rgba(100,181,246,0.7)','rgba(206,147,216,0.7)'],
        fert8:   ['rgba(76,175,80,0.75)','rgba(0,150,136,0.75)','rgba(3,169,244,0.75)','rgba(156,39,176,0.75)','rgba(255,152,0,0.75)','rgba(255,87,34,0.75)','rgba(121,85,72,0.75)','rgba(96,125,139,0.75)'],
        wild8:   ['rgba(255,183,77,0.8)','rgba(129,199,132,0.8)','rgba(100,181,246,0.8)','rgba(206,147,216,0.8)','rgba(255,138,101,0.8)','rgba(77,208,225,0.8)','rgba(174,213,129,0.8)','rgba(255,213,79,0.8)'],
        water6:  ['rgba(33,150,243,0.7)','rgba(100,181,246,0.7)','rgba(144,202,249,0.7)','rgba(66,165,245,0.7)','rgba(30,136,229,0.7)','rgba(187,222,251,0.7)'],
        season8: ['rgba(255,183,77,0.7)','rgba(76,175,80,0.7)','rgba(244,67,54,0.7)','rgba(100,181,246,0.7)','rgba(255,235,59,0.7)','rgba(206,147,216,0.7)','rgba(255,138,101,0.7)','rgba(129,199,132,0.7)'],
        grow10:  ['rgba(76,175,80,0.7)','rgba(139,195,74,0.7)','rgba(205,220,57,0.7)','rgba(255,235,59,0.7)','rgba(255,193,7,0.7)','rgba(255,152,0,0.7)','rgba(255,87,34,0.7)','rgba(121,85,72,0.7)','rgba(158,158,158,0.7)','rgba(96,125,139,0.7)'],
        rainbow12: ['rgba(244,67,54,0.7)','rgba(233,30,99,0.7)','rgba(156,39,176,0.7)','rgba(103,58,183,0.7)','rgba(63,81,181,0.7)','rgba(33,150,243,0.7)','rgba(0,188,212,0.7)','rgba(0,150,136,0.7)','rgba(76,175,80,0.7)','rgba(205,220,57,0.7)','rgba(255,193,7,0.7)','rgba(255,87,34,0.7)'],
        top12:   ['rgba(76,175,80,0.7)','rgba(129,199,132,0.7)','rgba(165,214,167,0.7)','rgba(46,125,50,0.7)','rgba(104,159,56,0.7)','rgba(174,213,129,0.7)','rgba(56,142,60,0.7)','rgba(27,94,32,0.7)','rgba(200,230,201,0.7)','rgba(220,237,200,0.7)','rgba(100,221,23,0.7)','rgba(139,195,74,0.7)'],
    };

    const edibilityColors = {
        'Fully Edible':'rgba(76,175,80,0.8)','Non-toxic':'rgba(129,199,132,0.8)',
        'Wildlife Only':'rgba(255,183,77,0.8)','Caution (Pets)':'rgba(255,152,0,0.8)',
        'Caution (Humans)':'rgba(255,87,34,0.8)','Toxic':'rgba(244,67,54,0.8)',
        'Not Specified':'rgba(189,189,189,0.7)',
        // Added 2026-08-28 with the new bucket in widget_data.php. Sits between
        // "Caution (Humans)" and "Toxic" on purpose -- it IS more serious than
        // either single-audience caution, and reading darker says so without a
        // legend. Every colour above is untouched.
        'Caution (Humans & Pets)':'rgba(216,67,21,0.85)',
    };

    function deerColor(lbl) {
        var l = lbl.toLowerCase();
        if (l.includes('resistant') || l.includes('yes')) return 'rgba(76,175,80,0.8)';
        if (l.includes('not') || l.includes('no')) return 'rgba(244,67,54,0.8)';
        if (l.includes('moderate') || l.includes('sometimes')) return 'rgba(255,193,7,0.8)';
        return 'rgba(189,189,189,0.7)';
    }

    function mapColors(labels, map, fallback) {
        return labels.map(function (l, i) { return map[l] || (fallback ? fallback[i % fallback.length] : 'rgba(150,150,150,0.7)'); });
    }

    // Color maps for specific widgets
    var toxColors = {'Non-Toxic':'rgba(76,175,80,0.8)','Non-toxic':'rgba(76,175,80,0.8)','Toxic to Dogs':'rgba(255,152,0,0.8)','Toxic to Cats':'rgba(255,152,0,0.8)','Toxic to Dogs and Cats':'rgba(255,87,34,0.8)','Toxic to Humans':'rgba(244,67,54,0.9)','Toxic to Humans and Pets':'rgba(183,28,28,0.9)','Mildly Toxic':'rgba(255,193,7,0.8)'};
    var rootColors = {'Fibrous':'rgba(139,195,74,0.8)','Taproot':'rgba(255,152,0,0.8)','Specialized':'rgba(156,39,176,0.8)','Rhizomatous':'rgba(0,150,136,0.8)','Bulbous':'rgba(255,193,7,0.8)'};
    var evergreenColors = {'Evergreen':'rgba(27,94,32,0.85)','Semi-evergreen':'rgba(76,175,80,0.75)','Semi-Evergreen':'rgba(76,175,80,0.75)','Deciduous':'rgba(255,183,77,0.75)','Herbaceous':'rgba(174,213,129,0.75)'};
    var growthRateColors = {'Slow':'rgba(33,150,243,0.8)','Moderate':'rgba(76,175,80,0.8)','Fast':'rgba(255,152,0,0.8)'};
    var nativeColors = {'Native':'rgba(76,175,80,0.85)','Cultivar':'rgba(156,39,176,0.75)','Non-native':'rgba(255,152,0,0.75)','Introduced':'rgba(255,87,34,0.75)'};

    // Widget-specific color resolver
    function getColors(key, labels, count) {
        switch (key) {
            case 'sun': return COLORS.sun6.slice(0, count);
            case 'fertilizer': return COLORS.fert8.slice(0, count);
            case 'wildlife': return COLORS.wild8.slice(0, count);
            case 'water': return COLORS.water6.slice(0, count);
            case 'season': return COLORS.season8.slice(0, count);
            case 'grow-season': return COLORS.grow10.slice(0, count);
            case 'deer': return labels.map(deerColor);
            case 'edibility': return mapColors(labels, edibilityColors);
            case 'toxicity': return mapColors(labels, toxColors, COLORS.rainbow12);
            case 'roots': return mapColors(labels, rootColors, COLORS.green6);
            case 'evergreen': return mapColors(labels, evergreenColors, COLORS.green6);
            case 'growth-rate': return mapColors(labels, growthRateColors, COLORS.green6);
            case 'native': return mapColors(labels, nativeColors, COLORS.green6);
            case 'beds': return COLORS.green12.slice(0, count);
            case 'store': return COLORS.rainbow12.slice(0, count);
            case 'top-plants': return COLORS.top12.slice(0, count);
            case 'flower-shapes': return COLORS.rainbow12.slice(0, count);
            default: return null; // single color
        }
    }

    // Render a single widget chart
    function renderWidget(key, w) {
        var canvas = document.getElementById(w.id);
        if (!canvas) return;
        var ctx = canvas.getContext('2d');
        var count = w.labels.length;
        var colors = getColors(key, w.labels, count);

        var datasets, opts;

        // Special: survival (stacked bar with 2 datasets)
        if (key === 'survival') {
            datasets = [
                { label: 'Survived', data: w.alive, backgroundColor: 'rgba(76,175,80,0.8)', borderColor: '#2e7d32', borderWidth: 1, borderRadius: 4 },
                { label: 'Lost', data: w.died, backgroundColor: 'rgba(244,67,54,0.8)', borderColor: '#c62828', borderWidth: 1, borderRadius: 4 },
            ];
            opts = {
                responsive: true, maintainAspectRatio: true,
                plugins: { legend: { display: true } },
                scales: { x: { stacked: true }, y: { stacked: true, beginAtZero: true, ticks: { precision: 0 } } },
            };
            new Chart(ctx, { type: 'bar', data: { labels: w.labels, datasets: datasets }, options: opts });
            if (window.registerWidgetChart) window.registerWidgetChart(w.id, w.labels, w.alive, { label: 'Survived', indexAxis: 'x' });
            return;
        }

        // Special: losses (doughnut + sub bar)
        if (key === 'losses') {
            new Chart(ctx, {
                type: 'doughnut',
                data: { labels: w.labels, datasets: [{ data: w.data, backgroundColor: ['rgba(76,175,80,0.8)', 'rgba(244,67,54,0.8)'], borderWidth: 2 }] },
                options: { responsive: true, maintainAspectRatio: true, plugins: { legend: { position: 'bottom' } } },
            });
            if (window.registerWidgetChart) window.registerWidgetChart('wgt-losses', w.labels, w.data, { label: 'Losses vs Survivors', indexAxis: 'x' });
            // Sub chart
            var sub = document.getElementById('wgt-losses-beds');
            if (sub && w.bedLabels && w.bedLabels.length) {
                new Chart(sub.getContext('2d'), {
                    type: 'bar',
                    data: { labels: w.bedLabels, datasets: [{ label: 'Lost', data: w.bedData, backgroundColor: 'rgba(244,67,54,0.6)', borderRadius: 4 }] },
                    options: { responsive: true, maintainAspectRatio: true, plugins: { legend: { display: false } }, scales: { y: { beginAtZero: true, ticks: { precision: 0 } } } },
                });
                if (window.registerWidgetChart) window.registerWidgetChart('wgt-losses-beds', w.bedLabels, w.bedData, { label: 'Losses by Bed', indexAxis: 'x' });
            }
            return;
        }

        // Special: store (doughnut with varieties tooltip)
        if (key === 'store' && w.varieties) {
            var varieties = w.varieties;
            new Chart(ctx, {
                type: 'doughnut',
                data: { labels: w.labels, datasets: [{ data: w.data, backgroundColor: colors || COLORS.rainbow12.slice(0, count), borderWidth: 2 }] },
                options: {
                    responsive: true, maintainAspectRatio: true,
                    plugins: {
                        legend: { position: 'bottom' },
                        tooltip: { callbacks: { label: function (c) { return c.label + ': ' + c.parsed + ' plants (' + varieties[c.dataIndex] + ' varieties)'; } } },
                    },
                },
            });
            if (window.registerWidgetChart) window.registerWidgetChart(w.id, w.labels, w.data, { label: 'Total Plants', indexAxis: 'x' });
            return;
        }

        // Standard chart
        var bgColor = colors || (w.chart === 'doughnut' ? COLORS.green6.slice(0, count) : 'rgba(76,175,80,0.7)');
        var borderColor = colors ? undefined : '#2e7d32';

        var dsOpts = { label: w.label, data: w.data, backgroundColor: bgColor, borderWidth: 2, borderRadius: 6 };
        if (borderColor) dsOpts.borderColor = borderColor;

        opts = { responsive: true, maintainAspectRatio: true };
        if (w.chart === 'doughnut') {
            opts.plugins = { legend: { position: 'bottom' } };
        } else {
            // ── AXES ────────────────────────────────────────────────────
            // Three things were wrong here, all on this one path.
            //
            // 1. ONLY ONE AXIS WAS EVER CONFIGURED. A horizontal bar set
            //    options for `x` and left `y` -- the axis carrying the
            //    category names -- entirely at defaults. So the 20 flower
            //    beds and 12 plant names, the longest labels on the page,
            //    were the ones with no tick settings at all.
            //
            // 2. NOTHING SAID WHAT THE NUMBERS WERE. `w.label` ("Varieties",
            //    "Qty in Garden", "Total Spent ($)") was passed as the
            //    dataset label and then hidden, because these charts turn the
            //    legend off -- one series needs no legend. The result was a
            //    bar chart of unlabelled numbers. It is an axis title now.
            //
            // 3. TWENTY BARS IN 260 PIXELS. maintainAspectRatio kept every
            //    card the same shape regardless of how many rows it held, so
            //    "Plants per Flower Bed" got about 13px per bed and Chart.js
            //    silently dropped labels to cope. Horizontal charts now grow
            //    with their content; nothing else changes shape.
            opts.plugins = { legend: { display: false } };

            var valueTicks = { beginAtZero: true, precision: 0 };
            if (key === 'investment') {
                valueTicks.callback = function (v) { return '$' + v; };
            }
            var valueAxis = {
                beginAtZero: true,
                ticks: valueTicks,
                // The axis title is the whole point of the fix -- it is the
                // only thing on the card that says what is being counted.
                title: { display: !!w.label, text: w.label || '' }
            };
            // Category names are shown IN FULL in the tooltip, so truncating
            // the tick is safe: the long name is one hover away, and a label
            // Chart.js drops entirely is not.
            var categoryAxis = {
                ticks: {
                    autoSkip: false,
                    callback: function (value) {
                        var raw = this.getLabelForValue(value);
                        var txt = String(raw == null ? '' : raw);
                        return txt.length > 24 ? txt.slice(0, 23) + '…' : txt;
                    }
                }
            };

            if (w.indexAxis === 'y') {
                opts.indexAxis = 'y';
                opts.scales = { x: valueAxis, y: categoryAxis };
                // ~26px a row keeps the bars readable and the labels intact.
                //
                // THE HEIGHT GOES ON THE CONTAINER, NOT THE CANVAS. Setting
                // canvas.style.height looks right and does nothing: with
                // responsive:true Chart.js sizes the canvas FROM its parent
                // on every resize, so it overwrites the value immediately.
                // Measured 2026-08-28 -- asking for 580px on the 20-bed chart
                // and reading back 260px is what gave this away.
                if (count > 8) {
                    opts.maintainAspectRatio = false;
                    var box = canvas.parentNode;
                    if (box) box.style.height = Math.max(260, count * 26 + 60) + 'px';
                    // AND LIFT THE CSS CAP. style.css carries
                    //     .widget-body canvas { max-height: 260px; }
                    // which quietly won over the container height -- the
                    // container really was 580px tall and the canvas inside it
                    // really was still 260. Measured, not guessed: 20 beds at
                    // 13px each. An inline style beats the stylesheet rule, and
                    // doing it here rather than editing the CSS keeps the cap
                    // in place for every chart that is not being grown.
                    canvas.style.maxHeight = 'none';
                }
            } else {
                // Upright bars: keep long names horizontal where they fit and
                // let them lean rather than disappear where they do not.
                categoryAxis.ticks.maxRotation = 45;
                categoryAxis.ticks.minRotation = 0;
                opts.scales = { y: valueAxis, x: categoryAxis };
            }

            opts.plugins.tooltip = {
                callbacks: {
                    // The untruncated category name, since the tick may be cut.
                    title: function (items) {
                        return items.length ? items[0].label : '';
                    }
                }
            };
        }

        // Gradient for investment
        if (key === 'investment') {
            var gradient = ctx.createLinearGradient(0, 0, 0, 400);
            gradient.addColorStop(0, 'rgba(76,175,80,0.9)');
            gradient.addColorStop(1, 'rgba(76,175,80,0.3)');
            dsOpts.backgroundColor = gradient;
        }

        new Chart(ctx, { type: w.chart, data: { labels: w.labels, datasets: [dsOpts] }, options: opts });
        if (window.registerWidgetChart) window.registerWidgetChart(w.id, w.labels, w.data, { label: w.label, indexAxis: w.indexAxis || 'x' });
    }

    // Build widget shell HTML
    function buildShell(key, w) {
        var extra = '';
        if (key === 'losses') {
            extra = '<div style="margin-top:16px;"><h4 style="font-size:0.85rem;color:#c62828;margin:0 0 8px;">Losses by Bed</h4><canvas id="wgt-losses-beds"></canvas></div>';
        }
        return '<div class="widget-card" data-tags="' + (w.tags || '') + '">' +
            '<div class="widget-header"><span class="widget-icon">' + (w.icon || '📊') + '</span><h3>' + w.title + '</h3></div>' +
            '<div class="widget-body"><canvas id="' + w.id + '"></canvas>' + extra + '</div></div>';
    }

    // Main loader
    var widgetGrid = document.getElementById('widget-grid');
    if (!widgetGrid) return;

    // Collect IoT widget wrappers (they stay as server-rendered)
    var iotIds = ['ww-weather-forecast','ww-iot-current','ww-plant-weather','ww-intelligence',
                  'ww-iot-heatgrid','ww-iot-moisture','ww-iot-soil','ww-iot-weather','ww-iot-water-use'];
    var iotNodes = {};
    iotIds.forEach(function (id) {
        var el = document.getElementById(id);
        if (el) { iotNodes[id] = el; el.remove(); }
    });

    fetch('widget_data.php')
        .then(function (r) { return r.json(); })
        .then(function (data) {
            // Update summary counts
            if (data._summary) {
                var hdr = document.querySelector('.stats-header');
                if (hdr) {
                    hdr.innerHTML = '🌿 You have ' + Number(data._summary.plantCount).toLocaleString() +
                        ' plants across ' + Number(data._summary.varietyCount).toLocaleString() +
                        ' varieties — look at you grow!';
                }
            }

            // Known widget order (matches original layout)
            var order = ['growth','sun','families','fertilizer','wildlife','vendors','beds','water',
                         'zones','deer','season','grow-season','edibility','losses','top-plants',
                         'store','investment','toxicity','survival','flower-shapes','roots',
                         'evergreen','growth-rate','native'];

            // Remove existing placeholder wrappers for plant widgets
            order.forEach(function (key) {
                var el = document.getElementById('ww-' + key);
                if (el) el.remove();
            });

            // Insert plant widget shells
            var firstIot = null;
            iotIds.forEach(function (id) { if (!firstIot && iotNodes[id]) firstIot = id; });

            order.forEach(function (key) {
                var w = data[key];
                if (!w || !w.id) return;
                var wrap = document.createElement('div');
                wrap.className = 'widget-wrapper ww-visible';
                wrap.id = 'ww-' + key;
                wrap.setAttribute('data-widget-key', key);
                wrap.innerHTML = buildShell(key, w);
                widgetGrid.appendChild(wrap);
            });

            // Re-append IoT widgets after plant widgets
            iotIds.forEach(function (id) {
                if (iotNodes[id]) widgetGrid.appendChild(iotNodes[id]);
            });

            // Lazy render with IntersectionObserver
            var rendered = {};
            var observer = new IntersectionObserver(function (entries) {
                entries.forEach(function (entry) {
                    if (!entry.isIntersecting) return;
                    var wrap = entry.target;
                    var key = wrap.getAttribute('data-widget-key');
                    if (!key || rendered[key]) return;
                    rendered[key] = true;
                    observer.unobserve(wrap);
                    var w = data[key];
                    if (w) renderWidget(key, w);
                });
            }, { rootMargin: '200px' }); // start rendering 200px before visible

            order.forEach(function (key) {
                var el = document.getElementById('ww-' + key);
                if (el) observer.observe(el);
            });

            // Re-init widget picker if data-widgets.js is loaded
            if (window.initWidgetPicker) window.initWidgetPicker();
        })
        .catch(function (err) {
            console.error('Widget data load failed:', err);
            var hdr = document.querySelector('.stats-header');
            if (hdr) hdr.innerHTML = '⚠️ Widget data failed to load. Refresh to try again.';
        });
})();
