# /// script
# requires-python = ">=3.12"
# dependencies = ["pillow>=11"]
# ///
"""Render every brand file from its two sources: mark.typ and palette.json.

Run with `just brand` (uv run brand/build.py). Needs `typst` on PATH.
Outputs (all in brand/, plus the two copies the site serves):
  mark.svg, mark-dark.svg            the mark, light and dark variant
  png/mark[-dark]-<size>.png         16 to 1024 px
  favicon.ico                        16, 32, 48, 64 px (light variant)
  png/wallpaper-{dark,light}.png     the NULL wallpaper (wallpaper.typ), 3024 x 1964
  palette.css                        CSS custom properties from palette.json
  ../static/assets/favicon.{svg,ico}    what 0xbf.ch serves
"""

import json
import shutil
import subprocess
from pathlib import Path

from PIL import Image

BRAND = Path(__file__).resolve().parent
FONTS = BRAND.parent / "assets"
SITE = BRAND.parent / "static" / "assets"
SIZES = [16, 32, 48, 64, 128, 180, 192, 256, 512, 1024]
ICO_SIZES = [16, 32, 48, 64]


def typst(variant: str, fmt: str, out: Path, ppi: float | None = None,
          source: str = "mark.typ") -> None:
    cmd = ["typst", "compile", "--format", fmt, "--font-path", str(FONTS),
           "--input", f"variant={variant}"]
    if ppi:
        cmd += ["--ppi", f"{ppi:g}"]
    subprocess.run([*cmd, str(BRAND / source), str(out)], check=True)


def main() -> None:
    (BRAND / "png").mkdir(exist_ok=True)
    for variant, suffix in (("light", ""), ("dark", "-dark")):
        typst(variant, "svg", BRAND / f"mark{suffix}.svg")
        for size in SIZES:
            # The page is 32 pt; 72 pt per inch.
            typst(variant, "png", BRAND / "png" / f"mark{suffix}-{size}.png",
                  ppi=size * 72 / 32)

    # 144 ppi: 1 pt = 2 px, the page is the display size in pixels.
    for variant in ("dark", "light"):
        typst(variant, "png", BRAND / "png" / f"wallpaper-{variant}.png", ppi=144,
              source="wallpaper.typ")

    big = Image.open(BRAND / "png" / "mark-256.png")
    big.save(BRAND / "favicon.ico", sizes=[(s, s) for s in ICO_SIZES])

    palette = json.loads((BRAND / "palette.json").read_text())
    lines = ["/* The 0xbf palette. Generated from palette.json by `just brand`; edit the JSON. */",
             ":root {", "  color-scheme: light dark;"]
    for role in palette["light"]:
        light, dark = palette["light"][role]["oklch"], palette["dark"][role]["oklch"]
        lines.append(f"  --{role}: light-dark({light}, {dark});")
    lines.append("}")
    (BRAND / "palette.css").write_text("\n".join(lines) + "\n")

    shutil.copyfile(BRAND / "mark.svg", SITE / "favicon.svg")
    shutil.copyfile(BRAND / "favicon.ico", SITE / "favicon.ico")


if __name__ == "__main__":
    main()
