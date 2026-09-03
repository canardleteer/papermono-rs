//! M5PM1 **board nets** present on both SKUs.
//!
//! Registers live in `m5pm1`. LoRa_EN (`G2`) is C153 only.
//! RGB red is PMIC `LED_EN_PP` (not PWM).

pub use m5pm1::{
    adc_mv, pwm0_bytes, pwm_freq_bytes, CHG_EN, DEVICE_ID, FRONTLIGHT_DUTY, FRONTLIGHT_PWM_HZ,
    GPIO3_FUNC_MASK, GPIO3_FUNC_PWM, GPIO_DRV, GPIO_FUNC0, GPIO_IN, GPIO_MODE, GPIO_PULL_UP,
    GPIO_PUPD0, GPIO_WAKE_CFG, GPIO_WAKE_EN, HOLD_CFG, HOLD_LDO, HOLD_VIN, IRQ_MASK1, IRQ_STATUS1,
    IRQ_STATUS2, IRQ_STATUS3, PWM0_DUTY_MAX, PWM0_EN, PWM0_HC, PWM0_L, PWM_FREQ_L, PWR_CFG,
    PWR_SRC, PWR_SRC_BAT, PWR_SRC_VIN, PWR_SRC_VINOUT, SYS_CMD, SYS_CMD_KEY, SYS_CMD_SHUTDOWN,
    VBAT_L, VIN_L, VIN_PRESENT_MV, WAKE_SRC, WAKE_SRC_EXT,
};

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
        assert_eq!(FRONTLIGHT_PWM, 3);
    }

    #[test]
    fn lora_enable_is_not_a_lite_pmic_net() {
        assert_ne!(WAKE_RTC_INT, 2);
        assert_ne!(FRONTLIGHT_PWM, 2);
        assert_ne!(WAKE_IMU_INT, 2);
    }
}
