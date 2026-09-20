//! ISO/IEC 14443-4 Section 5: Protocol activation of PICC Type A (RATS and ATS).
//!
//! Citations:
//! - ISO/IEC 14443-4:2008 / 2018:
//!   - Section 5.1: "Request for answer to select"
//!   - Section 5.2: "Answer to select"
//!   - Table 1: "FSDI to FSD conversion"

use super::error::IsoDepErrorKind;

/// RATS command start byte (`0xE0`).
pub const RATS_CMD_START: u8 = 0xE0;

/// Default PCD frame size integer (`8` = 256 bytes per Table 1).
pub const FSDI_256_BYTES: u8 = 8;

/// Converts a 4-bit Frame Size Integer (FSDI or FSCI) into byte capacity per Table 1.
#[must_use]
pub const fn fsi_to_bytes(fsi: u8) -> usize {
    match fsi & 0x0F {
        0 => 16,
        1 => 24,
        2 => 32,
        3 => 40,
        4 => 48,
        5 => 64,
        6 => 96,
        7 => 128,
        _ => 256,
    }
}

/// Constructs a Request for Answer To Select (RATS) command frame.
///
/// Returns a 2-byte frame `[0xE0, param]` where `param = (fsdi << 4) | (cid & 0x0F)`.
#[inline]
#[must_use]
pub const fn build_rats(fsdi: u8, cid: u8) -> [u8; 2] {
    [RATS_CMD_START, ((fsdi & 0x0F) << 4) | (cid & 0x0F)]
}

/// Parsed Answer To Select (ATS) response emitted by a PICC Type A transponder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ats {
    /// Total length byte including `tl` itself.
    pub tl: u8,
    /// Format byte `T0` indicating interface byte presence and `FSCI`.
    pub t0: u8,
    /// Maximum frame size accepted by PICC in bytes.
    pub fsc: usize,
    /// Interface byte `TA(1)` (bit rates), if present.
    pub ta1: Option<u8>,
    /// Interface byte `TB(1)` (FWI and SFGI), if present.
    pub tb1: Option<u8>,
    /// Interface byte `TC(1)` (protocol options: CID/NAD support), if present.
    pub tc1: Option<u8>,
    /// Historical bytes containing operating system / smart card metadata.
    pub historical_bytes: heapless::Vec<u8, 16>,
}

impl Ats {
    /// Parses a raw ATS byte slice received from a PICC.
    pub fn parse(raw: &[u8]) -> Result<Self, IsoDepErrorKind> {
        if raw.is_empty() {
            return Err(IsoDepErrorKind::InvalidAts);
        }
        let tl = raw[0];
        if (tl as usize) > raw.len() || tl < 1 {
            return Err(IsoDepErrorKind::InvalidAts);
        }

        if tl == 1 {
            return Ok(Self {
                tl,
                t0: 0,
                fsc: 32,
                ta1: None,
                tb1: None,
                tc1: None,
                historical_bytes: heapless::Vec::new(),
            });
        }

        let t0 = raw[1];
        let fsci = t0 & 0x0F;
        let fsc = fsi_to_bytes(fsci);

        let has_ta1 = (t0 & 0x10) != 0;
        let has_tb1 = (t0 & 0x20) != 0;
        let has_tc1 = (t0 & 0x40) != 0;

        let mut offset = 2;
        let mut ta1 = None;
        if has_ta1 {
            if offset >= raw.len() {
                return Err(IsoDepErrorKind::InvalidAts);
            }
            ta1 = Some(raw[offset]);
            offset += 1;
        }

        let mut tb1 = None;
        if has_tb1 {
            if offset >= raw.len() {
                return Err(IsoDepErrorKind::InvalidAts);
            }
            tb1 = Some(raw[offset]);
            offset += 1;
        }

        let mut tc1 = None;
        if has_tc1 {
            if offset >= raw.len() {
                return Err(IsoDepErrorKind::InvalidAts);
            }
            tc1 = Some(raw[offset]);
            offset += 1;
        }

        let end = (tl as usize).min(raw.len());
        let mut historical_bytes = heapless::Vec::new();
        if offset < end {
            let hist_slice = &raw[offset..end];
            let len = hist_slice.len().min(16);
            let _ = historical_bytes.extend_from_slice(&hist_slice[..len]);
        }

        Ok(Self {
            tl,
            t0,
            fsc,
            ta1,
            tb1,
            tc1,
            historical_bytes,
        })
    }

