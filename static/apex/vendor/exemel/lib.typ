#let _plugin = plugin("xml_plugin.wasm")

#let _rss-weekdays = ("Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat")
#let _rss-months = ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")

#let _space-func = [ ].func()

#let _to-text(v) = {
  if v == none {
    ""
  } else if type(v) == str {
    v
  } else if type(v) == int or type(v) == float {
    str(v)
  } else if type(v) == content {
    if v == [ ] or v.func() == _space-func {
      " "
    } else if v.func() == parbreak or v.func() == linebreak {
      "\n"
    } else if v.func() == smartquote {
      if v.at("double", default: true) { "\"" } else { "'" }
    } else if v.func() == metadata {
      ""
    } else if v.has("text") {
      if type(v.text) == str { v.text } else { _to-text(v.text) }
    } else if v.has("children") {
      if v.children.len() == 0 { "" } else { v.children.map(_to-text).join("") }
    } else if v.has("body") {
      _to-text(v.body)
    } else if v.has("child") {
      _to-text(v.child)
    } else if v.has("base") {
      let s = _to-text(v.base)
      for k in ("tl", "bl") {
        if v.has(k) and v.at(k) != none { s = _to-text(v.at(k)) + s }
      }
      if v.has("accent") and v.accent != none { s += _to-text(v.accent) }
      if v.has("b") and v.b != none { s += "_" + _to-text(v.b) }
      if v.has("t") and v.t != none { s += "^" + _to-text(v.t) }
      for k in ("br", "tr") {
        if v.has(k) and v.at(k) != none { s += _to-text(v.at(k)) }
      }
      s
    } else if v.has("num") and v.has("denom") {
      _to-text(v.num) + "/" + _to-text(v.denom)
    } else if v.has("radicand") {
      let idx = if v.has("index") and v.index != none { _to-text(v.index) } else { "" }
      idx + "√(" + _to-text(v.radicand) + ")"
    } else if v.func() == math.primes and v.has("count") {
      "'" * v.count
    } else {
      ""
    }
  } else {
    str(v)
  }
}

#let _join-url(base, path) = {
  str(base).trim("/", at: end) + "/" + str(path).trim("/", at: start)
}

#let _normalize-datetime(dt, field-name: "datetime") = {
  if type(dt) != datetime {
    panic("`" + field-name + "` must be a `datetime`!")
  }
  if dt.year() == none or dt.month() == none or dt.day() == none {
    panic("`" + field-name + "` must include `year`, `month`, and `day`!")
  }
  if dt.hour() == none {
    datetime(
      year: dt.year(),
      month: dt.month(),
      day: dt.day(),
      hour: 0,
      minute: 0,
      second: 0,
    )
  } else {
    dt
  }
}

#let _normalize-rfc3339-tz(tz) = {
  let s = str(tz).trim()
  if s == "" or s == "Z" or s == "z" or s == "+0000" or s == "-0000" {
    "Z"
  } else if s.len() == 5 and (s.starts-with("+") or s.starts-with("-")) and not s.contains(":") {
    s.slice(0, 3) + ":" + s.slice(3, 5)
  } else {
    s
  }
}

/// Format a `datetime` as an RFC 3339 timestamp for Atom 1.0 (`<updated>`, `<published>`) and XML sitemaps.
#let format-rfc3339(dt, tz: "Z", field-name: "datetime") = {
  _normalize-datetime(dt, field-name: field-name).display(
    "[year]-[month]-[day]T[hour]:[minute]:[second]",
  ) + _normalize-rfc3339-tz(tz)
}

#let _weekday-index(year, month, day) = {
  let offsets = (0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4)
  let y = if month < 3 { year - 1 } else { year }
  calc.rem(
    y + calc.quo(y, 4) - calc.quo(y, 100) + calc.quo(y, 400) + offsets.at(month - 1) + day,
    7,
  )
}

#let _find-child-text(el, tag) = {
  if type(el) != dictionary { return none }
  for ch in el.at("children", default: ()) {
    if type(ch) == dictionary and ch.at("tag", default: none) == tag {
      let kids = ch.at("children", default: ())
      if kids.len() > 0 and type(kids.first()) == str {
        return kids.first()
      }
    }
  }
  none
}

#let _has-namespaced-child(el, ns) = {
  if type(el) != dictionary { return false }
  if el.at("namespace", default: none) == ns { return true }
  for ch in el.at("children", default: ()) {
    if _has-namespaced-child(ch, ns) { return true }
  }
  false
}

