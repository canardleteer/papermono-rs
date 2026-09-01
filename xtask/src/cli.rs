//! Clap derive CLI for PaperMono host xtask.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use papermono_host::{
    backup_import, backup_live, build_fw, confirm_live, detect_connected, flash_app, load_manifest,
    monitor, refuse_if_legacy_backups_at_repo_root, restore, vet_idle_log, BackupRequest,
    BuildFwArgs, Error, FirmwareImage, Layout, MonitorOptions, SnapshotKind, VetIdleLogArgs,
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
`ci`, `vet-idle-log`, and `build-fw`. There is no `learn-uart` command.

Use `<COMMAND> --help` for flags.";

const CI_ABOUT: &str = "\
Host-only CI gate: `cargo fmt --check --all`; host clippy and test on \
default-members (default features, then `--all-features`); firmware \
`cargo +esp clippy -p simple-debug-fw` and `-p embassy-debug-fw` \
(default, `--no-default-features`, then `touch` / `mic` / `panel` / \
`touch,radio` / `touch,sleep` with `--no-default-features`); then \
`rumdl check`, \
`cargo machete`, \
and `cargo audit`.

Needs the esp toolchain (`espup`, often `$HOME/export-esp.sh`). Does \
not open a USB session. Do not pass `--workspace` (Xtensa members).";

const BUILD_FW_ABOUT: &str = "\
Host-only. `cargo +esp` build (`--profile release-fw`, \
`xtensa-esp32s3-none-elf`, `-Zbuild-std=core,alloc`) then \
`espflash save-image --chip esp32s3 --flash-size 16mb` into \
`target/xtensa-esp32s3-none-elf/release-fw/`.

IMAGE is `simple-debug` or `embassy-debug`. `cargo +esp` uses
`--locked`. Does not open USB.";

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
    /// Write-bin a custom image into snapshot `factory` (needs `--yes`)
    FlashApp(FlashAppArgs),
    /// Host-only: idle-grammar check of a CDC capture
    VetIdleLog(VetIdleLogCliArgs),
    /// Xtensa build plus host-only `save-image`
    #[command(long_about = BUILD_FW_ABOUT)]
    BuildFw(BuildFwCliArgs),
    /// Host-only CI gate (fmt, clippy, test, extra tools)
    #[command(long_about = CI_ABOUT)]
    Ci,
    /// Read USB-Serial/JTAG at 115200 (`--reset` does not recapture)
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
    /// Do not use: Lite DTR/RTS core reset left CDC silent
    ///
    /// `C153-Lite` 2026-09-01: `UsbJtagSerialReset` then 25 s listen
    /// produced 0 CDC bytes. USB stayed `303a:1001`; ACM did not
    /// reappear. Follow-up listen without this flag was also silent
    /// until a short-press red. Not download. Not `--after
    /// watchdog-reset`. After `flash-app` still short-press red.
    #[arg(long)]
    pub reset: bool,
}

/// Snapshot `factory`-only custom image write.
#[derive(Debug, Args)]
pub struct FlashAppArgs {
    /// Serial device. Also `ESPFLASH_PORT`.
    #[arg(long, env = "ESPFLASH_PORT", hide_env_values = true)]
    pub port: Option<String>,
    /// Application flash payload from `espflash save-image` (not an ELF).
    #[arg(long, value_name = "FILE")]
    pub image: PathBuf,
    /// Required. flash-app writes `factory`.
    #[arg(long)]
    pub yes: bool,
    /// Use this capture slug instead of `original/`.
    #[arg(long, value_name = "SLUG")]
    pub capture: Option<String>,
}

/// Host-only idle-grammar check.
#[derive(Debug, Args)]
pub struct VetIdleLogCliArgs {
    /// Capture file (`idle-simple.log` is gitignored).
    #[arg(long, value_name = "FILE")]
    pub input: PathBuf,
    /// Expected `hello image=` (`simple-debug` or `embassy-debug`).
    #[arg(long, default_value = "simple-debug")]
    pub image: String,
    /// Expected `hello sku=`.
    #[arg(long, default_value = "C153-Lite")]
    pub sku: String,
    /// Grammar-check a busy capture (`edge` / pressed buttons).
    #[arg(long)]
    pub allow_activity: bool,
}

/// Host-only firmware image name (`firmware/<name>`).
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum FirmwareImageArg {
    /// Blocking `esp-hal` proof-of-life.
    #[value(name = "simple-debug")]
    SimpleDebug,
    /// Embassy staged image (default `touch` + `panel`).
    #[value(name = "embassy-debug")]
    EmbassyDebug,
}

impl From<FirmwareImageArg> for FirmwareImage {
    fn from(value: FirmwareImageArg) -> Self {
        match value {
            FirmwareImageArg::SimpleDebug => Self::SimpleDebug,
            FirmwareImageArg::EmbassyDebug => Self::EmbassyDebug,
        }
    }
}

