# m5stack-papermono

Board support for the full M5Stack PaperMono (`C153`) device.

Documentation:
[PaperMono SCH V0.6.2 Schematic](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf)
and [M5Stack PaperMono Documentation](https://docs.m5stack.com/en/core/PaperMono).

This package re-exports the common pin map and panel definitions from
`m5stack-papermono-lite`. It introduces hardware definitions, safe discovery
primitives, and dedicated drivers for features unique to the standard PaperMono
model (`C153`):

- **ST25R3916 NFC**: I2C `0x50` controller with `PYG4` power-gating, oscillator
  stabilization, ISO14443-A polling (WUPA/REQA), anticollision cascades (CL1/CL2),
  SAK acquisition, and UID detection for physical contactless cards.
- **Stamp LoRa-1262 (SX1262)**: Dedicated SPI interface (GPIO38/39/40/41 muxed
  off JTAG), power-gated via M5PM1 `G2` (`3V3_L2_LoRa`), reset via M5IOE1
  `PYG10`, RF antenna switch via M5IOE1 `PYG2`, status queries, packet
  configuration, TX burst, and continuous RX packet sniffing with live over-the-air
  frame demodulation.

The crate is `#![no_std]` and fully testable on the host compiler.
