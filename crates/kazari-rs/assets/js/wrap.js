document.addEventListener('click', function(e) {
  var btn = e.target.closest('.kazari-block .kz-wrap-btn');
  if (!btn) return;
  var block = btn.closest('.kazari-block');
  if (!block) return;
  var pre = block.querySelector('pre');
  if (!pre) return;
  var wrapped = pre.classList.toggle('wrap');
  btn.setAttribute('aria-pressed', wrapped);
  var label = wrapped ? btn.getAttribute('data-disable') : btn.getAttribute('data-enable');
  btn.setAttribute('data-tooltip', label);
  btn.setAttribute('aria-label', label);
});
