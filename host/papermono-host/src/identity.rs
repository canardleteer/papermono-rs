//! USB serial hashes, MAC hashes, and `board-info` text.
//!
//! Raw MAC and iSerial stay out of stdout and out of committed docs.
//! Gitignored MANIFEST files store SHA-256 only.

use sha2::{Digest, Sha256};

use crate::Error;

/// Official product-table flash size (16 MiB). Live dumps use the
/// measured `board-info` size, not this constant, once `--probe` has run.
pub const OFFICIAL_FLASH_SIZE: usize = 16 * 1024 * 1024;

/// Live chip identity used to bind a unit to a snapshot (hashes only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveIdentity {
    /// SHA-256 hex of the normalized station MAC (`aa:bb:…`).
    pub mac_sha256: Option<String>,
    /// SHA-256 hex of the USB iSerial when known.
    pub usb_serial_sha256: Option<String>,
}

/// Parsed flasher board-info. Printed text must not include a raw MAC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardInfo {
    /// Live identity hashes.
    pub identity: LiveIdentity,
    /// Raw `Flash size:` field (`16MB`, …).
    pub flash_size: String,
    /// Parsed full-chip length when the size field is usable.
    pub flash_size_bytes: Option<usize>,
    /// `Secure Boot:` reported enabled.
    pub secure_boot: bool,
    /// `Flash Encryption:` reported enabled.
    pub flash_encryption: bool,
}

/// Espressif udev by-id marker (USB-Serial/JTAG product string).
pub(crate) fn espressif_jtag_marker() -> &'static str {
    "usb-Espressif_USB_JTAG_serial_debug_unit"
}

/// QinHeng udev by-id marker (wrong product for this board).
pub(crate) fn qinheng_marker() -> &'static str {
    "usb-1a86_USB_Single_Serial"
}

/// SHA-256 hex of a UTF-8 string.
#[must_use]
pub fn sha256_text(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Last four characters of a USB serial, when it is long enough.
#[must_use]
pub fn serial_last4(serial: &str) -> Option<&str> {
    let trimmed = serial.trim();
    if trimmed.len() >= 4 {
        Some(&trimmed[trimmed.len() - 4..])
    } else {
        None
    }
}

/// Parse an Espressif USB-Serial/JTAG serial out of a by-id `ESPFLASH_PORT`.
#[must_use]
pub fn parse_usb_serial_from_port(port: &str) -> Option<String> {
    let marker = espressif_jtag_marker();
    let rest = port.split(marker).nth(1)?;
    let rest = rest.trim_start_matches(['_', '-']);
    let serial = rest.split("-if").next()?.trim();
    if serial.is_empty() {
        None
    } else {
        Some(serial.to_string())
    }
}

/// Directory names: no slashes, no `..`, printable ASCII.
pub fn validate_unit_id(id: &str) -> Result<(), Error> {
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(Error::InvalidUnitId(id.to_string()));
    }
    Ok(())
}

/// Local directory id: `lite-<last4>` when we have an iSerial, else
/// `id-<8 hex>` from the USB or MAC hash. Never a raw MAC.
pub fn unit_id(identity: &LiveIdentity, usb_serial: Option<&str>) -> Result<String, Error> {
    if let Some(serial) = usb_serial.and_then(serial_last4) {
        let id = format!("lite-{serial}");
        validate_unit_id(&id)?;
        return Ok(id);
    }
    let hash = identity
        .usb_serial_sha256
        .as_deref()
        .or(identity.mac_sha256.as_deref())
        .ok_or_else(|| Error::Device("no USB serial or MAC hash to bind this unit".into()))?;
    let id = format!("id-{}", &hash[..8.min(hash.len())]);
    validate_unit_id(&id)?;
    Ok(id)
}

/// Parse `16MB` / `16 MB` / `32MB` into a byte length.
#[must_use]
pub fn parse_flash_size_bytes(raw: &str) -> Option<usize> {
    let compact: String = raw
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    let digits = compact
        .trim_end_matches("MB")
        .trim_end_matches('B')
        .trim_end_matches('M');
    let megabytes: usize = digits.parse().ok()?;
    if megabytes == 0 || megabytes > 256 {
        return None;
    }
    Some(megabytes.saturating_mul(1024 * 1024))
}

/// Require a measured full-chip length (live dump / restore).
pub fn require_flash_size_bytes(raw: &str) -> Result<usize, Error> {
    parse_flash_size_bytes(raw).ok_or_else(|| Error::FlashSizeUnknown(raw.to_string()))
}

