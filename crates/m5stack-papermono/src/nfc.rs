//! ST25R3916 NFC on PaperMono (`C153`) only.
//!
//! Do not construct this module from a Lite image. Official Lite HTML
//! PinMap omits NFC; leftover pads: hardware skill `nyc-lite-nfc-pads`.
//! UserDemo probes this address at runtime on one ELF.

/// NFC IRQ (GPIO6). Full SKU HTML PinMap.
pub const IRQ: u8 = 6;

/// 7-bit I2C address. Sheet `50h`. Schematic `I2C_EN=VDD`.
pub const ADDRESS: u8 = 0x50;

/// M5IOE1 `PYG4` (`M5IOE1_PIN_4`): NFC enable.
pub const IOE1_ENABLE: u8 = 4;

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
}
