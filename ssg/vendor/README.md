# Vendored Typst packages

`--package-path .packages` (see `just packages`) resolves `@preview/…` imports
from `preview/<name>/<version>/`. Two packages are vendored here, because no
package registry provides them:

## `thyme/0.1.0`, from `github.com/bfeitknecht/thyme`

Typst Universe does not publish `@preview/thyme` yet, so the site builds against
a snapshot of the source.

| | |
| --- | --- |
| repository | `https://github.com/bfeitknecht/thyme` |
| snapshot of | `master` @ `ca01807` (PRs #1–#3 merged: palette tokens, `@font-face`, heading ids, theme-derived `theme-color`, `robots.txt`, bundle-path fixes) |
| package version | `0.1.0` (`typst.toml`), import stays `@preview/thyme:0.1.0` |
| contents | `src/`, `template/`, `typst.toml`, `LICENSE`, `README.md`, `CHANGELOG.md`, `thumbnail.png`: exactly what `typst.toml` publishes |
| refreshed | 2026-11-09 (second pass, after the upstream merges) |

To refresh, copy the new `src/`, `template/`, `typst.toml`, `LICENSE`,
`README.md`, `CHANGELOG.md` and `thumbnail.png` over this directory, then update
the row above. Do not copy `examples/`, `docs/`, `assets/`, `tests/` or
`vendor/`. The published package excludes them, so they only go out of date.

These changes affect this site. The snapshot moved from `1187b2d` to `ca01807`:

- **`robots.txt` is emitted now.** The function was documented and exported, but
  nothing called it, so the bundle had no `robots.txt`.
- **A custom `not-found-page` goes into the core shell.** `site()` draws it with
  the stylesheet, the favicon and `theme-color` of the site, so the value must be
  a body fragment. `ssg/site.typ` returns one. A whole document there would put
  one `<html>` inside another.
- **`theme-color` comes from the theme** (`css-hex(paper-light/dark)`). The code
  no longer pins `#f2f2f2` and `#101010`. With the default palette, the light
  value does not change.

API differences in this snapshot:

- `#raw-file(…)` is now `#file(…)`.
- The blog preset is reached as `blog.site`. `site.with(preset: …)` and the
  `route` alias are gone. This site never passed a preset to `site()`, so only
  the `#file` rename applied here.
- `html.*` is the built-in module of Typst 0.15 and no longer comes from exemel
  through thyme. `#import "@preview/thyme:0.1.0": html` fails; use the global.
- The blog preset imports `@preview/cheq`, `showybox`, `tablem`, `zebraw` and
  `marge`, and `lib.typ` imports the preset at once. A build of this site
  therefore needs those five packages from Typst Universe. Typst caches them in
  `~/.cache/typst/packages` after the first build.

## `exemel/0.3.0`, vendored by upstream

`src/seo.typ` imports `@preview/exemel:0.3.0` for XML (sitemap and feeds). That
release is not published either. Upstream vendors the same snapshot, and this
directory carries it unchanged, so the sitemap uses the code that upstream
tests.
