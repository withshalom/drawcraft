#!/usr/bin/env python3
"""Draw DRAWcraft's logo: the app icon beside the wordmark "DRAWcraft", DRAW in capitals.

Original artwork made for DRAWcraft (MIT OR Apache-2.0). The wordmark is set in Source Sans 3 (the app's
own interface face, SIL Open Font License 1.1, assets/fonts/OFL-SourceSans3.txt), Bold for DRAW and Regular
for craft, and converted to outlines, so the logo needs no font to show. The icon is
assets/app-icon/vectorcraft.svg (packaging/knight-icon.py).

    python3 packaging/drawcraft-logo.py

writes docs/images/drawcraft-logo.svg (ink type, for light backgrounds) and drawcraft-logo-dark.svg (paper
type, for dark ones). Needs fontTools (pip install fonttools).
"""

import os
import re
import sys

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
INK = "#0b0b0c"
PAPER = "#efe9dc"
FIELD = "#2d5c9e"

ICON = 360  # the icon's side in the logo
GAP = 48  # icon to wordmark
CAP = 170  # cap height of the wordmark


def word(path, text, x, baseline, cap_height):
    """`text` in the font at `path` as one SVG path: (d, advance, scale)."""
    font = TTFont(path)
    glyphs = font.getGlyphSet()
    cmap = font.getBestCmap()
    cap = getattr(font["OS/2"], "sCapHeight", 0) or font["head"].unitsPerEm * 0.66
    k = cap_height / cap
    pen = SVGPathPen(glyphs, lambda v: f"{v:.1f}".rstrip("0").rstrip("."))
    cursor = 0.0
    prev = None
    kern = {}
    if "kern" in font:
        for t in font["kern"].kernTables:
            kern.update(t.kernTable)
    for ch in text:
        name = cmap.get(ord(ch))
        if name is None:
            continue
        cursor += kern.get((prev, name), 0) if prev else 0
        glyphs[name].draw(TransformPen(pen, (k, 0, 0, -k, x + cursor * k, baseline)))
        cursor += glyphs[name].width
        prev = name
    return pen.getCommands(), cursor * k


def logo(type_colour):
    icon = open(os.path.join(ROOT, "assets/app-icon/vectorcraft.svg"), encoding="utf-8").read()
    # The icon as a nested picture, its clip id made unique to the logo.
    inner = re.sub(r'id="t"', 'id="icon-tile"', icon)
    inner = inner.replace('url(#t)', 'url(#icon-tile)')
    inner = inner.replace('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">', f'<svg x="0" y="0" width="{ICON}" height="{ICON}" viewBox="0 0 512 512">', 1)
    baseline = ICON / 2 + CAP / 2
    x = ICON + GAP
    bold = os.path.join(ROOT, "assets/fonts/SourceSans3-Bold.ttf")
    regular = os.path.join(ROOT, "assets/fonts/SourceSans3-Regular.ttf")
    d_draw, w_draw = word(bold, "DRAW", x, baseline, CAP)
    d_craft, w_craft = word(regular, "craft", x + w_draw + CAP * 0.04, baseline, CAP)
    width = int(x + w_draw + CAP * 0.04 + w_craft + 8)
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {ICON}" width="{width}" height="{ICON}">'
        f"<title>DRAWcraft</title>{inner.strip()}"
        f'<path d="{d_draw}" fill="{type_colour}"/><path d="{d_craft}" fill="{type_colour}"/></svg>\n'
    )


def main():
    out = os.path.join(ROOT, "docs/images")
    for name, colour in (("drawcraft-logo.svg", INK), ("drawcraft-logo-dark.svg", PAPER)):
        with open(os.path.join(out, name), "w", encoding="utf-8") as f:
            f.write(logo(colour))
    sys.stdout.write("logos written to docs/images\n")


if __name__ == "__main__":
    main()
