# cargo xtask

Invoke from the repo root (`cargo xtask <subcommand>`).
`cargo xtask --help` and `cargo xtask <cmd> --help` are the flag
source of truth. Keep this page and
[firmware-snapshot-management.md](../../../../docs/firmware-snapshot-management.md)
in the same change as any CLI change.

Live commands take `--port` or `ESPFLASH_PORT`; if unset they
require exactly one Espressif `303a:1001` and refuse QinHeng
`1a86:55d3` **before** opening a port. `303a:1001` is also the
stock ESP32-S3 USB-Serial/JTAG id on DevKits: two boards on
one host need `--port`. `classify()` treats any `303a:*` as
this product if sysfs PID is missing. Live commands that
connect or listen take the [USB session lock](#usb-session-lock).

Do not open a port unless a human asked. Silicon facts live in
[m5stack-papermono-hardware measure.md](../../m5stack-papermono-hardware/references/measure.md).

Implemented is not proven. Read the
[tool verification ledger](#tool-verification-ledger) before
assuming a command works on a unit.

## Catalog

| Command | USB? | How to use it |
| --- | --- | --- |
| `detect-connected` | no, unless `--probe` | USB inventory of `303a:1001`. `--all-devices` includes other adapters. iSerial is redacted (`present`). `--probe` is flasher board-info with `NoReset` (prefer power-button download). No MAC printed |
| `backup-factory-firmware` | live dump yes; `--import` no | Uncertain stock: `--name SLUG` or `--as-original`. Write-once under `developer-data/backups/`. Dump length is the measured size. Alias `backup-firmware`. `--import DIR` is host-only (`flash.bin` / `flash-16mb.bin`; refuse `flash-32mb.bin` unless length matches) |
| `confirm-factory-firmware` | yes | Compare live flash to the matching original, or `--capture SLUG`. Writes gitignored divergence JSON. Does not rewrite the snapshot |
| `restore-factory-firmware` | yes | `write_bin` of **that unit's** original, or `--capture SLUG`. Requires `--yes`. Full image at `0x0`, or `--part LABEL`. Never a full-chip erase |
| `vet-idle-log` | no | Stub. Exits “no image grammar” |
| `ci` | no | Host-only: fmt, clippy, test, rumdl, machete, audit |
| `monitor` | yes | USB-Serial/JTAG listen at 115200 via usbfs CDC (no ACM TTY, no `--acm-tty`). **Not live-tested.** Opening the kernel ACM node can still reset; this path claims usbfs instead |

```shell
cargo xtask detect-connected
# cargo xtask detect-connected --all-devices
# cargo xtask detect-connected --probe
# cargo xtask backup-factory-firmware --name stock-lite
# cargo xtask backup-factory-firmware --as-original
# cargo xtask confirm-factory-firmware --capture stock-lite
# cargo xtask restore-factory-firmware --yes
# cargo xtask ci
# cargo xtask monitor --for 20 --output idle.log
```

`flash-app`, `learn-uart`, and `build-fw` are **not ported**.

## Tool verification ledger

Status vocabulary (only these):

- **Host-only tested** — `ci`, clap tests, or a command that never
  opens a port. Name what ran.
- **Live tested** — that exact command ran on a named SKU. Record
  date and what was observed (VID:PID / product / by-id shape). No
  MAC, iSerial, or dump paths.
- **Implemented, not live-tested** — code exists; silicon is
  unproven.
- **Stub** — intentional refuse.
- **Not ported** — command does not exist.

Rules for advancing a row:

1. Do not run a live command unless a human asked in that message.
2. Do not skip a precondition that is still
   **Implemented, not live-tested**. `--probe`, backup, and
   confirm `--capture` are **Live tested** on Lite (16 MB).
   Restore / `monitor` still need a human ask. A Lite result
   does not confirm `C153`.
3. A result on `C153-Lite` does not confirm `C153`.
4. Silicon facts go to the hardware skill. This table is **tool**
   status.

The same rows are in
[firmware-snapshot-management.md](../../../../docs/firmware-snapshot-management.md#tool-verification-ledger).

| Command | Status | SKU / date | Next safe step |
| --- | --- | --- | --- |
| `ci` | Host-only tested | host / 2026-08-31 | Keep in the gate (`fmt`, clippy, test, rumdl, machete, audit) |
| `detect-connected` | Live tested | `C153-Lite` / 2026-08-31 | Run **and** download inventory: same `303a:1001`, product “USB JTAG/serial debug unit”, by-id present (iSerial redacted), kernel `ttyACM*` |
| `detect-connected --probe` | Live tested | `C153-Lite` / 2026-08-31 | After red-blink download, `NoReset` board-info: ESP32-S3 v0.2, 40 MHz, 16 MB flash. MAC redacted. `security_info` Display is printed; do not paste unique fields. JEDEC/PSRAM still `nyc-flash-id` |
| `backup-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--name stock-lite` after red-blink download: `NoReset`, flash stub, 16×1 MiB windows, 16777216 bytes (`0x1000000`). Capture under `developer-data/backups/captures/` (uncertain stock). espflash warns above 115200 (`ESPFLASH_BAUD` 921600); dump finished (~10.6 s/MiB). Do not commit the tree. Confirm `--capture stock-lite` matched later the same day |
| `confirm-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--capture stock-lite` after red-blink download: same `NoReset` / flash stub / 16×1 MiB windows as backup. Two flasher connects (board-info, then dump); baud warning both times. `elapsed=` is cumulative at **window start** (1/16 ≈ 0). ~10.6 s/MiB, ~3 min for 16 MB. Match stdout is `confirm: <unit-id> matches original` even for a capture; do not paste the id. Writes gitignored `confirm-records/` JSON even on match (region SHA; do not paste). Does not rewrite the snapshot. Default (no `--capture`, `original/`) untested. Next: restore / `monitor` |
| `restore-factory-firmware` | Implemented, not live-tested | — | Do not run until a human wants a write of **that unit’s** image |
| `monitor` | Implemented, not live-tested | — | Human ask; record silent vs printed |
| `vet-idle-log` | Stub | — | Leave until firmware grammar exists |
| `flash-app` / `learn-uart` / `build-fw` | Not ported | — | After a 16 MB-aware table is read from this unit and a snapshot exists |

## USB session lock

`papermono_host::try_acquire` is the **one** exclusive USB session
for this board. Inventory without `--probe` does not take it.

## Contracts

Do not add a Cargo `runner`. Do not put a device path in tracked
source. xtask may use the `espflash` library for region read/write
only; never a full-chip erase, never `espflash flash`.

Never commit a MAC address, serial number, USB serial string, NVS
blob, flash image, dump SHA, or unit-id. `developer-data/` is
gitignored on purpose (`backups/` and `confirm-records/`).
Confirm stdout prints a unit-id; do not paste it. Persist fills
`MANIFEST.app0_desc` only when a partition is labeled `app0`.
Lite stock uses `factory` (no `otadata`); that field stays empty
even though the factory slice has an ESP-IDF app descriptor.
