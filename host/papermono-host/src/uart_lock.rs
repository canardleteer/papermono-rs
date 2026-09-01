//! Exclusive USB session lock so one command cannot reset another.
//!
//! Inventory without `--probe` does not take this lock.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};

use fs4::fs_std::FileExt;
use sha2::{Digest, Sha256};

use crate::Error;

/// Environment variable [`UartSession::prepare_command`] sets on children.
pub const UART_LOCK_ENV: &str = "PAPERMONO_UART_LOCK";

/// Held exclusive flock (or a child joined to a parent's flock).
#[derive(Debug)]
#[must_use = "dropping the USB session releases the lock"]
pub struct UartSession {
    path: PathBuf,
    _file: Option<File>,
}

/// Acquire an exclusive session for `port`, or refuse without waiting.
pub fn try_acquire(port: &str, command: &str) -> Result<UartSession, Error> {
    try_acquire_in(&std::env::temp_dir(), port, command)
}

/// Testable [`try_acquire`] with an explicit lock directory.
pub fn try_acquire_in(lock_dir: &Path, port: &str, command: &str) -> Result<UartSession, Error> {
    acquire(lock_dir, port, command, inherit_from_env().as_deref())
}

fn inherit_from_env() -> Option<PathBuf> {
    std::env::var_os(UART_LOCK_ENV).map(PathBuf::from)
}

fn acquire(
    lock_dir: &Path,
    port: &str,
    command: &str,
    inherit: Option<&Path>,
) -> Result<UartSession, Error> {
    fs::create_dir_all(lock_dir)?;
    let path = lock_path(lock_dir, port);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)?;
    match file.try_lock_exclusive() {
        Ok(true) => {
            write_holder(&file, command)?;
            Ok(UartSession {
                path,
                _file: Some(file),
            })
        }
        Ok(false) if can_join(&path, inherit) => Ok(UartSession { path, _file: None }),
        Ok(false) => Err(busy_from(&path)),
        Err(error) => Err(Error::from(error)),
    }
}

impl UartSession {
    /// Lock file this session uses (also the value of [`UART_LOCK_ENV`]).
    #[must_use]
    pub fn lock_path(&self) -> &Path {
        &self.path
    }

    #[cfg(test)]
    fn owns_flock(&self) -> bool {
        self._file.is_some()
    }

    /// Point a child at this session.
    pub fn prepare_command(&self, cmd: &mut Command) {
        cmd.env(UART_LOCK_ENV, &self.path);
    }

    /// Run `cmd` and wait. The flock stays held until the child exits.
    pub fn status(&self, cmd: &mut Command) -> Result<ExitStatus, Error> {
        self.prepare_command(cmd);
        cmd.status().map_err(Error::from)
    }

    /// Like [`Self::status`], but captures stdout/stderr.
    pub fn output(&self, cmd: &mut Command) -> Result<Output, Error> {
        self.prepare_command(cmd);
        cmd.output().map_err(Error::from)
    }
}

fn lock_path(lock_dir: &Path, port: &str) -> PathBuf {
    lock_dir.join(format!("papermono-rs-xtask-uart-{}.lock", lock_stem(port)))
}

fn lock_stem(port: &str) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = fs::metadata(port) {
            return format!("{}-{}", meta.dev(), meta.ino());
        }
    }
    let digest = Sha256::digest(port.as_bytes());
    format!("{digest:x}")
}

fn write_holder(file: &File, command: &str) -> io::Result<()> {
    file.set_len(0)?;
    let mut file = file;
    writeln!(file, "pid={}", std::process::id())?;
    writeln!(file, "command={}", command.trim())?;
    file.sync_all()
}

fn busy_from(path: &Path) -> Error {
    let text = fs::read_to_string(path).unwrap_or_default();
    let (pid, command) = parse_holder(&text);
    Error::UartBusy { pid, command }
}

fn can_join(lock_path: &Path, inherit: Option<&Path>) -> bool {
    let Some(inherit) = inherit else {
        return false;
    };
    if inherit != lock_path {
        return false;
    }
    let text = fs::read_to_string(lock_path).unwrap_or_default();
    let Some(pid) = parse_holder(&text).0 else {
        return false;
    };
    is_self_or_ancestor(pid)
}

fn is_self_or_ancestor(pid: u32) -> bool {
    let mut current = std::process::id();
    for _ in 0..64 {
        if current == pid {
            return true;
        }
        match ppid_of(current) {
            Some(0 | 1) | None => return false,
            Some(next) if next == current => return false,
            Some(next) => current = next,
        }
    }
    false
}

fn ppid_of(pid: u32) -> Option<u32> {
    let text = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("PPid:") {
            return rest.trim().parse().ok();
        }
    }
    None
}

fn parse_holder(text: &str) -> (Option<u32>, Option<String>) {
    let mut pid = None;
    let mut command = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("pid=") {
            pid = rest.trim().parse().ok();
        }
        if let Some(rest) = line.strip_prefix("command=") {
            let value = rest.trim();
            if !value.is_empty() {
                command = Some(value.to_string());
            }
        }
    }
    (pid, command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn port_file(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"").unwrap();
        path
    }

    #[test]
    fn second_acquire_on_the_same_inode_refuses() {
        let dir = tempfile::tempdir().unwrap();
        let lock_dir = dir.path().join("locks");
        let port = port_file(dir.path(), "uart");
        let port = port.to_str().unwrap();
        let first = try_acquire_in(&lock_dir, port, "restore-factory-firmware").unwrap();
        let err = try_acquire_in(&lock_dir, port, "detect-connected --probe").unwrap_err();
        assert!(matches!(err, Error::UartBusy { .. }));
        drop(first);
        let _released = try_acquire_in(&lock_dir, port, "confirm-factory-firmware").unwrap();
    }

    #[test]
    fn nested_acquire_joins_when_inherit_path_matches() {
        let dir = tempfile::tempdir().unwrap();
        let lock_dir = dir.path().join("locks");
        let port = port_file(dir.path(), "uart");
        let port = port.to_str().unwrap();
        let parent = try_acquire_in(&lock_dir, port, "backup-factory-firmware").unwrap();
        let child = acquire(
            &lock_dir,
            port,
            "detect-connected --probe",
            Some(parent.lock_path()),
        )
        .unwrap();
        assert!(!child.owns_flock());
    }
}
