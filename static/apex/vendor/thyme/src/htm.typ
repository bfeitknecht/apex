// Ergonomic HTML authoring in Typst:
// 1. `h(selector, ..args)` — Mithril-style CSS selector hyperscript (`"section#projects.entries"`).
// 2. `htm(template, ..slots)` — `developit/htm`-inspired Hypertagged Markup parser with `{slot}`
//    Typst content interpolation, O(n) slice-based tokenization, numeric & named entity decoding,
//    `<pre>`/`<textarea>` whitespace preservation, HTML5 optional-close tag handling, and
//    tag-balance diagnostics, backed by Typst 0.15's typed `html.*` constructors.

#import "text.typ": to-plain-text, word-hint

#let _html-dict = if "html" in dictionary(std) { dictionary(std.html) } else { (:) }

#let _join-content(arr) = if arr.len() > 0 { arr.join() } else { [] }

#let _void-tags = (
  "area",
  "base",
  "br",
  "col",
  "embed",
  "hr",
  "img",
  "input",
  "link",
  "meta",
  "param",
  "source",
  "track",
  "wbr",
)

#let _preformatted-tags = ("pre", "textarea")
#let _nonprose-tags = ("pre", "script", "style", "template")

// HTML5 elements whose end tag may be omitted when followed by a sibling or parent close tag.
#let _optional-close-tags = (
  "colgroup",
  "dd",
  "dt",
  "li",
  "option",
  "p",
  "tbody",
  "td",
  "tfoot",
  "th",
  "thead",
  "tr",
)

#let _auto-close-peers = (
  li: ("li",),
  dt: ("dt", "dd"),
  dd: ("dt", "dd"),
  p: (
    "address", "article", "aside", "blockquote", "details", "div", "dl", "fieldset",
    "figcaption", "figure", "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6",
    "header", "hgroup", "hr", "main", "nav", "ol", "p", "pre", "section", "table", "ul",
  ),
  tr: ("tr",),
  td: ("td", "th"),
  th: ("td", "th"),
  thead: ("tbody", "tfoot"),
  tbody: ("tbody", "tfoot"),
  option: ("option",),
)

#let _paged-hidden-tags = ("script", "style", "meta", "link", "head", "base", "template")

#let _paged-block-tags = (
  "address",
  "article",
  "aside",
  "blockquote",
  "details",
  "dialog",
  "dd",
  "div",
  "dl",
  "dt",
  "fieldset",
  "figcaption",
  "figure",
  "footer",
  "form",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "header",
  "hgroup",
  "li",
  "main",
  "nav",
  "ol",
  "p",
  "pre",
  "section",
  "summary",
  "table",
  "ul",
)

#let _ws-boundary-tags = (
  _paged-block-tags
    + _paged-hidden-tags
    + ("html", "body", "thead", "tbody", "tfoot", "tr", "td", "th", "colgroup", "col", "hr", "br", "wbr")
)

// Peek at the tag opening at index `idx` (`cs.at(idx) == "<"`) to determine whether
// trailing newline whitespace precedes an inline opening tag (e.g. `<a>`, `<code>`, `<em>`).
#let _peek-next-inline-open(cs, idx, n) = {
  if idx + 1 >= n { return false }
  let c1 = cs.at(idx + 1)
  if c1 in ("!", "/", "?") { return false }
  let j = idx + 1
  while j < n and cs.at(j) in (" ", "\t", "\r", "\n") { j += 1 }
  let ts = j
  while j < n and cs.at(j) not in (" ", "\t", "\r", "\n", ">", "/") { j += 1 }
  let tag = lower(cs.slice(ts, j).join())
  tag != "" and tag not in _ws-boundary-tags
}

#let _typed-str-attrs = (
  "action",
  "alt",
  "charset",
  "cite",
  "class",
  "content",
  "download",
  "for",
  "href",
  "hreflang",
  "id",
  "label",
  "lang",
  "media",
  "name",
  "placeholder",
  "poster",
  "sizes",
  "slot",
  "src",
  "srcset",
  "style",
  "title",
  "type",
  "value",
)

