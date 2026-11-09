#import "@preview/thyme:0.1.0": *
#import "config.typ": base, profile, destination, contact, mail, age, footer

#show: webpage.with(
  title: [Contact],
  description: "Email, socials, and public keys of " + profile.name + ".",
  wrap-main: false,
  // The default backlink text is `/..`, which reads like a broken path.
  home-text: "0xbf.ch",
  // `webpage` writes a whole document of its own, so `site`'s `stylesheets:` and
  // `favicon:` never reached this page: it was served unstyled.
  head: {
    html.link(rel: "stylesheet", href: base + "/styles.css")
    html.link(rel: "icon", href: base + "/assets/favicon.svg", type: "image/svg+xml")
  },
)

#htm(
  ```html
  <header>
    <p class="eyebrow"><a href="{base}/">{name}</a></p>
    <h1>Contact</h1>
  </header>

  <main>
    <p>
      Reach me by mail at <a href="mailto:bfeitknecht@ethz.ch">bfeitknecht@ethz.ch</a>
      or {mail}.
      I am also on <a href="{github}">github</a> and
      <a href="{linkedin}">linkedin</a>, and my CV is at
      <a href="{cv}">/cv</a>.
    </p>

    <section id="keys">
      <h2><span class="num">01</span>Keys</h2>
      <p>
        Encrypt messages with my plaintext <a href="/age.txt">age recipient</a>
        or <a href="/pgp.txt">PGP public key</a>.
      </p>
      <p class="where">{where}</p>
    </section>
  </main>{footer}
  ```,
  base: base,
  name: profile.name,
  github: contact("github"),
  linkedin: contact("linkedin"),
  cv: destination("cv"),
  mail: mail,
  where: age,
  footer: footer,
)
