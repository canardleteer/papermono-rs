//! Device I/O via the `espflash` library (the crate `cargo-espflash` wraps).
//!
//! Host tests inject [`MockDevice`] and never open a port.
//! Connect prefers [`ResetBeforeOperation::NoReset`]: the operator puts
//! the unit in download with the power button.

use std::fmt::Write as _;
use std::path::Path;

use espflash::connection::{Connection, ResetAfterOperation, ResetBeforeOperation};
use espflash::flasher::Flasher;
use espflash::target::{Chip, ProgressCallbacks};
use serialport::{FlowControl, UsbPortInfo};

use crate::detect::{
    require_papermono_usb, usb_device_key_for_port, ESPRESSIF_JTAG_PID, ESPRESSIF_VID,
};
use crate::identity::mac_sha256;
use crate::{Error, CHUNK_SIZE};

/// Baud after the flasher stub is loaded (same as `cargo espflash --baud`).
pub const ESPFLASH_BAUD: u32 = 921_600;
/// ROM connect baud. [`Flasher::connect`] then raises to [`ESPFLASH_BAUD`].
pub const CONNECT_BAUD: u32 = 115_200;
/// `read_flash` packet size used by `cargo-espflash` (`FLASH_SECTOR_SIZE`).
const READ_BLOCK: u32 = 0x1000;
/// Un-acked packets allowed by `cargo-espflash read-flash`.
const MAX_IN_FLIGHT: u32 = 64;

/// Operations that would touch a PaperMono. Mocked in unit tests.
pub trait DeviceIo {
    /// Text matching `cargo espflash board-info` (MAC redacted).
    fn board_info(&self, port: &str) -> Result<String, Error>;
    /// One flash window. [`RealDevice`] keeps one flasher session for the call.
    fn read_flash(&self, port: &str, offset: u32, size: u32) -> Result<Vec<u8>, Error>;
    /// `write_bin_to_flash` of `file` at `offset`. Never a full-chip erase.
    fn write_bin(&self, port: &str, offset: u32, file: &Path) -> Result<(), Error>;
}

/// In-process `espflash` flasher.
pub struct RealDevice;

fn map_espflash(error: espflash::Error) -> Error {
    Error::Device(error.to_string())
}

fn connect(port: &str, after_baud: Option<u32>) -> Result<Flasher, Error> {
    require_papermono_usb(port)?;
    let serial = serialport::new(port, CONNECT_BAUD)
        .flow_control(FlowControl::None)
        .open_native()
        .map_err(|error| Error::Device(format!("serial open failed: {error}")))?;
    let (vid, pid) = match usb_device_key_for_port(port) {
        Ok(key) => (key.vid, key.pid),
        Err(_) => (ESPRESSIF_VID, ESPRESSIF_JTAG_PID),
    };
    // Real Espressif ids so espflash picks USB-JTAG, not a dummy UART
    // strategy. NoReset: operator download (power-button hold) is primary.
    let usb = UsbPortInfo {
        vid,
        pid,
        serial_number: None,
        manufacturer: None,
        product: None,
    };
    let connection = Connection::new(
        serial,
        usb,
        ResetAfterOperation::NoReset,
        ResetBeforeOperation::NoReset,
        CONNECT_BAUD,
    );
    let mut flasher =
        Flasher::connect(connection, true, true, true, None, after_baud).map_err(map_espflash)?;
    if flasher.chip() != Chip::Esp32s3 {
        return Err(Error::Device(format!(
            "expected ESP32-S3, found {}",
            flasher.chip()
        )));
    }
    if let Ok(info) = flasher.device_info() {
        flasher.set_flash_size(info.flash_size);
    }
    Ok(flasher)
}

fn format_board_info(flasher: &mut Flasher) -> Result<String, Error> {
    let info = flasher.device_info().map_err(map_espflash)?;
    let mut text = String::new();
    let _ = writeln!(text, "Chip type:         {}", info.chip);
    if let Some((major, minor)) = info.revision {
        let _ = writeln!(text, "Chip revision:     v{major}.{minor}");
    }
    let _ = writeln!(text, "Crystal frequency: {}", info.crystal_frequency);
    let _ = writeln!(text, "Flash size:        {}", info.flash_size);
    if !info.features.is_empty() {
        let _ = writeln!(text, "Features:          {}", info.features.join(", "));
    }
    // Hash internally so MANIFEST can bind; never print octets.
    let _ = writeln!(text, "MAC address:       (redacted)");
    if let Some(mac) = &info.mac_address {
        let _ = writeln!(text, "MAC sha256:        {}", mac_sha256(mac));
    }
    if info.chip != Chip::Esp32 {
        match flasher.security_info() {
            Ok(security) => {
                let _ = write!(text, "{security}");
            }
            Err(_) => {
                let _ = writeln!(text, "Secure Boot: Disabled");
                let _ = writeln!(text, "Flash Encryption: Disabled");
            }
        }
    }
    Ok(text)
}

