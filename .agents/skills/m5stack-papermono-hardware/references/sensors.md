# Sensors, mic, RGB

I2C devices share GPIO47/48. Addresses:
[pin-map.md](pin-map.md). Lite official M5IOE1 `begin`
ACKed at board `0x4F` (`ioe_addr=4f`); named-register /
address-only `read` at `0x4F` had NAKed. PM1, RX8130,
BMI270 ACK; leftover `0x50` NAK; FT `0x38` ACK after
EN/RST (`tp=1`). Lite advertised roster
(2026-09-02): `ack=32,38,4f,68,6e nak=50,6f,75`.
Name `C153` vs `C153-Lite`
([measure.md](measure.md),
[nyc-i2c-ack](../resources/not-yet-confirmed.md#nyc-i2c-ack)).

## BMI270

Official / schematic `0x68`. Sheet default if SDO is to GND
(`0x69` if SDO to VDDIO). INT1 goes to M5PM1 G4 (wake), not to
a raw ESP32 GPIO. Do not drive an ESP pin
as IMU INT. Any-motion wake through the PMIC is documented
Arduino intent; UserDemo `configureBmi270AnyMotion` tries
`0x68` then `0x69` and maps any-motion to INT1
([user-demo.md](user-demo.md)). Confirm on a unit:
[nyc-pm1-wake](../resources/not-yet-confirmed.md#nyc-pm1-wake).
`CHIP_ID` payload: Lite CDC `imu_id=24` (2026-09-01 and
2026-09-02). Optional later: a motion sample. `C153` still
[nyc-bmi270](../resources/not-yet-confirmed.md#nyc-bmi270).

Registers: cache id `bmi270`.

## RX8130CE

Official / schematic `0x32`. The Epson extract’s address bits
are garbled — do not invent a 7-bit from it. I2C up to 400 kHz.
INT to M5PM1 G0. L0 keeps RTC on battery. Timer wake is
documented
([nyc-pm1-wake](../resources/not-yet-confirmed.md#nyc-pm1-wake)).
Read-only `FLAG` (`0x1D`): Lite CDC `rtc_flag=31`
(2026-09-02). Catalog id `rx8130ce`, Register Table Flag
Register bits `[7:0]` `VBLF` / `0` / `UF` / `TF` / `AF` /
`RSF` / `VLF` / `VBFF`. `0x31` is `UF|TF|VBFF`. Do not
write `SEC`. `C153` still
[nyc-rx8130](../resources/not-yet-confirmed.md#nyc-rx8130).

Registers: cache id `rx8130ce`. UserDemo uses four bytes of
user RAM from register base `0x20` (battery UI mode in index
0; brightness + mute in index 1). That packing is firmware
NVRAM, not a datasheet map.

## PDM microphone

Part: LMD4737T261-AC02. CLK GPIO45, DAT GPIO46, enable M5IOE1
`PYG12`. These are separate from the ESP32-S3 USB pads (GPIO19/20).

Rate / slot / hole energy:
[nyc-pdm-mic](../resources/not-yet-confirmed.md#nyc-pdm-mic).
Lite PDM bring-up idle (80 ms, parked): `rms≈1356–1422`
`peak=12917`. Later live 16 kHz **right**: quiet
`rms≈1370–1395` (`peak≈14029` is the window-start
spike). Phone A through the hole, BUTTON A dump:
`mic pcm hz=0 n=256`, DC floor **−8**, sine-like tail
period ~32–44 samples
([measure.md](measure.md)). UserDemo
`hal_mic.cpp` intent: `I2S_NUM_0`, 16 kHz,
`input_only_right`, `PYG12` off then on before `M5.Mic.begin`.
Hold `PYG12` off when unused so the capsule is not
half-powered. After `Hal::initRgb`, UserDemo leaves `PDM_EN` **high**
until the mic app toggles it.

GPIO45/46 are also strapping pins (WPD). Do not treat PDM bring-up
as a power latch.

## RGB LED

Three dies. Red: M5PM1 `LED_EN_PP` (no PWM; register `0x13` bit `0x20`
enables push-pull drive, and register `0x06` bit `0x10` sets output
level; see
[LED_PaperMono_Class.cpp](https://github.com/m5stack/M5Unified/blob/8530f5377d782e4a25a6c482de2e71c3f75ca8eb/src/utility/led/LED_PaperMono_Class.cpp)).
Green: M5IOE1 PYG8 PWM channel 2. Blue: M5IOE1 PYG9 PWM channel 3.
Official: adjustable color range is limited because red is not PWM.
[nyc-rgb-led](../resources/not-yet-confirmed.md#nyc-rgb-led).
Factory demo / M5Unified PWM: 5 kHz, 8-bit duty.

After power-on, red follows M5PM1 default (download-mode blink
uses this die).

Confirmed live on `C153-Lite` (2026-09-03): writing `0` to bit 4
(`0x10`) of M5PM1 register `0x06` (`PWR_CFG`) extinguishes the
red status LED, and writing `1` turns it back on. Used in
`embassy-debug-fw` to eliminate the 1–3 mA ballast current drain
during low-power light sleep.
