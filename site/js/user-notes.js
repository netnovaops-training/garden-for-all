// js/user-notes.js
// ─────────────────────────────────────────────────────────────────────────────
// Jun's Notes — per-plant rich text notes visible only to the note author.
// When Joshua (or any logged-in user) clicks a plant card, their note pops up
// as an overlay BEFORE the plant modal opens.  Guests see nothing.
// ─────────────────────────────────────────────────────────────────────────────

let notesIndex = new Set();   // plant IDs that have notes
let quillInstance = null;
let pendingModalPlant = null; // plant data waiting for modal after overlay closes

// ══════════════════════════════════════════════════════════════════════════════
// INIT — fetch the notes index so we know which plants have notes
// ══════════════════════════════════════════════════════════════════════════════
export async function initUserNotes() {
    const config = window.GARDEN_CONFIG;
    if (!config.currentUser) return;       // guest — nothing to do

    try {
        const res = await fetch(`catalog.php?notes_index=1&_t=${Date.now()}`);
        const ids = await res.json();
        notesIndex = new Set(ids.map(Number));
    } catch (_) { /* silent */ }

    _injectOverlayHTML();
}

// ══════════════════════════════════════════════════════════════════════════════
// PUBLIC: Called instead of openModal when user is logged in
// ══════════════════════════════════════════════════════════════════════════════
export async function interceptCardClick(plantData, originalOpenModal) {
    const config = window.GARDEN_CONFIG;

    // Not logged in → go straight to modal
    if (!config.currentUser) {
        originalOpenModal(plantData);
        return;
    }

    const hasNote = notesIndex.has(plantData.id);

    if (hasNote) {
        // Show existing note overlay, then modal after dismiss
        pendingModalPlant = plantData;
        await _showNoteOverlay(plantData, false);
    } else {
        // No note — open modal directly (user can add note from modal)
        originalOpenModal(plantData);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// PUBLIC: Open the note editor for a plant (called from modal "Add Note" btn)
// ══════════════════════════════════════════════════════════════════════════════
export async function openNoteEditor(plantData) {
    pendingModalPlant = null; // editing from within modal, don't re-open it
    await _showNoteOverlay(plantData, true);
}

// ══════════════════════════════════════════════════════════════════════════════
// INTERNAL: Inject the overlay HTML once
// ══════════════════════════════════════════════════════════════════════════════
function _injectOverlayHTML() {
    if (document.getElementById('jun-notes-overlay')) return;

    const overlay = document.createElement('div');
    overlay.id = 'jun-notes-overlay';
    overlay.innerHTML = `
        <div class="jun-notes-card">
            <div class="jun-notes-header">
                <span class="jun-notes-icon">📝</span>
                <h3 class="jun-notes-title">Jun's Notes</h3>
                <span class="jun-notes-plant-name"></span>
            </div>
            <div class="jun-notes-body">
                <div id="jun-notes-viewer" class="jun-notes-viewer"></div>
                <div id="jun-notes-editor-wrap" style="display:none;">
                    <div id="jun-notes-editor"></div>
                </div>
            </div>
            <div class="jun-notes-actions">
                <button id="jun-notes-edit-btn"   class="btn jun-btn-edit">✏️ Edit</button>
                <button id="jun-notes-save-btn"   class="btn jun-btn-save"   style="display:none;">💾 Save</button>
                <button id="jun-notes-delete-btn" class="btn jun-btn-delete" style="display:none;">🗑️ Delete</button>
                <button id="jun-notes-cancel-btn" class="btn jun-btn-cancel" style="display:none;">Cancel</button>
                <div style="flex:1;"></div>
                <button id="jun-notes-close-btn"  class="btn jun-btn-close">Continue to Plant →</button>
            </div>
        </div>
    `;
    document.body.appendChild(overlay);

    // Close overlay → open pending modal
    document.getElementById('jun-notes-close-btn').addEventListener('click', _closeOverlay);
    overlay.addEventListener('click', (e) => { if (e.target === overlay) _closeOverlay(); });

    // Edit mode
    document.getElementById('jun-notes-edit-btn').addEventListener('click', () => _enterEditMode());

    // Cancel edit
    document.getElementById('jun-notes-cancel-btn').addEventListener('click', () => _exitEditMode());

    // Save
    document.getElementById('jun-notes-save-btn').addEventListener('click', _saveNote);

    // Delete
    document.getElementById('jun-notes-delete-btn').addEventListener('click', _deleteNote);
}

// ══════════════════════════════════════════════════════════════════════════════
// INTERNAL: Show overlay
// ══════════════════════════════════════════════════════════════════════════════
let _currentPlantId = null;

async function _showNoteOverlay(plantData, startInEdit) {
    _currentPlantId = plantData.id;
    const overlay   = document.getElementById('jun-notes-overlay');
    const viewer    = document.getElementById('jun-notes-viewer');
    const nameEl    = overlay.querySelector('.jun-notes-plant-name');

    // Dynamic title: "The author's Notes", "Jun's Notes", "Kona's Notes", etc.
    const currentUser = window.GARDEN_CONFIG?.currentUser || 'Your';
    const titleEl = overlay.querySelector('.jun-notes-title');
    if (titleEl) titleEl.textContent = `${currentUser}'s Notes`;

    nameEl.textContent = plantData.plant_name || 'Unknown Plant';

    // Fetch current note
    let noteHtml = '';
    try {
        const res = await fetch(`catalog.php?get_note=${plantData.id}&_t=${Date.now()}`);
        const data = await res.json();
        noteHtml = data.note_html || '';
    } catch (_) {}

    viewer.innerHTML = noteHtml || '<em style="color:#999;">No notes yet. Click Edit to add one.</em>';

    if (startInEdit) {
        _enterEditMode(noteHtml);
    } else {
        _exitEditMode();
    }

    overlay.classList.add('active');
    document.body.style.overflow = 'hidden';
}

function _closeOverlay() {
    const overlay = document.getElementById('jun-notes-overlay');
    overlay.classList.remove('active');
    document.body.style.overflow = 'auto';

    // Destroy Quill fully — clear the wrapper so toolbar doesn't stack on reopen
    quillInstance = null;
    const wrap = document.getElementById('jun-notes-editor-wrap');
    if (wrap) wrap.innerHTML = '<div id="jun-notes-editor"></div>';

    // If there's a pending modal, open it now
    if (pendingModalPlant && window._originalOpenModal) {
        const p = pendingModalPlant;
        pendingModalPlant = null;
        window._originalOpenModal(p);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// INTERNAL: Edit / view mode toggle
// ══════════════════════════════════════════════════════════════════════════════
function _enterEditMode(initialHtml) {
    const viewer     = document.getElementById('jun-notes-viewer');
    const editorWrap = document.getElementById('jun-notes-editor-wrap');
    const editorEl   = document.getElementById('jun-notes-editor');

    viewer.style.display     = 'none';
    editorWrap.style.display = 'block';

    document.getElementById('jun-notes-edit-btn').style.display   = 'none';
    document.getElementById('jun-notes-save-btn').style.display   = 'inline-flex';
    document.getElementById('jun-notes-delete-btn').style.display = 'inline-flex';
    document.getElementById('jun-notes-cancel-btn').style.display = 'inline-flex';
    document.getElementById('jun-notes-close-btn').textContent    = 'Cancel & Close';

    // Init Quill if not already loaded
    if (!quillInstance) {
        if (typeof Quill === 'undefined') {
            alert('Editor is still loading — please try again in a moment.');
            return;
        }
        const html = typeof initialHtml === 'string' ? initialHtml : viewer.innerHTML;
        editorEl.innerHTML = '';
        quillInstance = new Quill(editorEl, {
            theme: 'snow',
            placeholder: 'Write your notes here... links, images, anything!',
            modules: {
                toolbar: [
                    [{ 'header': [1, 2, 3, false] }],
                    ['bold', 'italic', 'underline', 'strike'],
                    [{ 'color': [] }, { 'background': [] }],
                    [{ 'list': 'ordered' }, { 'list': 'bullet' }],
                    ['link', 'image'],
                    ['clean']
                ]
            }
        });
        // Set initial content
        if (html && html !== '<em style="color:#999;">No notes yet. Click Edit to add one.</em>') {
            quillInstance.root.innerHTML = html;
        }
    }
}

function _exitEditMode() {
    const viewer     = document.getElementById('jun-notes-viewer');
    const editorWrap = document.getElementById('jun-notes-editor-wrap');

    viewer.style.display     = 'block';
    editorWrap.style.display = 'none';

    document.getElementById('jun-notes-edit-btn').style.display   = 'inline-flex';
    document.getElementById('jun-notes-save-btn').style.display   = 'none';
    document.getElementById('jun-notes-delete-btn').style.display = 'none';
    document.getElementById('jun-notes-cancel-btn').style.display = 'none';
    document.getElementById('jun-notes-close-btn').textContent    = 'Continue to Plant →';

    // Fully reset editor wrap so Quill toolbar doesn't duplicate
    quillInstance = null;
    const wrapExit = document.getElementById('jun-notes-editor-wrap');
    if (wrapExit) wrapExit.innerHTML = '<div id="jun-notes-editor"></div>';
}

// ══════════════════════════════════════════════════════════════════════════════
// INTERNAL: Save / Delete
// ══════════════════════════════════════════════════════════════════════════════
async function _saveNote() {
    if (!quillInstance || !_currentPlantId) return;

    const noteHtml = quillInstance.root.innerHTML;
    const isEmpty  = !quillInstance.getText().trim();

    const btn = document.getElementById('jun-notes-save-btn');
    btn.textContent = '⏳ Saving...';
    btn.disabled = true;

    try {
        const form = new FormData();
        form.append('save_note', '1');
        form.append('plant_id', _currentPlantId);
        form.append('note_html', isEmpty ? '' : noteHtml);

        await fetch('catalog.php', { method: 'POST', body: form });

        // Update index
        if (isEmpty) {
            notesIndex.delete(_currentPlantId);
        } else {
            notesIndex.add(_currentPlantId);
        }

        // Update viewer
        const viewer = document.getElementById('jun-notes-viewer');
        viewer.innerHTML = isEmpty
            ? '<em style="color:#999;">No notes yet. Click Edit to add one.</em>'
            : noteHtml;

        _exitEditMode();
        _updateCardBadges();
    } catch (e) {
        alert('Failed to save note: ' + e.message);
    } finally {
        btn.textContent = '💾 Save';
        btn.disabled = false;
    }
}

async function _deleteNote() {
    if (!_currentPlantId) return;
    if (!confirm('Delete this note permanently?')) return;

    try {
        const form = new FormData();
        form.append('delete_note', '1');
        form.append('plant_id', _currentPlantId);

        await fetch('catalog.php', { method: 'POST', body: form });

        notesIndex.delete(_currentPlantId);

        const viewer = document.getElementById('jun-notes-viewer');
        viewer.innerHTML = '<em style="color:#999;">Note deleted.</em>';

        _exitEditMode();
        _updateCardBadges();
    } catch (e) {
        alert('Failed to delete note: ' + e.message);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// BADGE: Show a small 📝 icon on cards that have notes
// ══════════════════════════════════════════════════════════════════════════════
function _updateCardBadges() {
    document.querySelectorAll('.jun-note-badge').forEach(b => b.remove());

    if (!window.GARDEN_CONFIG.currentUser) return;

    notesIndex.forEach(plantId => {
        const card = document.querySelector(`[data-card-bg][onclick*="${plantId}"]`);
        if (card && !card.querySelector('.jun-note-badge')) {
            const badge = document.createElement('div');
            badge.className = 'jun-note-badge';
            badge.innerHTML = '📝';
            badge.title = 'You have notes on this plant';
            card.appendChild(badge);
        }
    });
}

// Re-apply badges whenever new cards are rendered
export function refreshNoteBadges() {
    if (window.GARDEN_CONFIG?.currentUser) _updateCardBadges();
}

// Check if a plant has a note (used by modal to show "Add Note" button)
export function hasNote(plantId) {
    return notesIndex.has(plantId);
}
