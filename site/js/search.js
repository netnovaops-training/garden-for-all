// js/search.js — Autocomplete + A-Z lookup bar + Bed accordion toggle + bed/tag search chip behavior
// Loaded as a module by catalog.php.
// Calls window.runSearch() which is registered by app.js.

// ── Search Autocomplete ───────────────────────────────────────────────────────
(function initAutocomplete() {
    const TYPE_LABEL = { plant: '🌿', family: '🍃', brand: '🏪', future: '⭐' };

    let activeIdx = -1;   // keyboard-highlighted row index
    let lastQuery = '';
    let fetchTimer = null;
    let currentSuggestions = [];

    // Build the dropdown element once and attach it to the search container
    function setup() {
        const box = document.getElementById('search-box');
        if (!box) return;

        // Wrap the input so we can position the dropdown relative to it
        const wrapper = document.createElement('div');
        wrapper.className = 'ac-wrapper';
        box.parentNode.insertBefore(wrapper, box);
        wrapper.appendChild(box);

        const dropdown = document.createElement('div');
        dropdown.id = 'ac-dropdown';
        dropdown.className = 'ac-dropdown';
        dropdown.setAttribute('role', 'listbox');
        wrapper.appendChild(dropdown);

        // Input handler — debounced fetch
        box.addEventListener('input', () => {
            const q = box.value.trim();
            if (q.length < 2) { close(); return; }
            if (q === lastQuery) return;
            lastQuery = q;
            clearTimeout(fetchTimer);
            fetchTimer = setTimeout(() => fetchSuggestions(q, dropdown, box), 220);
        });

        // Keyboard navigation
        box.addEventListener('keydown', (e) => {
            const items = dropdown.querySelectorAll('.ac-item');
            if (!items.length) return;

            if (e.key === 'ArrowDown') {
                e.preventDefault();
                activeIdx = Math.min(activeIdx + 1, items.length - 1);
                highlight(items);
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                activeIdx = Math.max(activeIdx - 1, -1);
                highlight(items);
                if (activeIdx === -1) box.value = lastQuery;
            } else if (e.key === 'Enter') {
                if (activeIdx >= 0 && items[activeIdx]) {
                    e.preventDefault();
                    selectSuggestion(items[activeIdx].dataset.value, box, dropdown);
                }
            } else if (e.key === 'Escape') {
                close();
            }
        });

        // Close on outside click
        document.addEventListener('click', (e) => {
            if (!wrapper.contains(e.target)) close();
        });

        function close() {
            dropdown.innerHTML = '';
            dropdown.classList.remove('ac-open');
            activeIdx = -1;
        }

        function highlight(items) {
            items.forEach((el, i) => el.classList.toggle('ac-highlighted', i === activeIdx));
            if (activeIdx >= 0 && items[activeIdx]) {
                box.value = items[activeIdx].dataset.value;
            }
        }

        async function fetchSuggestions(q, dropdown, box) {
            try {
                const res = await fetch(`catalog.php?autocomplete=1&q=${encodeURIComponent(q)}`);
                const suggestions = await res.json();
                currentSuggestions = suggestions;
                renderDropdown(suggestions, dropdown, box, close);
            } catch { close(); }
        }

        function renderDropdown(suggestions, dropdown, box, closeFn) {
            dropdown.innerHTML = '';
            activeIdx = -1;
            if (!suggestions.length) { dropdown.classList.remove('ac-open'); return; }

            suggestions.forEach((s) => {
                const item = document.createElement('div');
                item.className = 'ac-item';
                item.setAttribute('role', 'option');
                item.dataset.value = s.value;

                const icon = document.createElement('span');
                icon.className = 'ac-icon';
                icon.textContent = TYPE_LABEL[s.type] || '🔍';

                const text = document.createElement('span');
                text.className = 'ac-text';
                // Bold the matching portion
                const q = lastQuery;
                const lower = s.value.toLowerCase();
                const matchIdx = lower.indexOf(q.toLowerCase());
                if (matchIdx >= 0) {
                    text.innerHTML =
                        escHtml(s.value.slice(0, matchIdx)) +
                        '<strong>' + escHtml(s.value.slice(matchIdx, matchIdx + q.length)) + '</strong>' +
                        escHtml(s.value.slice(matchIdx + q.length));
                } else {
                    text.textContent = s.value;
                }

                item.appendChild(icon);
                item.appendChild(text);

                // Future plants: show a "Wish List" badge for context, but the
                // search value stays the clean plant name so results still match.
                if (s.type === 'future') {
                    const badge = document.createElement('span');
                    badge.className = 'ac-badge';
                    badge.textContent = 'Wish List';
                    item.appendChild(badge);
                }

                item.addEventListener('mousedown', (e) => {
                    e.preventDefault(); // prevent blur before click
                    selectSuggestion(s.value, box, dropdown);
                });
                item.addEventListener('mouseover', () => {
                    activeIdx = Array.from(dropdown.querySelectorAll('.ac-item')).indexOf(item);
                    highlight(dropdown.querySelectorAll('.ac-item'));
                });

                dropdown.appendChild(item);
            });

            dropdown.classList.add('ac-open');
        }

        function selectSuggestion(value, box, dropdown) {
            box.value = value;
            lastQuery = value;
            dropdown.innerHTML = '';
            dropdown.classList.remove('ac-open');
            activeIdx = -1;
            if (typeof window.clearAZActive === 'function') window.clearAZActive();
            if (typeof window.runSearch === 'function') window.runSearch(value);
        }
    }

    function escHtml(str) {
        return str.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
    }

    // Run after DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', setup);
    } else {
        setup();
    }
})();

