#import "@preview/thyme:0.1.3": *
#import "config.typ": base, profile, projects, footer

#let _font-files = (
  "CommitMono-400-Regular.woff2",
  "CommitMono-400-Italic.woff2",
  "CommitMono-700-Regular.woff2",
  "CommitMono-700-Italic.woff2",
  "LICENSE.txt",
)

// The destinations of the hero: `cv`, `blog` and `gallery`, in the order of
// `content/`. The links are plain, with dots between them. Mail, github and
// linkedin are in the contact section below.
#let _links = for row in profile.link {
  html.elem("li", html.a(href: row.url, row.label))
}

// The contact rows in the order of `content/profile.toml`. The terminal prints
// the same list in the same order under "get in touch". A row without a url
// (`tangled`) is text and not a dead link.
#let _contacts = for row in profile.contact {
  if row.url == "" {
    html.elem("li", attrs: (class: "pending"), row.label)
  } else {
    html.elem("li", html.a(href: row.url, row.label))
  }
}

// One entry per project, from `content/projects.toml`. The terminal prints
// `summary` on one line. This page prints `description` and `period`.
#let _entries = for project in projects {
  html.elem("li", attrs: (class: "entry"), {
    html.elem("div", attrs: (class: "entry-head"), {
      html.h3(html.a(href: project.url, [#project.name]))
      html.elem("span", attrs: (class: "when"), [#project.period])
    })
    html.p([#project.description])
  })
}

// A body fragment. `site()` of thyme 0.1.2 puts a custom `not-found-page` into
// its core shell, which adds the stylesheet, the favicon and `theme-color`. A
// whole document here would put one `<html>` inside another.
#let _not-found = html.elem("header", attrs: (class: "notfound"), {
  html.h1("404")
  html.p(class: "lede", "No such page.")
  html.elem("nav", attrs: (class: "links", "aria-label": "Navigation"), {
    html.a(href: base + "/", "back to /")
  })
})

#show: site.with(
  url: "https://0xbf.ch" + base,
  title: profile.site_title,
  description: profile.site_description,
  author: profile.name,
  stylesheets: (path("styles.css"),),
  favicon: path("assets/favicon.svg"),
  src: path("site.typ"),
  wrap-main: false,
  rss-path: none,
  atom-path: none,
  not-found-page: _not-found,
  // The boot script goes in the head. `defer` waits for the DOM.
  head: html.script(src: "/boot.js", defer: true),
  // thyme copies both files byte for byte. `llms.txt` is written by hand
  // (https://llmstxt.org).
  robots: path("robots.txt"),
  llms: path("llms.txt"),
  assets: _font-files.map(f => (
    path: "assets/fonts/" + f,
    data: read("assets/fonts/" + f, encoding: none),
  )),
)

// Plain files, served from the root next to the page: the keys that people copy
// and the CV that the footer links to. These are data and not markup, and
// `content/` is the source. (`#file`: thyme 0.1.0 renamed `#raw-file` to this.)
#file("age.txt", path("../content/age.txt"))
#file("pgp.txt", path("../content/pgp.txt"))
#file("cv.pdf", path("../content/cv.pdf"))
// Root fallbacks: Safari, iOS and tools ask for these without a <link>.
// Rendered from assets/favicon.svg (the round mark).
#file("favicon.ico", path("assets/favicon.ico"))
#file("apple-touch-icon.png", path("assets/apple-touch-icon.png"))

#mount(path("contact.typ")) <contact>

// Light-scheme tone mapping for the colony canvas. The SVG names are camelCase
// (`feFuncR`) and have no typed constructor, so `html.elem` writes them.
#let tone_filter = html.elem(
  "svg",
  attrs: (width: "0", height: "0", "aria-hidden": "true", style: "position:absolute"),
  html.elem("defs", {
    html.elem(
      "filter",
      attrs: (id: "lichen-light-tones", color-interpolation-filters: "sRGB"),
      html.elem(
        "feComponentTransfer",
        ("feFuncR", "feFuncG", "feFuncB").map(channel => html.elem(
          channel,
          attrs: (type: "table", tableValues: "0 .25 .5 1 1"),
        )).join(),
      ),
    )
  }),
)

// The section heading of the page: a number and a title.
#let _heading(num, title) = html.h2({
  html.span(class: "num", num)
  title
})

#html.div(id: "plain", {
  html.header({
    html.p(class: "eyebrow", profile.eyebrow)
    html.h1(profile.name)
    html.p(class: "lede", profile.lede)
    html.p(class: "intro", profile.intro.join(" "))
    html.ul(class: "links", _links)
  })
  html.main({
    html.section(id: "about", {
      _heading("01", "About")
      // The same paragraph as the terminal (`about_terminal` in
      // `src/content.rs`), trimmed in the same way. The terminal lowercases it
      // and the page does not.
      html.p(profile.about.trim())
    })
    html.section(id: "projects", {
      _heading("02", "Projects")
      html.ul(class: "entries", _entries)
    })
    html.section(id: "contact", {
      _heading("03", "Contact")
      html.ul(class: "contacts", _contacts)
    })
  })
  footer
})

// The surfaces of the app. They stay hidden until the boot script decides that
// the browser can run the app. The page above is the fallback.
#html.div(id: "app-shell", hidden: true, {
  html.pre(id: "selectable-text")
  html.canvas(id: "lichen-pixels", width: 320, height: 200)
  // The old markup had a space here. Keep it, so the layout does not change.
  [ ]
  html.canvas(id: "unicode-glyph")
  html.div(id: "cell-cursor", aria-hidden: true)
  html.div(id: "selection-highlights", aria-hidden: true)
  tone_filter
})
