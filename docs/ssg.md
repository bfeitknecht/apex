# The page (`ssg/`)

`ssg/` is a [thyme](https://github.com/bfeitknecht/thyme) site: pure Typst,
zero client-side JavaScript, built to static HTML + CSS + the fonts the page
embeds. It is **the document at `https://0xbf.ch/`** — not a fallback behind a
URL. A browser that can run WebAssembly replaces it in place (see
`docs/architecture.md`); everything else reads it as it is.

## One document root

The container serves a single tree with a single URL space:

```text
public/
├── index.html                     the page (and the app's boot script)
├── site.typ                       its plaintext source (Tadi Web view)
├── contact/                       a content page + contact.typ
├── 404.html, sitemap.xml, robots.txt
├── styles.css, assets/fonts/*.woff2, assets/favicon.svg
├── age.txt, pgp.txt, cv.pdf       emitted by the same build, from content/
├── apex.js, apex_bg.wasm          the app (Trunk)
├── app.css, boot.js, cursor.js    the app's shell
└── *.ttf, *.otf, *-LICENSE.txt    the app's specimen fonts
```

Two toolchains write into that directory and nothing overlaps: `ssg/` builds
the document, `browser/` + `src/` build the app's assets. `just docroot` (and the Dockerfile) do
`typst … ssg/site.typ public` then copy `dist/` over it.

`sws.toml` describes how the tree is served: `/cv` rewrites to `/cv.pdf`,
`.typ` files are `text/plain`, fonts are immutable, everything else is
`no-cache` (the app's asset names are unhashed, so the shell must revalidate),
and unknown paths fall through to `404.html`.

## Paths belong to content

Every URL on the site is a piece of content. `/contact/` is a page; `/age.txt`,
`/pgp.txt`, `/cv` are artifacts. No path selects a frontend — that decision is
the browser's, made once at `/`.

`ssg/config.typ` states the prefix in one place:

```typst
#let base = ""   // the site is the document root
```

and pages write absolute hrefs as `{base}/…`, so a future move (say, into a
subdirectory of another host) is one edit plus the canonical URL.

## Shared content

`content/` is the single definition, read by both toolchains:

| File | Rust | Typst |
| --- | --- | --- |
| `content/profile.toml` | `include_str!` + `toml` → `content::profile()` | `toml("../content/profile.toml")` |
| `content/projects.toml` | same → `profile().projects` | `toml("../content/projects.toml").project` |
| `content/age.txt`, `pgp.txt`, `cv.pdf` | embedded / linked | emitted with `#file(…)` |
| `content/portrait.txt` | the `me` document | – |

`ssg/config.typ` is the Typst side of that contract: it parses the TOML and
exposes `destination("cv")`, `contact("github")`, `mail`, `age` and `footer`, so a page
never repeats a URL.

Projects render from data in `ssg/site.typ`:

```typst
#let _entries = for project in projects {
  html.elem("li", attrs: (class: "entry"), {
    html.elem("div", attrs: (class: "entry-head"), {
      html.h3(html.a(href: project.url, [#project.name]))
      html.elem("span", attrs: (class: "when"), [#project.period])
    })
    html.p([#project.description])
  })
}
```

The terminal prints `summary` on one line; the page prints `description` and
`period`. Same entry, two densities — typography, not a second copy.

## Build

```sh
just dev        # both toolchains, then serve ./public on :8080
just ssg        # ./public: page + artifacts
just app        # ./dist: app assets only
just docroot    # ./public: page, then dist over it
```

The entry point is `ssg/site.typ`, thyme's own convention: the file that calls
`#show: site.with(…, src: path("site.typ"), …)` is the root document, and thyme
emits that source view in the bundle under the entry file's own name, so the
plaintext view is `/site.typ`. Everything else in `ssg/` is either pulled in by it
(`config.typ`, `contact.typ`, `styles.css`, `assets/`) or vendored
(`vendor/`).

Typst (local and container use identical flags):

```sh
typst compile --features html,bundle --format bundle \
  --package-path .packages --root . ssg/site.typ public
```

`--root .` is what lets `ssg/config.typ` read `../content/…` and what lets
`#raw-file` emit the artifacts at the root. `just packages` lays the vendored
`@preview/thyme:0.1.0` and `@preview/exemel:0.3.0` out in the
`preview/<name>/<version>` layout Typst expects.

## Typst packages

`@preview/thyme:0.1.0` and `@preview/exemel:0.3.0` are vendored under
`ssg/vendor/` (provenance and refresh steps in `ssg/vendor/README.md`) and
resolved through `just packages` → `--package-path`. Two things follow from
that directory being a snapshot:

- The import name never changes: bumping thyme means replacing `src/` and
  updating the commit in `ssg/vendor/README.md`, not editing imports.
- thyme's blog preset imports `@preview/cheq`, `showybox`, `tablem`, `zebraw`
  and `marge`, and `lib.typ` imports the preset eagerly. Those five come from
  Typst Universe, so **the first build needs network** and caches them in
  `~/.cache/typst/packages`. CI and the image build have it; a fully offline
  build would have to vendor them too.

## Writing HTML in Typst

The page is built with `html.*` (Typst 0.15's built-in module) and `#htm(…)`
from thyme for the markup blocks. Three rules earn their place:

- **`html.elem` for anything case-sensitive.** The HTML parser lowercases tag
  names, so `<feComponentTransfer>` becomes `<fecomponenttransfer>` and an SVG
  filter silently stops working. The lichen tone filter is therefore built with
  `html.elem("feComponentTransfer", …)`, not markup.
- **No bare boolean attributes in `#htm`.** `hidden` and `defer` reach the
  typed constructors as `true` where a string is expected. Give them a value
  (`hidden="hidden"`) or use the constructor (`html.script(src: "/boot.js",
  defer: true)`), which is why the boot script is passed through `head:`.
- **`<script>` needs an empty body via the constructor.** Markup makes its
  content `content`, and `html.script` wants a string.

## Adding a page

```typst
// ssg/site.typ
#mount(path("notes.typ"))
```

`#mount` handles the route, the sitemap entry and the link. The 404 page is
`_not-found` in `site.typ`. Pages are static: the app lives at `/` and a content
page never boots it.
