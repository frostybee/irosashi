/* accordion.js */
;(function(){
document.addEventListener('toggle', function (e) {
    var details = e.target;
    if (details.tagName !== 'DETAILS' || !details.open) return;

    var group = details.closest('.sarde-details-group');
    if (!group || group.hasAttribute('data-independent')) return;

    var siblings = group.querySelectorAll(':scope > details');
    for (var i = 0; i < siblings.length; i++) {
        if (siblings[i] !== details && siblings[i].open) {
            siblings[i].open = false;
        }
    }
}, true);

})();
/* center-toggle.js */
;(function(){
(function () {
  'use strict';

  var STORAGE_KEY = 'sd-docs-centered';
  var BREAKPOINT = 1280;
  var CLASS_CENTERED = 'sd-docs-centered';

  var btn = document.getElementById('sarde-center-toggle');
  if (!btn) return;

  var mq = window.matchMedia('(min-width: ' + BREAKPOINT + 'px)');

  function isCentered() {
    return document.documentElement.classList.contains(CLASS_CENTERED);
  }

  function setAria(centered) {
    btn.setAttribute('aria-label', centered ? 'Expand content width' : 'Center content');
  }

  function center() {
    document.documentElement.classList.add(CLASS_CENTERED);
    setAria(true);
    try { localStorage.setItem(STORAGE_KEY, '1'); } catch (e) {}
  }

  function expand() {
    document.documentElement.classList.remove(CLASS_CENTERED);
    setAria(false);
    try { localStorage.removeItem(STORAGE_KEY); } catch (e) {}
  }

  function toggle() {
    if (isCentered()) {
      expand();
    } else {
      center();
    }
  }

  setAria(isCentered());

  btn.addEventListener('click', toggle);

  function onBreakpoint(e) {
    if (!e.matches) {
      document.documentElement.classList.remove(CLASS_CENTERED);
    } else {
      try {
        if (localStorage.getItem(STORAGE_KEY) === '1') {
          document.documentElement.classList.add(CLASS_CENTERED);
          setAria(true);
        }
      } catch (e) {}
    }
  }

  if (mq.addEventListener) {
    mq.addEventListener('change', onBreakpoint);
  } else {
    mq.addListener(onBreakpoint);
  }
})();

})();
/* copy-text.js */
;(function(){
(function () {
  'use strict';

  // Shared visually-hidden live region so copy confirmations reach screen
  // readers, not just the visual checkmark swap (WCAG 4.1.3).
  var liveRegion = null;
  function announce(text) {
    if (!liveRegion) {
      liveRegion = document.createElement('span');
      liveRegion.className = 'sr-only';
      liveRegion.setAttribute('role', 'status');
      document.body.appendChild(liveRegion);
    }
    liveRegion.textContent = '';
    setTimeout(function () { liveRegion.textContent = text; }, 50);
  }

  document.addEventListener('click', function (e) {
    var btn = e.target.closest('.sarde-copy-text__btn');
    if (!btn) return;

    var widget = btn.closest('.sarde-copy-text');
    if (!widget) return;

    var text = widget.getAttribute('data-copy-text');
    if (!text) return;

    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(function () {
        widget.classList.add('is-copied');
        announce('Copied to clipboard');
        setTimeout(function () {
          widget.classList.remove('is-copied');
        }, 1500);
      });
    }
  });
})();

})();
/* docs-sidebar-drawer.js */
;(function(){
(function () {
  'use strict';

  var BREAKPOINT = 1024;

  var toggle = document.getElementById('sarde-menu-toggle');
  var sidebar = document.getElementById('sarde-sidebar');
  var backdrop = document.getElementById('sarde-sidebar-backdrop');
  var mainFrame = document.querySelector('.sarde-main-frame');
  var header = document.querySelector('.sarde-header');
  var mobileToc = document.getElementById('sarde-mobile-toc');
  var skipLink = document.querySelector('.sarde-skip-link');

  if (!toggle || !sidebar) return;

  var isOpen = false;
  var mq = window.matchMedia('(min-width: ' + BREAKPOINT + 'px)');

  function setInert(flag) {
    var els = [mainFrame, header, mobileToc, skipLink];
    for (var i = 0; i < els.length; i++) {
      if (els[i]) {
        if (flag) els[i].setAttribute('inert', '');
        else els[i].removeAttribute('inert');
      }
    }
  }

  function open() {
    if (isOpen) return;
    isOpen = true;

    toggle.setAttribute('aria-expanded', 'true');
    sidebar.classList.add('is-open');
    sidebar.removeAttribute('inert');
    document.body.classList.add('sarde-sidebar-open');

    if (backdrop) {
      backdrop.style.display = 'block';
      backdrop.offsetHeight;
      backdrop.classList.add('is-visible');
    }

    setInert(true);

    var firstLink = sidebar.querySelector('a[href]');
    if (firstLink) {
      setTimeout(function () { firstLink.focus(); }, 50);
    }
  }

  function close() {
    if (!isOpen) return;
    isOpen = false;
    setInert(false);

    toggle.setAttribute('aria-expanded', 'false');
    sidebar.classList.remove('is-open');
    if (!mq.matches) {
      sidebar.setAttribute('inert', '');
    }
    document.body.classList.remove('sarde-sidebar-open');

    if (backdrop) {
      backdrop.classList.remove('is-visible');
      var hidden = false;
      function hide() {
        if (hidden || isOpen) return;
        hidden = true;
        backdrop.style.display = 'none';
      }
      backdrop.addEventListener('transitionend', function handler() {
        hide();
        backdrop.removeEventListener('transitionend', handler);
      });
      setTimeout(hide, 350);
    }

    toggle.focus();
  }

  function reset() {
    isOpen = false;
    setInert(false);
    toggle.setAttribute('aria-expanded', 'false');
    sidebar.classList.remove('is-open');
    sidebar.removeAttribute('inert');
    document.body.classList.remove('sarde-sidebar-open');
    if (backdrop) {
      backdrop.classList.remove('is-visible');
      backdrop.style.display = 'none';
    }
  }

  // Initialize
  if (!mq.matches) {
    sidebar.setAttribute('inert', '');
  }
  if (backdrop) {
    backdrop.style.display = 'none';
  }

  // Hamburger toggle
  toggle.addEventListener('click', function () {
    if (isOpen) close(); else open();
  });

  // Backdrop click
  if (backdrop) {
    backdrop.addEventListener('click', close);
  }

  // Escape key
  document.addEventListener('keydown', function (e) {
    if (e.key === 'Escape' && isOpen) {
      close();
    }
  });

  // Close on navigation link click
  sidebar.addEventListener('click', function (e) {
    var link = e.target.closest('a[href]');
    if (link && link.getAttribute('href') && link.getAttribute('href').charAt(0) !== '#') {
      close();
    }
  });

  // Resize: reset when crossing to desktop
  function onBreakpoint(e) {
    if (e.matches) {
      reset();
    } else if (!isOpen) {
      sidebar.setAttribute('inert', '');
    }
  }

  if (mq.addEventListener) {
    mq.addEventListener('change', onBreakpoint);
  } else {
    mq.addListener(onBreakpoint);
  }
})();

})();
/* docs-tab-switcher.js */
;(function(){
(function () {
  'use strict';

  function closeAll() {
    document.querySelectorAll('[data-sarde-tab-trigger][aria-expanded="true"]').forEach(function (trigger) {
      trigger.setAttribute('aria-expanded', 'false');
      var menu = trigger.nextElementSibling;
      if (menu) menu.setAttribute('hidden', '');
    });
  }

  document.addEventListener('click', function (e) {
    var trigger = e.target.closest('[data-sarde-tab-trigger]');
    if (trigger) {
      e.stopPropagation();
      var expanded = trigger.getAttribute('aria-expanded') === 'true';
      closeAll();
      if (!expanded) {
        trigger.setAttribute('aria-expanded', 'true');
        var menu = trigger.nextElementSibling;
        if (menu) menu.removeAttribute('hidden');
      }
      return;
    }
    if (!e.target.closest('[data-sarde-tab-menu]')) {
      closeAll();
    }
  });

  document.addEventListener('keydown', function (e) {
    if (e.key === 'Escape') closeAll();
  });

  // Close when keyboard focus leaves the trigger/menu entirely (Tab-away),
  // so aria-expanded never lies while focus is elsewhere on the page.
  document.addEventListener('focusout', function (e) {
    document.querySelectorAll('[data-sarde-tab-trigger][aria-expanded="true"]').forEach(function (t) {
      var menu = t.nextElementSibling;
      var next = e.relatedTarget;
      if (next && (t.contains(next) || (menu && menu.contains(next)))) return;
      t.setAttribute('aria-expanded', 'false');
      if (menu) menu.setAttribute('hidden', '');
    });
  });
})();

})();
/* header-nav.js */
;(function(){
(function () {
  'use strict';

  var toggle = document.getElementById('sarde-nav-toggle');
  if (!toggle) return;

  var nav = document.getElementById(toggle.getAttribute('aria-controls'));
  if (!nav) return;

  function setOpen(open) {
    toggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    nav.classList.toggle('is-open', open);
  }

  function isOpen() {
    return toggle.getAttribute('aria-expanded') === 'true';
  }

  // Toggle on button click.
  toggle.addEventListener('click', function (e) {
    e.stopPropagation();
    setOpen(!isOpen());
  });

  // Close when clicking outside the menu.
  document.addEventListener('click', function (e) {
    if (isOpen() && !nav.contains(e.target) && !toggle.contains(e.target)) {
      setOpen(false);
    }
  });

  // Close on Escape.
  document.addEventListener('keydown', function (e) {
    if (e.key === 'Escape' && isOpen()) {
      setOpen(false);
      toggle.focus();
    }
  });

  // Close after following a link.
  nav.addEventListener('click', function (e) {
    if (e.target.closest('a[href]')) {
      setOpen(false);
    }
  });

  // Reset when crossing to the desktop breakpoint.
  var mq = window.matchMedia('(min-width: 1024px)');
  function onBreakpoint(e) {
    if (e.matches) setOpen(false);
  }
  if (mq.addEventListener) {
    mq.addEventListener('change', onBreakpoint);
  } else {
    mq.addListener(onBreakpoint);
  }
})();

})();
/* lang-switcher.js */
;(function(){
document.addEventListener('click', (e) => {
  const trigger = e.target.closest('[data-sarde-lang-switcher-trigger]');
  if (trigger) {
    const menu = trigger.nextElementSibling;
    const expanded = trigger.getAttribute('aria-expanded') === 'true';
    trigger.setAttribute('aria-expanded', !expanded);
    menu.hidden = expanded;
    return;
  }
  document.querySelectorAll('[data-sarde-lang-switcher-trigger][aria-expanded="true"]').forEach(t => {
    t.setAttribute('aria-expanded', 'false');
    t.nextElementSibling.hidden = true;
  });
});

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    document.querySelectorAll('[data-sarde-lang-switcher-trigger][aria-expanded="true"]').forEach(t => {
      t.setAttribute('aria-expanded', 'false');
      t.nextElementSibling.hidden = true;
      t.focus();
    });
  }
});

