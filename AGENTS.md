# papermono-rs

Board contract for the M5Stack PaperMono and PaperMono-Lite.
Host tools live under `host/papermono-host/` and `xtask/`.
Board crates live under `crates/`. Firmware images:
`simple-debug-fw` and `embassy-debug-fw` are workspace members,
not default-members.

## Hardware safety (read before writing code)

1. **OTP waveforms first.** Do not invent an MCU `0x32` LUT.
   Custom external waveforms must stay DC-balanced. After about
   ten partial refreshes, do a full refresh. Uninterrupted
   partials can damage the panel.
2. **Park IP2315 off the system I2C bus** except for the charge
   transaction (M5IOE1 `PYG11_PWM3`). Leaving it mounted can hang
   the bus, especially at low VBAT.
3. **Do not drive GPIO45/46 as a power latch.** Those pins are
   PDM CLK/DAT here. Power is the M5PM1 button.
4. **GPIO0 and GPIO3 are strapping pins.** GPIO0 is M5PM1
   `BOOT_OUT`; GPIO3 is BUTTON B (DOWN) / `USER_KEY2`.
5. **Download mode is a power-button hold** (~2 s until the red
   LED blinks), not DTR on a CH343.
6. **Snapshot first to preserve factory state.** The Wi-Fi/BLE MAC
   is burned in hardware eFuses and ESP-IDF auto-calibrates RF into
   NVS, but taking a snapshot preserves the stock partition layout,
   factory demo image, and existing flash state without needing
   vendor downloads.

Full hazard table: [docs/SAFETY.md](docs/SAFETY.md). Board facts
and source precedence:
[m5stack-papermono-hardware](.agents/skills/m5stack-papermono-hardware/SKILL.md).
`docs/SAFETY.md`, [docs/DATASHEETS.md](docs/DATASHEETS.md), and
[docs/not-yet-confirmed.md](docs/not-yet-confirmed.md) are
symlinks into that skill (`references/safety.md`,
`resources/datasheets.md`, `resources/not-yet-confirmed.md`).
Edit the skill files; rumdl lints those pages and excludes the
symlinks.

## Layout

Cargo workspace: `crates/*`, `host/*`, and `xtask` are
default-members. `firmware/simple-debug` and
`firmware/embassy-debug` are members, not default-members.
rust-analyzer excludes `simple-debug-fw` and `embassy-debug-fw`.
Board crate SKU split (two crates, not a feature):
[crates/AGENTS.md](crates/AGENTS.md).
Host CLI catalog and the tool verification ledger:
[papermono-rs xtask](.agents/skills/papermono-rs/references/xtask.md).
Fresh-start how-to:
[docs/getting-started.md](docs/getting-started.md).
A topical `AGENTS.md` in a subdirectory wins on conflict
with this one (nearest file).

## Do not connect to a physical device

Landing xtask source is not permission to open a port. Do not
run `espflash`, `esptool`, `idf.py flash`, PlatformIO upload,
`probe-rs`, or any serial monitor against hardware unless the
human **explicitly asked to run** that live command on a device
in that message. The only in-repo device I/O is `cargo xtask`.

Host-only (no port): `detect-connected` without `--probe`,
`backup-factory-firmware --import`, `vet-idle-log`,
`build-fw`, `ci`.

A device may be attached for unrelated reasons; ignore it. Never
commit a MAC address, serial number, USB serial string, NVS blob,
or flash image. `developer-data/` is gitignored on purpose.

## Pack one flash

When a human **accepts** a firmware flash, put every **safe
unattended** probe in that image. Do not spend a download and
boot on I2C alone if lamp, `FLAG`, leftover inputs, or (when
asked) radio can ride along. Close as many NYC rows as that
listen can, so they do not hold the red button again for
something that was already safe.

Safe unattended (no extra hands on the unit): I2C roster,
`FLAG`, `CHIP_ID`, lamp + `EPD_VDD`, leftover **input**
levels. Host-only captures that need no button (USB
interfaces, `probe-rs list` without reset) run in the same
session. Radio (`wifi n=` / `ble n=`, no MAC/BSSID/IRK, no
NVS write) stays default-off until they ask; then it rides
**that** listen, not a second image. Sleep (`wake src=` /
`sleep rtc=`) is the same: default-off, one image when
they ask. Do not pack a current-meter step.

