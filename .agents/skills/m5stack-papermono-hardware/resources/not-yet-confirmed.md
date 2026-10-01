# Not yet confirmed

Measurement backlog for this product. **Closed items leave this
file**; the fact goes into the matching `references/` page. Do
not grow a confirmed-history section here.

Both PaperMono (`C153`) and PaperMono-Lite (`C153-Lite`) have
live measurements in [measure.md](../references/measure.md),
including USB, flash size, and factory partition tables. C153 NFC
and LoRa results are also recorded there. This file lists remaining
questions; source documentation and firmware intent do not close
electrical measurements.

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

When a human accepts a firmware flash, pack every safe
unattended probe into that image (root `AGENTS.md`,
**Pack one flash**). Do not spend a boot on I2C alone
if lamp, `FLAG`, leftover inputs, or (when asked)
`wifi n=` / `ble n=` can ride along. Packed listen-only scan
counts stay ask-first. The default `embassy-debug` image
includes interactive BLE / Wi-Fi cards; they are idle until
touch. No foreign MAC / BSSID / IRK. No NVS write.

Rule: a new named part in the SKILL product snapshot gets an
NYC row in the same change.

## Chip surface coverage

Every named IC has at least one functional NYC (not ACK-only).
Shared-chip recipes need separate evidence per SKU. Do **not**
initialize NFC/LoRa on Lite.

