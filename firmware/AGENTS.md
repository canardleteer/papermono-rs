# firmware/

Firmware packages are not Cargo workspace members yet. This
file is here so the **nearest** `AGENTS.md` wins when they
land. Root rules still apply:
[AGENTS.md](../AGENTS.md). Board contract:
[m5stack-papermono-hardware](../.agents/skills/m5stack-papermono-hardware/SKILL.md).

Do not `idf.py flash`, `espflash flash`, or `erase-flash`
from this tree. Host I/O stays `cargo xtask`.

## Named constants and datasheets

Do not put magic numbers or bytes in firmware. Use grouped
`enum` / `const` values with logical names. Every definition
comments **what it means** and **where it came from**.
Markdown (including rustdoc) prefers those titles, not the
raw encoding.

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

## EPD refresh modes

Use these M5GFX labels as the enum titles (Rust: `EpdQuality`
and friends, documented as `epd_quality`, …). Lab single-refresh
times for **both** SKUs live in
[display.md](../.agents/skills/m5stack-papermono-hardware/references/display.md):

| Enum title | Lab time |
| --- | ---: |
| `epd_quality` | 4.71 s |
| `epd_text` | 0.45 s |
| `epd_fast` | 0.34 s |
| `epd_fastest` | 0.07 s |

Do not invent a 105-byte LUT. OTP first.
