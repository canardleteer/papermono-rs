# m5pm1

MCU-agnostic M5PM1 registers. PWM0 duty / freq helpers. I2C
wrapper takes the bus (C-CTOR / C-FREE). `SYS_CMD` shutdown is
an explicit method.

Do not write PWM1 for PaperMono frontlight. Do not invent
registers. Catalog id `m5pm1`.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
