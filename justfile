# Apex: one URL, two surfaces, one definition of the content.
#
#   just dev      build the document root, then serve it on :8080
#   just native   the terminal
#   just ssh      the SSH server (russh)
#   just app      the browser app (Trunk/Ratzilla)
#   just ssg      the page (Typst/thyme)
#
# Every command this repository has lives here. Run `just` to list the recipes.

set shell := ["sh", "-cu"]

# Keep build artifacts in the repo (gitignored) unless the caller says otherwise.
export CARGO_TARGET_DIR := env("CARGO_TARGET_DIR", justfile_directory() / "target")

# Local Typst package cache resolving the vendored @preview packages.
packages_dir := justfile_directory() / ".packages"

default:
    @just --list

# ---------------------------------------------------------------- development

# Format the whole tree.
fmt:
    cargo fmt --all

# Lints, tests and the wasm32 build (what CI runs).
check:
    cargo clippy --all-targets --features ssh
    cargo test --features ssh
    cargo check --target wasm32-unknown-unknown

# Native terminal portfolio.
native *args:
    cargo run --release -- {{args}}

# SSH server on 0.0.0.0:2222; needs APEX_SSH_HOST_KEY (see README.md).
ssh:
    cargo run --release --features ssh --bin ssh

# ----------------------------------------------------------------------- page

# Link the vendored Typst packages into a throwaway package path.
packages:
    #!/usr/bin/env sh
    # thyme's blog preset also needs cheq/showybox/tablem/zebraw/marge from
    # Typst Universe; the first build fetches and caches them.
    set -eu
    rm -rf "{{packages_dir}}"
    mkdir -p "{{packages_dir}}/preview/exemel" "{{packages_dir}}/preview/thyme"
    cp -R ssg/vendor/exemel "{{packages_dir}}/preview/exemel/0.3.0"
    cp -R ssg/vendor/thyme "{{packages_dir}}/preview/thyme/0.1.0"

# The page and its plain artifacts into ./public (this is the document root).
ssg: packages
    #!/usr/bin/env sh
    set -eu
    mkdir -p public
    typst compile --features html,bundle --format bundle \
        --package-path "{{packages_dir}}" --root . ssg/site.typ public

# ---------------------------------------------------------------- browser app

# The browser app into ./dist (assets only; the page is what serves it).
app:
    trunk build --dist dist --public-url /

# The browser app, production build.
app-release:
    trunk build --release --dist dist --public-url /

# Assemble the document root exactly as the container does.
docroot: ssg app-release
    #!/usr/bin/env sh
    set -eu
    # Trunk renames its entry document to `index.html`; dropping it from `dist`
    # first is what keeps the page the document, and the app a set of assets.
    rm -f dist/index.html dist/app.html
    cp -R dist/. public/

# ------------------------------------------------------------- content, fonts

