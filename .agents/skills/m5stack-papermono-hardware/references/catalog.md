# Official docs and firmware catalog

This page is a map, not a pinout. Pins and rails stay in the
other reference files. Do not copy pin numbers from reTerminal
Sticky or M5Paper (IT8951) pages.

When official pages disagree with each other or with FreeInk,
name both sides ([sources.md](sources.md)); the skill user
weighs them.

## Official documentation

| Page | URL |
| --- | --- |
| PaperMono (`C153`) | https://docs.m5stack.com/en/core/PaperMono |
| PaperMono-Lite (`C153-Lite`) | https://docs.m5stack.com/en/core/PaperMono-Lite |
| Product SKU aliases (UserDemo README) | https://docs.m5stack.com/en/products/sku/C153 and https://docs.m5stack.com/en/products/sku/C153-LITE |
| Product PDF (full) | https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono.pdf |
| Product PDF (Lite) | https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono-Lite.pdf |
| Shop (full) | https://shop.m5stack.com/products/m5papermono-with-lora-nfc-800x480-3-97-eink-display |
| Shop (Lite) | https://shop.m5stack.com/products/papermono-lite-dev-kit-800x480-3-97-e-ink-display |
| Arduino M5PM1 / M5IOE1 | https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1 |
| UiFlow2 flash | https://docs.m5stack.com/en/uiflow2/papermono/program |
| Schematic PDF (full) V0.6.2 2026-05-22 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf |
| Schematic PDF (Lite) V0.6.2 2026-05-22 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522.pdf |
| Model size PDF (full) | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_model_size.pdf |
| Model size PDF (Lite) | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/C153-Lite_PaperMono-Lite_model_size.pdf |

Schematic PDFs are cache ids `papermono-schematic` and
`papermono-lite-schematic` in
[datasheets.md](../resources/datasheets.md) (**V0.6.2,
2026-05-22**). Walk the PDF pages, not only the extract
(wires drop). Nets absorbed there and in
[pin-map.md](pin-map.md). Gallery PNGs exist next to those
PDFs on OSS; prefer the PDF.

## Firmware you can actually run

| Firmware | Kind | Notes |
| --- | --- | --- |
| [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo) | Official ESP-IDF eval | MIT. Arduino-on-IDF v5.5.1, firmware V1.2 at `c109910`. One ELF; runtime NFC probe chooses Pro vs Lite. HAL under `main/hal/`. `partitions.csv` is intent ([user-demo.md](user-demo.md)) |
| [M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo) | Official ESP-IDF OTP | MIT. SSD1677 OTP partial / mono full / 4-gray. Names `DEPG0397BBS770F3HP-XM` |
| M5GFX / M5Unified `develop` | Official Arduino | Product page: LUTs currently unstable; prefer OTP example |
| [M5PM1](https://github.com/m5stack/M5PM1) / [M5IOE1](https://github.com/m5stack/M5IOE1) | Official drivers | MIT |
| UiFlow2 / M5Burner | Official | Flash `PaperMono` or `PaperMono-Lite` image for that SKU |
| Easyloader “User Demo” | Official binary | Linked from the product page |
| Factory reset firmware | Official binary | Linked from the product page. Does not close `nyc-nvs-phy` |
| CrossPoint e-reader | Partner | Linked from the product page as “PaperMono CrossPoint E-Reader” |
| FreeInk `PAPERMONO` | Third-party | [external.md](../resources/external.md) |

SKU: burn Lite firmware on Lite. UserDemo is one binary that
**skips** NFC/LoRa apps when `Hal::hasNfcHardware()` is false;
that is not a license to init those chips on Lite. Do not
assume a full-SKU **UiFlow2 / Easyloader** image is safe on
Lite.

## Native ESP-IDF (no Arduino)

OTP-Demo is the panel path to read first. UserDemo is the HAL
path (Arduino-on-IDF; see [user-demo.md](user-demo.md)). This
repository does not add an IDF project.
