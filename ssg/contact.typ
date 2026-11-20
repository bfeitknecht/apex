#import "@preview/thyme:0.1.3": *
#import "config.typ": base, profile, destination, contact, mail, age, footer

#show: webpage.with(
  title: [Contact],
  description: "Email, socials, and public keys of " + profile.name + ".",
  wrap-main: false,
  // The default backlink text is `/..`, which looks like a broken path.
  home-text: "0xbf.ch",
  // `webpage` writes a document of its own, so `stylesheets:` and
  // `favicon:` of `site` never reached this page and the page had no style.
  head: {
    html.link(rel: "stylesheet", href: base + "/styles.css")
    html.link(rel: "icon", href: base + "/assets/favicon.svg", type: "image/svg+xml")
  },
)

#html.header({
  html.p(class: "eyebrow", html.a(href: base + "/", profile.name))
  html.h1("Contact")
})

#html.main({
  html.p[
    Reach me by mail at
    #html.a(href: "mailto:bfeitknecht@ethz.ch", "bfeitknecht@ethz.ch")
    or #mail;.
    I am also on #html.a(href: contact("github"), "github") and
    #html.a(href: contact("linkedin"), "linkedin"), and my CV is at
    #html.a(href: destination("cv"), "/cv").
  ]
  html.section(id: "keys", {
    html.h2({
      html.span(class: "num", "01")
      "Keys"
    })
    html.p[
      Encrypt messages with my plaintext
      #html.a(href: "/age.txt", "age recipient") or
      #html.a(href: "/pgp.txt", "PGP public key").
    ]
    html.p(class: "where", age)
  })
})

#footer