// ── A-Z Dynamic Plant Lookup Bar ──────────────────────────────────────────────
// Fetches live letter counts from catalog.php?letter_counts=1, then clusters
// adjacent low-count letters into groups (e.g. "B-C") so each group holds
// roughly TARGET_SIZE plants. Renders clickable pill buttons into #az-bar.

(function initAZBar() {
    const TARGET_SIZE = 20; // aim for ~20 plants per group

    async function fetchLetterCounts() {
        const res = await fetch('catalog.php?letter_counts=1');
        return res.json(); // { A: 42, B: 3, C: 8, ... }
    }

    function buildGroups(counts) {
        const letters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');
        const groups  = [];
        let   bucket  = [];
        let   total   = 0;

        for (const letter of letters) {
            const n = counts[letter] || 0;
            if (n === 0) continue; // skip letters with no plants

            // If this single letter already meets the target, flush any
            // pending bucket first, then make it a solo group.
            if (n >= TARGET_SIZE) {
                if (bucket.length > 0) {
                    groups.push({ letters: [...bucket], count: total });
                    bucket = [];
                    total  = 0;
                }
                groups.push({ letters: [letter], count: n });
            } else {
                bucket.push(letter);
                total += n;
                // Flush when bucket meets the target
                if (total >= TARGET_SIZE) {
                    groups.push({ letters: [...bucket], count: total });
                    bucket = [];
                    total  = 0;
                }
            }
        }
        // Flush any remaining letters at the end
        if (bucket.length > 0) {
            groups.push({ letters: [...bucket], count: total });
        }
        return groups;
    }

    function groupLabel(group) {
        if (group.letters.length === 1) return group.letters[0];
        return group.letters[0] + '–' + group.letters[group.letters.length - 1];
    }

    function groupSearchTerm(group) {
        // "^A" or "^B-C-D"  — catalog.php splits on '-'
        return '^' + group.letters.join('-');
    }

    function render(groups) {
        const bar = document.getElementById('az-bar');
        if (!bar) return;
        bar.innerHTML = '';
        groups.forEach(group => {
            const btn = document.createElement('button');
            btn.className   = 'az-btn';
            btn.textContent = groupLabel(group);
            btn.title       = group.count + ' plant' + (group.count !== 1 ? 's' : '');
            const term      = groupSearchTerm(group);
            btn.addEventListener('click', () => {
                // Highlight active button
                bar.querySelectorAll('.az-btn').forEach(b => b.classList.remove('az-active'));
                btn.classList.add('az-active');
                if (typeof window.runSearch === 'function') {
                    window.runSearch(term);
                }
            });
            bar.appendChild(btn);
        });
    }

    // Allow external code to clear the active highlight (e.g. when user does a
    // normal text search or picks a bed, the A-Z highlight should go away).
    window.clearAZActive = function () {
        document.querySelectorAll('#az-bar .az-btn').forEach(b => b.classList.remove('az-active'));
    };

    // Boot
    const bar = document.getElementById('az-bar');
    if (!bar) return;

    fetchLetterCounts()
        .then(counts => {
            const groups = buildGroups(counts);
            render(groups);
        })
        .catch(() => {
            if (bar) bar.innerHTML = '<span class="az-loading">A–Z unavailable</span>';
        });
})();

// ── Bed accordion toggle ──────────────────────────────────────────────────────
// Only one group stays open at a time.
window.toggleBedGroup = function (btn) {
    const isOpen = btn.getAttribute('aria-expanded') === 'true';

    // Collapse all groups first
    document.querySelectorAll('.bed-group-btn[aria-expanded="true"]').forEach(b => {
        b.setAttribute('aria-expanded', 'false');
        b.nextElementSibling.setAttribute('hidden', '');
    });

    // Open the clicked group unless it was already open (click-to-close)
    if (!isOpen) {
        btn.setAttribute('aria-expanded', 'true');
        btn.nextElementSibling.removeAttribute('hidden');
    }
};

// ── Bed chip click ────────────────────────────────────────────────────────────
// bedName comes from data-bed attribute via this.dataset.bed in the onclick,
// so apostrophes and any other characters are never parsed as JS literals.
window.bedSearch = function (chipBtn, bedName) {
    if (typeof window.clearAZActive === 'function') window.clearAZActive();
    if (typeof window.runSearch === 'function') {
        window.runSearch(bedName);
    }
    // Collapse the accordion group after a bed is selected
    const groupBtn = chipBtn.closest('.bed-group')?.querySelector('.bed-group-btn');
    if (groupBtn) {
        groupBtn.setAttribute('aria-expanded', 'false');
        groupBtn.nextElementSibling.setAttribute('hidden', '');
    }
};

// ── Clear A-Z highlight when quick-filter selects / text searches fire ────────
// Wrap window.runSearch so any non-alpha-lookup call clears the A-Z active state.
// This runs after search.js loads; app.js registers runSearch before catalog boots.
document.addEventListener('DOMContentLoaded', () => {
    const _orig = window.runSearch;
    if (typeof _orig === 'function') {
        window.runSearch = function (term, ...args) {
            // Only clear highlight if this isn't an alpha lookup (those set it themselves)
            if (typeof term === 'string' && term[0] !== '^') {
                if (typeof window.clearAZActive === 'function') window.clearAZActive();
            }
            return _orig.call(this, term, ...args);
        };
    }
});
