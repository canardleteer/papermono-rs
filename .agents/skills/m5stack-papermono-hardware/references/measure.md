# Measure silicon on a unit

Observed hardware on a **real PaperMono (`C153`) or
PaperMono-Lite (`C153-Lite`)** outranks SDK pin sheets and
compiled profiles that were never run on that SKU. Name the
variant. A result on one does not confirm the other. Layers
and conflicts:
[sources.md](sources.md). The skill user weighs disagreements.

This page records **what was seen on silicon**. It does **not**
include GPIO numbers — those still come from firmware pin maps
and official tables until a recipe closes them. Do not record
another person’s MAC, serial number, USB serial string, NVS, or
flash image into shared docs.

Consuming projects supply their own host capture tools. Do not
open a port unless a human asked.

Vendor C++ trees are wiring evidence in
[cpp-platformio.md](cpp-platformio.md).

**PaperMono (`C153`) has not been measured.** PaperMono-Lite
(`C153-Lite`) has run- and download-mode USB IDs in
[flashing.md](flashing.md#usb-measured) and a `--probe`
board-info row below. Remaining recipes:
[not-yet-confirmed.md](../resources/not-yet-confirmed.md).

## Find the USB device

**Lite, run mode:** `303a:1001` Espressif “USB JTAG/serial
debug unit” (`bcdDevice` 1.01, full-speed). Details:
[flashing.md](flashing.md#usb-measured). Vendor Arduino CDC
flags remain **intent**. Do not treat USB-C as QinHeng
`1a86:55d3`.

Still open ([nyc-usb-vid](../resources/not-yet-confirmed.md#nyc-usb-vid)):
`C153`, `probe-rs list`, extra CDC. Lite download IDs match
run mode (`303a:1001`).

- Prefer a stable by-id node. ACM numbers move.
- The host user needs `dialout` (or equivalent).
- Do not commit a USB serial string (Lite iSerial was
  MAC-shaped).

Download mode is a **power-button hold** until the red LED
blinks, then release. Confirmed on Lite: after that hold,
`detect-connected --probe` (`NoReset`) got board-info.
Hold seconds and which red die still
[nyc-download-mode](../resources/not-yet-confirmed.md#nyc-download-mode).
That is not Sticky DTR/RTS into a CH343.

```shell
# only if a human asked to talk to a device
lsusb
espflash board-info
esptool.py flash-id
```

`probe-rs list` may or may not see a probe on this connector.
Record the result under `nyc-usb-vid`; do not assume Sticky’s
“no probes”.

## Confirmed live

Product-class results on a physical unit. Name the SKU in
every row. A `C153` result does not confirm `C153-Lite`.
Per-unit MAC, USB serial, and factory serial omitted.

| Item | SKU | Confirmed |
| --- | --- | --- |
| USB run mode | `C153-Lite` | `303a:1001` Espressif USB JTAG/serial debug unit; `bcdDevice` 1.01; full-speed. Serial omitted. [flashing.md](flashing.md#usb-measured) |
| USB download mode | `C153-Lite` | Same VID:PID and product string as run mode. `lsusb` `303a:1001` Espressif USB JTAG/serial debug unit. Serial omitted |
| Chip | `C153-Lite` | ESP32-S3 revision v0.2; crystal 40 MHz; features Wi-Fi, BLE, embedded flash. MAC omitted |
| Flash size | `C153-Lite` | 16 MB (`0x1000000`) from flasher `board-info`. JEDEC bytes not printed |
| Secure boot / flash encryption | `C153-Lite` | Both disabled (`SPI_BOOT_CRYPT_CNT` 0) |

## Factory image (intent only)

[M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)
is published ESP-IDF source for hardware evaluation. Sequences
and the CSV live in
[user-demo.md](user-demo.md). `partitions.csv` is **vendor
intent**, not a live dump of a shipping unit
([nyc-partition-table](../resources/not-yet-confirmed.md#nyc-partition-table)).

OTP path:
[M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo)
names panel **DEPG0397BBS770F3HP-XM**. Lite flash **size** is
measured (16 MB); JEDEC manufacturer bytes and PSRAM still
[nyc-flash-id](../resources/not-yet-confirmed.md#nyc-flash-id).

## What this page is not

- Not a pinout ([pin-map.md](pin-map.md)).
- Not permission to invent USB VID, JEDEC, or partition offsets.
- Not Sticky’s CH343 / 32 MB / `0x90000` notes.
