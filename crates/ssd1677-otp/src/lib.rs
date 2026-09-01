//! SSD1677 **panel OTP** sequences (PaperMono OTP-Demo).
//!
//! Catalog id `ssd1677`, Rev 1.0 Table 7-1 Command Table and
//! section 8 windowing. No MCU look-up table (`0x32`). Do not
//! map official HTML `epd_*` titles onto [`OtpRefresh`] `0x22`
//! bytes. Not sticky-rs `ssd1677-gray4`.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(test)]
extern crate std;

use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;

/// OTP-Demo controller RAM X extent (`DISPLAY_WIDTH`).
pub const OTP_RAM_WIDTH: u16 = 800;
/// OTP-Demo controller RAM Y extent (`DISPLAY_HEIGHT`).
pub const OTP_RAM_HEIGHT: u16 = 480;
/// Bytes per row at [`OTP_RAM_WIDTH`].
pub const OTP_BYTES_PER_ROW: usize = (OTP_RAM_WIDTH as usize) / 8;
/// Bytes in one OTP RAM plane (`800×480/8`).
pub const OTP_PLANE_BYTES: usize = OTP_BYTES_PER_ROW * (OTP_RAM_HEIGHT as usize);

/// SSD1677 Rev 1.0 write `fSCL` maximum (sheet cap).
pub const WRITE_FSCL_MAX_HZ: u32 = 20_000_000;
/// OTP-Demo `edp_spi` clock.
pub const OTP_SPI_HZ: u32 = 20_000_000;
/// OTP-Demo `wait_ready` default timeout (ms).
pub const OTP_BUSY_TIMEOUT_MS: u64 = 15_000;
/// OTP-Demo delay after [`DEEP_SLEEP`] (ms).
pub const OTP_SLEEP_MS: u64 = 100;

/// Catalog id `ssd1677`, Table 7-1, Software Reset.
pub const SW_RESET: u8 = 0x12;
/// Catalog id `ssd1677`, section 8.2 Data Entry Mode Setting (11h).
pub const DATA_ENTRY_MODE: u8 = 0x11;
/// POR A[1:0]=11: X increment and Y increment (Table 8-3 / 8-4).
pub const DATA_ENTRY_XY_INC: u8 = 0x03;
/// OTP-Demo 4-gray: X decrement, Y increment (`init_gray_mode`).
pub const DATA_ENTRY_XDEC_YINC: u8 = 0x02;
/// Catalog id `ssd1677`, Table 7-1, Booster Soft Start (0Ch).
pub const BOOSTER_SOFT_START: u8 = 0x0C;
/// OTP-Demo `init_gray_mode` / `init_mono_mode` booster payload.
pub const BOOSTER_SOFT_START_OTP: [u8; 5] = [0xAE, 0xC7, 0xC3, 0xC0, 0x80];
/// Catalog id `ssd1677`, Table 7-1, Driver Output Control (01h).
pub const DRIVER_OUTPUT: u8 = 0x01;
/// OTP-Demo: 480 gate outputs (`0x01DF`) plus GD/SM/TB byte `0x02`.
pub const DRIVER_OUTPUT_480_GATES: [u8; 3] = [0xDF, 0x01, 0x02];
/// Catalog id `ssd1677`, Table 7-1, Border Waveform Control (3Ch).
pub const BORDER_WAVEFORM: u8 = 0x3C;
/// OTP-Demo full-refresh border.
pub const BORDER_OTP_FULL: u8 = 0x01;
/// OTP-Demo `wake_for_partial_update` border (`0x80`, float).
pub const BORDER_OTP_PARTIAL: u8 = 0x80;
/// Catalog id `ssd1677`, Table 7-1, Temperature Sensor Control (18h).
pub const TEMP_SENSOR: u8 = 0x18;
/// OTP-Demo internal temperature sensor.
pub const TEMP_SENSOR_INTERNAL: u8 = 0x80;
/// Catalog id `ssd1677`, Table 7-1, Temperature Sensor Control (1Ah).
pub const TEMP_VALUE: u8 = 0x1A;
/// OTP-Demo 4-gray temperature value.
pub const TEMP_VALUE_GRAY_OTP: u8 = 0x5A;
/// Catalog id `ssd1677`, section 8.3 Set RAM X window (44h).
pub const RAM_X_WINDOW: u8 = 0x44;
/// Catalog id `ssd1677`, section 8.4 Set RAM Y window (45h).
pub const RAM_Y_WINDOW: u8 = 0x45;
/// Catalog id `ssd1677`, section 8.5 Set RAM Address Counter (4Eh).
pub const RAM_X_COUNTER: u8 = 0x4E;
/// Catalog id `ssd1677`, section 8.5 Set RAM Address Counter (4Fh).
pub const RAM_Y_COUNTER: u8 = 0x4F;
/// Catalog id `ssd1677`, Table 7-1, Write RAM (Black White) / RAM 0x24.
pub const WRITE_RAM_BW: u8 = 0x24;
/// Catalog id `ssd1677`, Table 7-1, Write RAM (RED) / RAM 0x26.
pub const WRITE_RAM_RED: u8 = 0x26;
/// Catalog id `ssd1677`, Table 7-1, Display Update Control 2.
pub const DISPLAY_UPDATE_CONTROL_2: u8 = 0x22;
/// Datasheet Mode 1 parameter `C7`. **Not** an OTP-Demo sequence.
pub const UPDATE_SEQ_OTP_MODE1: u8 = 0xC7;
/// OTP-Demo `refresh_gray_full`.
pub const UPDATE_SEQ_OTP_4GRAY: u8 = 0xD7;
/// Catalog id `ssd1677`, Table 7-1, Display Update Control 1 (21h).
pub const DISPLAY_UPDATE_CONTROL_1: u8 = 0x21;
/// OTP-Demo `init_mono_mode` / `refresh_partial` RAM display mode.
pub const DISPLAY_CTRL1_NORMAL: u8 = 0x00;
/// OTP-Demo `refresh_partial`.
pub const UPDATE_SEQ_OTP_PARTIAL: u8 = 0xFF;
/// OTP-Demo `refresh_mono_full` invert sync (`0xF8`).
pub const UPDATE_SEQ_OTP_MONO_SYNC: u8 = 0xF8;
/// OTP-Demo `refresh_mono_full` Mode 1 (`0x14`). Both planes.
pub const UPDATE_SEQ_OTP_MONO: u8 = 0x14;
/// Catalog id `ssd1677`, Table 7-1, Master Activation.
pub const MASTER_ACTIVATION: u8 = 0x20;
/// Catalog id `ssd1677`, Table 7-1, Deep Sleep Mode (`10h`).
pub const DEEP_SLEEP: u8 = 0x10;
/// OTP-Demo Deep Sleep Mode 1. Leave with a hardware reset.
pub const DEEP_SLEEP_MODE1: u8 = 0x01;

