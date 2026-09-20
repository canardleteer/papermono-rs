//! NIST SP 800-73-4: Personal Identity Verification (PIV) smart card interface over ISO-DEP.
//!
//! Citations:
//! - NIST Special Publication 800-73-4: "Interfaces for Personal Identity Verification":
//!   - Part 1: "PIV Card Application Namespace, Data Model and Representation"
//!   - Part 2: "PIV Card Application Card Command Interface"
//!     - Section 3.1: "SELECT Command"
//!     - Section 3.2: "GET DATA Command" (`INS = 0xCB`)
//!     - Table 2: "PIV Card Application AID" (`A0 00 00 03 08 00 00 10 00 01 00`)

use embedded_hal::i2c::I2c;

use crate::isodep::{IsoDepError, IsoDepSession, MAX_ISODEP_FRAME};
use crate::St25r3916;

/// NIST SP 800-73-4 Table 2: PIV Card Application AID (`A0 00 00 03 08 00 00 10 00 01 00`).
pub const AID_PIV: [u8; 11] = [
    0xA0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00, 0x01, 0x00,
];

/// NIST SP 800-73-4 Section 2.2: PIV Card Application right-truncated AID (`A0 00 00 03 08 00 00 10 00`).
pub const AID_PIV_TRUNCATED: [u8; 9] = [0xA0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00];

/// SELECT APDU command targeting full PIV AID (16 bytes, no Le).
pub const CMD_SELECT_PIV: [u8; 16] = [
    0x00, 0xA4, 0x04, 0x00, 0x0B, 0xA0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00, 0x01, 0x00,
];

/// SELECT APDU command targeting truncated PIV AID (14 bytes, no Le).
pub const CMD_SELECT_PIV_TRUNCATED: [u8; 14] = [
    0x00, 0xA4, 0x04, 0x00, 0x09, 0xA0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00,
];

/// GET DATA instruction (`0xCB`).
pub const INS_GET_DATA: u8 = 0xCB;

/// GET DATA command for Cardholder Unique Identifier (CHUID, Object ID `0x5FC102`, 10 bytes, no Le).
pub const CMD_GET_DATA_CHUID: [u8; 10] =
    [0x00, 0xCB, 0x3F, 0xFF, 0x05, 0x5C, 0x03, 0x5F, 0xC1, 0x02];

/// Discovered metadata from a PIV card application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PivCardInfo {
    /// 16-byte Card UUID (GUID) extracted from CHUID Tag `0x34`.
    pub card_uuid: Option<[u8; 16]>,
    /// Whether the CHUID data container was successfully read.
    pub chuid_present: bool,
}

/// Attempts to select and probe the NIST PIV card application over an active ISO-DEP session,
/// returning the discovered metadata and the raw response status word.
pub fn probe_piv_detail<I2C: I2c>(
    session: &mut IsoDepSession,
    st: &mut St25r3916<I2C>,
) -> (Option<PivCardInfo>, u16) {
    let mut resp = [0u8; MAX_ISODEP_FRAME];
    let res = session.transceive_apdu_auto_get_response(st, &CMD_SELECT_PIV, &mut resp);
    let sw = match res {
        Ok((_len, sw)) if sw == 0x9000 || (sw >> 8) == 0x61 => sw,
        _ => {
            match session.transceive_apdu_auto_get_response(
                st,
                &CMD_SELECT_PIV_TRUNCATED,
                &mut resp,
            ) {
                Ok((_len, sw)) => sw,
                Err(_) => return (None, 0x0000),
            }
        }
    };

    if sw != 0x9000 && (sw >> 8) != 0x61 {
        return (None, sw);
    }

    let mut info = PivCardInfo {
        card_uuid: None,
        chuid_present: false,
    };

    // Query CHUID (Cardholder Unique Identifier)
    if let Ok((chuid_len, chuid_sw)) =
        session.transceive_apdu_auto_get_response(st, &CMD_GET_DATA_CHUID, &mut resp)
    {
        if (chuid_sw == 0x9000 || (chuid_sw >> 8) == 0x61) && chuid_len > 0 {
            info.chuid_present = true;
            info.card_uuid = extract_chuid_uuid(&resp[..chuid_len]);
        }
    }

    (Some(info), sw)
}

/// Attempts to select and probe the NIST PIV card application over an active ISO-DEP session.
pub fn probe_piv<I2C: I2c>(
    session: &mut IsoDepSession,
    st: &mut St25r3916<I2C>,
) -> Result<Option<PivCardInfo>, IsoDepError<I2C::Error>> {
    let (info, _sw) = probe_piv_detail(session, st);
    Ok(info)
}

/// Extracts the 16-byte Card UUID from CHUID BER-TLV payload (Tag `0x34`).
pub fn extract_chuid_uuid(data: &[u8]) -> Option<[u8; 16]> {
    let mut i = 0;
    while i < data.len() {
        // Tag parsing (single byte or two-byte tag)
        let tag = data[i];
        i += 1;
        if i >= data.len() {
            break;
        }

        // Length parsing (short or long form)
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

        if tag == 0x34 && len == 16 && i + 16 <= data.len() {
            let mut uuid = [0u8; 16];
            uuid.copy_from_slice(&data[i..i + 16]);
            return Some(uuid);
        }

        // If composite tag (e.g. 0x53), enter it, otherwise advance
        if tag == 0x53 {
            // Container tag, continue scanning inside value
            continue;
        }

        i += len;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_chuid_uuid() {
        // Encapsulated CHUID: Tag 0x53 len 0x14, Tag 0x34 len 0x10 (16 bytes), UUID
        let fake_uuid = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        let mut tlv = std::vec![0x53, 0x12, 0x34, 0x10];
        tlv.extend_from_slice(&fake_uuid);

        let parsed = extract_chuid_uuid(&tlv).unwrap();
        assert_eq!(parsed, fake_uuid);
    }
}
