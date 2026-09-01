//! Programmatic host API for per-unit PaperMono / PaperMono-Lite flash
//! originals.
//!
//! `cargo xtask` and a later standalone CLI both call these methods. Do
//! not open a USB session unless a human explicitly asked.

#![allow(missing_docs)]

pub mod backup;
pub mod build_fw;
pub mod cdc_listen;
pub mod confirm;
pub mod detect;
pub mod device;
pub mod dump;
pub mod error;
pub mod flash_app;
pub mod identity;
pub mod layout;
pub mod learn_uart;
pub mod manifest;
pub mod monitor;
pub mod partitions;
pub mod restore;
pub mod uart_lock;

use std::path::{Path, PathBuf};

pub use backup::BackupRequest;
pub use build_fw::{build_fw, BuildFwArgs, BuildFwOutput, FirmwareImage};
pub use confirm::DivergenceReport;
pub use device::{DeviceIo, RealDevice};
pub use error::Error;
pub use identity::OFFICIAL_FLASH_SIZE;
pub use layout::{load_manifest, refuse_if_legacy_backups_at_repo_root, Layout};
pub use learn_uart::{vet_idle_log, VetIdleLogArgs};
pub use manifest::SnapshotKind;
pub use monitor::MonitorOptions;
pub use uart_lock::{try_acquire, UartSession, UART_LOCK_ENV};

/// Chunk size for `espflash` library reads.
pub const CHUNK_SIZE: usize = 1024 * 1024;

/// USB inventory, optionally `--probe`.
///
/// Inventory without probe does not take the USB lock. Probe does.
pub fn detect_connected(probe: bool, port: Option<String>, all_devices: bool) -> Result<(), Error> {
    detect::run(&RealDevice, probe, port, all_devices)
}

/// Live capture: board-info, chunked read sized from JEDEC, write-once dir.
pub fn backup_live<F>(
    layout: &Layout,
    port: Option<String>,
    request: &BackupRequest,
    ask_name: F,
) -> Result<PathBuf, Error>
where
    F: FnOnce(&str) -> Result<Option<String>, Error>,
{
    let port = detect::resolve_papermono_port(port)?;
    let _uart = uart_lock::try_acquire(&port, "backup-factory-firmware")?;
    backup::backup_live(&RealDevice, layout, &port, request, ask_name)
}

/// Host-only copy of an already-taken dump tree. Still write-once.
pub fn backup_import<F>(
    layout: &Layout,
    source: &Path,
    request: &BackupRequest,
    ask_name: F,
) -> Result<PathBuf, Error>
where
    F: FnOnce(&str) -> Result<Option<String>, Error>,
{
    backup::backup_import(layout, source, request, ask_name)
}

/// Read live flash, compare to the matching original (or `--capture`).
pub fn confirm_live(
    layout: &Layout,
    port: Option<String>,
    capture: Option<&str>,
) -> Result<DivergenceReport, Error> {
    let port = detect::resolve_papermono_port(port)?;
    let _uart = uart_lock::try_acquire(&port, "confirm-factory-firmware")?;
    confirm::confirm_live(&RealDevice, layout, &port, capture)
}

/// Restore that unit's original (or `--capture`) via `write-bin` only.
pub fn restore(
    layout: &Layout,
    port: Option<String>,
    yes: bool,
    part: Option<&str>,
    capture: Option<&str>,
) -> Result<(), Error> {
    let port = detect::resolve_papermono_port(port)?;
    let _uart = uart_lock::try_acquire(&port, "restore-factory-firmware")?;
    restore::restore(&RealDevice, layout, &port, yes, part, capture)
}

/// Write-bin a custom `save-image` payload into snapshot `factory`.
pub fn flash_app(
    layout: &Layout,
    port: Option<String>,
    image: &Path,
    yes: bool,
    capture: Option<&str>,
) -> Result<(), Error> {
    let port = detect::resolve_papermono_port(port)?;
    let _uart = uart_lock::try_acquire(&port, "flash-app")?;
    flash_app::flash_app(&RealDevice, layout, &port, image, yes, capture)
}

/// Copy USB-Serial/JTAG to stdout (and optionally a file).
pub fn monitor(port: Option<String>, options: &MonitorOptions) -> Result<(), Error> {
    let port = detect::resolve_papermono_port(port)?;
    let _uart = uart_lock::try_acquire(&port, "monitor")?;
    monitor::monitor(&port, options)
}
