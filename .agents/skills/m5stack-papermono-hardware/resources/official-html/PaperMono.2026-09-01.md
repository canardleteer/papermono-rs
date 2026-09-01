# PaperMono

<span class="product-sku">SKU:C153</span>

<PictureViewer>
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_01.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_02.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_03.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_04.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_05.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_06.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_07.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_08.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_09.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_10.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_11.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_12.webp">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153-weight.jpg">
</PictureViewer>

## Description

**PaperMono** is an e-paper development board built around low-power display, near-field identification, and long-range communication capabilities. It features a touch-enabled 3.97-inch 4-level grayscale monochrome e-paper display, driven by SSD1677 with a resolution of 480 x 800, and integrates an FT6336G touch controller and a frontlight system for a more comfortable reading and interaction experience in low-light environments. The core adopts an ESP32-S3R8, paired with 16MB Flash, 8MB PSRAM, and 2.4 GHz Wi-Fi, along with an onboard buzzer, PDM microphone (LMD4737T261-AC02), RGB LED, BMI270 6-axis IMU, M5IOE1 IO expander, microSD card slot, and RX8130CE real-time clock. The NFC section is based on the ST25R3916, supporting ISO14443A, ISO14443B, FeliCa™, and ISO15693; LoRa communication adopts the SX1262 (Stamp LoRa-1262) solution with a built-in FPC antenna, supporting the 868MHz ~ 923MHz band. Combined with the M5PM1 multi-level power management system and a 1150mAh battery, it meets the requirements of low-power IoT and embedded applications such as e-readers, e-paper signage, access control terminals, identity authentication, and intelligent transportation.

## Tutorial

learn>| ![UiFlow2](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/assets/img/uiflow2/uiflow2.0_banner_01.png) | [UiFlow2](/en/uiflow2/papermono/program) | This tutorial will introduce you to controlling the PaperMono device through the UiFlow2 graphical programming platform. |

learn>| ![Arduino IDE](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/assets/img/arduino/arduino_banner_01.png) | [Arduino IDE](/en/arduino/papermono/program) | This tutorial introduces how to program and control the PaperMono device using Arduino IDE. |

## Note

### E-Paper Usage Precautions

1. Avoid direct sunlight/prolonged sun exposure during use; high temperatures or strong UV can damage the panel.
2. After approximately 10 partial fast refreshes in software, it is recommended to perform one full-screen refresh to clear the screen and avoid severe accumulated ghosting.
3. The screen has built-in OTP waveforms that can be called directly. When using custom external waveforms, pay attention to DC balance; otherwise, irreversible damage may be caused to the panel.
4. Avoid uninterrupted continuous partial fast refreshes, to prevent long-term DC imbalance from causing irreversible damage to the panel.

### Touch Active Area

#> PaperMono Touch Active Area | The touch IC firmware internally shrinks the touch boundary. The effective touch coordinate range is limited to: **X-axis: 5 ~ 475** (total width 480px), **Y-axis: 5 ~ 795** (total height 800px).

## Features

- ESP32-S3R8 Core Controller
  - 16MB Flash
  - 8MB PSRAM
  - 2.4 GHz Wi-Fi
- 3.97-inch 4-level grayscale monochrome e-paper display with touch
  - SSD1677
  - 480x800 resolution
  - 4-level grayscale display
  - FT6336G touch
- Integrated frontlight system
- Audio interaction
  - PDM microphone (LMD4737T261-AC02)
  - Built-in buzzer
- Onboard peripherals
  - BMI270 6-axis IMU
  - M5IOE1 IO expander
  - microSD card slot
  - RX8130CE RTC
  - RGB LED
- NFC function
  - ST25R3916
  - Supports ISO14443A / ISO14443B / FeliCa™ / ISO15693
- LoRa communication
  - SX1262 (Stamp LoRa-1262)
  - 868MHz ~ 923MHz
  - Built-in FPC antenna
- Built-in 1150mAh battery

## Includes

- 1 x PaperMono

## Applications

- E-Reader
- E-Paper Signage
- Access Control Terminal
- Identity Authentication Device
- Intelligent Transportation Terminal

## Specifications

