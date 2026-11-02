//! Stamp LoRa-1262 (SX1262) on PaperMono (`C153`) only.
//!
//! Product HTML PinMap names this bus SPI1. The official factory firmware
//! [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo) uses `SPI3_HOST`
//! on these GPIOs. Name both; do not flatten. Official Lite PinMap
//! omits LoRa; leftover pads: hardware skill `nyc-lite-lora-pads`.
//!
//! Citations:
//! - Semtech SX1261/2 Datasheet (catalog `sx1262`, Rev 2.2, Dec 2024):
//!   - Section 13.1.2: "SetStandby"
//!   - Section 13.4.3: "GetPacketType"
//!   - Section 13.5.1: "GetStatus"
//!   - Section 13.6.1: "GetDeviceErrors"
//! - M5Stack Stamp LoRa-1262 Module Specification
//!   ([stamp-lora-1262](../../.agents/skills/m5stack-papermono-hardware/resources/stamp-lora-1262.md))
//! - Official factory demo firmware
//!   ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))

/// LoRa IRQ / DIO1 (GPIO5).
pub const IRQ: u8 = 5;

/// SX1262 BUSY line (GPIO21). No internal pull on ESP32-S3 (Table 2-1).
///
/// MCU must wait for this pin to be low before asserting NSS for an SPI transaction.
pub const BUSY: u8 = 21;

/// LoRa SPI MOSI (GPIO38).
pub const SPI_MOSI: u8 = 38;

/// LoRa SPI CLK (GPIO39). Multiplex off default ESP32-S3 JTAG `MTCK`.
pub const SPI_CLK: u8 = 39;

/// LoRa SPI MISO (GPIO40). Multiplex off default ESP32-S3 JTAG `MTDO`.
pub const SPI_MISO: u8 = 40;

/// SX1262 NSS chip select (GPIO41). Active low. Multiplex off default ESP32-S3 JTAG `MTDI`.
pub const NSS: u8 = 41;

/// M5IOE1 `PYG2` (`PYB_LoRa_ANT_SW`): LoRa RF antenna switch gate control.
///
/// Per PaperMono Schematic V0.6.2 Page 5 and official factory firmware
/// (`M5PaperMono-UserDemo` `hal_lora.cpp`), this expander output controls an RF
/// switch connecting the built-in FPC antenna to the transceiver front-end.
/// Driving this line HIGH engages the antenna; driving LOW isolates it.
pub const IOE1_ANTENNA_SWITCH: u8 = 2;

/// M5IOE1 `PYG10` (`PYB_LoRa_RST`): LoRa hardware reset line (active low).
pub const IOE1_RESET: u8 = 10;

/// M5PM1 `G2` (`LoRa_EN`): Enables `3V3_L2_LoRa` switched power rail.
pub const PMIC_ENABLE: u8 = 2;

/// SX1262 SPI specification maximum clock rate in hertz (16 MHz). Not a measured board clock.
pub const SPI_SPEC_MAX_HZ: u32 = 16_000_000;

/// SPI clock rate used by the official factory demo firmware (8 MHz).
pub const SPI_USERDEMO_HZ: u32 = 8_000_000;

/// Generic SX1262 API retained at the BSP import path.
pub use sx1262_phy::*;

/// PaperMono power/reset/antenna hooks with borrowed system I2C context.
pub use crate::lora_lifecycle::*;

/// Default TCXO stabilization delay: 320 ticks (5.0 ms at 15.625 us/tick).
pub const TCXO_DEFAULT_DELAY_TICKS: u32 = 0x000140;

/// Image calibration frequency step 1 for 902–928 MHz ISM band (`0xE1` = 900 MHz).
pub const CAL_IMG_902_MHZ: u8 = 0xE1;

/// Image calibration frequency step 2 for 902–928 MHz ISM band (`0xE9` = 932 MHz).
pub const CAL_IMG_928_MHZ: u8 = 0xE9;

/// Retained diagnostic PA duty-cycle setting (`0x02`).
/// Catalog `sx1262` §13.1.14.1 “PA Optimal Settings”; power depends on PA load.
pub const PA_DUTY_CYCLE_14DBM: u8 = 0x02;

