//! ST25R3916 ISO/IEC 14443-A Initiator (Reader / Poller) mode.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet DS12484 Rev 8:
//!   - Section 4.3.4: "I2C interface"
//!   - Section 4.4: "Direct commands"
//!   - Section 4.5.3: "Operation control register"
//!   - Section 4.5.4: "Mode definition register"
//!   - Section 4.5.6: "ISO14443A and NFC 106kb/s settings register"
//!   - Section 4.5.11: "Auxiliary definition register"
//!   - Section 4.5.12: "Receiver configuration register 1"
//!   - Section 4.5.13: "Receiver configuration register 2"
//!   - Section 4.5.14: "Receiver configuration register 3"
//!   - Section 4.5.15: "Receiver configuration register 4"
//!   - Section 4.5.34: "Main interrupt register"
//!   - Section 4.5.38: "FIFO status register 1"
//!   - Section 4.5.42: "Number of transmitted bytes register 1"
//!   - Section 4.5.43: "Number of transmitted bytes register 2"
//! - ISO/IEC 14443-3: "Identification cards — Contactless integrated circuit cards — Proximity cards — Part 3: Initialization and anticollision"

use crate::commands::{
    CMD_ADJUST_REGULATORS, CMD_CLEAR_FIFO, CMD_NFC_INITIAL_FIELD_ON, CMD_RESET_RX_GAIN,
    CMD_SET_DEFAULT, CMD_STOP_ALL, CMD_TEST_ACCESS, CMD_TRANSMIT_REQA, CMD_TRANSMIT_WITHOUT_CRC,
    CMD_TRANSMIT_WITH_CRC, CMD_TRANSMIT_WUPA,
};
use crate::registers::{
    AUX_DISPLAY_OSC_OK, IO_CONF2_AAT_EN, IO_CONF2_IO_DRV_LVL, IO_CONF2_SUP3V,
    MODE_INITIATOR_ISO14443A, MODE_NFC_AR8_AUTO, OP_CONTROL_EN, OP_CONTROL_RX_EN, OP_CONTROL_TX_EN,
    REG_ANTENNA_TUNING1, REG_ANTENNA_TUNING2, REG_AUX_DEFINITION, REG_AUX_DISPLAY, REG_BIT_RATE,
    REG_EXT_FIELD_DETECTOR_ACT, REG_EXT_FIELD_DETECTOR_DEACT, REG_IO_CONF1, REG_IO_CONF2,
    REG_ISO14443A_SETTINGS, REG_MAIN_IRQ, REG_MODE_DEFINITION, REG_NUM_TX_BYTES1,
    REG_NUM_TX_BYTES2, REG_OP_CONTROL, REG_RECEIVER_CONF1, REG_RECEIVER_CONF2, REG_RECEIVER_CONF3,
    REG_RECEIVER_CONF4, REG_TX_DRIVER,
};
use embedded_hal::i2c::I2c;

/// ISO14443A and NFC 106kb/s settings register bit 0: anticollision frame (`antcl`).
pub const ISO14443A_ANTCL: u8 = 1 << 0;

/// Auxiliary definition register bit 7: receive without CRC (`no_crc_rx`).
pub const AUX_DEF_NO_CRC_RX: u8 = 1 << 7;

/// Receiver configuration register 1 bit 3: 600k input impedance (`z_600k`).
pub const RX_CONF1_Z600K: u8 = 1 << 3;

/// Receiver configuration register 2 default settings (`0x2D`).
pub const RX_CONF2_DEFAULT: u8 = 0x2D;

/// Receiver configuration register 3 stability-focused gain (`0xD8`).
pub const RX_CONF3_STABILITY: u8 = 0xD8;

/// Receiver configuration register 4 stability-focused gain (`0x22`).
pub const RX_CONF4_STABILITY: u8 = 0x22;

/// TX driver register default 22% AM modulation index (`0xD0`).
pub const TX_DRIVER_DEFAULT: u8 = 0xD0;

/// ISO/IEC 14443-3 Select Command Cascade Level 1 (`0x93`).
pub const ISO14443A_CMD_SEL_CL1: u8 = 0x93;

