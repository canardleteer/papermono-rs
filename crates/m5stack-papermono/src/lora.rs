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

// =========================================================================
// Semtech SX1261/2 Direct Command Opcodes
// Citations: Semtech SX1261/2 Datasheet Rev 2.2, Section 13
// =========================================================================

/// Semtech SX1262 Section 13.1.1 "SetSleep" command opcode (`0x84`).
pub const CMD_SET_SLEEP: u8 = 0x84;

/// Semtech SX1262 Section 13.1.2 "SetStandby" command opcode (`0x80`).
pub const CMD_SET_STANDBY: u8 = 0x80;

/// Semtech SX1262 Section 13.1.3 "SetFs" command opcode (`0xC1`).
pub const CMD_SET_FS: u8 = 0xC1;

/// Semtech SX1262 Section 13.1.4 "SetTx" command opcode (`0x83`).
pub const CMD_SET_TX: u8 = 0x83;

/// Semtech SX1262 Section 13.1.5 "SetRx" command opcode (`0x82`).
pub const CMD_SET_RX: u8 = 0x82;

/// Semtech SX1262 Section 13.1.8 "SetCad" command opcode (`0xC5`).
pub const CMD_SET_CAD: u8 = 0xC5;

/// Semtech SX1262 Section 13.1.6 "StopTimerOnPreamble" command opcode (`0x9F`).
pub const CMD_STOP_TIMER_ON_PREAMBLE: u8 = 0x9F;

/// Semtech SX1262 Section 13.1.7 "SetRxDutyCycle" command opcode (`0x94`).
pub const CMD_SET_RX_DUTY_CYCLE: u8 = 0x94;

/// Semtech SX1262 Section 13.1.8 "SetRegulatorMode" command opcode (`0x96`).
pub const CMD_SET_REGULATOR_MODE: u8 = 0x96;

/// Semtech SX1262 Section 13.1.9 "CalibrateFunction" command opcode (`0x89`).
pub const CMD_CALIBRATE_FUNCTION: u8 = 0x89;

/// Semtech SX1262 Section 13.1.10 "CalibrateImage" command opcode (`0x98`).
pub const CMD_CALIBRATE_IMAGE: u8 = 0x98;

/// Semtech SX1262 Section 13.1.11 "SetPaConfig" command opcode (`0x95`).
pub const CMD_SET_PA_CONFIG: u8 = 0x95;

/// Semtech SX1262 Section 13.1.12 "SetRxTxFallbackMode" command opcode (`0x93`).
pub const CMD_SET_RX_TX_FALLBACK_MODE: u8 = 0x93;

/// Semtech SX1262 Section 13.2.1 "WriteRegister" command opcode (`0x0D`).
pub const CMD_WRITE_REGISTER: u8 = 0x0D;

/// Semtech SX1262 Section 13.2.2 "ReadRegister" command opcode (`0x1D`).
pub const CMD_READ_REGISTER: u8 = 0x1D;

/// Semtech SX1262 Section 13.2.3 "WriteBuffer" command opcode (`0x0E`).
pub const CMD_WRITE_BUFFER: u8 = 0x0E;

/// Semtech SX1262 Section 13.2.4 "ReadBuffer" command opcode (`0x1E`).
pub const CMD_READ_BUFFER: u8 = 0x1E;

/// Semtech SX1262 Section 13.3.1 "SetDioIrqParams" command opcode (`0x08`).
pub const CMD_SET_DIO_IRQ_PARAMS: u8 = 0x08;

/// Semtech SX1262 Section 13.3.2 "GetIrqStatus" command opcode (`0x12`).
pub const CMD_GET_IRQ_STATUS: u8 = 0x12;

/// Semtech SX1262 Section 13.3.3 "ClearIrqStatus" command opcode (`0x02`).
pub const CMD_CLEAR_IRQ_STATUS: u8 = 0x02;

/// Semtech SX1262 Section 13.3.4 "SetDio2AsRfSwitchCtrl" command opcode (`0x9D`).
pub const CMD_SET_DIO2_AS_RF_SWITCH_CTRL: u8 = 0x9D;

/// Semtech SX1262 Section 13.3.5 "SetDio3AsTcxoCtrl" command opcode (`0x97`).
pub const CMD_SET_DIO3_AS_TCXO_CTRL: u8 = 0x97;

/// Semtech SX1262 Section 13.4.1 "SetRfFrequency" command opcode (`0x86`).
pub const CMD_SET_RF_FREQUENCY: u8 = 0x86;

/// Semtech SX1262 Section 13.4.2 "SetPacketType" command opcode (`0x8A`).
pub const CMD_SET_PACKET_TYPE: u8 = 0x8A;