/// Parse board-info text. A `MAC address: (redacted)` line is allowed;
/// the hash then comes from the caller / MANIFEST.
pub fn parse_board_info(text: &str) -> Result<BoardInfo, Error> {
    let mut mac = None;
    let mut mac_hash = None;
    let mut flash_size = None;
    let mut secure_boot = false;
    let mut flash_encryption = false;

    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("MAC sha256:") {
            let rest = rest.trim();
            if rest.len() == 64 && rest.chars().all(|c| c.is_ascii_hexdigit()) {
                mac_hash = Some(rest.to_ascii_lowercase());
            }
        }
        if let Some(rest) = line.strip_prefix("MAC address:") {
            let rest = rest.trim();
            if !rest.is_empty() && !rest.eq_ignore_ascii_case("(redacted)") {
                mac = Some(normalize_mac(rest)?);
            }
        }
        if let Some(rest) = line.strip_prefix("MAC:") {
            let rest = rest.trim();
            if mac.is_none() && !rest.eq_ignore_ascii_case("(redacted)") {
                mac = Some(normalize_mac(rest)?);
            }
        }
        if let Some(rest) = line.strip_prefix("Flash size:") {
            flash_size = Some(rest.trim().to_string());
        }
        if line.to_ascii_lowercase().contains("secure boot:") {
            secure_boot = line.to_ascii_lowercase().contains("enabled");
        }
        if line.to_ascii_lowercase().contains("flash encryption:") {
            flash_encryption = line.to_ascii_lowercase().contains("enabled");
        }
    }

    let flash_size = flash_size.unwrap_or_default();
    let flash_size_bytes = parse_flash_size_bytes(&flash_size);
    Ok(BoardInfo {
        identity: LiveIdentity {
            mac_sha256: mac_hash.or_else(|| mac.as_deref().map(sha256_text)),
            usb_serial_sha256: None,
        },
        flash_size,
        flash_size_bytes,
        secure_boot,
        flash_encryption,
    })
}

fn normalize_mac(raw: &str) -> Result<String, Error> {
    let token = raw.split_whitespace().next().unwrap_or("");
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 6 || parts.iter().any(|p| p.len() != 2) {
        return Err(Error::Device(
            "board-info MAC address was not six octets".into(),
        ));
    }
    Ok(parts
        .iter()
        .map(|p| p.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(":"))
}

/// Hash a live MAC without keeping the octets.
#[must_use]
pub fn mac_sha256(mac: &str) -> String {
    sha256_text(mac)
}

#[cfg(test)]
pub(crate) fn test_mac() -> String {
    [0x10u8, 0x20, 0x30, 0x40, 0x50, 0x60]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(":")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_unit_id() {
        assert!(matches!(
            validate_unit_id("../evil"),
            Err(Error::InvalidUnitId(_))
        ));
    }

    #[test]
    fn usb_serial_from_espressif_by_id() {
        let marker = espressif_jtag_marker();
        let path = format!("/dev/serial/by-id/{marker}_ABCDEF012345-if00");
        assert_eq!(
            parse_usb_serial_from_port(&path).as_deref(),
            Some("ABCDEF012345")
        );
        assert_eq!(serial_last4("ABCDEF012345"), Some("2345"));
    }

    #[test]
    fn flash_size_16mb() {
        assert_eq!(parse_flash_size_bytes("16MB"), Some(OFFICIAL_FLASH_SIZE));
        assert_eq!(parse_flash_size_bytes("16 MB"), Some(OFFICIAL_FLASH_SIZE));
        assert_eq!(parse_flash_size_bytes("32MB"), Some(32 * 1024 * 1024));
        assert_eq!(parse_flash_size_bytes(""), None);
    }

    #[test]
    fn board_info_accepts_16mb_and_hashes_mac() {
        let mac = test_mac();
        let text = format!(
            "Flash size:        16MB\nMAC address:       {mac}\nSecure Boot: Disabled\nFlash Encryption: Disabled\n"
        );
        let info = parse_board_info(&text).unwrap();
        assert_eq!(info.flash_size_bytes, Some(OFFICIAL_FLASH_SIZE));
        assert_eq!(
            info.identity.mac_sha256.as_deref(),
            Some(sha256_text(&mac).as_str())
        );
        assert!(!info.secure_boot);
    }

    #[test]
    fn board_info_allows_redacted_mac() {
        let text = "Flash size: 16MB\nMAC address: (redacted)\n";
        let info = parse_board_info(text).unwrap();
        assert!(info.identity.mac_sha256.is_none());
        assert_eq!(info.flash_size_bytes, Some(OFFICIAL_FLASH_SIZE));
    }

    #[test]
    fn unit_id_prefers_lite_last4() {
        let identity = LiveIdentity {
            mac_sha256: Some("abcd".into()),
            usb_serial_sha256: Some("efgh".into()),
        };
        assert_eq!(
            unit_id(&identity, Some("ABCDEF012345")).unwrap(),
            "lite-2345"
        );
    }
}
