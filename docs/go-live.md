# Go-live shortlist for 0xbf.ch

Ordered. Group 2 blocks the launch, 3 is the launch, 4 is afterwards.
Everything the infra repo needs lives in `infra/` — see `infra/README.md`; that
directory is deleted once it has been merged.

## 1. What CI has already proven

The restructure was written in a sandbox that could not reach
`static.rust-lang.org` or `crates.io`, so the first real build happened in the
`CI` workflow. On every push it runs, and currently passes:

- `cargo fmt --all -- --check`, `cargo clippy --all-targets --features ssh -- -D warnings`
  and `cargo test` — the `content::` and `app::` tests included;
- `cargo check --target wasm32-unknown-unknown`;
- `trunk build --release`, then asserts the names the page refers to exist
  (`apex.js`, `apex_bg.wasm`, `boot.js`, `app.css`, `cursor.js`) and that the
  fonts are flat in the document root, where `app.css`'s bare `@font-face` URLs
  look for them;
- `typst compile --format bundle` for the page, then asserts the bundle's shape
  (`index.html`, `site.typ`, `contact/`, `404.html`, `sitemap.xml`, the emitted
  `age.txt`/`pgp.txt`/`cv.pdf`, the app shell).

So the code compiles, the tests pass, and both artifacts build. Not verified by
CI: `just docroot`, the Docker stages, and everything in section 2.

## 2. Build both surfaces and look at them

- [ ] `just dev` → `http://localhost:8080/` is the page; the console shows
      `/apex.js` loading and the interface replacing it. Then block WebAssembly
      (`delete window.WebAssembly` in the console, reload) and confirm the page
      stays.
- [ ] `just ssg` on its own: `public/index.html`, `site.typ`, `contact/`,
      `404.html`, `sitemap.xml`, `age.txt`, `pgp.txt`, `cv.pdf` exist, and
      `grep app-shell public/index.html` matches.
- [ ] Disable JavaScript entirely and reload: the page is unchanged, `/cv`
      works, the contact page renders. No `/static`, no redirect.
- [ ] Touch/keyboard/selection in the app; lichen, Unicode, fractal, **crash**.

## 3. Content before anyone looks

- [ ] `content/projects.toml` holds one merged list (10 entries). It is the
      only copy, and it is what both the page and the app render.
- [ ] The `gallery` destination (`https://gallery.0xbf.ch`) is linked but does
      not exist yet: build it, or empty its `url` in `content/profile.toml`.
- [ ] `content/portrait.txt` is a cat. Replace it with the real ASCII portrait.
- [ ] Read the page's voice in `content/profile.toml`: `lede` and `about` are
      shared, `intro` is the terminal's flourish.

## 4. Ship

1. [ ] Create the SSH host key and the `apex-ssh-host-key` Secret
   (`infra/README.md`, steps 1–3).
2. [ ] Copy `infra/kubernetes/.` over the infra repo, uncomment
   `ssh-host-key.sops.yaml`, commit, push, `flux reconcile` `traefik` and
   `apex`.
3. [ ] Router: WAN **2223/tcp** → `10.10.20.10:2223` (git). WAN 22 already
   points at the Pi and now reaches the portfolio.
4. [ ] Update git remotes/clients for `git.0xbf.ch` on **2223**.
5. [ ] `just release v0.1.0 "apex v0.1.0"`; wait for `:latest` and for Keel to
   roll both pods.
6. [ ] Delete `infra/` from this repo in the same commit that lands it in the
   infra repo.

## 5. Verify, then leave it alone

- [ ] `curl -sI https://0xbf.ch` → 200; the HTML contains the page content and
      `boot.js`.
- [ ] `curl -s https://0xbf.ch/ | grep -c app-shell` → 1; `curl -s
      https://0xbf.ch/contact/ | grep -c '<h1>Contact'` → 1.
- [ ] `ssh 0xbf.ch` → the portfolio; the fingerprint matches the key from
      group 4.
- [ ] `/cv`, `/age.txt`, `/pgp.txt` resolve; `/404.html` shows for a typo'd path.
- [ ] `blog.0xbf.ch` and `git.0xbf.ch` still work after the Traefik roll.
- [ ] Gatus: `apex` and `apex-ssh` green; both pods `Running`.
- [ ] Back up the host key with the other secrets; note the fingerprint in
      `infra/README.md` so a future rebuild can be compared.

## Later, not blocking

- Font payload: the app ships every specimen family. Measure `dist/` and
  consider loading a family only when the Unicode visual first needs it.
- Caching: unhashed asset names mean `no-cache`; if the wasm turns out heavy on
  revalidation, hash only the app bundle and inject its names into the page at
  build time.
- Rate limiting on 22 if the public SSH listener attracts abuse, or
  `APEX_SSH_PUBLIC=0` + `APEX_SSH_AUTHORIZED_KEYS`.
