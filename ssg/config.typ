// The content and the shared pieces of the page.
//
// The Rust core reads `content/` at build time (`include_str!`) and this page
// reads the same files at build time (`toml(..)`). The terminal, the SSH
// session, the browser app and this page therefore show the same project, link
// and key.

#let origin = "https://0xbf.ch"

#let profile = toml("../content/profile.toml")
#let projects = toml("../content/projects.toml").project
#let age = read("../content/age.txt").trim()

/// The destination of a top link, by label (`cv`, `blog`, `gallery`).
#let destination(label) = profile.link.find(entry => entry.label == label).url

/// The URL of a contact row, by label (`github`, `linkedin`, ...).
#let contact(label) = profile.contact.find(row => row.label == label).url

/// A TOML text block as one paragraph: its line breaks are only for the
/// source (Typst would turn them into `<br>`).
#let flow(text) = text.trim().replace(regex("\\s*\\n\\s*"), " ")

/// Items joined by a separator, as inline content.
#let joined(items, sep: " · ") = items.join(sep)

/// Polyring, the ETH Zurich webring: previous, hub, next. The same links as the
/// terminal (`content/polyring.toml`, `just polyring`).
#let _ring = toml("../content/polyring.toml")
#let polyring = joined(
  (
    html.a(href: _ring.previous, title: "Previous site in Polyring", "<<"),
    html.a(href: _ring.hub, title: "Polyring, the ETH Zurich webring", "O"),
    html.a(href: _ring.next, title: "Next site in Polyring", ">>"),
  ),
  sep: " ",
)

/// The footer of every page: the keys, the sitemap, the webring, the way back
/// to the app and the colophon. `/` without `?static` starts the app again in
/// a browser that can run it.
#let footer = {
  html.hr()
  html.p(joined((
    html.a(href: "/age.txt", "age.txt"),
    html.a(href: "/pgp.txt", "pgp.txt"),
    html.a(href: "/sitemap.xml", "sitemap.xml"),
    polyring,
    html.a(href: "/", title: "The terminal version of this site", "app"),
  )))
  html.p(html.small("made with <3 by me. plain html."))
}
