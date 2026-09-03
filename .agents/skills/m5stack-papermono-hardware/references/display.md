# Display

3.97" monochrome e-paper, SSD1677, official 480×800, 4-level
gray. Module named in OTP-Demo:
**DEPG0397BBS770F3HP-XM**. Frontlight is M5PM1 G3 PWM into
schematic **AW9967DNR** (`EINK_BL`); see
[power-and-sleep.md](power-and-sleep.md). Lite: right-edge
slide **drives** G3 / PWM0 brightness. PWM1 writes the
same day did not. One AW9967; no color-temp pair.

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
`ssd1677`. Prohibitions:
[safety.md](safety.md).

## What to do (this repo)

Call the panel OTP. Encode that as board-crate
`OtpRefresh` (`otp_gray` / `otp_mono` / `otp_partial`).
Firmware call sites use that enum. Official HTML `epd_*`
titles stay on `RefreshMode` as a catalog only.

| Do | Sequence |
| --- | --- |
| 4-gray card | `OtpRefresh::GrayFull` (`0xD7`) |
| Rebuild mono after gray, or a full clear | `OtpRefresh::MonoFull` (`0xF8` then `0x14`, both planes) |
| Fast flip after a mono baseline | `OtpRefresh::Partial` (`0xFF`, RAM 1 only) |
| After N partials | one `MonoFull` (embassy-debug `PARTIALS_BEFORE_FULL` = 6) |

Do **not** upload `0x32`. Do **not** send `Partial` after
`GrayFull` until `MonoFull`. Do **not** send a second bare
Mode 1. Do **not** stamp `epd_*` from an OTP path.
OTP-Demo deep-sleeps after each refresh and wakes with a
hardware reset (M5IOE1 `EPD_RST`). Software reset only on
full updates so partials keep RAM. Skip-`MonoFull` trials
are **abandoned**.

## Refresh trials (Lite, 2026-09-01)

Recorded as successes and failures. `C153` unmeasured.
Silicon table: [measure.md](measure.md).

### Successes

- OTP-Demo state machine in embassy-debug: `GrayFull` /
  `MonoFull` / `Partial`, deep sleep, HW reset, no `0x32`.
- `GrayFull` changes glass (4-gray quadrants; Ferris
  mid-tones when splash used `GrayFull`). Splash is now
  1-bit line art (`paint_mono_fast`).
- `Partial` after a **mono** baseline advances marks
  (`busy_rose=1`).
- `MonoFull` both planes white: glass went **white**.
- White-vs-white `Partial` does **not** clear leftover
  ink (copy the new frame to RAM 2 after each `Partial`).
- OTP RAM orientation USB-C down: RAM X = physical Y,
  RAM Y = physical X.
- Fast A/B on shapes / legend when those cards use
  `Partial` after a real mono baseline.
- Leaving gray **must** `MonoFull` before `Partial` —
  now **measured**, not only OTP-Demo policy.

### Failures

- Panel OTP `epd_text` + datasheet Mode 1: no BUSY rise, no
  glass change.
- Bare second Mode 1: `busy=1`, Ferris stuck (with and
  without GPIO42 chirp). Chirp stays parked.
- `Partial` immediately after `GrayFull` (no `MonoFull`):
  flip was **fast**; Ferris stayed **until overdrawn**,
  not a faint ghost. Official HTML does not name that
  pair as a destroy-the-panel step. OTP-Demo refuses it
  (`baseline_ready = false`). **Abandoned.**
- Staying on `GrayFull` for B/W cards does **not** make
  refreshes fast (waveform duration, not “using two of
  four tones”).
- Right-edge lamp slide: brightness stayed **constant**.
  Firmware wrote PWM1; G3 mux is PWM0. Targets-card
  first top-right dot `(400, 80)` sat in the 80 px
  gutter, so that tap was eaten.

## Orientation

Official tables: 480×800. UserDemo `Hal::init` calls
`display.setRotation(0)`. Touch active area is in that space.
FreeInk profile: 800×480. OTP-Demo addresses 800×480 RAM
(X vertical, Y horizontal in that tree).

