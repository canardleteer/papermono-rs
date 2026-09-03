//! OTP-Demo [`display::OtpRefresh`] only. No `0x32` LUT.
//! Do not stamp [`display::RefreshMode`] titles.

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

/// USB-C-down mark in OTP RAM (white field, black ink).
#[derive(Clone, Copy)]
pub enum Mark {
    /// Disk at official-portrait `(x, y)`.
    Disk { x: u16, y: u16, r: u16 },
    /// Horizontal line through `y` (full X slide).
    HLine { y: u16, half: u16 },
    /// Vertical line through `x` (full Y slide).
    VLine { x: u16, half: u16 },
    /// White field (walk finished).
    Blank,
}

/// SPI + DC/CS after OTP init. Rails stay up.
pub struct Panel {
    epd: Epd,
    /// Mono RAM baseline is valid for `0xFF` (not after `0xD7`).
    mono_ready: bool,
    /// Partials since the last Mode 1 full.
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

/// HW reset pulse. OTP-Demo `hardware_reset` waits 10 ms around RST.
const RST_MS: u64 = 10;
/// How long to wait for BUSY to rise after Master Activation.
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

/// Yield the executor this often during a blocking RAM walk.
const YIELD_EVERY_ROWS: u16 = 16;

/// Cumulative [`display::OtpRefresh::Partial`]s before the
/// next mono path is [`display::OtpRefresh::MonoFull`].
/// `0` never promotes.
///
/// Follows the official PaperMono display safety rule: uninterrupted
/// partial refreshes can damage the panel. Official documentation
/// mandates a full refresh after roughly ten partial refreshes.
const PARTIALS_BEFORE_FULL: u8 = 6;

/// Last OTP stamp. Reprinted on 10 s `hello`. `busy_rose=0` means the
/// waveform never started.
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

pub async fn begin(
    i2c: &mut SysI2c,
    spi2: esp_hal::peripherals::SPI2<'static>,
    mosi: esp_hal::gpio::AnyPin<'static>,
    sclk: esp_hal::gpio::AnyPin<'static>,
    cs: esp_hal::gpio::AnyPin<'static>,
    dc: esp_hal::gpio::AnyPin<'static>,
    busy: &Input<'static>,
) -> Option<Panel> {
    if ioe::set_push_pull_output(i2c, ioe1::EPD_VDD_ENABLE, true).is_err() {
        return None;
    }
    Timer::after(Duration::from_millis(RST_MS)).await;
    let _ = ioe::set_push_pull_output(i2c, ioe1::EPD_RST, false);
    Timer::after(Duration::from_millis(RST_MS)).await;
    let _ = ioe::set_push_pull_output(i2c, ioe1::EPD_RST, true);
    Timer::after(Duration::from_millis(RST_MS)).await;
    wait_busy_low(busy).await;

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
    /// OTP-Demo `refresh_gray_full` (`OtpRefresh::GrayFull`).
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
        // Lite (2026-09-01): `Partial` after this left Ferris
        // on glass until those pixels were overdrawn. Not a
        // faint ghost. OTP-Demo `baseline_ready = false`.
        self.mono_ready = false;
        self.partials = 0;
    }

    /// [`OtpRefresh::MonoFull`] until a mono baseline exists, then
    /// [`OtpRefresh::Partial`].
    pub async fn paint_mono_fast(
        &mut self,
        i2c: &mut SysI2c,
        bw: &[u8],
        red: &[u8],
        busy: &Input<'static>,
    ) {
        if !self.mono_ready {
            self.refresh_mono_full(i2c, Some((bw, red)), busy).await;
            return;
        }
        self.refresh_partial_official(i2c, bw, red, busy).await;
    }

    /// OTP-Demo `refresh_mono_full` (white) before the target walk.
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

    /// Mono RAM window and X/Y increment. No software reset.
    /// After gray, data entry was X decrement; a later `Partial`
    /// write must not keep that.
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

    /// OTP-Demo `wake_for_partial_update`: HW reset, no SW reset.
    /// Also applies mono addressing so a `Partial` after `GrayFull`
    /// does not keep X-decrement.
    async fn wake_for_partial(&mut self, i2c: &mut SysI2c, busy: &Input<'_>) {
        self.hardware_reset(i2c, busy).await;
        self.apply_mono_addressing();
        let _ = self
            .epd
            .cmd(display::BORDER_WAVEFORM, &[display::BORDER_OTP_PARTIAL]);
    }

    /// OTP-Demo `refresh_mono_full`. `None` is both planes white.
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

    /// White field + one black mark. OTP-Demo partial (`0xFF`).
    /// `Blank` is `OtpRefresh::MonoFull` so leftover ink clears.
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
        self.partials = self.partials.saturating_add(1);
        if PARTIALS_BEFORE_FULL > 0 && self.partials >= PARTIALS_BEFORE_FULL {
            self.mono_ready = false;
        }
    }
}

/// Official-portrait plane → OTP RAM with X decrement (OTP-Demo).
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

/// Official portrait → OTP RAM, XY increment. Gray-black (1) → mono 0.
/// `invert` is OTP-Demo `write_ram(..., invert)` for `0xF8`.
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
    // OTP-Demo mono: 1 is white, 0 is black. MSB first, X increment.
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
