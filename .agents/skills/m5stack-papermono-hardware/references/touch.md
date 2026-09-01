# Touch

FT6336G at 7-bit **`0x38`** on the system I2C bus (GPIO47/48).
That address is the official pin map; it is **not** in the
public Version 1.0 chip PDF. INT is ESP32 GPIO4. Rail and
reset are M5IOE1 `PYG13` / `PYG6`.

Sheet: `/INT` = data ready; `RSTN` active-low; hold INT and
I2C low before power-on. SCL **10–400 kHz** (Table 2-2).

This is not GT911. Do not run an INT-during-reset address dance
from the Sticky skill.

## Active area

Touch IC firmware shrinks the boundary (official tables):

- X: 5–475 (panel width 480)
- Y: 5–795 (panel height 800)

**Lite (`C153-Lite`, 2026-09-01):** midline slides printed
those extents (`x=5` and `x=475`; `y=5` and `y=795`). Dots
at the official-portrait corners hit within ~22 px of the
drawn center. `C153` still
[nyc-ft6336-area](../resources/not-yet-confirmed.md#nyc-ft6336-area).

## Contacts / multi-touch

FocalTech FT6336G is a self-cap controller. Version 1.0
FEATURES: **1 point and gestures / 2 points**. Official
PaperMono Arduino example `MAX_TOUCH_POINTS = 2`. Do not
invent a five-point GT911 map.

**Lite (`C153-Lite`, 2026-09-01):** one-finger walk only.
CDC `n=1` on every contact. Two-finger count on **this
FPC** is still
[nyc-ft6336-points](../resources/not-yet-confirmed.md#nyc-ft6336-points).

## Registers (M5GFX example, not a G-sheet map)

The public `ft6336g` PDF has **no register map**. Do not copy
GT911 `0x814E`. Do not treat a sibling FT6236 sheet as this
G part.

Official PaperMono Arduino / M5GFX `Touch_FT5x06::getTouchRaw`
**intent**:

- Skip the I2C read while the INT pin is high.
- Start at register **`2`**. First byte low nibble is the
  contact count (example caps at **2**).
- Point `idx` at `idx*6`: X from bytes `+1`,`+2` (low
  nibble of `+1` is the high bits). Y from `+3`,`+4`.
- Continuation length is `points*6-2` (M5GFX quirk).

**Lite:** that decode printed official-portrait XY that
matched ink (USB-C down). Cite M5GFX / the official example,
not a FocalTech map. Board crate:
`m5stack_papermono_lite::touch::decode_m5gfx`.

## `/INT` while sliding

Idle GPIO4 is **high**. A contact drives it **low** (UserDemo
`ext0` wake). During a stroke the pad **blips high** (M5GFX
polling). Do not treat a rising edge as “finger up”: that
splits a full swipe into short spans. Accumulate XY while
`n>=1`; accept a midline slide when both active-area ends
have been reached (firmware: `SLIDE_END_INSET`).

## On a physical unit (`C153-Lite`)

After M5IOE1 EN/RST, address-only `read` at `0x38` ACKs
(`tp=1`). embassy-debug target walk (2026-09-01):

- Center and four-corner dots: `x=`/`y=` vs drawn `tx=`/`ty=`
  within **2–22 px** (100 px slop).
- Horizontal midline: `y≈391–408`, `x` 5→433 (and earlier
  5→470).
- Vertical midline: `x≈248–274`, `y` 795→47 (and 5).
- Same space as official 480×800 / USB-C-down physical.
  Do not silently rotate.

## Lamp gutter vs targets

Embassy-debug right-edge strip is 80 px
(`LAMP_GUTTER_PX`): official `x >= 395`. The targets-card
first top-right dot is `(400, 80)` (also `(400, 720)`).
**Lite (2026-09-01) failure:** that tap no longer scored —
`LampSlide::feed` ate it. The walk now scores a slop hit
before the gutter. Image with that order is on the unit
(150336 bytes); not a scored-hit silicon row yet.

## Orientation

Lite USB-C-down canvas is in
[display.md](display.md). Touch samples in that walk lined
up with that map without an extra affine. Official portrait
480×800 vs FreeInk 800×480 is still a conflict on paper,
not a silent rotate. UserDemo uses `M5.Touch` after
`setRotation(0)` and wakes ESP deep sleep with `ext0` on
GPIO4 low ([user-demo.md](user-demo.md)). `C153` canvas
still
[nyc-canvas-orient](../resources/not-yet-confirmed.md#nyc-canvas-orient).
