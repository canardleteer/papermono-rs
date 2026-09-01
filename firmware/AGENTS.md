# firmware/

Firmware packages are **not** Cargo workspace members yet. This
file is here so the **nearest** `AGENTS.md` wins when they land.
Root rules still apply: [AGENTS.md](../AGENTS.md). Board contract:
[m5stack-papermono-hardware](../.agents/skills/m5stack-papermono-hardware/SKILL.md).

Do not `idf.py flash`, `espflash flash`, or `erase-flash` from this
tree. Host I/O stays `cargo xtask`. `flash-app` / `build-fw` are
not ported. Do not add a Cargo `runner`.

## Planned packages

Keep Xtensa packages out of `default-members`. When they land,
exclude them from host rust-analyzer check.

| Path | Stack | First SKU |
| --- | --- | --- |
| `simple-debug/` | blocking `esp-hal` | Lite (`m5stack-papermono-lite`) |
| `embassy-debug/` | `esp-hal` + Embassy | Lite (`m5stack-papermono-lite`) |

`esp-idf-hal` remains a valid stack (hardware skill `rust.md`); it
is not a first image. A Cargo feature may pull `m5stack-papermono`
for `C153`. Default images depend on the Lite board crate only.

Envelope for the first images:

- No NFC, no LoRa
- No waveform LUT (OTP first)
- GPIO45/46 are PDM, not a power latch
- Park IP2315 off the system I2C bus except a gated charge
  transaction
- GPIO0 and GPIO3 are straps (M5PM1 `BOOT_OUT`, KEY2)

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

Use these M5GFX labels as the enum titles (board crate:
`m5stack-papermono-lite::display::RefreshMode`). Lab
single-refresh times for **both** SKUs live in
[display.md](../.agents/skills/m5stack-papermono-hardware/references/display.md):

| Enum title | Lab time |
| --- | ---: |
| `epd_quality` | 4.71 s |
| `epd_text` | 0.45 s |
| `epd_fast` | 0.34 s |
| `epd_fastest` | 0.07 s |

Do not invent a 105-byte LUT. OTP first.
