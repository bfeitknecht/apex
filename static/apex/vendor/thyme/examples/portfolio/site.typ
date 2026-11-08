#import "@preview/thyme:0.1.0": *

#show: site.with(
  url: "https://example.org",
  title: "Ada Lovelace",
  description: "Analytical engine notes, formal systems, and symbolic computation.",
  author: "Ada Lovelace",
  stylesheets: (path("styles.css"),),
  src: path("site.typ"),
  wrap-main: false,
)

#raw-file("pgp.txt", path("pgp.txt"))
#mount(path("contact.typ")) <contact>

#htm(
  ```html
  <header class="hero">
    <p class="eyebrow">{role}</p>
    <h1>Ada Lovelace</h1>
    <p>{intro}</p>
    <nav class="meta-links" aria-label="Links">
      {contact_link}
      <a href="pgp.txt">pgp.txt</a>
      <a href="sitemap.xml">sitemap.xml</a>
    </nav>
  </header>
  <main>
    <section id="research">
      <h2>01 Research</h2>
      <p>{research}</p>
    </section>
  </main>
  ```,
  role: tr(
    en: [Symbolic Computation · London],
    de: [Symbolische Berechnung · London],
  ),
  intro: tr(
    en: [Designing mechanical notation and verifiable static documents without runtime dependencies.],
    de: [Entwurf mechanischer Notation und überprüfbarer statischer Dokumente ohne Laufzeitabhängigkeiten.],
  ),
  contact_link: link(<contact>)[contact],
  research: [ Bernoulli number recurrence schedules in $O(n^2)$ arithmetic operations. ],
)
