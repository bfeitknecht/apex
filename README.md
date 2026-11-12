# Apex

The personal site at `0xbf.ch`. One URL, three surfaces and one set of content.

| Visitor | Gets | Served by |
| --- | --- | --- |
| modern browser, `https://0xbf.ch/` | the app, drawn by Ratzilla | `src/frontends/web` |
| old browser, or JavaScript off, same URL | the plain Typst page | `ssg/` (thyme) |
| terminal, `ssh 0xbf.ch` | the same app over SSH | `src/frontends/ssh` |
| your own terminal | the same app natively | `src/frontends/native` |

**One URL.** `/` returns one document. Typst builds the page. A boot script
replaces the page with the WebAssembly app if the browser can run it. The site
has no `/tui`, no `/static` and no second hostname. All other paths hold content.

Every surface shows the same items: introduction, projects, contacts, public
keys and a hidden menu of four visuals (Unicode specimens, a lichen colony, a
quaternion Julia explorer and an EGA framebuffer sweep).

## Quick start

```sh
just                 # list the recipes
just dev             # build both artifacts, then serve :8080 in sws
just native          # terminal portfolio
just ssh             # SSH server on 0.0.0.0:2222 (needs APEX_SSH_HOST_KEY)
just check           # clippy, tests, wasm32 build
```

`just dev` builds the whole site. The build has two parts. Typst builds the
page, Trunk builds the app, and both write into one document root. `just serve`
then serves `./public` with the same `static-web-server` and the same `sws.toml`
as the container. It rewrites `/cv`, it answers unknown paths with the 404 page
of the site, and it sends the same headers. This step needs Docker.

Toolchains: stable Rust, `cargo install trunk` and the `wasm32-unknown-unknown`
target, `typst >= 0.15.1`, `just` and `python3`. Typst downloads
`@preview/thyme` and the packages it imports on the first build.

## Layout

```text
content/            the content of the site: profile.toml, projects.toml,
                    age.txt, pgp.txt, cv.pdf, portrait.txt
src/                the app (see docs/architecture.md)
  content.rs        reads content/ once and freezes it to &'static
  app/              state machine: view, scroll, keys, pointer, tick
  ui/               rendering: &App → cells, hit ranges
  visuals/          simulations: lichen, unicode, fractal, crash
  frontends/        native, ssh, web
ssg/                the page: Typst sources, stylesheet, favicon and webfonts
                    (docs/ssg.md)
browser/            the shell of the app: app.html (Trunk entry), boot.js,
                    app.css, cursor.js
assets/             fonts, Unicode cmaps, licences
sws.toml            server configuration for the document root
justfile            every command: build, docroot, serve, image, release,
                    `just cmaps` for the font catalogue and `just favicon`
CHANGELOG.md        the release notes
```

## Release

`CHANGELOG.md` holds the notes: one dated section per version, in the format of
Keep a Changelog. `just release` reads the version from `Cargo.toml` and the
notes from that section.

```sh
just release v0.1.0 --check   # Cargo.toml, CHANGELOG.md and the tree agree
just release v0.1.0           # gate, tag and push
just release --notes          # the notes alone (CI uses this)
```

The command refuses a dirty tree and a tag that origin does not have. It runs
`just check`, tags the commit with the notes, and pushes the tag. The workflows
then build the image and publish the GitHub release.

## Documentation

- `docs/architecture.md`: the layers, the one-URL model and the rendering split.
- `docs/navigation.md`: views, keys, pointer input and the owner of each part.
- `docs/ssg.md`: the page, its document root, the shared content and the build.
- `docs/go-live.md`: the tasks between now and the launch.

## The SSH portfolio

`ssh 0xbf.ch` shows the same interface as the terminal. It needs no account. The
server accepts any username and asks for no client key. It runs the portfolio
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
  `r` resets the camera. In crash, `r` starts the sweep again. The full table is
  in `docs/navigation.md`.

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
just cmaps /path/to/UnicodeData.txt
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
file names that the page refers to, and the Typst bundle with assertions on its
content.

CI cannot check the work that needs a person: the interface at 80×24, the fonts
in a real browser, the SSH handshake and the image. That list is in
`docs/go-live.md` § 2.

Names: `ssg/` holds the input of the static site generator (Typst sources, the
stylesheet, the favicon and the webfonts). `browser/` holds
the shell of the app (`app.html` for Trunk, `boot.js` which replaces the page
with the app, `app.css`, `cursor.js`). The names describe the toolchains, not a
neutral concept: `ssg/` names the toolchain that produces the document,
`browser/` names the surface that starts the app.
