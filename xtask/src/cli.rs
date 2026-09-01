//! Clap derive CLI for PaperMono host xtask.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use papermono_host::{
    backup_import, backup_live, confirm_live, detect_connected, load_manifest, monitor,
    refuse_if_legacy_backups_at_repo_root, restore, BackupRequest, Error, Layout, MonitorOptions,
    SnapshotKind,
};

/// PaperMono host CLI: USB inventory, snapshots, and host-only CI.
#[derive(Debug, Parser)]
#[command(name = "xtask", version, long_about = LONG_ABOUT)]
pub struct Cli {
    /// Subcommand.
    #[command(subcommand)]
    pub command: Command,
}

const LONG_ABOUT: &str = "\
Live commands need `--port` or `ESPFLASH_PORT`, or exactly one Espressif \
USB-Serial/JTAG (`303a:1001`). They take an exclusive USB session lock. \
Do not open a port unless a human asked.

QinHeng CH343 (`1a86:55d3`) is refused. Download is a power-button hold \
(~2 s until the red LED blinks), not DTR. Snapshots live under gitignored \
`developer-data/backups/`. Never erase-flash. Never `espflash flash`.

Host-only (no USB): `detect-connected` without `--probe`, `backup --import`, \
`ci`, and `vet-idle-log`.

Use `<COMMAND> --help` for flags.";

const CI_ABOUT: &str = "\
Host-only CI gate: `cargo fmt --check --all`; host clippy and test on \
default-members (default features, then `--all-features`); then `rumdl \
check`, `cargo machete`, and `cargo audit`.

No firmware clippy until a firmware package exists. Does not open a USB \
session. Do not pass `--workspace` once Xtensa members exist.";

/// Host operations.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// List PaperMono USB-Serial/JTAG nodes (no port unless `--probe`)
    DetectConnected(DetectArgs),
    /// Dump this unit: `--as-original` → `original/`, else a named capture
    #[command(visible_alias = "backup-firmware")]
    BackupFactoryFirmware(BackupArgs),
    /// Compare live flash to the matching original or `--capture`
    ConfirmFactoryFirmware(ConfirmArgs),
    /// Write-bin this unit's original or `--capture` (needs `--yes`)
    RestoreFactoryFirmware(RestoreArgs),
    /// Stub: no firmware idle-token grammar yet
    VetIdleLog,
    /// Host-only CI gate (fmt, clippy, test, extra tools)
    #[command(long_about = CI_ABOUT)]
    Ci,
    /// Read USB-Serial/JTAG at 115200 (no DTR download)
    Monitor(MonitorArgs),
}

/// USB inventory (default) or flasher probe.
#[derive(Debug, Args)]
pub struct DetectArgs {
    /// Open USB-Serial/JTAG for board-info (NoReset). Prefer download via
    /// power-button hold. Needs exactly one `303a:1001`, or `--port`.
    #[arg(long)]
    pub probe: bool,
    /// Serial device. Also `ESPFLASH_PORT`. Used only with `--probe`.
    #[arg(long, env = "ESPFLASH_PORT", hide_env_values = true)]
    pub port: Option<String>,
    /// Also list USB-serial nodes that are not Espressif `303a:1001`.
    #[arg(long)]
    pub all_devices: bool,
}

/// Backup flags.
#[derive(Debug, Args)]
pub struct BackupArgs {
    /// Serial device. Also `ESPFLASH_PORT`.
    #[arg(long, env = "ESPFLASH_PORT", hide_env_values = true)]
    pub port: Option<String>,
    /// Host-only: copy an existing dump tree (needs `board-info.txt`).
    #[arg(long, value_name = "DIR")]
    pub import: Option<PathBuf>,
    /// Named capture slug (`developer-data/backups/captures/<unit-id>/<slug>/`).
    #[arg(long)]
    pub name: Option<String>,
    /// Store an uncertain-stock dump under `original/` (this unit only).
    #[arg(long)]
    pub as_original: bool,
}

/// Live confirm.
#[derive(Debug, Args)]
pub struct ConfirmArgs {
    /// Serial device. Also `ESPFLASH_PORT`.
    #[arg(long, env = "ESPFLASH_PORT", hide_env_values = true)]
    pub port: Option<String>,
    /// Compare against this capture slug instead of `original/`.
    #[arg(long, value_name = "SLUG")]
    pub capture: Option<String>,
}

/// USB-Serial/JTAG listen.
#[derive(Debug, Args)]
pub struct MonitorArgs {
    /// Serial device. Also `ESPFLASH_PORT`.
    #[arg(long, env = "ESPFLASH_PORT", hide_env_values = true)]
    pub port: Option<String>,
    /// Stop after this many seconds (from port open).
    #[arg(long = "for", value_name = "SECS", value_parser = clap::value_parser!(u64).range(1..))]
    pub for_secs: Option<u64>,
    /// Stop after this many newline-terminated device lines.
    #[arg(short = 'n', long, value_name = "N", value_parser = clap::value_parser!(u64).range(1..))]
    pub lines: Option<u64>,
    /// Write a copy of the stream to FILE (still prints unless `--quiet`).
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    /// Do not print to stdout. Requires `--output`.
    #[arg(long, visible_alias = "output-only", requires = "output")]
    pub quiet: bool,
}

/// Restore flags.
#[derive(Debug, Args)]
pub struct RestoreArgs {
    /// Serial device. Also `ESPFLASH_PORT`.
    #[arg(long, env = "ESPFLASH_PORT", hide_env_values = true)]
    pub port: Option<String>,
    /// Required. Restore writes flash.
    #[arg(long)]
    pub yes: bool,
    /// Restore one partition (`nvs`, `app0`, …) instead of the full image.
    #[arg(long)]
    pub part: Option<String>,
    /// Restore this capture slug instead of `original/`.
    #[arg(long, value_name = "SLUG")]
    pub capture: Option<String>,
}

