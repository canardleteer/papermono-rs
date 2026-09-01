# crates/

Host-testable board and chip crates. Default-members. No
`esp-hal`. Chip drivers stay MCU-agnostic (`embedded-hal` 1.0
only). Rules: [docs/API-RULES.md](../docs/API-RULES.md).

| Path | SKU | Role |
| --- | --- | --- |
| `papermono-log/` | — | Host-tested USB-Serial/JTAG lines for `simple-debug-fw` and `embassy-debug-fw` |
| `ssd1677-otp/` | — | Panel OTP sequences. No MCU LUT |
| `m5pm1/` | — | PMIC registers, PWM0, ADC |
| `m5ioe1/` | — | Expander banks + IP2315 gate typestate |
| `m5stack-papermono-lite/` | `C153-Lite` | Shared pin map (both SKUs) |
| `m5stack-papermono/` | `C153` | Re-exports Lite; adds NFC + LoRa |

## SKU split

Two crates, not a `lite` / `nfc` Cargo feature. They are not two
boards: Lite is the shared map; `m5stack-papermono` is a radio
add-on. Do not fork examples or drivers along the SKU line.

Put new code where the hardware is:

- Shared pins, buses, panel, PMIC, touch, buzzer:
  `m5stack-papermono-lite` (or a chip-driver crate both SKUs
  use). Panel call site is `display::OtpRefresh`.
  `display::RefreshMode` is the HTML `epd_*` catalog only.
- NFC / LoRa pins and bring-up: `m5stack-papermono` only. A
  C153 firmware package depends on that crate when the image
  actually uses a radio.
- Proof-of-life, EPD demos, host tools: one package, Lite
  board crate, unless the image talks to a radio.

Lite firmware depends on `m5stack-papermono-lite` only so it
cannot name radio GPIOs. Do not init NFC or LoRa on Lite. Do
not `#[cfg(feature = "lite")]` (or `nfc` / `lora`) through
display, I2C, or Embassy. UserDemo’s one-ELF runtime NFC probe
is vendor app policy, not this BSP; leftover pads stay
undriven until `nyc-lite-nfc-pads` / `nyc-lite-lora-pads`
close.

Collapsing the thin C153 crate into optional features later is
a rename. cfg-gating application code is the expensive
mistake.

Xtensa `simple-debug-fw` is a workspace member, **not** a
default-member. Nearest rules:
[firmware/AGENTS.md](../firmware/AGENTS.md). Root live-ask:
[AGENTS.md](../AGENTS.md).

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
