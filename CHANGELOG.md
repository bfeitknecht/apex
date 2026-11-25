# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.22] - 2026-11-25

### Changed

- Two names for the two versions, everywhere: **static** (plain HTML from Typst) and **dynamic** (the Ratatui program; frontends browser, ssh, native). No more page, app, TUI, surface or ssg for them.
- The static version is its own thing: raw HTML, still no CSS, but loud with HTML's own elements (scrolling banners, the ^@ mark bouncing in a marquee, highlighted destinations, a bordered projects table, the keys folded into `<details>`) and its own words in `static/site.typ`. It shares only the projects, the keys and the links with the dynamic version. The contact page is gone: `/` has everything.
- The dynamic version's footer links to the static one with `</>` (was `static`).
- `ssg/` is `static/`; `just ssg` is `just static`, `just web` is `just browser`, `just serve --app` is `just serve --dynamic`; `src/frontends/web` is `src/frontends/browser`; `browser/app.{html,css}` are `browser/dynamic.{html,css}` (served as `/dynamic.css`); `#app-shell` is `#dynamic-shell`; Docker stages `static` and `dynamic`.

## [0.1.21] - 2026-11-25

### Changed

- The static page is plain HTML without CSS: the browser's own styles, no webfont, no thyme. Typst's bundle export writes every file (`document`, `asset`); the build needs no network.
- omni: 2025 – Present.

### Added

- `static` in the footer of the app: the plain page (`/?static`), in the same tab in the browser. The page's footer links back to the app.

### Removed

- `ssg/styles.css`, the webfonts of the page, `ssg/contact.typ` (now in `site.typ`) and the published Typst source (`/site.typ`).

## [0.1.20] - 2026-11-24

### Changed

- The page uses the shared 0xbf palette of the blog and the gallery (`brand/palette.json`): light `96/21/47`, dark `17/95/70` (was `99/18/52` and `18/93/65`). Rules and frames follow its surface and text steps.

### Added

- `brand/`: the mark's Typst source (light and dark variant), SVGs, PNGs from 16 to 1024 px, `favicon.ico`, the palette as JSON and CSS. `just brand` renders them and replaces `just favicon`.

## [0.1.19] - 2026-11-24

### Changed

- Top bar on narrow screens: `gallery` shortens to `glry`, then to `g` on the narrowest phones (was `gal`). Labels of four characters or fewer (`cv`, `blog`) stay whole.
- The blog and gallery links are `blog.0xbf.ch` and `gallery.0xbf.ch` again (0.1.18 renamed them by mistake).

## [0.1.18] - 2026-11-24

### Changed

- The blog and the gallery have new hostnames: `blg.0xbf.ch` and `glry.0xbf.ch` (profile, projects, `llms.txt`). The old names redirect.

## [0.1.17] - 2026-11-20

### Changed

- The page builds on `@preview/thyme:0.1.3` (was 0.1.2).

## [0.1.16] - 2026-11-19

### Added

- `/favicon.ico` and `/apple-touch-icon.png` at the root, rendered from the round mark. Safari, iOS and tools that ignore the SVG link now show the same icon.

## [0.1.15] - 2026-11-19

### Changed

- The page builds on `@preview/thyme:0.1.2` (was 0.1.0).

## [0.1.14] - 2026-11-19

### Fixed

- The cell cursor, the text selection, the specimen glyph and the hover and click hit-tests use the renderer's own cell size. They took `window / columns`, which is a little too large, so they drifted off the characters towards the right and the bottom.
- The selection is one highlight per row, snapped to whole cells. Overlapping and gapped rects drew thin vertical lines under the inverting filter.

## [0.1.13] - 2026-11-19

### Changed

- The cell cursor and the text selection in the browser swap black and white. Muted and dim text under them turns full black (dark theme) or full white (light theme) instead of another grey.

## [0.1.12] - 2026-11-19

### Added