/// Serialize one or more XML element dictionaries into an XML document string.
#let to-xml(value, pretty: false, header: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>") = {
  if type(value) != array {
    value = (value,)
  }
  header
  if pretty { "\n" }
  for val in value {
    str(_plugin.to_xml(cbor.encode((
      root: val,
      pretty: pretty,
    ))))
  }
}

/// Format a `datetime` as an RFC 2822 / RFC 822 timestamp for RSS 2.0 (`<pubDate>`, `<lastBuildDate>`).
#let format-rfc2822(dt, tz: "+0000") = {
  let utc = _normalize-datetime(dt, field-name: "dt")
  let wd = _rss-weekdays.at(_weekday-index(utc.year(), utc.month(), utc.day()))
  let mon = _rss-months.at(utc.month() - 1)
  let day = utc.display("[day padding:zero]")
  let hms = utc.display("[hour]:[minute]:[second]")
  wd + ", " + day + " " + mon + " " + str(utc.year()) + " " + hms + " " + tz
}

/// Build an RFC 4151 `tag:` URI for Atom feed and entry `<id>` elements.
#let atom-site-id(
  scheme: "tag",
  authority: none,
  time: none,
  identifier: "blog/feed",
) = {
  if authority == none {
    panic("`authority` cannot be `none`! Enter a domain you control (e.g. example.com)")
  }
  if type(time) != datetime {
    panic("`time` must be a `datetime`!")
  }
  scheme + ":" + _to-text(authority) + "," + time.display("[year]-[month]-[day]") + ":" + _to-text(identifier)
}

/// Build an Atom `<author>` element.
#let atom-author(
  name,
  email: none,
  uri: none,
) = {
  (
    tag: "author",
    children: (
      (tag: "name", children: (_to-text(name),)),
      ..if email != none { ((tag: "email", children: (_to-text(email),)),) } else { () },
      ..if uri != none { ((tag: "uri", children: (_to-text(uri),)),) } else { () },
    ),
  )
}

/// Build an Atom 1.0 `<entry>` element.
#let atom-post(
  title,
  link: none,
  id: none,
  updated: auto,
  published: none,
  tz: "Z",
  summary: none,
  content: none,
  content-type: "html",
  authors: (),
  categories: (),
) = {
  if link == none {
    panic("`link` cannot be `none`!")
  }
  if id == none {
    panic("`id` cannot be `none`! See `atom-site-id` for a helper.")
  }
  let eff-updated = if updated == auto { published } else { updated }
  if eff-updated == none {
    panic("`updated` cannot be `none` (pass `updated` or `published`)!")
  }

  (
    tag: "entry",
    children: (
      (tag: "title", children: (_to-text(title),)),
      (
        tag: "link",
        attrs: (rel: "alternate", type: "text/html", href: _to-text(link)),
      ),
      (tag: "id", children: (_to-text(id),)),
      (tag: "updated", children: (format-rfc3339(eff-updated, tz: tz, field-name: "updated"),)),
      ..if published != none {
        ((tag: "published", children: (format-rfc3339(published, tz: tz, field-name: "published"),)),)
      } else { () },
      ..if summary != none { ((tag: "summary", children: (_to-text(summary),)),) } else { () },
      ..if content != none {
        ((tag: "content", attrs: (type: _to-text(content-type)), children: (_to-text(content),)),)
      } else { () },
      ..authors,
      ..categories.map(term => (tag: "category", attrs: (term: _to-text(term)))),
    ),
  )
}

/// Encode an Atom 1.0 `<feed>` document to XML.
#let atom-encode(
  title: none,
  subtitle: none,
  site-link: none,
  feed-link: auto,
  id: none,
  updated: auto,
  tz: "Z",
  authors: (),
  posts: (),
) = {
  if title == none {
    panic("`title` cannot be `none`!")
  }
  if site-link == none {
    panic("`site-link` cannot be `none`!")
  }
  if id == none {
    panic("`id` cannot be `none`! See `atom-site-id` for a helper.")
  }

  let updated-str = if updated == auto {
    let post-times = posts.map(p => _find-child-text(p, "updated")).filter(t => t != none)
    if post-times.len() == 0 {
      panic("`updated` cannot be inferred when `posts` is empty; pass a `datetime` to `updated`!")
    }
    post-times.sorted().last()
  } else {
    format-rfc3339(updated, tz: tz, field-name: "updated")
  }

  let resolved-feed-link = if feed-link == auto {
    _join-url(site-link, "feed.xml")
  } else {
    _to-text(feed-link)
  }

  let c = (
    tag: "feed",
    attrs: (xmlns: "http://www.w3.org/2005/Atom"),
    children: (
      (tag: "title", children: (_to-text(title),)),
      ..if subtitle != none { ((tag: "subtitle", children: (_to-text(subtitle),)),) } else { () },
      (
        tag: "link",
        attrs: (rel: "self", type: "application/atom+xml", href: resolved-feed-link),
      ),
      (
        tag: "link",
        attrs: (rel: "alternate", type: "text/html", href: _to-text(site-link)),
      ),
      (tag: "id", children: (_to-text(id),)),
      (tag: "updated", children: (updated-str,)),
      ..authors,
      ..posts,
    ),
  )

  to-xml(c, pretty: true)
}

