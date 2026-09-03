//! Board support for the **M5Stack PaperMono-Lite** (`C153-Lite`).
//!
//! This crate holds the **shared** electrical map: GPIOs, I2C addresses,
//! expander and PMIC nets, and panel geometry that both SKUs use. Chip
//! registers live in `m5pm1`, `m5ioe1`, and `ssd1677-otp`. MCU
//! peripherals live in firmware.
//! There is no `esp-hal` dependency. Host tests compile on the workspace
//! default rustc.
//!
//! PaperMono (`C153`) is the sibling crate `m5stack-papermono`: it
//! re-exports this crate and adds NFC / LoRa only. Do not init those
//! radios from a Lite image. Official Lite HTML PinMap omits them;
//! leftover pads stay undriven until the hardware skill recipes close.
//!
//! # Not in this crate
//!
//! - Waveform LUTs (call [`display::OtpRefresh`]; do not invent
//!   a 105-byte `0x32` table or map [`display::RefreshMode`]
//!   onto OTP `0x22` bytes)
//! - GPIO45/46 power-latch sequences (those pins are PDM here)
//! - IP2315 traffic except a gated charge transaction
//! - `esp-hal` pin types (firmware maps [`pins`] once)

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(test)]
extern crate std;

/// M5IOE1 chip crate (registers + IP2315 gate typestate).
pub use m5ioe1;
/// M5PM1 chip crate (registers + PWM0 / ADC).
pub use m5pm1;
/// SSD1677 panel-OTP chip crate (`OtpRefresh`, no MCU LUT).
pub use ssd1677_otp;

pub mod addresses;
pub mod buzzer;
pub mod display;
pub mod imu;
pub mod ioe1;
pub mod pins;
pub mod pmic;
pub mod rtc;
pub mod touch;

/// Official SKU code for this crate (`C153-Lite`).
pub const SKU: &str = "C153-Lite";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sku_is_lite() {
        assert_eq!(SKU, "C153-Lite");
    }

    #[test]
    fn radio_gpios_are_not_assigned_here() {
        for &assigned in pins::ASSIGNED {
            assert_ne!(assigned, 5, "LoRa IRQ is full-SKU only");
            assert_ne!(assigned, 6, "NFC IRQ is full-SKU only");
            assert_ne!(assigned, 21, "SX1262 BUSY is full-SKU only");
            assert_ne!(assigned, 38, "LoRa MOSI is full-SKU only");
            assert_ne!(assigned, 39, "LoRa CLK is full-SKU only");
            assert_ne!(assigned, 40, "LoRa MISO is full-SKU only");
            assert_ne!(assigned, 41, "SX1262 NSS is full-SKU only");
        }
    }
}
