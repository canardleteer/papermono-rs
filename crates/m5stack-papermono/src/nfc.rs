//! ST25R3916 NFC on PaperMono (`C153`) only.
//!
//! Do not construct this module from a Lite image. Official Lite HTML
//! PinMap omits NFC; leftover pads: hardware skill `nyc-lite-nfc-pads`.
//!
//! Re-exports the MCU-agnostic [`st25r3916`] chip driver crate and defines
//! PaperMono board-specific nets.

pub use st25r3916::*;

/// NFC IRQ (GPIO6). PaperMono (`C153`) HTML PinMap.
pub const IRQ: u8 = 6;

/// 7-bit I2C address on the system I2C bus (`0x50`).
///
/// ST25R3916 Section 4.3.4 "I2C interface" (`50h`).
/// Schematic `I2C_EN=VDD`.
pub const ADDRESS: u8 = 0x50;

/// M5IOE1 `PYG4` (`M5IOE1_PIN_4` / `PYB_NFC_EN`): NFC power enable.
///
/// Must be driven high to power the ST25R3916. On PaperMono-Lite (`C153-Lite`),
/// this line is an unpopulated leftover pin.
pub const IOE1_ENABLE: u8 = 4;

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    #[test]
    fn nfc_is_not_a_lite_system_address() {
        assert_ne!(ADDRESS, crate::addresses::FT6336G);
        assert_ne!(ADDRESS, crate::addresses::M5IOE1);
        assert_ne!(ADDRESS, crate::addresses::M5PM1);
        assert_ne!(ADDRESS, crate::addresses::IP2315);
        assert_ne!(ADDRESS, crate::addresses::BMI270);
        assert_ne!(ADDRESS, crate::addresses::RX8130CE);
    }

    #[test]
    fn identity_command_encodes_register_3f() {
        assert_eq!(CMD_READ_IC_IDENTITY, 0x7F);
        assert_eq!(register_read_cmd(REG_IC_IDENTITY), 0x7F);
        assert_eq!(register_read_cmd(REG_OP_CONTROL), 0x42);
        assert_eq!(register_read_cmd(REG_FIFO_STATUS1), 0x5E);
    }

    #[test]
    fn parse_st25r3916_identity() {
        let default_ident = IcIdentity::from_byte(0x2A);
        assert_eq!(default_ident.ic_type, 5);
        assert_eq!(default_ident.ic_rev, 2);
        assert!(default_ident.is_st25r3916());

        let rev3_ident = IcIdentity::from_byte((5 << 3) | 3);
        assert!(rev3_ident.is_st25r3916());

        let foreign_ident = IcIdentity::from_byte(0x00);
        assert!(!foreign_ident.is_st25r3916());
    }

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
    fn st25r3916_read_identity_mock() {
        let txns = [Transaction::write_read(
            ADDRESS,
            std::vec![CMD_READ_IC_IDENTITY],
            std::vec![0x2A],
        )];
        let i2c = Mock::new(&txns);
        let mut st = St25r3916::new(i2c, ADDRESS);
        let id = st.read_identity().unwrap();
        assert_eq!(id.ic_type, 5);
        assert_eq!(id.ic_rev, 2);
        st.release().done();
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
        let mut st = St25r3916::new(i2c, ADDRESS);
        st.enable_field().unwrap();
        st.disable_field().unwrap();
        st.release().done();
    }
}
