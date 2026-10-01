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

impl<SPI, BUSY, DELAY, H> Sx1262<SPI, BUSY, DELAY, H>
where
    SPI: embedded_hal_async::spi::SpiDevice,
    BUSY: embedded_hal::digital::InputPin,
    DELAY: embedded_hal_async::delay::DelayNs,
{
    /// Configures upstream modulation and its bandwidth workaround. LDRO is
    /// computed by lora-modulation for every supported symbol duration, then
    /// explicitly propagated into upstream's command parameters.
    pub async fn configure_lora_modulation(
        &mut self,
        params: &BaseBandModulationParams,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        let calculated = BaseBandModulationParams::new(params.sf, params.bw, params.cr);
        if params.ldro != calculated.ldro {
            return Err(SxError::InvalidParam);
        }
        let frequency = self.frequency_hz.unwrap_or(960_000_000);
        let mut fault = None;
        let saved = self.begin_io();
        let result = {
            let mut radio = self.backend(&mut fault);
            match radio.create_modulation_params(params.sf, params.bw, params.cr, frequency) {
                Ok(mut modulation) => {
                    modulation.low_data_rate_optimize = u8::from(calculated.ldro);
                    radio.set_modulation_params(&modulation).await
                }
                Err(error) => Err(error),
            }
        };
        self.end_io(saved, backend::finish(result, fault))?;
        self.modulation = Some(calculated);
        Ok(())
    }
    /// Configures packet fields and upstream IQ workaround. SF5/SF6 preambles
    /// below twelve symbols are raised to twelve, matching the airtime helper.
    pub async fn configure_lora_packet(
        &mut self,
        params: &LoRaPacketParams,
    ) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        let modulation = self.modulation.ok_or(SxError::InvalidParam)?;
        let frequency = self.frequency_hz.unwrap_or(960_000_000);
        let mut fault = None;
        let saved = self.begin_io();
        let result = {
            let mut radio = self.backend(&mut fault);
            match radio
                .create_modulation_params(modulation.sf, modulation.bw, modulation.cr, frequency)
                .and_then(|m| {
                    radio.create_packet_params(
                        params.preamble_symbols,
                        params.implicit_header,
                        params.payload_len,
                        params.crc,
                        params.invert_iq,
                        &m,
                    )
                }) {
                Ok(packet) => radio.set_packet_params(&packet).await,
                Err(error) => Err(error),
            }
        };
        self.end_io(saved, backend::finish(result, fault))?;
        self.packet = Some(*params);
        Ok(())
    }
    /// Applies the TX clamp threshold after reset, without altering PA or OCP.
    /// Catalog `sx1262` §15.2.2 “Workaround”; preserves unrelated register bits.
    pub async fn configure_tx_clamp(&mut self) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        let value = self.read_reg(REG_TX_CLAMP_CONFIG).await?;
        self.write_reg(REG_TX_CLAMP_CONFIG, value | TX_CLAMP_MASK)
            .await
    }
    /// Calibrates selected blocks in standby RC; `sx1262` §13.1.12 “Calibrate Function”.
    pub async fn calibrate(&mut self, mask: u8) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(CMD_CALIBRATE_FUNCTION, &[mask]).await
    }
    /// Reads original device error bits; `sx1262` §13.6.1 “GetDeviceErrors”.
    pub async fn get_device_errors(&mut self) -> Result<u16, SxError<SPI::Error, BUSY::Error>> {
        let rx = self.read_response::<4>(CMD_GET_DEVICE_ERRORS).await?;
        Ok(u16::from_be_bytes([rx[2], rx[3]]))
    }
    /// Clears device errors; `sx1262` §13.6.2 “ClearDeviceErrors”.
    pub async fn clear_device_errors(&mut self) -> Result<(), SxError<SPI::Error, BUSY::Error>> {
        self.write_cmd(CMD_CLEAR_DEVICE_ERRORS, &[0, 0]).await
    }
    /// Queries active modem; `sx1262` §13.4.3 “GetPacketType”.
    pub async fn get_packet_type(&mut self) -> Result<u8, SxError<SPI::Error, BUSY::Error>> {
        Ok(self.read_response::<3>(CMD_GET_PACKET_TYPE).await?[2])
    }
    /// Reads instantaneous RSSI multiplied by two, preserving half-dBm values.
    /// Catalog `sx1262` §13.5.4 “GetRssiInst”.
    pub async fn get_rssi_inst_half_dbm(
        &mut self,
    ) -> Result<i16, SxError<SPI::Error, BUSY::Error>> {
        Ok(-i16::from(
            self.read_response::<3>(CMD_GET_RSSI_INST).await?[2],
        ))
    }
    /// Last explicitly calibrated band; reset/startup discards this cache.
    pub const fn calibration_band(&self) -> Option<CalibrationBand> {
        self.calibration_band
    }
}

