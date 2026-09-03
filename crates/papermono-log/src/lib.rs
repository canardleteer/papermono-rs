//! USB-Serial/JTAG snapshot lines for PaperMono proof-of-life images.
//!
//! The firmware owns pins. This crate owns the **strings** it prints so
//! the log contract can be tested on the host. Every line starts with
//! [`LOG_PREFIX`]. Kinds: `hello`, `git=`, `hb`, `edge`, `gpio`,
//! `leftover`, `i2c`, `touch`, `mic`, `pcm`, `panel`, `scene=`,
//! `lamp=`, `wifi`, `ble`, `charge`, `sleep`, `wake`, `snowflake`. No MAC /
//! BSSID / IRK / USB serial fields.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod parse;

use core::fmt::{self, Write};
use core::str;

/// Token before every log line (`simple-debug: …`).
pub const LOG_PREFIX: &str = "simple-debug";

/// Image name for the blocking proof-of-life firmware.
pub const IMAGE: &str = "simple-debug";

/// Image name for the Embassy staged firmware.
pub const IMAGE_EMBASSY: &str = "embassy-debug";

/// Chip this image is built for. Not a unique unit id.
pub const CHIP: &str = "esp32s3";

/// Bytes reserved for a heartbeat (`hb` plus two button bits).
pub const HEARTBEAT_CAPACITY: usize = 64;

/// Bytes reserved for a git identity line (`git=<40 hex> dirty=0`).
pub const GIT_CAPACITY: usize = 80;

/// Bytes reserved for a repeating hello line.
pub const HELLO_CAPACITY: usize = 192;

/// Bytes reserved for a button-edge line.
pub const EDGE_CAPACITY: usize = 80;

/// Bytes reserved for a GPIO sample line.
pub const GPIO_CAPACITY: usize = 80;

/// Bytes reserved for a leftover-pad input line.
pub const LEFTOVER_CAPACITY: usize = 80;

/// Bytes reserved for a Wi-Fi scan-count line (`wifi n=`).
pub const WIFI_CAPACITY: usize = 40;

/// Bytes reserved for a BLE scan-count line (`ble n=`).
pub const BLE_CAPACITY: usize = 40;

/// Bytes reserved for a gated charge line (`charge vbat=`).
pub const CHARGE_CAPACITY: usize = 80;

/// Bytes reserved for a sleep-arm line (`sleep rtc=`).
pub const SLEEP_CAPACITY: usize = 40;

/// Bytes reserved for a wake-source line (`wake src=`).
pub const WAKE_CAPACITY: usize = 40;

/// Bytes reserved for an I2C ACK line.
pub const I2C_CAPACITY: usize = 192;

/// Bytes reserved for a touch line (INT, n, optional XY).
pub const TOUCH_CAPACITY: usize = 128;

/// Bytes reserved for a mic energy line.
pub const MIC_CAPACITY: usize = 64;

/// Bytes reserved for a PCM dump header (`mic pcm hz=… n=…`).
pub const PCM_HEADER_CAPACITY: usize = 80;

/// Bytes reserved for one PCM row (`pcm` plus 16 i16 samples).
pub const PCM_ROW_CAPACITY: usize = 192;

/// Bytes reserved for a panel stamp line.
pub const PANEL_CAPACITY: usize = 80;

/// Bytes reserved for a scene token (`scene=splash`).
pub const SCENE_CAPACITY: usize = 48;

/// Bytes reserved for a lamp duty line (`lamp=1024`).
pub const LAMP_CAPACITY: usize = 40;

/// Bytes reserved for a snowflake render timing line (`snowflake us=12345`).
pub const SNOWFLAKE_CAPACITY: usize = 48;

/// How long BUTTON A must stay low to dump PCM (page-prev is a tap).
pub const BUTTON_HOLD_PCM_MS: u32 = 1_000;

/// Phone-band A the human may hold on the hole. Not printed on CDC.
pub const TONE_A4_HZ: u32 = 440;

/// `mic pcm hz=` when the board plays nothing. Period is read from the rows.
pub const PCM_DUMP_NO_TONE_HZ: u32 = 0;

/// PDM window length in i16 samples.
pub const PCM_WINDOW_SAMPLES: usize = 256;

/// Samples per `pcm` CDC row.
pub const PCM_ROW_SAMPLES: usize = 16;

/// Windows to dump at each known-tone mark.
pub const TONE_DUMP_WINDOWS: u32 = 2;

/// Live energy period while `PYG12` is up.
pub const MIC_REPORT_MS: u32 = 250;

