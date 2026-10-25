/**
 * Route table: pathname → Response. Pages are server-rendered per request;
 * embedded text assets are served straight out of the binary.
 */
import type { VNode } from "preact";
import { render } from "preact-render-to-string";
import ageKey from "@/age.txt" with { type: "text" };
import favicon from "@/favicon.svg" with { type: "text" };
import pgpKey from "@/pgp.txt" with { type: "text" };
import robots from "@/robots.txt" with { type: "text" };
import { fontResponse } from "@/fonts.ts";
import { Home } from "@/pages/home.ts";
import { NotFound } from "@/pages/not-found.ts";

function page(node: VNode, status = 200): Response {
  return new Response("<!doctype html>" + render(node), {
    status,
    headers: { "content-type": "text/html; charset=utf-8" },
  });
}

function text(body: string, contentType: string, cache: string): Response {
  return new Response(body, {
    headers: {
      "content-type": `${contentType}; charset=utf-8`,
      "cache-control": cache,
    },
  });
}

const routes: Record<string, () => Response> = {
  "/": () => page(Home()),
  "/favicon.svg": () => text(favicon, "image/svg+xml", "public, max-age=86400"),
  "/robots.txt": () => text(robots, "text/plain", "public, max-age=3600"),
  "/age.txt": () => text(ageKey, "text/plain", "no-store"),
  "/pgp.txt": () => text(pgpKey, "text/plain", "no-store"),
};

export async function handle(request: Request): Promise<Response> {
  const { pathname } = new URL(request.url);

  const route = routes[pathname];
  if (route) return route();

  return (await fontResponse(pathname)) ?? page(NotFound(), 404);
}
