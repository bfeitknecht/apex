import { html } from "@/html.ts";

export function Hero() {
  return html`
    <div class="row">
      <div class="rail">
        <span class="rail-label">ETH Zürich — BSc Computer Science</span>
      </div>
      <div class="cell hero">
        <h1>Close to the metal, down to the math<span class="cursor">.</span></h1>
        <p class="lede">
          I like building things that are precise, fast, and honest about how
          they work — from Rust and Zig to Kubernetes clusters, with category
          theory and Typst along the way.
        </p>
      </div>
    </div>
  `;
}
