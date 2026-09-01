# m5stack-papermono-lite

Shared board types for PaperMono-Lite (`C153-Lite`) and the nets both
SKUs share. Pins, addresses, expander, PMIC `Gn`, panel geometry,
`RefreshMode` (`epd_quality` / `epd_text` / `epd_fast` /
`epd_fastest`). No `esp-hal`. No LUT. No NFC. No LoRa.

Firmware maps `pins::*` onto HAL pins in one place. Do not init NFC or
LoRa from an image that depends only on this crate. Leftover pads:
hardware skill `nyc-lite-nfc-pads` / `nyc-lite-lora-pads`.

Live-ask, never-erase, and flash I/O: root
[AGENTS.md](../../AGENTS.md). Pin map:
[m5stack-papermono-hardware](../../.agents/skills/m5stack-papermono-hardware/SKILL.md).

This crate’s `README.md` is the crates.io landing page. Relative
markdown links there only resolve inside this package.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