/// ISO/IEC 14443-3 Select Command Cascade Level 2 (`0x95`).
pub const ISO14443A_CMD_SEL_CL2: u8 = 0x95;

/// ISO/IEC 14443-3 Number of Valid Bits for initial Anticollision (`0x20`).
pub const ISO14443A_NVB_ANTICOLLISION: u8 = 0x20;

/// ISO/IEC 14443-3 Number of Valid Bits for complete Select (`0x70`).
pub const ISO14443A_NVB_SELECT: u8 = 0x70;

/// ISO/IEC 14443-3 Cascade Tag byte (`0x88`) indicating a multi-byte UID.
pub const ISO14443A_CASCADE_TAG: u8 = 0x88;

/// Detected ISO/IEC 14443-A contactless card or synthetic tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iso14443aCard {
    /// 2-byte Answer To Request Type A (ATQA).
    pub atqa: [u8; 2],
    /// 1-byte Select Acknowledge (SAK).
    pub sak: u8,
    /// Length of the extracted UID in bytes (4 for single-size, 7 for double-size).
    pub uid_len: usize,
    /// Extracted Unique Identifier (UID) bytes.
    pub uid: [u8; 10],
}

impl Iso14443aCard {
    /// Returns the active UID bytes as a slice.
    #[inline]
    #[must_use]
    pub fn uid(&self) -> &[u8] {
        &self.uid[..self.uid_len]
    }

    /// Whether this card has a single-size (4-byte) UID.
    #[inline]
    #[must_use]
    pub const fn is_single_size(&self) -> bool {
        self.uid_len == 4
    }

    /// Whether this card has a double-size (7-byte) UID.
    #[inline]
    #[must_use]
    pub const fn is_double_size(&self) -> bool {
        self.uid_len == 7
    }
}

impl<I2C: I2c> crate::St25r3916<I2C> {
    /// Powers up the oscillator, receiver, and transmitter for ISO14443-A polling.
    pub fn enable_field(&mut self) -> Result<(), I2C::Error> {
        let _ = self.direct_cmd(CMD_STOP_ALL);
        self.init_hardware()
    }

    /// Initializes ST25R3916 hardware for ISO14443-A Initiator operation.
    ///
    /// Executes the full power-up, errata workaround, crystal stabilization,
    /// receiver frontend tuning, and RF field energization sequence.
    pub fn init_hardware(&mut self) -> Result<(), I2C::Error> {
        // 1. Reset state
        let _ = self.write_reg(REG_OP_CONTROL, 0x00);
        let _ = self.direct_cmd(CMD_SET_DEFAULT);

        // 2. Errata workaround: prevent internal overheat protection from triggering prematurely.
        let _ = self.i2c.write(self.address, &[CMD_TEST_ACCESS, 0x04, 0x10]);

        // 3. IO & Antenna Configuration
        self.write_reg(REG_IO_CONF1, 0x07)?;
        self.write_reg(
            REG_IO_CONF2,
            IO_CONF2_SUP3V | IO_CONF2_IO_DRV_LVL | IO_CONF2_AAT_EN,
        )?;
        self.write_reg(REG_TX_DRIVER, TX_DRIVER_DEFAULT)?;
        self.write_reg(REG_ANTENNA_TUNING1, 0x82)?;
        self.write_reg(REG_ANTENNA_TUNING2, 0x82)?;
        self.write_reg(REG_EXT_FIELD_DETECTOR_ACT, 0x13)?;
        self.write_reg(REG_EXT_FIELD_DETECTOR_DEACT, 0x02)?;

        // 4. Ready mode
        self.write_reg(REG_OP_CONTROL, OP_CONTROL_EN | 0x03)?;

        // 5. Wait for crystal oscillator stability
        for _ in 0..100 {
            let aux = self.read_reg(REG_AUX_DISPLAY)?;
            if (aux & AUX_DISPLAY_OSC_OK) != 0 {
                break;
            }
        }

        // 6. Adjust regulators
        self.direct_cmd(CMD_ADJUST_REGULATORS)?;

        // 7. Mode definition: ISO14443A Initiator mode
        self.write_reg(
            REG_MODE_DEFINITION,
            MODE_INITIATOR_ISO14443A | MODE_NFC_AR8_AUTO,
        )?;
        self.write_reg(REG_BIT_RATE, 0x00)?;
        self.write_reg(REG_ISO14443A_SETTINGS, 0x00)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;

        // 8. Receiver frontend configuration
        self.write_reg(REG_RECEIVER_CONF1, RX_CONF1_Z600K)?;
        self.write_reg(REG_RECEIVER_CONF2, RX_CONF2_DEFAULT)?;
        self.write_reg(REG_RECEIVER_CONF3, RX_CONF3_STABILITY)?;
        self.write_reg(REG_RECEIVER_CONF4, RX_CONF4_STABILITY)?;
        self.direct_cmd(CMD_RESET_RX_GAIN)?;

        // 9. Field activation
        self.direct_cmd(CMD_NFC_INITIAL_FIELD_ON)?;
        self.write_reg(
            REG_OP_CONTROL,
            OP_CONTROL_EN | OP_CONTROL_RX_EN | OP_CONTROL_TX_EN | 0x03,
        )
    }

