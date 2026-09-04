# m5stack-papermono-lite

Board support and pin definitions for the M5Stack PaperMono-Lite
(`C153-Lite`) development board.

Hardware details:
[PaperMono-Lite PRJ V0.6.2 Schematic](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522.pdf)
and [M5Stack PaperMono-Lite Documentation](https://docs.m5stack.com/en/core/PaperMono-Lite).

This crate provides the core pin map common to PaperMono hardware. It exports
ESP32-S3 package pins, addresses for the system I2C bus, expander nets, power
rail controls, battery charge bounds and calculation helpers, and portrait panel
dimensions without taking a direct dependency on `esp-hal`. Because the package
compiles under `#![no_std]` without architecture-specific HAL primitives, unit
tests run directly on host tooling.

Peripheral register details reside in companion chip crates such as `m5pm1`,
`m5ioe1`, and `ssd1677-otp`. Radio components like NFC and LoRa are absent on
the Lite model and are provided by the sibling `m5stack-papermono` crate.
