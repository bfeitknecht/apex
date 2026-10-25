/**
 * Inline theme scripts, injected verbatim into the document.
 *
 * themeInit runs pre-paint in <head>: apply the stored override, else the
 * CSS falls back to the system preference via prefers-color-scheme.
 * themeToggle wires the header button at the end of <body>.
 */

export const themeInit = `
  (function () {
    try {
      var stored = localStorage.getItem("theme");
      if (stored === "dark" || stored === "light") {
        document.documentElement.setAttribute("data-theme", stored);
      }
    } catch {}
  })();
`;

export const themeToggle = `
  (function () {
    var root = document.documentElement;
    var button = document.getElementById("theme-toggle");
    if (!button) return;

    var paint = function () {
      var meta = document.querySelector('meta[name="theme-color"]');
      if (meta) {
        meta.setAttribute(
          "content",
          getComputedStyle(root).getPropertyValue("--bg").trim(),
        );
      }
    };

    paint();
    button.addEventListener("click", function () {
      var current = root.getAttribute("data-theme") ||
        (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
      var next = current === "dark" ? "light" : "dark";
      root.setAttribute("data-theme", next);
      try {
        localStorage.setItem("theme", next);
      } catch {}
      paint();
    });
  })();
`;
