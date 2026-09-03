# m5stack-papermono

PaperMono (`C153`) board types: re-export `m5stack-papermono-lite`,
then NFC (`nfc`) and LoRa (`lora`). Panel call site is still
`display::OtpRefresh`. Do not use this crate on PaperMono-Lite.
No `esp-hal`. No LUT. SKU split (two crates, not a `nfc` /
`lora` feature): [crates/AGENTS.md](../AGENTS.md).

Lite (2026-09-01) FT XY / OTP partial+mono facts are **Lite
only**. Do not copy them onto `C153` (`nyc-canvas-orient`,
`nyc-ft6336-area`, `nyc-ft6336-points`).

Product PinMap names LoRa SPI as SPI1; UserDemo uses `SPI3_HOST` on
those GPIOs. Keep both names. Do not flatten.

Live-ask, never-erase, and flash I/O: root
[AGENTS.md](../../AGENTS.md). Pin map:
[m5stack-papermono-hardware](../../.agents/skills/m5stack-papermono-hardware/SKILL.md).

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