| Surface | SKU | NYC |
| --- | --- | --- |
| ESP32-S3R8 (USB, flash, straps) | C153, C153-Lite | [nyc-flash-id](#nyc-flash-id), [nyc-usb-vid](#nyc-usb-vid), [nyc-cpu-flash-runtime](#nyc-cpu-flash-runtime), [nyc-gpio0-strap](#nyc-gpio0-strap), [nyc-nvs-phy](#nyc-nvs-phy) |
| Octal PSRAM | C153, C153-Lite | [nyc-flash-id](#nyc-flash-id) |
| Wi-Fi 2.4 GHz + BLE | C153, C153-Lite | [nyc-wifi-ble](#nyc-wifi-ble) |
| SSD1677 | C153, C153-Lite | [nyc-epd-spi-clock](#nyc-epd-spi-clock), [nyc-otp-busy](#nyc-otp-busy), [nyc-lut-path](#nyc-lut-path), [nyc-canvas-orient](#nyc-canvas-orient), [nyc-partial-ghost](#nyc-partial-ghost), [nyc-panel-sheet](#nyc-panel-sheet) |
| FT6336G | C153, C153-Lite | [nyc-ft6336-area](#nyc-ft6336-area), [nyc-ft6336-points](#nyc-ft6336-points) |
| M5PM1 | C153, C153-Lite | [nyc-sleep-current](#nyc-sleep-current), [nyc-power-button](#nyc-power-button), [nyc-pm1-wake](#nyc-pm1-wake), [nyc-gpio0-strap](#nyc-gpio0-strap) |
| AW9967 frontlight (PWM0) | C153, C153-Lite | [nyc-frontlight](#nyc-frontlight) |
| M5IOE1 | C153, C153-Lite | leftover-pad rows; full roster [nyc-i2c-ack](#nyc-i2c-ack) |
| IP2315 | C153, C153-Lite | [nyc-ip2315-bus](#nyc-ip2315-bus), [nyc-charge-stat](#nyc-charge-stat) |
| BMI270 | C153, C153-Lite | [nyc-bmi270](#nyc-bmi270); INT [nyc-pm1-wake](#nyc-pm1-wake) |
| RX8130CE | C153, C153-Lite | [nyc-rx8130](#nyc-rx8130); INT [nyc-pm1-wake](#nyc-pm1-wake) |
| LMD4737 PDM | C153, C153-Lite | [nyc-pdm-mic](#nyc-pdm-mic) |
| Buzzer GPIO42 | C153, C153-Lite | [nyc-buzzer](#nyc-buzzer) |
| RGB LED | C153, C153-Lite | [nyc-rgb-led](#nyc-rgb-led) |
| microSD | C153, C153-Lite | [nyc-sdmmc-width](#nyc-sdmmc-width), [nyc-tf-det](#nyc-tf-det) |
| ST25R3916 | C153 | Confirmed live; Lite pads: [nyc-lite-nfc-pads](#nyc-lite-nfc-pads) |
| SX1262 die | C153 | Confirmed live; open work is listed under radio reception and safety recipes |
| Stamp LoRa-1262 module | C153 | Confirmed live; Lite pads: [nyc-lite-lora-pads](#nyc-lite-lora-pads) |
| Full I2C roster incl. `0x50` | C153 | [nyc-i2c-ack](#nyc-i2c-ack) |

## Index

| ID | Topic | Write the answer |
| --- | --- | --- |
| [nyc-flash-id](#nyc-flash-id) | JEDEC, flash size, PSRAM, eFuse, chip rev | [measure.md](../references/measure.md) |
| [nyc-usb-vid](#nyc-usb-vid) | Download IDs; `C153`; `probe-rs`; extra CDC | [flashing.md](../references/flashing.md) |
| [nyc-cpu-flash-runtime](#nyc-cpu-flash-runtime) | Runtime CPU MHz and DIO vs QIO | [measure.md](../references/measure.md) |
| [nyc-nvs-phy](#nyc-nvs-phy) | Per-unit PHY cal vs M5 restore image | [safety.md](../references/safety.md) |
| [nyc-wifi-ble](#nyc-wifi-ble) | On-unit Wi-Fi **and** BLE scan counts | [measure.md](../references/measure.md) |
| [nyc-gpio0-strap](#nyc-gpio0-strap) | `BOOT_OUT` during reset vs download | [pin-map.md](../references/pin-map.md) |
| [nyc-sleep-current](#nyc-sleep-current) | L0 / L1 / L2 / L3 currents | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-power-button](#nyc-power-button) | Short / double / hold timings | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-ip2315-bus](#nyc-ip2315-bus) | PYG11 isolate; low-VBAT hang | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-charge-stat](#nyc-charge-stat) | Charge current and done | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-pm1-wake](#nyc-pm1-wake) | IMU INT and RTC INT wake L1 | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-frontlight](#nyc-frontlight) | PWM0 duty vs visible lamp (AW9967) | [power-and-sleep.md](../references/power-and-sleep.md) |
| [nyc-epd-spi-clock](#nyc-epd-spi-clock) | OTP-Demo / M5GFX clock vs 20 MHz | [display.md](../references/display.md) |
| [nyc-otp-busy](#nyc-otp-busy) | BUSY polarity; standby recovery | [display.md](../references/display.md) |
| [nyc-lut-path](#nyc-lut-path) | OTP 4-gray vs FreeInk host LUT | [display.md](../references/display.md) |
| [nyc-canvas-orient](#nyc-canvas-orient) | 480×800 vs 800×480 USB-C down | [display.md](../references/display.md) |
| [nyc-partial-ghost](#nyc-partial-ghost) | ~10 partials then full | [display.md](../references/display.md) |
| [nyc-panel-sheet](#nyc-panel-sheet) | `DEPG0397BBS770F3HP-XM` vs `epd-module` PDF | [datasheets.md](datasheets.md) |
| [nyc-ft6336-area](#nyc-ft6336-area) | 5–475 / 5–795 (`C153` still) | [touch.md](../references/touch.md) |
| [nyc-ft6336-points](#nyc-ft6336-points) | Contacts this FPC delivers | [touch.md](../references/touch.md) |
| [nyc-i2c-ack](#nyc-i2c-ack) | Probe advertised addresses (full) | [pin-map.md](../references/pin-map.md) |
| [nyc-bmi270](#nyc-bmi270) | `CHIP_ID` payload `0x24` | [sensors.md](../references/sensors.md) |
| [nyc-rx8130](#nyc-rx8130) | Read `FLAG` `0x1D`; do not write `SEC` | [sensors.md](../references/sensors.md) |
| [nyc-pdm-mic](#nyc-pdm-mic) | Rate / slot / hole energy GPIO45/46 | [sensors.md](../references/sensors.md) |
| [nyc-sdmmc-width](#nyc-sdmmc-width) | 1-bit vs DAT0–DAT3 | [input-storage.md](../references/input-storage.md) |
| [nyc-tf-det](#nyc-tf-det) | Insert = 0 | [input-storage.md](../references/input-storage.md) |
| [nyc-lite-nfc-pads](#nyc-lite-nfc-pads) | Lite GPIO6 / PYG4 NC vs routed | [pin-map.md](../references/pin-map.md) |
| [nyc-lite-lora-pads](#nyc-lite-lora-pads) | Lite SPI1 / IRQ NC vs routed | [pin-map.md](../references/pin-map.md) |
| [nyc-rgb-led](#nyc-rgb-led) | Red not PWM; G/B range | [sensors.md](../references/sensors.md) |
| [nyc-buzzer](#nyc-buzzer) | GPIO42 resonance | [input-storage.md](../references/input-storage.md) |
| [nyc-enclosure-edges](#nyc-enclosure-edges) | Power / BUTTON A (UP) / BUTTON B (DOWN) / USB-C / SD vs photos | [enclosure.md](../references/enclosure.md) |

Software knobs (CPU 160 vs 240 MHz as a *choice*, QIO vs DIO as
a *choice*) are not listed once runtime is recorded. Do not add
a row that asks anyone to invent an unread register map.

## Recipes

### nyc-flash-id

**Lite and C153 confirmed:** chip ESP32-S3 v0.2, crystal 40 MHz, flash
**16 MB**, secure boot and flash encryption disabled. Written
in [measure.md](../references/measure.md). Still need JEDEC
manufacturer bytes, PSRAM size (Features did not name it),
and eFuse flash mode/voltage.

With the unit in download mode and a human ask, run `esptool.py
flash-id` / `espflash board-info`. Record **SKU**, chip rev,
crystal, flash JEDEC + size, PSRAM, eFuse mode and voltage.
Do not print MAC.

### nyc-usb-vid

**Lite and C153 run and download are written** (`303a:1001`, Espressif
USB JTAG/serial debug unit). Lite run (2026-09-02): one
ACM + vendor JTAG; no second CDC; no CH343;
`probe-rs list` → `EspJtag`. C153 run and download mode confirmed
identical (`303a:1001`, 2026-09-05). Do not commit iSerial. Write
[flashing.md](../references/flashing.md).

### nyc-cpu-flash-runtime

**Lite partial (`simple-debug-fw`, 2026-09-01):** CDC hello
`cpu_mhz=80` `xtal_mhz=40`. That is this image’s
`Config::default()`, not UserDemo. Still need UserDemo / stock
CDC for CPU MHz and DIO vs QIO, and the same on **`C153`**.
UserDemo `sdkconfig.defaults` asks for 240 MHz and octal
SPIRAM; that is intent, not a log.
[measure.md](../references/measure.md).

### nyc-nvs-phy

Does factory NVS contain PHY cal that M5 restore firmware does
not regenerate? Compare a snapshot of `nvs` / `phy_init` before
and after the official restore image (only if the owner accepts
the risk). [safety.md](../references/safety.md).

### nyc-wifi-ble

On-unit 2.4 GHz Wi-Fi **and** BLE scan in one recipe. CDC
counts only (`wifi n=` / `ble n=`). No BSSID, MAC, or IRK.
Close per SKU. C153 scan counts remain unmeasured. Official HTML
“2.4 GHz Wi-Fi”, UserDemo’s Wi-Fi app, and silicon BLE in
board-info do **not** close this. A Wi-Fi-only result leaves
BLE open under this same id. Interactive cards remain idle
until selected; packed scan counts require an explicit ask.

**Lite written (2026-09-02):** listen-only
`wifi n=21` `ble n=418` in
[measure.md](../references/measure.md). No connect.
C153 remains unmeasured for this item.
[docs/CRATES.md](../../../../docs/CRATES.md).

**Lite BLE pairing (2026-09-04):** `embassy-debug` live.
Smartphone connects to `PaperMono`: `pair state=connected`.
DisplayOnly passkey generated and displayed on card / CDC:
passkey value omitted. Panel fast partial refreshes. Passkey entered on phone:
pairing completes with `pair ok`, panel displays paired banner.
Bonds held in RAM. C153 remains unmeasured for this item.

**Lite SoftAP / survey cards (2026-09-04):** `embassy-debug`
live SoftAP confirmed on `C153-Lite`. Touch `[ START HOTSPOT ]`:
CDC `wifi_ap state=active ssid=PaperMono-AP pass=mono2026
ip=192.168.4.1`. A host Wi-Fi adapter saw WPA2 ch1, connected,
DHCP `192.168.4.50`. `curl http://192.168.4.1/` JSON included
battery and `wifi.requests`. CDC `client=connected` and
`wifi_http req=`. Survey card + mutual exclusion written
(channels 1–13, top 4 APs on glass, CDC counts only); SoftAP
path host-verified. WPA3/SAE unavailable in precompiled
`esp-radio` blob (not a future NYC close — use WPA2). No
foreign MAC/BSSID/IRK. C153 remains unmeasured for this item.
[measure.md](../references/measure.md).

### nyc-gpio0-strap

Meter or scope GPIO0 vs `CHIP_PU` during a button reset and
during download entry. Is `BOOT_OUT` high, low, or Hi-Z at
strap sample? [pin-map.md](../references/pin-map.md).

### nyc-sleep-current

Meter **battery** current at L0 (`shutdown`), L1 retain, L2
(ESP sleep), L3A+L3B idle with panel asleep. USB unplugged.
Needs a current meter on the battery path (case open or a
battery lead). A USB wattmeter does not close this.
[power-and-sleep.md](../references/power-and-sleep.md).

### nyc-power-button

Stopwatch short / double / hold-to-download on a unit. Compare
to the “~2 s” note.

**Lite written (2026-09-02, 2026-09-03):** short ~0.25 s
resets (red off during reboot, solid red after; also cleanly
resets when the MCU is in light sleep). Hold to first blink
~2 s (same red die). Double-press gap too short to time;
USB unplugged, lamp and red go fully off; one short press
turns the unit back on. C153 remains unmeasured for this item.
[power-and-sleep.md](../references/power-and-sleep.md).

### nyc-ip2315-bus

With VBAT low and USB connected, scan I2C with IP2315 mounted
vs isolated (`PYG11`). Confirm a hang and that isolate recovers
the bus. [power-and-sleep.md](../references/power-and-sleep.md).

### nyc-charge-stat

Read M5PM1 battery telemetry and, briefly, IP2315 while charging
on USB, then isolate. Record current and done indication.

**Lite partial (2026-09-02):** USB in. M5PM1
`vbat=4198` `vin=5012` `src=05` `chg_en=1` (battery +
5VIN valid, charge enable on). Gated `PYG11` high: IP2315
did not ACK (`ip=0`). After park: still NAK (`then=0`,
roster `chg=0`). Human: rear charge LED stayed red, no
color change; front lamp on. No IP2315 current or done
register (chip stayed off the bus). Live UI telemetry
(2026-09-03) verified on Legend card: M5PM1 `vbat=4190`, `vin=5030`,
`battery_percent` mapping 3300..4150 mV with 60 s auto-refresh and IP2315
parked. Battery drain rate and IP2315 registers remain open.
C153 remains unmeasured for this item.
[power-and-sleep.md](../references/power-and-sleep.md),
[measure.md](../references/measure.md).

### nyc-pm1-wake

Run the official IMU-wake and RTC-timer-wake Arduino recipes,
or UserDemo `enterImuWakeShutdown` / `enterRtc10sWakeShutdown`.
Confirm ESP runs `setup()` / `app_main` again. UserDemo also
has ESP `ext0` GPIO4 touch deep sleep; that is a different
path. Not a current measurement.

**Partial (`C153-Lite`, 2026-09-02, trial stopped):** USB-in
`SYS_CMD` bounced (sheet 5VIN recovery). Unplug after
lamp-on: human lamp off, then on again in **2–3 s**. CDC
same boot: `wake src=08` / `0a`, `sleep abort`. Not a 10 s
RTC GPIO wake. In `embassy-debug-fw`, sleep is implemented as
interactive ESP32-S3 light sleep with low-power GPIO2/GPIO3 button
wake (2026-09-03). IMU / touch `ext0` still open. C153 remains
unmeasured for this item.
[power-and-sleep.md](../references/power-and-sleep.md),
[user-demo.md](../references/user-demo.md).

### nyc-frontlight

PWM0 duty vs visible lamp (M5PM1 G3 into AW9967). Do not invent
an AW9967 register map. PWM1 writes are the wrong channel.

**Partial (2026-09-01, `C153-Lite`):** right-edge slide
**changes brightness** with PWM0 at 5 kHz 12-bit. Same day,
PWM1 left the lamp constant.
**Lite (2026-09-02, rail verification):** PWM0 `lamp=1024` needs
`PYG3` (`EPD_VDD`) or the lamp stays dark. No `EPD_RST`.
[power-and-sleep.md](../references/power-and-sleep.md),
[measure.md](../references/measure.md). C153 remains unmeasured for this item.

### nyc-epd-spi-clock

Read OTP-Demo and M5GFX SPI Hz. If a human asks, log BUSY
timeouts at 10 MHz vs 20 MHz on a physical unit.
Name the SKU.
[display.md](../references/display.md).

### nyc-otp-busy

OTP-Demo: BUSY idle level, busy-during-refresh level, and
whether a Seeed-style analog-off standby recovers without a
hardware reset. Factory demo analog-off sequence is named in
[user-demo.md](../references/user-demo.md); do not treat that
as a measured recovery.
[display.md](../references/display.md).

**Partial (2026-09-01, `C153-Lite`):** First panel test stamp
(`0xC7`, 480×800) never raised BUSY. Later same day OTP-Demo
`0xD7` / 800×480: CDC `busy_rose=1`, idle `gpio busy=0`.
Same day: partial `0xFF` and mono `0xF8`/`0x14` also
`busy_rose=1`. Analog-off recovery still untested. Do not
invent `0x32`.

### nyc-lut-path

On a physical unit: OTP 4-gray quadrants (OTP-Demo) vs FreeInk
host LUT appearance. Name the SKU. Do not copy LUT bytes into
git.
[display.md](../references/display.md) and a
[sources.md](../references/sources.md) update.

**Partial (2026-09-01, `C153-Lite`):** OTP-Demo 4-gray ran
(`busy_rose=1`). Flashlight: four quadrants (two darker
levels on the right were easy to miss). Same day: OTP
partial `0xFF` and mono full `0x14` (no `0x32`). No host
LUT. FreeInk compare still open.

### nyc-canvas-orient

USB-C down, paint known corners (OTP-Demo quadrants or a
labeled frame). Record whether firmware 480×800 or 800×480
matches ink. [display.md](../references/display.md).

**Partial (2026-09-01, `C153-Lite` settled; `C153` open):**
USB-C down, `otp_orient` bars: 1 bottom-left, 2 top-left, 3
bottom-right, 4 top-right. OTP RAM X = physical Y, RAM Y =
physical X. Write-up:
[display.md](../references/display.md). Do not copy onto
`C153`.

### nyc-partial-ghost

Ten partials then a full, vs twenty partials with no full.
Operator observation of ghosting. Not a license to skip OTP.
[display.md](../references/display.md).

**Partial (2026-09-01, `C153-Lite`):** ~7 OTP `0xFF`
partials then `refresh_mono_full` (`0xF8`/`0x14`) cleared
to white. Not a 10-vs-20 ghost count.

### nyc-panel-sheet

Search cached `epd-module.md` for `DEPG0397BBS770F3HP-XM` (or
the FPC silkscreen on a unit). If the PN is absent, leave the
gap. [datasheets.md](datasheets.md).

### nyc-ft6336-area

Tap outside 5–475 / 5–795 and on the inner rectangle. Confirm
the official shrink. [touch.md](../references/touch.md).

**Partial (2026-09-01, `C153-Lite` settled; `C153` open):**
Midline slides printed `x=5`/`x=475` and `y=5`/`y=795`.
Write-up: [touch.md](../references/touch.md). Do not copy
onto `C153`.

### nyc-ft6336-points

How many simultaneous contacts report? Public sheet says 1–2.
[touch.md](../references/touch.md).

**Partial (2026-09-01, `C153-Lite`):** one-finger walk only
(`n=1`). Two-point on this FPC still untested.

### nyc-i2c-ack

C153, system I2C scan after M5PM1/M5IOE1 init, IP2315
isolated: expect `0x32`, `0x38`, `0x4F`, `0x50` (when M5IOE1
`PYG4` / `IOE1_ENABLE` is driven high), `0x68`, `0x6E`.
`0x75` only while gated on via `PYG11`.
[pin-map.md](../references/pin-map.md).

**Lite written (2026-09-02):** `ack=32,38,4f,68,6e`
`nak=50,6f,75` in [measure.md](../references/measure.md).
Do not copy that NAK list onto `C153` (`0x50` must ACK
when gated on).

### nyc-bmi270

Read BMI270 `CHIP_ID` (register `0x00`). Payload is `0x24`.
Do not invent a FIFO map. INT wake stays
[nyc-pm1-wake](#nyc-pm1-wake).

**Partial (2026-09-01, `C153-Lite`):** CDC `imu_id=24` in
[measure.md](../references/measure.md). Optional later: a
sample. C153 remains unmeasured for this item. [sensors.md](../references/sensors.md).

### nyc-rx8130

Read RX8130CE `FLAG` (`0x1D`). Do not write `SEC` (that
clears the sub-second chain). INT wake stays
[nyc-pm1-wake](#nyc-pm1-wake). ACK at `0x32` is not enough.
[sensors.md](../references/sensors.md).

**Partial (2026-09-02, `C153-Lite`):** CDC `rtc_flag=31`
(`UF|TF|VBFF` from catalog `rx8130ce` Flag Register).
Did not write `SEC`. C153 remains unmeasured for this item.

### nyc-pdm-mic

Enable `PYG12`, 16 kHz right (UserDemo `hal_mic.cpp`). Log
RMS/peak. Mute is a failed experiment, not a destroy row.
[sensors.md](../references/sensors.md).

**Partial (2026-09-01, `C153-Lite`):** 16 kHz right, live
energy, and a through-hole phone-A dump (`hz=0`, period
~32–44 samples) are in
[measure.md](../references/measure.md) /
[sensors.md](../references/sensors.md). Still open: slot
A/B, hole vs waveform polarity, `PYG12` settle to first
valid window, `C153`.

### nyc-sdmmc-width

Identify a card at 1-bit and at 4-bit. Official DAT0–3 and
UserDemo 4-bit vs FreeInk 1-bit.
[input-storage.md](../references/input-storage.md).

### nyc-tf-det

Empty slot vs inserted: `PYG1` level. Official insert = 0.
[input-storage.md](../references/input-storage.md).

### nyc-lite-nfc-pads

Lite HTML **PinMap** and SKU compare omit NFC. Lite V0.6.2
gallery **page 05** / PDF still **draws** ST25R3916 and
`PYB_NFC_EN`. That is not DNP. Confirm GPIO6 / PYG4 NC vs
routed vs unpopulated on a unit or a PDF-page callout that
the extract dropped. [pin-map.md](../references/pin-map.md).

**Partial (2026-09-02, desk):** Lite page-05 extract:
`PYB_NFC_EN is a spare GPIO pin`. Block still drawn.
`0x50` NAK already on the wire. Idle GPIO6
`nfc_irq=0` (input, no pull). Not a DNP close.

### nyc-lite-lora-pads

Lite HTML **PinMap** and SKU compare omit LoRa. Lite V0.6.2
gallery **page 05** / PDF still **draws** Stamp-LoRa-1262 and
SPI1 nets. Confirm GPIO5/21/39–41, PYG2/10, M5PM1 G2 NC vs
routed vs unpopulated.
**Partial (2026-09-02):** idle GPIO5/21
`lora_irq=0` `sx_busy=0` (input, no pull). GPIO38–41
not remuxed. [pin-map.md](../references/pin-map.md).

### nyc-rgb-led

Confirm red ignores PWM; sweep G/B. Download-mode blink is red.

**Lite partial (`C153-Lite`, 2026-09-03):** confirmed red status
LED is driven by M5PM1 register `0x06` (`PWR_CFG`) bit 4
(`LED_EN_PP`). Clearing bit 4 turns the red LED off; setting bit 4
turns it back on. Used in `embassy-debug-fw` to turn off the red LED
during low-power light sleep. C153 remains unmeasured for this item.
[sensors.md](../references/sensors.md).

### nyc-buzzer

GPIO42 PWM sweep; note resonance. Optional SPL.
`embassy-debug` drives LEDC low-speed timer 3 / channel 7 at
2000 Hz with an interactive left-edge volume slider (0..=100%,
scaling 0..=25% duty) and click feedback on navigation and touch
buttons. Live hardware testing on `C153-Lite` confirmed acoustic
output peaks at ~25% duty cycle rather than 50% due to coil
inductance saturation and rich harmonic output in the ear's
3–4 kHz sensitivity band; duty cycles above 25% decrease perceived
loudness. EPD card flips and lamp PWM operate concurrently without
interference. Sweep for resonant peak and sound pressure level
remains open.
[input-storage.md](../references/input-storage.md).

### nyc-enclosure-edges

**Lite written** (`C153-Lite`, 2026-09-01): default hold
(e-paper facing the human, USB-C down, keys / mic / SD on
the left) matches the photos. Upper black key = GPIO2
(BUTTON A), lower = GPIO3 (BUTTON B). Idle high, press low.
Still need the same row on **`C153`**. Write leftovers in
[enclosure.md](../references/enclosure.md).

### nyc-lora-session-confirmation

Open for `C153`: capture startup/high and shutdown/low antenna mode,
drive, latch and input-sample evidence from the new lifecycle wrapper.
Check `lora_session` totals: one ping at initial cadence one has three
checks, a receive window or channel sweep has two. A sustained session
must retain antenna control high across packet, standby and channel
changes and preserve counters on repeated active startup. Exercise
mismatch/readback failure and successful recovery with controlled hardware
faults. Do not infer this policy from historical RX/TX observations.

Host tests cover cadence one/twenty/forty and cleanup failure behavior.
Physical evidence is required before deliberately changing firmware from
one to twenty or forty. Digital control confirmation is distinct from
RF-path validation, which remains a separate hardware task. No new
physical session or RF validation occurred during this implementation.

For `C153`, identify the fitted Stamp module revision and characterize
RF output for the retained PA/OCP profile. Compare against catalog
`sx1262` Rev 2.2 §13.1.14.1 “PA Optimal Settings” using the board's actual
matching network. Module connectors and a successful digital readback
cannot close these measurements. See [source leads](../references/sources.md#sx1262-document-and-module-variants).

Confirm the retained 15 ms rail delay across C153 supply conditions;
the official factory HAL uses 100 ms before reset release. Recheck
cleanup recovery across diagnostic wrappers, including an interrupted
operation followed by a new card action.

Async chip I/O also needs C153 bench evidence: confirm one-us post-NSS settling,
cooperative BUSY timing and reception during packet bursts. Exercise timed-RX
RTC cleanup and observe hardware TX timeout with the ping's 300 ms limit.
The host tests cover IRQ acknowledgement and wrapping FIFO offsets, but sticky
IRQ snapshots cannot distinguish repeated arrivals of the same bit. Registry
publication remains outstanding. Keep this row open until these measurements
are made; historical status/RX/TX results do not confirm the async path.
