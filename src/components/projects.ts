import { html } from "@/html.ts";

type Project = {
  name: string;
  description: string;
  url: string;
};

const PROJECTS: Project[] = [
  {
    name: "impli",
    description:
      "Interpreter and REPL for the IMP toy language from ETH's Formal Methods and Functional Programming course — natural semantics, written in Haskell.",
    url: "https://github.com/bfeitknecht/impli",
  },
];

function ProjectCard({ project }: { project: Project }) {
  return html`
    <li>
      <a class="project-link" href=${project.url} rel="noopener noreferrer">
        <span class="project-name">${project.name}</span>
        <span class="project-desc">${project.description}</span>
        <span class="project-cta">GitHub ↗</span>
      </a>
    </li>
  `;
}

export function Projects() {
  return html`
    <ul class="projects">
      ${PROJECTS.map((project) =>
        html`<${ProjectCard} key=${project.name} project=${project} />`
      )}
    </ul>
  `;
}
