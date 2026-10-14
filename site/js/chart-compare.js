// =============================================================================
// js/chart-compare.js — Cross-tabulation tool for data.php
//
// Views: Heatmap Grid (default) / Year-over-Year % Change
// 3rd dimension filter/slicer supported
// =============================================================================
(function () {
    'use strict';

    // Sorted alphabetically
    const DIMENSIONS = [
        { key: 'bloom_season', label: 'Bloom Season' },
        { key: 'company',      label: 'Brand / Company' },
        { key: 'deer',         label: 'Deer Resistance' },
        { key: 'evergreen',    label: 'Evergreen / Deciduous' },
        { key: 'fertilizer',   label: 'Fertilizer' },
        { key: 'flower_shape', label: 'Flower Shape' },
        { key: 'edibility',    label: 'Fruit Edibility' },
        { key: 'bed',          label: 'Garden Bed' },
        { key: 'grow_season',  label: 'Grow Season' },
        { key: 'growth_rate',  label: 'Growth Rate' },
        { key: 'zones',        label: 'Hardiness Zone' },
        { key: 'invasive',     label: 'Invasive Status' },
        { key: 'store',        label: 'Local Store' },
        { key: 'native',       label: 'Native Status' },
        { key: 'family',       label: 'Plant Family' },
        { key: 'plant_type',   label: 'Plant Type' },
        { key: 'reproduction', label: 'Reproduction Type' },
        { key: 'root_system',  label: 'Root System' },
        { key: 'sun',          label: 'Sun Exposure' },
        { key: 'toxicity',     label: 'Toxicity' },
        { key: 'water',        label: 'Water Needs' },
        { key: 'year',         label: 'Year Planted' },
        { key: 'plant_name',  label: 'Plant Name' },
        { key: 'bloom_color', label: 'Bloom Color' },
        { key: 'foliage_color',label: 'Foliage Color' },
        { key: 'leaf_shape',  label: 'Leaf Shape' },
        { key: 'flower_size', label: 'Flower Size' },
        { key: 'soil_ph',     label: 'Soil pH' },
    ].sort((a, b) => a.label.localeCompare(b.label));

    let lastData = null;

    // Makes any text safe to put inside a block of HTML that is built as a string
    // (innerHTML): & < > " ' become harmless codes, so a label can only ever show
    // as text. It looks the same on screen for every normal value.
    function escHtml(t) {
        return String(t == null ? '' : t).replace(/[&<>"']/g,
            c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
    }

    // WHICH REQUEST IS STILL THE CURRENT ONE.
    //
    // Every selection change fires an async fetch, and nothing used to cancel
    // the one before it. Picking dimension A and then dimension B starts two
    // requests: the first for a SINGLE-dimension chart (B was still empty),
    // the second for the cross-tab. If the first came back last -- which it
    // routinely did, being the smaller query -- it painted its chart over the
    // comparison the user had actually asked for.
    //
    // Reproduced 2026-08-28: setting A then B rendered #single-dim-chart;
    // setting both and firing once rendered the heatmap correctly. That is the
    // whole bug, and it is why the comparison tool "did not work" while every
    // individual piece of it tested fine.
    //
    // Each request takes a ticket on the way out and checks it on the way
    // back. A stale response returns without touching the DOM.
    let reqSeq = 0;
    let weatherChart = null;

    // ══════════════════════════════════════════════════════════════════════════
    // INIT
    // ══════════════════════════════════════════════════════════════════════════
    function init() {
        if (!document.getElementById('compare-tool')) return;
        populateDropdowns();

        ['compare-chart-a', 'compare-chart-b'].forEach(id =>
            document.getElementById(id)?.addEventListener('change', onSelectionChange));
        document.querySelectorAll('input[name="compare-mode"]').forEach(r =>
            r.addEventListener('change', () => renderFromCache()));
        document.querySelectorAll('input[name="compare-scale"]').forEach(r =>
            r.addEventListener('change', () => renderFromCache()));
        document.getElementById('compare-filter-dim')?.addEventListener('change', function(){ onFilterDimChange('compare-filter-dim', 'compare-filter-val'); });
        document.getElementById('compare-filter-val')?.addEventListener('change', onSelectionChange);
        document.getElementById('compare-weather-metric')?.addEventListener('change', onWeatherOverlayChange);
        document.getElementById('compare-weather-year')?.addEventListener('change', onWeatherOverlayChange);

        // AI mode toggle — show/hide weather scenario controls
        const aiToggle = document.getElementById('compare-ai-toggle');
        const aiWeatherControls = document.getElementById('ai-weather-controls');
        if (aiToggle) {
            aiToggle.addEventListener('change', function() {
                if (aiWeatherControls) {
                    aiWeatherControls.style.display = this.checked ? 'flex' : 'none';
                }
                onSelectionChange();
            });
        }
        document.getElementById('ai-temp-scenario')?.addEventListener('change', onSelectionChange);
        document.getElementById('ai-rain-scenario')?.addEventListener('change', onSelectionChange);

        // WAS the "IoT Mock Data (Demo)" checkbox and its five invented
        // sensors. Those elements are gone from data.php (2026-08-28), and the
        // old handler onIotMockChange() below is now unreachable -- left in
        // place rather than deleted because it is the only description of what
        // those charts looked like, for whenever real sensors exist.
        //
        // In its place: ask the model which pairs of columns are worth looking
        // at, instead of requiring you to guess two and hope.
        document.querySelectorAll('input[name="compare-source"]').forEach(function (r) {
            r.addEventListener('change', onCompareSourceChange);
        });
        document.getElementById('compare-filter-dim2')?.addEventListener('change', function(){ onFilterDimChange('compare-filter-dim2', 'compare-filter-val2'); });
        document.getElementById('compare-filter-val2')?.addEventListener('change', onSelectionChange);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // DROPDOWNS
    // ══════════════════════════════════════════════════════════════════════════
    // Year is handled by the filter row (dimension C), so exclude it from A/B
    const MAIN_DIMENSIONS = DIMENSIONS.filter(d => d.key !== 'year');

    function populateDropdowns() {
        const opts = '<option value="">— Pick a dimension —</option>' +
            MAIN_DIMENSIONS.map(d => `<option value="${d.key}">${d.label}</option>`).join('');
        ['compare-chart-a', 'compare-chart-b'].forEach(id => {
            const el = document.getElementById(id);
            if (el) el.innerHTML = opts;
        });
        // Filter dropdowns still include year so users can slice by a specific year
        const filterOpts = '<option value="">— No filter —</option>' +
            DIMENSIONS.map(d => `<option value="${d.key}">${d.label}</option>`).join('');
        ['compare-filter-dim', 'compare-filter-dim2'].forEach(id => {
            const el = document.getElementById(id);
            if (el) el.innerHTML = filterOpts;
        });

        // Default filter 1 to "Year Planted"
        const f1 = document.getElementById('compare-filter-dim');
        if (f1) {
            f1.value = 'year';
            onFilterDimChange('compare-filter-dim', 'compare-filter-val');
        }
    }

    async function onFilterDimChange(dimSelectId, valSelectId) {
        const filterDim = document.getElementById(dimSelectId)?.value;
        const filterVal = document.getElementById(valSelectId);
        if (!filterVal) return;
        if (!filterDim) {
            filterVal.innerHTML = '<option value="">— pick filter first —</option>';
            filterVal.disabled = true;
            onSelectionChange();
            return;
        }
        filterVal.innerHTML = '<option value="">Loading...</option>';
        filterVal.disabled = true;
        try {
            const dimA = document.getElementById('compare-chart-a')?.value || 'year';
            const dimB = document.getElementById('compare-chart-b')?.value || 'sun';
            const res = await fetch(`data.php?cross_tab=1&dim_a=${dimA}&dim_b=${dimB}&dim_filter=${filterDim}&get_filter_values=1&_t=${Date.now()}`);
            const data = await res.json();
            filterVal.innerHTML = '<option value="">— All (no filter) —</option>' +
                (data.values || []).map(v => `<option value="${v}">${v}</option>`).join('');
            filterVal.disabled = false;
        } catch (_) {
            filterVal.innerHTML = '<option value="">Error</option>';
        }
    }

    // ══════════════════════════════════════════════════════════════════════════
    // FETCH + RENDER
    // ══════════════════════════════════════════════════════════════════════════
    async function onSelectionChange() {
        const myReq = ++reqSeq;
        const dimA = document.getElementById('compare-chart-a')?.value;
        const dimB = document.getElementById('compare-chart-b')?.value;
        const output   = document.getElementById('compare-output');
        const hint     = document.getElementById('compare-hint');
        const insights = document.getElementById('compare-insights');

        if (!dimA && !dimB) { output.innerHTML = ''; insights.innerHTML = ''; hint.style.display = 'block'; return; }
        if (dimA && dimB && dimA === dimB) { output.innerHTML = ''; insights.innerHTML = '<p style="color:#c00;text-align:center;">Pick two <em>different</em> dimensions.</p>'; hint.style.display = 'none'; return; }

        hint.style.display = 'none';
        output.innerHTML = '<p style="text-align:center;padding:40px;color:#888;">Loading data...</p>';
        insights.innerHTML = '';

        const filterDim = document.getElementById('compare-filter-dim')?.value || '';
        const filterVal = document.getElementById('compare-filter-val')?.value || '';
        const filterDim2 = document.getElementById('compare-filter-dim2')?.value || '';
        const filterVal2 = document.getElementById('compare-filter-val2')?.value || '';

        // Single dimension mode — just Dim A or just Dim B
        const singleDim = dimA && !dimB ? dimA : (!dimA && dimB ? dimB : null);
        if (singleDim) {
            let sUrl = `data.php?cross_tab=1&dim_a=${encodeURIComponent(singleDim)}&dim_b=${encodeURIComponent(singleDim)}&single=1`;
            if (filterDim && filterVal) sUrl += `&dim_filter=${encodeURIComponent(filterDim)}&filter_val=${encodeURIComponent(filterVal)}`;
            if (filterDim2 && filterVal2) sUrl += `&dim_filter2=${encodeURIComponent(filterDim2)}&filter_val2=${encodeURIComponent(filterVal2)}`;
            sUrl += `&_t=${Date.now()}`;
            try {
                const res = await fetch(sUrl);
                const data = await res.json();
                if (myReq !== reqSeq) return;   // a newer selection is already in flight
                if (data.error) { output.innerHTML = `<p style="color:#c00;text-align:center;">${escHtml(data.error)}</p>`; return; }
                renderSingleDimChart(output, data, singleDim);
                lastData = data;
                // Still trigger weather overlay
                onWeatherOverlayChange();
            } catch (e) {
                if (myReq !== reqSeq) return;
                output.innerHTML = `<p style="color:#c00;">${escHtml(e.message)}</p>`;
            }
            return;
        }

        const useAI      = document.getElementById('compare-ai-toggle')?.checked;
        const aiStatus   = document.getElementById('compare-ai-status');
        const tempScene  = document.getElementById('ai-temp-scenario')?.value || '';
        const rainScene  = document.getElementById('ai-rain-scenario')?.value || '';

        let url;
        if (useAI) {
            url = `data.php?ai_fuse=1&dim_a=${encodeURIComponent(dimA)}&dim_b=${encodeURIComponent(dimB)}`;
            if (filterDim && filterVal) url += `&dim_filter=${encodeURIComponent(filterDim)}&filter_val=${encodeURIComponent(filterVal)}`;
            if (tempScene) url += `&temp_scenario=${encodeURIComponent(tempScene)}`;
            if (rainScene) url += `&rain_scenario=${encodeURIComponent(rainScene)}`;
            if (aiStatus) { aiStatus.style.display = ''; aiStatus.textContent = '⏳ Asking AI...'; }
        } else {
            url = `data.php?cross_tab=1&dim_a=${encodeURIComponent(dimA)}&dim_b=${encodeURIComponent(dimB)}`;
            if (filterDim && filterVal) url += `&dim_filter=${encodeURIComponent(filterDim)}&filter_val=${encodeURIComponent(filterVal)}`;
            if (filterDim2 && filterVal2) url += `&dim_filter2=${encodeURIComponent(filterDim2)}&filter_val2=${encodeURIComponent(filterVal2)}`;
            if (aiStatus) aiStatus.style.display = 'none';
        }
        url += `&_t=${Date.now()}`;

        try {
            const res = await fetch(url);
            const data = await res.json();
            if (myReq !== reqSeq) return;   // a newer selection is already in flight
            if (data.error) { output.innerHTML = `<p style="color:#c00;text-align:center;">${escHtml(data.error)}</p>`; return; }
            const filterParts = [];
            if (filterDim && filterVal) {
                filterParts.push((DIMENSIONS.find(d => d.key === filterDim)?.label || filterDim) + ': ' + filterVal);
            }
            if (filterDim2 && filterVal2) {
                filterParts.push((DIMENSIONS.find(d => d.key === filterDim2)?.label || filterDim2) + ': ' + filterVal2);
            }
            if (filterParts.length) data.filterLabel = filterParts.join(' + ');
            lastData = data;
            renderFromCache();
            // Keep any active weather overlay in sync with the new dimensions.
            if (document.getElementById('compare-weather-metric')?.value) onWeatherOverlayChange();
        } catch (e) {
            if (myReq !== reqSeq) return;
            output.innerHTML = `<p style="color:#c00;text-align:center;">Failed: ${escHtml(e.message)}</p>`;
        }
    }

    function renderFromCache() {
        if (!lastData) return;
        const mode  = document.querySelector('input[name="compare-mode"]:checked')?.value || 'heatmap';
        const scale = document.querySelector('input[name="compare-scale"]:checked')?.value || 'raw';
        const output   = document.getElementById('compare-output');
        const insights = document.getElementById('compare-insights');

        if (mode === 'heatmap') {
            renderHeatmap(output, lastData, scale, document.getElementById('compare-ai-toggle')?.checked);
        } else {
            renderYoY(output, lastData);
        }
        renderInsights(insights, lastData);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // HEATMAP
    // ══════════════════════════════════════════════════════════════════════════
    // Z-score helper
    function calcZScores(values) {
        const nums = values.filter(v => v !== null && v !== 0);
        if (nums.length < 2) return values.map(() => 0);
        const mean = nums.reduce((a,b) => a+b, 0) / nums.length;
        const variance = nums.reduce((a,b) => a + Math.pow(b - mean, 2), 0) / nums.length;
        const stdDev = Math.sqrt(variance) || 1;
        return values.map(v => (v === null || v === 0) ? 0 : Math.round(((v - mean) / stdDev) * 100) / 100);
    }

    function renderHeatmap(container, data, scale, aiMode) {
        const isPct = (scale === 'pct');
        const isZ = (scale === 'zscore');
        let maxVal = 0;
        let minVal = 0;

        // Flatten all values for Z-score calculation
        const allVals = [];
        if (isZ) data.matrix.forEach(row => row.forEach(v => allVals.push(v)));
        const zScores = isZ ? calcZScores(allVals) : [];
        let zIdx = 0;

        const display = data.matrix.map((row, ri) => {
            const rowTotal = data.rowTotals[ri];
            return row.map(v => {
                let val;
                if (isZ) { val = zScores[zIdx++]; }
                else if (isPct) { val = rowTotal > 0 ? Math.round(v / rowTotal * 1000) / 10 : 0; }
                else { val = v; }
                if (val > maxVal) maxVal = val;
                if (val < minVal) minVal = val;
                return val;
            });
        });
        if (maxVal === 0 && !isZ) maxVal = 1;

        function zColor(z) {
            if (z <= -2) return { bg: '#C62828', fg: '#fff' };
            if (z <= -1) return { bg: '#FF8A65', fg: '#333' };
            if (z < -0.5) return { bg: '#FFCC80', fg: '#333' };
            if (z <= 0.5) return { bg: '#f5f5f5', fg: '#333' };
            if (z <= 1) return { bg: '#A5D6A7', fg: '#333' };
            if (z <= 2) return { bg: '#4CAF50', fg: '#fff' };
            return { bg: '#1B5E20', fg: '#fff' };
        }

        const colHeaders = data.colLabels.map(c => `<th class="hm-col-hdr">${c}</th>`).join('');
        const rows = data.rowLabels.map((rowLabel, ri) => {
            const cells = data.colLabels.map((colLabel, ci) => {
                const val = display[ri][ci];
                const raw = data.matrix[ri][ci];

                let bg, fg, label, title;
                if (isZ) {
                    const zc = zColor(val);
                    bg = zc.bg; fg = zc.fg;
                    label = val !== 0 ? (val > 0 ? '+' : '') + val.toFixed(1) + 'σ' : '';
                    title = `${rowLabel} × ${colLabel}: ${raw} plants (z=${val.toFixed(2)})`;
                } else {
                    const intensity = val / (maxVal || 1);
                    bg = val === 0 ? '#f9f9f9' : `rgba(46, 125, 50, ${0.1 + intensity * 0.8})`;
                    fg = intensity > 0.5 ? '#fff' : '#333';
                    label = isPct ? val + '%' : val;
                    title = `${rowLabel} × ${colLabel}: ${raw} plants` + (isPct ? ` (${val}%)` : '');
                }
                return `<td class="hm-cell" style="background:${bg};color:${fg};" title="${title}">${val ? label : ''}</td>`;
            }).join('');
            return `<tr><th class="hm-row-hdr">${rowLabel}</th>${cells}<td class="hm-total">${data.rowTotals[ri]}</td></tr>`;
        }).join('');

        const scaleLabel = isZ ? '(Z-Score: how far from average)' : (isPct ? '(% of row)' : '');
        const legendHtml = isZ ?
            `<span style="background:#C62828;color:#fff;padding:2px 8px;border-radius:4px;">-2σ Far below</span>
             <span style="background:#FFCC80;padding:2px 8px;border-radius:4px;">-1σ Below</span>
             <span style="background:#f5f5f5;border:1px solid #ddd;padding:2px 8px;border-radius:4px;">0 Average</span>
             <span style="background:#A5D6A7;padding:2px 8px;border-radius:4px;">+1σ Above</span>
             <span style="background:#1B5E20;color:#fff;padding:2px 8px;border-radius:4px;">+2σ Far above</span>` :
            `<span style="background:#f9f9f9;border:1px solid #ddd;padding:2px 8px;border-radius:4px;">0</span>
             <span style="background:rgba(46,125,50,0.3);color:#333;padding:2px 8px;border-radius:4px;">Low</span>
             <span style="background:rgba(46,125,50,0.6);color:#fff;padding:2px 8px;border-radius:4px;">Med</span>
             <span style="background:rgba(46,125,50,0.9);color:#fff;padding:2px 8px;border-radius:4px;">High</span>`;

        container.innerHTML = `
        ${aiMode ? `<div style="display:inline-flex;align-items:center;gap:6px;background:#d1fae5;color:#065f46;font-size:0.78rem;font-weight:700;padding:4px 12px;border-radius:10px;margin-bottom:8px;">🧠 AI Predictions — Garden Brain${data.scenario ? ' · ' + data.scenario : ''}</div>` : ''}
        ${data.filterLabel ? `<div style="text-align:center;padding:4px 0 8px;font-size:0.82rem;font-weight:700;color:#1565c0;">Filtered by: ${data.filterLabel}</div>` : ''}
        <div class="hm-wrap">
            <div class="hm-labels">
                <span>${data.dim_a_label} ↓</span>
                <span>${data.dim_b_label} →</span>
                ${scaleLabel ? `<span style="color:#1565c0;">${scaleLabel}</span>` : ''}
            </div>
            <div class="hm-scroll">
                <table class="hm-table">
                    <thead><tr><th></th>${colHeaders}<th class="hm-total-hdr">Total</th></tr></thead>
                    <tbody>${rows}</tbody>
                </table>
            </div>
            <div class="hm-legend">${legendHtml}</div>
        </div>`;
    }

    // ══════════════════════════════════════════════════════════════════════════
    // YEAR-OVER-YEAR % CHANGE
    // Shows how each Dimension B category changed from one Dimension A
    // period to the next (works best when Dim A = Year Planted)
    // ══════════════════════════════════════════════════════════════════════════
    function renderYoY(container, data) {
        if (data.rowLabels.length < 2) {
            container.innerHTML = '<p style="text-align:center;padding:30px;color:#888;">Need at least 2 periods (rows) for year-over-year comparison.</p>';
            return;
        }

        // Sort row labels to ensure chronological order
        const sortedIndices = data.rowLabels
            .map((label, i) => ({ label, i }))
            .sort((a, b) => String(a.label).localeCompare(String(b.label)))
            .map(x => x.i);

        const sortedRows = sortedIndices.map(i => data.rowLabels[i]);
        const sortedMatrix = sortedIndices.map(i => data.matrix[i]);

        // Build period pairs: each row vs previous row
        const periods = [];
        for (let p = 1; p < sortedRows.length; p++) {
            periods.push({
                from: sortedRows[p - 1],
                to: sortedRows[p],
                prevRow: sortedMatrix[p - 1],
                currRow: sortedMatrix[p],
            });
        }

        // Build the table
        const colHeaders = data.colLabels.map(c => `<th class="hm-col-hdr">${c}</th>`).join('');

        const rows = periods.map(period => {
            const cells = data.colLabels.map((_, ci) => {
                const prev = period.prevRow[ci] || 0;
                const curr = period.currRow[ci] || 0;
                const diff = curr - prev;

                let pctText = '';
                let bg = '#f9f9f9';
                let color = '#888';
                let arrow = '';

                if (prev === 0 && curr === 0) {
                    pctText = '—';
                } else if (prev === 0 && curr > 0) {
                    pctText = 'NEW';
                    bg = 'rgba(33, 150, 243, 0.2)';
                    color = '#1565c0';
                    arrow = '🆕';
                } else if (curr === 0 && prev > 0) {
                    pctText = '-100%';
                    bg = 'rgba(244, 67, 54, 0.15)';
                    color = '#c62828';
                    arrow = '↓';
                } else {
                    const pct = Math.round((diff / prev) * 100);
                    if (pct > 0) {
                        const intensity = Math.min(pct / 100, 1);
                        bg = `rgba(46, 125, 50, ${0.08 + intensity * 0.3})`;
                        color = '#1b5e20';
                        arrow = '↑';
                        pctText = '+' + pct + '%';
                    } else if (pct < 0) {
                        const intensity = Math.min(Math.abs(pct) / 100, 1);
                        bg = `rgba(244, 67, 54, ${0.05 + intensity * 0.25})`;
                        color = '#c62828';
                        arrow = '↓';
                        pctText = pct + '%';
                    } else {
                        pctText = '0%';
                        color = '#888';
                        arrow = '→';
                    }
                }

                const title = `${period.from} → ${period.to}: ${prev} → ${curr} (${diff >= 0 ? '+' : ''}${diff})`;
                return `<td class="hm-cell" style="background:${bg};color:${color};font-size:0.75rem;" title="${title}">
                    ${arrow ? `<span style="font-size:0.65rem;">${arrow}</span> ` : ''}${pctText}
                </td>`;
            }).join('');

            // Row totals: overall change
            const prevTotal = period.prevRow.reduce((s, v) => s + v, 0);
            const currTotal = period.currRow.reduce((s, v) => s + v, 0);
            const totalDiff = currTotal - prevTotal;
            const totalPct = prevTotal > 0 ? Math.round((totalDiff / prevTotal) * 100) : (currTotal > 0 ? 100 : 0);
            const totalColor = totalDiff > 0 ? '#1b5e20' : totalDiff < 0 ? '#c62828' : '#888';

            return `<tr>
                <th class="hm-row-hdr" style="white-space:nowrap;">${period.from} → ${period.to}</th>
                ${cells}
                <td class="hm-total" style="color:${totalColor};font-weight:800;">
                    ${totalDiff >= 0 ? '+' : ''}${totalPct}%
                    <div style="font-size:0.65rem;opacity:0.7;">${prevTotal}→${currTotal}</div>
                </td>
            </tr>`;
        }).join('');

        container.innerHTML = `
        ${data.filterLabel ? `<div style="text-align:center;padding:4px 0 8px;font-size:0.82rem;font-weight:700;color:#1565c0;">Filtered by: ${data.filterLabel}</div>` : ''}
        <div class="hm-wrap">
            <div class="hm-labels">
                <span>${data.dim_a_label} Period Changes ↓</span>
                <span>${data.dim_b_label} →</span>
                <span style="color:#1565c0;">(% change period-to-period)</span>
            </div>
            <div class="hm-scroll">
                <table class="hm-table">
                    <thead><tr><th>Period</th>${colHeaders}<th class="hm-total-hdr">Net Change</th></tr></thead>
                    <tbody>${rows}</tbody>
                </table>
            </div>
            <div class="hm-legend" style="margin-top:8px;">
                <span style="background:rgba(46,125,50,0.25);color:#1b5e20;padding:2px 10px;border-radius:4px;">↑ Growth</span>
                <span style="background:#f9f9f9;border:1px solid #ddd;padding:2px 10px;border-radius:4px;">→ No change</span>
                <span style="background:rgba(244,67,54,0.15);color:#c62828;padding:2px 10px;border-radius:4px;">↓ Decline</span>
                <span style="background:rgba(33,150,243,0.2);color:#1565c0;padding:2px 10px;border-radius:4px;">🆕 New</span>
            </div>
        </div>`;
    }

    // ══════════════════════════════════════════════════════════════════════════
    // INSIGHTS
    // ══════════════════════════════════════════════════════════════════════════
    function renderInsights(container, data) {
        const insights = [];

        // ── phi4's reading, when the AI toggle was on ───────────────────
        // Kept visually separate from the computed insights below, and
        // labelled with the model that wrote it, because the two are not the
        // same kind of statement: everything below this block is arithmetic
        // on the matrix and is always true, while this is a language model's
        // interpretation and can be wrong. Blending them into one list would
        // hide which is which.
        let aiHtml = '';
        if (data.ai) {
            if (data.ai.error) {
                aiHtml = `<div class="cmp-ai cmp-ai-off">🧠 ${escHtml(data.ai.error)}</div>`;
            } else {
                const esc = t => String(t).replace(/[&<>]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));
                const bits = [];
                if (data.ai.headline) bits.push(`<p class="cmp-ai-head">${esc(data.ai.headline)}</p>`);
                if (data.ai.findings && data.ai.findings.length) {
                    bits.push('<ul>' + data.ai.findings.map(f => `<li>${esc(f)}</li>`).join('') + '</ul>');
                }
                if (data.ai.scenario_effect) {
                    bits.push(`<p><strong>Under this weather:</strong> ${esc(data.ai.scenario_effect)}</p>`);
                }
                if (data.ai.watch && data.ai.watch.length) {
                    bits.push('<p><strong>Watch:</strong> ' + data.ai.watch.map(esc).join(' · ') + '</p>');
                }
                aiHtml = `<div class="cmp-ai">
                    <div class="cmp-ai-tag">🧠 Read by ${esc(data.ai.model || 'local AI')}${data.scenario ? ' · ' + esc(data.scenario) : ''}</div>
                    ${bits.join('')}
                    <p class="cmp-ai-note">The numbers in the table are counted from the database. This paragraph is the model's interpretation of them.</p>
                </div>`;
            }
        }

        let maxVal = 0, maxRow = '', maxCol = '';
        data.matrix.forEach((row, ri) => {
            row.forEach((v, ci) => {
                if (v > maxVal) { maxVal = v; maxRow = data.rowLabels[ri]; maxCol = data.colLabels[ci]; }
            });
        });
        if (maxVal > 0) insights.push(`<strong>Strongest link:</strong> <em>${escHtml(maxRow)}</em> × <em>${escHtml(maxCol)}</em> → <strong>${maxVal}</strong> plants.`);

        if (data.rowTotals.length > 0) {
            const i = data.rowTotals.indexOf(Math.max(...data.rowTotals));
            insights.push(`<strong>Largest ${escHtml(data.dim_a_label)}:</strong> <em>${escHtml(data.rowLabels[i])}</em> (${data.rowTotals[i]} plants).`);
        }
        if (data.colTotals.length > 0) {
            const i = data.colTotals.indexOf(Math.max(...data.colTotals));
            insights.push(`<strong>Largest ${escHtml(data.dim_b_label)}:</strong> <em>${escHtml(data.colLabels[i])}</em> (${data.colTotals[i]} plants).`);
        }

        // YoY-specific: biggest growth / biggest decline
        if (data.rowLabels.length >= 2) {
            const sorted = data.rowLabels.slice().sort();
            // Compare last two periods
            const lastIdx = data.rowLabels.indexOf(sorted[sorted.length - 1]);
            const prevIdx = data.rowLabels.indexOf(sorted[sorted.length - 2]);
            if (lastIdx >= 0 && prevIdx >= 0) {
                let bigGrowth = { col: '', pct: 0 };
                let bigDecline = { col: '', pct: 0 };
                data.colLabels.forEach((col, ci) => {
                    const prev = data.matrix[prevIdx][ci] || 0;
                    const curr = data.matrix[lastIdx][ci] || 0;
                    if (prev > 0) {
                        const pct = Math.round((curr - prev) / prev * 100);
                        if (pct > bigGrowth.pct) bigGrowth = { col, pct };
                        if (pct < bigDecline.pct) bigDecline = { col, pct };
                    }
                });
                if (bigGrowth.pct > 0) {
                    insights.push(`<strong>Fastest growing</strong> (${escHtml(sorted[sorted.length - 2])}→${escHtml(sorted[sorted.length - 1])}): <em>${escHtml(bigGrowth.col)}</em> (+${bigGrowth.pct}%).`);
                }
                if (bigDecline.pct < 0) {
                    insights.push(`<strong>Biggest decline</strong> (${escHtml(sorted[sorted.length - 2])}→${escHtml(sorted[sorted.length - 1])}): <em>${escHtml(bigDecline.col)}</em> (${bigDecline.pct}%).`);
                }
            }
        }

        // Concentrations
        data.rowLabels.forEach((rowLabel, ri) => {
            const rt = data.rowTotals[ri];
            if (rt < 3) return;
            data.colLabels.forEach((colLabel, ci) => {
                const pct = data.matrix[ri][ci] / rt * 100;
                if (pct >= 70 && data.matrix[ri][ci] >= 3) {
                    insights.push(`<em>${escHtml(rowLabel)}</em> is ${Math.round(pct)}% concentrated in <em>${escHtml(colLabel)}</em>.`);
                }
            });
        });

        if (data.filterLabel) insights.unshift(`<strong>Filter active:</strong> ${escHtml(data.filterLabel)}`);
        // aiHtml alone is enough to render: if the model answered but the
        // matrix was too sparse for a computed insight, the reading should
        // still show rather than the panel going blank.
        if (insights.length === 0 && !aiHtml) { container.innerHTML = ''; return; }

        container.innerHTML = aiHtml + (insights.length ? `
        <div class="compare-insights-box">
            <h4>💡 Data Insights</h4>
            <ul>${insights.slice(0, 8).map(i => `<li>${i}</li>`).join('')}</ul>
        </div>` : '');
    }

    // ══════════════════════════════════════════════════════════════════════════
    // SINGLE DIMENSION — simple bar chart when only one dim is selected
    // ══════════════════════════════════════════════════════════════════════════
    let singleChart = null;

    function renderSingleDimChart(container, data, dimKey) {
        if (singleChart) { singleChart.destroy(); singleChart = null; }

        const dimLabel = DIMENSIONS.find(d => d.key === dimKey)?.label || dimKey;
        const labels = data.rowLabels || [];
        const values = data.rowTotals || [];

        const COLORS = [
            '#4CAF50','#2196F3','#FF9800','#E91E63','#9C27B0','#00BCD4',
            '#FF5722','#607D8B','#8BC34A','#FFC107','#3F51B5','#009688',
            '#F44336','#795548','#CDDC39',
        ];

        container.innerHTML = `
        <div style="padding:8px;">
            <canvas id="single-dim-chart" style="min-height:350px;"></canvas>
        </div>`;

        const canvas = document.getElementById('single-dim-chart');

        singleChart = new Chart(canvas.getContext('2d'), {
            type: 'bar',
            data: {
                labels: labels,
                datasets: [{
                    label: dimLabel,
                    data: values,
                    backgroundColor: values.map((_, i) => COLORS[i % COLORS.length]),
                    borderWidth: 1,
                    borderRadius: 6,
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                indexAxis: labels.length > 8 ? 'y' : 'x',
                plugins: {
                    legend: { display: false },
                    tooltip: {
                        callbacks: {
                            label: function(ctx) { return ctx.raw + ' plants'; }
                        }
                    }
                },
                scales: {
                    [labels.length > 8 ? 'x' : 'y']: {
                        beginAtZero: true,
                        title: { display: true, text: 'Plant Count' },
                        ticks: { precision: 0 },
                    },
                    [labels.length > 8 ? 'y' : 'x']: {
                        title: { display: true, text: dimLabel },
                    },
                },
            },
        });
    }

    // ══════════════════════════════════════════════════════════════════════════
    // WEATHER OVERLAY — fetch InfluxDB data, fuse with plant cross-tab
    // Two views: annotated heatmap or dual-axis bar+line chart
    // ══════════════════════════════════════════════════════════════════════════
    let lastWeatherData = null;

    const METRIC_CONFIG = {
        temperature:   { label: 'Avg Temperature (°F)', color: '#F44336', unit: '°F', short: 'Temp' },
        precipitation: { label: 'Total Precipitation (in)', color: '#2196F3', unit: '"', short: 'Precip' },
        snowfall:      { label: 'Total Snowfall (in)', color: '#7B1FA2', unit: '"', short: 'Snow' },
        humidity:      { label: 'Avg Humidity (%)', color: '#0097A7', unit: '%', short: 'Hum' },
        wind_speed:    { label: 'Avg Wind Speed (mph)', color: '#455A64', unit: 'mph', short: 'Wind' },
        uv_index:      { label: 'Avg UV Index', color: '#FF9800', unit: '', short: 'UV' },
        dew_point:     { label: 'Avg Dew Point (°F)', color: '#607D8B', unit: '°F', short: 'Dew' },
        pressure:      { label: 'Avg Pressure (hPa)', color: '#795548', unit: 'hPa', short: 'Press' },
        soil_temp:     { label: 'Avg Soil Temp (°F)', color: '#5D4037', unit: '°F', short: 'Soil' },
        soil_moisture: { label: 'Avg Soil Moisture', color: '#00695C', unit: '', short: 'SoilM' },
    };

    async function onWeatherOverlayChange() {
        const metric = document.getElementById('compare-weather-metric')?.value;
        const container = document.getElementById('compare-weather-overlay');
        if (!container) return;

        if (!metric) {
            container.innerHTML = '';
            lastWeatherData = null;
            if (weatherChart) { weatherChart.destroy(); weatherChart = null; }
            return;
        }

        container.innerHTML = '<p style="text-align:center;padding:20px;color:#888;">Loading weather data...</p>';

        const dimA = document.getElementById('compare-chart-a')?.value || '';

        // Which year? The dedicated Year dropdown wins; otherwise fall back to a
        // year chosen in either refine filter (keeps old behaviour working).
        let selectedYear = document.getElementById('compare-weather-year')?.value || '';
        if (!selectedYear) {
            const fD1 = document.getElementById('compare-filter-dim')?.value || '';
            const fV1 = document.getElementById('compare-filter-val')?.value || '';
            const fD2 = document.getElementById('compare-filter-dim2')?.value || '';
            const fV2 = document.getElementById('compare-filter-val2')?.value || '';
            if (fD1 === 'year' && fV1.match(/^\d{4}$/)) selectedYear = fV1;
            else if (fD2 === 'year' && fV2.match(/^\d{4}$/)) selectedYear = fV2;
        }

        // Server now always returns a merged monthly series plus yearly /
        // quarterly roll-ups — no need to request a period. Just narrow to a
        // year when one is picked.
        let weatherUrl = `data.php?weather_data=1&metric=${metric}`;
        if (selectedYear) weatherUrl += `&year=${selectedYear}`;
        weatherUrl += `&_t=${Date.now()}`;

        try {
            const res = await fetch(weatherUrl);
            const wData = await res.json();

            if (wData.error) {
                container.innerHTML = `<p style="color:#c00;text-align:center;padding:20px;">${escHtml(wData.error)}</p>`;
                return;
            }
            if (!wData.labels || wData.labels.length === 0) {
                container.innerHTML = '<p style="text-align:center;padding:20px;color:#888;">No weather data available yet.</p>';
                return;
            }

            lastWeatherData = wData;

            // Routing:
            //  • A specific year  → 12-month seasonal wave for that year.
            //  • Dim A = Year + plant data → fused quarterly story chart.
            //  • Otherwise        → standalone / split monthly wave.
            if (selectedYear) {
                renderYearWeatherOverlay(container, wData, metric, selectedYear);
            } else if (dimA === 'year' && lastData) {
                renderFusedView(container, metric);
            } else {
                renderStandaloneWeather(container, wData, metric);
            }
        } catch (e) {
            container.innerHTML = `<p style="color:#c00;text-align:center;">${escHtml(e.message)}</p>`;
        }
    }

    // ── Stats helpers ─────────────────────────────────────────────────────────
    // Pearson correlation between two equal-length numeric arrays (nulls dropped
    // pairwise). Returns null when there isn't enough overlapping data.
    function pearson(xs, ys) {
        const px = [], py = [];
        for (let i = 0; i < xs.length; i++) {
            if (xs[i] === null || ys[i] === null || xs[i] === undefined || ys[i] === undefined) continue;
            if (Number.isNaN(xs[i]) || Number.isNaN(ys[i])) continue;
            px.push(+xs[i]); py.push(+ys[i]);
        }
        const n = px.length;
        if (n < 3) return null;
        const mx = px.reduce((a, b) => a + b, 0) / n;
        const my = py.reduce((a, b) => a + b, 0) / n;
        let num = 0, dx = 0, dy = 0;
        for (let i = 0; i < n; i++) {
            const a = px[i] - mx, b = py[i] - my;
            num += a * b; dx += a * a; dy += b * b;
        }
        const den = Math.sqrt(dx * dy);
        return den === 0 ? null : Math.round((num / den) * 100) / 100;
    }

    // Plain-English strength + direction for a correlation coefficient.
    function describeR(r) {
        if (r === null) return { txt: 'not enough overlapping years to correlate', color: '#888' };
        const a = Math.abs(r);
        const strength = a >= 0.7 ? 'strong' : a >= 0.4 ? 'moderate' : a >= 0.2 ? 'weak' : 'little-to-no';
        const dir = r > 0 ? 'positive' : 'negative';
        const color = a >= 0.4 ? (r > 0 ? '#1b5e20' : '#c62828') : '#666';
        return { txt: `${strength} ${dir} link (r = ${r > 0 ? '+' : ''}${r})`, color };
    }

    // Least-squares slope of y over an index 0..n-1 (per-step trend).
    function trendSlope(ys) {
        const pts = ys.map((y, i) => [i, y]).filter(p => p[1] !== null && !Number.isNaN(p[1]));
        const n = pts.length;
        if (n < 2) return null;
        const mx = pts.reduce((a, p) => a + p[0], 0) / n;
        const my = pts.reduce((a, p) => a + p[1], 0) / n;
        let num = 0, den = 0;
        pts.forEach(p => { num += (p[0] - mx) * (p[1] - my); den += (p[0] - mx) ** 2; });
        return den === 0 ? null : num / den;
    }

    // Full least-squares trend line evaluated at every index (for overlaying a
    // straight regression line on the monthly series). Returns null if too few
    // points.
    function linTrendLine(ys) {
        const pts = ys.map((y, i) => [i, y]).filter(p => p[1] !== null && !Number.isNaN(p[1]));
        const n = pts.length;
        if (n < 2) return null;
        const mx = pts.reduce((a, p) => a + p[0], 0) / n;
        const my = pts.reduce((a, p) => a + p[1], 0) / n;
        let num = 0, den = 0;
        pts.forEach(p => { num += (p[0] - mx) * (p[1] - my); den += (p[0] - mx) ** 2; });
        if (den === 0) return null;
        const slope = num / den, intercept = my - slope * mx;
        return ys.map((_, i) => Math.round((slope * i + intercept) * 100) / 100);
    }

    // A "nice" round tick step for a value range (1/2/5 × 10ⁿ).
    function niceStep(range) {
        if (!(range > 0)) return 1;
        const raw = range / 5;
        const mag = Math.pow(10, Math.floor(Math.log10(raw)));
        const norm = raw / mag;
        const s = norm < 1.5 ? 1 : norm < 3 ? 2 : norm < 7 ? 5 : 10;
        return s * mag;
    }

    // Frame an axis tightly around the data: modest padding, then snap min/max
    // out to round multiples of a nice step so the line fills the panel and the
    // ticks read cleanly. noZero=false anchors the floor at 0.
    function niceBounds(vals, noZero) {
        const clean = (vals || []).filter(v => v !== null && v !== undefined && !Number.isNaN(v));
        if (!clean.length) return { min: 0, max: 100, step: 20 };
        let lo = Math.min(...clean), hi = Math.max(...clean);
        if (lo === hi) { lo -= 1; hi += 1; }
        const pad = Math.max((hi - lo) * 0.12, 1);
        let floor = noZero ? lo - pad : 0;
        let ceil = hi + pad;
        const step = niceStep(ceil - floor);
        floor = Math.floor(floor / step) * step;
        ceil = Math.ceil(ceil / step) * step;
        return { min: floor, max: ceil, step };
    }

    let splitChartLeft = null;
    let splitChartRight = null;
    let yearOverlayChart = null;

    function renderYearWeatherOverlay(container, wData, metric, year) {
        if (yearOverlayChart) { yearOverlayChart.destroy(); yearOverlayChart = null; }
        if (weatherChart) { weatherChart.destroy(); weatherChart = null; }

        const cfg = METRIC_CONFIG[metric] || { label: metric, color: '#333', unit: '', short: metric };
        const noBeginZero = ['temperature', 'pressure', 'dew_point', 'soil_temp'].includes(metric);
        const vals = wData.data.filter(v => v !== null);
        const nb = niceBounds(vals, noBeginZero);

        // Determine highlights for gardening context
        let annotations = [];
        if (metric === 'temperature') {
            const frostMonths = [];
            const heatMonths = [];
            const idealMonths = [];
            wData.data.forEach((v, i) => {
                if (v !== null) {
                    if (v < 32) frostMonths.push(wData.labels[i]);
                    else if (v > 85) heatMonths.push(wData.labels[i]);
                    else if (v >= 55 && v <= 80) idealMonths.push(wData.labels[i]);
                }
            });
            if (frostMonths.length) annotations.push('❄️ Frost risk: <strong>' + escHtml(frostMonths.join(', ')) + '</strong>');
            if (heatMonths.length) annotations.push('🔥 Heat stress: <strong>' + escHtml(heatMonths.join(', ')) + '</strong>');
            if (idealMonths.length) annotations.push('🌱 Ideal growing: <strong>' + escHtml(idealMonths.join(', ')) + '</strong>');
        } else if (metric === 'precipitation') {
            const totalRain = vals.reduce((a,b) => a+b, 0).toFixed(1);
            annotations.push('🌧️ Total precipitation: <strong>' + escHtml(totalRain) + '"</strong> (Clifton Park avg: ~35"/year)');
            const dryMonths = [];
            wData.data.forEach((v, i) => { if (v !== null && v < 1) dryMonths.push(wData.labels[i]); });
            if (dryMonths.length) annotations.push('🏜️ Dry months (< 1"): <strong>' + escHtml(dryMonths.join(', ')) + '</strong>');
        }

        container.innerHTML = `
        <div style="background:#fff;border:2px solid #222;border-radius:12px;box-shadow:3px 3px 0px rgba(0,0,0,0.4);padding:16px;margin:0 16px 16px;">
            <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;flex-wrap:wrap;">
                <span style="font-size:1.1rem;">📅</span>
                <strong style="font-size:0.95rem;color:#333;">${escHtml(year)} Weather: ${escHtml(cfg.label)}</strong>
                <span style="font-size:0.78rem;color:#888;">monthly · ${(wData.coverage && wData.coverage.noaa_fill_months) ? 'NOAA station' : 'live Open-Meteo'}</span>
            </div>
            ${wData.stats ? `<div style="display:flex;flex-wrap:wrap;gap:8px;font-size:0.76rem;margin-bottom:8px;">
                <span style="background:#f5f5f5;border-radius:6px;padding:3px 10px;">avg <strong>${escHtml(wData.stats.mean ?? '–')}${escHtml(cfg.unit)}</strong></span>
                <span style="background:#e3f2fd;border-radius:6px;padding:3px 10px;color:#1565c0;">low <strong>${escHtml(wData.stats.min ?? '–')}${escHtml(cfg.unit)}</strong></span>
                <span style="background:#fff3e0;border-radius:6px;padding:3px 10px;color:#e65100;">high <strong>${escHtml(wData.stats.max ?? '–')}${escHtml(cfg.unit)}</strong></span>
            </div>` : ''}
            ${annotations.length ? `<div style="font-size:0.82rem;color:#333;line-height:1.7;margin-bottom:10px;padding:8px 12px;background:#f9fbe7;border-radius:8px;border-left:4px solid #4CAF50;">
                ${annotations.join('<br>')}
            </div>` : ''}
            <canvas id="year-weather-overlay" style="max-height:280px;"></canvas>
        </div>`;

        const canvas = document.getElementById('year-weather-overlay');

        // Background bands for temperature
        const bgPlugin = metric === 'temperature' ? {
            id: 'tempBands',
            beforeDraw: function(chart) {
                const ctx = chart.ctx;
                const yScale = chart.scales.y;
                const xScale = chart.scales.x;

                // Frost zone (below 32)
                const y32 = yScale.getPixelForValue(32);
                const yBottom = yScale.getPixelForValue(yScale.min);
                if (y32 > yScale.top) {
                    ctx.fillStyle = 'rgba(33,150,243,0.08)';
                    ctx.fillRect(xScale.left, y32, xScale.width, yBottom - y32);
                }

                // Heat zone (above 85)
                const y85 = yScale.getPixelForValue(85);
                const yTop = yScale.getPixelForValue(yScale.max);
                if (y85 < yScale.bottom) {
                    ctx.fillStyle = 'rgba(244,67,54,0.08)';
                    ctx.fillRect(xScale.left, yTop, xScale.width, y85 - yTop);
                }

                // Ideal zone (55-80)
                const y55 = yScale.getPixelForValue(55);
                const y80 = yScale.getPixelForValue(80);
                ctx.fillStyle = 'rgba(76,175,80,0.08)';
                ctx.fillRect(xScale.left, y80, xScale.width, y55 - y80);
            }
        } : null;

        yearOverlayChart = new Chart(canvas.getContext('2d'), {
            type: 'line',
            data: {
                labels: wData.labels,
                datasets: [{
                    label: cfg.label,
                    data: wData.data,
                    borderColor: cfg.color,
                    backgroundColor: cfg.color + '15',
                    borderWidth: 3, pointRadius: 5,
                    pointBackgroundColor: wData.data.map(v => {
                        if (metric !== 'temperature' || v === null) return cfg.color;
                        if (v < 32) return '#2196F3';
                        if (v > 85) return '#F44336';
                        if (v >= 55 && v <= 80) return '#4CAF50';
                        return cfg.color;
                    }),
                    fill: true, tension: 0.25, cubicInterpolationMode: 'monotone',
                }]
            },
            plugins: bgPlugin ? [bgPlugin] : [],
            options: {
                responsive: true, maintainAspectRatio: true,
                plugins: {
                    legend: { display: false },
                    tooltip: {
                        callbacks: {
                            label: function(ctx) {
                                let label = ctx.raw + cfg.unit;
                                if (metric === 'temperature') {
                                    if (ctx.raw < 32) label += ' ❄️ FROST';
                                    else if (ctx.raw > 85) label += ' 🔥 HEAT STRESS';
                                    else if (ctx.raw >= 55 && ctx.raw <= 80) label += ' 🌱 IDEAL';
                                }
                                return label;
                            }
                        }
                    }
                },
                scales: {
                    y: {
                        min: nb.min,
                        max: nb.max,
                        title: { display: true, text: cfg.label },
                        ticks: { stepSize: nb.step, callback: v => v + cfg.unit },
                    },
                    x: { ticks: { maxRotation: 45 } },
                },
            },
        });
    }

    function renderStandaloneWeather(container, wData, metric) {
        if (weatherChart) { weatherChart.destroy(); weatherChart = null; }
        if (splitChartLeft) { splitChartLeft.destroy(); splitChartLeft = null; }
        if (splitChartRight) { splitChartRight.destroy(); splitChartRight = null; }

        const cfg = METRIC_CONFIG[metric] || { label: metric, color: '#333', unit: '', short: metric };
        const pData = lastData;

        // If we have plant data, show side-by-side split view
        if (pData && pData.rowLabels && pData.rowLabels.length > 0) {
            renderSplitView(container, pData, wData, cfg, metric);
            return;
        }

        // No plant data yet — full multi-year monthly wave with a trend line.
        const noBeginZero = ['temperature', 'pressure', 'dew_point', 'soil_temp'].includes(metric);
        const series = wData.monthly?.data || wData.data;
        const labels = wData.monthly?.labels || wData.labels;
        const vals = series.filter(v => v !== null);
        const nb = niceBounds(vals, noBeginZero);

        const trend = linTrendLine(series);
        const slope = trendSlope(series);
        // 12 months per step → ×12 to express change per year.
        const perYear = slope === null ? null : Math.round(slope * 12 * 100) / 100;
        const span = labels.length ? `${labels[0]} – ${labels[labels.length - 1]}` : '';
        const cov = wData.coverage || {};
        const covTxt = cov.noaa_fill_months
            ? `${cov.om_months} mo live Open-Meteo + ${cov.noaa_fill_months} mo NOAA history`
            : `${cov.total_months || labels.length} months`;
        const st = wData.stats || {};

        container.innerHTML = `
        <div style="background:#fff;border:2px solid #222;border-radius:12px;box-shadow:3px 3px 0px rgba(0,0,0,0.4);padding:16px;margin:0 16px 16px;">
            <div style="display:flex;align-items:center;gap:8px;margin-bottom:6px;flex-wrap:wrap;">
                <span style="font-size:1.1rem;">🌦️</span>
                <strong style="font-size:0.95rem;color:#333;">${escHtml(cfg.label)}</strong>
                <span style="font-size:0.78rem;color:#888;">${escHtml(span)}</span>
            </div>
            <div style="display:flex;flex-wrap:wrap;gap:8px;font-size:0.76rem;margin-bottom:10px;">
                <span style="background:#f5f5f5;border-radius:6px;padding:3px 10px;">avg <strong>${escHtml(st.mean ?? '–')}${escHtml(cfg.unit)}</strong></span>
                <span style="background:#e3f2fd;border-radius:6px;padding:3px 10px;color:#1565c0;">low <strong>${escHtml(st.min ?? '–')}${escHtml(cfg.unit)}</strong></span>
                <span style="background:#fff3e0;border-radius:6px;padding:3px 10px;color:#e65100;">high <strong>${escHtml(st.max ?? '–')}${escHtml(cfg.unit)}</strong></span>
                ${perYear !== null ? `<span style="background:#ede7f6;border-radius:6px;padding:3px 10px;color:#5e35b1;">trend <strong>${perYear >= 0 ? '▲' : '▼'} ${escHtml(Math.abs(perYear))}${escHtml(cfg.unit)}/yr</strong></span>` : ''}
                <span style="background:#e8f5e9;border-radius:6px;padding:3px 10px;color:#2e7d32;">🗄️ ${escHtml(covTxt)}</span>
            </div>
            <canvas id="standalone-weather-chart" style="max-height:300px;"></canvas>
        </div>`;

        const bandPlugin = metric === 'temperature' ? {
            id: 'standaloneTempBands',
            beforeDatasetsDraw(chart) {
                const ys = chart.scales.y, xa = chart.chartArea;
                if (!ys || !xa) return;
                const ctx = chart.ctx;
                const band = (lo, hi, color) => {
                    const yHi = ys.getPixelForValue(Math.min(hi, ys.max));
                    const yLo = ys.getPixelForValue(Math.max(lo, ys.min));
                    ctx.fillStyle = color;
                    ctx.fillRect(xa.left, yHi, xa.right - xa.left, yLo - yHi);
                };
                ctx.save();
                band(ys.min, 32, 'rgba(33,150,243,0.06)');
                band(55, 80, 'rgba(76,175,80,0.08)');
                band(85, ys.max, 'rgba(244,67,54,0.06)');
                ctx.restore();
            }
        } : null;

        const datasets = [{
            label: cfg.label, data: series,
            borderColor: cfg.color, backgroundColor: cfg.color + '20',
            borderWidth: 2.5, pointRadius: labels.length > 30 ? 0 : 3,
            fill: true, tension: 0.25, cubicInterpolationMode: 'monotone',
        }];
        if (trend) datasets.push({
            label: 'Trend', data: trend, type: 'line',
            borderColor: '#5e35b1', borderWidth: 2, borderDash: [6, 4],
            pointRadius: 0, fill: false, tension: 0,
        });

        weatherChart = new Chart(document.getElementById('standalone-weather-chart').getContext('2d'), {
            type: 'line',
            data: { labels: labels, datasets: datasets },
            plugins: bandPlugin ? [bandPlugin] : [],
            options: {
                responsive: true, maintainAspectRatio: true,
                plugins: { legend: { display: trend ? true : false, position: 'top', labels: { boxWidth: 12, font: { size: 11 } } } },
                interaction: { mode: 'index', intersect: false },
                scales: {
                    y: { min: nb.min, max: nb.max,
                         title: { display: true, text: cfg.label },
                         ticks: { stepSize: nb.step, callback: v => v + cfg.unit } },
                    x: { ticks: { maxRotation: 45, autoSkip: true, maxTicksLimit: 16 } },
                },
            },
        });
    }

    function renderSplitView(container, pData, wData, cfg, metric) {
        if (splitChartLeft) { splitChartLeft.destroy(); splitChartLeft = null; }
        if (splitChartRight) { splitChartRight.destroy(); splitChartRight = null; }

        const noBeginZero = ['temperature', 'pressure', 'dew_point', 'soil_temp'].includes(metric);
        const nb = niceBounds((wData.monthly?.data || wData.data).filter(v => v !== null), noBeginZero);

        // Plant chart colors
        const COLORS = ['#4CAF50','#2196F3','#FF9800','#E91E63','#9C27B0','#00BCD4','#FF5722','#8BC34A'];

        container.innerHTML = `
        <div style="background:#fff;border:2px solid #222;border-radius:12px;box-shadow:3px 3px 0px rgba(0,0,0,0.4);padding:16px;margin:0 16px 16px;">
            <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;flex-wrap:wrap;">
                <span style="font-size:1rem;">📊</span>
                <strong style="font-size:0.9rem;color:#333;">Side-by-Side: ${escHtml(pData.dim_a_label)} vs ${escHtml(cfg.label)}</strong>
                <span style="font-size:0.75rem;color:#888;">scroll both charts on the same time axis</span>
            </div>
            <div style="font-size:0.75rem;color:#2E7D32;margin-bottom:12px;font-weight:600;">
                💡 Compare the plant chart (left) with weather (right) — look for patterns where spikes or dips align.
            </div>
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
                <div style="border:1px solid #e0e0e0;border-radius:8px;padding:10px;background:#fafafa;">
                    <div style="font-size:0.8rem;font-weight:700;color:#2E7D32;margin-bottom:6px;text-align:center;">🌿 ${escHtml(pData.dim_a_label)} × ${escHtml(pData.dim_b_label)}</div>
                    <canvas id="split-plant-chart"></canvas>
                </div>
                <div style="border:1px solid #e0e0e0;border-radius:8px;padding:10px;background:#fafafa;">
                    <div style="font-size:0.8rem;font-weight:700;color:${escHtml(cfg.color)};margin-bottom:6px;text-align:center;">🌦️ ${escHtml(cfg.label)}</div>
                    <canvas id="split-weather-chart"></canvas>
                </div>
            </div>
        </div>`;

        // Left: plant data as stacked bar
        const topCols = pData.colLabels.slice(0, 6);
        const plantDatasets = topCols.map((col, ci) => ({
            label: col,
            data: pData.rowLabels.map((_, ri) => pData.matrix[ri]?.[ci] || 0),
            backgroundColor: COLORS[ci % COLORS.length] + 'BB',
            borderRadius: 3,
        }));

        splitChartLeft = new Chart(document.getElementById('split-plant-chart').getContext('2d'), {
            type: 'bar',
            data: { labels: pData.rowLabels, datasets: plantDatasets },
            options: {
                responsive: true, maintainAspectRatio: true,
                plugins: { legend: { display: true, position: 'bottom', labels: { boxWidth: 10, font: { size: 10 } } } },
                scales: {
                    x: { stacked: true, ticks: { maxRotation: 45 } },
                    y: { stacked: true, beginAtZero: true, title: { display: true, text: 'Plants' } },
                },
            },
        });

        // Right: weather line (full merged monthly series)
        const wLabels = wData.monthly?.labels || wData.labels;
        const wSeries = wData.monthly?.data || wData.data;
        splitChartRight = new Chart(document.getElementById('split-weather-chart').getContext('2d'), {
            type: 'line',
            data: { labels: wLabels, datasets: [{
                label: cfg.label, data: wSeries,
                borderColor: cfg.color, backgroundColor: cfg.color + '15',
                borderWidth: 2.5, pointRadius: wLabels.length > 30 ? 0 : 4, pointBackgroundColor: cfg.color,
                fill: true, tension: 0.25, cubicInterpolationMode: 'monotone',
            }]},
            options: {
                responsive: true, maintainAspectRatio: true,
                plugins: { legend: { display: false } },
                interaction: { mode: 'index', intersect: false },
                scales: {
                    y: { min: nb.min, max: nb.max,
                         title: { display: true, text: cfg.label },
                         ticks: { stepSize: nb.step, callback: v => v + cfg.unit } },
                    x: { ticks: { maxRotation: 45, autoSkip: true, maxTicksLimit: 14 } },
                },
            },
        });
    }

    function renderFusedView(container, metric) {
        if (weatherChart) { weatherChart.destroy(); weatherChart = null; }

        const cfg = METRIC_CONFIG[metric] || { label: metric, color: '#333', unit: '', short: metric };
        const wData = lastWeatherData;
        const pData = lastData; // plant cross-tab data

        // Build weather lookup: label → value
        const weatherMap = {};
        if (wData) {
            wData.labels.forEach((l, i) => { weatherMap[l] = wData.data[i]; });
            // Also map by year number for matching
            wData.labels.forEach((l, i) => {
                const yr = l.match(/\d{4}/);
                if (yr) weatherMap[yr[0]] = wData.data[i];
            });
        }

        // Current view mode — default to the quarterly story chart.
        const viewMode = container.querySelector('input[name="fused-view"]:checked')?.value || 'chart';

        container.innerHTML = `
        <div style="background:#fff;border:2px solid #222;border-radius:12px;box-shadow:3px 3px 0px rgba(0,0,0,0.4);padding:16px;margin:0 16px 16px;">
            <div style="display:flex;align-items:center;gap:10px;margin-bottom:12px;flex-wrap:wrap;">
                <span style="font-size:1.1rem;">🌦️</span>
                <strong style="font-size:0.95rem;color:#333;">Fused View: Plants + ${escHtml(cfg.label)}</strong>
                <span style="font-size:0.78rem;color:#888;">from InfluxDB</span>
                <div style="margin-left:auto;display:flex;gap:10px;">
                    <label style="font-size:0.82rem;font-weight:600;cursor:pointer;display:flex;align-items:center;gap:4px;">
                        <input type="radio" name="fused-view" value="chord" ${viewMode==='chord'?'checked':''} style="accent-color:#2e7d32;"> Chord Diagram
                    </label>
                    <label style="font-size:0.82rem;font-weight:600;cursor:pointer;display:flex;align-items:center;gap:4px;">
                        <input type="radio" name="fused-view" value="chart" ${viewMode==='chart'?'checked':''} style="accent-color:#2e7d32;"> Quarterly Story
                    </label>
                </div>
            </div>
            <div id="fused-content"></div>
        </div>`;

        // Wire toggle
        container.querySelectorAll('input[name="fused-view"]').forEach(r => {
            r.addEventListener('change', () => renderFusedContent(metric));
        });

        renderFusedContent(metric);
    }

    function renderFusedContent(metric) {
        const container = document.getElementById('fused-content');
        if (!container) return;

        const viewMode = document.querySelector('input[name="fused-view"]:checked')?.value || 'chart';
        const cfg = METRIC_CONFIG[metric] || { label: metric, color: '#333', unit: '', short: metric };
        const wData = lastWeatherData;
        const pData = lastData;

        if (viewMode === 'chord') {
            renderChordDiagram(container, pData, wData, cfg);
        } else {
            renderFusedChart(container, pData, wData, cfg);
        }
    }

    // Chord diagram: shows relationships between Dim A rows and Dim B columns
    function renderChordDiagram(container, pData, wData, cfg) {
        if (!pData) {
            container.innerHTML = '<p style="text-align:center;color:#888;">Select both plant dimensions to see the chord diagram.</p>';
            return;
        }

        const rows = pData.rowLabels.slice(0, 8);
        const cols = pData.colLabels.slice(0, 8);
        const n = rows.length + cols.length;
        if (n < 3) {
            container.innerHTML = '<p style="text-align:center;color:#888;">Not enough data for chord diagram.</p>';
            return;
        }

        // Build names array: rows first, then cols
        const names = [...rows, ...cols];

        // Build n×n matrix (symmetric-ish for chord layout)
        const matrix = Array.from({length: n}, () => Array(n).fill(0));
        for (let ri = 0; ri < rows.length; ri++) {
            for (let ci = 0; ci < cols.length; ci++) {
                const origRi = pData.rowLabels.indexOf(rows[ri]);
                const origCi = pData.colLabels.indexOf(cols[ci]);
                const val = (origRi >= 0 && origCi >= 0) ? (pData.matrix[origRi]?.[origCi] || 0) : 0;
                matrix[ri][rows.length + ci] = val;
                matrix[rows.length + ci][ri] = val;
            }
        }

        const width = 500;
        const height = 500;
        const innerRadius = Math.min(width, height) * 0.35;
        const outerRadius = innerRadius + 14;

        const CHORD_COLORS = [
            '#4CAF50','#2196F3','#FF9800','#E91E63','#9C27B0','#00BCD4',
            '#FF5722','#607D8B','#8BC34A','#FFC107','#3F51B5','#009688',
            '#F44336','#795548','#CDDC39','#673AB7',
        ];

        container.innerHTML = `<div id="chord-svg-wrap" style="display:flex;justify-content:center;overflow:hidden;"></div>
            <div style="display:flex;flex-wrap:wrap;gap:6px;justify-content:center;margin-top:10px;font-size:0.78rem;">
                ${names.map((name, i) => `<span style="display:inline-flex;align-items:center;gap:4px;">
                    <span style="width:12px;height:12px;border-radius:3px;background:${CHORD_COLORS[i % CHORD_COLORS.length]};display:inline-block;"></span>
                    <span style="font-weight:600;color:#333;">${name}</span>
                </span>`).join('')}
            </div>
            <div style="text-align:center;margin-top:8px;font-size:0.78rem;color:#888;">
                Ribbons show plant count flowing between <strong>${pData.dim_a_label}</strong> and <strong>${pData.dim_b_label}</strong>. Thicker = more plants.
            </div>`;

        const svg = d3.select('#chord-svg-wrap')
            .append('svg')
            .attr('viewBox', `0 0 ${width} ${height}`)
            .attr('width', '100%')
            .attr('style', 'max-width:500px;')
            .append('g')
            .attr('transform', `translate(${width/2},${height/2})`);

        const chord = d3.chord().padAngle(0.04).sortSubgroups(d3.descending)(matrix);

        // Arcs
        const arc = d3.arc().innerRadius(innerRadius).outerRadius(outerRadius);
        svg.append('g')
            .selectAll('path')
            .data(chord.groups)
            .join('path')
            .attr('d', arc)
            .attr('fill', d => CHORD_COLORS[d.index % CHORD_COLORS.length])
            .attr('stroke', '#fff')
            .attr('stroke-width', 1.5)
            .append('title')
            .text(d => `${names[d.index]}: ${d.value} plants`);

        // Labels
        svg.append('g')
            .selectAll('text')
            .data(chord.groups)
            .join('text')
            .each(d => { d.angle = (d.startAngle + d.endAngle) / 2; })
            .attr('dy', '0.35em')
            .attr('transform', d => `
                rotate(${(d.angle * 180 / Math.PI - 90)})
                translate(${outerRadius + 8})
                ${d.angle > Math.PI ? 'rotate(180)' : ''}
            `)
            .attr('text-anchor', d => d.angle > Math.PI ? 'end' : 'start')
            .attr('font-size', '11px')
            .attr('font-weight', '700')
            .attr('fill', '#333')
            .text(d => names[d.index].length > 15 ? names[d.index].substring(0,14) + '…' : names[d.index]);

        // Ribbons
        const ribbon = d3.ribbon().radius(innerRadius);
        svg.append('g')
            .attr('fill-opacity', 0.6)
            .selectAll('path')
            .data(chord)
            .join('path')
            .attr('d', ribbon)
            .attr('fill', d => CHORD_COLORS[d.source.index % CHORD_COLORS.length])
            .attr('stroke', '#fff')
            .attr('stroke-width', 0.5)
            .append('title')
            .text(d => `${names[d.source.index]} → ${names[d.target.index]}: ${d.source.value} plants`);
    }

    // Dual-axis "story" chart: plant counts as stacked bars per year +
    // quarterly weather lines (Q1–Q4) so the seasonal swing is visible and the
    // relationship between climate and plantings reads at a glance.
    function renderFusedChart(container, pData, wData, cfg) {
        if (weatherChart) { weatherChart.destroy(); weatherChart = null; }

        if (!pData || !wData) {
            container.innerHTML = '<p style="text-align:center;color:#888;">Select both plant dimensions and a weather metric.</p>';
            return;
        }

        // Order the year axis chronologically (matrix may arrive newest-first).
        const order = pData.rowLabels
            .map((y, i) => i)
            .sort((a, b) => (parseInt(pData.rowLabels[a]) || 0) - (parseInt(pData.rowLabels[b]) || 0));
        const years = order.map(i => pData.rowLabels[i]);

        // Plant bars — top 8 Dim-B segments, stacked, per year.
        const topCols = pData.colLabels.slice(0, 8);
        const COLORS = ['#4CAF50','#2196F3','#FF9800','#E91E63','#9C27B0','#00BCD4','#FF5722','#8BC34A'];
        const barDatasets = topCols.map((col, ci) => ({
            label: col,
            data: order.map(i => (pData.matrix[i] && pData.matrix[i][ci]) || 0),
            backgroundColor: COLORS[ci % COLORS.length] + 'C0',
            borderWidth: 1,
            borderRadius: 3,
            stack: 'plants',
            yAxisID: 'y',
        }));

        // Quarterly weather lookup: { 2024: {1:v,2:v,3:v,4:v}, … }
        const qmap = {};
        const q = wData.quarterly || { year: [], q: [], data: [] };
        (q.year || []).forEach((yr, i) => { (qmap[yr] = qmap[yr] || {})[q.q[i]] = q.data[i]; });

        const QSEASON = [
            { q: 1, label: 'Q1 Jan–Mar', color: '#42A5F5', dash: [5, 3] },
            { q: 2, label: 'Q2 Apr–Jun', color: '#66BB6A', dash: [] },
            { q: 3, label: 'Q3 Jul–Sep', color: '#EF5350', dash: [] },
            { q: 4, label: 'Q4 Oct–Dec', color: '#FB8C00', dash: [5, 3] },
        ];
        const quarterLines = QSEASON.map(s => ({
            label: s.label,
            data: years.map(y => { const yy = qmap[parseInt(y)]; return yy && yy[s.q] != null ? yy[s.q] : null; }),
            type: 'line',
            borderColor: s.color,
            backgroundColor: s.color + '22',
            borderWidth: 2.5,
            borderDash: s.dash,
            pointRadius: 4,
            pointBackgroundColor: s.color,
            spanGaps: true,
            tension: 0.25,
            cubicInterpolationMode: 'monotone',
            fill: false,
            yAxisID: 'y1',
        }));

        // Weather axis range across every quarterly value shown.
        const allQ = quarterLines.flatMap(l => l.data).filter(v => v !== null && !Number.isNaN(v));
        const metric = document.getElementById('compare-weather-metric')?.value || '';
        const noBeginZero = ['temperature', 'pressure', 'dew_point', 'soil_temp'].includes(metric);
        const nb = niceBounds(allQ, noBeginZero);

        // ── Statistics: correlate plantings with each year's climate ──────────
        const ymap = {};
        (wData.yearly?.labels || []).forEach((lbl, i) => { ymap[lbl] = wData.yearly.data[i]; });
        const plantTotals = order.map(i => pData.rowTotals[i]);
        const yearlyVals  = years.map(y => (ymap[String(parseInt(y))] ?? ymap[y] ?? null));
        const r     = pearson(plantTotals, yearlyVals);
        const rDesc = describeR(r);
        const slope = trendSlope(yearlyVals);
        const trendTxt = slope === null ? '' :
            `${cfg.short} trend ${slope >= 0 ? '▲' : '▼'} ${Math.abs(slope).toFixed(2)}${cfg.unit}/yr`;

        const cov = wData.coverage || {};
        const covTxt = cov.noaa_fill_months
            ? `${cov.om_months} mo live + ${cov.noaa_fill_months} mo NOAA history`
            : `${cov.total_months || ''} months`;

        container.innerHTML = `
            <canvas id="fused-chart" style="min-height:400px;"></canvas>
            <div style="margin-top:10px;display:flex;flex-wrap:wrap;gap:10px;font-size:0.8rem;">
                <span style="background:#f1f8e9;border:1px solid #c5e1a5;border-radius:8px;padding:6px 12px;">
                    📈 Plantings vs ${escHtml(cfg.short)}: <strong style="color:${escHtml(rDesc.color)};">${escHtml(rDesc.txt)}</strong>
                </span>
                ${trendTxt ? `<span style="background:#fff3e0;border:1px solid #ffe0b2;border-radius:8px;padding:6px 12px;">🌡️ ${escHtml(trendTxt)}</span>` : ''}
                <span style="background:#e3f2fd;border:1px solid #bbdefb;border-radius:8px;padding:6px 12px;color:#1565c0;">🗄️ ${escHtml(covTxt)}</span>
            </div>
            <div style="margin-top:6px;font-size:0.74rem;color:#888;">
                Bars = plants added that year (stacked by ${escHtml(pData.dim_b_label)}). Lines = each year's quarterly ${escHtml(cfg.label.toLowerCase())} — summer (red) vs winter (blue) shows the seasonal swing.
            </div>`;
        const canvas = document.getElementById('fused-chart');

        // Soft comfort-zone bands behind the temperature lines.
        const bandPlugin = metric === 'temperature' ? {
            id: 'fusedTempBands',
            beforeDatasetsDraw(chart) {
                const y1 = chart.scales.y1, xa = chart.chartArea;
                if (!y1 || !xa) return;
                const ctx = chart.ctx;
                const band = (lo, hi, color) => {
                    const yHi = y1.getPixelForValue(Math.min(hi, y1.max));
                    const yLo = y1.getPixelForValue(Math.max(lo, y1.min));
                    ctx.fillStyle = color;
                    ctx.fillRect(xa.left, yHi, xa.right - xa.left, yLo - yHi);
                };
                ctx.save();
                band(y1.min, 32, 'rgba(33,150,243,0.06)');   // frost
                band(55, 80, 'rgba(76,175,80,0.08)');        // ideal
                band(85, y1.max, 'rgba(244,67,54,0.06)');    // heat
                ctx.restore();
            }
        } : null;

        weatherChart = new Chart(canvas.getContext('2d'), {
            type: 'bar',
            data: { labels: years, datasets: [...barDatasets, ...quarterLines] },
            plugins: bandPlugin ? [bandPlugin] : [],
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: true, position: 'top', labels: { boxWidth: 12, font: { size: 11 } } },
                    tooltip: {
                        mode: 'index', intersect: false,
                        callbacks: {
                            label(ctx) {
                                const v = ctx.raw;
                                if (v === null) return null;
                                if (ctx.dataset.yAxisID === 'y1') return `${ctx.dataset.label}: ${v}${cfg.unit}`;
                                return `${ctx.dataset.label}: ${v} plants`;
                            }
                        }
                    },
                },
                interaction: { mode: 'index', intersect: false },
                scales: {
                    y: {
                        stacked: true, beginAtZero: true, position: 'left',
                        title: { display: true, text: 'Plants added' },
                        ticks: { precision: 0 },
                    },
                    y1: {
                        position: 'right',
                        grid: { drawOnChartArea: false },
                        min: nb.min,
                        max: nb.max,
                        title: { display: true, text: cfg.label, color: cfg.color },
                        ticks: { stepSize: nb.step, color: cfg.color, callback: v => v + cfg.unit },
                    },
                    x: { stacked: true, title: { display: true, text: pData.dim_a_label } },
                },
            },
        });
    }

    // ══════════════════════════════════════════════════════════════════════════
    // IoT MOCK OVERLAY — simulated sensor data per garden bed
    // ══════════════════════════════════════════════════════════════════════════
    let iotChart = null;

    // ══════════════════════════════════════════════════════════════════════════
    // ASK THE BRAIN WHAT IS WORTH LOOKING AT
    // ══════════════════════════════════════════════════════════════════════════
    async function onCompareSourceChange() {
        const mode   = document.querySelector('input[name="compare-source"]:checked')?.value || 'manual';
        const out    = document.getElementById('discover-output');
        const status = document.getElementById('discover-status');
        if (!out) return;

        if (mode !== 'discover') {
            out.style.display = 'none';
            if (status) status.textContent = '';
            return;
        }

        out.style.display = '';
        out.innerHTML = '<p style="padding:14px;color:#888;font-size:0.85rem;">Reading the whole collection…</p>';
        if (status) status.textContent = '';

        try {
            // No &warm=1 on purpose. A visitor reads the cached answer; only
            // the collector's warm tick is allowed to spend the ~10s it takes
            // to rescan and re-ask the model. See discover.php.
            const res  = await fetch('data.php?discover=1&_t=' + Date.now());
            const data = await res.json();
            renderDiscoveries(out, status, data);
        } catch (e) {
            out.innerHTML = `<p style="padding:14px;color:#c00;font-size:0.85rem;">Could not run the scan: ${escHtml(e.message)}</p>`;
        }
    }

    function renderDiscoveries(out, status, data) {
        const esc = t => String(t == null ? '' : t).replace(/[&<>]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));

        if (status) {
            status.textContent = `${data.plants} plants · ${data.pairs_tested} pairs tested`
                               + (data.stale ? ' · last scan' : '');
        }

        // The numbers exist whether or not the model answered, so say so
        // rather than showing an empty panel.
        if (!data.ai || !data.ai.findings || !data.ai.findings.length) {
            const raw = (data.top_raw || []).slice(0, 5).map(f => {
                const c = f.cells[0];
                return `<li><strong>${esc(f.a)}</strong> × <strong>${esc(f.b)}</strong> —
                        ${esc(c.a_value)} + ${esc(c.b_value)}: ${c.plants} plants,
                        ${c.lift}× more than chance</li>`;
            }).join('');
            out.innerHTML = `<div class="cmp-ai cmp-ai-off">
                🧠 The model has not read this scan yet — here are the raw standouts.
                <ul style="margin:.5rem 0 0 1.1rem;font-size:.84rem;">${raw}</ul></div>`;
            return;
        }

        const cards = data.ai.findings.map((f, i) => `
            <div style="background:#fff;border:2px solid #86EFAC;border-radius:10px;padding:12px 14px;margin-bottom:10px;">
                <div style="font-weight:700;color:#14532D;font-size:0.92rem;margin-bottom:4px;">
                    ${i + 1}. ${esc(f.title)}
                </div>
                <div style="font-size:0.86rem;color:#166534;margin-bottom:5px;">${esc(f.what)}</div>
                <div style="font-size:0.84rem;color:#166534;"><strong>Why it matters:</strong> ${esc(f.why)}</div>
                ${f.caution ? `<div style="font-size:0.8rem;color:#8a6d3b;margin-top:5px;"><strong>But:</strong> ${esc(f.caution)}</div>` : ''}
            </div>`).join('');

        out.innerHTML = `
        <div class="cmp-ai" style="margin:0 0 12px;">
            <div class="cmp-ai-tag">🧠 ${esc(data.ai.model || 'local AI')} · top 3 of ${data.pairs_tested} pairs</div>
            ${data.ai.overall ? `<p class="cmp-ai-head">${esc(data.ai.overall)}</p>` : ''}
            ${cards}
            <p class="cmp-ai-note">
                Every count and ratio above is measured from the database; the wording is the
                model's reading of them. These are correlations across ${data.plants} plants —
                with ${data.pairs_tested} pairs tested, a few standouts are expected by chance
                alone, and most reflect what was bought and where it was planted rather than
                what the plants did.
            </p>
        </div>`;
    }

    async function onIotMockChange() {
        const enabled = document.getElementById('compare-iot-toggle')?.checked;
        const container = document.getElementById('compare-iot-overlay');
        if (!container) return;

        if (!enabled) {
            container.innerHTML = '';
            if (iotChart) { iotChart.destroy(); iotChart = null; }
            return;
        }

        const sensor = document.getElementById('compare-iot-sensor')?.value || 'soil_moisture';
        container.innerHTML = '<p style="text-align:center;padding:20px;color:#888;">Loading mock IoT data...</p>';

        try {
            const res = await fetch(`data.php?mock_iot=1&sensor=${sensor}&_t=${Date.now()}`);
            const data = await res.json();

            if (data.error) {
                container.innerHTML = `<p style="color:#c00;text-align:center;">${escHtml(data.error)}</p>`;
                return;
            }

            renderIotChart(container, data);
        } catch (e) {
            container.innerHTML = `<p style="color:#c00;text-align:center;">${escHtml(e.message)}</p>`;
        }
    }

    const IOT_COLORS = ['#4CAF50','#2196F3','#FF9800','#E91E63','#9C27B0','#00BCD4','#FF5722','#795548'];

    function renderIotChart(container, data) {
        if (iotChart) { iotChart.destroy(); iotChart = null; }

        // Render as a heatmap grid instead of line chart
        const datasets = data.datasets || [];
        const labels = data.labels || [];
        const unit = data.unit || '';

        // Find min/max for color scaling
        let allVals = [];
        datasets.forEach(ds => ds.data.forEach(v => { if (v !== null) allVals.push(v); }));
        const minVal = Math.min(...allVals);
        const maxVal = Math.max(...allVals);
        const range = maxVal - minVal || 1;

        function heatColor(val) {
            const ratio = (val - minVal) / range;
            if (ratio < 0.25) return { bg: '#e3f2fd', fg: '#1565c0' };
            if (ratio < 0.5)  return { bg: '#c8e6c9', fg: '#2e7d32' };
            if (ratio < 0.75) return { bg: '#fff3e0', fg: '#e65100' };
            return { bg: '#ffcdd2', fg: '#c62828' };
        }

        // Build heatmap table: rows = beds, cols = months
        let colHeaders = labels.map(l => `<th style="font-size:0.75rem;font-weight:700;color:#fff;background:var(--pw-green,#2e7d32);padding:6px 4px;white-space:nowrap;">${l}</th>`).join('');

        let rows = datasets.map(ds => {
            let cells = ds.data.map((v, i) => {
                const c = heatColor(v);
                return `<td style="background:${c.bg};color:${c.fg};text-align:center;font-weight:700;font-size:0.82rem;padding:8px 4px;border:1px solid rgba(0,0,0,0.06);" title="${ds.label} · ${labels[i]}: ${v}${unit}">${v}</td>`;
            }).join('');
            return `<tr><th style="font-size:0.82rem;font-weight:700;color:#333;padding:6px 10px;text-align:left;background:#f5f5f5;white-space:nowrap;">${ds.label}</th>${cells}</tr>`;
        }).join('');

        container.innerHTML = `
        <div style="background:#fff;border:2px solid #E65100;border-radius:12px;box-shadow:3px 3px 0px rgba(0,0,0,0.4);padding:16px;margin:0 16px 16px;">
            <div style="display:flex;align-items:center;gap:8px;margin-bottom:12px;flex-wrap:wrap;">
                <span style="font-size:1.1rem;">🔌</span>
                <strong style="font-size:0.95rem;color:#E65100;">IoT Demo: ${data.sensorLabel}</strong>
                <span style="background:#FFF3E0;color:#E65100;font-size:0.72rem;font-weight:700;padding:2px 8px;border-radius:10px;border:1px solid #FFE0B2;">SIMULATED</span>
            </div>
            <div style="overflow-x:auto;">
                <table style="border-collapse:collapse;width:100%;">
                    <thead><tr><th></th>${colHeaders}</tr></thead>
                    <tbody>${rows}</tbody>
                </table>
            </div>
            <div style="display:flex;gap:6px;justify-content:center;margin-top:10px;font-size:0.72rem;font-weight:600;">
                <span style="background:#e3f2fd;color:#1565c0;padding:2px 10px;border-radius:4px;">Low</span>
                <span style="background:#c8e6c9;color:#2e7d32;padding:2px 10px;border-radius:4px;">Med-Low</span>
                <span style="background:#fff3e0;color:#e65100;padding:2px 10px;border-radius:4px;">Med-High</span>
                <span style="background:#ffcdd2;color:#c62828;padding:2px 10px;border-radius:4px;">High</span>
            </div>
            <div style="margin-top:8px;font-size:0.75rem;color:#888;text-align:center;">
                Demo data — connect real IoT sensors to replace with live readings per bed
            </div>
        </div>`;
    }

    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
    else init();
}());
