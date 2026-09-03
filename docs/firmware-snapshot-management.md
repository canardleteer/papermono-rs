# Firmware snapshot management

How to capture, confirm, and restore a PaperMono (`C153`) or
PaperMono-Lite (`C153-Lite`) without destroying data you cannot
get back.

Hazard rules stay in the hardware skill
[safety.md](../.agents/skills/m5stack-papermono-hardware/references/safety.md)
(`docs/SAFETY.md` is a symlink). This page is the operator
manual. Flag catalog:
[`.agents/skills/papermono-rs/references/xtask.md`](../.agents/skills/papermono-rs/references/xtask.md).

Do not commit `developer-data/`. Do not print MAC addresses or
USB serials in issues or chat. A leftover repo-root `backups/` is
also gitignored; do not use it.

## Honest limit

A snapshot is **this unit’s bytes**, bound by SHA-256 of the USB
iSerial (and MAC hash when `--probe` has run). It is not a
factory-reset image you can share.

- The ESP32-S3 Wi-Fi/BLE MAC address is permanently burned into
  physical eFuses at the factory, and standard ESP-IDF
  automatically performs RF calibration on boot into NVS when
  wiped. However, taking a snapshot preserves the exact stock
  partition layout, vendor demo binaries, and device state
  without needing external restore packages.
- Never `erase-flash`. Never `espflash flash` (default bootloader
  and table).
- Do not assume an arbitrary partition offset like `0x90000`.
- Dump length is the **measured** flash size (official 16 MB →
  `0x1000000`). Refuse `flash-32mb.bin` unless that length
  matches.
- Official factory demo firmware
  ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))
  `partitions.csv` matches **Lite stock** at `0x8000`. PIO
  `default_16MB.csv` is still a different table. `C153` is
  [nyc-partition-table](../.agents/skills/m5stack-papermono-hardware/resources/not-yet-confirmed.md#nyc-partition-table).

## Two snapshot kinds

| Kind | Path | When |
| --- | --- | --- |
| Original | `developer-data/backups/original/<unit-id>/` | You passed `--as-original` on uncertain stock. Write-once |
| Capture | `developer-data/backups/captures/<unit-id>/<slug>/` | `--name SLUG`. Named “what is on the chip now” |

`unit-id` on disk is `id-<8 hex>` of a bind hash (persist does
not pass the raw iSerial into the directory name). `unit_id()`
can still emit `lite-<last4>` if a caller supplies the serial;
colon-bearing MAC-shaped iSerials fail `validate_unit_id`. Bind
confirm / restore by the hashes, not by the directory name.

There is **no** in-repo factory catalog. Treat stock as uncertain.

## On-disk tree

Each snapshot directory has:

- `flash.bin` and `flash-<n>mb.bin` when the length is exact MiB
- `bootloader.bin`, `partition-table.bin`, `part-*.bin`
- optional `chunks/`
- `board-info.txt` (MAC redacted; may include `MAC sha256:`)
- `SHA256SUMS`
- `MANIFEST.json` (schema `papermono-firmware-snapshot/v1`)

The dest tree is sealed read-only after persist.

Confirm writes JSON under
`developer-data/confirm-records/<unit-id>/divergence-<unix>.json`
on every run, including a match. That file has unit-id and
region SHA-256. Do not commit it or paste it.

## Operator steps

Hold red ~2 s until blink (download), then:

```shell
cargo xtask backup-factory-firmware --name SLUG
cargo xtask confirm-factory-firmware --capture SLUG
# cargo xtask restore-factory-firmware --yes --capture SLUG
```

Use `--capture SLUG` for a `--name` dump. Without it the tool
looks under `original/` (still untested). Confirm does not
rewrite the snapshot. Stdout names a unit-id; do not paste it.

Run-mode listen (needs
[usbfs udev](../.agents/skills/papermono-rs/references/xtask.md#usbfs-udev-for-monitor)):

```shell
cargo xtask monitor --for 20 --output idle-simple.log
```

Stock factory demo firmware
([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))
is silent. Custom images print `simple-debug:`
lines. After `flash-app`, short-press red. Do not use
`monitor --reset` to recapture on Lite.

## Tool verification ledger

Implemented is not proven. Agents: read the same table in
[xtask.md](../.agents/skills/papermono-rs/references/xtask.md#tool-verification-ledger)
before a live command.

Status vocabulary: **Host-only tested**, **Live tested**,
**Implemented, not live-tested**, **Stub**, **Not ported**.

| Command | Status | SKU / date | Note |
| --- | --- | --- | --- |
| `ci` | Host-only tested | host / 2026-08-31 | fmt, clippy, test, rumdl, machete, audit |
| `detect-connected` | Live tested | `C153-Lite` / 2026-08-31 | Run and download: `303a:1001` |
| `detect-connected --probe` | Live tested | `C153-Lite` / 2026-08-31 | ESP32-S3 v0.2, 40 MHz, 16 MB |
| `backup-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--name stock-lite`. Do not commit dumps |
| `confirm-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--capture stock-lite` matched |
| `restore-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--yes --capture stock-lite` |
| `monitor` | Live tested | `C153-Lite` / 2026-09-01 | Stock silent. Custom images print `simple-debug:` |
| `monitor --reset` | Live tested | `C153-Lite` / 2026-09-01 | Not a recapture path |
| `vet-idle-log` | Host-only tested | host / 2026-09-01 | Parser in `papermono-log` |
| `build-fw` | Host-only tested | host / 2026-09-01 | `simple-debug` or `embassy-debug` |
| `encode-assets` | Host-only tested | host / 2026-09-02 | Rasterize and pack SVG line art into 1bpp bitmaps |
| `flash-app` | Live tested | `C153-Lite` / 2026-09-01 | `factory` at `0x10000`. Short-press red |

`--probe`, `backup-factory-firmware`, confirm `--capture`,
`monitor`, restore `--capture`, and `flash-app` are **Live
tested** on Lite. `build-fw` and `encode-assets` are host-only.
Flash size is 16 MB; the live table matches official factory
demo firmware
([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))
`partitions.csv`. A Lite result does not confirm `C153`. Do not
commit `developer-data/` or print dump SHA / unit-id / MAC.
