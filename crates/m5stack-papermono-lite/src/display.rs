//! Panel geometry shared by both SKUs.
//!
//! OTP opcodes and [`OtpRefresh`] live in `ssd1677-otp`. This
//! module keeps official 480×800 tables, M5GFX `epd_*` catalog
//! titles, and the Lite USB-C-down RAM map.

use core::time::Duration;

pub use ssd1677_otp::{
    gray_planes, ram_addr10, OtpRefresh, BOOSTER_SOFT_START, BOOSTER_SOFT_START_OTP,
    BORDER_OTP_FULL, BORDER_OTP_PARTIAL, BORDER_WAVEFORM, DATA_ENTRY_MODE, DATA_ENTRY_XDEC_YINC,
    DATA_ENTRY_XY_INC, DEEP_SLEEP, DEEP_SLEEP_MODE1, DISPLAY_CTRL1_NORMAL,
    DISPLAY_UPDATE_CONTROL_1, DISPLAY_UPDATE_CONTROL_2, DRIVER_OUTPUT, DRIVER_OUTPUT_480_GATES,
    GRAY_BLACK, GRAY_DARK, GRAY_LIGHT, GRAY_WHITE, MASTER_ACTIVATION, OTP_BUSY_TIMEOUT_MS,
    OTP_BYTES_PER_ROW, OTP_GRAY_TITLE, OTP_MONO_TITLE, OTP_PARTIAL_TITLE, OTP_PLANE_BYTES,
    OTP_RAM_HEIGHT, OTP_RAM_WIDTH, OTP_SLEEP_MS, OTP_SPI_HZ, RAM_X_COUNTER, RAM_X_WINDOW,
    RAM_Y_COUNTER, RAM_Y_WINDOW, SW_RESET, TEMP_SENSOR, TEMP_SENSOR_INTERNAL, TEMP_VALUE,
    TEMP_VALUE_GRAY_OTP, UPDATE_SEQ_OTP_4GRAY, UPDATE_SEQ_OTP_MODE1, UPDATE_SEQ_OTP_MONO,
    UPDATE_SEQ_OTP_MONO_SYNC, UPDATE_SEQ_OTP_PARTIAL, WRITE_FSCL_MAX_HZ, WRITE_RAM_BW,
    WRITE_RAM_RED,
};

/// Official panel width (pixels). HTML tables / factory demo rotation 0.
pub const WIDTH: u16 = 480;
/// Official panel height (pixels).
pub const HEIGHT: u16 = 800;

/// Bytes per row in one 1-bit plane at official [`WIDTH`].
pub const BYTES_PER_ROW: usize = (WIDTH as usize) / 8;

/// Bytes in one 1-bit plane (`480×800/8`).
pub const PLANE_BYTES: usize = BYTES_PER_ROW * (HEIGHT as usize);

/// Same OTP 4-gray plus unique RAM-corner bars (`nyc-canvas-orient`).
pub const OTP_ORIENT_TITLE: &str = "otp_orient";
/// Scene stamp for the target walk (not an [`OtpRefresh`] title).
pub const OTP_TARGET_TITLE: &str = "otp_target";
/// Legacy CDC stamp that mixed first Mode 1 and later partial.
pub const OTP_FAST_TITLE: &str = "otp_fast";
/// Catalog default for “partials then one Mode 1 full”.
///
/// Official: after about ten. Firmware policy lives on the image
/// (`embassy-debug` `PARTIALS_BEFORE_FULL`), not this number.
pub const PARTIALS_BEFORE_FULL: u8 = 7;

/// Official HTML **M5GFX LUT Refresh Speed** titles.
///
/// Laboratory results for PaperMono under M5GFX modes. Lite docs
/// reprint the same table. Not a timeout and **not** the firmware
/// call site.
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
    /// Official HTML laboratory reference for **one** refresh
    /// on PaperMono. Not a timeout.
    pub const fn lab_duration(self) -> Duration {
        match self {
            Self::EpdQuality => Duration::from_millis(4710),
            Self::EpdText => Duration::from_millis(450),
            Self::EpdFast => Duration::from_millis(340),
            Self::EpdFastest => Duration::from_millis(70),
        }
    }

    /// M5GFX title used in CDC `panel mode=`.
    pub const fn title(self) -> &'static str {
        match self {
            Self::EpdQuality => "epd_quality",
            Self::EpdText => "epd_text",
            Self::EpdFast => "epd_fast",
            Self::EpdFastest => "epd_fastest",
        }
    }
}

/// Inverse of [`otp_ram_to_usb_down`] (Lite, USB-C down).
#[must_use]
pub const fn usb_down_to_otp_ram(phys_x: u16, phys_y: u16) -> (u16, u16) {
    (phys_y, phys_x)
}

/// Official USB-C-down portrait from OTP-Demo RAM.
///
/// Lite (`C153-Lite`): RAM X is physical Y; RAM Y is physical X.
/// `C153` is not this measurement (`nyc-canvas-orient`).
#[must_use]
pub const fn otp_ram_to_usb_down(ram_x: u16, ram_y: u16) -> (u16, u16) {
    (ram_y, ram_x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_geometry_is_portrait_tables() {
        assert_eq!(WIDTH, 480);
        assert_eq!(HEIGHT, 800);
        assert_eq!(BYTES_PER_ROW, 60);
        assert_eq!(PLANE_BYTES, OTP_PLANE_BYTES);
        const { assert!(WIDTH < HEIGHT) };
        const { assert!(WIDTH % 8 == 0) };
    }

    #[test]
    fn quality_is_slower_than_fastest() {
        const {
            assert!(
                RefreshMode::EpdQuality.lab_duration().as_millis()
                    > RefreshMode::EpdText.lab_duration().as_millis()
            );
            assert!(
                RefreshMode::EpdText.lab_duration().as_millis()
                    > RefreshMode::EpdFast.lab_duration().as_millis()
            );
            assert!(
                RefreshMode::EpdFast.lab_duration().as_millis()
                    > RefreshMode::EpdFastest.lab_duration().as_millis()
            );
        }
    }

    #[test]
    fn lite_usb_down_corner_bars() {
        assert_eq!(otp_ram_to_usb_down(799, 0), (0, 799));
        assert_eq!(otp_ram_to_usb_down(0, 0), (0, 0));
        assert_eq!(otp_ram_to_usb_down(799, 479), (479, 799));
        assert_eq!(otp_ram_to_usb_down(0, 479), (479, 0));
        assert_eq!(usb_down_to_otp_ram(0, 799), (799, 0));
        assert_eq!(RefreshMode::EpdText.title(), "epd_text");
        assert_eq!(OTP_TARGET_TITLE, "otp_target");
        const { assert!(PARTIALS_BEFORE_FULL > 0) };
    }
}
