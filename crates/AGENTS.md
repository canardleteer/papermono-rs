# crates/

Host-testable board and chip crates. Default-members. No
`esp-hal`. Chip drivers stay MCU-agnostic (`embedded-hal` 1.0
only). Rules: [docs/API-RULES.md](../docs/API-RULES.md).

| Path | SKU | Role |
| --- | --- | --- |
| `papermono-log/` | — | Host-tested USB-Serial/JTAG lines for `simple-debug-fw` and `embassy-debug-fw`. Scene carousel includes `wifi_survey` / `wifi_ap`; line kinds `wifi_survey` / `wifi_ap` / `wifi_http` |
| `ssd1677-otp/` | — | Panel OTP sequences. No MCU LUT |
| `m5pm1/` | — | PMIC registers, PWM0, ADC, battery %, red LED |
| `m5ioe1/` | — | Expander banks + IP2315 gate typestate |
| `st25r3916/` | — | ST25R3916 NFC transceiver driver (initiator, target profiles, PT_Memory, framing) |
| `m5stack-papermono-lite/` | `C153-Lite` | Shared pin map (both SKUs) + `BoardModel` enum and profile |
| `m5stack-papermono/` | `C153` | Re-exports Lite; adds NFC + LoRa |

## SKU split

Two crates, not a `lite` / `nfc` Cargo feature. They are not two
boards: Lite is the shared map; `m5stack-papermono` is a radio
add-on. Do not fork examples or drivers along the SKU line.

Put new code where the hardware is:

- Shared pins, buses, panel, PMIC, touch, buzzer, board profile:
  `m5stack-papermono-lite` (or a chip-driver crate both SKUs
  use). Panel call site is `display::OtpRefresh`.
  `display::RefreshMode` is the HTML `epd_*` catalog only.
- NFC / LoRa pins and bring-up: `m5stack-papermono` only. A
  C153 firmware package depends on that crate when the image
  actually uses a radio.
- Proof-of-life, EPD demos, host tools: one package, Lite
  board crate, unless the image talks to a radio.

Runtime profile detection uses `BoardModel` in `m5stack-papermono-lite`:
a unified firmware binary probes ST25R3916 NFC IC identity at `0x50`
at boot. If unpopulated (`C153-Lite`), the board model is set to
`PaperMonoLite` and all radio GPIOs stay undriven and floating.
For minimal images, `--no-default-features --features lite` prunes
the `m5stack-papermono` dependency at compile time.

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
