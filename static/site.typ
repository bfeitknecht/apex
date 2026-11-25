// The static version of 0xbf.ch: plain HTML, the browser's own styles, no
// stylesheet, no webfont. Typst writes every file with its bundle export
// (`document` and `asset`); no package is involved.
//
//   typst compile --features html,bundle --format bundle --root . static/site.typ public
//
// It is also the shell of the dynamic version: `boot.js` in the head replaces
// it with the dynamic version (browser frontend, WebAssembly) when the browser
// can run that, unless the URL has `?static`. The hidden `#dynamic-shell` at
// the end of `/` holds the elements the dynamic version draws on.

#import "config.typ": origin, profile, projects, age, destination, contact, flow, joined, footer

/// One HTML file. `path` is the public path without the leading slash and
/// `index.html` (`""` is the root). The head holds what a reader or a
/// crawler needs and nothing for looks.
#let page(path, file, title: none, description: none, boot: false, body) = document(
  file,
  html.html(lang: "en", {
    html.head({
      html.meta(charset: "utf-8")
      html.meta(name: "viewport", content: "width=device-width, initial-scale=1")
      html.title(title)
      if description != none {
        html.meta(name: "description", content: flow(description))
      }
      html.meta(name: "author", content: profile.name)
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

// ---- / ----------------------------------------------------------------------

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

#page("", "index.html", title: profile.site_title, description: profile.site_description, boot: true, {
  html.h1(profile.name)
  html.p({
    profile.eyebrow + "! "
    flow(profile.lede)
  })
  html.p(joined(profile.link.map(row => html.a(href: row.url, row.label))))

  html.h2("about")
  html.p(flow(profile.about))

  html.h2("projects")
  // The order of `content/projects.toml`, which both versions share.
  html.ul(for project in projects {
    html.li({
      if project.url == "" { project.name } else { html.a(href: project.url, project.name) }
      " (" + project.period + "): "
      flow(project.description)
    })
  })

  html.h2("contact")
  // A row without a url (`tangled`) is text, not a dead link.
  html.p(joined(profile.contact.map(row => {
    if row.url == "" { row.label } else { html.a(href: row.url, row.label) }
  })))
  html.p({
    "Keys and more: "
    html.a(href: "/contact/", "contact")
    "."
  })

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

// ---- /contact/ ----------------------------------------------------------------

#page(
  "contact/",
  "contact/index.html",
  title: "Contact · " + profile.name,
  description: "Email, socials, and public keys of " + profile.name + ".",
  {
    html.p(html.a(href: "/", "← " + profile.name))
    html.h1("contact")
    html.p({
      "Mail: "
      html.a(href: "mailto:bfeitknecht@ethz.ch", "bfeitknecht@ethz.ch")
      " or "
      html.a(href: "mailto:" + profile.email, profile.email_label)
      ". Also on "
      html.a(href: contact("github"), "github")
      " and "
      html.a(href: contact("linkedin"), "linkedin")
      "; my cv is at "
      html.a(href: destination("cv"), "/cv")
      "."
    })
    html.h2("keys")
    html.p({
      "Encrypt to my "
      html.a(href: "/age.txt", "age recipient")
      " or my "
      html.a(href: "/pgp.txt", "PGP key")
      ":"
    })
    html.pre(age)
    footer
  },
)

// ---- /404.html ------------------------------------------------------------------

#page(none, "404.html", title: "404 · " + profile.name, {
  html.h1("404")
  html.p({
    "No such page. "
    html.a(href: "/", "Back to /")
    "."
  })
})

// ---- plain files ------------------------------------------------------------------

#let _pages = ("", "contact/")
#asset("sitemap.xml", {
  "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"
  "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n"
  for p in _pages { "  <url><loc>" + origin + "/" + p + "</loc></url>\n" }
  "</urlset>\n"
})
#asset("robots.txt", read("robots.txt"))
// Written by hand (https://llmstxt.org): update it when a page or a project
// changes.
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
