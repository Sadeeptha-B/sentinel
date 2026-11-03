"""Hand-authored pixel-art knight, designed from the brief rather than traced from art.

The figure is symmetric about the sword, so most parts are drawn on the left half and
mirrored. The pipeline is:

1. Parts: flat shapes, back to front.
2. Shading: light from the upper left, with selective dark lines where parts overlap.
3. Hand-placed detail: helm, hilt, gauntlets, heraldry.
4. Asymmetric battle wear and a torn, embroidered cloak.
5. Outline, plus a cool rim light so he holds up on dark themes.

Two builds share the design:
- stocky: broad shoulders, thicker limbs and shorter legs (after v1);
- slim: v2's taller proportions.

Usage: python pixel_knight.py [stocky|slim]   (default: both)
       -> out/pixel-knight-<build>.png (1x) and out/pixel-knight-<build>-6x.png
       python pixel_knight.py --export [stocky]
       -> the app's knight assets (see `export`); then run
          `npx tauri icon tools/asset-pipeline/out/icon-source.png`
"""
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parent / "out"
ROOT = Path(__file__).resolve().parents[2]
W = 56
HIP = 52  # rows below this stretch or squash with the build's leg length
BAND_ROWS = 3  # rows below the feet for the contact shadow; knight.rs reads it from meta
SHADOW = (8, 10, 18)


def hexes(*cs):
    return [tuple(int(c[i : i + 2], 16) for i in (1, 3, 5)) for c in cs]


# Hue-shifted ramps: shadows lean blue/violet, highlights lean warm.
OUTLINE = hexes("#0a0b12")[0]
PALETTE = {
    "cloak": hexes("#0a0f24", "#121b3d", "#1b2953", "#28396f"),
    "lining": hexes("#06081a", "#0a0f26", "#0f1633", "#141d40"),
    "steel": hexes("#1f2333", "#353c4f", "#586176", "#8e98aa"),
    "cloth": hexes("#0e1636", "#172452", "#21356c", "#2e4a88"),
    "gold": hexes("#4b3218", "#7a5826", "#ad853f", "#dcbf78"),
    "blade": hexes("#363c4b", "#687183", "#a1a9b9", "#d5dbe4"),
}
SHINE = hexes("#d4dae3")[0]
RIM = hexes("#4a5f96")[0]
RUST = hexes("#5e2a1e")[0]
S, G, B = PALETTE["steel"], PALETTE["gold"], PALETTE["blade"]
STEEL = {"o": OUTLINE, "w": SHINE, "h": S[3], "m": S[2], "d": S[1], "s": S[0]}
GOLD = {"o": OUTLINE, "h": G[3], "g": G[2], "m": G[1], "s": G[0], "r": RUST}

BUILDS = {
    # d: how much further out the shoulders and arms sit; t: extra torso and limb
    # width; ground: lowest row (shorter legs when smaller).
    "stocky": dict(d=2, t=1, ground=88),
    "slim": dict(d=0, t=0, ground=93),
}


def mx(x: int) -> int:
    """Mirror a column about the sword's axis (between columns 27 and 28)."""
    return W - 1 - x


