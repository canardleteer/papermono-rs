//! Snapshot `MANIFEST.json`.

use serde::{Deserialize, Serialize};

use crate::identity::LiveIdentity;
use crate::partitions::{AppDesc, Partition};

/// Schema id written on new snapshots.
pub const MANIFEST_SCHEMA: &str = "papermono-firmware-snapshot/v1";

fn default_schema() -> String {
    MANIFEST_SCHEMA.to_string()
}

/// Whether this tree is a factory original or a named capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
    /// Human-confirmed factory (`developer-data/backups/original/`).
    #[default]
    Original,
    /// Named “what is on the chip now”.
    Capture,
}

/// Provenance and hashes for a snapshot directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Document identity.
    #[serde(default = "default_schema")]
    pub schema: String,
    /// Original vs named capture.
    #[serde(default)]
    pub kind: SnapshotKind,
    /// Classification tag (`uncertain_stock`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<String>,
    /// Capture slug when [`SnapshotKind::Capture`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_name: Option<String>,
    /// Local directory id (`lite-<last4>` or `id-<hash>`). Not a MAC.
    pub unit_id: String,
    /// SHA-256 of the USB iSerial when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usb_serial_sha256: Option<String>,
    /// SHA-256 of the station MAC when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mac_sha256: Option<String>,
    /// Raw flash-size field.
    pub flash_size: String,
    /// Full-chip length in bytes.
    pub flash_size_bytes: usize,
    /// Secure boot reported enabled.
    pub secure_boot: bool,
    /// Flash encryption reported enabled.
    pub flash_encryption: bool,
    /// SHA-256 of the full-chip dump file.
    pub dump_sha256: String,
    /// On-disk dump file name (`flash-16mb.bin` or `flash.bin`).
    pub dump_file: String,
    /// SHA-256 of the bootloader slice.
    pub bootloader_sha256: String,
    /// SHA-256 of the partition table slice.
    pub partition_table_sha256: String,
    /// Active OTA slot if `otadata` was present.
    pub boot_slot: Option<String>,
    /// App descriptor from `app0` if present.
    pub app0_desc: Option<AppDesc>,
    /// Partition table as parsed from the dump.
    pub partitions: Vec<Partition>,
    /// SHA-256 of each `part-{label}.bin`.
    pub partition_sha256: Vec<PartitionHash>,
}

impl Manifest {
    /// Identity hashes used for bind-check.
    #[must_use]
    pub fn identity(&self) -> LiveIdentity {
        LiveIdentity {
            mac_sha256: self.mac_sha256.clone(),
            usb_serial_sha256: self.usb_serial_sha256.clone(),
        }
    }
}

/// Hash of one named slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionHash {
    /// Partition label.
    pub label: String,
    /// Hex SHA-256.
    pub sha256: String,
}