/// CDC `panel mode=` for [`OtpRefresh::GrayFull`].
pub const OTP_GRAY_TITLE: &str = "otp_gray";
/// CDC `panel mode=` for [`OtpRefresh::MonoFull`].
pub const OTP_MONO_TITLE: &str = "otp_mono";
/// CDC `panel mode=` for [`OtpRefresh::Partial`].
pub const OTP_PARTIAL_TITLE: &str = "otp_partial";

/// OTP-Demo `make_gray_quadrants` tone: black.
pub const GRAY_BLACK: u8 = 0;
/// OTP-Demo `make_gray_quadrants` tone: dark gray.
pub const GRAY_DARK: u8 = 1;
/// OTP-Demo `make_gray_quadrants` tone: light gray.
pub const GRAY_LIGHT: u8 = 2;
/// OTP-Demo `make_gray_quadrants` tone: white.
pub const GRAY_WHITE: u8 = 3;

/// OTP-Demo refresh this firmware should call.
///
/// Built-in panel waveforms. Source: M5PaperMono-OTP-Demo
/// `EDP_OTP_LUT_demo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtpRefresh {
    /// 4-gray full. `0xD7`. Both planes. Invalidates the mono baseline.
    GrayFull,
    /// Mono full. Invert-sync `0xF8`, then Mode 1 `0x14` on both planes.
    MonoFull,
    /// Partial. `0xFF`. RAM 1 only. Requires a mono baseline.
    Partial,
}

impl OtpRefresh {
    /// CDC `panel mode=` title for this sequence.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::GrayFull => OTP_GRAY_TITLE,
            Self::MonoFull => OTP_MONO_TITLE,
            Self::Partial => OTP_PARTIAL_TITLE,
        }
    }

    /// `0x22` payload. [`Self::MonoFull`] is two activations.
    #[must_use]
    pub const fn update_seq(self) -> &'static [u8] {
        match self {
            Self::GrayFull => &[UPDATE_SEQ_OTP_4GRAY],
            Self::MonoFull => &[UPDATE_SEQ_OTP_MONO_SYNC, UPDATE_SEQ_OTP_MONO],
            Self::Partial => &[UPDATE_SEQ_OTP_PARTIAL],
        }
    }

    /// `0xFF` is legal only after a mono RAM baseline.
    #[must_use]
    pub const fn requires_mono_baseline(self) -> bool {
        matches!(self, Self::Partial)
    }

    /// `0xD7` drops the mono baseline.
    #[must_use]
    pub const fn invalidates_mono_baseline(self) -> bool {
        matches!(self, Self::GrayFull)
    }

    /// `0xF8` then `0x14` restores the mono baseline.
    #[must_use]
    pub const fn rebuilds_mono_baseline(self) -> bool {
        matches!(self, Self::MonoFull)
    }
}

