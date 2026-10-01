//! LoRa types shared with upstream lora-rs and chip encodings.
use crate::*;
use lora_phy::mod_traits::RadioKind;

/// Upstream LoRa modulation types and symbol timing from `lora-modulation`.
pub use lora_modulation::{Bandwidth, BaseBandModulationParams, CodingRate, SpreadingFactor};

/// LoRa packet fields from catalog `sx1262` §13.4.6 “SetPacketParams”.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoRaPacketParams {
    /// Preamble length in symbols. SF5/SF6 require at least twelve symbols.
    pub preamble_symbols: u16,
    /// Fixed length header when true, explicit length header otherwise.
    pub implicit_header: bool,
    /// Payload length (TX) or maximum accepted length (explicit-header RX).
    pub payload_len: u8,
    /// Append/check the LoRa payload CRC.
    pub crc: bool,
    /// Invert IQ polarity; peers must use matching settings.
    pub invert_iq: bool,
}

impl<SPI, NSS, BUSY, H> Sx1262<SPI, NSS, BUSY, H>
where
    SPI: embedded_hal::spi::SpiBus,
    NSS: embedded_hal::digital::OutputPin,
    BUSY: embedded_hal::digital::InputPin,
{
    /// Encodes upstream modulation parameters for SX1262. The caller selects
    /// LDRO through the upstream constructor; inconsistent overrides are rejected.
    pub fn configure_lora_modulation(
        &mut self,
        params: &BaseBandModulationParams,
    ) -> Result<(), SxError<SPI::Error>> {
        let calculated = BaseBandModulationParams::new(params.sf, params.bw, params.cr);
        if params.ldro != calculated.ldro {
            return Err(SxError::InvalidParam);
        }
        let mut fault = None;
        let frequency = self.frequency_hz.unwrap_or(960_000_000);
        let result = {
            let mut radio = self.backend(&mut fault, false);
            radio
                .create_modulation_params(params.sf, params.bw, params.cr, frequency)
                .and_then(|modulation| backend::run_ready(radio.set_modulation_params(&modulation)))
        };
        backend::finish(result, fault)?;
        self.modulation = Some(*params);
        Ok(())
    }

    /// Encodes packet fields; upstream raises SF5/SF6 preambles to twelve symbols.
    pub fn configure_lora_packet(
        &mut self,
        params: &LoRaPacketParams,
    ) -> Result<(), SxError<SPI::Error>> {
        let modulation = self.modulation.ok_or(SxError::InvalidParam)?;
        let mut fault = None;
        let frequency = self.frequency_hz.unwrap_or(960_000_000);
        let result = {
            let mut radio = self.backend(&mut fault, false);
            radio
                .create_modulation_params(modulation.sf, modulation.bw, modulation.cr, frequency)
                .and_then(|modulation| {
                    radio.create_packet_params(
                        params.preamble_symbols,
                        params.implicit_header,
                        params.payload_len,
                        params.crc,
                        params.invert_iq,
                        &modulation,
                    )
                })
                .and_then(|packet| backend::run_ready(radio.set_packet_params(&packet)))
        };
        backend::finish(result, fault)
    }

    /// Calibrates selected blocks in standby RC (`sx1262` §13.1.12 “Calibrate Function”).
    pub fn calibrate(&mut self, mask: u8) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_CALIBRATE_FUNCTION, &[mask])
    }

    /// Returns latched device error bits (`sx1262` §13.6.1 “GetDeviceErrors”).
    pub fn get_device_errors(&mut self) -> Result<u16, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.select()?;
        let mut rx = [0; 4];
        let result = self
            .spi
            .transfer(&mut rx, &[CMD_GET_DEVICE_ERRORS, 0, 0, 0]);
        self.finish_transaction(result)?;
        Ok(u16::from_be_bytes([rx[2], rx[3]]))
    }

    /// Clears device errors (`sx1262` §13.6.2 “ClearDeviceErrors”).
    pub fn clear_device_errors(&mut self) -> Result<(), SxError<SPI::Error>> {
        self.write_cmd(CMD_CLEAR_DEVICE_ERRORS, &[0, 0])
    }

    /// Queries the active modem (`sx1262` §13.4.3 “GetPacketType”).
    pub fn get_packet_type(&mut self) -> Result<u8, SxError<SPI::Error>> {
        self.wait_busy()?;
        self.select()?;
        let mut rx = [0; 3];
        let result = self.spi.transfer(&mut rx, &[CMD_GET_PACKET_TYPE, 0, 0]);
        self.finish_transaction(result)?;
        Ok(rx[2])
    }
}

