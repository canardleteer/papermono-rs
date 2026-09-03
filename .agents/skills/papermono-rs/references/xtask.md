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
| `flash-app` | yes | `write_bin` of `--image FILE` (a `save-image` payload, not an ELF) into snapshot **`factory`** only. Requires `--yes` and a matching original or `--capture`. Lite factory is `0x10000` / `0xF00000`. Flasher stays in bootloader; short-press red to run. Never a caller-chosen offset, never `espflash flash`, never erase |
| `vet-idle-log` | no | Idle grammar: `hello image=` / `sku=`, at least one `hb`, no `mac`, no `edge`, idle `btn_a=1 btn_b=1`. `--image embassy-debug` for the Embassy image. `--allow-activity` for a busy capture. Parse lives in `papermono-log`. There is no `learn-uart` |
| `ci` | no | Host + firmware clippy: fmt, host clippy/test, `cargo +esp clippy -p simple-debug-fw`, `embassy-debug-fw` (default, `--no-default-features`, then `touch` / `mic` / `panel` / `touch,radio` / `touch,sleep` with `--no-default-features`), rumdl, machete, audit. Needs esp toolchain |
| `monitor` | yes | USB-Serial/JTAG listen at 115200 via usbfs CDC (no ACM TTY, no `--acm-tty`). Needs the [usbfs udev rule](#usbfs-udev-for-monitor). After a listen, ACM may be gone; `--port /dev/bus/usb/BBB/DDD` from `lsusb` still works. `--reset` is in `--help` as **do not use**: Lite live DTR/RTS left CDC silent. Short-press red. Not download |
| `build-fw` | no | Host-only. `cargo +esp` (`--locked`) then `espflash save-image --flash-size 16mb`. IMAGE is `simple-debug` or `embassy-debug`. ELF and `.bin` under `target/xtensa-esp32s3-none-elf/release-fw/` |

```shell
cargo xtask detect-connected
# cargo xtask detect-connected --all-devices
# cargo xtask detect-connected --probe
# cargo xtask backup-factory-firmware --name stock-lite
# cargo xtask backup-factory-firmware --as-original
# cargo xtask confirm-factory-firmware --capture stock-lite
# cargo xtask restore-factory-firmware --yes --capture stock-lite
# cargo xtask build-fw simple-debug
# cargo xtask build-fw embassy-debug
# cargo xtask build-fw embassy-debug --no-default-features \
#   --features touch
# cargo xtask build-fw embassy-debug --no-default-features \
#   --features touch,mic,radio
# cargo xtask flash-app --image target/xtensa-esp32s3-none-elf/release-fw/simple-debug.bin --yes --capture stock-lite
# cargo xtask vet-idle-log --input idle-simple.log
# cargo xtask ci
# cargo xtask monitor --for 20 --output idle-simple.log
```

`idle-simple.log` is gitignored. `flash-app` needs a `save-image`
payload; run `cargo xtask build-fw` first.

## Intentional xtask scope exclusions

The following xtask features are excluded by design:

| Excluded item | Rationale |
| --- | --- |
| Live `learn-uart` YAML, `learn-uart-only`, `diff-learn-uart` | Intentional: parse and `vet-idle-log` only |
| `simple-debug --features operator` | Attended session not needed on USB-Serial/JTAG |
| `flash-app --allow-unknown-layout` | Lite factory geometry is known (`0x10000` / `0xF00000`) |
| `monitor --acm-tty` | Papermono is usbfs CDC. ACM DTR is unreliable |

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
   **Implemented, not live-tested**. `--probe`, backup, confirm
   `--capture`, restore `--capture`, `monitor`, and `flash-app`
   are **Live tested** on Lite.
   A Lite result does not confirm `C153`.
3. A result on `C153-Lite` does not confirm `C153`.
4. Silicon facts go to the hardware skill. This table is **tool**
   status.

The same rows are in
[firmware-snapshot-management.md](../../../../docs/firmware-snapshot-management.md#tool-verification-ledger).

| Command | Status | SKU / date | Note |
| --- | --- | --- | --- |
| `ci` | Host-only tested | host / 2026-08-31 | fmt, clippy, test, rumdl, machete, audit. Firmware clippy matrix |
| `detect-connected` | Live tested | `C153-Lite` / 2026-08-31 | Run and download: `303a:1001`. iSerial redacted |
| `detect-connected --probe` | Live tested | `C153-Lite` / 2026-08-31 | Download, `NoReset`: ESP32-S3 v0.2, 40 MHz, 16 MB. MAC redacted |
| `backup-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--name stock-lite` 16 MB capture. Do not commit dumps |
| `confirm-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--capture stock-lite` matched. No `--capture` untested |
| `restore-factory-firmware` | Live tested | `C153-Lite` / 2026-09-01 | `--yes --capture stock-lite`. `--part` / `original/` untested |
| `monitor` | Live tested | `C153-Lite` / 2026-09-01 | Stock silent. Custom images print `simple-debug:`. [udev](#usbfs-udev-for-monitor) |
| `monitor --reset` | Live tested | `C153-Lite` / 2026-09-01 | DTR/RTS: 0 CDC bytes. Not a recapture path |
| `vet-idle-log` | Host-only tested | host / 2026-09-01 | Parser in `papermono-log`. `--image embassy-debug` |
| `build-fw` | Host-only tested | host / 2026-09-01 | `simple-debug` or `embassy-debug`; `save-image --flash-size 16mb` |
| `flash-app` | Live tested | `C153-Lite` / 2026-09-01 | `factory` at `0x10000`. Short-press red after |

## USB session lock

`papermono_host::try_acquire` is the **one** exclusive USB session
for this board. Inventory without `--probe` does not take it.

## usbfs udev for `monitor`

Backup / confirm / restore talk to the ACM node. `dialout` is
enough for those. **`monitor` claims usbfs** so Linux `cdc-acm`
never opens the TTY (that can assert DTR and reset the S3).

Copy
[99-papermono-usb.rules](../../../../host/papermono-host/udev/99-papermono-usb.rules)
to `/etc/udev/rules.d/`:

```shell
sudo cp host/papermono-host/udev/99-papermono-usb.rules \
  /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Unplug and replug. Stay in `dialout` (re-login if you just
joined the group). Then:

```shell
lsusb | grep 303a:1001
# Bus 003 Device 013 → /dev/bus/usb/003/013
ls -l /dev/bus/usb/003/013
```

Expect `crw-rw---- 1 root dialout`. `00B/00D` in older notes
were placeholders; zero-pad the **Bus** and **Device** from
`lsusb` to three digits. `root root` `crw-rw-r--` is still
denied.

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
