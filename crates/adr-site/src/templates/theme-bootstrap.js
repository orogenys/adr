(function () {
  try {
    var theme = localStorage.getItem("adr-site-theme");
    if (theme === "light" || theme === "dark") {
      document.documentElement.setAttribute("data-theme", theme);
    }
  } catch (_) {
    // ignore storage errors during bootstrap
  }
})();