/// How many times to reconnect and retry the current 1 MiB write window.
const WRITE_WINDOW_RETRIES: u32 = 3;

/// Windows needed to write `len` bytes in [`CHUNK_SIZE`] slices.
#[must_use]
pub(crate) fn write_bin_window_count(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        len.div_ceil(CHUNK_SIZE)
    }
}

/// Percent complete from espflash chunk counts.
#[must_use]
pub(crate) fn write_bin_percent(current_chunks: usize, total_chunks: usize) -> u8 {
    if total_chunks == 0 {
        100
    } else {
        current_chunks
            .saturating_mul(100)
            .checked_div(total_chunks)
            .unwrap_or(100)
            .min(100) as u8
    }
}

struct WriteBinProgress {
    window: usize,
    windows: usize,
    image_bytes: usize,
    total_chunks: usize,
    last_percent: u8,
    started: std::time::Instant,
}

impl WriteBinProgress {
    fn new(window: usize, windows: usize, image_bytes: usize) -> Self {
        Self {
            window,
            windows,
            image_bytes,
            total_chunks: 0,
            last_percent: 0,
            started: std::time::Instant::now(),
        }
    }

    fn paint(&self, current: usize, extra: &str) {
        let pct = write_bin_percent(current, self.total_chunks);
        let written = self
            .image_bytes
            .saturating_mul(current)
            .checked_div(self.total_chunks)
            .unwrap_or(self.image_bytes);
        eprint!(
            "\rwrite-bin window {}/{} {current}/{} ({pct}%) {}/{} bytes elapsed={:?}{extra}   ",
            self.window,
            self.windows,
            self.total_chunks,
            written,
            self.image_bytes,
            self.started.elapsed(),
        );
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }
}

impl ProgressCallbacks for WriteBinProgress {
    fn init(&mut self, addr: u32, total: usize) {
        self.total_chunks = total;
        self.last_percent = 0;
        self.started = std::time::Instant::now();
        eprintln!(
            "write-bin window {}/{} offset={addr:#010x} chunks={total} bytes={}",
            self.window, self.windows, self.image_bytes
        );
        self.paint(0, "");
    }

    fn update(&mut self, current: usize) {
        let pct = write_bin_percent(current, self.total_chunks);
        if pct != self.last_percent || current == self.total_chunks {
            self.last_percent = pct;
            self.paint(current, "");
        }
    }

    fn verifying(&mut self) {
        self.paint(self.total_chunks, " verifying");
    }

    fn finish(&mut self, skipped: bool) {
        if skipped {
            self.paint(self.total_chunks, " skipped (checksum match)");
        } else {
            self.paint(self.total_chunks, " done");
        }
        eprintln!();
    }
}

impl DeviceIo for RealDevice {
    fn board_info(&self, port: &str) -> Result<String, Error> {
        log::info!("connecting flasher for board-info (NoReset)");
        let mut flasher = connect(port, Some(ESPFLASH_BAUD))?;
        format_board_info(&mut flasher)
    }

    fn read_flash(&self, port: &str, offset: u32, size: u32) -> Result<Vec<u8>, Error> {
        let total_chunks = size.div_ceil(CHUNK_SIZE as u32).max(1);
        log::info!(
            "connecting flasher for read-flash {size} bytes in {total_chunks}×{} KiB windows",
            CHUNK_SIZE / 1024
        );
        let mut flasher = connect(port, Some(ESPFLASH_BAUD))?;
        let mut dump = Vec::with_capacity(size as usize);
        let mut remaining = size;
        let mut addr = offset;
        let mut index = 0u32;
        let started = std::time::Instant::now();
        while remaining > 0 {
            index += 1;
            let chunk = remaining.min(CHUNK_SIZE as u32);
            log::info!(
                "read-flash {index}/{total_chunks} offset={addr:#010x} size={chunk} elapsed={:?}",
                started.elapsed()
            );
            let tmp = tempfile::Builder::new()
                .prefix("papermono-xtask-")
                .suffix(".bin")
                .tempfile()
                .map_err(Error::from)?;
            let path = tmp.path().to_path_buf();
            flasher
                .read_flash(addr, chunk, READ_BLOCK, MAX_IN_FLIGHT, path.clone())
                .map_err(map_espflash)?;
            let bytes = std::fs::read(&path)?;
            if bytes.len() != chunk as usize {
                return Err(Error::Device(format!(
                    "read at {addr:#x} returned {} bytes, expected {chunk}",
                    bytes.len()
                )));
            }
            dump.extend(bytes);
            addr = addr.saturating_add(chunk);
            remaining -= chunk;
        }
        Ok(dump)
    }

