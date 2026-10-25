default:
    @just --list

# Start the dev server with live reload → http://localhost:8080
dev:
    deno task dev

# Type-check, lint and verify formatting
check:
    deno task check
    deno task lint
    deno fmt --check

# Build the production image (same build as CI)
build:
    docker build -t ghcr.io/bfeitknecht/apex:dev .

# Run the production image locally
# OrbStack: published-port proxy is unreliable for scratch images — use the
# container DNS name it provides. Plain Docker/Linux: localhost:8080 works.
preview:
    docker rm -f apex-preview >/dev/null 2>&1 || true
    docker run -d --rm --name apex-preview -p 8080:8080 ghcr.io/bfeitknecht/apex:dev
    @echo "→ http://apex-preview.orb.local:8080 (OrbStack) or http://localhost:8080 (Linux)"
