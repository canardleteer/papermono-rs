---
name: m5stack-papermono-hardware
description: >-
  Use when writing or reviewing firmware or software for the M5Stack
  PaperMono (SKU C153) or PaperMono-Lite (SKU C153-Lite): ESP32-S3R8,
  800x480/480x800 SSD1677 e-paper, FT6336G, M5PM1, M5IOE1, native USB,
  including GPIO and expander maps, power-rail bring-up, display/touch/
  IMU/battery wiring, SDMMC, NFC/LoRa on the full SKU, deep sleep,
  flashing, destroy-the-board hazards, or when sources disagree about
  this board. Vendor datasheet citations use the skill catalog and a
  gitignored local PDF/markdown cache; populate that cache when the
  work is registers, opcodes, or timings. Also use when the user
  mentions PaperMono, PaperMono-Lite, C153, M5PM1, M5IOE1, or vendor
  Arduino / ESP-IDF / FreeInk firmware as evidence of how this board
  is wired.
---

# M5Stack PaperMono Hardware

Board contract for the M5Stack PaperMono (`C153`) and PaperMono-Lite
(`C153-Lite`). Read this file first. Hardware facts live in the
subsystem pages. Do not mix a stack’s APIs into the pin map.

Host discovery and flash I/O belong to the consuming project’s
tools, not this skill. Vendor Arduino / ESP-IDF / PlatformIO trees
are wiring evidence, not a flash path here.

