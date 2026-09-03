//! RX8130CE on the system I2C bus.

/// 7-bit address. Same as [`crate::addresses::RX8130CE`].
pub const ADDRESS: u8 = crate::addresses::RX8130CE;

/// Catalog id `rx8130ce`, section 13.2 Register Table, Flag Register.
///
/// Read-only probe. Do not write `SEC` (that clears the sub-second
/// chain). Timer clear is a write of [`FLAG_CLEAR_TF`], not `SEC`.
pub const FLAG: u8 = 0x1D;
/// M5Unified `FLAG_CLEAR_TF`: write-0-to-clear timer flag only.
pub const FLAG_CLEAR_TF: u8 = 0xAF;
/// M5Unified `clearIRQ`: write-0-to-clear TF and AF.
pub const FLAG_CLEAR_IRQ: u8 = 0xA7;
/// M5Unified `getIRQstatus`: TF or AF.
pub const FLAG_IRQ: u8 = 0x18;
/// Timer counter low. M5Unified `setTimerIRQ` preset at `0x1A`.
pub const TIMER_COUNTER_L: u8 = 0x1A;
/// Extension register (TE / TSEL). M5Unified `0x1C`.
pub const EXTENSION: u8 = 0x1C;
/// Control register (TIE). M5Unified `0x1E`.
pub const CONTROL: u8 = 0x1E;
/// [`EXTENSION`] / [`CONTROL`] timer-enable / timer-IRQ bit.
pub const TIMER_EN: u8 = 0x10;
/// TSEL 64 Hz. M5Unified `clk_t` for a 10 s count of 640.
pub const TSEL_64HZ: u8 = 0x01;
/// 10 s at 64 Hz (factory demo `RTC_WAKE_TIMER_MS`).
pub const TIMER_10S_COUNTS: u16 = 640;
/// User RAM byte we use as a one-shot “already woke” mark.
///
/// Not factory demo index 0/1 packing. Do not write `SEC`.
pub const USER_RAM_MARK: u8 = 0x23;
/// Value in [`USER_RAM_MARK`] after we arm RTC sleep.
pub const USER_RAM_SLEPT: u8 = 0xA5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_is_not_sec() {
        assert_eq!(ADDRESS, crate::addresses::RX8130CE);
        assert_ne!(FLAG, 0x10);
        assert_eq!(TIMER_COUNTER_L, 0x1A);
        assert_eq!(EXTENSION, 0x1C);
        assert_eq!(CONTROL, 0x1E);
        assert_ne!(USER_RAM_MARK, FLAG);
        assert_eq!(TIMER_10S_COUNTS, 640);
        assert_eq!(FLAG_CLEAR_IRQ, 0xA7);
        assert_eq!(FLAG_IRQ, 0x18);
    }
}
