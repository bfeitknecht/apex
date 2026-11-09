# Apex

The personal site at `0xbf.ch`: one interface, three surfaces, one definition of
the content they all show.

| Visitor | Gets | Served by |
| --- | --- | --- |
| modern browser, `https://0xbf.ch/` | the app, drawn by Ratzilla | `src/frontends/web` |
| old browser, or JavaScript off, same URL | the plain Typst page | `ssg/` (thyme) |
| terminal, `ssh 0xbf.ch` | the same app over SSH | `src/frontends/ssh` |
| your own terminal | the same app natively | `src/frontends/native` |

**One URL.** `/` is a single document: the page Typst builds, plus a boot script
that swaps in the WebAssembly app when the browser can run it. There is no
`/tui`, no `/static`, no second hostname — every other path is content.

The portfolio is identical everywhere: introduction, projects, contacts, public
keys, and a hidden menu of four visuals (Unicode specimens, a lichen colony, a
quaternion Julia explorer, an EGA framebuffer sweep).

## Quick start

```sh
just                 # list the recipes
just dev             # both toolchains, then serve :8080 in sws
just native          # terminal portfolio
just ssh             # SSH server on 0.0.0.0:2222 (needs APEX_SSH_HOST_KEY)
just check           # clippy, tests, wasm32 build
```

`just dev` is the one to use: the site is two builds into one document root
(Typst for the page, Trunk for the app), and `just serve` then serves `./public`
with the same `static-web-server` and `sws.toml` the container uses — rewriting
`/cv`, answering 404 with the page's own 404, sending the same headers. It needs
Docker, nothing else.

Toolchains: stable Rust, `cargo install trunk` plus the
`wasm32-unknown-unknown` target, `typst >= 0.15.1`, `just`, `python3`. The
vendored Typst packages need no installation — `just packages` lays them out for
`--package-path`.

## Layout

```text
content/            THE definition: profile.toml, projects.toml,
                    age.txt, pgp.txt, cv.pdf, portrait.txt
src/                the app (see docs/architecture.md)
  content.rs        parses content/ once, freezes it to &'static
  app/              state machine: view, scroll, keys, pointer, tick
  ui/               rendering: &App → cells, hit ranges
  visuals/          render-free simulation: lichen, unicode, fractal, crash
  frontends/        native, ssh, web
ssg/                the page: Typst sources + vendored thyme (docs/ssg.md)
browser/            the app's shell: app.html (Trunk entry), boot.js,
                    app.css, cursor.js
assets/             fonts, Unicode cmaps, licences
infra/              everything the infra repo needs; deleted at go-live
sws.toml            how the container serves the document root
justfile            the recipes (build, docroot, serve, image, release)
```

## Documentation

- `docs/architecture.md` — layering, the one-URL model, the rendering split.
- `docs/navigation.md` — views, keys, pointer input, what each frontend owns.
- `docs/ssg.md` — the page: document root, shared content, build.
- `docs/go-live.md` — the shortlist between here and going live.
- `infra/README.md` — the infra integration, with exact paths and merge notes.

## The SSH portfolio

`ssh 0xbf.ch` serves the same interface as the terminal, without an account:
**by default any username is accepted** and no client key is required. It runs
the portfolio only — no shell, no commands, no SFTP, no forwarding — requires a
PTY, caps sessions at 32, times out after 15 idle minutes and clamps the
terminal to 20–240 × 8–120 cells.

```sh
ssh-keygen -t ed25519 -N '' -f ~/.config/apex/ssh_host_ed25519_key   # once
APEX_SSH_HOST_KEY="$HOME/.config/apex/ssh_host_ed25519_key" just ssh
ssh -p 2222 localhost                                              # another terminal
```

The server prints its SHA-256 host-key fingerprint at startup; verify it out of
band before trusting the host. `infra/sshd_config.example` documents the
alternative of an external `sshd` with `ForceCommand`.

| Variable | Default | Meaning |
| --- | --- | --- |
| `APEX_SSH_LISTEN` | `0.0.0.0:2222` | bind address (the cluster routes 22 → 2222) |
| `APEX_SSH_HOST_KEY` | — (required) | private host key |
| `APEX_SSH_PUBLIC` | `1` | accept any username |
| `APEX_SSH_USER` | `apex` | the only username in key-only mode |
| `APEX_SSH_AUTHORIZED_KEYS` | — | allowlist file; `APEX_SSH_PUBLIC=0` for key-only |

Setting an allowlist without choosing a mode explicitly makes the server refuse
to start, so an accidental public service is impossible.

## Navigation

- `j`/`k`, arrows, wheel, swipe: scroll. `gg`/`G`, Home/End: top/bottom.
- `h`/`l`, `1`–`3`: about/projects/contact. Tab, `n`/`N`: next/previous link.
- Enter opens the focused link; in a terminal the destination is shown beside
  it. `?` toggles help, Esc closes, `q` quits.
- Hidden visuals: activate `^@` three times (or Tab + Enter three times on the
  logo), then `h`/`l` to choose among unicode, lichen, fractal and crash.
- In lichen: space pauses, `-`/`+` set the interval, `0` resets it, scrolling
  changes pixelation. In fractal: `h/j/k/l` pan, `+`/`-` zoom, `r` resets.
  In crash: `r` starts the sweep over. Full table: `docs/navigation.md`.

## Lichen and the Unicode visual

The colony is a differential-growth simulation, rendered as half-blocks in
terminals and as pooled pixels in the browser; the browser sizes its cells from
the viewport, so zooming never re-runs the simulation. It is not a verified
pixel-perfect reconstruction of the original, and it is not sixel.

The Unicode visual samples a code point from the bundled font cmaps — 106,701
candidates after dropping control, formatting, whitespace, combining,
private-use and unassigned characters — and shows its Unicode 17 name, block,
font and position. 79,761 of them have outline fonts; GNU Unifont covers the
rest, which is why some glyphs are visibly pixelated. This is not every Unicode
glyph: some assigned characters have no coverage, some need contextual shaping.
Font licences live in `assets/`.

Rebuild the catalogue (needs `fontTools` and Unicode 17 `UnicodeData.txt`):

```sh
python3 scripts/unicode-candidates.py /path/to/UnicodeData.txt
```

That rewrites `assets/{unifont,noto}-cmap.bin`, which `src/visuals/unicode.rs`
embeds at compile time. The 52 MB BabelStoneHan TTF is only an input to that
script; the browser ships the 7 MB WOFF2.

## Status

The tree compiles on both targets, its tests pass, and both artifacts build.
That is checked by CI on every push, because it could not be checked here: the
sandbox this was written in reaches GitHub, PyPI and npm but not
`static.rust-lang.org` or `crates.io`, so the first `cargo fmt`/`clippy`/build
happened in Actions. `CI` now runs formatting, lints, tests, the `wasm32` check,
`trunk build --release` with assertions on the names the page refers to, and the
Typst bundle with assertions on its shape.

What CI cannot see is anything that needs eyes: the interface at 80×24, the
fonts actually loading, the SSH handshake, the image. That list is
`docs/go-live.md` § 2.

Naming: `ssg/` is the static site generator's input (Typst sources, the vendored
thyme and exemel, the stylesheet and webfonts) and `browser/` is the app's shell
(`app.html` for Trunk, `boot.js` that swaps the page for the app, `app.css`,
`cursor.js`). Neither directory is technology-neutral, on purpose: `ssg/` names
the toolchain that produces the document, `browser/` names the surface that
boots the app over it.
