import { html } from "@/html.ts";
import { Document } from "@/components/document.ts";
import { Header } from "@/components/header.ts";
import { Hero } from "@/components/hero.ts";
import { Section } from "@/components/section.ts";
import { Interests } from "@/components/interests.ts";
import { Projects } from "@/components/projects.ts";
import { Footer } from "@/components/footer.ts";

export function Home() {
  return html`
    <${Document} title="0xbf.ch — software, systems & mathematics">
      <div class="sheet">
        <canvas id="fx-lichen" class="fx fx-lichen" aria-hidden="true"></canvas>
        <a class="skip" href="#main">Skip to content</a>
        <${Header} />
        <main id="main" class="grid">
          <${Hero} />
          <${Section} index="01" label="Interests">
            <${Interests} />
            <//>
            <${Section} index="02" label="Project">
              <${Projects} />
              <//>
              <${Section} index="03" label="Beyond code">
                <p class="beyond">
              Away from the keyboard: chess, volleyball, and snowboarding.
            </p>
                <//>
                </main>
                <${Footer} />
                </div>
                <//>
  `;
}
