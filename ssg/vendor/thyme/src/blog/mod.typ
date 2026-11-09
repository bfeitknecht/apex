// Public exports of the `blog` scholarly monochrome blog preset inside `@preview/thyme`.
//
// Usage:
//   #import "@preview/thyme:0.1.0": *
//   #import blog: *
//   // or order-independently:
//   #show: blog.site.with(...)
//
// Generic core functions (`site`, `webpage`, `mount`, `file`, ...) are exported by
// `lib.typ` itself; this module only re-exports them so `#import blog: *` alone is
// enough to build a site. In Typst every top-level `#let`/`#import` of a file is part
// of that file's module, so the imports above are the public export surface.

// Upstream packages re-exported so `#import blog: *` matches the documented surface
// (the manual and examples rely on `checklist`, `showybox`, `zebraw-init`, ...).
#import "@preview/cheq:0.4.0": checklist
#import "@preview/showybox:2.0.4": showybox
#import "@preview/zebraw:0.6.3": (
  zebraw-init,
  zebraw-themes,
)

#import "../core.typ": (
  backlinks,
  file,
  mount,
  page-asset,
  query-pages,
  webpage,
  with-html-math,
)

#import "../htm.typ": (
  enable-htm,
  h,
  htm,
)

#import "theme.typ": (
  active-theme,
  stylesheet-css,
  theme-override-css,
  css-hex,
  css-oklch,
  default-theme,
  default-woff2-files,
  resolve-theme,
  stylesheet-asset-path,
  stylesheet-href,
)

#import "notes.typ": (
  container,
  marginnote,
  mcite,
  mn,
  sidenote,
  sn,
)

#import "components.typ": (
  callout,
  codeblock,
  corollary,
  definition,
  diagram,
  epigraph,
  example,
  grid-table,
  hr,
  important-box,
  kbd,
  lemma,
  links,
  note-box,
  proof,
  proposition,
  recipe,
  remark,
  sbox,
  tablem,
  theorem,
  three-line-table,
  tip-box,
  toc,
  warning-box,
  wide,
  zebraw,
)

#import "post.typ": (
  post,
  post-asset,
  post-nav,
)

#import "site.typ": (
  post-list,
  site,
  tag-cloud,
)

// Public surface of `blog` (everything imported above is re-exported by this module).
//
//   engine      site, webpage, mount, file, page-asset, query-pages, backlinks
//   theme       default-theme, resolve-theme, active-theme, css-oklch, css-hex,
//               stylesheet-css, stylesheet-href, stylesheet-asset-path,
//               theme-override-css, default-woff2-files
//   typography  htm, h, enable-htm, with-html-math
//   notes       sidenote, sn, marginnote, mn, mcite, container
//   blocks      callout, codeblock, sbox, academic environments
//               (definition, theorem, lemma, proposition, corollary, example,
//               recipe, remark, proof), note-box, tip-box, warning-box,
//               important-box, epigraph, wide, diagram, kbd, hr, links, toc
//   tables      tablem, three-line-table, grid-table, zebraw
//   posts       post, post-asset, post-nav, post-list, tag-cloud
//   upstream    checklist, showybox, zebraw-init, zebraw-themes