Do not add OTP, SDMMC, buzzer, IP2315 hang, or RGB sweep
unless they asked (those need eyes, a card, or parked
pins). Do not init NFC or LoRa on Lite. Recipes:
[not-yet-confirmed.md](docs/not-yet-confirmed.md).

## Bluetooth testing options

When testing Bluetooth pairing in firmware (such as `embassy-debug-fw`),
always offer the human the option to test with their own personal
devices (e.g., smartphone). If the host environment has an available,
unblocked Bluetooth controller (e.g., via BlueZ `bluetoothctl`), agents
may also offer an automated host self-diagnostic workflow: initiating
pairing via host Bluetooth, extracting the generated PIN from the active
CDC stream (`pair pin=`), submitting it to complete authentication, and
asking the human to visually confirm that the identical PIN was rendered
on the e-paper panel. This host-driven pathway enables fast agent
self-diagnostics while preserving manual testing flexibility.

## Keep skills updated

Project-local skills must stay aligned with the tree. When you
change a topic below, update the matching skill in the **same
change**. State source conflicts in the hardware skill instead of
flattening them.

| When you change | Also update |
| --- | --- |
| Pin, rail, display, touch, sensor, enclosure, measurement backlog, or datasheet catalog | [m5stack-papermono-hardware](.agents/skills/m5stack-papermono-hardware/SKILL.md) (and a [sources.md](.agents/skills/m5stack-papermono-hardware/references/sources.md) conflict row if sources disagree). `docs/DATASHEETS.md` is a symlink of `resources/datasheets.md`. `docs/not-yet-confirmed.md` is a symlink of `resources/not-yet-confirmed.md` |
| Hardware safety | [safety.md](.agents/skills/m5stack-papermono-hardware/references/safety.md) (`docs/SAFETY.md` is a symlink) **and** this file if it restates a row |
| Agent rules that belong to one directory | that directory’s `AGENTS.md` (nearest file wins on conflict) |
| Host CLI, xtask catalog, or tool verification ledger | [papermono-rs](.agents/skills/papermono-rs/SKILL.md) |
| Board crates under `crates/` | [crates/AGENTS.md](crates/AGENTS.md) **and** the hardware skill pin-map |
| Firmware packages under `firmware/` | [firmware/AGENTS.md](firmware/AGENTS.md) **and** the hardware skill (pins, opcodes, refresh modes) |
| Flash-session packing (one image, many NYC) | this file **and** [papermono-rs](.agents/skills/papermono-rs/SKILL.md) **and** [not-yet-confirmed.md](.agents/skills/m5stack-papermono-hardware/resources/not-yet-confirmed.md) |
| Human how-to (firmware examples, getting-started, firmware READMEs) | those files **and** [Human READMEs](#human-readmes) |

Skills:
[m5stack-papermono-hardware](.agents/skills/m5stack-papermono-hardware/SKILL.md),
[papermono-rs](.agents/skills/papermono-rs/SKILL.md).

## Working rules

- **Named constants, not magic bytes.** Code uses grouped
  `enum` / `const` values. Comments state meaning and
  provenance. Markdown (including rustdoc) prefers those
  titles, not raw hex. Cite datasheets by catalog **Id**,
  **section number** (when the sheet has one), and **section
  title** — not page numbers. Mapping tables (name ↔ encoding
  ↔ source) live in the hardware skill. Board nets: living
  HTML **PinMap**
  ([catalog.md](.agents/skills/m5stack-papermono-hardware/references/catalog.md),
  [pin-map.md](.agents/skills/m5stack-papermono-hardware/references/pin-map.md)).
  EPD call site is `OtpRefresh` (`otp_gray` / `otp_mono` /
  `otp_partial`). `RefreshMode` / `epd_*` is the official
  HTML catalog only. What to do:
  [display.md](.agents/skills/m5stack-papermono-hardware/references/display.md).
  What not to do:
  [docs/SAFETY.md](docs/SAFETY.md).
  When writing under `firmware/` or `crates/`, also read
  [firmware/AGENTS.md](firmware/AGENTS.md) and
  [crates/AGENTS.md](crates/AGENTS.md).
- **Do not invent registers or opcodes.** If the datasheet has
  not been read, expose a documented raw primitive and record the
  gap in the hardware skill catalog
  ([docs/DATASHEETS.md](docs/DATASHEETS.md), a symlink of
  [datasheets.md](.agents/skills/m5stack-papermono-hardware/resources/datasheets.md)).
  Cached PDFs and extracted markdown are gitignored under that
  skill’s `resources/datasheets/`. Ask the user to populate the
  cache (`scripts/fetch_datasheets.py` from the skill directory);
  do not download vendor files unless they asked. Capture safe
  datasheet rows even if unused; leave hazardous encodings
  commented.
- Do not copy foreign pin maps, latch sequences, CH343 VID, 32 MB
  flash rules, or GT911 dances onto this product.
- **Official datasheets and documentation outrank secondary references.**
  In all cases where a statement in crates, firmware, documentation, or
  skills can be matched to an official datasheet section or vendor
  documentation, do so directly instead of citing third-party, community,
  or secondary sources.
- **Exact Store SKUs, no informal aliases.** Always use official
  M5Stack commercial SKUs: `C153` (PaperMono) and `C153-Lite`
  (PaperMono-Lite). Never use informal aliases like `full`,
  `both`, `full-SKU`, or `Pro` in SKU tables, code comments, or
  documentation. In tables with an `SKU` column, list `C153`,
  `C153-Lite`, or `C153, C153-Lite`.
- **No ephemeral or non-specific references.** Never refer to
  ephemeral development phases or milestones ("Stage A",
  "Stage B", "Stage C", "Stage D", "Phase 1"). Always use
  descriptive hardware terms (such as minimal unfeatured bring-up,
  system I2C and touch rails, PDM audio energy sampling, or
  OTP e-paper panel refresh). Never use bare, unexplained
  references like "UserDemo"; always identify the full provenance
  and link the official factory demo firmware
  ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)).
