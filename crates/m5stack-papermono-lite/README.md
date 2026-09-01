# m5stack-papermono-lite

Board support for the **M5Stack PaperMono-Lite** (`C153-Lite`).

This crate is the **shared pin map** for both PaperMono SKUs: ESP32-S3
GPIO numbers, system I2C addresses, M5IOE1 / M5PM1 nets both products
use, and panel geometry. It is `no_std`, has no `esp-hal` dependency,
and is host-testable.

NFC and LoRa are **not** in this crate. PaperMono (`C153`) is the
sibling `m5stack-papermono` crate. Do not drive leftover radio pads on
Lite as extra GPIO.

Chip registers belong in driver crates. MCU peripherals belong in
firmware. This crate does not ship a waveform LUT (OTP first). GPIO45
and GPIO46 are PDM, not a Sticky-style power latch. Park IP2315 off the
system I2C bus except a gated charge transaction.

This README is the crates.io landing page. Relative markdown links here
only resolve inside this package.

## Agent notes

Portable agent rules for this crate live in `AGENTS.md` in this
directory.