    /// Frame Waiting time Integer (FWI) in the range `0..=14`.
    ///
    /// If `TB(1)` is absent, the default value is `4` (~4.8 ms).
    #[inline]
    #[must_use]
    pub fn fwi(&self) -> u8 {
        self.tb1.map_or(4, |b| (b >> 4) & 0x0F)
    }

    /// Computes nominal Frame Waiting Time (FWT) in microseconds.
    ///
    /// Formula: `FWT = (256 * 16 / fc) * 2^FWI` where `fc = 13.56 MHz`.
    /// Base unit `256 * 16 / 13.56 MHz ≈ 302.06 μs`.
    #[inline]
    #[must_use]
    pub fn fwt_us(&self) -> u32 {
        let fwi = self.fwi().min(14);
        302 << fwi
    }

    /// Whether the card supports logical Card Identifiers (CID).
    ///
    /// If `TC(1)` is absent, default is `true` (`b2 = 1`).
    #[inline]
    #[must_use]
    pub fn cid_supported(&self) -> bool {
        self.tc1.is_none_or(|b| (b & 0x02) != 0)
    }

    /// Whether the card supports Node Address (NAD).
    ///
    /// If `TC(1)` is absent, default is `false` (`b1 = 0`).
    #[inline]
    #[must_use]
    pub fn nad_supported(&self) -> bool {
        self.tc1.is_some_and(|b| (b & 0x01) != 0)
    }

    /// Whether the ATS historical bytes identify a YubiKey authenticator.
    ///
    /// YubiKey 5 Series ATS historical bytes (13 bytes):
    /// `[0x80, 0x73, 0xC0, 0x21, 0xC0, 0x57, 0x59, 0x75, 0x62, 0x69, 0x4B, 0x65, 0x79]`
    /// ending in ASCII `"YubiKey"`.
    #[inline]
    #[must_use]
    pub fn is_yubikey(&self) -> bool {
        self.historical_bytes
            .windows(7)
            .any(|w| w == b"YubiKey" || w == b"Yubikey")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rats() {
        let rats = build_rats(FSDI_256_BYTES, 0);
        assert_eq!(rats, [0xE0, 0x80]);

        let rats_cid3 = build_rats(FSDI_256_BYTES, 3);
        assert_eq!(rats_cid3, [0xE0, 0x83]);
    }

    #[test]
    fn test_parse_minimal_ats() {
        let raw = [0x01];
        let ats = Ats::parse(&raw).unwrap();
        assert_eq!(ats.tl, 1);
        assert_eq!(ats.fsc, 32);
        assert_eq!(ats.fwi(), 4);
        assert!(ats.cid_supported());
        assert!(!ats.nad_supported());
    }

    #[test]
    fn test_parse_full_ats_with_historical_bytes() {
        // TL=0x0E (14 bytes), T0=0x78 (TA1, TB1, TC1 present; FSCI=8 -> 256 bytes)
        // TA1=0x80 (same D both directions), TB1=0x80 (FWI=8, SFGI=0), TC1=0x02 (CID=1, NAD=0)
        // Historical: [0x80, 0x4F, 0x0C, 0xA0, 0x00, 0x00, 0x03, 0x06]
        let raw = [
            0x0E, 0x78, 0x80, 0x80, 0x02, 0x80, 0x4F, 0x0C, 0xA0, 0x00, 0x00, 0x03, 0x06, 0x03,
        ];
        let ats = Ats::parse(&raw).unwrap();
        assert_eq!(ats.tl, 14);
        assert_eq!(ats.fsc, 256);
        assert_eq!(ats.ta1, Some(0x80));
        assert_eq!(ats.tb1, Some(0x80));
        assert_eq!(ats.tc1, Some(0x02));
        assert_eq!(ats.fwi(), 8);
        assert!(ats.cid_supported());
        assert!(!ats.nad_supported());
        assert_eq!(ats.historical_bytes.len(), 9);
        assert_eq!(ats.historical_bytes[0], 0x80);
        assert!(!ats.is_yubikey());
    }

    #[test]
    fn test_parse_yubikey_historical_bytes() {
        // YubiKey 5 NFC ATS
        let raw = [
            0x12, 0x78, 0x80, 0x80, 0x02, // TL=18, T0, TA1, TB1, TC1
            0x80, 0x73, 0xC0, 0x21, 0xC0, 0x57, 0x59, 0x75, 0x62, 0x69, 0x4B, 0x65,
            0x79, // "YubiKey"
        ];
        let ats = Ats::parse(&raw).unwrap();
        assert_eq!(ats.fwi(), 8);
        assert_eq!(ats.fsc, 256);
        assert!(ats.is_yubikey());
    }
}
