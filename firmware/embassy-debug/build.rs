//! Stamp the Embassy image with the repo git hash and verify asset cache.

use sha2::{Digest, Sha256};
use std::fs;

include!("../../scripts/git_env.rs");

fn check_ferris_asset() {
    let manifest =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let assets_dir = manifest.join("assets");
    let svg_path = assets_dir.join("ferris-happy-line-art.svg");
    let bpp_path = assets_dir.join("ferris.1bpp");
    let sha_path = assets_dir.join("ferris-happy-line-art.svg.sha256");

    println!("cargo:rerun-if-changed={}", svg_path.display());
    println!("cargo:rerun-if-changed={}", bpp_path.display());
    println!("cargo:rerun-if-changed={}", sha_path.display());

    let bpp_metadata = match fs::metadata(&bpp_path) {
        Ok(m) => m,
        Err(_) => panic!(
            "\n\nERROR: Required firmware asset is missing: {}\n\
             Please run `cargo xtask encode-assets` to generate it.\n\n",
            bpp_path.display()
        ),
    };

    const EXPECTED_LEN: u64 = (360 * 240) / 8;
    if bpp_metadata.len() != EXPECTED_LEN {
        panic!(
            "\n\nERROR: Asset size mismatch: {} has {} bytes, expected {}\n\
             Please run `cargo xtask encode-assets` to regenerate it.\n\n",
            bpp_path.display(),
            bpp_metadata.len(),
            EXPECTED_LEN
        );
    }

    let recorded_sha = match fs::read_to_string(&sha_path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => panic!(
            "\n\nERROR: Asset hash file is missing: {}\n\
             Please run `cargo xtask encode-assets` to record asset checksums.\n\n",
            sha_path.display()
        ),
    };

    let svg_bytes = match fs::read(&svg_path) {
        Ok(b) => b,
        Err(e) => panic!(
            "\n\nERROR: Failed to read source SVG: {}: {}\n\n",
            svg_path.display(),
            e
        ),
    };

    let actual_sha = format!("{:x}", Sha256::digest(&svg_bytes));
    if actual_sha != recorded_sha {
        panic!(
            "\n\nERROR: Source SVG has changed since {} was generated!\n\
             Source SVG hash:   {}\n\
             Recorded hash:     {}\n\
             Please run `cargo xtask encode-assets` to update the encoded firmware bitmap.\n\n",
            bpp_path.display(),
            actual_sha,
            recorded_sha
        );
    }
}

fn main() {
    emit_git_env("EMBASSY_DEBUG_GIT", "EMBASSY_DEBUG_GIT_DIRTY");
    check_ferris_asset();
}
