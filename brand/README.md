# brand

The 0xbf mark and palette, in one place. Two sources, everything else is
generated: edit `mark.typ` or `palette.json`, then run `just brand`.

| File | What |
| --- | --- |
| `mark.typ` | the mark: `^@` on a disc, CommitMono 19pt on a 32pt page. `--input variant=dark` inverts it |
| `wallpaper.typ` | the NULL wallpaper: `^@` on a terminal grid, the menu bar row left empty. Inputs `variant`, `width`, `height`, `band`, `pad`, `cell`, `line` (defaults: MacBook Pro 14", Ghostty grid of CommitMono 12 pt) |
| `palette.json` | the palette, light and dark, as `oklch` and hex |
| `build.py` | renders the rest (`just brand` = `uv run brand/build.py`, needs `typst`) |
| `mark.svg` | black disc, white glyphs: light backgrounds, and the favicon everywhere |
| `mark-dark.svg` | white disc, black glyphs: dark backgrounds |
| `png/mark-<px>.png`, `png/mark-dark-<px>.png` | 16, 32, 48, 64, 128, 180 (Apple touch), 192, 256, 512, 1024 px, transparent corners |
| `png/wallpaper-dark.png`, `png/wallpaper-light.png` | 3024 x 1964 wallpapers; omni sets them on anti |
| `favicon.ico` | 16, 32, 48, 64 px, light variant |
| `palette.css` | the palette as `light-dark()` custom properties |

`just brand` also copies `mark.svg` and `favicon.ico` to `static/assets/favicon.{svg,ico}`,
which 0xbf.ch serves at `/assets/favicon.svg`.

## Palette

One grey scale, `oklch(L 0 0)`, no hue.

| Role | Light | Dark | Use |
| --- | --- | --- | --- |
| `bg` | `oklch(96% 0 0)` `#f2f2f2` | `oklch(17% 0 0)` `#0f0f0f` | page |
| `fg` | `oklch(21% 0 0)` `#181818` | `oklch(95% 0 0)` `#eeeeee` | text |
| `muted` | `oklch(47% 0 0)` `#5b5b5b` | `oklch(70% 0 0)` `#9e9e9e` | secondary text, rules |
| `surface` | `oklch(92% 0 0)` `#e4e4e4` | `oklch(27% 0 0)` `#262626` | raised blocks, code |

The blog (thyme's default theme), the gallery, the Forgejo theme `0xbf` and the
authentik brand CSS use these values. 0xbf.ch does not: its static version has
no CSS (the browser's own styles), and its dynamic version stays black and
white on purpose: it is a terminal.

## Copies elsewhere

These are copies, not links. After a change, copy again:

| Where | What | How |
| --- | --- | --- |
| authentik | logo and favicon | none needed: it loads `https://0xbf.ch/assets/favicon.svg` |
| glob | `favicon.svg`, `favicon.ico` | copy `mark.svg`, `favicon.ico` |
| exhib | `static/favicon.svg`, `favicon.ico`, `apple-touch-icon.png` | copy `mark.svg`, `favicon.ico`, `png/mark-180.png` |
| infra Forgejo | `branding/logo.svg`, `branding/favicon.svg` | copy `mark.svg` |
| infra Forgejo | `branding-png.yaml` (`logo.png` 512, `favicon.png` and `apple-touch-icon.png` 180) | base64 of `png/mark-512.png`, `png/mark-180.png` |
| infra Jellyfin | `jellyfin-branding/branding.xml` | base64 of `mark.svg` in a CSS `data:` URL |
| infra Jellyfin | `jellyfin-branding/favicon.svg`, `favicon.ico`, `apple-touch-icon.png`, `icon-192.png`, `icon-512.png` | copy `mark.svg`, `favicon.ico`, `png/mark-180.png`, `png/mark-192.png`, `png/mark-512.png` |
| infra Immich | `immich/branding/` (same five files) | same as Jellyfin |
| infra copyparty | favicon | none needed: it loads `https://0xbf.ch/assets/favicon.svg` |