    /// Safely halts transmission and shuts down the RF field and oscillator.
    pub fn disable_field(&mut self) -> Result<(), I2C::Error> {
        let _ = self.direct_cmd(CMD_STOP_ALL);
        self.write_reg(REG_OP_CONTROL, 0x00)
    }

    /// Clears the internal FIFO and resets FIFO status flags.
    pub fn clear_fifo(&mut self) -> Result<(), I2C::Error> {
        self.direct_cmd(CMD_CLEAR_FIFO)
    }

    /// Transmits an ISO14443-A WUPA short frame (`Transmit WUPA`, direct command `0xC7`).
    pub fn send_wupa(&mut self) -> Result<(), I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, ISO14443A_ANTCL)?;
        self.write_reg(REG_AUX_DEFINITION, AUX_DEF_NO_CRC_RX)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 0x00)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.clear_fifo()?;
        self.direct_cmd(CMD_TRANSMIT_WUPA)
    }

    /// Transmits an ISO14443-A REQA short frame (`Transmit REQA`, direct command `0xC6`).
    pub fn send_reqa(&mut self) -> Result<(), I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, ISO14443A_ANTCL)?;
        self.write_reg(REG_AUX_DEFINITION, AUX_DEF_NO_CRC_RX)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 0x00)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.clear_fifo()?;
        self.direct_cmd(CMD_TRANSMIT_REQA)
    }

    /// Reads the 2-byte ATQA response from the FIFO if available.
    pub fn read_atqa(&mut self) -> Result<Option<[u8; 2]>, I2C::Error> {
        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 2 {
                break;
            }
            count += 1;
        }
        if n < 2 {
            return Ok(None);
        }
        let mut atqa = [0u8; 2];
        self.read_fifo(&mut atqa)?;
        Ok(Some(atqa))
    }

    /// Performs ISO14443-A Cascade Level 1 anticollision (`0x93`, `0x20`).
    ///
    /// Transmits without CRC with collision detection enabled.
    /// Returns 5 bytes: 4 UID bytes plus 1 BCC byte.
    pub fn anticollision_cl1(&mut self) -> Result<Option<[u8; 5]>, I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, ISO14443A_ANTCL)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 2 << 3)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        self.write_fifo(&[ISO14443A_CMD_SEL_CL1, ISO14443A_NVB_ANTICOLLISION])?;
        self.direct_cmd(CMD_TRANSMIT_WITHOUT_CRC)?;

        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 5 {
                break;
            }
            count += 1;
        }
        if n < 5 {
            return Ok(None);
        }
        let mut resp = [0u8; 5];
        self.read_fifo(&mut resp)?;
        let expected_bcc = resp[0] ^ resp[1] ^ resp[2] ^ resp[3];
        if resp[4] != expected_bcc {
            return Ok(None);
        }
        Ok(Some(resp))
    }

    /// Selects Cascade Level 1 with CRC. Returns SAK byte.
    pub fn select_cl1(&mut self, uid_cl1: [u8; 4], bcc: u8) -> Result<Option<u8>, I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, 0x00)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 7 << 3)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        let payload = [
            ISO14443A_CMD_SEL_CL1,
            ISO14443A_NVB_SELECT,
            uid_cl1[0],
            uid_cl1[1],
            uid_cl1[2],
            uid_cl1[3],
            bcc,
        ];
        self.write_fifo(&payload)?;
        self.direct_cmd(CMD_TRANSMIT_WITH_CRC)?;

        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 1 {
                break;
            }
            count += 1;
        }
        if n < 1 {
            return Ok(None);
        }
        let mut sak = [0u8; 1];
        self.read_fifo(&mut sak)?;
        Ok(Some(sak[0]))
    }

    /// Performs ISO14443-A Cascade Level 2 anticollision (`0x95`, `0x20`).
    ///
    /// Transmits without CRC. Returns 5 bytes: 4 UID bytes plus 1 BCC byte.
    pub fn anticollision_cl2(&mut self) -> Result<Option<[u8; 5]>, I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, ISO14443A_ANTCL)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 2 << 3)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        self.write_fifo(&[ISO14443A_CMD_SEL_CL2, ISO14443A_NVB_ANTICOLLISION])?;
        self.direct_cmd(CMD_TRANSMIT_WITHOUT_CRC)?;

        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 5 {
                break;
            }
            count += 1;
        }
        if n < 5 {
            return Ok(None);
        }
        let mut resp = [0u8; 5];
        self.read_fifo(&mut resp)?;
        let expected_bcc = resp[0] ^ resp[1] ^ resp[2] ^ resp[3];
        if resp[4] != expected_bcc {
            return Ok(None);
        }
        Ok(Some(resp))
    }

    /// Selects Cascade Level 2 with CRC. Returns SAK byte.
    pub fn select_cl2(&mut self, uid_cl2: [u8; 4], bcc: u8) -> Result<Option<u8>, I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, 0x00)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 7 << 3)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        let payload = [
            ISO14443A_CMD_SEL_CL2,
            ISO14443A_NVB_SELECT,
            uid_cl2[0],
            uid_cl2[1],
            uid_cl2[2],
            uid_cl2[3],
            bcc,
        ];
        self.write_fifo(&payload)?;
        self.direct_cmd(CMD_TRANSMIT_WITH_CRC)?;

        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 1 {
                break;
            }
            count += 1;
        }
        if n < 1 {
            return Ok(None);
        }
        let mut sak = [0u8; 1];
        self.read_fifo(&mut sak)?;
        Ok(Some(sak[0]))
    }

    /// Polls for an ISO/IEC 14443-A contactless card.
    pub fn poll_iso14443a(&mut self) -> Result<Option<Iso14443aCard>, I2C::Error> {
        self.send_wupa()?;
        let mut atqa = self.read_atqa()?;
        if atqa.is_none() {
            self.send_reqa()?;
            atqa = self.read_atqa()?;
        }
        let atqa = match atqa {
            Some(a) => a,
            None => {
                self.clear_fifo()?;
                self.write_reg(REG_ISO14443A_SETTINGS, 0x00)?;
                self.write_reg(REG_AUX_DEFINITION, 0x00)?;
                return Ok(None);
            }
        };

        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_ISO14443A_SETTINGS, 0x00)?;

        let cl1 = match self.anticollision_cl1()? {
            Some(c) => c,
            None => return Ok(None),
        };
        let uid_cl1 = [cl1[0], cl1[1], cl1[2], cl1[3]];
        let bcc1 = cl1[4];

        let sak_cl1 = match self.select_cl1(uid_cl1, bcc1)? {
            Some(s) => s,
            None => return Ok(None),
        };

        if (sak_cl1 & 0x04) == 0 {
            let mut full_uid = [0u8; 10];
            full_uid[..4].copy_from_slice(&uid_cl1);
            return Ok(Some(Iso14443aCard {
                atqa,
                sak: sak_cl1,
                uid_len: 4,
                uid: full_uid,
            }));
        }

        if uid_cl1[0] == ISO14443A_CASCADE_TAG {
            let cl2 = match self.anticollision_cl2()? {
                Some(c) => c,
                None => return Ok(None),
            };
            let uid_cl2 = [cl2[0], cl2[1], cl2[2], cl2[3]];
            let bcc2 = cl2[4];

            let sak_cl2 = match self.select_cl2(uid_cl2, bcc2)? {
                Some(s) => s,
                None => return Ok(None),
            };

            let mut full_uid = [0u8; 10];
            full_uid[..3].copy_from_slice(&uid_cl1[1..4]);
            full_uid[3..7].copy_from_slice(&uid_cl2);

            return Ok(Some(Iso14443aCard {
                atqa,
                sak: sak_cl2,
                uid_len: 7,
                uid: full_uid,
            }));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{register_read_cmd, MODE_FIFO_LOAD, MODE_FIFO_READ};
    use crate::registers::REG_FIFO_STATUS1;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    const ADDRESS: u8 = 0x50;

    #[test]
    fn card_uid_and_size_properties() {
        let single = Iso14443aCard {
            atqa: [0x04, 0x00],
            sak: 0x08,
            uid_len: 4,
            uid: [0x08, 0x2C, 0xA1, 0x3F, 0, 0, 0, 0, 0, 0],
        };
        assert!(single.is_single_size());
        assert!(!single.is_double_size());
        assert_eq!(single.uid(), &[0x08, 0x2C, 0xA1, 0x3F]);

        let double = Iso14443aCard {
            atqa: [0x44, 0x00],
            sak: 0x00,
            uid_len: 7,
            uid: [0x04, 0xA2, 0x3B, 0x5C, 0x1D, 0x8E, 0x4A, 0, 0, 0],
        };
        assert!(!double.is_single_size());
        assert!(double.is_double_size());
        assert_eq!(double.uid(), &[0x04, 0xA2, 0x3B, 0x5C, 0x1D, 0x8E, 0x4A]);
    }

    #[test]
    fn st25r3916_field_enable_and_disable_mock() {
        let txns = [
            // enable_field:
            Transaction::write(ADDRESS, std::vec![CMD_STOP_ALL]),
            Transaction::write(ADDRESS, std::vec![REG_OP_CONTROL, 0x00]),
            Transaction::write(ADDRESS, std::vec![CMD_SET_DEFAULT]),
            Transaction::write(ADDRESS, std::vec![CMD_TEST_ACCESS, 0x04, 0x10]),
            Transaction::write(ADDRESS, std::vec![REG_IO_CONF1, 0x07]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    REG_IO_CONF2,
                    IO_CONF2_SUP3V | IO_CONF2_IO_DRV_LVL | IO_CONF2_AAT_EN,
                ],
            ),
            Transaction::write(ADDRESS, std::vec![REG_TX_DRIVER, TX_DRIVER_DEFAULT]),
            Transaction::write(ADDRESS, std::vec![REG_ANTENNA_TUNING1, 0x82]),
            Transaction::write(ADDRESS, std::vec![REG_ANTENNA_TUNING2, 0x82]),
            Transaction::write(ADDRESS, std::vec![REG_EXT_FIELD_DETECTOR_ACT, 0x13]),
            Transaction::write(ADDRESS, std::vec![REG_EXT_FIELD_DETECTOR_DEACT, 0x02]),
            Transaction::write(ADDRESS, std::vec![REG_OP_CONTROL, OP_CONTROL_EN | 0x03]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_AUX_DISPLAY)],
                std::vec![AUX_DISPLAY_OSC_OK],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_ADJUST_REGULATORS]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    REG_MODE_DEFINITION,
                    MODE_INITIATOR_ISO14443A | MODE_NFC_AR8_AUTO,
                ],
            ),
            Transaction::write(ADDRESS, std::vec![REG_BIT_RATE, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_ISO14443A_SETTINGS, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_AUX_DEFINITION, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_RECEIVER_CONF1, RX_CONF1_Z600K]),
            Transaction::write(ADDRESS, std::vec![REG_RECEIVER_CONF2, RX_CONF2_DEFAULT]),
            Transaction::write(ADDRESS, std::vec![REG_RECEIVER_CONF3, RX_CONF3_STABILITY]),
            Transaction::write(ADDRESS, std::vec![REG_RECEIVER_CONF4, RX_CONF4_STABILITY]),
            Transaction::write(ADDRESS, std::vec![CMD_RESET_RX_GAIN]),
            Transaction::write(ADDRESS, std::vec![CMD_NFC_INITIAL_FIELD_ON]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    REG_OP_CONTROL,
                    OP_CONTROL_EN | OP_CONTROL_RX_EN | OP_CONTROL_TX_EN | 0x03,
                ],
            ),
            // disable_field:
            Transaction::write(ADDRESS, std::vec![CMD_STOP_ALL]),
            Transaction::write(ADDRESS, std::vec![REG_OP_CONTROL, 0x00]),
        ];
        let i2c = Mock::new(&txns);
        let mut st = crate::St25r3916::new(i2c, ADDRESS);
        st.enable_field().unwrap();
        st.disable_field().unwrap();
        st.release().done();
    }

    #[test]
    fn st25r3916_single_size_poll_sequence_mock() {
        let txns = [
            // send_reqa:
            Transaction::write(ADDRESS, std::vec![REG_ISO14443A_SETTINGS, ISO14443A_ANTCL]),
            Transaction::write(ADDRESS, std::vec![REG_AUX_DEFINITION, AUX_DEF_NO_CRC_RX]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES2, 0x00]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(ADDRESS, std::vec![CMD_TRANSMIT_REQA]),
            // read_atqa:
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![2],
            ),
            Transaction::write_read(ADDRESS, std::vec![MODE_FIFO_READ], std::vec![0x04, 0x00]),
            // anticollision_cl1:
            Transaction::write(ADDRESS, std::vec![REG_ISO14443A_SETTINGS, ISO14443A_ANTCL]),
            Transaction::write(ADDRESS, std::vec![REG_AUX_DEFINITION, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES2, 2 << 3]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    MODE_FIFO_LOAD,
                    ISO14443A_CMD_SEL_CL1,
                    ISO14443A_NVB_ANTICOLLISION,
                ],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_TRANSMIT_WITHOUT_CRC]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![5],
            ),
            Transaction::write_read(
                ADDRESS,
                std::vec![MODE_FIFO_READ],
                std::vec![0x08, 0x2C, 0xA1, 0x3F, 0x08 ^ 0x2C ^ 0xA1 ^ 0x3F],
            ),
            // select_cl1:
            Transaction::write(ADDRESS, std::vec![REG_ISO14443A_SETTINGS, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_AUX_DEFINITION, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES2, 7 << 3]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    MODE_FIFO_LOAD,
                    ISO14443A_CMD_SEL_CL1,
                    ISO14443A_NVB_SELECT,
                    0x08,
                    0x2C,
                    0xA1,
                    0x3F,
                    0x08 ^ 0x2C ^ 0xA1 ^ 0x3F,
                ],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_TRANSMIT_WITH_CRC]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![1],
            ),
            Transaction::write_read(ADDRESS, std::vec![MODE_FIFO_READ], std::vec![0x08]),
        ];
        let i2c = Mock::new(&txns);
        let mut st = crate::St25r3916::new(i2c, ADDRESS);

        st.send_reqa().unwrap();
        let atqa = st.read_atqa().unwrap().expect("atqa");
        assert_eq!(atqa, [0x04, 0x00]);

        let cl1 = st.anticollision_cl1().unwrap().expect("cl1");
        assert_eq!(&cl1[..4], &[0x08, 0x2C, 0xA1, 0x3F]);

        let sak = st
            .select_cl1([cl1[0], cl1[1], cl1[2], cl1[3]], cl1[4])
            .unwrap()
            .expect("sak");
        assert_eq!(sak, 0x08);

        st.release().done();
    }
}
