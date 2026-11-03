"""Render candidate art directions for the knight side by side, for choosing a style.

Usage: python styles.py      (needs out/knight.png from extract.py)

Writes:
  out/brief-sheet.png, out/brief-zoom.png   the same for knights designed from the brief
                       (reference/ v1 and v2, pixel_knight.py v3 builds) vs the painting
  out/style-sheet.png  each candidate at real desktop size (~384 px tall) over a dark
                       IDE, a light window and a photo-like wallpaper
  out/style-zoom.png   helmet and hands of each candidate at 2x, to judge the detail
  out/pixel-96.png, out/pixel-128.png   the raw pixel sprites (1x)
"""
from pathlib import Path

import cv2
import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = Path(__file__).resolve().parent
OUT = HERE / "out"
DISPLAY_H = 384  # figure height on screen, px at 100% scaling
OUTLINE = np.array([11, 13, 20], np.uint8)


def font(size: int):
    for name in ("segoeui.ttf", "arial.ttf", "DejaVuSans.ttf"):
        try:
            return ImageFont.truetype(name, size)
        except OSError:
            pass
    return ImageFont.load_default()


# ── Candidates ───────────────────────────────────────────────────────────────


def painterly(knight: Image.Image, height: int = DISPLAY_H) -> Image.Image:
    """Current art, smoothly resampled to display size."""
    w = round(knight.width * height / knight.height)
    return knight.resize((w, height), Image.LANCZOS)


def _kmeans_palette(rgb: np.ndarray, k: int) -> np.ndarray:
    lab = cv2.cvtColor(rgb.reshape(-1, 1, 3), cv2.COLOR_RGB2LAB).reshape(-1, 3).astype(np.float32)
    criteria = (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 50, 0.2)
    cv2.setRNGSeed(7)
    _, labels, centres = cv2.kmeans(lab, k, None, criteria, 6, cv2.KMEANS_PP_CENTERS)
    centres = cv2.cvtColor(centres.astype(np.uint8).reshape(-1, 1, 3), cv2.COLOR_LAB2RGB).reshape(-1, 3)
    return centres[labels.ravel()]


def pixel_sprite(knight: Image.Image, height: int, colours: int = 16) -> Image.Image:
    """Scripted pixel-art pass: flatten the painted texture into planes, lift and
    sharpen, area-downsample, quantise to a small palette, clean orphans, then add a
    dark outline and a light rim."""
    arr = np.asarray(knight)
    # Mean-shift flattens battle-damage texture into readable planes of colour;
    # without it the sprite reads as camouflage noise.
    flat = cv2.pyrMeanShiftFiltering(np.ascontiguousarray(arr[..., :3]), sp=10, sr=28)
    rgb = flat.astype(np.float32) / 255.0
    alpha = arr[..., 3].astype(np.float32) / 255.0

    # The painting is very dark; lift mids so form survives at sprite size.
    rgb = np.clip(rgb, 0, 1) ** 0.78
    blur = cv2.GaussianBlur(rgb, (0, 0), 3)
    rgb = np.clip(rgb + 0.6 * (rgb - blur), 0, 1)

    w = round(height * knight.width / knight.height)
    pre = rgb * alpha[..., None]
    small_rgb = cv2.resize(pre, (w, height), interpolation=cv2.INTER_AREA)
    small_a = cv2.resize(alpha, (w, height), interpolation=cv2.INTER_AREA)
    mask = small_a > 0.45
    small_rgb = np.where(mask[..., None], small_rgb / np.maximum(small_a, 1e-4)[..., None], 0)

    # Quantise opaque pixels to a shared palette.
    px = (np.clip(small_rgb[mask], 0, 1) * 255).astype(np.uint8)
    q = small_rgb.copy()
    q[mask] = _kmeans_palette(px, colours) / 255.0
    q8 = (q * 255).astype(np.uint8)

    # Orphan cleanup: a pixel unlike all 4 neighbours takes its neighbours' majority.
    out = q8.copy()
    for y in range(1, height - 1):
        for x in range(1, w - 1):
            if not mask[y, x]:
                continue
            nb = [tuple(q8[y + dy, x + dx]) for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0)) if mask[y + dy, x + dx]]
            if len(nb) >= 3 and tuple(q8[y, x]) not in nb:
                out[y, x] = max(set(nb), key=nb.count)
    q8 = out

    # Light rim on the inside edge (reads on dark themes) ...
    k4 = np.array([[0, 1, 0], [1, 1, 1], [0, 1, 0]], np.uint8)
    inner = mask & ~cv2.erode(mask.astype(np.uint8), k4).astype(bool)
    rim = np.array([150, 158, 178], np.float32)
    q8[inner] = (q8[inner] * 0.6 + rim * 0.4).astype(np.uint8)

    # ... and a dark outline outside it (reads on light ones). Pad by one for room.
    q8 = np.pad(q8, ((1, 1), (1, 1), (0, 0)))
    mask = np.pad(mask, 1)
    outer = cv2.dilate(mask.astype(np.uint8), k4).astype(bool) & ~mask
    q8[outer] = OUTLINE
    a8 = ((mask | outer) * 255).astype(np.uint8)
    return Image.fromarray(np.dstack([q8, a8]), "RGBA")


