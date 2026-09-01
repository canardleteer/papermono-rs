# Sensors, mic, RGB

I2C devices share GPIO47/48. Addresses:
[pin-map.md](pin-map.md). None ACKed on a physical unit yet
(name `C153` vs `C153-Lite`)
([nyc-i2c-ack](../resources/not-yet-confirmed.md#nyc-i2c-ack)).

## BMI270

Official / schematic `0x68`. Sheet default if SDO is to GND
(`0x69` if SDO to VDDIO). INT1 goes to M5PM1 G4 (wake), not to
a raw ESP32 GPIO as on Sticky GPIO7. Do not drive an ESP pin
as IMU INT. Any-motion wake through the PMIC is documented
Arduino intent; UserDemo `configureBmi270AnyMotion` tries
`0x68` then `0x69` and maps any-motion to INT1
([user-demo.md](user-demo.md)). Confirm on a unit:
[nyc-pm1-wake](../resources/not-yet-confirmed.md#nyc-pm1-wake).

Registers: cache id `bmi270`.

## RX8130CE

Official / schematic `0x32`. The Epson extract’s address bits
are garbled — do not invent a 7-bit from it. I2C up to 400 kHz.
INT to M5PM1 G0. L0 keeps RTC on battery. Timer wake is
documented
([nyc-pm1-wake](../resources/not-yet-confirmed.md#nyc-pm1-wake)).

Registers: cache id `rx8130ce`. UserDemo uses four bytes of
user RAM from register base `0x20` (battery UI mode in index
0; brightness + mute in index 1). That packing is firmware
NVRAM, not a datasheet map.

## PDM microphone

Part: LMD4737T261-AC02. CLK GPIO45, DAT GPIO46, enable M5IOE1
`PYG12`. These are **not** ESP32-S3 USB pads (those are
GPIO19/20). Sticky PDM-on-19/20 notes do not apply.

Rate / slot / hole energy:
[nyc-pdm-mic](../resources/not-yet-confirmed.md#nyc-pdm-mic).
UserDemo `hal_mic.cpp` intent: `I2S_NUM_0`, 16 kHz,
`input_only_right`, `PYG12` off then on before `M5.Mic.begin`.
Hold `PYG12` off when unused so the capsule is not
half-powered (same class of caution as Sticky GPIO38, different
pin). After `Hal::initRgb`, UserDemo leaves `PDM_EN` **high**
until the mic app toggles it.

GPIO45/46 are also strapping pins (WPD). Do not treat PDM bring-up
as a power latch.

## RGB LED

Three dies. Red: M5PM1 `LED_EN_PP` (no PWM). Green: M5IOE1
PYG8. Blue: M5IOE1 PYG9. Official: adjustable color range is
limited because red is not PWM.
[nyc-rgb-led](../resources/not-yet-confirmed.md#nyc-rgb-led).
UserDemo PWM: 5 kHz, green `M5IOE1_PWM_CH2`, blue
`M5IOE1_PWM_CH1`, red `pm1.setLedEnLevel`.

After power-on, red follows M5PM1 default (download-mode blink
uses this die).
