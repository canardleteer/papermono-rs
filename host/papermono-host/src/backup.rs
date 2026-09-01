//! Write-once factory originals and named captures.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::device::DeviceIo;
use crate::dump::{dump_file_name, require_full_dump, sha256_hex, split_image};
use crate::identity::{unit_id, validate_unit_id, BoardInfo};
use crate::layout::{refuse_if_capture_exists, refuse_if_original_exists, Layout};
use crate::manifest::{Manifest, PartitionHash, SnapshotKind, MANIFEST_SCHEMA};
use crate::{Error, CHUNK_SIZE};

/// Flags for a backup (live dump or `--import`).
#[derive(Debug, Clone, Default)]
pub struct BackupRequest {
    /// Capture slug. Required unless [`Self::as_original`].
    pub name: Option<String>,
    /// Store under `original/` (this unit only; no in-repo factory catalog).
    pub as_original: bool,
}

/// Live capture: board-info, chunked read sized from JEDEC, write-once dir.
pub fn backup_live<D, F>(
    device: &D,
    layout: &Layout,
    port: &str,
    request: &BackupRequest,
    ask_name: F,
) -> Result<PathBuf, Error>
where
    D: DeviceIo,
    F: FnOnce(&str) -> Result<Option<String>, Error>,
{
    let (info_text, board) = crate::detect::read_live_board(device, port)?;
    let expected = board.flash_size_bytes.ok_or(Error::SizeNotMeasured)?;
    let dump = device.read_flash(port, 0, expected as u32)?;
    require_full_dump(&dump, expected)?;
    persist_classified(layout, &dump, &board, &info_text, request, ask_name)
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
    let info_text = read_optional_text(source, "board-info.txt").unwrap_or_default();
    if info_text.is_empty() {
        return Err(Error::Import(
            "board-info.txt missing; cannot bind flash size".into(),
        ));
    }
    let mut board = crate::identity::parse_board_info(&info_text)?;
    if let Ok(manifest) = crate::layout::load_manifest(source) {
        if board.identity.mac_sha256.is_none() {
            board.identity.mac_sha256 = manifest.mac_sha256;
        }
        if board.identity.usb_serial_sha256.is_none() {
            board.identity.usb_serial_sha256 = manifest.usb_serial_sha256;
        }
        if board.flash_size_bytes.is_none() {
            board.flash_size_bytes = Some(manifest.flash_size_bytes);
        }
    }
    let expected = board.flash_size_bytes.ok_or(Error::SizeNotMeasured)?;
    let dump = read_import_dump(source, expected)?;
    persist_classified(layout, &dump, &board, &info_text, request, ask_name)
}

fn read_import_dump(source: &Path, expected: usize) -> Result<Vec<u8>, Error> {
    let named = dump_file_name(expected);
    let candidates = [
        source.join("flash.bin"),
        source.join(&named),
        source.join("flash-16mb.bin"),
        source.join("flash-32mb.bin"),
    ];
    for path in candidates {
        if !path.is_file() {
            continue;
        }
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "flash-32mb.bin" && expected != 32 * 1024 * 1024 {
            return Err(Error::Import(
                "refusing flash-32mb.bin; measured size is not 32 MiB".into(),
            ));
        }
        let dump = fs::read(&path).map_err(|error| {
            Error::Import(format!("failed to read {}: {error}", path.display()))
        })?;
        require_full_dump(&dump, expected)?;
        return Ok(dump);
    }
    Err(Error::Import(format!(
        "no flash.bin or {named} in {}",
        source.display()
    )))
}

fn read_optional_text(dir: &Path, name: &str) -> Option<String> {
    fs::read_to_string(dir.join(name)).ok()
}

fn persist_classified<F>(
    layout: &Layout,
    dump: &[u8],
    board: &BoardInfo,
    board_info_text: &str,
    request: &BackupRequest,
    ask_name: F,
) -> Result<PathBuf, Error>
where
    F: FnOnce(&str) -> Result<Option<String>, Error>,
{
    let expected = board.flash_size_bytes.ok_or(Error::SizeNotMeasured)?;
    require_full_dump(dump, expected)?;
    let evidence = "uncertain stock: no in-repo factory catalog; Lite snapshot is this unit only";
    eprintln!("backup: {evidence}");
    let dest = decide_dest(request, &board.identity, None, evidence, ask_name)?;
    persist_tree(layout, dest, dump, board, board_info_text)
}

enum Dest {
    Original { unit_id: String },
    Capture { unit_id: String, slug: String },
}

