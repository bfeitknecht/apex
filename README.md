# 0xbf.ch

Personal site — [0xbf.ch](https://0xbf.ch). Static HTML + CSS, no build step,
no client framework.

## Layout

`src/` is the web root — the directory the image serves, one-to-one with URLs.

```
favicon.typ          typst source of assets/favicon.svg — kept at the root, not served
sws.toml             image-only server config: /cv rewrite, /noscript* 301s, no-cache
src/                 the web root — one-to-one with URLs
├── index.html       the whole site: about, projects, experience, education, contact
├── contact/         /contact/ — the share-URL: email, socials, keys
├── age.txt          /age.txt — age recipient, plaintext
├── pgp.txt          /pgp.txt — PGP public key, plaintext
├── styles.css       the whole stylesheet, handwritten
├── 404.html         served for every unknown path
├── cv.pdf           the CV, built from the cv repo (typst compile) — /cv serves it
├── robots.txt       must sit at the root — crawlers only read /robots.txt
└── assets/          favicon + fonts
```

Page URLs are directories with `index.html`; `/contact` 308s to `/contact/`
(SWS's trailing-slash redirect) and the page carries a canonical link.

The routes are few on purpose: /, /cv, /contact, plus the plaintext keys.
The future JS version answers the same URLs — the URL bar never reveals which
one is serving. Every route the JS app gets, the no-JS tree has too, so no
SPA fallback is ever needed. When the JS version lands (Rust/WASM or
htm-style), it ships as separate assets and hooks in through one line per
page — `<script type="module" src="/boot.js"></script>`, a loader that no-ops
until the app exists, then takes over the body and navigates client-side.
Nothing is embedded into anything: the Rust build never touches these HTML
files.

Refresh the CV after editing the cv repo:

```sh
typst compile ../cv/src/cv.typ src/cv.pdf
```

## Develop

```sh
just dev        # → http://localhost:8080, serves src/ from disk
```

`just dev` runs a small Python `http.server` (inline in the justfile) that
answers unknown paths with `404.html` and rewrites `/cv` to `cv.pdf`, so dev
matches the image's 404 and rewrite behaviour.

## Deploy

The cluster (Talos/Kubernetes, Keel) pulls `ghcr.io/bfeitknecht/apex:latest` and
re-rolls the pod when the digest changes:

```sh
just release v1.0.0 "feat: noscript site"   # commit, tag, push — CI does the rest
```

`just release` takes the tag and the commit message; the message also annotates
the tag. Either one is prompted for when left out. Everything is staged first,
so a clean tree just tags `HEAD`. The tag must start with `v` for CI to fire.

The build+push logic is not in this repo. `.github/workflows/publish-image.yaml`
is an 11-line caller for the reusable workflow in the infra repo
(`bfeitknecht/infra/.github/workflows/publish-image.yaml`), which derives the
image name from this repository and pushes a multi-arch build to GHCR. Fix the
build once there and every tier-2 app gets it.

Nothing else to do — the infra tree is never touched for site updates.

## Image

`Dockerfile` copies `src/` and `sws.toml` into an empty filesystem next to
[static-web-server](https://static-web-server.net) (Rust, ~4 MB, no runtime
dependencies). It listens on 8080, runs as uid 1000, and writes nothing, so the
Deployment's `readOnlyRootFilesystem` holds. Settings are the `SERVER_*` env
vars in the Dockerfile; `sws.toml` only carries the `/cv` rewrite, which has no
env equivalent.
