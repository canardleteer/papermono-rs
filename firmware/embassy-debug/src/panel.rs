//! SSD1677 e-paper controller OTP waveform driver and panel safety manager.
//!
//! # Architecture & Display Safety Contract
//! This module coordinates low-level SPI communication and waveform management
//! for the 800×480 (portrait 480×800) SSD1677 active-matrix electrophoretic display.
//!
//! ### Hardware Safety Mandates
//! 1. **Factory OTP Waveforms Exclusively**: Do not define or upload custom Look-Up
//!    Tables (LUTs via command `0x32`). Custom waveforms without factory calibration
//!    can induce severe DC bias across microcapsules, causing permanent physical degradation.
//! 2. **Periodic Full Waveform Refresh**: Electrophoretic microcapsules accumulate residual
//!    charges during partial updates. To preserve display contrast and prevent burn-in/ghosting,
//!    this driver mandates a full refresh cycle ([`display::OtpRefresh::MonoFull`]) every
//!    [`PARTIALS_BEFORE_FULL`] (18) partial updates.
//! 3. **Deep Sleep Between Updates**: After every update sequence, the panel is placed
//!    into hardware Deep Sleep Mode 1 (`0x10`) to deactivate high-voltage charge pumps.
//! 4. **Hardware BUSY Line Synchronization**: The SSD1677 `BUSY` signal (`GPIO18`) goes high
//!    during internal charge pump ramp-up and gate/source driving. Software must block until
//!    `BUSY` returns low before initiating further transactions.

use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, Ordering};

use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::{Input, Level, Output, OutputConfig};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::Rate;
use esp_hal::Blocking;
use m5stack_papermono_lite::display;
use m5stack_papermono_lite::ioe1;
use m5stack_papermono_lite::pins;
use m5stack_papermono_lite::ssd1677_otp::Ssd1677;
use papermono_log::PanelStamp;

type Epd = Ssd1677<Spi<'static, Blocking>, Output<'static>, Output<'static>>;

/// Target mark primitive drawn directly into display controller RAM.
#[derive(Clone, Copy)]
pub enum Mark {
    /// Solid black circle centered at `(x, y)` with radius `r`.
    Disk { x: u16, y: u16, r: u16 },
    /// Horizontal line crossing the screen at ordinate `y` with thickness `half * 2`.
    HLine { y: u16, half: u16 },
    /// Vertical line crossing the screen at abscissa `x` with thickness `half * 2`.
    VLine { x: u16, half: u16 },
    /// Clears the field to solid white.
    Blank,
}

/// E-paper panel controller handle managing SPI transactions and refresh budgets.
pub struct Panel {
    epd: Epd,
    /// Indicates whether a valid monochromatic baseline has been written to controller RAM.
    mono_ready: bool,
    /// Number of partial refresh cycles executed since the last full refresh.
    partials: u8,
}

use crate::cdc;
use crate::ioe::{self, SysI2c};

const _: () = {
    assert!(pins::EPD_MOSI == 14);
    assert!(pins::EPD_SCLK == 15);
    assert!(pins::EPD_CS == 16);
    assert!(pins::EPD_DC == 17);
    assert!(pins::EPD_BUSY == 18);
    assert!(display::OTP_SPI_HZ <= display::WRITE_FSCL_MAX_HZ);
    assert!(display::OTP_PLANE_BYTES == display::PLANE_BYTES);
};

/// Hardware reset pulse duration in milliseconds.
const RST_MS: u64 = 10;

/// Maximum duration to wait for the BUSY signal to rise after Master Activation.
const BUSY_RISE_MS: u64 = 100;

static HAVE_STAMP: AtomicBool = AtomicBool::new(false);
static STAMP_W: AtomicU16 = AtomicU16::new(0);
static STAMP_H: AtomicU16 = AtomicU16::new(0);
static STAMP_MODE: AtomicU8 = AtomicU8::new(0);
static BUSY_ROSE: AtomicBool = AtomicBool::new(false);