    fn write_bin(&self, port: &str, offset: u32, file: &Path) -> Result<(), Error> {
        let data = std::fs::read(file)?;
        let windows = write_bin_window_count(data.len());
        eprintln!(
            "write-bin: {} bytes at {offset:#x} in {windows}×{} KiB windows",
            data.len(),
            CHUNK_SIZE / 1024
        );
        if data.is_empty() {
            return Err(Error::Device("write-bin image is empty".into()));
        }
        let mut flasher = connect(port, Some(ESPFLASH_BAUD))?;
        let started = std::time::Instant::now();
        let mut index = 0usize;
        let mut retries_left = WRITE_WINDOW_RETRIES;
        while index < windows {
            let start = index.saturating_mul(CHUNK_SIZE);
            let end = start.saturating_add(CHUNK_SIZE).min(data.len());
            let addr = offset.saturating_add(start as u32);
            let slice = &data[start..end];
            let window = index.saturating_add(1);
            eprintln!(
                "write-bin window {window}/{windows} offset={addr:#010x} bytes={} elapsed={:?}",
                slice.len(),
                started.elapsed()
            );
            let mut progress = WriteBinProgress::new(window, windows, slice.len());
            match flasher.write_bin_to_flash(addr, slice, &mut progress) {
                Ok(()) => {
                    index = index.saturating_add(1);
                    retries_left = WRITE_WINDOW_RETRIES;
                }
                Err(error) => {
                    let mapped = map_espflash(error);
                    if retries_left == 0 {
                        return Err(mapped);
                    }
                    retries_left -= 1;
                    eprintln!(
                        "write-bin window {window}/{windows} dropped ({mapped}); reconnecting, {retries_left} retries left"
                    );
                    drop(flasher);
                    flasher = connect(port, Some(ESPFLASH_BAUD))?;
                }
            }
        }
        Ok(())
    }
}

/// In-memory device for host tests. Never opens a port or constructs a flasher.
#[derive(Debug, Clone, Default)]
pub struct MockDevice {
    /// `board-info` text (MAC redacted or fixture).
    pub board_info: String,
    /// Full (or fixture-sized) flash contents.
    pub flash: Vec<u8>,
    /// Recorded write-bin calls: offset and file bytes.
    pub writes: Vec<(u32, Vec<u8>)>,
}

impl DeviceIo for std::cell::RefCell<MockDevice> {
    fn board_info(&self, _port: &str) -> Result<String, Error> {
        Ok(self.borrow().board_info.clone())
    }

    fn read_flash(&self, _port: &str, offset: u32, size: u32) -> Result<Vec<u8>, Error> {
        let flash = &self.borrow().flash;
        let start = offset as usize;
        let end = start.saturating_add(size as usize);
        if end > flash.len() {
            return Err(Error::Device("mock flash shorter than read window".into()));
        }
        Ok(flash[start..end].to_vec())
    }

    fn write_bin(&self, _port: &str, offset: u32, file: &Path) -> Result<(), Error> {
        let bytes = std::fs::read(file)?;
        self.borrow_mut().writes.push((offset, bytes));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{write_bin_percent, write_bin_window_count};
    use crate::CHUNK_SIZE;

    #[test]
    fn write_bin_percent_treats_counts_as_chunks() {
        assert_eq!(write_bin_percent(0, 238), 0);
        assert_eq!(write_bin_percent(119, 238), 50);
        assert_eq!(write_bin_percent(238, 238), 100);
    }

    #[test]
    fn write_bin_window_count_matches_chunks() {
        assert_eq!(write_bin_window_count(0), 0);
        assert_eq!(write_bin_window_count(1), 1);
        assert_eq!(write_bin_window_count(CHUNK_SIZE), 1);
        assert_eq!(write_bin_window_count(CHUNK_SIZE + 1), 2);
    }
}