fn decide_dest<F>(
    request: &BackupRequest,
    identity: &crate::identity::LiveIdentity,
    usb_serial: Option<&str>,
    evidence: &str,
    ask_name: F,
) -> Result<Dest, Error>
where
    F: FnOnce(&str) -> Result<Option<String>, Error>,
{
    let unit = unit_id(identity, usb_serial)?;
    if request.as_original {
        return Ok(Dest::Original { unit_id: unit });
    }
    let slug = resolve_name(request, evidence, ask_name)?;
    Ok(Dest::Capture {
        unit_id: unit,
        slug,
    })
}

fn resolve_name<F>(request: &BackupRequest, evidence: &str, ask_name: F) -> Result<String, Error>
where
    F: FnOnce(&str) -> Result<Option<String>, Error>,
{
    if let Some(name) = request.name.as_deref() {
        validate_unit_id(name)?;
        return Ok(name.to_string());
    }
    match ask_name(evidence)? {
        Some(name) => {
            validate_unit_id(&name)?;
            Ok(name)
        }
        None => Err(Error::NeedsSnapshotName {
            evidence: evidence.to_string(),
        }),
    }
}

fn persist_tree(
    layout: &Layout,
    dest: Dest,
    dump: &[u8],
    board: &BoardInfo,
    board_info_text: &str,
) -> Result<PathBuf, Error> {
    let expected = board.flash_size_bytes.ok_or(Error::SizeNotMeasured)?;
    require_full_dump(dump, expected)?;
    let split = split_image(dump)?;
    let dump_file = dump_file_name(dump.len());
    let (final_dir, partial, kind, unit, image_name) = match &dest {
        Dest::Original { unit_id } => {
            refuse_if_original_exists(layout, unit_id)?;
            let dest_dir = layout.original_dir(unit_id);
            let partial = layout.originals_dir().join(format!("{unit_id}.partial"));
            (
                dest_dir,
                partial,
                SnapshotKind::Original,
                unit_id.clone(),
                None,
            )
        }
        Dest::Capture { unit_id, slug } => {
            refuse_if_capture_exists(layout, unit_id, slug)?;
            let dest_dir = layout.capture_dir(unit_id, slug);
            let partial = layout
                .captures_dir()
                .join(unit_id)
                .join(format!("{slug}.partial"));
            (
                dest_dir,
                partial,
                SnapshotKind::Capture,
                unit_id.clone(),
                Some(slug.clone()),
            )
        }
    };
    if partial.exists() {
        fs::remove_dir_all(&partial)?;
    }
    if let Some(parent) = partial.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir_all(&partial)?;
    fs::write(partial.join("flash.bin"), dump)?;
    fs::write(partial.join(&dump_file), dump)?;
    if dump.len().is_multiple_of(CHUNK_SIZE) {
        let chunks = partial.join("chunks");
        fs::create_dir_all(&chunks)?;
        for (index, chunk) in dump.chunks(CHUNK_SIZE).enumerate() {
            fs::write(chunks.join(format!("{index:02}.bin")), chunk)?;
        }
    }
    fs::write(partial.join("bootloader.bin"), &split.bootloader)?;
    fs::write(partial.join("partition-table.bin"), &split.table)?;
    let mut partition_sha256 = Vec::new();
    for (part, data) in &split.parts {
        fs::write(partial.join(format!("part-{}.bin", part.label)), data)?;
        partition_sha256.push(PartitionHash {
            label: part.label.clone(),
            sha256: sha256_hex(data),
        });
    }
    fs::write(
        partial.join("board-info.txt"),
        redact_board_info(board_info_text),
    )?;
    let manifest = Manifest {
        schema: MANIFEST_SCHEMA.into(),
        kind,
        classification: Some("uncertain_stock".into()),
        image_name,
        unit_id: unit,
        usb_serial_sha256: board.identity.usb_serial_sha256.clone(),
        mac_sha256: board.identity.mac_sha256.clone(),
        flash_size: board.flash_size.clone(),
        flash_size_bytes: expected,
        secure_boot: board.secure_boot,
        flash_encryption: board.flash_encryption,
        dump_sha256: sha256_hex(dump),
        dump_file,
        bootloader_sha256: sha256_hex(&split.bootloader),
        partition_table_sha256: sha256_hex(&split.table),
        boot_slot: split.boot_slot,
        app0_desc: split.app0_desc,
        partitions: split.partitions,
        partition_sha256,
    };
    fs::write(
        partial.join("MANIFEST.json"),
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;
    write_sha256sums(&partial, dump, &manifest)?;
    if final_dir.exists() {
        return Err(match dest {
            Dest::Original { .. } => Error::OriginalExists(final_dir),
            Dest::Capture { .. } => Error::CaptureExists(final_dir),
        });
    }
    fs::rename(&partial, &final_dir)?;
    seal_tree(&final_dir)?;
    Ok(final_dir)
}

fn redact_board_info(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("MAC address:") && !trimmed.contains("(redacted)") {
            out.push_str("MAC address:       (redacted)\n");
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn write_sha256sums(dir: &Path, dump: &[u8], manifest: &Manifest) -> Result<(), Error> {
    let mut lines = vec![format!("{}  {}", sha256_hex(dump), manifest.dump_file)];
    if manifest.dump_file != "flash.bin" {
        lines.push(format!("{}  flash.bin", sha256_hex(dump)));
    }
    fs::write(dir.join("SHA256SUMS"), lines.join("\n") + "\n")?;
    Ok(())
}

fn seal_tree(dir: &Path) -> Result<(), Error> {
    for entry in walkdir(dir)? {
        let meta = fs::metadata(&entry)?;
        let mut perms = meta.permissions();
        if meta.is_dir() {
            perms.set_mode(0o555);
        } else {
            perms.set_mode(0o444);
        }
        fs::set_permissions(&entry, perms)?;
    }
    Ok(())
}

fn walkdir(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut out = vec![dir.to_path_buf()];
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                out.extend(walkdir(&path)?);
            } else {
                out.push(path);
            }
        }
    }
    Ok(out)
}

