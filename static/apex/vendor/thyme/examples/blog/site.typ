#import "@preview/thyme:0.1.0": *
#import blog: *

#show: site.with(
  preset: blog,
  description: "Algebraic invariants, automata, and bilingual static notes.",
  url: "https://notes.example.org",
  author: "Emmy Noether",
  src: path("site.typ"),
  feed-content: "full",
  heading-permalinks: true,
  backlinks: true,
  post-nav: true,
  open-graph: true,
  scholar-meta: true,
)

#mount(path("about.typ")) <about>

= `noether`

Algebraic invariants, finite-state machines, and static scholarly notes.

#links-row(
  link(<about>)[colophon],
  link(<tags>)[tags],
  [@rss[rss]],
  [@atom[atom]],
)

== Notes

#post(path("posts/bilingual-notes.typ")) <bilingual-notes>
#post(path("posts/state-machine/index.typ")) <state-machine>
