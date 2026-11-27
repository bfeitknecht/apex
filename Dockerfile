# syntax=docker/dockerfile:1
#
# One image, one document root and two entrypoints:
#
#   static-web-server   /            the static version (Typst) with the assets
#                                    of the dynamic version (browser frontend)
#                       /age.txt /pgp.txt /cv.pdf
#   apex-ssh            the dynamic version over SSH
#
# Both entrypoints use this image, so one tag updates the whole site.

# ---------- dynamic, browser frontend: wasm32 + Trunk ----------
FROM --platform=$BUILDPLATFORM rust:1-bookworm AS dynamic

# Trunk downloads the matching wasm-bindgen CLI on the first build. Trunk
# itself is the release binary (pinned, checked): `cargo install` compiled it
# on every image build.
# A bump also needs the two TRUNK_SHA256 values below.
# renovate: datasource=github-releases depName=trunk-rs/trunk extractVersion=^v(?<version>.*)$
ARG TRUNK_VERSION=0.22.2
ARG BUILDARCH
RUN set -eu \
 && case "${BUILDARCH}" in \
      amd64) TRUNK_ARCH=x86_64 TRUNK_SHA256=26fca8a9952e46cd7d8b8b9a2e2c1786c090fc14b8e10b6678ab4d0f9f1a385f ;; \
      arm64) TRUNK_ARCH=aarch64 TRUNK_SHA256=be7b50f2d24e9cfbaea5551be2abf3fbabac10412113213a8efe94448e514414 ;; \
      *) echo "no trunk for ${BUILDARCH}" >&2; exit 1 ;; \
    esac \
 && curl -fsSL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-${TRUNK_ARCH}-unknown-linux-gnu.tar.gz" -o /tmp/trunk.tar.gz \
 && echo "${TRUNK_SHA256}  /tmp/trunk.tar.gz" | sha256sum -c - \
 && tar -xzf /tmp/trunk.tar.gz -C /usr/local/bin trunk \
 && rm /tmp/trunk.tar.gz \
 && rustup target add wasm32-unknown-unknown

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