def heraldic(knight: Image.Image) -> Image.Image:
    """Near-black three-tone silhouette with tarnished-gold details and edge."""
    arr = np.asarray(knight).astype(np.float32) / 255.0
    rgb, alpha = arr[..., :3], arr[..., 3]
    hsv = cv2.cvtColor((rgb * 255).astype(np.uint8), cv2.COLOR_RGB2HSV).astype(np.float32)
    h, s, v = hsv[..., 0] * 2, hsv[..., 1] / 255, hsv[..., 2] / 255

    # Three tones of night-steel from luminance, smoothed so they read as shapes.
    lum = cv2.GaussianBlur(v, (0, 0), 2.5)
    tones = np.array([[22, 26, 38], [44, 52, 74], [86, 98, 128]], np.float32)
    idx = np.digitize(lum, [0.16, 0.34])
    base = tones[idx]

    # Gold: the lion emblems, trim and hilt picked out of the painting.
    gold = ((h > 18) & (h < 58) & (s > 0.28) & (v > 0.30)).astype(np.float32)
    gold = cv2.GaussianBlur(cv2.morphologyEx(gold, cv2.MORPH_OPEN, np.ones((2, 2))), (0, 0), 0.8)
    gold_col = np.array([196, 160, 92], np.float32)
    base = base * (1 - gold[..., None]) + gold_col * gold[..., None]

    # Thin gold edge just inside the silhouette.
    solid = (alpha > 0.5).astype(np.uint8)
    edge = solid - cv2.erode(solid, np.ones((11, 11), np.uint8))
    edge = cv2.GaussianBlur(edge.astype(np.float32), (0, 0), 1.2)
    base = base * (1 - 0.9 * edge[..., None]) + gold_col * 0.9 * edge[..., None]

    img = Image.fromarray(np.dstack([np.clip(base, 0, 255), alpha * 255]).astype(np.uint8), "RGBA")
    w = round(img.width * DISPLAY_H / img.height)
    return img.resize((w, DISPLAY_H), Image.LANCZOS)


def upscale(sprite: Image.Image, target_h: int) -> Image.Image:
    k = max(1, round(target_h / sprite.height))
    return sprite.resize((sprite.width * k, sprite.height * k), Image.NEAREST)


# ── Backdrops ────────────────────────────────────────────────────────────────