/// How often the firmware polls buttons, in milliseconds.
pub const POLL_PERIOD_MS: u32 = 50;

/// Heartbeat period used by the Xtensa image, in milliseconds.
pub const HEARTBEAT_PERIOD_MS: u32 = 1000;

/// How often hello / git / gpio repeat, in seconds.
pub const HELLO_PERIOD_S: u32 = 10;

/// Milliseconds in one second. Heartbeat `t=` is `t_ms` divided by this.
pub const MILLIS_PER_SEC: u32 = 1000;

/// [`HELLO_PERIOD_S`] in milliseconds.
pub const HELLO_PERIOD_MS: u32 = HELLO_PERIOD_S * MILLIS_PER_SEC;

const _: () = {
    assert!(HEARTBEAT_PERIOD_MS % POLL_PERIOD_MS == 0);
    assert!(HELLO_PERIOD_MS % HEARTBEAT_PERIOD_MS == 0);
};

/// Why a format into a caller buffer failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    /// The buffer was shorter than the formatted line.
    Truncated,
}

/// One sample of the raw BUTTON A / BUTTON B levels the image may print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    /// Seconds since the poll loop started.
    pub t_s: u32,
    /// BUTTON A (UP), GPIO2. `true` is high. Lite: idle high, press low.
    pub btn_a: bool,
    /// BUTTON B (DOWN), GPIO3. `true` is high. Lite: idle high, press low.
    pub btn_b: bool,
}

/// Identity fields that repeat so a late CDC attach still sees them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hello {
    /// Seconds since the poll loop started.
    pub t_s: u32,
    /// Firmware image token (`simple-debug` / `embassy-debug`).
    pub image: &'static str,
    /// Board SKU token (`C153-Lite`). Not a unique unit id.
    pub sku: &'static str,
    /// CPU clock from `esp_hal::clock::cpu_clock`, in MHz.
    pub cpu_mhz: u32,
    /// XTAL clock from `esp_hal::clock::xtal_clock`, in MHz.
    pub xtal_mhz: u32,
    /// Stable reset-reason token. Firmware maps `SocResetReason`.
    pub reset: &'static str,
}

/// One button transition. Omit a field when that key did not change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    /// Milliseconds since the poll loop started.
    pub t_ms: u32,
    /// BUTTON A (UP) `(was_high, is_high)` when it changed.
    pub btn_a: Option<(bool, bool)>,
    /// BUTTON B (DOWN) `(was_high, is_high)` when it changed.
    pub btn_b: Option<(bool, bool)>,
}

/// Input-only GPIO sample. No I2C, no EPD, no PDM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpioSample {
    /// GPIO0 M5PM1 `BOOT_OUT`. High means high.
    pub boot: bool,
    /// GPIO1 M5PM1 IRQ.
    pub pmic_irq: bool,
    /// GPIO4 FT6336G INT.
    pub tp: bool,
    /// GPIO7 M5IOE1 IRQ.
    pub ioe: bool,
    /// GPIO18 EPD BUSY. Panel is not inited.
    pub busy: bool,
}

/// Lite leftover MCU inputs. Full-SKU NFC/LoRa nets. Do not drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeftoverSample {
    /// GPIO5 LoRa IRQ net. High means high.
    pub lora_irq: bool,
    /// GPIO6 ST25R3916 IRQ net. High means high.
    pub nfc_irq: bool,
    /// GPIO21 SX1262 BUSY net. High means high.
    pub sx_busy: bool,
}

/// One gated M5PM1 / IP2315 charge sample. Millivolts only.
///
/// `ip` is ACK while the gate is on. `then` is ACK after park
/// (expect `false`). No IP2315 register payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChargeSample {
    /// M5PM1 `VBAT` millivolts, or `0` if the read NAKed.
    pub vbat: u16,
    /// M5PM1 `VIN` millivolts, or `0` if the read NAKed.
    pub vin: u16,
    /// M5PM1 `PWR_SRC` payload, or `0` if the read NAKed.
    pub src: u8,
    /// M5PM1 `PWR_CFG` `CHG_EN`.
    pub chg_en: bool,
    /// IP2315 ACK while `PYG11` is high.
    pub ip: bool,
    /// IP2315 ACK after `PYG11` is low again.
    pub then: bool,
}

/// On-unit radio scan counts. No BSSID / MAC / IRK.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioSample {
    /// Wi-Fi 2.4 GHz AP count from one scan.
    pub wifi_n: u16,
    /// BLE advertisement reports in the scan window.
    pub ble_n: u16,
}

