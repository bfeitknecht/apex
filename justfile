default:
    @just --list

# Serve src/ straight from disk → http://localhost:8080
dev:
    #!/usr/bin/env python3
    """Dev server for src/.

    Serves the web root straight from disk and answers unknown paths with
    noscript/404.html and a 404 status, the way the production image does.
    Plain `python3 -m http.server` would return its own error page instead.
    """
    import functools
    import http.server
    import pathlib

    root = pathlib.Path("src").resolve()
    if not root.is_dir():
        raise SystemExit(f"no src/ in {pathlib.Path.cwd()} — run just from the repo root")

    not_found = root / "404.html"

    class Handler(http.server.SimpleHTTPRequestHandler):
        def translate_path(self, path):
            # Mirror the image's rewrites (sws.toml). The image 301s retired
            # /noscript* paths to /; dev serves the target directly.
            p = path.split("?", 1)[0]
            if p == "/cv":
                return str(root / "cv.pdf")
            if p in ("/noscript", "/noscript/"):
                return str(root / "index.html")
            return super().translate_path(path)

        def send_error(self, code, message=None, explain=None):
            if code == 404 and not_found.is_file():
                body = not_found.read_bytes()
                self.send_response(404)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                if self.command != "HEAD":
                    self.wfile.write(body)
                return
            super().send_error(code, message, explain)

    handler = functools.partial(Handler, directory=str(root))
    with http.server.ThreadingHTTPServer(("", 8080), handler) as httpd:
        print(f"serving {root} on http://localhost:8080")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nstopped")

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

# Commit, tag and push. TAG is the git tag; MESSAGE is the commit message and
# the tag annotation. Either one is prompted for when empty. Everything is
# staged first, so a clean tree just tags HEAD.
release $tag="" $message="":
    #!/usr/bin/env sh
    set -eu

    tag="${tag:-}"
    message="${message:-}"

    if [ -z "$tag" ]; then
        printf 'release tag: '
        read -r tag || tag=""
    fi
    if [ -z "$message" ]; then
        printf 'release message: '
        read -r message || message=""
    fi

    if [ -z "$tag" ]; then
        echo "release: no tag given" >&2
        exit 1
    fi
    if [ -z "$message" ]; then
        echo "release: no message given" >&2
        exit 1
    fi
    if ! git check-ref-format "refs/tags/$tag" >/dev/null 2>&1; then
        echo "release: '$tag' is not a valid tag name" >&2
        exit 1
    fi
    if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
        echo "release: tag $tag already exists" >&2
        exit 1
    fi
    if ! git remote get-url origin >/dev/null 2>&1; then
        echo "release: no 'origin' remote configured" >&2
        exit 1
    fi

    git add -A
    if ! git diff --cached --quiet; then
        git commit -m "$message"
    elif git rev-parse -q --verify HEAD >/dev/null; then
        echo "release: nothing to commit, tagging HEAD"
    else
        echo "release: nothing to commit and no commits yet" >&2
        exit 1
    fi

    git tag -a "$tag" -m "$message"
    git push origin HEAD
    git push origin "$tag"
    echo "release: pushed $tag"
