// The NULL wallpaper: the screen filled with "^@", the way `cat -v` prints
// NUL bytes, in CommitMono on a terminal grid.
//
// Inputs (`--input key=value`), all optional:
// - variant: dark (default: white on black) or light (black on white)
// - width, height: the display in pixels (default 3024 x 1964, MacBook Pro 14")
// - band: pixels left empty at the top for the menu bar (default 74: the
//   notch row of the 14" display at 2x, so no glyph sits under the menu bar)
// - pad: pixels kept free left and right (default 10, Ghostty's window padding);
//   the grid starts at `pad`, like a terminal's
// - cell, line: one glyph cell in pixels (default 14 x 26: Ghostty's grid for
//   CommitMono 12 pt at 2x)
//
// Render at 144 ppi (2x) so 1 pt = 2 px; `just brand` does that and writes
// png/wallpaper-<variant>.png.
#let num(key, default) = int(sys.inputs.at(key, default: str(default)))
#let variant = sys.inputs.at("variant", default: "dark")
#let (bg, ink) = if variant == "light" { (white, black) } else { (black, white) }

#let px = 0.5pt
#let (width, height) = (num("width", 3024), num("height", 1964))
#let band = num("band", 74)
#let pad = num("pad", 10)
#let (cell, line) = (num("cell", 14), num("line", 26))

// Whole cells only.
#let cols = calc.floor((width - 2 * pad) / cell)
#let rows = calc.floor((height - band) / line)
#let row = "^@" * calc.floor(cols / 2) + if calc.odd(cols) { "^" } else { "" }

#set page(width: width * px, height: height * px, margin: 0pt, fill: bg)
// CommitMono advances 0.6 em: 12 pt gives 7.2 pt, the grid wants `cell` px.
// Cap height as the top edge: a row's ink starts at its top, 2 px below
// the band, as in a terminal row.
// The thin stroke on dark stands in for macOS font thickening: light text on
// black draws heavier in Ghostty than the bare outline; dark on white does not.
#let thicken = if variant == "light" { none } else { 0.2pt + ink }
#set text(font: "CommitMono", size: 12pt, fill: ink, stroke: thicken,
  tracking: cell * px - 7.2pt,
  top-edge: "cap-height", bottom-edge: "baseline")

#place(top + left, dx: pad * px, dy: (band + 2) * px,
  stack(..range(rows).map(_ => box(height: line * px, align(top, row)))))
