#!/usr/bin/env python3
"""Rasterize Ferris line art to 360×240 packed 1bpp (+ invert).

Attribution: SOURCE.md (Karen Rustad Tölva / rustacean.net;
in-repo line-art monification).
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image

WIDTH = 360
HEIGHT = 240
THRESHOLD = 200
PAD_PCT = 3


def raster_svg(svg: Path, png: Path) -> None:
    subprocess.run(
        ["convert", "-background", "white", "-density", "300", str(svg), str(png)],
        check=True,
    )


def crop_ink(src: Image.Image) -> Image.Image:
    ink = src.point(lambda p: 255 if p < 250 else 0)
    bbox = ink.getbbox()
    if bbox is None:
        raise SystemExit("no ink in raster")
    pad = max(bbox[2] - bbox[0], bbox[3] - bbox[1]) * PAD_PCT // 100
    x0 = max(0, bbox[0] - pad)
    y0 = max(0, bbox[1] - pad)
    x1 = min(src.size[0], bbox[2] + pad)
    y1 = min(src.size[1], bbox[3] + pad)
    return src.crop((x0, y0, x1, y1))


def fit_canvas(crop: Image.Image) -> Image.Image:
    cw, ch = crop.size
    scale = min(WIDTH / cw, HEIGHT / ch)
    nw = max(1, int(round(cw * scale)))
    nh = max(1, int(round(ch * scale)))
    fitted = crop.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas = Image.new("L", (WIDTH, HEIGHT), 255)
    canvas.paste(fitted, ((WIDTH - nw) // 2, (HEIGHT - nh) // 2))
    return canvas.point(lambda p: 0 if p < THRESHOLD else 255)


def pack_1bpp(bw: Image.Image) -> bytes:
    """MSB-first, 1 = ink (black). WIDTH must be divisible by 8."""
    if bw.size != (WIDTH, HEIGHT):
        raise SystemExit(f"expected {WIDTH}×{HEIGHT}, got {bw.size}")
    if WIDTH % 8 != 0:
        raise SystemExit("WIDTH must be a multiple of 8")
    out = bytearray(WIDTH * HEIGHT // 8)
    px = bw.load()
    for y in range(HEIGHT):
        for x in range(WIDTH):
            if px[x, y] < 128:
                i = y * WIDTH + x
                out[i // 8] |= 0x80 >> (i % 8)
    return bytes(out)


def unpack_preview(blob: bytes) -> Image.Image:
    im = Image.new("L", (WIDTH, HEIGHT), 255)
    px = im.load()
    for y in range(HEIGHT):
        for x in range(WIDTH):
            i = y * WIDTH + x
            if blob[i // 8] & (0x80 >> (i % 8)):
                px[x, y] = 0
    return im


def main() -> int:
    here = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--svg",
        type=Path,
        default=here / "ferris-happy-line-art.svg",
    )
    parser.add_argument("--out-dir", type=Path, default=here)
    args = parser.parse_args()
    svg = args.svg.expanduser().resolve()
    out_dir = args.out_dir.expanduser().resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory() as tmp:
        page = Path(tmp) / "page.png"
        raster_svg(svg, page)
        canvas = fit_canvas(crop_ink(Image.open(page).convert("L")))

    packed = pack_1bpp(canvas)
    inverted = bytes(b ^ 0xFF for b in packed)
    (out_dir / "ferris.1bpp").write_bytes(packed)
    (out_dir / "ferris-inv.1bpp").write_bytes(inverted)
    unpack_preview(packed).save(out_dir / "ferris.png")
    unpack_preview(inverted).save(out_dir / "ferris-inv.png")

    expect = WIDTH * HEIGHT // 8
    if len(packed) != expect or packed == inverted:
        raise SystemExit("pack failed")
    if any(a ^ 0xFF != b for a, b in zip(packed, inverted, strict=True)):
        raise SystemExit("invert is not a pair")
    print(f"wrote {expect} bytes ×2 to {out_dir}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
