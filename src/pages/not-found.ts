import { html } from "@/html.ts";
import { Document } from "@/components/document.ts";
import { Header } from "@/components/header.ts";
import { Footer } from "@/components/footer.ts";

export function NotFound() {
  return html`
    <${Document} title="404 — 0xbf.ch">
      <div class="sheet">
        <canvas id="fx-lichen" class="fx fx-lichen" aria-hidden="true"></canvas>
        <a class="skip" href="#main">Skip to content</a>
        <${Header} />
        <main id="main" class="grid">
          <div class="row">
            <div class="rail">
              <span class="rail-index">404</span>
              <span class="rail-label">Error</span>
            </div>
            <div class="cell hero">
              <h1>Nothing here<span class="cursor">.</span></h1>
              <p class="lede"><a href="/">Back to the start.</a></p>
            </div>
          </div>
        </main>
        <${Footer} />
      </div>
      <//>
  `;
}
