/**
 * 0xbf.ch — personal site.
 *
 * Deno + Preact (via htm), server-rendered HTML with no client runtime —
 * the only scripts are the theme toggle and the fx layer (grain, lichen,
 * clock). Text assets (CSS, favicon, robots, keys) are imported as text
 * modules so `deno compile` embeds them; the image is built FROM scratch.
 */
import { handle } from "@/router.ts";

const port = Number(Deno.env.get("PORT") ?? 8080);

Deno.serve({ port, hostname: "0.0.0.0" }, handle);
