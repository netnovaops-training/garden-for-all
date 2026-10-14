// =============================================================================
// js/home-blog.js — Blog post expand/collapse + admin inline edit
//
// Reads window.SITE_CONFIG.isAdmin to gate admin behaviour.
// =============================================================================
(function () {
    'use strict';

    // ── Blog post expand / collapse ───────────────────────────────
    window.toggleHomepagePost = function (id) {
        const wrapper = document.getElementById('home-wrapper-' + id);
        const button  = document.getElementById('toggle-btn-' + id);
        if (!wrapper || !button) return;

        if (wrapper.classList.contains('expanded')) {
            wrapper.classList.remove('expanded');
            button.innerHTML = '📖 Read More';
            wrapper.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        } else {
            wrapper.classList.add('expanded');
            button.innerHTML = '📕 Show Less';
        }
    };

    // ── Admin: inline blog edit ───────────────────────────────────
    document.addEventListener('DOMContentLoaded', function () {
        const isAdmin = (window.SITE_CONFIG || {}).isAdmin;
        if (!isAdmin) return;

        document.querySelectorAll('.edit-blog-btn').forEach(function (btn) {
            btn.addEventListener('click', function () {
                document.getElementById('edit_blog_id').value    = this.getAttribute('data-id');
                document.getElementById('edit_blog_title').value = this.getAttribute('data-title');
                // Decode HTML entities in the content attribute
                const tmp = document.createElement('textarea');
                tmp.innerHTML = this.getAttribute('data-content');
                document.getElementById('edit_blog_content').value = tmp.value;
                document.getElementById('edit-blog-modal').style.display = 'flex';
            });
        });
    });
}());