const MODE_TARGET: u8 = 0;
const MODE_GRAY: u8 = 1;
const MODE_MONO: u8 = 2;
const MODE_PARTIAL: u8 = 3;

/// Yield the Embassy executor every N rows during blocking RAM transfers.
const YIELD_EVERY_ROWS: u16 = 16;

/// Cumulative partial updates allowed before mandating a full clearing refresh.
///
/// Uninterrupted partials can damage the panel; vendor guidance is roughly
/// ten. This image uses 18 (3× the prior budget of 6) so B&W card walks flash
/// `MonoFull` less often. Soft same-card redraws still skip this budget.
const PARTIALS_BEFORE_FULL: u8 = 18;

/// Retrieves the most recent panel refresh telemetry stamp for periodic reporting.
pub fn last() -> Option<PanelStamp> {
    if !HAVE_STAMP.load(Ordering::Relaxed) {
        return None;
    }
    Some(PanelStamp {
        mode: mode_title(STAMP_MODE.load(Ordering::Relaxed)),
        w: STAMP_W.load(Ordering::Relaxed),
        h: STAMP_H.load(Ordering::Relaxed),
        busy_rose: BUSY_ROSE.load(Ordering::Relaxed),
    })
}

fn mode_id(title: &'static str) -> u8 {
    if core::ptr::eq(title, display::OtpRefresh::GrayFull.title()) {
        MODE_GRAY
    } else if core::ptr::eq(title, display::OtpRefresh::MonoFull.title()) {
        MODE_MONO
    } else if core::ptr::eq(title, display::OtpRefresh::Partial.title()) {
        MODE_PARTIAL
    } else {
        MODE_TARGET
    }
}

fn mode_title(id: u8) -> &'static str {
    match id {
        MODE_GRAY => display::OtpRefresh::GrayFull.title(),
        MODE_MONO => display::OtpRefresh::MonoFull.title(),
        MODE_PARTIAL => display::OtpRefresh::Partial.title(),
        _ => display::OTP_TARGET_TITLE,
    }
}

fn store(stamp: PanelStamp) {
    STAMP_W.store(stamp.w, Ordering::Relaxed);
    STAMP_H.store(stamp.h, Ordering::Relaxed);
    STAMP_MODE.store(mode_id(stamp.mode), Ordering::Relaxed);
    BUSY_ROSE.store(stamp.busy_rose, Ordering::Relaxed);
    HAVE_STAMP.store(true, Ordering::Relaxed);
    cdc::panel(&stamp);
}

/// Initializes the SSD1677 display controller and powers up the panel subsystem.
pub async fn begin(
    i2c: &mut SysI2c,
    spi2: esp_hal::peripherals::SPI2<'static>,
    mosi: esp_hal::gpio::AnyPin<'static>,
    sclk: esp_hal::gpio::AnyPin<'static>,
    cs: esp_hal::gpio::AnyPin<'static>,
    dc: esp_hal::gpio::AnyPin<'static>,
    busy: &Input<'static>,
) -> Option<Panel> {
    // Enable display VDD power switch via M5IOE1 expander.
    if ioe::set_push_pull_output(i2c, ioe1::EPD_VDD_ENABLE, true).is_err() {
        return None;
    }
    Timer::after(Duration::from_millis(RST_MS)).await;
    // Issue hardware reset pulse on EPD_RST.
    let _ = ioe::set_push_pull_output(i2c, ioe1::EPD_RST, false);
    Timer::after(Duration::from_millis(RST_MS)).await;
    let _ = ioe::set_push_pull_output(i2c, ioe1::EPD_RST, true);
    Timer::after(Duration::from_millis(RST_MS)).await;
    wait_busy_low(busy).await;

    // Configure SPI2 master for e-paper data transfer at 20 MHz.
    let Ok(spi) = Spi::new(
        spi2,
        Config::default().with_frequency(Rate::from_hz(display::OTP_SPI_HZ)),
    ) else {
        return None;
    };
    let spi = spi.with_mosi(mosi).with_sck(sclk);
    let cs = Output::new(cs, Level::High, OutputConfig::default());
    let dc = Output::new(dc, Level::Low, OutputConfig::default());
    let mut epd = Ssd1677::new(spi, dc, cs);

    // Issue software reset command.
    let _ = epd.cmd(display::SW_RESET, &[]);
    Timer::after(Duration::from_millis(RST_MS)).await;
    wait_busy_low(busy).await;

    Some(Panel {
        epd,
        mono_ready: false,
        partials: 0,
    })
}

