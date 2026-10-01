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
| SX1262 die | [`lora-phy`](https://crates.io/crates/lora-phy) 3.0.1 and [`lora-modulation`](https://crates.io/crates/lora-modulation) | **pass-with-wrapper** | Generic [`sx1262-phy`](../crates/sx1262-phy) delegates modem operations and errata workarounds to upstream, adding blocking transport, diagnostic commands and explicit lifecycle/TX guards. New session policy awaits physical C153 validation. [Upstream comparison](#sx1262-upstream-comparison) |
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

The published `lora-phy` 3.0.1 driver uses asynchronous `SpiDevice` and
`InterfaceVariant`; the wrapper adapts our dedicated blocking SPI bus.
The adapter futures complete in one poll, enforce BUSY before and after
SPI, flush before deselecting NSS and preserve detailed bus failures.
A borrowed upstream instance executes each chip operation without
owning reset or antenna controls. Its RF-switch callbacks are no-ops;
PaperMono lifecycle hooks own `PYG2` throughout a session. The upstream
instance is private so its TX and continuous-carrier APIs cannot bypass
our guard. Timed raw TX uses the same guard and a local command.

The wrapper delegates the Semtech catalog `sx1262` Rev 2.2 §15.1
“Modulation Quality with 500 kHz LoRa Bandwidth” and §15.4 “Optimizing
the Inverted IQ Operation” workarounds to upstream. PA and OCP remain
explicit to retain the existing PaperMono diagnostic settings. Upstream
LDRO calculation and SF5/SF6 preamble handling are retained. Diagnostic
readback, IRQ masks, arbitrary FIFO offsets and calibration/error queries
remain local where the published API does not expose the same controls.

Reviewed upstream issues and pull requests inform regression coverage:

- [PR 428](https://github.com/lora-rs/lora-rs/pull/428): published transport
  checks BUSY after commands; our adapter also checks before commands.
- [Issue 350](https://github.com/lora-rs/lora-rs/issues/350): cancellation
  hazards motivate explicit cleanup after interrupted lifecycle futures.
- [PR 487](https://github.com/lora-rs/lora-rs/pull/487): firmware rejects
  CRC/header-failed frames before reporting received packets. That newer
  fix is not assumed to exist in the published dependency.
- [PR 456](https://github.com/lora-rs/lora-rs/pull/456): Semtech SWL2001
  comparison exposed a difference between the SX1262 datasheet and
  ST's STM32WL power table in the 14 dBm row. This change preserves the
  existing PA/OCP configuration and leaves
  RF power characterization open.

The new session verification defaults to one. Raising it to twenty or
forty requires deliberate configuration and C153 hardware evidence.
Physical RF validation and registry publication are separate tasks.