/// Test helper: seal-on-drop temp dir that stays writable for cleanup.
pub struct UnsealOnDrop {
    dir: tempfile::TempDir,
}

impl UnsealOnDrop {
    /// New temp developer-data root.
    #[must_use]
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().expect("tempdir"),
        }
    }

    /// Path of the temp root.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

impl Default for UnsealOnDrop {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for UnsealOnDrop {
    fn drop(&mut self) {
        let _ = unseal(self.dir.path());
    }
}

fn unseal(dir: &Path) -> Result<(), Error> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in walkdir(dir)? {
        let meta = fs::metadata(&entry)?;
        let mut perms = meta.permissions();
        perms.set_mode(if meta.is_dir() { 0o755 } else { 0o644 });
        let _ = fs::set_permissions(&entry, perms);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{parse_board_info, sha256_text, test_mac};
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
    fn live_backup_needs_name_or_as_original() {
        let tmp = UnsealOnDrop::new();
        let layout = Layout::from_developer_data_root(tmp.path());
        let mac = test_mac();
        let info = format!(
            "Flash size: 16MB\nMAC address: {mac}\nMAC sha256: {}\n",
            sha256_text(&mac)
        );
        let dump = tiny_dump(16 * 1024 * 1024);
        let mock = RefCell::new(crate::device::MockDevice {
            board_info: info,
            flash: dump,
            ..Default::default()
        });
        let err = backup_live(&mock, &layout, "PORT", &BackupRequest::default(), |_| {
            Ok(None)
        })
        .unwrap_err();
        assert!(matches!(err, Error::NeedsSnapshotName { .. }));
    }

    #[test]
    fn as_original_is_write_once() {
        let tmp = UnsealOnDrop::new();
        let layout = Layout::from_developer_data_root(tmp.path());
        let mac = test_mac();
        let info = format!(
            "Flash size: 16MB\nMAC address: (redacted)\nMAC sha256: {}\n",
            sha256_text(&mac)
        );
        let dump = tiny_dump(16 * 1024 * 1024);
        let mock = RefCell::new(crate::device::MockDevice {
            board_info: info.clone(),
            flash: dump.clone(),
            ..Default::default()
        });
        let req = BackupRequest {
            name: None,
            as_original: true,
        };
        let dest = backup_live(&mock, &layout, "PORT", &req, |_| Ok(None)).unwrap();
        assert!(dest.join("flash-16mb.bin").is_file());
        assert!(dest.join("MANIFEST.json").is_file());
        let err = backup_live(&mock, &layout, "PORT", &req, |_| Ok(None)).unwrap_err();
        assert!(matches!(err, Error::OriginalExists(_)));
        let _ = parse_board_info(&info);
    }

    #[test]
    fn import_refuses_32mb_name_when_size_is_16() {
        let tmp = UnsealOnDrop::new();
        let source = tmp.path().join("src");
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("board-info.txt"),
            "Flash size: 16MB\nMAC address: (redacted)\n",
        )
        .unwrap();
        fs::write(source.join("flash-32mb.bin"), vec![0u8; 32]).unwrap();
        let layout = Layout::from_developer_data_root(tmp.path().join("data"));
        let err = backup_import(
            &layout,
            &source,
            &BackupRequest {
                name: Some("stock-lite".into()),
                as_original: false,
            },
            |_| Ok(None),
        )
        .unwrap_err();
        assert!(matches!(err, Error::Import(_)));
    }
}
