# Architecture

Two versions of one site, **static** and **dynamic**, and one URL. The static
version is plain HTML from Typst. The dynamic version is one crate and one
state machine with three frontends: **browser**, **ssh** and **native**. The
document at the URL is the static version until a browser with WebAssembly
replaces it with the dynamic one.

```text
                    https://0xbf.ch/                ssh -p 2222 0xbf.ch
                 ┌───────────────────┐           ┌──────────────┐
                 │  static (Typst)   │           │    russh     │
                 │  + dynamic if can │           │    (tcp)     │
                 └─────────┬─────────┘           └──────┬───────┘
                           │                            │
               static/ ────┤                            │
                           │ boot.js → /apex.js         │
                  frontends/browser                frontends/ssh
                           │                            │
                           └──────────┬─────────────────┘
                                      │
                     dynamic: app → ui → visuals
                                      │
                              content/*.toml  ← the only copy
```

## Layout

```text
content/               the content of the site: profile.toml, projects.toml,
                       age.txt, pgp.txt, cv.pdf, portrait.txt
src/
├── main.rs            apex        native frontend (and the browser's wasm entry point)
├── bin/ssh.rs         ssh         SSH server entry point (feature `ssh`)
├── lib.rs             module map
├── content.rs         reads content/ and freezes it to &'static
├── app/               state: view, selection, scroll, keys, pointer, tick
├── ui/                rendering: &App → cells (draw, nav, portfolio,
│                      document, visuals, theme)
├── visuals/           simulations: lichen, unicode, fractal, crash
└── frontends/         browser, ssh, native
static/                the static version: Typst sources and the favicon
browser/               the browser frontend's shell: dynamic.html (Trunk entry),
                       boot.js, dynamic.css, cursor.js
assets/                fonts, Unicode cmaps, licences
sws.toml               server configuration for the document root
```

## Five rules

1. **`frontends` → `app` → `content`.** A frontend owns the event loop. It does
   not own navigation. `app` never names a frontend.
2. **`ui` is a pure function of `&App`.** It reads state and returns cells and
   hit ranges. It does not change state and it does not call a backend.
3. **`visuals` does no rendering.** It uses no `ratatui`, no `web-sys` and no
   DOM. Crash returns palette indices, and `ui` decides how a terminal shows a
   colour.
4. **`content/` is data and it is the only copy.** Rust embeds it with
   `include_str!`. Typst reads the same files with `toml(..)`. No sentence, URL
   or project appears twice.
5. **One clock for all frontends.** `App::tick(elapsed_ms) -> bool` advances the
   animation of the current view and reports whether a new frame is due. Native,
   SSH and the browser all call this function.

## One URL, one document

The site has no second path and no second host. `https://0xbf.ch/` returns one
document, the static version:

| Visitor | What happens |
| --- | --- |
| WebAssembly available | `browser/boot.js` removes the static markup, shows the elements of the dynamic version, loads `dynamic.css` and `cursor.js` and imports `/apex.js` |
| `/?static` | `boot.js` returns at once: the static version stays |
| no WebAssembly | nothing runs and the static version stays as Typst built it |
| scripting off | the same result, because the script is the only thing that could change it |
| terminal | `ssh -p 2222 0xbf.ch` shows the dynamic version through russh (the
| sshmux owns port 2222: any username is apex) |

Details of the swap and the document root: section "The static version" below.

## Rendering split in the browser

Ratzilla draws with WebGL2, with Canvas 2D as the first fallback and DOM as the
second. It uses the same cell grid and the same glyphs as the terminal. Two
overlays sit on top:

- `#lichen-pixels` holds the colony as `ImageData`, max-pooled at
  `1 << pixelation`. Zooming therefore does not run the simulation again.
- `#unicode-glyph` holds the current specimen at up to 15× the cell size.

`#selectable-text` mirrors every cell as real text at the same positions. This
layer provides selection, copy and find without one DOM node per cell.

## SSH rendering

