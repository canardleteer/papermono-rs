---
name: papermono-rs
description: >-
  Use when working in the papermono-rs repository: cargo xtask, ci,
  detect-connected, monitor, backup / confirm / restore, flash-app,
  build-fw, the USB session lock, clap / espflash host CLI rules, board
  crates (`m5stack-papermono-lite`, `m5stack-papermono`), or this
  repository's Rust path on the M5Stack PaperMono or PaperMono-Lite.
  Board pins, rails, and datasheets live in the sibling
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
2. **Rust** — [references/rust.md](references/rust.md). Chip
   crates, `docs/API-RULES.md`, `docs/CRATES.md`, Xtensa
   `build-fw`.
3. **Layout** — [references/layout.md](references/layout.md).
   Workspace paths, clap/espflash/MSRV, lockfile. Splash
   Ferris (SVG + 1bpp) lives under
   `firmware/embassy-debug/assets/`
   ([SOURCE.md](../../../firmware/embassy-debug/assets/SOURCE.md)).
   Human how-to:
   [getting-started.md](../../../docs/getting-started.md),
   firmware package READMEs, root firmware-examples.
4. **Hardware** —
   [`m5stack-papermono-hardware`](../m5stack-papermono-hardware/SKILL.md).
   Pins, rails, datasheets, SKU differences.

## Do not connect unless asked

Discovery and flash I/O go through `cargo xtask`, not bare
`espflash`, `esptool`, `idf.py flash`, or PlatformIO upload. Do not
run those tools, `probe-rs`, or `cargo xtask` against hardware unless
the human asked to run that live command:

- Live: `detect-connected --probe`, live `backup-factory-firmware`,
  `confirm-factory-firmware`, `restore-factory-firmware`,
  `flash-app`, `monitor`
- Host-only (no USB): `detect-connected` without `--probe`,
  `backup-factory-firmware --import`, `vet-idle-log`,
  `build-fw`, `ci`

When a live ask is present, the **only** in-repo device I/O is
`cargo xtask` as catalogued in [xtask.md](references/xtask.md).
`monitor` needs the usbfs udev rule
([xtask.md](references/xtask.md#usbfs-udev-for-monitor)).

When they accept `flash-app`, pack every **safe unattended**
probe into that image (I2C roster, `FLAG`, `CHIP_ID`, lamp +
`EPD_VDD`, leftover input levels). Do not split those across
downloads. Host-only captures that need no button run in the
same session. Packed listen-only radio (`wifi n=` / `ble n=`,
no MAC/BSSID/IRK, no NVS write) stays ask-first; then those
counts ride that listen. The landing `embassy-debug` image
already defaults `--features radio` for BLE pairing + Wi-Fi
survey / SoftAP cards (idle until touch). Sleep
(`wake src=` / `sleep rtc=`) is the same ask-first rule for
packed probes. The always-on copy is root `AGENTS.md`
(**Pack one flash**).

Implemented is not proven. Read the
[tool verification ledger](references/xtask.md#tool-verification-ledger)
before assuming a command works on silicon.

**Lite (`C153-Lite`) live so far:** `detect-connected` (run
and download), `--probe` (`NoReset`), named backup /
confirm / restore, `flash-app` (`factory` at `0x10000`;
short-press red after), and `monitor` (stock silent;
custom images print `simple-debug:`). Silicon facts:
hardware
[measure.md](../m5stack-papermono-hardware/references/measure.md).
`C153` USB / JEDEC / partition table have not been
measured. Official HTML `epd_*` times are PaperMono lab
reference only
([display.md](../m5stack-papermono-hardware/references/display.md)).

A device may be attached for unrelated reasons; ignore it.

## Ferris splash art

In-repo under
[`firmware/embassy-debug/assets/`](../../../firmware/embassy-debug/assets/).
Catalog and encode:
[SOURCE.md](../../../firmware/embassy-debug/assets/SOURCE.md).

- **Original Ferris:** Karen Rustad Tölva,
  [rustacean.net](https://rustacean.net/)
  (`rustacean-flat-happy`). rustacean.net waives copyright
  and neighboring rights (CC0-style dedication).
- **Line-art monification:** in-repo line art
  (`ferris-happy-line-art.svg`).
- Firmware include is `ferris.1bpp` (360×240). Invert pair
  is stored beside it, not linked into the image.

## Crate map

| Path | Role |
| --- | --- |
| `crates/papermono-log/` | Host-tested USB-Serial/JTAG lines for both images |
| `crates/ssd1677-otp/` | Panel OTP sequences. No MCU LUT |
| `crates/m5pm1/` | PMIC registers + PWM0 |
| `crates/m5ioe1/` | Expander banks + IP2315 gate typestate |
| `crates/m5stack-papermono-lite/` | Board crate. `C153-Lite` + shared pin map |
| `crates/m5stack-papermono/` | Board crate. `C153`; re-exports Lite; NFC + LoRa |
| `host/papermono-host/` | Host library. Live methods take the USB lock |
| `host/papermono-host/udev/` | usbfs udev rule for `monitor` |
| `xtask/` | Clap front-end (`cargo xtask`, `publish = false`) |
| `developer-data/` | Gitignored. Sealed snapshots under `backups/`; confirm JSON under `confirm-records/` |
| `firmware/simple-debug/` | Workspace member, not a default-member. Blocking `esp-hal` Lite heartbeat |
| `firmware/embassy-debug/` | Workspace member, not a default-member. Embassy cards + lamp |

Crate verdicts before adoption:
[docs/CRATES.md](../../../docs/CRATES.md).

Never commit a MAC, serial number, USB serial string, NVS blob,
flash image, dump SHA, or unit-id. `developer-data/` is
gitignored on purpose. Never add a Cargo `runner`. Never
`erase-flash`. Never `espflash flash`. Do not assume arbitrary
partition offsets or 32 MB geometry.
Do not init NFC or LoRa on PaperMono-Lite. Official eval
firmware is
[M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)
(one ELF; runtime NFC probe). Sequences live in the hardware
skill
[user-demo.md](../m5stack-papermono-hardware/references/user-demo.md).
Do not `idf.py flash` it from this repo.
