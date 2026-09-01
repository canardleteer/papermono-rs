# crates/

Host-testable board crates. Default-members. No `esp-hal`. Chip
drivers (when they land) stay MCU-agnostic and live beside these
packages.

| Path | SKU | Role |
| --- | --- | --- |
| `m5stack-papermono-lite/` | `C153-Lite` | Shared pin map (both SKUs) |
| `m5stack-papermono/` | `C153` | Re-exports Lite; adds NFC + LoRa |

Lite firmware depends on `m5stack-papermono-lite` only. Full-SKU
firmware depends on `m5stack-papermono`. Do not init NFC or LoRa on
Lite.

Xtensa images stay under `firmware/` and are **not** members yet.
Nearest rules: [firmware/AGENTS.md](../firmware/AGENTS.md). Root
live-ask: [AGENTS.md](../AGENTS.md).

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