/// Semtech SX1262 Section 13.4.3 "GetPacketType" command opcode (`0x11`).
pub const CMD_GET_PACKET_TYPE: u8 = 0x11;

/// Semtech SX1262 Section 13.4.4 "SetTxParams" command opcode (`0x8E`).
pub const CMD_SET_TX_PARAMS: u8 = 0x8E;

/// Semtech SX1262 Section 13.4.5 "SetModulationParams" command opcode (`0x8B`).
pub const CMD_SET_MODULATION_PARAMS: u8 = 0x8B;

/// Semtech SX1262 Section 13.4.6 "SetPacketParams" command opcode (`0x8C`).
pub const CMD_SET_PACKET_PARAMS: u8 = 0x8C;

/// Semtech SX1262 Section 13.4.7 "SetCadParams" command opcode (`0x88`).
pub const CMD_SET_CAD_PARAMS: u8 = 0x88;

/// Semtech SX1262 Section 13.4.8 "SetBufferBaseAddress" command opcode (`0x8F`).
pub const CMD_SET_BUFFER_BASE_ADDRESS: u8 = 0x8F;

/// Semtech SX1262 Section 13.5.1 "GetStatus" command opcode (`0xC0`).
pub const CMD_GET_STATUS: u8 = 0xC0;

/// Semtech SX1262 Section 13.5.2 "GetRxBufferStatus" command opcode (`0x13`).
pub const CMD_GET_RX_BUFFER_STATUS: u8 = 0x13;

/// Semtech SX1262 Section 13.5.3 "GetPacketStatus" command opcode (`0x14`).
pub const CMD_GET_PACKET_STATUS: u8 = 0x14;

/// Semtech SX1262 Section 13.5.4 "GetRssiInst" command opcode (`0x15`).
pub const CMD_GET_RSSI_INST: u8 = 0x15;

/// Semtech SX1262 Section 13.6.1 "GetDeviceErrors" command opcode (`0x17`).
pub const CMD_GET_DEVICE_ERRORS: u8 = 0x17;

/// Semtech SX1262 Section 13.6.2 "ClearDeviceErrors" command opcode (`0x07`).
pub const CMD_CLEAR_DEVICE_ERRORS: u8 = 0x07;

// =========================================================================
// Register Addresses
// Citations: Semtech SX1261/2 Datasheet Rev 2.2
// =========================================================================

/// SX1262 LoRa Sync Word MSB register address (`0x0740`).
pub const REG_LORA_SYNC_WORD_MSB: u16 = 0x0740;

/// SX1262 LoRa Sync Word LSB register address (`0x0741`).
pub const REG_LORA_SYNC_WORD_LSB: u16 = 0x0741;

/// SX1262 Over-Current Protection (OCP) register address (`0x08E7`).
pub const REG_OCP: u16 = 0x08E7;

// =========================================================================
// Configuration Parameters & Bitfield Constants
// =========================================================================

/// Standby configuration: STDBY_RC (13 MHz RC oscillator, opcode param `0x00`).
pub const STDBY_CONFIG_RC: u8 = 0x00;

/// Standby configuration: STDBY_XOSC (32 MHz crystal oscillator, opcode param `0x01`).
pub const STDBY_CONFIG_XOSC: u8 = 0x01;

/// Regulator mode: internal LDO enabled (opcode param `0x00`).
pub const REGULATOR_LDO: u8 = 0x00;

/// Regulator mode: internal DC-DC converter enabled (opcode param `0x01`).
pub const REGULATOR_DC_DC: u8 = 0x01;

/// Fallback mode: return to FS mode after packet handling (opcode param `0x40`).
pub const FALLBACK_FS: u8 = 0x40;

/// Fallback mode: return to STDBY_XOSC mode after packet handling (opcode param `0x30`).
pub const FALLBACK_STDBY_XOSC: u8 = 0x30;

/// Fallback mode: return to STDBY_RC mode after packet handling (opcode param `0x20`).
pub const FALLBACK_STDBY_RC: u8 = 0x20;

/// Packet type: GFSK modem (opcode param `0x00`).
pub const PACKET_TYPE_GFSK: u8 = 0x00;

/// Packet type: LoRa modem (opcode param `0x01`).
pub const PACKET_TYPE_LORA: u8 = 0x01;

/// TCXO control voltage: 3.0 V (opcode param `0x06`).
pub const TCXO_CTRL_3_0V: u8 = 0x06;

/// Default TCXO stabilization delay: 320 ticks (5.0 ms at 15.625 us/tick).
pub const TCXO_DEFAULT_DELAY_TICKS: u32 = 0x000140;

/// Image calibration frequency step 1 for 902–928 MHz ISM band (`0xE1` = 900 MHz).
pub const CAL_IMG_902_MHZ: u8 = 0xE1;

