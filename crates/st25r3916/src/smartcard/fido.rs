//! FIDO CTAP1 / CTAP2 Authenticator over NFC.
//!
//! Citations:
//! - FIDO Alliance: "FIDO 2.0: Client to Authenticator Protocol (CTAP) Implementation Draft"
//!   - Section 11: "NFC Protocol"
//!   - Section 11.2: "Framing" (NFCCTAP_MSG `0x80 0x10 0x00 0x00`)
//!   - Section 5.4: "authenticatorGetInfo (0x04)"
//! - ISO/IEC 7816-4: "Identification cards — Integrated circuit cards — Part 4: Organization, security and commands for interchange"

use embedded_hal::i2c::I2c;

use crate::isodep::{IsoDepError, IsoDepSession, MAX_ISODEP_FRAME};
use crate::St25r3916;

/// FIDO Alliance Application Identifier (AID) for U2F / FIDO2 (`A0 00 00 06 47 2F 00 01`).
pub const AID_FIDO: [u8; 8] = [0xA0, 0x00, 0x00, 0x06, 0x47, 0x2F, 0x00, 0x01];

/// SELECT APDU command bytes targeting the FIDO application (13 bytes, without Le per CTAP 11.2.1).
pub const CMD_SELECT_FIDO: [u8; 13] = [
    0x00, 0xA4, 0x04, 0x00, 0x08, 0xA0, 0x00, 0x00, 0x06, 0x47, 0x2F, 0x00, 0x01,
];

/// NFC CTAP message class (`0x80`).
pub const CLA_CTAP: u8 = 0x80;

/// NFC CTAP message instruction (`0x10`).
pub const INS_NFCCTAP_MSG: u8 = 0x10;

/// CTAP2 `authenticatorGetInfo` command opcode (`0x04`).
pub const CTAP2_CMD_GET_INFO: u8 = 0x04;

/// CTAP response status code for success (`0x00`).
pub const CTAP_STATUS_OK: u8 = 0x00;

/// Discovered metadata from a contactless FIDO authenticator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidoCardInfo {
    /// Primary version string (e.g. `"U2F_V2"`, `"FIDO_2_0"`, `"FIDO_2_1"`).
    pub version: heapless::String<16>,
    /// Authenticator Attestation GUID (AAGUID) 16-byte identifier, if CTAP2 supported.
    pub aaguid: Option<[u8; 16]>,
    /// Whether the authenticator supports Resident Keys (rk).
    pub resident_key: bool,
    /// Whether the authenticator has a Client PIN configured.
    pub client_pin: bool,
    /// Whether the authenticator supports User Verification (uv).
    pub user_verification: bool,
}

impl FidoCardInfo {
    /// Whether this card supports CTAP2 (e.g. starts with "FIDO_2").
    #[must_use]
    pub fn is_fido2(&self) -> bool {
        self.version.starts_with("FIDO_2")
    }
}

/// Attempts to select and probe the FIDO CTAP authenticator application over an active ISO-DEP session,
/// returning the discovered metadata and the raw response status word.
pub fn probe_fido_detail<I2C: I2c>(
    session: &mut IsoDepSession,
    st: &mut St25r3916<I2C>,
) -> (Option<FidoCardInfo>, u16) {
    let mut resp = [0u8; MAX_ISODEP_FRAME];
    let res = session.transceive_apdu_auto_get_response(st, &CMD_SELECT_FIDO, &mut resp);
    let (len, sw) = match res {
        Ok(pair) => pair,
        Err(_) => {
            // Retry with Le=00 if Case 3 without Le was rejected
            let cmd_with_le = [
                0x00, 0xA4, 0x04, 0x00, 0x08, 0xA0, 0x00, 0x00, 0x06, 0x47, 0x2F, 0x00, 0x01, 0x00,
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

    // FIDO SELECT response is a UTF-8 version string (e.g. "U2F_V2" or "FIDO_2_0")
    let raw_version = &resp[..len];
    let parsed_str = core::str::from_utf8(raw_version).unwrap_or("");
    let version_str = if parsed_str.is_empty() {
        "FIDO"
    } else {
        parsed_str
    };
    let mut version = heapless::String::new();
    let _ = version.push_str(version_str);

    let mut info = FidoCardInfo {
        version,
        aaguid: None,
        resident_key: false,
        client_pin: false,
        user_verification: false,
    };

    // Send authenticatorGetInfo (0x04) to probe CTAP2 capabilities
    let get_info_apdu = [
        CLA_CTAP,
        INS_NFCCTAP_MSG,
        0x80, // P1 = 0x80 per FIDO CTAP 11.2 (first/only block)
        0x00,
        0x01,
        CTAP2_CMD_GET_INFO,
        0x00,
    ];
    if let Ok((resp_len, get_info_sw)) =
        session.transceive_apdu_auto_get_response(st, &get_info_apdu, &mut resp)
    {
        if (get_info_sw == 0x9000 || (get_info_sw >> 8) == 0x61)
            && resp_len > 1
            && resp[0] == CTAP_STATUS_OK
        {
            let cbor_bytes = &resp[1..resp_len];
            if let Ok(cbor_info) =
                cbor_smol::cbor_deserialize::<ctap_types::ctap2::get_info::Response>(cbor_bytes)
            {
                let mut aaguid = [0u8; 16];
                aaguid.copy_from_slice(cbor_info.aaguid.as_ref());
                info.aaguid = Some(aaguid);
                if let Some(opts) = cbor_info.options {
                    info.resident_key = opts.rk;
                    info.client_pin = opts.client_pin.unwrap_or(false);
                    info.user_verification = opts.uv.unwrap_or(false);
                }
            }
        }
    }

    (Some(info), sw)
}

/// Attempts to select and probe the FIDO CTAP authenticator application over an active ISO-DEP session.
pub fn probe_fido<I2C: I2c>(
    session: &mut IsoDepSession,
    st: &mut St25r3916<I2C>,
) -> Result<Option<FidoCardInfo>, IsoDepError<I2C::Error>> {
    let (info, _sw) = probe_fido_detail(session, st);
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fido_aid_constants() {
        assert_eq!(AID_FIDO.len(), 8);
        assert_eq!(CMD_SELECT_FIDO[4], 8);
        assert_eq!(&CMD_SELECT_FIDO[5..13], &AID_FIDO);
    }
}
