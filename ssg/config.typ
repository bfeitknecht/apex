// Shared site configuration and the definition of the content.
//
// The Rust core reads `content/` at build time (`include_str!`) and this site
// reads the same files at build time (`toml(..)`). The terminal, the SSH
// session, the browser app and this page therefore show the same project, link
// and key.
//
// `html.*` is the built-in module of Typst 0.15. thyme 0.1.2 does not export a
// copy of it, so no import is necessary.

/// The site is the document root: no prefix and no separate path for the app.
/// Pages write absolute hrefs as `{base}/...`, so the prefix appears in one
/// place only.
#let base = ""

#let profile = toml("../content/profile.toml")
#let projects = toml("../content/projects.toml").project
#let age = read("../content/age.txt").trim()

/// The destination of a top-bar link, by label (`cv`, `blog`, `gallery`).
#let destination(label) = profile.link.find(entry => entry.label == label).url

/// The URL of a contact row, by label (`github`, `linkedin`, ...).
#let contact(label) = profile.contact.find(row => row.label == label).url

/// The mail link, so a page only has to place it.
#let mail = html.a(href: "mailto:" + profile.email, profile.email_label)

/// Polyring, the ETH Zurich webring: previous, hub, next. The same links as the
/// terminal (`content/polyring.toml`, `just polyring`).
#let _ring = toml("../content/polyring.toml")
#let polyring = html.elem("li", attrs: (class: "ring"), (
  html.a(href: _ring.previous, title: "Previous site in Polyring", "<<"),
  html.a(href: _ring.hub, title: "Polyring, the ETH Zurich webring", "O"),
  html.a(href: _ring.next, title: "Next site in Polyring", ">>"),
).join(" "))

/// The footer, the same on every page: the two keys, the sitemap, the webring
/// and the colophon on one line. These are the three rows that the terminal prints. The
/// markup lives here, so two pages cannot differ.
#let footer = html.elem("footer", html.elem(
  "ul",
  attrs: (class: "links"),
  (
    html.elem("li", html.a(href: "/age.txt", { "age"; html.span(class: "ext", ".txt") })),
    html.elem("li", html.a(href: "/pgp.txt", { "pgp"; html.span(class: "ext", ".txt") })),
    html.elem("li", attrs: (class: "wide"), html.a(href: base + "/sitemap.xml", "sitemap.xml")),
    polyring,
    // The footer stays one line: CSS sheds `.nojs`, then `.long` for
    // `.short`, then `.ext`, as the footer narrows.
    html.elem("li", {
      html.span(class: "long", { "Made with <3 by me."; html.span(class: "nojs", " No JavaScript.") })
      html.span(class: "short", "<3 by me")
    }),
  ).join(),
))
