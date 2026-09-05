# m5stack-papermono

Board support for the full M5Stack PaperMono (`C153`) device.

Documentation:
[PaperMono SCH V0.6.2 Schematic](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf)
and [M5Stack PaperMono Documentation](https://docs.m5stack.com/en/core/PaperMono).

This package re-exports the common pin map and panel definitions from
`m5stack-papermono-lite`. It introduces hardware definitions and safe discovery
primitives for features unique to the standard PaperMono model, specifically the
ST25R3916 near-field communication controller (I2C `0x50` identity queries and
`PYG4` power-gating) and the Semtech SX1262-based Stamp LoRa-1262 radio module
(dedicated SPI bus, JTAG pin multiplexing, and status decoding).

The crate is `#![no_std]` and fully testable on the host compiler.
