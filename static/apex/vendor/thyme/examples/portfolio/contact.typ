#import "@preview/thyme:0.1.0": *

#show: webpage.with(
  title: [Contact],
  description: "Direct email and OpenPGP fingerprint.",
  modified: datetime(year: 2026, month: 10, day: 28),
)

= Contact

#tr(
  en: [Reach out directly by electronic mail or verify signatures against `/pgp.txt`.],
  de: [Direkter Kontakt per E-Mail oder Signaturprüfung über `/pgp.txt`.],
)

#h("ul",
  h("li", [Email: ], link("mailto:ada@example.org")[`ada@example.org`]),
  h("li", [OpenPGP: ], link("../pgp.txt")[`pgp.txt`]),
)
