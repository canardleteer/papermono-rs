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

Official tables: 480×800. UserDemo `Hal::init` calls
`display.setRotation(0)`. Touch active area is in that space.
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
UserDemo `hal_display.cpp` names analog-off as
`writeCommand(0x22)` / `0x03` then Master Activation `0x20`,
and analog-on as `0xC0` ([user-demo.md](user-demo.md)). That
is eval intent. Do not copy Sticky’s “`0xC0` did not drop
BUSY” result onto this module.

## Refresh modes (lab, both SKUs)

M5GFX mode labels. Use these as firmware `enum` titles
(`epd_quality`, `epd_text`, `epd_fast`, `epd_fastest` — in
Rust, variants such as `EpdQuality` documented under those
titles). Do not pass a raw M5GFX integer or an ad-hoc delay
at the call site. Times are **lab wall-clock for a single
refresh**, measured on PaperMono (`C153`) **and**
PaperMono-Lite (`C153-Lite`). Official HTML product pages
list the same numbers; name both layers
([sources.md](sources.md), [measure.md](measure.md)).

| Enum title | Single refresh |
| --- | ---: |
| `epd_quality` | 4.71 s |
| `epd_text` | 0.45 s |
| `epd_fast` | 0.34 s |
| `epd_fastest` | 0.07 s |

Unstable LUT warning still applies. UserDemo still drives the
panel through M5GFX `epd_quality` / `epd_text` / `epd_fast` /
`epd_fastest` (fastest: ten local then one fast; five fast
then one quality). Partial ghosting recipe:
[nyc-partial-ghost](../resources/not-yet-confirmed.md#nyc-partial-ghost).

## Panel datasheet

OTP-Demo names the module. A public module sheet may be
`EPD_Module_User_Manual.pdf` on M5Stack OSS (cache id
`epd-module`). Confirm the PN in that PDF before treating it as
this panel
([nyc-panel-sheet](../resources/not-yet-confirmed.md#nyc-panel-sheet)).
