# Repository layout and host conventions

## Paths

| Path | Contents |
| --- | --- |
| `crates/` | Default-members. Host-testable board and chip crates |
| `crates/papermono-log/` | Host-tested USB-Serial/JTAG line format for both images |
| `crates/ssd1677-otp/` | Panel OTP sequences. No MCU LUT |
| `crates/m5pm1/` | PMIC registers + PWM0 |
| `crates/m5ioe1/` | Expander banks + IP2315 gate typestate |
| `crates/m5stack-papermono-lite/` | `C153-Lite`. Shared pin map for both SKUs |
| `crates/m5stack-papermono/` | `C153`. Re-exports Lite; adds NFC + LoRa |
| `host/` | Default-members. Host libraries (not `xtask`) |
| `host/papermono-host/` | Host library. Detect, backup / confirm / restore, `build-fw`, `flash-app`, monitor |
| `host/papermono-host/udev/` | usbfs udev rule for `monitor` |
| `xtask/` | Clap front-end (`cargo xtask`) |
| `developer-data/` | Gitignored. Snapshots under `backups/`; confirm JSON under `confirm-records/` |
| `docs/` | [getting-started.md](../../../../docs/getting-started.md), [firmware-snapshot-management.md](../../../../docs/firmware-snapshot-management.md), [CRATES.md](../../../../docs/CRATES.md), [SAFETY.md](../../../../docs/SAFETY.md) (symlink), [DATASHEETS.md](../../../../docs/DATASHEETS.md) (symlink), [not-yet-confirmed.md](../../../../docs/not-yet-confirmed.md) (symlink), [assets/first-ferris.png](../../../../docs/assets/first-ferris.png) |
| `firmware/simple-debug/` | Workspace member, not a default-member. [firmware/AGENTS.md](../../../../firmware/AGENTS.md) |
| `firmware/embassy-debug/` | Workspace member, not a default-member. Embassy staged image |
| `firmware/embassy-debug/assets/` | Splash Ferris: line-art SVG, 1bpp + invert, encode script. [SOURCE.md](../../../../firmware/embassy-debug/assets/SOURCE.md) |
| `rust-analyzer.toml` | Excludes `simple-debug-fw` and `embassy-debug-fw` from host check |
| `.cargo/config.toml` | `xtask` alias and Xtensa `-Tlinkall.x`. No Cargo `runner` |
| `.agents/skills/m5stack-papermono-hardware/` | Board contract |
| `.agents/skills/papermono-rs/` | This skill |

Chip drivers stay MCU-agnostic under `crates/`
(`docs/API-RULES.md`). Board specifics belong in the two SKU
crates. Lite
firmware depends on `m5stack-papermono-lite` only. SKU split
(two crates, not a feature flag):
[crates/AGENTS.md](../../../../crates/AGENTS.md).

`simple-debug-fw` and `embassy-debug-fw` are workspace members,
not default-members. Nearest rules:
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
  `cargo fmt --check`. `cargo xtask ci` is the full gate (host
  trio, firmware `cargo +esp` clippy, rumdl, machete, audit).
  Needs the esp toolchain. Do not advertise
  `cargo test --workspace` (that pulls Xtensa firmware).
  rust-analyzer excludes `simple-debug-fw` and
  `embassy-debug-fw` via
  [rust-analyzer.toml](../../../../rust-analyzer.toml). Owned
  Markdown is `rumdl check`.
- One workspace lockfile is committed. Pass `--locked`. After
  changing `crates/*/Cargo.toml`, `host/papermono-host/Cargo.toml`,
  `xtask/Cargo.toml`, `firmware/*/Cargo.toml`, or workspace
  members, refresh it with `cargo generate-lockfile`.
- Host-only `cargo xtask build-fw` wraps `cargo +esp` and
  `espflash save-image --flash-size 16mb` (no port) for a
  `flash-app` payload. `flash-app` writes snapshot `factory`
  only (Lite `0x10000`).
- Named `enum` / `const` values, not magic bytes. Comments
  state meaning and source (datasheet Id + section number and
  title; HTML **PinMap** for nets). Markdown prefers those
  titles. Refresh modes:
  [display.md](../../m5stack-papermono-hardware/references/display.md).
- Use [Conventional Commits](https://www.conventionalcommits.org/).
