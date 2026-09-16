(function () {
  var root = document.documentElement;
  var announcement = document.getElementById('showcase-announcement');

  function announce(message) {
    if (!announcement) return;
    announcement.textContent = '';
    window.setTimeout(function () { announcement.textContent = message; }, 10);
  }

  function fallbackCopy(text) {
    var textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    textarea.remove();
  }

  function setCopiedState(button, message) {
    if (!button) return;
    var label = button.querySelector('span');
    var icon = button.querySelector('svg');
    if (button._showcaseCopyTimer) window.clearTimeout(button._showcaseCopyTimer);
    if (button._showcaseOriginalLabel === undefined) {
      button._showcaseOriginalLabel = label ? label.textContent : '';
    }
    if (button._showcaseOriginalIcon === undefined) {
      button._showcaseOriginalIcon = icon ? icon.innerHTML : '';
    }
    button.classList.add('copied');
    if (label) label.textContent = message;
    if (icon) icon.innerHTML = '<path d="m5 12 4 4L19 6"/>';
    button._showcaseCopyTimer = window.setTimeout(function () {
      button.classList.remove('copied');
      if (label) label.textContent = button._showcaseOriginalLabel;
      if (icon) icon.innerHTML = button._showcaseOriginalIcon;
      button._showcaseCopyTimer = null;
    }, 2000);
  }

  function copyText(text, message, button) {
    function onSuccess() {
      announce(message);
      setCopiedState(button, message);
    }
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(onSuccess).catch(function () {
        fallbackCopy(text);
        onSuccess();
      });
      return;
    }
    fallbackCopy(text);
    onSuccess();
  }

  function applyTheme(isDark) {
    root.classList.toggle('dark', isDark);
    document.body.classList.toggle('dark', isDark);
    document.querySelectorAll('.kazari-tinted, .kazari-scoped, .kazari-customizer').forEach(function (element) {
      element.classList.toggle('dark', isDark);
    });
    try { localStorage.setItem('kz-demo-theme', isDark ? 'dark' : 'light'); } catch (e) {}
  }

  var saved;
  try { saved = localStorage.getItem('kz-demo-theme'); } catch (e) {}
  if (saved === 'dark') applyTheme(true);

  var themeToggle = document.getElementById('theme-toggle');
  if (themeToggle && themeToggle.tagName === 'BUTTON') {
    if (saved === 'dark') {
      themeToggle.setAttribute('aria-pressed', 'true');
      themeToggle.textContent = 'Light mode';
    }
    themeToggle.addEventListener('click', function () {
      var isDark = root.classList.toggle('dark');
      applyTheme(isDark);
      themeToggle.setAttribute('aria-pressed', String(isDark));
      themeToggle.textContent = isDark ? 'Light mode' : 'Dark mode';
      var navToggle = document.getElementById('dark-toggle');
      if (navToggle) navToggle.checked = isDark;
    });
  }

  var darkToggle = document.getElementById('dark-toggle');
  if (darkToggle) {
    if (saved === 'dark') darkToggle.checked = true;
    darkToggle.addEventListener('change', function () {
      var isDark = darkToggle.checked;
      applyTheme(isDark);
      if (themeToggle) {
        themeToggle.setAttribute('aria-pressed', String(isDark));
        themeToggle.textContent = isDark ? 'Light mode' : 'Dark mode';
      }
    });
  }

  document.querySelectorAll('[data-backend-link]').forEach(function (link) {
    link.addEventListener('click', function (event) {
      event.preventDefault();
      window.location.href = link.getAttribute('href').split('#')[0] + window.location.hash;
    });
  });

  var exampleJump = document.getElementById('example-jump');
  if (exampleJump) {
    exampleJump.addEventListener('change', function () {
      if (exampleJump.value) window.location.hash = exampleJump.value;
    });
  }

  document.querySelectorAll('[data-copy-link]').forEach(function (button) {
    button.addEventListener('click', function () {
      var id = button.getAttribute('data-copy-link');
      var url = window.location.href.split('#')[0] + '#' + id;
      copyText(url, 'Link copied', button);
    });
  });

  document.querySelectorAll('.copy-recipe').forEach(function (button) {
    button.addEventListener('click', function () {
      var code = button.parentElement.querySelector('code');
      var label = button.getAttribute('data-copy-label') || 'Recipe';
      if (code) copyText(code.textContent, label + ' copied', button);
    });
  });

  document.querySelectorAll('[data-recipe-group]').forEach(function (group, groupIndex) {
    var tabs = Array.prototype.slice.call(group.querySelectorAll('[data-recipe-tab]'));
    var panels = Array.prototype.slice.call(group.querySelectorAll('[data-recipe-panel]'));
    if (tabs.length < 2) return;

    function activate(index, focus) {
      tabs.forEach(function (tab, tabIndex) {
        var selected = tabIndex === index;
        tab.setAttribute('aria-selected', String(selected));
        tab.setAttribute('tabindex', selected ? '0' : '-1');
        panels[tabIndex].hidden = !selected;
      });
      if (focus) tabs[index].focus();
    }

    tabs.forEach(function (tab, index) {
      var tabID = 'showcase-recipe-' + groupIndex + '-tab-' + index;
      var panelID = 'showcase-recipe-' + groupIndex + '-panel-' + index;
      tab.id = tabID;
      tab.setAttribute('aria-controls', panelID);
      panels[index].id = panelID;
      panels[index].setAttribute('role', 'tabpanel');
      panels[index].setAttribute('aria-labelledby', tabID);
      tab.addEventListener('click', function () { activate(index, false); });
      tab.addEventListener('keydown', function (event) {
        var next = -1;
        if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
        if (event.key === 'ArrowLeft') next = (index - 1 + tabs.length) % tabs.length;
        if (event.key === 'Home') next = 0;
        if (event.key === 'End') next = tabs.length - 1;
        if (next >= 0) {
          event.preventDefault();
          activate(next, true);
        }
      });
    });
  });

  var searchInput = document.getElementById('example-search');
  var categorySelect = document.getElementById('category-filter');
  var clearButton = document.getElementById('clear-filters');
  var resultCount = document.getElementById('result-count');
  var noResults = document.getElementById('no-results');
  var examples = Array.prototype.slice.call(document.querySelectorAll('[data-example]'));
  var categories = Array.prototype.slice.call(document.querySelectorAll('[data-category]'));

  function applyFilters() {
    var query = searchInput ? searchInput.value.trim().toLowerCase() : '';
    var selectedCategory = categorySelect ? categorySelect.value : 'all';
    var visibleCount = 0;

    examples.forEach(function (example) {
      var category = example.closest('[data-category]').getAttribute('data-category');
      var matchesText = !query || example.getAttribute('data-search').indexOf(query) >= 0;
      var matchesCategory = selectedCategory === 'all' || category === selectedCategory;
      var visible = matchesText && matchesCategory;
      example.hidden = !visible;
      if (visible) visibleCount++;
      var navLink = document.querySelector('[data-example-link="' + example.id + '"]');
      if (navLink) navLink.hidden = !visible;
      var jumpOption = document.querySelector('[data-jump-example="' + example.id + '"]');
      if (jumpOption) jumpOption.hidden = !visible;
    });

    categories.forEach(function (category) {
      var categoryID = category.getAttribute('data-category');
      var hasVisibleExamples = category.querySelector('[data-example]:not([hidden])') !== null;
      category.hidden = !hasVisibleExamples;
      var navCategory = document.querySelector('[data-nav-category="' + categoryID + '"]');
      if (navCategory) navCategory.hidden = !hasVisibleExamples;
      var jumpCategory = document.querySelector('[data-jump-category="' + categoryID + '"]');
      if (jumpCategory) jumpCategory.hidden = !hasVisibleExamples;
    });

    if (resultCount) resultCount.textContent = visibleCount + (visibleCount === 1 ? ' example' : ' examples');
    if (noResults) noResults.hidden = visibleCount !== 0;
    updateScrollSpy();
  }

  if (searchInput) searchInput.addEventListener('input', applyFilters);
  if (categorySelect) categorySelect.addEventListener('change', applyFilters);
  if (clearButton) {
    clearButton.addEventListener('click', function () {
      searchInput.value = '';
      categorySelect.value = 'all';
      applyFilters();
      searchInput.focus();
    });
  }

  var demoNav = document.querySelector('.demo-nav');
  var scrollSpyLinks = new Map();
  document.querySelectorAll('[data-category-link], [data-example-link]').forEach(function (link) {
    scrollSpyLinks.set(link.getAttribute('data-category-link') || link.getAttribute('data-example-link'), link);
  });
  var scrollSpyTargets = Array.prototype.slice.call(document.querySelectorAll('[data-category], [data-example]'));
  var activeScrollSpyID = '';
  var scrollSpyScheduled = false;

  function targetIsVisible(target) {
    if (target.hidden) return false;
    var category = target.matches('[data-category]') ? target : target.closest('[data-category]');
    return !category || !category.hidden;
  }

  function keepNavLinkVisible(link) {
    if (!demoNav || !link || demoNav.scrollHeight <= demoNav.clientHeight) return;
    var linkTop = link.offsetTop;
    var linkBottom = linkTop + link.offsetHeight;
    var visibleTop = demoNav.scrollTop;
    var visibleBottom = visibleTop + demoNav.clientHeight;
    if (linkTop < visibleTop) demoNav.scrollTop = Math.max(0, linkTop - 12);
    if (linkBottom > visibleBottom) demoNav.scrollTop = linkBottom - demoNav.clientHeight + 12;
  }

  function setActiveScrollSpyTarget(target) {
    var targetID = target ? target.id : '';
    if (targetID === activeScrollSpyID) return;
    activeScrollSpyID = targetID;
    scrollSpyLinks.forEach(function (link) { link.removeAttribute('aria-current'); });
    document.querySelectorAll('[data-nav-category]').forEach(function (category) {
      category.classList.remove('is-active');
    });
    if (!target) {
      if (exampleJump) exampleJump.value = '';
      return;
    }

    var activeLink = scrollSpyLinks.get(targetID);
    if (activeLink) {
      activeLink.setAttribute('aria-current', 'location');
      keepNavLinkVisible(activeLink);
    }
    var category = target.matches('[data-category]') ? target : target.closest('[data-category]');
    if (category) {
      var navCategory = document.querySelector('[data-nav-category="' + category.id + '"]');
      if (navCategory) navCategory.classList.add('is-active');
    }
    if (exampleJump) {
      exampleJump.value = target.matches('[data-example]') ? '#' + targetID : '';
    }
  }

  function updateScrollSpy() {
    scrollSpyScheduled = false;
    var visibleTargets = scrollSpyTargets.filter(targetIsVisible);
    if (!visibleTargets.length) {
      setActiveScrollSpyTarget(null);
      return;
    }

    var header = document.querySelector('.demo-header');
    var headerHeight = header ? header.getBoundingClientRect().height : 0;
    var activationLine = headerHeight + Math.min(120, window.innerHeight * .2);
    var activeTarget = visibleTargets[0];
    visibleTargets.forEach(function (target) {
      if (target.getBoundingClientRect().top <= activationLine) activeTarget = target;
    });
    if (window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2) {
      activeTarget = visibleTargets[visibleTargets.length - 1];
    }
    setActiveScrollSpyTarget(activeTarget);
  }

  function scheduleScrollSpyUpdate() {
    if (scrollSpyScheduled) return;
    scrollSpyScheduled = true;
    window.requestAnimationFrame(updateScrollSpy);
  }

  window.addEventListener('scroll', scheduleScrollSpyUpdate, { passive: true });
  window.addEventListener('resize', scheduleScrollSpyUpdate);
  updateScrollSpy();
})();

(function() {
  var btn = document.querySelector('.back-to-top');
  if (!btn) return;
  var footer = document.querySelector('.site-footer');
  var threshold = 300;
  var visible = false;
  window.addEventListener('scroll', function() {
    var pastTop = window.scrollY > threshold;
    var atFooter = footer && (window.innerHeight + window.scrollY >= footer.offsetTop);
    var show = pastTop && !atFooter;
    if (show !== visible) {
      visible = show;
      btn.classList.toggle('is-visible', show);
    }
  }, { passive: true });
  btn.addEventListener('click', function() {
    window.scrollTo({ top: 0, behavior: 'smooth' });
  });
})();
