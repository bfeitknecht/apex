// Shared site configuration and the one definition of the content.
//
// `content/` is read by the Rust core at compile time (`include_str!`) and by
// this site at build time (`toml(..)`), so the terminal, the SSH session, the
// browser app and this page cannot disagree about a project, a link or a key.
//
// `html.*` is Typst 0.15's built-in module; thyme 0.1.0 stopped re-exporting
// exemel's copy, so nothing needs importing for it.

/// The site *is* the document root: no prefix, and no separate path for the
/// app either. Absolute hrefs are written as `{base}/...` so the prefix stays
/// stated in exactly one place.
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

/// The one footer, identical on every page: the two keys, the sitemap, and the
/// colophon on the same line — the same three rows the terminal prints. The
/// markup lives here so two pages cannot drift apart.
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
