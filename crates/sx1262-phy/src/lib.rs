//! SX1262 commands over async `embedded-hal` SPI and cooperative BUSY polling.
//!
//! The caller supplies power, reset, oscillator and antenna policy through
//! [`Hooks`]. Hooks and chip commands await caller-supplied async I/O. A session begins
//! with [`Sx1262::startup`] and ends with [`Sx1262::shutdown`]. Packet operations,
//! channel changes and standby never change that lifetime.
//!
//! Command definitions follow Semtech SX1261/2 Rev 2.2 (Dec 2024), catalog
//! `sx1262`, §8.3.1 “BUSY Control Line” and §13 “Commands Interface”.
//! Choose PA, OCP, regulator, TCXO and DIO2 settings from the module circuit.
//! DIO2 RF switching is distinct from caller-controlled antenna readiness.
//! Readback of a control signal does not validate the RF path or antenna load.

#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]

mod backend;
mod session;
use lora_phy::mod_traits::RadioKind;
mod params;
mod receive;
pub use params::*;
pub use receive::*;
pub use session::*;

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

/// Semtech SX1262 Section 13.1.11 "SetRegulatorMode" command opcode (`0x96`).
pub const CMD_SET_REGULATOR_MODE: u8 = 0x96;

/// Semtech SX1262 Section 13.1.12 "Calibrate Function" command opcode (`0x89`).
pub const CMD_CALIBRATE_FUNCTION: u8 = 0x89;

/// Semtech SX1262 Section 13.1.13 "CalibrateImage" command opcode (`0x98`).
pub const CMD_CALIBRATE_IMAGE: u8 = 0x98;

/// Semtech SX1262 Section 13.1.14 "SetPaConfig" command opcode (`0x95`).
pub const CMD_SET_PA_CONFIG: u8 = 0x95;

/// Semtech SX1262 Section 13.1.15 "SetRxTxFallbackMode" command opcode (`0x93`).
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

/// Semtech SX1262 Section 13.3.3 "GetIrqStatus" command opcode (`0x12`).
pub const CMD_GET_IRQ_STATUS: u8 = 0x12;

/// Semtech SX1262 Section 13.3.4 "ClearIrqStatus" command opcode (`0x02`).
pub const CMD_CLEAR_IRQ_STATUS: u8 = 0x02;

/// Semtech SX1262 Section 13.3.5 "SetDIO2AsRfSwitchCtrl" command opcode (`0x9D`).
pub const CMD_SET_DIO2_AS_RF_SWITCH_CTRL: u8 = 0x9D;

/// Semtech SX1262 Section 13.3.6 "SetDIO3AsTCXOCtrl" command opcode (`0x97`).
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

/// Maximum six-bit OCP code, 157.5 mA in 2.5 mA steps.
/// Catalog `sx1262` §5.1 “Selecting DC-DC Converter or LDO Regulation”.
pub const OCP_MAX_CODE: u8 = 0x3F;

// =========================================================================
// Configuration Parameters & Bitfield Constants
// =========================================================================

/// Standby configuration: STDBY_RC (13 MHz RC oscillator, opcode param `0x00`).
pub const STDBY_CONFIG_RC: u8 = 0x00;

/// Standby configuration: STDBY_XOSC (32 MHz crystal oscillator, opcode param `0x01`).
pub const STDBY_CONFIG_XOSC: u8 = 0x01;

/// Internal LDO regulator mode; select according to the module circuit.
/// Semtech SX1261/2 Rev 2.2 §13.1.11 “SetRegulatorMode”.
pub const REGULATOR_LDO: u8 = 0x00;

/// Semtech SX1262 Section 13.1.11 "SetRegulatorMode": internal DC-DC converter enabled (opcode param `0x01`).
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

/// PA configuration: SX1262 device selection (`0x00`).
pub const PA_DEVICE_SEL_SX1262: u8 = 0x00;

/// PA configuration: default lookup table (`0x01`).
pub const PA_LUT_DEFAULT: u8 = 0x01;

/// Power amplifier ramp time: 40 us (`0x02`).
pub const RAMP_40_US: u8 = 0x02;

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

