//! Host-only CI gate. No USB, no [`papermono_host::Layout`].
//!
//! One workspace, one lockfile. Never `--workspace` on the host rustc
//! once firmware members exist (that would pull Xtensa). There are no
//! firmware packages yet; do not add `cargo +esp` until one lands.
//! Board crates under `crates/` are default-members.

use std::env;
use std::path::Path;
use std::process::Command;

use papermono_host::Error;

/// Run the full gate from the repository root. First failure wins.
pub fn run(repo_root: &Path) -> Result<(), Error> {
    step(repo_root, "cargo", &["fmt", "--check", "--all"])?;
    host_clippy_test(repo_root, &[])?;
    host_clippy_test(repo_root, &["--all-features"])?;

    require_on_path("rumdl", "cargo install rumdl")?;
    step(repo_root, "rumdl", &["check"])?;

    require_on_path("cargo-machete", "cargo install cargo-machete")?;
    step(repo_root, "cargo-machete", &[])?;

    require_on_path("cargo-audit", "cargo install cargo-audit")?;
    step(repo_root, "cargo", &["audit"])?;

    Ok(())
}

fn host_clippy_test(repo_root: &Path, extra: &[&str]) -> Result<(), Error> {
    let mut clippy = vec!["clippy", "--locked", "--all-targets"];
    clippy.extend_from_slice(extra);
    clippy.extend_from_slice(&["--", "-D", "warnings"]);
    step(repo_root, "cargo", &clippy)?;

    let mut test = vec!["test", "--locked"];
    test.extend_from_slice(extra);
    step(repo_root, "cargo", &test)
}

fn require_on_path(bin: &str, install: &str) -> Result<(), Error> {
    if executable_on_path(bin) {
        return Ok(());
    }
    eprintln!("ci: `{bin}` is not on PATH");
    eprintln!("install: {install}");
    Err(Error::Device(format!(
        "{bin} is not on PATH; install with `{install}`"
    )))
}

fn executable_on_path(name: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|dir| dir.join(name).is_file())
}

fn step(repo_root: &Path, program: &str, args: &[&str]) -> Result<(), Error> {
    let shown = std::iter::once(program)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("==> {shown}");
    let status = Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .status()
        .map_err(|error| Error::Device(format!("failed to spawn {program}: {error}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Device(format!("ci failed: {shown}")))
    }
}
