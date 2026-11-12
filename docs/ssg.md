# The page (`ssg/`)

`ssg/` is a [thyme](https://github.com/bfeitknecht/thyme) site. It uses pure
Typst with no client-side JavaScript, and it builds to static HTML, CSS and the
fonts that the page embeds. It is the document at `https://0xbf.ch/`. It is not
a fallback behind a separate URL. A browser that can run WebAssembly replaces
the page in place (see `docs/architecture.md`). All other browsers read the page
as it is.

## One document root

The container serves one tree with one URL space:

```text
public/
├── index.html                     the page (and the boot script of the app)
├── site.typ                       its plaintext source (Tadi Web view)
├── contact/                       a content page + contact.typ
├── 404.html, sitemap.xml, robots.txt
├── styles.css, assets/fonts/*.woff2, assets/favicon.svg
├── age.txt, pgp.txt, cv.pdf       emitted by the same build, from content/
├── apex.js, apex_bg.wasm          the app (Trunk)
├── app.css, boot.js, cursor.js    the shell of the app
└── *.ttf, *.otf, *-LICENSE.txt    the specimen fonts of the app
```

Two toolchains write into this directory and their output does not overlap.
`ssg/` builds the document. `browser/` and `src/` build the assets of the app.
`just docroot` and the Dockerfile run `typst … ssg/site.typ public` and then
copy `dist/` over the result.

`sws.toml` describes how the server serves the tree. `/cv` rewrites to
`/cv.pdf`. `.typ` files are `text/plain`. Fonts are immutable. All other files
use `no-cache`, because the asset names of the app have no hash and the shell
must revalidate. Unknown paths fall through to `404.html`.

## Paths belong to content

Every URL on the site is a piece of content. `/contact/` is a page. `/age.txt`,
`/pgp.txt` and `/cv` are files. No path selects a frontend. The browser makes
that decision once, at `/`.

`ssg/config.typ` defines the prefix in one place:

```typst
#let base = ""   // the site is the document root
```

Pages write absolute hrefs as `{base}/…`. A move to another location, for
example a subdirectory on another host, needs one edit and the canonical URL.

## Shared content

`content/` is the only definition and both toolchains read it:

| File | Rust | Typst |
| --- | --- | --- |
| `content/profile.toml` | `include_str!` + `toml` → `content::profile()` | `toml("../content/profile.toml")` |
| `content/projects.toml` | same → `profile().projects` | `toml("../content/projects.toml").project` |
| `content/age.txt`, `pgp.txt`, `cv.pdf` | embedded / linked | emitted with `#file(…)` |
| `content/portrait.txt` | the `me` document | – |

`ssg/config.typ` is the Typst side of this contract. It parses the TOML and
exposes `destination("cv")`, `contact("github")`, `mail`, `age` and `footer`, so
no page repeats a URL.

The page renders projects from data in `ssg/site.typ`:

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

The terminal prints `summary` on one line. The page prints `description` and
`period`. Both show the same entry, and the difference is typography, not a
second copy.

## Build

```sh
just dev        # both toolchains, then serve ./public on :8080
just ssg        # ./public: page + artifacts
just app        # ./dist: app assets only
just docroot    # ./public: page, then dist over it
```

The entry point is `ssg/site.typ`, following the convention of thyme. The file
that calls `#show: site.with(…, src: path("site.typ"), …)` is the root document.
thyme writes the source view of that file into the bundle under the name of the
entry file, so the plaintext view is `/site.typ`. All other files in `ssg/` are
either imported by the entry file (`config.typ`, `contact.typ`) or read by it
(`styles.css`, `assets/`). `favicon.typ` is not part of the build: `just
favicon` renders it to `ssg/assets/favicon.svg`, which the page and the app
share.

Typst uses the same flags locally and in the container:

```sh
typst compile --features html,bundle --format bundle --root . ssg/site.typ public
```

`--root .` lets `ssg/config.typ` read `../content/…` and lets `#file` write the
artifacts at the root.

## Typst packages

`ssg/site.typ` imports `@preview/thyme:0.1.0`, and thyme imports
`@preview/syndicate` for the sitemap and `@preview/cheq`, `showybox`, `tablem`,
`zebraw` and `marge` for its blog preset. All six come from Typst Universe.
Typst downloads them on the first build and caches them in
`~/.cache/typst/packages`. CI and the image build have network access. A fully
offline build needs these packages in a local directory, and `--package-path`
must point at it.

A thyme update touches three places: `ssg/site.typ` and `ssg/contact.typ` carry
the import, and this directory names the version.

## Writing HTML in Typst

The page uses `html.*` (the built-in module of Typst 0.15) and `#htm(…)` from
thyme for the markup blocks. Three rules matter:

- **Use `html.elem` for case-sensitive names.** The HTML parser lowercases tag
  names, so `<feComponentTransfer>` becomes `<fecomponenttransfer>` and an SVG
  filter stops working without a message. The lichen tone filter therefore uses
  `html.elem("feComponentTransfer", …)`, not markup.
- **Do not use bare boolean attributes in `#htm`.** `hidden` and `defer` reach
  the typed constructors as `true` where the code expects a string. Give a value
  (`hidden="hidden"`) or use the constructor (`html.script(src: "/boot.js",
  defer: true)`). For this reason the boot script goes through `head:`.
- **Pass an empty body through the constructor for `<script>`.** Markup makes
  the content a `content` value, and `html.script` needs a string.

## Adding a page

```typst
// ssg/site.typ
#mount(path("notes.typ"))
```

`#mount` creates the route, the sitemap entry and the link. The 404 page is
`_not-found` in `site.typ`. Content pages are static: the app runs at `/` and a
content page does not start it.
