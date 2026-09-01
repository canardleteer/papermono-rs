# Vendor C++ / PlatformIO (wiring evidence)

Official Arduino, ESP-IDF, and PlatformIO trees prove **intent
and ordering**, never electrical fact. Pin numbers they use
should match [pin-map.md](pin-map.md); if they disagree, add a
[sources.md](sources.md) row. Do not flash from this skill.

This repository does not add an IDF project and does not flash
with `idf.py` / PlatformIO.

## PlatformIO (official product page)

PaperMono (`C153`):

```ini
[env:m5stack-papermono]
platform = espressif32@6.12.0
board = esp32-s3-devkitm-1
framework = arduino
board_build.partitions = default_16MB.csv
board_upload.flash_size = 16MB
board_upload.maximum_size = 16777216
board_build.arduino.memory_type = qio_opi
build_flags =
    -DESP32S3
    -DBOARD_HAS_PSRAM
    -mfix-esp32-psram-cache-issue
    -DCORE_DEBUG_LEVEL=0
    -DARDUINO_USB_CDC_ON_BOOT=1
    -DARDUINO_USB_MODE=1
lib_deps =
    M5Unified = https://github.com/m5stack/M5Unified#develop
    M5PM1 = https://github.com/m5stack/M5PM1
    M5IOE1 = https://github.com/m5stack/M5IOE1
    M5Unit-NFC=https://github.com/m5stack/M5Unit-NFC
    RadioLib = https://github.com/jgromes/RadioLib
```

PaperMono-Lite: same env name
`m5stack-papermono-lite`, **without** `M5Unit-NFC` and
`RadioLib`.

`board = esp32-s3-devkitm-1` is a PIO board id, not this
product’s pinout. Pins still come from M5Unified + this skill.

## ESP-IDF

- Factory / eval:
  [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)
  (MIT). HAL under `main/hal/`. `partitions.csv` is intent
  ([nyc-partition-table](../resources/not-yet-confirmed.md#nyc-partition-table)).
- OTP:
  [M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo)
  (MIT). Direct SSD1677 + OTP; names
  `DEPG0397BBS770F3HP-XM`.

Include order (ESP-IDF): `M5Unified.h` before `M5PM1.h` /
`M5IOE1.h` unless `CONFIG_I2C_BUS_BACKWARD_CONFIG` (upstream
README).

## Arduino / UiFlow2

M5Unified `develop` branch is what the product page pins.
Power-rail examples:
[M5PM1 & M5IOE1](https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1).

## FreeInk

`-DFREEINK_DEVICE_PAPERMONO`. Wiring comments in
[external.md](../resources/external.md). LUT / SDMMC / canvas
conflicts stay in [sources.md](sources.md).
