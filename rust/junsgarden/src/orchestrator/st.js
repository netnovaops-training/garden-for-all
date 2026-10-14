// Small helpers for the Rust plant admin (served at /streamlit/st.js).
//
// 1. Multiselect chips: every <select multiple data-ms> becomes Streamlit's
//    box of red chips with an "add" drop-down. The real <select> stays in the
//    form (hidden), so what is submitted is exactly what the chips show.
// 2. The plant editor's calendar: "Multi-Period Mode" switches between the
//    start/end drop-downs and the twelve month boxes, and "Does not bloom"
//    hides the bloom inputs, without a round trip, as Streamlit did.
(function () {
  function chips(sel) {
    var box = document.createElement('div');
    box.className = 'st-ms';
    var add = document.createElement('select');
    var max = +(sel.getAttribute('data-max') || 0);
    sel.style.display = 'none';
    sel.parentNode.insertBefore(box, sel);
    // Tell the form something changed (a form with onchange reloads).
    function changed() { sel.dispatchEvent(new Event('change', { bubbles: true })); }
    function draw() {
      box.innerHTML = '';
      Array.prototype.forEach.call(sel.options, function (o) {
        if (!o.selected) return;
        var c = document.createElement('span');
        c.className = 'st-chip';
        c.textContent = o.text;
        var x = document.createElement('button');
        x.type = 'button';
        x.textContent = '×';
        x.onclick = function () { o.selected = false; draw(); changed(); };
        c.appendChild(x);
        box.appendChild(c);
      });
      add.innerHTML = '<option value="">Choose an option</option>';
      Array.prototype.forEach.call(sel.options, function (o, i) {
        if (o.selected) return;
        var n = document.createElement('option');
        n.value = String(i);
        n.textContent = o.text;
        add.appendChild(n);
      });
      // Streamlit's max_selections: no "add" box once the limit is reached.
      var count = Array.prototype.filter.call(sel.options, function (o) { return o.selected; }).length;
      if (!max || count < max) box.appendChild(add);
    }
    add.onchange = function (e) {
      e.stopPropagation();
      if (add.value !== '') { sel.options[+add.value].selected = true; draw(); changed(); }
    };
    draw();
  }

  function calendar(form) {
    var multi = form.querySelector('[data-cal-multi]');
    var nobloom = form.querySelector('[data-cal-nobloom]');
    function sync() {
      var m = multi && multi.checked, nb = nobloom && nobloom.checked;
      form.querySelectorAll('[data-cal=range]').forEach(function (e) { e.style.display = m ? 'none' : ''; });
      form.querySelectorAll('[data-cal=multi]').forEach(function (e) { e.style.display = m ? '' : 'none'; });
      form.querySelectorAll('[data-cal=bloom]').forEach(function (e) { e.style.display = nb ? 'none' : ''; });
      form.querySelectorAll('[data-cal=flowerless]').forEach(function (e) { e.style.display = nb ? '' : 'none'; });
    }
    if (multi) multi.addEventListener('change', sync);
    if (nobloom) nobloom.addEventListener('change', sync);
    sync();
  }

  // The image manager's upload: show the chosen photos and the
  // "Process & Save" button only once files are picked, as Streamlit did.
  function uploadPreview(form) {
    var input = form.querySelector('input[type=file]');
    var box = form.querySelector('[data-upload-preview]');
    var btn = form.querySelector('[data-upload-save]');
    if (!input || !box || !btn) return;
    input.addEventListener('change', function () {
      box.innerHTML = '';
      Array.prototype.forEach.call(input.files, function (f) {
        var fig = document.createElement('div');
        fig.style.display = 'inline-block';
        fig.style.margin = '0 1rem 1rem 0';
        var img = document.createElement('img');
        img.width = 220;
        img.src = URL.createObjectURL(f);
        var cap = document.createElement('div');
        cap.className = 'st-img-cap';
        cap.textContent = f.name;
        fig.appendChild(img);
        fig.appendChild(cap);
        box.appendChild(fig);
      });
      btn.style.display = input.files.length ? '' : 'none';
    });
  }

  // Slow buttons (the AI, the barcode batch): show Streamlit's spinner with
  // the form's data-busy words while the server works.
  function busy(form) {
    form.addEventListener('submit', function () {
      var s = document.createElement('div');
      s.className = 'st-status';
      s.innerHTML = '<span class="st-spin"></span>';
      s.appendChild(document.createTextNode(form.getAttribute('data-busy')));
      form.parentNode.insertBefore(s, form.nextSibling);
      form.querySelectorAll('button').forEach(function (b) { b.disabled = true; });
    });
  }

  document.addEventListener('DOMContentLoaded', function () {
    document.querySelectorAll('form').forEach(uploadPreview);
    document.querySelectorAll('form[data-busy]').forEach(busy);
    // Keep the open tab in view, as Streamlit's tab bar does.
    document.querySelectorAll('.st-tabs a.on').forEach(function (a) {
      a.parentNode.scrollLeft = a.offsetLeft - a.parentNode.offsetLeft - 40;
    });
    document.querySelectorAll('select[data-ms]').forEach(chips);
    document.querySelectorAll('form[data-plant-form]').forEach(calendar);
  });
})();
