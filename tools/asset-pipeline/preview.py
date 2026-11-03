"""Composite the extracted knight over several backdrops for visual QA.

Writes out/preview-sheet.png (full figure on 4 backdrops) and out/preview-crops.png
(2x crops of the hardest regions: helmet, cloak edge, feet) on white and black.
"""
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
OUT = HERE / "out"
knight = Image.open(OUT / "knight.png").convert("RGBA")
w, h = knight.size


def busy(size):
    # Bright, high-frequency backdrop resembling a typical photo wallpaper.
    yy, xx = np.mgrid[0 : size[1], 0 : size[0]]
    r = 0.5 + 0.5 * np.sin(xx / 23.0)
    g = 0.5 + 0.5 * np.sin(yy / 31.0 + xx / 57.0)
    b = 0.6 + 0.4 * np.cos((xx + yy) / 41.0)
    arr = (np.dstack([r * 0.9 + 0.1, g * 0.7 + 0.3, b]) * 255).astype(np.uint8)
    return Image.fromarray(arr, "RGB").convert("RGBA")


backdrops = [
    Image.new("RGBA", knight.size, (245, 245, 242, 255)),
    Image.new("RGBA", knight.size, (128, 128, 128, 255)),
    Image.new("RGBA", knight.size, (12, 14, 18, 255)),
    busy(knight.size),
]
sheet = Image.new("RGB", (w * 4, h), "white")
for i, bg in enumerate(backdrops):
    sheet.paste(Image.alpha_composite(bg, knight).convert("RGB"), (i * w, 0))
sheet.resize((sheet.width // 2, sheet.height // 2), Image.LANCZOS).save(OUT / "preview-sheet.png")

# Crop regions in asset-relative fractions: (x0, y0, x1, y1)
regions = {
    "helmet": (0.38, 0.0, 0.62, 0.16),
    "cloak-left": (0.0, 0.50, 0.25, 0.70),
    "feet": (0.15, 0.85, 0.85, 1.0),
}
tiles = []
for name, (fx0, fy0, fx1, fy1) in regions.items():
    box = (int(fx0 * w), int(fy0 * h), int(fx1 * w), int(fy1 * h))
    crop = knight.crop(box)
    row = []
    for colour in [(250, 250, 250, 255), (10, 10, 10, 255)]:
        bg = Image.new("RGBA", crop.size, colour)
        row.append(Image.alpha_composite(bg, crop).convert("RGB"))
    pair = Image.new("RGB", (crop.width * 2 + 8, crop.height), (255, 0, 255))
    pair.paste(row[0], (0, 0))
    pair.paste(row[1], (crop.width + 8, 0))
    ImageDraw.Draw(pair).text((4, 4), name, fill=(255, 0, 0))
    tiles.append(pair)

cw = max(t.width for t in tiles)
crops = Image.new("RGB", (cw, sum(t.height + 8 for t in tiles)), (255, 0, 255))
y = 0
for t in tiles:
    crops.paste(t, (0, y))
    y += t.height + 8
crops.save(OUT / "preview-crops.png")
print("previews written")
