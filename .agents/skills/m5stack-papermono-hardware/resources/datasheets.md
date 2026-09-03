# Datasheets and provenance

Vendor documents are the official source for registers, opcodes,
and timings of parts **named on this model**. Observed hardware
still outranks a datasheet default. Wiring authority is the
board contract in [SKILL.md](../SKILL.md). Precedence:
[sources.md](../references/sources.md). Lite USB, chip, 16 MB
flash, and the stock partition table are measured
([measure.md](../references/measure.md),
[flashing.md](../references/flashing.md#usb-measured)).
Official HTML `epd_*` times are PaperMono lab reference
only ([display.md](../references/display.md)). Pins and
registers below are official unless a row says otherwise.
Name PaperMono (`C153`) vs PaperMono-Lite (`C153-Lite`).

This file is the committed catalog. Links below are relative to
this skill `resources/` directory.

**Vendor the local cache** when the work is registers, opcodes,
timings, strapping, or a datasheet-versus-docs conflict. Search
extracted markdown rather than loading a whole TRM. The cache
does not replace the pin map or enclosure. Official product
pages as **view as markdown** (2026-09-01) are
[official-html/SOURCE.md](official-html/SOURCE.md), not this
cache.

**No PDFs, extracts, or schematic gallery PNGs are committed.**
They live under [datasheets/](datasheets/README.md) (`pdf/`,
`md/`, `png/`; gitignored). If those files are missing, ask
the user to populate the cache before inventing a constant.
Do not download vendor files unless they asked. Fetching a
schematic id also pulls its dated OSS gallery.

```shell
# from this skill directory
python3 scripts/fetch_datasheets.py status
# if the user asked to populate the cache:
python3 scripts/fetch_datasheets.py fetch
```

Some vendor portals refuse a scripted GET. In that case the user
saves the PDF into `datasheets/pdf/` using the filename in the
table, then runs `fetch_datasheets.py convert`.

When a cached markdown file exists, search that rather than
loading a whole TRM. The extraction is text for agents; figures
stay in the PDF (or the schematic gallery PNGs).

## How to cite a sheet

Use the catalog **Id**, then the sheet’s **section number**
(when it has one) and **section title**. Example: cache
`ssd1677`, Table 7-1 `Write LUT register`. Do not cite page
numbers; M5 copies and translations shift them.

Code still uses a named `enum` / `const`. This catalog (and
[pin-map.md](../references/pin-map.md),
[display.md](../references/display.md)) is the mapping from
those titles to encodings. HTML **PinMap** lives on the
product pages ([catalog.md](../references/catalog.md)).

## Documents

| Id | Part | Document | Revision used | Cache | Datasheet notations |
| --- | --- | --- | --- | --- | --- |
| `ssd1677` | SSD1677 EPD controller | [M5Stack copy](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/SSD1677.pdf) ([Waveshare Rev 1.0](https://files.waveshare.com/upload/2/2a/SSD1677_1.0.pdf)) | **Rev 1.0, Nov 2018** | `pdf/ssd1677.pdf`, `md/ssd1677.md` | Table 7-1 opcodes; 105-byte LUT (0x32); dual RAM; BUSY high; write 20 MHz |
| `epd-module` | E-paper module | [M5Stack EPD module user manual](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/EPD_Module_User_Manual.pdf) | Scanned; extract empty | `pdf/epd-module.pdf`, `md/epd-module.md` | Walk PDF figures. [nyc-panel-sheet](not-yet-confirmed.md#nyc-panel-sheet) |
| `m5pm1` | M5PM1 PMIC | [M5Stack M5PM1 datasheet](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1207/M5PM1_Datasheet_EN.pdf) | **V 1.9** | `pdf/m5pm1.pdf`, `md/m5pm1.md` | I2C `0x6E`; 100/400 kHz; GPIO default open-drain |
| `m5ioe1` | M5IOE1 expander | [M5Stack IO expander datasheet](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1210/IO_Expander_Datasheet_EN.pdf) | **V 1.4** | `pdf/m5ioe1.pdf`, `md/m5ioe1.md` | Chip UM `0x6F`–`0x76` from IO7; **board is `0x4F`**. Driver: REV `'W'` at `0x4F`, `'A'` on UM range |
| `papermono-schematic` | PaperMono board | [SCH V0.6.2 2026-05-22](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf) ([docs](https://docs.m5stack.com/en/core/PaperMono)) | **V0.6.2, 2026-05-22** | `pdf/papermono-schematic.pdf`, `md/papermono-schematic.md`, `png/papermono-schematic-page-0N.png` (6) | Walk PDF/PNGs. Extract drops wires |
| `papermono-lite-schematic` | PaperMono-Lite board | [PRJ V0.6.2 2026-05-22](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1268/PaperMono-Lite_PRJ_V0.6.2_20260522.pdf) ([docs](https://docs.m5stack.com/en/core/PaperMono-Lite)) | **V0.6.2, 2026-05-22** | `pdf/papermono-lite-schematic.pdf`, `md/papermono-lite-schematic.md`, `png/papermono-lite-schematic-page-0N.png` (5) | Lite. Page 05 still draws LoRa/NFC |
| `papermono-product` | PaperMono (docs PDF) | [Product PDF](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono.pdf) | Snapshot of docs | `pdf/papermono-product.pdf`, `md/papermono-product.md` | Pin tables; may lag HTML size/weight |
| `papermono-lite-product` | PaperMono-Lite (docs PDF) | [Product PDF](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/static/pdf/static/en/core/PaperMono-Lite.pdf) | Snapshot of docs | `pdf/papermono-lite-product.pdf`, `md/papermono-lite-product.md` | Lite pin tables |
| `ft6336g` | FT6336G touch | [Public copy](https://www.display-lcd.com/data/upload/admin/202503/67e3663dbe0d2.pdf) ([Crystalfontz HTML](https://www.crystalfontz.com/controllers/FocalTech/FT6336G/455/)) | **Version 1.0** (10-page) | `pdf/ft6336g.pdf`, `md/ft6336g.md` | 1–2 points; SCL 10–400 kHz; **no `0x38` in this PDF** |
| `bmi270` | BMI270 IMU | [Bosch BST-BMI270-DS000](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmi270-ds000.pdf) ([Arduino copy](https://content.arduino.cc/assets/bmi270-ds000.pdf)) | BST-BMI270-DS000 | `pdf/bmi270.pdf`, `md/bmi270.md` | Default 7-bit `0x68` if SDO to GND |
| `rx8130ce` | RX8130CE RTC | [Epson EN](https://download.epsondevice.com/td/pdf/app/RX8130CE_en.pdf) ([M5Stack register PDF](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1132/RX8130CE_cn-Register-Datasheet.pdf)) | Epson app manual | `pdf/rx8130ce.pdf`, `md/rx8130ce.md` | I2C ≤400 kHz; 7-bit from schematic/`0x32`, not a garbled extract |
| `ip2315` | IP2315 charger | [ChipSourceTek copy](https://www.chipsourcetek.com/DataSheet/IP2315.pdf) | Chinese extract | `pdf/ip2315.pdf`, `md/ip2315.md` | 8-bit `0xEA`/`0xEB` → 7-bit `0x75`; LED vs I2C detect |
| `st25r3916` | ST25R3916 NFC | [M5Stack copy](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1205/ST25R3916_EN.pdf) ([ST](https://www.st.com/resource/en/datasheet/st25r3916.pdf)) | **DS12484 Rev 8** | `pdf/st25r3916.pdf`, `md/st25r3916.md` | `C153`; I2C address `50h`; `I2C_EN` selects SPI vs I2C |
| `sx1262` | SX1261/2 LoRa **die** | [M5Stack DS_SX1261-2 V2.2](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1177/DS_SX1261_2_V2-2.pdf) | **V2.2** (Dec 2024 footer) | `pdf/sx1262.pdf`, `md/sx1262.md` | `C153`; SPI ≤16 MHz; BUSY line. Die 150–960 MHz. Do not flatten onto Stamp LoRa-1262 |
| `stamp-lora-1262` | Stamp LoRa-1262 **module** | [Product HTML](https://docs.m5stack.com/en/stamp/Stamp_LoRa-1262) | Living HTML | none (no PDF in this cache) | SKU S014 / S014-IF / S014-I. Contains SX1262. Module 868–923 MHz. [nyc-stamp-lora](not-yet-confirmed.md#nyc-stamp-lora) |
| `esp32-s3-datasheet` | ESP32-S3 | [Datasheet PDF](https://documentation.espressif.com/esp32-s3_datasheet_en.pdf) ([M5Stack copy](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/472/esp32-s3_datasheet_en.pdf)) | **Version 2.2** | `pdf/esp32-s3-datasheet.pdf`, `md/esp32-s3-datasheet.md` | Straps GPIO0/3/45/46; JTAG 39–42; USB 19/20 |
| `esp32-s3-trm` | ESP32-S3 | [TRM PDF](https://documentation.espressif.com/esp32-s3_technical_reference_manual_en.pdf) | — | `pdf/esp32-s3-trm.pdf`, `md/esp32-s3-trm.md` | GPIO hold, USB-Serial/JTAG, `ext1` |

Cache paths are relative to [datasheets/](datasheets/README.md).
Download URL order lives in `scripts/fetch_datasheets.py`
(vendor / M5Stack first, then public copies). Schematic
gallery URLs and HTML **PinMap** pages:
[catalog.md](../references/catalog.md). Those HTML pages are
the living index; OSS filenames are a dated snapshot.

## Captured SHA-256

PDFs, extracted markdown, and schematic gallery PNGs are
gitignored. Their SHA-256 digests are committed so a later
IPFS CIDv1 (raw codec + sha2-256) can be derived without
re-hosting the files in git.

- [datasheets.sha256](datasheets.sha256) — `sha256sum` format
- [datasheets.sha256.json](datasheets.sha256.json) — same records
  plus byte lengths

`fetch_datasheets.py hash` rewrites both after a convert
(or after a schematic gallery fetch). `status` checks the
local files against this list.

| Id | PDF SHA-256 | bytes |
| --- | --- | ---: |
| `ssd1677` | `daf8b726f822f6907529386b8af6fbea6ec5e8ecc7b7af361340be911d632544` | 3602487 |
| `epd-module` | `6b65f56d5d549a132e5c807eef329fb3e997feab04ac06e8a107f01591618130` | 8666208 |
| `m5pm1` | `c6daffd0ab89d8de50c0a19f35f94321bc6c9511ad017dd242a3e765d88a96b6` | 814731 |
| `m5ioe1` | `9204d99cb2e03395ffc905e05fbeddf7f9f25de5698a31a65fa439dee700d8d2` | 398463 |
| `papermono-schematic` | `b5abecac97cb043a6d1af7a673f2111d40d9254b0b5108635cb924d9b37ad6fa` | 1462039 |
| `papermono-lite-schematic` | `e27b138e9104d3550531c9a9757ddc3d14941a546acc698ba7df4f40cd37095e` | 1409590 |
| `papermono-product` | `1f40d77fe525c8619f17ce8a9bb590b381e133160496e4d4702fcc4b8f717865` | 15293568 |
| `papermono-lite-product` | `e541d8977eb5613fb4eb1140f870e509c7b11fcacae5ffadcace041e59a3ac3a` | 14578435 |
| `ft6336g` | `f7091da373e3c23d2ae734b73aa82ff5d40cbac4da4592bb83723c9edbf902b5` | 473748 |
| `bmi270` | `f68ce3c74f011a80bbe4b0279c4abe9609a71a5dcda75a951381d77112e005fb` | 2636042 |
| `rx8130ce` | `db027a5fa2dd17b333da81e95063764eb5536d2878c63b64949348d2ec4b8a22` | 2008845 |
| `ip2315` | `e6fd50db74f33c01aa6e743feabb209a13249de1a1d56f953633f74c988350fd` | 4152348 |
| `st25r3916` | `1170f4d74ff501917262ae42bf81c60c38c11d121e2ea2af6b1e845433befef1` | 2241765 |
| `sx1262` | `6d783125dbef567954ce53095ad068d0950b2a8de52f95bb2cfde4146c4306c2` | 4459535 |
| `esp32-s3-datasheet` | `2d5a7cb7fd559d8d972bd88db32669c0196d23f22d7afaafb0f63d099b589a3f` | 1098115 |
| `esp32-s3-trm` | `4484bf8a69035ec42a731c58c64ada6fbd1f1618c5559409f134d9ea083f444f` | 15215232 |

Gallery PNG hashes (same SHA files; `png/` gitignored):

| Cache file | SHA-256 | bytes |
| --- | --- | ---: |
| `png/papermono-schematic-page-01.png` | `d60bc2217ff36b152ed179c527b5aabd3a8484cc2a74b68069d39dcb68005037` | 2002888 |
| `png/papermono-schematic-page-02.png` | `52af03768fafce8555187def932925b674513bc266a2c5be839f1a82d787f240` | 1353606 |
| `png/papermono-schematic-page-03.png` | `f547ea1ebc4507cebe53cfcfc940f6d37138931dab077b2f20d658aa8d666231` | 985447 |
| `png/papermono-schematic-page-04.png` | `88b42aecfd86fecc6580f11ea29fb876322ff1aa451c6e4d5b62c53dc8e14e73` | 1192874 |
| `png/papermono-schematic-page-05.png` | `920bd393c29979622128e6900db23dd6ea1c6bfa8d86f34a2badc3e44411349d` | 438327 |
| `png/papermono-schematic-page-06.png` | `41848ade172b00ccafceb123b1b9b7a585fb65fcfd87cd7ac0d4a215cc9c0714` | 1154286 |
| `png/papermono-lite-schematic-page-01.png` | `abcbf879ce9efaefdea43aef0fe2c7a605a3d27bba4b7593efa5ec9ff1fb88a9` | 256189 |
| `png/papermono-lite-schematic-page-02.png` | `fcf5a820e5f2fb3b691c13d1f9e7d9f6a41c741aa5e8ba4f0d07b7d0e975f40b` | 2433266 |
| `png/papermono-lite-schematic-page-03.png` | `431347b4c4addf9ac195282f0817c9b03094773983f5d470d7a512957cb3265f` | 1697035 |
| `png/papermono-lite-schematic-page-04.png` | `ae396bd191420610ba9943a9018c0013b16bc29de1b3d63bb7bfd562dd0c6174` | 1152424 |
| `png/papermono-lite-schematic-page-05.png` | `0cdf2ccee1a8167dc74f050961f3ca34f00229b03790eea52a2dc65fe52982c1` | 1613112 |

## Verified against the SSD1677 datasheet

Facts below were read out of **Rev 1.0, Nov 2018** (M5Stack
copy; same family as the Waveshare sheet). A plausible guess
here produces a corrupt frame or panel stress:

- **Opcodes** come from Table 7-1. Use the datasheet name for
  each opcode.
- **Write max is 20 MHz** (features; `fSCL` Write Mode). Read
  mode max is 2.5 MHz. Clock on a physical unit is still
  [nyc-epd-spi-clock](not-yet-confirmed.md#nyc-epd-spi-clock).
- **`Write LUT register` (0x32) takes 105 bytes.** Section
  `6.6 Programmable Waveform for Gate, Source and VCOM`
  describes on-chip waveform storage; the MCU-facing Table 7-1
  command is 105 bytes. Do not invent those bytes.
- **Two RAM planes exist** (`Write RAM (Black White)` 0x24 and
  `Write RAM (RED)` 0x26). The bit pair selects a LUT index
  (`6.5 RAM`). That is the mechanism four-gray is built on.
- **BUSY high means do not operate.** Pin table: when Busy is
  high, do not interrupt the chip or send a command. Many Table
  7-1 rows say the pad stays high during the op. Idle / done is
  low. Net polarity on a physical unit is still
  [nyc-otp-busy](not-yet-confirmed.md#nyc-otp-busy).
- **4-wire vs 3-wire is `BS1`.** `BS1` L = 4-wire SPI; H =
  3-wire (`6.1.1` / pin table). This board wires DC (GPIO17),
  which is the 4-wire class.
- **Deep sleep is 0x10 with `A[1:0] = 11`.** BUSY stays high.
  Exit needs **HWRESET**. Do not copy another product’s
  analog-off standby as if it were this opcode.

## Verified against the FT6336G datasheet

Facts below were read out of the **Version 1.0** public 10-page
sheet. It has no 7-bit slave address and no coordinate register
map.

- **1 point + gesture, or 2 points** (INTRODUCTION / FEATURES).
  Contacts **this FPC** delivers stay
  [nyc-ft6336-points](not-yet-confirmed.md#nyc-ft6336-points).
- **SCL 10–400 kHz** (Table 2-2 I2C Timing Characteristics).
- **`/INT` means data ready.** `RSTN` is active-low (pin table;
  §2 architectural notes).
- **Hold INT and I2C low before power-on** (power-on / reset
  note in the extract). Board reset is M5IOE1 `PYG6`.
- **`0x38` is not in this PDF.** That 7-bit address is the
  official pin-map / docs value. Do not invent `0x02` /
  TD_STATUS encodings from a GT911 map.

## Verified against the ESP32-S3 datasheet

Facts below were read out of **Version 2.2**. GPIO hold and
pad-JTAG eFuse details stay in the TRM.

- **Strapping pins** are GPIO0, GPIO3, GPIO45, GPIO46 (§3).
  Table 3-1 defaults: GPIO0 WPU, GPIO3 floating, GPIO45/46 WPD.
  Latched at chip reset; ordinary IO after hold time `tH` ≥
  3 ms (Table 3-2). This board: GPIO0 = `BOOT_OUT`, GPIO3 =
  BUTTON B (DOWN) / `USER_KEY2`, GPIO45/46 = PDM (not a
  power latch).
- **GPIO19/20** default to USB Serial/JTAG (§2.3.4 /
  `4.2.1.8`).
- **GPIO39–42** default IO MUX F0 is JTAG `MTCK` / `MTDO` /
  `MTDI` / `MTMS` (Table 2-4; §2.3.4 pad JTAG). Mux to GPIO
  before LoRa SPI (39–41) or the buzzer (42).
- **GPIO21** (SX1262 BUSY on C153) has empty At Reset /
  After Reset pull columns (Table 2-1): no internal WPU/WPD.
- **Two I2C controllers** (`4.2.1.2`): Standard 100 kbit/s,
  Fast 400 kbit/s, up to 800 kbit/s limited by pull-up
  strength. Touch ≤400 kHz is in spec.

## Verified against the M5PM1 user manual

Facts below were read out of **V 1.9**:

- **I2C address `0x6E`.** 100 kHz default / 400 kHz.
- **GPIO outputs default to open-drain**, including PWM. Need
  an external pull-up, or configure push-pull, or the pin does
  not drive high. Frontlight PWM on G3 is in this class.
- Device_ID register `0x00` defaults to `0x50`. That is a
  *register* value, not the ST25R3916 slave `0x50`.

G3 mux vs PWM engine is **not** a V 1.9 re-read. GPIO3
`GPIO_FUNC0` bits `[7:6] = 11` (`0xC0`) is the PWM
alternate. FreeInk Paper Mono names that engine **PWM0**
(`PWM0_L` / `PWM0_HC`, frequency at `PWM_FREQ_L`, 5 kHz,
high-byte enable). PWM1 is the next duty pair. This
repo’s first lamp path wrote PWM1; Lite brightness stayed
constant.

## Verified against the M5IOE1 user manual

Facts below were read out of **V 1.4**:

- Chip UM I2C range is **`0x6F`–`0x76`**, sampled from **IO7
  (PA6)** at power-on (Table 6 Voltage vs. I²C Address
  Mapping). Do not leave IO7 floating (random address).
- **This board’s schematic and official pin map label
  `IIC Adress:0x4F`.** That is outside the UM strap table.
  Name both sides; do not silently treat `0x4F` as the chip
  default or `0x6F` as the board address.
- GPIO outputs default **open-drain** (same pull-up / push-pull
  rule as M5PM1).
- 100 kHz default / 400 kHz.
- `I2C_CFG` (`0x23`): SPD 100/400 kHz; SLEEP `0` = no idle
  sleep. Official driver still sends a START+STOP wake before
  UID/REV ([user-demo.md](../references/user-demo.md)).

## Verified against the IP2315 datasheet

Facts below were read out of the Chinese ChipSourceTek copy:

- I2C **high level is VBAT**. Pins 8/9 mux LED vs I2C. At VIN
  power-on, **both must sample high** or the chip stays in LED
  mode (matches the official “low VBAT hangs the bus” note).
- Max **400 kbps**. 8-bit slave write **`0xEA`** / read
  **`0xEB`** → 7-bit **`0x75`**.
- Register map rows stay unnamed unless read from this extract.
  Prefer M5PM1 telemetry; isolate via `PYG11` after a charge
  transaction.

## Verified against the other named-part datasheets

- **BMI270:** default 7-bit `0x68` if SDO is to GND; `0x69` if
  SDO to VDDIO. I2C 100/400/1000 kHz. Schematic labels
  `IIC Adress:0x68`.
- **RX8130CE:** I2C up to 400 kHz. The extract’s slave-address
  bits are garbled. Use schematic `IIC Adress:0x32` and the
  official pin map. Do not invent a 7-bit from a shifted
  warning.
- **ST25R3916 (DS12484 Rev 8):** `I2C_EN` to VDD_D = I2C, to
  GND = SPI (`4.3.2`). I2C address **`50h`**. Fast-mode up to
  400 kbit/s (also names Fast-mode Plus / HS). `C153` schematic
  note: `I2C_EN=VDD; I2C mode`. `C153` only.
- **SX1262 die:** SPI up to 16 MHz; Table 8-1 `t2` SCK period
  62.5 ns. Honor the BUSY line (timing in §8.2 / 8.3.1). `C153`
  only. Do not invent opcodes from an unread heading. Die
  coverage 150–960 MHz is **not** the PaperMono product band.
- **Stamp LoRa-1262:** module around that die. Catalog id
  `stamp-lora-1262` is the product HTML (no PDF in this cache).
  PaperMono PinMap: `LoRa_EN` / `SX_NRST` / `SX_ANT_SW`.
  [nyc-stamp-lora](not-yet-confirmed.md#nyc-stamp-lora).

## Verified against the schematic

Facts below were read out of **V0.6.2 (2026-05-22)** full and
Lite PDFs (and the dated OSS gallery PNGs linked from the
product HTML). Walk the PDF or PNGs for a net; the extract
drops wires. Re-check the HTML docs pages if a newer dated
file appears.

- **Frontlight is AW9967DNR** on `EINK_BL`. Official HTML
  “M5PM1 G3 PWM” and FreeInk “AW9967” are both
  official-electrical: PWM drives the AW9967.
- **I2C address labels** on the full sheet: M5PM1 `0x6E`,
  M5IOE1 `0x4F`, RTC `0x32`, BMI270 `0x68`. System I2C has
  `IIC PULL_UP`.
- **USB-C CC** uses **5.1 kΩ** (Rd): 5 V sink, no PD
  controller in the extract.
- **GPIO45/46** are labeled strap and PDM. GPIO0 is
  `BOOT_OUT` / `G0_BOOT_OUT`.
- **IP2315** is present; `PYG11_PWM3` sits on the charger I2C
  gate (`PYB_CHG_IIC`).
- **NFC (full):** ST25R3916 sheet note `I2C_EN=VDD; I2C mode`.
  `PYB_NFC_EN` is called out as a spare expander GPIO (the
  enable net), not as “NFC absent”.
- **microSD:** extract names `DAT0` and `CD/DAT3` (4-bit
  wiring class). Firmware width on a physical unit is still
  [nyc-sdmmc-width](not-yet-confirmed.md#nyc-sdmmc-width).
- **Lite extract** still names SX1262 / ST25R3916 / `PYB_NFC_EN`
  / `G5_LoRa_INT`. Gallery **page 05** still **draws** Stamp
  LoRa-1262 and RFID. Lite HTML **PinMap** and SKU compare omit
  those modules. That does **not** prove DNP vs still-routed.
  [nyc-lite-nfc-pads](not-yet-confirmed.md#nyc-lite-nfc-pads)
  and
  [nyc-lite-lora-pads](not-yet-confirmed.md#nyc-lite-lora-pads)
  stay open.
- **No panel PN** in the extract (`DEPG0397BBS770F3HP-XM` is
  OTP-Demo README). `epd-module` extract is empty (scanned).
- No BOM in these PDFs.

## Gaps, deliberately not filled by guessing

| Gap | What to do |
| --- | --- |
| Four-gray LUT contents for this panel | No default LUT. Official path is OTP. MCU table stays optional and attributed; record source and license here before adding one |
| FT6336G coordinate / status encodings | Version 1.0 public sheet has no register map. Do not invent `0x02` / GT911 names |
| FT6336G 7-bit address in the chip PDF | Not present. Official pin map `0x38`. Lite ACK 2026-09-02. `C153` still [nyc-i2c-ack](not-yet-confirmed.md#nyc-i2c-ack) |
| IP2315 register map beyond 8-bit `0xEA`/`0xEB` | Raw read/write from the Chinese extract; do not invent |
| RX8130CE 7-bit from the Epson extract | Garbled. Schematic / docs `0x32` |
| M5IOE1 why board `0x4F` vs UM `0x6F`–`0x76` | Official M5IOE1 driver: `0x4F` REV `'W'` (board firmware); UM range REV `'A'`. UserDemo begins at `0x4F` then library fallback `0x6F`. Lite: bare `read` at `0x4F` NAK; official `begin` ACK at board `0x4F` (`ioe_addr=4f`). UM `0x6F` not required on this unit |
| Panel PN vs `epd-module` PDF | Extract empty. Walk PDF figures; [nyc-panel-sheet](not-yet-confirmed.md#nyc-panel-sheet) |
| Lite NFC/LoRa population | HTML PinMap: absent. Lite schematic page 05 still draws the modules. [nyc-lite-nfc-pads](not-yet-confirmed.md#nyc-lite-nfc-pads) / [nyc-lite-lora-pads](not-yet-confirmed.md#nyc-lite-lora-pads) |
| ACK list on a physical unit | Lite written ([measure.md](../references/measure.md)). `C153` still [nyc-i2c-ack](not-yet-confirmed.md#nyc-i2c-ack) |
| ESP32-S3 GPIO hold, pad-JTAG eFuse, `ext1` | Datasheet v2.2 names the pads. Search `esp32-s3-trm` when citing |

## Waveform provenance

| LUT | Source | License | Status |
| --- | --- | --- | --- |
| *(none shipped)* | Official: OTP; M5GFX currently unstable | — | Do not add a row for bytes extracted from vendor firmware |

Do not add a row for bytes extracted from vendor firmware.
FreeInk host LUTs stay third-party until
[nyc-lut-path](not-yet-confirmed.md#nyc-lut-path) and a license
row exist.
