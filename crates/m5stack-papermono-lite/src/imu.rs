//! BMI270 on the system I2C bus.

/// 7-bit address. Same as [`crate::addresses::BMI270`].
pub const ADDRESS: u8 = crate::addresses::BMI270;

/// Catalog id `bmi270`, section 5.2.1 Register (0x00) `CHIP_ID`.
///
/// POR value is `0x24`. This is the **register address**, not the
/// ID byte.
pub const CHIP_ID: u8 = 0x00;

/// POR `CHIP_ID` payload. Catalog id `bmi270`, section 5.2.1.
pub const CHIP_ID_VALUE: u8 = 0x24;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_id_register_is_not_the_payload() {
        assert_eq!(ADDRESS, crate::addresses::BMI270);
        assert_eq!(CHIP_ID, 0x00);
        assert_eq!(CHIP_ID_VALUE, 0x24);
        assert_ne!(CHIP_ID, CHIP_ID_VALUE);
    }
}
