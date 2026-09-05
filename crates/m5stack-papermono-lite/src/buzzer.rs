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
/// Factory demo nominal 50% on-time.
pub const DUTY_PCT: u8 = 50;

/// Peak acoustic amplitude duty cycle percentage on PaperMono magnetic buzzer (25%).
///
/// While theoretical square waves deliver maximum fundamental power at 50% duty,
/// the PaperMono magnetic transducer and drive circuit acoustically peak at ~25%
/// duty cycle on hardware due to coil inductance saturation and rich harmonic
/// content in the ear's peak sensitivity range (3–4 kHz). Duty cycles above 25%
/// reduce perceived loudness.
pub const DUTY_MAX_PCT: u8 = 25;

/// Maps a volume percentage (0..=100) to LEDC square-wave duty cycle percentage (0..=25).
///
/// Maps volume 0..=100 linearly to duty cycle 0..=25%, ensuring the slider is quietest
/// at the bottom (0%) and loudest at the top (100%).
#[must_use]
pub const fn volume_to_duty_pct(volume: u8) -> u8 {
    let vol = if volume > 100 { 100 } else { volume };
    ((vol as u16 * DUTY_MAX_PCT as u16) / 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_demo_ledc_window() {
        const { assert!(BEEP_HZ >= 40 && BEEP_HZ <= 12_000) };
        const { assert!(TIMER == 3 && CHANNEL == 7 && DUTY_BITS == 10) };
        const { assert!(DUTY_PCT == 50 && DUTY_MAX_PCT == 25 && BEEP_MS > 0) };
    }

    #[test]
    fn volume_to_duty_pct_mapping() {
        assert_eq!(volume_to_duty_pct(0), 0);
        assert_eq!(volume_to_duty_pct(50), 12);
        assert_eq!(volume_to_duty_pct(100), 25);
        assert_eq!(volume_to_duty_pct(150), 25);
    }
}