/// Documented image calibration bands from `sx1262` §13.1.13 “CalibrateImage”,
/// §9.2.1 “Image Calibration for Specific Frequency Bands”, Table 9-2.
/// Frequencies outside these ranges need caller-selected codes;
/// the driver never guesses a calibration range for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationBand {
    /// 430–440 MHz calibration.
    Mhz430_440,
    /// 470–510 MHz calibration.
    Mhz470_510,
    /// 779–787 MHz calibration.
    Mhz779_787,
    /// 863–870 MHz calibration.
    Mhz863_870,
    /// 902–928 MHz calibration.
    Mhz902_928,
}
impl CalibrationBand {
    /// Returns a documented band containing the requested frequency.
    pub const fn for_frequency(hz: u32) -> Option<Self> {
        match hz {
            430_000_000..=440_000_000 => Some(Self::Mhz430_440),
            470_000_000..=510_000_000 => Some(Self::Mhz470_510),
            779_000_000..=787_000_000 => Some(Self::Mhz779_787),
            863_000_000..=870_000_000 => Some(Self::Mhz863_870),
            902_000_000..=928_000_000 => Some(Self::Mhz902_928),
            _ => None,
        }
    }
    /// Documented frequency endpoint encodings from Table 9-2.
    pub const fn codes(self) -> (u8, u8) {
        match self {
            Self::Mhz430_440 => (0x6B, 0x6F),
            Self::Mhz470_510 => (0x75, 0x81),
            Self::Mhz779_787 => (0xC1, 0xC5),
            Self::Mhz863_870 => (0xD7, 0xDB),
            Self::Mhz902_928 => (0xE1, 0xE9),
        }
    }
    /// Recognizes raw calibration codes without asserting an undocumented band.
    pub const fn from_codes(a: u8, b: u8) -> Option<Self> {
        match (a, b) {
            (0x6B, 0x6F) => Some(Self::Mhz430_440),
            (0x75, 0x81) => Some(Self::Mhz470_510),
            (0xC1, 0xC5) => Some(Self::Mhz779_787),
            (0xD7, 0xDB) => Some(Self::Mhz863_870),
            (0xE1, 0xE9) => Some(Self::Mhz902_928),
            _ => None,
        }
    }
}

/// Supported write command lengths and documented parameter encodings.
/// Sleep, duty-cycle RX and continuous TX need protocols outside this API.
pub(crate) fn validate_command<E, B>(cmd: u8, p: &[u8]) -> Result<(), SxError<E, B>> {
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

/// LoRa airtime in microseconds, rounded up, with CRC selection and a 16-bit
/// preamble. Uses the configured SF5/SF6 minimum of twelve preamble symbols.
/// Arithmetic is widened to u64 so long preambles at narrow bandwidths fit.
///
/// The symbol count follows Semtech's reference function
/// [`sx126x_get_lora_time_on_air_numerator`](https://github.com/Lora-net/sx126x_driver/blob/a10c5dfdf89788c6ac805e9fe98889de44175aa2/src/sx126x.c#L1083).
/// It treats SF5/SF6 separately and includes the header and optional CRC bits.
/// The caller should use modulation produced by BaseBandModulationParams::new.
/// Reference algorithm attribution and terms are in LICENSE-Semtech.
pub fn lora_airtime_us(modulation: &BaseBandModulationParams, packet: &LoRaPacketParams) -> u64 {
    let sf = modulation.sf.factor();
    let short_sf = sf <= 6;
    let preamble = if short_sf {
        packet.preamble_symbols.max(12)
    } else {
        packet.preamble_symbols
    };
    let bits = i32::from(packet.payload_len) * 8
        + if packet.crc { 16 } else { 0 }
        + if packet.implicit_header { 0 } else { 20 }
        - 4 * sf as i32
        + if short_sf { 0 } else { 8 };
    let divisor = 4 * if !short_sf && modulation.ldro {
        sf - 2
    } else {
        sf
    };
    let coded_symbols = (bits.max(0) as u32).div_ceil(divisor) * modulation.cr.denom();
    let whole_symbols =
        u64::from(coded_symbols) + u64::from(preamble) + 12 + if short_sf { 2 } else { 0 };
    // Quarter-symbol term avoids premature symbol-duration truncation.
    ((4 * whole_symbols + 1) * (1_u64 << (sf - 2)) * 1_000_000)
        .div_ceil(u64::from(modulation.bw.hz()))
}