/// Lossless LoRa packet metrics, in half-dBm RSSI and quarter-dB SNR units.
/// Catalog `sx1262` §13.5.3 “GetPacketStatus”. Whole-unit accessors truncate
/// toward zero to preserve existing CDC/UI formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketStatus {
    /// Average packet RSSI multiplied by two.
    pub rssi_pkt_half_dbm: i16,
    /// Signed packet SNR multiplied by four.
    pub snr_pkt_quarter_db: i8,
    /// Signal RSSI multiplied by two.
    pub signal_rssi_pkt_half_dbm: i16,
}
impl PacketStatus {
    /// Whole packet dBm, truncated toward zero.
    pub const fn rssi_pkt_dbm(self) -> i16 {
        self.rssi_pkt_half_dbm / 2
    }
    /// Whole packet dB, truncated toward zero.
    pub const fn snr_pkt_db(self) -> i8 {
        self.snr_pkt_quarter_db / 4
    }
    /// Whole signal dBm, truncated toward zero.
    pub const fn signal_rssi_pkt_dbm(self) -> i16 {
        self.signal_rssi_pkt_half_dbm / 2
    }
}

/// Original transport faults or documented command failures.
#[derive(Debug, PartialEq)]
pub enum SxError<S, B = core::convert::Infallible> {
    /// Original SpiDevice error, including its chip-select error if available.
    Spi(S),
    /// Original BUSY input error.
    Busy(B),
    /// BUSY stayed high through the cooperative delay budget.
    BusyTimeout,
    /// Parameter outside a documented encoding or buffer range.
    InvalidParam,
    /// Original upstream command or validation error.
    Upstream(lora_phy::mod_params::RadioError),
    /// Command outside the supported non-continuous command set.
    UnsupportedCommand,
    /// Raw SetTx needs borrowed context and the session TX guard.
    ContextRequired,
}

/// Delay after NSS rises before checking BUSY, in microseconds.
/// Catalog `sx1262` §8.3.1 “BUSY Control Line”; bench validation is open.
pub const NSS_SETTLE_US: u32 = 1;
/// Maximum encoded RTC timeout, catalog `sx1262` §13.1.4 “SetTx”.
pub const MAX_TIMEOUT_TICKS: u32 = 0xFF_FFFF;
/// Continuous reception timeout, catalog `sx1262` §13.1.5 “SetRx”.
pub const RX_CONTINUOUS: u32 = MAX_TIMEOUT_TICKS;
/// TX-clamp register, catalog `sx1262` §12.1 “Registers”.
pub const REG_TX_CLAMP_CONFIG: u16 = 0x08D8;
/// PA-clamp threshold bits, catalog `sx1262` §15.2.2 “Workaround”.
pub const TX_CLAMP_MASK: u8 = 0x1E;
/// RTC counter control, catalog `sx1262` §15.3.2 “Workaround”.
pub const REG_RTC_CONTROL: u16 = 0x0902;
/// RTC event clear register, catalog `sx1262` §15.3.2 “Workaround”.
pub const REG_RTC_EVENT_CLEAR: u16 = 0x0944;
/// Clear pending RTC event while preserving unrelated bits (same section).
pub const RTC_EVENT_CLEAR_MASK: u8 = 1 << 1;

/// Cooperative BUSY delay budget. Scheduler latency is additional; this is
/// not a hard wall-clock deadline. Defaults to 100 ms with one-ms polls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BusyTiming {
    /// Total requested delay before timing out; zero tests the pin once.
    pub budget_ms: u32,
    /// Positive poll spacing; the final delay is capped to the remaining budget.
    pub poll_ms: core::num::NonZeroU32,
}
impl Default for BusyTiming {
    fn default() -> Self {
        Self {
            budget_ms: 100,
            poll_ms: core::num::NonZeroU32::MIN,
        }
    }
}

