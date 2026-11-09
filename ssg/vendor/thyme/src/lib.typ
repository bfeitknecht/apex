// Public entrypoint for `@preview/thyme:0.1.0`.
//
// Generic zero-CSS SSG core exports are available directly via `#import "@preview/thyme:0.1.0": *`.
// Scholarly monochrome blog preset exports live in `blog` (`#import blog: *`,
// `#show: blog.site.with(...)`).
//
// `site` is always the core engine. The blog preset ships its own bundler as
// `blog.site`, so the two never collide and import order never matters.

#import "blog/mod.typ" as blog

#import "core.typ": (
  backlinks,
  file,
  mount,
  page-asset,
  query-pages,
  site,
  webpage,
  with-html-math,
)

#import "htm.typ": (
  enable-htm,
  h,
  htm,
)

#import "i18n.typ": (
  i18n-css,
  lang-switcher,
  normalize-alternates,
  resolve-page-langs,
  tr,
)

#import "paths.typ": (
  canonical-page-url,
  heading-anchor-id,
  query-backlinks,
  slugify,
  source-path-for,
)

#import "seo.typ": (
  atom-xml,
  robots-txt,
  rss-xml,
  sitemap-xml,
  content-to-feed-html,
  render-seo-meta,
)

#import "text.typ": (
  count-words,
  estimate-reading-minutes,
  short-hash,
  to-plain-text,
  word-hint,
)

#import "tree.typ": (
  any-content,
  collect-content,
  find-content,
  map-containers,
)
