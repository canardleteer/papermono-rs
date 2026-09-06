//! Passive target memory (PT_Memory) layout and chunked loader commands.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet DS12484 Rev 8:
//!   - Section 2.2.17: "Passive target memory"
//!   - Section 2.2.17 Table 7: "PT_Memory address space"
//!   - Section 2.2.17 Table 8: "NFC-212/424k SENS_RES format"
//!   - Section 4.3.4 Table 11: "SPI operation modes" / "I2C interface"

use crate::commands::{
    MODE_PT_MEM_A_CONFIG, MODE_PT_MEM_F_CONFIG, MODE_PT_MEM_READ, MODE_PT_MEM_TSN,
};

/// Total size of the ST25R3916 passive target memory in bytes (48 bytes).
pub const PT_MEMORY_SIZE: usize = 48;

/// Starting offset of NFC-A configuration in PT_Memory (index 0).
pub const A_CONFIG_OFFSET: usize = 0;

/// Length of NFC-A configuration in PT_Memory (15 bytes: locations 0..14).
pub const A_CONFIG_LEN: usize = 15;

/// Starting offset of NFC-F configuration in PT_Memory (index 15).
pub const F_CONFIG_OFFSET: usize = 15;

/// Length of NFC-F configuration in PT_Memory (21 bytes: locations 15..35).
pub const F_CONFIG_LEN: usize = 21;

/// Starting offset of Time Slot Number (TSN) random slots in PT_Memory (index 36).
pub const TSN_OFFSET: usize = 36;

/// Length of TSN random slot storage in PT_Memory (12 bytes: locations 36..47).
pub const TSN_LEN: usize = 12;

/// Standard response code for NFC-212/424k SENSF_RES byte 1 (`0x01`).
pub const SENSF_RES_BYTE1: u8 = 0x01;

/// SENSF_RES parameters for NFC-F anticollision according to Section 2.2.17 Table 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NfcFParams {
    /// 2-byte System Code (e.g. `[0xFF, 0xFF]` for wildcard or `[0x12, 0xFC]`).
    pub system_code: [u8; 2],
    /// 8-byte Manufacturer ID and Card ID (`NFCID2`).
    pub nfcid2: [u8; 8],
    /// 2-byte `PAD0` field.
    pub pad0: [u8; 2],
    /// 3-byte `PAD1` field.
    pub pad1: [u8; 3],
    /// Maximum response time for Check command (`MRTICHECK`).
    pub mrti_check: u8,
    /// Maximum response time for Update command (`MRTIUPDATE`).
    pub mrti_update: u8,
    /// 1-byte `PAD2` field.
    pub pad2: u8,
}

impl Default for NfcFParams {
    #[inline]
    fn default() -> Self {
        Self {
            system_code: [0xFF, 0xFF],
            nfcid2: [0x02, 0xFE, 0, 0, 0, 0, 0, 0],
            pad0: [0x00, 0x00],
            pad1: [0x00, 0x00, 0x00],
            mrti_check: 0x00,
            mrti_update: 0x00,
            pad2: 0x00,
        }
    }
}

/// 48-byte Passive Target Memory buffer according to Section 2.2.17 Table 7.
///
/// Stores autonomous response data for NFC-A anticollision (NFCID1, SENS_RES,
/// SELR SAK bytes), NFC-F anticollision (System Code, NFCID2, PAD bytes, MRTI),
/// and TSN random time slot numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtMemory {
    bytes: [u8; PT_MEMORY_SIZE],
}

