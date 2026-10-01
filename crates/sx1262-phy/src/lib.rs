//! SX1262 commands over blocking `embedded-hal` 1.0 SPI and GPIO.
//!
//! The caller supplies power, reset, oscillator and antenna policy through
//! [`Hooks`]. Hooks may await; SPI commands remain blocking. A session begins
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
pub use params::*;
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

/// Errors from blocking chip operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SxError<E> {
    /// SPI bus error; NSS release was attempted.
    Spi(E),
    /// BUSY stayed high for the bounded poll budget.
    BusyTimeout,
    /// Parameter outside a documented encoding or buffer range.
    InvalidParam,
    /// NSS or BUSY GPIO error.
    Gpio,
    /// Command outside the supported non-continuous command set.
    UnsupportedCommand,
    /// Raw `SetTx` needs borrowed hook context and the TX guard.
    ContextRequired,
}

/// BUSY poll budget in CPU-dependent iterations. Duration depends on the caller's CPU.
pub const BUSY_TIMEOUT_POLLS: u32 = 50_000;

/// Owns a dedicated SPI bus, NSS, BUSY, hooks and session state.
/// No synchronization or executor is supplied by this crate.
pub struct Sx1262<SPI, NSS, BUSY, H = DenyTx> {
    spi: SPI,
    nss: NSS,
    busy: BUSY,
    hooks: H,
    modulation: Option<BaseBandModulationParams>,
    frequency_hz: Option<u32>,
    state: SessionState,
    interval: core::num::NonZeroU32,
    stats: SessionStats,
}

impl<SPI, NSS, BUSY> Sx1262<SPI, NSS, BUSY> {
    /// Constructs without device I/O. The default policy denies every TX.
    pub const fn new(spi: SPI, nss: NSS, busy: BUSY) -> Self {
        Self::with_hooks(spi, nss, busy, DenyTx)
    }
}

impl<SPI, NSS, BUSY, H> Sx1262<SPI, NSS, BUSY, H> {
    /// Constructs without device I/O. Explicit startup is required for TX.
    pub const fn with_hooks(spi: SPI, nss: NSS, busy: BUSY, hooks: H) -> Self {
        Self {
            spi,
            nss,
            busy,
            hooks,
            modulation: None,
            frequency_hz: None,
            state: SessionState::Inactive,
            interval: core::num::NonZeroU32::MIN,
            stats: SessionStats::new(),
        }
    }

    /// Returns buses, pins and hooks without shutdown. Call shutdown explicitly
    /// before release if the caller needs the module powered off.
    pub fn release(self) -> (SPI, NSS, BUSY, H) {
        (self.spi, self.nss, self.busy, self.hooks)
    }
}

