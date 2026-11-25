# Apex: one URL, two versions and one set of content.
#
#   static   the HTML version (Typst, static/)
#   dynamic  the Ratatui program (src/), with three frontends:
#            browser (wasm, Trunk), ssh, native (your terminal)
#
#   just serve     build the document root, then serve it on :8080
#   just build     the document root only
#   just static    the static version (Typst)
#   just browser   the browser frontend of the dynamic version
#   just ssh       the SSH frontend
#   just native    the native frontend
#
# Every command this repository has lives here. Run `just` to list the recipes.

set shell := ["sh", "-cu"]

# Keep the build output in the repository (gitignored) unless the caller says
# otherwise.
export CARGO_TARGET_DIR := env("CARGO_TARGET_DIR", justfile_directory() / "target")

# List the recipes.
default:
    @just --list

# ---------------------------------------------------------------- development

# Format the whole tree. `just fmt --check` only reports (CI uses it).
fmt *flags:
    cargo fmt --all {{flags}}

# Refresh the Polyring neighbours in content/polyring.toml from the ring's embed script.
polyring:
    #!/usr/bin/env python3
    import re, urllib.request, pathlib
    js = urllib.request.urlopen("https://polyring.ch/embed.js").read().decode()
    urls = re.findall(r'url:"([^"]+)"', js)
    me = next(i for i, u in enumerate(urls) if u.rstrip("/") == "https://0xbf.ch")
    path = pathlib.Path("content/polyring.toml")
    text = path.read_text()
    text = re.sub(r'^previous = ".*"$', f'previous = "{urls[me - 1]}"', text, flags=re.M)
    text = re.sub(r'^next = ".*"$', f'next = "{urls[(me + 1) % len(urls)]}"', text, flags=re.M)
    path.write_text(text)
    print(f"previous {urls[me - 1]}, next {urls[(me + 1) % len(urls)]} ({len(urls)} members)")

# Lints, tests and the wasm32 build. CI runs the same commands.
check:
    cargo clippy --all-targets --features ssh
    cargo test --features ssh
    cargo check --target wasm32-unknown-unknown

# The dynamic version in this terminal (native frontend).
native *args:
    cargo run --release -- {{args}}

# The dynamic version over SSH on 0.0.0.0:2222. It needs APEX_SSH_HOST_KEY (see README.md).
ssh:
    cargo run --release --features ssh --bin ssh

# ----------------------------------------------------------------------- site

# Build the static version and its plain files into ./public, the document root.
static:
    #!/usr/bin/env sh
    set -eu
    mkdir -p public
    typst compile --features html,bundle --format bundle --root . static/site.typ public

# Build the browser frontend of the dynamic version into ./dist (`--release` for production).
browser *args:
    trunk build --dist dist --public-url / {{args}}

# Build the document root: the static version, then the browser frontend over it.
build: static
    #!/usr/bin/env sh
    set -eu
    just browser --release
    # Trunk renames its entry document to `index.html`. Remove it from `dist`
    # first, so the static version stays the document and the dynamic version
    # stays a set of assets.
    rm -f dist/index.html dist/dynamic.html
    cp -R dist/. public/

# --------------------------------------------------------------------- assets

# Render brand/ (mark SVGs and PNGs, favicon.ico, palette.css) from
# brand/mark.typ and brand/palette.json; copies the favicon into static/assets.
brand:
    # The mark uses CommitMono, which is in `assets/`. Both versions share the
    # favicon: the boot script swaps the static version for the dynamic one in
    # place, so the document keeps its icon.
    uv run brand/build.py

