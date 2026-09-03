# firmware/

Xtensa images. `simple-debug` and `embassy-debug` are workspace
members, **not** default-members. Host `cargo test` must not
compile them. Root rules still apply: [AGENTS.md](../AGENTS.md).
Board contract:
[m5stack-papermono-hardware](../.agents/skills/m5stack-papermono-hardware/SKILL.md).

Human how-to (what the image does, numbered flash / listen
steps) lives in each package `README.md` and
[docs/getting-started.md](../docs/getting-started.md). Keep
live-ask, envelope, named constants, and silicon notes here.

Do not `idf.py flash`, `espflash flash`, or `erase-flash` from this
tree. Host I/O stays `cargo xtask`. `build-fw` then `flash-app`
(`factory` at `0x10000` on Lite). Do not add a Cargo `runner`.
Xtensa rustflags `-Tlinkall.x` live in `.cargo/config.toml`.

## Packages

Keep Xtensa packages out of `default-members`. rust-analyzer
excludes `simple-debug-fw` and `embassy-debug-fw`.

| Path | Stack | First SKU | Status |
| --- | --- | --- | --- |
| `simple-debug/` | blocking `esp-hal` | Lite (`m5stack-papermono-lite`) | Member. USB-Serial/JTAG hello / hb / edge. No I2C / EPD / latch |
| `embassy-debug/` | `esp-hal` + Embassy | Lite (`m5stack-papermono-lite`) | Member. `image=embassy-debug`. Default `touch` + `panel`. `mic` / `radio` / `sleep` opt-in. Five OTP cards, no LUT |

`esp-idf-hal` remains a valid stack (hardware skill `rust.md`); it
is not a first image. Default images depend on
`m5stack-papermono-lite` only. A C153 image depends on
`m5stack-papermono` when it uses NFC or LoRa. SKU split:
[crates/AGENTS.md](../crates/AGENTS.md).

Envelope for the first images:

- No NFC, no LoRa
- No waveform LUT (OTP first)
- GPIO45/46 are PDM, not a power latch
- Park IP2315 off the system I2C bus except a gated charge
  transaction
- GPIO0 and GPIO3 are straps (M5PM1 `BOOT_OUT`, BUTTON B /
  `USER_KEY2`)

Packing (one flash, many safe probes) is a root rule:
[AGENTS.md](../AGENTS.md#pack-one-flash). Do not treat this
file as the only copy.

## Named constants and datasheets

Do not put magic numbers or bytes in firmware. Use grouped
`enum` / `const` values with logical names. Every definition
comments **what it means** and **where it came from**.
Markdown (including rustdoc) prefers those titles, not the
raw encoding. Prefer the board crate over a second copy of a
GPIO number.

The hardware skill may print hex and GPIO numbers in mapping
tables. Keep the map next to the name: meaning, and source.

Cite a datasheet by catalog **Id**, **section number** (when
the sheet has one), and **section title**. Do not cite page
numbers (they drift across translations and M5 copies).
Catalog:
[datasheets.md](../.agents/skills/m5stack-papermono-hardware/resources/datasheets.md).

Board nets: living HTML **PinMap** on the product pages
([catalog.md](../.agents/skills/m5stack-papermono-hardware/references/catalog.md)),
absorbed in
[pin-map.md](../.agents/skills/m5stack-papermono-hardware/references/pin-map.md).
Rust encoding: `crates/m5stack-papermono-lite` and
`crates/m5stack-papermono`.

## EPD refresh modes

Call site is `m5stack-papermono-lite::display::OtpRefresh`.
No MCU LUT. Do not stamp `RefreshMode` / `epd_*`. What to
do:
[display.md](../.agents/skills/m5stack-papermono-hardware/references/display.md).
What not to do:
[safety.md](../.agents/skills/m5stack-papermono-hardware/references/safety.md).

| `OtpRefresh` | CDC title | `0x22` |
| --- | --- | --- |
| `GrayFull` | `otp_gray` | `0xD7` |
| `MonoFull` | `otp_mono` | `0xF8` then `0x14` |
| `Partial` | `otp_partial` | `0xFF` |

`GrayFull` invalidates the mono baseline. `Partial` needs
that baseline. After gray, `MonoFull` before any `Partial`.
**Lite (2026-09-01) failure, abandoned:** skip left Ferris
until overdrawn; the `Partial` was fast. Successes and
failures: hardware skill
[display.md](../.agents/skills/m5stack-papermono-hardware/references/display.md#refresh-trials-lite-2026-09-01).
After `PARTIALS_BEFORE_FULL` (6) partials, one `MonoFull`
(`0` never). That cadence satisfies the official display safety
contract (a full refresh after roughly ten partials to prevent DC
imbalance). Do not send a second bare Mode 1.

`RefreshMode` is the official HTML **M5GFX LUT Refresh
Speed** catalog only (PaperMono lab, reference only). Not a
timeout and not a `0x22` map.

embassy-debug: tones use `GrayFull`. Splash / shapes /
legend use `paint_mono_fast` (`MonoFull` then `Partial`).
Target enter uses `enter_mono` (`MonoFull`). Marks use
`Partial`. After `PARTIALS_BEFORE_FULL` (6) partials, the
next mono path is `MonoFull`. Deep sleep after each
refresh; hardware reset to wake. No `otp_fast` stamp.