/// I2C ACK bits (`1` = ACK). Lite NFC at `0x50` must be `0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct I2cSample {
    /// M5PM1 `0x6E`.
    pub pm1: bool,
    /// M5IOE1 ACK (board `0x4F` or UM `0x6F`).
    pub ioe: bool,
    /// 7-bit address that answered official `begin`, or `0`.
    pub ioe_addr: u8,
    /// RX8130CE `0x32`.
    pub rtc: bool,
    /// RX8130CE `FLAG` (`0x1D`) payload, or `0` when the read NAKed.
    ///
    /// Read-only. Do not write `SEC`.
    pub rtc_flag: u8,
    /// BMI270 `0x68`.
    pub imu: bool,
    /// BMI270 `CHIP_ID` payload, or `0` when the read NAKed.
    pub imu_id: u8,
    /// FT6336G `0x38`.
    pub tp: bool,
    /// ST25R3916 `0x50` (expect NAK on Lite).
    pub nfc: bool,
    /// IP2315 `0x75` (expect NAK when parked).
    pub chg: bool,
    /// M5IOE1 library / UM address `0x6F`.
    pub ioe_um: bool,
    /// M5IOE1 `PYG1` `TF_DET` after `PYG14` high. High = `1`.
    pub tf: bool,
}

/// Touch INT sample. XY is M5GFX `getTouchRaw` intent when `n>0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchSample {
    /// GPIO4 level (idle high on Lite). `/INT` data-ready is low.
    pub int_ready: bool,
    /// Contacts from M5GFX status byte, 0–2.
    pub n: u8,
    /// Point 0 X (official portrait) when `n>=1`.
    pub x: u16,
    /// Point 0 Y (official portrait) when `n>=1`.
    pub y: u16,
    /// Point 1 X when `n>=2`.
    pub x2: u16,
    /// Point 1 Y when `n>=2`.
    pub y2: u16,
}

/// PDM window energy. USB-Serial/JTAG only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MicSample {
    /// RMS of the PCM window (absolute i16 / 256, saturating).
    pub rms: u32,
    /// Peak absolute sample.
    pub peak: u32,
}

/// Cards the Embassy image can show. Cycle wraps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    /// Ferris + `papermono-rs` + A/B hints.
    Splash,
    /// Procedural 3-degree Koch snowflake with microsecond benchmark and test patterns.
    Shapes,
    /// Physical buttons, sleep/wake controls, and right lamp gutter legend.
    Legend,
    /// Four OTP gray boxes.
    Tones,
    /// Dots + midline slides + mono-full white clear.
    Targets,
}

impl Scene {
    /// Walk order for BUTTON B (next).
    pub const ALL: [Self; 5] = [
        Self::Splash,
        Self::Shapes,
        Self::Legend,
        Self::Tones,
        Self::Targets,
    ];

    /// Token after `scene=`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Splash => "splash",
            Self::Shapes => "shapes",
            Self::Legend => "legend",
            Self::Tones => "tones",
            Self::Targets => "targets",
        }
    }

    /// Next card, wrapping.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Splash => Self::Shapes,
            Self::Shapes => Self::Legend,
            Self::Legend => Self::Tones,
            Self::Tones => Self::Targets,
            Self::Targets => Self::Splash,
        }
    }

    /// Previous card, wrapping.
    #[must_use]
    pub const fn prev(self) -> Self {
        match self {
            Self::Splash => Self::Targets,
            Self::Shapes => Self::Splash,
            Self::Legend => Self::Shapes,
            Self::Tones => Self::Legend,
            Self::Targets => Self::Tones,
        }
    }

    /// OTP 4-gray `0xD7` on the tones card only.
    ///
    /// Splash is 1-bit Ferris line art (same 360×240
    /// canvas). Shapes / legend / splash stay black/white:
    /// `MonoFull` then `Partial`. Targets stay mono.
    /// Leaving a gray card rebuilds the mono baseline
    /// before any `0xFF`.
    #[must_use]
    pub const fn uses_gray(self) -> bool {
        matches!(self, Self::Tones)
    }
}

/// One OTP refresh stamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelStamp {
    /// OTP sequence title (`otp_gray` / `otp_mono` /
    /// `otp_partial`). Not an M5GFX `epd_*` title.
    pub mode: &'static str,
    /// RAM width used for this refresh.
    pub w: u16,
    /// RAM height used for this refresh.
    pub h: u16,
    /// BUSY went high after Master Activation.
    pub busy_rose: bool,
}

