// One URL and two versions: static and dynamic.
//
// The document at `/` is the static version that Typst generates from
// `static/`. This script upgrades it in place. If the browser can run
// WebAssembly, the dynamic version (its browser frontend, drawn by Ratzilla)
// replaces the static one at the same URL. The site has no second path and no
// second host. If the browser cannot run WebAssembly, or if scripting is off,
// the static version stays as it is.
//
// The script swaps only after the module of the dynamic version has loaded. A
// failed download therefore leaves the reader on the static version and not on
// a black screen.
(function () {
  "use strict";

  // `?static` keeps the static version, in any browser
  // (https://0xbf.ch/?static).
  if (/[?&]static(?:[=&]|$)/.test(location.search)) {
    return;
  }

  var canRun =
    "WebAssembly" in window && typeof WebAssembly.instantiate === "function";
  if (!canRun) {
    return;
  }

  var shell = document.getElementById("dynamic-shell");
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

  // The dynamic version draws the whole viewport. The code removes everything
  // that the static version brought with it: its text, its links and its
  // footer. Only the elements of the dynamic version stay. If both remained,
  // the mirrored text layer would appear twice and selection would be
  // ambiguous. The static version has no stylesheet of its own;
  // any `<link rel=stylesheet>` or `<style>` goes too, so the canvas of the
  // dynamic version always fills the viewport.
  //
  // `children` is a live collection. Copy it first: removal in a live loop
  // skips the next element.
  function remove(elements) {
    Array.prototype.slice.call(elements).forEach(function (element) {
      element.remove();
    });
  }

  function swap() {
    remove(
      Array.prototype.filter.call(document.body.children, function (element) {
        return element !== shell;
      })
    );
    remove(document.head.querySelectorAll('link[rel="stylesheet"], style'));
    shell.hidden = false;
    link("/dynamic.css");
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
      // Leave the static version alone: it is the fallback, not an error.
      console.warn("apex: the dynamic version did not start, staying on the static version", error);
    });
})();