impl Panel {
    /// Renders a 4-level grayscale frame using factory OTP 4-gray waveforms.
    pub async fn paint_gray(
        &mut self,
        i2c: &mut SysI2c,
        bw: &[u8],
        red: &[u8],
        busy: &Input<'static>,
    ) {
        self.hardware_reset(i2c, busy).await;
        self.init_gray(busy).await;
        write_gray_plane(&mut self.epd, display::WRITE_RAM_BW, bw).await;
        write_gray_plane(&mut self.epd, display::WRITE_RAM_RED, red).await;
        let _ = self.epd.cmd(
            display::DISPLAY_UPDATE_CONTROL_2,
            &[display::UPDATE_SEQ_OTP_4GRAY],
        );
        let _ = self.epd.activate();
        let busy_rose = wait_busy_cycle(busy).await;
        crate::share::BUSY.store(busy.is_high(), core::sync::atomic::Ordering::Relaxed);
        self.deep_sleep().await;
        store(PanelStamp {
            mode: display::OtpRefresh::GrayFull.title(),
            w: display::OTP_RAM_WIDTH,
            h: display::OTP_RAM_HEIGHT,
            busy_rose,
        });
        self.mono_ready = false;
        self.partials = 0;
    }

    /// Renders a monochromatic frame using OTP waveforms.
    ///
    /// - When no mono baseline exists (`mono_ready` is false), always runs
    ///   [`Self::refresh_mono_full`].
    /// - When `soft` is true (same-card status/telemetry redraw: Bluetooth PIN,
    ///   Wi-Fi survey/hotspot counters, Legend battery; or same-card orientation
    ///   remap), prefers [`Self::refresh_partial_official`] even after the
    ///   partial budget so a black-and-white→black-and-white update does not
    ///   flash a full mono wipe.
    ///   The next non-soft paint (card navigation) still takes a mono full refresh once
    ///   [`PARTIALS_BEFORE_FULL`] is reached, preserving the DC-balance contract.
    /// - When `soft` is false (card change), runs a full mono refresh after
    ///   [`PARTIALS_BEFORE_FULL`] partials since the last full.
    pub async fn paint_mono_fast(
        &mut self,
        i2c: &mut SysI2c,
        bw: &[u8],
        red: &[u8],
        busy: &Input<'static>,
        soft: bool,
    ) {
        let budget_exhausted = PARTIALS_BEFORE_FULL > 0 && self.partials >= PARTIALS_BEFORE_FULL;
        if !self.mono_ready || (!soft && budget_exhausted) {
            self.refresh_mono_full(i2c, Some((bw, red)), busy).await;
            return;
        }
        self.refresh_partial_official(i2c, bw, red, busy).await;
    }

    /// Initializes monochromatic RAM to solid white before starting touch calibration.
    pub async fn enter_mono(&mut self, i2c: &mut SysI2c, busy: &Input<'static>) {
        self.refresh_mono_full(i2c, None, busy).await;
    }

    async fn init_gray(&mut self, busy: &Input<'_>) {
        wait_ready(busy).await;
        let _ = self.epd.cmd(display::SW_RESET, &[]);
        Timer::after(Duration::from_millis(RST_MS)).await;
        wait_busy_low(busy).await;
        let _ = self.epd.init_gray();
    }

    async fn init_mono(&mut self, busy: &Input<'_>) {
        wait_ready(busy).await;
        let _ = self.epd.cmd(display::SW_RESET, &[]);
        Timer::after(Duration::from_millis(RST_MS)).await;
        wait_busy_low(busy).await;
        let _ = self.epd.init_mono();
    }

