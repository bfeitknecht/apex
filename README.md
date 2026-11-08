# Apex

A monochrome, keyboard-first portfolio built with Rust and Ratatui. Native/SSH and browser use the same portfolio and navigation state. The browser uses Ratzilla for the interface. A hidden visuals menu provides access to Unicode, Lichen and an ASCII fractal explorer. The browser has the full Unicode study; terminals use a small glyph sampler.

## Site plan

Apex is the portfolio at `0xbf.ch`: a Rust/Ratatui interface served both through the built-in SSH service and in the browser using Ratzilla. It presents an introduction, selected projects, contact details and links to the wider personal site. It is not the blog or gallery itself.

The parenthesized navigation links point to separate destinations: the CV at `https://0xbf.ch/cv`, the future blog at `https://blog.0xbf.ch`, and a future gallery at `https://gallery.0xbf.ch`. The gallery is intended to be a separate website built next; its design, content, hosting and implementation are not yet decided. The planned blog is represented by the `glob` project, built with the planned Typst-based static site generator `thyme`. These links are destinations, not a claim that the services are already deployed.

A no-JavaScript (`<noscript>`) section is planned for Apex: a readable, keyboard-accessible HTML fallback with the essential introduction, projects, contact information, and links, so visitors can reach the content without running WebAssembly or JavaScript. It should follow the same content and destinations as the TUI, remain usable on small screens, and not be presented as an implemented fallback until built and tested.

## Run

```sh
./scripts/native.sh
./scripts/web.sh
```

The browser build needs Trunk and the `wasm32-unknown-unknown` Rust target. Build output is stored under `/tmp`. For SSH, use the built-in russh server described below; `deploy/sshd_config.example` provides an alternative using an external SSH server.

## Your content

Edit `src/portfolio.rs` to change the introduction, about text, project list and destinations. Empty URLs are non-opening placeholders: `macho` is awaiting its renamed repository, and `thyme`, `glob`, and `tangled` need their public URLs. Update the CV URL if its destination changes.

The footer uses bundled public keys in `assets/age.txt` and `assets/pgp.txt`. Update these files when the published keys change; the browser and TUI both use these copies. Replace `assets/placeholder-me.svg` and `ME_ART` in `src/portfolio.rs` with your image and ASCII art when ready.

## Navigation

The browser follows the system light or dark appearance automatically. Native and SSH sessions use the terminal foreground and background. The portfolio is one scrollable page. The fixed navigation jumps to sections. The footer appears at the bottom.

- **J / K**, up/down, wheel or swipe: scroll. **gg / G** and Home/End: top / bottom.
- **H / L**, left/right or **1–3**: jump between about, projects and contact. Click `^@` or use `gg` to return to the top. Triple-click `^@`, or focus it with Tab and press Enter three times, to open the hidden visuals menu. Use `h`/`l`, `j`/`k` or Tab and Enter to select a visual; Escape returns.
- **Tab / Shift-Tab** or **N / Shift-N**: focus the next / previous link, including CV, blog and gallery in the top bar. **Enter** opens it in the browser; in a terminal the focused URL is displayed beside the item when space allows. Footer keys and `me` open within the terminal; use `j`/`k` to scroll and Escape to return.
- Mouse and touch activate links and navigation; links highlight on mouse hover. Drag browser UI text to select and copy it.
- **?** toggles Help; **Esc** closes it; **Q** quits the terminal.

## Hidden visuals

The Unicode and Lichen studies are available from the hidden menu. The third visual is a fullscreen ASCII quaternion Julia explorer: an animated two-dimensional slice through a four-dimensional fractal, with orbit-trap detail and a slow zoom. Use `h`/`j`/`k`/`l` or arrow keys to pan, `+` and `-` to zoom, `r` or `0` to reset, and Escape to return. The renderer works in the browser, native terminal and SSH session; character proportions depend on the font and terminal.

## Unicode study

The browser study randomly samples Unicode blocks, then a supported code point within the chosen block. Each specimen shows its Unicode 17 name, code point, block, font, and two position rulers. Previous specimens remain in the browsing history. It is an independent character explorer, not a copy of the referenced video.

The 106,701 candidate code points are taken from the bundled font cmaps and Unicode 17 data, excluding control, formatting, whitespace, combining, private-use and unassigned characters. The explorer has outline fonts for 79,761 of these code points, including 33 additional script-specific Noto Sans fonts, Noto Symbols/Math and BabelStone Han. GNU Unifont is the fallback for the rest. Its bitmap-derived glyphs are naturally pixelated at large sizes. This is **not every Unicode glyph**: some assigned characters lack coverage, some characters need contextual shaping, and individual font glyphs may differ. Unicode names and block ranges come from `unicode_names2` and `unicode-blocks`. Font licenses are in `assets/`.

