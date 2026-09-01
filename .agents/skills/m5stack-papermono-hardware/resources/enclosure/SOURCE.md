# Source provenance

Captured M5Stack product photos. These are vendor illustrations of
**the enclosure**, not a pinout and not a schematic.

Callout names on the photos are the user-facing labels:
**BUTTON A (UP)**, **BUTTON B (DOWN)**, and the red power
button (PRESS ON/RST, DOUBLE OFF, HOLD BOOT). GPIO numbers
still come from official tables
([pin-map.md](../../references/pin-map.md)), not from the
back-sticker text.

| Field | PaperMono (`C153`) | PaperMono-Lite (`C153-Lite`) |
| --- | --- | --- |
| Upstream page | https://docs.m5stack.com/en/core/PaperMono | https://docs.m5stack.com/en/core/PaperMono-Lite |
| Upstream image | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_01.webp | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/C153-Lite_PaperMono-Lite_main_pictures_01.webp |
| WebP (upstream bytes) | `C153_PaperMono_main_pictures_01.webp` | `C153-Lite_PaperMono-Lite_main_pictures_01.webp` |
| WebP format | 1600 × 1600, RGB | 1600 × 1600, RGB |
| WebP SHA-256 | `f3919ac7b8c6f1ec40abc140b05c5c73ca7505ec91fe8a96cd93a633d568d8d5` | `3d478f4913ea6ff4391a6a19ae54dcc9c7b9e4b288d210f73bd0198e5d63856b` |
| PNG (decode, agent-readable) | `C153_PaperMono_main_pictures_01.png` | `C153-Lite_PaperMono-Lite_main_pictures_01.png` |
| PNG SHA-256 | `f8370b58db7101f8f3336c2ffa711a4af5bc65a5305ef0d62fb774f12c91ed0a` | `1f9227f06ce63cac8c4ec5beb9f5116138c37a5c2c3689078c8f1bfb11cc085b` |
| Vendored on | 2026-08-31 (WebP); 2026-09-01 (PNG decode) | Same |
| Copyright | M5Stack (product photography). Vendored for offline agent use. | Same |

Prefer the **PNG** when reading pixels. The WebP is the
upstream file; re-decode PNG after a WebP re-fetch (`dwebp`).

Re-fetch (do not invent a substitute URL):

```shell
curl -fsSL -o C153_PaperMono_main_pictures_01.webp \
    https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_01.webp
curl -fsSL -o C153-Lite_PaperMono-Lite_main_pictures_01.webp \
    https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/C153-Lite_PaperMono-Lite_main_pictures_01.webp
dwebp C153_PaperMono_main_pictures_01.webp \
    -o C153_PaperMono_main_pictures_01.png
dwebp C153-Lite_PaperMono-Lite_main_pictures_01.webp \
    -o C153-Lite_PaperMono-Lite_main_pictures_01.png
sha256sum C153_PaperMono_main_pictures_01.webp \
    C153-Lite_PaperMono-Lite_main_pictures_01.webp \
    C153_PaperMono_main_pictures_01.png \
    C153-Lite_PaperMono-Lite_main_pictures_01.png
```

Layout facts absorbed from these files:
[enclosure.md](../../references/enclosure.md). Lite edge and
GPIO-vs-press are written there. `C153` still:
[nyc-enclosure-edges](../not-yet-confirmed.md#nyc-enclosure-edges).
