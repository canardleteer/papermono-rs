//! Passive buzzer on GPIO42 (`BB_PWM`).
//!
//! Factory demo ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))
//! `hal_buzzer.cpp`: LEDC low-speed timer 3 / channel 7,
//! 10-bit, 50% duty, 40–12000 Hz. Mux off JTAG `MTMS` before PWM.
//! Resonance is still `nyc-buzzer`.

/// Factory demo LEDC timer index.
pub const TIMER: u8 = 3;
/// Factory demo LEDC channel index.
pub const CHANNEL: u8 = 7;
/// Factory demo duty width (bits).
pub const DUTY_BITS: u8 = 10;
/// Click tone (Hz). Inside the factory demo 40–12000 range.
pub const BEEP_HZ: u32 = 2_000;
/// How long a key click sounds, in milliseconds.
pub const BEEP_MS: u64 = 40;
/// Factory demo 50% on-time.
pub const DUTY_PCT: u8 = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_demo_ledc_window() {
        const { assert!(BEEP_HZ >= 40 && BEEP_HZ <= 12_000) };
        const { assert!(TIMER == 3 && CHANNEL == 7 && DUTY_BITS == 10) };
        const { assert!(DUTY_PCT == 50 && BEEP_MS > 0) };
    }
}