# Rebuild assets/{unifont,noto}-cmap.bin from a Unicode 17 UnicodeData.txt.
cmaps udata:
    #!/usr/bin/env python3
    # needs fontTools; src/visuals/unicode.rs embeds the two files at compile time
    #   just cmaps ~/Downloads/UnicodeData.txt
    import struct
    from pathlib import Path

    from fontTools.ttLib import TTFont

    root = Path("""{{justfile_directory()}}""")
    low = TTFont(root / 'assets/Unifont-17.0.05.otf').getBestCmap()
    high = TTFont(root / 'assets/UnifontUpper-17.0.05.otf').getBestCmap()
    unifont = {p for p in low if p <= 0xffff} | {p for p in high if p >= 0x10000}
    han = set(TTFont(root / 'assets/BabelStoneHan.ttf').getBestCmap())
    extra_fonts = ['NotoSansArabic-Regular.ttf', 'NotoSansArmenian-Regular.ttf', 'NotoSansBamum-Regular.ttf', 'NotoSansBengali-Regular.ttf', 'NotoSansCanadianAboriginal-Regular.ttf', 'NotoSansCherokee-Regular.ttf', 'NotoSansCuneiform-Regular.ttf', 'NotoSansDevanagari-Regular.ttf', 'NotoSansEgyptianHieroglyphs-Regular.ttf', 'NotoSansEthiopic-Regular.ttf', 'NotoSansGeorgian-Regular.ttf', 'NotoSansGlagolitic-Regular.ttf', 'NotoSansGothic-Regular.ttf', 'NotoSansGujarati-Regular.ttf', 'NotoSansGurmukhi-Regular.ttf', 'NotoSansHebrew-Regular.ttf', 'NotoSansKhmer-Regular.ttf', 'NotoSansLao-Regular.ttf', 'NotoSansLinearB-Regular.ttf', 'NotoSansMalayalam-Regular.ttf', 'NotoSansMongolian-Regular.ttf', 'NotoSansMyanmar-Regular.ttf', 'NotoSansOldItalic-Regular.ttf', 'NotoSansOldPersian-Regular.ttf', 'NotoSansOsmanya-Regular.ttf', 'NotoSansRunic-Regular.ttf', 'NotoSansSinhala-Regular.ttf', 'NotoSansSyriac-Regular.ttf', 'NotoSansTamil-Regular.ttf', 'NotoSansThaana-Regular.ttf', 'NotoSansThai-Regular.ttf', 'NotoSansUgaritic-Regular.ttf', 'NotoSansYi-Regular.ttf']
    extra_cmaps = [TTFont(root / 'assets' / filename).getBestCmap() for filename in extra_fonts]
    covered = unifont | han | set().union(*(set(cmap) for cmap in extra_cmaps))
    excluded = {'Cc','Cf','Cs','Co','Cn','Zs','Zl','Zp','Mn','Mc','Me'}
    assigned = set()
    start = None
    for line in Path("""{{udata}}""").read_text(encoding='utf-8').splitlines():
        fields = line.split(';'); cp = int(fields[0], 16); name = fields[1]; category = fields[2]
        if name.endswith(', First>'):
            start = (cp, category)
        elif name.endswith(', Last>'):
            if start and start[1] not in excluded:
                assigned.update(range(start[0], cp + 1))
            start = None
        elif category not in excluded:
            assigned.add(cp)
    points = sorted(p for p in (covered & assigned) if not 0xfff0 <= p <= 0xffff)
    (root / 'assets/unifont-cmap.bin').write_bytes(b''.join(struct.pack('<I',p) for p in points))
    print(f'{len(points)} covered assigned standalone code points')

    # Record preferred outline fonts, including BabelStone Han; Unifont is fallback.
    # The numeric font choice is read by the browser frontend.
    font_names = ['NotoSans-Regular.ttf', 'NotoSansSymbols-Regular.ttf', 'NotoSansSymbols2-Regular.ttf', 'NotoSansMath-Regular.ttf'] + extra_fonts + ['BabelStoneHan.ttf']
    choice = {}
    for index, filename in enumerate(font_names, 1):
        for cp in TTFont(root / 'assets' / filename).getBestCmap():
            if cp in covered and cp in assigned and not 0xfff0 <= cp <= 0xffff:
                choice.setdefault(cp, index)
    (root / 'assets/noto-cmap.bin').write_bytes(b''.join(struct.pack('<IB',cp,i) for cp,i in sorted(choice.items())))
    print(f'{len(choice)} have preferred outline fonts; {sum(i == len(font_names) for i in choice.values())} use BabelStone Han')

# -------------------------------------------------------------------- serving

# Serve ./public with the container's own `sws` and `sws.toml`.
serve:
    # `/cv` rewrites, the 404 page and the headers are the ones that ship.
    # Rebuild in another shell (`just ssg`, `just app-release`) and reload.
    docker run --rm -p 8080:8080 \
        -v "{{justfile_directory()}}/public:/public" \
        -v "{{justfile_directory()}}/sws.toml:/sws.toml:ro" \
        -e SERVER_HOST=0.0.0.0 -e SERVER_PORT=8080 -e SERVER_ROOT=/public \
        -e SERVER_ERROR_PAGE_404=./404.html -e SERVER_CONFIG_FILE=/sws.toml \
        -e SERVER_CACHE_CONTROL_HEADERS=false -e SERVER_LOG_LEVEL=info \
        joseluisq/static-web-server:2.44.1-alpine

# Build the document root, then serve it on :8080.
dev: docroot serve

# --------------------------------------------------------------- container, release

# Container image: one document root plus the SSH binary.
image:
    docker build -t ghcr.io/bfeitknecht/apex:dev .

# Run the image on :8080 and :2222 with the local host key.
image-run:
    docker run --rm -p 8080:8080 -p 2222:2222 \
        -e APEX_SSH_HOST_KEY=/run/secrets/apex_ssh_host_key \
        -v "${PWD}/.private/ssh_host_ed25519_key:/run/secrets/apex_ssh_host_key:ro" \
        ghcr.io/bfeitknecht/apex:dev

# Tag a release; the tag drives the image publish workflow.
release tag message:
    #!/usr/bin/env sh
    set -eu
    tag="{{tag}}"
    git check-ref-format "refs/tags/$tag"
    ! git rev-parse -q --verify "refs/tags/$tag"
    git tag -a "$tag" -m "{{message}}"
    git push origin "$tag"
    echo "pushed $tag"
