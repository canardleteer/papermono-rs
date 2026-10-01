# Official docs and firmware catalog

This page is a map, not a pinout. Pins and rails stay in the
other reference files. Do not copy pin numbers from other board
pages like M5Paper (IT8951).

When official pages disagree with each other or with community
sources, name both sides ([sources.md](sources.md)); the skill
user weighs them. Current product-page exports (2026-10-01) and
the prior dated export are in:
[official-html/SOURCE.md](../resources/official-html/SOURCE.md).
The living HTML can still change.

## Official documentation

| Page | URL |
| --- | --- |
| PaperMono (`C153`) | https://docs.m5stack.com/en/core/PaperMono |
| PaperMono-Lite (`C153-Lite`) | https://docs.m5stack.com/en/core/PaperMono-Lite |
| HTML **view as markdown** (2026-10-01) | [PaperMono.2026-10-01.md](../resources/official-html/PaperMono.2026-10-01.md), [PaperMono-Lite.2026-10-01.md](../resources/official-html/PaperMono-Lite.2026-10-01.md) ([SOURCE.md](../resources/official-html/SOURCE.md)) |
| Product SKU aliases (UserDemo README) | https://docs.m5stack.com/en/products/sku/C153 and https://docs.m5stack.com/en/products/sku/C153-LITE |
| Product PDF (C153) | https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono.pdf |
| Product PDF (C153-Lite) | https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono-Lite.pdf |
| Shop (C153) | https://shop.m5stack.com/products/m5papermono-with-lora-nfc-800x480-3-97-eink-display |
| Shop (C153-Lite) | https://shop.m5stack.com/products/papermono-lite-dev-kit-800x480-3-97-e-ink-display |
| Stamp LoRa-1262 | https://docs.m5stack.com/en/stamp/Stamp_LoRa-1262 |
| M5Unified (`develop`) | https://github.com/m5stack/M5Unified |
| Arduino M5PM1 / M5IOE1 | https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1 |
| UiFlow2 flash | https://docs.m5stack.com/en/uiflow2/papermono/program |
| M5Unified LED driver | https://github.com/m5stack/M5Unified/blob/8530f5377d782e4a25a6c482de2e71c3f75ca8eb/src/utility/led/LED_PaperMono_Class.hpp |
| UiFlow2 MicroPython board config | https://github.com/m5stack/uiflow-micropython/tree/587e134c61b31431335351e04ebfc05f69064bb7/m5stack/boards/M5STACK_PaperMono |
| M5Unified Power IP2315 gate sequence | https://github.com/m5stack/M5Unified/blob/8530f5377d782e4a25a6c482de2e71c3f75ca8eb/src/utility/Power_Class.cpp#L72-L96 |
| Schematic PDF (C153) V0.6.2 2026-05-22 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf |
| Schematic PDF (C153-Lite) V0.6.2 2026-05-22 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522.pdf |
| Model size PDF (C153) | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_model_size.pdf |
| Model size PDF (C153-Lite) | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/C153-Lite_PaperMono-Lite_model_size.pdf |

Schematic PDFs are cache ids `papermono-schematic` and
`papermono-lite-schematic` in
[datasheets.md](../resources/datasheets.md) (**V0.6.2,
2026-05-22**). Walk the PDF pages (or the gallery PNGs), not
only the extract (wires drop). Nets absorbed there and in
[pin-map.md](pin-map.md).

## Pin maps (HTML, living)

Official pin tables live under the **PinMap** heading on the
same product pages. This skill’s [pin-map.md](pin-map.md) is
absorbed from those tables plus the schematic. Re-read the
HTML when a net or SKU row looks stale.

| SKU | Page | PinMap notes |
| --- | --- | --- |
| `C153` | https://docs.m5stack.com/en/core/PaperMono | E-Paper, Touch, microSD, HMI, KEY, Audio, M5PM1, M5IOE1, **RFID**, **LoRa** |
| `C153-Lite` | https://docs.m5stack.com/en/core/PaperMono-Lite | Same headings **without** RFID and LoRa; still has IP2315 / M5IOE1 notes |

Product PDFs (`papermono-product` /
`papermono-lite-product`) also carry pin tables and can lag
the HTML.

## Schematics (dated OSS)

Linked from those HTML **Schematics** carousels. Filenames are
dated **V0.6.2 / 2026-05-22**. The HTML pages are the living
index: a later board rev may ship a new PDF/PNG set without
this skill noticing until someone re-checks the docs.

**PaperMono (`C153`)** — six pages (HTML carousel 1/6):

| Page | URL |
| ---: | --- |
| PDF | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf |
| 1 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_01.png |
| 2 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_02.png |
| 3 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_03.png |
| 4 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_04.png |
| 5 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_05.png |
| 6 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_06.png |

