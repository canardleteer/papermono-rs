//! Snapshots under `developer-data/backups/original/` and
//! `developer-data/backups/captures/`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::identity::LiveIdentity;
use crate::manifest::{Manifest, SnapshotKind};
use crate::Error;

/// Filesystem layout for gitignored private / personalized files.
#[derive(Debug, Clone)]
pub struct Layout {
    /// `{repo}/developer-data` (or a test stand-in).
    pub developer_data_root: PathBuf,
    /// `developer-data/backups/`.
    pub backups_root: PathBuf,
}

impl Layout {
    /// `{root}/backups` as the snapshot tree (tests pass a temp dir).
    #[must_use]
    pub fn from_developer_data_root(developer_data_root: impl Into<PathBuf>) -> Self {
        let developer_data_root = developer_data_root.into();
        let backups_root = developer_data_root.join("backups");
        Self {
            developer_data_root,
            backups_root,
        }
    }

    /// `{repo}/developer-data/backups`.
    #[must_use]
    pub fn from_repo_root(repo_root: impl Into<PathBuf>) -> Self {
        Self::from_developer_data_root(repo_root.into().join("developer-data"))
    }

    /// `developer-data/backups/original/`.
    #[must_use]
    pub fn originals_dir(&self) -> PathBuf {
        self.backups_root.join("original")
    }

    /// `developer-data/backups/original/{unit-id}/`.
    #[must_use]
    pub fn original_dir(&self, unit_id: &str) -> PathBuf {
        self.originals_dir().join(unit_id)
    }

    /// `developer-data/backups/captures/`.
    #[must_use]
    pub fn captures_dir(&self) -> PathBuf {
        self.backups_root.join("captures")
    }

    /// `developer-data/backups/captures/{unit-id}/{slug}/`.
    #[must_use]
    pub fn capture_dir(&self, unit_id: &str, slug: &str) -> PathBuf {
        self.captures_dir().join(unit_id).join(slug)
    }

    /// `developer-data/confirm-records/{unit-id}/`.
    #[must_use]
    pub fn confirm_records_dir(&self, unit_id: &str) -> PathBuf {
        self.developer_data_root
            .join("confirm-records")
            .join(unit_id)
    }
}

/// Refuse a leftover repo-root `backups/` directory. Do not auto-move it.
pub fn refuse_if_legacy_backups_at_repo_root(repo_root: &Path) -> Result<(), Error> {
    let leftover = repo_root.join("backups");
    if leftover.is_dir() {
        Err(Error::LegacyBackupsDir(leftover))
    } else {
        Ok(())
    }
}

/// An on-disk snapshot plus its MANIFEST.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// Directory path.
    pub dir: PathBuf,
    /// Parsed MANIFEST.
    pub manifest: Manifest,
}

impl Snapshot {
    /// True when this tree is a factory original (path or kind).
    #[must_use]
    pub fn is_original(&self) -> bool {
        matches!(self.manifest.kind, SnapshotKind::Original)
            && self
                .dir
                .parent()
                .and_then(|p| p.file_name())
                .is_some_and(|name| name == "original")
    }
}

/// Load `MANIFEST.json`.
pub fn load_manifest(dir: &Path) -> Result<Manifest, Error> {
    let json = dir.join("MANIFEST.json");
    if json.is_file() {
        let text = fs::read_to_string(json)?;
        return Ok(serde_json::from_str(&text)?);
    }
    Err(Error::Import(format!(
        "no MANIFEST.json in {}",
        dir.display()
    )))
}

fn is_usable_dir(path: &Path) -> bool {
    path.is_dir()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| !name.ends_with(".partial"))
}

/// Refuse unless a matching original exists.
pub fn require_original_backup(layout: &Layout, live: &LiveIdentity) -> Result<Snapshot, Error> {
    let mut matches = snapshots_for_identity(&list_originals(layout)?, live);
    match matches.len() {
        0 => Err(Error::MissingOriginal),
        1 => {
            let found = matches.pop().unwrap();
            bind_check(&found.manifest.identity(), live)?;
            Ok(found)
        }
        _ => Err(Error::AmbiguousOriginal),
    }
}

