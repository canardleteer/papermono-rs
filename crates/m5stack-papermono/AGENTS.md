# m5stack-papermono

Board support for the full M5Stack PaperMono (`C153`) device.
Workspace member, default-member for host builds. Re-exports
`m5stack-papermono-lite`, then NFC (`nfc`) and LoRa (`lora`).

Panel call site is still `display::OtpRefresh`. Do not use this crate on
PaperMono-Lite (`C153-Lite`). No `esp-hal` in this crate. No LUT. SKU split
(two crates, not a `nfc` / `lora` Cargo feature):
[crates/AGENTS.md](../AGENTS.md).

Lite (2026-09-01) FT XY / OTP partial+mono facts are **Lite only**. Do not
copy them onto `C153` (`nyc-canvas-orient`, `nyc-ft6336-area`,
`nyc-ft6336-points`).

Product PinMap names LoRa SPI as SPI1; official factory demo firmware
([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))
uses `SPI3_HOST` on those GPIOs. Keep both names. Do not flatten.

Live-ask, never-erase, and flash I/O: root [AGENTS.md](../../AGENTS.md).
Pin map:
[m5stack-papermono-hardware](../../.agents/skills/m5stack-papermono-hardware/SKILL.md).

## PaperMono (C153) Hardware Differences

The PaperMono (`C153`) incorporates two additional hardware blocks beyond the
PaperMono-Lite (`C153-Lite`) baseline:

1. **NFC (`nfc` module, ST25R3916)**:
   - System I2C bus at 7-bit address `0x50` (SDA GPIO47, SCL GPIO48).
   - Interrupt on GPIO6 (`G6_RFID_INT`, input).
   - Power gate on M5IOE1 `PYG4` (`PYB_NFC_EN`, active high).
   - **Safety contract**: The RF transmitter must remain unpowered (`tx_en=0` in
     Operation Control Register `0x02` and `PYG4` low) except during brief
     active polling to prevent unwanted power draw or RF emission.
   - Identification: Register Read mode byte `0x7F` queries IC identity register
     `0x3F`, returning IC type `0x05` (`IC_TYPE_ST25R3916`).

2. **Stamp LoRa-1262 (`lora` module, Semtech SX1262)**:
   - Dedicated SPI bus: MOSI GPIO38, CLK GPIO39, MISO GPIO40, NSS GPIO41.
   - ESP32-S3 GPIO39–41 default to JTAG (`MTCK`, `MTDO`, `MTDI`) and must be
     multiplexed to GPIO/SPI.
   - Handshake lines: `BUSY` on GPIO21 (input, no internal pull; must wait for
     low before asserting NSS), `IRQ` on GPIO5 (`DIO1`).
   - Power and reset: M5PM1 GPIO `G2` (`LoRa_EN` / `3V3_L2_LoRa` LDO enable),
     M5IOE1 `PYG10` (`SX_NRST`, active low), M5IOE1 `PYG2` (`SX_ANT_SW`).
   - Module vs Die: Stamp LoRa-1262 module (868–923 MHz with internal FPC
     antenna) housing the Semtech SX1262 transceiver.
   - **Safety contract**: Keep transmitter in standby/idle; do not emit
     continuous RF carrier. Check `BUSY` before every transaction.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without assumptions about
user-specific paths or session state.