| Specification     | Parameter                                                       |
| ----------------- | --------------------------------------------------------------- |
| SoC               | ESP32-S3R8 @ Xtensa® 32-bit LX7 dual-core processor, 240MHz     |
| Flash             | 16MB                                                            |
| PSRAM             | 8MB Octal                                                       |
| Wi-Fi             | 2.4 GHz Wi-Fi                                                   |
| Screen            | 3.97" E-Paper (Mono) SSD1677 @ 480x800, Touch FT6336G           |
| Screen Grayscale  | 4-level grayscale display                                       |
| Frontlight        | Integrated e-paper frontlight                                   |
| Input Power       | USB Type-C DC 5V                                                |
| Battery Capacity  | 1150mAh                                                         |
| Microphone        | PDM microphone LMD4737T261-AC02                                 |
| Buzzer            | Built-in buzzer                                                 |
| IMU               | BMI270                                                          |
| Power Management  | M5PM1                                                           |
| IO Expander       | M5IOE1                                                          |
| Expansion Storage | microSD                                                         |
| RTC               | RX8130CE                                                        |
| RGB LED           | Built-in RGB LED                                                |
| NFC               | ST25R3916 (ISO14443A/B, FeliCa™, ISO15693)                      |
| LoRa              | SX1262 (Stamp LoRa-1262), 868MHz ~ 923MHz, built-in FPC antenna |
| User Buttons      | 2x user buttons + 1x power button (ON / OFF / RESET / BOOT)     |
| Product Size      | 62.0 x 101.0 x 8.0mm                                            |
| Product Weight    | 74.7g                                                           |
| Package Size      | 113.2 x 69.6 x 21.0mm                                           |
| Gross Weight      | 91.3g                                                           |

## Learn

### Power On/Off

- Power on / Reset: press the power button once briefly
- Power off: press the power button twice in succession

<video style="width:50%;" muted playsinline preload="auto" controls>
    <source src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_poweron_EN.mp4" type="video/mp4">
</video>

<!--English video: https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_poweron_EN.mp4-->

### Download Mode

Press and hold the power button (about 2 seconds) until the red LED blinks, then release. The device enters download mode and is ready for flashing.

<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_download.gif" width="50%">

### M5GFX LUT Refresh Speed

The following data is the laboratory test result of PaperMono under different M5GFX refresh modes. The actual refresh time may vary depending on the display content and operating environment, and is provided for reference only.

| Refresh Mode  | Time per Refresh |
| ------------- | ---------------- |
| `epd_quality` | 4.71 s           |
| `epd_text`    | 0.45 s           |
| `epd_fast`    | 0.34 s           |
| `epd_fastest` | 0.07 s           |

**Refresh Mode Demonstration:**

<video style="width:100%;" muted playsinline preload="auto" controls>
    <source src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_C153-Lite_reflash_mode_demo.mp4" type="video/mp4">
</video>

## Schematics

- [PaperMono Schematics PDF](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf)

<SchViewer>
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_01.png">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_02.png">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_03.png">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_04.png">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_05.png">
<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522_page_06.png">
</SchViewer>

## PinMap

### E-Paper

| ESP32-S3R8 | G14_SPI2_MOSI | G15_SPI2_CLK | G16_EINK_CS | G17_EINK_DC | G18_EINK_BUSY |
| ---------- | ------------- | ------------ | ----------- | ----------- | ------------- |
| SSD1677    | MOSI          | SCLK         | CS          | DC          | BUSY          |

| M5IOE1  | PYG5_ADC3 | PYG3       |
| ------- | --------- | ---------- |
| SSD1677 | RST       | EPD_3V3_EN |

| M5PM1      | G3_WAKEin / IRQout / PWM |
| ---------- | ------------------------ |
| Frontlight | BL_FB                    |

The e-paper reset is controlled via `PYG5_ADC3` of the `M5IOE1`, and `PYG3` controls the 3.3V power supply of the e-paper; the frontlight brightness is controlled via `PYG3_BL_PWM` of the `M5PM1`.

### Touch

| ESP32-S3R8    | G47_SYS_SDA | G48_SYS_SCL | G4_TP_INT |
| ------------- | ----------- | ----------- | --------- |
| FT6336G(0x38) | SDA         | SCL         | INT       |

| M5IOE1  | PYG6   | PYG13     |
| ------- | ------ | --------- |
| FT6336G | TP_RST | TP_VDD_EN |

### microSD

| ESP32-S3R8 | G12 | G13 | G11  | G10  | G9   | G8   |
| ---------- | --- | --- | ---- | ---- | ---- | ---- |
| microSD    | CMD | CLK | DAT0 | DAT1 | DAT2 | DAT3 |

| M5IOE1  | PYG14 | PYG1   |
| ------- | ----- | ------ |
| microSD | TF_EN | TF_DET |

`TF_DET` is pulled up by the microSD power domain. When the card slot switch is closed, the detection pin is pulled low to identify microSD card insertion.

### HMI

