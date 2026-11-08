#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/apex-target}"
exec trunk serve --address 0.0.0.0 --port "${PORT:-8080}" --dist "${TRUNK_DIST:-/tmp/apex-dist}"
