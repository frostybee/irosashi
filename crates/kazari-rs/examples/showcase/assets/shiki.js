var toggle = document.getElementById("dark-toggle");
var status = document.getElementById("shiki-status");

var saved; try { saved = localStorage.getItem("kz-demo-theme"); } catch (e) {}
if (saved === "dark") {
  document.body.classList.add("dark");
  document.documentElement.classList.add("dark");
  toggle.checked = true;
}

toggle.addEventListener("change", function () {
  var dark = toggle.checked;
  document.body.classList.toggle("dark", dark);
  document.documentElement.classList.toggle("dark", dark);
  try { localStorage.setItem("kz-demo-theme", dark ? "dark" : "light"); } catch (e) {}
  if (window.__shikiHL) renderAll(window.__shikiHL, dark ? "github-dark" : "github-light");
});

function renderAll(hl, theme) {
  document.querySelectorAll(".shiki-target").forEach(function (target) {
    var lang = target.dataset.lang;
    var id = target.id.replace("shiki-", "");
    var src = document.getElementById("src-" + id);
    if (!src) return;
    var code = src.textContent.replace(/<\\\/script>/g, "<\/script>");
    target.innerHTML = hl.codeToHtml(code, { lang: lang, theme: theme });
  });
}

async function init() {
  try {
    var m = await import("https://esm.sh/shiki@4");
    var hl = await m.createHighlighter({
      themes: ["github-light", "github-dark"],
      langs: ["go", "javascript", "typescript", "python", "bash", "php", "css", "html"],
    });
    window.__shikiHL = hl;
    renderAll(hl, toggle.checked ? "github-dark" : "github-light");
    status.textContent = "Shiki ready.";
    status.className = "cmp-status ready";
  } catch (err) {
    status.textContent = "Shiki CDN load failed: " + err.message;
    status.className = "cmp-status error";
    console.error(err);
  }
}
init();