/// Writes `snapshot` into `buf` as a heartbeat line without a trailing newline.
pub fn format_heartbeat<'a>(
    snapshot: &Snapshot,
    buf: &'a mut [u8],
) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: hb t={} btn_a={} btn_b={}",
            LOG_PREFIX,
            snapshot.t_s,
            u8::from(snapshot.btn_a),
            u8::from(snapshot.btn_b),
        ),
    )
}

/// Writes a repeating identity line without a trailing newline.
pub fn format_hello<'a>(hello: &Hello, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: hello t={} image={} sku={} chip={} cpu_mhz={} xtal_mhz={} reset={}",
            LOG_PREFIX,
            hello.t_s,
            hello.image,
            hello.sku,
            CHIP,
            hello.cpu_mhz,
            hello.xtal_mhz,
            hello.reset,
        ),
    )
}

/// Writes `simple-debug: git=<hash> dirty=<0|1>` into `buf` without a trailing newline.
pub fn format_git<'a>(hash: &str, dirty: bool, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!("{}: git={hash} dirty={}", LOG_PREFIX, u8::from(dirty)),
    )
}

/// Writes a button-edge line without a trailing newline.
pub fn format_edge<'a>(edge: &Edge, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    let pos = {
        let mut writer = SliceWriter { buf, pos: 0 };
        write!(writer, "{}: edge t_ms={}", LOG_PREFIX, edge.t_ms)
            .map_err(|_| FormatError::Truncated)?;
        append_btn_edge(&mut writer, "btn_a", edge.btn_a)?;
        append_btn_edge(&mut writer, "btn_b", edge.btn_b)?;
        writer.pos
    };
    finish(buf, pos)
}

/// Writes a GPIO sample line without a trailing newline.
pub fn format_gpio<'a>(sample: &GpioSample, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: gpio boot={} pmic_irq={} tp={} ioe={} busy={}",
            LOG_PREFIX,
            u8::from(sample.boot),
            u8::from(sample.pmic_irq),
            u8::from(sample.tp),
            u8::from(sample.ioe),
            u8::from(sample.busy),
        ),
    )
}

/// Writes leftover-pad input levels without a trailing newline.
pub fn format_leftover<'a>(
    sample: &LeftoverSample,
    buf: &'a mut [u8],
) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: leftover lora_irq={} nfc_irq={} sx_busy={}",
            LOG_PREFIX,
            u8::from(sample.lora_irq),
            u8::from(sample.nfc_irq),
            u8::from(sample.sx_busy),
        ),
    )
}

/// Writes `wifi n=` without a trailing newline. Count only.
pub fn format_wifi(n: u16, buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(buf, format_args!("{}: wifi n={n}", LOG_PREFIX))
}

/// Writes `ble n=` without a trailing newline. Count only.
pub fn format_ble(n: u16, buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(buf, format_args!("{}: ble n={n}", LOG_PREFIX))
}

/// Writes `sleep rtc=` without a trailing newline.
pub fn format_sleep_rtc(secs: u8, buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(buf, format_args!("{}: sleep rtc={secs}", LOG_PREFIX))
}

/// Writes `sleep abort` without a trailing newline.
pub fn format_sleep_abort(buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(buf, format_args!("{}: sleep abort", LOG_PREFIX))
}

/// Writes `wake src=` without a trailing newline.
pub fn format_wake(src: u8, buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(buf, format_args!("{}: wake src={src:02x}", LOG_PREFIX))
}

/// Writes a gated charge sample without a trailing newline.
pub fn format_charge<'a>(sample: &ChargeSample, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: charge vbat={} vin={} src={:02x} chg_en={} ip={} then={}",
            LOG_PREFIX,
            sample.vbat,
            sample.vin,
            sample.src,
            u8::from(sample.chg_en),
            u8::from(sample.ip),
            u8::from(sample.then),
        ),
    )
}

/// Writes an I2C ACK sample without a trailing newline.
///
/// `ack=` / `nak=` list the advertised 7-bit addresses
/// (`0x32` RTC, `0x38` FT, `0x4F` board IOE, leftover
/// `0x50`, `0x68` IMU, `0x6E` PM1, UM `0x6F`, gated
/// `0x75`). Do not walk `0x70`–`0x76`.
pub fn format_i2c<'a>(sample: &I2cSample, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    let pos = {
        let mut writer = SliceWriter { buf, pos: 0 };
        write!(
            writer,
            "{}: i2c pm1={} ioe={} ioe_addr={:02x} rtc={} rtc_flag={:02x} imu={} imu_id={:02x} tp={} nfc={} chg={} tf={}",
            LOG_PREFIX,
            u8::from(sample.pm1),
            u8::from(sample.ioe),
            sample.ioe_addr,
            u8::from(sample.rtc),
            sample.rtc_flag,
            u8::from(sample.imu),
            sample.imu_id,
            u8::from(sample.tp),
            u8::from(sample.nfc),
            u8::from(sample.chg),
            u8::from(sample.tf),
        )
        .map_err(|_| FormatError::Truncated)?;
        writer
            .write_str(" ack=")
            .map_err(|_| FormatError::Truncated)?;
        write_i2c_scan(&mut writer, sample, true)?;
        writer
            .write_str(" nak=")
            .map_err(|_| FormatError::Truncated)?;
        write_i2c_scan(&mut writer, sample, false)?;
        writer.pos
    };
    finish(buf, pos)
}

