#!/usr/bin/env python3
"""Dev server for src/.

Serves the web root straight from disk and answers unknown paths with
noscript/404.html and a 404 status, the way the production image does.
Plain `python3 -m http.server` would return its own error page instead.
"""

import functools
import http.server
import pathlib

ROOT = pathlib.Path(__file__).parent / "src"
PORT = 8080
NOT_FOUND = ROOT / "noscript" / "404.html"


class Handler(http.server.SimpleHTTPRequestHandler):
    def send_error(self, code, message=None, explain=None):
        if code == 404 and NOT_FOUND.is_file():
            body = NOT_FOUND.read_bytes()
            self.send_response(404)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            if self.command != "HEAD":
                self.wfile.write(body)
            return
        super().send_error(code, message, explain)


if __name__ == "__main__":
    handler = functools.partial(Handler, directory=str(ROOT))
    with http.server.ThreadingHTTPServer(("", PORT), handler) as httpd:
        print(f"serving {ROOT} on http://localhost:{PORT}")
        httpd.serve_forever()
