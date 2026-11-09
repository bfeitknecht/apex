# `thyme`

**Pure-Typst static site generator with typed hypertagged HTML, Tadi Web `.typ` source transparency, zero-JS CSS i18n, XML sitemaps & feeds, and a monochrome scholarly `blog` preset.**

[![Manual PDF](https://img.shields.io/badge/docs-manual.pdf-black)](docs/manual.pdf)
[![Typst >= 0.15.1](https://img.shields.io/badge/typst-%3E%3D%200.15.1-239dad)](https://typst.app)
[![License: MIT AND MIT-0](https://img.shields.io/badge/license-MIT%20AND%20MIT--0-444444)](LICENSE)

| Light (`oklch` monochrome) | Dark (`light-dark()`) |
| :---: | :---: |
| ![Light preview](assets/preview-light.png) | ![Dark preview](assets/preview-dark.png) |

`thyme` turns Typst `0.15+`'s native `bundle` and typed `html` targets into a complete static site generator — driven **100% by the standard `typst` CLI** (`typst compile` & `typst watch`) with **zero external wrapper CLIs** and **zero client-side JavaScript frameworks**.

Use the **unopinionated zero-CSS core** (`#import "@preview/thyme:0.1.0": *`) to build custom static sites and portfolios with your own stylesheet, or activate the batteries-included **monochrome scholarly `blog` preset** (`#import "@preview/thyme:0.1.0": blog` then `#show: blog.site.with(...)`) with native MathML, right-margin sidenotes & citations, static `/tags/` archives, bidirectional backlinks, Obsidian frontmatter, theorem/proof environments, and print-first two-column `@media print` preservation.

---

## Highlights

- **Pure `typst` CLI workflow**: `typst watch --features html,bundle -f bundle site.typ dist` runs Typst's built-in live-reloading dev server; `typst compile` emits the complete production web root in one pass.
- **Typed Hypertagged HTML (`htm` & `h`)**: Author raw HTML fragments (`developit/htm`-style) with `{slot}` Typst `content` interpolation (`math`, `@label` links, components), full named/decimal/hex HTML entity decoding, and HTML5 tag-balance validation, or CSS-selector hyperscript (`#h("section#work.grid", ...)`), backed directly by Typst `0.15.1`'s built-in [typed `html.*` constructors](https://typst.app/docs/reference/html/typed/) (`#h(1fr)` still forwards transparently to `std.h`).
- **Tadi Web `.typ` Source Transparency**: Mounting pages via capability `path(...)` (`src: path("site.typ")`, `#mount(path("contact.typ"))`, `#post(path("posts/foo.typ"))`) co-emits verbatim `.typ` source files (`/site.typ`, `/contact.typ`, `/posts/foo.typ`) and adds a monospace `.typ` link in the top bar.
- **Single-URL Zero-JS CSS i18n & `hreflang` Alternates**: Write `#tr(en: [...], de: [...])` inline or look up keys from a `@preview/linguify`-compatible `lang.toml` dictionary (`#tr("key", from: toml("lang.toml"))`); `thyme` emits a CSS `:root:has(#thyme-lang-de:checked)` switcher (`en · de`) in the top bar **only on bilingual pages**, honors `set text(lang: ...)` in paged PDFs, and supports multi-URL `alternates` across `<head>` and `sitemap.xml`.
- **SEO, Full-HTML Syndication & Academic Metadata**: Emits valid `sitemap.xml`, `robots.txt`, `404.html`, and summary or **full semantic HTML** RSS 2.0 (`rss.xml`) & Atom 1.0 (`atom.xml`) feeds (`feed-content: "summary" | "full"`) via `@preview/exemel`, plus opt-in OpenGraph (`og:*`) and Google Scholar Highwire (`citation_*`) `<meta>` tags.
- **Scholarly `blog` Preset**: Perceptual `oklch` / `light-dark()` monochrome palette, unified _STIX Two Text_ (`smcp` / `c2sc` small caps) + _STIX Two Math_ + _Helvetica Neue_ + _Commit Mono_ stacks (`font-faces: auto | none | str`, deterministic zero-`local()` `@font-face` rules, and one-line self-hosting via `font-base: "/fonts/"` + `font-version`), AST-normalized native MathML (`with-html-math` — crisp inline `$F(e)$` & `$F(d^*)` without `measure()` or SVG hacks), content-hashed `css/style.css?v=<hash>`, conditional copy-button JS (`copy-button: auto` — zero `<script>` on codeless pages), `@preview/marge` sidenotes + `#mcite` margin bibliography citations, static `/tags/<tag>/` archive pages, `#toc()`, compile-time `#backlinks()`, `#post-nav()`, `spicky`-style math/proof environments, `zebraw`, `tablem`, and `cheq`.

---

## Quick Start

### 1. Scholarly Blog (`blog` Preset)

Scaffold a ready-to-compile blog with `typst init`:

```sh
typst init @preview/thyme:0.1.0 my-blog
cd my-blog
typst compile --features html,bundle --format bundle site.typ public
```

Or wire `site.typ` by hand:

```typst
#import "@preview/thyme:0.1.0": *
#import "@preview/thyme:0.1.0": blog

#show: blog.site.with(
  description: "Notes on programming languages, formal methods, and systems.",
  url: "https://blog.example.org",
  author: "Ada Lovelace",
  src: path("site.typ"),
)

= `folio`

Notes on programming languages, formal methods, mathematics, and systems.

#blog.links(
  direction: "row",
  link("https://example.org")[example.org],
  [@rss[rss]],
  [@atom[atom]],
)

== Posts

#blog.post(path("posts/fixed-points.typ")) <fixed-points>
#blog.post(path("posts/minimal-bundles.typ")) <minimal-bundles>
```

### 2. Custom Portfolio / Landing Site (Unopinionated Core)

Bring your own `styles.css` with zero preset styles injected (see [`examples/portfolio/`](examples/portfolio)):

```typst
#import "@preview/thyme:0.1.0": *

#show: site.with(
  url: "https://example.org",
  title: "Ada Lovelace",
  description: "Symbolic computation and analytical engines.",
  stylesheets: (path("styles.css"),),
  src: path("site.typ"),
  wrap-main: false,
)

#file("pgp.txt", path("pgp.txt"))
#mount(path("contact.typ")) <contact>

#htm(
  ```html
  <header class="hero">
    <p class="eyebrow">{role}</p>
    <h1>Ada Lovelace</h1>
    <p>{intro} — {contact}</p>
  </header>
  ```,
  role: tr(en: [Symbolic Computation], de: [Symbolische Berechnung]),
  intro: [Bernoulli schedules in $O(n^2)$ operations],
  contact: link(<contact>)[contact],
)
```

---

## Serving `.typ` Sources Inline (`Content-Type: text/plain`)

To ensure browsers display co-emitted `.typ` sources inline in a tab rather than downloading them, configure your static web server to send `Content-Type: text/plain; charset=utf-8` for `*.typ`. With [`static-web-server`](https://static-web-server.net) (`sws.toml`):

```toml
[[advanced.headers]]
source = "**/*.typ"
headers = { Content-Type = "text/plain; charset=utf-8", Content-Disposition = "inline" }
```

---

## Documentation & Examples

- **[Package Reference Manual (`docs/manual.pdf`)](docs/manual.pdf)** — complete API documentation generated from source doc-comments with `@preview/tidy:0.4.3`.
- **[`template/`](template)** — starter monochrome blog (`typst init @preview/thyme:0.1.0`).
- **[`examples/portfolio/`](examples/portfolio)** — zero-CSS core `thyme` site using custom `styles.css`, `htm`, `h`, `#tr(en: ..., de: ...)`, `file`, and `webpage`.
- **[`examples/blog/`](examples/blog)** — unified `blog.site` showcase featuring bilingual `en`/`de` switching (`#tr(...)` + `lang.toml`), co-located folder assets (`posts/state-machine/{site.typ,dfa.svg}`), mounted `webpage` subpages (`about.typ`), full-HTML Atom/RSS feeds, static `/tags/` archives, `#toc()`, `#mcite(...)` margin citations, compile-time backlinks, chronological post navigation, and OpenGraph + Google Scholar `<meta>` tags.

---

## License

Dual-licensed under **`MIT AND MIT-0`** ([`LICENSE`](LICENSE)):
- Library source (`src/`): **MIT**
- Starter template & examples (`template/`, `examples/`): **MIT-0** (No Attribution)
