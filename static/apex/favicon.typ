#set page(width: auto, height: auto, margin: 2pt, fill: black)
#set text(font: "CommitMono", size: 19pt, fill: white) // need to load from assets/

#let caret = text(fill: white)[^]
#let at = box(fill: white, inset: 0pt)[#text(fill: black)[@]]

#box(
  fill: black,
  width: 23pt,
  height: 23pt,
  inset: 0pt,
  align(center + horizon, [#caret#at]),
)