#let _typed-bool-attrs = (
  "async",
  "autofocus",
  "autoplay",
  "checked",
  "controls",
  "defer",
  "disabled",
  "hidden",
  "loop",
  "multiple",
  "muted",
  "novalidate",
  "open",
  "readonly",
  "required",
  "reversed",
  "selected",
  "spellcheck",
  "translate",
)

#let _typed-int-attrs = (
  "cols",
  "colspan",
  "height",
  "maxlength",
  "minlength",
  "rows",
  "rowspan",
  "size",
  "span",
  "start",
  "tabindex",
  "width",
)

#let _named-entities = (
  amp: "&",
  lt: "<",
  gt: ">",
  quot: "\"",
  apos: "'",
  nbsp: "\u{00A0}",
  middot: "·",
  mdash: "—",
  ndash: "–",
  hellip: "…",
  bull: "•",
  sect: "§",
  para: "¶",
  copy: "©",
  reg: "®",
  trade: "™",
  times: "×",
  divide: "÷",
  plusmn: "±",
  minus: "−",
  prime: "′",
  Prime: "″",
  dagger: "†",
  ddagger: "‡",
  rarr: "→",
  larr: "←",
  uarr: "↑",
  darr: "↓",
  harr: "↔",
  rArr: "⇒",
  lArr: "⇐",
  hArr: "⇔",
  iff: "⇔",
  forall: "∀",
  exist: "∃",
  empty: "∅",
  isin: "∈",
  notin: "∉",
  sub: "⊂",
  sup: "⊃",
  sube: "⊆",
  supe: "⊇",
  cup: "∪",
  cap: "∩",
  infin: "∞",
  ne: "≠",
  le: "≤",
  ge: "≥",
  approx: "≈",
  equiv: "≡",
  ldquo: "“",
  rdquo: "”",
  lsquo: "‘",
  rsquo: "’",
  laquo: "«",
  raquo: "»",
  lsaquo: "‹",
  rsaquo: "›",
  hearts: "♥",
)

#let _ws-clusters = (" ", "\t", "\n", "\r", "\r\n")

#let _is-int-str(s) = {
  type(s) == str and s.matches(regex("^-?[0-9]+$")).len() == 1
}

// Single-pass HTML character reference decoder supporting named entities (`&mdash;`, `&sect;`)
// and arbitrary decimal (`&#8212;`) or hexadecimal (`&#x2014;`) Unicode code points without double-unescaping.
#let _decode-entities(s) = {
  if not s.contains("&") {
    return s
  }
  s.replace(regex("&(?:#[xX]([0-9a-fA-F]+)|#([0-9]+)|([a-zA-Z][a-zA-Z0-9]*));"), m => {
    let raw-tok = m.text
    let body = raw-tok.slice(1, raw-tok.len() - 1)
    if body.starts-with("#x") or body.starts-with("#X") {
      let cp = int(body.slice(2), base: 16)
      if cp > 0 and cp <= 0x10FFFF { str.from-unicode(cp) } else { raw-tok }
    } else if body.starts-with("#") {
      let cp = int(body.slice(1))
      if cp > 0 and cp <= 0x10FFFF { str.from-unicode(cp) } else { raw-tok }
    } else if body in _named-entities {
      _named-entities.at(body)
    } else {
      raw-tok
    }
  })
}