Specimen fonts load on demand as their glyphs are encountered; the previous specimen stays visible until the next font is ready. The portfolio interface remains in Commit Mono. BabelStone Han is provided as WOFF2 with the original TTF as a compatibility fallback, which is substantially larger. BabelStone Han is distributed under the Arphic Public License (included alongside the fonts). The study places compact metadata and rulers beneath the specimen on desktop and mobile. Ratzilla draws the UI and metadata, while browser Canvas 2D draws the enlarged specimen; it does not scale a single Ratatui cell into a fullscreen glyph. Rebuild the catalogue with `scripts/unicode-candidates.py` and Unicode 17 `UnicodeData.txt` (requires Python fontTools).

## Code

`src/app.rs` owns shared navigation, `src/portfolio.rs` contains editable content, `src/ui.rs` draws the shared TUI, `src/lichen.rs` simulates Lichen, `src/unicode_study.rs` supplies the browser specimen catalogue, and `src/frontends/` contains browser and native adapters. The inactive non-destructive Crash study remains behind `--features crash-study`.

Browser Lichen uses dynamically sized square pixel cells. Native rendering uses Unicode half-blocks; their physical proportions depend on terminal font and cell geometry. Neither is the sixel protocol, and the simulation is not a verified pixel-perfect reconstruction of the original.

```sh
CARGO_TARGET_DIR=/tmp/payload-target cargo test --features crash-study
CARGO_TARGET_DIR=/tmp/payload-target cargo check --target wasm32-unknown-unknown
```

## Public SSH portfolio (russh)

The built-in SSH server serves the same Rust/Ratatui interface without a user account or client key. **By default it accepts any SSH username and listens on `0.0.0.0:2222`.** It runs the portfolio only: no system shell, commands, SFTP, or port forwarding. A PTY is required. Each visitor has independent navigation and Lichen state; `q` disconnects.

On the machine that will host the service, create a persistent host identity outside the project. Never commit or share its private key. Back it up securely: replacing it changes the server identity and causes SSH host-key warnings.

```sh
mkdir -p ~/.config/apex
chmod 700 ~/.config/apex
ssh-keygen -t ed25519 -N '' -f ~/.config/apex/ssh_host_ed25519_key
APEX_SSH_HOST_KEY="$HOME/.config/apex/ssh_host_ed25519_key" ./scripts/ssh.sh
```

From a client, assuming the host and firewall allow TCP port 2222:

```sh
ssh -p 2222 your-domain.example
```

The server prints its SHA-256 host-key fingerprint at startup; verify it through a trusted channel when connecting for the first time. To use `ssh your-domain.example` without a port argument, arrange for port 22 to reach the service—for example via a firewall redirect or a properly configured TCP proxy. Avoid binding a privileged port as root. Do not displace an existing administrative SSH service; use a separate host or address, and keep an independent admin access path.

Public access is intentional here. Run as a dedicated non-root user with a service manager, firewall, connection/rate limits and monitoring. The built-in server is an application service, not a hardened general-purpose SSH server. Inactive connections time out after 15 minutes; there is a 32-channel limit and terminal dimensions are capped at 240×120. These do not replace network-level protections.

### Optional key-only mode

Set `APEX_SSH_PUBLIC=0` and point `APEX_SSH_AUTHORIZED_KEYS` at a file containing one plain OpenSSH public key per line. The default username is `apex`, configurable with `APEX_SSH_USER`. Blank lines and comments are accepted; per-key `authorized_keys` options are not supported. If an allowlist is specified without explicitly choosing public or key-only mode, the server refuses to start to prevent an accidental public service.

```sh
APEX_SSH_PUBLIC=0 \
APEX_SSH_HOST_KEY="$HOME/.config/apex/ssh_host_ed25519_key" \
APEX_SSH_AUTHORIZED_KEYS="$HOME/.config/apex/authorized_keys" \
APEX_SSH_LISTEN=127.0.0.1:2222 \
./scripts/ssh.sh
```

`APEX_SSH_LISTEN` overrides the bind address. The built-in server still needs an actual reachable host, DNS and network configuration for Internet access. Arena's browser preview does not provide a public SSH endpoint.

In Lichen, `-` slows generation, `+` speeds it up, and `0` restores the default speed. Space pauses; `r` resets the simulation. Scrolling changes pixelation independently.
