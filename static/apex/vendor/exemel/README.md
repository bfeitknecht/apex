# Exemel
Encodes Typst values to XML. This package also provides built-in support for generating Atom 1.0 feeds, RSS 2.0 feeds, and XML sitemaps.

## Encoding XML
You should structure your Typst values as follows:
```typ
#import "@preview/exemel:0.3.0": to-xml

// An XML node is either:
// - a dictionary describing an element, or
// - a plain string, which becomes a text node

#let value = (
  tag: "bookstore",
  // `namespace` is optional and defaults to `none`
  namespace: none,
  // `attrs` is optional and defaults to an empty dictionary
  attrs: (:),
  // `children` is optional and defaults to no children
  children: (
    (
      tag: "book",
      attrs: (id: "bk101", category: "programming"),
      children: (
        (tag: "title", attrs: (lang: "en"), children: ("XML Developer's Guide",)),
        (tag: "author", children: ("Gambardella, Matthew",)),
        (tag: "year", children: ("2000",)),
        (tag: "price", children: ("44.95",)),
      ),
    ),
  ),
)

#to-xml(value, pretty: true)
```

Notes on the shape:
- `tag` is required on every element; `attrs` and `children` may be omitted and default to empty.
- `attrs` is a dictionary of string keys to string values.
- `children` is an array whose entries are either nested element dictionaries or plain strings (text content). An element can't currently have both mixed markup *and* be namespace-prefixed. Typst's own `xml()` reader doesn't preserve namespace prefixes either, so round-tripping a parsed document through `xml()` and back through `to-xml` will lose any `foo:bar`-style prefixes.
- Passing `pretty: true` indents the output with two spaces per level. The default, `pretty: false`, produces compact output with no added whitespace.
- `to-xml` also accepts an array of top-level values, in which case it returns one XML string per value.

## Generating Atom Feed
```typ
#import "@preview/exemel:0.3.0": atom-encode, atom-post, atom-author, atom-site-id

#let me = atom-author(
  "Hachimi",
  email: "Namerudo@example.com",
  uri: "https://example.com/ashiga",
)

#let posts = (
  atom-post(
    "Hayaku",
    link: "https://example.com/blog/hayaku-naro",
    id: atom-site-id(
      authority: "example.com",
      time: datetime(year: 2026, month: 7, day: 1),
      identifier: "blog/typst-xml-plugin",
    ),
    updated: datetime(year: 2026, month: 7, day: 3, hour: 14, minute: 30, second: 0),
    published: datetime(year: 2026, month: 7, day: 1),
    summary: "Hachimi hachimi hachimi hachimi hachimi hachimi o namerudo",
    content: "Ashiga ashiga ashiga ashiga ashiga hayaku naro",
    authors: (me,),
    categories: ("umamusume", "clannad", "baka futari"),
  ),
)

#atom-encode(
  title: "Honey Drink",
  subtitle: "Hachimi, hachimi, and hachimi",
  site-link: "https://example.com/",
  id: atom-site-id(
    authority: "example.com",
    time: datetime(year: 2026, month: 1, day: 1),
  ),
  authors: (me,),
  posts: posts,
)
```

`atom-encode` builds the `<feed>` root, and accepts:
- `title` (required), `subtitle` (optional) — plain `str` or Typst `content`
- `site-link` (required): the human-readable site URL
- `feed-link`: the URL of the feed itself; defaults to `site-link + "/feed.xml"`
- `id` (required): a stable, unique feed identifier; `atom-site-id` is a helper for building spec-compliant `tag:` URIs (RFC 4151) from a domain you control
- `updated` (`datetime` or `auto`): when the feed was last modified; defaults to `auto` (inferred from the latest entry in `posts`), and date-only `datetime(year: ..., month: ..., day: ...)` values default their time component to `00:00:00` formatted with `tz` (default `"Z"`, via `format-rfc3339`)
- `authors`: an array built from `atom-author(...)` calls
- `posts`: an array built from `atom-post(...)` calls

