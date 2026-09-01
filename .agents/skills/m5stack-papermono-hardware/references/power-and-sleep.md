# Power and sleep

M5PM1 / M5IOE1 rails. Pin numbers:
[pin-map.md](pin-map.md). Hazards:
[safety.md](safety.md). Official Arduino page:
[M5PM1 & M5IOE1](https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1).

Sequences below are **vendor intent**. Currents, button timings,
and wake-from-L1 are **not measured**. Name the SKU
(`C153` vs `C153-Lite`). See
[nyc-sleep-current](../resources/not-yet-confirmed.md#nyc-sleep-current),
[nyc-power-button](../resources/not-yet-confirmed.md#nyc-power-button),
[nyc-pm1-wake](../resources/not-yet-confirmed.md#nyc-pm1-wake).

This is not Sticky’s GPIO45/46 latch.

## Power button (M5PM1)

Official product notes:

- Power on / reset: short press once
- Power off: two presses in succession
- Download: hold ~2 s until the red LED blinks, then release

## Rail levels (not a series stack)

L1–L3B inputs all come from L0 (`SYS_VBUS`), independently.
After M5PM1 powers on, L1, L2, and L3A enable automatically.
M5Unified then uses M5IOE1 to enable L3B.

| Level | What is newly on | Switch |
| --- | --- | --- |
| L0 | M5PM1 + RTC from battery; button on/off; RTC wake | Always, until battery is dead |
| L1 | IMU (`3V3_L1_EN` / `setLdoEnable`) | M5PM1 LDO |
| L2 / L3A | ESP32-S3, LoRa, NFC, M5IOE1, key pull-ups, touch INT, buzzer, LED red (`3V3_L2_EN` / `setDcdcEnable`) | M5PM1 DCDC. ESP sleeping = L2; running = L3A |
| L3B | Display, touch, microSD detect, PDM, RGB G/B | M5IOE1, per peripheral |

`pm1.shutdown()` returns to L0 by default (only M5PM1 powered).
ESP32 can ask the PMIC to drop L2 → L1/L0.

L1 retain + IMU/RTC wake: `setLdoEnable(true)`,
`ldoSetPowerHold(true)`, then `shutdown()`. After wake, M5PM1
repeats L0/L1/L2 and the ESP32 runs `setup()` again. That is
documented Arduino behavior, not a measured current.

## Charger (IP2315)

I2C 7-bit `0x75` (sheet 8-bit write `0xEA` / read `0xEB`),
gated by M5IOE1 `PYG11_PWM3`. I2C high is **VBAT**. Pins 8/9
mux LED vs I2C; at VIN power-on both must sample high or the
chip stays in LED mode. Official: do not keep it on the bus.
Low VBAT + USB can fail that detect and hang neighbors.
Max 400 kbps.
[nyc-ip2315-bus](../resources/not-yet-confirmed.md#nyc-ip2315-bus),
[nyc-charge-stat](../resources/not-yet-confirmed.md#nyc-charge-stat).

M5PM1 also exposes battery/charging telemetry in its driver.
Prefer that for UI; use IP2315 only when a register on that
chip is required, then isolate.

## Frontlight

Official pin map: M5PM1 G3 PWM (`BL_FB`). Schematic V0.6.2:
**AW9967DNR** on `EINK_BL`. Those are the same path (PWM
drives the boost), not a pick-one conflict. FreeInk’s AW9967
name is schematic-true. M5PM1 PWM defaults to open-drain.

## Do not

- Copy Sticky `PWR_HOLD` / `PWR_LOCK`.
- Leave L3B EPD rail on across a shutdown without an OTP
  deep-sleep command if the OTP example says otherwise (read
  OTP-Demo; do not guess analog rails).
- Keep IP2315 mounted “so the gauge is always there”.
