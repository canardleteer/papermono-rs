//! Stamp LoRa-1262 (SX1262) on PaperMono (`C153`) only.
//!
//! Product HTML PinMap names this bus SPI1. UserDemo uses `SPI3_HOST`
//! on these GPIOs. Name both; do not flatten. Official Lite PinMap
//! omits LoRa; leftover pads: hardware skill `nyc-lite-lora-pads`.
//!
//! SX1262 SPI spec max 16 MHz. Board clock is unmeasured.

/// LoRa IRQ (GPIO5).
pub const IRQ: u8 = 5;
/// SX1262 BUSY (GPIO21). No internal pull (ESP32-S3 Table 2-1).
pub const BUSY: u8 = 21;
/// LoRa SPI MOSI (GPIO38). Mux off default JTAG on neighboring pads.
pub const SPI_MOSI: u8 = 38;
/// LoRa SPI CLK (GPIO39). Mux off JTAG `MTCK`.
pub const SPI_CLK: u8 = 39;
/// LoRa SPI MISO (GPIO40). Mux off JTAG `MTDO`.
pub const SPI_MISO: u8 = 40;
/// SX1262 NSS (GPIO41). Mux off JTAG `MTDI`.
pub const NSS: u8 = 41;

/// M5IOE1 `PYG2`: LoRa antenna switch.
pub const IOE1_ANTENNA_SWITCH: u8 = 2;
/// M5IOE1 `PYG10`: LoRa reset.
pub const IOE1_RESET: u8 = 10;
/// M5PM1 `G2`: LoRa_EN.
pub const PMIC_ENABLE: u8 = 2;

/// SX1262 SPI specification maximum, in hertz. Not a measured board clock.
pub const SPI_SPEC_MAX_HZ: u32 = 16_000_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lora_spi_is_not_epd_spi2() {
        assert_ne!(SPI_MOSI, crate::pins::EPD_MOSI);
        assert_ne!(SPI_CLK, crate::pins::EPD_SCLK);
        assert_ne!(NSS, crate::pins::EPD_CS);
    }

    #[test]
    fn lora_spi_is_not_sdmmc() {
        for sd in [
            crate::pins::SDMMC_DAT0,
            crate::pins::SDMMC_DAT1,
            crate::pins::SDMMC_DAT2,
            crate::pins::SDMMC_DAT3,
            crate::pins::SDMMC_CMD,
            crate::pins::SDMMC_CLK,
        ] {
            assert_ne!(SPI_MOSI, sd);
            assert_ne!(SPI_CLK, sd);
            assert_ne!(SPI_MISO, sd);
            assert_ne!(NSS, sd);
        }
    }

    #[test]
    fn lora_pins_are_unique() {
        let mut pins = [IRQ, BUSY, SPI_MOSI, SPI_CLK, SPI_MISO, NSS];
        pins.sort_unstable();
        for pair in pins.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }
}
