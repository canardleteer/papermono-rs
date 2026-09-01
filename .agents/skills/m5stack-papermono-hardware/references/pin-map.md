# Pin and bus map

GPIO numbers are ESP32-S3 package pins. Levels and expander
polarity below are **official pin tables**, not measured.
Primary table source: the **PinMap** heading on
[PaperMono](https://docs.m5stack.com/en/core/PaperMono) and
[PaperMono-Lite](https://docs.m5stack.com/en/core/PaperMono-Lite)
([catalog.md](catalog.md)). Those HTML pages can change; re-read
them when a net disagrees with this file. Dated **view as
markdown** snapshot (2026-09-01):
[official-html/SOURCE.md](../resources/official-html/SOURCE.md).
[nyc-i2c-ack](../resources/not-yet-confirmed.md#nyc-i2c-ack).
SKU column: both, full (`C153`) only, or Lite-absent.

Do not copy Sticky GPIO numbers onto this product.

## ESP32-S3 GPIO table

| GPIO | Direction | Signal | SKU | Notes |
| ---: | --- | --- | --- | --- |
| 0 | I/O | M5PM1 `BOOT_OUT` | both | **Strapping** (WPU). [nyc-gpio0-strap](../resources/not-yet-confirmed.md#nyc-gpio0-strap) |
| 1 | Input | M5PM1 IRQ | both | `G1_PY_IRQ` |
| 2 | Input | USER_KEY1, BUTTON A (UP) | both | Upper black key. Lite: idle high, press low |
| 3 | Input | USER_KEY2, BUTTON B (DOWN) | both | Lower black key. **Strapping** (floating at reset). Lite: idle high, press low |
| 4 | Input | FT6336G INT | both | `G4_TP_INT` |
| 5 | Input | LoRa IRQ | full | Lite: [nyc-lite-lora-pads](../resources/not-yet-confirmed.md#nyc-lite-lora-pads) |
| 6 | Input | ST25R3916 IRQ | full | Lite: [nyc-lite-nfc-pads](../resources/not-yet-confirmed.md#nyc-lite-nfc-pads) |
| 7 | Input | M5IOE1 IRQ | both | `PYB_IRQ` |
| 8 | SDMMC | DAT3 | both | |
| 9 | SDMMC | DAT2 | both | |
| 10 | SDMMC | DAT1 | both | |
| 11 | SDMMC | DAT0 | both | |
| 12 | SDMMC | CMD | both | |
| 13 | SDMMC | CLK | both | |
| 14 | Output | EPD MOSI | both | SPI2 |
| 15 | Output | EPD SCLK | both | SPI2 |
| 16 | Output | EPD CS | both | |
| 17 | Output | EPD DC | both | |
| 18 | Input | EPD BUSY | both | Sheet: high = busy. Glass: [nyc-otp-busy](../resources/not-yet-confirmed.md#nyc-otp-busy) |
| 19 | USB | USB D− | both | Native USB pads. PDM is **not** here |
| 20 | USB | USB D+ | both | Native USB pads |
| 21 | Input | SX1262 BUSY | full | No internal pull (Table 2-1). Lite: [nyc-lite-lora-pads](../resources/not-yet-confirmed.md#nyc-lite-lora-pads) |
| 38 | Output | LoRa SPI MOSI | full | Schematic/product “SPI1”. UserDemo `SPI3_HOST` |
| 39 | Output | LoRa SPI CLK | full | Mux off JTAG `MTCK` |
| 40 | Input | LoRa SPI MISO | full | Mux off JTAG `MTDO` |
| 41 | Output | SX1262 NSS | full | Mux off JTAG `MTDI` |
| 42 | PWM | Buzzer `BB_PWM` | both | Mux off JTAG `MTMS` |
| 45 | Output | PDM CLK | both | **Strapping** (WPD). Not a power latch |
| 46 | Input | PDM DAT | both | **Strapping** (WPD). Not a power latch |
| 47 | OD data | System I2C SDA | both | `G47_SYS_SDA` |
| 48 | OD clock | System I2C SCL | both | `G48_SYS_SCL` |

UART0 TX/RX (GPIO43/44) are ESP32-S3 defaults. Lite run mode
enumerates native Espressif USB-Serial/JTAG (`303a:1001`),
not a CH343 on UART0
([flashing.md](flashing.md#usb-measured)).
Whether UART0 also enumerates, and `C153` / download:
[nyc-usb-vid](../resources/not-yet-confirmed.md#nyc-usb-vid).

## I2C (system bus)

| Device | 7-bit | SKU | Notes |
| --- | ---: | --- | --- |
| FT6336G | `0x38` | both | Official pin map. **Not in** the public FT6336G PDF |
| RX8130CE | `0x32` | both | Schematic `IIC Adress:0x32`. Epson extract garbled |
| BMI270 | `0x68` | both | Sheet default if SDO to GND; schematic labels `0x68` |
| M5PM1 | `0x6E` | both | UM V 1.9; GPIO default open-drain |
| M5IOE1 | `0x4F` | both | Schematic / docs / UserDemo. Chip UM V 1.4 is `0x6F`–`0x76` from IO7. Library fallback `0x6F`; REV `'W'` at `0x4F`. Lite official `begin`: `ioe_addr=4f`. Lite UM `0x6F` NAK |
| IP2315 | `0x75` | both | 8-bit `0xEA`/`0xEB`. Gated by `PYG11_PWM3` |
| ST25R3916 | `0x50` | full | Sheet `50h`. Schematic `I2C_EN=VDD`. UserDemo SKU probe at this address |

One bus: SDA=47, SCL=48, schematic `IIC PULL_UP`. Chip caps:
M5PM1/M5IOE1 100 kHz default / 400 kHz; FT6336G SCL 10–400 kHz;
IP2315 400 kbps; ESP32-S3 Fast 400 kbit/s. Vendor examples are
often 100 kHz. Do not assume 400 kHz until measured.
Lite (2026-09-02) advertised scan after official `begin`
and FT EN/RST: `ack=32,38,4f,68,6e` (RTC / FT / board IOE /
IMU / PM1) and `nak=50,6f,75` (leftover NFC / UM IOE /
parked IP2315). `C153` still
[nyc-i2c-ack](../resources/not-yet-confirmed.md#nyc-i2c-ack).

M5PM1 and M5IOE1 GPIO outputs default **open-drain** (including
PWM). Pull-up or push-pull, or the pin does not drive high.

## M5PM1 pins used on this board

| M5PM1 | Function |
| --- | --- |
| SDA/SCL/IRQ/BOOT_OUT | ESP32 GPIO47/48/1/0 |
| G0 `WAKEin` | RTC INT |
| G2 | LoRa_EN (full SKU) |
| G3 PWM0 | Frontlight `BL_FB` into AW9967 (`EINK_BL`) |
| G4 `WAKEin` | IMU INT |
| `LED_EN_PP` | RGB red. Not PWM |

## M5IOE1 pins used on this board

Official Arduino table names `M5IOE1_PIN_n` as `PYGn`.

| M5IOE1 | Function | SKU |
| --- | --- | --- |
| PYG1 | microSD detect (`TF_DET`) | both |
| PYG2 | LoRa antenna switch | full |
| PYG3 | EPD 3.3 V enable | both |
| PYG4 | NFC enable | full |
| PYG5 | EPD RST | both |
| PYG6 | Touch RST | both |
| PYG8 PWM | RGB green | both |
| PYG9 PWM | RGB blue | both |
| PYG10 | LoRa reset | full |
| PYG11 | IP2315 I2C gate | both |
| PYG12 | PDM VDD enable | both |
| PYG13 | Touch VDD enable | both |
| PYG14 | microSD enable | both |

`TF_DET` is pulled up by the microSD power domain. Slot switch
closed pulls detect low.
[nyc-tf-det](../resources/not-yet-confirmed.md#nyc-tf-det).

## SPI / SDMMC

| Bus | Pins | Devices |
| --- | --- | --- |
| SPI2 | MOSI 14, SCLK 15, CS 16, DC 17, BUSY 18 | SSD1677 (MOSI-only in the pin table) |
| LoRa SPI | MOSI 38, MISO 40, CLK 39, NSS 41, BUSY 21, IRQ 5 | Stamp LoRa-1262 (full). Product table: SPI1. UserDemo: `SPI3_HOST` on those GPIOs ([user-demo.md](user-demo.md)) |
| SDMMC | CMD 12, CLK 13, DAT0–3 11/10/9/8 | microSD |

EPD and SD do **not** share a controller. Schematic names
`DAT0` and `CD/DAT3`. UserDemo mounts 4-bit SDMMC; FreeInk
says 1-bit. Live width on a physical unit:
[nyc-sdmmc-width](../resources/not-yet-confirmed.md#nyc-sdmmc-width).
SSD1677 write spec max 20 MHz; clock on a unit:
[nyc-epd-spi-clock](../resources/not-yet-confirmed.md#nyc-epd-spi-clock).
SX1262 SPI spec max 16 MHz (full SKU). That is the **die**.
The populated part is Stamp LoRa-1262 (module): 868–923 MHz,
FPC antenna, `LoRa_EN` / `SX_NRST` / `SX_ANT_SW`.
[nyc-stamp-lora](../resources/not-yet-confirmed.md#nyc-stamp-lora).
Die SPI status: [nyc-lora-ack](../resources/not-yet-confirmed.md#nyc-lora-ack).

USB-C CC1/CC2 are **5.1 kΩ Rd** to GND on the schematic: 5 V
sink, no PD controller in the extract.

## Lite leftover pads

Official HTML **PinMap** and product comparison: Lite has no
NFC and no LoRa modules (those headings exist only on the
full-SKU page). The Lite V0.6.2 schematic still **draws**
Stamp LoRa-1262 and RFID/NFC blocks (gallery page 05 / PDF).
The extract names SX1262, ST25R3916, `PYB_NFC_EN`, and LoRa
SPI nets. Lite page-05 extract also says `PYB_NFC_EN is a
spare GPIO pin`. Gallery still **draws** the RFID and Stamp
blocks. That is not DNP.
**Lite idle (2026-09-02):** GPIO5 / GPIO6 / GPIO21 as
inputs (no pull, not driven): CDC
`leftover lora_irq=0 nfc_irq=0 sx_busy=0`. Do not drive
those nets as extra GPIO until
[nyc-lite-nfc-pads](../resources/not-yet-confirmed.md#nyc-lite-nfc-pads)
/
[nyc-lite-lora-pads](../resources/not-yet-confirmed.md#nyc-lite-lora-pads)
close.
