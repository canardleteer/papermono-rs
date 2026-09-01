# `papermono-rs`

> Embedded Rust tooling and crates for the
> [M5Stack PaperMono](https://docs.m5stack.com/en/core/PaperMono)
> and
> [PaperMono-Lite](https://docs.m5stack.com/en/core/PaperMono-Lite).

This repository is a peer of
[`sticky-rs`](https://github.com/canardleteer/sticky-rs).
The board contract and a safety-first host CLI (`cargo xtask`)
are here. Board crates:
`m5stack-papermono-lite` (`C153-Lite`, shared map) and
`m5stack-papermono` (`C153`, NFC + LoRa). Firmware images are
not workspace members yet.
[firmware/AGENTS.md](firmware/AGENTS.md) is in place for when
they land.

Host tools (clap + `papermono-host`): detect, snapshot
backup / confirm / restore, and a no-reset monitor. **Implemented
is not proven.** See the
[tool verification ledger](docs/firmware-snapshot-management.md#tool-verification-ledger)
before treating a command as working on a unit. Operator how-to:
[docs/firmware-snapshot-management.md](docs/firmware-snapshot-management.md).

- **Read [docs/SAFETY.md](docs/SAFETY.md) before flashing or
  probing a unit.** A mistake can damage the panel (DC imbalance /
  invented LUTs), hang the system I2C bus (IP2315), or lose
  per-unit PHY calibration.
- [Hardware details](.agents/skills/m5stack-papermono-hardware/SKILL.md).
  Pin map, rails, SKU differences, datasheet cache. Official
  eval HAL:
  [user-demo.md](.agents/skills/m5stack-papermono-hardware/references/user-demo.md).
- Open measurements:
  [not-yet-confirmed.md](.agents/skills/m5stack-papermono-hardware/resources/not-yet-confirmed.md).
  Lite run and download USB is measured (`303a:1001`); Lite
  flash size is 16 MB; Lite stock partition table matches
  UserDemo `partitions.csv`. Lab EPD refresh times
  (`epd_quality` / `epd_text` / `epd_fast` / `epd_fastest`)
  are measured on **both** SKUs
  ([display.md](.agents/skills/m5stack-papermono-hardware/references/display.md)).
  Official docs are **not measured** unless a `nyc-*` recipe
  closed or a Confirmed-live row exists. Name PaperMono
  (`C153`) vs PaperMono-Lite (`C153-Lite`).
- Other docs live in [`docs/`](./docs).
  `docs/SAFETY.md` and `docs/DATASHEETS.md` are symlinks into the
  hardware skill.

## Host CLI

```shell
cargo xtask detect-connected
cargo xtask ci
```

`--probe`, live backup, confirm, restore, and `monitor` need a
human ask. Download is a power-button hold (~2 s until the red
LED blinks), not DTR. QinHeng `1a86:55d3` is refused.

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