// Construct a single HTML element using Typst's built-in typed `html.<tag>` functions
// when all attributes match safe typed signatures, falling back to `html.elem` for
// `data-*`, `aria-*`, custom `rel` tokens (e.g. `rel="me"`), SVG attributes, or custom tags.
#let _emit-element(tag, raw-attrs, body) = {
  let clean-tag = lower(str(tag).trim())
  let is-void = clean-tag in _void-tags
  let safe-body = if body == none { [] } else { body }

  if target() == "html" {
    let can-use-typed = (
      clean-tag in _html-dict
        and clean-tag not in ("elem", "frame", "html", "head", "body")
    )
    let typed-kwargs = (:)
    if can-use-typed {
      for (k, v) in raw-attrs.pairs() {
        if v == none or v == false {
          continue
        } else if k == "class" {
          if type(v) == array {
            typed-kwargs.insert("class", v.map(to-plain-text))
          } else {
            typed-kwargs.insert("class", to-plain-text(v))
          }
        } else if k in _typed-bool-attrs {
          let plain-v = lower(to-plain-text(v))
          if v == true or v == "" or plain-v == k or plain-v == "true" {
            typed-kwargs.insert(k, true)
          } else if plain-v != "false" {
            can-use-typed = false
          }
        } else if k in _typed-int-attrs {
          if type(v) == int {
            typed-kwargs.insert(k, v)
          } else if _is-int-str(to-plain-text(v)) {
            typed-kwargs.insert(k, int(to-plain-text(v)))
          } else {
            can-use-typed = false
          }
        } else if k in _typed-str-attrs {
          typed-kwargs.insert(k, to-plain-text(v))
        } else {
          can-use-typed = false
        }
      }
    }

    if can-use-typed {
      let fn = _html-dict.at(clean-tag)
      if is-void {
        fn(..typed-kwargs)
      } else {
        fn(..typed-kwargs, safe-body)
      }
    } else {
      let str-attrs = (:)
      for (k, v) in raw-attrs.pairs() {
        if v == none or v == false {
          continue
        } else if v == true {
          str-attrs.insert(k, "")
        } else if k in _typed-bool-attrs and lower(to-plain-text(v)) == "false" {
          continue
        } else if type(v) == array {
          str-attrs.insert(k, v.map(to-plain-text).join(" "))
        } else {
          str-attrs.insert(k, to-plain-text(v))
        }
      }
      if is-void {
        html.elem(clean-tag, attrs: str-attrs)
      } else {
        html.elem(clean-tag, attrs: str-attrs, safe-body)
      }
    }
  } else {
    // Graceful paged PDF fallback so `typst watch` previews never fail on HTML blocks
    if clean-tag in _paged-hidden-tags or clean-tag == "input" {
      []
    } else if clean-tag == "br" {
      linebreak()
    } else if clean-tag == "hr" {
      line(length: 100%, stroke: 0.5pt + luma(80%))
    } else if clean-tag == "a" and "href" in raw-attrs and raw-attrs.href != none {
      link(to-plain-text(raw-attrs.href), safe-body)
    } else if clean-tag in ("strong", "b") {
      strong(safe-body)
    } else if clean-tag in ("em", "i") {
      emph(safe-body)
    } else if clean-tag == "h1" {
      block(above: 1.2em, below: 0.5em, text(size: 1.6em, weight: "bold", safe-body))
    } else if clean-tag == "h2" {
      block(above: 1.1em, below: 0.45em, text(size: 1.3em, weight: "semibold", safe-body))
    } else if clean-tag == "h3" {
      block(above: 0.9em, below: 0.35em, text(size: 1.1em, weight: "semibold", safe-body))
    } else if clean-tag in _paged-block-tags {
      block(width: 100%, above: 0.45em, below: 0.45em, safe-body)
    } else {
      safe-body
    }
  }
}

// O(n) slice-based parser for CSS-style hyperscript selectors `"section#projects.entries[data-x=1][open]"`.
#let _parse-h-selector(sel) = {
  let cs = str(sel).trim().clusters()
  let n = cs.len()
  let i = 0
  let id = none
  let classes = ()
  let attrs = (:)

  let t-start = i
  while i < n and cs.at(i) not in ("#", ".", "[") { i += 1 }
  let raw-tag = cs.slice(t-start, i).join()
  let tag = if raw-tag == "" { "div" } else { raw-tag }

  while i < n {
    let ch = cs.at(i)
    if ch == "#" {
      i += 1
      let s = i
      while i < n and cs.at(i) not in ("#", ".", "[") { i += 1 }
      let buf = cs.slice(s, i).join()
      if buf != "" { id = buf }
    } else if ch == "." {
      i += 1
      let s = i
      while i < n and cs.at(i) not in ("#", ".", "[") { i += 1 }
      let buf = cs.slice(s, i).join()
      if buf != "" { classes.push(buf) }
    } else if ch == "[" {
      i += 1
      let ks = i
      while i < n and cs.at(i) not in ("=", "]") { i += 1 }
      let key = cs.slice(ks, i).join().trim()
      if i < n and cs.at(i) == "=" {
        i += 1
        let val = if i < n and cs.at(i) in ("\"", "'") {
          let q = cs.at(i)
          i += 1
          let vs = i
          while i < n and cs.at(i) != q { i += 1 }
          let v = cs.slice(vs, i).join()
          if i < n and cs.at(i) == q { i += 1 }
          v
        } else {
          let vs = i
          while i < n and cs.at(i) != "]" { i += 1 }
          cs.slice(vs, i).join()
        }
        if key != "" { attrs.insert(key, val) }
      } else if key != "" {
        attrs.insert(key, true)
      }
      if i < n and cs.at(i) == "]" { i += 1 }
    } else {
      i += 1
    }
  }

  (tag: tag, id: id, classes: classes, attrs: attrs)
}