impl<SPI, NSS, BUSY, H> Sx1262<SPI, NSS, BUSY, H>
where
    SPI: embedded_hal::spi::SpiBus,
    NSS: embedded_hal::digital::OutputPin,
    BUSY: embedded_hal::digital::InputPin,
{
    /// Waits for the BUSY pin to drop low, indicating transceiver readiness.
    pub fn wait_busy(&mut self) -> Result<(), SxError<SPI::Error>> {
        for _ in 0..BUSY_TIMEOUT_POLLS {
            if self.busy.is_low().map_err(|_| SxError::Gpio)? {
                return Ok(());
            }
        }
        Err(SxError::BusyTimeout)
    }

    /// Attempts NSS release even if selecting the chip reports a GPIO failure.
    fn select(&mut self) -> Result<(), SxError<SPI::Error>> {
        if self.nss.set_low().is_err() {
            let _ = self.nss.set_high();
            return Err(SxError::Gpio);
        }
        Ok(())
    }

    /// Writes a supported non-TX command. Unknown commands and sleep are rejected.
    /// `SetTx` requires [`Self::write_cmd_with_context`] so hooks cannot be bypassed.
    pub fn write_cmd(&mut self, cmd: u8, params: &[u8]) -> Result<(), SxError<SPI::Error>> {
        if cmd == CMD_SET_TX {
            return Err(SxError::ContextRequired);
        }
        validate_command(cmd, params)?;
        self.write_unchecked(cmd, params)
    }

    /// Sends validated bytes while BUSY is low; flushes SPI before releasing NSS.
    fn write_unchecked(&mut self, cmd: u8, params: &[u8]) -> Result<(), SxError<SPI::Error>> {
        self.wait_busy()?;
        self.select()?;
        let res = (|| {
            self.spi.write(&[cmd])?;
            if !params.is_empty() {
                self.spi.write(params)?;
            }
            Ok(())
        })();
        self.finish_transaction(res)
    }

    /// Completes the bus transfer and attempts NSS release even after SPI failure.
    fn finish_transaction(
        &mut self,
        result: Result<(), SPI::Error>,
    ) -> Result<(), SxError<SPI::Error>> {
        let result = result.and_then(|()| self.spi.flush());
        let deselect = self.nss.set_high();
        result.map_err(SxError::Spi)?;
        deselect.map_err(|_| SxError::Gpio)
    }

    /// Writes an 8-bit value to a 16-bit register address.
    pub fn write_reg(&mut self, reg: u16, val: u8) -> Result<(), SxError<SPI::Error>> {
        let params = [(reg >> 8) as u8, reg as u8, val];
        self.write_cmd(CMD_WRITE_REGISTER, &params)
    }

    /// Reads an 8-bit value from a 16-bit register address.
    pub fn read_reg(&mut self, reg: u16) -> Result<u8, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.select()?;
        let mut out = [0u8; 1];
        let res = (|| -> Result<(), SPI::Error> {
            self.spi
                .write(&[CMD_READ_REGISTER, (reg >> 8) as u8, reg as u8, 0x00])?;
            self.spi.read(&mut out)?;
            Ok(())
        })();
        self.finish_transaction(res)?;
        Ok(out[0])
    }

    /// Writes data into the internal 256-byte data buffer starting at the given offset.
    pub fn write_buffer(&mut self, offset: u8, data: &[u8]) -> Result<(), SxError<SPI::Error>> {
        if data.len() > 256 - usize::from(offset) {
            return Err(SxError::InvalidParam);
        }
        if offset == 0 {
            let mut fault = None;
            let result = backend::run_ready(self.backend(&mut fault, false).set_payload(data));
            return backend::finish(result, fault);
        }
        self.wait_busy()?;
        self.select()?;
        let res = (|| -> Result<(), SPI::Error> {
            self.spi.write(&[CMD_WRITE_BUFFER, offset])?;
            self.spi.write(data)?;
            Ok(())
        })();
        self.finish_transaction(res)
    }

    /// Reads data from the internal 256-byte data buffer starting at the given offset.
    pub fn read_buffer(&mut self, offset: u8, buf: &mut [u8]) -> Result<(), SxError<SPI::Error>> {
        if buf.len() > 256 - usize::from(offset) {
            return Err(SxError::InvalidParam);
        }
        self.wait_busy()?;
        self.select()?;
        let res = (|| -> Result<(), SPI::Error> {
            self.spi.write(&[CMD_READ_BUFFER, offset, 0x00])?;
            self.spi.read(buf)?;
            Ok(())
        })();
        self.finish_transaction(res)
    }

    /// Queries the transceiver status byte (`CMD_GET_STATUS` `0xC0`).
    pub fn get_status(&mut self) -> Result<RadioStatus, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.select()?;
        let mut rx = [0u8; 2];
        let res = self.spi.transfer(&mut rx, &[CMD_GET_STATUS, 0x00]);
        self.finish_transaction(res)?;
        Ok(RadioStatus::from_byte(rx[1]))
    }

    /// Places the transceiver into standby mode (`STDBY_CONFIG_RC` or `STDBY_CONFIG_XOSC`).
    ///
    /// Semtech SX1262 Section 13.1.2 "SetStandby" (opcode `0x80`).
    pub fn set_standby(&mut self, config: u8) -> Result<(), SxError<SPI::Error>> {
        if config != STDBY_CONFIG_RC {
            return self.write_cmd(CMD_SET_STANDBY, &[config]);
        }
        let mut fault = None;
        let result = backend::run_ready(self.backend(&mut fault, false).set_standby());
        backend::finish(result, fault)
    }

    /// Configures regulator mode: LDO (`REGULATOR_LDO`) or DC-DC (`REGULATOR_DC_DC`).
    ///
    /// Semtech SX1262 Section 13.1.11 "SetRegulatorMode" (opcode `0x96`).
    pub fn set_regulator_mode(&mut self, mode: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_REGULATOR_MODE, &[mode])
    }

    /// Configures DIO3 as a regulated TCXO supply voltage with stabilization delay ticks.
    ///
    /// Semtech SX1262 Section 13.3.6 "SetDIO3AsTCXOCtrl" (opcode `0x97`).
    pub fn set_dio3_as_tcxo_ctrl(
        &mut self,
        voltage: u8,
        delay_ticks: u32,
    ) -> Result<(), SxError<SPI::Error>> {
        if delay_ticks > 0xFF_FFFF {
            return Err(SxError::InvalidParam);
        }
        let params = [
            voltage,
            (delay_ticks >> 16) as u8,
            (delay_ticks >> 8) as u8,
            delay_ticks as u8,
        ];
        self.write_cmd(CMD_SET_DIO3_AS_TCXO_CTRL, &params)
    }

    /// Calibrates image rejection for the given frequency range.
    ///
    /// Semtech SX1262 Section 13.1.13 "CalibrateImage" (opcode `0x98`).
    pub fn calibrate_image(&mut self, freq1: u8, freq2: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_CALIBRATE_IMAGE, &[freq1, freq2])
    }

    /// Configures internal DIO2 to control the external RF switch.
    ///
    /// Semtech SX1262 Section 13.3.5 "SetDIO2AsRfSwitchCtrl" (opcode `0x9D`).
    pub fn set_dio2_as_rf_switch_ctrl(&mut self, enable: bool) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(
            CMD_SET_DIO2_AS_RF_SWITCH_CTRL,
            &[if enable { 0x01 } else { 0x00 }],
        )
    }

    /// Sets the packet type modem: `PACKET_TYPE_GFSK` (`0x00`) or `PACKET_TYPE_LORA` (`0x01`).
    ///
    /// Semtech SX1262 Section 13.4.2 "SetPacketType" (opcode `0x8A`).
    pub fn set_packet_type(&mut self, packet_type: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_PACKET_TYPE, &[packet_type])
    }

    /// Configures the RF carrier frequency in hertz.
    ///
    /// Semtech SX1262 Section 13.4.1 "SetRfFrequency" (opcode `0x86`).
    pub fn set_rf_frequency(&mut self, freq_hz: u32) -> Result<(), SxError<SPI::Error>> {
        if !(150_000_000..=960_000_000).contains(&freq_hz) {
            return Err(SxError::InvalidParam);
        }
        let mut fault = None;
        let result = backend::run_ready(self.backend(&mut fault, false).set_channel(freq_hz));
        backend::finish(result, fault)?;
        self.frequency_hz = Some(freq_hz);
        Ok(())
    }

    /// Configures the Power Amplifier (PA) parameters.
    ///
    /// Semtech SX1262 Section 13.1.14 "SetPaConfig" (opcode `0x95`).
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
    ///
    /// Semtech SX1262 Section 13.4.4 "SetTxParams" (opcode `0x8E`).
    pub fn set_tx_params(
        &mut self,
        power_dbm: i8,
        ramp_time: u8,
    ) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_SET_TX_PARAMS, &[power_dbm as u8, ramp_time])
    }

    /// Configures Over-Current Protection (OCP) clamp in register `0x08E7`.
    ///
    /// Catalog `sx1262` §5.1 “Selecting DC-DC Converter or LDO Regulation”
    /// limits OCP to six bits, in 2.5 mA steps. PA changes reset this register.
    /// Register address: §12.1 “Registers” (OCP).
    pub fn set_ocp(&mut self, ocp_val: u8) -> Result<(), SxError<SPI::Error>> {
        if ocp_val > OCP_MAX_CODE {
            return Err(SxError::InvalidParam);
        }
        self.write_reg(REG_OCP, ocp_val)
    }

    /// Configures TX and RX FIFO buffer base addresses.
    pub fn set_buffer_base_address(
        &mut self,
        tx_base: u8,
        rx_base: u8,
    ) -> Result<(), SxError<SPI::Error>> {
        let mut fault = None;
        let result = backend::run_ready(
            self.backend(&mut fault, false)
                .set_tx_rx_buffer_base_address(usize::from(tx_base), usize::from(rx_base)),
        );
        backend::finish(result, fault)
    }

    /// Configures LoRa modulation parameters: Spreading Factor, Bandwidth, Coding Rate, and Low Data Rate Optimize.
    pub fn set_lora_modulation_params(
        &mut self,
        sf: u8,
        bw: u8,
        cr: u8,
        ldro: u8,
    ) -> Result<(), SxError<SPI::Error>> {
        validate_command::<SPI::Error>(CMD_SET_MODULATION_PARAMS, &[sf, bw, cr, ldro])?;
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
        self.configure_lora_modulation(&params)
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
        self.select()?;
        let mut rx = [0u8; 4];
        let res = self
            .spi
            .transfer(&mut rx, &[CMD_GET_IRQ_STATUS, 0x00, 0x00, 0x00]);
        self.finish_transaction(res)?;
        Ok(((rx[2] as u16) << 8) | (rx[3] as u16))
    }

    /// Clears active interrupt status flags matching the bitmask.
    pub fn clear_irq_status(&mut self, clear_mask: u16) -> Result<(), SxError<SPI::Error>> {
        let params = [(clear_mask >> 8) as u8, clear_mask as u8];
        self.write_cmd(CMD_CLEAR_IRQ_STATUS, &params)
    }

    /// Commands the transceiver to enter reception with timeout ticks (`0xFFFFFF` = continuous).
    pub fn set_rx(&mut self, timeout_ticks: u32) -> Result<(), SxError<SPI::Error>> {
        if timeout_ticks > 0xFF_FFFF {
            return Err(SxError::InvalidParam);
        }
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
    /// Semtech SX1261/2 Section 13.4.7 "SetCadParams".
    pub fn set_cad_params(
        &mut self,
        cad_symbol_num: u8,
        cad_det_peak: u8,
        cad_det_min: u8,
        cad_exit_mode: u8,
        cad_timeout: u32,
    ) -> Result<(), SxError<SPI::Error>> {
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
        self.write_cmd(CMD_SET_CAD_PARAMS, &params)
    }

    /// Reads instantaneous RSSI while in reception mode (returns value in dBm, e.g. -105 dBm).
    pub fn get_rssi_inst(&mut self) -> Result<i16, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.select()?;
        let mut rx = [0u8; 3];
        let res = self.spi.transfer(&mut rx, &[CMD_GET_RSSI_INST, 0x00, 0x00]);
        self.finish_transaction(res)?;
        let rssi_dbm = -((rx[2] as i16) / 2);
        Ok(rssi_dbm)
    }

    /// Queries RX buffer status: returns `(payload_length, rx_start_buffer_pointer)`.
    pub fn get_rx_buffer_status(&mut self) -> Result<(u8, u8), SxError<SPI::Error>> {
        self.wait_busy()?;
        self.select()?;
        let mut rx = [0u8; 4];
        let res = self
            .spi
            .transfer(&mut rx, &[CMD_GET_RX_BUFFER_STATUS, 0x00, 0x00, 0x00]);
        self.finish_transaction(res)?;
        Ok((rx[2], rx[3]))
    }

    /// Queries decoded packet status (packet RSSI in dBm, estimated SNR in dB).
    pub fn get_packet_status(&mut self) -> Result<PacketStatus, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.select()?;
        let mut rx = [0u8; 5];
        let res = self
            .spi
            .transfer(&mut rx, &[CMD_GET_PACKET_STATUS, 0x00, 0x00, 0x00, 0x00]);
        self.finish_transaction(res)?;
        Ok(PacketStatus {
            rssi_pkt_dbm: -((rx[2] as i16) / 2),
            snr_pkt_db: (rx[3] as i8) / 4,
            signal_rssi_pkt_dbm: -((rx[4] as i16) / 2),
        })
    }
}
