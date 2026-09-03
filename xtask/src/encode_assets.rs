//! Rasterize and pack SVG line art into 1bpp firmware bitmaps.

use std::fs;
use std::path::Path;
use std::process::Command;

use papermono_host::Error;
use sha2::{Digest, Sha256};

/// Encode assets under `firmware/embassy-debug/assets`.
pub fn run(repo_root: &Path) -> Result<(), Error> {
    let assets_dir = repo_root.join("firmware/embassy-debug/assets");
    let svg_path = assets_dir.join("ferris-happy-line-art.svg");
    let script_path = assets_dir.join("encode_ferris.py");
    let sha_path = assets_dir.join("ferris-happy-line-art.svg.sha256");
    let bpp_path = assets_dir.join("ferris.1bpp");
    let inv_bpp_path = assets_dir.join("ferris-inv.1bpp");

    if !svg_path.exists() {
        return Err(Error::Device(format!(
            "source SVG not found: {}",
            svg_path.display()
        )));
    }
    if !script_path.exists() {
        return Err(Error::Device(format!(
            "encoder script not found: {}",
            script_path.display()
        )));
    }

    println!("rasterizing {}", svg_path.display());
    let status = Command::new("python3")
        .arg(&script_path)
        .arg("--svg")
        .arg(&svg_path)
        .arg("--out-dir")
        .arg(&assets_dir)
        .status()
        .map_err(|e| Error::Device(format!("failed to execute python3: {e}")))?;

    if !status.success() {
        return Err(Error::Device("encode_ferris.py failed".into()));
    }

    let bpp_bytes = fs::read(&bpp_path)
        .map_err(|e| Error::Device(format!("failed to read {}: {e}", bpp_path.display())))?;
    let inv_bytes = fs::read(&inv_bpp_path)
        .map_err(|e| Error::Device(format!("failed to read {}: {e}", inv_bpp_path.display())))?;

    const EXPECTED_BYTES: usize = (360 * 240) / 8;
    if bpp_bytes.len() != EXPECTED_BYTES || inv_bytes.len() != EXPECTED_BYTES {
        return Err(Error::Device(format!(
            "unexpected bitmap size: got {}/{} bytes, expected {EXPECTED_BYTES}",
            bpp_bytes.len(),
            inv_bytes.len()
        )));
    }

    let svg_bytes = fs::read(&svg_path)
        .map_err(|e| Error::Device(format!("failed to read {}: {e}", svg_path.display())))?;
    let svg_hash = format!("{:x}", Sha256::digest(&svg_bytes));

    fs::write(&sha_path, format!("{svg_hash}\n"))
        .map_err(|e| Error::Device(format!("failed to write {}: {e}", sha_path.display())))?;

    println!("encoded 1bpp assets in {}", assets_dir.display());
    println!("sha256: {svg_hash}");
    Ok(())
}
