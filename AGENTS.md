# papermono-rs

Board contract for the M5Stack PaperMono and PaperMono-Lite.
Firmware packages and host tools are not in this repository yet.

## Hardware safety (read before writing code)

1. **OTP waveforms first.** Do not invent an MCU `0x32` LUT.
   Custom external waveforms must stay DC-balanced. After about
   ten partial refreshes, do a full refresh. Uninterrupted
   partials can damage the panel.
2. **Park IP2315 off the system I2C bus** except for the charge
   transaction (M5IOE1 `PYG11_PWM3`). Leaving it mounted can hang
   the bus, especially at low VBAT.
3. **Do not copy Sticky GPIO45/46 latch code.** Those pins are
   PDM CLK/DAT here. Power is the M5PM1 button.
4. **GPIO0 and GPIO3 are strapping pins.** GPIO0 is M5PM1
   `BOOT_OUT`; GPIO3 is KEY2.
5. **Download mode is a power-button hold** (~2 s until the red
   LED blinks), not DTR on a CH343.
6. **Snapshot first if you care about that unit’s PHY.** ESP32-S3
   RF calibration lives in NVS. Do not invent Sticky’s `0x90000`
   geometry. M5Stack publishes factory-restore images; that is
   not a license to skip a snapshot.

Full hazard table: [docs/SAFETY.md](docs/SAFETY.md). Board facts
and source precedence:
[m5stack-papermono-hardware](.agents/skills/m5stack-papermono-hardware/SKILL.md).
`docs/SAFETY.md` and [docs/DATASHEETS.md](docs/DATASHEETS.md) are
symlinks into that skill (`references/safety.md`,
`resources/datasheets.md`). Edit the skill files; rumdl lints
those pages and excludes the symlinks.

## Layout

This tree is hardware-docs first. A Cargo workspace, `firmware/`,
and `cargo xtask` are not here yet. Some directories may later
have a topical `AGENTS.md`; the nearest file wins on conflict
with this one.

## Do not connect to a physical device

This repository has **no in-repo flash or UART tool yet**.
Landing skill source is not permission to open a port. Do not
run `espflash`, `esptool`, `idf.py flash`, PlatformIO upload,
`probe-rs`, or any serial monitor against hardware unless the
human **explicitly asked to run** that live command on a device
in that message.

A device may be attached for unrelated reasons; ignore it. Never
commit a MAC address, serial number, USB serial string, NVS blob,
or flash image. `developer-data/` is gitignored on purpose.

## Keep skills updated

Project-local skills must stay aligned with the tree. When you
change a topic below, update the matching skill in the **same
change**. State source conflicts in the hardware skill instead of
flattening them.

| When you change | Also update |
| --- | --- |
| Pin, rail, display, touch, sensor, enclosure, measurement backlog, or datasheet catalog | [m5stack-papermono-hardware](.agents/skills/m5stack-papermono-hardware/SKILL.md) (and a [sources.md](.agents/skills/m5stack-papermono-hardware/references/sources.md) conflict row if sources disagree). `docs/DATASHEETS.md` is a symlink of `resources/datasheets.md` |
| Hardware safety | [safety.md](.agents/skills/m5stack-papermono-hardware/references/safety.md) (`docs/SAFETY.md` is a symlink) **and** this file if it restates a row |
| Agent rules that belong to one directory | that directory’s `AGENTS.md` (nearest file wins on conflict) |

One skill for now:
[m5stack-papermono-hardware](.agents/skills/m5stack-papermono-hardware/SKILL.md).

## Working rules

- **Do not invent registers or opcodes.** If the datasheet has
  not been read, expose a documented raw primitive and record the
  gap in the hardware skill catalog
  ([docs/DATASHEETS.md](docs/DATASHEETS.md), a symlink of
  [datasheets.md](.agents/skills/m5stack-papermono-hardware/resources/datasheets.md)).
  Cached PDFs and extracted markdown are gitignored under that
  skill’s `resources/datasheets/`. Ask the user to populate the
  cache (`scripts/fetch_datasheets.py` from the skill directory);
  do not download vendor files unless they asked.
- Prefer a named `enum` or `const` over a magic number. Prefer
  the vendor datasheet’s name when the sheet has one. Cite extract
  **heading titles**, not page numbers. Capture safe datasheet
  rows even if unused; leave hazardous encodings commented.
- Do not copy Sticky pin maps, latch sequences, CH343 VID, 32 MB
  flash rules, or GT911 dances onto this product.
- Use [Conventional Commits](https://www.conventionalcommits.org/).
- Measurement-backlog items in the hardware skill stay open until
  someone confirms them on a physical PaperMono (`C153`) or
  PaperMono-Lite (`C153-Lite`). Name the SKU. A result on one
  variant does not confirm the other. Firmware evidence proves
  intent and sequencing, never electrical fact. Do not write a
  “Confirmed live” row without a board in hand.
- Owned Markdown is checked with `rumdl check` (config
  [`.rumdl.toml`](.rumdl.toml)). Do not run rumdl on vendor PDF
  extracts under the hardware skill `resources/datasheets/md/`.

## Agent Documentation Standards

Project-local skills exist under `.agents/skills/` and should
remain discoverable by agents working in this repository.
Maintain those skills according to the
[Agent Skills specification](https://agentskills.io/specification),
and maintain this file according to the
[AGENTS.md standard](https://agents.md/). Keep both portable
across compatible agent clients, without assumptions about
user-specific paths or session state.

One skill:

- [m5stack-papermono-hardware](.agents/skills/m5stack-papermono-hardware/SKILL.md)
  — board contract (pins, rails, datasheets, SKU differences,
  source precedence). The skill user weighs conflicts.

Vendor datasheets are official for registers of chips named on
this model; observed hardware still outranks a datasheet default.
See
[Authority](.agents/skills/m5stack-papermono-hardware/SKILL.md#authority).
PaperMono (`C153`) has not been measured. PaperMono-Lite
(`C153-Lite`) has a run-mode USB ID in the hardware skill
([flashing.md](.agents/skills/m5stack-papermono-hardware/references/flashing.md#usb-measured)).
