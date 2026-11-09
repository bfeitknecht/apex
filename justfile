# Apex: one URL, two surfaces, one definition of the content.
#
#   just dev      build both, rebuild on change, serve on :8080
#   just native   the terminal
#   just ssh      the SSH server (russh)
#   just app      the browser app (Trunk/Ratzilla)
#   just ssg      the page (Typst/thyme)
#
# Run `just` to list the recipes.

set shell := ["sh", "-cu"]

# Keep build artifacts in the repo (gitignored) unless the caller says otherwise.
export CARGO_TARGET_DIR := env("CARGO_TARGET_DIR", justfile_directory() / "target")

# Local Typst package cache resolving the vendored @preview packages.
packages_dir := justfile_directory() / ".packages"

default:
    @just --list

# Format the whole tree.
fmt:
    cargo fmt --all

# Lints, tests and the wasm32 build (what CI runs).
check:
    cargo clippy --all-targets --features ssh
    cargo test --features ssh
    cargo check --target wasm32-unknown-unknown

# Native terminal portfolio.
native *args:
    cargo run --release -- {{args}}

# SSH server on 0.0.0.0:2222; needs APEX_SSH_HOST_KEY (see README.md).
ssh:
    cargo run --release --features ssh --bin ssh

# Link the vendored Typst packages into a throwaway package path.
# (thyme's blog preset also needs cheq/showybox/tablem/zebraw/marge from
# Typst Universe; the first build fetches and caches them.)
packages:
    #!/usr/bin/env sh
    set -eu
    rm -rf "{{packages_dir}}"
    mkdir -p "{{packages_dir}}/preview/exemel" "{{packages_dir}}/preview/thyme"
    cp -R ssg/vendor/exemel "{{packages_dir}}/preview/exemel/0.3.0"
    cp -R ssg/vendor/thyme "{{packages_dir}}/preview/thyme/0.1.0"

# The page and its plain artifacts into ./public (this is the document root).
ssg: packages
    #!/usr/bin/env sh
    set -eu
    mkdir -p public
    typst compile --features html,bundle --format bundle \
        --package-path "{{packages_dir}}" --root . ssg/site.typ public

# The browser app into ./dist (assets only; the page is what serves it).
app:
    trunk build --dist dist --public-url /

# The browser app, production build.
app-release:
    trunk build --release --dist dist --public-url /

# Assemble the document root exactly as the container does.
docroot: ssg app-release
    #!/usr/bin/env sh
    set -eu
    # Trunk renames its entry document to `index.html`; dropping it from `dist`
    # first is what keeps the page the document, and the app a set of assets.
    rm -f dist/index.html dist/app.html
    cp -R dist/. public/

# Serve ./public the way the container does: the same `sws` binary, the same
# `sws.toml`, so `/cv` rewrites, the 404 page and the headers are the ones that
# ship. Rebuild in another shell (`just ssg`, `just app-release`) and reload.
serve:
    docker run --rm -p 8080:8080 \
        -v "{{justfile_directory()}}/public:/public" \
        -v "{{justfile_directory()}}/sws.toml:/sws.toml:ro" \
        -e SERVER_HOST=0.0.0.0 -e SERVER_PORT=8080 -e SERVER_ROOT=/public \
        -e SERVER_ERROR_PAGE_404=./404.html -e SERVER_CONFIG_FILE=/sws.toml \
        -e SERVER_CACHE_CONTROL_HEADERS=false -e SERVER_LOG_LEVEL=info \
        joseluisq/static-web-server:2.44.1-alpine

# Build the document root, then serve it on :8080.
dev: docroot serve

# Container image: one document root plus the SSH binary.
image:
    docker build -t ghcr.io/bfeitknecht/apex:dev .

image-run:
    docker run --rm -p 8080:8080 -p 2222:2222 \
        -e APEX_SSH_HOST_KEY=/run/secrets/apex_ssh_host_key \
        -v "${PWD}/.private/ssh_host_ed25519_key:/run/secrets/apex_ssh_host_key:ro" \
        ghcr.io/bfeitknecht/apex:dev

# Tag a release; the tag drives the image publish workflow.
release tag message:
    #!/usr/bin/env sh
    set -eu
    tag="{{tag}}"
    git check-ref-format "refs/tags/$tag"
    ! git rev-parse -q --verify "refs/tags/$tag"
    git tag -a "$tag" -m "{{message}}"
    git push origin "$tag"
    echo "pushed $tag"
