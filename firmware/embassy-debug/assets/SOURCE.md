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

**Line-art monification:** Source file:
[`ferris-happy-line-art.svg`](ferris-happy-line-art.svg).

## Encode

`ferris.1bpp` is 360×240 packed 1 bit-per-pixel (8
pixels/byte, MSB-first, `1` = ink). `cargo xtask encode-assets`
crops ink and scales to 360×240 pixels before generating the output files:

| File | Role |
| --- | --- |
| `ferris-happy-line-art.svg` | In-repo line art |
| `ferris.1bpp` | Light card (black ink, white skip). Firmware include |
| `ferris-inv.1bpp` | Bitwise invert. Dark-mode pair; not in the image |
| `ferris.png` | 1-bit preview of the light pack |
| `ferris-inv.png` | 1-bit preview of the invert |

```shell
cargo xtask encode-assets
```
