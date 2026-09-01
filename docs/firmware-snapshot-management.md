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

- M5Stack publishes a restore image. That is **not** a substitute
  for this unit’s NVS / PHY cal.
- Never `erase-flash`. Never `espflash flash` (default bootloader
  and table).
- Do not invent Sticky’s “never write below `0x90000`” line.
- Dump length is the **measured** flash size (official 16 MB →
  `0x1000000`). Refuse `flash-32mb.bin` unless that length
  matches.
- UserDemo `partitions.csv` matches **Lite stock** at `0x8000`.
  PIO `default_16MB.csv` is still a different table. `C153` is
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

## Confirm (Lite, live)

`cargo xtask confirm-factory-firmware --capture stock-lite`
(2026-09-01) re-read the chip and matched the named capture.
Use `--capture SLUG` for a `--name` dump. Without it the tool
looks under `original/` (that path is still untested).

Same download-mode read as backup: `NoReset`, flash stub,
16×1 MiB windows, 16777216 bytes. Two flasher connects
(board-info, then the dump), so the flash-stub line and the
“baud rate higher than 115,200” warning appear twice.
`ESPFLASH_BAUD` 921600 still finished (~10.6 s/MiB, ~3 min
for 16 MB).

`read-flash N/16 ... elapsed=` is time since dump **start**,
logged **before** that window’s read. Window 1 is ~0; do not
treat it as a 1 µs transfer. espflash writes each window to a
`/tmp/papermono-xtask-*.bin` tempfile.

Stdout on match is `confirm: <unit-id> matches original` even
when the baseline is a capture. Do not paste that unit-id.
Confirm does not rewrite the snapshot.

## Tool verification ledger

Implemented is not proven. Agents: read the same table in
[xtask.md](../.agents/skills/papermono-rs/references/xtask.md#tool-verification-ledger)
before a live command.

Status vocabulary: **Host-only tested**, **Live tested**,
**Implemented, not live-tested**, **Stub**, **Not ported**.

| Command | Status | SKU / date | Next safe step |
| --- | --- | --- | --- |
| `ci` | Host-only tested | host / 2026-08-31 | Keep in the gate (`fmt`, clippy, test, rumdl, machete, audit) |
| `detect-connected` | Live tested | `C153-Lite` / 2026-08-31 | Run **and** download: same `303a:1001`, product “USB JTAG/serial debug unit”, by-id present (iSerial redacted), kernel `ttyACM*` |
| `detect-connected --probe` | Live tested | `C153-Lite` / 2026-08-31 | After red-blink download, `NoReset` board-info: ESP32-S3 v0.2, 40 MHz, 16 MB flash. MAC redacted. `security_info` Display is printed; do not paste unique fields. JEDEC/PSRAM still `nyc-flash-id` |
| `backup-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--name stock-lite` after red-blink download: `NoReset`, flash stub, 16×1 MiB windows, 16777216 bytes. Capture (uncertain stock). Baud warning at 921600; dump finished. Do not commit dumps. Confirm `--capture stock-lite` matched later the same day |
| `confirm-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--capture stock-lite`: same windowed read as backup; live dump matched. Stdout names a unit-id (do not paste). `elapsed=` is cumulative at window start. Writes `confirm-records/` JSON even on match. Default (no `--capture`) untested. Next: restore / `monitor` |
| `restore-factory-firmware` | Implemented, not live-tested | — | Snapshot + human write ask |
| `monitor` | Implemented, not live-tested | — | Human ask; read-only |
| `vet-idle-log` | Stub | — | Firmware grammar |
| `flash-app` / `learn-uart` / `build-fw` | Not ported | — | After live table + snapshot |

`--probe`, `backup-factory-firmware`, and confirm `--capture`
are **Live tested** on Lite. Flash size is 16 MB; the live
table matches UserDemo `partitions.csv`. Restore / `monitor`
still need a human ask. A Lite result does not confirm
`C153`. Do not commit `developer-data/` or print dump SHA /
unit-id / MAC.