# Rebuild assets/{unifont,noto}-cmap.bin from a Unicode 17 UnicodeData.txt.
unicode data:
    #!/usr/bin/env python3
    # Needs fontTools. src/visuals/unicode.rs embeds the two files at compile
    # time.
    #   just unicode ~/Downloads/UnicodeData.txt
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
    for line in Path("""{{data}}""").read_text(encoding='utf-8').splitlines():
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

    # Record the preferred outline fonts, BabelStone Han included. Unifont is the
    # fallback. The browser frontend reads the numeric font choice.
    font_names = ['NotoSans-Regular.ttf', 'NotoSansSymbols-Regular.ttf', 'NotoSansSymbols2-Regular.ttf', 'NotoSansMath-Regular.ttf'] + extra_fonts + ['BabelStoneHan.ttf']
    choice = {}
    for index, filename in enumerate(font_names, 1):
        for cp in TTFont(root / 'assets' / filename).getBestCmap():
            if cp in covered and cp in assigned and not 0xfff0 <= cp <= 0xffff:
                choice.setdefault(cp, index)
    (root / 'assets/noto-cmap.bin').write_bytes(b''.join(struct.pack('<IB',cp,i) for cp,i in sorted(choice.items())))
    print(f'{len(choice)} have preferred outline fonts; {sum(i == len(font_names) for i in choice.values())} use BabelStone Han')

# `--static`: the static version only (the live site shows it with `?static`).
# `--dynamic`: rebuild only the browser frontend over the existing static
# version, the quick loop for work on the dynamic version. No flag: both.
# PORT=8090 for another port.
# Build the document root and serve it with the shipped sws.toml on :8080 (`--static`, `--dynamic`).
serve *flags:
    #!/usr/bin/env sh
    set -eu
    mode=both
    for flag in {{flags}}; do
        case "$flag" in
            --static) mode=static ;;
            --dynamic) mode=dynamic ;;
            *) echo "serve: unknown flag '$flag' (known: --static, --dynamic)" >&2; exit 1 ;;
        esac
    done
    case "$mode" in
        # Wipe first: a boot script or wasm left from an earlier build would
        # upgrade the static version to the dynamic one.
        static) rm -rf public; just static ;;
        dynamic)
            [ -f public/index.html ] || just static
            just browser --release
            rm -f dist/index.html dist/dynamic.html
            cp -R dist/. public/ ;;
        both) just build ;;
    esac
    # The `/cv` rewrite, the 404 page and the headers are the ones that ship.
    # PORT picks the local port (default 8080): `PORT=8090 just serve --static`.
    # IPv4 loopback only: Docker on macOS publishes [::] but does not pass IPv6
    # to the container, so `localhost` (::1 first) would hang. Bound to
    # 127.0.0.1, ::1 is refused at once and the browser falls back to IPv4.
    port="${PORT:-8080}"
    echo "serving on http://127.0.0.1:${port}/"
    docker run --rm -p "127.0.0.1:${port}:8080" \
        -v "{{justfile_directory()}}/public:/public" \
        -v "{{justfile_directory()}}/sws.toml:/sws.toml:ro" \
        -e SERVER_HOST=0.0.0.0 -e SERVER_PORT=8080 -e SERVER_ROOT=/public \
        -e SERVER_ERROR_PAGE_404=./404.html -e SERVER_CONFIG_FILE=/sws.toml \
        -e SERVER_CACHE_CONTROL_HEADERS=false -e SERVER_LOG_LEVEL=info \
        joseluisq/static-web-server:2.44.1-alpine

# --------------------------------------------------------------- container, release

# Build the container image. `--run` also starts it on :8080 and :2222.
image *args:
    #!/usr/bin/env sh
    set -eu
    docker build -t ghcr.io/bfeitknecht/apex:dev .
    case " {{args}} " in
        *" --run "*)
            docker run --rm -p 127.0.0.1:8080:8080 -p 127.0.0.1:2222:2222 \
                -e APEX_SSH_HOST_KEY=/run/secrets/apex_ssh_host_key \
                -v "${PWD}/.private/ssh_host_ed25519_key:/run/secrets/apex_ssh_host_key:ro" \
                ghcr.io/bfeitknecht/apex:dev
            ;;
    esac

