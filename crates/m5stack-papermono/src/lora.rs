//! Stamp LoRa-1262 (SX1262) on PaperMono (`C153`) only.
//!
//! Product HTML PinMap names this bus SPI1. UserDemo uses `SPI3_HOST`
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

/// M5IOE1 `PYG2` (`PYB_LoRa_ANT_SW`): LoRa RF antenna switch.
pub const IOE1_ANTENNA_SWITCH: u8 = 2;

/// M5IOE1 `PYG10` (`PYB_LoRa_RST`): LoRa hardware reset line (active low).
pub const IOE1_RESET: u8 = 10;

/// M5PM1 `G2` (`LoRa_EN`): Enables `3V3_L2_LoRa` switched power rail.
pub const PMIC_ENABLE: u8 = 2;

/// SX1262 SPI specification maximum clock rate in hertz (16 MHz). Not a measured board clock.
pub const SPI_SPEC_MAX_HZ: u32 = 16_000_000;

/// SPI clock rate used by the official factory demo firmware (8 MHz).
pub const SPI_USERDEMO_HZ: u32 = 8_000_000;

/// Semtech SX1262 Section 13.5.1 "GetStatus" command opcode (`0xC0`).
pub const CMD_GET_STATUS: u8 = 0xC0;

/// Semtech SX1262 Section 13.4.3 "GetPacketType" command opcode (`0x11`).
pub const CMD_GET_PACKET_TYPE: u8 = 0x11;

/// Semtech SX1262 Section 13.1.2 "SetStandby" command opcode (`0x80`).
pub const CMD_SET_STANDBY: u8 = 0x80;

/// Semtech SX1262 Section 13.6.1 "GetDeviceErrors" command opcode (`0x17`).
pub const CMD_GET_DEVICE_ERRORS: u8 = 0x17;

/// Decoded operating mode of the SX1262 transceiver.
///
/// Semtech SX1262 Section 13.5.1 Table 13-76 "Status Bytes Definition" bits 6:4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipMode {
    /// Mode 0: Unused / RFU.
    Unused,
    /// Mode 2: Standby with RC 13 MHz oscillator (`STBY_RC`).
    StbyRc,
    /// Mode 3: Standby with XOSC 32 MHz crystal oscillator (`STBY_XOSC`).
    StbyXosc,
    /// Mode 4: Frequency synthesis mode (`FS`).
    Fs,
    /// Mode 5: Receive mode (`RX`).
    Rx,
    /// Mode 6: Transmit mode (`TX`).
    Tx,
    /// Reserved or unrecognized mode encoding.
    Other(u8),
}

impl ChipMode {
    /// Decodes a 3-bit mode field from bits 6:4 of a status byte.
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0x07 {
            0x0 => Self::Unused,
            0x2 => Self::StbyRc,
            0x3 => Self::StbyXosc,
            0x4 => Self::Fs,
            0x5 => Self::Rx,
            0x6 => Self::Tx,
            other => Self::Other(other),
        }
    }
}

/// Decoded status of the most recently processed command.
///
/// Semtech SX1262 Section 13.5.1 Table 13-76 "Status Bytes Definition" bits 3:1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    /// Status 0: Reserved / RFU.
    Reserved,
    /// Status 2: Data is available to the host.
    DataAvailable,
    /// Status 3: Command timeout (internal watchdog triggered).
    Timeout,
    /// Status 4: Command processing error (invalid opcode or parameters).
    ProcessingError,
    /// Status 5: Failure to execute command.
    ExecutionFailure,
    /// Status 6: Command TX done (transmission terminated).
    TxDone,
    /// Reserved or unrecognized status encoding.
    Other(u8),
}

impl CommandStatus {
    /// Decodes a 3-bit command status field from bits 3:1 of a status byte.
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0x07 {
            0x0 => Self::Reserved,
            0x2 => Self::DataAvailable,
            0x3 => Self::Timeout,
            0x4 => Self::ProcessingError,
            0x5 => Self::ExecutionFailure,
            0x6 => Self::TxDone,
            other => Self::Other(other),
        }
    }
}

/// Decoded transceiver status byte returned by SX1262 commands.
///
/// Semtech SX1262 Section 13.5.1 Table 13-76 "Status Bytes Definition".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioStatus {
    /// Raw status byte.
    pub raw: u8,
    /// Current transceiver chip mode (bits 6:4).
    pub chip_mode: ChipMode,
    /// Command execution status (bits 3:1).
    pub command_status: CommandStatus,
}

impl RadioStatus {
    /// Parses a raw status byte returned by `GetStatus` (opcode `0xC0`).
    #[inline]
    #[must_use]
    pub const fn from_byte(raw: u8) -> Self {
        let chip_mode = ChipMode::from_bits((raw >> 4) & 0x07);
        let command_status = CommandStatus::from_bits((raw >> 1) & 0x07);
        Self {
            raw,
            chip_mode,
            command_status,
        }
    }

    /// Returns `true` if the radio is in either RC or XOSC standby mode.
    #[inline]
    #[must_use]
    pub const fn is_standby(&self) -> bool {
        matches!(self.chip_mode, ChipMode::StbyRc | ChipMode::StbyXosc)
    }

    /// Returns `true` if the status does not indicate a processing or execution error.
    #[inline]
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        !matches!(
            self.command_status,
            CommandStatus::ProcessingError
                | CommandStatus::ExecutionFailure
                | CommandStatus::Timeout
        )
    }
}

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
    fn decode_radio_status() {
        // Table 13-76: StbyRc (0x2 = 0b010) + DataAvailable (0x2 = 0b010):
        // bits: reserved(0) | mode(010) | cmd(010) | reserved(0) = 0b0010_0100 = 0x24
        let stby_data = RadioStatus::from_byte(0x24);
        assert_eq!(stby_data.chip_mode, ChipMode::StbyRc);
        assert_eq!(stby_data.command_status, CommandStatus::DataAvailable);
        assert!(stby_data.is_standby());
        assert!(stby_data.is_ok());

        // StbyXosc (0x3 = 0b011) + Reserved (0) = 0b0011_0000 = 0x30
        let stby_xosc = RadioStatus::from_byte(0x30);
        assert_eq!(stby_xosc.chip_mode, ChipMode::StbyXosc);
        assert!(stby_xosc.is_standby());
        assert!(stby_xosc.is_ok());

        // Error status: ExecutionFailure (0x5 = 0b101) in bits 3:1 = 0x0A
        let err_status = RadioStatus::from_byte(0x2A);
        assert_eq!(err_status.command_status, CommandStatus::ExecutionFailure);
        assert!(!err_status.is_ok());
    }

    #[test]
    fn opcodes_match_sx1262_specification() {
        assert_eq!(CMD_GET_STATUS, 0xC0);
        assert_eq!(CMD_GET_PACKET_TYPE, 0x11);
        assert_eq!(CMD_SET_STANDBY, 0x80);
        assert_eq!(CMD_GET_DEVICE_ERRORS, 0x17);
    }
}
