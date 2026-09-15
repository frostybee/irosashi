(function () {
  var STORAGE_PREFIX = "kz-theme-";

  function getBlockId(block) {
    return block.getAttribute("data-kz-id") || "";
  }

  function isPageDark(btn) {
    var mode = btn.getAttribute("data-kz-dark-mode");
    if (mode === "media") {
      return window.matchMedia("(prefers-color-scheme: dark)").matches;
    }
    var sel = btn.getAttribute("data-kz-dark-selector") || ".dark";
    return document.documentElement.matches(sel) || document.body.matches(sel);
  }

  function loadState(id) {
    if (!id) return "";
    try {
      return localStorage.getItem(STORAGE_PREFIX + id) || "";
    } catch (e) {
      return "";
    }
  }

  function saveState(id, state) {
    if (!id) return;
    try {
      if (state) {
        localStorage.setItem(STORAGE_PREFIX + id, state);
      } else {
        localStorage.removeItem(STORAGE_PREFIX + id);
      }
    } catch (e) {}
  }

  function applyState(block, btn, state) {
    if (state) {
      block.setAttribute("data-kz-theme", state);
    } else {
      block.removeAttribute("data-kz-theme");
    }
    btn.setAttribute("aria-pressed", state ? "true" : "false");
    var label = state
      ? btn.getAttribute("data-toggled")
      : btn.getAttribute("data-label");
    btn.setAttribute("aria-label", label);
    btn.setAttribute("data-tooltip", label);
    var announce = block.querySelector(".kz-sr-announce");
    if (announce && state !== null) {
      announce.textContent = btn.getAttribute("data-announcement") || "";
      setTimeout(function () {
        announce.textContent = "";
      }, 1000);
    }
  }

  function init() {
    var buttons = document.querySelectorAll(".kz-theme-toggle-btn");
    buttons.forEach(function (btn) {
      var block = btn.closest(".kazari-block");
      if (!block) return;

      var id = getBlockId(block);
      var saved = loadState(id);
      if (saved) {
        applyState(block, btn, saved);
      }

      btn.addEventListener("click", function (e) {
        e.stopPropagation();
        var current = block.getAttribute("data-kz-theme");
        var next;
        if (current) {
          next = "";
        } else {
          next = isPageDark(btn) ? "light" : "dark";
        }
        applyState(block, btn, next);
        saveState(getBlockId(block), next);
      });
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
