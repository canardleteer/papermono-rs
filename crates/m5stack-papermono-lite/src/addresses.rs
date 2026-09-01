//! 7-bit addresses on the **system** I2C bus (SDA [`crate::pins::SYS_I2C_SDA`],
//! SCL [`crate::pins::SYS_I2C_SCL`]).
//!
//! Source: hardware skill pin-map (official pin tables / schematic).
//! Do not assume 400 kHz until measured (`nyc-i2c-ack`).
//!
//! Park [`IP2315`] off this bus except the charge transaction
//! (M5IOE1 [`crate::ioe1::IP2315_I2C_GATE`]). Leaving it mounted can
//! hang the bus.

/// FT6336G touch. Official pin map. **Not in** the public FT6336G PDF.
pub const FT6336G: u8 = 0x38;
/// RX8130CE RTC. Schematic `IIC Adress:0x32`.
pub const RX8130CE: u8 = 0x32;
/// BMI270 IMU. Schematic labels this default (SDO to GND).
pub const BMI270: u8 = 0x68;
/// M5PM1 PMIC. UM V 1.9.
pub const M5PM1: u8 = 0x6E;
/// M5IOE1 on **this board**. Schematic / docs / UserDemo.
///
/// Chip UM V 1.4 lists `0x6F`–`0x76` from IO7. Do not flatten that
/// range onto this constant. Library fallback is [`M5IOE1_UM`].
pub const M5IOE1: u8 = 0x4F;
/// M5IOE1 library default / UM floor. UserDemo `begin(0x4F)` fallback.
///
/// Not a walk of `0x70`–`0x76` (`0x75` is IP2315).
pub const M5IOE1_UM: u8 = 0x6F;
/// IP2315 charger. 8-bit pair `0xEA`/`0xEB`. Gated; see module docs.
pub const IP2315: u8 = 0x75;
/// ST25R3916 leftover on Lite. Probe for NAK only. Do not init the chip.
///
/// Full-SKU address is `m5stack-papermono::nfc::ADDRESS`. Hardware skill
/// Lite leftover: `0x50` must NAK. Full SKU: `nyc-i2c-ack`.
pub const ST25R3916_LEFTOVER: u8 = 0x50;
/// Catalog id `st25r3916`, Device_ID (read-only). Leftover probe only.
///
/// Do not send UserDemo `NFC_READ_IC_IDENTITY_CMD` (`0x7F`).
pub const ST25R3916_LEFTOVER_DEVICE_ID: u8 = 0x00;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_bus_addresses_are_distinct() {
        let mut addresses = [FT6336G, RX8130CE, BMI270, M5PM1, M5IOE1, IP2315];
        addresses.sort_unstable();
        for pair in addresses.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }

    #[test]
    fn ioe1_board_address_is_not_the_chip_um_base() {
        assert_eq!(M5IOE1_UM, 0x6F);
        assert_ne!(M5IOE1, M5IOE1_UM);
        assert_ne!(M5IOE1_UM, IP2315);
    }

    #[test]
    fn leftover_nfc_address_is_sheet_50h() {
        assert_eq!(ST25R3916_LEFTOVER, 0x50);
        assert_eq!(ST25R3916_LEFTOVER_DEVICE_ID, 0x00);
        assert_ne!(ST25R3916_LEFTOVER_DEVICE_ID, 0x7F);
    }
}
