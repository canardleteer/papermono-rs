# Stamp LoRa-1262 Module

Hardware specification and provenance for the **Stamp LoRa-1262**
communication module populated on the M5Stack PaperMono (`C153`).

## Overview

The Stamp LoRa-1262 is an SMD module (M5Stack SKU `S014` / `S014-IF` /
`S014-I`) based on the Semtech SX1262 transceiver. On the PaperMono
(`C153`), it connects to the host microcontroller via a dedicated SPI bus
and discrete control lines. It is **not** populated on the PaperMono-Lite
(`C153-Lite`), where the corresponding pads remain unpopulated leftover lines.

- **Module**: M5Stack Stamp LoRa-1262
- **Transceiver IC**: Semtech SX1262 (catalog `sx1262`, Rev 2.2)
- **Supported Frequency Range**: 868 MHz to 923 MHz (built-in FPC antenna)
- **Modulation**: LoRa, (G)FSK
- **Supply Domain**: `3V3_L2_LoRa` (switched 3.3 V LDO rail, gated by M5PM1 `G2`)

## Signal Connections

The module interfaces with the ESP32-S3, M5PM1 PMIC, and M5IOE1 expander:

| Signal | Source Pin | Direction | Description |
| --- | --- | --- | --- |
| `SPI_MOSI` | ESP32-S3 `GPIO38` | Output | SPI data from MCU to SX1262 |
| `SPI_MISO` | ESP32-S3 `GPIO40` | Input | SPI data from SX1262 to MCU (JTAG `MTDO` mux) |
| `SPI_CLK` | ESP32-S3 `GPIO39` | Output | SPI serial clock (JTAG `MTCK` mux) |
| `SX_NSS` | ESP32-S3 `GPIO41` | Output | Active-low SPI chip select (JTAG `MTDI` mux) |
| `SX_BUSY` | ESP32-S3 `GPIO21` | Input | Status line; high indicates internal processing |
| `LORA_IRQ` | ESP32-S3 `GPIO5` | Input | SX1262 `DIO1` interrupt line to MCU |
| `LoRa_EN` | M5PM1 `G2` | Output | Enables `3V3_L2_LoRa` power rail (active high) |
| `SX_NRST` | M5IOE1 `PYG10` | Output | Hardware reset line (active low) |
| `SX_ANT_SW`| M5IOE1 `PYG2` | Output | RF antenna switch control line |

## Hardware Operation and Safety Constraints

1. **JTAG Multiplexing**: GPIO39, GPIO40, and GPIO41 default to JTAG
   functionality (`MTCK`, `MTDO`, `MTDI`) on the ESP32-S3. Firmware must
   configure them as general-purpose GPIOs / SPI signals before issuing
   transactions.
2. **BUSY Line Timing**: Per Semtech SX1262 Section 13.5.1, the MCU must verify
   that `SX_BUSY` is low before asserting `SX_NSS` for an SPI transaction.
   `GPIO21` does not have an internal pull resistor on the ESP32-S3.
3. **Power-Gated Domain**: The module is powered from `3V3_L2_LoRa`. To safely
   probe the device, M5PM1 `G2` must be enabled, `SX_NRST` (`PYG10`) held high,
   and sufficient boot delay allowed before querying `GetStatus` (opcode `0xC0`).
4. **RF Safety**: Do not configure continuous transmission or unmodulated
   carrier in standard diagnostic runs. Keep transmission duty cycles compliant
   with local regulatory provisions (868 MHz EU / 915 MHz US).
5. **Antenna Path and Switch Configuration**:
   The schematic routes `PYB_LoRa_ANT_SW` to M5IOE1 `PYG2`. On hardware,
   this line controls an RF switch that gates the built-in FPC antenna to
   the module's RF front-end: `PYG2` MUST be driven HIGH to connect the
   antenna (driving it LOW disconnects the antenna, severely attenuating
   signals). In addition, SX1262 `DIO2` must be configured as the internal
   RF switch control (`set_dio2_as_rf_switch_ctrl(true)`), TCXO powered at
   3.0 V via DIO3 (`set_dio3_as_tcxo_ctrl`), and internal regulator set to
   `REGULATOR_LDO`.
6. **Confirmed Live Verification**:
   Hardware verified live on PaperMono (`C153`) (2026-09-05):
   - SPI interface, status queries (`0xC0` → `0xAA` `STBY_RC`), and power
     gating via M5PM1 `G2` and M5IOE1 `PYG10`.
   - Bench-safe test TX pings clamped to +14 dBm (25 mW) with 60 mA OCP.
   - Live over-the-air packet demodulation of 50-byte Meshtastic LongFast
     broadcast frames on 906.875 MHz (SF11 / BW 250 kHz / CR 4/5 /
     Sync Word `0x24B4`) at -107 dBm RSSI, -16 dB SNR, confirming the
     complete RF receive chain through the built-in FPC antenna.
   - Continuous 104-channel US915 sweeper with double-duty scanning.
