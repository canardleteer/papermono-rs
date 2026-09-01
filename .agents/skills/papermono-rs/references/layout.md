# Repository layout and host conventions

## Paths

| Path | Contents |
| --- | --- |
| `host/` | Default-members. Host libraries (not `xtask`) |
| `host/papermono-host/` | Host library. Detect, backup / confirm / restore, monitor |
| `xtask/` | Clap front-end (`cargo xtask`) |
| `developer-data/` | Gitignored. Snapshots under `backups/` |
| `docs/` | [SAFETY.md](../../../../docs/SAFETY.md) (symlink), [firmware-snapshot-management.md](../../../../docs/firmware-snapshot-management.md), [DATASHEETS.md](../../../../docs/DATASHEETS.md) (symlink) |
| `.agents/skills/m5stack-papermono-hardware/` | Board contract |
| `.agents/skills/papermono-rs/` | This skill |

Firmware packages are not workspace members yet. When they land,
keep them out of `default-members`.

## Working rules (this repository)

- Host CLIs (`xtask` and any future CLI) use
  [`clap`](https://docs.rs/clap) **derive** (`Parser`, `Subcommand`,
  `Args`). Do not put clap types in `papermono-host`.
  `hide_env_values` on `ESPFLASH_PORT`.
- Device I/O lives in `papermono-host` via the
  [`espflash`](https://crates.io/crates/espflash) **library**
  (`default-features = false`, `serialport`). Do not enable
  espflash's `cli` feature. `papermono-host` and xtask require rustc
  1.88 (espflash 4.5).
- Live `papermono-host` methods take
  [`uart_lock::try_acquire`](xtask.md#usb-session-lock). New
  USB-touching in-repo tools reuse that same lock.
- Accept Espressif `303a:1001`. Refuse QinHeng `1a86:55d3`.
- Dump length comes from live `board-info` / JEDEC. Lite measured
  16 MB (`0x1000000`). Do not hardcode 32 MB.
- Verify with `cargo test --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`, and
  `cargo fmt --check`. `cargo xtask ci` is the full host gate.
  Do not advertise `cargo test --workspace` once firmware members
  exist. Owned Markdown is `rumdl check`.
- One workspace lockfile is committed. Pass `--locked`.
- Use [Conventional Commits](https://www.conventionalcommits.org/).
