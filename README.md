# 0xbf.ch

Personal site — [0xbf.ch](https://0xbf.ch). One landing page, monochrome, no
client framework runtime.

## Stack

- **Deno** — server, type-checking, tooling
- **Preact + htm** — components authored with tagged templates (no TSX),
  rendered to static HTML on the server
- **Minimal client JS** — theme toggle + an fx layer (grain, lichen, clock), no framework runtime
- **Commit Mono** (self-hosted, OFL — license in `static/fonts/`) for mono
  accents, Helvetica Neue for text

## Develop

```sh
just dev        # → http://localhost:8080, live reload
just check      # type-check + lint + format check
```

## Deploy

The cluster (Talos/Kubernetes, Keel) pulls `ghcr.io/bfeitknecht/apex:latest` and
re-rolls the pod when the digest changes. This repo's CI
(`.github/workflows/publish-image.yaml`) builds a multi-arch image and pushes it
to GHCR on every `v*` tag:

```sh
git tag v1.0.0 && git push --tags
```

Nothing else to do — the infra tree is never touched for site updates.
(QEMU-emulated `deno compile` for arm64 makes CI take a few minutes; if the repo
is public, switching the runner to `ubuntu-24.04-arm` and building `linux/arm64`
only is faster.)

## Layout

```
main.ts                  server bootstrap (port, Deno.serve)
src/router.ts            route table: pathname → Response
src/fonts.ts             font files from disk, cache headers
src/html.ts              the `html` tagged template (htm bound to preact's h)
src/theme.ts             inline pre-paint + toggle scripts
src/fx.js                client fx layer: grain, lichen colony, clock, cursor
                         (plain JS, no build step, respects prefers-reduced-motion)
src/components/          document, header, hero, section, interests, projects,
                         footer — each carries its own content, edit in place
src/pages/               home, not-found — page composition
src/styles.css           design tokens (colors, fonts, fx) at the top
src/favicon.svg          embedded, theme-aware via prefers-color-scheme
src/age.txt              age recipient — replace the placeholder
src/pgp.txt              PGP public key — replace the placeholder
src/robots.txt           embedded
static/fonts/            Commit Mono woff2 (served from disk, cached)
Dockerfile               deno compile → FROM scratch (binary + fonts + glibc only)
```

Imports use the `@/` mapping (`deno.json`) which resolves to `src/` — e.g. `import { html } from "@/html.ts"`.

Text assets are imported as Deno text modules, so they are embedded in the
compiled binary — the production image contains only the binary, the font files,
tini, and the glibc libraries the binary links against. No shell, no package
manager, runs as uid 1000 with a read-only root filesystem.

## Theme

Default follows the system preference (`prefers-color-scheme`); the toggle
stores an explicit `light`/`dark` override in `localStorage`. A pre-paint inline
script applies the stored value, so there is no flash of the wrong theme.
Without JS, the system preference still applies.
