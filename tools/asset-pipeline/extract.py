"""Extract the knight from the reference painting into a transparent, de-haloed cutout.

Reference tooling only: the app's knight is the pixel sprite (pixel_knight.py --export).
This writes out/knight.png, which styles.py uses to compare art directions; it never
touches the app's assets.

Usage: python extract.py

Steps:
  1. BiRefNet-general alpha (cached in out/).
  2. Keep only the largest connected component (drops embers / stray specks).
  3. Clean the alpha: crush near-zero noise, slightly tighten semi-transparent fringe.
  4. Estimate true foreground colours (pymatting) so dark sky does not bleed into edges.
  5. Trim to the figure with a little padding.
"""
from pathlib import Path

import cv2
import numpy as np
from PIL import Image
from pymatting import estimate_foreground_ml

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SRC = ROOT / "docs" / "Battle-Worn Knight Amid Embers.png"
OUT = HERE / "out"
OUT.mkdir(exist_ok=True)

PAD = 6  # px of transparent margin kept around the trimmed figure


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


def main() -> None:
    src = Image.open(SRC).convert("RGB")
    rgb = np.asarray(src, dtype=np.float64) / 255.0

    alpha = clean_alpha(largest_component(birefnet_alpha(src)))
    fg = estimate_foreground_ml(rgb, alpha)

    x0, y0, x1, y1 = trim_box(alpha)
    alpha, fg = alpha[y0:y1, x0:x1], fg[y0:y1, x0:x1]

    rgba = np.dstack([np.clip(fg, 0, 1), alpha])
    Image.fromarray((rgba * 255 + 0.5).astype(np.uint8), "RGBA").save(OUT / "knight.png")
    print("wrote", OUT / "knight.png")


if __name__ == "__main__":
    main()
