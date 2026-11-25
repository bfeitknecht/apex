// The content and the shared pieces of the static version.
//
// The dynamic version reads `content/` at build time (`include_str!`) and the
// static version reads the same files at build time (`toml(..)`). Both
// versions, and every frontend of the dynamic one (browser, ssh, native),
// therefore show the same project, link and key.

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

/// The footer of every page: the keys, the sitemap, the webring, the way to
/// the dynamic version and the colophon. `/` without `?static` starts the
/// dynamic version in a browser that can run it.
#let footer = {
  html.hr()
  html.p(joined((
    html.a(href: "/age.txt", "age.txt"),
    html.a(href: "/pgp.txt", "pgp.txt"),
    html.a(href: "/sitemap.xml", "sitemap.xml"),
    polyring,
    html.a(href: "/", title: "The dynamic version of this site", "dynamic"),
  )))
  html.p(html.small("made with <3 by me. plain html."))
}
