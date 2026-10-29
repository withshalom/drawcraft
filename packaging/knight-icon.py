#!/usr/bin/env python3
"""Draw DRAWcraft's app icon: an engraved knight's helm on a full-bleed tile.

Original artwork made for DRAWcraft (MIT OR Apache-2.0), generated in code so its provenance is this
file. It follows the Crafting Apps icon template (assets/app-icon/README.md): a head-and-shoulders
portrait in three colours (ink, paper, the app's colour), a 512 viewBox and a rounded tile (rx 112).
The shading is engraving: lines that swell from a highlight into the shadow, never filled tones.

    python3 packaging/knight-icon.py > assets/app-icon/vectorcraft.svg

Only the standard library; the output is deterministic.
"""

import math
import sys

INK = "#0b0b0c"
PAPER = "#efe9dc"
FIELD = "#2d5c9e"  # DRAWcraft's colour: heraldic azure

out = []
defs = []
_ids = [0]


def nid(prefix):
    _ids[0] += 1
    return f"{prefix}{_ids[0]}"


def f(v):
    s = f"{v:.1f}"
    return s[:-2] if s.endswith(".0") else s


def clip(d):
    i = nid("c")
    defs.append(f'<clipPath id="{i}"><path d="{d}"/></clipPath>')
    return i


def shape(d, stroke=5.0, fill=PAPER):
    out.append(f'<path d="{d}" fill="{fill}" stroke="{INK}" stroke-width="{f(stroke)}" stroke-linejoin="round"/>')


def ink(d):
    out.append(f'<path d="{d}" fill="{INK}"/>')


def line(d, w=2.0):
    out.append(f'<path d="{d}" fill="none" stroke="{INK}" stroke-width="{f(w)}" stroke-linecap="round" stroke-linejoin="round"/>')


def poly(pts):
    return "M" + "L".join(f"{f(x)} {f(y)}" for x, y in pts)


def grouped(inside, items):
    """Draw (d, width) items clipped to `inside`, merging equal widths into one path."""
    by_w = {}
    for d, w in items:
        if w >= 0.35:
            by_w.setdefault(round(w * 4) / 4, []).append(d)
    if not by_w:
        return
    c = clip(inside)
    body = "".join(
        f'<path d="{"".join(ds)}" fill="none" stroke="{INK}" stroke-width="{f(w)}" stroke-linecap="round"/>'
        for w, ds in sorted(by_w.items())
    )
    out.append(f'<g clip-path="url(#{c})">{body}</g>')


def swell_rings(inside, cx, cy, r0, r1, gap, w_max, power=1.4):
    """Concentric lines round a highlight at (cx, cy): absent within r0, swelling to w_max at r1."""
    items = []
    r = r0
    while r <= r1 + gap * 3:
        t = min(1.0, (r - r0) / (r1 - r0))
        w = w_max * t**power
        items.append((f"M{f(cx - r)} {f(cy)}a{f(r)} {f(r)} 0 1 0 {f(2 * r)} 0a{f(r)} {f(r)} 0 1 0 {f(-2 * r)} 0", w))
        r += gap
    grouped(inside, items)


def swell_hatch(inside, angle, gap, w_of, box=(-20, -20, 532, 532)):
    """Straight lines at `angle`, `gap` apart; each line's width is w_of(x, y) at its middle."""
    x0, y0, x1, y1 = box
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    r = math.hypot(x1 - x0, y1 - y0) / 2
    a = math.radians(angle)
    dx, dy = math.cos(a), math.sin(a)
    nx, ny = -dy, dx
    items = []
    k = -r
    while k <= r:
        px, py = cx + nx * k, cy + ny * k
        # Split each line into short pieces so the width can change along it.
        steps = 10
        for i in range(steps):
            s0 = -r + 2 * r * i / steps
            s1 = -r + 2 * r * (i + 1) / steps
            mx, my = px + dx * (s0 + s1) / 2, py + dy * (s0 + s1) / 2
            w = w_of(mx, my)
            if w >= 0.35:
                items.append((f"M{f(px + dx * s0)} {f(py + dy * s0)}L{f(px + dx * s1)} {f(py + dy * s1)}", w))
        k += gap
    grouped(inside, items)


