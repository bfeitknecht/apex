#import "@preview/thyme:0.1.0": *

#show: webpage.with(
  title: [Contact],
  description: "Email, socials, and public keys of Basil Feitknecht.",
  home-text: "/..",
  wrap-main: false,
)

#htm(
  ```html
  <header>
    <p class="eyebrow"><a href="/">Basil Feitknecht</a></p>
    <h1>Contact</h1>
  </header>

  <main>
    <p>
      Reach me by mail at <a href="mailto:bfeitknecht@ethz.ch">bfeitknecht@ethz.ch</a>
      or <a href="mailto:io+www@0xbf.ch">io@0xbf.ch</a>.
      I am also on <a href="https://github.com/bfeitknecht">github</a> and
      <a href="https://linkedin.com/in/bfeitknecht">linkedin</a>, and my CV is at
      <a href="/cv">/cv</a>.
    </p>

    <section id="keys">
      <h2><span class="num">01</span>Keys</h2>
      <p>
        Encrypt messages with my plaintext <a href="/age.txt">age recipient</a>
        or <a href="/pgp.txt">PGP public key</a>.
      </p>
      <p class="where">age170rmcek6zuhx4gd837elruddvhxha56ekqlkfu2ty6lpagxhsdrq6ra8ju</p>
    </section>
  </main>

  <footer>
    <p class="colophon">Made with &lt;3 by me. No JavaScript.</p>
  </footer>
  ```,
)
