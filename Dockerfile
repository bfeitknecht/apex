# syntax=docker/dockerfile:1
#
# One image, one document root and two entrypoints:
#
#   static-web-server   /            the page (Typst) with the assets of the app
#                       /contact/    a content page
#                       /age.txt /pgp.txt /cv.pdf
#   apex-ssh            the same portfolio over SSH
#
# Both entrypoints use this image, so one tag updates the whole site.

# ---------- the app: wasm32 + Trunk ----------
FROM --platform=$BUILDPLATFORM rust:1-bookworm AS app

# Trunk downloads the matching wasm-bindgen CLI on the first build.
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/* \
 && rustup target add wasm32-unknown-unknown \
 && cargo install trunk --locked

WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY content ./content
COPY assets ./assets
COPY browser ./browser
COPY Trunk.toml ./
# Assets only. The `ssg` stage below builds the document that visitors get.
RUN trunk build --release --dist /dist --public-url /

# ---------- the page: Typst + thyme ----------
FROM --platform=$BUILDPLATFORM ghcr.io/typst/typst:0.15.1 AS ssg

WORKDIR /work
COPY content ./content
COPY ssg ./ssg

# thyme and the packages that it imports come from Typst Universe. This layer
# downloads them, so the build needs network access.
RUN typst compile --features html,bundle --format bundle --root . ssg/site.typ /public

# ---------- the SSH frontend: static musl binary ----------
# This is the target platform, not the build platform. The binary runs on the
# node, and the nodes are arm64. A native build here gives the wrong architecture.
FROM rust:1-alpine AS ssh

RUN apk add --no-cache build-base

WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY content ./content
COPY assets/unifont-cmap.bin assets/noto-cmap.bin ./assets/
RUN cargo build --release --features ssh --bin ssh

# ---------- runtime ----------
FROM joseluisq/static-web-server:2.44.1-alpine AS runtime

# A binary called `ssh` would hide the OpenSSH client in a debugging shell. The
# image therefore installs it as `apex-ssh`.
COPY --from=ssh /work/target/release/ssh /usr/local/bin/apex-ssh
# The assets of the app come first, and the page goes over them. Trunk renames
# its entry document to `index.html`, so a later copy of `dist` would replace the
# page with the stub of Trunk. This order makes the page the document and leaves
# the app as `apex.js`, `apex_bg.wasm`, `app.css`, `boot.js`, `cursor.js` and the
# fonts that the page already refers to by name.
COPY --from=app /dist /public
COPY --from=ssg /public /public
COPY sws.toml /sws.toml

ENV SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    SERVER_ROOT=/public \
    SERVER_ERROR_PAGE_404=./404.html \
    SERVER_CACHE_CONTROL_HEADERS=false \
    SERVER_CONFIG_FILE=/sws.toml \
    SERVER_LOG_LEVEL=info

USER 1000:1000
EXPOSE 8080 2222
ENTRYPOINT ["/static-web-server"]
