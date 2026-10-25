import { html } from "@/html.ts";

type Group = {
  label: string;
  items: string[];
};

const GROUPS: Group[] = [
  {
    label: "Languages",
    items: ["Rust", "Zig", "C", "Go", "Haskell", "Gleam", "Lua", "WebAssembly"],
  },
  { label: "Shell", items: ["Nushell"] },
  {
    label: "Infrastructure",
    items: ["Talos Linux", "Kubernetes", "Docker", "Flux"],
  },
  {
    label: "Mathematics",
    items: [
      "Category theory",
      "Information security",
      "Discrete mathematics",
      "Linear algebra",
      "Typst",
    ],
  },
  { label: "Hardware", items: ["ARM", "RISC-V", "MIPS", "VLSI"] },
];

function Interest({ group }: { group: Group }) {
  return html`
    <div class="interest">
      <dt>${group.label}</dt>
      <dd>${group.items.map((item) => html`<span>${item}</span>`)}</dd>
    </div>
  `;
}

export function Interests() {
  return html`
    <dl class="interests">
      ${GROUPS.map((group) =>
        html`<${Interest} key=${group.label} group=${group} />`
      )}
    </dl>
  `;
}
