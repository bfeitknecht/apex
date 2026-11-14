// The plain-text files that a machine reads: `robots.txt` and `llms.txt`
// (https://llmstxt.org). Both come from `content/`, the same files that the
// page and the terminal read, so neither repeats a fact.

#import "config.typ": base, profile, projects

// The origin, without a trailing slash.
#let _origin = "https://0xbf.ch" + base

// One list line, `- [title](url)`, with `: notes` when there are notes.
#let _entry(title, url, notes: none) = {
  let line = "- [" + str(title) + "](" + str(url) + ")"
  if notes == none { line } else { line + ": " + str(notes) }
}

// Every path is open, and the crawler gets the sitemap. thyme 0.1.0 no longer
// writes this file, so the site does.
#let robots = "User-agent: *\nAllow: /\n\nSitemap: " + _origin + "/sitemap.xml\n"

// The destinations of the hero, in the order of `content/profile.toml`. The
// first entry is the page itself.
#let _pages = (
  _entry(profile.name, _origin + "/", notes: profile.lede),
  ..profile.link.map(row => _entry(row.label, row.url)),
)

// One line per project, in the order of `content/projects.toml`. The note is
// the line that the terminal prints.
#let _project-rows = projects.map(project => _entry(
  project.name,
  project.url,
  notes: project.summary,
))

// The contacts that have an address. A row without one is text on the page.
#let _contact-rows = profile.contact
  .filter(row => row.url != "")
  .map(row => _entry(row.label, row.url))

// The files that a reader does not need in order to answer a question.
#let _optional-rows = (
  _entry("age.txt", _origin + "/age.txt"),
  _entry("pgp.txt", _origin + "/pgp.txt"),
  _entry("sitemap.xml", _origin + "/sitemap.xml"),
)

// The document: the title, the summary with the about paragraph, then the
// sections in the order of the page.
#let llms = (
  "# " + profile.site_title,
  "> " + profile.site_description + "\n\n" + profile.about.trim(),
  "## Pages\n\n" + _pages.join("\n"),
  "## Projects\n\n" + _project-rows.join("\n"),
  "## Contact\n\n" + _contact-rows.join("\n"),
  "## Optional\n\n" + _optional-rows.join("\n"),
).join("\n\n") + "\n"