/// OTP-Demo four-gray bits: `(plane1 0x24, plane2 0x26)`.
#[must_use]
pub const fn gray_planes(tone: u8) -> (bool, bool) {
    match tone & 0b11 {
        GRAY_WHITE => (false, false),
        GRAY_LIGHT => (true, false),
        GRAY_DARK => (false, true),
        _ => (true, true),
    }
}

/// Split a 10-bit RAM address into the two data bytes Table 8-5 expects.
#[must_use]
pub const fn ram_addr10(value: u16) -> [u8; 2] {
    [value as u8, (value >> 8) as u8]
}

/// SPI + DC + CS for OTP command traffic.
pub struct Ssd1677<SPI, DC, CS> {
    spi: SPI,
    dc: DC,
    cs: CS,
}

/// Pin or SPI error from [`Ssd1677`].
#[derive(Debug)]
pub enum Error<S, D, C> {
    /// `SpiBus` write failed.
    Spi(S),
    /// DC pin failed.
    Dc(D),
    /// CS pin failed.
    Cs(C),
}

impl<SPI, DC, CS> Ssd1677<SPI, DC, CS> {
    /// Take the bus and pins. CS should already be high.
    #[inline]
    pub const fn new(spi: SPI, dc: DC, cs: CS) -> Self {
        Self { spi, dc, cs }
    }

    /// Return the bus and pins (C-FREE).
    #[inline]
    pub fn release(self) -> (SPI, DC, CS) {
        (self.spi, self.dc, self.cs)
    }
}

