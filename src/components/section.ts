import { type ComponentChildren } from "preact";
import { html } from "@/html.ts";

type SectionProps = {
  index: string;
  label: string;
  children: ComponentChildren;
};

export function Section({ index, label, children }: SectionProps) {
  return html`
    <section class="row">
      <h2 class="rail">
        <span class="rail-index">${index}</span>
        <span class="rail-label">${label}</span>
      </h2>
      <div class="cell">${children}</div>
    </section>
  `;
}
