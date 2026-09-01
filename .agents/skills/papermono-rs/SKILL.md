---
name: papermono-rs
description: >-
  Use when working in the papermono-rs repository: cargo xtask, ci,
  detect-connected, monitor, backup / confirm / restore, the USB
  session lock, clap / espflash host CLI rules, or this repository's
  Rust path on the M5Stack PaperMono or PaperMono-Lite. Board pins,
  rails, and datasheets live in the sibling
  m5stack-papermono-hardware skill — read that first for wiring.
---

# papermono-rs

Host tools and Rust software path for **this repository**. Board
wiring, enclosure, and datasheets are
[`m5stack-papermono-hardware`](../m5stack-papermono-hardware/SKILL.md).
Read that skill first for pins and rails. Do not mix a stack’s APIs
into the pin map.

This repository is **host-verified by default**. Landing xtask source
is not permission to open a port. Do not open USB-Serial/JTAG unless
the human **explicitly asked to run** a live command on a device in
that message. The always-on copy of that gate is the root
`AGENTS.md`.

## How to read this skill

1. **xtask** — [references/xtask.md](references/xtask.md). Command
   catalog, **tool verification ledger**, USB session lock,
   `ESPFLASH_PORT`, no Cargo runner. Snapshot how-to:
   [firmware-snapshot-management.md](../../../docs/firmware-snapshot-management.md).
2. **Layout** — [references/layout.md](references/layout.md).
   Workspace paths, clap/espflash/MSRV, lockfile.
3. **Hardware** —
   [`m5stack-papermono-hardware`](../m5stack-papermono-hardware/SKILL.md).
   Pins, rails, datasheets, SKU differences.

## Do not connect unless asked

Discovery and flash I/O go through `cargo xtask`, not bare
`espflash`, `esptool`, `idf.py flash`, or PlatformIO upload. Do not
run those tools, `probe-rs`, or `cargo xtask` against hardware unless
the human asked to run that live command:

- Live: `detect-connected --probe`, live `backup-factory-firmware`,
  `confirm-factory-firmware`, `restore-factory-firmware`, `monitor`
- Host-only (no USB): `detect-connected` without `--probe`,
  `backup-factory-firmware --import`, `vet-idle-log`, `ci`

When a live ask is present, the **only** in-repo device I/O is
`cargo xtask` as catalogued in [xtask.md](references/xtask.md).
`monitor` needs the usbfs udev rule
([xtask.md](references/xtask.md#usbfs-udev-for-monitor)).

Implemented is not proven. Read the
[tool verification ledger](references/xtask.md#tool-verification-ledger)
before assuming a command works on silicon.

**Lite (`C153-Lite`) live so far:** `detect-connected` (run and
download), `--probe` (`NoReset`),
`backup-factory-firmware --name stock-lite`,
`confirm-factory-firmware --capture stock-lite`, `monitor`
(`--for 20`, run mode, silent), and
`restore-factory-firmware --yes --capture stock-lite` (then
confirm still matched; unit still booted). Same USB IDs
(`303a:1001`). Chip ESP32-S3 v0.2, 40 MHz, 16 MB flash.
Capture is this unit only (uncertain stock). Silicon rows:
hardware
[measure.md](../m5stack-papermono-hardware/references/measure.md).
`C153` USB / JEDEC / partition table have not been measured.
Lab EPD refresh times are on both SKUs
([display.md](../m5stack-papermono-hardware/references/display.md)).

A device may be attached for unrelated reasons; ignore it.

## Crate map

| Path | Role |
| --- | --- |
| `host/papermono-host/` | Host library. Live methods take the USB lock |
| `host/papermono-host/udev/` | usbfs udev rule for `monitor` |
| `xtask/` | Clap front-end (`cargo xtask`, `publish = false`) |
| `developer-data/` | Gitignored. Sealed snapshots under `backups/`; confirm JSON under `confirm-records/` |
| `firmware/` | Not members yet. [firmware/AGENTS.md](../../../firmware/AGENTS.md) |

Never commit a MAC, serial number, USB serial string, NVS blob,
flash image, dump SHA, or unit-id. `developer-data/` is
gitignored on purpose. Never add a Cargo `runner`. Never
`erase-flash`. Never `espflash flash`. Do not invent Sticky
`0x90000` / 32 MB geometry.
Do not init NFC or LoRa on PaperMono-Lite. Official eval
firmware is
[M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)
(one ELF; runtime NFC probe). Sequences live in the hardware
skill
[user-demo.md](../m5stack-papermono-hardware/references/user-demo.md).
Do not `idf.py flash` it from this repo.
