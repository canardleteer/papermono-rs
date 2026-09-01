# m5stack-papermono

PaperMono (`C153`) board types: re-export `m5stack-papermono-lite`,
then NFC (`nfc`) and LoRa (`lora`). Do not use this crate on
PaperMono-Lite. No `esp-hal`. No LUT.

Product PinMap names LoRa SPI as SPI1; UserDemo uses `SPI3_HOST` on
those GPIOs. Keep both names. Do not flatten.

Live-ask, never-erase, and flash I/O: root
[AGENTS.md](../../AGENTS.md). Pin map:
[m5stack-papermono-hardware](../../.agents/skills/m5stack-papermono-hardware/SKILL.md).

This crate’s `README.md` is the crates.io landing page. Relative
markdown links there only resolve inside this package.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
