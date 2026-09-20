//! Functional Specification for the OpenPGP Application on ISO/IEC 7816-4 Compatible Smart Cards (v3.4.1).
//!
//! Citations:
//! - OpenPGP application on ISO/IEC 7816-4 compatible smart cards (v3.4.1):
//!   - Section 4.1.1: "SELECT Command"
//!   - Section 4.1.2: "Application Identifier (AID)" (`D2 76 00 01 24 01`)
//!   - Section 4.1.3: "Application Related Data" (Tag `0x006E`)
//!   - Section 4.1.3.1: "AID (Tag 0x4F)"
//!   - Section 4.1.3.2: "Cardholder Related Data" (Name Tag `0x5B`)

use embedded_hal::i2c::I2c;

use crate::isodep::{IsoDepError, IsoDepSession, MAX_ISODEP_FRAME};
use crate::St25r3916;

/// OpenPGP Card Application Identifier (AID) prefix (`D2 76 00 01 24 01`).
pub const AID_OPENPGP: [u8; 6] = [0xD2, 0x76, 0x00, 0x01, 0x24, 0x01];

/// SELECT APDU command targeting the OpenPGP card application (11 bytes, no Le).
pub const CMD_SELECT_OPENPGP: [u8; 11] = [
    0x00, 0xA4, 0x04, 0x00, 0x06, 0xD2, 0x76, 0x00, 0x01, 0x24, 0x01,
];

/// GET DATA command for Application Related Data (Tag `0x006E`).
pub const CMD_GET_DATA_APP_RELATED: [u8; 5] = [0x00, 0xCA, 0x00, 0x6E, 0x00];

/// Discovered metadata from an OpenPGP smart card application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPgpCardInfo {
    /// OpenPGP Card specification version `(major, minor)` e.g. `(3, 4)`.
    pub version: (u8, u8),
    /// Manufacturer ID from AID bytes 8..10 (e.g. `0x0006` for Yubico).
    pub manufacturer: u16,
    /// 4-byte serial number from AID bytes 10..14.
    pub serial: [u8; 4],
    /// Cardholder name from Tag `0x5B`, if populated.
    pub cardholder_name: heapless::String<32>,
}

impl OpenPgpCardInfo {
    /// Major specification version (e.g. 3 for OpenPGP v3.4).
    #[inline]
    #[must_use]
    pub const fn version_major(&self) -> u8 {
        self.version.0
    }

    /// Minor specification version (e.g. 4 for OpenPGP v3.4).
    #[inline]
    #[must_use]
    pub const fn version_minor(&self) -> u8 {
        self.version.1
    }

    /// 2-byte manufacturer ID.
    #[inline]
    #[must_use]
    pub const fn manufacturer_id(&self) -> u16 {
        self.manufacturer
    }
}

/// Known manufacturer lookup from OpenPGP 2-byte manufacturer ID.
#[must_use]
pub fn manufacturer_name(mfg: u16) -> &'static str {
    match mfg {
        0x0005 => "ZeitControl",
        0x0006 => "Yubico",
        0x0007 => "OpenPGP Card Project",
        0x0008 => "FSIJ",
        0x000A => "Nitrokey",
        0x000D => "CardContact",
        0x000E => "Feitian",
        0x002A => "CanoKey",
        _ => "Unknown",
    }
}

/// Attempts to select and probe the OpenPGP application over an active ISO-DEP session,
/// returning the discovered metadata and the raw response status word.
pub fn probe_openpgp_detail<I2C: I2c>(
    session: &mut IsoDepSession,
    st: &mut St25r3916<I2C>,
) -> (Option<OpenPgpCardInfo>, u16) {
    let mut resp = [0u8; MAX_ISODEP_FRAME];
    let res = session.transceive_apdu_auto_get_response(st, &CMD_SELECT_OPENPGP, &mut resp);
    let (_len, sw) = match res {
        Ok(pair) => pair,
        Err(_) => {
            // Retry with Le=00 if Case 3 without Le was rejected
            let cmd_with_le = [
                0x00, 0xA4, 0x04, 0x00, 0x06, 0xD2, 0x76, 0x00, 0x01, 0x24, 0x01, 0x00,
            ];
            match session.transceive_apdu_auto_get_response(st, &cmd_with_le, &mut resp) {
                Ok(pair) => pair,
                Err(_) => return (None, 0x0000),
            }
        }
    };

    if sw != 0x9000 && (sw >> 8) != 0x61 {
        return (None, sw);
    }

    // Query Application Related Data (Tag 0x006E)
    if let Ok((data_len, app_sw)) =
        session.transceive_apdu_auto_get_response(st, &CMD_GET_DATA_APP_RELATED, &mut resp)
    {
        if (app_sw == 0x9000 || (app_sw >> 8) == 0x61) && data_len > 0 {
            return (parse_openpgp_app_data(&resp[..data_len]), sw);
        }
    }

    (
        Some(OpenPgpCardInfo {
            version: (3, 4),
            manufacturer: 0,
            serial: [0u8; 4],
            cardholder_name: heapless::String::new(),
        }),
        sw,
    )
}

