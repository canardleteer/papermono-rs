//! Write a custom application image into the snapshot `factory` partition.

use std::fs;
use std::path::Path;

use crate::device::DeviceIo;
use crate::layout::{require_capture_backup, require_original_backup, Layout};
use crate::Error;

/// Lite measured `factory` start (matching factory demo `partitions.csv`). Nothing
/// below this is an app-flash target (nvs `0x9000`, phy `0xf000`).
pub const FACTORY_MIN_OFFSET: u32 = 0x10000;

/// Partition label on Lite stock.
pub const FACTORY_LABEL: &str = "factory";

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// `write_bin` of `image` at this unit's `factory` offset. Never erase,
/// never the `espflash` `flash` subcommand, never a caller-chosen address.
pub fn flash_app<D: DeviceIo>(
    device: &D,
    layout: &Layout,
    port: &str,
    image: &Path,
    yes: bool,
    capture: Option<&str>,
) -> Result<(), Error> {
    if !yes {
        return Err(Error::FlashNotConfirmed);
    }
    let (_, board) = crate::detect::read_live_board(device, port)?;
    let snapshot = if let Some(slug) = capture {
        require_capture_backup(layout, &board.identity, slug)?
    } else {
        require_original_backup(layout, &board.identity)?
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
        let err = flash_app(&mock, &layout, "PORT", &image, false, None).unwrap_err();
        assert!(matches!(err, Error::FlashNotConfirmed));
    }

    #[test]
    fn flash_writes_factory_only() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let image = write_image(layout.developer_data_root.as_path(), &[0xAA, 0xBB]);
        flash_app(&mock, &layout, "PORT", &image, true, None).unwrap();
        let writes = &mock.borrow().writes;
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, FACTORY_MIN_OFFSET);
        assert_eq!(writes[0].1, vec![0xAA, 0xBB]);
    }

    #[test]
    fn flash_refuses_app0_only_table() {
        let (_tmp, layout, mock) = mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "app0"));
        let image = write_image(layout.developer_data_root.as_path(), &[0xAA]);
        let err = flash_app(&mock, &layout, "PORT", &image, true, None).unwrap_err();
        assert!(matches!(err, Error::UnknownPartition(label) if label == "factory"));
    }

    #[test]
    fn flash_refuses_elf() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let image = write_image(layout.developer_data_root.as_path(), b"\x7fELFnot-an-app");
        let err = flash_app(&mock, &layout, "PORT", &image, true, None).unwrap_err();
        assert!(matches!(err, Error::ImageNotApp));
    }

    #[test]
    fn flash_refuses_image_larger_than_factory() {
        let (_tmp, layout, mock) =
            mock_and_original(dump_with_factory(FACTORY_MIN_OFFSET, "factory"));
        let too_big = vec![0x11; (FACTORY_SIZE as usize) + 1];
        let image = write_image(layout.developer_data_root.as_path(), &too_big);
        let err = flash_app(&mock, &layout, "PORT", &image, true, None).unwrap_err();
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
        let err = flash_app(&mock, &layout, "PORT", &image, true, None).unwrap_err();
        assert!(matches!(err, Error::UnsafeFactoryOffset(0x9000)));
    }
}
