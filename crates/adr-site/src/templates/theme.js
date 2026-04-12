(function () {
  const STORAGE_KEY = "adr-site-theme";
  const root = document.documentElement;
  const colorScheme = window.matchMedia("(prefers-color-scheme: dark)");

  function readStoredTheme() {
    try {
      const theme = localStorage.getItem(STORAGE_KEY);
      return theme === "light" || theme === "dark" ? theme : null;
    } catch (_) {
      return null;
    }
  }

  function activeTheme() {
    const explicit = root.getAttribute("data-theme");
    if (explicit === "light" || explicit === "dark") {
      return explicit;
    }

    return colorScheme.matches ? "dark" : "light";
  }

  function updateButton() {
    const button = document.querySelector("[data-theme-toggle]");
    if (!button) {
      return;
    }

    const theme = activeTheme();
    const nextTheme = theme === "dark" ? "light" : "dark";
    button.textContent = theme === "dark" ? "Light mode" : "Dark mode";
    button.setAttribute("aria-label", `Switch to ${nextTheme} mode`);
    button.setAttribute("title", `Switch to ${nextTheme} mode`);
  }

  function applyStoredTheme() {
    const theme = readStoredTheme();
    if (theme) {
      root.setAttribute("data-theme", theme);
    } else {
      root.removeAttribute("data-theme");
    }
    updateButton();
  }

  function toggleTheme() {
    const nextTheme = activeTheme() === "dark" ? "light" : "dark";
    try {
      localStorage.setItem(STORAGE_KEY, nextTheme);
    } catch (_) {
      // ignore storage errors and still apply for this page load
    }
    root.setAttribute("data-theme", nextTheme);
    updateButton();
  }

  document.addEventListener("DOMContentLoaded", function () {
    applyStoredTheme();

    const button = document.querySelector("[data-theme-toggle]");
    if (button) {
      button.addEventListener("click", toggleTheme);
    }
  });

  const handleSystemThemeChange = function () {
    if (!readStoredTheme()) {
      root.removeAttribute("data-theme");
      updateButton();
    }
  };

  if (typeof colorScheme.addEventListener === "function") {
    colorScheme.addEventListener("change", handleSystemThemeChange);
  } else if (typeof colorScheme.addListener === "function") {
    colorScheme.addListener(handleSystemThemeChange);
  }
})();