`frontends::ssh` gives each session a fixed-size `Buffer`. It renders into the
buffer with the same `ui::draw`, compares it with the previous frame and flushes
every 10 ms. The sessions are independent: each one has its own scroll position,
seeds and visual state.

## Features

- `ssh`: adds `russh` and `tokio`. The `ssh` binary sets
  `required-features = ["ssh"]`, so a plain `cargo build` still targets wasm.

## Navigation and input

This document describes what the code implements. The interface has one
portfolio page with five sections (about, projects, contact, hire me, click
them), a document viewer, four visuals behind a hidden menu and a
help box on top (`?`, or the `?!` footer link). The same keys work in the browser, in the native terminal and
in an SSH session. Pointer input repeats the keyboard.

### View graph

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

When you leave a view, the dynamic version returns to the portfolio section, the scroll
position and the focused link that opened it (`App::previous`,
`App::document_scroll`).

### Keys

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

### Pointer and touch

Hover reports the item under the cursor (nav item, link, menu entry or visual)
and `click` activates it. Both use `App::hover`, `App::clear_hover` and
`App::click`, so the browser and a terminal with mouse reporting share one hit
test: `ui::nav_ranges` for the top bar, `PortfolioPage::links` for the portfolio and
`ui::visual_ranges` for the hidden menu.

The browser handles text selection natively. A mirrored text layer sits at the
same cell positions, so selection, copy and the browser search work. Cell
positions come from the renderer's own cell size (`set_cell_size`), not from
`window / columns`. The cell cursor and the selection swap black and white:
the selection is one rect per row, snapped to whole cells.

### Boundaries

- Every view is a `View` variant and every transition is an `App` method. A
  frontend only decodes input and calls the method.
- Links come from `content` and carry a `LinkKind`. The terminal shows the
  destination next to the focused link. The browser opens the link.
- Reduced motion: `tick` is the only clock. A frontend that does not call it, or
  calls it with `0`, stops lichen, fractal and crash and changes no state.

## The static version (`static/`)

`static/` is plain Typst. Its bundle export writes every file with the two
built-in functions `document(path, …)` and `asset(path, data)`; no package is
involved. It is unstyled on purpose: no stylesheet, no webfont, no inline
style, the browser's own defaults (serif text, blue links), like the early web.
It is the document at `https://0xbf.ch/`, not a fallback behind a separate URL.
A browser that can run WebAssembly replaces it in place with the dynamic
version (see above), unless the URL has `?static`. All other browsers read it
as it is. Its footer links to the dynamic version (`dynamic`, `/`), and the
dynamic version's footer links back (`static`, `/?static`).

### One document root

The container serves one tree with one URL space:

```text
public/
├── index.html                     static (and boot.js, which may swap in dynamic)
├── contact/index.html             a content page
├── 404.html, sitemap.xml, robots.txt, llms.txt
├── assets/favicon.svg, favicon.ico, apple-touch-icon.png
├── age.txt, pgp.txt, cv.pdf       emitted by the same build, from content/
├── apex.js, apex_bg.wasm          dynamic, browser frontend (Trunk)
├── dynamic.css, boot.js, cursor.js  its shell
└── *.ttf, *.otf, *-LICENSE.txt    its specimen fonts
```

Two toolchains write into this directory and their output does not overlap.
`static/` builds the document. `browser/` and `src/` build the assets of the
dynamic version.

The fallback has no separate URL. No visitor is sent to another address, and no
crawler must know about the second version. The cost is the download order: a
modern visitor downloads the static HTML (a few KB) before the dynamic version
replaces it. The asset names of the dynamic version cannot include a content
hash, because a different toolchain builds the static version that loads them. `Trunk.toml` therefore sets
`filehash = false`, and `sws.toml` sends `no-cache` (ETag revalidation).

### Content

`content/` is the only definition and both toolchains read it:

