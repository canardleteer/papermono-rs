# m5stack-papermono

Board support for the M5Stack PaperMono (`C153`) device.

Documentation:
[PaperMono SCH V0.6.2 Schematic](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf)
and [M5Stack PaperMono Documentation](https://docs.m5stack.com/en/core/PaperMono).

This package re-exports the common pin map and panel definitions from
`m5stack-papermono-lite`. It introduces hardware definitions, safe discovery
primitives, and dedicated drivers for features unique to the standard PaperMono
model (`C153`):

- **ST25R3916 NFC**: I2C `0x50` controller with `PYG4` power-gating, oscillator
  stabilization, ISO14443-A polling (WUPA/REQA), anticollision cascades (CL1/CL2),
  SAK acquisition, UID detection, ISO-DEP (ISO 14443-4) activation (RATS/ATS),
  and smart card authentication discovery (FIDO CTAP, PIV, OpenPGP Card).
- **Stamp LoRa-1262 (SX1262)**: Dedicated SPI interface (GPIO38/39/40/41 muxed
  off JTAG), power-gated via M5PM1 `G2` (`3V3_L2_LoRa`), reset via M5IOE1
  `PYG10`, RF antenna switch via M5IOE1 `PYG2`, status queries, packet
  configuration, TX burst, and continuous RX packet sniffing with live over-the-air
  frame demodulation.

The crate is `#![no_std]` and fully testable on the host compiler.

## LoRa sessions

`lora` re-exports the generic `sx1262-phy` API, backed by the revision-pinned
`lora-phy` SX126x driver at reviewed revision b47cbdf. PaperMono pins,
presets and `RadioHooks` remain in this BSP. `RadioContext` lends system I2C
and an asynchronous delay
for each startup, TX confirmation or shutdown operation; the wrapper
holds no system-bus borrow between calls.

Use `Sx1262::with_hooks(device, busy, delay, RadioHooks::default())`, then
explicit `startup`, chip configuration, packet operations and `shutdown`.
Keep that same wrapper active for sustained operation. Antenna control
stays high across RX, TX, standby and channel changes until shutdown.
`release` returns the SPI device, BUSY, delay and hooks without powering down.

Startup and shutdown confirm output mode, push-pull drive, latch and
sampled level. Fresh confirmation precedes each TX at the initial
interval of one. A denial, mismatch or readback error blocks TX and
requires shutdown/startup recovery. Digital readback confirms the
control signal; it cannot measure the RF path. DIO2 switching is a
separate SX1262 function. The sequence follows the official
[M5PaperMono-UserDemo LoRa HAL](https://github.com/m5stack/M5PaperMono-UserDemo/blob/main/main/hal/hal_lora.cpp).
The new lifecycle readback policy has host tests and awaits physical
validation on `C153`.

Chip operations await async `SpiDevice`, BUSY and caller-supplied delay.
Firmware owns NSS composition and recovery. Cancellation invalidates readiness;
restore NSS and complete shutdown/startup before another TX. The generic
wrapper requires a nonzero hardware TX timeout and returns the SPI device,
BUSY, delay and hooks on release without I/O. Registry publication is deferred.