/// Image calibration frequency step 2 for 902–928 MHz ISM band (`0xE9` = 932 MHz).
pub const CAL_IMG_928_MHZ: u8 = 0xE9;

/// PA configuration: SX1262 device selection (`0x00`).
pub const PA_DEVICE_SEL_SX1262: u8 = 0x00;

/// PA configuration: default lookup table (`0x01`).
pub const PA_LUT_DEFAULT: u8 = 0x01;

/// PA configuration: bench-safe +14 dBm duty cycle setting (`0x02`).
pub const PA_DUTY_CYCLE_14DBM: u8 = 0x02;

/// PA configuration: bench-safe +14 dBm hpMax setting (`0x03`).
pub const PA_HP_MAX_14DBM: u8 = 0x03;

/// PA configuration: full +22 dBm duty cycle setting (`0x04`).
pub const PA_DUTY_CYCLE_22DBM: u8 = 0x04;

/// PA configuration: full +22 dBm hpMax setting (`0x07`).
pub const PA_HP_MAX_22DBM: u8 = 0x07;

/// Power amplifier ramp time: 40 us (`0x02`).
pub const RAMP_40_US: u8 = 0x02;

/// Over-current protection clamp: 60 mA (`0x18`) for bench test safety.
pub const OCP_60_MA: u8 = 0x18;

/// Over-current protection clamp: 140 mA (`0x38`) default.
pub const OCP_140_MA: u8 = 0x38;

/// LoRa spreading factor: SF7 (`0x07`).
pub const LORA_SF7: u8 = 0x07;

/// LoRa spreading factor: SF8 (`0x08`).
pub const LORA_SF8: u8 = 0x08;

/// LoRa spreading factor: SF9 (`0x09`).
pub const LORA_SF9: u8 = 0x09;

/// LoRa spreading factor: SF10 (`0x0A`).
pub const LORA_SF10: u8 = 0x0A;

/// LoRa spreading factor: SF11 (`0x0B`).
pub const LORA_SF11: u8 = 0x0B;

/// LoRa spreading factor: SF12 (`0x0C`).
pub const LORA_SF12: u8 = 0x0C;

/// LoRa signal bandwidth: 125 kHz (`0x04`).
pub const LORA_BW_125_KHZ: u8 = 0x04;

/// LoRa signal bandwidth: 250 kHz (`0x05`).
pub const LORA_BW_250_KHZ: u8 = 0x05;

/// LoRa signal bandwidth: 500 kHz (`0x06`).
pub const LORA_BW_500_KHZ: u8 = 0x06;

/// LoRa coding rate: 4/5 (`0x01`).
pub const LORA_CR_4_5: u8 = 0x01;

/// LoRa coding rate: 4/6 (`0x02`).
pub const LORA_CR_4_6: u8 = 0x02;

/// LoRa coding rate: 4/7 (`0x03`).
pub const LORA_CR_4_7: u8 = 0x03;

/// LoRa coding rate: 4/8 (`0x04`).
pub const LORA_CR_4_8: u8 = 0x04;

/// LoRa Low Data Rate Optimization: disabled (`0x00`).
pub const LORA_LDRO_OFF: u8 = 0x00;

/// LoRa Low Data Rate Optimization: enabled (`0x01`).
pub const LORA_LDRO_ON: u8 = 0x01;

/// LoRa packet header type: variable / explicit header (`0x00`).
pub const LORA_HEADER_VARIABLE: u8 = 0x00;

/// LoRa packet header type: fixed / implicit header (`0x01`).
pub const LORA_HEADER_FIXED: u8 = 0x01;

/// LoRa packet CRC: disabled (`0x00`).
pub const LORA_CRC_OFF: u8 = 0x00;

/// LoRa packet CRC: enabled (`0x01`).
pub const LORA_CRC_ON: u8 = 0x01;

/// LoRa packet IQ polarity: standard (`0x00`).
pub const LORA_IQ_STANDARD: u8 = 0x00;

/// LoRa packet IQ polarity: inverted (`0x01`).
pub const LORA_IQ_INVERTED: u8 = 0x01;

/// Encodes an 8-bit logical LoRa sync word into the 16-bit register value
/// required by SX1261/2 registers 0x0740 (MSB) and 0x0741 (LSB).
///
/// In SX1262, the low nibble of each register must hold hardware control bits `0x04`.
/// The upper nibbles hold the high and low nibbles of the 8-bit sync word.
#[inline]
#[must_use]
pub const fn encode_sync_word(sync_word: u8) -> u16 {
    let msb = ((sync_word & 0xF0) | 0x04) as u16;
    let lsb = (((sync_word & 0x0F) << 4) | 0x04) as u16;
    (msb << 8) | lsb
}