# Cut a release: `just release v0.1.0`, `just release v0.1.0 --check`, `just release --notes`
release *args:
    #!/usr/bin/env python3
    import pathlib, re, subprocess, sys, tomllib

    version = tomllib.loads(pathlib.Path("Cargo.toml").read_text())["package"]["version"]
    changelog = pathlib.Path("CHANGELOG.md").read_text()

    flags = [a for a in "{{args}}".split() if a.startswith("--")]
    positional = [a for a in "{{args}}".split() if not a.startswith("--")]
    unknown = [f for f in flags if f not in ("--check", "--notes")]
    if unknown:
        sys.exit(f"release: unknown flag {unknown[0]} (expected --check or --notes)")
    if len(flags) > 1:
        sys.exit(f"release: --check and --notes are mutually exclusive (got {flags})")
    if len(positional) > 1:
        sys.exit(f"release: expected at most one argument (the tag), got {positional}")
    # The tag comes from Cargo.toml. An explicit tag is optional and must match.
    tag = positional[0] if positional else f"v{version}"

    # The release notes are this section of CHANGELOG.md. `--notes` prints them
    # for the GitHub release, and the tag carries them.
    match = re.search(
        rf"^## \[{re.escape(version)}\][^\n]*\n(.*?)(?=^## \[|^\[[^\]]+\]:|\Z)",
        changelog,
        re.M | re.S,
    )
    notes = match.group(1).strip() if match else ""

    if "--notes" in flags:
        if not notes:
            sys.exit(f"CHANGELOG.md has no '## [{version}]' section to publish")
        print(notes)
        sys.exit(0)

    problems = []
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        problems.append(f"Cargo.toml version {version!r} is not plain X.Y.Z")
    if tag != f"v{version}":
        problems.append(
            f"tag {tag!r} does not match the Cargo.toml version (want 'v{version}')"
        )
    if not re.search(rf"^## \[{re.escape(version)}\] - \d\d\d\d-\d\d-\d\d", changelog, re.M):
        problems.append(f"CHANGELOG.md needs a dated '## [{version}] - YYYY-MM-DD' section")
    elif not notes:
        problems.append(f"'## [{version}]' in CHANGELOG.md is empty")
    if "--check" not in flags:
        if subprocess.run(["git", "check-ref-format", f"refs/tags/{tag}"], capture_output=True).returncode:
            problems.append(f"{tag!r} is not a valid git tag")
        elif not subprocess.run(["git", "rev-parse", "-q", "--verify", f"refs/tags/{tag}"], capture_output=True).returncode:
            problems.append(f"tag {tag} already exists")
        if subprocess.run(["git", "remote", "get-url", "origin"], capture_output=True).returncode:
            problems.append("no 'origin' remote configured")
        if subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True).stdout.strip():
            problems.append("the working tree is not clean; commit first")

        # A tag on a commit that origin does not have produces a release whose
        # workflow runs against a different tree.
        branch = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"], capture_output=True, text=True,
        ).stdout.strip()
        head = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
        remote = subprocess.run(
            ["git", "rev-parse", f"origin/{branch}"], capture_output=True, text=True,
        )
        if remote.returncode:
            problems.append(f"origin has no branch {branch!r}; push it first")
        elif remote.stdout.strip() != head:
            problems.append(f"HEAD is not what origin has for {branch!r}; push or pull first")

    for problem in problems:
        print(f"error: {problem}", file=sys.stderr)
    if problems:
        sys.exit(1)
    if "--check" in flags:
        print(f"{tag} is ready: Cargo.toml, CHANGELOG.md and the tree agree")
        sys.exit(0)

    # The same gate that CI runs, so the tag cannot carry a tree that does not build.
    subprocess.run(["just", "check"], check=True)

    subprocess.run(["git", "tag", "-a", tag, "-m", notes or tag], check=True)
    subprocess.run(["git", "push", "origin", tag], check=True)
    print(f"pushed {tag}; the workflows build the image and publish the release")