    fn apply_mono_addressing(&mut self) {
        let _ = self.epd.apply_mono_addressing();
    }

    async fn hardware_reset(&mut self, i2c: &mut SysI2c, busy: &Input<'_>) {
        let _ = ioe::set_push_pull_output(i2c, ioe1::EPD_RST, false);
        Timer::after(Duration::from_millis(RST_MS)).await;
        let _ = ioe::set_push_pull_output(i2c, ioe1::EPD_RST, true);
        Timer::after(Duration::from_millis(RST_MS)).await;
        wait_busy_low(busy).await;
    }

    async fn deep_sleep(&mut self) {
        let _ = self.epd.deep_sleep_mode1();
        Timer::after(Duration::from_millis(display::OTP_SLEEP_MS)).await;
    }

    async fn wake_for_partial(&mut self, i2c: &mut SysI2c, busy: &Input<'_>) {
        self.hardware_reset(i2c, busy).await;
        self.apply_mono_addressing();
        let _ = self
            .epd
            .cmd(display::BORDER_WAVEFORM, &[display::BORDER_OTP_PARTIAL]);
    }

    async fn refresh_mono_full(
        &mut self,
        i2c: &mut SysI2c,
        official: Option<(&[u8], &[u8])>,
        busy: &Input<'_>,
    ) {
        self.hardware_reset(i2c, busy).await;
        self.init_mono(busy).await;
        let _ = self.epd.cmd(
            display::DISPLAY_UPDATE_CONTROL_2,
            &[display::UPDATE_SEQ_OTP_MONO_SYNC],
        );
        match official {
            Some((bw, red)) => {
                write_official_mono(&mut self.epd, display::WRITE_RAM_BW, bw, red, true).await;
            }
            None => write_solid(&mut self.epd, display::WRITE_RAM_BW, 0x00),
        }
        let _ = self.epd.activate();
        let _ = wait_busy_cycle(busy).await;

        match official {
            Some((bw, red)) => {
                write_official_mono(&mut self.epd, display::WRITE_RAM_RED, bw, red, false).await;
                write_official_mono(&mut self.epd, display::WRITE_RAM_BW, bw, red, false).await;
            }
            None => {
                write_white(&mut self.epd, display::WRITE_RAM_RED);
                write_white(&mut self.epd, display::WRITE_RAM_BW);
            }
        }
        let _ = self
            .epd
            .cmd(display::BORDER_WAVEFORM, &[display::BORDER_OTP_FULL]);
        let _ = self.epd.cmd(
            display::DISPLAY_UPDATE_CONTROL_2,
            &[display::UPDATE_SEQ_OTP_MONO],
        );
        let _ = self.epd.activate();
        let busy_rose = wait_busy_cycle(busy).await;
        self.deep_sleep().await;
        store(PanelStamp {
            mode: display::OtpRefresh::MonoFull.title(),
            w: display::OTP_RAM_WIDTH,
            h: display::OTP_RAM_HEIGHT,
            busy_rose,
        });
        self.mono_ready = true;
        self.partials = 0;
    }

    async fn refresh_partial_official(
        &mut self,
        i2c: &mut SysI2c,
        bw: &[u8],
        red: &[u8],
        busy: &Input<'_>,
    ) {
        self.wake_for_partial(i2c, busy).await;
        write_official_mono(&mut self.epd, display::WRITE_RAM_BW, bw, red, false).await;
        let _ = self.epd.cmd(
            display::DISPLAY_UPDATE_CONTROL_1,
            &[display::DISPLAY_CTRL1_NORMAL],
        );
        let _ = self.epd.cmd(
            display::DISPLAY_UPDATE_CONTROL_2,
            &[display::UPDATE_SEQ_OTP_PARTIAL],
        );
        let _ = self.epd.activate();
        let busy_rose = wait_busy_cycle(busy).await;
        write_official_mono(&mut self.epd, display::WRITE_RAM_RED, bw, red, false).await;
        self.deep_sleep().await;
        self.note_partial();
        store(PanelStamp {
            mode: display::OtpRefresh::Partial.title(),
            w: display::OTP_RAM_WIDTH,
            h: display::OTP_RAM_HEIGHT,
            busy_rose,
        });
    }

