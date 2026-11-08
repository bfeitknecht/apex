// Public entrypoint for `@preview/thyme:0.1.0`.
//
// Generic zero-CSS SSG core exports are available directly via `#import "@preview/thyme:0.1.0": *`.
// Scholarly monochrome blog preset exports live in `blog` (`#import blog: *`, `#show: blog-site.with(...)`,
// or order-independently via `#show: site.with(preset: blog, ...)`).

#import "blog/mod.typ" as blog
#import "blog/site.typ": site as blog-site

#import "core.typ": (
  backlinks,
  mount,
  page-asset,
  query-pages,
  raw-file,
  route,
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
  build-i18n-css,
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
  build-atom-feed,
  build-robots-txt,
  build-rss-feed,
  build-sitemap,
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