/// Host-only Xtensa build + `save-image`.
#[derive(Debug, Args)]
pub struct BuildFwCliArgs {
    /// `simple-debug` or `embassy-debug`.
    pub image: FirmwareImageArg,
    /// Cargo features on that package (`mic` / `radio` / `sleep` on embassy-debug).
    #[arg(long)]
    pub features: Vec<String>,
    /// Build the debug profile instead of `--profile release-fw`.
    #[arg(long)]
    pub debug: bool,
    /// `cargo --no-default-features` (Stage A, or Stage B with
    /// `--features touch`).
    #[arg(long)]
    pub no_default_features: bool,
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
        match self.command {
            Command::Ci => crate::ci::run(&repo),
            Command::BuildFw(args) => {
                let out = build_fw(
                    &repo,
                    &BuildFwArgs {
                        image: args.image.into(),
                        features: args.features,
                        release: !args.debug,
                        no_default_features: args.no_default_features,
                    },
                )?;
                println!("elf {}", out.elf.display());
                println!("bin {}", out.bin.display());
                Ok(())
            }
            Command::VetIdleLog(args) => vet_idle_log(
                &args.input,
                VetIdleLogArgs {
                    image: &args.image,
                    sku: &args.sku,
                    allow_activity: args.allow_activity,
                },
            ),
            command => {
                refuse_if_legacy_backups_at_repo_root(&repo)?;
                let layout = Layout::from_repo_root(repo);
                match command {
                    Command::Ci | Command::BuildFw(_) | Command::VetIdleLog(_) => {
                        unreachable!(
                            "ci, build-fw, and vet-idle-log return before leftover-backup refuse"
                        )
                    }
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
                    Command::FlashApp(args) => {
                        flash_app(
                            &layout,
                            args.port,
                            &args.image,
                            args.yes,
                            args.capture.as_deref(),
                        )?;
                        println!("flash-app write-bin factory finished");
                        Ok(())
                    }
                    Command::Monitor(args) => monitor(
                        args.port,
                        &MonitorOptions {
                            for_secs: args.for_secs,
                            lines: args.lines,
                            output: args.output,
                            quiet: args.quiet,
                            reset: args.reset,
                        },
                    ),
                }
            }
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
                assert!(!args.reset);
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
    fn monitor_accepts_reset() {
        use clap::Parser;

        let cli = Cli::try_parse_from(["xtask", "monitor", "--reset", "--for", "25"])
            .expect("monitor --reset");
        match cli.command {
            super::Command::Monitor(args) => {
                assert!(args.reset);
                assert_eq!(args.for_secs, Some(25));
            }
            other => panic!("expected Monitor, got {other:?}"),
        }
    }

    #[test]
    fn monitor_help_says_reset_does_not_recapture() {
        let mut cmd = Cli::command();
        let monitor = cmd.find_subcommand_mut("monitor").expect("monitor");
        let reset = monitor
            .get_arguments()
            .find(|arg| arg.get_long() == Some("reset"))
            .expect("--reset");
        let short = reset.get_help().expect("short help").to_string();
        assert!(
            short.contains("Do not use") && short.contains("CDC silent"),
            "{short}"
        );
        let long = reset.get_long_help().expect("long help").to_string();
        assert!(
            long.contains("0 CDC bytes") && long.contains("short-press red"),
            "{long}"
        );
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

    #[test]
    fn flash_app_parses_image_yes_and_capture() {
        use clap::Parser;

        let cli = Cli::try_parse_from([
            "xtask",
            "flash-app",
            "--image",
            "target/xtensa-esp32s3-none-elf/release-fw/simple-debug.bin",
            "--yes",
            "--capture",
            "stock-lite",
        ])
        .expect("flash-app");
        match cli.command {
            super::Command::FlashApp(args) => {
                assert!(args.yes);
                assert_eq!(args.capture.as_deref(), Some("stock-lite"));
            }
            other => panic!("expected FlashApp, got {other:?}"),
        }
    }

    #[test]
    fn build_fw_parses_simple_debug() {
        use clap::Parser;

        let cli = Cli::try_parse_from(["xtask", "build-fw", "simple-debug"]).expect("build-fw");
        match cli.command {
            super::Command::BuildFw(args) => {
                assert!(!args.debug);
                assert!(args.features.is_empty());
                assert!(!args.no_default_features);
            }
            other => panic!("expected BuildFw, got {other:?}"),
        }
    }

    #[test]
    fn vet_idle_log_parse() {
        use clap::Parser;

        let vet = Cli::try_parse_from([
            "xtask",
            "vet-idle-log",
            "--input",
            "idle-simple.log",
            "--image",
            "embassy-debug",
            "--allow-activity",
        ])
        .expect("vet-idle-log");
        match vet.command {
            super::Command::VetIdleLog(args) => {
                assert_eq!(args.image, "embassy-debug");
                assert!(args.allow_activity);
            }
            other => panic!("expected VetIdleLog, got {other:?}"),
        }
    }

    #[test]
    fn build_fw_parses_embassy_debug() {
        use clap::Parser;

        let cli = Cli::try_parse_from(["xtask", "build-fw", "embassy-debug"]).expect("embassy");
        match cli.command {
            super::Command::BuildFw(args) => {
                assert!(args.features.is_empty());
                assert!(!args.no_default_features);
            }
            other => panic!("expected BuildFw, got {other:?}"),
        }

        let stage_b = Cli::try_parse_from([
            "xtask",
            "build-fw",
            "embassy-debug",
            "--no-default-features",
            "--features",
            "touch",
        ])
        .expect("stage-b");
        match stage_b.command {
            super::Command::BuildFw(args) => {
                assert_eq!(args.features, ["touch"]);
                assert!(args.no_default_features);
            }
            other => panic!("expected BuildFw, got {other:?}"),
        }
    }
}
