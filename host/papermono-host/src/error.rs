//! Errors from PaperMono host operations.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Recoverable failure. Printed on stderr by the binary.
#[derive(Debug)]
pub enum Error {
    /// Filesystem failure.
    Io(io::Error),
    /// JSON encode/decode failure.
    Json(serde_json::Error),
    /// Serial is empty or not a safe directory name.
    InvalidUnitId(String),
    /// `developer-data/backups/original/{unit-id}/` already exists (write-once).
    OriginalExists(PathBuf),
    /// `developer-data/backups/captures/{unit-id}/{slug}/` already exists.
    CaptureExists(PathBuf),
    /// Leftover repo-root `backups/` must be moved by the operator.
    LegacyBackupsDir(PathBuf),
    /// No original directory matches the live unit.
    MissingOriginal,
    /// `--capture` did not match a snapshot for this unit.
    MissingCapture(String),
    /// More than one capture MANIFEST matches this unit (and no original).
    AmbiguousCapture,
    /// Classification needs `--name` (or a TTY prompt in xtask).
    NeedsSnapshotName {
        /// Why a slug is required.
        evidence: String,
    },
    /// Live identity hashes do not match the snapshot MANIFEST.
    IdentityMismatch {
        /// Why the bind-check failed.
        reason: String,
    },
    /// More than one original MANIFEST matches this unit.
    AmbiguousOriginal,
    /// `board-info` did not report a usable flash size.
    FlashSizeUnknown(String),
    /// Dump length is not the measured full-chip size.
    DumpLength {
        /// Bytes in the dump.
        got: usize,
        /// Bytes expected from board-info / JEDEC.
        expected: usize,
    },
    /// Partition table magic or bounds failed.
    PartitionTable(String),
    /// Live command needs `ESPFLASH_PORT`.
    MissingPort,
    /// Live command needs exactly one PaperMono USB node, or `--port`.
    AmbiguousPaperMonoUsb,
    /// No Espressif `303a:1001` USB-Serial/JTAG found.
    MissingPaperMonoUsb,
    /// `--port` is QinHeng CH343 (`1a86:55d3`), not this product.
    QinHengCh343,
    /// `--port` is a USB-serial device that is not Espressif `303a:1001`.
    NotPaperMonoUsb {
        /// USB vendor id from sysfs, when known.
        vid: Option<u16>,
        /// USB product id from sysfs, when known.
        pid: Option<u16>,
    },
    /// `--port` could not be classified as Espressif USB-Serial/JTAG.
    UnclassifiedUsbPort,
    /// Another xtask already holds the USB session.
    UartBusy {
        /// Holder pid, when the lock file could be read.
        pid: Option<u32>,
        /// Holder command name, when the lock file could be read.
        command: Option<String>,
    },
    /// Restore without `--yes`.
    RestoreNotConfirmed,
    /// `--part` label is not in the snapshot table.
    UnknownPartition(String),
    /// Live dump or restore refused because flash size was never measured.
    SizeNotMeasured,
    /// Import source is unusable.
    Import(String),
    /// The `espflash` library or USB/UART sample failed.
    Device(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::InvalidUnitId(id) => {
                write!(f, "refusing unit id as a directory name: {id:?}")
            }
            Self::OriginalExists(path) => write!(
                f,
                "original already exists (write-once): {}\nrun confirm-factory-firmware to measure drift",
                path.display()
            ),
            Self::CaptureExists(path) => write!(f, "capture already exists: {}", path.display()),
            Self::LegacyBackupsDir(path) => write!(
                f,
                "leftover backups/ at {}; mkdir -p developer-data && mv backups developer-data/backups",
                path.display()
            ),
            Self::MissingOriginal => write!(
                f,
                "no snapshot matches this unit; run cargo xtask backup-factory-firmware first \
                 (originals in developer-data/backups/original/<unit-id>/, captures in \
                 developer-data/backups/captures/<unit-id>/<slug>/)"
            ),
            Self::MissingCapture(slug) => write!(
                f,
                "no developer-data/backups/captures/<unit-id>/{slug}/ matches this unit"
            ),
            Self::AmbiguousCapture => {
                write!(f, "multiple captures match this unit; pass --capture SLUG")
            }
            Self::NeedsSnapshotName { evidence } => write!(
                f,
                "this dump is not a known factory image; pass --name SLUG (or --as-original if it is factory). {evidence}"
            ),
            Self::IdentityMismatch { reason } => write!(f, "identity mismatch: {reason}"),
            Self::AmbiguousOriginal => write!(
                f,
                "multiple originals; pass a by-id port from cargo xtask detect-connected"
            ),
            Self::FlashSizeUnknown(found) => {
                write!(f, "board-info flash size is unusable: {found:?}")
            }
            Self::DumpLength { got, expected } => {
                write!(f, "expected {expected} byte dump, got {got} bytes")
            }
            Self::PartitionTable(reason) => write!(f, "partition table: {reason}"),
            Self::MissingPort => write!(f, "set ESPFLASH_PORT or pass --port"),
            Self::AmbiguousPaperMonoUsb => write!(
                f,
                "multiple Espressif 303a:1001 ports; pass --port or set ESPFLASH_PORT"
            ),
            Self::MissingPaperMonoUsb => write!(
                f,
                "no Espressif USB JTAG/serial debug unit (303a:1001) found; plug in PaperMono / PaperMono-Lite or pass --port"
            ),
            Self::QinHengCh343 => write!(
                f,
                "refusing QinHeng CH343 (1a86:55d3); this product's USB-C is Espressif 303a:1001. Run cargo xtask detect-connected"
            ),
            Self::NotPaperMonoUsb { vid, pid } => match (vid, pid) {
                (Some(vid), Some(pid)) => write!(
                    f,
                    "refusing USB {vid:04x}:{pid:04x}; expected Espressif 303a:1001. Run cargo xtask detect-connected"
                ),
                _ => write!(
                    f,
                    "refusing this serial port; expected Espressif 303a:1001. Run cargo xtask detect-connected"
                ),
            },
            Self::UnclassifiedUsbPort => write!(
                f,
                "could not confirm this port is Espressif 303a:1001; pass a by-id node from cargo xtask detect-connected"
            ),
            Self::UartBusy { pid, command } => {
                write!(
                    f,
                    "refusing to open the USB session: another xtask already holds it"
                )?;
                match (pid, command) {
                    (Some(pid), Some(command)) => write!(f, " ({command}, pid {pid})")?,
                    (Some(pid), None) => write!(f, " (pid {pid})")?,
                    (None, Some(command)) => write!(f, " ({command})")?,
                    (None, None) => {}
                }
                write!(f, ". Wait for that dump, restore, probe, etc. to finish")
            }
            Self::RestoreNotConfirmed => write!(f, "restore refuses to write without --yes"),
            Self::UnknownPartition(label) => write!(f, "no partition labelled {label:?}"),
            Self::SizeNotMeasured => write!(
                f,
                "refusing live dump or restore: flash size is unknown. Run cargo xtask detect-connected --probe first (human ask; prefer download via power-button hold)"
            ),
            Self::Import(reason) => write!(f, "import: {reason}"),
            Self::Device(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
