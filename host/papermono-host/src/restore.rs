//! Restore that unit's original via `write-bin` only.

use std::path::{Path, PathBuf};

use crate::device::DeviceIo;
use crate::identity::LiveIdentity;
use crate::layout::{require_capture_backup, require_original_backup, Layout};
use crate::Error;

/// Restore the full measured image or one named partition.
pub fn restore<D: DeviceIo>(
    device: &D,
    layout: &Layout,
    port: &str,
    yes: bool,
    part: Option<&str>,
    capture: Option<&str>,
) -> Result<(), Error> {
    if !yes {
        return Err(Error::RestoreNotConfirmed);
    }
    let live = live_identity(device, port)?;
    let original = if let Some(slug) = capture {
        require_capture_backup(layout, &live, slug)?
    } else {
        require_original_backup(layout, &live)?
    };
    if original.manifest.flash_size_bytes == 0 {
        return Err(Error::SizeNotMeasured);
    }
    match part {
        None => {
            let image = original.dir.join(&original.manifest.dump_file);
            let image = if image.is_file() {
                image
            } else {
                original.dir.join("flash.bin")
            };
            if !image.is_file() {
                return Err(Error::Import("snapshot missing flash dump".into()));
            }
            if image
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == "flash-32mb.bin")
                && original.manifest.flash_size_bytes != 32 * 1024 * 1024
            {
                return Err(Error::Import(
                    "refusing flash-32mb.bin; snapshot size is not 32 MiB".into(),
                ));
            }
            let bytes = std::fs::metadata(&image)?.len();
            if bytes as usize != original.manifest.flash_size_bytes {
                return Err(Error::DumpLength {
                    got: bytes as usize,
                    expected: original.manifest.flash_size_bytes,
                });
            }
            eprintln!(
                "restore: writing {} ({bytes} bytes) at 0x0 in 1 MiB windows",
                image.display()
            );
            device.write_bin(port, 0, &image)
        }
        Some(label) => {
            let part = original
                .manifest
                .partitions
                .iter()
                .find(|p| p.label == label)
                .ok_or_else(|| Error::UnknownPartition(label.into()))?;
            let image = original.dir.join(format!("part-{label}.bin"));
            if !image.is_file() {
                return Err(Error::UnknownPartition(label.into()));
            }
            let bytes = std::fs::metadata(&image)?.len();
            eprintln!(
                "restore: writing part-{label}.bin ({bytes} bytes) at {:#x}",
                part.offset
            );
            device.write_bin(port, part.offset, &image)
        }
    }
}

fn live_identity<D: DeviceIo>(device: &D, port: &str) -> Result<LiveIdentity, Error> {
    let (_, board) = crate::detect::read_live_board(device, port)?;
    Ok(board.identity)
}

/// Used by tests that restore against a mock without a full-chip file.
pub fn restore_paths(original_dir: &Path, part: Option<&str>) -> Result<(u32, PathBuf), Error> {
    match part {
        None => {
            let manifest = crate::layout::load_manifest(original_dir)?;
            let named = original_dir.join(&manifest.dump_file);
            if named.is_file() {
                Ok((0, named))
            } else {
                Ok((0, original_dir.join("flash.bin")))
            }
        }
        Some(label) => {
            let manifest = crate::layout::load_manifest(original_dir)?;
            let part = manifest
                .partitions
                .iter()
                .find(|p| p.label == label)
                .ok_or_else(|| Error::UnknownPartition(label.into()))?;
            Ok((part.offset, original_dir.join(format!("part-{label}.bin"))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::{backup_live, BackupRequest, UnsealOnDrop};
    use crate::device::MockDevice;
    use crate::identity::{sha256_text, test_mac};
    use crate::partitions::{test_entry, PARTITION_TABLE_OFFSET};
    use std::cell::RefCell;

    fn tiny_dump(len: usize) -> Vec<u8> {
        let nvs_off = 0x9000u32;
        let mut dump = vec![0u8; len.max((nvs_off + 16) as usize)];
        dump[PARTITION_TABLE_OFFSET..PARTITION_TABLE_OFFSET + 32]
            .copy_from_slice(&test_entry("nvs", 0x01, 0x02, nvs_off, 16));
        dump[nvs_off as usize] = 0xEE;
        dump
    }

    #[test]
    fn restore_without_yes_refuses() {
        let tmp = UnsealOnDrop::new();
        let layout = Layout::from_developer_data_root(tmp.path());
        let mock = RefCell::new(MockDevice::default());
        let err = restore(&mock, &layout, "PORT", false, None, None).unwrap_err();
        assert!(matches!(err, Error::RestoreNotConfirmed));
    }

    #[test]
    fn restore_part_nvs_records_write() {
        let tmp = UnsealOnDrop::new();
        let layout = Layout::from_developer_data_root(tmp.path());
        let mac = test_mac();
        let info = format!(
            "Flash size: 16MB\nMAC address: (redacted)\nMAC sha256: {}\n",
            sha256_text(&mac)
        );
        let dump = tiny_dump(16 * 1024 * 1024);
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
        restore(&mock, &layout, "PORT", true, Some("nvs"), None).unwrap();
        let writes = &mock.borrow().writes;
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, 0x9000);
        assert_eq!(writes[0].1[0], 0xEE);
    }
}
