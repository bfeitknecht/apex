# Architecture

One crate, one state machine, three ways to look at it — and one URL, where the
document is the plain page until a WebAssembly-capable browser replaces it with
the app.

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
                              content/*.toml  ← the one definition
```

## Layout

```text
content/               THE definition: profile.toml, projects.toml,
                       age.txt, pgp.txt, cv.pdf, portrait.txt
src/
├── main.rs            apex        native terminal (and the wasm entry point)
├── bin/ssh.rs         ssh         SSH server entry point (feature `ssh`)
├── lib.rs             module map
├── content.rs         parses content/, freezes it to &'static
├── app/               state: view, selection, scroll, keys, pointer, tick
├── ui/                rendering: &App → cells (draw, nav, portfolio,
│                      document, visuals, theme)
├── visuals/           render-free simulation: lichen, unicode, fractal, crash
└── frontends/         native, ssh, web
ssg/                   the page: Typst sources, styles, vendor/{thyme,exemel}
browser/               the app's shell: app.html (Trunk entry), boot.js,
                       app.css, cursor.js
assets/                fonts, Unicode cmaps, licences
sws.toml               how the container serves the document root
```

## Five rules

1. **`frontends` → `app` → `content`.** A frontend owns the event loop; it
   never owns navigation. `app` never names a frontend.
2. **`ui` is a pure function of `&App`.** It reads, renders and returns hit
   ranges — it never mutates and never talks to a backend.
3. **`visuals` is render-free.** No `ratatui`, no `web-sys`, no DOM. Crash
   returns palette _indices_; `ui` decides that a terminal colour is a colour.
4. **`content/` is data, and it is the only copy.** Rust embeds it with
   `include_str!`, Typst reads the same files with `toml(..)`. No sentence, URL
   or project exists twice.
5. **One pacing for everyone.** `App::tick(elapsed_ms) -> bool` advances
   whatever the current view animates and answers "is a new frame due?". Native,
   SSH and the browser all call exactly that.

## One URL, two surfaces

There is no `/tui`, no `/static`, no second host. `https://0xbf.ch/` is a
single document:

| Visitor               | What happens                                                                                                              |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| WebAssembly available | `browser/boot.js` drops the static markup, shows the app's surfaces, loads `app.css` + `cursor.js` and imports `/apex.js` |
| no WebAssembly        | nothing runs; the page stays as Typst built it                                                                            |
| scripting off         | same as above — the script is the only thing that could change it                                                         |
| terminal              | `ssh 0xbf.ch` → the same interface through russh                                                                          |

The consequence worth naming: the fallback is not _behind_ a URL, so no visitor
is ever sent somewhere else, and no crawler needs to know that a second surface
exists. The price is that a modern visitor downloads the page's HTML and CSS
before the app replaces it (≈30 KB, against a multi-MB wasm payload) — and that
the app's asset names cannot be content-hashed, because the page that boots them
is built by a different toolchain. `Trunk.toml` therefore sets
`filehash = false` and `sws.toml` sends `no-cache` (ETag revalidation) for
everything except the webfonts.

## Rendering split in the browser

Ratzilla (WebGL2, falling back to Canvas 2D, then DOM) draws the interface
exactly as the terminal draws it — same cell grid, same glyphs. Two overlays
sit on top:

- `#lichen-pixels` — the colony as `ImageData`, max-pooled at
  `1 << pixelation`, so zooming never re-runs the simulation.
- `#unicode-glyph` — the current specimen at up to 15× its cell size.

Underneath, `#selectable-text` mirrors every cell as real text at the same
positions, which is what makes selection, copy and text-layer hit testing work
without a DOM node per cell.

## SSH rendering

`frontends::ssh` gives each session a fixed-size `Buffer`, renders into it with
the same `ui::draw`, diffs it and flushes at 10 ms. Sessions are independent:
their own scroll position, their own seeds, their own visual state.

## Features

- `ssh` — pulls in `russh` + `tokio`; the `ssh` binary is
  `required-features = ["ssh"]`, so a plain `cargo build` stays wasm-friendly.
