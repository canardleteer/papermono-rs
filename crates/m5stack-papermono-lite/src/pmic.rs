//! M5PM1 nets present on **both** SKUs.
//!
//! Values are numbered `Gn` indices from the hardware skill pin-map.
//! LoRa_EN (`G2`) is full-SKU only and lives in `m5stack-papermono`.
//! RGB red is PMIC `LED_EN_PP` (not PWM, not a numbered `Gn`): there
//! is no GPIO constant here for it.

/// `G0` `WAKEin`: RTC INT.
pub const WAKE_RTC_INT: u8 = 0;
/// `G3` PWM: frontlight `BL_FB` into AW9967 (`EINK_BL`).
pub const FRONTLIGHT_PWM: u8 = 3;
/// `G4` `WAKEin`: IMU INT.
pub const WAKE_IMU_INT: u8 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbered_pmic_gpios_are_distinct() {
        assert_ne!(WAKE_RTC_INT, FRONTLIGHT_PWM);
        assert_ne!(WAKE_RTC_INT, WAKE_IMU_INT);
        assert_ne!(FRONTLIGHT_PWM, WAKE_IMU_INT);
    }

    #[test]
    fn lora_enable_is_not_a_lite_pmic_net() {
        assert_ne!(WAKE_RTC_INT, 2);
        assert_ne!(FRONTLIGHT_PWM, 2);
        assert_ne!(WAKE_IMU_INT, 2);
    }
}
