import { type ComponentChildren } from "preact";
import { html } from "@/html.ts";
import { themeInit, themeToggle } from "@/theme.ts";
import fxScript from "@/fx.js" with { type: "text" };
import css from "@/styles.css" with { type: "text" };

const DESCRIPTION =
  "Computer science at ETH Zürich. Systems programming, infrastructure, and mathematics — Rust, Zig, Kubernetes, category theory.";

type DocumentProps = {
  title: string;
  description?: string;
  children: ComponentChildren;
};

export function Document(
  { title, description = DESCRIPTION, children }: DocumentProps,
) {
  return html`
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>${title}</title>
        <meta name="description" content=${description} />
        <meta name="theme-color" content="#f4f4f3" />
        <meta property="og:title" content=${title} />
        <meta property="og:description" content=${description} />
        <meta property="og:url" content="https://0xbf.ch" />
        <meta property="og:type" content="website" />
        <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
        <link rel="preload" href="/fonts/commit-mono-400.woff2" as="font"
          type="font/woff2" crossorigin />
        <link rel="preload" href="/fonts/commit-mono-700.woff2" as="font"
          type="font/woff2" crossorigin />
        <style dangerouslySetInnerHTML=${{ __html: css }} />
        <script dangerouslySetInnerHTML=${{ __html: themeInit }} />
      </head>
      <body>
        <canvas id="fx-grain" aria-hidden="true"></canvas>
        ${children}
        <script dangerouslySetInnerHTML=${{ __html: themeToggle }} />
        <script defer dangerouslySetInnerHTML=${{ __html: fxScript }} />
      </body>
    </html>
  `;
}