- syndicate in the project list (#06).

### Changed

- The help box is shorter and plainer. Its right border no longer cuts through long lines.

## [0.1.11] - 2026-11-18

### Added

- **`?!` in the footer opens a floating site overview**: navigation (keys, mouse, the
  section bar), the ssh route (`ssh -p 2222 0xbf.ch`, the sshmux port; `browser@`
  gets browsh), the easter eggs (the triple-click logo menu, the Polyring, `?!`
  itself) and the visuals. `?` still toggles it from the keyboard, `esc` closes.
- The overview is a centred box over a cleared frame, on every frontend
  (terminal, SSH, browser).

### Changed

- Docs and the help name the new ssh port: `-p 2222` (the sshmux; any username
  is apex).


## [0.1.10] - 2026-11-18

### Added

- Terminal sections "hire me" (the word cv links to the cv) and "click them" (blog, gallery). Arrow keys, h/l and 1-5 reach them; the top bar lights (cv) in "hire me" and (blog) and (gallery) together in "click them".
- Project exhib in the list.

### Changed

- One space around every pipe, top bar and footer alike; the footer starts one column further left, like the top bar.
- Profile text reflowed into shorter lines.

## [0.1.9] - 2026-11-17

### Changed

- The footer is one line at every width, in the terminal and on the page. It sheds text in steps (`made with <3 by me.` > `<3 by me.` > `me.`, `age.txt` > `age`, the sitemap link) instead of wrapping. The page uses container queries on the footer's own width.

## [0.1.8] - 2026-11-17

### Changed

- The mark is "^@" on a filled black circle (was a square). Transparent around the circle.
- Polyring footer is `<< O >>` (previous, hub, next). No random member: the ring has no redirect service, and a member list would need upkeep. `content/polyring.toml` holds the two neighbours; `just polyring` refreshes them.
- `?static` (was `?plain`) shows the Typst page. `just serve --static` (page only) and `just serve --app` (rebuild only the app over the page); `just plain` is gone.

## [0.1.7] - 2026-11-17

### Added

- Polyring webring in the footer: `O << ?? >>` (hub, previous, random, next) in the terminal, the SSH session and the page. Member snapshot `content/polyring.txt`, refreshed by `just polyring`.
- `?plain` shows the static page in any browser. `just serve --plain` serves only the page locally (was `just plain` before release).

### Changed

- The footer packs its segments to the width and draws the divider marks from them.
- The top bar shortens in steps (`projects` > `proj` > `prj` > `p`, `(gallery)` > `(gal)` > `gal`) and keeps words as long as they fit. `(c)` for cv no longer sits next to `c` for contact.
- Touch: the page follows the finger row by row and glides on after release (was one jump at the end of a swipe).

### Fixed

- Light mode: the cell cursor was invisible. Its parent had the light-mode `filter`, which made it the backdrop root of the cursor's `backdrop-filter`.
- iOS Safari: the footer sat behind the tab bar. The canvas uses `100dvh`.

## [0.1.6] - 2026-11-17

### Fixed

- Firefox: after a link opened a new tab, scroll, hover and the cursor stopped until a reload. `window.open` ran while the app state was borrowed. Firefox fires events synchronously inside it, a listener panicked on the second borrow, and the borrow stayed stuck. The borrow now ends before `open`, and hover, leave and wheel skip an event while the state is busy.

### Changed

- `just release` takes no tag argument. It tags `v<Cargo.toml version>`.

## [0.1.5] - 2026-11-17

### Fixed

- Cell cursor and selection invert what is behind them in light mode too (`backdrop-filter: invert(1)` instead of `mix-blend-mode: difference`, which Safari does not blend with the page background).
- Site: selected text shows inverted theme colors.

## [0.1.4] - 2026-11-17

### Changed

- Favicon fills its square: no margin, 25pt glyphs in a 32pt box.

## [0.1.3] - 2026-11-16

### Fixed

- The two position bars of the unicode visual span the full width. They stopped at 100 columns.

### Changed

- Crash takes the speed keys of lichen: `-`/`+` for the speed, `0` for the default speed, space to pause. `r` restarts the sweep; `0` no longer does.

## [0.1.2] - 2026-11-16

### Fixed

- The browser app takes the whole viewport. The boot script skipped every second element when it removed the page, so `#plain` stayed, and the styles of the page put the canvas in a box.

### Added

- OSC 8 hyperlinks over SSH and in the native terminal: the top-bar destinations, the projects, the contacts and the key files.

## [0.1.1] - 2026-11-16

### Fixed

- Container entrypoint is `/usr/local/bin/static-web-server`. The alpine base image has no `/static-web-server`, so v0.1.0 did not start.

## [0.1.0] - 2026-11-12

The first release: one URL, three surfaces and one set of content.

### Added

- **One document at `/`.** Typst builds the page. A boot script replaces the
  page with the WebAssembly app when the browser can run it. The site has no
  `/tui`, no `/static` and no second hostname.
- **Terminal portfolio** (`src/frontends/native`). A Ratatui interface with
  about, projects and contact. `j`/`k`, the arrow keys, the wheel and a swipe
  scroll by one line. `h`/`l` and `1`–`3` select a section, and Enter opens the
  focused link.
- **Web app** (`src/frontends/web`). Ratzilla draws the same interface into a
  canvas. Keyboard, pointer, touch and text selection work.
- **SSH server** (`src/frontends/ssh`). `ssh 0xbf.ch` shows the same interface
  through `russh`. It needs a PTY. It runs the portfolio only: no shell, no
  commands, no SFTP and no forwarding.
- **Four hidden visuals.** Unicode specimens, a lichen colony, a quaternion
  Julia explorer and an EGA framebuffer sweep.
- **The page** (`ssg/`). A Typst site built with
  [thyme](https://github.com/bfeitknecht/thyme). One build writes `index.html`,
  `site.typ`, `contact/`, `404.html`, `sitemap.xml`, `robots.txt`, the public
  keys, the CV and the webfonts into the document root.
- **One set of content.** `content/` holds the profile, the eight projects and
  the keys. `src/content.rs` reads it at build time and `ssg/config.typ` does the
  same, so both surfaces show the same text in the same order.
- **Machine-readable files.** `robots.txt` and `llms.txt` at the root. thyme
  copies both byte for byte. `ssg/llms.txt` is written by hand and follows
  [llmstxt.org](https://llmstxt.org).
- **`justfile`.** Every command: `serve`, `build`, `ssg`, `web`, `native`,
  `ssh`, `image`, `release`, `favicon`, `unicode`, `check` and `fmt`.
- **Container image.** One arm64 image with the document root and the SSH
  binary. `just release v0.1.0` tags the tree with the notes of this file, and
  the workflows build `:latest` and `:v0.1.0` and publish the GitHub release.
- **Documentation.** `docs/architecture.md`, `docs/navigation.md`,
  `docs/ssg.md` and `docs/go-live.md`.
