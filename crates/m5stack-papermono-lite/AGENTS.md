# m5stack-papermono-lite

Shared board types for PaperMono-Lite (`C153-Lite`) and the
nets both SKUs share. Chip registers live in `m5pm1`,
`m5ioe1`, and `ssd1677-otp`. This crate keeps board nets,
official 480×800 geometry, `otp_ram_to_usb_down`,
`RefreshMode` (HTML `epd_*` catalog), FT M5GFX
`decode_m5gfx` (not a FocalTech map), lamp gutter, and
GPIO42 LEDC window. Firmware call site is
`display::OtpRefresh`. No `esp-hal`. No LUT. No NFC. No
LoRa. Hardware skill `touch.md` / `display.md`.

Firmware maps `pins::*` onto HAL pins in one place. Do not init NFC or
LoRa from an image that depends only on this crate. Leftover pads:
hardware skill `nyc-lite-nfc-pads` / `nyc-lite-lora-pads`. SKU split
(two crates, not a `lite` feature): [crates/AGENTS.md](../AGENTS.md).

Live-ask, never-erase, and flash I/O: root
[AGENTS.md](../../AGENTS.md). Pin map:
[m5stack-papermono-hardware](../../.agents/skills/m5stack-papermono-hardware/SKILL.md).

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
