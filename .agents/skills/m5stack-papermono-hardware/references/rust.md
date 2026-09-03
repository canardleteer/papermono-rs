# Rust software paths

This skill is the board contract, not a host toolchain.
Consuming projects supply their own flash and UART tools. Two
stacks are valid on this MCU:

| Stack | When |
| --- | --- |
| `no_std`: `esp-hal` + `esp-rtos` / Embassy | Bare-metal async |
| `std`: `esp-idf-hal` + `esp-idf-svc` | Share ESP-IDF drivers/partition story with vendor C++ firmware |

Encode [pin-map.md](pin-map.md) in the two board crates:
`m5stack-papermono-lite` (`C153-Lite`, shared map) and
`m5stack-papermono` (`C153`, NFC + LoRa). That split is two
crates, not a Cargo feature
([crates/AGENTS.md](../../../../crates/AGENTS.md)). Chip drivers stay
MCU-agnostic. Register facts come from
[datasheets.md](../resources/datasheets.md). USB/flash geometry:
[flashing.md](flashing.md). Observed silicon:
[measure.md](measure.md) (Lite USB/flash). Official HTML
`epd_*` times are PaperMono lab reference
([display.md](display.md)).

Named `enum` / `const` values, not magic bytes. Comments cite
catalog id plus datasheet section number and title. Markdown
prefers those titles. Firmware call site is `OtpRefresh`
(`otp_gray` / `otp_mono` / `otp_partial`). `RefreshMode`
is the official HTML **M5GFX LUT Refresh Speed** catalog
only ([display.md](display.md)). Firmware packages also
read
[firmware/AGENTS.md](../../../../firmware/AGENTS.md).

Do not mix this page with PlatformIO / `idf.py`. Those trees are
wiring evidence in [cpp-platformio.md](cpp-platformio.md). Do not
treat `esp-hal` as the only legal Rust stack. Never a generic
SSD1677 four-gray LUT. Never GPIO45/46 power latching. Never `bq27xxx`.

Board crates live under `crates/` (host-testable, no `esp-hal`).
`simple-debug-fw` is a workspace member, not a default-member.
Nearest rules:
[firmware/AGENTS.md](../../../../firmware/AGENTS.md),
[crates/AGENTS.md](../../../../crates/AGENTS.md).
