// Shared site configuration and the definition of the content.
//
// The Rust core reads `content/` at build time (`include_str!`) and this site
// reads the same files at build time (`toml(..)`). The terminal, the SSH
// session, the browser app and this page therefore show the same project, link
// and key.
//
// `html.*` is the built-in module of Typst 0.15. thyme 0.1.0 no longer
// re-exports the copy from exemel, so no import is necessary.

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

/// The footer, the same on every page: the two keys, the sitemap and the
/// colophon on one line. These are the three rows that the terminal prints. The
/// markup lives here, so two pages cannot differ.
#let footer = html.elem("footer", html.elem(
  "ul",
  attrs: (class: "links"),
  (
    html.elem("li", html.a(href: "/age.txt", "age.txt")),
    html.elem("li", html.a(href: "/pgp.txt", "pgp.txt")),
    html.elem("li", html.a(href: base + "/sitemap.xml", "sitemap.xml")),
    html.elem("li", "Made with <3 by me. No JavaScript."),
  ).join(),
))
