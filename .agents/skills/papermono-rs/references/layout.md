# Repository layout and host conventions

## Paths

| Path | Contents |
| --- | --- |
| `crates/` | Default-members. Host-testable board crates |
| `crates/m5stack-papermono-lite/` | `C153-Lite`. Shared pin map for both SKUs |
| `crates/m5stack-papermono/` | `C153`. Re-exports Lite; adds NFC + LoRa |
| `host/` | Default-members. Host libraries (not `xtask`) |
| `host/papermono-host/` | Host library. Detect, backup / confirm / restore, monitor |
| `host/papermono-host/udev/` | usbfs udev rule for `monitor` |
| `xtask/` | Clap front-end (`cargo xtask`) |
| `developer-data/` | Gitignored. Snapshots under `backups/`; confirm JSON under `confirm-records/` |
| `docs/` | [SAFETY.md](../../../../docs/SAFETY.md) (symlink), [firmware-snapshot-management.md](../../../../docs/firmware-snapshot-management.md), [DATASHEETS.md](../../../../docs/DATASHEETS.md) (symlink) |
| `firmware/` | Not members yet. Planned `simple-debug` / `embassy-debug`. [firmware/AGENTS.md](../../../../firmware/AGENTS.md) |
| `.agents/skills/m5stack-papermono-hardware/` | Board contract |
| `.agents/skills/papermono-rs/` | This skill |

Chip drivers (when they land) stay MCU-agnostic and live under
`crates/`. Board specifics belong in the two SKU crates. Lite
firmware depends on `m5stack-papermono-lite` only.

Firmware packages are not workspace members yet. When they land,
keep them out of `default-members`. Nearest rules:
[firmware/AGENTS.md](../../../../firmware/AGENTS.md).

## Working rules (this repository)

- Host CLIs (`xtask` and any future CLI) use
  [`clap`](https://docs.rs/clap) **derive** (`Parser`, `Subcommand`,
  `Args`). Do not put clap types in `papermono-host`.
  `hide_env_values` on `ESPFLASH_PORT`.
- Device I/O lives in `papermono-host` via the
  [`espflash`](https://crates.io/crates/espflash) **library**
  (`default-features = false`, `serialport`). Do not enable
  espflash's `cli` feature. **MSRV is split on purpose:**
  workspace `package.rust-version` is **1.85** (board crates).
  `papermono-host` and `xtask` set
  **1.88** because espflash 4.5 requires it.
- Live `papermono-host` methods take
  [`uart_lock::try_acquire`](xtask.md#usb-session-lock). New
  USB-touching in-repo tools reuse that same lock.
- Accept Espressif `303a:1001`. Refuse QinHeng `1a86:55d3`.
  DevKits often use the same VID:PID; pass `--port` when more
  than one Espressif USB-Serial/JTAG is plugged in.
- Dump length comes from live `board-info` / JEDEC. Lite measured
  16 MB (`0x1000000`). Do not hardcode 32 MB.
- Verify with `cargo test --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`, and
  `cargo fmt --check`. `cargo xtask ci` is the full host gate.
  Do not advertise `cargo test --workspace` once firmware members
  exist. Owned Markdown is `rumdl check`.
- One workspace lockfile is committed. Pass `--locked`. After
  changing `crates/*/Cargo.toml`, `host/papermono-host/Cargo.toml`,
  `xtask/Cargo.toml`, or workspace members, refresh it with
  `cargo generate-lockfile`.
- Named `enum` / `const` values, not magic bytes. Comments
  state meaning and source (datasheet Id + section number and
  title; HTML **PinMap** for nets). Markdown prefers those
  titles. Refresh modes:
  [display.md](../../m5stack-papermono-hardware/references/display.md).
- Use [Conventional Commits](https://www.conventionalcommits.org/).