def build(name: str) -> Image.Image:
    d, t, ground = (BUILDS[name][k] for k in ("d", "t", "ground"))
    h = ground + 3
    hem = ground - 2

    def Y(y: int) -> int:
        """Rows are authored for the slim build's 93-row figure; squash legs to fit."""
        return y if y <= HIP else round(HIP + (y - HIP) * (ground - HIP) / (93 - HIP))

    parts: list[str] = []
    z = np.zeros((h, W), np.int16)

    def part(material: str, *shapes) -> None:
        parts.append(material)
        img = Image.new("L", (W, h), 0)
        draw = ImageDraw.Draw(img)
        for s in shapes:
            if isinstance(s, tuple):
                draw.ellipse(s, fill=255)
            else:
                draw.polygon(s, fill=255)
        m = np.asarray(img) > 0
        z[m | m[:, ::-1]] = len(parts)

    # ── 1. Parts ─────────────────────────────────────────────────────────────
    cloak_slim = [(17, 16), (15, 18), (14, 24), (13, 34), (12, 46), (10, 58), (8, 70), (6, 80), (5, 88), (5, 91)]
    cloak_stocky = [(16, 16), (13, 18), (11, 24), (10, 34), (9, 46), (7, 58), (5, 70), (4, 80), (3, 88), (3, 91)]
    cloak = [(x, Y(y)) for x, y in (cloak_stocky if d else cloak_slim)]
    part("cloak", cloak + [(27, Y(91)), (27, 16)])

    # A wider, planted stance: legs angle out from the hips to the sabatons.
    part("steel", [(20 - t, 50), (26, 50), (26, Y(62)), (20 - t, Y(62))])
    part("steel", [(18 - t, Y(68)), (24, Y(68)), (24, Y(76)), (23, Y(85)), (19 - t, Y(85)), (17 - t, Y(76))])
    part("steel", (18 - t, Y(61), 24, Y(68)))
    part("steel", [(18, Y(84)), (23, Y(84)), (23, ground), (13 - t, ground), (13 - t, ground - 1), (15 - t, ground - 3), (17, ground - 4)])
    part("steel", [(20 - t, 42), (27, 42), (27, Y(53)), (19 - t, Y(56)), (19 - t, 47)])
    part("cloth", [(22, 42), (27, 42), (27, Y(70)), (25, Y(68)), (22, Y(64))])
    part("steel", [(19 - t, 15), (27, 15), (27, 42), (21 - t, 42), (20 - t, 36), (18 - t, 28)])
    part("cloth", [(19 - t, 19), (27, 19), (27, 41), (22 - t, 41), (20 - t, 34), (18 - t, 26)])
    part("steel", [(15 - d, 20), (18 - t, 20), (18 - t, 27), (16 - d, 31), (14 - d, 28)])
    part("steel", [(16 - d, 30), (19 - t, 28), (23, 26), (23, 33), (19 - t, 35), (16 - d, 34)])
    part("steel", (14 - d, 27, 17 - d, 32))
    if d:
        part("steel", [(12, 19), (14, 15), (18, 13), (22, 13), (24, 15), (24, 21), (21, 23), (13, 23)])
    else:
        part("steel", [(14, 19), (16, 16), (19, 14), (22, 14), (24, 16), (24, 20), (21, 22), (15, 22)])
    part("steel", [(23, 12), (27, 12), (27, 17), (22, 17)])
    part("steel", [(27, 0), (27, 1), (25, 2), (24, 3), (23, 5), (23, 12), (24, 13), (27, 13)])
    part("blade", [(26, 37), (27, 37), (27, ground), (27, Y(87)), (26, Y(86))])

    # ── 2. Shading ───────────────────────────────────────────────────────────
    material = np.full((h, W), "", dtype=object)
    for i, m in enumerate(parts, start=1):
        material[z == i] = m

    def at(a, y, x):
        return a[y, x] if 0 <= y < h and 0 <= x < W else 0

    tone = np.full((h, W), 2, np.int8)
    for y in range(h):
        for x in range(W):
            me = z[y, x]
            if not me:
                continue
            lit = at(z, y - 1, x) != me or at(z, y, x - 1) != me
            shaded = at(z, y + 1, x) != me or at(z, y, x + 1) != me
            v = 2 + lit - shaded
            if x >= 31 and material[y, x] in ("steel", "cloth"):
                v -= 1  # the right side turns away from the light
            if any(at(z, y + dy, x + dx) > me for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0))):
                v = 0  # selective line: a part in front casts a thin shadow
            tone[y, x] = max(0, min(3, v))

    # Cloak: lining between the legs; irregular hand-placed folds, each a dark valley
    # with a lit ridge beside it; darker toward the hem.
    for y in range(16, hem + 1):
        for x in range(20, 36):
            if material[y, x] == "cloak" and y >= Y(62):
                material[y, x] = "lining"
                tone[y, x] = 1 if y < Y(82) else 0
    folds = [
        [(15, 22), (14, 36), (13, 52), (11, 70), (9, 91)],
        [(12, 40), (10, 58), (8, 76), (7, 91)],
        [(17, 48), (16, 62), (15, 80)],
        [(mx(16), 24), (mx(15), 44), (mx(13), 66), (mx(11), 91)],
        [(mx(11), 54), (mx(9), 72), (mx(7), 91)],
        [(mx(18), 36), (mx(18), 50), (mx(17), 58)],
    ]
    for fold in folds:
        pts = [(x - d // 2 if x < W // 2 else x + d // 2, Y(y)) for x, y in fold]
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            for y in range(y0, y1 + 1):
                x = round(x0 + (x1 - x0) * (y - y0) / max(1, y1 - y0))
                if material[y, x] == "cloak":
                    tone[y, x] = 0
                    if material[y, x - 1] == "cloak":
                        tone[y, x - 1] = 3 if y < Y(70) else 2
    for y in range(Y(80), hem + 1):
        for x in range(W):
            if material[y, x] == "cloak":
                tone[y, x] = min(tone[y, x], 1)

    rgb = np.zeros((h, W, 3), np.uint8)
    for y in range(h):
        for x in range(W):
            if z[y, x]:
                rgb[y, x] = PALETTE[material[y, x]][tone[y, x]]
    alpha = z > 0

    def put(x, y, c):
        rgb[y, x] = c
        alpha[y, x] = True

    def put_sym(x, y, c):
        put(x, y, c)
        put(mx(x), y, c)

    def paint(x0, y0, rows, key):
        for dy, row in enumerate(rows):
            for dx, ch in enumerate(row):
                if ch in key:
                    put(x0 + dx, y0 + dy, key[ch])

    # ── 3. Detail ────────────────────────────────────────────────────────────
    # Helm: a lit ridge down the centre, brow plate, a single inset eye slit.
    for y in range(1, 13):
        put(27, y, SHINE if y < 4 else S[3])
        put(28, y, S[1])
    for x in range(24, 27):
        put(x, 5, S[3])
        put(mx(x), 5, S[2])
    for x in range(24, 32):
        put(x, 6, OUTLINE)
    for x in range(25, 31):
        if x not in (27, 28):
            put(x, 7, S[0])
    put(24, 3, S[3])

    # Pauldrons: one lame, a rounded highlight, a gold lion roundel on each.
    lame = 21 if d else 20
    for x in range(15 - d, 23):
        if alpha[lame, x]:
            put_sym(x, lame, S[0])
    paint(16 - d, 16 - d // 2, ["..hh", ".hwh", "hhh."], STEEL)
    paint(mx(19 - d), 16 - d // 2, ["mm..", "mhm.", ".mm."], STEEL)
    for cx in (16 - d, mx(18 - d)):
        paint(cx, 15, [".h.", "hgm", ".m."], GOLD)
        put(cx + 1, 16, G[0])

    # Pommel, then two broad gauntleted fists stacked on the grip.
    paint(26, 20, [".hg.", "hggm", ".gm.", ".mm."], GOLD)
    paint(23, 24, [
        ".hhhwhhhm.",
        "hwhmhmhmhd",
        "hmmmmmmmmd",
        "dmmmmmmmdd",
        ".ssssssss.",
        ".hhhhhhhm.",
        "hmhmhmhmhd",
        "hmmmmmmmmd",
        "dmmmmmmmdd",
        ".ssssssss.",
    ], STEEL)

    # Crossguard: gold quillons turning down at the tips, a dark stone at the centre.
    paint(16, 34, [
        "oooooooooooooooooooooooo",
        "ohhhhhhhhhhhhhhhhhhhhhho",
        "osmmmmmmmmmrrmmmmmmmmmso",
        "os                    so",
        "oo                    oo",
    ], GOLD)

    # Blade: bright leading edge, a dark fuller, tapering to a point at his feet.
    for y in range(37, Y(87)):
        put(26, y, B[3])
        put(27, y, SHINE if y % 9 == 0 else B[2])
        put(28, y, B[1])
        put(29, y, B[0] if y > 40 else B[1])
    for y in range(Y(87), ground + 1):
        put(27, y, B[2] if y < ground - 2 else B[1])
        put(28, y, B[1] if y < ground - 2 else B[0])

    # Tabard: dark gold trim and a rampant lion left of the blade.
    for y in range(43, Y(64)):
        put_sym(22, y, G[1] if y % 3 else G[0])
    lion = ["..gg..", ".ghgg.", "gg.g..", ".ggg..", "..ggg.", ".g..g.", "g...g."]
    for dy, row in enumerate(lion):
        for dx, ch in enumerate(row):
            if ch != ".":
                put(21 + dx, 48 + dy, G[3] if ch == "h" else G[2])

    # Faulds, knees, shin ridges, sabaton lames.
    for y in (46, 50):
        for x in range(19 - t, 22):
            put_sym(x, y, S[0])
    put_sym(20, Y(63), S[3])
    put_sym(21, Y(63), SHINE)
    for y in range(Y(70), Y(84)):
        put(21, y, S[3])
        put(mx(22), y, S[2])
    for x in range(13 - t, 24):
        if alpha[ground - 3, x]:
            put_sym(x, ground - 3, S[1])
    put_sym(14 - t, ground - 1, S[3])
    put_sym(15 - t, ground - 1, S[3])

    # ── 4. Battle wear (deliberately asymmetric) ─────────────────────────────
    for x, y in [(17 - d, 18), (23, Y(56)), (32, Y(57)), (35, Y(74)), (36, Y(75)), (35, Y(76)), (20, 30)]:
        put(x, y, S[0])
    for x, y in [(18 - d, 19), (24, Y(57)), (21, 31)]:
        put(x, y, S[3])
    for x, y in [(20, Y(83)), (35, Y(82)), (36, Y(81)), (21, 46), (35, 47)]:
        put(x, y, RUST)
    alpha[19, 14 - d] = alpha[18, 15 - d] = False  # a chip out of the left pauldron
    for y in (Y(60), Y(61)):  # nicks in the blade's edge
        alpha[y, 29] = False
        rgb[y, 28] = B[0]

    # Faint tarnished-gold embroidery just above the hem.
    for x in range(W):
        if material[hem - 5, x] == "cloak" and x % 2 == 0:
            put(x, hem - 5, G[0])
        if material[hem - 4, x] == "cloak" and x % 4 == 1:
            put(x, hem - 4, G[1])

    # Torn hem: V-shaped tears of varying width and depth, which also break the
    # embroidery.
    def tear(xc, half_width, depth):
        for x in range(xc - half_width, xc + half_width + 1):
            cut = round(depth * (1 - abs(x - xc) / (half_width + 1)))
            for y in range(hem - cut, hem + 1):
                if 0 <= x < W and material[y, x] == "cloak":
                    alpha[y, x] = False

    for xc, hw, dp in [(5, 2, 5), (10, 1, 7), (14, 2, 4), (18, 1, 6), (mx(6), 2, 6), (mx(11), 2, 4), (mx(15), 1, 8), (mx(19), 1, 5)]:
        tear(xc - d // 2 if xc < W // 2 else xc + d // 2, hw, dp)
    # Between the legs the cloak falls short of the ground, so he stands on his feet.
    for x in range(20, 36):
        for y in range(Y(80) + (x * 7) % 4, hem + 1):
            if material[y, x] == "lining":
                alpha[y, x] = False
    for x, y, depth in [(10, 46, 2), (9, Y(57), 3), (7, Y(69), 2), (mx(10), 51, 2), (mx(8), Y(63), 3), (mx(6), Y(77), 2)]:
        x = x - d if x < W // 2 else x + d
        for k in range(depth):
            if material[y + k, x] == "cloak":
                alpha[y + k, x] = False
    for x, y in [(10, Y(84)), (13, Y(86)), (mx(11), Y(85)), (mx(14), Y(82))]:
        if material[y, x] == "cloak":
            alpha[y, x] = False

    # ── 5. Outline and rim light ─────────────────────────────────────────────
    canvas = np.zeros((h + 2, W + 2, 4), np.uint8)
    canvas[1:-1, 1:-1, :3] = rgb
    canvas[1:-1, 1:-1, 3] = alpha * 255
    a = canvas[..., 3] > 0

    def neighbours(m):
        n = np.zeros_like(m)
        n[1:, :] |= m[:-1, :]
        n[:-1, :] |= m[1:, :]
        n[:, 1:] |= m[:, :-1]
        n[:, :-1] |= m[:, 1:]
        return n

    ring = neighbours(a) & ~a
    canvas[ring, :3] = OUTLINE
    canvas[ring, 3] = 255

    # Rim: the cloak's side edges catch a cool light, so the navy holds against dark themes.
    mat = np.full((h + 2, W + 2), "", dtype=object)
    mat[1:-1, 1:-1] = material
    side_ring = np.zeros_like(ring)
    side_ring[:, 1:] |= ring[:, :-1]
    side_ring[:, :-1] |= ring[:, 1:]
    edge = a & ~ring & side_ring
    edge[Y(86) :, :] = False
    canvas[edge & (mat == "cloak"), :3] = RIM
    return Image.fromarray(canvas, "RGBA")


def export(name: str) -> None:
    """Write the app's knight assets from one build.

    - src/assets/knight.png: the 1x sprite plus a baked, pixel-stepped contact shadow
      in BAND_ROWS extra rows, so the shadow scales with the pixels.
    - src-tauri/assets/knight.hitmask.bin: 1 bit per sprite pixel, rows packed
      little-endian. Covers the figure only, outline included, shadow excluded.
    - src-tauri/assets/knight.meta.json: sizes and stance, for sizing, hit-testing and
      quote placement.
    - out/icon-source.png: helm and shoulders, for `npx tauri icon`.
    """
    sprite = np.asarray(build(name))
    h, w = sprite.shape[:2]
    figure = sprite[..., 3] > 0

    ground = int(np.nonzero(figure.any(axis=1))[0].max())
    stance = np.nonzero(figure[ground - 4 : ground + 1].any(axis=0))[0]
    image = np.zeros((h + BAND_ROWS, w, 4), np.uint8)
    yy, xx = np.mgrid[0 : h + BAND_ROWS, 0:w]
    cx, rx = (stance.min() + stance.max()) / 2, (stance.max() - stance.min()) / 2 + 3
    dist = ((xx - cx) / rx) ** 2 + ((yy - (ground + 0.5)) / 2.5) ** 2
    image[..., :3] = SHADOW
    image[..., 3] = np.where(dist <= 0.45, 96, np.where(dist <= 1.0, 48, 0))
    image[:h][figure] = sprite[figure]

    assets, rust_assets = ROOT / "src" / "assets", ROOT / "src-tauri" / "assets"
    Image.fromarray(image, "RGBA").save(assets / "knight.png", optimize=True)
    (rust_assets / "knight.hitmask.bin").write_bytes(np.packbits(figure, axis=1, bitorder="little").tobytes())
    meta = {
        "width": w,
        "height": h,
        "bandRows": BAND_ROWS,
        "mask": {"width": w, "height": h, "scale": 1},
        "feetLeft": float(stance.min()) / w,
        "feetRight": float(stance.max() + 1) / w,
    }
    (rust_assets / "knight.meta.json").write_text(json.dumps(meta, indent=2) + "\n")

    # Icon: a square around the helm and shoulders, enlarged without smoothing.
    side = 30
    x0 = w // 2 - side // 2
    head = Image.fromarray(sprite[0:side, x0 : x0 + side], "RGBA")
    k = 1024 // side
    icon = Image.new("RGBA", (1024, 1024), (0, 0, 0, 0))
    icon.alpha_composite(head.resize((side * k, side * k), Image.NEAREST), ((1024 - side * k) // 2,) * 2)
    icon.save(OUT / "icon-source.png")
    print(f"exported {name}: sprite {w}x{h} + {BAND_ROWS} shadow rows")


if __name__ == "__main__":
    OUT.mkdir(exist_ok=True)
    if sys.argv[1:2] == ["--export"]:
        export(sys.argv[2] if len(sys.argv) > 2 else "stocky")
        sys.exit()
    for name in sys.argv[1:] or list(BUILDS):
        sprite = build(name)
        sprite.save(OUT / f"pixel-knight-{name}.png")
        sprite.resize((sprite.width * 6, sprite.height * 6), Image.NEAREST).save(OUT / f"pixel-knight-{name}-6x.png")
        print(f"pixel knight ({name})", sprite.size)
