# cargo xtask

Invoke from the repo root (`cargo xtask <subcommand>`).
`cargo xtask --help` and `cargo xtask <cmd> --help` are the flag
source of truth. Keep this page and
[firmware-snapshot-management.md](../../../../docs/firmware-snapshot-management.md)
in the same change as any CLI change.

Live commands take `--port` or `ESPFLASH_PORT`; if unset they
require exactly one Espressif `303a:1001` and refuse QinHeng
`1a86:55d3` **before** opening a port. Live commands that connect
or listen take the [USB session lock](#usb-session-lock).

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
| `monitor` | yes | USB-Serial/JTAG listen at 115200 via usbfs CDC (no ACM TTY, no `--acm-tty`) |

```shell
cargo xtask detect-connected
# cargo xtask detect-connected --all-devices
# cargo xtask detect-connected --probe
# cargo xtask backup-factory-firmware --name stock-lite
# cargo xtask backup-factory-firmware --as-original
# cargo xtask confirm-factory-firmware
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
   **Implemented, not live-tested**. `--probe` is **Live tested**
   on Lite and flash size is 16 MB; dump still needs a human
   ask. A Lite result does not confirm `C153`.
3. A result on `C153-Lite` does not confirm `C153`.
4. Silicon facts go to the hardware skill. This table is **tool**
   status.

The same rows are in
[firmware-snapshot-management.md](../../../../docs/firmware-snapshot-management.md#tool-verification-ledger).

| Command | Status | SKU / date | Next safe step |
| --- | --- | --- | --- |
| `ci` | Host-only tested | host / 2026-08-31 | Keep in the gate (`fmt`, clippy, test, rumdl, machete, audit) |
| `detect-connected` | Live tested | `C153-Lite` / 2026-08-31 | Run **and** download inventory: same `303a:1001`, product “USB JTAG/serial debug unit”, by-id present (iSerial redacted), kernel `ttyACM*` |
| `detect-connected --probe` | Live tested | `C153-Lite` / 2026-08-31 | After red-blink download, `NoReset` board-info: ESP32-S3 v0.2, 40 MHz, 16 MB flash. MAC redacted. JEDEC/PSRAM still `nyc-flash-id` |
| `backup-factory-firmware` | Implemented, not live-tested | — | Human ask while in download (or re-enter via red blink). Dump 16 MiB |
| `confirm-factory-firmware` | Implemented, not live-tested | — | After a Lite snapshot exists |
| `restore-factory-firmware` | Implemented, not live-tested | — | Do not run until a matching snapshot exists **and** a human wants a write |
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
blob, or flash image. `developer-data/` is gitignored on purpose.
