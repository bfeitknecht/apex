# syntax=docker/dockerfile:1
#
# One image, one document root, two entrypoints:
#
#   static-web-server   /            the page (Typst) with the app's assets;
#                                    the page boots the app in capable browsers
#                       /contact/    a content page
#                       /age.txt /pgp.txt /cv.pdf
#   apex-ssh            the same portfolio over SSH
#
# Both run from this image, so one tag rolls the whole site.

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
# Assets only: the document visitors get is built by the `ssg` stage below.
RUN trunk build --release --dist /dist --public-url /

# ---------- the page: Typst + thyme ----------
FROM --platform=$BUILDPLATFORM ghcr.io/typst/typst:0.15.1 AS ssg

WORKDIR /work
COPY content ./content
COPY ssg ./ssg

# Typst resolves @preview packages from a package path; the vendored copies are
# pinned snapshots of thyme 0.1.0 and exemel 0.3.0.
RUN set -eu; \
    mkdir -p /packages/preview/exemel /packages/preview/thyme; \
    cp -R ssg/vendor/exemel /packages/preview/exemel/0.3.0; \
    cp -R ssg/vendor/thyme /packages/preview/thyme/0.1.0; \
    typst compile --features html,bundle --format bundle \
        --package-path /packages --root . ssg/site.typ /public

# ---------- the SSH frontend: static musl binary ----------
# Target platform, not build platform: this binary runs on the node and the
# nodes are arm64, so building it natively here would produce the wrong arch.
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

# `ssh` would shadow the OpenSSH client in a debugging shell, so the binary is
# installed under the site's name.
COPY --from=ssh /work/target/release/ssh /usr/local/bin/apex-ssh
# The app's assets first, then the page over them. Trunk renames its entry
# document to `index.html`, so copying `dist` last would replace the page with
# Trunk's stub; this order makes the page the document and leaves the app as
# `apex.js`, `apex_bg.wasm`, `app.css`, `boot.js`, `cursor.js` and the fonts the
# page already refers to by name.
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
