//! FT6336G on the system I2C bus.
//!
//! Reset and VDD are expander nets ([`crate::ioe1::TOUCH_RST`],
//! [`crate::ioe1::TOUCH_VDD_ENABLE`]). INT is [`crate::pins::TOUCH_INT`].

/// 7-bit address. Same as [`crate::addresses::FT6336G`].
pub const ADDRESS: u8 = crate::addresses::FT6336G;

/// Touch active-area minimum X (official tables).
pub const ACTIVE_MIN_X: u16 = 5;
/// Touch active-area maximum X (official tables).
pub const ACTIVE_MAX_X: u16 = 475;
/// Touch active-area minimum Y (official tables).
pub const ACTIVE_MIN_Y: u16 = 5;
/// Touch active-area maximum Y (official tables).
pub const ACTIVE_MAX_Y: u16 = 795;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_area_fits_official_panel() {
        const { assert!(ACTIVE_MIN_X < ACTIVE_MAX_X) };
        const { assert!(ACTIVE_MIN_Y < ACTIVE_MAX_Y) };
        const { assert!(ACTIVE_MAX_X <= crate::display::WIDTH) };
        const { assert!(ACTIVE_MAX_Y <= crate::display::HEIGHT) };
    }

    #[test]
    fn address_matches_the_system_bus_table() {
        assert_eq!(ADDRESS, crate::addresses::FT6336G);
    }
}
