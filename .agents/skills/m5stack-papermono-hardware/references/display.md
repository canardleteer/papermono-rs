# Display

3.97" monochrome e-paper, SSD1677, official 480×800, 4-level
gray. Module named in OTP-Demo:
**DEPG0397BBS770F3HP-XM**. Frontlight is M5PM1 G3 PWM into
schematic **AW9967DNR** (`EINK_BL`); see
[power-and-sleep.md](power-and-sleep.md).

Pins: SPI2 MOSI 14, SCLK 15, CS 16, DC 17, BUSY 18. Reset and
3.3 V are M5IOE1 `PYG5` / `PYG3`. Not shared with microSD.

## OTP vs MCU LUT

Official:

- Panel OTP waveforms can be called directly.
- Custom external waveforms must stay DC-balanced.
- M5GFX LUTs for PaperMono are currently **unstable**; prefer
  [M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo).
- After ~10 partials, one full refresh. Idle + full refresh can
  clear deposited black pixels.

FreeInk: host-authored LUTs and “3-level grayscale”. Name both
sides ([sources.md](sources.md)). Confirm on a physical unit
(name `C153` or `C153-Lite`):
[nyc-lut-path](../resources/not-yet-confirmed.md#nyc-lut-path).

Do not invent a 105-byte `0x32` table. SSD1677 Table 7-1 names
`Write LUT register` as **[105 bytes]**. Dual RAM is 0x24 BW /
0x26 RED (`6.5 RAM`). Command names come from cache id
`ssd1677`.

## Orientation

Official tables: 480×800. Touch active area is in that space.
FreeInk profile: 800×480. USB-down mapping is
[nyc-canvas-orient](../resources/not-yet-confirmed.md#nyc-canvas-orient).

## SPI clock

SSD1677 Rev 1.0: write **`fSCL` max 20 MHz**; read mode max
2.5 MHz. `BS1` L selects 4-wire (this board wires DC on
GPIO17). Vendor OTP-Demo / M5GFX Hz are **not measured**.
[nyc-epd-spi-clock](../resources/not-yet-confirmed.md#nyc-epd-spi-clock).
Do not copy Sticky’s 10 MHz measured number here as if it
were this product.

## BUSY / standby

Sheet: BUSY **high** = do not send a command; idle / done is
low. Deep sleep is opcode **0x10** with `A[1:0] = 11`; BUSY
stays high; exit needs **HWRESET**.

Idle level on a physical unit, and whether a Seeed-style
analog-off standby recovers without that reset:
[nyc-otp-busy](../resources/not-yet-confirmed.md#nyc-otp-busy).
Do not copy Sticky’s “`0xC0` did not drop BUSY” result onto this
module.

## M5GFX refresh times (lab, official)

Reference only, PaperMono, M5GFX modes:

| Mode | Time per refresh |
| --- | --- |
| `epd_quality` | 4.71 s |
| `epd_text` | 0.45 s |
| `epd_fast` | 0.34 s |
| `epd_fastest` | 0.07 s |

Unstable LUT warning still applies. Partial ghosting recipe:
[nyc-partial-ghost](../resources/not-yet-confirmed.md#nyc-partial-ghost).

## Panel datasheet

OTP-Demo names the module. A public module sheet may be
`EPD_Module_User_Manual.pdf` on M5Stack OSS (cache id
`epd-module`). Confirm the PN in that PDF before treating it as
this panel
([nyc-panel-sheet](../resources/not-yet-confirmed.md#nyc-panel-sheet)).
