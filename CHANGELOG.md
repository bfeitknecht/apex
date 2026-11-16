# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
