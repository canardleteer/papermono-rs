# Sources, conflicts, and gaps

## Precedence

The skill user is authoritative. When sources disagree, name both
sides and their layers; do not silently flatten the conflict.
Full wording: [SKILL.md](../SKILL.md#authority).

1. **Skill user** — they weigh the facts.
2. **Observed hardware** on this product (batch variation
   allowed). Lite USB in [flashing.md](flashing.md#usb-measured);
   otherwise still thin ([measure.md](measure.md)).
3. **Official** board docs, vendor SDKs, schematics, and chip
   datasheets for parts named on this model. Registers and
   timings when they have **not been measured**. Official
   stock/SDK sequences prove **intent and ordering, never
   electrical fact**.
4. **Third-party** firmware (FreeInk, community). Often first
   with new valid detail; often stale or wrong.

Observed outranks a datasheet default. Do not apply a datasheet
to a chip that is not on this model. Do not treat Sticky
measured numbers as PaperMono or PaperMono-Lite facts. Name
the SKU (`C153` vs `C153-Lite`).

URL and firmware map: [catalog.md](catalog.md). Vendor
datasheets: [datasheets.md](../resources/datasheets.md). Open
measurements:
[not-yet-confirmed.md](../resources/not-yet-confirmed.md).
External: [external.md](../resources/external.md). Vendor C++:
[cpp-platformio.md](cpp-platformio.md).

## Citations

| Source | Layer | Use |
| --- | --- | --- |
| Live silicon ([measure.md](measure.md), [flashing.md](flashing.md#usb-measured)) | Observed | Lite run-mode USB `303a:1001`. Chip, flash, ACK list, `C153` still empty |
| [PaperMono docs](https://docs.m5stack.com/en/core/PaperMono) | Official | Pin map, specs, e-paper notes, SKU compare |
| [PaperMono-Lite docs](https://docs.m5stack.com/en/core/PaperMono-Lite) | Official | Lite pin map (no NFC/LoRa sections) |
| Schematic PDFs V0.6.2 2026-05-22 ([datasheets.md](../resources/datasheets.md)) | Official | Nets. Walk PDF pages |
| [M5PM1 & M5IOE1 Arduino](https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1) | Official (intent) | L0–L3B, expander pin names, wake examples |
| [M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo) | Official (intent) | OTP path; panel PN `DEPG0397BBS770F3HP-XM` |
| [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo) | Official (intent) | HAL, `partitions.csv` |
| ESP32-S3 datasheet v2.2 ([datasheets.md](../resources/datasheets.md)) | Official (named MCU) | Straps GPIO0/3/45/46, JTAG 39–42, USB 19/20, I2C |
| SSD1677 Rev 1.0 / M5PM1 V 1.9 / M5IOE1 V 1.4 / FT6336G / IP2315 / BMI270 / ST25R3916 / SX1262 ([datasheets.md](../resources/datasheets.md)) | Official (named parts) | Registers, opcodes, timings. Observed still outranks a default |
| Product photos ([enclosure.md](enclosure.md)) | Official | Case color, overall look. Not a pinout |
| FreeInk `PAPERMONO` ([external.md](../resources/external.md)) | Third-party | PMIC/expander sequencing comments; LUT/SDMMC/canvas conflicts |

Do not cite a host checkout path, a one-off dump directory, or
another person’s MAC / serial / NVS / flash image as if they were
product facts.

## Conflicts

State both columns when a page or issue touches a row. The skill
user weighs them. None of these are confirmed on a physical
unit. Name `C153` vs `C153-Lite` when one is measured.

| Topic | Official / named | Other sources |
| --- | --- | --- |
| Gray / LUT | 4-gray OTP; M5GFX LUTs “currently unstable”; prefer OTP-Demo | FreeInk host-authored LUTs, “3-level grayscale” |
| Canvas | 480×800 | FreeInk 800×480 |
| Frontlight | Official HTML: M5PM1 G3 PWM `BL_FB`. Schematic V0.6.2: AW9967DNR on `EINK_BL` (PWM drives the IC) | FreeInk README AW9967 (schematic-true) |
| M5IOE1 address | Schematic / pin map `0x4F` | Chip UM V 1.4: `0x6F`–`0x76` from IO7 at power-on |
| microSD | DAT0–DAT3 in the pin table | FreeInk “native 1-bit SDMMC” |
| Size / weight | HTML: 62.0 × 101.0 × 8.0 mm; 74.7 g / Lite 72.4 g | Older product PDF: 61 mm / “work in progress” |
| USB debug | Lite run mode: `303a:1001` Espressif USB JTAG/serial debug unit ([flashing.md](flashing.md#usb-measured)). Vendor Arduino: CDC flags | Sticky skill: CH343 `1a86:55d3` (wrong product). `C153` / download not measured |
| Flash | 16 MB | Sticky: 32 MB (wrong product) |
| Power | M5PM1 button | Sticky GPIO45/46 latch (wrong product; those pins are PDM here) |

## Gaps

Everything in
[not-yet-confirmed.md](../resources/not-yet-confirmed.md).
Firmware will not close those rows.
