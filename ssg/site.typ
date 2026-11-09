#import "@preview/thyme:0.1.0": *
#import "config.typ": base, profile, projects, destination, contact, mail, footer

#let _font-files = (
  "CommitMono-400-Regular.woff2",
  "CommitMono-400-Italic.woff2",
  "CommitMono-700-Regular.woff2",
  "CommitMono-700-Italic.woff2",
  "LICENSE.txt",
)

// The contact rows, in the order `content/profile.toml` lists them — the same
// list, in the same order, that the terminal prints under "get in touch". A row
// with no url yet (`tangled`) is text, not a dead link.
#let _contacts = for row in profile.contact {
  if row.url == "" {
    html.elem("li", attrs: (class: "pending"), row.label)
  } else {
    html.elem("li", html.a(href: row.url, row.label))
  }
}

// One entry per project, straight from `content/projects.toml`. The terminal
// prints `summary` on one line; this page prints `description` and `period`.
#let _entries = for project in projects {
  html.elem("li", attrs: (class: "entry"), {
    html.elem("div", attrs: (class: "entry-head"), {
      html.h3(html.a(href: project.url, [#project.name]))
      html.elem("span", attrs: (class: "when"), [#project.period])
    })
    html.p([#project.description])
  })
}

#let _not-found = html.html(lang: "en", {
  html.head({
    html.meta(charset: "utf-8")
    html.meta(name: "viewport", content: "width=device-width, initial-scale=1")
    html.meta(name: "robots", content: "noindex")
    html.title("404 - 0xbf.ch")
    html.link(rel: "icon", href: base + "/assets/favicon.svg", type: "image/svg+xml")
    html.link(rel: "stylesheet", href: base + "/styles.css")
  })
  html.body({
    html.header(class: "notfound", {
      html.h1("404")
      html.p(class: "lede", "No such page.")
      html.elem("nav", attrs: (class: "links", "aria-label": "Navigation"), {
        html.a(href: base + "/", "back to /")
      })
    })
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
  // The boot script lives in the head: `defer` waits for the DOM, and the
  // typed constructor is the only way to emit a bare boolean attribute
  // (`htm`'s parser passes `true` where a string is expected).
  head: html.script(src: "/boot.js", defer: true),
  assets: _font-files.map(f => (
    path: "assets/fonts/" + f,
    data: read("assets/fonts/" + f, encoding: none),
  )),
)

// Plain artifacts, served from the root next to the page: the keys people copy
// and the CV the footer links to. Data, not markup — `content/` is the source.
// (`#file` — thyme 0.1.0 renamed `#raw-file` to this.)
#file("age.txt", path("../content/age.txt"))
#file("pgp.txt", path("../content/pgp.txt"))
#file("cv.pdf", path("../content/cv.pdf"))

#mount(path("contact.typ")) <contact>

// Light-scheme tone mapping for the colony canvas. Built with `html.elem` rather
// than markup: the HTML parser lowercases tag names, and SVG's transfer
// functions are camelCase (`feFuncR`), so a raw fragment would be broken HTML.
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

#htm(
  ```html
  <div id="plain">
    <header>
      <p class="eyebrow">{eyebrow}</p>
      <h1>{name}</h1>
      <p class="lede">{lede}</p>
      <p class="intro">{intro}</p>
      <ul class="links">
        <li>{mail}</li>
        <li><a href="{blog}">blog</a></li>
        <li><a href="{github}">github</a></li>
        <li><a href="{linkedin}">linkedin</a></li>
        <li><a href="{cv}">cv</a></li>
      </ul>
    </header>

    <main>
      <section id="about">
        <h2><span class="num">01</span>About</h2>
        <p>{about}</p>
      </section>

      <section id="projects">
        <h2><span class="num">02</span>Projects</h2>
        <ul class="entries">{entries}</ul>
      </section>

      <section id="contact">
        <h2><span class="num">03</span>Contact</h2>
        <ul class="contacts">{contacts}</ul>
      </section>
    </main>{footer}</div>

  <!-- The interactive version's surfaces. Hidden until the boot script below
       decides the browser can run it; the page above is the fallback. -->
  <div id="app-shell" hidden="hidden">
    <pre id="selectable-text"></pre>
    <canvas id="lichen-pixels" width="320" height="200"></canvas>
    <canvas id="unicode-glyph"></canvas>
    <div id="cell-cursor" aria-hidden="true"></div>
    <div id="selection-highlights" aria-hidden="true"></div>
    {tone_filter}
  </div>

  ```,
  name: profile.name,
  eyebrow: profile.eyebrow,
  lede: profile.lede,
  intro: profile.intro.join(" "),
  // The terminal lowercases the shared paragraph once at load
  // (`about_terminal` in `src/content.rs`); the page does the same, so both
  // surfaces print the same string.
  about: lower(profile.about),
  entries: _entries,
  contacts: _contacts,
  footer: footer,
  mail: mail,
  cv: destination("cv"),
  blog: destination("blog"),
  github: contact("github"),
  linkedin: contact("linkedin"),
  tone_filter: tone_filter,
)
