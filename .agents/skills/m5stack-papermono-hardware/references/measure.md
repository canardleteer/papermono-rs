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

**PaperMono (`C153`) USB, JEDEC, and partition table have not
been measured.** Official HTML **M5GFX LUT Refresh Speed**
times are laboratory results for **PaperMono**, reference
only ([display.md](display.md)). They are not a row on this
page. PaperMono-Lite (`C153-Lite`) has run- and
download-mode USB IDs in
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
`C153` only. Lite extra CDC and `probe-rs list` are in
the table below. Lite download IDs match run mode
(`303a:1001`).

- Prefer a stable by-id node. ACM numbers move.
- The host user needs `dialout` (or equivalent). `monitor`
  also needs the usbfs udev rule in the papermono-rs skill
  [xtask.md](../../papermono-rs/references/xtask.md#usbfs-udev-for-monitor).
- Do not commit a USB serial string (Lite iSerial was
  MAC-shaped).

Download mode is a **power-button hold** until the red LED
blinks, then release. Confirmed on Lite: after that hold,
`detect-connected --probe` (`NoReset`) got board-info.
**Lite (2026-09-02):** ~2 s to first blink; the small red
next to the power button (`LED_EN_PP`), not the RGB
window. `C153` still
[nyc-download-mode](../resources/not-yet-confirmed.md#nyc-download-mode).
That is not Sticky DTR/RTS into a CH343.

```shell
# only if a human asked to talk to a device
lsusb
espflash board-info
esptool.py flash-id
```

Lite `probe-rs list` (2026-09-02, run mode) saw `EspJtag`.
Do not assume Sticky’s “no probes”. Do not commit the
probe serial.

## Confirmed live

Product-class results on a physical unit. Name the SKU in
every row. A `C153` result does not confirm `C153-Lite`.
Per-unit MAC, USB serial, and factory serial omitted.

| Item | SKU | Confirmed |
| --- | --- | --- |
| USB run mode | `C153-Lite` | `303a:1001` Espressif USB JTAG/serial debug unit; `bcdDevice` 1.01; full-speed. Serial omitted. 2026-09-02: three interfaces (CDC comm, CDC data, vendor JTAG `ff/ff/01`). One ACM. No second CDC. No `1a86:55d3`. [flashing.md](flashing.md#usb-measured) |
| `probe-rs list` | `C153-Lite` | 2026-09-02 run mode. `EspJtag` `303a:1001`. Probe serial MAC-shaped; omitted. [flashing.md](flashing.md#usb-measured) |
| USB download mode | `C153-Lite` | Same VID:PID and product string as run mode. `lsusb` `303a:1001` Espressif USB JTAG/serial debug unit. Serial omitted |
| Chip | `C153-Lite` | ESP32-S3 revision v0.2; crystal 40 MHz; features Wi-Fi, BLE, embedded flash. MAC omitted |
| Flash size | `C153-Lite` | 16 MB (`0x1000000`) from flasher `board-info`. JEDEC bytes not printed |
| Partition table | `C153-Lite` | At `0x8000`: nvs `0x9000`/`0x6000`, phy_init `0xf000`/`0x1000`, factory `0x10000`/`0xF00000`. Matches UserDemo `partitions.csv`. No `otadata`. PIO `default_16MB.csv` still different. `C153` open |
| Secure boot / flash encryption | `C153-Lite` | Both disabled (`SPI_BOOT_CRYPT_CNT` 0) |
| `simple-debug-fw` clocks | `C153-Lite` | CDC `hello`: `cpu_mhz=80` `xtal_mhz=40` (`esp-hal` `Config::default`). Not UserDemo. [nyc-cpu-flash-runtime](../resources/not-yet-confirmed.md#nyc-cpu-flash-runtime) |
| `embassy-debug-fw` hello | `C153-Lite` | 2026-09-01 run mode after `flash-app`. `image=embassy-debug` `sku=C153-Lite` `cpu_mhz=80` `xtal_mhz=40` `reset=chip_power_on`. 1 Hz `hb` idle `btn_a=1 btn_b=1`. Same idle `gpio` as `simple-debug` (`boot=1 pmic_irq=0 tp=0 ioe=1 busy=0`). First CDC attach glued `i2c` onto `hb` |
| I2C advertised roster | `C153-Lite` | 2026-09-02. Official `begin` at board `0x4F`. `ack=32,38,4f,68,6e nak=50,6f,75` `ioe_addr=4f` `imu_id=24` `rtc_flag=31` `tf=1` (empty slot; [nyc-tf-det](../resources/not-yet-confirmed.md#nyc-tf-det)). No `0x70`–`0x76` walk. `C153` still [nyc-i2c-ack](../resources/not-yet-confirmed.md#nyc-i2c-ack) |
| RX8130CE `FLAG` | `C153-Lite` | 2026-09-02. Read-only `0x1D`: CDC `rtc_flag=31`. Did not write `SEC`. Do not invent bit names. `C153` open. [sensors.md](sensors.md) |
| Stage B lamp + `EPD_VDD` | `C153-Lite` | 2026-09-02. PWM0 `lamp=1024` with `PYG3` off: human **lamp dark**. Same PWM0 after `PYG3` high (no `EPD_RST`, no OTP): human **lamp on**. Idle `gpio busy=1` (rail up, no refresh). [power-and-sleep.md](power-and-sleep.md) |
| FT `/INT` (GPIO4) | `C153-Lite` | 2026-09-01 attended taps. Idle `touch int=1`. Each contact: `int=0` then `int=1`. Active-low data-ready, matches UserDemo `ext0` GPIO4 low. During a slide the pad blips high (do not score “lift” on that). [touch.md](touch.md) |
| FT XY (M5GFX `getTouchRaw`) | `C153-Lite` | 2026-09-01 embassy-debug walk. Register-2 decode: official-portrait `x=`/`y=` matched drawn targets (dots within 2–22 px). Area 5–475 / 5–795 on the wire. `n=1` only. [touch.md](touch.md) |
| PDM mic (`PYG12` gated) | `C153-Lite` | 2026-09-01 Stage C. 80 ms window then `PYG12` parked. CDC `mic rms=1422 peak=12917` on 10 s `hello` (same pair at `t=10`/`t=20`/`t=30`: reprint). Earlier same-day reprint was `rms=1356 peak=12917`. **Unprompted idle** — no spoken/noise request. Firmware intent 16 kHz right. USB only. Superseded for hole energy by the PCM-dump row |
| PDM hole tone (PCM dump) | `C153-Lite` | 2026-09-01. Live `mic rms≈1370–1395` (`peak≈14029`, window-start spike). BUTTON A: `mic pcm hz=0 n=256` (board played nothing), two windows. Prefix `0` / spike / DC floor **−8**. Last ~48 samples: sine-like swing (mins ≈−3116 / −3571), period **~32–44** samples at 16 kHz (phone-band). Energy ~1500–1580 while the phone played A at the hole. Right slot. USB only. [nyc-pdm-mic](../resources/not-yet-confirmed.md#nyc-pdm-mic) |
| Frontlight (AW9967) | `C153-Lite` | 2026-09-01. Human: right-edge slide **drives brightness** after embassy-debug wrote M5PM1 G3 / **PWM0** (5 kHz, 12-bit). No meter, no %. Same day, PWM1 duty writes left the lamp **constant** (gutter still fired). One PWM into one AW9967; no warm/cool pair. `C153` open. [power-and-sleep.md](power-and-sleep.md) |
| Lamp gutter vs targets | `C153-Lite` | 2026-09-01. Official portrait `ACTIVE_MAX_X` 475, gutter 80 px → `x >= 395`. Targets first top-right is `(400, 80)` (also `(400, 720)`). Human: that tap no longer scored. Walk now scores slop before `LampSlide::feed`. Image on unit (150336 bytes); scored-hit not observed |
| OTP-Demo 4-gray (`otp_gray`) | `C153-Lite` | 2026-09-01. `panel mode=otp_gray w=800 h=480 busy_rose=1`. Human: four quadrants. OTP-Demo `GrayFull`. Not a LUT. Datasheet Mode 1 / `epd_text` stamp did not move glass. [nyc-otp-busy](../resources/not-yet-confirmed.md#nyc-otp-busy) / [nyc-lut-path](../resources/not-yet-confirmed.md#nyc-lut-path) |
| OTP canvas (`otp_orient`) | `C153-Lite` | 2026-09-01. USB-C down, buttons left. Corner bars: **1 bottom-left, 2 top-left, 3 bottom-right, 4 top-right**. OTP RAM X = physical Y (799 toward USB-C); RAM Y = physical X. [display.md](display.md). `C153` still [nyc-canvas-orient](../resources/not-yet-confirmed.md#nyc-canvas-orient) |
| OTP-Demo partial / mono | `C153-Lite` | 2026-09-01 target walk. `Partial` (`busy_rose=1`) advanced marks. White-vs-white `Partial` left ink. `MonoFull` both planes white: human **glass white**. Mono 1=white. [display.md](display.md) |
| `Partial` after `GrayFull` | `C153-Lite` | 2026-09-01 embassy-debug trial. Splash `GrayFull` then shapes `Partial` (no `MonoFull`). Human: flip was **fast**; Ferris stayed **until overdrawn**, not a faint ghost. OTP-Demo refuses this path. Not a destroy-the-panel test. `C153` unmeasured. [display.md](display.md) |
| USER_KEY polarity | `C153-Lite` | BUTTON A/B idle high, press low. [enclosure.md](enclosure.md) |
| Lite leftover MCU inputs | `C153-Lite` | 2026-09-02 packed image (no OTP). Idle CDC `leftover lora_irq=0 nfc_irq=0 sx_busy=0` (GPIO5/6/21, pull none, not driven). Same 10 s reprint through `t=40`. Not NC vs routed. [pin-map.md](pin-map.md) |
| Packed radio CDC | `C153-Lite` | 2026-09-02 `--features radio`. `wifi n=21` `ble n=418`. No BSSID/MAC/IRK. No connect. `C153` still [nyc-wifi-ble](../resources/not-yet-confirmed.md#nyc-wifi-ble) |
| Power button | `C153-Lite` | 2026-09-02 operator stopwatch. Short ~0.25 s resets (red off during reboot, solid red after). Hold ~2 s to first blink (small red `LED_EN_PP`, not RGB). Double-press gap too short to time; USB unplugged, lamp and red fully off; one short press turns it on. [power-and-sleep.md](power-and-sleep.md). `C153` still [nyc-power-button](../resources/not-yet-confirmed.md#nyc-power-button) |
| Charge (M5PM1 + gate) | `C153-Lite` | 2026-09-02 packed `--features touch,mic`, `flash-app` 127264 bytes. USB in. CDC `charge vbat=4198 vin=5012 src=05 chg_en=1 ip=0 then=0`. Roster `chg=0` `nak` includes `75`. Human: rear charge LED stayed red, no color change; front lamp on. No IP2315 current/done. [power-and-sleep.md](power-and-sleep.md). `C153` still [nyc-charge-stat](../resources/not-yet-confirmed.md#nyc-charge-stat) |
| RTC 10 s sleep/wake | `C153-Lite` | 2026-09-02 trial **stopped**. `--features sleep` (default off). USB-in `SYS_CMD` bounced. Unplug: lamp off, back in 2–3 s. CDC same boot `wake src=08`/`0a` `sleep abort` (not EXT). [power-and-sleep.md](power-and-sleep.md). `C153` still [nyc-pm1-wake](../resources/not-yet-confirmed.md#nyc-pm1-wake) |

## Factory image (Lite stock, measured table)

A `C153-Lite` full-chip capture (`backup-factory-firmware
--name stock-lite`, 2026-09-01) parsed the table at `0x8000`.
Offsets match UserDemo `partitions.csv`
([user-demo.md](user-demo.md)). The factory app descriptor
names project **PaperMono-UserDemo**, IDF **v5.5.1**, version
`c78f6c5-dirty`, compile date **Aug 6 2026**. That is older
than the GitHub V1.2 pin (`c109910`, 2026-08-10). Do not
treat shipping stock as that later commit.

The same day, `confirm-factory-firmware --capture stock-lite`
re-read the chip; live flash matched that capture (full
dump). After `restore-factory-firmware --yes --capture
stock-lite` the same day, confirm matched again and a
short-press power left the unit looking as before. Confirm
does not rewrite the snapshot. Do not commit dumps, NVS, PHY,
image SHA, or the confirm-records JSON.

`C153` table still
[nyc-partition-table](../resources/not-yet-confirmed.md#nyc-partition-table).

OTP path:
[M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo)
names panel **DEPG0397BBS770F3HP-XM**. Lite flash **size** is
measured (16 MB); JEDEC manufacturer bytes and PSRAM still
[nyc-flash-id](../resources/not-yet-confirmed.md#nyc-flash-id).

## What this page is not

- Not a pinout ([pin-map.md](pin-map.md)).
- Not permission to invent USB VID, JEDEC, or partition offsets.
- Not Sticky’s CH343 / 32 MB / `0x90000` notes.