/// Attempts to select and probe the OpenPGP application over an active ISO-DEP session.
pub fn probe_openpgp<I2C: I2c>(
    session: &mut IsoDepSession,
    st: &mut St25r3916<I2C>,
) -> Result<Option<OpenPgpCardInfo>, IsoDepError<I2C::Error>> {
    let (info, _sw) = probe_openpgp_detail(session, st);
    Ok(info)
}

/// Parses the BER-TLV payload returned by OpenPGP GET DATA `0x006E`.
pub fn parse_openpgp_app_data(data: &[u8]) -> Option<OpenPgpCardInfo> {
    let mut version = (3, 4);
    let mut manufacturer = 0;
    let mut serial = [0u8; 4];
    let mut cardholder_name = heapless::String::new();
    let mut found_aid = false;

    let mut i = 0;
    while i < data.len() {
        let tag = data[i];
        i += 1;
        if i >= data.len() {
            break;
        }

        // Two-byte tag support (e.g. 0x5F 0x2D or 0x00 0x6E)
        let (full_tag, tag_len_bytes) = if tag == 0x5F || tag == 0x7F {
            if i >= data.len() {
                break;
            }
            let t2 = data[i];
            i += 1;
            (((tag as u16) << 8) | (t2 as u16), 2)
        } else {
            (tag as u16, 1)
        };
        let _ = tag_len_bytes;

        if i >= data.len() {
            break;
        }

        let len = if data[i] < 0x80 {
            let l = data[i] as usize;
            i += 1;
            l
        } else if data[i] == 0x81 {
            if i + 1 >= data.len() {
                break;
            }
            let l = data[i + 1] as usize;
            i += 2;
            l
        } else if data[i] == 0x82 {
            if i + 2 >= data.len() {
                break;
            }
            let l = ((data[i + 1] as usize) << 8) | (data[i + 2] as usize);
            i += 3;
            l
        } else {
            break;
        };

        if full_tag == 0x6E {
            // Container tag, descend into value
            continue;
        }

        if full_tag == 0x4F && len >= 14 && i + len <= data.len() {
            // Tag 0x4F: 16-byte AID
            let aid_slice = &data[i..i + len];
            version = (aid_slice[6], aid_slice[7]);
            manufacturer = u16::from_be_bytes([aid_slice[8], aid_slice[9]]);
            serial.copy_from_slice(&aid_slice[10..14]);
            found_aid = true;
        } else if full_tag == 0x5B && i + len <= data.len() {
            // Tag 0x5B: Cardholder Name
            if let Ok(name_str) = core::str::from_utf8(&data[i..i + len]) {
                let _ = cardholder_name.push_str(name_str);
            }
        }

        i += len;
    }

    if found_aid {
        Some(OpenPgpCardInfo {
            version,
            manufacturer,
            serial,
            cardholder_name,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openpgp_aid_and_name() {
        // Tag 0x6E len 0x1A:
        // Tag 0x4F len 0x10 (16 bytes): [D2 76 00 01 24 01, ver=03 04, mfg=00 06, ser=01 02 03 04, rfu=00 00]
        // Tag 0x5B len 0x05: "ALICE"
        let tlv = [
            0x6E, 0x19, //
            0x4F, 0x10, //
            0xD2, 0x76, 0x00, 0x01, 0x24, 0x01, 0x03, 0x04, 0x00, 0x06, 0x01, 0x02, 0x03, 0x04,
            0x00, 0x00, //
            0x5B, 0x05, 0x41, 0x4C, 0x49, 0x43, 0x45,
        ];
        let info = parse_openpgp_app_data(&tlv).unwrap();
        assert_eq!(info.version, (3, 4));
        assert_eq!(info.manufacturer, 0x0006);
        assert_eq!(manufacturer_name(info.manufacturer), "Yubico");
        assert_eq!(info.serial, [0x01, 0x02, 0x03, 0x04]);
        assert_eq!(info.cardholder_name.as_str(), "ALICE");
    }
}
