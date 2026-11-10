#import "@preview/thyme:0.1.0": *
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

// A body fragment. `site()` of thyme 0.1.0 puts a custom `not-found-page` into
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
  // The boot script goes in the head. `defer` waits for the DOM. The typed
  // constructor is the only way to emit a boolean attribute without a value,
  // because the parser of `htm` passes `true` where a string is expected.
  head: html.script(src: "/boot.js", defer: true),
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

#mount(path("contact.typ")) <contact>

// Light-scheme tone mapping for the colony canvas. It uses `html.elem` and not
// markup, because the HTML parser lowercases tag names and the transfer
// functions of SVG are camelCase (`feFuncR`). A raw fragment would be broken
// HTML.
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
      <ul class="links">{links}</ul>
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
  // The same paragraph as the terminal (`about_terminal` in `src/content.rs`),
  // trimmed in the same way. The terminal lowercases it and the page does not.
  about: profile.about.trim(),
  entries: _entries,
  contacts: _contacts,
  links: _links,
  footer: footer,
  tone_filter: tone_filter,
)
