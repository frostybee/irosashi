var toggle = document.getElementById("dark-toggle");
var saved; try { saved = localStorage.getItem("kz-demo-theme"); } catch (e) {}
if (saved === "dark") {
  document.body.classList.add("dark");
  document.documentElement.classList.add("dark");
  toggle.checked = true;
}
toggle.addEventListener("change", function () {
  var dark = this.checked;
  document.body.classList.toggle("dark", dark);
  document.documentElement.classList.toggle("dark", dark);
  try { localStorage.setItem("kz-demo-theme", dark ? "dark" : "light"); } catch (e) {}
});
