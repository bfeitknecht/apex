import { html } from "@/html.ts";

type FooterLinkProps = {
  label: string;
  url: string;
  external: boolean;
};

const LINKS: FooterLinkProps[] = [
  { label: "GitHub", url: "https://github.com/bfeitknecht", external: true },
  { label: "Blog", url: "https://blog.0xbf.ch", external: true },
  { label: "PGP", url: "/pgp.txt", external: false },
  { label: "age", url: "/age.txt", external: false },
];

function FooterLink({ label, url, external }: FooterLinkProps) {
  return external
    ? html`<a href=${url} target="_blank" rel="noopener noreferrer">${label}</a>`
    : html`<a href=${url}>${label}</a>`;
}

export function Footer() {
  return html`
    <footer class="bar">
      <span class="colophon">
        <span>© ${new Date().getFullYear()} 0xbf.ch — Zürich</span>
        <span id="fx-clock" aria-hidden="true"></span>
      </span>
      <nav class="links">
        ${LINKS.map((link) =>
          html`<${FooterLink} key=${link.label} ...${link} />`
        )}
      </nav>
    </footer>
  `;
}
