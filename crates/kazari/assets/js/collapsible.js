document.addEventListener('click', function(e) {
    var btn = e.target.closest('.kazari-block .kz-collapse-btn, .kazari-block .kz-collapse-toggle');
    if (!btn) return;
    var block = btn.closest('.kazari-block');
    if (!block) return;
    var bottomBtn = block.querySelector('.kz-collapse-btn');
    var toggleBtn = block.querySelector('.kz-collapse-toggle');
    var announce = block.querySelector('.kz-sr-announce');
    var isCollapsed = block.classList.contains('kz-collapsed');
    if (isCollapsed) {
        block.classList.remove('kz-collapsed');
        if (bottomBtn) {
            bottomBtn.setAttribute('aria-expanded', 'true');
            bottomBtn.textContent = bottomBtn.getAttribute('data-collapse');
        }
        if (toggleBtn) {
            toggleBtn.setAttribute('aria-expanded', 'true');
            toggleBtn.setAttribute('aria-label', toggleBtn.getAttribute('data-collapse') || '');
            toggleBtn.setAttribute('data-tooltip', toggleBtn.getAttribute('data-collapse') || '');
        }
        if (announce && bottomBtn) announce.textContent = bottomBtn.getAttribute('data-expanded-msg') || '';
    } else {
        block.classList.add('kz-collapsed');
        if (bottomBtn) {
            bottomBtn.setAttribute('aria-expanded', 'false');
            bottomBtn.textContent = bottomBtn.getAttribute('data-expand');
        }
        if (toggleBtn) {
            toggleBtn.setAttribute('aria-expanded', 'false');
            toggleBtn.setAttribute('aria-label', toggleBtn.getAttribute('data-expand') || '');
            toggleBtn.setAttribute('data-tooltip', toggleBtn.getAttribute('data-expand') || '');
        }
        if (announce && bottomBtn) announce.textContent = bottomBtn.getAttribute('data-collapsed-msg') || '';
    }
});
