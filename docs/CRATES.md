# Crate audit

Every third-party driver needs a recorded verdict before
adoption. Catalog presence is not a verdict.

Verdicts are **pass** (use as-is), **pass-with-wrapper** (use,
but board specifics stay in the SKU crate), **fail** (write our
own or wait), **written-here** (in-tree `embedded-hal` driver),
or **constants-in-BSP** (named registers in
`m5stack-papermono-lite` / `m5stack-papermono`; I/O still in
firmware).

Do not path-dep foreign SSD1677 drivers with differing OTP
sequences. Do not wrap [M5Unified](https://github.com/m5stack/M5Unified)
(C++ board HAL, `develop` branch). Panel LUTs belong to M5GFX /
OTP-Demo, not M5Unified.

## Constants in the BSP

Empty crates are not a verdict. FT6336G has no public map;
AW9967 is PWM-only here. Do not add empty BMI270 / RX8130 /
IP2315 crates.

| Part | crates.io? | Verdict | Basis |
| --- | --- | --- | --- |
| FT6336G | public PDF has **no register map** | **constants-in-BSP** | M5GFX `decode_m5gfx` in the board crate. Do not invent a FocalTech map |
| AW9967 | no crate | **constants-in-BSP** | PWM into `EINK_BL`. No invented AW9967 register map. [nyc-frontlight](not-yet-confirmed.md#nyc-frontlight) |
| BMI270 | possible later | **constants-in-BSP** (+ accel bring-up helpers) | `CHIP_ID` `0x00` / payload `0x24`. Soft-reset + Bosch standard 8 KiB config (`INIT_ADDR_*` + `INIT_DATA`) + raw `DATA_8`…`DATA_13`. Orientation classify is sticky-rs policy; Lite axis map USB-C down = −X. [nyc-bmi270](not-yet-confirmed.md#nyc-bmi270) |
| RX8130CE | possible later | **constants-in-BSP** | Read `FLAG` `0x1D`. Do not write `SEC`. [nyc-rx8130](not-yet-confirmed.md#nyc-rx8130) |
| IP2315 | possible later | **constants-in-BSP** | Park via `PYG11` except a gated charge transaction |
| ST25R3916 | in-tree [`st25r3916`](../crates/st25r3916). [`st25r95`](https://crates.io/crates/st25r95) is a **different** chip | **written-here** | MCU-agnostic `embedded-hal` driver crate in `crates/st25r3916`. I2C `0x50`, `I2C_EN=VDD`. ISO14443-A initiator (WUPA/REQA, anticollision CL1/CL2, SAK read), ISO/IEC 14443-4 (ISO-DEP / T=CL) activation and APDU half-duplex block protocol with chaining/WTX, smart card application discovery (FIDO CTAP, PIV, OpenPGP), target/card emulation profiles (NFC-A, NFC-F, NFCIP-1), PT_Memory layout, and Type 2/4A / NDEF protocol framing. Re-exported with board nets in `m5stack-papermono::nfc`. Confirmed live on C153. [nyc-nfc-ack](not-yet-confirmed.md#nyc-nfc-ack) |
| SX1262 die | `lora-phy` and `lora-modulation` at [revision b47cbdf](https://github.com/lora-rs/lora-rs/tree/b47cbdf8d3935e9bfe44c4d407bbad087fcfc179) | **pass-with-wrapper** | Generic [`sx1262-phy`](../crates/sx1262-phy) delegates modem operations and errata workarounds to upstream, adding async transport, diagnostic commands and explicit lifecycle/TX guards. New session policy awaits physical C153 validation. [Upstream comparison](#sx1262-upstream-comparison) |
| Stamp LoRa-1262 | none | **constants-in-BSP / module wrapper** | Module rails `LoRa_EN` / `SX_NRST` / `SX_ANT_SW`, 868–923 MHz, FPC in `m5stack-papermono::lora` and [stamp-lora-1262](../.agents/skills/m5stack-papermono-hardware/resources/stamp-lora-1262.md). Confirmed live on C153. [nyc-stamp-lora](not-yet-confirmed.md#nyc-stamp-lora) |

## Rejected

| Crate | Why |
| --- | --- |
| [`ssd1677`](https://crates.io/crates/ssd1677) | No four-gray OTP path. Occupies the obvious name |
| [`ssd1677-driver`](https://crates.io/crates/ssd1677-driver) | Same gap |
| [`epd-waveshare`](https://crates.io/crates/epd-waveshare) | Different controller / panel families |
| [`st25r95`](https://crates.io/crates/st25r95) | ST25R95, not ST25R3916. Typically SPI |

## Written here

| Crate | Why |
| --- | --- |
| [`m5stack-papermono-lite`](../crates/m5stack-papermono-lite) | Shared pin map and `BoardModel` runtime profile. `C153-Lite` firmware depends on this only |
| [`m5stack-papermono`](../crates/m5stack-papermono) | `C153` board nets, ST25R3916 support and SX1262 lifecycle hooks. Re-exports the generic driver; pruned via `--no-default-features --features lite` |
| [`st25r3916`](../crates/st25r3916) | ST25R3916 NFC transceiver driver: initiator (reader), ISO-DEP / APDU block protocol, smart card app discovery (FIDO/PIV/OpenPGP), target (card emulation) profiles, PT_Memory, Type 2/4A framing, and NDEF |
| [`ssd1677-otp`](../crates/ssd1677-otp) | Panel OTP sequences. `OtpRefresh`. No `0x32` LUT |
| [`m5pm1`](../crates/m5pm1) | Register map, ADC, battery %, PWM0, red LED. Board nets stay in the BSP |
| [`m5ioe1`](../crates/m5ioe1) | Register map, bank helpers, `PYG11` typestate. Board `0x4F` |
| [`sx1262-phy`](../crates/sx1262-phy) | Generic wrapper around upstream SX126x operations, with caller lifecycle hooks, guarded TX and local diagnostic commands |
| [`papermono-log`](../crates/papermono-log) | CDC line format for **both** `simple-debug-fw` and `embassy-debug-fw` |

## Radio

`embassy-debug-fw` lands with `--features radio` **on**
(BLE pairing card + Wi-Fi survey / SoftAP cards). Survey and
SoftAP stay **idle until touch**. Packed listen-only
`wifi n=` / `ble n=` counts still wait for an explicit human
ask (root **Pack one flash**). No NVS writes. No foreign
MAC / BSSID / IRK in CDC. SoftAP CDC may print the fixed
demo SSID / password (`PaperMono-AP` / `mono2026`). Survey
glass may show truncated nearby SSIDs; do not echo those on
CDC. [nyc-wifi-ble](not-yet-confirmed.md#nyc-wifi-ble).

| Crate | Use |
| --- | --- |
| `esp-radio` (esp-hal git tag, same as the images) | `wifi` + `ble` + `coex`. WPA2 SoftAP only: precompiled ESP32-S3 blob does not enable WPA3/SAE |
| [`trouble-host`](https://crates.io/crates/trouble-host) 0.7 / [`bt-hci`](https://crates.io/crates/bt-hci) | BLE peripheral passkey pairing (and listen-only scan counts). No addresses on CDC |
| [`embassy-net`](https://crates.io/crates/embassy-net) 0.9 | SoftAP IPv4 stack (`192.168.4.1/24`) |
| [`edge-dhcp`](https://crates.io/crates/edge-dhcp) 0.8 / [`edge-nal`](https://crates.io/crates/edge-nal) / [`edge-nal-embassy`](https://crates.io/crates/edge-nal-embassy) | DHCP server + tiny HTTP JSON on port 80 |

Survey and SoftAP are mutually exclusive in firmware (one
Wi-Fi mode at a time). Official HTML advertises 2.4 GHz
Wi-Fi; silicon BLE in board-info does not close the NYC
item alone. Lite SoftAP host-verified 2026-09-04:
[measure.md](../.agents/skills/m5stack-papermono-hardware/references/measure.md).

## Infrastructure

`papermono-host` uses [`espflash`](https://crates.io/crates/espflash)
4.5 as a library (`default-features = false`, feature
`serialport`). Do not enable espflash’s `cli` feature. Never
call full-chip erase APIs. `cargo xtask` is clap over
`papermono-host`.

[`sha2`](https://crates.io/crates/sha2) (0.10) for firmware asset
integrity checking at build time and during `cargo xtask encode-assets`.

Firmware images take `esp-hal`, `esp-println`, `esp-backtrace`,
`esp-bootloader-esp-idf` from git tag `esp-hal-v1.2.0-rc.0`.
`embassy-debug-fw` also takes `esp-rtos` from that tag. One
workspace lockfile. `esp-bootloader-esp-idf` is only for
`esp_app_desc!()`. Do not `--merge`.

`embedded-hal` 1.0 and dev-only `embedded-hal-mock` (`eh1`) for
`ssd1677-otp`, `m5pm1`, and `m5ioe1`.

## SX1262 upstream comparison

Both dependencies are pinned to reviewed
[revision b47cbdf](https://github.com/lora-rs/lora-rs/tree/b47cbdf8d3935e9bfe44c4d407bbad087fcfc179), with default and
LoRaWAN features disabled. Upstream let-chains require Rust 1.88; the
workspace minimum is raised accordingly. Registry publication is deferred.
The generic wrapper borrows upstream SX126x backends for configuration, standby,
channel and supported FIFO operations. Lifecycle hooks, TX guard, IRQ snapshots,
diagnostics, calibration-band cache and timed-RX management remain local.
The upstream instance is private; its timer-disabled TX and continuous-carrier
operations cannot bypass our guard. Typed and raw TX require a nonzero timer.

All chip I/O awaits async `SpiDevice`. BUSY checks surround every transaction;
a one-us delay after NSS rises precedes the second check. The configurable
budget starts at 100 ms with one-ms polls and excludes scheduler/SPI latency.
Original SPI-device, BUSY-pin and upstream errors survive. Firmware composes
NSS with `embedded-hal-bus::ExclusiveDevice` and retains access for recovery.
Await sequences to completion; cancellation invalidates readiness and requires
NSS recovery, shutdown and startup before TX. See upstream
[issue 350](https://github.com/lora-rs/lora-rs/issues/350).

The wrapper retains upstream bandwidth and IQ workarounds from catalog
`sx1262` Rev 2.2 §15.1.2 and §15.4.2 “Workaround”. It propagates the LDRO
calculated by `lora-modulation`, including narrow bandwidths, and retains
SF5/SF6 minimum preambles. Local TX-clamp and timed-RX RTC workarounds follow
§15.2.2 and §15.3.2 “Workaround”, preserving unrelated register bits and the
existing PaperMono PA/OCP command profile. Calibration-band caching follows
§9.2.1 “Image Calibration for Specific Frequency Bands”.

Managed RX rejects CRC/header failures before FIFO, acknowledges observed
interrupt bits during continuous RX, and supports wrapping offsets. Lossless
half-dBm/quarter-dB metrics supplement existing whole-unit CDC records.
The widened airtime helper includes optional CRC, 16-bit preambles and SF5/SF6
rules from
[Semtech's reference algorithm](https://github.com/Lora-net/sx126x_driver/blob/a10c5dfdf89788c6ac805e9fe98889de44175aa2/src/sx126x.c#L1084).
A sticky IRQ snapshot cannot distinguish repeated arrivals of the same bit.

Firmware preserves ping/listen/sweep interaction patterns and presets. Ping
programs the existing 300 ms completion budget into the hardware timer.
Read failures and IRQ timeouts warn and trigger cleanup; no signal readings
are invented after failures. Session verification starts at one; twenty or
forty requires deliberate configuration and C153 hardware evidence.
Settling, async timing, burst reception, RF power and antenna behavior still
need C153 bench work. Historical hardware results do not validate this policy.
