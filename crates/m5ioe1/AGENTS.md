# m5ioe1

MCU-agnostic M5IOE1 GPIO. IP2315 gate typestate: `Parked` after
`new`, `Mounted` only inside the charge transaction. Leaving
the charger mounted can hang the bus.

Official `begin` (START+STOP, UID/REV) stays in firmware
(async settle). This crate is register RMW and the gate.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