#[allow(clippy::type_complexity)]
impl<SPI, DC, CS> Ssd1677<SPI, DC, CS>
where
    SPI: SpiBus,
    DC: OutputPin,
    CS: OutputPin,
{
    /// Command byte, optional data. Catalog Table 7-1.
    pub fn cmd(
        &mut self,
        opcode: u8,
        data: &[u8],
    ) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cs.set_low().map_err(Error::Cs)?;
        self.dc.set_low().map_err(Error::Dc)?;
        self.spi.write(&[opcode]).map_err(Error::Spi)?;
        if !data.is_empty() {
            self.dc.set_high().map_err(Error::Dc)?;
            self.spi.write(data).map_err(Error::Spi)?;
        }
        self.cs.set_high().map_err(Error::Cs)
    }

    /// Hold CS, send a RAM write opcode, leave DC high for payload.
    pub fn begin_ram(
        &mut self,
        ram_cmd: u8,
    ) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cs.set_low().map_err(Error::Cs)?;
        self.dc.set_low().map_err(Error::Dc)?;
        self.spi.write(&[ram_cmd]).map_err(Error::Spi)?;
        self.dc.set_high().map_err(Error::Dc)
    }

    /// Write bytes while a RAM transfer is open.
    pub fn write_bytes(
        &mut self,
        data: &[u8],
    ) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.spi.write(data).map_err(Error::Spi)
    }

    /// Release CS after [`Self::begin_ram`].
    pub fn end_ram(&mut self) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cs.set_high().map_err(Error::Cs)
    }

    /// Table 7-1 Master Activation.
    pub fn activate(&mut self) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cmd(MASTER_ACTIVATION, &[])
    }

    /// OTP-Demo `init_gray_mode` commands (no reset wait).
    pub fn init_gray(&mut self) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cmd(BOOSTER_SOFT_START, &BOOSTER_SOFT_START_OTP)?;
        self.cmd(DRIVER_OUTPUT, &DRIVER_OUTPUT_480_GATES)?;
        self.cmd(DATA_ENTRY_MODE, &[DATA_ENTRY_XDEC_YINC])?;
        let x0 = ram_addr10(OTP_RAM_WIDTH.saturating_sub(1));
        let y_end = ram_addr10(OTP_RAM_HEIGHT.saturating_sub(1));
        self.cmd(RAM_X_WINDOW, &[x0[0], x0[1], 0, 0])?;
        self.cmd(RAM_Y_WINDOW, &[0, 0, y_end[0], y_end[1]])?;
        self.cmd(RAM_X_COUNTER, &x0)?;
        self.cmd(RAM_Y_COUNTER, &[0, 0])?;
        self.cmd(BORDER_WAVEFORM, &[BORDER_OTP_FULL])?;
        self.cmd(TEMP_SENSOR, &[TEMP_SENSOR_INTERNAL])?;
        self.cmd(TEMP_VALUE, &[TEMP_VALUE_GRAY_OTP])
    }

    /// Mono RAM window and X/Y increment.
    pub fn apply_mono_addressing(&mut self) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cmd(DATA_ENTRY_MODE, &[DATA_ENTRY_XY_INC])?;
        let x_end = ram_addr10(OTP_RAM_WIDTH.saturating_sub(1));
        let y_end = ram_addr10(OTP_RAM_HEIGHT.saturating_sub(1));
        self.cmd(RAM_X_WINDOW, &[0, 0, x_end[0], x_end[1]])?;
        self.cmd(RAM_Y_WINDOW, &[0, 0, y_end[0], y_end[1]])?;
        self.cmd(DISPLAY_UPDATE_CONTROL_1, &[DISPLAY_CTRL1_NORMAL])
    }

    /// OTP-Demo `init_mono_mode` after a software reset.
    pub fn init_mono(&mut self) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cmd(BOOSTER_SOFT_START, &BOOSTER_SOFT_START_OTP)?;
        self.cmd(DRIVER_OUTPUT, &DRIVER_OUTPUT_480_GATES)?;
        self.apply_mono_addressing()?;
        self.cmd(BORDER_WAVEFORM, &[BORDER_OTP_FULL])?;
        self.cmd(TEMP_SENSOR, &[TEMP_SENSOR_INTERNAL])
    }

    /// Deep Sleep Mode 1 (keep RAM).
    pub fn deep_sleep_mode1(&mut self) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cmd(DEEP_SLEEP, &[DEEP_SLEEP_MODE1])
    }

    /// RAM counters at origin (XY increment).
    pub fn rewind(&mut self) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        self.cmd(RAM_X_COUNTER, &[0, 0])?;
        self.cmd(RAM_Y_COUNTER, &[0, 0])
    }

    /// RAM counters for X-decrement gray writes.
    pub fn rewind_gray(&mut self) -> Result<(), Error<SPI::Error, DC::Error, CS::Error>> {
        let x0 = ram_addr10(OTP_RAM_WIDTH.saturating_sub(1));
        self.cmd(RAM_X_COUNTER, &x0)?;
        self.cmd(RAM_Y_COUNTER, &[0, 0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::digital::{Mock as PinMock, State, Transaction as PinTxn};
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTxn};

    #[test]
    fn otp_demo_ram_is_landscape_same_plane() {
        assert_eq!(OTP_RAM_WIDTH, 800);
        assert_eq!(OTP_RAM_HEIGHT, 480);
        assert_eq!(OTP_BYTES_PER_ROW, 100);
        assert_eq!(OTP_PLANE_BYTES, 48_000);
        assert_eq!(UPDATE_SEQ_OTP_4GRAY, 0xD7);
        assert_ne!(UPDATE_SEQ_OTP_4GRAY, UPDATE_SEQ_OTP_MODE1);
        assert_eq!(OtpRefresh::GrayFull.title(), OTP_GRAY_TITLE);
        assert_eq!(OtpRefresh::GrayFull.update_seq(), &[UPDATE_SEQ_OTP_4GRAY]);
        assert!(OtpRefresh::GrayFull.invalidates_mono_baseline());
        assert!(OtpRefresh::Partial.requires_mono_baseline());
        assert!(OtpRefresh::MonoFull.rebuilds_mono_baseline());
        assert_eq!(ram_addr10(799), [0x1F, 0x03]);
        assert_eq!(gray_planes(GRAY_WHITE), (false, false));
        assert_eq!(gray_planes(GRAY_BLACK), (true, true));
        const { assert!(OTP_RAM_WIDTH % 8 == 0) };
        const { assert!(OTP_SPI_HZ <= WRITE_FSCL_MAX_HZ) };
    }

    #[test]
    fn cmd_drives_dc_and_cs() {
        let spi = SpiMock::new(&[SpiTxn::write(SW_RESET)]);
        let dc = PinMock::new(&[PinTxn::set(State::Low)]);
        let cs = PinMock::new(&[PinTxn::set(State::Low), PinTxn::set(State::High)]);
        let mut epd = Ssd1677::new(spi, dc, cs);
        epd.cmd(SW_RESET, &[]).unwrap();
        let (mut spi, mut dc, mut cs) = epd.release();
        spi.done();
        dc.done();
        cs.done();
    }
}
