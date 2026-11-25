// What the static version shares with the dynamic one: the project list, the
// keys and the links (`content/`). Nothing else. Every word of the static
// version itself (name, greeting, headings, footer) lives in `site.typ`.

#let origin = "https://0xbf.ch"

#let _profile = toml("../content/profile.toml")

/// Projects, in the shared order (`content/projects.toml`).
#let projects = toml("../content/projects.toml").project

/// The keys, as plain text (`content/age.txt`, `content/pgp.txt`).
#let age = read("../content/age.txt").trim()
#let pgp = read("../content/pgp.txt").trim()

/// The destinations (`cv`, `blog`, `gallery`), in the shared order.
#let destinations = _profile.link

/// The contact rows (mail, github, ...), in the shared order. A row without a
/// url (`tangled`) is not a link yet.
#let contacts = _profile.contact

/// Polyring, the ETH Zurich webring: previous, hub, next
/// (`content/polyring.toml`, `just polyring`).
#let ring = toml("../content/polyring.toml")
