# Architecture

One crate, one state machine and three ways to show it. The site has one URL.
The document there is the plain page until a browser with WebAssembly replaces
it with the app.

```text
                    https://0xbf.ch/                ssh -p 2222 0xbf.ch
                 ┌───────────────────┐           ┌──────────────┐
                 │  the page (Typst) │           │    russh     │
                 │  + app if it can  │           │    (tcp)     │
                 └─────────┬─────────┘           └──────┬───────┘
                           │                            │
                  ssg/ ────┤                            │
                           │ boot.js → /apex.js         │
                    frontends/web                  frontends/ssh
                           │                            │
                           └──────────┬─────────────────┘
                                      │
                              app → ui → visuals
                                      │
                              content/*.toml  ← the only copy
```

## Layout

```text
content/               the content of the site: profile.toml, projects.toml,
                       age.txt, pgp.txt, cv.pdf, portrait.txt
src/
├── main.rs            apex        native terminal (and the wasm entry point)
├── bin/ssh.rs         ssh         SSH server entry point (feature `ssh`)
├── lib.rs             module map
├── content.rs         reads content/ and freezes it to &'static
├── app/               state: view, selection, scroll, keys, pointer, tick
├── ui/                rendering: &App → cells (draw, nav, portfolio,
│                      document, visuals, theme)
├── visuals/           simulations: lichen, unicode, fractal, crash
└── frontends/         native, ssh, web
ssg/                   the page: Typst sources, stylesheet, favicon, webfonts
browser/               the shell of the app: app.html (Trunk entry), boot.js,
                       app.css, cursor.js
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
5. **One clock for all surfaces.** `App::tick(elapsed_ms) -> bool` advances the
   animation of the current view and reports whether a new frame is due. Native,
   SSH and the browser all call this function.

## One URL, one document

The site has no `/tui`, no `/static` and no second host. `https://0xbf.ch/`
returns one document:

| Visitor | What happens |
| --- | --- |
| WebAssembly available | `browser/boot.js` removes the static markup, shows the surfaces of the app, loads `app.css` and `cursor.js` and imports `/apex.js` |
| no WebAssembly | nothing runs and the page stays as Typst built it |
| scripting off | the same result, because the script is the only thing that could change the page |
| terminal | `ssh -p 2222 0xbf.ch` shows the same interface through russh (the
| sshmux owns port 2222: any username is apex) |

The fallback has no separate URL. No visitor is sent to another address, and no
crawler must know about the second surface. The cost is the download order: a
modern visitor downloads the page HTML and CSS before the app replaces them,
which is about 30 KB against a wasm payload of several MB. The asset names of
the app cannot include a content hash, because a different toolchain builds the
page that loads them. `Trunk.toml` therefore sets `filehash = false`, and
`sws.toml` sends `no-cache` (ETag revalidation) for everything except the
webfonts.

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

When you leave a view, the app returns to the portfolio section, the scroll
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
test: `ui::nav_ranges` for the top bar, `PortfolioPage::links` for the page and
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

## The page (`ssg/`)

`ssg/` is plain Typst. Its bundle export writes every file with the two
built-in functions `document(path, …)` and `asset(path, data)`; no package is
involved. The page is unstyled on purpose: no stylesheet, no webfont, no
inline style, the browser's own defaults (serif text, blue links), like the
early web. It is the document at `https://0xbf.ch/`. It is not a fallback
behind a separate URL. A browser that can run WebAssembly replaces the page in
place (see above), unless the URL has `?static`. All other browsers read the
page as it is. The page's footer links back to the app (`/`), and the app's
footer links to the page (`static`, `/?static`).

### One document root

The container serves one tree with one URL space:

```text
public/
├── index.html                     the page (and the boot script of the app)
├── contact/index.html             a content page
├── 404.html, sitemap.xml, robots.txt, llms.txt
├── assets/favicon.svg, favicon.ico, apple-touch-icon.png
├── age.txt, pgp.txt, cv.pdf       emitted by the same build, from content/
├── apex.js, apex_bg.wasm          the app (Trunk)
├── app.css, boot.js, cursor.js    the shell of the app
└── *.ttf, *.otf, *-LICENSE.txt    the specimen fonts of the app
```

Two toolchains write into this directory and their output does not overlap.
`ssg/` builds the document. `browser/` and `src/` build the assets of the app.