def ramp(v, a, b):
    """0 at a, 1 at b, clamped."""
    if b == a:
        return 0.0
    return max(0.0, min(1.0, (v - a) / (b - a)))


def rivet(x, y, r=6.5):
    out.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(r)}" fill="{PAPER}" stroke="{INK}" stroke-width="{f(max(2.0, r * 0.42))}"/>')
    out.append(f'<circle cx="{f(x + r * 0.3)}" cy="{f(y + r * 0.3)}" r="{f(r * 0.42)}" fill="{INK}"/>')


def bezier(p0, c1, c2, p3):
    def at(t):
        u = 1 - t
        return (
            u**3 * p0[0] + 3 * u * u * t * c1[0] + 3 * u * t * t * c2[0] + t**3 * p3[0],
            u**3 * p0[1] + 3 * u * u * t * c1[1] + 3 * u * t * t * c2[1] + t**3 * p3[1],
        )

    return at


def plume(spine, width, seed, droop=1.0):
    """An ostrich plume along `spine`: a soft blade of barbs swept towards the tip, curling under."""
    at = bezier(*spine)
    n = 60
    ts = [i / n for i in range(n + 1)]
    pts = [at(t) for t in ts]

    def normal(i):
        a, b = pts[max(0, i - 1)], pts[min(n, i + 1)]
        dx, dy = b[0] - a[0], b[1] - a[1]
        m = math.hypot(dx, dy) or 1.0
        return -dy / m, dx / m

    upper, lower = [], []
    for i, t in enumerate(ts):
        x, y = pts[i]
        nx, ny = normal(i)
        w = width * math.sin(math.pi * min(1.0, 0.08 + t * 1.02)) ** 0.55
        # Barb tufts: the edge scallops where the barbs part.
        scallop = 1 - 0.22 * abs(math.sin(t * 19 + seed))
        upper.append((x + nx * w * scallop, y + ny * w * scallop))
        lower.append((x - nx * w * 0.9 * droop * (2 - scallop), y - ny * w * 0.9 * droop * (2 - scallop)))
    outline = poly(upper + lower[::-1]) + "Z"
    shape(outline, 3.2)
    items = []
    for i in range(2, n - 1, 1):
        x, y = pts[i]
        j = min(n, i + 3)
        for edge, side in ((upper, 1), (lower, -1)):
            ex, ey = edge[j]
            # Curve each barb: out from the spine, bending towards the tip.
            mx, my = x + (ex - x) * 0.55, y + (ey - y) * 0.55
            items.append((f"M{f(x)} {f(y)}Q{f(mx)} {f(my)} {f(ex)} {f(ey)}", 1.5 if side > 0 else 1.9))
    grouped(outline, items)
    # The underside of the blade is in shadow: denser, heavier barbs.
    under = poly(pts + lower[::-1]) + "Z"
    swell_hatch(under, -35, 3.4, lambda x, y: 1.1)
    line(poly(pts), 2.6)


# --- The figure, back to front ---------------------------------------------------------------

# Plume: three ostrich feathers from the crest socket, the first rising and curling back, the
# others falling behind the helm and off the tile.
SOCKET = (352, 132)
plume((SOCKET, (370, 40), (470, 10), (498, 92)), 36, 0.0)
plume((SOCKET, (430, 70), (520, 110), (500, 230)), 34, 1.7)
plume((SOCKET, (420, 140), (480, 230), (452, 330)), 28, 3.1)

# Pauldron (the shoulder nearer us), running off the tile at the bottom right.
PAULDRON = "M232 524C244 454 312 404 404 396C456 392 498 400 524 412L524 524Z"
shape(PAULDRON, 5.5)
swell_rings(PAULDRON, 330, 430, 30, 220, 4.6, 2.3)
for d in [
    "M262 524C274 474 328 438 402 432C452 428 494 436 524 446",
    "M302 524C312 492 352 468 410 464C454 461 492 468 524 478",
    "M344 524C352 508 380 496 418 494C458 492 494 498 524 506",
]:
    line(d, 4.2)
