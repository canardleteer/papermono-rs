//! Write a custom application image into the snapshot `factory` partition.

use std::fs;
use std::path::Path;

use crate::device::DeviceIo;
use crate::layout::{
    list_originals, require_capture_backup, require_original_backup, Layout, Snapshot,
};
use crate::Error;

/// Measured `factory` start shared by the C153 and C153-Lite stock tables.
/// Nothing below is an app-flash target (`nvs` and `phy_init` sit there).
pub const FACTORY_MIN_OFFSET: u32 = 0x10000;

/// Partition label on Lite stock.
pub const FACTORY_LABEL: &str = "factory";

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// `write_bin` of `image` at the selected snapshot's `factory` offset.
///
/// Normally the snapshot must match the live unit identity. `force` allows an
/// original snapshot fallback only when every local original agrees on the
/// factory offset and size. It never permits a capture, unsafe offset, unknown
/// target, ELF, empty image, or image larger than the factory partition.
/// The operation never erases and never accepts a caller-chosen address.
pub fn flash_app<D: DeviceIo>(
    device: &D,
    layout: &Layout,
    port: &str,
    image: &Path,
    yes: bool,
    capture: Option<&str>,
    force: bool,
) -> Result<(), Error> {
    if !yes {
        return Err(Error::FlashNotConfirmed);
    }
    if force && capture.is_some() {
        return Err(Error::ForceWithCapture);
    }
    let (_, board) = crate::detect::read_live_board(device, port)?;
    let snapshot = if let Some(slug) = capture {
        require_capture_backup(layout, &board.identity, slug)?
    } else {
        match require_original_backup(layout, &board.identity) {
            Ok(snapshot) => snapshot,
            Err(Error::MissingOriginal | Error::AmbiguousOriginal) if force => {
                eprintln!(
                    "flash-app: --force ignores unit identity; originals must match live flash size and agree on factory geometry"
                );
                let live_size = board
                    .flash_size_bytes
                    .ok_or_else(|| Error::FlashSizeUnknown(board.flash_size.clone()))?;
                forced_original_with_consistent_geometry(layout, live_size)?
            }
            Err(error) => return Err(error),
        }
    };
    if !snapshot.is_original() {
        eprintln!(
            "flash-app: using capture {} — this is not a factory restore; lost nvs is not recoverable",
            snapshot.dir.display()
        );
    }
    let factory = snapshot
        .manifest
        .partitions
        .iter()
        .find(|part| part.label == FACTORY_LABEL)
        .ok_or_else(|| Error::UnknownPartition(FACTORY_LABEL.into()))?;
    if factory.offset < FACTORY_MIN_OFFSET {
        return Err(Error::UnsafeFactoryOffset(factory.offset));
    }
    let bytes = fs::read(image)?;
    validate_app_image(&bytes, factory.size)?;
    device.write_bin(port, factory.offset, image)
}

/// Select any local original only when each original agrees on the app target.
///
/// `--force` bypasses this unit's identity binding, but it never allows an
/// unknown, conflicting, or unsafe factory partition to determine the write.
fn forced_original_with_consistent_geometry(
    layout: &Layout,
    live_flash_size: usize,
) -> Result<Snapshot, Error> {
    let mut snapshots = list_originals(layout)?.into_iter();
    let first = snapshots.next().ok_or(Error::MissingOriginal)?;
    let expected = checked_factory_geometry(&first, live_flash_size)?;
    for snapshot in snapshots {
        if checked_factory_geometry(&snapshot, live_flash_size)? != expected {
            return Err(Error::InconsistentFactoryGeometry);
        }
    }
    Ok(first)
}

/// Return the app-flash offset and size after validating the snapshot's table.
fn checked_factory_geometry(
    snapshot: &Snapshot,
    live_flash_size: usize,
) -> Result<(u32, u32), Error> {
    if snapshot.manifest.flash_size_bytes != live_flash_size {
        return Err(Error::SnapshotFlashSizeMismatch {
            snapshot: snapshot.manifest.flash_size_bytes,
            live: live_flash_size,
        });
    }
    let factory = snapshot
        .manifest
        .partitions
        .iter()
        .find(|part| part.label == FACTORY_LABEL)
        .ok_or_else(|| Error::UnknownPartition(FACTORY_LABEL.into()))?;
    if factory.offset < FACTORY_MIN_OFFSET {
        return Err(Error::UnsafeFactoryOffset(factory.offset));
    }
    if u64::from(factory.offset) + u64::from(factory.size) > live_flash_size as u64 {
        return Err(Error::FactoryOutOfBounds {
            offset: factory.offset,
            size: factory.size,
            flash_size: live_flash_size,
        });
    }
    Ok((factory.offset, factory.size))
}

