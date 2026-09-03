# Sources, conflicts, and gaps

## Precedence

The skill user is authoritative. When sources disagree, name both
sides and their layers; do not silently flatten the conflict.
Full wording: [SKILL.md](../SKILL.md#authority).

1. **Skill user** — they weigh the facts.
2. **Observed hardware** on this product (batch variation
   allowed). Lite USB in [flashing.md](flashing.md#usb-measured);
   Lite size and stock table in [measure.md](measure.md).
   Official HTML `epd_*` times are PaperMono lab reference
   in [display.md](display.md), not a silicon row.
3. **Official** board docs, vendor SDKs, schematics, and chip
   datasheets for parts named on this model. Registers and
   timings when they have **not been measured**. Official
   stock/SDK sequences prove **intent and ordering, never
   electrical fact**.
4. **Third-party** firmware (FreeInk, community). Often first
   with new valid detail; often stale or wrong.

Observed outranks a datasheet default. Do not apply a datasheet
to a chip that is not on this model. Do not treat external
board measurements as PaperMono or PaperMono-Lite facts. Name
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
| Live silicon ([measure.md](measure.md), [flashing.md](flashing.md#usb-measured), [display.md](display.md)) | Observed | Lite USB `303a:1001` (run and download); ESP32-S3 v0.2; 16 MB flash; stock table matches UserDemo CSV. Lite I2C 2026-09-02: `ack=32,38,4f,68,6e nak=50,6f,75`, `rtc_flag=31`, `imu_id=24`. JEDEC, PSRAM, `C153` USB/table still empty. Official HTML `epd_*` times are not a row here |
| [PaperMono docs](https://docs.m5stack.com/en/core/PaperMono) | Official | Living **PinMap**, specs, e-paper notes, SKU compare, heading **M5GFX LUT Refresh Speed**. Re-read when nets look stale. Snapshot 2026-09-01: [PaperMono.2026-09-01.md](../resources/official-html/PaperMono.2026-09-01.md) |
| [PaperMono-Lite docs](https://docs.m5stack.com/en/core/PaperMono-Lite) | Official | Living **PinMap** (no RFID/LoRa headings). Same **M5GFX LUT Refresh Speed** table. Same page: M5GFX LUTs unstable; prefer OTP-Demo. Snapshot 2026-09-01: [PaperMono-Lite.2026-09-01.md](../resources/official-html/PaperMono-Lite.2026-09-01.md) |
| Official **M5GFX LUT Refresh Speed** (`epd_quality` / `epd_text` / `epd_fast` / `epd_fastest`) | Official | PaperMono laboratory results under M5GFX modes; reference only; times vary with content. Lite page reprints the same table. Snapshot 2026-09-01: [official-html/SOURCE.md](../resources/official-html/SOURCE.md). [display.md](display.md) |
| Schematic PDFs + gallery PNGs V0.6.2 2026-05-22 ([datasheets.md](../resources/datasheets.md), [catalog.md](catalog.md)) | Official | Dated OSS snapshot from those HTML pages. Walk PDF/PNGs. Nets. HTML may ship a newer set |
| [M5PM1 & M5IOE1 Arduino](https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1) | Official (intent) | L0–L3B, expander pin names, wake examples |
| [M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo) | Official (intent) | OTP path; panel PN `DEPG0397BBS770F3HP-XM`. Direct dep M5Unified; M5GFX is transitive. Panel SPI is `EDP_OTP_LUT_demo` |
| [M5GFX](https://github.com/m5stack/M5GFX) (`Panel_SSD1677_4Gray`) | Official (intent) | UserDemo / M5Unified panel. Four `epd_*` LUT modes. Autodetect `board_M5PaperMono`. Product page: LUTs unstable |
| [M5Unified](https://github.com/m5stack/M5Unified) (`develop`) | Official (intent) | C++ board HAL. PlatformIO `#develop`. `board_M5PaperMono` PMIC / SDMMC / charge / RTC INT. Not a Rust crate. Does not close NYC. Radio tracking is [nyc-wifi-ble](../resources/not-yet-confirmed.md#nyc-wifi-ble) |
| [M5Unified LED](https://github.com/m5stack/M5Unified/blob/8530f5377d782e4a25a6c482de2e71c3f75ca8eb/src/utility/led/LED_PaperMono_Class.hpp) | Official (intent) | `LED_PaperMono_Class`: red on PM1 `0x13`/`0x06`, green on IOE1 PYG8 PWM ch2, blue on PYG9 PWM ch3 (5 kHz, 8-bit) |
| [M5Unified Power](https://github.com/m5stack/M5Unified/blob/8530f5377d782e4a25a6c482de2e71c3f75ca8eb/src/utility/Power_Class.cpp#L72-L96) | Official (intent) | IP2315 gate on IOE1 PYG11: 2 ms wait then 64-loop ready check before charge read |
| [uiflow-micropython PaperMono](https://github.com/m5stack/uiflow-micropython/tree/587e134c61b31431335351e04ebfc05f69064bb7/m5stack/boards/M5STACK_PaperMono) | Official (intent) | Board ID 29, USB `303a:816b`, 240 MHz, 8 MB Octal PSRAM, 16 MB QIO Flash |
| [Stamp LoRa-1262](https://docs.m5stack.com/en/stamp/Stamp_LoRa-1262) | Official (module) | SKU S014 / S014-IF / S014-I. **Contains** SX1262. Module band 868–923 MHz. Not the Semtech die sheet. [nyc-stamp-lora](../resources/not-yet-confirmed.md#nyc-stamp-lora) |
| [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo) | Official (intent) | Eval HAL: SKU probe, rails, SDMMC 4-bit, LoRa `SPI3_HOST`, sleep paths, `partitions.csv`. See [user-demo.md](user-demo.md) (`c109910`, V1.2) |
| ESP32-S3 datasheet v2.2 ([datasheets.md](../resources/datasheets.md)) | Official (named MCU) | Straps GPIO0/3/45/46, JTAG 39–42, USB 19/20, I2C |
| SSD1677 Rev 1.0 / M5PM1 V 1.9 / M5IOE1 V 1.4 / FT6336G / IP2315 / BMI270 / ST25R3916 / SX1262 ([datasheets.md](../resources/datasheets.md)) | Official (named parts) | Registers, opcodes, timings. Observed still outranks a default |
| Product photos ([enclosure.md](enclosure.md)) | Official | Case color, **BUTTON A (UP)** / **BUTTON B (DOWN)** / red power callouts. Not a GPIO pinout |
| FreeInk `PAPERMONO` ([external.md](../resources/external.md)) | Third-party | PMIC/expander sequencing comments; LUT/SDMMC/canvas conflicts |

Do not cite a host checkout path, a one-off dump directory, or
another person’s MAC / serial / NVS / flash image as if they were
product facts.

## Conflicts

State both columns when a page or issue touches a row. The skill
user weighs them. Lite USB IDs and flash **size** are
measured; official HTML `epd_*` times are PaperMono lab
reference, not a Lite timing. The rest of this table is
not measured. Name `C153` vs `C153-Lite`.

| Topic | Official / named | Other sources |
| --- | --- | --- |
| Gray / LUT | 4-gray OTP; M5GFX LUTs “currently unstable”; prefer OTP-Demo. UserDemo uses M5GFX `epd_*` plus analog-off `0x22`/`0x03` then `0x20` ([user-demo.md](user-demo.md), [display.md](display.md)). Official HTML `epd_*` times are PaperMono lab, reference only | FreeInk host-authored LUTs, “3-level grayscale”. Standby recovery without reset is unconfirmed |
| EPD SPI clock | SSD1677 write `fSCL` max 20 MHz. OTP-Demo `EDP_SPI` is 20 MHz | M5GFX PaperMono autodetect sets `freq_write` 40 MHz. [nyc-epd-spi-clock](../resources/not-yet-confirmed.md#nyc-epd-spi-clock) |
| Canvas | 480×800. UserDemo `setRotation(0)`. Lite USB-C down: OTP RAM X = physical Y, RAM Y = physical X ([display.md](display.md), [measure.md](measure.md)) | FreeInk 800×480. OTP-Demo addresses 800×480 RAM. `C153` unmeasured |
| Frontlight | Official HTML: M5PM1 G3 PWM `BL_FB` (brightness). Schematic V0.6.2: one AW9967DNR on `EINK_BL`. UserDemo `display.setBrightness`. Lite: PWM0 slide **drives** the lamp ([measure.md](measure.md)) | FreeInk README AW9967 (schematic-true). FreeInk Paper Mono: G3 → **PWM0**, no `gpioWarm`. CrossPoint warmth UI is for dual-channel boards (X4 Pro / Murphy M4), not this SKU. PWM1 writes left Lite constant |
| M5IOE1 address | Schematic / pin map / UserDemo `IO_EXPANDER_ADDR = 0x4F`. Library: `0x4F` REV `'W'`; fallback candidate `0x6F` | Chip UM V 1.4: `0x6F`–`0x76` from IO7, REV `'A'`. Library default is `0x6F`. Auto-detect `0xFF` also walks `0x70`–`0x76` (includes `0x75`); UserDemo does not use it |
| microSD | DAT0–DAT3 in the pin table. UserDemo `slot_config.width = 4` | FreeInk “native 1-bit SDMMC” |
| Size / weight | HTML: 62.0 × 101.0 × 8.0 mm; 74.7 g / Lite 72.4 g | Older product PDF: 61 mm / “work in progress” |
| USB debug | Lite run **and** download: `303a:1001` Espressif USB JTAG/serial debug unit ([flashing.md](flashing.md#usb-measured)). Vendor Arduino: CDC flags | Generic DevKit or CH343 assumptions do not apply. `C153` not measured |
| Flash | Official 16 MB. Lite **measured** 16 MB (`0x1000000`) and UserDemo-matching table at `0x8000`. PIO `default_16MB.csv` is a different table | 32 MB assumptions do not apply. `C153` table: [nyc-partition-table](../resources/not-yet-confirmed.md#nyc-partition-table). JEDEC / PSRAM / `C153` still [nyc-flash-id](../resources/not-yet-confirmed.md#nyc-flash-id) |
| Power / wake | M5PM1 button. Arduino: IMU/RTC wake via PM1 G4/G0 then `shutdown()`. UserDemo adds ESP `ext0` GPIO4 touch deep sleep. Lite 2026-09-02: short ~0.25 s reset, hold ~2 s download, double-press off. USB-in `SYS_CMD` bounced; unplug lamp-off then 2–3 s `sleep abort` same boot ([power-and-sleep.md](power-and-sleep.md)) | Foreign GPIO45/46 latch code (those pins are PDM here). Sleep current and GPIO0 strap still need a meter |
| LoRa SPI host | Pin table / schematic name those GPIOs SPI1 | UserDemo `hal_lora.cpp` uses ESP-IDF `SPI3_HOST` (SPI0/1 are flash). 868.0 MHz RadioLib begin vs product 868–923 MHz |
| LoRa module vs die | Stamp LoRa-1262: 868–923 MHz, `LoRa_EN` / `SX_NRST` / `SX_ANT_SW`, FPC on C153 | SX1262 sheet: 150–960 MHz ISM. UserDemo DIO2 as RF switch vs PinMap `SX_ANT_SW`. Do not flatten. [nyc-stamp-lora](../resources/not-yet-confirmed.md#nyc-stamp-lora) |
| Bring-up | Arduino: M5PM1 then M5IOE1 then peripherals | UserDemo: 500 ms, `M5.begin`, then PM1/IOE1, then NFC identity probe for SKU |
| Lite NFC/LoRa | HTML **PinMap** and SKU compare: modules absent | Lite schematic V0.6.2 gallery page 05 / PDF still draws Stamp LoRa-1262 and RFID/`PYB_NFC_EN`. Do not flatten |

## Gaps

Everything in
[not-yet-confirmed.md](../resources/not-yet-confirmed.md).
Firmware will not close those rows.
