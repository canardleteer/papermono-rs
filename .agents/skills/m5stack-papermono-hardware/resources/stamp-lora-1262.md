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
2. **BUSY Line Timing**: Per catalog `sx1262` §8.3.1 “BUSY Control Line”,
   the MCU must verify
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
   signals). SX1262 `DIO2` must also be configured as the internal
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

## Session control and confirmation

Software policy for PaperMono (`C153`): assert `SX_NRST`, drive
`SX_ANT_SW` high, enable `LoRa_EN`, wait the existing 15 ms rail delay,
release reset and wait the existing 20 ms boot delay plus BUSY readiness.
Confirm antenna output mode, push-pull drive, high latch and sampled high
before marking the session ready. Shutdown invalidates readiness first,
asserts reset, lowers antenna control, disables the rail, then confirms
low latch and sampled low. Remaining cleanup steps run after failures.
Control nets and polarities follow the official
[M5PaperMono-UserDemo LoRa HAL](https://github.com/m5stack/M5PaperMono-UserDemo/blob/main/main/hal/hal_lora.cpp).
Its startup raises the rail before asserting reset and waits 100 ms before
release, then 20 ms. This wrapper asserts reset first and retains our
existing 15/20 ms delays. Its ordering, readback and cadence policy await
C153 measurement; the factory delay does not validate our shorter delay.

Keep `SX_ANT_SW` high across RX, TX, standby and frequency changes.
Short diagnostics stop after their operation; sustained sessions retain
the same wrapper and counters across operations. System I2C is borrowed
only during control/verification. Firmware uses initial cadence one,
with fresh checks before every TX. Startup/shutdown checks always run.
Twenty/forty requires a deliberate configuration change and evidence.
Denial, mismatch or unavailable readback blocks TX, restores cadence one,
warns on serial and requires shutdown/startup recovery.

DIO2 controls the SX1262 RF switch according to catalog `sx1262` Rev 2.2
§13.3.5 “SetDIO2AsRfSwitchCtrl”; `PYG2` is the board's separate antenna
control. Readback confirms the digital control signal. RF connectivity
and impedance require separate measurements. Physical RF validation and
registry publication remain separate.

| Evidence / encoding | Meaning | Official source |
| --- | --- | --- |
| `GPIO_M_L`, bit 1 = 1 | PYG2 configured as output | `m5ioe1` UM V1.4 Table 3 “Register Map”, “GPIO control (IO1–IO14)” |
| `GPIO_DRV_L`, bit 1 = 0 | PYG2 push-pull drive | Same M5IOE1 sections |
| `GPIO_O_L`, bit 1 | PYG2 output latch | Same M5IOE1 sections |
| `GPIO_I_L`, bit 1 | PYG2 sampled level | Same M5IOE1 sections |
| `TxModulation`, 0x0889 bit 2 | Clear for 500 kHz LoRa BW, set otherwise; delegated to upstream | `sx1262` §15.1.2 “Workaround” |
| IQ polarity, 0x0736 bit 2 | Clear for inverted IQ, set for standard IQ; delegated to upstream | `sx1262` §15.4.2 “Workaround” |
| `REG_TX_CLAMP_CONFIG`, 0x08D8, `TX_CLAMP_MASK` 0x1E | Set PA-clamp threshold bits while preserving unrelated bits | `sx1262` §12.1 “Registers”, §15.2.2 “Workaround” |
| `REG_RTC_CONTROL`, 0x0902 = 0 | Stop RTC after timed reception, including early stop | `sx1262` §15.3.2 “Workaround” |
| `REG_RTC_EVENT_CLEAR`, 0x0944, `RTC_EVENT_CLEAR_MASK` 0x02 | Set timeout-event clear bit while preserving unrelated bits | Same section |
| `CalibrationBand` endpoints 0x6B/0x6F, 0x75/0x81, 0xC1/0xC5, 0xD7/0xDB, 0xE1/0xE9 | Cache 430–440, 470–510, 779–787, 863–870, 902–928 MHz bands; reset on startup | `sx1262` §9.2.1 “Image Calibration for Specific Frequency Bands”, Table 9-2 |
| Packet RSSI = negative byte; SNR = signed byte | RSSI in half-dBm, SNR in quarter-dB; whole-unit accessors truncate toward zero | `sx1262` §13.5.3 “GetPacketStatus” |
| Instant RSSI = negative byte | Half-dBm; whole-unit accessor truncates toward zero | `sx1262` §13.5.4 “GetRssiInst” |

Serial `lora_control` records phase, cadence, attempts/checks, expected
level, mode/drive/latch/sample and failure reason. `lora_session` records
verification totals and cleanup status. `lora_metrics` preserves fractional
packet readings alongside the existing whole-unit records. These records become
live measurements when captured on a physical board named by SKU.

## Module variants and PA settings

The official [Stamp product comparison](https://docs.m5stack.com/en/stamp/Stamp_LoRa-1262)
identifies S014 with its RF pad connected; S014-I and S014-IF route the
antenna to IPEX-4 and leave that pad unconnected. Only S014-IF includes
the 12-pin FPC connector. These connectors describe module variants;
they do not establish which module revision is fitted to a physical C153.
Use PaperMono's wiring for its PYG2 control.

The historical “+14 dBm” diagnostic uses command +14, PA duty cycle 2
and hpMax 3. This is retained, but differs from the cached datasheet's
optimal +14 reference row. Actual output power requires measurement.
See
[PA source comparison and document revisions](../references/sources.md#sx1262-pa-profile-comparison).

## Async chip-operation evidence

`lora-phy` and `lora-modulation` are pinned to reviewed
[revision b47cbdf](https://github.com/lora-rs/lora-rs/tree/b47cbdf8d3935e9bfe44c4d407bbad087fcfc179),
with default/LoRaWAN features disabled. The wrapper propagates LDRO computed
from symbol duration into upstream parameters. Its widened airtime calculation
covers CRC selection, 16-bit preambles and SF5/SF6 rules from
[Semtech's reference implementation](https://github.com/Lora-net/sx126x_driver/blob/a10c5dfdf89788c6ac805e9fe98889de44175aa2/src/sx126x.c#L1084).

Every SPI transaction checks BUSY first, waits one microsecond after NSS
rises, then checks BUSY again, following `sx1262` §8.3.1 “BUSY Control Line”.
The initial BUSY delay budget is 100 ms with one-ms cooperative polls;
scheduler latency is additional. Firmware awaits sequences to completion.
An interrupted sequence requires caller NSS recovery and shutdown/startup
before TX, as motivated by upstream
[cancellation discussion](https://github.com/lora-rs/lora-rs/issues/350).
Managed RX acknowledges observed IRQ bits, rejects CRC/header failures before
FIFO access and cleans up RTC after timed reception or early stop. Snapshots
cannot distinguish repeated arrivals of the same sticky IRQ bit.

Host tests establish intent and software sequencing. C153 bench work remains
open for NSS settling, async timing, reception bursts, antenna behavior and
RF power. The existing 3.0 V TCXO, PA/OCP command profile, explicit shutdown
and verification interval one remain. A +14 power command is not measured
RF output. No new physical confirmation is recorded here.
