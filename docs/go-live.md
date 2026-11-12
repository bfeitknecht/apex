# Checklist for the launch of 0xbf.ch

The groups are in order. Group 2 blocks the launch. Group 3 is the launch.
Group 4 comes afterwards. The deployment (manifests, Traefik routes, Gatus
checks and the host-key Secret) is in the `infra` repository and is integrated
there. This repository builds the image and the document root, so group 4 lists
only the steps that belong here.

## 1. What CI already checks

The restructure was written in a sandbox without access to
`static.rust-lang.org` or `crates.io`. The first real build therefore ran in the
`CI` workflow. CI runs on every push and currently passes:

- `cargo fmt --all -- --check`, `cargo clippy --all-targets --features ssh -- -D warnings`
  and `cargo test`, including the `content::` and `app::` tests;
- `cargo check --target wasm32-unknown-unknown`;
- `trunk build --release`, with assertions that the names which the page refers
  to exist (`apex.js`, `apex_bg.wasm`, `boot.js`, `app.css`, `cursor.js`) and
  that the fonts are flat in the document root, where the bare `@font-face` URLs
  of `app.css` look for them;
- `typst compile --format bundle` for the page, with assertions on the content
  of the bundle (`index.html`, `site.typ`, `contact/`, `404.html`,
  `sitemap.xml`, `robots.txt`, the emitted `age.txt`/`pgp.txt`/`cv.pdf` and the
  shell of the app).

CI therefore proves that the code compiles, that the tests pass and that both
artifacts build. CI does not check `just docroot`, the Docker stages or the
items in group 2.

## 2. Build both surfaces and look at them

- [ ] Run `just docroot` and open `public/index.html`. The file must contain the
      page (with `id="app-shell"` and without `<link data-trunk>`), not the
      Trunk stub. Both builds write into one root and the entry document of
      Trunk also has the name `index.html`.
- [ ] Run `just dev` and open `http://localhost:8080/`. The page loads and the
      console shows `/apex.js`; then the interface replaces the page. Block
      WebAssembly (`delete window.WebAssembly` in the console) and reload. The
      page must stay.
- [ ] Run `just ssg` alone. `public/index.html`, `site.typ`, `contact/`,
      `404.html`, `sitemap.xml`, `age.txt`, `pgp.txt` and `cv.pdf` must exist,
      and `grep app-shell public/index.html` must match.
- [ ] Disable JavaScript and reload. The page must not change, `/cv` must work
      and the contact page must render. There must be no `/static` and no
      redirect.
- [ ] Test touch, keyboard and selection in the app. Test lichen, Unicode,
      fractal and crash.

## 3. Content before anyone looks

- [ ] `content/projects.toml` holds one merged list of 8 entries in display
      order. It is the only copy. Both the page and the app render this file, so
      the order in the file is the order on both surfaces.
- [ ] The `gallery` destination (`https://gallery.0xbf.ch`) is linked but does
      not exist yet. Build it, or empty its `url` in `content/profile.toml`.
- [ ] `content/portrait.txt` is a cat. Replace it with the real ASCII portrait.
- [ ] Read the text in `content/profile.toml`. `lede` and `about` appear on both
      surfaces. `intro` belongs to the terminal only.

## 4. Ship

1. [ ] `@preview/thyme` is on Typst Universe. The page build and the image build
   download it, so both fail without the package.
2. [ ] Create the SSH host key and the `apex-ssh-host-key` Secret. Then
   uncomment `ssh-host-key.sops.yaml` in the `infra` repo, commit, push and run
   `flux reconcile` for `traefik` and `apex`.
3. [ ] Router: WAN **2223/tcp** → `10.10.20.10:2223` (git). WAN 22 already
   points at the Pi and now reaches the portfolio.
4. [ ] Update the git remotes and clients for `git.0xbf.ch` on **2223**.
5. [ ] Run `just release 0.1.0`. The command refuses a dirty tree and a version
   that `Cargo.toml` does not declare, then tags `v0.1.0`. The publish workflow
   builds `:latest` and `:v0.1.0`. Wait for Keel to roll both pods.

## 5. Verify, then leave it alone

- [ ] `curl -sI https://0xbf.ch` returns 200, and the HTML contains the page
      content and `boot.js`.
- [ ] `curl -s https://0xbf.ch/ | grep -c app-shell` returns 1, and `curl -s
      https://0xbf.ch/contact/ | grep -c '<h1>Contact'` returns 1.
- [ ] `ssh 0xbf.ch` shows the portfolio, and the fingerprint matches the key
      from group 4.
- [ ] `/cv`, `/age.txt` and `/pgp.txt` resolve, and a mistyped path shows
      `/404.html`.
- [ ] `blog.0xbf.ch` and `git.0xbf.ch` still work after the Traefik roll.
- [ ] Gatus shows `apex` and `apex-ssh` green, and both pods are `Running`.
- [ ] Back up the host key with the other secrets. Note its fingerprint next to
      them, so a future rebuild can be compared.

## Later, not blocking

- Font payload: the app ships every specimen family. Measure `dist/` and
  consider loading a family only when the Unicode visual first needs it.
- Caching: the asset names have no hash, so responses use `no-cache`. If the
  wasm becomes heavy on revalidation, hash only the app bundle and inject the
  names into the page at build time.
- Rate limiting on port 22 if the public SSH listener attracts abuse, or
  `APEX_SSH_PUBLIC=0` with `APEX_SSH_AUTHORIZED_KEYS`.
