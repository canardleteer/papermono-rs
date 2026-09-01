# embassy-debug

Planned `esp-hal` + Embassy proof-of-life image. **Not** a workspace
member yet. Host `cargo test` must not compile this package.

First SKU: PaperMono-Lite via `m5stack-papermono-lite`. Do not init
NFC or LoRa. No LUT. No Cargo `runner`. Flash I/O stays
`cargo xtask` after `build-fw` / `flash-app` exist.

Envelope, named constants, and `epd_*` titles: parent
[AGENTS.md](../AGENTS.md). Live-ask: root
[AGENTS.md](../../AGENTS.md).

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
