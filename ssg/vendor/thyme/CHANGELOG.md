# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-04

### Added
- **Unopinionated Core SSG (`#import "@preview/thyme:0.1.0": *`)**:
  - Pure-Typst `bundle`, `html`, and `paged` compilation driven exclusively by standard `typst compile` and `typst watch` (zero external wrapper CLI, zero client-side JavaScript runtime).
  - `site(...)` (with order-independent `blog.site` delegation), `webpage(...)` (alias `route`), `mount(path(...))`, `file(...)`, `page-asset(...)`, `query-pages()`, and `backlinks(...)` with $O(1)$ enclosing-document lookup, cross-bundle `@label` links, collision-free heading anchor `id`s (`h1`–`h6`, `-2`/`-3` deduplication + `"section"` symbol fallback) + optional `§` permalinks (`heading-permalinks`), `404.html`, `robots.txt`, `sitemap.xml`, and optional `rss.xml` / `atom.xml`.
  - **Shared AST Traversal (`src/tree.typ`)**: `find-content`, `any-content`, `collect-content`, and subtree-short-circuiting `map-containers` across layout blocks and `html.elem` wrappers.
  - **Typed Hypertagged HTML & Hyperscript (`src/htm.typ`)**: `htm(...)` (`developit/htm`-style HTML fragments with `{slot}` Typst `content` interpolation) and `h(...)` CSS-selector hyperscript (delegating numeric/fractional spacing to `std.h`) with $O(n)$ cluster slicing, `<pre>`/`<textarea>` whitespace preservation, inline-boundary newline space preservation across tags, empty-element `[]` safety, prose word-count hints, 55+ named + decimal `&#...;` + hex `&#x...;` HTML entity decoding, HTML5 optional-close unwinding (`<li>`, `<p>`, `<dt>`/`<dd>`, `<tr>`/`<td>`), explicit tag-mismatch diagnostics, `"false"` boolean attribute omission, and typed `html.*` construction.
  - **Syntactic HTML MathML Normalizer (`with-html-math`)**: Pure AST `show` rules (zero `measure()`, zero SVG) unwrapping single-line auto-`math.lr` delimiters (`$F(e)$`) while keeping tall delimiters stretchy (`$F(union.sq.big_n d_n)$`) and coercing lone operator glyphs in scripts (`$d^*$`) to `math.class("normal", ...)`.
  - **Tadi Web `.typ` Source Transparency**: Capability `path(...)` mounting (`src: path("site.typ")`, `#mount(path("..."))`, `#post(path("..."))`) reads and co-emits verbatim `.typ` sources at sibling URLs (`/index.typ`, `/<slug>.typ`, `/posts/<slug>.typ`) with `.typ` links in the top utility bar and `<link rel="alternate" type="text/plain">` discovery.
  - **Single-URL Zero-JS CSS i18n & `hreflang` (`src/i18n.typ`)**: `#tr(en: [...], de: [...])` and `@preview/linguify`-compatible `lang.toml` dictionary lookup (`#tr("key", from: toml("lang.toml"))`) with per-branch `set text(lang: code)` (honored in paged PDF output as well), primary-branch-only word-count isolation, keyboard-accessible visually-hidden radio controls + `:focus-visible` ring, conditional `en · de` top-bar switcher only on bilingual pages, and `alternates` (`<link rel="alternate" hreflang="...">` + `xhtml:link` in `sitemap.xml`).
  - **Syndication, Full-HTML Feeds & Academic SEO (`src/seo.typ`)**: Structured XML `sitemap.xml`, Atom 1.0 (`atom.xml`), and RSS 2.0 (`rss.xml` with inferred `<lastBuildDate>`) via `@preview/exemel:0.3.0` supporting both `"summary"` and `"full"` semantic HTML (`content-to-feed-html` with `<table>` and cross-bundle `<label>` URL resolution), dynamic Atom `<id>` epoch inference, and opt-in OpenGraph (`og:*`, `article:*`) + Google Scholar Highwire (`citation_*`) `<meta>` tags (`render-seo-meta`).
- **Scholarly Monochrome Blog Preset (`#import "@preview/thyme:0.1.0": blog` / `blog.site`)**:
  - Unified directly onto `core.typ` so `webpage` subpages (`#mount`), `favicon`, per-post `theme` CSS custom-property overrides, and user `stylesheets` work seamlessly inside `blog` sites.
  - Perceptual `oklch` / `oklab` + `light-dark()` monochrome stylesheet with full-byte 32-bit FNV-1a content-addressed `?v=<hash>` cache keys (`short-hash`), zero-reflow keyboard-accessible narrow-screen sidenote expansion, `@media (prefers-reduced-motion: reduce)`, and dedicated two-column `@media print` preservation.
  - Unified _STIX Two Text_ + _STIX Two Math_, _Helvetica Neue_, and _Commit Mono_ stacks with deterministic zero-`local()` `@font-face` rules (preserving upstream `smcp`/`c2sc` small caps), one-line self-hosting via `font-base` + `font-version` (`default-woff2-files`), and `font-faces: auto | "cdn" | none | str` control.
  - Conditional copy-button JavaScript (`copy-button: auto | bool`) injected only on pages containing `#zebraw` code blocks (zero `<script>` on codeless pages).
  - Right-margin sidenotes via `@preview/marge:0.1.0` (`sidenote`, `#footnote` redirect, `mcite` & `citations-as-sidenotes` margin bibliography citations, `wide` clearance, and superscript-baseline underlines in paged PDF output via `typographic: false` + `is-sidenote-link`), static `/tags/` & `/tags/<tag>/` archive pages (`tag-pages`, live `tag-cloud` links), `#toc()`, compile-time `#backlinks()`, chronological `#post-nav()`, semantic `<time datetime>` Obsidian `<dl>` frontmatter (`author`, optional `monograph`, `created`, `modified`, prose-only `words` & `reading`, linked `#tag`s), Reader-Mode-safe `spicky`-style math environments (`div.blog-sbox`: `definition`, `theorem`, `lemma`, `proposition`, `corollary`, `example`, `recipe`, `remark`; `proof` with hollow QED `□`; `div.blog-callout[role="note"]`), `#diagram(invert-dark: bool)`, `zebraw`, `tablem`, and `cheq`.
- **Documentation, Tests & Tooling**:
  - `@preview/tidy:0.4.3`-generated reference manual (`docs/manual.pdf`), unit & integration test suite (`tests/unit.typ` + `just check`), `template/`, `examples/{portfolio,blog}/`, `scripts/stage-universe.sh`, `justfile`, and CI + automated Typst Universe Draft PR workflow.

[Unreleased]: https://github.com/bfeitknecht/thyme/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/bfeitknecht/thyme/releases/tag/v0.1.0
