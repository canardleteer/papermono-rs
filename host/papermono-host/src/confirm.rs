//! Compare live flash to a write-once snapshot; do not modify the snapshot.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::device::DeviceIo;
use crate::dump::sha256_hex;
use crate::layout::{require_capture_backup, require_original_backup, Layout, Snapshot};
use crate::partitions::{BOOTLOADER_LEN, PARTITION_TABLE_LEN, PARTITION_TABLE_OFFSET};
use crate::Error;

/// One region compared during confirm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionDiff {
    /// `bootloader`, `partition-table`, or a partition label.
    pub name: String,
    /// Whether SHA-256 matches the original slice.
    pub matches: bool,
    /// Original SHA-256 hex.
    pub original_sha256: String,
    /// Live SHA-256 hex.
    pub live_sha256: String,
}

/// Confirm report written under `developer-data/confirm-records/<unit-id>/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceReport {
    /// Local unit id of the snapshot.
    pub unit_id: String,
    /// Unix timestamp when the comparison ran.
    pub compared_at_unix: u64,
    /// Whether the full dump hash matches.
    pub dump_sha256_match: Option<bool>,
    /// Per-region comparison.
    pub regions: Vec<RegionDiff>,
}

/// Read live flash, compare to the matching original (or `--capture`).
pub fn confirm_live<D: DeviceIo>(
    device: &D,
    layout: &Layout,
    port: &str,
    capture: Option<&str>,
) -> Result<DivergenceReport, Error> {
    let (_, board) = crate::detect::read_live_board(device, port)?;
    let snapshot = if let Some(slug) = capture {
        require_capture_backup(layout, &board.identity, slug)?
    } else {
        require_original_backup(layout, &board.identity)?
    };
    let expected = snapshot.manifest.flash_size_bytes;
    let live = device.read_flash(port, 0, expected as u32)?;
    let original_dump = fs::read(snapshot.dir.join(&snapshot.manifest.dump_file))
        .or_else(|_| fs::read(snapshot.dir.join("flash.bin")))?;
    let report = compare_dumps(&snapshot, &original_dump, &live)?;
    write_report(layout, &report)?;
    Ok(report)
}

/// Compare two dumps using the original's partition table.
pub fn compare_dumps(
    original: &Snapshot,
    original_dump: &[u8],
    live_dump: &[u8],
) -> Result<DivergenceReport, Error> {
    let mut regions = Vec::new();
    push_region(
        &mut regions,
        "bootloader",
        slice(original_dump, 0, BOOTLOADER_LEN),
        slice(live_dump, 0, BOOTLOADER_LEN),
    );
    push_region(
        &mut regions,
        "partition-table",
        slice(original_dump, PARTITION_TABLE_OFFSET, PARTITION_TABLE_LEN),
        slice(live_dump, PARTITION_TABLE_OFFSET, PARTITION_TABLE_LEN),
    );
    for part in &original.manifest.partitions {
        let len = part.size as usize;
        let off = part.offset as usize;
        push_region(
            &mut regions,
            &part.label,
            slice(original_dump, off, len),
            slice(live_dump, off, len),
        );
    }
    let dump_sha256_match = if original_dump.len() == live_dump.len() {
        Some(sha256_hex(original_dump) == sha256_hex(live_dump))
    } else {
        None
    };
    Ok(DivergenceReport {
        unit_id: original.manifest.unit_id.clone(),
        compared_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        dump_sha256_match,
        regions,
    })
}

fn slice(dump: &[u8], offset: usize, len: usize) -> &[u8] {
    let end = offset.saturating_add(len).min(dump.len());
    if offset >= dump.len() {
        &[]
    } else {
        &dump[offset..end]
    }
}

fn push_region(regions: &mut Vec<RegionDiff>, name: &str, original: &[u8], live: &[u8]) {
    regions.push(RegionDiff {
        name: name.to_string(),
        matches: original == live,
        original_sha256: sha256_hex(original),
        live_sha256: sha256_hex(live),
    });
}

fn write_report(layout: &Layout, report: &DivergenceReport) -> Result<(), Error> {
    let dir = layout.confirm_records_dir(&report.unit_id);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("divergence-{}.json", report.compared_at_unix));
    fs::write(path, serde_json::to_string_pretty(report)? + "\n")?;
    Ok(())
}
