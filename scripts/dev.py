#!/usr/bin/env python3
"""Build both surfaces, build them again when a source changes, and serve them.

The site is not one toolchain: Typst produces the page (and the plain
artifacts) and Trunk produces the app that the page boots. This script keeps
both in one document root and serves that root the way the container does —
same rewrite for `/cv`, same `.typ` content type, same 404 page — so what you
look at in development is what ships.

    python3 scripts/dev.py            # http://localhost:8080
    python3 scripts/dev.py --port 9000
"""

from __future__ import annotations

import argparse
import functools
import http.server
import pathlib
import shutil
import subprocess
import sys
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
PUBLIC = ROOT / "public"
DIST = ROOT / "dist"
PACKAGES = ROOT / ".packages"

# Directories and files whose mtime means "rebuild".
WATCHED = ("ssg", "content", "src", "browser", "assets", "Cargo.toml", "Cargo.lock", "Trunk.toml")


def run(*command: str) -> None:
    print(f"$ {' '.join(command)}", flush=True)
    subprocess.run(command, cwd=ROOT, check=True)


def link_packages() -> None:
    """Lay the vendored Typst packages out where `--package-path` looks."""
    shutil.rmtree(PACKAGES, ignore_errors=True)
    for name, version in (("exemel", "0.3.0"), ("thyme", "0.1.0")):
        source = ROOT / "ssg" / "vendor" / name
        target = PACKAGES / "preview" / name / version
        shutil.copytree(source, target)


def build_site() -> None:
    PUBLIC.mkdir(exist_ok=True)
    run(
        "typst",
        "compile",
        "--features",
        "html,bundle",
        "--format",
        "bundle",
        "--package-path",
        str(PACKAGES),
        "--root",
        ".",
        "ssg/site.typ",
        "public",
    )


def build_app() -> None:
    run("trunk", "build", "--dist", "dist", "--public-url", "/")
    if not DIST.is_dir():
        raise SystemExit("trunk produced no dist/ directory")
    shutil.copytree(DIST, PUBLIC, dirs_exist_ok=True)
    # Trunk also emits its own entry document; Typst's page is the document.
    (PUBLIC / "app.html").unlink(missing_ok=True)


def snapshot() -> dict[str, float]:
    """Newest mtime per watched path, so one comparison covers the tree."""
    seen: dict[str, float] = {}
    for name in WATCHED:
        path = ROOT / name
        if not path.exists():
            continue
        if path.is_file():
            seen[name] = path.stat().st_mtime
            continue
        newest = 0.0
        for child in path.rglob("*"):
            if child.is_file() and "vendor" not in child.parts:
                try:
                    newest = max(newest, child.stat().st_mtime)
                except OSError:
                    pass
        seen[name] = newest
    return seen


class Handler(http.server.SimpleHTTPRequestHandler):
    """`python -m http.server`, told how the container serves this tree."""

    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".typ": "text/plain; charset=utf-8",
        ".wasm": "application/wasm",
        ".woff2": "font/woff2",
        ".otf": "font/otf",
        ".ttf": "font/ttf",
    }

    def end_headers(self) -> None:  # noqa: D102 - stdlib hook
        # The container revalidates through `sws.toml` (ETag, no-cache), but a
        # preview pane will happily keep showing a page from before the build.
        # Nothing here is worth caching, so nothing here is cached.
        self.send_header("Cache-Control", "no-store, must-revalidate")
        self.send_header("Pragma", "no-cache")
        super().end_headers()

    def translate_path(self, path: str) -> str:
        if path.split("?", 1)[0] == "/cv":
            return str(PUBLIC / "cv.pdf")
        return super().translate_path(path)

    def send_error(self, code, message=None, explain=None):  # noqa: D102 - stdlib hook
        not_found = PUBLIC / "404.html"
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


def serve(port: int) -> http.server.ThreadingHTTPServer:
    handler = functools.partial(Handler, directory=str(PUBLIC))
    server = http.server.ThreadingHTTPServer(("0.0.0.0", port), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--no-watch", action="store_true", help="build once and serve")
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="serve what is already in ./public instead of building it",
    )
    options = parser.parse_args()
    port = options.port

    if not options.no_build:
        link_packages()
        build_site()
        build_app()
    state = snapshot()

    server = serve(port)
    print(f"\nserving {PUBLIC} on http://localhost:{port}\n", flush=True)
    if options.no_watch or options.no_build:
        try:
            while True:
                time.sleep(3600)
        except KeyboardInterrupt:
            print("\nstopped")
        finally:
            server.shutdown()
        return 0

    site_changed = True
    app_changed = True
    try:
        while True:
            time.sleep(0.8)
            current = snapshot()
            changed = {name for name, mtime in current.items() if state.get(name) != mtime}
            if not changed:
                continue
            state = current
            site_changed |= bool(changed & {"ssg", "content"})
            app_changed |= bool(changed & {"src", "browser", "assets", "Cargo.toml", "Cargo.lock", "Trunk.toml"})
            try:
                if site_changed:
                    build_site()
                    site_changed = False
                if app_changed:
                    build_app()
                    app_changed = False
                print("rebuilt\n", flush=True)
            except subprocess.CalledProcessError:
                print("build failed; waiting for the next change\n", flush=True)
    except KeyboardInterrupt:
        print("\nstopped")
    finally:
        server.shutdown()
    return 0


if __name__ == "__main__":
    sys.exit(main())
