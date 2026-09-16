(function () {
  if (/Mac/.test(navigator.platform) || /Mac/.test(navigator.userAgent)) {
    document.documentElement.classList.add("is-mac");
  }

  var modal = document.getElementById("sarde-search-modal");
  var input = document.getElementById("sarde-search-input");
  var results = document.getElementById("sarde-search-results");
  var empty = document.getElementById("sarde-search-empty");
  var emptyTips = document.getElementById("sarde-search-empty-tips");
  var initial = document.getElementById("sarde-search-initial");
  var filtersEl = document.getElementById("sarde-search-filters");
  var triggers = document.querySelectorAll("[data-search-trigger]");
  if (!modal || !input || !results) return;

  var dbPromise = null;
  var _base = ((window.__SARDE__ && window.__SARDE__.basePath) || "/").replace(/\/$/, "");
  var selectedIndex = -1;
  var currentHits = [];
  var currentTotal = 0;
  var currentOffset = 0;
  var isLoading = false;
  var activeSection = "";
  var availableSections = [];
  var filtersRendered = false;
  var PAGE_SIZE = 30;
  var RECENT_KEY = "sarde-recent-searches:" + ((window.__SARDE__ && window.__SARDE__.siteId) || "default");
  var MAX_RECENTS = 5;
  var isFullMode = false;
  var panel = modal ? modal.querySelector(".sarde-search-modal-panel") : null;
  var previewEl = document.getElementById("sarde-search-preview");
  var modeToggleBtn = document.getElementById("sarde-search-mode-toggle");
  var MIN_TERM_LENGTH = 2;
  // This script is deferred, so every inline plugin config script has run.
  var pluginConfig = (window.__SARDE__ && window.__SARDE__.pluginConfig) || {};
  var searchCfg = pluginConfig.search || {};

  // --- i18n ---
  // The search plugin injects pluginConfig.search.strings per page language,
  // resolved through the site's i18n layers. Missing or unresolved keys (a
  // raw "search.*" key) fall back to English.
  var FALLBACK = {
    recent: "Recent",
    clear: "Clear",
    type_to_search: "Type to start searching",
    min_length: "Type at least {min} characters",
    filter_all: "All",
    try_all_sections: "Try searching in all sections",
    results_count_one: "{count} result for '{term}'",
    results_count_other: "{count} results for '{term}'",
    full_search: "Full Search",
    switch_full_search: "Switch to Full Search",
    simple_search: "Simple Search",
    switch_simple_search: "Switch to Simple Search",
    fuzzy_match: "Fuzzy match",
    preview: "Preview",
    open_page: "Open page",
    matches_in_page_one: "{count} match in this page",
    matches_in_page_other: "{count} matches in this page",
    matching_sections: "Matching sections",
    no_preview: "No preview available",
    group_other: "Other",
    loading: "Loading results"
  };

  function hasString(key) {
    var v = searchCfg.strings && searchCfg.strings[key];
    return typeof v === "string" && v !== "" && v.indexOf("search.") !== 0;
  }

  function str(key) {
    if (hasString(key)) return searchCfg.strings[key];
    return FALLBACK[key] || key;
  }

  function fmt(key, vars) {
    return str(key).replace(/\{(\w+)\}/g, function (m, name) {
      return Object.prototype.hasOwnProperty.call(vars, name) ? String(vars[name]) : m;
    });
  }

  // plural picks "<base>_one" or "<base>_other" from the page language's
  // plural rules; locales whose rule yields another category (few, many,
  // zero) fall back to "_other".
  function plural(base, count, vars) {
    var form = count === 1 ? "one" : "other";
    try {
      if (window.Intl && Intl.PluralRules) {
        form = new Intl.PluralRules(document.documentElement.lang || undefined).select(count);
      }
    } catch (e) {}
    var key = base + "_" + form;
    if (!hasString(key) && !FALLBACK[key]) key = base + "_other";
    return fmt(key, vars);
  }

  // --- Result links ---
  // When the search_highlighter client plugin is active on this page (its
  // slug is present in pluginConfig), result links carry the query as ?q=
  // so the target page highlights the matches. The query is inserted before
  // any #fragment so it lands in location.search rather than the hash.
  function highlighterOn() {
    return !!pluginConfig.search_highlighter;
  }

  function withQuery(url, term) {
    if (!url || !highlighterOn()) return url;
    var q = (term || "").trim();
    if (!q) return url;
    var hashAt = url.indexOf("#");
    var base = hashAt >= 0 ? url.slice(0, hashAt) : url;
    var hash = hashAt >= 0 ? url.slice(hashAt) : "";
    if (/[?&]q=/.test(base)) return url;
    return base + (base.indexOf("?") >= 0 ? "&" : "?") + "q=" + encodeURIComponent(q) + hash;
  }

  var SEARCH_ICON = '<svg viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21l-4.3-4.3"/></g></svg>';
  var HISTORY_ICON = '<svg viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M3 12a9 9 0 1 0 9-9a9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5m4-1v5l4 2"/></g></svg>';
  var PAGE_ICON = '<svg viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z"/><path d="M14 2v5a1 1 0 0 0 1 1h5"/></g></svg>';
  var HASH_ICON = '<svg viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 9h16M4 15h16M10 3L8 21m8-18l-2 18"/></svg>';
  var FOLDER_ICON = '<svg viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>';
  var ARROW_ICON = '<svg viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m9 18l6-6l-6-6"/></svg>';
  var CHEVRON_SEP = '<svg class="sarde-breadcrumb-sep" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m9 18l6-6l-6-6"/></svg>';
  var COLUMNS_ICON = '<svg viewBox="0 0 24 24"><rect x="3" y="3" width="7" height="18" rx="1" fill="none" stroke="currentColor" stroke-width="2"/><rect x="14" y="3" width="7" height="18" rx="1" fill="none" stroke="currentColor" stroke-width="2"/></svg>';
  var LIST_ICON = '<svg viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg>';

  // --- Recent searches (localStorage) ---

  function getRecentSearches() {
    try {
      var data = JSON.parse(localStorage.getItem(RECENT_KEY));
      return Array.isArray(data) ? data.slice(0, MAX_RECENTS) : [];
    } catch (e) { return []; }
  }

  function saveRecentSearch(term) {
    try {
      var recents = getRecentSearches().filter(function (r) { return r !== term; });
      recents.unshift(term);
      localStorage.setItem(RECENT_KEY, JSON.stringify(recents.slice(0, MAX_RECENTS)));
    } catch (e) {}
  }

  function clearRecentSearches() {
    try { localStorage.removeItem(RECENT_KEY); } catch (e) {}
    renderInitialState();
  }

  function renderInitialState() {
    if (!initial) return;
    clearChildren(initial);
    var recents = getRecentSearches();
    if (recents.length) {
      var header = document.createElement("div");
      header.className = "sarde-search-recent-header";
      var label = document.createElement("span");
      label.textContent = str("recent");
      header.appendChild(label);
      var clearBtn = document.createElement("button");
      clearBtn.type = "button";
      clearBtn.className = "sarde-search-recent-clear";
      clearBtn.textContent = str("clear");
      clearBtn.addEventListener("click", function () { clearRecentSearches(); });
      header.appendChild(clearBtn);
      initial.appendChild(header);

      recents.forEach(function (term) {
        var btn = document.createElement("button");
        btn.type = "button";
        btn.className = "sarde-search-recent-item";
        btn.innerHTML = HISTORY_ICON + "<span>" + escapeHTML(term) + "</span>";
        btn.addEventListener("click", function () {
          input.value = term;
          lastTerm = term;
          runSearch(term);
        });
        initial.appendChild(btn);
      });
    } else {
      var fallback = document.createElement("div");
      fallback.className = "sarde-search-initial-fallback";
      fallback.innerHTML = SEARCH_ICON + "<p>" + escapeHTML(str("type_to_search")) + "</p>";
      initial.appendChild(fallback);
    }
  }

  // --- Index loading ---

  function loadIndex() {
    if (dbPromise) return dbPromise;
    // Base language subtag ("pt-BR" -> "pt") names the stemmer module the
    // build emitted for this lane; a missing module (unsupported language)
    // resolves to null and search falls back to default tokenization.
    var langBase = (searchLang || "en").toLowerCase().split(/[-_]/)[0];
    dbPromise = Promise.all([
      import(_base + "/assets/vendor/orama/orama.esm.js"),
      fetch(_base + "/search-index." + (searchLang || "en") + ".json").then(function (r) { return r.json(); }),
      import(_base + "/assets/vendor/orama/stemmers/" + langBase + ".js").catch(function () { return null; })
    ]).then(function (arr) {
      var orama = arr[0], docs = arr[1], langMod = arr[2];
      var schema = { id: "string", title: "string", url: "string", content: "string", description: "string", section: "enum", tags: "string[]", version: "enum", breadcrumb: "string", kind: "enum", anchor: "string" };
      var db;
      if (langMod && langMod.language && langMod.stemmer) {
        db = orama.create({
          schema: schema,
          components: {
            tokenizer: {
              language: langMod.language,
              stemming: true,
              stemmer: langMod.stemmer,
              stopWords: langMod.stopwords || undefined
            }
          }
        });
      } else {
        db = orama.create({ schema: schema });
      }
      orama.insertMultiple(db, docs);

      var sectionSet = {};
      docs.forEach(function (d) { if (d.section) sectionSet[d.section] = true; });
      availableSections = Object.keys(sectionSet).sort();
      renderFilterChips();

      return { orama: orama, db: db };
    });
    return dbPromise;
  }

  // --- Section filter chips ---

  function renderFilterChips() {
    if (!filtersEl || filtersRendered || !availableSections.length) return;
    filtersRendered = true;
    clearChildren(filtersEl);

    var allChip = document.createElement("button");
    allChip.type = "button";
    allChip.className = "sarde-search-filter-chip is-active";
    allChip.setAttribute("data-section", "");
    allChip.textContent = str("filter_all");
    allChip.addEventListener("click", function () { setActiveSection(""); });
    filtersEl.appendChild(allChip);

    availableSections.forEach(function (sec) {
      var chip = document.createElement("button");
      chip.type = "button";
      chip.className = "sarde-search-filter-chip";
      chip.setAttribute("data-section", sec);
      chip.textContent = sec.charAt(0).toUpperCase() + sec.slice(1);
      chip.addEventListener("click", function () { setActiveSection(sec); });
      filtersEl.appendChild(chip);
    });

    filtersEl.hidden = false;
  }

  function setActiveSection(section) {
    activeSection = section;
    if (filtersEl) {
      filtersEl.querySelectorAll(".sarde-search-filter-chip").forEach(function (chip) {
        chip.classList.toggle("is-active", chip.getAttribute("data-section") === section);
      });
    }
    var term = input.value.trim();
    if (term) {
      lastTerm = term;
      runSearch(term);
    }
  }

  // --- Open / Close ---

  function open() {
    if (modal.open) return;
    modal.showModal();
    input.focus();
    renderInitialState();
    if (initial) initial.hidden = false;
    results.hidden = true;
    hideEmpty();
    clearPreview();
    loadIndex();
  }

  function close() {
    if (modal.open) modal.close();
    input.value = "";
    clearChildren(results);
    results.hidden = true;
    hideEmpty();
    if (initial) initial.hidden = false;
    selectedIndex = -1;
    currentHits = [];
    currentTotal = 0;
    currentOffset = 0;
    isLoading = false;
    activeSection = "";
    isFullMode = false;
    if (panel) panel.classList.remove("is-full-mode");
    clearPreview();
    resetToggleButton();
    if (filtersEl) {
      filtersEl.querySelectorAll(".sarde-search-filter-chip").forEach(function (chip) {
        chip.classList.toggle("is-active", chip.getAttribute("data-section") === "");
      });
    }
  }

  // --- Event listeners ---

  triggers.forEach(function (t) {
    t.addEventListener("click", function (e) { e.preventDefault(); open(); });
  });

  modal.querySelector("[data-search-close]").addEventListener("click", function (e) {
    e.preventDefault(); close();
  });

  modal.addEventListener("cancel", function (e) {
    e.preventDefault();
    close();
  });

  modal.addEventListener("click", function (e) {
    if (e.target === modal) close();
  });

  document.addEventListener("keydown", function (e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "k") {
      e.preventDefault();
      open();
    } else if ((e.ctrlKey || e.metaKey) && e.key === " " && modal.open) {
      e.preventDefault();
      setFullMode(!isFullMode);
    }
  });

  results.addEventListener("scroll", function () {
    if (isLoading || currentOffset >= currentTotal) return;
    var scrollBottom = results.scrollHeight - results.scrollTop - results.clientHeight;
    if (scrollBottom < 200) {
      loadMore();
    }
  });

  var debounceId = null;
  var lastTerm = "";
  input.addEventListener("input", function () {
    clearTimeout(debounceId);
    var term = input.value.trim();
    lastTerm = term;
    debounceId = setTimeout(function () { runSearch(term); }, 120);
  });

  input.addEventListener("keydown", function (e) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (currentHits.length) {
        selectedIndex = Math.min(selectedIndex + 1, currentHits.length - 1);
        updateSelection();
        if (selectedIndex >= currentHits.length - 3 && currentOffset < currentTotal && !isLoading) {
          loadMore();
        }
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (currentHits.length) {
        selectedIndex = Math.max(selectedIndex - 1, 0);
        updateSelection();
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < currentHits.length) {
        var links = results.querySelectorAll(".sarde-search-result");
        if (links[selectedIndex]) {
          var href = links[selectedIndex].href;
          close();
          window.location.href = href;
        }
      }
    }
  });

  var searchVersion = modal.getAttribute("data-search-version") || "";
  var searchLang = modal.getAttribute("data-search-lang") || "";

  // --- Search ---

  function buildSearchOpts(term, offset) {
    var opts = {
      term: term,
      properties: ["title", "content", "description", "tags"],
      boost: { title: 5, tags: 2.5, description: 2 },
      limit: PAGE_SIZE,
      offset: offset,
      tolerance: 1
    };
    var where = {};
    if (searchVersion) where.version = { eq: searchVersion };
    if (activeSection) where.section = { eq: activeSection };
    if (Object.keys(where).length) opts.where = where;
    return opts;
  }

  function showMinLengthMessage() {
    if (!initial) return;
    clearChildren(initial);
    var msg = document.createElement("div");
    msg.className = "sarde-search-initial-fallback";
    msg.innerHTML = SEARCH_ICON + "<p>" + escapeHTML(fmt("min_length", { min: MIN_TERM_LENGTH })) + "</p>";
    initial.appendChild(msg);
    initial.hidden = false;
  }

  function runSearch(term) {
    if (!term) {
      clearChildren(results);
      results.hidden = true;
      hideEmpty();
      renderInitialState();
      if (initial) initial.hidden = false;
      selectedIndex = -1;
      currentHits = [];
      currentTotal = 0;
      currentOffset = 0;
      return;
    }
    if (term.length < MIN_TERM_LENGTH) {
      clearChildren(results);
      results.hidden = true;
      hideEmpty();
      showMinLengthMessage();
      selectedIndex = -1;
      currentHits = [];
      currentTotal = 0;
      currentOffset = 0;
      return;
    }
    if (initial) initial.hidden = true;
    currentOffset = 0;
    clearChildren(results);
    results.hidden = false;
    showSpinner(results);
    loadIndex().then(function (ctx) {
      return ctx.orama.search(ctx.db, buildSearchOpts(term, 0));
    }).then(function (res) {
      var hits = res && res.hits ? res.hits : [];
      currentTotal = res && res.count ? res.count : hits.length;
      currentHits = hits;
      currentOffset = hits.length;
      selectedIndex = -1;
      if (hits.length) saveRecentSearch(term);
      if (isFullMode) {
        renderFullMode(currentHits, term, currentTotal);
        if (currentHits.length) {
          selectedIndex = 0;
          updateSelection();
        }
      } else {
        render(currentHits, term, currentTotal);
      }
    }).catch(function () {
      clearChildren(results);
      currentHits = [];
      currentTotal = 0;
      currentOffset = 0;
      selectedIndex = -1;
      showEmpty();
    });
  }

  function loadMore() {
    var term = lastTerm;
    if (!term || currentOffset >= currentTotal || isLoading) return;
    isLoading = true;
    showSpinner(results);
    loadIndex().then(function (ctx) {
      return ctx.orama.search(ctx.db, buildSearchOpts(term, currentOffset));
    }).then(function (res) {
      removeSpinner();
      var hits = res && res.hits ? res.hits : [];
      currentHits = currentHits.concat(hits);
      currentOffset += hits.length;
      if (isFullMode) {
        appendCompactResults(hits, term);
      } else {
        appendResults(hits, term);
      }
      isLoading = false;
    }).catch(function () {
      removeSpinner();
      isLoading = false;
    });
  }

  // --- Empty state ---

  function cleanupEmptyAction() {
    if (!empty) return;
    var existing = empty.querySelector(".sarde-search-empty-action");
    if (existing) existing.remove();
  }

  function showEmpty() {
    if (!empty) return;
    results.hidden = true;
    cleanupEmptyAction();
    if (activeSection && emptyTips) {
      var action = document.createElement("button");
      action.type = "button";
      action.className = "sarde-search-empty-action";
      action.textContent = str("try_all_sections");
      action.addEventListener("click", function () { setActiveSection(""); });
      emptyTips.parentNode.insertBefore(action, emptyTips.nextSibling);
    }
    empty.hidden = false;
  }

  function hideEmpty() {
    if (!empty) return;
    cleanupEmptyAction();
    empty.hidden = true;
  }

  // --- Rendering ---

  function escapeHTML(s) {
    var div = document.createElement("div");
    div.textContent = s;
    return div.innerHTML;
  }

  function highlight(text, term) {
    if (!term) return escapeHTML(text);
    var escaped = term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    var re = new RegExp("(" + escaped + ")", "gi");
    return escapeHTML(text).replace(re, "<mark>$1</mark>");
  }

  function snippet(content, term, maxLen) {
    if (!content) return "";
    maxLen = maxLen || 120;
    var lower = content.toLowerCase();
    var tLower = term.toLowerCase();
    var idx = lower.indexOf(tLower);
    var text;
    if (idx === -1) {
      text = content.length > maxLen ? content.slice(0, maxLen) + "..." : content;
    } else {
      var start = Math.max(0, idx - Math.floor(maxLen / 2));
      var end = Math.min(content.length, start + maxLen);
      text = (start > 0 ? "..." : "") + content.slice(start, end) + (end < content.length ? "..." : "");
    }
    return highlight(text, term);
  }

  function countMatches(content, term) {
    if (!content || !term) return 0;
    var escaped = term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    var m = content.match(new RegExp(escaped, "gi"));
    return m ? m.length : 0;
  }

  function extractSnippets(content, term, windowLen, maxSnippets) {
    if (!content || !term) return [];
    windowLen = windowLen || 100;
    maxSnippets = maxSnippets || 5;
    var escaped = term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    var re = new RegExp(escaped, "gi");
    var match;
    var windows = [];
    while ((match = re.exec(content)) !== null) {
      var s = Math.max(0, match.index - Math.floor(windowLen / 2));
      var e = Math.min(content.length, s + windowLen);
      windows.push([s, e]);
    }
    if (!windows.length) return [];
    var merged = [windows[0]];
    for (var i = 1; i < windows.length; i++) {
      var last = merged[merged.length - 1];
      if (windows[i][0] <= last[1]) {
        last[1] = Math.max(last[1], windows[i][1]);
      } else {
        merged.push(windows[i]);
      }
    }
    if (merged.length > maxSnippets) merged = merged.slice(0, maxSnippets);
    return merged.map(function (w) {
      var text = (w[0] > 0 ? "..." : "") + content.slice(w[0], w[1]) + (w[1] < content.length ? "..." : "");
      return { text: text, startIdx: w[0] };
    });
  }

  function render(hits, term, totalCount) {
    clearChildren(results);
    if (!hits.length) {
      results.hidden = true;
      showEmpty();
      return;
    }
    hideEmpty();
    results.hidden = false;

    var header = document.createElement("li");
    header.className = "sarde-search-results-header";
    header.textContent = plural("results_count", totalCount, { count: totalCount, term: term });
    results.appendChild(header);

    var groups = {};
    var groupOrder = [];
    hits.forEach(function (h) {
      var d = h.document || h;
      var groupKey = d.section || str("group_other");
      if (!groups[groupKey]) {
        groups[groupKey] = [];
        groupOrder.push(groupKey);
      }
      groups[groupKey].push(d);
    });

    groupOrder.forEach(function (key) {
      var groupHeader = document.createElement("li");
      groupHeader.className = "sarde-search-group-header";
      groupHeader.setAttribute("data-search-group", key);
      groupHeader.innerHTML = '<span class="sarde-search-group-icon">' + FOLDER_ICON + "</span>" + escapeHTML(key);
      results.appendChild(groupHeader);

      groups[key].forEach(function (d) {
        results.appendChild(buildResultItem(d, term));
      });
    });
  }

  function buildResultItem(d, term) {
    var li = document.createElement("li");
    var a = document.createElement("a");
    a.className = "sarde-search-result";
    a.href = withQuery(d.url, term) || "#";

    var icon = document.createElement("span");
    icon.className = "sarde-search-result-icon";
    icon.innerHTML = d.kind === "heading" ? HASH_ICON : PAGE_ICON;
    a.appendChild(icon);

    var contentDiv = document.createElement("div");
    contentDiv.className = "sarde-search-result-content";

    var title = document.createElement("div");
    title.className = "sarde-search-result-title";
    title.innerHTML = highlight(d.title || d.url || "", term);
    contentDiv.appendChild(title);

    var bc = d.breadcrumb || d.section || "";
    if (bc) {
      var breadcrumb = document.createElement("div");
      breadcrumb.className = "sarde-search-result-breadcrumb";
      var parts = bc.split(" > ");
      var bcHTML = parts.map(function (p) { return highlight(p, term); }).join(CHEVRON_SEP);
      breadcrumb.innerHTML = bcHTML;
      contentDiv.appendChild(breadcrumb);
    }

    if (d.content || d.description) {
      var exc = document.createElement("div");
      exc.className = "sarde-search-result-excerpt";
      exc.innerHTML = d.content ? snippet(d.content, term) : highlight(d.description, term);
      contentDiv.appendChild(exc);
    }

    a.appendChild(contentDiv);

    var arrow = document.createElement("span");
    arrow.className = "sarde-search-result-arrow";
    arrow.innerHTML = ARROW_ICON;
    a.appendChild(arrow);

    li.appendChild(a);
    return li;
  }

  function appendResults(hits, term) {
    var groups = {};
    var groupOrder = [];
    hits.forEach(function (h) {
      var d = h.document || h;
      var groupKey = d.section || str("group_other");
      if (!groups[groupKey]) {
        groups[groupKey] = [];
        groupOrder.push(groupKey);
      }
      groups[groupKey].push(d);
    });

    groupOrder.forEach(function (key) {
      var existing = null;
      results.querySelectorAll(".sarde-search-group-header").forEach(function (el) {
        if (el.getAttribute("data-search-group") === key) existing = el;
      });
      if (!existing) {
        var groupHeader = document.createElement("li");
        groupHeader.className = "sarde-search-group-header";
        groupHeader.setAttribute("data-search-group", key);
        groupHeader.innerHTML = '<span class="sarde-search-group-icon">' + FOLDER_ICON + "</span>" + escapeHTML(key);
        results.appendChild(groupHeader);
      }
      groups[key].forEach(function (d) {
        results.appendChild(buildResultItem(d, term));
      });
    });
  }

  // --- Utilities ---

  function updateSelection() {
    var links = results.querySelectorAll(".sarde-search-result");
    links.forEach(function (el, i) {
      if (i === selectedIndex) {
        el.classList.add("is-selected");
        el.scrollIntoView({ block: "nearest" });
      } else {
        el.classList.remove("is-selected");
      }
    });
    if (isFullMode && selectedIndex >= 0) {
      updatePreview(selectedIndex);
    }
  }

  function showSpinner(container) {
    var li = document.createElement("li");
    li.className = "sarde-search-loading";
    li.setAttribute("role", "status");
    li.setAttribute("aria-label", str("loading"));
    li.innerHTML = '<div class="sarde-search-spinner"></div>';
    container.appendChild(li);
    return li;
  }

  function removeSpinner() {
    var el = results.querySelector(".sarde-search-loading");
    if (el) el.remove();
  }

  function clearChildren(node) {
    while (node.firstChild) node.removeChild(node.firstChild);
  }

  // --- Full search mode ---

  function resetToggleButton() {
    if (!modeToggleBtn) return;
    var iconEl = modeToggleBtn.querySelector("svg");
    var labelEl = modeToggleBtn.querySelector(".sarde-search-mode-label");
    if (iconEl) iconEl.outerHTML = COLUMNS_ICON;
    if (labelEl) labelEl.textContent = str("full_search");
    modeToggleBtn.setAttribute("aria-label", str("switch_full_search"));
  }

  function setFullMode(enabled) {
    isFullMode = enabled;
    if (panel) panel.classList.toggle("is-full-mode", enabled);

    if (modeToggleBtn) {
      var iconEl = modeToggleBtn.querySelector("svg");
      var labelEl = modeToggleBtn.querySelector(".sarde-search-mode-label");
      if (enabled) {
        if (iconEl) iconEl.outerHTML = LIST_ICON;
        if (labelEl) labelEl.textContent = str("simple_search");
        modeToggleBtn.setAttribute("aria-label", str("switch_simple_search"));
      } else {
        resetToggleButton();
      }
    }

    if (enabled) {
      if (currentHits.length) {
        renderFullMode(currentHits, lastTerm, currentTotal);
        selectedIndex = 0;
        updateSelection();
      } else {
        clearPreview();
      }
    } else {
      clearPreview();
      if (currentHits.length) {
        render(currentHits, lastTerm, currentTotal);
        updateSelection();
      }
    }
  }

  if (modeToggleBtn) {
    modeToggleBtn.addEventListener("click", function () {
      setFullMode(!isFullMode);
    });
  }

  function renderFullMode(hits, term, totalCount) {
    clearChildren(results);
    if (!hits.length) {
      results.hidden = true;
      showEmpty();
      return;
    }
    hideEmpty();
    results.hidden = false;

    var header = document.createElement("li");
    header.className = "sarde-search-results-header";
    header.textContent = plural("results_count", totalCount, { count: totalCount, term: term });
    results.appendChild(header);

    hits.forEach(function (h) {
      var d = h.document || h;
      results.appendChild(buildCompactResultItem(d, term));
    });
  }

  function buildCompactResultItem(d, term) {
    var li = document.createElement("li");
    var a = document.createElement("a");
    a.className = "sarde-search-result is-compact";
    a.href = withQuery(d.url, term) || "#";
    a.addEventListener("click", function (e) {
      if (isFullMode) {
        e.preventDefault();
        var allItems = results.querySelectorAll(".sarde-search-result");
        for (var i = 0; i < allItems.length; i++) {
          if (allItems[i] === a) { selectedIndex = i; break; }
        }
        updateSelection();
      }
    });

    var contentDiv = document.createElement("div");
    contentDiv.className = "sarde-search-result-content";

    var titleRow = document.createElement("div");
    titleRow.className = "sarde-search-compact-title-row";
    var titleEl = document.createElement("span");
    titleEl.className = "sarde-search-result-title";
    titleEl.innerHTML = highlight(d.title || d.url || "", term);
    titleRow.appendChild(titleEl);
    var mc = countMatches(d.content || "", term);
    if (mc > 0) {
      var badge = document.createElement("span");
      badge.className = "sarde-search-compact-badge";
      badge.textContent = mc;
      titleRow.appendChild(badge);
    } else if (d.content) {
      var fuzzBadge = document.createElement("span");
      fuzzBadge.className = "sarde-search-compact-badge sarde-search-compact-badge--fuzzy";
      fuzzBadge.textContent = "~";
      fuzzBadge.title = str("fuzzy_match");
      titleRow.appendChild(fuzzBadge);
    }
    contentDiv.appendChild(titleRow);

    var metaRow = document.createElement("div");
    metaRow.className = "sarde-search-compact-meta";
    var pathEl = document.createElement("span");
    pathEl.className = "sarde-search-result-compact-path";
    pathEl.textContent = d.url || "";
    metaRow.appendChild(pathEl);
    if (d.section) {
      var secEl = document.createElement("span");
      secEl.className = "sarde-search-compact-section";
      secEl.textContent = d.section.charAt(0).toUpperCase() + d.section.slice(1);
      metaRow.appendChild(secEl);
    }
    contentDiv.appendChild(metaRow);

    a.appendChild(contentDiv);
    li.appendChild(a);
    return li;
  }

  function appendCompactResults(hits, term) {
    hits.forEach(function (h) {
      var d = h.document || h;
      results.appendChild(buildCompactResultItem(d, term));
    });
  }

  function updatePreview(index) {
    if (!previewEl || !isFullMode) return;
    var h = currentHits[index];
    if (!h) { clearPreview(); return; }
    var doc = h.document || h;
    previewEl.removeAttribute("hidden");
    clearChildren(previewEl);

    var headerEl = document.createElement("div");
    headerEl.className = "sarde-search-preview-header";
    var headerLabel = document.createElement("span");
    headerLabel.textContent = str("preview");
    headerEl.appendChild(headerLabel);
    if (doc.url) {
      var openHref = withQuery(doc.url, lastTerm);
      var openLink = document.createElement("a");
      openLink.className = "sarde-search-preview-open";
      openLink.href = openHref;
      openLink.innerHTML = ARROW_ICON;
      openLink.setAttribute("aria-label", str("open_page"));
      openLink.title = str("open_page");
      openLink.addEventListener("click", function (e) {
        e.preventDefault();
        close();
        window.location.href = openHref;
      });
      headerEl.appendChild(openLink);
    }
    previewEl.appendChild(headerEl);

    var pathEl = document.createElement("div");
    pathEl.className = "sarde-search-preview-path";
    pathEl.textContent = doc.url || "";
    previewEl.appendChild(pathEl);

    var bc = doc.breadcrumb || doc.section || "";
    if (bc) {
      var bcEl = document.createElement("div");
      bcEl.className = "sarde-search-preview-breadcrumb";
      var parts = bc.split(" > ");
      bcEl.innerHTML = parts.map(function (p) { return highlight(p, lastTerm); }).join(CHEVRON_SEP);
      previewEl.appendChild(bcEl);
    }

    var titleEl = document.createElement("div");
    titleEl.className = "sarde-search-preview-title";
    titleEl.innerHTML = highlight(doc.title || doc.url || "", lastTerm);
    previewEl.appendChild(titleEl);

    var tags = doc.tags;
    if (tags && tags.length) {
      var tagsEl = document.createElement("div");
      tagsEl.className = "sarde-search-preview-tags";
      tags.forEach(function (tag) {
        var pill = document.createElement("span");
        pill.className = "sarde-search-preview-tag";
        pill.innerHTML = highlight(tag, lastTerm);
        tagsEl.appendChild(pill);
      });
      previewEl.appendChild(tagsEl);
    }

    var mc = countMatches(doc.content || "", lastTerm);
    if (mc > 0) {
      var mcEl = document.createElement("div");
      mcEl.className = "sarde-search-preview-match-count";
      mcEl.textContent = plural("matches_in_page", mc, { count: mc });
      previewEl.appendChild(mcEl);
    }

    var baseUrl = (doc.url || "").split("#")[0];
    var headings = [];
    currentHits.forEach(function (hit) {
      var d = hit.document || hit;
      if (d.kind === "heading" && d.url && d.url.indexOf(baseUrl + "#") === 0) {
        headings.push(d);
      }
    });
    if (headings.length) {
      var hdSection = document.createElement("div");
      hdSection.className = "sarde-search-preview-headings";
      var hdLabel = document.createElement("div");
      hdLabel.className = "sarde-search-preview-headings-label";
      hdLabel.textContent = str("matching_sections");
      hdSection.appendChild(hdLabel);
      headings.forEach(function (hd) {
        var hdHref = withQuery(hd.url, lastTerm);
        var link = document.createElement("a");
        link.className = "sarde-search-preview-heading";
        link.href = hdHref;
        link.innerHTML = HASH_ICON + "<span>" + highlight(hd.title || "", lastTerm) + "</span>";
        link.addEventListener("click", function (e) {
          e.preventDefault();
          close();
          window.location.href = hdHref;
        });
        hdSection.appendChild(link);
      });
      previewEl.appendChild(hdSection);
    }

    var content = doc.content || "";
    if (content) {
      var snippets = extractSnippets(content, lastTerm);
      if (snippets.length) {
        var snippetsEl = document.createElement("div");
        snippetsEl.className = "sarde-search-preview-snippets";
        snippets.forEach(function (s, i) {
          if (i > 0) {
            var sep = document.createElement("div");
            sep.className = "sarde-search-preview-snippet-sep";
            sep.textContent = "...";
            snippetsEl.appendChild(sep);
          }
          var snipEl = document.createElement("div");
          snipEl.className = "sarde-search-preview-snippet";
          snipEl.innerHTML = highlight(s.text, lastTerm);
          snippetsEl.appendChild(snipEl);
        });
        previewEl.appendChild(snippetsEl);
      } else {
        var fallbackEl = document.createElement("div");
        fallbackEl.className = "sarde-search-preview-excerpt";
        var preview = content.length > 300 ? content.slice(0, 300) + "..." : content;
        fallbackEl.textContent = preview;
        previewEl.appendChild(fallbackEl);
      }
    } else if (doc.description) {
      var descEl = document.createElement("div");
      descEl.className = "sarde-search-preview-excerpt";
      descEl.innerHTML = highlight(doc.description, lastTerm);
      previewEl.appendChild(descEl);
    } else {
      var emptyEl = document.createElement("div");
      emptyEl.className = "sarde-search-preview-empty";
      emptyEl.textContent = str("no_preview");
      previewEl.appendChild(emptyEl);
    }
  }

  function clearPreview() {
    if (!previewEl) return;
    previewEl.setAttribute("hidden", "");
    clearChildren(previewEl);
  }
})();