/// Supported write command lengths and documented parameter encodings.
/// Sleep, duty-cycle RX and continuous TX need protocols outside this API.
pub(crate) fn validate_command<E>(cmd: u8, p: &[u8]) -> Result<(), SxError<E>> {
    let length = match cmd {
        CMD_SET_FS | CMD_SET_CAD => 0,
        CMD_SET_STANDBY
        | CMD_SET_REGULATOR_MODE
        | CMD_CALIBRATE_FUNCTION
        | CMD_SET_RX_TX_FALLBACK_MODE
        | CMD_SET_DIO2_AS_RF_SWITCH_CTRL
        | CMD_SET_PACKET_TYPE
        | CMD_STOP_TIMER_ON_PREAMBLE => 1,
        CMD_CALIBRATE_IMAGE
        | CMD_SET_TX_PARAMS
        | CMD_SET_BUFFER_BASE_ADDRESS
        | CMD_CLEAR_IRQ_STATUS
        | CMD_CLEAR_DEVICE_ERRORS => 2,
        CMD_SET_RX | CMD_WRITE_REGISTER => 3,
        CMD_SET_DIO3_AS_TCXO_CTRL
        | CMD_SET_RF_FREQUENCY
        | CMD_SET_PA_CONFIG
        | CMD_SET_MODULATION_PARAMS => 4,
        CMD_SET_PACKET_PARAMS => 6,
        CMD_SET_CAD_PARAMS => 7,
        CMD_SET_DIO_IRQ_PARAMS => 8,
        _ => return Err(SxError::UnsupportedCommand),
    };
    if p.len() != length {
        return Err(SxError::InvalidParam);
    }
    let valid = match cmd {
        CMD_SET_STANDBY
        | CMD_SET_REGULATOR_MODE
        | CMD_SET_PACKET_TYPE
        | CMD_SET_DIO2_AS_RF_SWITCH_CTRL
        | CMD_STOP_TIMER_ON_PREAMBLE => p[0] <= 1,
        CMD_CALIBRATE_FUNCTION => p[0] <= 0x7F,
        CMD_SET_RX_TX_FALLBACK_MODE => {
            matches!(p[0], FALLBACK_FS | FALLBACK_STDBY_XOSC | FALLBACK_STDBY_RC)
        }
        CMD_SET_DIO3_AS_TCXO_CTRL => p[0] <= 7,
        CMD_SET_PA_CONFIG => {
            p[0] <= 4 && p[1] <= 7 && p[2] == PA_DEVICE_SEL_SX1262 && p[3] == PA_LUT_DEFAULT
        }
        CMD_SET_TX_PARAMS => (-9..=22).contains(&(p[0] as i8)) && p[1] <= 7,
        CMD_SET_MODULATION_PARAMS => {
            (5..=12).contains(&p[0])
                && matches!(p[1], 0..=6 | 8..=10)
                && (1..=4).contains(&p[2])
                && p[3] <= 1
        }
        CMD_SET_PACKET_PARAMS => p[2] <= 1 && p[4] <= 1 && p[5] <= 1,
        CMD_SET_CAD_PARAMS => p[0] <= 4 && p[3] <= 1,
        CMD_CLEAR_DEVICE_ERRORS => p == [0, 0],
        CMD_SET_RF_FREQUENCY => {
            let value = u32::from_be_bytes([p[0], p[1], p[2], p[3]]);
            (calculate_rf_freq_reg(150_000_000)..=calculate_rf_freq_reg(960_000_000))
                .contains(&value)
        }
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(SxError::InvalidParam)
    }
}
