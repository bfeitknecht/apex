# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- **`justfile`.** Every command: `just dev`, `native`, `ssh`, `check`, `app`,
  `ssg`, `docroot`, `serve`, `image`, `image-run`, `cmaps` and `release`.
- **Container image.** One arm64 image with the document root and the SSH
  binary. `just release 0.1.0` tags the tree, and the publish workflow builds
  `:latest` and `:v0.1.0`.
- **Documentation.** `docs/architecture.md`, `docs/navigation.md`,
  `docs/ssg.md` and `docs/go-live.md`.