impl Default for PtMemory {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl PtMemory {
    /// Creates an empty PT_Memory initialized to zero.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bytes: [0u8; PT_MEMORY_SIZE],
        }
    }

    /// Creates PT_Memory pre-configured for single-size (4-byte) NFC-A UID (Type 2 Tag).
    ///
    /// - `uid`: 4-byte NFCID1 (written to bytes 0..4).
    /// - `sens_res`: 2-byte ATQA response (written to bytes 10..12).
    /// - `sak`: Level 1 SAK byte (written to byte 12).
    #[must_use]
    pub const fn with_nfc_a_single(uid: [u8; 4], sens_res: [u8; 2], sak: u8) -> Self {
        let mut mem = Self::new();
        mem.bytes[0] = uid[0];
        mem.bytes[1] = uid[1];
        mem.bytes[2] = uid[2];
        mem.bytes[3] = uid[3];
        mem.bytes[10] = sens_res[0];
        mem.bytes[11] = sens_res[1];
        mem.bytes[12] = sak;
        mem
    }

    /// Creates PT_Memory pre-configured for double-size (7-byte) NFC-A UID (Type 4A Tag).
    ///
    /// - `uid`: 7-byte NFCID1 (written to bytes 0..7).
    /// - `sens_res`: 2-byte ATQA response (written to bytes 10..12).
    /// - `sak1`: Level 1 SAK byte with cascade bit set (`0x04`).
    /// - `sak2`: Level 2 SAK byte (e.g. `0x20` for ISO-DEP).
    #[must_use]
    pub const fn with_nfc_a_double(uid: [u8; 7], sens_res: [u8; 2], sak1: u8, sak2: u8) -> Self {
        let mut mem = Self::new();
        mem.bytes[0] = uid[0];
        mem.bytes[1] = uid[1];
        mem.bytes[2] = uid[2];
        mem.bytes[3] = uid[3];
        mem.bytes[4] = uid[4];
        mem.bytes[5] = uid[5];
        mem.bytes[6] = uid[6];
        mem.bytes[10] = sens_res[0];
        mem.bytes[11] = sens_res[1];
        mem.bytes[12] = sak1;
        mem.bytes[13] = sak2;
        mem
    }

    /// Appends FeliCa / NFC-F configuration (bytes 15..35) according to Section 2.2.17 Table 8.
    #[must_use]
    pub const fn with_nfc_f(mut self, params: &NfcFParams) -> Self {
        // Bytes 15, 16: System code
        self.bytes[15] = params.system_code[0];
        self.bytes[16] = params.system_code[1];
        // Byte 17: SENSF_RES Byte 1 (always 0x01)
        self.bytes[17] = SENSF_RES_BYTE1;
        // Bytes 18..26: NFCID2 (8 bytes)
        self.bytes[18] = params.nfcid2[0];
        self.bytes[19] = params.nfcid2[1];
        self.bytes[20] = params.nfcid2[2];
        self.bytes[21] = params.nfcid2[3];
        self.bytes[22] = params.nfcid2[4];
        self.bytes[23] = params.nfcid2[5];
        self.bytes[24] = params.nfcid2[6];
        self.bytes[25] = params.nfcid2[7];
        // Bytes 26..28: PAD0 (2 bytes)
        self.bytes[26] = params.pad0[0];
        self.bytes[27] = params.pad0[1];
        // Bytes 28..31: PAD1 (3 bytes)
        self.bytes[28] = params.pad1[0];
        self.bytes[29] = params.pad1[1];
        self.bytes[30] = params.pad1[2];
        // Byte 31: MRTICHECK
        self.bytes[31] = params.mrti_check;
        // Byte 32: MRTIUPDATE
        self.bytes[32] = params.mrti_update;
        // Byte 33: PAD2
        self.bytes[33] = params.pad2;
        // Bytes 34, 35: RD (RFU, default 0x00)
        self.bytes[34] = 0x00;
        self.bytes[35] = 0x00;
        self
    }

    /// Appends TSN random numbers (bytes 36..47, 12 bytes storing 24 4-bit random numbers).
    #[must_use]
    pub const fn with_tsn(mut self, tsn_slots: [u8; TSN_LEN]) -> Self {
        let mut i = 0;
        while i < TSN_LEN {
            self.bytes[TSN_OFFSET + i] = tsn_slots[i];
            i += 1;
        }
        self
    }

    /// Returns the underlying 48-byte buffer.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; PT_MEMORY_SIZE] {
        &self.bytes
    }

    /// Returns a slice of the 15-byte NFC-A configuration section.
    #[inline]
    #[must_use]
    pub fn a_config(&self) -> &[u8] {
        &self.bytes[A_CONFIG_OFFSET..A_CONFIG_OFFSET + A_CONFIG_LEN]
    }

    /// Returns a slice of the 21-byte NFC-F configuration section.
    #[inline]
    #[must_use]
    pub fn f_config(&self) -> &[u8] {
        &self.bytes[F_CONFIG_OFFSET..F_CONFIG_OFFSET + F_CONFIG_LEN]
    }

    /// Returns a slice of the 12-byte TSN slot section.
    #[inline]
    #[must_use]
    pub fn tsn(&self) -> &[u8] {
        &self.bytes[TSN_OFFSET..TSN_OFFSET + TSN_LEN]
    }
}

