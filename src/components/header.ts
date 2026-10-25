import { html } from "@/html.ts";

function SunMoon() {
  return html`
    <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true">
      <circle cx="8" cy="8" r="6.25" fill="none" stroke="currentColor"
        stroke-width="1.5" />
      <path d="M8 1.75 A6.25 6.25 0 0 0 8 14.25 Z" fill="currentColor" />
    </svg>
  `;
}

export function Header() {
  return html`
    <header class="bar">
      <a class="wordmark" href="/">0xbf</a>
      <button
        id="theme-toggle"
        type="button"
        aria-label="Toggle color theme"
        title="Toggle color theme"
      >
        <${SunMoon} />
      </button>
    </header>
  `;
}
