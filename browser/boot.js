// One URL, two surfaces.
//
// The document that arrives at `/` is the static page Typst generates from
// `ssg/`. This script upgrades it in place: if the browser can run
// WebAssembly, the page is replaced by the Ratzilla app on the same URL —
// there is no `/tui`, no `/static`, no second host. If it cannot (or scripting
// is off, in which case this never runs), the page stays as it is.
//
// The swap happens only once the app's module is loaded, so a failed download
// leaves the reader with the page instead of a black screen.
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

  // The app draws the whole viewport: everything the page brought with it (its
  // own header/footer, thyme's source-link bar) goes, and only the app's
  // surfaces stay. Keeping both would duplicate the mirrored text layer and
  // make selection ambiguous.
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
      // The glue is named after the crate; naming the wasm explicitly keeps
      // this independent of how `import.meta.url` resolves behind a proxy.
      return module.default("/apex_bg.wasm");
    })
    .catch(function (error) {
      // Leave the page alone: the static version is the fallback, not an error.
      console.warn("apex: the app did not start, staying on the plain page", error);
    });
})();