| File | Rust | Typst |
| --- | --- | --- |
| `content/profile.toml` | `include_str!` + `toml` → `content::profile()` | `toml("../content/profile.toml")` |
| `content/projects.toml` | same → `profile().projects` | `toml("../content/projects.toml").project` |
| `content/age.txt`, `pgp.txt`, `cv.pdf` | embedded / linked | emitted with `#asset(…)` |
| `content/portrait.txt` | the `me` document | – |

`static/llms.txt` is written by hand, as in glob. It follows
[llmstxt.org](https://llmstxt.org). `static/robots.txt` points a crawler at the
sitemap. `site.typ` copies both into the bundle root with `#asset`. No build
step generates them, so update `llms.txt` when `projects.toml` changes.

`static/config.typ` is the Typst side of this contract. It parses the TOML and
exposes `destination("cv")`, `contact("github")`, `age`, `flow` (a TOML text
block as one paragraph), `joined` and `footer`, so no page repeats a URL.

The static version renders projects from data in `static/site.typ`, one list item each:
the name as a link, the period and the description.

The dynamic version prints `summary` on one line. The static one prints `description` and
`period`. Both show the same entry, and the difference is typography, not a
second copy.

### Build

```sh
just serve      # both toolchains, then serve ./public on :8080
just serve --static  # Typst only: the static version
just serve --dynamic # Trunk only: the browser frontend over the existing static version
just build      # ./public: page, then dist over it
just static        # ./public: page + artifacts
just browser    # ./dist: the browser frontend's assets only
```

The entry point is `static/site.typ`. It imports `config.typ` and reads
`robots.txt`, `llms.txt` and `assets/` (the favicon from `brand/`). Typst uses
the same flags locally and in the container, and needs no network:

```sh
typst compile --features html,bundle --format bundle --root . static/site.typ public
```

`--root .` lets `static/config.typ` read `../content/…`. Typst marks the bundle
export as experimental; it prints a warning on every build.

### Writing HTML in Typst

The static version uses only `html.*`, the built-in module of Typst 0.15. Three rules
matter:

- **Use the typed constructors** (`html.div`, `html.section`, `html.p`, …).
  Attributes are arguments: `html.div(id: "dynamic-shell", hidden: true)`. Boolean
  attributes take `true`, not a string.
- **Use `html.elem` for names without a constructor.** The lichen tone filter
  needs camelCase SVG names (`feComponentTransfer`, `feFuncR`), so it uses
  `html.elem("feComponentTransfer", …)`.
- **End an expression before a full stop in markup.** `#mail.` reads as a field
  access. Write `#mail;.`.

### Adding a page

```typst
// static/site.typ
#page("notes/", "notes/index.html", title: "Notes · " + profile.name, {
  html.h1("notes")
  footer
})
```

Add the path to `_pages` for the sitemap and link it from the index. The 404
page is the `404.html` document in `site.typ`. Content pages are static only:
the dynamic version runs at `/`, and a content page does not start it.

## Checks CI cannot do

Before a release that changes the look or the build, by hand:

- Run `just build` and open `public/index.html`. The file must contain the
      static version (with `id="dynamic-shell"` and without `<link data-trunk>`),
      not the Trunk stub. Both builds write into one root and the entry document of
      Trunk also has the name `index.html`.
- Run `just serve` and open `http://localhost:8080/`. The static version loads
      and the console shows `/apex.js`; then the dynamic version replaces it.
      `static` in its footer goes to `/?static` in the same tab, and `dynamic`
      there goes back. Block WebAssembly (`delete window.WebAssembly` in the
      console) and reload: the static version must stay.
- Run `just static` alone. `public/index.html`, `contact/`,
      `404.html`, `sitemap.xml`, `robots.txt`, `llms.txt`, `age.txt`, `pgp.txt`
      and `cv.pdf` must exist, and `grep dynamic-shell public/index.html` must
      match.
- Disable JavaScript and reload. The static version must not change, `/cv`
      must work and the contact page must render. There must be no redirect.
- Test touch, keyboard and selection in the dynamic version. Test lichen, Unicode,
      fractal and crash.