impl Cli {
    /// Parse argv and run.
    pub fn exec() -> ExitCode {
        match Self::parse().run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        }
    }

    /// Run against the repo's `developer-data/backups/`.
    pub fn run(self) -> Result<(), Error> {
        let repo = repo_root();
        if matches!(self.command, Command::Ci) {
            return crate::ci::run(&repo);
        }
        refuse_if_legacy_backups_at_repo_root(&repo)?;
        let layout = Layout::from_repo_root(repo);
        match self.command {
            Command::Ci => unreachable!("ci returns before leftover-backup refuse"),
            Command::DetectConnected(args) => {
                detect_connected(args.probe, args.port, args.all_devices)
            }
            Command::BackupFactoryFirmware(args) => run_backup(&layout, args),
            Command::ConfirmFactoryFirmware(args) => {
                let report = confirm_live(&layout, args.port, args.capture.as_deref())?;
                let drifted: Vec<_> = report
                    .regions
                    .iter()
                    .filter(|r| !r.matches)
                    .map(|r| r.name.as_str())
                    .collect();
                if drifted.is_empty() {
                    println!("confirm: {} matches original", report.unit_id);
                } else {
                    println!(
                        "confirm: {} drifted in {}",
                        report.unit_id,
                        drifted.join(", ")
                    );
                }
                Ok(())
            }
            Command::RestoreFactoryFirmware(args) => {
                restore(
                    &layout,
                    args.port,
                    args.yes,
                    args.part.as_deref(),
                    args.capture.as_deref(),
                )?;
                println!("restore write-bin finished");
                Ok(())
            }
            Command::VetIdleLog => Err(Error::Device(
                "vet-idle-log: no image grammar (no papermono-rs firmware yet)".into(),
            )),
            Command::Monitor(args) => monitor(
                args.port,
                &MonitorOptions {
                    for_secs: args.for_secs,
                    lines: args.lines,
                    output: args.output,
                    quiet: args.quiet,
                },
            ),
        }
    }
}

fn run_backup(layout: &Layout, args: BackupArgs) -> Result<(), Error> {
    let request = BackupRequest {
        name: args.name,
        as_original: args.as_original,
    };
    let dest = if let Some(source) = args.import {
        backup_import(layout, &source, &request, prompt_snapshot_name)?
    } else {
        backup_live(layout, args.port, &request, prompt_snapshot_name)?
    };
    let manifest = load_manifest(&dest)?;
    let kind = match manifest.kind {
        SnapshotKind::Original => "original",
        SnapshotKind::Capture => "capture",
    };
    println!(
        "wrote {kind} {} unit={} sha256={} ({} bytes)",
        dest.display(),
        manifest.unit_id,
        manifest.dump_sha256,
        manifest.flash_size_bytes,
    );
    Ok(())
}

fn prompt_snapshot_name(evidence: &str) -> Result<Option<String>, Error> {
    use std::io::{self, IsTerminal, Write};

    eprintln!("{evidence}");
    if !io::stdin().is_terminal() {
        return Ok(None);
    }
    eprint!("name this snapshot (directory-safe): ");
    let _ = io::stderr().flush();
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let name = line.trim();
    if name.is_empty() {
        Ok(None)
    } else {
        Ok(Some(name.to_string()))
    }
}

/// Repository root (parent of the `xtask` package).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn clap_debug_assert() {
        Cli::command().debug_assert();
    }

    #[test]
    fn short_about_fits_one_line() {
        let about = Cli::command().get_about().expect("Cli about").to_string();
        assert!(
            about.len() <= 72,
            "about is {} chars (keep it one terminal line):\n{about}",
            about.len()
        );
    }

    #[test]
    fn backup_accepts_name_and_as_original() {
        use clap::Parser;

        let cli = Cli::try_parse_from([
            "xtask",
            "backup-firmware",
            "--name",
            "stock-lite",
            "--as-original",
        ])
        .expect("backup-firmware alias");
        match cli.command {
            super::Command::BackupFactoryFirmware(args) => {
                assert_eq!(args.name.as_deref(), Some("stock-lite"));
                assert!(args.as_original);
            }
            other => panic!("expected BackupFactoryFirmware, got {other:?}"),
        }
    }

    #[test]
    fn ci_parses() {
        use clap::Parser;

        let cli = Cli::try_parse_from(["xtask", "ci"]).expect("ci");
        match cli.command {
            super::Command::Ci => {}
            other => panic!("expected Ci, got {other:?}"),
        }
    }

    #[test]
    fn monitor_accepts_for_lines_and_quiet_output() {
        use clap::Parser;

        let cli = Cli::try_parse_from([
            "xtask", "monitor", "--for", "12", "--lines", "40", "--output", "uart.log", "--quiet",
        ])
        .expect("monitor listen budget");
        match cli.command {
            super::Command::Monitor(args) => {
                assert_eq!(args.for_secs, Some(12));
                assert_eq!(args.lines, Some(40));
                assert!(args.quiet);
            }
            other => panic!("expected Monitor, got {other:?}"),
        }
    }

    #[test]
    fn monitor_quiet_requires_output() {
        use clap::Parser;

        assert!(Cli::try_parse_from(["xtask", "monitor", "--quiet"]).is_err());
    }

    #[test]
    fn restore_needs_yes_flag_in_help() {
        use clap::Parser;

        let cli = Cli::try_parse_from(["xtask", "restore-factory-firmware", "--yes"])
            .expect("restore parses");
        match cli.command {
            super::Command::RestoreFactoryFirmware(args) => assert!(args.yes),
            other => panic!("expected Restore, got {other:?}"),
        }
    }
}
