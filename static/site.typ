// The static version of 0xbf.ch: raw HTML. No stylesheet, no style attribute,
// no webfont: everything bold or loud is an HTML element the browser already
// knows (h1, mark, marquee, table border, details). Typst writes every file
// with its bundle export (`document` and `asset`); no package is involved.
//
//   typst compile --features html,bundle --format bundle --root . static/site.typ public
//
// It is its own thing: only the projects, the keys and the links come from
// `content/` (config.typ). The words below are the static version's own.
//
// It is also the shell of the dynamic version: `boot.js` in the head replaces
// it with the dynamic version (browser frontend, WebAssembly) when the browser
// can run that, unless the URL has `?static`. The hidden `#dynamic-shell` at
// the end of `/` holds the elements the dynamic version draws on.

#import "config.typ": origin, projects, age, pgp, destinations, contacts, ring

// ---- the words of the static version (edit freely) ---------------------------

#let words = (
  title: "basil feitknecht",
  description: "basil feitknecht. computer science at eth zürich, projects, links and keys.",
  ticker: "~~~ grüezi and welcome ~~~ you found the static version ~~~ no css, no javascript, just html ~~~ thanks for stopping by ~~~",
  ticker-back: "under construction since 2026 ~~~ under construction forever ~~~",
  name: "basil feitknecht",
  hello: [*hi, i'm basil.* i study computer science at eth zürich and take things apart to see how they work. sometimes they even go back together.],
  tagline: "young, curious and full of ~~~semicolons~~~",
  projects: "things i made (some of them even work)",
  elsewhere: "where else i lurk",
  keys: "secret handshakes",
  age: "age, for the modern spy",
  pgp: "pgp, for the vintage spy",
  ring: [this site is part of polyring, the eth zürich webring. hop to the],
  dynamic: "too plain? try the dynamic version",
  colophon: "made by hand with typst. no css was harmed.",
  not-found: "this page wandered off.",
)

// ---- page shell ----------------------------------------------------------------

/// One HTML file. `path` is the public path without the leading slash (`""` is
/// the root), or none (404). The head holds what a reader or a crawler needs
/// and nothing for looks.
#let page(path, file, title: none, description: none, boot: false, body) = document(
  file,
  html.html(lang: "en", {
    html.head({
      html.meta(charset: "utf-8")
      html.meta(name: "viewport", content: "width=device-width, initial-scale=1")
      html.title(title)
      if description != none {
        html.meta(name: "description", content: description)
      }
      if path != none {
        html.link(rel: "canonical", href: origin + "/" + path)
      }
      html.link(rel: "icon", href: "/assets/favicon.svg", type: "image/svg+xml")
      if boot {
        html.script(src: "/boot.js", defer: true)
      }
    })
    html.body(body)
  }),
)

#let link(url, label) = if url == "" { label } else { html.a(href: url, label) }

#let footer = {
  html.hr()
  html.p({
    words.ring
    " "
    html.a(href: ring.previous, title: "Previous site in Polyring", "previous one")
    ", the "
    html.a(href: ring.hub, title: "Polyring, the ETH Zurich webring", "ring itself")
    " or the "
    html.a(href: ring.next, title: "Next site in Polyring", "next one")
    "."
  })
  html.p(html.b(html.a(href: "/", title: "The dynamic version of this site", words.dynamic)))
  html.p(html.small({
    words.colophon
    " "
    html.a(href: "/sitemap.xml", "sitemap")
  }))
}

// ---- / ---------------------------------------------------------------------------

// The colony canvas in the light scheme: a tone map `dynamic.css` refers to.
// The SVG names are camelCase (`feFuncR`) and have no typed constructor.
#let _tone-filter = html.elem(
  "svg",
  attrs: (width: "0", height: "0", "aria-hidden": "true", style: "position:absolute"),
  html.elem("defs", html.elem(
    "filter",
    attrs: (id: "lichen-light-tones", color-interpolation-filters: "sRGB"),
    html.elem(
      "feComponentTransfer",
      ("feFuncR", "feFuncG", "feFuncB")
        .map(channel => html.elem(channel, attrs: (type: "table", tableValues: "0 .25 .5 1 1")))
        .join(),
    ),
  )),
)

#page("", "index.html", title: words.title, description: words.description, boot: true, {
  html.elem("marquee", attrs: (scrollamount: "8"), html.b(words.ticker))
  html.h1(html.elem("big", words.name))
  // The mark, bouncing from wall to wall.
  html.elem(
    "marquee",
    attrs: (behavior: "alternate", scrollamount: "4"),
    html.img(src: "/assets/favicon.svg", width: 48, height: 48, alt: "the ^@ mark of 0xbf"),
  )
  html.p(words.hello)
  html.p(html.i(words.tagline))
  // The destinations, loud: they are why most people come here.
  html.h2(destinations.map(row => html.mark(link(row.url, row.label))).join(" ~ "))

  html.h2(words.projects)
  // The shared order of `content/projects.toml`.
  html.elem("table", attrs: (border: "1", cellpadding: "6", cellspacing: "0"), {
    for project in projects {
      html.tr({
        html.td(html.b(link(project.url, project.name)))
        html.td(project.period)
        html.td(project.description)
      })
    }
  })

  html.h2(words.elsewhere)
  html.ul(for row in contacts { html.li(link(row.url, row.label)) })

  html.h2(words.keys)
  html.details({
    html.summary(words.age)
    html.pre(age)
    html.p(html.a(href: "/age.txt", "age.txt"))
  })
  html.details({
    html.summary(words.pgp)
    html.pre(pgp)
    html.p(html.a(href: "/pgp.txt", "pgp.txt"))
  })

  html.elem("marquee", attrs: (direction: "right", scrollamount: "5"), html.small(words.ticker-back))

  footer

  // The elements of the dynamic version, hidden until boot.js swaps to it.
  html.div(id: "dynamic-shell", hidden: true, {
    html.pre(id: "selectable-text")
    html.canvas(id: "lichen-pixels", width: 320, height: 200)
    html.canvas(id: "unicode-glyph")
    html.div(id: "cell-cursor", aria-hidden: true)
    html.div(id: "selection-highlights", aria-hidden: true)
    _tone-filter
  })
})

// ---- /404.html ---------------------------------------------------------------------

#page(none, "404.html", title: "404 · " + words.title, {
  html.elem("marquee", attrs: (scrollamount: "12"), html.b("404 ~~~ 404 ~~~ 404 ~~~ 404 ~~~ 404"))
  html.h1(html.elem("big", "404"))
  html.p({
    words.not-found
    " "
    html.mark(html.a(href: "/", "go home"))
  })
})

// ---- plain files ---------------------------------------------------------------------

#asset("sitemap.xml", {
  "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"
  "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n"
  "  <url><loc>" + origin + "/</loc></url>\n"
  "</urlset>\n"
})
#asset("robots.txt", read("robots.txt"))
// Written by hand (https://llmstxt.org): update it when the site changes.
#asset("llms.txt", read("llms.txt"))
// The keys people copy and the CV: data, with `content/` as the source.
#asset("age.txt", read("../content/age.txt", encoding: none))
#asset("pgp.txt", read("../content/pgp.txt", encoding: none))
#asset("cv.pdf", read("../content/cv.pdf", encoding: none))
// Icons: the brand mark (brand/README.md). Safari and iOS ask for the root
// files without a <link>.
#asset("assets/favicon.svg", read("assets/favicon.svg", encoding: none))
#asset("favicon.ico", read("assets/favicon.ico", encoding: none))
#asset("apple-touch-icon.png", read("assets/apple-touch-icon.png", encoding: none))