/// Standard LoRa private network sync word (`0x1424`, logical `0x12`).
pub const SYNC_WORD_PRIVATE: u16 = 0x1424;

/// Standard LoRa public / LoRaWAN network sync word (`0x3444`, logical `0x34`).
pub const SYNC_WORD_PUBLIC: u16 = 0x3444;

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
// IRQ Bitmask Constants
// Citations: Semtech SX1261/2 Datasheet Rev 2.2, Section 13.3.1 Table 13-43
// =========================================================================

/// Packet transmission completed (`0x0001`).
pub const IRQ_TX_DONE: u16 = 1 << 0;

/// Packet reception completed (`0x0002`).
pub const IRQ_RX_DONE: u16 = 1 << 1;

/// Preamble detected (`0x0004`).
pub const IRQ_PREAMBLE_DETECTED: u16 = 1 << 2;

/// Valid sync word detected (`0x0008`).
pub const IRQ_SYNC_WORD_VALID: u16 = 1 << 3;

/// Valid header received (`0x0010`).
pub const IRQ_HEADER_VALID: u16 = 1 << 4;

/// Header error detected (`0x0020`).
pub const IRQ_HEADER_ERR: u16 = 1 << 5;

/// CRC error detected on payload (`0x0040`).
pub const IRQ_CRC_ERR: u16 = 1 << 6;

/// Channel Activity Detection (CAD) completed (`0x0080`).
pub const IRQ_CAD_DONE: u16 = 1 << 7;

/// Channel activity detected during CAD (`0x0100`).
pub const IRQ_CAD_DETECTED: u16 = 1 << 8;

/// RX or TX operation timed out (`0x0200`).
pub const IRQ_TIMEOUT: u16 = 1 << 9;

/// All IRQ bits combined (`0x03FF`).
pub const IRQ_ALL: u16 = 0x03FF;

// =========================================================================
// Standard Test Frequency Presets
// =========================================================================

/// Test bench ping frequency: 915.000 MHz.
pub const FREQ_BENCH_PING_HZ: u32 = 915_000_000;

/// Primary packet sniffer frequency: 917.625 MHz (US Slot 63).
pub const FREQ_RX_SNIFFER_PRI_HZ: u32 = 917_625_000;

/// Secondary packet sniffer frequency: 906.875 MHz (US Slot 20).
pub const FREQ_RX_SNIFFER_SEC_HZ: u32 = 906_875_000;

/// Calculates the 32-bit RF frequency register parameter for a given frequency in hertz.
///
/// Semtech SX1261/2 Section 13.4.1 "SetRfFrequency":
/// `rf_freq = (freq_hz * 2^25) / 32_000_000`
#[inline]
#[must_use]
pub const fn calculate_rf_freq_reg(freq_hz: u32) -> u32 {
    let num = (freq_hz as u64) * 33_554_432;
    (num / 32_000_000) as u32
}

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

/// Decoded packet reception metrics from `GetPacketStatus` (`0x14`).
///
/// Semtech SX1261/2 Section 13.5.3 Table 13-80.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketStatus {
    /// Average packet RSSI in dBm.
    pub rssi_pkt_dbm: i16,
    /// Estimated packet Signal-to-Noise Ratio (SNR) in dB.
    pub snr_pkt_db: i8,
    /// Estimated signal RSSI in dBm.
    pub signal_rssi_pkt_dbm: i16,
}

/// Driver errors encountered during SX1262 SPI communication or pin polling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SxError<SpiErr> {
    /// SPI bus communication error.
    Spi(SpiErr),
    /// BUSY line remained high beyond polling limit.
    BusyTimeout,
    /// Invalid parameter passed to driver function.
    InvalidParam,
    /// GPIO error toggling NSS or reading BUSY.
    Gpio,
}

/// Bounded loop iterations to wait for SX1262 BUSY line to drop low before timeout.
pub const BUSY_TIMEOUT_POLLS: u32 = 50_000;

/// Stamp LoRa-1262 (Semtech SX1262) transceiver driver.
pub struct Sx1262<SPI, NSS, BUSY> {
    spi: SPI,
    nss: NSS,
    busy: BUSY,
}

