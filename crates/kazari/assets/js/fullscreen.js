(function() {
  var SCALE_KEY = 'kz-fs-font-scale';
  var STEP = 0.1;
  var MIN = 0.6;
  var MAX = 5.0;
  var DEFAULT = 1;

  function getScale(block) {
    if (block) {
      var current = parseFloat(block.style.getPropertyValue('--kz-fs-font-scale'));
      if (current) return current;
    }
    try { return parseFloat(localStorage.getItem(SCALE_KEY)) || DEFAULT; }
    catch(e) { return DEFAULT; }
  }

  function setScale(block, scale) {
    scale = Math.max(MIN, Math.min(MAX, Math.round(scale * 100) / 100));
    block.style.setProperty('--kz-fs-font-scale', scale);
    try { localStorage.setItem(SCALE_KEY, scale); } catch(e) {}
  }

  document.addEventListener('click', function(e) {
    var btn = e.target.closest('.kazari-block .kz-fs-btn');
    if (!btn) return;
    var block = btn.closest('.kazari-block');
    if (!block) return;
    if (document.fullscreenElement === block) {
      document.exitFullscreen();
    } else if (block.requestFullscreen) {
      block.requestFullscreen();
    } else if (block.webkitRequestFullscreen) {
      block.webkitRequestFullscreen();
    }
  });

  document.addEventListener('click', function(e) {
    var btn = e.target.closest('.kazari-block .kz-font-inc');
    if (!btn) return;
    var block = btn.closest('.kazari-block');
    if (block) setScale(block, getScale(block) + STEP);
  });

  document.addEventListener('click', function(e) {
    var btn = e.target.closest('.kazari-block .kz-font-dec');
    if (!btn) return;
    // The second click of a double click is handled by the dblclick reset.
    if (e.detail > 1) return;
    var block = btn.closest('.kazari-block');
    if (block) setScale(block, getScale(block) - STEP);
  });

  document.addEventListener('dblclick', function(e) {
    var btn = e.target.closest('.kazari-block .kz-font-dec');
    if (!btn) return;
    var block = btn.closest('.kazari-block');
    if (block) setScale(block, DEFAULT);
  });

  document.addEventListener('fullscreenchange', function() {
    var el = document.fullscreenElement;
    if (el && el.classList.contains('kazari-block')) {
      setScale(el, getScale(el));
      var btn = el.querySelector('.kz-fs-btn');
      if (btn) btn.setAttribute('aria-expanded', 'true');
    } else {
      var btns = document.querySelectorAll('.kz-fs-btn[aria-expanded="true"]');
      for (var i = 0; i < btns.length; i++) {
        btns[i].setAttribute('aria-expanded', 'false');
      }
    }
  });
})();
