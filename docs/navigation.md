# Navigation and input

This document describes what the code implements. The interface has one
portfolio page with five sections (about, projects, contact, hire me, click
them), a document viewer, four visuals behind a hidden menu and a
help box on top (`?`, or the `?!` footer link). The same keys work in the browser, in the native terminal and
in an SSH session. Pointer input repeats the keyboard.

## View graph

```text
                 ┌─────────────── portfolio ───────────────┐
                 │ about ⇄ projects ⇄ contact ⇄ hire ⇄ click │
                 └───────┬─────────────────────────┬───────┘
                         │ cv / blog / gallery      │ age, pgp, me
                         ▼                          ▼
                     external link             document ──Esc──▶ portfolio
                         │
   ^@ ^@ ^@  (or Tab + Enter three times on the logo)
                         ▼
                 ┌──────── visual menu ────────┐
                 │ (unicode) (lichen)          │
                 │ (fractal) (crash)           │
                 └──────┬──────────────────────┘
                        │ Enter
                        ▼
             unicode | lichen | fractal | crash ──Esc/h──▶ portfolio
```

When you leave a view, the app returns to the portfolio section, the scroll
position and the focused link that opened it (`App::previous`,
`App::document_scroll`).

## Keys

| Key                              | Portfolio                           | Visuals                                                           |
| -------------------------------- | ----------------------------------- | ----------------------------------------------------------------- |
| `j` `k` / arrows / wheel / swipe | scroll a line                       | –                                                                 |
| `d` `u` / PgDn, PgUp             | scroll half a page                  | –                                                                 |
| `gg` / `G`, Home / End           | top / bottom                        | –                                                                 |
| `h` `l` / `1`–`5`                | previous / next / jump to section   | choose a visual (menu only)                                       |
| Tab / `n` `N`                    | next / previous link                | –                                                                 |
| Enter                            | open the focused link               | open the highlighted visual                                       |
| `-` `+`                          | –                                   | lichen / crash: speed; fractal: zoom                              |
| `0`                              | –                                   | lichen / crash: default speed; fractal: reset                     |
| space                            | –                                   | lichen / crash: pause / resume                                    |
| `r`                              | –                                   | lichen: new seed; fractal: reset camera; crash: restart the sweep |
| `?`                              | toggle help                         | toggle help                                                       |
| Esc                              | close help, then close the document | back to the menu, then to the portfolio                           |
| `q`                              | quit (native / SSH only)            | quit (native / SSH only)                                          |

In lichen, scrolling while the simulation runs changes the pixelation, not the
position. The simulation has a fixed resolution, so zoom is a property of the
display.

## Pointer and touch

Hover reports the item under the cursor (nav item, link, menu entry or visual)
and `click` activates it. Both use `App::hover`, `App::clear_hover` and
`App::click`, so the browser and a terminal with mouse reporting share one hit
test: `ui::nav_ranges` for the top bar, `PortfolioPage::links` for the page and
`ui::visual_ranges` for the hidden menu.

The browser handles text selection natively. A mirrored text layer sits at the
same cell positions, so selection, copy and the browser search work. Cell
positions come from the renderer's own cell size (`set_cell_size`), not from
`window / columns`. The cell cursor and the selection swap black and white:
the selection is one rect per row, snapped to whole cells.

## Boundaries

- Every view is a `View` variant and every transition is an `App` method. A
  frontend only decodes input and calls the method.
- Links come from `content` and carry a `LinkKind`. The terminal shows the
  destination next to the focused link. The browser opens the link.
- Reduced motion: `tick` is the only clock. A frontend that does not call it, or
  calls it with `0`, stops lichen, fractal and crash and changes no state.
