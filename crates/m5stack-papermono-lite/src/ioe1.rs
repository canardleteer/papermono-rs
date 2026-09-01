//! M5IOE1 expander nets present on **both** SKUs.
//!
//! Values are Arduino `M5IOE1_PIN_n` / official `PYGn` indices.
//! NFC enable, LoRa antenna switch, and LoRa reset are **not** here.
//!
//! Outputs default open-drain (including PWM). Pull-up or push-pull,
//! or the pin does not drive high.

/// `PYG1`: microSD detect (`TF_DET`). Insert = low when the slot
/// switch is closed. Hardware skill `nyc-tf-det`.
pub const MICROSD_DETECT: u8 = 1;
/// `PYG3`: EPD 3.3 V enable.
pub const EPD_VDD_ENABLE: u8 = 3;
/// `PYG5`: EPD RST.
pub const EPD_RST: u8 = 5;
/// `PYG6`: touch RST.
pub const TOUCH_RST: u8 = 6;
/// `PYG8` PWM: RGB green.
pub const RGB_GREEN: u8 = 8;
/// `PYG9` PWM: RGB blue.
pub const RGB_BLUE: u8 = 9;
/// `PYG11`: IP2315 I2C gate. Keep the charger off the system bus
/// except the charge transaction.
pub const IP2315_I2C_GATE: u8 = 11;
/// `PYG12`: PDM VDD enable.
pub const PDM_VDD_ENABLE: u8 = 12;
/// `PYG13`: touch VDD enable.
pub const TOUCH_VDD_ENABLE: u8 = 13;
/// `PYG14`: microSD enable.
pub const MICROSD_ENABLE: u8 = 14;

/// Expander indices this crate assigns (Lite / shared).
pub const ASSIGNED: &[u8] = &[
    MICROSD_DETECT,
    EPD_VDD_ENABLE,
    EPD_RST,
    TOUCH_RST,
    RGB_GREEN,
    RGB_BLUE,
    IP2315_I2C_GATE,
    PDM_VDD_ENABLE,
    TOUCH_VDD_ENABLE,
    MICROSD_ENABLE,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigned_expander_pins_are_unique() {
        let mut seen = [false; 16];
        for &pin in ASSIGNED {
            assert!(!seen[pin as usize], "PYG{pin} is assigned twice");
            seen[pin as usize] = true;
        }
    }

    #[test]
    fn radio_expander_pins_are_not_assigned_here() {
        for assigned in ASSIGNED {
            assert_ne!(*assigned, 2, "LoRa antenna switch is full-SKU only");
            assert_ne!(*assigned, 4, "NFC enable is full-SKU only");
            assert_ne!(*assigned, 10, "LoRa reset is full-SKU only");
        }
    }
}
