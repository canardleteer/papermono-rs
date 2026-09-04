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
| BMI270 | possible later | **constants-in-BSP** | `CHIP_ID` `0x00` / payload `0x24`. [nyc-bmi270](not-yet-confirmed.md#nyc-bmi270) |
| RX8130CE | possible later | **constants-in-BSP** | Read `FLAG` `0x1D`. Do not write `SEC`. [nyc-rx8130](not-yet-confirmed.md#nyc-rx8130) |
| IP2315 | possible later | **constants-in-BSP** | Park via `PYG11` except a gated charge transaction |
| ST25R3916 | no `st25r3916`. [`st25r95`](https://crates.io/crates/st25r95) is a **different** chip | **fail / wait for C153** | I2C `0x50`, `I2C_EN=VDD`. Official stack is ST RFAL (C) and official factory demo firmware ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)) M5Unit-NFC (Arduino). Do not wrap `st25r95`. [nyc-nfc-ack](not-yet-confirmed.md#nyc-nfc-ack) |
| SX1262 die | [`lora-phy`](https://crates.io/crates/lora-phy) `Sx1262` (live tree [lora-rs](https://github.com/lora-rs/lora-rs)) | **pass-with-wrapper, audit when C153 arrives** | Chip opcodes. Do not adopt in the lockfile yet. [nyc-lora-ack](not-yet-confirmed.md#nyc-lora-ack) |
| Stamp LoRa-1262 | none | **fail (board wrapper)** | Module rails `LoRa_EN` / `SX_NRST` / `SX_ANT_SW`, 868–923 MHz, FPC. Not in `lora-phy`. RadioLib is C++. [nyc-stamp-lora](not-yet-confirmed.md#nyc-stamp-lora) |

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
| [`m5stack-papermono-lite`](../crates/m5stack-papermono-lite) | Shared pin map. `C153-Lite` firmware depends on this only |
| [`m5stack-papermono`](../crates/m5stack-papermono) | `C153` radio add-on. Not a `lite` Cargo feature |
| [`ssd1677-otp`](../crates/ssd1677-otp) | Panel OTP sequences. `OtpRefresh`. No `0x32` LUT |
| [`m5pm1`](../crates/m5pm1) | Register map, ADC, battery %, PWM0, red LED. Board nets stay in the BSP |
| [`m5ioe1`](../crates/m5ioe1) | Register map, bank helpers, `PYG11` typestate. Board `0x4F` |
| [`papermono-log`](../crates/papermono-log) | CDC line format for **both** `simple-debug-fw` and `embassy-debug-fw` |

## Radio (default off)

Do not enable these in firmware until a human asks for a radio
image. No NVS writes. No MAC / BSSID / IRK in CDC.
[nyc-wifi-ble](not-yet-confirmed.md#nyc-wifi-ble).

| Crate | Later use |
| --- | --- |
| `esp-radio` (esp-hal git tag, same as the images) | `embassy-debug-fw --features radio` (`wifi` + `ble` + `coex`) |
| [`trouble-host`](https://crates.io/crates/trouble-host) 0.7 / [`bt-hci`](https://crates.io/crates/bt-hci) | BLE scan count. Do not print addresses |

Official HTML advertises 2.4 GHz Wi-Fi only. Silicon BLE in
board-info does not close the NYC item.

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
