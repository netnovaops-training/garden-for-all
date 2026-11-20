// js/source-link.js
// The plant "Source" link: the grower's product page (the product_url column).
// The server already sends null for a bad address (weblink.rs); this checks again
// with the same rules, so an old or bad value never becomes a link.
// Rules: https:// only, a host with a dot (letters, digits, dots, dashes, optional
// :port), no spaces or control characters, at most 500 characters.

const HOST_RE = /^[A-Za-z0-9](?:[A-Za-z0-9.-]*[A-Za-z0-9])?(?::\d{1,5})?$/;

// The good address, or '' when it is empty or fails the rules.
export function safeSourceUrl(raw) {
    if (typeof raw !== 'string') return '';
    const v = raw.trim();
    if (!v || [...v].length > 500) return '';
    if (/[\s\u0000-\u001f\u007f-\u009f"<>\\`]/.test(v)) return '';
    if (v.slice(0, 8).toLowerCase() !== 'https://') return '';
    const rest = v.slice(8);
    const cut = rest.search(/[/?#]/);
    const host = cut === -1 ? rest : rest.slice(0, cut);
    const name = host.split(':')[0];
    if (!HOST_RE.test(host) || !name.includes('.') || /^[.-]|[.-]$/.test(name)) return '';
    return v;
}

// The link as HTML, or '' when there is no good address. The address goes only
// into the escaped href; the visible text is always "Source" (EN and KO pages alike).
export function sourceLinkHTML(raw, cls = 'plant-source') {
    const u = safeSourceUrl(raw);
    if (!u) return '';
    const esc = s => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
    return `<a class="${esc(cls)}" href="${esc(u)}" target="_blank" rel="noopener noreferrer nofollow" onclick="event.stopPropagation()">Source</a>`;
}
