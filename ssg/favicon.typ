// No page margin and a 25pt glyph in a 32pt square: "^@" fills ~90 % of the
// width. 27pt is too wide (the glyphs wrap to a second line).
#set page(width: auto, height: auto, margin: 0pt, fill: black)
// `just favicon` passes `--font-path assets`, so the family resolves without an
// installation.
#set text(font: "CommitMono", size: 25pt, fill: white)

#let caret = text(fill: white)[^]
#let at = box(fill: white, inset: 0pt)[#text(fill: black)[@]]

#box(
  fill: black,
  width: 32pt,
  height: 32pt,
  inset: 0pt,
  align(center + horizon, [#caret#at]),
)
