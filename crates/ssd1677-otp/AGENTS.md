# ssd1677-otp

MCU-agnostic SSD1677 OTP sequences. No `0x32` LUT. No Sticky
`ssd1677-gray4`. Host tests: `embedded-hal-mock` transaction
scripts from catalog id `ssd1677` (Rev 1.0 Table 7-1 / §8).

`Partial` needs a mono baseline. `GrayFull` invalidates it.
After gray, `MonoFull` before any `Partial`. Do not send a
second bare Mode 1.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
