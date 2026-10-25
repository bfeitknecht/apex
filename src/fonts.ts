/**
 * Font files live on disk next to the binary (static/fonts) and are streamed
 * with long-lived cache headers. Everything else the site serves is embedded
 * in the binary as a text module.
 */

const FONTS = new Map<string, string>([
  ["/fonts/commit-mono-400.woff2", "font/woff2"],
  ["/fonts/commit-mono-700.woff2", "font/woff2"],
]);

export async function fontResponse(pathname: string): Promise<Response | null> {
  const contentType = FONTS.get(pathname);
  if (!contentType) return null;

  try {
    const file = await Deno.open(`${Deno.cwd()}/static${pathname}`);
    return new Response(file.readable, {
      headers: {
        "content-type": contentType,
        "cache-control": "public, max-age=86400",
      },
    });
  } catch {
    return null;
  }
}