// Close when keyboard focus leaves the trigger/menu entirely (Tab-away),
// so aria-expanded never lies while focus is elsewhere on the page.
document.addEventListener('focusout', (e) => {
  document.querySelectorAll('[data-sarde-lang-switcher-trigger][aria-expanded="true"]').forEach(t => {
    const menu = t.nextElementSibling;
    const next = e.relatedTarget;
    if (next && (t.contains(next) || (menu && menu.contains(next)))) return;
    t.setAttribute('aria-expanded', 'false');
    if (menu) menu.hidden = true;
  });
});

})();
/* sidebar-groups.js */
;(function(){
(function () {
  'use strict';

  var STORAGE_KEY = 'sd-sb';

  function getNav() {
    return document.querySelector('.sarde-sidebar-nav');
  }

  function groupToggle(group) {
    return group.querySelector('button[aria-expanded]');
  }

  function saveState() {
    try {
      var nav = getNav();
      if (!nav) return;
      var hash = nav.getAttribute('data-sd-hash');
      if (!hash) return;
      var groups = nav.querySelectorAll('[data-sd-idx]');
      var open = [];
      for (var i = 0; i < groups.length; i++) {
        var idx = parseInt(groups[i].getAttribute('data-sd-idx'), 10);
        var btn = groupToggle(groups[i]);
        if (idx >= 0 && btn) open[idx] = btn.getAttribute('aria-expanded') === 'true';
      }
      sessionStorage.setItem(STORAGE_KEY, JSON.stringify({
        hash: hash,
        open: open,
        scroll: nav.scrollTop
      }));
    } catch (e) {}
  }

  var nav = getNav();
  if (!nav) return;

  nav.addEventListener('click', function (e) {
    var btn = e.target.closest('button[aria-controls]');
    if (!btn || !nav.contains(btn)) return;
    var list = document.getElementById(btn.getAttribute('aria-controls'));
    if (!list) return;
    var expanded = btn.getAttribute('aria-expanded') === 'true';
    btn.setAttribute('aria-expanded', expanded ? 'false' : 'true');
    if (expanded) {
      list.setAttribute('hidden', '');
    } else {
      list.removeAttribute('hidden');
    }
    saveState();
  });

  document.addEventListener('visibilitychange', function () {
    if (document.visibilityState === 'hidden') saveState();
  });

  window.addEventListener('pagehide', saveState);
})();

})();
/* sidebar-toggle.js */
;(function(){
(function () {
  'use strict';

  var STORAGE_KEY = 'sd-sidebar';
  var BREAKPOINT = 1024;
  var CLASS_COLLAPSED = 'sd-sidebar-collapsed';

  var btn = document.getElementById('sarde-sidebar-toggle');
  if (!btn) return;

  var mq = window.matchMedia('(min-width: ' + BREAKPOINT + 'px)');

  function isCollapsed() {
    return document.documentElement.classList.contains(CLASS_COLLAPSED);
  }

  var i18n = (window.__SARDE__ && window.__SARDE__.i18n) || {};
  var LABEL_EXPAND = i18n.expandSidebar || 'Expand sidebar';
  var LABEL_COLLAPSE = i18n.collapseSidebar || 'Collapse sidebar';

  function setAria(collapsed) {
    btn.setAttribute('aria-expanded', collapsed ? 'false' : 'true');
    btn.setAttribute('aria-label', collapsed ? LABEL_EXPAND : LABEL_COLLAPSE);
  }

  function collapse() {
    document.documentElement.classList.add(CLASS_COLLAPSED);
    setAria(true);
    try { localStorage.setItem(STORAGE_KEY, 'collapsed'); } catch (e) {}
  }

  function expand() {
    document.documentElement.classList.remove(CLASS_COLLAPSED);
    setAria(false);
    try { localStorage.setItem(STORAGE_KEY, 'expanded'); } catch (e) {}
  }

  function toggle() {
    if (isCollapsed()) {
      expand();
    } else {
      collapse();
    }
  }

  setAria(isCollapsed());

  btn.addEventListener('click', toggle);

  var sidebar = document.getElementById('sarde-sidebar');
  if (sidebar) {
    sidebar.addEventListener('click', function (e) {
      if (isCollapsed() && !btn.contains(e.target)) {
        expand();
      }
    });
  }

  function onBreakpoint(e) {
    if (!e.matches) {
      document.documentElement.classList.remove(CLASS_COLLAPSED);
    } else {
      try {
        if (localStorage.getItem(STORAGE_KEY) === 'collapsed') {
          document.documentElement.classList.add(CLASS_COLLAPSED);
          setAria(true);
        }
      } catch (e) {}
    }
  }

  if (mq.addEventListener) {
    mq.addEventListener('change', onBreakpoint);
  } else {
    mq.addListener(onBreakpoint);
  }
})();

})();
/* spoiler.js */
;(function(){
(function () {
  'use strict';
  var i18n = (window.__SARDE__ && window.__SARDE__.i18n) || {};
  var LABEL_REVEAL = i18n.revealSpoiler || 'Reveal spoiler';
  var LABEL_HIDE = i18n.hideSpoiler || 'Hide spoiler';
  function toggle(el) {
    var revealed = el.classList.toggle('revealed');
    el.setAttribute('aria-label', revealed ? LABEL_HIDE : LABEL_REVEAL);
    el.setAttribute('aria-expanded', revealed ? 'true' : 'false');
  }
  // The server-rendered initial label is English; localize it at load.
  document.querySelectorAll('.sarde-spoiler:not(.revealed)').forEach(function (el) {
    el.setAttribute('aria-label', LABEL_REVEAL);
  });
  document.addEventListener('click', function (e) {
    var s = e.target.closest && e.target.closest('.sarde-spoiler');
    if (s) toggle(s);
  });
  document.addEventListener('keydown', function (e) {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    var s = e.target.closest && e.target.closest('.sarde-spoiler');
    if (s) {
      e.preventDefault();
      toggle(s);
    }
  });
})();

})();
/* tabs.js */
;(function(){
(function () {
  'use strict';
  var STORAGE_KEY = 'sarde-tabs';

  function getStore() {
    try { return JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}'); } catch (e) { return {}; }
  }

  function saveLabel(label) {
    var s = getStore();
    s[label] = 1;
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(s)); } catch (e) {}
  }

  function activateTab(container, btnSel, panelSel, label) {
    var buttons = container.querySelectorAll(btnSel);
    var found = false;
    buttons.forEach(function (b) {
      if (b.dataset.tabLabel === label) { found = true; }
    });
    if (!found) return;
    buttons.forEach(function (b) {
      var match = b.dataset.tabLabel === label;
      b.setAttribute('aria-selected', match ? 'true' : 'false');
      b.classList.toggle('is-active', match);
    });
    container.querySelectorAll(panelSel).forEach(function (p) {
      var match = p.dataset.tabLabel === label;
      p.classList.toggle('is-active', match);
      if (match) p.removeAttribute('hidden'); else p.setAttribute('hidden', '');
    });
  }

  function syncAll(label) {
    document.querySelectorAll('.sarde-tabs').forEach(function (c) {
      activateTab(c, '.sarde-tab-button', '.sarde-tab-panel', label);
    });

  }

  document.addEventListener('click', function (e) {
    if (!e.target.matches) return;
    if (e.target.matches('.sarde-tabs .sarde-tab-button')) {
      var label = e.target.dataset.tabLabel;
      if (label) {
        saveLabel(label);
        syncAll(label);
      }
    }
  });

  var stored = getStore();
  var labels = Object.keys(stored);
  for (var i = 0; i < labels.length; i++) {
    syncAll(labels[i]);
  }
})();

})();
/* theme-toggle.js */
;(function(){
(function () {
  var STORAGE_KEY = 'sd-theme';

  function getTheme() {
    return localStorage.getItem(STORAGE_KEY) || 'system';
  }

  function isDark(theme) {
    return theme === 'dark' || (theme === 'system' && matchMedia('(prefers-color-scheme: dark)').matches);
  }

  function applyTheme(theme) {
    var root = document.documentElement;
    root.classList.add('sd-no-transition');
    root.setAttribute('data-theme', isDark(theme) ? 'dark' : 'light');
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        root.classList.remove('sd-no-transition');
      });
    });
  }

  function syncToggles(theme) {
    document.querySelectorAll('[data-sarde-theme-toggle]').forEach(function (toggle) {
      var indicator = toggle.querySelector('[data-sarde-theme-indicator]');
      toggle.querySelectorAll('.sarde-theme-toggle-btn').forEach(function (btn) {
        var active = btn.getAttribute('data-theme') === theme;
        btn.classList.toggle('selected', active);
        btn.setAttribute('aria-pressed', active);
      });
      if (indicator) {
        indicator.classList.toggle('pos-system', theme === 'system');
        indicator.classList.toggle('pos-dark', theme === 'dark');
      }
    });
  }

  function setTheme(theme) {
    localStorage.setItem(STORAGE_KEY, theme);
    applyTheme(theme);
    syncToggles(theme);
  }

  document.addEventListener('click', function (e) {
    var btn = e.target.closest('.sarde-theme-toggle-btn');
    if (btn) setTheme(btn.getAttribute('data-theme'));
  });

  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function () {
    applyTheme(getTheme());
  });

  var theme = getTheme();
  applyTheme(theme);
  syncToggles(theme);
})();

})();
/* toc-scrollspy.js */
;(function(){
(function () {
  'use strict';

  var desktopLinks = document.querySelectorAll('.sarde-toc-nav a');
  var mobileLinks = document.querySelectorAll('.sarde-mobile-toc-link');
  var mobileToc = document.getElementById('sarde-mobile-toc');
  var currentSpan = mobileToc ? mobileToc.querySelector('.sarde-mobile-toc-current') : null;

  var headingIds = [];
  var seen = {};
  [].forEach.call(desktopLinks, function (link) {
    var id = (link.getAttribute('href') || '').replace('#', '');
    if (id && id !== '_top' && !seen[id]) { seen[id] = true; headingIds.push(id); }
  });
  [].forEach.call(mobileLinks, function (link) {
    var id = (link.getAttribute('href') || '').replace('#', '');
    if (id && id !== '_top' && !seen[id]) { seen[id] = true; headingIds.push(id); }
  });

  if (headingIds.length === 0) return;

  var firstHeadingId = headingIds[0];
  var lastHeadingId = headingIds[headingIds.length - 1];

  // ── Narrow 53px detection strip just below the nav ────────────────
  function getStripRootMargin() {
    var header = document.querySelector('.sarde-header');
    var navHeight = header ? header.offsetHeight : 64;
    var mobileTocH = (window.innerWidth < 1280 && mobileToc) ? 48 : 0;
    var topOffset = navHeight + mobileTocH + 32;
    var bottomOffset = topOffset + 53 - window.innerHeight;
    return '-' + topOffset + 'px 0% ' + bottomOffset + 'px';
  }

  // ── Attribute a content element to its nearest preceding heading ───
  function getElementHeading(el) {
    var current = el;
    while (current) {
      if (/^H[2-6]$/.test(current.nodeName) && current.id) return current;
      var prev = current.previousElementSibling;
      while (prev) {
        if (/^H[2-6]$/.test(prev.nodeName) && prev.id) return prev;
        var last = prev.lastElementChild;
        while (last) {
          if (/^H[2-6]$/.test(last.nodeName) && last.id) return last;
          last = last.lastElementChild;
        }
        prev = prev.previousElementSibling;
      }
      current = current.parentElement;
    }
    return null;
  }

  // ── Set the active TOC link ───────────────────────────────────────
  function setActive(id) {
    [].forEach.call(desktopLinks, function (link) {
      var isActive = link.getAttribute('href') === '#' + id;
      link.classList.toggle('active', isActive);
      link.setAttribute('aria-current', isActive ? 'true' : 'false');
    });

    var activeText = '';
    [].forEach.call(mobileLinks, function (link) {
      var isActive = link.getAttribute('href') === '#' + id;
      link.setAttribute('aria-current', isActive ? 'true' : 'false');
      if (isActive) activeText = link.textContent.trim();
    });
    if (currentSpan) currentSpan.textContent = activeText;
  }

  // ── Collect all elements to observe ───────────────────────────────
  var elementsToObserve = [];

  function collectElements() {
    elementsToObserve = [];
    var content = document.querySelector('article.sarde-markdown-content');
    if (!content) {
      headingIds.forEach(function (id) {
        var el = document.getElementById(id);
        if (el) elementsToObserve.push(el);
      });
      return;
    }
    var children = content.children;
    for (var i = 0; i < children.length; i++) {
      elementsToObserve.push(children[i]);
    }
    var deepHeadings = content.querySelectorAll('h2[id], h3[id], h4[id], h5[id], h6[id]');
    [].forEach.call(deepHeadings, function (h) {
      if (h.parentElement !== content) {
        elementsToObserve.push(h);
        var sibling = h.nextElementSibling;
        while (sibling) {
          if (/^H[2-6]$/.test(sibling.nodeName)) break;
          elementsToObserve.push(sibling);
          sibling = sibling.nextElementSibling;
        }
      }
    });
  }

  // ── IntersectionObserver ──────────────────────────────────────────
  var observer = null;

  function observerCallback(entries) {
    for (var i = 0; i < entries.length; i++) {
      if (entries[i].isIntersecting) {
        var heading = getElementHeading(entries[i].target);
        if (heading && heading.id) {
          setActive(heading.id);
        } else {
          setActive('_top');
        }
        break;
      }
    }
  }

  function buildObserver() {
    if (observer) observer.disconnect();
    observer = new IntersectionObserver(observerCallback, {
      rootMargin: getStripRootMargin(),
      threshold: 0
    });
    elementsToObserve.forEach(function (el) { observer.observe(el); });
  }

  // ── Resize: rebuild observer (rootMargin depends on header height)
  var resizeTimer = null;
  window.addEventListener('resize', function () {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(buildObserver, 200);
  }, { passive: true });

  // ── Smooth scroll on TOC link click ───────────────────────────────
  function getNavOffset() {
    var header = document.querySelector('.sarde-header');
    var base = header ? header.offsetHeight : 64;
    var mobileTocH = (window.innerWidth < 1024 && mobileToc) ? 48 : 0;
    return base + mobileTocH + 8;
  }

  function scrollBehavior() {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth';
  }

  [].forEach.call(desktopLinks, function (link) {
    link.addEventListener('click', function (e) {
      e.preventDefault();
      var id = this.getAttribute('href').replace('#', '');
      if (id === '_top') {
        window.scrollTo({ top: 0, behavior: scrollBehavior() });
        history.replaceState(null, '', window.location.pathname);
        setActive('_top');
      } else {
        var el = document.getElementById(id);
        if (el) {
          var rect = el.getBoundingClientRect();
          window.scrollTo({ top: rect.top + window.scrollY - getNavOffset(), behavior: scrollBehavior() });
        }
        history.replaceState(null, '', '#' + id);
      }
    });
  });

  // ── Mobile TOC behavior ───────────────────────────────────────────
  if (mobileToc) {
    [].forEach.call(mobileLinks, function (link) {
      link.addEventListener('click', function () {
        setTimeout(function () { mobileToc.open = false; }, 0);
      });
    });

    document.addEventListener('click', function (e) {
      if (mobileToc.open && !mobileToc.contains(e.target)) {
        mobileToc.open = false;
      }
    });

    document.addEventListener('keydown', function (e) {
      if (e.key === 'Escape' && mobileToc.open) {
        mobileToc.open = false;
        var summary = mobileToc.querySelector('summary');
        if (summary) summary.focus();
      }
    });
  }

  // ── Circular progress indicator ───────────────────────────────────
  var progressFill = mobileToc ? mobileToc.querySelector('.sarde-mobile-toc-progress-fill') : null;
  var circumference = 2 * Math.PI * 8;
  if (progressFill) {
    progressFill.style.strokeDasharray = circumference;
    progressFill.style.strokeDashoffset = circumference;
  }

  var scrollRaf = null;
  window.addEventListener('scroll', function () {
    if (scrollRaf) return;
    scrollRaf = requestAnimationFrame(function () {
      scrollRaf = null;
      if (progressFill) {
        var scrollTop = window.scrollY;
        var docHeight = document.documentElement.scrollHeight - window.innerHeight;
        var progress = docHeight > 0 ? Math.min(scrollTop / docHeight, 1) : 0;
        progressFill.style.strokeDashoffset = circumference * (1 - progress);
      }
      var atBottom = (window.innerHeight + window.scrollY) >=
        (document.documentElement.scrollHeight - 50);
      if (atBottom) setActive(lastHeadingId);
    });
  }, { passive: true });

  // ── Init (deferred to avoid blocking first paint) ─────────────────
  function init() {
    setActive('_top');
    collectElements();
    buildObserver();
  }

  if ('requestIdleCallback' in window) {
    requestIdleCallback(init);
  } else {
    setTimeout(init, 16);
  }
})();

})();
/* version-switcher.js */
;(function(){
document.addEventListener('click', (e) => {
  const trigger = e.target.closest('[data-sarde-version-switcher-trigger]');
  if (trigger) {
    const menu = trigger.nextElementSibling;
    const expanded = trigger.getAttribute('aria-expanded') === 'true';
    trigger.setAttribute('aria-expanded', !expanded);
    menu.hidden = expanded;
    return;
  }
  document.querySelectorAll('[data-sarde-version-switcher-trigger][aria-expanded="true"]').forEach(t => {
    t.setAttribute('aria-expanded', 'false');
    t.nextElementSibling.hidden = true;
  });
});

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    document.querySelectorAll('[data-sarde-version-switcher-trigger][aria-expanded="true"]').forEach(t => {
      t.setAttribute('aria-expanded', 'false');
      t.nextElementSibling.hidden = true;
      t.focus();
    });
  }
});

// Close when keyboard focus leaves the trigger/menu entirely (Tab-away),
// so aria-expanded never lies while focus is elsewhere on the page.
document.addEventListener('focusout', (e) => {
  document.querySelectorAll('[data-sarde-version-switcher-trigger][aria-expanded="true"]').forEach(t => {
    const menu = t.nextElementSibling;
    const next = e.relatedTarget;
    if (next && (t.contains(next) || (menu && menu.contains(next)))) return;
    t.setAttribute('aria-expanded', 'false');
    if (menu) menu.hidden = true;
  });
});

})();
