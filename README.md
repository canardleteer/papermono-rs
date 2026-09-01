# `papermono-rs`

> Embedded Rust tooling and crates for the
> [M5Stack PaperMono](https://docs.m5stack.com/en/core/PaperMono)
> and
> [PaperMono-Lite](https://docs.m5stack.com/en/core/PaperMono-Lite).

This repository is a peer of
[`sticky-rs`](https://github.com/canardleteer/sticky-rs).
The first drop is the board contract. Firmware packages and host
tools are not here yet.

- **Read [docs/SAFETY.md](docs/SAFETY.md) before flashing or
  probing a unit.** A mistake can damage the panel (DC imbalance /
  invented LUTs), hang the system I2C bus (IP2315), or lose
  per-unit PHY calibration.
- [Hardware details](.agents/skills/m5stack-papermono-hardware/SKILL.md).
  Pin map, rails, SKU differences, datasheet cache.
- Open measurements:
  [not-yet-confirmed.md](.agents/skills/m5stack-papermono-hardware/resources/not-yet-confirmed.md).
  Lite run-mode USB is measured (`303a:1001`). Official docs
  are **not measured** unless a `nyc-*` recipe closed. Name
  PaperMono (`C153`) vs PaperMono-Lite (`C153-Lite`).
- Other docs live in [`docs/`](./docs).
  `docs/SAFETY.md` and `docs/DATASHEETS.md` are symlinks into the
  hardware skill.

> [!IMPORTANT]
>
> Anything we have not measured on a PaperMono or PaperMono-Lite
> in hand is still official-docs intent. The backlog is
> [`nyc-*`](.agents/skills/m5stack-papermono-hardware/resources/not-yet-confirmed.md).

## SKUs

| SKU | Product | Difference |
| --- | --- | --- |
| `C153` | PaperMono | ST25R3916 NFC + Stamp LoRa-1262; gray case |
| `C153-Lite` | PaperMono-Lite | No NFC, no LoRa; white case |

Shared: ESP32-S3R8, 16 MB flash, 8 MB octal PSRAM, 3.97" 480×800
SSD1677 4-gray, FT6336G, frontlight, M5PM1, M5IOE1, 1150 mAh.

## License

Sources in this repository are licensed under the MIT license. See
[LICENSE](LICENSE).

M5Stack, PaperMono, Espressif, and other product or company names
are trademarks of their respective owners. This project does not
claim those marks or their copyrights, and is not affiliated with
or endorsed by those owners.