def ide_dark(w, h):
    img = Image.new("RGB", (w, h), (30, 30, 30))
    d = ImageDraw.Draw(img)
    d.rectangle((0, 0, 46, h), fill=(37, 37, 38))
    rng = np.random.default_rng(1)
    cols = [(86, 156, 214), (206, 145, 120), (156, 220, 254), (106, 153, 85), (212, 212, 212)]
    for y in range(14, h, 19):
        x = 60 + int(rng.integers(0, 4)) * 16
        for _ in range(int(rng.integers(1, 5))):
            ln = int(rng.integers(20, 110))
            d.rectangle((x, y, x + ln, y + 7), fill=cols[int(rng.integers(0, len(cols)))])
            x += ln + 8
    return img


def light_window(w, h):
    img = Image.new("RGB", (w, h), (243, 243, 243))
    d = ImageDraw.Draw(img)
    d.rectangle((20, 20, w - 20, h - 20), fill=(255, 255, 255), outline=(220, 220, 220))
    rng = np.random.default_rng(2)
    for y in range(44, h - 40, 18):
        d.rectangle((40, y, 40 + int(rng.integers(120, w - 90)), y + 6), fill=(196, 196, 200))
    return img


def wallpaper(w, h):
    yy = np.linspace(0, 1, h)[:, None, None]
    sky = (1 - yy) * np.array([92, 140, 186]) + yy * np.array([238, 176, 132])
    img = Image.fromarray(np.broadcast_to(sky, (h, w, 3)).astype(np.uint8))
    d = ImageDraw.Draw(img)
    d.polygon([(0, h * 0.62), (w * 0.3, h * 0.42), (w * 0.55, h * 0.6), (w * 0.8, h * 0.38), (w, h * 0.55), (w, h), (0, h)], fill=(70, 86, 92))
    d.polygon([(0, h * 0.8), (w * 0.4, h * 0.66), (w, h * 0.78), (w, h), (0, h)], fill=(42, 58, 52))
    return img.filter(ImageFilter.GaussianBlur(3))


# ── Composition ──────────────────────────────────────────────────────────────


