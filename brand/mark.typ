// The 0xbf mark: "^@" (the caret, then "@" inverted in a box) on a filled
// disc. Transparent around the disc, so it sits on any background. 19pt keeps
// both glyphs inside the round edge of a 32pt mark.
//
// Two variants, chosen with `--input variant=...`:
// - light (default): black disc, white glyphs. For light backgrounds, and the
//   favicon everywhere (the black disc reads on light and dark tabs).
// - dark: white disc, black glyphs. For dark backgrounds.
//
// `just brand` renders every file in this directory; it passes
// `--font-path ../assets` so CommitMono resolves without an installation.
#let variant = sys.inputs.at("variant", default: "light")
#let (disc, ink) = if variant == "dark" { (white, black) } else { (black, white) }

#set page(width: 32pt, height: 32pt, margin: 0pt, fill: none)
#set text(font: "CommitMono", size: 19pt, fill: ink)

#let caret = text(fill: ink)[^]
#let at = box(fill: ink, inset: 0pt)[#text(fill: disc)[@]]

#place(circle(radius: 16pt, fill: disc, stroke: none))
#place(center + horizon, dx: -0.8pt, dy: 0.1pt, [#caret#at])