**Lite (`C153-Lite`, 2026-09-01), USB-C down:** OTP RAM X is
physical Y (0 at the top, 799 toward USB-C). OTP RAM Y is
physical X (0 left, 479 right). Corner bars: 1 bottom-left,
2 top-left, 3 bottom-right, 4 top-right. Draw official
portrait with `otp_ram_to_usb_down`. FT XY from M5GFX
`getTouchRaw` matched that portrait (dots + midlines;
[touch.md](touch.md)). `C153` still
[nyc-canvas-orient](../resources/not-yet-confirmed.md#nyc-canvas-orient).

## OTP-Demo sequences (Lite)

Cite
[M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo),
not an invented LUT. Mono RAM: **1 = white, 0 = black**.
`init_mono_mode` uses data entry **`0x03`** (X and Y
increment), window from RAM (0,0). Four-gray still uses
X-decrement `0x02` and `0xD7`.

**Lite (`C153-Lite`, 2026-09-01)** embassy-debug walk
(conclusions; silicon table [measure.md](measure.md)):

- `Partial` after a mono baseline advances marks.
- White-vs-white `Partial` does not clear leftover ink;
  copy the new frame to RAM 2 after each `Partial`.
- `Partial` after `GrayFull` without `MonoFull` is
  **abandoned** (Ferris stayed until overdrawn).
- `MonoFull` both planes white: glass went **white**.
- After about six partials, one `MonoFull`. Official HTML
  still says about ten
  ([nyc-partial-ghost](../resources/not-yet-confirmed.md#nyc-partial-ghost)).

## SPI clock

SSD1677 Rev 1.0: write **`fSCL` max 20 MHz**; read mode max
2.5 MHz. `BS1` L selects 4-wire (this board wires DC on
GPIO17). Vendor OTP-Demo / M5GFX Hz are **not measured**.
[nyc-epd-spi-clock](../resources/not-yet-confirmed.md#nyc-epd-spi-clock).
Do not assume a 10 MHz limit without measurement on this product.

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
is eval intent. Do not assume “`0xC0` did not drop
BUSY” without direct observation on this module.

## Two vendor refresh stacks

Do not treat the four M5GFX titles as OTP-Demo's three
`0x22` bytes. They are different drivers.

**OTP-Demo**
([M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo)
`EDP_OTP_LUT_demo`): built-in OTP only. No `0x32` LUT.
Deep sleep (`0x10` / `0x01`) after every refresh; hardware
reset to leave it; software reset only on full updates so
partials keep RAM. Sequences:

| Intent | `0x22` | Notes |
| --- | --- | --- |
| Partial | `0xFF` | Needs a **mono** RAM baseline. Write next frame to RAM 1 only. Gray `0xD7` **invalidates** that baseline |
| Mono full | `0xF8` then `0x14` | Invert-sync first, then Mode 1 with **both** planes. Rebuilds the partial baseline |
| 4-gray full | `0xD7` | Both planes. After this, rebuild mono with `refresh_mono_full` before any `0xFF` |

Lite silicon for those three rows:
[measure.md](measure.md). First boot loads white into both
planes **without** Master Activation, then a partial (no
full-screen flash).

**M5GFX LUT modes** — official HTML heading **M5GFX LUT
Refresh Speed** on both product pages
([PaperMono](https://docs.m5stack.com/en/core/PaperMono),
[PaperMono-Lite](https://docs.m5stack.com/en/core/PaperMono-Lite)).
Dated **view as markdown** (2026-09-01):
[official-html/SOURCE.md](../resources/official-html/SOURCE.md).
Four titles. Both snapshots name laboratory results for
**PaperMono** under M5GFX modes; times may vary with
content and environment; **reference only**. The two pages
do not share one English sentence; numbers match. The Lite
page reprints that PaperMono lab table. That is not a Lite
measurement, and this repo has not timed those four modes
on a desk unit. Driver:
[M5GFX](https://github.com/m5stack/M5GFX)
`Panel_SSD1677_4Gray` (UserDemo / M5Unified). Each
activation uploads an MCU LUT (`0x32`). The same pages:
those LUTs are currently **unstable**; prefer the OTP
example for panel life.

| Enum title | M5GFX path | Single refresh |
| --- | --- | ---: |
| `epd_quality` | Mode 1 absolute + `lut_quality`; analog down after | 4.71 s |
| `epd_text` | Mode 1 absolute + `lut_text`; analog down after | 0.45 s |
| `epd_fast` | Mode 2 absolute + `lut_fast` | 0.34 s |
| `epd_fastest` | Mode 2 vs last frame + `lut_fastest`; no history → one `epd_fast` first | 0.07 s |

Cite the HTML table as official reference, not observed
silicon ([sources.md](sources.md)). Encode the titles when
naming this catalog. Do not pass a raw M5GFX integer, an
ad-hoc delay, or an OTP `0x22` as if it were one of these
four modes.

UserDemo (`hal_display.cpp`) wraps that driver: ten
`epd_fastest` then one `epd_fast`; five fast then one
`epd_quality`. After a non-quality refresh it may analog-off
(`0x22` / `0x03` / `0x20`); quality/text then need
`powerSaveOff` so the 4-gray RAM face is rebuilt. Analog-off
invalidates Fast/Fastest history in M5GFX.

Partial ghosting recipe:
[nyc-partial-ghost](../resources/not-yet-confirmed.md#nyc-partial-ghost).

## Panel datasheet

OTP-Demo names the module. A public module sheet may be
`EPD_Module_User_Manual.pdf` on M5Stack OSS (cache id
`epd-module`). Confirm the PN in that PDF before treating it as
this panel
([nyc-panel-sheet](../resources/not-yet-confirmed.md#nyc-panel-sheet)).