    /// Draws a target mark in monochromatic RAM and triggers an OTP partial refresh.
    pub async fn paint(&mut self, i2c: &mut SysI2c, mark: Mark, busy: &Input<'static>) {
        if matches!(mark, Mark::Blank) {
            self.refresh_mono_full(i2c, None, busy).await;
            return;
        }
        if !self.mono_ready {
            self.refresh_mono_full(i2c, None, busy).await;
        }
        self.wake_for_partial(i2c, busy).await;
        write_mark(&mut self.epd, display::WRITE_RAM_BW, mark);
        let _ = self.epd.cmd(
            display::DISPLAY_UPDATE_CONTROL_1,
            &[display::DISPLAY_CTRL1_NORMAL],
        );
        let _ = self.epd.cmd(
            display::DISPLAY_UPDATE_CONTROL_2,
            &[display::UPDATE_SEQ_OTP_PARTIAL],
        );
        let _ = self.epd.activate();
        let busy_rose = wait_busy_cycle(busy).await;
        write_mark(&mut self.epd, display::WRITE_RAM_RED, mark);
        self.deep_sleep().await;
        store(PanelStamp {
            mode: display::OtpRefresh::Partial.title(),
            w: display::OTP_RAM_WIDTH,
            h: display::OTP_RAM_HEIGHT,
            busy_rose,
        });
        self.note_partial();
    }

    fn note_partial(&mut self) {
        // Count partials toward the DC-balance full-refresh budget. Clearing
        // `mono_ready` is deferred to [`Self::paint_mono_fast`] for non-soft
        // paints so live Bluetooth / Wi-Fi / Legend status redraws can stay on
        // OTP Partial instead of flashing MonoFull mid-card.
        self.partials = self.partials.saturating_add(1);
    }
}

async fn write_gray_plane(epd: &mut Epd, ram_cmd: u8, official: &[u8]) {
    let _ = epd.rewind_gray();
    let _ = epd.begin_ram(ram_cmd);
    let mut row = [0u8; display::OTP_BYTES_PER_ROW];
    for ram_y in 0..display::OTP_RAM_HEIGHT {
        for (byte_i, slot) in row.iter_mut().enumerate() {
            let mut v = 0u8;
            for bit in 0..8u16 {
                let ram_x = display::OTP_RAM_WIDTH
                    .saturating_sub(1)
                    .saturating_sub((byte_i as u16) * 8 + bit);
                let (px, py) = display::otp_ram_to_usb_down(ram_x, ram_y);
                if official_bit(official, px, py) {
                    v |= 0x80 >> bit;
                }
            }
            *slot = v;
        }
        let _ = epd.write_bytes(&row);
        if ram_y.is_multiple_of(YIELD_EVERY_ROWS) {
            Timer::after(Duration::from_millis(1)).await;
        }
    }
    let _ = epd.end_ram();
}

async fn write_official_mono(epd: &mut Epd, ram_cmd: u8, bw: &[u8], red: &[u8], invert: bool) {
    let _ = epd.rewind();
    let _ = epd.begin_ram(ram_cmd);
    let mut row = [0u8; display::OTP_BYTES_PER_ROW];
    for ram_y in 0..display::OTP_RAM_HEIGHT {
        row.fill(0xFF);
        for ram_x in 0..display::OTP_RAM_WIDTH {
            let (px, py) = display::otp_ram_to_usb_down(ram_x, ram_y);
            if official_bit(bw, px, py) || official_bit(red, px, py) {
                ink_black(&mut row, ram_x);
            }
        }
        if invert {
            for b in &mut row {
                *b = !*b;
            }
        }
        let _ = epd.write_bytes(&row);
        if ram_y.is_multiple_of(YIELD_EVERY_ROWS) {
            Timer::after(Duration::from_millis(1)).await;
        }
    }
    let _ = epd.end_ram();
}

