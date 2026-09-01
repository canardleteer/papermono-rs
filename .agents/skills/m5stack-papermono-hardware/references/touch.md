# Touch

FT6336G at 7-bit **`0x38`** on the system I2C bus (GPIO47/48).
That address is the official pin map; it is **not** in the
public Version 1.0 chip PDF. INT is ESP32 GPIO4. Rail and
reset are M5IOE1 `PYG13` / `PYG6`.

Sheet: `/INT` = data ready; `RSTN` active-low; hold INT and
I2C low before power-on. SCL **10–400 kHz** (Table 2-2).

This is not GT911. Do not run an INT-during-reset address dance
from the Sticky skill.

## Active area (official)

Touch IC firmware shrinks the boundary:

- X: 5–475 (panel width 480)
- Y: 5–795 (panel height 800)

Confirm on a physical unit (name the SKU):
[nyc-ft6336-area](../resources/not-yet-confirmed.md#nyc-ft6336-area).

## Contacts

FocalTech FT6336G is a self-cap controller. Version 1.0
FEATURES: **1 point and gestures / 2 points**. How many
contacts **this FPC** delivers:
[nyc-ft6336-points](../resources/not-yet-confirmed.md#nyc-ft6336-points).
Do not invent a five-point GT911 map.

## Registers

The public `ft6336g` PDF has **no register map**. Remaining
encodings stay unnamed. Do not copy GT911 `0x814E` or invent
`0x02` / TD_STATUS.

## Orientation

Map touch samples onto the framebuffer after
[nyc-canvas-orient](../resources/not-yet-confirmed.md#nyc-canvas-orient)
is known. Official portrait 480×800 vs FreeInk 800×480 is a
conflict, not a silent rotate. UserDemo uses `M5.Touch` after
`setRotation(0)` and wakes ESP deep sleep with `ext0` on
GPIO4 low ([user-demo.md](user-demo.md)).
