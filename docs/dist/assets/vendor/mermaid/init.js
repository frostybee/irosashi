(function () {
  function run() {
    if (typeof window.mermaid === "undefined") return;
    var dark = document.documentElement.getAttribute("data-theme") === "dark";
    window.mermaid.initialize({
      startOnLoad: false,
      theme: dark ? "dark" : "default",
      securityLevel: "strict"
    });
    window.mermaid.run({
      querySelector: ".sarde-mermaid"
    });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", run);
  } else {
    run();
  }
})();
