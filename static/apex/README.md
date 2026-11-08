# 0xbf.ch

Personal site at [0xbf.ch](https://0xbf.ch), built as a static HTML + CSS bundle with `@preview/thyme:0.1.0` in pure Typst (`>= 0.15.1`), set in Helvetica Neue and Commit Mono with neutral monochrome `oklch` tokens and zero client-side JavaScript.

## Layout

```text
site.typ       homepage (About, Projects, Contact), 404 page, sitemap, and static assets
contact.typ    /contact/ share page with mail, socials, and public keys
styles.css     monochrome Helvetica Neue + Commit Mono stylesheet
assets/        favicon.svg and self-hosted Commit Mono .woff2 files
age.txt        /age.txt plaintext age recipient
pgp.txt        /pgp.txt plaintext PGP public key
cv.pdf         /cv PDF served via sws.toml rewrite
sws.toml       static-web-server headers (/cv rewrite, inline .typ, immutable fonts)
Dockerfile     Typst bundle build into a scratch static-web-server image
justfile       compile, serve, dev, build, preview
```

## Build & Preview

```sh
just compile   # emits static web root into public/
just dev       # compiles and serves public/ on http://localhost:8080
```