**PaperMono-Lite (`C153-Lite`)** — five OSS PNGs (HTML
carousel may show 1/4; do not assume the carousel count is
the file count):

| Page | URL |
| ---: | --- |
| PDF | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522.pdf |
| 1 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_01.png |
| 2 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_02.png |
| 3 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_03.png |
| 4 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_04.png |
| 5 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522_page_05.png |

Page 1 on Lite is a revision-history title block (V0.6.2;
title-block date `26/5/21` vs filename `20260522`). Page 5
still draws Stamp LoRa-1262 and RFID/NFC even though the
Lite HTML PinMap omits those modules. Prefer the PDF for
nets; use the PNGs when the extract drops wires. Cache:
`png/` next to the PDFs ([datasheets.md](../resources/datasheets.md)).

## Firmware you can actually run

Upstream repository refs below were checked on 2026-10-01. Pinned
commits make the reviewed source reproducible; moving branches may
advance.

| Firmware | Kind | Notes |
| --- | --- | --- |
| [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo) | Official ESP-IDF eval | MIT. Reviewed `c1099107` (firmware V1.2, 2026-08-10); one ELF with runtime SKU detection. HAL and the live stock-image comparison: [user-demo.md](user-demo.md), [measure.md](measure.md) |
| [M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo) | Official ESP-IDF OTP | MIT. Reviewed `c7c02554` (2026-08-20). Direct SSD1677 OTP partial / mono full / 4-gray example. Names panel `DEPG0397BBS770F3HP-XM`. Read its current manifest before citing any transitive M5GFX version; the panel OTP recipe is separate from `Panel_SSD1677` LUTs |
| [M5PaperMono-PowerDemo](https://github.com/m5stack/M5PaperMono-PowerDemo) | Official ESP-IDF power tests | Reviewed `cf4f57c6` (2026-09-03). Separate firmware projects publish current measurements at two battery voltages, including PM1 shutdown, wake, display, radio, and frontlight cases. Test reports describe their test units and setup; they are not measurements of our units |
| [M5GFX](https://github.com/m5stack/M5GFX) | Official Arduino / IDF component | Reviewed release `0.2.31` (`cd363dde`, 2026-09-29) and development commit `2be6da8f` (2026-10-01). Current tree contains PaperMono board metadata and evolving four-mode SSD1677 LUT code. Product documentation still calls the PaperMono LUTs unstable and recommends OTP; development code does not supersede that warning |
| [M5Unified](https://github.com/m5stack/M5Unified) | Official Arduino / IDF component | Reviewed `44d0c52d` (2026-09-29). `board_M5PaperMono` sources describe PMIC, SDMMC, charger, and RTC behavior. C++ intent, not Rust API or measured silicon |
| [M5PM1](https://github.com/m5stack/M5PM1) / [M5IOE1](https://github.com/m5stack/M5IOE1) | Official drivers | Reviewed `be9a5456` / `846eec7d` (2026-05-29). Compare chip-driver behavior against current catalog entries and board observations; board `0x4F` is distinct from the chip UM's `0x6F`–`0x76` samples |
| [M5Stack UiFlow board configuration](https://github.com/m5stack/uiflow-micropython/tree/300e3ccde6314bcc30d03f05e33d05872e888e72/m5stack/boards/M5STACK_PaperMono) | Official | Reviewed `300e3ccd` (2026-09-18). Declares 8 MB OPI PSRAM and 16 MB flash; configuration is not a measurement of either unit |
| UiFlow2 / M5Burner | Official | Flash `PaperMono` or `PaperMono-Lite` image for that SKU |
| Easyloader “User Demo” | Official binary | Linked from the product page |
| Factory reset firmware | Official binary | Linked from the product page. Does not close `nyc-nvs-phy` |
| CrossPoint e-reader | Partner | Linked from the product page as “PaperMono CrossPoint E-Reader” |
| FreeInk `PAPERMONO` | Third-party | [external.md](../resources/external.md) |
| Public PaperMono repository search | Community index | 16 matches on 2026-10-01; relevant firmware examples are summarized in [external.md](../resources/external.md). Search results are discovery leads, not evidence of board facts |

SKU: use the image intended for that SKU. UserDemo is one binary
that **skips** NFC/LoRa apps when `Hal::hasNfcHardware()` is false;
that is not a license to init those chips on Lite. Do not
assume a `C153` **UiFlow2 / Easyloader** image is safe on
Lite.

## Native ESP-IDF (no Arduino)

OTP-Demo is the panel path to read first (built-in OTP
`0x22` bytes, no MCU LUT). UserDemo is the HAL path
(Arduino-on-IDF; M5GFX `Panel_SSD1677_4Gray`; see
[user-demo.md](user-demo.md)). This repository does not add
an IDF project.