impl<SPI, NSS, BUSY> Sx1262<SPI, NSS, BUSY>
where
    SPI: embedded_hal::spi::SpiBus,
    NSS: embedded_hal::digital::OutputPin,
    BUSY: embedded_hal::digital::InputPin,
{
    /// Creates a new SX1262 driver wrapping SPI bus, NSS chip select, and BUSY input.
    pub const fn new(spi: SPI, nss: NSS, busy: BUSY) -> Self {
        Self { spi, nss, busy }
    }

    /// Releases the hardware peripherals.
    pub fn release(self) -> (SPI, NSS, BUSY) {
        (self.spi, self.nss, self.busy)
    }

    /// Waits for the BUSY pin to drop low, indicating transceiver readiness.
    pub fn wait_busy(&mut self) -> Result<(), SxError<SPI::Error>> {
        for _ in 0..BUSY_TIMEOUT_POLLS {
            if self.busy.is_low().map_err(|_| SxError::Gpio)? {
                return Ok(());
            }
        }
        Err(SxError::BusyTimeout)
    }

    /// Writes a direct command with zero or more parameter bytes.
    pub fn write_cmd(&mut self, cmd: u8, params: &[u8]) -> Result<(), SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let res = (|| -> Result<(), SPI::Error> {
            self.spi.write(&[cmd])?;
            if !params.is_empty() {
                self.spi.write(params)?;
            }
            Ok(())
        })();
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)
    }

    /// Writes an 8-bit value to a 16-bit register address.
    pub fn write_reg(&mut self, reg: u16, val: u8) -> Result<(), SxError<SPI::Error>> {
        let params = [(reg >> 8) as u8, reg as u8, val];
        self.write_cmd(CMD_WRITE_REGISTER, &params)
    }

    /// Reads an 8-bit value from a 16-bit register address.
    pub fn read_reg(&mut self, reg: u16) -> Result<u8, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let mut out = [0u8; 1];
        let res = (|| -> Result<(), SPI::Error> {
            self.spi
                .write(&[CMD_READ_REGISTER, (reg >> 8) as u8, reg as u8, 0x00])?;
            self.spi.read(&mut out)?;
            Ok(())
        })();
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)?;
        Ok(out[0])
    }

    /// Writes data into the internal 256-byte data buffer starting at the given offset.
    pub fn write_buffer(&mut self, offset: u8, data: &[u8]) -> Result<(), SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let res = (|| -> Result<(), SPI::Error> {
            self.spi.write(&[CMD_WRITE_BUFFER, offset])?;
            self.spi.write(data)?;
            Ok(())
        })();
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)
    }

    /// Reads data from the internal 256-byte data buffer starting at the given offset.
    pub fn read_buffer(&mut self, offset: u8, buf: &mut [u8]) -> Result<(), SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let res = (|| -> Result<(), SPI::Error> {
            self.spi.write(&[CMD_READ_BUFFER, offset, 0x00])?;
            self.spi.read(buf)?;
            Ok(())
        })();
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)
    }

    /// Queries the transceiver status byte (`CMD_GET_STATUS` `0xC0`).
    pub fn get_status(&mut self) -> Result<RadioStatus, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let mut rx = [0u8; 2];
        let res = self.spi.transfer(&mut rx, &[CMD_GET_STATUS, 0x00]);
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)?;
        Ok(RadioStatus::from_byte(rx[1]))
    }

    /// Places the transceiver into standby mode (`STDBY_CONFIG_RC` or `STDBY_CONFIG_XOSC`).
    pub fn set_standby(&mut self, config: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_STANDBY, &[config])
    }

    /// Configures regulator mode: LDO (`REGULATOR_LDO`) or DC-DC (`REGULATOR_DC_DC`).
    pub fn set_regulator_mode(&mut self, mode: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_REGULATOR_MODE, &[mode])
    }

    /// Configures DIO3 as a regulated TCXO supply voltage with stabilization delay ticks.
    pub fn set_dio3_as_tcxo_ctrl(
        &mut self,
        voltage: u8,
        delay_ticks: u32,
    ) -> Result<(), SxError<SPI::Error>> {
        let params = [
            voltage,
            (delay_ticks >> 16) as u8,
            (delay_ticks >> 8) as u8,
            delay_ticks as u8,
        ];
        self.write_cmd(CMD_SET_DIO3_AS_TCXO_CTRL, &params)
    }

    /// Calibrates image rejection for the given frequency range.
    pub fn calibrate_image(&mut self, freq1: u8, freq2: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_CALIBRATE_IMAGE, &[freq1, freq2])
    }

    /// Configures internal DIO2 to control the external RF switch.
    pub fn set_dio2_as_rf_switch_ctrl(&mut self, enable: bool) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(
            CMD_SET_DIO2_AS_RF_SWITCH_CTRL,
            &[if enable { 0x01 } else { 0x00 }],
        )
    }

    /// Sets the packet type modem: `PACKET_TYPE_GFSK` (`0x00`) or `PACKET_TYPE_LORA` (`0x01`).
    pub fn set_packet_type(&mut self, packet_type: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_PACKET_TYPE, &[packet_type])
    }

    /// Configures the RF carrier frequency in hertz.
    pub fn set_rf_frequency(&mut self, freq_hz: u32) -> Result<(), SxError<SPI::Error>> {
        let reg = calculate_rf_freq_reg(freq_hz);
        let params = [
            (reg >> 24) as u8,
            (reg >> 16) as u8,
            (reg >> 8) as u8,
            reg as u8,
        ];
        self.write_cmd(CMD_SET_RF_FREQUENCY, &params)
    }

    /// Configures the Power Amplifier (PA) parameters.
    pub fn set_pa_config(
        &mut self,
        pa_duty_cycle: u8,
        hp_max: u8,
        device_sel: u8,
        pa_lut: u8,
    ) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(
            CMD_SET_PA_CONFIG,
            &[pa_duty_cycle, hp_max, device_sel, pa_lut],
        )
    }

    /// Sets the transmit output power in dBm and ramp time.
    pub fn set_tx_params(
        &mut self,
        power_dbm: i8,
        ramp_time: u8,
    ) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_TX_PARAMS, &[power_dbm as u8, ramp_time])
    }

    /// Configures Over-Current Protection (OCP) clamp in register `0x08E7`.
    pub fn set_ocp(&mut self, ocp_val: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_reg(REG_OCP, ocp_val)
    }

    /// Configures TX and RX FIFO buffer base addresses.
    pub fn set_buffer_base_address(
        &mut self,
        tx_base: u8,
        rx_base: u8,
    ) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_BUFFER_BASE_ADDRESS, &[tx_base, rx_base])
    }

    /// Configures LoRa modulation parameters: Spreading Factor, Bandwidth, Coding Rate, and Low Data Rate Optimize.
    pub fn set_lora_modulation_params(
        &mut self,
        sf: u8,
        bw: u8,
        cr: u8,
        ldro: u8,
    ) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_MODULATION_PARAMS, &[sf, bw, cr, ldro])
    }

    /// Configures LoRa packet parameters.
    pub fn set_lora_packet_params(
        &mut self,
        preamble_len: u16,
        header_type: u8,
        payload_len: u8,
        crc_type: u8,
        invert_iq: u8,
    ) -> Result<(), SxError<SPI::Error>> {
        let params = [
            (preamble_len >> 8) as u8,
            preamble_len as u8,
            header_type,
            payload_len,
            crc_type,
            invert_iq,
        ];
        self.write_cmd(CMD_SET_PACKET_PARAMS, &params)
    }

    /// Configures the 16-bit LoRa sync word registers (`0x0740` and `0x0741`).
    pub fn set_lora_sync_word(&mut self, sync_word: u16) -> Result<(), SxError<SPI::Error>> {
        self.write_reg(REG_LORA_SYNC_WORD_MSB, (sync_word >> 8) as u8)?;
        self.write_reg(REG_LORA_SYNC_WORD_LSB, sync_word as u8)
    }

    /// Configures DIO interrupt routing lines.
    pub fn set_dio_irq_params(
        &mut self,
        irq_mask: u16,
        dio1_mask: u16,
        dio2_mask: u16,
        dio3_mask: u16,
    ) -> Result<(), SxError<SPI::Error>> {
        let params = [
            (irq_mask >> 8) as u8,
            irq_mask as u8,
            (dio1_mask >> 8) as u8,
            dio1_mask as u8,
            (dio2_mask >> 8) as u8,
            dio2_mask as u8,
            (dio3_mask >> 8) as u8,
            dio3_mask as u8,
        ];
        self.write_cmd(CMD_SET_DIO_IRQ_PARAMS, &params)
    }

    /// Reads the active 16-bit interrupt status flags.
    pub fn get_irq_status(&mut self) -> Result<u16, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let mut rx = [0u8; 4];
        let res = self
            .spi
            .transfer(&mut rx, &[CMD_GET_IRQ_STATUS, 0x00, 0x00, 0x00]);
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)?;
        Ok(((rx[2] as u16) << 8) | (rx[3] as u16))
    }

    /// Clears active interrupt status flags matching the bitmask.
    pub fn clear_irq_status(&mut self, clear_mask: u16) -> Result<(), SxError<SPI::Error>> {
        let params = [(clear_mask >> 8) as u8, clear_mask as u8];
        self.write_cmd(CMD_CLEAR_IRQ_STATUS, &params)
    }

    /// Commands the transceiver to initiate transmission with timeout ticks (0 = timeout disabled).
    pub fn set_tx(&mut self, timeout_ticks: u32) -> Result<(), SxError<SPI::Error>> {
        let params = [
            (timeout_ticks >> 16) as u8,
            (timeout_ticks >> 8) as u8,
            timeout_ticks as u8,
        ];
        self.write_cmd(CMD_SET_TX, &params)
    }

    /// Commands the transceiver to enter reception with timeout ticks (`0xFFFFFF` = continuous).
    pub fn set_rx(&mut self, timeout_ticks: u32) -> Result<(), SxError<SPI::Error>> {
        let params = [
            (timeout_ticks >> 16) as u8,
            (timeout_ticks >> 8) as u8,
            timeout_ticks as u8,
        ];
        self.write_cmd(CMD_SET_RX, &params)
    }

    /// Puts the radio into Channel Activity Detection (CAD) mode.
    ///
    /// Semtech SX1261/2 Section 13.1.8 "SetCad".
    pub fn set_cad(&mut self) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_CAD, &[])
    }

    /// Sets Channel Activity Detection parameters.
    ///
    /// Semtech SX1261/2 Section 13.4.11 "SetCadParams".
    pub fn set_cad_params(
        &mut self,
        cad_symbol_num: u8,
        cad_det_peak: u8,
        cad_det_min: u8,
        cad_exit_mode: u8,
        cad_timeout: u32,
    ) -> Result<(), SxError<SPI::Error>> {
        let params = [
            cad_symbol_num,
            cad_det_peak,
            cad_det_min,
            cad_exit_mode,
            ((cad_timeout >> 16) & 0xFF) as u8,
            ((cad_timeout >> 8) & 0xFF) as u8,
            (cad_timeout & 0xFF) as u8,
        ];
        self.write_cmd(CMD_SET_CAD_PARAMS, &params)
    }

    /// Reads instantaneous RSSI while in reception mode (returns value in dBm, e.g. -105 dBm).
    pub fn get_rssi_inst(&mut self) -> Result<i16, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let mut rx = [0u8; 3];
        let res = self.spi.transfer(&mut rx, &[CMD_GET_RSSI_INST, 0x00, 0x00]);
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)?;
        let rssi_dbm = -((rx[2] as i16) / 2);
        Ok(rssi_dbm)
    }

    /// Queries RX buffer status: returns `(payload_length, rx_start_buffer_pointer)`.
    pub fn get_rx_buffer_status(&mut self) -> Result<(u8, u8), SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let mut rx = [0u8; 4];
        let res = self
            .spi
            .transfer(&mut rx, &[CMD_GET_RX_BUFFER_STATUS, 0x00, 0x00, 0x00]);
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)?;
        Ok((rx[2], rx[3]))
    }

    /// Queries decoded packet status (packet RSSI in dBm, estimated SNR in dB).
    pub fn get_packet_status(&mut self) -> Result<PacketStatus, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.nss.set_low().map_err(|_| SxError::Gpio)?;
        let mut rx = [0u8; 5];
        let res = self
            .spi
            .transfer(&mut rx, &[CMD_GET_PACKET_STATUS, 0x00, 0x00, 0x00, 0x00]);
        let _ = self.nss.set_high();
        res.map_err(SxError::Spi)?;
        Ok(PacketStatus {
            rssi_pkt_dbm: -((rx[2] as i16) / 2),
            snr_pkt_db: (rx[3] as i8) / 4,
            signal_rssi_pkt_dbm: -((rx[4] as i16) / 2),
        })
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
        assert_eq!(CMD_SET_RF_FREQUENCY, 0x86);
        assert_eq!(CMD_SET_PACKET_PARAMS, 0x8C);
        assert_eq!(CMD_SET_MODULATION_PARAMS, 0x8B);
        assert_eq!(CMD_SET_DIO_IRQ_PARAMS, 0x08);
        assert_eq!(CMD_GET_IRQ_STATUS, 0x12);
        assert_eq!(CMD_CLEAR_IRQ_STATUS, 0x02);
        assert_eq!(CMD_SET_CAD, 0xC5);
        assert_eq!(CMD_SET_CAD_PARAMS, 0x88);
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

    #[test]
    fn frequency_calculation_matches_semtech_formula() {
        // 915.000 MHz = 0x3930_0000 = 959447040
        assert_eq!(calculate_rf_freq_reg(FREQ_BENCH_PING_HZ), 0x3930_0000);
        assert_eq!(calculate_rf_freq_reg(915_000_000), 959_447_040);

        // 917.625 MHz = 0x395A_0000 = 962199552
        assert_eq!(calculate_rf_freq_reg(FREQ_RX_SNIFFER_PRI_HZ), 0x395A_0000);
        assert_eq!(calculate_rf_freq_reg(917_625_000), 962_199_552);

        // 906.875 MHz = 0x38AE_0000 = 950927360
        assert_eq!(calculate_rf_freq_reg(FREQ_RX_SNIFFER_SEC_HZ), 0x38AE_0000);
        assert_eq!(calculate_rf_freq_reg(906_875_000), 950_927_360);

        // 868.000 MHz = 0x3640_0000 = 910163968
        assert_eq!(calculate_rf_freq_reg(868_000_000), 0x3640_0000);
    }

    #[test]
    fn sx1262_write_cmd_and_get_status_mock() {
        use embedded_hal_mock::eh1::digital::{
            Mock as PinMock, State as PinState, Transaction as PinTransaction,
        };
        use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

        let busy = PinMock::new(&[
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::Low),
        ]);
        let nss = PinMock::new(&[
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ]);

        let spi_txns = [
            // write_cmd SetStandby:
            SpiTransaction::write_vec(std::vec![CMD_SET_STANDBY]),
            SpiTransaction::write_vec(std::vec![STDBY_CONFIG_RC]),
            // get_status transfer:
            SpiTransaction::transfer(std::vec![CMD_GET_STATUS, 0x00], std::vec![0x24, 0x24]),
        ];
        let spi = SpiMock::new(&spi_txns);

        let mut sx = Sx1262::new(spi, nss, busy);
        sx.set_standby(STDBY_CONFIG_RC).unwrap();
        let status = sx.get_status().unwrap();
        assert_eq!(status.chip_mode, ChipMode::StbyRc);
        assert_eq!(status.command_status, CommandStatus::DataAvailable);

        let (mut spi, mut nss, mut busy) = sx.release();
        spi.done();
        nss.done();
        busy.done();
    }

    #[test]
    fn sx1262_buffer_read_write_mock() {
        use embedded_hal_mock::eh1::digital::{
            Mock as PinMock, State as PinState, Transaction as PinTransaction,
        };
        use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

        let busy = PinMock::new(&[
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::Low),
        ]);
        let nss = PinMock::new(&[
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ]);

        let payload = [0x50, 0x49, 0x4E, 0x47]; // "PING"
        let spi_txns = [
            // write_buffer:
            SpiTransaction::write_vec(std::vec![CMD_WRITE_BUFFER, 0x00]),
            SpiTransaction::write_vec(std::vec![0x50, 0x49, 0x4E, 0x47]),
            // read_buffer:
            SpiTransaction::write_vec(std::vec![CMD_READ_BUFFER, 0x00, 0x00]),
            SpiTransaction::read_vec(std::vec![0x50, 0x49, 0x4E, 0x47]),
        ];
        let spi = SpiMock::new(&spi_txns);

        let mut sx = Sx1262::new(spi, nss, busy);
        sx.write_buffer(0x00, &payload).unwrap();

        let mut read_buf = [0u8; 4];
        sx.read_buffer(0x00, &mut read_buf).unwrap();
        assert_eq!(read_buf, payload);

        let (mut spi, mut nss, mut busy) = sx.release();
        spi.done();
        nss.done();
        busy.done();
    }

    #[test]
    fn sx1262_packet_status_and_rssi_mock() {
        use embedded_hal_mock::eh1::digital::{
            Mock as PinMock, State as PinState, Transaction as PinTransaction,
        };
        use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

        let busy = PinMock::new(&[
            PinTransaction::get(PinState::Low),
            PinTransaction::get(PinState::Low),
        ]);
        let nss = PinMock::new(&[
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ]);

        let spi_txns = [
            // get_packet_status:
            // rx bytes: [status, status, rssi_pkt=168 (-> -84 dBm), snr_pkt=28 (-> 7 dB), signal_rssi=176 (-> -88 dBm)]
            SpiTransaction::transfer(
                std::vec![CMD_GET_PACKET_STATUS, 0x00, 0x00, 0x00, 0x00],
                std::vec![0x24, 0x24, 168, 28, 176],
            ),
            // get_rssi_inst:
            // rx bytes: [status, status, rssi_inst=216 (-> -108 dBm)]
            SpiTransaction::transfer(
                std::vec![CMD_GET_RSSI_INST, 0x00, 0x00],
                std::vec![0x24, 0x24, 216],
            ),
        ];
        let spi = SpiMock::new(&spi_txns);

        let mut sx = Sx1262::new(spi, nss, busy);
        let pkt = sx.get_packet_status().unwrap();
        assert_eq!(pkt.rssi_pkt_dbm, -84);
        assert_eq!(pkt.snr_pkt_db, 7);
        assert_eq!(pkt.signal_rssi_pkt_dbm, -88);

        let rssi = sx.get_rssi_inst().unwrap();
        assert_eq!(rssi, -108);

        let (mut spi, mut nss, mut busy) = sx.release();
        spi.done();
        nss.done();
        busy.done();
    }
}
