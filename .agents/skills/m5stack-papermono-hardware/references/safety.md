# Safety

Every row below is something that can destroy hardware, destroy
data you cannot regenerate, or produce damage that shows up weeks
later. Sources are the vendor datasheet catalog
([datasheets.md](../resources/datasheets.md)) plus the board
contract in [SKILL.md](../SKILL.md). When those sources disagree,
name both sides; the skill user weighs them. Precedence:
[Authority](../SKILL.md#authority).

This file is the committed hazard table. Links to other skill
pages are relative to this `references/` directory. A consuming
repo may expose it as `docs/SAFETY.md` via a symlink (this
repository does). Host flash tools belong to the consuming
project.

**No unit was measured for this table.** Rows that depend on a
physical unit point at an `nyc-*` id. Name `C153` vs
`C153-Lite`. Official M5Stack warnings still apply.

## Hazard table

| Hazard | Safe default | Forbidden until proven |
| --- | --- | --- |
| E-paper OTP / LUT | Use panel OTP waveforms. M5Stack: M5GFX LUTs are currently unstable; prefer the OTP example. After ~10 partials, one full refresh | Invented or generic-example 105-byte `0x32` table; custom waveforms that are not DC-balanced; uninterrupted continuous partials |
| E-paper environment | Indoor, avoid prolonged sun / high UV / high temperature on the panel | Baking the glass in sun as a “feature” |
| IP2315 on I2C | Mount via M5IOE1 `PYG11_PWM3` only for the charge transaction, then disconnect. Sheet: I2C high is VBAT; VIN detect needs pins 8/9 high | Leaving `0x75` on the bus; assuming it always enumerates at low VBAT |
| Power path | M5PM1 button: short = on/reset, double = off, hold ~2 s until red blink = download | Sticky GPIO45/46 latch code; pulsing PDM pins as `PWR_HOLD` / `PWR_LOCK` |
| GPIO0 / GPIO3 straps | GPIO0 is M5PM1 `BOOT_OUT`; GPIO3 is KEY2. Ordinary IO only after `tH` | Driving them during strap sampling |
| GPIO45 / GPIO46 | PDM CLK/DAT. Also ESP32-S3 strapping (WPD) | Using them as a power latch |
| Lite SKU | Do not init ST25R3916 or SX1262 | Assuming NFC/LoRa GPIOs are free GPIO ([nyc-lite-nfc-pads](../resources/not-yet-confirmed.md#nyc-lite-nfc-pads), [nyc-lite-lora-pads](../resources/not-yet-confirmed.md#nyc-lite-lora-pads)) |
| Flash images | 16 MB-aware table. Snapshot that unit first if you care about PHY cal ([nyc-nvs-phy](../resources/not-yet-confirmed.md#nyc-nvs-phy)) | Sticky `0x90000` / 32 MB geometry; flashing one unit’s NVS onto another; assuming M5 factory-restore regenerates that unit’s PHY without checking |
| USB debug | Lite run **and** download: Espressif `303a:1001` USB JTAG/serial debug unit ([flashing.md](flashing.md#usb-measured)). Not CH343 | Treating USB-C as QinHeng `1a86:55d3`; assuming `probe-rs` until [nyc-usb-vid](../resources/not-yet-confirmed.md#nyc-usb-vid) |

## Why the panel rule comes first

M5Stack’s product notes:

1. Avoid direct sun / prolonged high temperature / strong UV.
2. After about ten software partial fast refreshes, do a
   full-screen refresh to clear ghosting.
3. The screen has built-in OTP waveforms. Custom external
   waveforms must stay DC-balanced or the panel can take
   irreversible damage.
4. Do not run uninterrupted continuous partials (long-term DC
   imbalance).

They also say M5GFX driver waveforms for this product are
currently unstable, and to prefer the manufacturer OTP example
([M5PaperMono-OTP-Demo](https://github.com/m5stack/M5PaperMono-OTP-Demo)).
FreeInk uses host-authored LUTs and “3-level gray” — name that
conflict; do not flatten it
([nyc-lut-path](../resources/not-yet-confirmed.md#nyc-lut-path)).

Do not invent or ship a default 105-byte LUT. Analog rails
(`0x03` / `0x04` / `0x2C`) are separate from the 105-byte
command; do not fill them from another product’s LUT tail.

## Why IP2315 stays off the bus

Official pin-map note plus the IP2315 sheet: I2C high is VBAT.
Pins 8/9 mux LED vs I2C. At VIN power-on both must sample high
or the chip stays in LED mode. USB connect triggers that
detect. If VBAT is too low, the charger may fail to enter I2C
mode and interfere with other devices on the same bus. M5IOE1
`PYG11_PWM3` gates that connection. During operation, do not keep
the IP2315 mounted; disconnect promptly after communication. A
short press of the power button resets the bus if it hangs.

## Why Sticky flash geometry does not apply

This product is documented as **16 MB** flash and native USB
CDC, not Sticky’s 32 MB + CH343. M5Stack publishes factory-reset
firmware. ESP32-S3 PHY calibration still typically lives in NVS.

Until [nyc-nvs-phy](../resources/not-yet-confirmed.md#nyc-nvs-phy)
and [nyc-partition-table](../resources/not-yet-confirmed.md#nyc-partition-table)
close:

1. Prefer a full-chip snapshot of **your** unit before the first
   custom write. Keep it gitignored under `developer-data/`.
2. Do not invent Sticky’s “never write below `0x90000`” line as
   if it were measured here.
3. Restore from **that unit’s** snapshot, or from M5Stack’s
   published restore image after you accept that PHY may differ.
4. Never flash one unit’s NVS onto another.

## Host-only scope

Device I/O belongs to the consuming project's tools. This
repository has none yet. Prefer region read/write over a
full-chip erase when a tool exists. Never use a flasher that
installs a default bootloader and partition table without you
choosing the 16 MB layout (plain `espflash flash` does).

Agents do not open a serial port unless a human asked. There is
no Cargo `runner`. Do not treat bare `espflash`, `esptool`,
`idf.py`, or PlatformIO as an in-repo path.

When firmware lands, encode M5PM1/M5IOE1 rail enables, IP2315
isolate, and EPD OTP-first as compile-time constraints so the
sequences above are type errors rather than field failures.