/// Owns an async SPI device, BUSY, delay, hooks and explicit session state.
/// Compose NSS and synchronization outside this crate. Await command futures
/// to completion; cancellation invalidates readiness and may leave NSS low.
pub struct Sx1262<SPI, BUSY, DELAY, H = DenyTx> {
    spi: SPI,
    busy: BUSY,
    delay: DELAY,
    hooks: H,
    busy_timing: BusyTiming,
    modulation: Option<BaseBandModulationParams>,
    packet: Option<LoRaPacketParams>,
    frequency_hz: Option<u32>,
    calibration_band: Option<CalibrationBand>,
    timed_rx: bool,
    state: SessionState,
    interval: core::num::NonZeroU32,
    stats: SessionStats,
}
impl<SPI, BUSY, DELAY> Sx1262<SPI, BUSY, DELAY> {
    /// Constructs without I/O. Default hooks deny every TX.
    pub fn new(spi: SPI, busy: BUSY, delay: DELAY) -> Self {
        Self::with_hooks(spi, busy, delay, DenyTx)
    }
}
impl<SPI, BUSY, DELAY, H> Sx1262<SPI, BUSY, DELAY, H> {
    /// Constructs without I/O; explicit startup is required for TX.
    pub fn with_hooks(spi: SPI, busy: BUSY, delay: DELAY, hooks: H) -> Self {
        Self {
            spi,
            busy,
            delay,
            hooks,
            busy_timing: BusyTiming::default(),
            modulation: None,
            packet: None,
            frequency_hz: None,
            calibration_band: None,
            timed_rx: false,
            state: SessionState::Inactive,
            interval: core::num::NonZeroU32::MIN,
            stats: SessionStats::new(),
        }
    }
    /// Returns device, BUSY, delay and hooks without any I/O. Shutdown is explicit.
    pub fn release(self) -> (SPI, BUSY, DELAY, H) {
        (self.spi, self.busy, self.delay, self.hooks)
    }
    /// Sets the delay budget without touching hardware or readiness.
    pub fn set_busy_timing(&mut self, timing: BusyTiming) {
        self.busy_timing = timing;
    }
    /// Current cooperative delay budget; no I/O.
    pub const fn busy_timing(&self) -> BusyTiming {
        self.busy_timing
    }
    /// Marks an in-flight sequence unsafe until its successful completion.
    /// Recording this before awaiting also catches cancellation without Drop I/O.
    fn begin_io(&mut self) -> (SessionState, core::num::NonZeroU32) {
        let saved = (self.state, self.interval);
        self.state = SessionState::NeedsShutdown;
        self.interval = core::num::NonZeroU32::MIN;
        saved
    }
    /// Restores prior readiness only on success. A later command cannot repair
    /// a previously interrupted sequence; shutdown/startup is required.
    fn end_io<T, S, B>(
        &mut self,
        saved: (SessionState, core::num::NonZeroU32),
        result: Result<T, SxError<S, B>>,
    ) -> Result<T, SxError<S, B>> {
        if result.is_ok() {
            (self.state, self.interval) = saved;
        } else {
            self.stats.failures = self.stats.failures.saturating_add(1);
        }
        result
    }
}
impl<SPI, BUSY, DELAY, H> Sx1262<SPI, BUSY, DELAY, H>
where
    SPI: embedded_hal_async::spi::SpiDevice,
    BUSY: embedded_hal::digital::InputPin,
    DELAY: embedded_hal_async::delay::DelayNs,
{
    /// Cooperatively waits for BUSY; interrupted/failed waits invalidate readiness.
    pub async fn wait_busy(&mut self) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        let saved = self.begin_io();
        let result = backend::wait_busy(&mut self.busy, &mut self.delay, self.busy_timing).await;
        self.end_io(saved, result)
    }
    /// Runs a device transaction with pre/post BUSY checks. NSS belongs to SPI.
    async fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        let saved = self.begin_io();
        let result = backend::transaction(
            &mut self.spi,
            &mut self.busy,
            &mut self.delay,
            self.busy_timing,
            operations,
        )
        .await;
        self.end_io(saved, result)
    }
    /// Writes a supported non-TX command. Raw TX cannot bypass session checks.
    pub async fn write_cmd(
        &mut self,
        cmd: u8,
        params: &[u8],
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if cmd == CMD_SET_TX {
            return Err(SxError::ContextRequired);
        }
        validate_command(cmd, params)?;
        // Leaving timed reception requires RTC cleanup even for raw commands.
        if self.timed_rx && matches!(cmd, CMD_SET_STANDBY | CMD_SET_FS | CMD_SET_RX | CMD_SET_CAD) {
            self.stop_rx().await?;
        }
        self.write_unchecked(cmd, params).await?;
        match cmd {
            CMD_SET_RX => self.timed_rx = params != [0, 0, 0] && params != [0xFF, 0xFF, 0xFF],
            CMD_CALIBRATE_IMAGE => {
                self.calibration_band = CalibrationBand::from_codes(params[0], params[1])
            }
            CMD_SET_MODULATION_PARAMS => self.modulation = None,
            CMD_SET_PACKET_PARAMS | CMD_SET_PACKET_TYPE => self.packet = None,
            _ => {}
        }
        Ok(())
    }
    /// Sends bytes through the checked transport; callers must validate first.
    async fn write_unchecked(
        &mut self,
        cmd: u8,
        params: &[u8],
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        use embedded_hal::spi::Operation;
        self.transaction(&mut [Operation::Write(&[cmd]), Operation::Write(params)])
            .await
    }
    /// Reads one complete command response, including command/status dummy bytes.
    async fn read_response<const N: usize>(
        &mut self,
        cmd: u8,
    ) -> Result<[u8; N], SxError<SPI::Error, BUSY::Error>> {
        let mut bytes = [0; N];
        bytes[0] = cmd;
        self.transaction(&mut [embedded_hal::spi::Operation::TransferInPlace(&mut bytes)])
            .await?;
        Ok(bytes)
    }
    /// Writes an 8-bit value to a 16-bit register address.
    pub async fn write_reg(
        &mut self,
        reg: u16,
        val: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        let params = [(reg >> 8) as u8, reg as u8, val];
        self.write_unchecked(CMD_WRITE_REGISTER, &params).await
    }

    /// Reads an 8-bit value from a 16-bit register address.
    pub async fn read_reg(&mut self, reg: u16) -> Result<u8, SxError<SPI::Error, BUSY::Error>> {
        let header = [CMD_READ_REGISTER, (reg >> 8) as u8, reg as u8, 0];
        let mut out = [0];
        self.transaction(&mut [
            embedded_hal::spi::Operation::Write(&header),
            embedded_hal::spi::Operation::Read(&mut out),
        ])
        .await?;
        Ok(out[0])
    }

    /// Writes data into the internal 256-byte data buffer starting at the given offset.
    pub async fn write_buffer(
        &mut self,
        offset: u8,
        data: &[u8],
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if data.len() > 256 {
            return Err(SxError::InvalidParam);
        }
        if offset == 0 {
            let saved = self.begin_io();
            let mut fault = None;
            let result = self.backend(&mut fault).set_payload(data).await;
            return self.end_io(saved, backend::finish(result, fault));
        }
        self.transaction(&mut [
            embedded_hal::spi::Operation::Write(&[CMD_WRITE_BUFFER, offset]),
            embedded_hal::spi::Operation::Write(data),
        ])
        .await
    }

    /// Reads data from the internal 256-byte data buffer starting at the given offset.
    pub async fn read_buffer(
        &mut self,
        offset: u8,
        buf: &mut [u8],
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if buf.len() > 256 {
            return Err(SxError::InvalidParam);
        }
        // The hardware FIFO wraps at 256; an offset near the end is valid.
        self.transaction(&mut [
            embedded_hal::spi::Operation::Write(&[CMD_READ_BUFFER, offset, 0]),
            embedded_hal::spi::Operation::Read(buf),
        ])
        .await
    }

    /// Queries the transceiver status byte (`CMD_GET_STATUS` `0xC0`).
    pub async fn get_status(&mut self) -> Result<RadioStatus, SxError<SPI::Error, BUSY::Error>> {
        Ok(RadioStatus::from_byte(
            self.read_response::<2>(CMD_GET_STATUS).await?[1],
        ))
    }

    /// Places the transceiver into standby mode (`STDBY_CONFIG_RC` or `STDBY_CONFIG_XOSC`).
    ///
    /// Semtech SX1262 Section 13.1.2 "SetStandby" (opcode `0x80`).
    pub async fn set_standby(
        &mut self,
        config: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if self.timed_rx {
            self.stop_rx().await?;
        }
        if config != STDBY_CONFIG_RC {
            return self.write_cmd(CMD_SET_STANDBY, &[config]).await;
        }
        let saved = self.begin_io();
        let mut fault = None;
        let result = self.backend(&mut fault).set_standby().await;
        self.end_io(saved, backend::finish(result, fault))
    }

    /// Configures regulator mode: LDO (`REGULATOR_LDO`) or DC-DC (`REGULATOR_DC_DC`).
    ///
    /// Semtech SX1262 Section 13.1.11 "SetRegulatorMode" (opcode `0x96`).
    pub async fn set_regulator_mode(
        &mut self,
        mode: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(CMD_SET_REGULATOR_MODE, &[mode]).await
    }

    /// Configures DIO3 as a regulated TCXO supply voltage with stabilization delay ticks.
    ///
    /// Semtech SX1262 Section 13.3.6 "SetDIO3AsTCXOCtrl" (opcode `0x97`).
    pub async fn set_dio3_as_tcxo_ctrl(
        &mut self,
        voltage: u8,
        delay_ticks: u32,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if delay_ticks > 0xFF_FFFF {
            return Err(SxError::InvalidParam);
        }
        let params = [
            voltage,
            (delay_ticks >> 16) as u8,
            (delay_ticks >> 8) as u8,
            delay_ticks as u8,
        ];
        self.write_cmd(CMD_SET_DIO3_AS_TCXO_CTRL, &params).await
    }

    /// Calibrates image rejection for the given frequency range.
    ///
    /// Semtech SX1262 Section 13.1.13 "CalibrateImage" (opcode `0x98`).
    pub async fn calibrate_image(
        &mut self,
        freq1: u8,
        freq2: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(CMD_CALIBRATE_IMAGE, &[freq1, freq2]).await
    }

    /// Configures internal DIO2 to control the external RF switch.
    ///
    /// Semtech SX1262 Section 13.3.5 "SetDIO2AsRfSwitchCtrl" (opcode `0x9D`).
    pub async fn set_dio2_as_rf_switch_ctrl(
        &mut self,
        enable: bool,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(
            CMD_SET_DIO2_AS_RF_SWITCH_CTRL,
            &[if enable { 0x01 } else { 0x00 }],
        )
        .await
    }

    /// Sets the packet type modem: `PACKET_TYPE_GFSK` (`0x00`) or `PACKET_TYPE_LORA` (`0x01`).
    ///
    /// Semtech SX1262 Section 13.4.2 "SetPacketType" (opcode `0x8A`).
    pub async fn set_packet_type(
        &mut self,
        packet_type: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(CMD_SET_PACKET_TYPE, &[packet_type]).await
    }

    /// Configures the RF carrier frequency in hertz.
    ///
    /// Semtech SX1262 Section 13.4.1 "SetRfFrequency" (opcode `0x86`).
    pub async fn set_rf_frequency(
        &mut self,
        freq_hz: u32,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if !(150_000_000..=960_000_000).contains(&freq_hz) {
            return Err(SxError::InvalidParam);
        }
        if let Some(band) = CalibrationBand::for_frequency(freq_hz) {
            if self.calibration_band != Some(band) {
                let (a, b) = band.codes();
                self.calibrate_image(a, b).await?;
            }
        } else {
            self.calibration_band = None;
        }
        let saved = self.begin_io();
        let mut fault = None;
        let result = self.backend(&mut fault).set_channel(freq_hz).await;
        self.end_io(saved, backend::finish(result, fault))?;
        self.frequency_hz = Some(freq_hz);
        Ok(())
    }

    /// Configures the Power Amplifier (PA) parameters.
    ///
    /// Semtech SX1262 Section 13.1.14 "SetPaConfig" (opcode `0x95`).
    pub async fn set_pa_config(
        &mut self,
        pa_duty_cycle: u8,
        hp_max: u8,
        device_sel: u8,
        pa_lut: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(
            CMD_SET_PA_CONFIG,
            &[pa_duty_cycle, hp_max, device_sel, pa_lut],
        )
        .await
    }

    /// Sets the transmit output power in dBm and ramp time.
    ///
    /// Semtech SX1262 Section 13.4.4 "SetTxParams" (opcode `0x8E`).
    pub async fn set_tx_params(
        &mut self,
        power_dbm: i8,
        ramp_time: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(CMD_SET_TX_PARAMS, &[power_dbm as u8, ramp_time])
            .await
    }

    /// Configures Over-Current Protection (OCP) clamp in register `0x08E7`.
    ///
    /// Catalog `sx1262` §5.1 “Selecting DC-DC Converter or LDO Regulation”
    /// limits OCP to six bits, in 2.5 mA steps. PA changes reset this register.
    /// Register address: §12.1 “Registers” (OCP).
    pub async fn set_ocp(&mut self, ocp_val: u8) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if ocp_val > OCP_MAX_CODE {
            return Err(SxError::InvalidParam);
        }
        self.write_reg(REG_OCP, ocp_val).await
    }

    /// Configures TX and RX FIFO buffer base addresses.
    pub async fn set_buffer_base_address(
        &mut self,
        tx_base: u8,
        rx_base: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        let saved = self.begin_io();
        let mut fault = None;
        let result = self
            .backend(&mut fault)
            .set_tx_rx_buffer_base_address(usize::from(tx_base), usize::from(rx_base))
            .await;
        self.end_io(saved, backend::finish(result, fault))
    }

    /// Configures LoRa modulation parameters: Spreading Factor, Bandwidth, Coding Rate, and Low Data Rate Optimize.
    pub async fn set_lora_modulation_params(
        &mut self,
        sf: u8,
        bw: u8,
        cr: u8,
        ldro: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        validate_command::<SPI::Error, BUSY::Error>(
            CMD_SET_MODULATION_PARAMS,
            &[sf, bw, cr, ldro],
        )?;
        let spreading = match sf {
            5 => SpreadingFactor::_5,
            6 => SpreadingFactor::_6,
            7 => SpreadingFactor::_7,
            8 => SpreadingFactor::_8,
            9 => SpreadingFactor::_9,
            10 => SpreadingFactor::_10,
            11 => SpreadingFactor::_11,
            _ => SpreadingFactor::_12,
        };
        let bandwidth = match bw {
            0 => Bandwidth::_7KHz,
            8 => Bandwidth::_10KHz,
            1 => Bandwidth::_15KHz,
            9 => Bandwidth::_20KHz,
            2 => Bandwidth::_31KHz,
            10 => Bandwidth::_41KHz,
            3 => Bandwidth::_62KHz,
            4 => Bandwidth::_125KHz,
            5 => Bandwidth::_250KHz,
            _ => Bandwidth::_500KHz,
        };
        let coding = match cr {
            1 => CodingRate::_4_5,
            2 => CodingRate::_4_6,
            3 => CodingRate::_4_7,
            _ => CodingRate::_4_8,
        };
        let mut params = BaseBandModulationParams::new(spreading, bandwidth, coding);
        params.ldro = ldro != 0;
        self.configure_lora_modulation(&params).await
    }

    /// Configures LoRa packet parameters.
    pub async fn set_lora_packet_params(
        &mut self,
        preamble_len: u16,
        header_type: u8,
        payload_len: u8,
        crc_type: u8,
        invert_iq: u8,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if header_type > 1 || crc_type > 1 || invert_iq > 1 {
            return Err(SxError::InvalidParam);
        }
        self.configure_lora_packet(&LoRaPacketParams {
            preamble_symbols: preamble_len,
            implicit_header: header_type != 0,
            payload_len,
            crc: crc_type != 0,
            invert_iq: invert_iq != 0,
        })
        .await
    }

    /// Configures the 16-bit LoRa sync word registers (`0x0740` and `0x0741`).
    pub async fn set_lora_sync_word(
        &mut self,
        sync_word: u16,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_reg(REG_LORA_SYNC_WORD_MSB, (sync_word >> 8) as u8)
            .await?;
        self.write_reg(REG_LORA_SYNC_WORD_LSB, sync_word as u8)
            .await
    }

    /// Configures DIO interrupt routing lines.
    pub async fn set_dio_irq_params(
        &mut self,
        irq_mask: u16,
        dio1_mask: u16,
        dio2_mask: u16,
        dio3_mask: u16,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
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
        self.write_cmd(CMD_SET_DIO_IRQ_PARAMS, &params).await
    }

    /// Reads the active 16-bit interrupt status flags.
    pub async fn get_irq_status(&mut self) -> Result<u16, SxError<SPI::Error, BUSY::Error>> {
        let rx = self.read_response::<4>(CMD_GET_IRQ_STATUS).await?;
        Ok(u16::from_be_bytes([rx[2], rx[3]]))
    }

    /// Clears active interrupt status flags matching the bitmask.
    pub async fn clear_irq_status(
        &mut self,
        clear_mask: u16,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        let params = [(clear_mask >> 8) as u8, clear_mask as u8];
        self.write_cmd(CMD_CLEAR_IRQ_STATUS, &params).await
    }

    /// Commands the transceiver to enter reception with timeout ticks (`0xFFFFFF` = continuous).
    pub async fn set_rx(
        &mut self,
        timeout_ticks: u32,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if timeout_ticks > 0xFF_FFFF {
            return Err(SxError::InvalidParam);
        }
        let params = [
            (timeout_ticks >> 16) as u8,
            (timeout_ticks >> 8) as u8,
            timeout_ticks as u8,
        ];
        self.write_cmd(CMD_SET_RX, &params).await
    }

    /// Puts the radio into Channel Activity Detection (CAD) mode.
    ///
    /// Semtech SX1261/2 Section 13.1.8 "SetCad".
    pub async fn set_cad(&mut self) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(CMD_SET_CAD, &[]).await
    }

    /// Sets Channel Activity Detection parameters.
    ///
    /// Semtech SX1261/2 Section 13.4.7 "SetCadParams".
    pub async fn set_cad_params(
        &mut self,
        cad_symbol_num: u8,
        cad_det_peak: u8,
        cad_det_min: u8,
        cad_exit_mode: u8,
        cad_timeout: u32,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        if cad_timeout > 0xFF_FFFF {
            return Err(SxError::InvalidParam);
        }
        let params = [
            cad_symbol_num,
            cad_det_peak,
            cad_det_min,
            cad_exit_mode,
            ((cad_timeout >> 16) & 0xFF) as u8,
            ((cad_timeout >> 8) & 0xFF) as u8,
            (cad_timeout & 0xFF) as u8,
        ];
        self.write_cmd(CMD_SET_CAD_PARAMS, &params).await
    }

    /// Reads instantaneous RSSI while in reception mode (returns value in dBm, e.g. -105 dBm).
    pub async fn get_rssi_inst(&mut self) -> Result<i16, SxError<SPI::Error, BUSY::Error>> {
        Ok(self.get_rssi_inst_half_dbm().await? / 2)
    }

    /// Queries RX buffer status: returns `(payload_length, rx_start_buffer_pointer)`.
    pub async fn get_rx_buffer_status(
        &mut self,
    ) -> Result<(u8, u8), SxError<SPI::Error, BUSY::Error>> {
        let rx = self.read_response::<4>(CMD_GET_RX_BUFFER_STATUS).await?;
        Ok((rx[2], rx[3]))
    }

    /// Queries lossless packet RSSI in half-dBm and estimated SNR in quarter-dB.
    pub async fn get_packet_status(
        &mut self,
    ) -> Result<PacketStatus, SxError<SPI::Error, BUSY::Error>> {
        let rx = self.read_response::<5>(CMD_GET_PACKET_STATUS).await?;
        Ok(PacketStatus {
            rssi_pkt_half_dbm: -i16::from(rx[2]),
            snr_pkt_quarter_db: rx[3] as i8,
            signal_rssi_pkt_half_dbm: -i16::from(rx[4]),
        })
    }
}
