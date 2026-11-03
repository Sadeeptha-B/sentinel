"""Run candidate matting models over the reference art and save raw masks for inspection."""
import sys
from pathlib import Path

from PIL import Image
from rembg import new_session, remove

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "docs" / "Battle-Worn Knight Amid Embers.png"
OUT = Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

models = sys.argv[1:] or ["birefnet-general"]
img = Image.open(SRC).convert("RGB")
for name in models:
    mask = remove(img, session=new_session(name), only_mask=True)
    mask.save(OUT / f"mask-{name}.png")
    print("saved", name)
