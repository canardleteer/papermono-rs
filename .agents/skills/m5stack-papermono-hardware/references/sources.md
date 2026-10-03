# Sources, conflicts, and gaps

## Precedence

The skill user is authoritative. When sources disagree, name both
sides and their layers; do not silently flatten the conflict.
Full wording: [SKILL.md](../SKILL.md#authority).

1. **Skill user** — they weigh the facts.
2. **Observed hardware** on this product (batch variation
   allowed). Both SKUs have USB, flash-size, and partition-table results in
   [measure.md](measure.md) and [flashing.md](flashing.md#usb-measured).
   Official HTML `epd_*` times are PaperMono lab reference
   in [display.md](display.md), not a silicon row.
3. **Official** board docs, vendor SDKs, schematics, and chip
   datasheets for parts named on this model. Registers and
   timings when they have **not been measured**. Official
   stock/SDK sequences prove **intent and ordering, never
   electrical fact**.
4. **Third-party** firmware (FreeInk, community). Often first
   with new valid detail; often stale or wrong.

Observed outranks a datasheet default. Do not apply a datasheet
to a chip that is not on this model. Do not treat external
board measurements as PaperMono or PaperMono-Lite facts. Name
the SKU (`C153` vs `C153-Lite`).

URL and firmware map: [catalog.md](catalog.md). Vendor
datasheets: [datasheets.md](../resources/datasheets.md). Open
measurements:
[not-yet-confirmed.md](../resources/not-yet-confirmed.md).
External: [external.md](../resources/external.md). Vendor C++:
[cpp-platformio.md](cpp-platformio.md).

## Citations

| Source | Layer | Use |
| --- | --- | --- |
| Live silicon ([measure.md](measure.md), [flashing.md](flashing.md#usb-measured), [display.md](display.md)) | Observed | Both SKUs USB `303a:1001` (run and download), ESP32-S3 v0.2, 16 MB flash, and factory tables matching UserDemo CSV. Lite I2C 2026-09-02: `ack=32,38,4f,68,6e nak=50,6f,75`, `rtc_flag=31`, `imu_id=24`; C153 roster includes `0x50` when enabled. C153 NFC and LoRa functions are confirmed. JEDEC bytes, PSRAM, and C153 `probe-rs` enumeration remain open. Official HTML `epd_*` times are not a row here |
| [PaperMono docs](https://docs.m5stack.com/en/core/PaperMono) | Official | Living **PinMap**, specs, e-paper notes, SKU compare, heading **M5GFX LUT Refresh Speed**. Re-read when nets look stale. Snapshot 2026-10-01: [PaperMono.2026-10-01.md](../resources/official-html/PaperMono.2026-10-01.md) |
| [PaperMono-Lite docs](https://docs.m5stack.com/en/core/PaperMono-Lite) | Official | Living **PinMap** (no RFID/LoRa headings). Same **M5GFX LUT Refresh Speed** table. Same page: M5GFX LUTs unstable; prefer OTP-Demo. Snapshot 2026-10-01: [PaperMono-Lite.2026-10-01.md](../resources/official-html/PaperMono-Lite.2026-10-01.md) |
| Official **M5GFX LUT Refresh Speed** (`epd_quality` / `epd_text` / `epd_fast` / `epd_fastest`) | Official | PaperMono laboratory results under M5GFX modes; reference only; times vary with content. Lite page reprints the same table. Snapshot 2026-10-01: [official-html/SOURCE.md](../resources/official-html/SOURCE.md). [display.md](display.md) |
| Schematic PDFs + gallery PNGs V0.6.2 2026-05-22 ([datasheets.md](../resources/datasheets.md), [catalog.md](catalog.md)) | Official | Dated OSS snapshot from those HTML pages. Walk PDF/PNGs. Nets. HTML may ship a newer set |
| [M5Stack PaperMono Arduino examples](https://docs.m5stack.com/en/arduino/papermono/program) | Official (intent; checked 2026-10-01) | Board Manager >=3.3.9; examples for display, buttons, touch, IMU, mic, microSD, NFC, LoRa, buzzer, power management, and wakeup. Individual pages and qualified observations: [catalog.md](catalog.md) |
| [M5Stack PaperMono LoRa tutorial](https://docs.m5stack.com/en/arduino/papermono/lora) | Official (intent; checked 2026-10-01) | `enableLoRaHardware`: rail first, 200 ms before reset, 100 ms low, 200 ms after release. RX sets PYG2 LOW; TX requests 22 dBm but labels the display 16 dBm. These antenna/power conflicts remain unresolved; see below |
| [M5PM1 & M5IOE1 Arduino](https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1) | Official (intent) | L0–L3B are independently switched from L0; current power/wake examples. Does not make their sequences physical measurements |
| [M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo) | Official (intent) | OTP path; panel PN `DEPG0397BBS770F3HP-XM`. Direct dep M5Unified; M5GFX is transitive. Panel SPI is `EDP_OTP_LUT_demo` |
| [M5GFX](https://github.com/m5stack/M5GFX) (`Panel_SSD1677_4Gray`) | Official (intent) | Reviewed 0.2.31 release and current development tree. UserDemo / M5Unified panel; four `epd_*` modes. Product page still warns LUTs unstable and recommends OTP |
| [M5Unified](https://github.com/m5stack/M5Unified) (`develop`) | Official (intent) | C++ board HAL. PlatformIO `#develop`. `board_M5PaperMono` PMIC / SDMMC / charge / RTC INT. Not a Rust crate. Does not close NYC. Radio tracking is [nyc-wifi-ble](../resources/not-yet-confirmed.md#nyc-wifi-ble) |
| [M5Unified LED](https://github.com/m5stack/M5Unified/blob/8530f5377d782e4a25a6c482de2e71c3f75ca8eb/src/utility/led/LED_PaperMono_Class.hpp) | Official (intent) | `LED_PaperMono_Class`: red on PM1 `0x13`/`0x06`, green on IOE1 PYG8 PWM ch2, blue on PYG9 PWM ch3 (5 kHz, 8-bit) |
| [M5Unified Power](https://github.com/m5stack/M5Unified/blob/8530f5377d782e4a25a6c482de2e71c3f75ca8eb/src/utility/Power_Class.cpp#L72-L96) | Official (intent) | IP2315 gate on IOE1 PYG11: 2 ms wait then 64-loop ready check before charge read |
| [uiflow-micropython PaperMono](https://github.com/m5stack/uiflow-micropython/tree/587e134c61b31431335351e04ebfc05f69064bb7/m5stack/boards/M5STACK_PaperMono) | Official (intent) | Board ID 29, USB `303a:816b`, 240 MHz, 8 MB Octal PSRAM, 16 MB QIO Flash |
| [Stamp LoRa-1262](https://docs.m5stack.com/en/stamp/Stamp_LoRa-1262) | Official (module) | SKU S014 / S014-IF / S014-I. **Contains** SX1262. Module band 868–923 MHz. Not the Semtech die sheet. [measure.md](measure.md) |
| [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo) | Official (intent) | Eval HAL: SKU probe, rails, SDMMC 4-bit, LoRa `SPI3_HOST`, sleep paths, `partitions.csv`. See [user-demo.md](user-demo.md) (`c109910`, V1.2) |
| ESP32-S3 datasheet v2.2 ([datasheets.md](../resources/datasheets.md)) | Official (named MCU) | Straps GPIO0/3/45/46, JTAG 39–42, USB 19/20, I2C |
| SSD1677 Rev 1.0 / M5PM1 V 1.9 / M5IOE1 V 1.4 / FT6336G / IP2315 / BMI270 / ST25R3916 / SX1262 ([datasheets.md](../resources/datasheets.md)) | Official (named parts) | Registers, opcodes, timings. Observed still outranks a default |
| Product photos ([enclosure.md](enclosure.md)) | Official | Case color, **BUTTON A (UP)** / **BUTTON B (DOWN)** / red power callouts. Not a GPIO pinout |
| FreeInk `PAPERMONO` ([external.md](../resources/external.md)) | Third-party | PMIC/expander sequencing comments; LUT/SDMMC/canvas conflicts |

Do not cite a host checkout path, a one-off dump directory, or
another person’s MAC / serial / NVS / flash image as if they were
product facts.

## Conflicts

State both columns when a page or issue touches a row. The skill
user weighs them. Both SKUs have measured USB IDs, flash size, and
factory tables. Official HTML `epd_*` times are PaperMono lab
reference, not separate Lite timings. Each statement below is scoped by
linked measurements. Name
`C153` vs `C153-Lite`.

| Topic | Official / named | Other sources |
| --- | --- | --- |
| Gray / LUT | 4-gray OTP; M5GFX LUTs “currently unstable”; prefer OTP-Demo. UserDemo uses M5GFX `epd_*` plus analog-off `0x22`/`0x03` then `0x20` ([user-demo.md](user-demo.md), [display.md](display.md)). Official HTML `epd_*` times are PaperMono lab, reference only | FreeInk host-authored LUTs, “3-level grayscale”. Standby recovery without reset is unconfirmed |
| EPD SPI clock | SSD1677 write `fSCL` max 20 MHz. OTP-Demo `EDP_SPI` is 20 MHz | M5GFX PaperMono autodetect sets `freq_write` 40 MHz. [nyc-epd-spi-clock](../resources/not-yet-confirmed.md#nyc-epd-spi-clock) |
| Canvas | 480×800. UserDemo `setRotation(0)`. Lite USB-C down: OTP RAM X = physical Y, RAM Y = physical X ([display.md](display.md), [measure.md](measure.md)) | FreeInk 800×480. OTP-Demo addresses 800×480 RAM. C153 orientation unmeasured |
| Frontlight | Official HTML: M5PM1 G3 PWM `BL_FB` (brightness). Schematic V0.6.2: one AW9967DNR on `EINK_BL`. UserDemo `display.setBrightness`. Lite: PWM0 slide **drives** the lamp ([measure.md](measure.md)) | FreeInk README AW9967 (schematic-true). FreeInk Paper Mono: G3 → **PWM0**, no `gpioWarm`. CrossPoint warmth UI is for dual-channel boards (X4 Pro / Murphy M4), not this SKU. PWM1 writes left Lite constant |
| M5IOE1 address | Schematic / pin map / UserDemo `IO_EXPANDER_ADDR = 0x4F`. Library: `0x4F` REV `'W'`; fallback candidate `0x6F` | Chip UM V 1.4: `0x6F`–`0x76` from IO7, REV `'A'`. Library default is `0x6F`. Auto-detect `0xFF` also walks `0x70`–`0x76` (includes `0x75`); UserDemo does not use it |
| microSD | DAT0–DAT3 in the pin table. UserDemo `slot_config.width = 4`; current Arduino example configures CLK, CMD, and DAT0–DAT3 with `SD_MMC.setPins` | FreeInk “native 1-bit SDMMC” |
| Size / weight | HTML: 62.0 × 101.0 × 8.0 mm; 74.7 g / Lite 72.4 g | Older product PDF: 61 mm / “work in progress” |
| USB debug | Both SKUs run **and** download: `303a:1001` Espressif USB JTAG/serial debug unit ([flashing.md](flashing.md#usb-measured)). Vendor Arduino: CDC flags | Generic DevKit or CH343 assumptions do not apply |
| Flash | Official 16 MB. Both SKUs **measured** 16 MB (`0x1000000`) and UserDemo-matching table at `0x8000`. PIO `default_16MB.csv` is a different table | 32 MB assumptions do not apply. JEDEC bytes and PSRAM remain [nyc-flash-id](../resources/not-yet-confirmed.md#nyc-flash-id) |
| Power / wake | M5PM1 button. Arduino: IMU/RTC wake via PM1 G4/G0 then `shutdown()`. UserDemo adds ESP `ext0` GPIO4 touch deep sleep. Lite 2026-09-02: short ~0.25 s reset, hold ~2 s download, double-press off. USB-in `SYS_CMD` bounced; unplug lamp-off then 2–3 s `sleep abort` same boot ([power-and-sleep.md](power-and-sleep.md)). Lite 2026-09-03: interactive light sleep via Button A 2 s hold, red LED off via PM1 `PWR_CFG` bit 4, Button A/B 1 s hold wake qualification | Foreign GPIO45/46 latch code (those pins are PDM here). Sleep current and GPIO0 strap still need a meter |
| Battery / telemetry | M5PM1 ADC `VBAT_L`/`VBAT_H` and `VIN_L`/`VIN_H`. 1S LiPo linear mapping 3300..4150 mV to 0..100% SoC. Lite 2026-09-03: live battery gauge and power status on Legend card, 60 s auto-refresh, IP2315 isolated ([measure.md](measure.md)) | IP2315 charge transaction requires PYG11 gate; low VBAT can hang bus. Live drain rate not yet profiled |
| LoRa SPI host | Pin table / schematic name those GPIOs SPI1 | UserDemo `hal_lora.cpp` uses ESP-IDF `SPI3_HOST` (SPI0/1 are flash). 868.0 MHz RadioLib begin vs product 868–923 MHz |
| LoRa module vs die | Stamp LoRa-1262: 868–923 MHz, `LoRa_EN` / `SX_NRST` / `SX_ANT_SW`, FPC on C153 | SX1262 sheet: 150–960 MHz ISM. Both switch lines are required on C153: DIO2 controls SX1262 internal RF switch; M5IOE1 `PYG2` (`SX_ANT_SW`) gates the built-in FPC antenna and must be driven HIGH. [stamp-lora-1262.md](../resources/stamp-lora-1262.md) |
| C153 LoRa RX antenna control | Current [M5Stack LoRa tutorial](https://docs.m5stack.com/en/arduino/papermono/lora) drives PYG2 LOW in RX startup and `resumeReceive` | Older factory HAL and historical C153 reception used HIGH. Runtime retains HIGH throughout the session; module revision and RF-path comparison remain open. [Antenna conflict](#papermono-lora-antenna-and-power-conflicts) |
| LoRa tutorial TX power | Current tutorial sets `kTxPowerDbm` to 22 and passes it to `radio.begin` | Its TX screen says 16 dBm. Neither value is measured RF output; the repository's PA/OCP settings remain unchanged. [Power conflict](#papermono-lora-antenna-and-power-conflicts) |
| Bring-up | Arduino: M5PM1 then M5IOE1 then peripherals | UserDemo: 500 ms, `M5.begin`, then PM1/IOE1, then NFC identity probe for SKU |
| Lite NFC/LoRa | HTML **PinMap** and SKU compare: modules absent | Lite schematic V0.6.2 gallery page 05 / PDF still draws Stamp LoRa-1262 and RFID/`PYB_NFC_EN`. Do not flatten |

## Gaps

Everything in
[not-yet-confirmed.md](../resources/not-yet-confirmed.md).
Firmware will not close those rows.

## SX1262 PA profile comparison

Catalog `sx1262` Rev 2.2 §13.1.14.1 “PA Optimal Settings”, Table 13-21,
remains authoritative for the discrete SX1262. The reference rows require
the corresponding matching network; configuration alone does not prove
power at the antenna.

| Profile | PA duty cycle | hpMax | SetTxParams | OCP |
| --- | --- | --- | --- | --- |
| Rev 2.2 optimal +14 dBm row | 2 | 2 | +22 | Select for the circuit |
| Rev 2.2 optimal +17 dBm row | 2 | 3 | +22 | Select for the circuit |
| Retained diagnostic, legacy +14 name | 2 | 3 | +14 | 60 mA |

The lead in [lora-rs PR 456](https://github.com/lora-rs/lora-rs/pull/456)
is a difference between the discrete SX1262 datasheet and ST's STM32WL
power table. The integrated ST part has its own PA table in current
upstream. The Semtech [SWL2001 reference BSP](https://github.com/Lora-net/SWL2001/blob/master/lbm_examples/radio_hal/ral_sx126x_bsp.c)
uses the maximum-size SX1262 PA configuration and a board power offset;
it does not supply that alternative optimal +14 row. Preserve the
existing diagnostic settings during this lifecycle change. A separate
C153 measurement must characterize RF output before changing its profile.

## SX1262 document and module variants

The [Semtech SX1262 resource listing](https://www.semtech.com/products/wireless-rf/lora-connect/sx1262)
dates its datasheet entry 2025-04-07. That listing date alone does not
establish a new revision; direct access was unavailable during review.
The [second M5Stack copy](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1198/DS_SX1261_2_V2-2.pdf)
identifies the same Rev 2.2, Dec 2024 and Table 13-21 values as our cache.
Keep the cached revision as the command reference. Its revision history
records the Rev 1.2 SetTxParams correction and the Rev 2.2 PA description
change; older driver citations need that revision context.

Official [Stamp LoRa-1262 documentation](https://docs.m5stack.com/en/stamp/Stamp_LoRa-1262)
distinguishes S014 RF-pad routing from S014-I/S014-IF IPEX-4 routing.
The [Cap LoRa-1262 comparison](https://docs.m5stack.com/en/cap/Cap_LoRa-1262)
also distinguishes a smaller current Stamp module with expander control
from a legacy module without that control. These variants explain why
antenna policy belongs to the caller. They do not identify the fitted
C153 module revision; PaperMono's own schematic and factory HAL remain
its wiring references.

## PaperMono LoRa startup timing

The current [M5Stack PaperMono LoRa tutorial](https://docs.m5stack.com/en/arduino/papermono/lora)
`enableLoRaHardware` (reviewed 2026-10-01) enables the rail and waits
200 ms before asserting reset, holds it low for 100 ms, then waits
200 ms after release. The C153 lifecycle follows these reset timings.
Host mocks check rail-first operation order and all three waits; BUSY
readiness and digital readback do not prove settling across supply conditions.

The older [M5PaperMono-UserDemo LoRa HAL](https://github.com/m5stack/M5PaperMono-UserDemo/blob/c1099107271d31a0678d661a896e2b04dbb331ea/main/hal/hal_lora.cpp)
(reviewed V1.2, 2026-08-10) enables the rail before antenna-high/reset-low,
holds reset for 100 ms and waits 20 ms after release. That dated sequence
is comparison evidence. Physical confirmation of the new lifecycle remains
open under
[nyc-lora-session-confirmation](../resources/not-yet-confirmed.md#nyc-lora-session-confirmation).

## PaperMono LoRa antenna and power conflicts

The current [M5Stack PaperMono LoRa tutorial](https://docs.m5stack.com/en/arduino/papermono/lora),
reviewed 2026-10-01, sets `kAntennaSwitchPin` (M5IOE1 PYG2) LOW
in the receiver's `enableLoRaHardware` and before `radio.startReceive`
in `resumeReceive`. The older [M5PaperMono-UserDemo LoRa HAL](https://github.com/m5stack/M5PaperMono-UserDemo/blob/c1099107271d31a0678d661a896e2b04dbb331ea/main/hal/hal_lora.cpp)
(V1.2, 2026-08-10) sets antenna control HIGH during startup.
Historical [C153 reception](measure.md) used HIGH too. These are
different layers of evidence; successful reception alone does not
characterize the switch, fitted module revision or RF impedance.
One controlled packet was received on C153 with the runtime HIGH policy on
2026-10-02. This confirms that path on that unit; it does not test the LOW
example, establish module-wide behavior or characterize antenna impedance.
The RX polarity conflict remains unresolved. Keep the repository session
policy HIGH across RX, TX, standby and channel changes until shutdown.

The tutorial's transmitter requests `kTxPowerDbm = 22` through
`radio.begin`, while `drawScreen` labels it 16 dBm. This is an
unresolved mismatch within the example, rather than a measured
calibration or a basis for changing our diagnostic PA/OCP settings.
The retained profile still needs C153 RF-output characterization;
see [PA profile comparison](#sx1262-pa-profile-comparison).

Normal C153 RX control and one-way reception are recorded in
[measure.md](measure.md). Remaining board work is tracked under
[NYC rows](../resources/not-yet-confirmed.md#nyc-lora-session-recovery). The
source comparison itself was reviewed on 2026-10-01, before that live test.