Official docs, UserDemo, OTP-Demo, and FreeInk are not a
measurement. Observed silicon:
[measure.md](references/measure.md). Lite run-mode USB is in
[flashing.md](references/flashing.md#usb-measured). There are
two SKUs: PaperMono (`C153`) and PaperMono-Lite
(`C153-Lite`). Name which one you measured.
Open nets live in
[not-yet-confirmed.md](resources/not-yet-confirmed.md).

## How to read this skill

1. **Authority** — [references/sources.md](references/sources.md).
   Precedence and the conflict inventory. The skill user weighs
   disagreements.
2. **Hazards** — [references/safety.md](references/safety.md).
   Destroy the board or irreplaceable data. A consuming repo may
   symlink this page as `docs/SAFETY.md`.
3. **Observed silicon** —
   [references/measure.md](references/measure.md). Chip, flash,
   USB, factory image, and which peripherals ACK **on a
   PaperMono or PaperMono-Lite in hand**. Lite run-mode USB is
   in [flashing.md](references/flashing.md#usb-measured). Name
   the SKU. A result on one variant does not confirm the
   other. That beats SDK profiles when they disagree on those
   fields.
4. **Enclosure** —
   [references/enclosure.md](references/enclosure.md). Where keys,
   USB-C, and the SD slot sit. Vendored product photos:
   [resources/enclosure/](resources/enclosure/SOURCE.md).
5. **Pin map and rails** — remaining hardware pages (official pin
   tables until [nyc-flash-id](resources/not-yet-confirmed.md#nyc-flash-id)
   and friends close).
6. **Official docs and firmware catalog** —
   [references/catalog.md](references/catalog.md).
7. **Vendor datasheets** —
   [resources/datasheets.md](resources/datasheets.md). Registers,
   opcodes, timings for parts named on this model. **Vendor the
   local cache** when that work needs a sheet (see
   [Vendor datasheets](#vendor-datasheets-local-cache)).
8. **Vendor C++ evidence** —
   [references/cpp-platformio.md](references/cpp-platformio.md).
   Sequences from Arduino / ESP-IDF trees (intent and ordering,
   never electrical fact).
9. **Measurement backlog** — remaining open nets and confirmation
   recipes live in
   [resources/not-yet-confirmed.md](resources/not-yet-confirmed.md).
10. **External sources** — FreeInk PaperMono board profile:
    [resources/external.md](resources/external.md).

Do not mix a stack’s APIs into the pin map. Do not commit another
person’s MAC, serial number, USB serial string, NVS, or flash
image.

## Authority

When sources disagree, name both sides and their layers. Do not
flatten a conflict to one number. The skill user is authoritative:
they decide how to weigh the facts. This skill presents the stack;
it does not silently pick a winner against the user.

**Precedence (highest first):**

1. **The skill user.** They resolve the conflict. Ask when a
   choice would change wiring, flash, or a hazardous write.
2. **Observed hardware** on this product, with batch variation
   allowed. Live USB, `flash-id`, ACKs, meter/schematic, and
   on-unit partition facts
   ([measure.md](references/measure.md)). **Name the SKU**
   (`C153` or `C153-Lite`). A measurement on one variant does
   not confirm the other. Lite run-mode USB is the first
   observed row; chip/flash/ACK lists are still empty.
3. **Official** board documentation, vendor SDKs, and **chip
   datasheets for parts named on this model.** Registers, opcodes,
   and timings belong here when they have **not been measured**.
   Official stock/SDK sequences still prove **intent and
   ordering, never electrical fact**. Do not apply a datasheet to
   a part that is not named on this model.
4. **Third-party** firmware (FreeInk, community). Often first to
   carry new valid detail; also the usual source of stale or
   wrong maps.

An observed address or pin (2) outranks a datasheet default (3).
Datasheets outrank a random SSD1677 example (4) for opcodes on
the named panel controller.

Inventory: [sources.md](references/sources.md). New mismatches
get a row there or a recipe in
[not-yet-confirmed.md](resources/not-yet-confirmed.md). Speak up
when a page, crate, or user issue sits on a known conflict.

Do not invent GPIOs. Do not use a generic ESP32-S3 DevKit pinout.
Do not invent registers. If the local datasheet cache is missing,
ask the user to populate it rather than guessing.

## Product snapshot

Official docs, **not measured**. Confirm JEDEC or USB IDs on a
physical unit (`C153` and/or `C153-Lite`) via
[nyc-flash-id](resources/not-yet-confirmed.md#nyc-flash-id)
before treating them as confirmed.

| Item | PaperMono (`C153`) | PaperMono-Lite (`C153-Lite`) |
| --- | --- | --- |
| MCU | ESP32-S3R8, Xtensa LX7 dual-core, up to 240 MHz | Same |
| RAM | 8 MB in-package octal PSRAM | Same |
| Flash | 16 MB | Same |
| Display | 3.97" 480×800, 4-gray, SSD1677 SPI | Same |
| Touch | FT6336G `0x38`; active area 5–475 / 5–795 | Same |
| Frontlight | M5PM1 G3 PWM → AW9967 (`EINK_BL`) | Same |
| USB debug | Native pads; Arduino CDC flags (intent) | Run mode: `303a:1001` Espressif USB JTAG/serial debug unit |
| Battery | 1150 mAh 1S, IP2315 charger `0x75` | Same |
| PMIC | M5PM1 `0x6E` | Same |
| Expander | M5IOE1 **board `0x4F`** (chip UM `0x6F`–`0x76`) | Same |
| IMU | BMI270 `0x68` | Same |
| RTC | RX8130CE `0x32` | Same |
| Audio | PDM LMD4737T261-AC02 (GPIO45/46); buzzer GPIO42 | Same |
| NFC | ST25R3916 `0x50` | **Absent** |
| LoRa | SX1262 Stamp LoRa-1262, 868–923 MHz | **Absent** |
| Storage | microSD SDMMC GPIO8–13 | Same |
| Case | Gray; 74.7 g | White; 72.4 g |
| Size | 62.0 × 101.0 × 8.0 mm | Same |

Xtensa target when using Rust: `xtensa-esp32s3-none-elf`
(`no_std`) or the ESP-IDF Rust target (`std`). USB-C is native
Espressif USB, not a CH343. **Lite run mode:** `303a:1001`
“USB JTAG/serial debug unit”
([flashing.md](references/flashing.md#usb-measured)). `C153`
and download mode:
[nyc-usb-vid](resources/not-yet-confirmed.md#nyc-usb-vid).

## Hard rules (all stacks)

1. **Do not copy Sticky power-latch code.** GPIO45 and GPIO46
   are PDM CLK/DAT here (and ESP32-S3 strapping pins). Power is
   the M5PM1 button and rails, not `PWR_HOLD` / `PWR_LOCK`.
2. **GPIO0 and GPIO3 are strapping pins** (ESP32-S3 datasheet
   v2.2 §3). GPIO0 is M5PM1 `BOOT_OUT`. GPIO3 is KEY2. Do not
   wiggle them until hold time after `CHIP_PU`.
3. **E-paper OTP first.** The panel has built-in OTP waveforms.
   M5Stack says M5GFX LUTs are currently unstable and warns that
   custom external waveforms must stay DC-balanced. Do not invent
   a 105-byte `0x32` table (Table 7-1 is 105 bytes). After ~10
   partials, do a full refresh.
4. **Park IP2315 off the system I2C bus** except for the charge
   transaction. M5IOE1 `PYG11_PWM3` gates `0x75`. Sheet: I2C
   high is VBAT; pins 8/9 mux LED vs I2C; at VIN both must
   sample high or the chip stays in LED mode and can hang
   neighbors.
5. **Download mode is the power button**, not DTR on a CH343.
   Hold ~2 s until the red LED blinks, then release.
   [nyc-download-mode](resources/not-yet-confirmed.md#nyc-download-mode).
6. **Ship a 16 MB-aware partition table.** Do not inherit 8 MB
   DevKit limits. Do not copy Sticky’s 32 MB / `0x90000` geometry.
7. **Lite has no NFC and no LoRa.** Do not init ST25R3916 or
   SX1262 on `C153-Lite`. Do not treat those GPIOs as free until
   [nyc-lite-nfc-pads](resources/not-yet-confirmed.md#nyc-lite-nfc-pads)
   / [nyc-lite-lora-pads](resources/not-yet-confirmed.md#nyc-lite-lora-pads)
   close.
8. **Display SPI is not shared with microSD.** EPD is SPI2
   (GPIO14–18). SD is SDMMC (GPIO8–13). LoRa (full SKU) is SPI1.
9. **M5PM1 and M5IOE1 GPIO default open-drain** (including
   PWM). Configure push-pull or provide a pull-up. M5IOE1 chip
   UM samples `0x6F`–`0x76` from IO7; this board is labeled
   `0x4F` — name both; do not leave IO7 floating on a rework.
10. **Mux GPIO39–42 off JTAG** (ESP32-S3 Table 2-4 / §2.3.4)
    before LoRa SPI (39–41) or the buzzer (42).

## Vendor datasheets (local cache)

Catalog: [resources/datasheets.md](resources/datasheets.md).
Cached PDFs and extracted markdown live in
[resources/datasheets/](resources/datasheets/README.md)
(`pdf/`, `md/`; gitignored).

**Vendor that cache** when the work is registers, opcodes,
timings, strapping/I2C/SPI limits, SSD1677 command tables, or a
datasheet-versus-docs conflict. Search
`resources/datasheets/md/<id>.md` rather than loading a whole
TRM. The cache does **not** replace the pin map or enclosure.

It does not help for board wiring you already have from official
pin tables, or for third-party project structure. For **official
M5Stack HTML docs** as an offline markdown corpus, use the
user-global `skill-corpus-vendoring` skill — do not invent a
second datasheet pipeline.

When citing a register, opcode, or timing:

1. Read the catalog (gaps live there).
2. If `resources/datasheets/md/<id>.md` exists, search that file.
3. If the markdown (or PDF) is missing, **ask the user to
   populate the cache** before inventing a constant. Do not
   download vendor files unless they asked.

```shell
# from this skill directory
python3 scripts/fetch_datasheets.py status
# only if the user asked to populate the cache:
python3 scripts/fetch_datasheets.py fetch
```

`status` is local-only. Some vendor portals need a browser save
into `pdf/` and then `fetch_datasheets.py convert`. SHA-256 of
the cached files is committed in
[resources/datasheets.sha256](resources/datasheets.sha256) for
later IPFS CIDv1.

## Bring-up order (official intent)

Vendor Arduino / M5Unified sequences. **Not measured.**

1. Device is already powered by the M5PM1 button. System I2C is
   GPIO47 SDA / GPIO48 SCL.
2. Talk to M5PM1 `0x6E`. L1, L2, and L3A come up with the PMIC.
3. Init M5IOE1 `0x4F`. Enable L3B rails as needed (EPD, touch,
   SD, PDM, LED G/B).
4. Isolate IP2315 (`PYG11_PWM3`) except for a short charge read.
5. EPD: `PYG3` 3.3 V, then `PYG5` reset; SPI GPIO14–18. Cold
   boot: OTP full refresh, not an invented LUT.
6. Touch: `PYG13` VDD, `PYG6` RST; FT6336G `0x38`; INT GPIO4.
7. Sensors on the same I2C: BMI270 `0x68`, RX8130CE `0x32`.
8. Full SKU only: ST25R3916 `0x50`, SX1262 on SPI1 after
   `LoRa_EN` (M5PM1 G2).

Factory UserDemo also exercises mic, SD, RGB, buzzer, Wi-Fi.

## Subsystem map

| Question | Read |
| --- | --- |
| What can destroy the board or irreplaceable data | [references/safety.md](references/safety.md) |
| How to read chip, flash, USB, factory image on your unit | [references/measure.md](references/measure.md) |
| Where keys, USB-C, and the SD slot sit | [references/enclosure.md](references/enclosure.md) |
| GPIO, expander pins, I2C, SPI, part numbers | [references/pin-map.md](references/pin-map.md) |
| M5PM1 rails, charger, sleep | [references/power-and-sleep.md](references/power-and-sleep.md) |
| Panel, orientation, OTP vs LUT | [references/display.md](references/display.md) |
| FT6336G address and active area | [references/touch.md](references/touch.md) |
| IMU, RTC, mic, RGB | [references/sensors.md](references/sensors.md) |
| Buttons, buzzer, SD, USB-C | [references/input-storage.md](references/input-storage.md) |
| USB, flash geometry, PSRAM | [references/flashing.md](references/flashing.md) |
| Rust stacks (not a host toolchain) | [references/rust.md](references/rust.md) |
| Vendor C++ / PlatformIO sequences | [references/cpp-platformio.md](references/cpp-platformio.md) |
| Official URLs, firmware list | [references/catalog.md](references/catalog.md) |
| Vendor datasheets (catalog; local cache) | [resources/datasheets.md](resources/datasheets.md) |
| Conflicts and citations | [references/sources.md](references/sources.md) |
| Measurement backlog | [resources/not-yet-confirmed.md](resources/not-yet-confirmed.md) |
| FreeInk PaperMono profile | [resources/external.md](resources/external.md) |

## Silicon defaults (official, not measured)

- **PSRAM:** 8 MB octal in the product table. Close
  [nyc-flash-id](resources/not-yet-confirmed.md#nyc-flash-id).
- **Flash:** 16 MB. PlatformIO uses `qio_opi` and
  `default_16MB.csv`. Runtime DIO vs QIO is
  [nyc-cpu-flash-runtime](resources/not-yet-confirmed.md#nyc-cpu-flash-runtime).
- **Canvas:** official 480×800. FreeInk uses 800×480. Conflict:
  [nyc-canvas-orient](resources/not-yet-confirmed.md#nyc-canvas-orient).
- **Wake:** M5PM1, not ESP32 `ext1` GPIO4 as on Sticky. IMU INT
  and RTC INT go to M5PM1 GPIOs.
- **Strapping (v2.2 §3):** GPIO0 (WPU, `BOOT_OUT`), GPIO3
  (floating, KEY2), GPIO45/46 (WPD, PDM). Latched at chip reset;
  ordinary IO after `tH` ≥ 3 ms.
- **JTAG pads:** GPIO39–42 default F0 is pad JTAG. Mux to GPIO
  before LoRa SPI or the buzzer.
- **Expander / PMIC GPIO:** default open-drain. Board M5IOE1
  is `0x4F`; chip UM samples `0x6F`–`0x76` from IO7.

## Do not

- Treat USB-C as a QinHeng CH343. This is not the Sticky.
- Drive GPIO45/46 as a power latch.
- Invent a four-gray LUT from a generic SSD1677 example.
- Keep IP2315 on the I2C bus after a charge read.
- Init NFC or LoRa on PaperMono-Lite.
- Overlap assumptions from reTerminal Sticky pin maps (GT911,
  BQ27220, BQ25616, CH343, 32 MB, GPIO7 shared INT).
- Use crate `bq27xxx` (this board has IP2315 + M5PM1, not a
  TI CEDV gauge).
- Invent registers or opcodes when the vendor PDF is unread. If
  the local datasheet cache is missing, ask the user to populate
  it.
- Treat a C++ file layout or PIO env as hardware, or treat
  `esp-hal` as the only legal Rust stack.
- Write a “Confirmed live” row without a board in hand, or
  copy a `C153` result onto `C153-Lite` (or the reverse).
