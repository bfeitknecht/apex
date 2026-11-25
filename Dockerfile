# syntax=docker/dockerfile:1
#
# One image, one document root and two entrypoints:
#
#   static-web-server   /            the static version (Typst) with the assets
#                                    of the dynamic version (browser frontend)
#                       /contact/    a content page
#                       /age.txt /pgp.txt /cv.pdf
#   apex-ssh            the dynamic version over SSH
#
# Both entrypoints use this image, so one tag updates the whole site.

# ---------- dynamic, browser frontend: wasm32 + Trunk ----------
FROM --platform=$BUILDPLATFORM rust:1-bookworm AS dynamic

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
# Assets of the dynamic version only. The `static` stage below builds the
# document that visitors get.
RUN trunk build --release --dist /dist --public-url /

# ---------- static: Typst (bundle export, no package) ----------
FROM --platform=$BUILDPLATFORM ghcr.io/typst/typst:0.15.1 AS static

WORKDIR /work
COPY content ./content
COPY static ./static

RUN typst compile --features html,bundle --format bundle --root . static/site.typ /public

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
# The assets of the dynamic version come first, and the static version goes over
# them. Trunk renames its entry document to `index.html`, so a later copy of
# `dist` would replace the static version with the stub of Trunk. This order
# makes the static version the document and leaves the dynamic version as
# `apex.js`, `apex_bg.wasm`, `dynamic.css`, `boot.js`, `cursor.js` and the fonts
# that it refers to by name.
COPY --from=dynamic /dist /public
COPY --from=static /public /public
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
# The alpine variant has the binary in PATH. /static-web-server exists only in the scratch variant.
ENTRYPOINT ["/usr/local/bin/static-web-server"]
