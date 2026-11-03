"""Extract the knight from the reference art into a transparent, de-haloed figure asset.

Usage: python extract.py   (writes src/assets, src-tauri/assets and out/icon-source.png;
       then run `npx tauri icon tools/asset-pipeline/out/icon-source.png`)

Steps:
  1. BiRefNet-general alpha (cached in out/).
  2. Keep only the largest connected component (drops embers / stray specks).
  3. Clean the alpha: crush near-zero noise, slightly tighten semi-transparent fringe.
  4. Estimate true foreground colours (pymatting) so dark sky does not bleed into edges.
  5. Trim to the figure with a little padding; emit full/half WebP, hit mask and metadata.
"""
import json
from pathlib import Path

import cv2
import numpy as np
from PIL import Image
from pymatting import estimate_foreground_ml

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SRC = ROOT / "docs" / "Battle-Worn Knight Amid Embers.png"
OUT = HERE / "out"
ASSETS = ROOT / "src" / "assets"
OUT.mkdir(exist_ok=True)
ASSETS.mkdir(parents=True, exist_ok=True)

PAD = 6  # px of transparent margin kept around the trimmed figure
HIT_SCALE = 4  # hit mask is 1/HIT_SCALE of the asset resolution


def birefnet_alpha(img: Image.Image) -> np.ndarray:
    cached = OUT / "mask-birefnet-general.png"
    if not cached.exists():
        from rembg import new_session, remove

        remove(img, session=new_session("birefnet-general"), only_mask=True).save(cached)
    return np.asarray(Image.open(cached).convert("L"), dtype=np.float64) / 255.0


def largest_component(alpha: np.ndarray) -> np.ndarray:
    solid = (alpha > 0.5).astype(np.uint8)
    n, labels, stats, _ = cv2.connectedComponentsWithStats(solid, connectivity=8)
    if n <= 2:
        return alpha
    keep = 1 + int(np.argmax(stats[1:, cv2.CC_STAT_AREA]))
    # Grow the kept region a little so its soft edge survives the mask.
    region = cv2.dilate((labels == keep).astype(np.uint8), np.ones((9, 9), np.uint8))
    return alpha * region


def clean_alpha(alpha: np.ndarray) -> np.ndarray:
    alpha = np.where(alpha < 0.04, 0.0, alpha)
    # Gentle contrast on the fringe: pulls faint halo in, keeps a soft anti-aliased edge.
    alpha = np.clip((alpha - 0.06) / 0.9, 0.0, 1.0)
    return alpha


def trim_box(alpha: np.ndarray) -> tuple[int, int, int, int]:
    ys, xs = np.nonzero(alpha > 0.01)
    h, w = alpha.shape
    return (
        max(int(xs.min()) - PAD, 0),
        max(int(ys.min()) - PAD, 0),
        min(int(xs.max()) + PAD + 1, w),
        min(int(ys.max()) + PAD + 1, h),
    )


def foot_metrics(alpha: np.ndarray) -> dict:
    """Locate where he meets the ground (lowest solid row) and how wide his stance is
    there, as fractions of the asset size, for placing the contact shadow."""
    solid = alpha > 0.5
    h, w = solid.shape
    rows = np.nonzero(solid.any(axis=1))[0]
    ground = int(rows.max())
    # Stance: the horizontal extent of everything in the bottom 3% (boots and blade).
    xs = np.nonzero(solid[int(ground - h * 0.03) : ground + 1].any(axis=0))[0]
    return {
        "groundY": ground / h,
        "feetLeft": float(xs.min()) / w,
        "feetRight": float(xs.max()) / w,
    }


def hit_mask(alpha: np.ndarray) -> tuple[bytes, int, int]:
    solid = (alpha > 0.5).astype(np.uint8)
    solid = cv2.dilate(solid, np.ones((5, 5), np.uint8))  # ~2px forgiveness
    h, w = solid.shape
    mw, mh = (w + HIT_SCALE - 1) // HIT_SCALE, (h + HIT_SCALE - 1) // HIT_SCALE
    small = cv2.resize(solid, (mw, mh), interpolation=cv2.INTER_AREA) > 0
    return np.packbits(small, axis=1, bitorder="little").tobytes(), mw, mh


def icon_source(figure: Image.Image) -> Image.Image:
    """Square crop of helmet and shoulders for app/tray icons (feed to `tauri icon`)."""
    a = np.asarray(figure)[..., 3]
    top = int(np.nonzero(a.any(axis=1))[0].min())
    cx = int(np.nonzero(a[top : top + 40].any(axis=0))[0].mean())
    size = int(figure.height * 0.20)
    box = (cx - size // 2, top - 4, cx + size // 2, top - 4 + size)
    return figure.crop(box).resize((1024, 1024), Image.LANCZOS)


def main() -> None:
    src = Image.open(SRC).convert("RGB")
    rgb = np.asarray(src, dtype=np.float64) / 255.0

    alpha = clean_alpha(largest_component(birefnet_alpha(src)))
    fg = estimate_foreground_ml(rgb, alpha)

    x0, y0, x1, y1 = trim_box(alpha)
    alpha, fg = alpha[y0:y1, x0:x1], fg[y0:y1, x0:x1]

    rgba = np.dstack([np.clip(fg, 0, 1), alpha])
    rgba8 = (rgba * 255 + 0.5).astype(np.uint8)
    full = Image.fromarray(rgba8, "RGBA")
    h, w = alpha.shape

    full.save(ASSETS / "knight@full.webp", lossless=True, quality=100, method=6)
    half = full.resize((w // 2, h // 2), Image.LANCZOS)
    half.save(ASSETS / "knight@half.webp", lossless=True, quality=100, method=6)
    full.save(OUT / "knight.png")
    icon_source(full).save(OUT / "icon-source.png")

    bits, mw, mh = hit_mask(alpha)
    (ASSETS / "knight.hitmask.bin").write_bytes(bits)
    # Rust embeds the mask; keep a copy beside the crate sources too.
    rust_assets = ROOT / "src-tauri" / "assets"
    if rust_assets.parent.exists():
        rust_assets.mkdir(exist_ok=True)
        (rust_assets / "knight.hitmask.bin").write_bytes(bits)

    meta = {
        "width": w,
        "height": h,
        "aspect": w / h,
        "mask": {"width": mw, "height": mh, "scale": HIT_SCALE},
        **foot_metrics(alpha),
    }
    (ASSETS / "knight.meta.json").write_text(json.dumps(meta, indent=2) + "\n")
    if rust_assets.exists():
        (rust_assets / "knight.meta.json").write_text(json.dumps(meta, indent=2) + "\n")
    print(json.dumps(meta, indent=2))


if __name__ == "__main__":
    main()
