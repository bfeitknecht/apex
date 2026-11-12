#set page(width: auto, height: auto, margin: 2pt, fill: black)
// `just favicon` passes `--font-path assets`, so the family resolves without an
// installation.
#set text(font: "CommitMono", size: 19pt, fill: white)

#let caret = text(fill: white)[^]
#let at = box(fill: white, inset: 0pt)[#text(fill: black)[@]]

#box(
  fill: black,
  width: 23pt,
  height: 23pt,
  inset: 0pt,
  align(center + horizon, [#caret#at]),
)