| ESP32-S3R8           | G47_SYS_SDA | G48_SYS_SCL | G2_KEY1 | G3_KEY2 |
| -------------------- | ----------- | ----------- | ------- | ------- |
| RTC - RX8130CE(0x32) | SDA         | SCL         |         |         |
| IMU - BMI270(0x68)   | SDA         | SCL         |         |         |
| KEY                  |             |             | KEY1    | KEY2    |

| M5PM1 | G0_WAKEin | G4_WAKEin | LED_EN_PP |
| ----- | --------- | --------- | --------- |
| HMI   | RTC_INT   | IMU_INT   | LED_R     |

`PYG0_RTC_INT` and `PYG4_IMU_INT` are connected to the `M5PM1` and can serve as low-power wake-up sources.

| M5IOE1 | PYG8_PWM2 | PYG9_PWM1 |
| ------ | --------- | --------- |
| HMI    | LED_G     | LED_B     |

The RGB LED indicator on the side of the PaperMono body consists of three color LED dies, with the red LED connected to `LED_EN_PP` of the M5PM1. After the device is powered on, it will light up according to the M5PM1 default behavior. Since `LED_EN_PP` does not support PWM output mode configuration, the adjustable colors of this indicator will be limited.

### KEY

| ESP32-S3R8 | G2                   | G3                   |
| ---------- | -------------------- | -------------------- |
| KEY        | USER_KEY1 (Button A) | USER_KEY2 (Button B) |

### Audio

| ESP32-S3R8 | G46_PDM_DAT | G45_PDM_CLK | G42_BB_PWM |
| ---------- | ----------- | ----------- | ---------- |
| PDM MIC    | DAT         | CLK         |            |
| BUZZER     |             |             | BB_PWM     |

| M5IOE1 | PYG12      |
| ------ | ---------- |
| PDM    | PDM_VDD_EN |

### M5PM1

| ESP32-S3R8  | G47_SYS_SDA | G48_SYS_SCL | G1_PY_IRQ | G0_BOOT_OUT |
| ----------- | ----------- | ----------- | --------- | ----------- |
| M5PM1(0x6E) | SDA         | SCL         | IRQ       | BOOT_OUT    |

### M5IOE1

| ESP32-S3R8   | G47_SYS_SDA | G48_SYS_SCL | G7      |
| ------------ | ----------- | ----------- | ------- |
| M5IOE1(0x4F) | SDA         | SCL         | PYB_IRQ |

| M5IOE1                 | PYG11_PWM3  |
| ---------------------- | ----------- |
| Charger - IP2315(0x75) | PYB_CHG_IIC |

?>Note|The I2C operating mode of the charging chip `IP2315` depends on the I2C pins being pulled up to VBAT voltage. Connecting the device to USB triggers mode detection; if the VBAT voltage is too low, the IP2315 may not initialize into I2C mode properly, which can interfere with other devices on the same I2C bus. M5IOE1 controls the connection between the IP2315 and the system I2C bus via PYG11_PWM3: during device operation, `do not keep the IP2315 mounted on the I2C bus for extended periods; disconnect it promptly after communication to avoid reducing bus communication stability`. If an I2C bus communication anomaly occurs, you can reset the device by briefly pressing the power button to restore normal bus operation.

### RFID

| ESP32-S3R8      | G47_SYS_SDA | G48_SYS_SCL | G6_RFID_INT |
| --------------- | ----------- | ----------- | ----------- |
| ST25R3916(0x50) | SDA         | SCL         | IRQ         |

### LoRa

| ESP32-S3R8      | G38_SPI1_MOSI | G40_SPI1_MISO | G39_SPI1_CLK | G41    | G21     | G5       |
| --------------- | ------------- | ------------- | ------------ | ------ | ------- | -------- |
| Stamp LoRa-1262 | SPI_MOSI      | SPI_MISO      | SPI_CLK      | SX_NSS | SX_BUSY | LORA_IRQ |

| M5IOE1          | PYG10_PWM4 | PYG2_ADC1 |
| --------------- | ---------- | --------- |
| Stamp LoRa-1262 | SX_NRST    | SX_ANT_SW |

| M5PM1           | G2      |
| --------------- | ------- |
| Stamp LoRa-1262 | LoRa_EN |

## Model Size

- [PaperMono Model Size PDF](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_model_size.pdf)

<img src="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_model_size_page_01.png" width="100%">

## Datasheets

