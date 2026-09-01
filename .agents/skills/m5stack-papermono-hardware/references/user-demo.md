# M5PaperMono-UserDemo (official eval HAL)

[m5stack/M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)
is official **ESP-IDF eval firmware** (MIT) for both SKUs.
It proves **intent and ordering**, never electrical fact.
Do not treat it as a live dump of a shipping unit, and do not
flash it from this skill.

Pinned while writing this page: commit
`c1099107271d31a0678d661a896e2b04dbb331ea` (2026-08-10,
README firmware **V1.2**). ESP-IDF **v5.5.1**, target
`esp32s3`. Re-read upstream if a later tag disagrees.

Product-page aliases in that README:
[C153](https://docs.m5stack.com/en/products/sku/C153) and
[C153-LITE](https://docs.m5stack.com/en/products/sku/C153-LITE).
This skill’s HTML map stays
[catalog.md](catalog.md).

## What it is

One IDF application. `main/hal/` is the board HAL. `main/apps/`
are Mooncake demos (display refresh, clock, battery, brightness,
IMU, mic waveform, RGB, buzzer, Wi-Fi scan, NFC scan, LoRa, TF
card, sleep/wake, shutdown). `patches/` plus `repos.json` /
`fetch_repos.py` pin M5GFX `develop`, M5Unified `develop`,
M5PM1, M5IOE1, M5Unit-NFC, RadioLib, and friends.

It is **Arduino-on-IDF** (`espressif/arduino-esp32`), not a
bare-metal IDF driver tree. OTP waveforms belong to
[M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo),
not this repo.

This repository does not add an IDF project and does not run
`idf.py`.

## One binary, runtime SKU

`Hal::detectBoardVariant` (`hal_board.cpp`) is the SKU gate,
not a compile-time Lite vs full image.

1. Init M5PM1 `0x6E` and M5IOE1 `0x4F` (100 kHz; up to three
   retries). If that fails, treat the unit as Lite and skip
   RGB / NFC / TF expander controls.
2. Drive `M5IOE1_PIN_4` (`NFC_EN`) high, wait 50 ms, then
   `Hal::probeNfcIdentity` at I2C `0x50` (400 kHz).
3. UserDemo names the command `NFC_READ_IC_IDENTITY_CMD`
   (`0x7F`) and accepts type `0x05` with a non-zero rev. Cite
   those **UserDemo constants**, not an unread ST25R3916
   register map.
4. Drop `NFC_EN` low. `BoardVariant::Pro` if the probe
   succeeded, else `Lite`. `hasNfcHardware()` is Pro only.

`app_main` installs `AppNfcScan` and `AppLora` **only** when
`hasNfcHardware()` is true. Lite backgrounds/logos are
`img_bg_lite` / `img_logo_lite`. Do not init ST25R3916 or
SX1262 on `C153-Lite` just because the same ELF can run on
both.

## Bring-up (`Hal::init` / `app_main`)

Vendor Arduino pages still say “talk to M5PM1, then M5IOE1.”
UserDemo’s **eval** order is different; name both
([sources.md](sources.md)).

1. `app_main` waits 500 ms (“M5IOE1 and board I2C peripherals
   … stable after power-up”).
2. `gpio_install_isr_service`, `initArduino`, `M5.begin` with
   `clear_display`, `internal_mic`, `internal_spk`, and
   `internal_imu` all false. Display and touch come from
   M5Unified here.
3. `display.setRotation(0)` and `setAutoDisplay(false)`.
4. `Hal::initRgb`: M5PM1 / M5IOE1, clear PM1 wake IRQs, set
   RGB G/B, `NFC_EN` low, `PDM_EN` (`M5IOE1_PIN_12`) **high**,
   TF enable/detect pins. RGB PWM 5 kHz; green
   `M5IOE1_PIN_8` / `M5IOE1_PWM_CH2`; blue `M5IOE1_PIN_9` /
   `M5IOE1_PWM_CH1`; red `pm1.setLedEnLevel`.
5. `detectBoardVariant` as above.
6. `Hal::initImu` (`M5.Imu.begin` on `M5.In_I2C`; accel+gyro).
7. `Hal::initBuzzer` (LEDC on GPIO42). Mic, LoRa, and TF mount
   stay deferred until those apps run.

GPIO numbers match [pin-map.md](pin-map.md). Expander polarity
is still not measured.

## M5IOE1 driver (`m5stack/M5IOE1`)

UserDemo calls `ioe1.begin(&M5.In_I2C, 0x4F, 100000)` after
`M5.begin` and `pm1.begin(0x6E)`. The Arduino library, not
the chip UM strap table, is the eval **sequence**.

- Library default address is `0x6F`. Board/UserDemo is
  `0x4F` (`M5IOE1_DEFAULT_ADDR_2`). If `0x4F` fails, the
  library’s next candidate is **`0x6F` only**. Auto-detect
  `0xFF` also walks `0x70`–`0x76` (that range includes
  charger `0x75`). UserDemo does **not** auto-detect. Do
  not copy that walk while `PYG11` is unknown.
- REV ASCII: `0x6F`–`0x76` expect `'A'` (UM). `0x4F`–`0x50`
  expect `'W'` (board firmware). A `'W'` read is a match at
  `0x4F`, not a bad chip.
- Wake is START + address-write + STOP with **no data**
  (`beginTransmission` / `endTransmission`, or M5Unified
  `start`+`stop`). ACK during wake may timeout; the library
  ignores that, waits 10 ms, then reads UID (16-bit) and
  REV. Retry: 100 kHz, wait **800 ms**, 100 kHz, then
  400 kHz.
- `I2C_CFG` (`0x23`) SLEEP default 0 = no idle sleep (UM).
  The wake is still part of `begin`. After success,
  UserDemo `pm1.setI2cSleepTime(0)` twice.

Bare-metal that only `read`s `0x4F` without that wake/retry
is not the eval probe. Lite silicon: that `read` NAKed;
official `begin` ACKed at board `0x4F` (`ioe_addr=4f`)
([measure.md](measure.md)).

## Display (`hal_display.cpp`)

Rotation 0 with official 480×800 tables. FreeInk 800×480 stays
a conflict
([nyc-canvas-orient](../resources/not-yet-confirmed.md#nyc-canvas-orient)).

Idle analog-off is UserDemo **intent**, not a closed
[nyc-otp-busy](../resources/not-yet-confirmed.md#nyc-otp-busy)
row. After a non-quality refresh it waits 500 ms, then:

- `writeCommand(0x22)` / `writeData(0x03)` / `writeCommand(0x20)`
  to drop analog supply and clock
- `0xC0` instead of `0x03` to turn them back on

Quality/text refreshes leave power-down to M5GFX. Fastest
refreshes: ten local updates then one `epd_fast`; five fast
then one `epd_quality`. Those four labels are the firmware
enum titles; official HTML lab times (PaperMono,
reference only) are in [display.md](display.md). Snapshot
cache uses four gray
representatives `{0, 96, 160, 255}`. Do not copy Sticky’s
analog-off result onto this panel.

## microSD (`hal_tf_card.cpp`)

Enable `M5IOE1_PIN_14`, detect `M5IOE1_PIN_1` (insert =
**LOW**). Host pins: DAT3 GPIO8, DAT2 GPIO9, DAT1 GPIO10,
DAT0 GPIO11, CMD GPIO12, CLK GPIO13. Mount is
`esp_vfs_fat_sdmmc_mount` at `/sdcard` with
`slot_config.width = 4` after `TF_EN` high and a 300 ms wait.

That is **4-bit intent**. Official pin tables also name
DAT0–DAT3. FreeInk says 1-bit. Live width:
[nyc-sdmmc-width](../resources/not-yet-confirmed.md#nyc-sdmmc-width).
Detect polarity:
[nyc-tf-det](../resources/not-yet-confirmed.md#nyc-tf-det).

## LoRa (`hal_lora.cpp`, full SKU / Pro only)

GPIOs match the pin table (MOSI 38, MISO 40, SCK 39, NSS 41,
DIO1 5, BUSY 21). Enable M5PM1 `GPIO_NUM_2`, reset
`M5IOE1_PIN_10`, antenna switch `M5IOE1_PIN_2`. RadioLib
`SX1262` at **8 MHz** SPI.

UserDemo attaches that bus to ESP-IDF **`SPI3_HOST`**, not
`SPI1_HOST` (SPI0/1 are flash/PSRAM on ESP32-S3). Pin-map
“SPI1” is the schematic / product-table name for those GPIOs.
Do not flatten the host id
([sources.md](sources.md)).

RadioLib `begin` in this tree: 868.0 MHz, 62.5 kHz BW, SF12,
CR 8, sync `0x34`, 22 dBm, TCXO 3.0 V, LDO regulator, 140 mA
current limit, DIO2 as RF switch. Product docs list
**868–923 MHz**; 868.0 is the **EU demo default**, not a
claim that the stamp is EU-only.

## Mic, buzzer, RTC RAM

PDM (`hal_mic.cpp`): `I2S_NUM_0`, CLK GPIO45, DAT GPIO46,
16 kHz, `input_only_right`, `PYG12` toggled off then on before
`M5.Mic.begin`. Rate/slot on a unit:
[nyc-pdm-mic](../resources/not-yet-confirmed.md#nyc-pdm-mic).

Buzzer (`hal_buzzer.cpp`): GPIO42, LEDC low-speed timer 3 /
channel 7, 10-bit, 50% duty, 40–12000 Hz.

RX8130CE user RAM in this firmware: four bytes from register
base `0x20`. Battery UI mode in index 0; brightness + mute
packed in index 1. That packing is **UserDemo NVRAM**, not a
chip datasheet map.

## Sleep and shutdown (`app_sleep_wake`, `app_shutdown`)

Three eval paths. Currents are not measured
([nyc-pm1-wake](../resources/not-yet-confirmed.md#nyc-pm1-wake),
[nyc-sleep-current](../resources/not-yet-confirmed.md#nyc-sleep-current)).

| Path | What runs | Wake |
| --- | --- | --- |
| IMU | Configure BMI270 any-motion on INT1 (`0x68`, fallback `0x69`); `pm1.shutdown()` with LDO hold | PM1 G4 falling |
| Touch | ESP `esp_deep_sleep_start`; EXT0 on GPIO4 low; keep touch EN/RST high; drop NFC/TF/PDM/LoRa/RGB | GPIO4 low |
| RTC 10 s | `rtc.setTimerIRQ(10000)` then `pm1.shutdown()` | PM1 G0 falling (RX8130) |
| Shutdown | Unmount TF, backlight 0, `ldoSetPowerHold(false)`, `pm1.shutdown()` | Power button |

Touch deep sleep also sets PM1/M5IOE1 I2C idle sleep to **1 s**
and isolates other RTC GPIOs (keys, LoRa IRQ, RFID IRQ, PM1
IRQ). Brightness uses `display.setBrightness` (M5GFX → M5PM1
G3 PWM into AW9967).

## Flash geometry (`partitions.csv`, `sdkconfig.defaults`)

UserDemo CSV (intent in source; **Lite stock matches**):

| Name | Type | Offset | Size |
| --- | --- | ---: | ---: |
| nvs | data/nvs | `0x9000` | `0x6000` |
| phy_init | data/phy | `0xf000` | `0x1000` |
| factory | app/factory | `0x10000` | `0xF00000` |

Lite capture 2026-09-01 parsed that table at `0x8000`
([measure.md](measure.md)). `C153` still
[nyc-partition-table](../resources/not-yet-confirmed.md#nyc-partition-table).

Lite factory app descriptor: project `PaperMono-UserDemo`,
IDF `v5.5.1`, version `c78f6c5-dirty`, date Aug 6 2026 —
not the GitHub V1.2 pin (`c109910`).

`sdkconfig.defaults`: octal SPIRAM, CPU 240 MHz
(`CONFIG_ESP_DEFAULT_CPU_FREQ_MHZ_240`),
`CONFIG_ESPTOOLPY_FLASHSIZE_16MB`, custom partition file,
FATFS LFN/UTF-8. Runtime MHz / DIO vs QIO on a unit:
[nyc-cpu-flash-runtime](../resources/not-yet-confirmed.md#nyc-cpu-flash-runtime).

PlatformIO product-page env still points at
`default_16MB.csv`. That is a **different** table than this
CSV. Do not copy Sticky `0x90000` / 32 MB.

## NFC app (Pro only)

`app_nfc_scan.cpp` uses M5Unit-NFC on `M5.In_I2C` after
`Hal::setNfcPower(true)` (120 ms). IRQ unused
(`cfg.using_irq = false`). Scans NFC-A/B/F/V. Power down
calls `disableField` then `setNfcPower(false)`. Do not leave
RF on.

## Do not

- Flash or `idf.py` from this skill or from papermono-rs
  xtask.
- Treat Lite leftover NFC/LoRa pads as confirmed because
  UserDemo probes `0x50` and skips apps on NAK.
- Cite `0x7F` / type `0x05` as ST25R3916 datasheet fact.
- Copy RadioLib 868.0 MHz or 4-bit SDMMC into firmware as
  measured.
- Invent LUTs from this HAL; prefer OTP-Demo for panel
  opcodes.
