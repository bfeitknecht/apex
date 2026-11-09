# Navigation and input

What is implemented, not what was proposed. Three portfolio sections, a
document viewer, four visuals behind a hidden menu, and help on top of it all.
The same keys work in the browser, the native terminal and an SSH session;
pointer input mirrors the keyboard.

## View graph

```text
                 ┌─────────────── portfolio ───────────────┐
                 │  about (0) ⇄ projects (1) ⇄ contact (2) │
                 └───────┬─────────────────────────┬───────┘
                         │ glob / cv / gallery      │ age, pgp, me
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

Leaving anything restores the portfolio section, scroll position and focused
link it was opened from (`App::previous`, `App::document_scroll`).

## Keys

| Key | Portfolio | Visuals |
| --- | --- | --- |
| `j` `k` / arrows / wheel / swipe | scroll a line | – |
| `d` `u` / PgDn, PgUp | scroll half a page | – |
| `gg` / `G`, Home / End | top / bottom | – |
| `h` `l`, `1`–`3` | previous / next section | choose a visual (menu only) |
| Tab / `n` `N` | next / previous link | – |
| Enter | open the focused link | open the highlighted visual |
| `-` `+` | – | lichen: interval; fractal: zoom |
| `0` | – | lichen / fractal: reset |
| space | – | lichen: pause / resume |
| `r` | – | lichen: new seed; fractal: reset camera; crash: restart the sweep |
| `?` / `H` | help | help |
| Esc | close help, then close the document | back to the menu, then to the portfolio |
| `q` | quit (native / SSH only) | quit (native / SSH only) |

In lichen, scrolling while it runs changes pixelation instead of position: the
simulation has a fixed resolution, so zoom is a display concern.

## Pointer and touch

Hover reports what is under the cursor (nav item, link, menu entry, visual) and
`click` activates it. Both go through `App::hover` / `App::clear_hover` /
`App::click`, so the browser and a terminal with mouse reporting share one hit
test: `ui::nav_ranges` for the top bar, `PortfolioPage::links` for the page,
`ui::visual_ranges` for the hidden menu.

Text selection in the browser is native: a mirrored text layer sits at the same
cell positions, so selection, copy and the browser's own find work.

## Boundaries

- Every view is a `View` variant and every transition is an `App` method. A
  frontend only decodes input and calls it.
- Links are rendered from `content` and carry a `LinkKind`; the terminal shows
  the destination next to the focused link, the browser opens it.
- Reduced motion: `tick` is the only clock. A frontend that does not call it
  (or calls it with `0`) stops lichen, fractal and crash without touching state.
