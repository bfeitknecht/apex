# Apex

The personal site at `0xbf.ch`. One URL, two versions and one set of content:

- **static**: plain HTML without CSS, built by Typst from `static/`.
- **dynamic**: a Ratatui program (`src/`) with three frontends: browser,
  ssh and native.

| Visitor | Gets | Served by |
| --- | --- | --- |
| modern browser, `https://0xbf.ch/` | dynamic, browser frontend (Ratzilla, WebAssembly) | `src/frontends/browser` |
| old browser, JavaScript off, or `/?static` | static | `static/` (Typst) |
| terminal, `ssh -p 2222 0xbf.ch` | dynamic, ssh frontend | `src/frontends/ssh` |
| your own terminal, `just native` | dynamic, native frontend | `src/frontends/native` |

**One URL.** `/` returns one document: the static version. A boot script
replaces it with the dynamic version if the browser can run WebAssembly. The
site has no second path and no second hostname. All other paths hold content.

Both versions show the same items: introduction, projects, contacts and public
keys. The dynamic version also has a hidden menu of four visuals (Unicode
specimens, a lichen colony, a quaternion Julia explorer and an EGA framebuffer
sweep).

`/robots.txt` points a crawler at the sitemap. `/llms.txt` describes the site to
a language model ([llmstxt.org](https://llmstxt.org)). Edit
`static/llms.txt` by hand when a page or a project changes.

## Quick start

```sh
just                 # list the recipes
just serve           # build both versions, then serve :8080 (PORT=8090 for another port)
just serve --static  # the static version only
just serve --dynamic # rebuild only the browser frontend over the static version
just native          # the dynamic version in this terminal
just ssh             # the dynamic version over SSH on 0.0.0.0:2222 (needs APEX_SSH_HOST_KEY)
just check           # clippy, tests, wasm32 build
```

`just serve` builds the whole site, then serves it. The build has two parts.
Typst builds the static version and Trunk builds the browser frontend of the
dynamic version, and both write into one
document root. The server is the same `static-web-server` with the same
`sws.toml` as the container: it rewrites `/cv`, it answers unknown paths with
the 404 page of the site, and it sends the same headers. This step needs Docker.
`just build` stops after the document root.

Toolchains: stable Rust, `cargo install trunk` and the `wasm32-unknown-unknown`
target, `typst >= 0.15.1`, `just` and `python3`.

## Layout

```text
content/            the content of the site: profile.toml, projects.toml,
                    age.txt, pgp.txt, cv.pdf, portrait.txt
src/                the dynamic version (see docs/architecture.md)
  content.rs        reads content/ once and freezes it to &'static
  app/              state machine: view, scroll, keys, pointer, tick
  ui/               rendering: &App → cells, hit ranges
  visuals/          simulations: lichen, unicode, fractal, crash
  frontends/        browser, ssh, native
static/             the static version: Typst sources (no stylesheet) and the
                    favicon (docs/architecture.md)
browser/            the shell of the dynamic version in the browser:
                    dynamic.html (Trunk entry), boot.js, dynamic.css, cursor.js
assets/             fonts, Unicode cmaps, licences
brand/              the 0xbf mark (Typst source, SVG, PNGs, favicon.ico) and
                    palette; every copy elsewhere is listed in brand/README.md
sws.toml            server configuration for the document root
justfile            every command: `serve`, `build`, `static`, `browser`, `native`,
                    `ssh`, `image`, `release`, `brand` and `unicode`
CHANGELOG.md        the release notes
```

## Static and dynamic

`https://0xbf.ch/?static` shows the static version in any browser: the boot
script skips the dynamic one. The dynamic version's footer links to it
(`</>`), and the static version's footer links back. Locally,
`just serve --static` builds `static/` and serves only the static version on
`:8080`.

## Polyring

The footer carries the Polyring webring (`<< O >>`: previous, hub, next) in the
dynamic version (every frontend) and the static one. The ring has no redirect service, so
`content/polyring.toml` holds the two neighbours. Refresh them with
`just polyring` (it reads the ring's own `embed.js`) and release.

## Release

`CHANGELOG.md` holds the notes: one dated section per version, in the format of
Keep a Changelog. `just release` reads the version from `Cargo.toml` and the
notes from that section.

```sh
just release --check           # Cargo.toml, CHANGELOG.md and the tree agree
just release                   # gate, tag v<Cargo.toml version> and push
just release --notes           # the notes alone (CI uses this)
```

The command refuses a dirty tree and a tag that origin does not have. It runs
`just check`, tags the commit with the notes, and pushes the tag. The workflows
then build the image and publish the GitHub release.

## Documentation

- `docs/architecture.md`: the layers, the one-URL model, the rendering split, navigation and input, the static version (`static/`), and the checks CI cannot do.

## The SSH portfolio

`ssh -p 2222 0xbf.ch` shows the same interface as the terminal. It needs no
account. Port `2222` is the sshmux (infra): the usernames `someone`, `apex` and
`guest` land in apex; press Enter at the mux password prompt. apex-ssh behind it
accepts any username and asks for no client key. It runs the portfolio
only: no shell, no commands, no SFTP and no forwarding. It needs a PTY. It
allows 32 sessions, it times out after 15 idle minutes and it limits the
terminal to 20–240 × 8–120 cells.

```sh
ssh-keygen -t ed25519 -N '' -f ~/.config/apex/ssh_host_ed25519_key   # once
APEX_SSH_HOST_KEY="$HOME/.config/apex/ssh_host_ed25519_key" just ssh
ssh -p 2222 localhost                                              # another terminal
```

The server prints the SHA-256 fingerprint of its host key at startup. Check the
fingerprint out of band before you trust the host. The deployment (two pods,
Traefik routes, Gatus checks and the host-key Secret) is in the `infra`
repository.

| Variable | Default | Meaning |
| --- | --- | --- |
| `APEX_SSH_LISTEN` | `0.0.0.0:2222` | bind address (the cluster routes 22 → 2222) |
| `APEX_SSH_HOST_KEY` | — (required) | private host key |
| `APEX_SSH_PUBLIC` | `1` | accept any username |
| `APEX_SSH_USER` | `apex` | the only username in key-only mode |
| `APEX_SSH_AUTHORIZED_KEYS` | — | allowlist file; `APEX_SSH_PUBLIC=0` for key-only |

If you set an allowlist and do not select a mode, the server does not start.
This behaviour prevents an accidental public service.

## Navigation

- `j`/`k`, arrow keys, wheel and swipe scroll by one line. `gg`/`G` and
  Home/End jump to the top or to the bottom.
- `h`/`l` and `1`–`3` select about, projects or contact. Tab and `n`/`N` move to
  the next or previous link.
- Enter opens the focused link. A terminal shows the destination next to the
  link. `?` opens the help, Esc closes it and `q` quits.
- Hidden visuals: press `^@` three times, or press Tab and Enter three times on
  the logo. Then use `h`/`l` to choose unicode, lichen, fractal or crash.
- In lichen, space pauses the run, `-`/`+` set the interval, `0` resets it and
  scrolling changes the pixelation. In fractal, `h/j/k/l` pan, `+`/`-` zoom and
  `r` resets the camera. Crash takes the speed keys of lichen (space, `-`/`+`,
`0`), and `r` starts the sweep again. The full table is
  in `docs/architecture.md`.

## Lichen and the Unicode visual

The lichen colony is a differential-growth simulation. Terminals draw it with
half-blocks. The browser draws it with pooled pixels. The browser sizes its
cells from the viewport, so zooming does not run the simulation again. The
output is an approximation: single pixels can differ from the original. It does
not use sixel.

The Unicode visual picks a code point from the bundled font cmaps and shows its
Unicode 17 name, block, font and position. The cmaps hold 106,701 candidates.
The build drops control, formatting, whitespace, combining, private-use and
unassigned characters. 79,761 candidates have outline fonts. GNU Unifont covers
the rest, so some glyphs look pixelated. The visual does not show every Unicode
glyph: some assigned characters have no font coverage and others need contextual
shaping. The font licences are in `assets/`.

To rebuild the catalogue you need `fontTools` and the Unicode 17
`UnicodeData.txt`:

```sh
just unicode /path/to/UnicodeData.txt
```

The command rewrites `assets/{unifont,noto}-cmap.bin`.
`src/visuals/unicode.rs` embeds both files at compile time. The BabelStoneHan
TTF (52 MB) is only an input for that command. The browser loads the 7 MB WOFF2
file.

## Status

The tree compiles for both targets, the tests pass and both artifacts build. CI
checks this on every push. The first `cargo fmt`, `clippy` and build ran in
Actions, because the sandbox where the code was written can reach GitHub, PyPI
and npm but not `static.rust-lang.org` or `crates.io`. `CI` runs formatting,
lints, tests, the `wasm32` check, `trunk build --release` with assertions on the
file names that the static version refers to, and the Typst bundle with assertions on its
content.

CI cannot check the work that needs a person: the interface at 80×24, the fonts
in a real browser, the SSH handshake and the image. That list is in
`docs/architecture.md`, "Checks CI cannot do".

Names: two versions, **static** and **dynamic**, and no other word for them.
`static/` holds the static version (Typst sources, the favicon). `src/` is the
dynamic version; its frontends are **browser**, **ssh** and **native**
(`src/frontends/`). `browser/` holds the browser frontend's shell
(`dynamic.html` for Trunk, `boot.js` which swaps the static version for the
dynamic one, `dynamic.css`, `cursor.js`). The Rust type `App` is the dynamic
version's state machine, not a third name.
