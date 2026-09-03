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

## Power button (M5PM1)

Official product notes:

- Power on / reset: short press once
- Power off: two presses in succession
- Download: hold ~2 s until the red LED blinks, then release

**Lite (`C153-Lite`, 2026-09-02, operator stopwatch):**
short press ~0.25 s **resets** (red off during reboot,
solid red after). Hold to first blink **~2 s** (matches
the official note). The blinking die is the small red
next to the power button (`LED_EN_PP`), not the RGB
window. Double-press gap is too short to time; with USB
unplugged the lamp and red go fully off; one short press
turns the unit back on. Not a current measurement.
`C153` still
[nyc-power-button](../resources/not-yet-confirmed.md#nyc-power-button)
/
[nyc-download-mode](../resources/not-yet-confirmed.md#nyc-download-mode).

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

UserDemo eval paths (`app_sleep_wake.cpp`, `app_shutdown.cpp`;
[user-demo.md](user-demo.md)):

- IMU: BMI270 any-motion INT1 → PM1 G4 falling, then
  `pm1.shutdown()` with LDO hold. Probe `0x68`, fallback
  `0x69`.
- RTC: `rtc.setTimerIRQ(10000)` → PM1 G0 falling, then
  `pm1.shutdown()`.
- Touch: ESP `esp_deep_sleep_start` with `ext0` on GPIO4 low;
  touch EN/RST stay high; PM1/M5IOE1 I2C idle sleep 1 s.
- Shutdown: `ldoSetPowerHold(false)` then `pm1.shutdown()`.

Sheet `BATT_LVP` power-on recovery includes **5VIN
inserted**. A USB-in `SYS_CMD` off can bounce straight
back to L2.

**Lite RTC trial (stopped 2026-09-02).** Code is
`embassy-debug` `--features sleep` (default **off**). Do
not resume unless asked. Human watched the front lamp
only; no current meter.

- USB-in `SYS_CMD`: lamp never stayed dark. CDC went
  silent like `monitor --reset`. Sheet recovery: 5VIN
  present.
- Unplug after lamp-on: lamp **does** go off. It came
  back in **2–3 s**. Not a 10 s RTC G0 wake.
- CDC same boot both times: `reset=chip_power_on`,
  `sleep abort`, `wake src=0a` then `08` (`VIN` /
  `RSTBTN`, never `EXT_WAKE` `0x20`). ESP never lost
  power. `rtc_flag=31` at bring-up (leftover TF/AF).
- 3 s matches the G0-idle timeout: shutdown was never
  sent. An earlier 2 s match was `SYS_CMD` then
  firmware restored the lamp (`ALIVE_AFTER_OFF`).
- PWM0 `0` turns the lamp off; `PYG3` must stay high
  for it to light. Rear charge LED stays red on USB.
- Official timer for 10 s is RX8130 TSEL 64 Hz, count
  640, TE last; `FLAG` W0C `0xAF` / `0xA7`. Not proven
  here. IMU G4 and touch `ext0` not tried.

[nyc-pm1-wake](../resources/not-yet-confirmed.md#nyc-pm1-wake)
stays open. `C153` open.

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

**Lite (`C153-Lite`, 2026-09-02):** one gated boot
transaction, USB in. M5PM1 `VBAT` 4198 mV, `VIN` 5012 mV,
`PWR_SRC` `0x05` (battery + 5VIN), `CHG_EN` on. IP2315
address ACK was **0** while `PYG11` was high and **0**
after park. Rear window LED stayed red (operator: that
lamp is red whenever USB is charging; no color change).
No IP2315 current or done bit. Matches the sheet’s LED
vs I2C mux more than a hung bus (VBAT was not low).
`C153` still
[nyc-charge-stat](../resources/not-yet-confirmed.md#nyc-charge-stat).

## Frontlight

Official pin map: M5PM1 G3 PWM (`BL_FB`). Schematic V0.6.2:
**AW9967DNR** on `EINK_BL`. Those are the same path (PWM
drives the boost), not a pick-one conflict. FreeInk’s AW9967
name is schematic-true. M5PM1 PWM defaults to open-drain.
UserDemo sets the lamp with `display.setBrightness` (M5GFX →
that PWM). FreeInk Paper Mono: G3 alt-function **PWM0**,
duty pair `PWM0_L` / `PWM0_HC`, frequency 5 kHz
(`PWM_FREQ_L`), 12-bit, enable bit on the high byte.
GPIO3 mux bits `11` (`0xC0`) select that engine, not PWM1.

One CTRL into one AW9967. Official HTML names **brightness**
only. There is no second LED string / warm PWM on this
product. FreeInk Paper Mono sets `gpioWarm`
`PIN_UNASSIGNED`; `setColorTemperature` records a percent
and does not drive hardware. CrossPoint Reader’s warmth
slider is gated on `hasColorTemperature()` (cool **and**
warm GPIOs). That is true on X4 Pro / Murphy M4, not here.

**Lite (`C153-Lite`, 2026-09-01):** right-edge slide
**changes brightness** with G3 / PWM0. Earlier the same
day, PWM1 writes left the lamp constant. Not a lux or
percent meter.

**Lite (2026-09-02, Stage B):** PWM0 `lamp=1024` with
`PYG3` (`EPD_VDD`) off left the lamp **dark**. The same
duty after `PYG3` high (no `EPD_RST`, no OTP) was **on**.
AW9967 needs that L3B rail, not only G3 PWM. `C153` open.
[nyc-frontlight](../resources/not-yet-confirmed.md#nyc-frontlight).

## Do not

- Drive GPIO45/46 as a software power latch.
- Leave L3B EPD rail on across a shutdown without an OTP
  deep-sleep command if the OTP example says otherwise (read
  OTP-Demo; do not guess analog rails).
- Keep IP2315 mounted “so the gauge is always there”.