/// Construct an HTML element from a concise CSS-style selector string
/// (`"section#projects.entries[data-tier=2]"`) backed by Typst's typed `html.*` module.
/// When passed a `length`, `relative`, `ratio`, or `fraction` (e.g. `#h(1fr)` or `#h(0.5em)`),
/// transparently delegates to Typst's built-in horizontal spacing function `std.h`.
///
/// ```example
/// #h("div.callout#intro")[Hello from *hyperscript*.]
/// ```
/// -> content
#let h(
  /// Tag + `#id` + `.class` + `[attr=val]` selector (`"div"` assumed when tag is omitted),
  /// or a spacing amount (`length` / `fraction`) forwarded to `std.h`.
  /// -> str | relative | fraction
  selector,
  /// Additional named attributes (`href: "..."`, `open: true`), optional attribute dictionaries,
  /// and child content blocks (or `weak: bool` when forwarding to `std.h`).
  /// -> any
  ..args,
) = {
  if type(selector) != str {
    return std.h(selector, ..args)
  }
  let parsed = _parse-h-selector(selector)
  let clean-tag = lower(parsed.tag.trim())
  let merged = parsed.attrs
  if parsed.id != none {
    merged.insert("id", parsed.id)
  }
  let classes = parsed.classes

  let children = ()
  for pos-arg in args.pos() {
    if type(pos-arg) == dictionary {
      for (k, v) in pos-arg.pairs() {
        if k == "class" and v != none {
          if type(v) == array { classes += v.map(to-plain-text) } else { classes.push(to-plain-text(v)) }
        } else {
          merged.insert(k, v)
        }
      }
    } else {
      children.push(pos-arg)
    }
  }

  for (k, v) in args.named().pairs() {
    if k == "class" and v != none {
      if type(v) == array { classes += v.map(to-plain-text) } else { classes.push(to-plain-text(v)) }
    } else {
      merged.insert(k, v)
    }
  }

  if classes.len() > 0 {
    merged.insert("class", classes)
  }

  context {
    if clean-tag not in _nonprose-tags and children.len() > 0 {
      word-hint(..children)
    }
    _emit-element(parsed.tag, merged, _join-content(children))
  }
}

// Look up a `{slot}` value from named or positional (`{0}`, `{1}`) arguments.
#let _lookup-slot(name, named-slots, pos-slots) = {
  let key = name.trim()
  if key in named-slots {
    named-slots.at(key)
  } else if _is-int-str(key) {
    let idx = int(key)
    if idx >= 0 and idx < pos-slots.len() {
      pos-slots.at(idx)
    } else {
      panic("`htm(...)`: positional slot `{" + key + "}` out of range (" + str(pos-slots.len()) + " positional args).")
    }
  } else {
    panic("`htm(...)`: unknown slot `{" + key + "}` (available: " + named-slots.keys().join(", ") + ").")
  }
}