/// Retained diagnostic hpMax setting (`0x03`), named for the +14 command profile.
/// Catalog `sx1262` §13.1.14.1 “PA Optimal Settings” lists this in the +17 row.
/// The legacy name does not establish the module's measured RF output.
pub const PA_HP_MAX_14DBM: u8 = 0x03;

/// PA duty-cycle setting for the +22 dBm reference row (`0x04`).
/// Catalog `sx1262` §13.1.14.1 “PA Optimal Settings”.
pub const PA_DUTY_CYCLE_22DBM: u8 = 0x04;

/// PA hpMax setting for the +22 dBm reference row (`0x07`).
/// Catalog `sx1262` §13.1.14.1 “PA Optimal Settings”.
pub const PA_HP_MAX_22DBM: u8 = 0x07;

/// OCP code for 60 mA (`0x18`), retained for diagnostics.
/// Catalog `sx1262` §12.1 “Registers” (OCP); RF power needs measurement.
pub const OCP_60_MA: u8 = 0x18;

/// Over-current protection clamp: 140 mA (`0x38`) default.
pub const OCP_140_MA: u8 = 0x38;

/// Meshtastic default network sync word (logical `0x2B`, encoded `0x24B4`).
pub const SYNC_WORD_MESHTASTIC: u16 = 0x24B4;

/// Alternate network sync word alias (`0x24B4`).
pub const SYNC_WORD_ALT: u16 = SYNC_WORD_MESHTASTIC;

// =========================================================================
// US915 Channel Layout Constants
// =========================================================================

/// Total number of 250 kHz bandwidth channels in the US915 band (902.0 to 928.0 MHz).
pub const US915_NUM_CHANNELS: usize = 104;

/// Calculates the center frequency in Hz for a US915 250 kHz channel slot (0..103).
///
/// `freq_hz = 902_125_000 + slot * 250_000`
#[inline]
#[must_use]
pub const fn us915_channel_freq_hz(slot: u8) -> u32 {
    902_125_000 + (slot as u32) * 250_000
}

// =========================================================================
// Standard Test Frequency Presets
// =========================================================================

/// Test bench ping frequency: 915.000 MHz.
pub const FREQ_BENCH_PING_HZ: u32 = 915_000_000;

/// Primary packet sniffer frequency: 917.625 MHz (US Slot 63).
pub const FREQ_RX_SNIFFER_PRI_HZ: u32 = 917_625_000;

/// Secondary packet sniffer frequency: 906.875 MHz (US Slot 20).
pub const FREQ_RX_SNIFFER_SEC_HZ: u32 = 906_875_000;

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

    #[test]
    fn lora_jtag_mux_pins() {
        // GPIO39, 40, 41 default to JTAG MTCK, MTDO, MTDI on ESP32-S3
        assert_eq!(SPI_CLK, 39);
        assert_eq!(SPI_MISO, 40);
        assert_eq!(NSS, 41);
    }

    #[test]
    fn sync_word_encoding_matches_semtech_and_meshtastic() {
        assert_eq!(encode_sync_word(0x12), SYNC_WORD_PRIVATE);
        assert_eq!(SYNC_WORD_PRIVATE, 0x1424);
        assert_eq!(encode_sync_word(0x34), SYNC_WORD_PUBLIC);
        assert_eq!(SYNC_WORD_PUBLIC, 0x3444);
        assert_eq!(encode_sync_word(0x2B), SYNC_WORD_MESHTASTIC);
        assert_eq!(SYNC_WORD_MESHTASTIC, 0x24B4);
        assert_eq!(SYNC_WORD_ALT, 0x24B4);
    }

    #[test]
    fn us915_channels_match_frequency_slots() {
        assert_eq!(US915_NUM_CHANNELS, 104);
        assert_eq!(us915_channel_freq_hz(0), 902_125_000);
        assert_eq!(us915_channel_freq_hz(19), FREQ_RX_SNIFFER_SEC_HZ);
        assert_eq!(us915_channel_freq_hz(62), FREQ_RX_SNIFFER_PRI_HZ);
        assert_eq!(us915_channel_freq_hz(103), 927_875_000);
    }
}