/// Build an RSS 2.0 `<enclosure>` element for podcast or media attachments.
#let rss-enclosure(
  url,
  length: 0,
  type: "application/octet-stream",
) = {
  if url == none {
    panic("`url` cannot be `none`!")
  }
  (
    tag: "enclosure",
    attrs: (
      url: _to-text(url),
      length: str(length),
      type: _to-text(type),
    ),
  )
}

/// Build an RSS 2.0 `<item>` element using the same parameter conventions as `atom-post`.
#let rss-post(
  title,
  link: none,
  id: auto,
  is-permalink: auto,
  published: none,
  tz: "+0000",
  summary: none,
  content: none,
  author: none,
  categories: (),
  comments: none,
  enclosure: none,
) = {
  if title == none and summary == none {
    panic("RSS `<item>` requires at least `title` or `summary`!")
  }
  let link-str = if link != none { _to-text(link) } else { none }
  let eff-id = if id == auto { link-str } else if id != none { _to-text(id) } else { none }
  let eff-permalink = if is-permalink == auto {
    eff-id != none and (eff-id.starts-with("http://") or eff-id.starts-with("https://"))
  } else {
    is-permalink == true
  }
  let pub-str = if published == none {
    none
  } else if type(published) == datetime {
    format-rfc2822(published, tz: tz)
  } else {
    _to-text(published)
  }

  (
    tag: "item",
    children: (
      ..if title != none { ((tag: "title", children: (_to-text(title),)),) } else { () },
      ..if link-str != none { ((tag: "link", children: (link-str,)),) } else { () },
      ..if eff-id != none {
        ((
          tag: "guid",
          attrs: (isPermaLink: if eff-permalink { "true" } else { "false" }),
          children: (eff-id,),
        ),)
      } else { () },
      ..if pub-str != none { ((tag: "pubDate", children: (pub-str,)),) } else { () },
      ..if summary != none { ((tag: "description", children: (_to-text(summary),)),) } else { () },
      ..if content != none {
        ((tag: "encoded", namespace: "content", children: (_to-text(content),)),)
      } else { () },
      ..if author != none { ((tag: "author", children: (_to-text(author),)),) } else { () },
      ..categories.map(term => (tag: "category", children: (_to-text(term),))),
      ..if comments != none { ((tag: "comments", children: (_to-text(comments),)),) } else { () },
      ..if enclosure != none { (enclosure,) } else { () },
    ),
  )
}

#let rss-item = rss-post

#let _rfc2822-sort-key(pub-str) = {
  let parts = str(pub-str).trim().split(" ").filter(s => s != "")
  if parts.len() < 5 {
    return none
  }
  let m-idx = _rss-months.position(m => m == parts.at(2))
  if m-idx == none {
    return none
  }
  let day-raw = parts.at(1)
  let day = if day-raw.len() == 1 { "0" + day-raw } else { day-raw }
  let mon-num = m-idx + 1
  let mon = if mon-num < 10 { "0" + str(mon-num) } else { str(mon-num) }
  parts.at(3) + "-" + mon + "-" + day + "T" + parts.at(4)
}

