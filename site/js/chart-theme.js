/* ============================================================================
   chart-theme.js — one consistent axis and type treatment for every chart
   on data.php. Added 2026-08-28.

   THE PROBLEM THIS SOLVES
   The Data page has 35 chart widgets, each written separately. An audit found
   only 21 of 35 configured their axes at all, and only 8 styled gridlines. So
   some charts had a boxed-in grid, some had none, some started their value
   axis at zero and some did not. Read together they looked like 35 unrelated
   charts rather than one dashboard.

   A truncated value axis is not only inconsistent, it misleads: bars starting
   at 40 instead of 0 make a 10% difference look like a 3x difference. Setting
   beginAtZero globally fixes that everywhere at once.

   HOW IT WORKS — AND WHY IT ONLY TOUCHES `Chart.defaults`
   Chart.js falls back to `Chart.defaults` for anything a chart does not set
   itself, so setting them here reaches all 35 widgets without editing one.

   AN EARLIER VERSION OF THIS FILE ALSO REWROTE chart.options FROM A PLUGIN.
   Do not do that. `chart.options` is a resolver proxy; reading from it while
   assigning into it makes Chart.js throw
       "Recursion detected: _scriptable->_scriptable"
   and rewriting dataset colours from the same hook threw
       "TypeError: t.startsWith is not a function".
   Both were reproduced on the live page and reverted. Defaults are declarative
   and cannot recurse, which is why this version only sets defaults.

   COLOUR IS DELIBERATELY NOT TOUCHED
   There are 51 hardcoded colour arrays across the widgets. A version of this
   file did reclassify them (semantic / sequential / categorical) and it worked,
   but the author's call on 2026-08-28 was to leave the colours exactly as they are.
   Every widget keeps the colours it was written with. If that is ever revisited,
   the thing to know is that the groups are NOT equivalent: red in the
   edible-vs-toxic and toxicity charts is a safety signal, and red/green in the
   losses and survival charts is the whole message. Those must never be
   repainted by a global rule.
   ========================================================================== */
(function () {
  'use strict';
  if (typeof Chart === 'undefined' || !Chart.defaults) return;

  var INK_MUTED = '#5a7a50';                 // the site's own muted green-grey
  var GRID      = 'rgba(51,64,47,0.10)';     // present, but never louder than the data

  /* --- type ------------------------------------------------------------- */
  // Match the site's body font instead of Chart.js's default sans stack, so a
  // chart's labels look like the text around it.
  Chart.defaults.font.family =
    "ui-sans-serif,-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif";
  Chart.defaults.font.size = 12;
  // Labels stay grey. The coloured mark beside them carries the identity --
  // text wearing a series colour is harder to read and adds no information.
  Chart.defaults.color = INK_MUTED;
  Chart.defaults.borderColor = GRID;

  /* --- legend ----------------------------------------------------------- */
  Chart.defaults.plugins.legend.position = 'bottom';
  Chart.defaults.plugins.legend.labels.usePointStyle = true;
  Chart.defaults.plugins.legend.labels.pointStyle = 'circle';
  Chart.defaults.plugins.legend.labels.boxWidth = 8;
  Chart.defaults.plugins.legend.labels.padding = 14;
  Chart.defaults.plugins.legend.labels.color = INK_MUTED;

  /* --- tooltip ---------------------------------------------------------- */
  Chart.defaults.plugins.tooltip.backgroundColor = 'rgba(30,42,28,0.94)';
  Chart.defaults.plugins.tooltip.titleColor = '#ffffff';
  Chart.defaults.plugins.tooltip.bodyColor = '#e7efe4';
  Chart.defaults.plugins.tooltip.padding = 10;
  Chart.defaults.plugins.tooltip.cornerRadius = 8;
  Chart.defaults.plugins.tooltip.boxWidth = 8;
  Chart.defaults.plugins.tooltip.boxHeight = 8;
  Chart.defaults.plugins.tooltip.usePointStyle = true;

  /* --- axes: the actual fix --------------------------------------------
     Chart.js keys scale defaults by scale TYPE. A numeric value axis is
     `linear`; a text axis of names is `category`. Setting them separately is
     what lets the value axis keep its gridlines while the category axis drops
     them -- horizontal lines help you read a height, vertical ones just box
     the chart in.

     Individual widgets can still override any of this; these are only the
     values used when a widget says nothing, which is the case for the 14
     widgets that never configured axes at all. */
  function setScale(name, cfg) {
    Chart.defaults.scales[name] = Object.assign(
      Chart.defaults.scales[name] || {}, cfg);
  }

  setScale('linear', {
    // Never start a value axis part-way up. See the note at the top.
    beginAtZero: true,
    border: { display: false },              // the gridlines already imply it
    grid: { color: GRID, drawTicks: false },
    ticks: { padding: 8, color: INK_MUTED, precision: 0 }  // whole plants only
  });

  setScale('category', {
    border: { display: false },
    grid: { display: false },                // no vertical gridlines
    ticks: { padding: 6, color: INK_MUTED, autoSkip: true, maxRotation: 0 }
  });

  // Doughnut/pie use a radial scale; keep its grid quiet too.
  setScale('radialLinear', {
    beginAtZero: true,
    grid: { color: GRID },
    angleLines: { color: GRID },
    ticks: { color: INK_MUTED, backdropColor: 'transparent' }
  });

  // Time axes on the IoT weather charts.
  setScale('time', {
    border: { display: false },
    grid: { display: false },
    ticks: { padding: 6, color: INK_MUTED, maxRotation: 0 }
  });

  /* --- element shapes ---------------------------------------------------
     Thin marks with a small radius on the data end, and lines that do not
     draw a dot on every single point -- a dot per point on a 200-point
     weather series is noise, so points appear on hover instead. */
  if (Chart.defaults.elements) {
    if (Chart.defaults.elements.bar) {
      Chart.defaults.elements.bar.borderRadius = 4;
      Chart.defaults.elements.bar.borderWidth = 0;
    }
    if (Chart.defaults.elements.line) {
      Chart.defaults.elements.line.borderWidth = 2;
      Chart.defaults.elements.line.tension = 0.3;
    }
    if (Chart.defaults.elements.point) {
      Chart.defaults.elements.point.radius = 0;
      Chart.defaults.elements.point.hoverRadius = 5;
      Chart.defaults.elements.point.hitRadius = 12;   // easy to hover
    }
    if (Chart.defaults.elements.arc) {
      Chart.defaults.elements.arc.borderWidth = 2;
      Chart.defaults.elements.arc.borderColor = '#ffffff';  // 2px slice gap
    }
  }

  // A validated palette, exposed but APPLIED TO NOTHING. Available if colour
  // is ever revisited; using it is an explicit choice, not the default. Nothing is repainted
  // automatically -- see "what this deliberately does not do" at the top.
  // Fixed order, colourblind-checked: worst adjacent pair dE 9.1 under
  // protanopia, 19.6 for normal vision.
  window.GardenChartTheme = {
    categorical: ['#2a78d6', '#eb6834', '#1baf7a', '#eda100',
                  '#e87ba4', '#008300', '#4a3aa7', '#e34948'],
    inkMuted: INK_MUTED,
    grid: GRID
  };
})();