// O(n) slice-based `{slot}` (and `{{` / `}}` escape) interpolation inside an attribute value.
#let _interpolate-attr-str(raw-str, named-slots, pos-slots) = {
  if not raw-str.contains("{") and not raw-str.contains("}") {
    return _decode-entities(raw-str)
  }
  let cs = raw-str.clusters()
  let n = cs.len()
  let i = 0
  let segs = ()

  while i < n {
    let start = i
    while i < n and cs.at(i) not in ("{", "}") { i += 1 }
    if i > start {
      segs.push(_decode-entities(cs.slice(start, i).join()))
    }
    if i >= n { break }

    let ch = cs.at(i)
    if ch == "{" {
      if i + 1 < n and cs.at(i + 1) == "{" {
        segs.push("{")
        i += 2
      } else {
        i += 1
        let ks = i
        while i < n and cs.at(i) != "}" { i += 1 }
        let key = cs.slice(ks, i).join()
        if i < n and cs.at(i) == "}" { i += 1 }
        segs.push(to-plain-text(_lookup-slot(key, named-slots, pos-slots)))
      }
    } else if ch == "}" {
      if i + 1 < n and cs.at(i + 1) == "}" {
        segs.push("}")
        i += 2
      } else {
        segs.push("}")
        i += 1
      }
    }
  }
  if segs.len() > 0 { segs.join() } else { "" }
}

// O(n) slice-based `{slot}` interpolation inside text nodes, returning an array of `content | str` pieces.
#let _interpolate-text-nodes(
  raw-str,
  named-slots,
  pos-slots,
  preserve-ws: false,
  keep-leading-space: false,
  keep-trailing-space: false,
) = {
  if not preserve-ws and raw-str.contains("\n") and raw-str.trim(regex("[ \\t\\r\\n]+")) == "" {
    return if keep-leading-space and keep-trailing-space { (" ",) } else { () }
  }
  let cleaned = if not preserve-ws and raw-str.contains("\n") {
    raw-str
      .replace(regex("^[ \\t]*\\r?\\n[ \\t]*"), if keep-leading-space { " " } else { "" })
      .replace(regex("[ \\t]*\\r?\\n[ \\t]*$"), if keep-trailing-space { " " } else { "" })
      .replace(regex("[ \\t]*\\r?\\n[ \\t]*"), " ")
  } else {
    raw-str
  }
  if cleaned == "" {
    return ()
  }
  if not cleaned.contains("{") and not cleaned.contains("}") {
    return (_decode-entities(cleaned),)
  }

  let cs = cleaned.clusters()
  let n = cs.len()
  let i = 0
  let buf-segs = ()
  let parts = ()

  while i < n {
    let start = i
    while i < n and cs.at(i) not in ("{", "}") { i += 1 }
    if i > start {
      buf-segs.push(cs.slice(start, i).join())
    }
    if i >= n { break }

    let ch = cs.at(i)
    if ch == "{" {
      if i + 1 < n and cs.at(i + 1) == "{" {
        buf-segs.push("{")
        i += 2
      } else {
        if buf-segs.len() > 0 {
          parts.push(_decode-entities(buf-segs.join()))
          buf-segs = ()
        }
        i += 1
        let ks = i
        while i < n and cs.at(i) != "}" { i += 1 }
        let key = cs.slice(ks, i).join()
        if i < n and cs.at(i) == "}" { i += 1 }
        parts.push(_lookup-slot(key, named-slots, pos-slots))
      }
    } else if ch == "}" {
      if i + 1 < n and cs.at(i + 1) == "}" {
        buf-segs.push("}")
        i += 2
      } else {
        buf-segs.push("}")
        i += 1
      }
    }
  }
  if buf-segs.len() > 0 {
    parts.push(_decode-entities(buf-segs.join()))
  }
  parts
}

