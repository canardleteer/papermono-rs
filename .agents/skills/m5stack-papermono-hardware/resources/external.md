# External skills and sources

Companion material, not this board contract. Prefer live silicon
([measure.md](../references/measure.md)) and the pin map in this
skill when they disagree. Lite USB, chip rev, and 16 MB flash
size are measured; JEDEC, PSRAM, and ACK lists stay open.

## FreeInk SDK (`FREEINK_DEVICE_PAPERMONO`)

[Free-Ink/freeink-sdk](https://github.com/Free-Ink/freeink-sdk)
MIT. Third-party. Snapshot: their README moves.

They describe Paper Mono as ESP32-S3, SSD1677, 800×480 B/W,
non-flashing fast refresh + 3-level grayscale (host-authored
LUTs), FT6336 touch, PMIC-PWM frontlight (AW9967 in the README
table), RX8130 RTC, PDM mic, LEDC buzzer, discrete RGB, native
1-bit SDMMC, M5PM1 telemetry; rails sequenced through M5PM1 +
M5IOE1. `PaperMonoBoard.h` owns bring-up. Consumers normally do
not touch the PMIC/expander except PDM `PIN_MIC_POWER`.

Where they disagree with official docs (LUT path, 3-level vs
4-gray, 800×480 vs 480×800, 1-bit vs DAT0–3): this skill
**names both sides** ([sources.md](../references/sources.md)).
It does not silently prefer FreeInk. Confirm on a physical
unit (`C153` and/or `C153-Lite`) via the matching `nyc-*`
ids. Frontlight AW9967 vs M5PM1 PWM is not a pick-one:
schematic V0.6.2 has PWM into AW9967.

They are useful for expander sequencing comments and for power
management notes (this product is not battery-latched on ESP GPIOs).

Full URL list for PaperMono: [catalog.md](../references/catalog.md).
