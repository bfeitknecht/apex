# Architecture

One crate, one state machine and three ways to show it. The site has one URL.
The document there is the plain page until a browser with WebAssembly replaces
it with the app.

```text
                    https://0xbf.ch/                ssh 0xbf.ch
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
| terminal | `ssh 0xbf.ch` shows the same interface through russh |

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