// O(n) slice-based HTML fragment tokenizer producing `(kind: "open" | "close" | "text", ...)` records
// while preserving verbatim whitespace inside `<pre>` / `<textarea>`.
#let _tokenize-htm(src, named-slots, pos-slots) = {
  let cs = src.clusters()
  let n = cs.len()
  let i = 0
  let pre-depth = 0
  let nonprose-depth = 0
  let tokens = ()
  let prose-pieces = ()

  while i < n {
    if cs.at(i) == "<" {
      if i + 3 < n and cs.at(i + 1) == "!" and cs.at(i + 2) == "-" and cs.at(i + 3) == "-" {
        i += 4
        while i + 2 < n and not (cs.at(i) == "-" and cs.at(i + 1) == "-" and cs.at(i + 2) == ">") {
          i += 1
        }
        i = calc.min(n, i + 3)
      } else if i + 1 < n and cs.at(i + 1) == "!" {
        while i < n and cs.at(i) != ">" { i += 1 }
        if i < n { i += 1 }
      } else if i + 1 < n and cs.at(i + 1) == "/" {
        i += 2
        let ts = i
        while i < n and cs.at(i) != ">" { i += 1 }
        let tag = lower(cs.slice(ts, i).join().trim(regex("[ \\t\\r\\n]+")))
        if i < n and cs.at(i) == ">" { i += 1 }
        if tag in _preformatted-tags and pre-depth > 0 {
          pre-depth -= 1
        }
        if tag in _nonprose-tags and nonprose-depth > 0 {
          nonprose-depth -= 1
        }
        tokens.push((kind: "close", tag: tag))
      } else {
        i += 1
        while i < n and cs.at(i) in _ws-clusters { i += 1 }
        let ts = i
        while i < n and cs.at(i) not in _ws-clusters and cs.at(i) not in (">", "/") { i += 1 }
        let tag = cs.slice(ts, i).join()
        let attrs = (:)
        let self-close = false

        while i < n {
          while i < n and cs.at(i) in _ws-clusters { i += 1 }
          if i >= n { break }
          if cs.at(i) == ">" {
            i += 1
            break
          }
          if cs.at(i) == "/" and i + 1 < n and cs.at(i + 1) == ">" {
            self-close = true
            i += 2
            break
          }

          let aks = i
          while i < n and cs.at(i) not in _ws-clusters and cs.at(i) not in ("=", ">", "/") { i += 1 }
          let akey = cs.slice(aks, i).join()
          while i < n and cs.at(i) in _ws-clusters { i += 1 }

          if i < n and cs.at(i) == "=" {
            i += 1
            while i < n and cs.at(i) in _ws-clusters { i += 1 }
            if i < n and cs.at(i) in ("\"", "'") {
              let q = cs.at(i)
              i += 1
              let avs = i
              while i < n and cs.at(i) != q { i += 1 }
              let aval = cs.slice(avs, i).join()
              if i < n and cs.at(i) == q { i += 1 }
              if akey != "" {
                attrs.insert(akey, _interpolate-attr-str(aval, named-slots, pos-slots))
              }
            } else if i < n and cs.at(i) == "{" {
              i += 1
              let sks = i
              while i < n and cs.at(i) != "}" { i += 1 }
              let skey = cs.slice(sks, i).join()
              if i < n and cs.at(i) == "}" { i += 1 }
              if akey != "" {
                attrs.insert(akey, _lookup-slot(skey, named-slots, pos-slots))
              }
            } else {
              let avs = i
              while i < n and cs.at(i) not in _ws-clusters and cs.at(i) not in (">", "/") { i += 1 }
              let aval = cs.slice(avs, i).join()
              if akey != "" {
                attrs.insert(akey, _decode-entities(aval))
              }
            }
          } else if akey != "" {
            attrs.insert(akey, true)
          }
        }

        let clean-tag = lower(tag)
        let is-self-close = self-close or clean-tag in _void-tags
        if not is-self-close {
          if clean-tag in _preformatted-tags {
            pre-depth += 1
          }
          if clean-tag in _nonprose-tags {
            nonprose-depth += 1
          }
        }
        tokens.push((
          kind: "open",
          tag: clean-tag,
          attrs: attrs,
          self-close: is-self-close,
        ))
      }
    } else {
      let tx-start = i
      while i < n and cs.at(i) != "<" { i += 1 }
      let text-buf = cs.slice(tx-start, i).join()
      let prev-tok = if tokens.len() > 0 { tokens.last() } else { none }
      let prev-is-inline = (
        prev-tok != none
          and prev-tok.tag not in _ws-boundary-tags
          and (prev-tok.kind == "close" or prev-tok.at("self-close", default: false))
      )
      let next-is-inline = i < n and _peek-next-inline-open(cs, i, n)
      let pieces = _interpolate-text-nodes(
        text-buf,
        named-slots,
        pos-slots,
        preserve-ws: pre-depth > 0,
        keep-leading-space: prev-is-inline,
        keep-trailing-space: next-is-inline,
      )
      if pieces.len() > 0 {
        if nonprose-depth == 0 {
          prose-pieces += pieces
        }
        tokens.push((kind: "text", pieces: pieces))
      }
    }
  }

  (tokens: tokens, prose-pieces: prose-pieces)
}

