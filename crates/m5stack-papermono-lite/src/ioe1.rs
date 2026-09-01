//! M5IOE1 **board nets** present on both SKUs.
//!
//! Registers and bank helpers live in `m5ioe1`. NFC enable, LoRa
//! antenna switch, and LoRa reset are **not** here.

pub use m5ioe1::{
    apply_bit, input_level, pin_bit, Bank, GPIO_DRV_H, GPIO_DRV_L, GPIO_I_H, GPIO_I_L, GPIO_M_H,
    GPIO_M_L, GPIO_O_H, GPIO_O_L, IP2315_I2C_GATE, REV, UID_L,
};

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

    #[test]
    fn pyg11_is_ip2315_gate() {
        assert_eq!(pin_bit(IP2315_I2C_GATE), Some((Bank::High, 2)));
        assert_eq!(pin_bit(EPD_VDD_ENABLE), Some((Bank::Low, 2)));
    }
}