fn official_bit(plane: &[u8], x: u16, y: u16) -> bool {
    if x >= display::WIDTH || y >= display::HEIGHT {
        return false;
    }
    let i = usize::from(y) * display::BYTES_PER_ROW + usize::from(x) / 8;
    let mask = 0x80u8 >> (x % 8);
    plane.get(i).is_some_and(|b| b & mask != 0)
}

fn write_white(epd: &mut Epd, ram_cmd: u8) {
    write_solid(epd, ram_cmd, 0xFF);
}

fn write_solid(epd: &mut Epd, ram_cmd: u8, byte: u8) {
    let _ = epd.rewind();
    let _ = epd.begin_ram(ram_cmd);
    let row = [byte; display::OTP_BYTES_PER_ROW];
    for _ in 0..display::OTP_RAM_HEIGHT {
        let _ = epd.write_bytes(&row);
    }
    let _ = epd.end_ram();
}

fn write_mark(epd: &mut Epd, ram_cmd: u8, mark: Mark) {
    let _ = epd.rewind();
    let _ = epd.begin_ram(ram_cmd);
    let mut row = [0u8; display::OTP_BYTES_PER_ROW];
    for ram_y in 0..display::OTP_RAM_HEIGHT {
        fill_mark_row(&mut row, ram_y, mark);
        let _ = epd.write_bytes(&row);
    }
    let _ = epd.end_ram();
}

fn fill_mark_row(row: &mut [u8], ram_y: u16, mark: Mark) {
    row.fill(0xFF);
    for ram_x in 0..display::OTP_RAM_WIDTH {
        let (px, py) = display::otp_ram_to_usb_down(ram_x, ram_y);
        if mark_hit(mark, px, py) {
            ink_black(row, ram_x);
        }
    }
}

fn mark_hit(mark: Mark, px: u16, py: u16) -> bool {
    match mark {
        Mark::Disk { x, y, r } => {
            let dx = i32::from(px) - i32::from(x);
            let dy = i32::from(py) - i32::from(y);
            dx * dx + dy * dy <= i32::from(r) * i32::from(r)
        }
        Mark::HLine { y, half } => py.abs_diff(y) <= half,
        Mark::VLine { x, half } => px.abs_diff(x) <= half,
        Mark::Blank => false,
    }
}

fn ink_black(row: &mut [u8], ram_x: u16) {
    if ram_x >= display::OTP_RAM_WIDTH {
        return;
    }
    let byte = (ram_x / 8) as usize;
    let bit = (7 - (ram_x % 8)) as u8;
    if byte < row.len() {
        row[byte] &= !((1u8) << bit);
    }
}

async fn wait_ready(busy: &Input<'_>) {
    if busy.is_high() {
        wait_busy_low(busy).await;
    }
}

async fn wait_busy_low(busy: &Input<'_>) {
    Timer::after(Duration::from_millis(1)).await;
    let deadline = Instant::now() + Duration::from_millis(display::OTP_BUSY_TIMEOUT_MS);
    while busy.is_high() && Instant::now() < deadline {
        crate::share::BUSY.store(true, core::sync::atomic::Ordering::Relaxed);
        Timer::after(Duration::from_millis(1)).await;
    }
    crate::share::BUSY.store(busy.is_high(), core::sync::atomic::Ordering::Relaxed);
}

async fn wait_busy_cycle(busy: &Input<'_>) -> bool {
    Timer::after(Duration::from_millis(1)).await;
    let rise_deadline = Instant::now() + Duration::from_millis(BUSY_RISE_MS);
    let mut rose = busy.is_high();
    while !rose && Instant::now() < rise_deadline {
        Timer::after(Duration::from_millis(1)).await;
        rose = busy.is_high();
    }
    wait_busy_low(busy).await;
    rose
}
