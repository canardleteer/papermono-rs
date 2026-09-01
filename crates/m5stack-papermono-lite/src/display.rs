//! Panel geometry and M5GFX refresh titles shared by both SKUs.
//!
//! Official tables: 480×800. FreeInk uses 800×480; do not flatten
//! that conflict (`nyc-canvas-orient`). Do not ship a waveform LUT.
//! OTP first. Catalog id `ssd1677` for controller opcodes.

use core::time::Duration;

/// Official panel width (pixels).
pub const WIDTH: u16 = 480;
/// Official panel height (pixels).
pub const HEIGHT: u16 = 800;

/// SSD1677 Rev 1.0 write `fSCL` maximum.
///
/// This is the **sheet cap**, not a measured board clock
/// (`nyc-epd-spi-clock`). Catalog id `ssd1677`.
pub const WRITE_FSCL_MAX_HZ: u32 = 20_000_000;

/// M5GFX refresh titles. Lab single-refresh times were measured on
/// PaperMono (`C153`) **and** PaperMono-Lite (`C153-Lite`).
///
/// Use these variants at the call site. Do not pass a raw M5GFX
/// integer or an ad-hoc delay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshMode {
    /// `epd_quality` — lab 4.71 s.
    EpdQuality,
    /// `epd_text` — lab 0.45 s.
    EpdText,
    /// `epd_fast` — lab 0.34 s.
    EpdFast,
    /// `epd_fastest` — lab 0.07 s.
    EpdFastest,
}

impl RefreshMode {
    /// Lab wall-clock for **one** refresh on both SKUs.
    pub const fn lab_duration(self) -> Duration {
        match self {
            Self::EpdQuality => Duration::from_millis(4710),
            Self::EpdText => Duration::from_millis(450),
            Self::EpdFast => Duration::from_millis(340),
            Self::EpdFastest => Duration::from_millis(70),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_geometry_is_portrait_tables() {
        assert_eq!(WIDTH, 480);
        assert_eq!(HEIGHT, 800);
        const { assert!(WIDTH < HEIGHT) };
    }

    #[test]
    fn quality_is_slower_than_fastest() {
        const {
            assert!(RefreshMode::EpdQuality.lab_duration() > RefreshMode::EpdText.lab_duration());
            assert!(RefreshMode::EpdText.lab_duration() > RefreshMode::EpdFast.lab_duration());
            assert!(RefreshMode::EpdFast.lab_duration() > RefreshMode::EpdFastest.lab_duration());
        }
    }
}
