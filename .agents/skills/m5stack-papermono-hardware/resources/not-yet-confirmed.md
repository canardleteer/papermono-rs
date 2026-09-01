# Not yet confirmed

Measurement backlog for this product. **Closed items leave this
file**; the fact goes into the matching `references/` page. Do
not grow a confirmed-history section here.

**No PaperMono (`C153`) has been measured for this skill.**
PaperMono-Lite (`C153-Lite`) has run- and download-mode USB
IDs in [flashing.md](../references/flashing.md#usb-measured)
and a chip / 16 MB flash row in
[measure.md](../references/measure.md). Official docs,
UserDemo, OTP-Demo, and FreeInk do not close a row.
Firmware evidence is intent.

**Name the SKU.** Confirm on a physical unit of that variant.
A result on PaperMono does not confirm PaperMono-Lite, and the
reverse, unless you measured both.

Do not record another person’s MAC, serial, NVS, or flash image.
UART/USB geometry: [measure.md](../references/measure.md).

## How to close an item

1. Run the recipe (or read a schematic net that answers it).
   Record **which SKU** (`C153` or `C153-Lite`).
2. Write the result into the **Write the answer** target.
3. Delete the row and the recipe section from this file only
   when that SKU is settled. Keep the row if the other variant
   is still **not measured**.

Status is only `open`. If blocked on tools, say so in the recipe,
do not add a second status column.

A human must ask before any command opens a port.

## Index

| ID | Topic | Write the answer |
| --- | --- | --- |
| [nyc-flash-id](#nyc-flash-id) | JEDEC, flash size, PSRAM, eFuse, chip rev | [measure.md](../references/measure.md) |
| [nyc-usb-vid](#nyc-usb-vid) | Download IDs; `C153`; `probe-rs`; extra CDC | [flashing.md](../references/flashing.md) |
| [nyc-download-mode](#nyc-download-mode) | Hold-until-red-blink enters ROM download | [flashing.md](../references/flashing.md) |
| [nyc-cpu-flash-runtime](#nyc-cpu-flash-runtime) | Runtime CPU MHz and DIO vs QIO | [measure.md](../references/measure.md) |
| [nyc-partition-table](#nyc-partition-table) | Live 16 MB table vs UserDemo CSV | [flashing.md](../references/flashing.md) |
| [nyc-nvs-phy](#nyc-nvs-phy) | Per-unit PHY cal vs M5 restore image | [safety.md](../references/safety.md) |
| [nyc-gpio0-strap](#nyc-gpio0-strap) | `BOOT_OUT` during reset vs download | [pin-map.md](../references/pin-map.md) |
| [nyc-sleep-current](#nyc-sleep-current) | L0 / L1 / L2 / L3 currents | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-power-button](#nyc-power-button) | Short / double / hold timings | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-ip2315-bus](#nyc-ip2315-bus) | PYG11 isolate; low-VBAT hang | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-charge-stat](#nyc-charge-stat) | Charge current and done | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-pm1-wake](#nyc-pm1-wake) | IMU INT and RTC INT wake L1 | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-epd-spi-clock](#nyc-epd-spi-clock) | OTP-Demo / M5GFX clock vs 20 MHz | [display.md](../references/display.md) |
| [nyc-otp-busy](#nyc-otp-busy) | BUSY polarity; standby recovery | [display.md](../references/display.md) |
| [nyc-lut-path](#nyc-lut-path) | OTP 4-gray vs FreeInk host LUT | [display.md](../references/display.md) |
| [nyc-canvas-orient](#nyc-canvas-orient) | 480×800 vs 800×480 USB-C down | [display.md](../references/display.md) |
| [nyc-partial-ghost](#nyc-partial-ghost) | ~10 partials then full | [display.md](../references/display.md) |
| [nyc-panel-sheet](#nyc-panel-sheet) | `DEPG0397BBS770F3HP-XM` vs `epd-module` PDF | [datasheets.md](datasheets.md) |
| [nyc-ft6336-area](#nyc-ft6336-area) | 5–475 / 5–795 on a unit | [touch.md](../references/touch.md) |
| [nyc-ft6336-points](#nyc-ft6336-points) | Contacts this FPC delivers | [touch.md](../references/touch.md) |
| [nyc-i2c-ack](#nyc-i2c-ack) | Probe advertised addresses (full) | [pin-map.md](../references/pin-map.md) |
| [nyc-i2c-ack-lite](#nyc-i2c-ack-lite) | Same probe on Lite; NFC/LoRa NAK | [pin-map.md](../references/pin-map.md) |
| [nyc-pdm-mic](#nyc-pdm-mic) | Rate / slot / hole energy GPIO45/46 | [sensors.md](../references/sensors.md) |
| [nyc-sdmmc-width](#nyc-sdmmc-width) | 1-bit vs DAT0–DAT3 | [input-storage.md](../references/input-storage.md) |
| [nyc-tf-det](#nyc-tf-det) | Insert = 0 | [input-storage.md](../references/input-storage.md) |
| [nyc-lora-ack](#nyc-lora-ack) | SX1262 on full SKU | [pin-map.md](../references/pin-map.md) |
| [nyc-nfc-ack](#nyc-nfc-ack) | ST25R3916 `0x50` on full SKU | [pin-map.md](../references/pin-map.md) |
| [nyc-lite-nfc-pads](#nyc-lite-nfc-pads) | Lite GPIO6 / PYG4 NC vs routed | [pin-map.md](../references/pin-map.md) |
| [nyc-lite-lora-pads](#nyc-lite-lora-pads) | Lite SPI1 / IRQ NC vs routed | [pin-map.md](../references/pin-map.md) |
| [nyc-rgb-led](#nyc-rgb-led) | Red not PWM; G/B range | [sensors.md](../references/sensors.md) |
| [nyc-buzzer](#nyc-buzzer) | GPIO42 resonance | [input-storage.md](../references/input-storage.md) |
| [nyc-enclosure-edges](#nyc-enclosure-edges) | Power / A / B / USB-C / SD vs photos | [enclosure.md](../references/enclosure.md) |

Software knobs (CPU 160 vs 240 MHz as a *choice*, QIO vs DIO as
a *choice*) are not listed once runtime is recorded. Do not add
a row that asks anyone to invent an unread register map.

## Recipes

### nyc-flash-id

**Lite partial:** chip ESP32-S3 v0.2, crystal 40 MHz, flash
**16 MB**, secure boot and flash encryption disabled. Written
in [measure.md](../references/measure.md). Still need JEDEC
manufacturer bytes, PSRAM size (Features did not name it),
eFuse flash mode/voltage, and the same row on **`C153`**.

With the unit in download mode and a human ask, run `esptool.py
flash-id` / `espflash board-info`. Record **SKU**, chip rev,
crystal, flash JEDEC + size, PSRAM, eFuse mode and voltage.
Do not print MAC.

### nyc-usb-vid

**Lite run and download are written** (`303a:1001`, Espressif
USB JTAG/serial debug unit). Still need:

- **PaperMono (`C153`)** run and download.
- Whether a second CDC interface or UART0/CH343 appears
  (CH343 should not).
- `probe-rs list` on a unit.

Do not commit iSerial. Write
[flashing.md](../references/flashing.md).

### nyc-download-mode

**Lite:** power-button hold until the red LED blinks, then
`cargo xtask detect-connected --probe` (`NoReset`) returned
board-info. USB IDs did not change. Still need hold seconds
and which LED die blinks. `C153` not measured.
Write [flashing.md](../references/flashing.md).

### nyc-cpu-flash-runtime

Boot factory or UserDemo. Read UART/CDC log for CPU MHz and
flash mode (DIO/QIO). That is runtime, not eFuse.
[measure.md](../references/measure.md).

### nyc-partition-table

Read the table at `0x8000` from **that unit**. Diff against
UserDemo `partitions.csv` and PIO `default_16MB.csv`. Do not
commit the dump. [flashing.md](../references/flashing.md).

### nyc-nvs-phy

Does factory NVS contain PHY cal that M5 restore firmware does
not regenerate? Compare a snapshot of `nvs` / `phy_init` before
and after the official restore image (only if the owner accepts
the risk). [safety.md](../references/safety.md).

### nyc-gpio0-strap

Meter or scope GPIO0 vs `CHIP_PU` during a button reset and
during download entry. Is `BOOT_OUT` high, low, or Hi-Z at
strap sample? [pin-map.md](../references/pin-map.md).

### nyc-sleep-current

Meter battery current at L0 (`shutdown`), L1 retain, L2 (ESP
sleep), L3A+L3B idle with panel asleep. USB unplugged.
[power-and-sleep.md](../references/power-and-sleep.md).

### nyc-power-button

Stopwatch short / double / hold-to-download on a unit. Compare
to the “~2 s” note.
[power-and-sleep.md](../references/power-and-sleep.md).

### nyc-ip2315-bus

With VBAT low and USB connected, scan I2C with IP2315 mounted
vs isolated (`PYG11`). Confirm a hang and that isolate recovers
the bus. [power-and-sleep.md](../references/power-and-sleep.md).

### nyc-charge-stat

Read M5PM1 battery telemetry and, briefly, IP2315 while charging
on USB, then isolate. Record current and done indication.
[power-and-sleep.md](../references/power-and-sleep.md).

### nyc-pm1-wake

Run the official IMU-wake and RTC-timer-wake Arduino recipes.
Confirm ESP runs `setup()` again. Not a current measurement.
[power-and-sleep.md](../references/power-and-sleep.md).

### nyc-epd-spi-clock

Read OTP-Demo and M5GFX SPI Hz. If a human asks, log BUSY
timeouts at 10 MHz vs 20 MHz on a physical unit.
Name the SKU.
[display.md](../references/display.md).

### nyc-otp-busy

OTP-Demo: BUSY idle level, busy-during-refresh level, and
whether a Seeed-style analog-off standby recovers without a
hardware reset. [display.md](../references/display.md).

### nyc-lut-path

On a physical unit: OTP 4-gray quadrants (OTP-Demo) vs FreeInk
host LUT appearance. Name the SKU. Do not copy LUT bytes into
git.
[display.md](../references/display.md) and a
[sources.md](../references/sources.md) update.

### nyc-canvas-orient

USB-C down, paint known corners (OTP-Demo quadrants or a
labeled frame). Record whether firmware 480×800 or 800×480
matches ink. [display.md](../references/display.md).

### nyc-partial-ghost

Ten partials then a full, vs twenty partials with no full.
Operator observation of ghosting. Not a license to skip OTP.
[display.md](../references/display.md).

### nyc-panel-sheet

Search cached `epd-module.md` for `DEPG0397BBS770F3HP-XM` (or
the FPC silkscreen on a unit). If the PN is absent, leave the
gap. [datasheets.md](datasheets.md).

### nyc-ft6336-area

Tap outside 5–475 / 5–795 and on the inner rectangle. Confirm
the official shrink. [touch.md](../references/touch.md).

### nyc-ft6336-points

How many simultaneous contacts report? Public sheet says 1–2.
[touch.md](../references/touch.md).

### nyc-i2c-ack

Full SKU, system I2C scan after M5PM1/M5IOE1 init, IP2315
isolated: expect `0x32`, `0x38`, `0x4F`, `0x50`, `0x68`,
`0x6E`. `0x75` only while gated on.
[pin-map.md](../references/pin-map.md).

### nyc-i2c-ack-lite

Same scan on Lite. `0x50` must NAK. LoRa is SPI; confirm no
surprise ACK. [pin-map.md](../references/pin-map.md).

### nyc-pdm-mic

Enable `PYG12`, 16 kHz left (or whatever the vendor demo uses).
Log RMS/peak. Mute is a failed experiment, not a destroy row.
[sensors.md](../references/sensors.md).

### nyc-sdmmc-width

Identify a card at 1-bit and at 4-bit. Official DAT0–3 vs
FreeInk 1-bit. [input-storage.md](../references/input-storage.md).

### nyc-tf-det

Empty slot vs inserted: `PYG1` level. Official insert = 0.
[input-storage.md](../references/input-storage.md).

### nyc-lora-ack

Full SKU only. M5PM1 G2 enable, expander reset/ant, SPI
read of SX1262 status. [pin-map.md](../references/pin-map.md).

### nyc-nfc-ack

Full SKU only. `0x50` ACK after NFC rail. Do not leave RF on.
[pin-map.md](../references/pin-map.md).

### nyc-lite-nfc-pads

Lite V0.6.2 extract still names ST25R3916 and `PYB_NFC_EN`.
That is not DNP. Confirm GPIO6 / PYG4 NC vs routed vs
unpopulated on a unit or a PDF-page callout that the extract
dropped. [pin-map.md](../references/pin-map.md).

### nyc-lite-lora-pads

Lite V0.6.2 extract still names Stamp-LoRa-1262 and SPI1
nets. Confirm GPIO5/21/39–41, PYG2/10, M5PM1 G2 NC vs routed
vs unpopulated. [pin-map.md](../references/pin-map.md).

### nyc-rgb-led

Confirm red ignores PWM; sweep G/B. Download-mode blink is red.
[sensors.md](../references/sensors.md).

### nyc-buzzer

GPIO42 PWM sweep; note resonance. Optional SPL.
[input-storage.md](../references/input-storage.md).

### nyc-enclosure-edges

With a unit in hand, USB-C down, name power / A / B / SD / LED
vs the vendored photos. Write
[enclosure.md](../references/enclosure.md).
