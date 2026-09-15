document.addEventListener('click', function(e) {
    var btn = e.target.closest('.kazari-block .kz-output-toggle');
    if (!btn) return;
    var panel = btn.closest('.kz-output');
    if (!panel) return;
    var isHidden = panel.classList.contains('kz-output-hidden');
    if (isHidden) {
        panel.classList.remove('kz-output-hidden');
        btn.setAttribute('aria-expanded', 'true');
    } else {
        panel.classList.add('kz-output-hidden');
        btn.setAttribute('aria-expanded', 'false');
    }
});