def with_shadow(bg: Image.Image, fig: Image.Image, pos, pixel_k: int | None = None, centre: float = 0.575):
    """Paste `fig` with its feet at the bottom of `pos` box, plus a contact shadow."""
    x, y = pos
    sh_w, sh_h = int(fig.width * 0.8), max(8, int(fig.height * 0.05))
    shadow = Image.new("L", (sh_w, sh_h), 0)
    ImageDraw.Draw(shadow).ellipse((0, 0, sh_w - 1, sh_h - 1), fill=95)
    if pixel_k:  # stepped, unblurred, on the sprite's pixel grid
        small = shadow.resize((sh_w // pixel_k, sh_h // pixel_k), Image.NEAREST)
        small = small.point(lambda a: 0 if a < 40 else 70)
        shadow = small.resize((small.width * pixel_k, small.height * pixel_k), Image.NEAREST)
    else:
        shadow = shadow.filter(ImageFilter.GaussianBlur(sh_h / 4))
    layer = Image.new("RGBA", bg.size, (0, 0, 0, 0))
    sx = x + int(fig.width * centre) - shadow.width // 2
    sy = y + fig.height - shadow.height // 2 - 2
    layer.paste(Image.new("RGBA", shadow.size, (0, 0, 0, 255)), (sx, sy), shadow)
    out = Image.alpha_composite(bg.convert("RGBA"), layer)
    out.alpha_composite(fig, (x, y))
    return out


def compose(candidates, name: str, cell_h: int = 440) -> None:
    """Write `name`-sheet.png and `name`-zoom.png for (label, figure, pixel_k, centre) rows."""
    backdrops = [("Dark IDE", ide_dark), ("Light window", light_window), ("Wallpaper", wallpaper)]
    cell_w, head, side = 330, 44, 120
    sheet = Image.new("RGB", (side + cell_w * len(candidates), head + cell_h * len(backdrops)), (18, 18, 20))
    d = ImageDraw.Draw(sheet)
    for c, (label, *_) in enumerate(candidates):
        d.text((side + c * cell_w + 14, 12), label, fill=(232, 232, 232), font=font(17))
    for r, (bname, make) in enumerate(backdrops):
        d.text((12, head + r * cell_h + cell_h // 2 - 10), bname, fill=(200, 200, 200), font=font(15))
        for c, (_, fig, k, centre) in enumerate(candidates):
            bg = make(cell_w, cell_h)
            pos = ((cell_w - fig.width) // 2, cell_h - fig.height - 22)
            cell = with_shadow(bg, fig, pos, k, centre)
            sheet.paste(cell.convert("RGB"), (side + c * cell_w, head + r * cell_h))
    sheet.save(OUT / f"{name}-sheet.png")

    # Zoom: helmet and hands at 2x over the dark IDE backdrop. Regions are found per
    # figure (painting is off-centre because the cloak flows left).
    tiles = []
    for _, fig, _, centre in candidates:
        row = []
        for y0, y1 in ((0.0, 0.22), (0.20, 0.42)):
            half = 0.24
            box = (int((centre - half) * fig.width), int(y0 * fig.height), int((centre + half) * fig.width), int(y1 * fig.height))
            crop = fig.crop(box)
            bg = ide_dark(*crop.size).convert("RGBA")
            bg.alpha_composite(crop)
            row.append(bg.resize((crop.width * 2, crop.height * 2), Image.NEAREST))
        tiles.append(row)
    tw = max(sum(t.width for t in row) + 12 for row in tiles)
    th = max(t.height for row in tiles for t in row)
    zoom = Image.new("RGB", (side + tw, head + (th + 12) * len(tiles)), (18, 18, 20))
    dz = ImageDraw.Draw(zoom)
    for r, row in enumerate(tiles):
        y = head + r * (th + 12)
        dz.text((12, y + th // 2 - 10), candidates[r][0][:1], fill=(232, 232, 232), font=font(22))
        x = side
        for t in row:
            zoom.paste(t.convert("RGB"), (x, y))
            x += t.width + 12
    dz.text((side, 12), "Detail at 2x: helmet | hands & hilt", fill=(232, 232, 232), font=font(17))
    zoom.save(OUT / f"{name}-zoom.png")


def main() -> None:
    knight = Image.open(OUT / "knight.png").convert("RGBA")

    # Round 1: styles derived from the reference painting.
    p96, p128 = pixel_sprite(knight, 96), pixel_sprite(knight, 128)
    p96.save(OUT / "pixel-96.png")
    p128.save(OUT / "pixel-128.png")
    compose(
        [
            ("A  Current painting", painterly(knight), None, 0.575),
            ("B  Pixel art, 96 px @ 4x", upscale(p96, DISPLAY_H), 4, 0.575),
            ("C  Pixel art, 128 px @ 3x", upscale(p128, DISPLAY_H), 3, 0.575),
            ("D  Heraldic silhouette", heraldic(knight), None, 0.575),
        ],
        "style",
    )

    # Round 2: the pixel knight designed from the brief, each pass side by side, all at
    # the same 4x so differences are the design, not the scale (run pixel_knight.py
    # first for the current builds).
    sprites = [
        ("B  v1", Image.open(HERE / "reference" / "pixel-knight-v1.png")),
        ("C  v2", Image.open(HERE / "reference" / "pixel-knight-v2.png")),
        ("D  v3 slim", Image.open(OUT / "pixel-knight-slim.png")),
        ("E  v3 stocky", Image.open(OUT / "pixel-knight-stocky.png")),
    ]
    rows = [("A  Current painting", painterly(knight, 392), None, 0.575)]
    for label, sprite in sprites:
        sprite = sprite.convert("RGBA")
        k = 4
        big = sprite.resize((sprite.width * k, sprite.height * k), Image.NEAREST)
        rows.append((f"{label}, {sprite.height} px @ {k}x", big, k, 0.5))
    compose(rows, "brief", cell_h=max(r[1].height for r in rows) + 50)
    print("wrote style-*.png and brief-*.png")


if __name__ == "__main__":
    main()
