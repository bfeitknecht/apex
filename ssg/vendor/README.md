# Vendored Typst packages

`--package-path .packages` (see `just packages`) resolves `@preview/…` imports
from `preview/<name>/<version>/`. Two packages are vendored here because they
cannot be fetched:

## `thyme/0.1.0` — `github.com/bfeitknecht/thyme`

Typst Universe does not publish `@preview/thyme` yet, so the site builds against
a snapshot of the source.

| | |
| --- | --- |
| repository | `https://github.com/bfeitknecht/thyme` |
| snapshot of | `master` @ `ca01807` (PRs #1–#3 merged: palette tokens, `@font-face`, heading ids, theme-derived `theme-color`, `robots.txt`, bundle-path fixes) |
| package version | `0.1.0` (`typst.toml`), import stays `@preview/thyme:0.1.0` |
| contents | `src/`, `template/`, `typst.toml`, `LICENSE`, `README.md`, `CHANGELOG.md`, `thumbnail.png` — exactly the surface `typst.toml` publishes, nothing else |
| refreshed | 2026-11-09 (second pass, after the upstream merges) |

To refresh: copy the new `src/`, `template/`, `typst.toml`, `LICENSE`,
`README.md`, `CHANGELOG.md`, `thumbnail.png` over this directory and update the
row above. Keep `examples/`, `docs/`, `assets/`, `tests/` and `vendor/` out —
they are excluded from the published package and only drift.

Behaviour that changed for this site when the snapshot moved from `1187b2d` to
`ca01807`:

- **`robots.txt` is emitted at last.** It was documented and exported but never
  called, so the bundle had none.
- **A custom `not-found-page` is wrapped in the core shell.** `site()` renders it
  with the stylesheet, favicon and `theme-color` of the site, so the value must
  be a *body fragment*; `ssg/site.typ` returns one now. A whole document there
  would nest one `<html>` in another.
- **`theme-color` is derived from the theme** (`css-hex(paper-light/dark)`)
  instead of being pinned to `#f2f2f2`/`#101010`; with the default palette the
  light value is unchanged.

API differences this snapshot carries over the previous one, for the record:

- `#raw-file(…)` → **`#file(…)`**.
- The blog preset is reached as `blog.site`; `site.with(preset: …)` and the
  `route` alias are gone. `site()` never took a preset in this site, so only
  the `#file` rename applied.
- `html.*` is **Typst 0.15's built-in module**, no longer exemel re-exported
  through thyme. `#import "@preview/thyme:0.1.0": html` is an unresolved
  import; use the global.
- The blog preset now imports `@preview/cheq`, `showybox`, `tablem`, `zebraw`
  and `marge`. `lib.typ` imports the preset eagerly, so *building this site*
  pulls those five from Typst Universe. Typst caches them in
  `~/.cache/typst/packages` after the first build.

## `exemel/0.3.0` — vendored by upstream

`src/seo.typ` imports `@preview/exemel:0.3.0` for XML (sitemap, feeds). That
release is not published yet either — upstream vendors the same snapshot, and
this directory carries it verbatim so the sitemap is built from the same code
upstream tests against.