/// Pin-map roster. `0x4F` vs `0x6F` follows which address
/// answered, plus a dedicated UM probe.
fn i2c_scan_map(sample: &I2cSample) -> [(u8, bool); 8] {
    [
        (0x32, sample.rtc),
        (0x38, sample.tp),
        (0x4F, sample.ioe_addr == 0x4F),
        (0x50, sample.nfc),
        (0x68, sample.imu),
        (0x6E, sample.pm1),
        (0x6F, sample.ioe_um || sample.ioe_addr == 0x6F),
        (0x75, sample.chg),
    ]
}

fn write_i2c_scan(
    writer: &mut SliceWriter<'_>,
    sample: &I2cSample,
    want_ack: bool,
) -> Result<(), FormatError> {
    let mut first = true;
    for &(addr, ack) in &i2c_scan_map(sample) {
        if ack != want_ack {
            continue;
        }
        if !first {
            writer.write_str(",").map_err(|_| FormatError::Truncated)?;
        }
        first = false;
        write!(writer, "{addr:02x}").map_err(|_| FormatError::Truncated)?;
    }
    Ok(())
}

/// Writes a touch INT line without a trailing newline.
pub fn format_touch<'a>(sample: &TouchSample, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    match sample.n {
        0 => write_into(
            buf,
            format_args!(
                "{}: touch int={} n=0",
                LOG_PREFIX,
                u8::from(sample.int_ready),
            ),
        ),
        1 => write_into(
            buf,
            format_args!(
                "{}: touch int={} n=1 x={} y={}",
                LOG_PREFIX,
                u8::from(sample.int_ready),
                sample.x,
                sample.y,
            ),
        ),
        _ => write_into(
            buf,
            format_args!(
                "{}: touch int={} n={} x={} y={} x2={} y2={}",
                LOG_PREFIX,
                u8::from(sample.int_ready),
                sample.n,
                sample.x,
                sample.y,
                sample.x2,
                sample.y2,
            ),
        ),
    }
}

/// Drawn target (`touch target=`). `kind` is `dot`, `slide_x`, or `slide_y`.
pub fn format_touch_target<'a>(
    id: u8,
    kind: &str,
    x: u16,
    y: u16,
    r: u16,
    buf: &'a mut [u8],
) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: touch target={id} kind={kind} x={x} y={y} r={r}",
            LOG_PREFIX,
        ),
    )
}

/// Live sample vs the drawn target (`tx`/`ty`).
pub fn format_touch_at<'a>(
    id: u8,
    sample: &TouchSample,
    tx: u16,
    ty: u16,
    buf: &'a mut [u8],
) -> Result<&'a str, FormatError> {
    if sample.n >= 2 {
        write_into(
            buf,
            format_args!(
                "{}: touch id={id} n={} x={} y={} x2={} y2={} tx={tx} ty={ty}",
                LOG_PREFIX, sample.n, sample.x, sample.y, sample.x2, sample.y2,
            ),
        )
    } else if sample.n == 1 {
        write_into(
            buf,
            format_args!(
                "{}: touch id={id} n=1 x={} y={} tx={tx} ty={ty}",
                LOG_PREFIX, sample.x, sample.y,
            ),
        )
    } else {
        write_into(
            buf,
            format_args!(
                "{}: touch id={id} n=0 int={} tx={tx} ty={ty}",
                LOG_PREFIX,
                u8::from(sample.int_ready),
            ),
        )
    }
}

/// Hit / miss / abort. `x`/`y` is the last sample; `tx`/`ty` is drawn.
#[allow(clippy::too_many_arguments)]
pub fn format_touch_verdict<'a>(
    id: u8,
    verdict: &str,
    x: u16,
    y: u16,
    tx: u16,
    ty: u16,
    d: u16,
    buf: &'a mut [u8],
) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: touch id={id} {verdict} x={x} y={y} tx={tx} ty={ty} d={d}",
            LOG_PREFIX,
        ),
    )
}

