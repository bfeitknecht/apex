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

`ssg/` is a [thyme](https://github.com/bfeitknecht/thyme) site. It uses pure
Typst with no client-side JavaScript, and it builds to static HTML, CSS and the
fonts that the page embeds. It is the document at `https://0xbf.ch/`. It is not
a fallback behind a separate URL. A browser that can run WebAssembly replaces
the page in place (see above). All other browsers read the page
as it is.

### One document root

The container serves one tree with one URL space:

```text
public/
├── index.html                     the page (and the boot script of the app)
├── site.typ                       its plaintext source (Tadi Web view)
├── contact/                       a content page + contact.typ
├── 404.html, sitemap.xml, robots.txt, llms.txt
├── styles.css, assets/fonts/*.woff2, assets/favicon.svg
├── age.txt, pgp.txt, cv.pdf       emitted by the same build, from content/
├── apex.js, apex_bg.wasm          the app (Trunk)
├── app.css, boot.js, cursor.js    the shell of the app
└── *.ttf, *.otf, *-LICENSE.txt    the specimen fonts of the app
```

Two toolchains write into this directory and their output does not overlap.
`ssg/` builds the document. `browser/` and `src/` build the assets of the app.
`just build` and the Dockerfile run `typst … ssg/site.typ public` and then
copy `dist/` over the result.

`sws.toml` describes how the server serves the tree. `/cv` rewrites to
`/cv.pdf`. `.typ` files are `text/plain`. Fonts are immutable. All other files
use `no-cache`, because the asset names of the app have no hash and the shell
must revalidate. Unknown paths fall through to `404.html`.

### Paths belong to content

Every URL on the site is a piece of content. `/contact/` is a page. `/age.txt`,
`/pgp.txt` and `/cv` are files. No path selects a frontend. The browser makes
that decision once, at `/`.

`ssg/config.typ` defines the prefix in one place:

```typst
#let base = ""   // the site is the document root
```

Pages write absolute hrefs as `{base}/…`. A move to another location, for
example a subdirectory on another host, needs one edit and the canonical URL.

### Shared content

`content/` is the only definition and both toolchains read it:

| File | Rust | Typst |
| --- | --- | --- |
| `content/profile.toml` | `include_str!` + `toml` → `content::profile()` | `toml("../content/profile.toml")` |
| `content/projects.toml` | same → `profile().projects` | `toml("../content/projects.toml").project` |
| `content/age.txt`, `pgp.txt`, `cv.pdf` | embedded / linked | emitted with `#file(…)` |
| `content/portrait.txt` | the `me` document | – |

`ssg/llms.txt` is written by hand, as in glob. It follows
[llmstxt.org](https://llmstxt.org). `ssg/robots.txt` points a crawler at the
sitemap. `site.typ` passes both to thyme (`llms: path(...)`, `robots:
path(...)`), which copies the bytes into the bundle root. No build step
generates them, so update `llms.txt` when `projects.toml` changes.

`ssg/config.typ` is the Typst side of this contract. It parses the TOML and
exposes `destination("cv")`, `contact("github")`, `mail`, `age` and `footer`, so
no page repeats a URL.

The page renders projects from data in `ssg/site.typ`:

```typst
#let _entries = for project in projects {
  html.elem("li", attrs: (class: "entry"), {
    html.elem("div", attrs: (class: "entry-head"), {
      html.h3(html.a(href: project.url, [#project.name]))
      html.elem("span", attrs: (class: "when"), [#project.period])
    })
    html.p([#project.description])
  })
}
```

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

The entry point is `ssg/site.typ`, following the convention of thyme. The file
that calls `#show: site.with(…, src: path("site.typ"), …)` is the root document.
thyme writes the source view of that file into the bundle under the name of the
entry file, so the plaintext view is `/site.typ`. All other files in `ssg/` are
either imported by the entry file (`config.typ`, `contact.typ`) or read by it
(`styles.css`, `robots.txt`, `llms.txt`, `assets/`). `favicon.typ` is not part of the build:
`just favicon` renders it to `ssg/assets/favicon.svg`, which the page and the app
share.

Typst uses the same flags locally and in the container:

```sh
typst compile --features html,bundle --format bundle --root . ssg/site.typ public
```

`--root .` lets `ssg/config.typ` read `../content/…` and lets `#file` write the
artifacts at the root.

### Typst packages

`ssg/site.typ` imports `@preview/thyme:0.1.3`, and thyme imports
`@preview/syndicate` for the sitemap and `@preview/cheq`, `showybox`, `tablem`,
`zebraw` and `marge` for its blog preset. All six come from Typst Universe.
Typst downloads them on the first build and caches them in
`~/.cache/typst/packages`. CI and the image build have network access. A fully
offline build needs these packages in a local directory, and `--package-path`
must point at it.

A thyme update touches three places: `ssg/site.typ` and `ssg/contact.typ` carry
the import, and this directory names the version.

### Writing HTML in Typst

The page uses only `html.*`, the built-in module of Typst 0.15. thyme 0.1.2 has
no HTML builder of its own. Three rules matter:

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
