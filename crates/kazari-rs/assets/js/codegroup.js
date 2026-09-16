(function () {
  function init() {
    var groups = document.querySelectorAll(".kz-group");
    var syncing = false;

    function findTabByLabel(tabs, label) {
      for (var i = 0; i < tabs.length; i++) {
        if (tabs[i].textContent === label) return i;
      }
      return -1;
    }

    function syncOtherGroups(syncKey, label, sourceGroup) {
      groups.forEach(function (otherGroup) {
        if (otherGroup === sourceGroup) return;
        if (otherGroup.getAttribute("data-sync") !== syncKey) return;
        var otherTabs = otherGroup.querySelectorAll('.kz-group-tabs button[role="tab"]');
        var otherPanels = otherGroup.querySelectorAll('.kz-group-panels > [role="tabpanel"]');
        var matchIndex = findTabByLabel(otherTabs, label);
        if (matchIndex < 0) return;
        otherTabs.forEach(function (t, i) {
          var selected = i === matchIndex;
          t.setAttribute("aria-selected", selected ? "true" : "false");
          t.setAttribute("tabindex", selected ? "0" : "-1");
          if (otherPanels[i]) {
            if (selected) {
              otherPanels[i].removeAttribute("hidden");
            } else {
              otherPanels[i].setAttribute("hidden", "");
            }
          }
        });
      });
    }

    groups.forEach(function (group, gi) {
      var tabs = group.querySelectorAll('.kz-group-tabs button[role="tab"]');
      var panels = group.querySelectorAll('.kz-group-panels > [role="tabpanel"]');
      if (tabs.length === 0) return;

      // Assign IDs for aria-labelledby linking.
      tabs.forEach(function (tab, i) {
        var tabId = "kz-g" + gi + "-tab-" + i;
        var panelId = "kz-g" + gi + "-panel-" + i;
        tab.id = tabId;
        if (panels[i]) {
          panels[i].id = panelId;
          panels[i].setAttribute("aria-labelledby", tabId);
          tab.setAttribute("aria-controls", panelId);
        }
      });

      function activate(index, shouldSync) {
        tabs.forEach(function (t, i) {
          var selected = i === index;
          t.setAttribute("aria-selected", selected ? "true" : "false");
          t.setAttribute("tabindex", selected ? "0" : "-1");
          if (panels[i]) {
            if (selected) {
              panels[i].removeAttribute("hidden");
            } else {
              panels[i].setAttribute("hidden", "");
            }
          }
        });
        if (shouldSync) tabs[index].focus();

        var syncKey = group.getAttribute("data-sync");
        if (syncKey && shouldSync && !syncing) {
          var label = tabs[index].textContent;
          syncing = true;
          try {
            localStorage.setItem("kz-tabs-" + syncKey, label);
          } catch (e) {}
          syncOtherGroups(syncKey, label, group);
          syncing = false;
        }
      }

      // Click handler.
      tabs.forEach(function (tab, i) {
        tab.addEventListener("click", function () {
          activate(i, true);
        });
      });

      // Keyboard navigation.
      group.querySelector(".kz-group-tabs").addEventListener("keydown", function (e) {
        var current = Array.prototype.indexOf.call(tabs, document.activeElement);
        if (current < 0) return;
        var next = -1;
        if (e.key === "ArrowRight") {
          next = (current + 1) % tabs.length;
        } else if (e.key === "ArrowLeft") {
          next = (current - 1 + tabs.length) % tabs.length;
        } else if (e.key === "Home") {
          next = 0;
        } else if (e.key === "End") {
          next = tabs.length - 1;
        }
        if (next >= 0) {
          e.preventDefault();
          activate(next, true);
        }
      });

      // Restore synced tab from localStorage on load.
      var syncKey = group.getAttribute("data-sync");
      if (syncKey) {
        try {
          var saved = localStorage.getItem("kz-tabs-" + syncKey);
          if (saved) {
            var matchIndex = findTabByLabel(tabs, saved);
            if (matchIndex >= 0) {
              activate(matchIndex, false);
            }
          }
        } catch (e) {}
      }
    });
  }

  // The DOM may already be ready when this script runs, for example when it
  // is injected dynamically or loaded async after the page has parsed.
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