impl<I2C: embedded_hal::i2c::I2c> crate::St25r3916<I2C> {
    /// Loads the NFC-A configuration portion (index 0..14) into PT_Memory.
    ///
    /// Emits operation mode prefix `0xA0` followed by 15 data bytes per Section 4.3.4 Table 11.
    pub fn load_pt_memory_a(&mut self, mem: &PtMemory) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1 + A_CONFIG_LEN];
        buf[0] = MODE_PT_MEM_A_CONFIG;
        buf[1..].copy_from_slice(mem.a_config());
        self.i2c.write(self.address, &buf)
    }

    /// Loads the NFC-F configuration portion (index 15..35) into PT_Memory.
    ///
    /// Emits operation mode prefix `0xA8` followed by 21 data bytes per Section 4.3.4 Table 11.
    pub fn load_pt_memory_f(&mut self, mem: &PtMemory) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1 + F_CONFIG_LEN];
        buf[0] = MODE_PT_MEM_F_CONFIG;
        buf[1..].copy_from_slice(mem.f_config());
        self.i2c.write(self.address, &buf)
    }

    /// Loads the TSN random slot portion (index 36..47) into PT_Memory.
    ///
    /// Emits operation mode prefix `0xAC` followed by 12 data bytes per Section 4.3.4 Table 11.
    pub fn load_pt_memory_tsn(&mut self, mem: &PtMemory) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1 + TSN_LEN];
        buf[0] = MODE_PT_MEM_TSN;
        buf[1..].copy_from_slice(mem.tsn());
        self.i2c.write(self.address, &buf)
    }

    /// Reads the entire 48-byte PT_Memory content into the provided buffer.
    ///
    /// Emits operation mode byte `0xBF` followed by reading 48 bytes per Section 4.3.4 Table 11.
    pub fn read_pt_memory(&mut self, buf: &mut [u8; PT_MEMORY_SIZE]) -> Result<(), I2C::Error> {
        self.i2c.write_read(self.address, &[MODE_PT_MEM_READ], buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    const ADDR: u8 = 0x50;

    #[test]
    fn pt_memory_single_a_layout() {
        let uid = [0x01, 0x02, 0x03, 0x04];
        let sens_res = [0x44, 0x00];
        let sak = 0x00;
        let mem = PtMemory::with_nfc_a_single(uid, sens_res, sak);

        assert_eq!(&mem.as_bytes()[0..4], &uid);
        assert_eq!(&mem.as_bytes()[4..10], &[0, 0, 0, 0, 0, 0]);
        assert_eq!(&mem.as_bytes()[10..12], &sens_res);
        assert_eq!(mem.as_bytes()[12], sak);
        assert_eq!(mem.as_bytes()[13], 0);
        assert_eq!(mem.a_config().len(), A_CONFIG_LEN);
    }

    #[test]
    fn pt_memory_double_a_layout() {
        let uid = [0x04, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        let sens_res = [0x44, 0x03];
        let sak1 = 0x04;
        let sak2 = 0x20;
        let mem = PtMemory::with_nfc_a_double(uid, sens_res, sak1, sak2);

        assert_eq!(&mem.as_bytes()[0..7], &uid);
        assert_eq!(&mem.as_bytes()[7..10], &[0, 0, 0]);
        assert_eq!(&mem.as_bytes()[10..12], &sens_res);
        assert_eq!(mem.as_bytes()[12], sak1);
        assert_eq!(mem.as_bytes()[13], sak2);
    }

    #[test]
    fn pt_memory_f_layout() {
        let params = NfcFParams {
            system_code: [0x12, 0xFC],
            nfcid2: [1, 2, 3, 4, 5, 6, 7, 8],
            pad0: [0xAA, 0xBB],
            pad1: [0xCC, 0xDD, 0xEE],
            mrti_check: 0x05,
            mrti_update: 0x07,
            pad2: 0x11,
        };
        let mem = PtMemory::new().with_nfc_f(&params);

        let f = mem.f_config();
        assert_eq!(f.len(), F_CONFIG_LEN);
        assert_eq!(&f[0..2], &[0x12, 0xFC]);
        assert_eq!(f[2], SENSF_RES_BYTE1);
        assert_eq!(&f[3..11], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(&f[11..13], &[0xAA, 0xBB]);
        assert_eq!(&f[13..16], &[0xCC, 0xDD, 0xEE]);
        assert_eq!(f[16], 0x05);
        assert_eq!(f[17], 0x07);
        assert_eq!(f[18], 0x11);
    }

    #[test]
    fn load_pt_memory_a_transaction() {
        let mem = PtMemory::with_nfc_a_single([1, 2, 3, 4], [0x04, 0], 0);
        let mut expected = std::vec![MODE_PT_MEM_A_CONFIG];
        expected.extend_from_slice(mem.a_config());

        let txns = [Transaction::write(ADDR, expected)];
        let i2c = Mock::new(&txns);
        let mut st = crate::St25r3916::new(i2c, ADDR);
        st.load_pt_memory_a(&mem).unwrap();
        st.release().done();
    }

    #[test]
    fn read_pt_memory_transaction() {
        let dummy = [0x55u8; PT_MEMORY_SIZE];
        let txns = [Transaction::write_read(
            ADDR,
            std::vec![MODE_PT_MEM_READ],
            dummy.to_vec(),
        )];
        let i2c = Mock::new(&txns);
        let mut st = crate::St25r3916::new(i2c, ADDR);
        let mut buf = [0u8; PT_MEMORY_SIZE];
        st.read_pt_memory(&mut buf).unwrap();
        assert_eq!(buf, dummy);
        st.release().done();
    }
}