The fallback has no separate URL. No visitor is sent to another address, and no
crawler must know about the second surface. The cost is the download order: a
modern visitor downloads the page HTML (a few KB) before the app replaces it.
The asset names of the app cannot include a content hash, because a different
toolchain builds the page that loads them. `Trunk.toml` therefore sets
`filehash = false`, and `sws.toml` sends `no-cache` (ETag revalidation).

### Content

`content/` is the only definition and both toolchains read it:

| File | Rust | Typst |
| --- | --- | --- |
| `content/profile.toml` | `include_str!` + `toml` → `content::profile()` | `toml("../content/profile.toml")` |
| `content/projects.toml` | same → `profile().projects` | `toml("../content/projects.toml").project` |
| `content/age.txt`, `pgp.txt`, `cv.pdf` | embedded / linked | emitted with `#asset(…)` |
| `content/portrait.txt` | the `me` document | – |

`ssg/llms.txt` is written by hand, as in glob. It follows
[llmstxt.org](https://llmstxt.org). `ssg/robots.txt` points a crawler at the
sitemap. `site.typ` copies both into the bundle root with `#asset`. No build
step generates them, so update `llms.txt` when `projects.toml` changes.

`ssg/config.typ` is the Typst side of this contract. It parses the TOML and
exposes `destination("cv")`, `contact("github")`, `age`, `flow` (a TOML text
block as one paragraph), `joined` and `footer`, so no page repeats a URL.

The page renders projects from data in `ssg/site.typ`, one list item each:
the name as a link, the period and the description.

The terminal prints `summary` on one line. The page prints `description` and
`period`. Both show the same entry, and the difference is typography, not a
second copy.

### Build

```sh
just serve      # both toolchains, then serve ./public on :8080
just serve --static # Typst only: the page without the app
just serve --app    # Trunk only: the app over the existing page
just build      # ./public: page, then dist over it
just ssg        # ./public: page + artifacts
just web        # ./dist: frontend assets only
```

The entry point is `ssg/site.typ`. It imports `config.typ` and reads
`robots.txt`, `llms.txt` and `assets/` (the favicon from `brand/`). Typst uses
the same flags locally and in the container, and needs no network:

```sh
typst compile --features html,bundle --format bundle --root . ssg/site.typ public
```

`--root .` lets `ssg/config.typ` read `../content/…`. Typst marks the bundle
export as experimental; it prints a warning on every build.

### Writing HTML in Typst

The page uses only `html.*`, the built-in module of Typst 0.15. Three rules
matter:

- **Use the typed constructors** (`html.div`, `html.section`, `html.p`, …).
  Attributes are arguments: `html.div(id: "app-shell", hidden: true)`. Boolean
  attributes take `true`, not a string.
- **Use `html.elem` for names without a constructor.** The lichen tone filter
  needs camelCase SVG names (`feComponentTransfer`, `feFuncR`), so it uses
  `html.elem("feComponentTransfer", …)`.
- **End an expression before a full stop in markup.** `#mail.` reads as a field
  access. Write `#mail;.` (see `contact.typ`).

### Adding a page

```typst
// ssg/site.typ
#mount(path("notes.typ"))
```

`#mount` creates the route, the sitemap entry and the link. The 404 page is
`_not-found` in `site.typ`. Content pages are static: the app runs at `/` and a
content page does not start it.

## Checks CI cannot do

Before a release that changes the look or the build, by hand:

- Run `just build` and open `public/index.html`. The file must contain the
      page (with `id="app-shell"` and without `<link data-trunk>`), not the
      Trunk stub. Both builds write into one root and the entry document of
      Trunk also has the name `index.html`.
- Run `just serve` and open `http://localhost:8080/`. The page loads and the
      console shows `/apex.js`; then the interface replaces the page. Block
      WebAssembly (`delete window.WebAssembly` in the console) and reload. The
      page must stay.
- Run `just ssg` alone. `public/index.html`, `site.typ`, `contact/`,
      `404.html`, `sitemap.xml`, `robots.txt`, `llms.txt`, `age.txt`, `pgp.txt`
      and `cv.pdf` must exist, and `grep app-shell public/index.html` must
      match.
- Disable JavaScript and reload. The page must not change, `/cv` must work
      and the contact page must render. There must be no `/static` and no
      redirect.
- Test touch, keyboard and selection in the app. Test lichen, Unicode,
      fractal and crash.
