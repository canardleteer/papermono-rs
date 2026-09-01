# Ferris

In-repo splash art for `embassy-debug`. Packed bitmaps are
derived from the SVG in this directory.

## Attribution

**Original Ferris** (happy / flat) is
[rustacean-flat-happy](https://rustacean.net/assets/rustacean-flat-happy.svg)
on [rustacean.net](https://rustacean.net/). Creator: **Karen
Rustad Tölva**. rustacean.net: to the extent possible under
law, they have waived all copyright and related or neighboring
rights to Ferris the Rustacean (published from the United
States). That is a CC0-style public-domain dedication.

**Black-and-white line-art monification:**
**canardleteer**. Source file:
[`ferris-happy-line-art.svg`](ferris-happy-line-art.svg).

## Encode

`ferris.1bpp` is 360×240 packed 1 bit-per-pixel (8
pixels/byte, MSB-first, `1` = ink). Same canvas as the old
4-gray `ferris.g4`. `encode_ferris.py` crops ink, fits
360×240, thresholds, and writes:

| File | Role |
| --- | --- |
| `ferris-happy-line-art.svg` | canardleteer line art (in repo) |
| `ferris.1bpp` | Light card (black ink, white skip). Firmware include |
| `ferris-inv.1bpp` | Bitwise invert. Dark-mode pair; not in the image |
| `ferris.png` | 1-bit preview of the light pack |
| `ferris-inv.png` | 1-bit preview of the invert |

```shell
python3 firmware/embassy-debug/assets/encode_ferris.py
```