/// Bind confirm/restore to a named capture for this identity.
pub fn require_capture_backup(
    layout: &Layout,
    live: &LiveIdentity,
    slug: &str,
) -> Result<Snapshot, Error> {
    crate::identity::validate_unit_id(slug)?;
    let mut matches: Vec<_> = list_captures(layout)?
        .into_iter()
        .filter(|snap| {
            snap.dir.file_name().and_then(|name| name.to_str()) == Some(slug)
                && identity_matches(&snap.manifest.identity(), live)
        })
        .collect();
    match matches.len() {
        0 => Err(Error::MissingCapture(slug.to_string())),
        1 => {
            let found = matches.pop().unwrap();
            bind_check(&found.manifest.identity(), live)?;
            Ok(found)
        }
        _ => Err(Error::AmbiguousCapture),
    }
}

fn snapshots_for_identity(listed: &[Snapshot], live: &LiveIdentity) -> Vec<Snapshot> {
    listed
        .iter()
        .filter(|snap| identity_matches(&snap.manifest.identity(), live))
        .cloned()
        .collect()
}

fn identity_matches(stored: &LiveIdentity, live: &LiveIdentity) -> bool {
    if let (Some(a), Some(b)) = (&stored.usb_serial_sha256, &live.usb_serial_sha256) {
        return a == b;
    }
    matches!(
        (&stored.mac_sha256, &live.mac_sha256),
        (Some(a), Some(b)) if a == b
    )
}

/// USB serial hash must match when both sides have one; else MAC hash.
pub fn bind_check(stored: &LiveIdentity, live: &LiveIdentity) -> Result<(), Error> {
    if let (Some(expected), Some(got)) = (&stored.usb_serial_sha256, &live.usb_serial_sha256) {
        if expected != got {
            return Err(Error::IdentityMismatch {
                reason: "USB serial hash does not match the snapshot MANIFEST".into(),
            });
        }
        return Ok(());
    }
    match (&stored.mac_sha256, &live.mac_sha256) {
        (Some(expected), Some(got)) if expected == got => Ok(()),
        (Some(_), Some(_)) => Err(Error::IdentityMismatch {
            reason: "MAC hash does not match the snapshot MANIFEST".into(),
        }),
        _ => Err(Error::IdentityMismatch {
            reason: "snapshot and live unit have no overlapping identity hash".into(),
        }),
    }
}

/// Readable `original/{unit-id}/` trees.
pub fn list_originals(layout: &Layout) -> Result<Vec<Snapshot>, Error> {
    list_snapshot_dirs(&layout.originals_dir())
}

/// Readable `captures/{unit-id}/{slug}/` trees.
pub fn list_captures(layout: &Layout) -> Result<Vec<Snapshot>, Error> {
    let root = layout.captures_dir();
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for unit in fs::read_dir(&root)? {
        let unit = unit?;
        if !is_usable_dir(&unit.path()) {
            continue;
        }
        out.extend(list_snapshot_dirs(&unit.path())?);
    }
    Ok(out)
}

fn list_snapshot_dirs(dir: &Path) -> Result<Vec<Snapshot>, Error> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !is_usable_dir(&path) {
            continue;
        }
        let Ok(manifest) = load_manifest(&path) else {
            continue;
        };
        out.push(Snapshot {
            dir: path,
            manifest,
        });
    }
    Ok(out)
}

/// Refuse a second write to `original/{unit-id}/`.
pub fn refuse_if_original_exists(layout: &Layout, unit_id: &str) -> Result<(), Error> {
    let dest = layout.original_dir(unit_id);
    if dest.is_dir() {
        Err(Error::OriginalExists(dest))
    } else {
        Ok(())
    }
}

/// Refuse a second write to `captures/{unit-id}/{slug}/`.
pub fn refuse_if_capture_exists(layout: &Layout, unit_id: &str, slug: &str) -> Result<(), Error> {
    let dest = layout.capture_dir(unit_id, slug);
    if dest.is_dir() {
        Err(Error::CaptureExists(dest))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_requires_matching_usb_hash() {
        let stored = LiveIdentity {
            mac_sha256: Some("aa".into()),
            usb_serial_sha256: Some("usb-a".into()),
        };
        let live = LiveIdentity {
            mac_sha256: Some("aa".into()),
            usb_serial_sha256: Some("usb-b".into()),
        };
        assert!(matches!(
            bind_check(&stored, &live),
            Err(Error::IdentityMismatch { .. })
        ));
    }
}