`atom-post` builds a single `<entry>`, and accepts:
- the entry title (positional, required)
- `link` (required): URL of the human-readable post
- `id` (required): unique entry identifier, same convention as the feed `id`
- `updated` (`datetime` or `auto`, falling back to `published`), `published` (optional `datetime`)
- `summary`, `content`: both optional; `content-type` (default `"html"`) is written as the `content` element's `type` attribute
- `authors`: optional per-entry authors; if omitted, readers fall back to the feed-level authors per the Atom spec
- `categories`: an array of category strings/content, each rendered as `<category term="...">`

`atom-author` builds an `<author>` element from a name plus optional `email` and `uri`, and is used for both the feed-level and per-entry `authors` arrays.

## Generating RSS 2.0 Feed
`rss-encode` and `rss-post` mirror the `atom-encode` / `atom-post` conventions so sites can emit both Atom 1.0 and RSS 2.0 from the same post metadata:

```typ
#import "@preview/exemel:0.3.0": rss-encode, rss-post, rss-enclosure

#let items = (
  rss-post(
    "Hayaku",
    link: "https://example.com/blog/hayaku-naro",
    published: datetime(year: 2026, month: 7, day: 1),
    summary: "Hachimi hachimi hachimi hachimi hachimi hachimi o namerudo",
    categories: ("umamusume", "clannad", "baka futari"),
  ),
)

#rss-encode(
  title: "Honey Drink",
  subtitle: "Hachimi, hachimi, and hachimi",
  site-link: "https://example.com/",
  updated: datetime(year: 2026, month: 7, day: 3),
  posts: items,
)
```

`rss-encode` builds the `<rss version="2.0">` root and `<channel>`, and accepts:
- `title` (required), `subtitle` (optional `<description>`, defaults to `title`)
- `site-link` (required `<link>`)
- `feed-link`: URL for `<atom:link rel="self" type="application/rss+xml" .../>`; defaults to `site-link + "/rss.xml"` (pass `none` to omit)
- `updated`: `auto` (infers `<lastBuildDate>` from the latest item `<pubDate>`, omitting it if none), `datetime` formatted as RFC 2822 via `format-rfc2822` (with optional `tz`, default `"+0000"`), or `none` to omit `<lastBuildDate>`
- `language`, `copyright`, `generator`, `ttl`: optional channel metadata
- `posts`: an array built from `rss-post(...)` (alias `rss-item`) calls

`rss-post` (alias `rss-item`) builds a single `<item>`, and accepts:
- the item title (positional)
- `link`: URL of the post
- `id`: `<guid>` string; defaults to `auto` (reuses `link`), with `is-permalink: auto` setting `isPermaLink="true"` for `http(s)://` URLs and `"false"` otherwise
- `published`: optional `datetime` formatted as RFC 2822 `<pubDate>`
- `summary` (`<description>`), `content` (`<content:encoded>`, automatically declaring `xmlns:content`)
- `author`, `categories`, `comments`, and `enclosure` (built with `rss-enclosure(url, length: ..., type: ...)`)

## Generating XML Sitemaps
`sitemap-encode` and `sitemap-url` build spec-compliant `sitemap.xml` documents (`http://www.sitemaps.org/schemas/sitemap/0.9`):

```typ
#import "@preview/exemel:0.3.0": sitemap-encode, sitemap-url

#sitemap-encode(
  urls: (
    sitemap-url("https://example.com/", lastmod: datetime(year: 2026, month: 7, day: 3)),
    sitemap-url("https://example.com/blog/hayaku-naro/", lastmod: datetime(year: 2026, month: 7, day: 1)),
  ),
)
```

`sitemap-url` builds a `<url>` entry from a canonical URL (`loc`) plus optional `lastmod` (`datetime` or `str`), `changefreq`, `priority`, and `alternates` (`((hreflang: "de", href: "..."),)` — automatically declaring `xmlns:xhtml` on `<urlset>`).