- **Firmware examples as tutorial code.** Firmware under `firmware/`
  (`simple-debug-fw` and `embassy-debug-fw`) must serve as
  educational reference code. Every function, method, struct, enum,
  and constant (public or private) must have comprehensive rustdoc
  explaining what it does, hardware nets/buses involved, expectations,
  and error handling. Include abundant in-line comments explaining
  hardware register sequencing, GPIO electrical configurations
  (pull-ups, input modes), bus arbitration, Embassy task scheduling,
  stack buffer usage, and reset/wake-up cycles. Ground descriptions
  in authoritative terminology from *The Embedded Rust Book*,
  *The Rust on ESP Book*, and *The Embassy Book*.
- Use [Conventional Commits](https://www.conventionalcommits.org/).
- Measurement-backlog items in the hardware skill stay open until
  someone confirms them on a physical PaperMono (`C153`) or
  PaperMono-Lite (`C153-Lite`). Name the SKU. A result on one
  variant does not confirm the other. Firmware evidence proves
  intent and sequencing, never electrical fact. Do not write a
  “Confirmed live” row without a board in hand.
- **All human-facing Markdown must be linted by rumdl.** Every file meant
  to be read by humans across the workspace—including the root `README.md`,
  `docs/*.md`, and all crate, host, and firmware `README.md` files—must be
  included in [`.rumdl.toml`](.rumdl.toml) and pass `rumdl check`. Vendor
  PDF extracts under the hardware skill `resources/datasheets/md/` remain
  excluded.
- Operator how-to (firmware README recipes, getting-started
  command blocks, the root firmware-examples list) is for a
  person at the desk. Numbered steps with human titles; what
  to type, then what they should see, then what to do with
  their hands; pass and fail as observations. Keep live-ask,
  pack-one-flash, envelope, and backlog ids in this file
  and the skills. Do not write those pages as agent notes.

## Human READMEs

`README.md` files and [docs/getting-started.md](docs/getting-started.md)
are for people. Agent contracts stay in `AGENTS.md` files.
When you change what an image does, how to flash it, or the
xtask catalog a human sees, update the matching row in the
**same change**.

| File | Audience | Keep current when |
| --- | --- | --- |
| [README.md](README.md) | humans | firmware-examples list, `docs/assets/*.png` gallery, xtask summary, SKUs, safety / getting-started / NYC / CRATES links |
| [docs/getting-started.md](docs/getting-started.md) | humans | host verify, Xtensa, snapshot, Path A / Path B, troubleshooting, NYC / CRATES |
| [docs/firmware-snapshot-management.md](docs/firmware-snapshot-management.md) | humans | capture, confirm, restore, tool ledger, honest limits |
| [docs/CRATES.md](docs/CRATES.md) | humans + agents | pass / fail / written-here / constants-in-BSP |
| [docs/API-RULES.md](docs/API-RULES.md) | humans + agents | chip-crate C-CTOR / C-FREE / no MCU LUT |
| [firmware/simple-debug/README.md](firmware/simple-debug/README.md) | humans | CDC heartbeat, button check, flash / listen steps |
| [firmware/embassy-debug/README.md](firmware/embassy-debug/README.md) | humans | cards, lamp, splash, flash / listen steps |
| [firmware/embassy-debug/assets/SOURCE.md](firmware/embassy-debug/assets/SOURCE.md) | humans | Ferris attribution and encode |
| [crates/papermono-log/README.md](crates/papermono-log/README.md) | crates.io | host-tested line kinds |
| [crates/ssd1677-otp/README.md](crates/ssd1677-otp/README.md) | crates.io | panel OTP, no MCU LUT |
| [crates/m5pm1/README.md](crates/m5pm1/README.md) | crates.io | PMIC registers + PWM0 |
| [crates/m5ioe1/README.md](crates/m5ioe1/README.md) | crates.io | expander + IP2315 gate |
| [crates/m5stack-papermono-lite/README.md](crates/m5stack-papermono-lite/README.md) | crates.io | Lite / shared pin-map role |
| [crates/m5stack-papermono/README.md](crates/m5stack-papermono/README.md) | crates.io | C153 radio add-on role |
| [host/papermono-host/README.md](host/papermono-host/README.md) | humans | host library, udev, flash contract |
| [datasheets/README.md](.agents/skills/m5stack-papermono-hardware/resources/datasheets/README.md) | humans | local datasheet cache only |

Published crate READMEs are crates.io landing pages. Relative
markdown links there only resolve to files **inside that
crate’s package**. Do not link `../../docs/...`, a sibling
crate, or a skill with a repo-relative path from a crate
README; use an absolute URL into this repository or name the
item in backticks. Relative links remain fine in repo-root
docs, this file, firmware READMEs, and `.agents/skills/`.

## Agent Documentation Standards

Project-local skills exist under `.agents/skills/` and should
remain discoverable by agents working in this repository.
Maintain those skills according to the
[Agent Skills specification](https://agentskills.io/specification),
and maintain this file according to the
[AGENTS.md standard](https://agents.md/). Keep both portable
across compatible agent clients, without assumptions about
user-specific paths or session state.

Skills:

- [m5stack-papermono-hardware](.agents/skills/m5stack-papermono-hardware/SKILL.md)
  — board contract (pins, rails, datasheets, SKU differences,
  source precedence). The skill user weighs conflicts.
- [papermono-rs](.agents/skills/papermono-rs/SKILL.md)
  — `cargo xtask`, USB session lock, tool verification ledger.
- [crates/AGENTS.md](crates/AGENTS.md) — board crates, chip
  crates, and `papermono-log` line format.
- [firmware/AGENTS.md](firmware/AGENTS.md) — named constants,
  datasheet citations, `epd_*` refresh enums, and Xtensa images
  (nearest file wins).

Vendor datasheets are official for registers of chips named on
this model; observed hardware still outranks a datasheet default.
See
[Authority](.agents/skills/m5stack-papermono-hardware/SKILL.md#authority).
PaperMono-Lite (`C153-Lite`) has run- and download-mode USB
IDs (`303a:1001`), a 16 MB flash size, and a partition table matching
the official factory demo firmware
([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))
([flashing.md](.agents/skills/m5stack-papermono-hardware/references/flashing.md#usb-measured),
[measure.md](.agents/skills/m5stack-papermono-hardware/references/measure.md)).
Official HTML `epd_*` times (`epd_quality` / `epd_text` /
`epd_fast` / `epd_fastest`) are PaperMono laboratory
results under M5GFX modes, reference only
([display.md](.agents/skills/m5stack-papermono-hardware/references/display.md)).
PaperMono (`C153`) USB, JEDEC, and partition table are still
unmeasured.
