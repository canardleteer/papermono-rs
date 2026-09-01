//! Board support for the **M5Stack PaperMono** (`C153`).
//!
//! Shared nets come from `m5stack-papermono-lite`. This crate adds
//! ST25R3916 NFC and Stamp LoRa-1262 only. Do not depend on this
//! crate from a Lite image.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(test)]
extern crate std;

pub use m5stack_papermono_lite::{addresses, display, ioe1, pins, pmic, touch, SKU as LITE_SKU};

pub mod lora;
pub mod nfc;

/// Official SKU code for this crate (`C153`).
pub const SKU: &str = "C153";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sku_is_full_papermono() {
        assert_eq!(SKU, "C153");
        assert_eq!(LITE_SKU, "C153-Lite");
        assert_ne!(SKU, LITE_SKU);
    }

    #[test]
    fn radio_gpios_are_not_on_the_shared_map() {
        for assigned in pins::ASSIGNED {
            assert_ne!(*assigned, nfc::IRQ);
            assert_ne!(*assigned, lora::IRQ);
            assert_ne!(*assigned, lora::BUSY);
            assert_ne!(*assigned, lora::SPI_MOSI);
            assert_ne!(*assigned, lora::SPI_CLK);
            assert_ne!(*assigned, lora::SPI_MISO);
            assert_ne!(*assigned, lora::NSS);
        }
    }

    #[test]
    fn radio_expander_pins_are_not_on_the_shared_map() {
        for assigned in ioe1::ASSIGNED {
            assert_ne!(*assigned, nfc::IOE1_ENABLE);
            assert_ne!(*assigned, lora::IOE1_ANTENNA_SWITCH);
            assert_ne!(*assigned, lora::IOE1_RESET);
        }
    }
}