- [ESP32-S3](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/472/esp32-s3_datasheet_en.pdf)
- [NFC ST25R3916](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1205/ST25R3916_EN.pdf)
- [SSD1677 Display Driver](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/SSD1677.pdf)
- [3.97-inch Touchscreen](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/EPD_Module_User_Manual.pdf)
- [RX8130CE Register Datasheet](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1132/RX8130CE_cn-Register-Datasheet.pdf)
- [BMI270](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/products/app/Stamp%20Fly/BMI270.PDF)
- [SX1262](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1177/DS_SX1261_2_V2-2.pdf)

## Softwares

?> E-Paper Driver Notes | 1. The e-paper driver waveforms for PaperMono in the M5GFX library are currently unstable. It is recommended to prioritize the e-paper manufacturer's OTP example below for refresh configuration to achieve better panel life and refresh stability.<br/>2. Software recommendation: After approximately 10 partial fast refreshes, perform one full-screen refresh to clear the screen and prevent ghosting from accumulating.<br/>3. After prolonged repeated refreshing, black deposited pixels may appear on the panel due to the physical characteristics of the ink particles. Leave the panel idle for a while, then perform one full-screen refresh to restore it.

### Arduino

- [PaperMono Arduino Quick Start](/en/arduino/papermono/program)
- [PaperMono M5PM1 & M5IOE1 Power Management](/en/arduino/papermono/m5pm1_m5ioe1)
- [M5PM1 Arduino Library](https://github.com/m5stack/M5PM1)
- [M5IOE1 Arduino Library](https://github.com/m5stack/M5IOE1)
- [M5Unified Arduino Library](https://github.com/m5stack/M5Unified)
- [M5GFX Arduino Library](https://github.com/m5stack/M5GFX)

### UiFlow2

- [PaperMono UiFlow2 Quick Start](/en/uiflow2/papermono/program)

### Protocol

- [M5PM1 Power Management Chip](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1207/M5PM1_Datasheet_EN.pdf)
- [M5IOE1 IO Expander Management Chip](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1210/IO_Expander_Datasheet_EN.pdf)

### ESP-IDF

- [PaperMono Factory Firmware Source Code](https://github.com/m5stack/M5PaperMono-UserDemo)
- [PaperMono OTP Example](https://github.com/m5stack/M5PaperMono-OTP-Demo)

### PlatformIO

```bash
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

### Easyloader

| Easyloader          | Download                                                                                           | Note |
| ------------------- | -------------------------------------------------------------------------------------------------- | ---- |
| PaperMono User Demo | [download](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153-PaperMono-UserDemo_0x00.exe) | /    |

### Other

- [PaperMono Factory Reset Firmware](https://burner.m5stack.com/firmware/2089640807996628993/)
- [PaperMono CrossPoint E-Reader](https://burner.m5stack.com/firmware/2091144466157694978/)

## Video

<VideoGallery>
  <VideoItem title="PaperMono / PaperMono-Lite Product Introduction and Feature Demonstration" url="https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_and_C153-Lite_PaperMono_video_EN.mp4" />
</VideoGallery>

## Product Comparison

::compare-table
| Product Compare   | [PaperMono](/en/core/PaperMono) ![PaperMono](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/C153_PaperMono_main_pictures_02.webp) | [PaperMono-Lite](/en/core/PaperMono-Lite) ![PaperMono-Lite](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/C153-Lite_PaperMono-Lite_main_pictures_02.webp) |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Main Controller   | ESP32-S3R8                                                                                                                               | ESP32-S3R8                                                                                                                                                        |
| Screen            | 3.97" 4-level grayscale monochrome e-paper, 480x800                                                                                      | 3.97" 4-level grayscale monochrome e-paper, 480x800                                                                                                               |
| Touch             | FT6336G                                                                                                                                  | FT6336G                                                                                                                                                           |
| Frontlight        | Integrated                                                                                                                               | Integrated                                                                                                                                                        |
| Wi-Fi             | 2.4 GHz                                                                                                                                  | 2.4 GHz                                                                                                                                                           |
| NFC               | ST25R3916                                                                                                                                | ❌                                                                                                                                                                 |
| LoRa              | Stamp LoRa-1262                                                                                                                          | ❌                                                                                                                                                                 |
| Expansion Storage | microSD                                                                                                                                  | microSD                                                                                                                                                           |
| IMU               | BMI270                                                                                                                                   | BMI270                                                                                                                                                            |
| RTC               | RX8130CE                                                                                                                                 | RX8130CE                                                                                                                                                          |
| Battery           | 1150mAh                                                                                                                                  | 1150mAh                                                                                                                                                           |
| Case Color        | Gray                                                                                                                                     | White                                                                                                                                                             |
:::
