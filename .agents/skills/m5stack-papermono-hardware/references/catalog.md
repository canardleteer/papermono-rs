# Official docs and firmware catalog

This page is a map, not a pinout. Pins and rails stay in the
other reference files. Do not copy pin numbers from other board
pages like M5Paper (IT8951).

When official pages disagree with each other or with FreeInk,
name both sides ([sources.md](sources.md)); the skill user
weighs them. Dated **view as markdown** exports of the two
product pages (2026-09-01):
[official-html/SOURCE.md](../resources/official-html/SOURCE.md).
The living HTML can still change.

## Official documentation

| Page | URL |
| --- | --- |
| PaperMono (`C153`) | https://docs.m5stack.com/en/core/PaperMono |
| PaperMono-Lite (`C153-Lite`) | https://docs.m5stack.com/en/core/PaperMono-Lite |
| HTML **view as markdown** (2026-09-01) | [PaperMono.2026-09-01.md](../resources/official-html/PaperMono.2026-09-01.md), [PaperMono-Lite.2026-09-01.md](../resources/official-html/PaperMono-Lite.2026-09-01.md) ([SOURCE.md](../resources/official-html/SOURCE.md)) |
| Product SKU aliases (UserDemo README) | https://docs.m5stack.com/en/products/sku/C153 and https://docs.m5stack.com/en/products/sku/C153-LITE |
| Product PDF (full) | https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono.pdf |
| Product PDF (Lite) | https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono-Lite.pdf |
| Shop (full) | https://shop.m5stack.com/products/m5papermono-with-lora-nfc-800x480-3-97-eink-display |
| Shop (Lite) | https://shop.m5stack.com/products/papermono-lite-dev-kit-800x480-3-97-e-ink-display |
| Stamp LoRa-1262 | https://docs.m5stack.com/en/stamp/Stamp_LoRa-1262 |
| M5Unified (`develop`) | https://github.com/m5stack/M5Unified |
| Arduino M5PM1 / M5IOE1 | https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1 |
| UiFlow2 flash | https://docs.m5stack.com/en/uiflow2/papermono/program |
| Schematic PDF (full) V0.6.2 2026-05-22 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf |
| Schematic PDF (Lite) V0.6.2 2026-05-22 | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522.pdf |
| Model size PDF (full) | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_model_size.pdf |
| Model size PDF (Lite) | https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/C153-Lite_PaperMono-Lite_model_size.pdf |

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

| Firmware | Kind | Notes |
| --- | --- | --- |
| [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo) | Official ESP-IDF eval | MIT. Arduino-on-IDF v5.5.1, firmware V1.2 at `c109910`. One ELF; runtime NFC probe chooses Pro vs Lite. HAL under `main/hal/`. `partitions.csv` is intent ([user-demo.md](user-demo.md)) |
| [M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo) | Official ESP-IDF OTP | MIT. Direct SSD1677 OTP (partial / mono full / 4-gray). Names `DEPG0397BBS770F3HP-XM`. Direct IDF dep is M5Unified; [M5GFX](https://github.com/m5stack/M5GFX) is a private transitive (`m5gfx` 0.2.27 in the lock). OTP waveforms live in `EDP_OTP_LUT_demo`, not `Panel_SSD1677` |
| [M5GFX](https://github.com/m5stack/M5GFX) | Official Arduino / IDF component | `Panel_SSD1677_4Gray` is the UserDemo / M5Unified panel path (`board_M5PaperMono`). Four `epd_*` titles. Product page: LUTs currently unstable; prefer OTP-Demo for panel life |
| [M5Unified](https://github.com/m5stack/M5Unified) | Official Arduino / IDF component | Product PlatformIO pins `#develop`. `board_M5PaperMono`: PMIC, SDMMC 4-bit, IP2315/IP2316 charge via IOE1 GPIO11, RTC INT. C++ HAL intent, not a Rust crate, not a flash path, does not close NYC. Panel LUTs stay M5GFX / OTP-Demo |
| [M5PM1](https://github.com/m5stack/M5PM1) / [M5IOE1](https://github.com/m5stack/M5IOE1) | Official drivers | MIT. IOE: wake START+STOP, UID then REV, 100 kHz / 800 ms / 400 kHz. Board `0x4F` REV `'W'`; UM `0x6F`–`0x76` REV `'A'`. [user-demo.md](user-demo.md) |
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

OTP-Demo is the panel path to read first (built-in OTP
`0x22` bytes, no MCU LUT). UserDemo is the HAL path
(Arduino-on-IDF; M5GFX `Panel_SSD1677_4Gray`; see
[user-demo.md](user-demo.md)). This repository does not add
an IDF project.