/// Writes a mic energy line without a trailing newline.
pub fn format_mic<'a>(sample: &MicSample, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: mic rms={} peak={}",
            LOG_PREFIX, sample.rms, sample.peak
        ),
    )
}

/// Header before a PCM dump. `hz=0` means the board played nothing.
pub fn format_mic_pcm_header(hz: u32, n: usize, buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(buf, format_args!("{}: mic pcm hz={hz} n={n}", LOG_PREFIX))
}

/// One row of signed i16 samples (`pcm <offset> s0 s1 …`).
pub fn format_mic_pcm_row<'a>(
    offset: usize,
    samples: &[i16],
    buf: &'a mut [u8],
) -> Result<&'a str, FormatError> {
    let pos = {
        let mut writer = SliceWriter { buf, pos: 0 };
        write!(writer, "{}: pcm {offset:03}", LOG_PREFIX).map_err(|_| FormatError::Truncated)?;
        for sample in samples {
            write!(writer, " {sample}").map_err(|_| FormatError::Truncated)?;
        }
        writer.pos
    };
    finish(buf, pos)
}

/// Writes `simple-debug: scene=<token>` without a trailing newline.
pub fn format_scene(scene: Scene, buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(
        buf,
        format_args!("{}: scene={}", LOG_PREFIX, scene.as_str()),
    )
}

/// Writes `simple-debug: lamp=<duty>` without a trailing newline.
pub fn format_lamp(duty: u16, buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(buf, format_args!("{}: lamp={duty}", LOG_PREFIX))
}

/// Writes `simple-debug: snowflake us=<us>` without a trailing newline.
pub fn format_snowflake(us: u32, buf: &mut [u8]) -> Result<&str, FormatError> {
    write_into(buf, format_args!("{}: snowflake us={us}", LOG_PREFIX))
}

/// Writes a panel stamp without a trailing newline.
pub fn format_panel<'a>(stamp: &PanelStamp, buf: &'a mut [u8]) -> Result<&'a str, FormatError> {
    write_into(
        buf,
        format_args!(
            "{}: panel mode={} w={} h={} busy_rose={}",
            LOG_PREFIX,
            stamp.mode,
            stamp.w,
            stamp.h,
            u8::from(stamp.busy_rose),
        ),
    )
}

fn append_btn_edge(
    writer: &mut SliceWriter<'_>,
    name: &str,
    change: Option<(bool, bool)>,
) -> Result<(), FormatError> {
    let Some((from, to)) = change else {
        return Ok(());
    };
    write!(writer, " {name}={}->{}", u8::from(from), u8::from(to))
        .map_err(|_| FormatError::Truncated)
}

fn write_into<'a>(buf: &'a mut [u8], args: fmt::Arguments<'_>) -> Result<&'a str, FormatError> {
    let pos = {
        let mut writer = SliceWriter { buf, pos: 0 };
        writer.write_fmt(args).map_err(|_| FormatError::Truncated)?;
        writer.pos
    };
    finish(buf, pos)
}

fn finish(buf: &[u8], pos: usize) -> Result<&str, FormatError> {
    str::from_utf8(&buf[..pos]).map_err(|_| FormatError::Truncated)
}