// Recursive descent HTML tree builder with HTML5 optional-end-tag auto-closing
// and explicit diagnostics on mismatched closing tags.
#let _build-htm-tree(tokens, start-idx, stop-tag, ancestors: ()) = {
  let idx = start-idx
  let n = tokens.len()
  let nodes = ()
  let peer-closers = if stop-tag != none { _auto-close-peers.at(stop-tag, default: ()) } else { () }

  while idx < n {
    let tok = tokens.at(idx)
    if tok.kind == "close" {
      if stop-tag != none and tok.tag == stop-tag {
        return (content: _join-content(nodes), next: idx + 1)
      } else if tok.tag in _void-tags {
        idx += 1
      } else if tok.tag in ancestors {
        if stop-tag in _optional-close-tags {
          return (content: _join-content(nodes), next: idx)
        }
        panic("`htm(...)`: unclosed `<" + stop-tag + ">` before `</" + tok.tag + ">`.")
      } else {
        let ctx = if stop-tag != none { " inside `<" + stop-tag + ">`" } else { "" }
        panic("`htm(...)`: unexpected closing tag `</" + tok.tag + ">`" + ctx + ".")
      }
    } else if tok.kind == "text" {
      nodes += tok.pieces
      idx += 1
    } else if tok.kind == "open" {
      if stop-tag != none and tok.tag in peer-closers {
        return (content: _join-content(nodes), next: idx)
      }
      if tok.self-close {
        nodes.push(_emit-element(tok.tag, tok.attrs, []))
        idx += 1
      } else {
        let next-ancestors = if stop-tag != none { ancestors + (stop-tag,) } else { ancestors }
        let sub = _build-htm-tree(tokens, idx + 1, tok.tag, ancestors: next-ancestors)
        nodes.push(_emit-element(tok.tag, tok.attrs, sub.content))
        idx = sub.next
      }
    }
  }

  if stop-tag != none and stop-tag not in _optional-close-tags {
    panic("`htm(...)`: unclosed `<" + stop-tag + ">` at end of HTML fragment.")
  }

  (content: _join-content(nodes), next: idx)
}

/// Parse Hypertagged Markup (`developit/htm`-style HTML fragments with `{slot}` Typst
/// content interpolation) directly into Typst's native typed `html.*` elements.
///
/// Accepts either a fenced `raw` block (`lang: "html"`) or a `str` template,
/// plus named (`name: [...]`) or positional (`{0}`) Typst content slots.
///
/// ```example
/// #htm(
///   "<section class=\"intro\"><p><strong>{greeting}</strong> — {note}</p></section>",
///   greeting: [Grüezi],
///   note: [compiled straight to native HTML elements with $O(n)$ parsing.],
/// )
/// ```
/// -> content
#let htm(
  /// HTML fragment (`raw` block or `str`) with optional `{slot}` placeholders (`{{` / `}}` for literal braces).
  /// -> content | str
  template,
  /// Named (`key: content`) or positional (`{0}`, `{1}`) Typst values spliced into `{slot}` positions.
  /// -> any
  ..slots,
) = {
  let src = if type(template) == str {
    template
  } else if type(template) == content and template.func() == raw {
    template.text
  } else {
    to-plain-text(template)
  }

  let parsed = _tokenize-htm(src, slots.named(), slots.pos())
  context {
    if parsed.prose-pieces.len() > 0 {
      word-hint(..parsed.prose-pieces)
    }
    _build-htm-tree(parsed.tokens, 0, none).content
  }
}

/// Optional show-rule helper (`#show: enable-htm`) that compiles `lang: "htm"`
/// raw blocks directly into HTML elements via `htm(...)`.
/// -> content
#let enable-htm(
  /// Document body.
  /// -> content
  body,
) = {
  show raw.where(lang: "htm"): it => htm(it)
  body
}