for x, y in [(298, 450), (356, 424), (424, 416), (484, 420)]:
    rivet(x, y, 6)

# Breastplate, bottom left, with its central ridge.
BREAST = "M-12 524C24 474 104 444 190 440L248 524Z"
shape(BREAST, 5.5)
swell_hatch(BREAST, 66, 3.8, lambda x, y: 2.2 * ramp(x, 100, 240) ** 1.3 + 0.5 * ramp(y, 470, 524))
line("M146 524C156 492 172 466 190 444", 3.2)

# Gorget: the neck plates, overlapping downwards and turning away to the right.
GORGET = [
    "M206 350C252 360 318 360 364 344L378 384C326 400 254 402 196 390Z",
    "M192 384C250 398 330 396 382 376L394 418C334 438 250 440 182 426Z",
    "M178 420C244 434 336 432 398 410L408 452C340 476 240 476 166 460Z",
]
for d in GORGET:
    shape(d, 4.5)
    swell_hatch(d, 82, 3.4, lambda x, y: 2.4 * ramp(x, 250, 400) ** 1.4)
for x, y in [(214, 370), (204, 404), (194, 440)]:
    rivet(x, y, 4.5)

# Helmet skull: the dome behind the visor, lit from the upper left.
SKULL = "M170 212C176 150 228 106 292 104C362 102 410 156 414 228C418 290 398 336 368 356L300 362L206 360Z"
shape(SKULL, 6)
swell_rings(SKULL, 262, 168, 46, 230, 4.2, 2.6, 1.6)
# The comb along the crown, and the plume socket on it.
line("M200 174C226 132 262 114 300 112C338 110 366 122 386 146", 3.6)
line("M206 168C232 130 266 120 300 118", 1.6)
SOCKET_TUBE = "M338 124L364 106L380 130L356 146Z"
shape(SOCKET_TUBE, 3.5)
swell_hatch(SOCKET_TUBE, 30, 2.6, lambda x, y: 1.3)
for x, y in [(338, 298), (378, 258), (384, 316)]:
    rivet(x, y, 5)

# Visor: the pointed snout, hinged at the temple.
VISOR = "M172 204C150 222 116 254 88 296C116 322 156 348 206 366L304 352C312 318 310 262 300 230C262 210 214 200 172 204Z"
shape(VISOR, 6)
# Lit along the upper ridge; the snout's underside and the cheek turn into shadow.
swell_rings(VISOR, 196, 214, 40, 170, 4.0, 2.4, 1.5)
swell_hatch(VISOR, 18, 3.6, lambda x, y: 1.6 * ramp(y, 300, 360) + 1.0 * ramp(x, 200, 300) * ramp(y, 290, 330))
# The ridge down the middle of the snout, and the seam of the lower plate.
line("M176 206C160 240 126 272 90 296", 3.0)
line("M100 302C150 300 220 304 302 296", 2.6)
# Eye slits: two openings either side of a bar.
ink("M128 264C178 252 238 244 296 242L298 254C238 256 182 264 134 278Z")
ink("M152 238C198 228 248 224 294 226L294 234C250 234 202 240 158 248Z")
# Breathing holes: three rows on the snout's lower plate.
for row, (y0, n, x0, dx) in enumerate([(310, 5, 168, 20), (326, 5, 182, 20), (342, 4, 206, 20)]):
    for k in range(n):
        x = x0 + k * dx
        y = y0 + k * 1.2
        out.append(f'<ellipse cx="{f(x)}" cy="{f(y)}" rx="4.6" ry="3.1" fill="{INK}" transform="rotate(10 {f(x)} {f(y)})"/>')
# The hinge.
rivet(300, 248, 10)

# --- Output ----------------------------------------------------------------------------------

svg = (
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">'
    f'<defs><clipPath id="t"><rect width="512" height="512" rx="112"/></clipPath>{"".join(defs)}</defs>'
    f'<g clip-path="url(#t)"><rect width="512" height="512" fill="{FIELD}"/>{"".join(out)}</g></svg>\n'
)
sys.stdout.write(svg)
