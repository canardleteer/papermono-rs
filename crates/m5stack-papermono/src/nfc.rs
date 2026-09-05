//! ST25R3916 NFC on PaperMono (`C153`) only.
//!
//! Do not construct this module from a Lite image. Official Lite HTML
//! PinMap omits NFC; leftover pads: hardware skill `nyc-lite-nfc-pads`.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet (catalog `st25r3916`, DS12484 Rev 8):
//!   - Section 4.3.4: "I2C interface"
//!   - Section 4.5.3: "Operation control register"
//!   - Section 4.5.80: "IC identity register"
//! - Official factory demo firmware
//!   ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))

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

/// ST25R3916 Section 4.5.3 "Operation control register" address (`0x02`).
pub const REG_OP_CONTROL: u8 = 0x02;

/// ST25R3916 Section 4.5.80 "IC identity register" address (`0x3F`).
pub const REG_IC_IDENTITY: u8 = 0x3F;

/// ST25R3916 Section 4.3.4 "I2C interface" Register Read mode command prefix (`0x40`).
///
/// Register Read mode bytes use bits `0b01xxxxxx` where `xxxxxx` is the 6-bit register address.
pub const READ_MODE_PREFIX: u8 = 0x40;

/// Register Read command byte for the IC identity register (`0x7F`).
///
/// Equivalent to `READ_MODE_PREFIX | REG_IC_IDENTITY`. Matches the
/// `NFC_READ_IC_IDENTITY_CMD` constant in the official factory demo firmware
/// ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)).
pub const CMD_READ_IC_IDENTITY: u8 = READ_MODE_PREFIX | REG_IC_IDENTITY;

/// ST25R3916 Section 4.5.80 "IC identity register" 5-bit IC type code for ST25R3916/7 (`0b00101` = `0x05`).
pub const IC_TYPE_ST25R3916: u8 = 0x05;

/// Operation control register bit 7: oscillator and regulator enable (`en`).
pub const OP_CONTROL_EN: u8 = 1 << 7;

/// Operation control register bit 6: receiver enable (`rx_en`).
pub const OP_CONTROL_RX_EN: u8 = 1 << 6;

/// Operation control register bit 3: transmitter enable (`tx_en`).
pub const OP_CONTROL_TX_EN: u8 = 1 << 3;

/// Operation control register bit 2: wake-up mode enable (`wu`).
pub const OP_CONTROL_WU: u8 = 1 << 2;

/// Generates a single-byte I2C register read command for an ST25R3916 6-bit register address.
///
/// ST25R3916 Section 4.3.4 Figure 22 "Reading a single byte from a register".
#[inline]
#[must_use]
pub const fn register_read_cmd(reg_addr: u8) -> u8 {
    READ_MODE_PREFIX | (reg_addr & 0x3F)
}

/// Decoded hardware identity returned by reading the ST25R3916 IC identity register (`0x3F`).
///
/// ST25R3916 Section 4.5.80 Table 117 "IC identity register".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IcIdentity {
    /// 5-bit IC type code (bits 7:3).
    pub ic_type: u8,
    /// 3-bit silicon revision code (bits 2:0).
    pub ic_rev: u8,
    /// Raw unparsed byte.
    pub raw: u8,
}

impl IcIdentity {
    /// Parses an IC identity register byte.
    #[inline]
    #[must_use]
    pub const fn from_byte(raw: u8) -> Self {
        Self {
            ic_type: (raw >> 3) & 0x1F,
            ic_rev: raw & 0x07,
            raw,
        }
    }

    /// Returns `true` if the IC type code matches the ST25R3916/7 silicon identifier (`0x05`).
    #[inline]
    #[must_use]
    pub const fn is_st25r3916(&self) -> bool {
        self.ic_type == IC_TYPE_ST25R3916
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn parse_st25r3916_identity() {
        // Table 117 default: 00101 (type 5) 010 (rev 2) -> 0x2A
        let default_ident = IcIdentity::from_byte(0x2A);
        assert_eq!(default_ident.ic_type, 5);
        assert_eq!(default_ident.ic_rev, 2);
        assert!(default_ident.is_st25r3916());

        // Any rev of type 5 is valid ST25R3916
        let rev3_ident = IcIdentity::from_byte((5 << 3) | 3);
        assert!(rev3_ident.is_st25r3916());

        // Non-matching type
        let foreign_ident = IcIdentity::from_byte(0x00);
        assert!(!foreign_ident.is_st25r3916());
    }

    #[test]
    fn operation_control_register_bits() {
        let all_off = 0u8;
        assert_eq!(all_off & OP_CONTROL_TX_EN, 0);
        assert_eq!(all_off & OP_CONTROL_EN, 0);

        let rx_only = OP_CONTROL_EN | OP_CONTROL_RX_EN;
        assert_eq!(rx_only & OP_CONTROL_TX_EN, 0);
    }
}