fn validate_app_image(bytes: &[u8], factory_size: u32) -> Result<(), Error> {
    if bytes.is_empty() || bytes.starts_with(&ELF_MAGIC) {
        return Err(Error::ImageNotApp);
    }
    let size = bytes.len() as u64;
    if size > u64::from(factory_size) {
        return Err(Error::ImageTooLarge {
            size,
            max: factory_size,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::{backup_live, BackupRequest, UnsealOnDrop};
    use crate::device::MockDevice;
    use crate::identity::{sha256_text, test_mac};
    use crate::partitions::{test_entry, PARTITION_TABLE_OFFSET};
    use std::cell::RefCell;

    const FACTORY_SIZE: u32 = 0x1000;

    fn dump_with_factory(factory_off: u32, factory_label: &str) -> Vec<u8> {
        let nvs_off = 0x9000u32;
        let mut dump = vec![0u8; 16 * 1024 * 1024];
        dump[PARTITION_TABLE_OFFSET..PARTITION_TABLE_OFFSET + 32]
            .copy_from_slice(&test_entry("nvs", 0x01, 0x02, nvs_off, 16));
        dump[PARTITION_TABLE_OFFSET + 32..PARTITION_TABLE_OFFSET + 64].copy_from_slice(
            &test_entry(factory_label, 0x00, 0x00, factory_off, FACTORY_SIZE),
        );
        dump
    }

    fn mock_and_original(dump: Vec<u8>) -> (UnsealOnDrop, Layout, RefCell<MockDevice>) {
        let tmp = UnsealOnDrop::new();
        let layout = Layout::from_developer_data_root(tmp.path());
        let mac = test_mac();
        let info = format!(
            "Flash size: 16MB\nMAC address: (redacted)\nMAC sha256: {}\n",
            sha256_text(&mac)
        );
        let mock = RefCell::new(MockDevice {
            board_info: info,
            flash: dump,
            ..MockDevice::default()
        });
        backup_live(
            &mock,
            &layout,
            "PORT",
            &BackupRequest {
                name: None,
                as_original: true,
            },
            |_| Ok(None),
        )
        .unwrap();
        (tmp, layout, mock)
    }

    fn write_image(dir: &std::path::Path, bytes: &[u8]) -> std::path::PathBuf {
        let path = dir.join("app.bin");
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn flash_without_yes_refuses() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let image = write_image(layout.developer_data_root.as_path(), &[0xAA, 0xBB]);
        let err = flash_app(&mock, &layout, "PORT", &image, false, None, false).unwrap_err();
        assert!(matches!(err, Error::FlashNotConfirmed));
    }

    #[test]
    fn flash_writes_factory_only() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let image = write_image(layout.developer_data_root.as_path(), &[0xAA, 0xBB]);
        flash_app(&mock, &layout, "PORT", &image, true, None, false).unwrap();
        let writes = &mock.borrow().writes;
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, FACTORY_MIN_OFFSET);
        assert_eq!(writes[0].1, vec![0xAA, 0xBB]);
    }

    #[test]
    fn force_allows_identity_mismatch_but_keeps_factory_target() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let other_mac = "10:20:30:40:50:61";
        mock.borrow_mut().board_info = format!(
            "Flash size: 16MB\nMAC address: (redacted)\nMAC sha256: {}\n",
            sha256_text(other_mac)
        );
        let image = write_image(layout.developer_data_root.as_path(), &[0xAA, 0xBB]);

        let err = flash_app(&mock, &layout, "PORT", &image, true, None, false).unwrap_err();
        assert!(matches!(err, Error::MissingOriginal));

        flash_app(&mock, &layout, "PORT", &image, true, None, true).unwrap();
        let writes = &mock.borrow().writes;
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, FACTORY_MIN_OFFSET);
        assert_eq!(writes[0].1, vec![0xAA, 0xBB]);
    }

    #[test]
    fn force_accepts_multiple_originals_with_same_geometry() {
        let (_tmp, layout, _mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let first = crate::layout::list_originals(&layout)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let mut manifest = serde_json::to_value(first.manifest).unwrap();
        manifest["unit_id"] = serde_json::Value::String("other-original".into());
        let second_dir = layout.original_dir("other-original");
        std::fs::create_dir(&second_dir).unwrap();
        std::fs::write(
            second_dir.join("MANIFEST.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let selected = forced_original_with_consistent_geometry(&layout, 16 * 1024 * 1024).unwrap();
        assert_eq!(
            checked_factory_geometry(&selected, 16 * 1024 * 1024).unwrap(),
            (FACTORY_MIN_OFFSET, FACTORY_SIZE)
        );
    }

    #[test]
    fn force_refuses_conflicting_factory_geometry() {
        let (_tmp, layout, _mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let first = crate::layout::list_originals(&layout)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let mut manifest = serde_json::to_value(first.manifest).unwrap();
        manifest["unit_id"] = serde_json::Value::String("other-original".into());
        let factory = manifest["partitions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|partition| partition["label"] == "factory")
            .unwrap();
        factory["offset"] = serde_json::json!(FACTORY_MIN_OFFSET + 0x1000);
        let second_dir = layout.original_dir("other-original");
        std::fs::create_dir(&second_dir).unwrap();
        std::fs::write(
            second_dir.join("MANIFEST.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let err = forced_original_with_consistent_geometry(&layout, 16 * 1024 * 1024).unwrap_err();
        assert!(matches!(err, Error::InconsistentFactoryGeometry));
    }

    #[test]
    fn force_requires_snapshot_flash_size_to_match_live_size() {
        let (_tmp, layout, _mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let err = forced_original_with_consistent_geometry(&layout, 8 * 1024 * 1024).unwrap_err();
        assert!(matches!(
            err,
            Error::SnapshotFlashSizeMismatch {
                snapshot: 16_777_216,
                live: 8_388_608
            }
        ));
    }

    #[test]
    fn force_cannot_be_combined_with_capture() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let image = write_image(layout.developer_data_root.as_path(), &[0xAA]);
        let err = flash_app(&mock, &layout, "PORT", &image, true, Some("named"), true).unwrap_err();
        assert!(matches!(err, Error::ForceWithCapture));
        assert!(mock.borrow().writes.is_empty());
    }

    #[test]
    fn flash_refuses_app0_only_table() {
        let (_tmp, layout, mock) = mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "app0"));
        let image = write_image(layout.developer_data_root.as_path(), &[0xAA]);
        let err = flash_app(&mock, &layout, "PORT", &image, true, None, false).unwrap_err();
        assert!(matches!(err, Error::UnknownPartition(label) if label == "factory"));
    }

    #[test]
    fn flash_refuses_elf() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let image = write_image(layout.developer_data_root.as_path(), b"\x7fELFnot-an-app");
        let err = flash_app(&mock, &layout, "PORT", &image, true, None, false).unwrap_err();
        assert!(matches!(err, Error::ImageNotApp));
    }

    #[test]
    fn flash_refuses_image_larger_than_factory() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let too_big = vec![0x11; (FACTORY_SIZE as usize) + 1];
        let image = write_image(layout.developer_data_root.as_path(), &too_big);
        let err = flash_app(&mock, &layout, "PORT", &image, true, None, false).unwrap_err();
        assert!(matches!(
            err,
            Error::ImageTooLarge {
                size,
                max: FACTORY_SIZE
            } if size == u64::from(FACTORY_SIZE) + 1
        ));
    }

    #[test]
    fn flash_refuses_factory_below_min_offset() {
        let (_tmp, layout, mock) = mock_and_original(dump_with_factory(0x9000, "factory"));
        let image = write_image(layout.developer_data_root.as_path(), &[0xAA]);
        let err = flash_app(&mock, &layout, "PORT", &image, true, None, false).unwrap_err();
        assert!(matches!(err, Error::UnsafeFactoryOffset(0x9000)));
    }
}
