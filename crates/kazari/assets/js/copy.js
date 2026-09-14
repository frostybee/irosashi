document.addEventListener('click', function(e) {
  var btn = e.target.closest('.kazari-block .kz-copy-btn');
  if (!btn) return;
  var code = btn.getAttribute('data-code');
  if (!code) return;
  code = code.replace(/\x7f/g, '\n');
  var iconEl = btn.querySelector('svg');
  var announce = btn.nextElementSibling;
  if (announce && !announce.classList.contains('kz-sr-announce')) announce = null;
  function onSuccess() {
    btn.classList.add('copied');
    if (iconEl) {
      iconEl.innerHTML = '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>';
    }
    if (announce) announce.textContent = btn.getAttribute('data-copied') || '';
    setTimeout(function() {
      btn.classList.remove('copied');
      if (iconEl) {
        iconEl.innerHTML = '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>';
      }
      if (announce) announce.textContent = '';
    }, 2000);
  }
  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(code).then(onSuccess).catch(function() {
      fallbackCopy(code);
      onSuccess();
    });
  } else {
    fallbackCopy(code);
    onSuccess();
  }
});
function fallbackCopy(text) {
  var ta = document.createElement('textarea');
  ta.value = text;
  ta.style.position = 'fixed';
  ta.style.opacity = '0';
  document.body.appendChild(ta);
  ta.select();
  document.execCommand('copy');
  ta.remove();
}
