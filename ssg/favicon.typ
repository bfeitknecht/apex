// The mark: "^@" (caret white, "@" inverted in a white box) on a filled black
// circle. Transparent around the circle, so it sits on any background. 19pt
// keeps both glyphs inside the round edge of a 32pt mark.
#set page(width: 32pt, height: 32pt, margin: 0pt, fill: none)
// `just favicon` passes `--font-path assets`, so the family resolves without an
// installation.
#set text(font: "CommitMono", size: 19pt, fill: white)

#let caret = text(fill: white)[^]
#let at = box(fill: white, inset: 0pt)[#text(fill: black)[@]]

#place(circle(radius: 16pt, fill: black, stroke: none))
#place(center + horizon, dx: -0.8pt, dy: 0.1pt, [#caret#at])
