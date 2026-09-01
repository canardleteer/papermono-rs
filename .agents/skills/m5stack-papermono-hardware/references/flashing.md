# USB, flash, PSRAM

How to read a unit: [measure.md](measure.md). Layers:
[sources.md](sources.md). This page is geometry, not a host-tool
cheatsheet. Consuming projects supply their own flash path.

Name `C153` vs `C153-Lite`. USB run- and download-mode on Lite
are measured below. Flash **size** on Lite is measured (16 MB).
Other rows stay official intent until their `nyc-*` ids close.

| Item | Official / vendor intent | Open |
| --- | --- | --- |
| Flash size | 16 MB | Lite **measured** 16 MB (`0x1000000`). JEDEC bytes and `C153`: [nyc-flash-id](../resources/not-yet-confirmed.md#nyc-flash-id) |
| PSRAM | 8 MB octal; PIO `qio_opi` | Not in Lite `board-info` Features. [nyc-flash-id](../resources/not-yet-confirmed.md#nyc-flash-id) |
| USB | Native pads. Vendor Arduino: CDC on boot | Lite run **and** download: `303a:1001`. `C153`, `probe-rs`: [nyc-usb-vid](../resources/not-yet-confirmed.md#nyc-usb-vid) |
| Download | Power-button hold ~2 s, red LED blink | Lite: hold-until-red-blink then `--probe` (`NoReset`) worked. Seconds / which die: [nyc-download-mode](../resources/not-yet-confirmed.md#nyc-download-mode) |
| Partition table | UserDemo `partitions.csv`; PIO `default_16MB.csv` | Lite **measured** matches UserDemo CSV (nvs `0x9000`/`0x6000`, phy `0xf000`/`0x1000`, factory `0x10000`/`0xF00000`). `C153`: [nyc-partition-table](../resources/not-yet-confirmed.md#nyc-partition-table) |
| Runtime DIO/QIO, CPU MHz | Capability 240 MHz. UserDemo `sdkconfig.defaults`: `CONFIG_ESP_DEFAULT_CPU_FREQ_MHZ_240`, octal SPIRAM, 16 MB flash | [nyc-cpu-flash-runtime](../resources/not-yet-confirmed.md#nyc-cpu-flash-runtime) |
| PHY / NVS | ESP32-S3 typically stores RF cal in NVS; M5 publishes restore images | [nyc-nvs-phy](../resources/not-yet-confirmed.md#nyc-nvs-phy) |

Lite measured flash length: `0x1000000`. Lite stock partition
table at `0x8000` matches UserDemo `partitions.csv` (not PIO
`default_16MB.csv`). Do not copy Sticky `0x2000000` / `0x90000`.

Keep `*.bin` flash images out of git. Do not restore one unit’s
full-chip image onto another until you know NVS contents are not
identity.

## USB (measured)

**PaperMono-Lite (`C153-Lite`), powered on (run mode).** Host
`dmesg` / `lsusb` from a live unit. USB serial string omitted
(it was a MAC-shaped iSerial; do not commit one).

| Field | Value |
| --- | --- |
| SKU | `C153-Lite` |
| Mode | Run (unit on; not download-button) |
| VID:PID | `303a:1001` |
| bcdDevice | `1.01` |
| Manufacturer | Espressif |
| Product | USB JTAG/serial debug unit |
| Speed | Full-speed |
| QinHeng CH343 (`1a86:55d3`) | Not in that log |

**PaperMono-Lite (`C153-Lite`), download mode** (power-button
hold until red LED blinks). Same host `lsusb` / xtask inventory
as run mode. Serial omitted.

| Field | Value |
| --- | --- |
| SKU | `C153-Lite` |
| Mode | Download (red LED blinking) |
| VID:PID | `303a:1001` |
| Manufacturer | Espressif |
| Product | USB JTAG/serial debug unit |
| QinHeng CH343 (`1a86:55d3`) | Not present |

This is native Espressif USB-Serial/JTAG on USB-C, not a
CH343. Vendor Arduino `USB_CDC_ON_BOOT` is still **intent**;
run and download enumeration used the JTAG/serial product
string. `C153`, whether a second CDC interface appears, and
`probe-rs list` stay
[nyc-usb-vid](../resources/not-yet-confirmed.md#nyc-usb-vid).

Prefer a stable by-id node. ACM numbers move. The host user
needs `dialout` (or equivalent). On Lite, udev by-id embeds the
USB iSerial (MAC-shaped). Host `detect-connected` redacts that
token; do not commit a by-id path.

`303a:1001` is also the usual ESP32-S3 DevKit USB-Serial/JTAG
id. Inventory cannot tell a DevKit from PaperMono. If more than
one such device is plugged in, pass `--port`. Host `classify()`
will treat any `303a:*` as this product when PID is missing from
sysfs.

## PSRAM

Octal 8 MB in the product table. 80 MHz is a firmware config,
not an eFuse field. Gray4 frames belong in PSRAM once size is
confirmed. DMA descriptors stay in internal RAM.

## Do not

- Use Sticky 32 MB `n16r8` notes (that was the *wrong* CrossPoint
  limit on a 32 MB Sticky). Here 16 MB is the documented size.
- Assume `probe-rs` works because the product string says
  JTAG. Run `probe-rs list` on a unit first (`nyc-usb-vid`).
- `erase-flash` before you have a snapshot you accept losing PHY
  for (`nyc-nvs-phy`).