/// Encode an RSS 2.0 `<rss>` document to XML using the same conventions as `atom-encode`.
#let rss-encode(
  title: none,
  subtitle: none,
  site-link: none,
  feed-link: auto,
  updated: auto,
  tz: "+0000",
  language: none,
  copyright: none,
  generator: none,
  ttl: none,
  posts: (),
) = {
  if title == none {
    panic("`title` cannot be `none`!")
  }
  if site-link == none {
    panic("`site-link` cannot be `none`!")
  }

  let title-str = _to-text(title)
  let desc-str = if subtitle != none { _to-text(subtitle) } else { title-str }
  let resolved-feed-link = if feed-link == auto {
    _join-url(site-link, "rss.xml")
  } else if feed-link != none {
    _to-text(feed-link)
  } else {
    none
  }
  let updated-str = if updated == none {
    none
  } else if updated == auto {
    let pub-pairs = posts
      .map(p => _find-child-text(p, "pubDate"))
      .filter(d => d != none and _rfc2822-sort-key(d) != none)
      .map(d => (key: _rfc2822-sort-key(d), raw: d))
    if pub-pairs.len() > 0 {
      pub-pairs.sorted(key: pair => pair.key).last().raw
    } else {
      none
    }
  } else if type(updated) == datetime {
    format-rfc2822(updated, tz: tz)
  } else {
    _to-text(updated)
  }
  let has-content-ns = posts.any(p => _has-namespaced-child(p, "content"))

  let rss-attrs = (
    version: "2.0",
    ..if resolved-feed-link != none { ("xmlns:atom": "http://www.w3.org/2005/Atom") } else { (:) },
    ..if has-content-ns { ("xmlns:content": "http://purl.org/rss/1.0/modules/content/") } else { (:) },
  )

  let c = (
    tag: "rss",
    attrs: rss-attrs,
    children: (
      (
        tag: "channel",
        children: (
          (tag: "title", children: (title-str,)),
          (tag: "link", children: (_to-text(site-link),)),
          (tag: "description", children: (desc-str,)),
          ..if resolved-feed-link != none {
            ((
              tag: "link",
              namespace: "atom",
              attrs: (
                href: resolved-feed-link,
                rel: "self",
                type: "application/rss+xml",
              ),
            ),)
          } else { () },
          ..if language != none { ((tag: "language", children: (_to-text(language),)),) } else { () },
          ..if copyright != none { ((tag: "copyright", children: (_to-text(copyright),)),) } else { () },
          ..if updated-str != none { ((tag: "lastBuildDate", children: (updated-str,)),) } else { () },
          ..if generator != none { ((tag: "generator", children: (_to-text(generator),)),) } else { () },
          ..if ttl != none { ((tag: "ttl", children: (str(ttl),)),) } else { () },
          ..posts,
        ),
      ),
    ),
  )

  to-xml(c, pretty: true)
}

#let _format-w3c-datetime(dt, tz: "Z", field-name: "lastmod") = {
  if type(dt) != datetime {
    panic("`" + field-name + "` must be a `datetime`!")
  }
  if dt.year() == none or dt.month() == none or dt.day() == none {
    panic("`" + field-name + "` must include `year`, `month`, and `day`!")
  }
  if dt.hour() == none {
    dt.display("[year]-[month]-[day]")
  } else {
    format-rfc3339(dt, tz: tz, field-name: field-name)
  }
}

/// Build a `<url>` entry for `sitemap-encode` (`http://www.sitemaps.org/schemas/sitemap/0.9`).
#let sitemap-url(
  loc,
  lastmod: none,
  tz: "Z",
  changefreq: none,
  priority: none,
  alternates: (),
) = {
  if loc == none {
    panic("`loc` cannot be `none`!")
  }
  let lastmod-str = if lastmod == none {
    none
  } else if type(lastmod) == datetime {
    _format-w3c-datetime(lastmod, tz: tz, field-name: "lastmod")
  } else {
    _to-text(lastmod)
  }

  (
    tag: "url",
    children: (
      (tag: "loc", children: (_to-text(loc),)),
      ..if lastmod-str != none { ((tag: "lastmod", children: (lastmod-str,)),) } else { () },
      ..if changefreq != none { ((tag: "changefreq", children: (_to-text(changefreq),)),) } else { () },
      ..if priority != none { ((tag: "priority", children: (_to-text(priority),)),) } else { () },
      ..alternates.map(alt => (
        tag: "link",
        namespace: "xhtml",
        attrs: (
          rel: "alternate",
          hreflang: _to-text(alt.hreflang),
          href: _to-text(alt.href),
        ),
      )),
    ),
  )
}

/// Encode an XML sitemap (`<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">`).
#let sitemap-encode(
  urls: (),
) = {
  let has-xhtml-ns = urls.any(u => _has-namespaced-child(u, "xhtml"))
  let c = (
    tag: "urlset",
    attrs: (
      xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9",
      ..if has-xhtml-ns { ("xmlns:xhtml": "http://www.w3.org/1999/xhtml") } else { (:) },
    ),
    children: urls,
  )
  to-xml(c, pretty: true)
}