struct SliceWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl Write for SliceWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let rest = self.buf.len().saturating_sub(self.pos);
        if s.len() > rest {
            return Err(fmt::Error);
        }
        self.buf[self.pos..self.pos + s.len()].copy_from_slice(s.as_bytes());
        self.pos += s.len();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn heartbeat_has_kind_and_buttons() {
        let mut buf = [0u8; HEARTBEAT_CAPACITY];
        let line = format_heartbeat(
            &Snapshot {
                t_s: 12,
                btn_a: true,
                btn_b: false,
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(line, "simple-debug: hb t=12 btn_a=1 btn_b=0");
    }

    #[test]
    fn hello_names_image_sku_and_reset_without_mac() {
        let mut buf = [0u8; HELLO_CAPACITY];
        let line = format_hello(
            &Hello {
                t_s: 0,
                image: IMAGE,
                sku: "C153-Lite",
                cpu_mhz: 80,
                xtal_mhz: 40,
                reset: "chip_power_on",
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(
            line,
            "simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on"
        );
        assert!(!line.to_ascii_lowercase().contains("mac"));
    }

    #[test]
    fn git_line_marks_dirty() {
        let mut buf = [0u8; GIT_CAPACITY];
        let line = format_git("deadbeef", true, &mut buf).unwrap();
        assert_eq!(line, "simple-debug: git=deadbeef dirty=1");
    }

    #[test]
    fn git_line_marks_clean() {
        let mut buf = [0u8; GIT_CAPACITY];
        let line = format_git("abc", false, &mut buf).unwrap();
        assert_eq!(line, "simple-debug: git=abc dirty=0");
    }

    #[test]
    fn edge_one_button() {
        let mut buf = [0u8; EDGE_CAPACITY];
        let line = format_edge(
            &Edge {
                t_ms: 1250,
                btn_a: Some((true, false)),
                btn_b: None,
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(line, "simple-debug: edge t_ms=1250 btn_a=1->0");
    }

    #[test]
    fn edge_both_buttons() {
        let mut buf = [0u8; EDGE_CAPACITY];
        let line = format_edge(
            &Edge {
                t_ms: 50,
                btn_a: Some((true, false)),
                btn_b: Some((true, false)),
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(line, "simple-debug: edge t_ms=50 btn_a=1->0 btn_b=1->0");
    }

    #[test]
    fn gpio_sample_is_five_bits() {
        let mut buf = [0u8; GPIO_CAPACITY];
        let line = format_gpio(
            &GpioSample {
                boot: true,
                pmic_irq: true,
                tp: false,
                ioe: true,
                busy: false,
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(
            line,
            "simple-debug: gpio boot=1 pmic_irq=1 tp=0 ioe=1 busy=0"
        );
    }

    #[test]
    fn leftover_sample_is_three_bits() {
        let mut buf = [0u8; LEFTOVER_CAPACITY];
        let line = format_leftover(
            &LeftoverSample {
                lora_irq: false,
                nfc_irq: true,
                sx_busy: false,
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(
            line,
            "simple-debug: leftover lora_irq=0 nfc_irq=1 sx_busy=0"
        );
    }

    #[test]
    fn radio_counts_have_no_mac() {
        let mut wifi = [0u8; WIFI_CAPACITY];
        let mut ble = [0u8; BLE_CAPACITY];
        let wifi_line = format_wifi(12, &mut wifi).unwrap();
        let ble_line = format_ble(4, &mut ble).unwrap();
        assert_eq!(wifi_line, "simple-debug: wifi n=12");
        assert_eq!(ble_line, "simple-debug: ble n=4");
        assert!(!wifi_line.contains("mac"));
        assert!(!ble_line.contains("mac"));
    }

    #[test]
    fn charge_sample_has_no_mac() {
        let mut buf = [0u8; CHARGE_CAPACITY];
        let line = format_charge(
            &ChargeSample {
                vbat: 3921,
                vin: 5080,
                src: 0x05,
                chg_en: true,
                ip: true,
                then: false,
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(
            line,
            "simple-debug: charge vbat=3921 vin=5080 src=05 chg_en=1 ip=1 then=0"
        );
        assert!(!line.contains("mac"));
    }

    #[test]
    fn sleep_and_wake_lines() {
        let mut sleep = [0u8; SLEEP_CAPACITY];
        let mut abort = [0u8; SLEEP_CAPACITY];
        let mut wake = [0u8; WAKE_CAPACITY];
        assert_eq!(
            format_sleep_rtc(10, &mut sleep).unwrap(),
            "simple-debug: sleep rtc=10"
        );
        assert_eq!(
            format_sleep_abort(&mut abort).unwrap(),
            "simple-debug: sleep abort"
        );
        assert_eq!(
            format_wake(0x20, &mut wake).unwrap(),
            "simple-debug: wake src=20"
        );
    }

    #[test]
    fn i2c_sample_marks_lite_nfc_nak() {
        let mut buf = [0u8; I2C_CAPACITY];
        let line = format_i2c(
            &I2cSample {
                pm1: true,
                ioe: true,
                ioe_addr: 0x4F,
                rtc: true,
                rtc_flag: 0x00,
                imu: true,
                imu_id: 0x24,
                tp: true,
                nfc: false,
                chg: false,
                ioe_um: false,
                tf: true,
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(
            line,
            "simple-debug: i2c pm1=1 ioe=1 ioe_addr=4f rtc=1 rtc_flag=00 imu=1 imu_id=24 tp=1 nfc=0 chg=0 tf=1 ack=32,38,4f,68,6e nak=50,6f,75"
        );
    }

    #[test]
    fn touch_int_has_no_xy() {
        let mut buf = [0u8; TOUCH_CAPACITY];
        let line = format_touch(
            &TouchSample {
                int_ready: true,
                n: 0,
                x: 0,
                y: 0,
                x2: 0,
                y2: 0,
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(line, "simple-debug: touch int=1 n=0");
        assert!(!line.contains("x="));
        let xy = format_touch(
            &TouchSample {
                int_ready: false,
                n: 1,
                x: 240,
                y: 400,
                x2: 0,
                y2: 0,
            },
            &mut buf,
        )
        .unwrap();
        assert_eq!(xy, "simple-debug: touch int=0 n=1 x=240 y=400");
        let tgt = format_touch_target(5, "slide_x", 240, 400, 6, &mut buf).unwrap();
        assert_eq!(
            tgt,
            "simple-debug: touch target=5 kind=slide_x x=240 y=400 r=6"
        );
        let at = format_touch_at(
            0,
            &TouchSample {
                int_ready: false,
                n: 1,
                x: 230,
                y: 410,
                x2: 0,
                y2: 0,
            },
            240,
            400,
            &mut buf,
        )
        .unwrap();
        assert_eq!(at, "simple-debug: touch id=0 n=1 x=230 y=410 tx=240 ty=400");
        let hit = format_touch_verdict(0, "hit", 230, 410, 240, 400, 14, &mut buf).unwrap();
        assert_eq!(
            hit,
            "simple-debug: touch id=0 hit x=230 y=410 tx=240 ty=400 d=14"
        );
    }

    #[test]
    fn mic_pcm_header_is_requested_tone() {
        let mut buf = [0u8; PCM_HEADER_CAPACITY];
        assert_eq!(
            format_mic_pcm_header(PCM_DUMP_NO_TONE_HZ, PCM_WINDOW_SAMPLES, &mut buf).unwrap(),
            "simple-debug: mic pcm hz=0 n=256"
        );
        let mut row = [0u8; PCM_ROW_CAPACITY];
        assert_eq!(
            format_mic_pcm_row(0, &[120, -30, 400], &mut row).unwrap(),
            "simple-debug: pcm 000 120 -30 400"
        );
        assert_eq!(PCM_DUMP_NO_TONE_HZ, 0);
        assert_eq!(TONE_A4_HZ, 440);
        assert_eq!(PCM_ROW_SAMPLES, 16);
    }

    #[test]
    fn mic_and_panel_stamps() {
        let mut mic_buf = [0u8; MIC_CAPACITY];
        assert_eq!(
            format_mic(&MicSample { rms: 12, peak: 40 }, &mut mic_buf).unwrap(),
            "simple-debug: mic rms=12 peak=40"
        );
        let mut panel_buf = [0u8; PANEL_CAPACITY];
        assert_eq!(
            format_panel(
                &PanelStamp {
                    mode: "otp_orient",
                    w: 800,
                    h: 480,
                    busy_rose: true,
                },
                &mut panel_buf,
            )
            .unwrap(),
            "simple-debug: panel mode=otp_orient w=800 h=480 busy_rose=1"
        );
    }

    #[test]
    fn scene_wraps_and_formats() {
        assert_eq!(Scene::Splash.next(), Scene::Shapes);
        assert_eq!(Scene::Targets.next(), Scene::Splash);
        assert_eq!(Scene::Splash.prev(), Scene::Targets);
        assert_eq!(Scene::Targets.prev(), Scene::Tones);
        assert!(Scene::Tones.uses_gray());
        assert!(!Scene::Splash.uses_gray());
        assert!(!Scene::Legend.uses_gray());
        assert!(!Scene::Shapes.uses_gray());
        assert!(!Scene::Targets.uses_gray());
        assert_eq!(Scene::ALL.len(), 5);
        let mut buf = [0u8; SCENE_CAPACITY];
        assert_eq!(
            format_scene(Scene::Splash, &mut buf).unwrap(),
            "simple-debug: scene=splash"
        );
        let mut lamp = [0u8; LAMP_CAPACITY];
        assert_eq!(
            format_lamp(1024, &mut lamp).unwrap(),
            "simple-debug: lamp=1024"
        );
        let mut snowflake = [0u8; SNOWFLAKE_CAPACITY];
        assert_eq!(
            format_snowflake(1234, &mut snowflake).unwrap(),
            "simple-debug: snowflake us=1234"
        );
        assert_eq!(BUTTON_HOLD_PCM_MS, 1_000);
    }

    #[test]
    fn heartbeat_truncated_when_buffer_is_tiny() {
        let mut buf = [0u8; 4];
        assert_eq!(
            format_heartbeat(
                &Snapshot {
                    t_s: 1,
                    btn_a: false,
                    btn_b: false,
                },
                &mut buf,
            ),
            Err(FormatError::Truncated)
        );
    }
}
