// One URL and two surfaces.
//
// The document at `/` is the static page that Typst generates from `ssg/`. This
// script upgrades the page in place. If the browser can run WebAssembly, the
// Ratzilla app replaces the page at the same URL. The site has no `/tui`, no
// `/static` and no second host. If the browser cannot run WebAssembly, or if
// scripting is off, the page stays as it is.
//
// The script swaps the page only after the module of the app has loaded. A
// failed download therefore leaves the reader on the page and not on a black
// screen.
(function () {
  "use strict";

  var canRun =
    "WebAssembly" in window && typeof WebAssembly.instantiate === "function";
  if (!canRun) {
    return;
  }

  var shell = document.getElementById("app-shell");
  if (!shell) {
    return;
  }

  function link(href) {
    var element = document.createElement("link");
    element.rel = "stylesheet";
    element.href = href;
    document.head.appendChild(element);
  }

  function script(src) {
    var element = document.createElement("script");
    element.src = src;
    element.defer = true;
    document.head.appendChild(element);
  }

  // The app draws the whole viewport. The code removes everything that the page
  // brought with it, including its header, its footer and the source-link bar of
  // thyme. Only the surfaces of the app stay. If both remained, the mirrored text
  // layer would appear twice and selection would be ambiguous.
  function swap() {
    Array.prototype.forEach.call(document.body.children, function (element) {
      if (element !== shell) {
        element.remove();
      }
    });
    shell.hidden = false;
    link("/app.css");
    script("/cursor.js");
  }

  import("/apex.js")
    .then(function (module) {
      swap();
      // The glue module takes the name of the crate. An explicit wasm path keeps
      // this code independent of how `import.meta.url` resolves behind a proxy.
      return module.default("/apex_bg.wasm");
    })
    .catch(function (error) {
      // Leave the page alone. The static version is the fallback and not an error.
      console.warn("apex: the app did not start, staying on the plain page", error);
    });
})();
