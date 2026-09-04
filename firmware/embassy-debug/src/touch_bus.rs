//! System I2C peripheral discovery, IP2315 bus isolation, and frontlight PWM control.
//!
//! # Architecture & Safety Protocol
//! This module coordinates board-level peripheral orchestration over the primary
//! I2C bus (`I2C0` on `GPIO47`/`GPIO48`):
//!
//! - **IP2315 Bus Isolation Safety Rule**: The IP2315 fast charger can hang or latch up
//!   the system I2C bus, especially at lower battery voltages. It must remain disconnected
//!   (parked) via M5IOE1 `PYG11_PWM3` (`ioe1::IP2315_I2C_GATE`) at all times except during
//!   the brief active charge status reading window ([`bring_up`]).
//! - **Peripheral Roster**: Probes the presence of:
//!   - M5PM1 PMIC (`0x55` or `0x6F`)
//!   - M5IOE1 I/O expander (`0x4F` or `0x6F`)
//!   - RX8130CE RTC (`0x32`)
//!   - BMI270 / QMA6100P IMU (`0x68`)
//!   - FT6336G capacitive touch digitizer (`0x38`)
//!   - ST25R3916 NFC (`0x50` - expected NAK on Lite)
//!   - MicroSD card slot presence (`MICROSD_DETECT`)
//! - **Frontlight Control**: Regulates the display frontlight LED using M5PM1 `G3`/`PWM0`
//!   driving the AW9967 backlight boost driver.

use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, Ordering};

use embassy_time::{Duration, Timer};
use m5stack_papermono_lite::addresses;
use m5stack_papermono_lite::imu;
use m5stack_papermono_lite::ioe1;
use m5stack_papermono_lite::pmic;
use m5stack_papermono_lite::rtc;
#[cfg(feature = "panel")]
use m5stack_papermono_lite::touch;
use papermono_log::{ChargeSample, I2cSample, TouchSample};

use crate::cdc;
use crate::ioe::{self, SysI2c};

/// Pulse widths for resetting the FT6336G touch controller following power rail enable.
const TOUCH_RST_LOW_MS: u64 = 10;
const TOUCH_RST_HIGH_MS: u64 = 50;

const PM1: u16 = 1 << 0;
const IOE: u16 = 1 << 1;
const RTC: u16 = 1 << 2;
const IMU: u16 = 1 << 3;
const TP: u16 = 1 << 4;
const NFC: u16 = 1 << 5;
const CHG: u16 = 1 << 6;
const TF: u16 = 1 << 7;
const IOE_UM: u16 = 1 << 8;

/// Delay following MicroSD card power enable before sampling detect pin.
const TF_POWER_MS: u64 = 300;

/// Duration to hold the IP2315 gate switch closed during the charge transaction.
#[allow(dead_code)]
const CHARGE_MOUNT_MS: u64 = 50;

/// Settling delay after opening the IP2315 gate switch to ensure bus lines release.
#[allow(dead_code)]
const CHARGE_PARK_MS: u64 = 20;

const CHARGE_EN: u8 = 1 << 0;
const CHARGE_IP: u8 = 1 << 1;
const CHARGE_THEN: u8 = 1 << 2;

static LAMP_DUTY: AtomicU16 = AtomicU16::new(pmic::FRONTLIGHT_DUTY);
static CHARGE_VBAT: AtomicU16 = AtomicU16::new(0);
static CHARGE_VIN: AtomicU16 = AtomicU16::new(0);
static CHARGE_SRC: AtomicU8 = AtomicU8::new(0);
static CHARGE_BITS: AtomicU8 = AtomicU8::new(0);
static HAVE_CHARGE: AtomicBool = AtomicBool::new(false);
static I2C_BITS: AtomicU16 = AtomicU16::new(0);
static I2C_ADDR: AtomicU8 = AtomicU8::new(0);
static I2C_IMU_ID: AtomicU8 = AtomicU8::new(0);
static I2C_RTC_FLAG: AtomicU8 = AtomicU8::new(0);
static HAVE_I2C: AtomicBool = AtomicBool::new(false);
static HAVE_LAMP: AtomicBool = AtomicBool::new(false);

fn flag(bit: bool, mask: u16) -> u16 {
    if bit {
        mask
    } else {
        0
    }
}

fn store_i2c(sample: I2cSample) {
    let bits = flag(sample.pm1, PM1)
        | flag(sample.ioe, IOE)
        | flag(sample.rtc, RTC)
        | flag(sample.imu, IMU)
        | flag(sample.tp, TP)
        | flag(sample.nfc, NFC)
        | flag(sample.chg, CHG)
        | flag(sample.tf, TF)
        | flag(sample.ioe_um, IOE_UM);
    I2C_BITS.store(bits, Ordering::Relaxed);
    I2C_ADDR.store(sample.ioe_addr, Ordering::Relaxed);
    I2C_IMU_ID.store(sample.imu_id, Ordering::Relaxed);
    I2C_RTC_FLAG.store(sample.rtc_flag, Ordering::Relaxed);
    HAVE_I2C.store(true, Ordering::Relaxed);
    cdc::i2c(&sample);
}

/// Retrieves the most recent I2C peripheral scan results for periodic banner emission.
pub fn last_i2c() -> Option<I2cSample> {
    if !HAVE_I2C.load(Ordering::Relaxed) {
        return None;
    }
    let bits = I2C_BITS.load(Ordering::Relaxed);
    Some(I2cSample {
        pm1: bits & PM1 != 0,
        ioe: bits & IOE != 0,
        ioe_addr: I2C_ADDR.load(Ordering::Relaxed),
        rtc: bits & RTC != 0,
        rtc_flag: I2C_RTC_FLAG.load(Ordering::Relaxed),
        imu: bits & IMU != 0,
        imu_id: I2C_IMU_ID.load(Ordering::Relaxed),
        tp: bits & TP != 0,
        nfc: bits & NFC != 0,
        chg: bits & CHG != 0,
        ioe_um: bits & IOE_UM != 0,
        tf: bits & TF != 0,
    })
}

/// Initializes system expander rails, discovers I2C devices, and isolates unneeded peripherals.
///
/// Returns the responsive I2C address of the M5IOE1 expander, if found.
pub async fn bring_up(i2c: &mut SysI2c) -> Option<u8> {
    Timer::after(Duration::from_millis(ioe::POWER_SETTLE_MS)).await;

    let pm1 = ioe::probe_read(i2c, addresses::M5PM1, pmic::DEVICE_ID);
    let ioe_addr = ioe::begin_ioe(i2c).await;
    let ioe_ack = ioe_addr.is_some();

    if ioe_ack {
        // Enforce IP2315 safety isolation before any probing. Keep PDM off.
        let _ = ioe::set_push_pull_output(i2c, ioe1::IP2315_I2C_GATE, false);
        let _ = ioe::set_push_pull_output(i2c, ioe1::PDM_VDD_ENABLE, false);
        let _ = ioe::set_input(i2c, ioe1::MICROSD_DETECT);
        let _ = ioe::set_push_pull_output(i2c, ioe1::MICROSD_ENABLE, true);
        Timer::after(Duration::from_millis(TF_POWER_MS)).await;

        // AW9967 sits on the EPD 3.3 V rail. Raise rails and pulse touch reset.
        let _ = ioe::set_push_pull_output(i2c, ioe1::EPD_VDD_ENABLE, true);
        let _ = ioe::set_push_pull_output(i2c, ioe1::TOUCH_VDD_ENABLE, true);
        let _ = ioe::set_push_pull_output(i2c, ioe1::TOUCH_RST, false);
        Timer::after(Duration::from_millis(TOUCH_RST_LOW_MS)).await;
        let _ = ioe::set_push_pull_output(i2c, ioe1::TOUCH_RST, true);
        Timer::after(Duration::from_millis(TOUCH_RST_HIGH_MS)).await;
    }

    let rtc_flag = ioe::read_at(i2c, rtc::ADDRESS, rtc::FLAG);
    let rtc_ack = rtc_flag.is_some();
    let imu_id = ioe::read_at(i2c, imu::ADDRESS, imu::CHIP_ID).unwrap_or(0);
    let imu_ack = imu_id == imu::CHIP_ID_VALUE;
    let tp = if ioe_ack {
        ioe::probe_addr(i2c, addresses::FT6336G)
    } else {
        false
    };
    let nfc = ioe::probe_read(
        i2c,
        addresses::ST25R3916_LEFTOVER,
        addresses::ST25R3916_LEFTOVER_DEVICE_ID,
    );
    let charge = charge_once(i2c, ioe_ack).await;
    store_charge(charge);
    let chg = charge.then;
    let ioe_um = ioe::probe_addr(i2c, addresses::M5IOE1_UM);
    let tf = ioe_ack && ioe::read_input(i2c, ioe1::MICROSD_DETECT).unwrap_or(false);
    if ioe_ack {
        let _ = ioe::set_push_pull_output(i2c, ioe1::MICROSD_ENABLE, false);
    }

    if ioe_ack && pm1 {
        lamp_on(i2c);
    }

    store_i2c(I2cSample {
        pm1,
        ioe: ioe_ack,
        ioe_addr: ioe_addr.unwrap_or(0),
        rtc: rtc_ack,
        rtc_flag: rtc_flag.unwrap_or(0),
        imu: imu_ack,
        imu_id,
        tp,
        nfc,
        chg,
        ioe_um,
        tf,
    });
    ioe_addr
}

/// Retrieves the most recent battery and charger telemetry.
pub fn last_charge() -> Option<ChargeSample> {
    if !HAVE_CHARGE.load(Ordering::Relaxed) {
        return None;
    }
    let bits = CHARGE_BITS.load(Ordering::Relaxed);
    Some(ChargeSample {
        vbat: CHARGE_VBAT.load(Ordering::Relaxed),
        vin: CHARGE_VIN.load(Ordering::Relaxed),
        src: CHARGE_SRC.load(Ordering::Relaxed),
        chg_en: bits & CHARGE_EN != 0,
        ip: bits & CHARGE_IP != 0,
        then: bits & CHARGE_THEN != 0,
    })
}

fn store_charge(sample: ChargeSample) {
    CHARGE_VBAT.store(sample.vbat, Ordering::Relaxed);
    CHARGE_VIN.store(sample.vin, Ordering::Relaxed);
    CHARGE_SRC.store(sample.src, Ordering::Relaxed);
    CHARGE_BITS.store(
        flag_u8(sample.chg_en, CHARGE_EN)
            | flag_u8(sample.ip, CHARGE_IP)
            | flag_u8(sample.then, CHARGE_THEN),
        Ordering::Relaxed,
    );
    HAVE_CHARGE.store(true, Ordering::Relaxed);
    crate::cdc::charge(&sample);
}

fn flag_u8(bit: bool, mask: u8) -> u8 {
    if bit {
        mask
    } else {
        0
    }
}

fn read_adc_mv(i2c: &mut SysI2c, lo_reg: u8) -> Option<u16> {
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    if let Ok(v) = pm1.read_le16(lo_reg) {
        return Some(v);
    }
    let lo = pm1.read_at(lo_reg).ok()?;
    let hi = pm1.read_at(lo_reg.wrapping_add(1)).ok()?;
    Some(pmic::adc_mv(lo, hi))
}

/// Reads fresh battery and voltage telemetry from M5PM1 and updates stored charge state.
///
/// Leaves IP2315 parked off the system I2C bus to eliminate bus lockup hazards.
#[cfg(feature = "panel")]
pub fn refresh_battery(i2c: &mut SysI2c) -> ChargeSample {
    let vbat = read_adc_mv(i2c, pmic::VBAT_L).unwrap_or(0);
    let vin = read_adc_mv(i2c, pmic::VIN_L).unwrap_or(0);
    let (src, cfg) = {
        let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
        (
            pm1.read_at(pmic::PWR_SRC).unwrap_or(0),
            pm1.read_at(pmic::PWR_CFG).unwrap_or(0),
        )
    };
    let sample = ChargeSample {
        vbat,
        vin,
        src,
        chg_en: cfg & pmic::CHG_EN != 0,
        ip: false,
        then: false,
    };
    store_charge(sample);
    sample
}

/// Executes a gated IP2315 charge transaction: reads M5PM1 telemetry while ensuring IP2315 remains parked.
async fn charge_once(i2c: &mut SysI2c, can_gate: bool) -> ChargeSample {
    let vbat = read_adc_mv(i2c, pmic::VBAT_L).unwrap_or(0);
    let vin = read_adc_mv(i2c, pmic::VIN_L).unwrap_or(0);
    let (src, cfg) = {
        let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
        (
            pm1.read_at(pmic::PWR_SRC).unwrap_or(0),
            pm1.read_at(pmic::PWR_CFG).unwrap_or(0),
        )
    };
    // Ensure IP2315 remains parked off the system I2C bus. Mounting IP2315 (especially
    // when VIN is present or at low VBAT) causes the charger in LED mode to pull SDA/SCL
    // to GND, permanently locking up the bus.
    if can_gate {
        let _ = ioe::set_push_pull_output(i2c, ioe1::IP2315_I2C_GATE, false);
    }
    let ip = false;
    let then = ioe::probe_addr(i2c, addresses::IP2315);
    ChargeSample {
        vbat,
        vin,
        src,
        chg_en: cfg & pmic::CHG_EN != 0,
        ip,
        then,
    }
}

/// Retrieves the active frontlight LED PWM duty cycle.
pub fn last_lamp() -> Option<u16> {
    if !HAVE_LAMP.load(Ordering::Relaxed) {
        return None;
    }
    Some(LAMP_DUTY.load(Ordering::Relaxed))
}

/// Applies a new frontlight LED PWM duty cycle to the M5PM1 PMIC.
#[cfg(feature = "sleep")]
pub fn apply_lamp(i2c: &mut SysI2c, duty: u16) {
    let _ = ioe::set_push_pull_output(i2c, ioe1::EPD_VDD_ENABLE, true);
    if duty == 0 {
        write_pwm0(i2c, 0);
    } else {
        enable_pwm0(i2c);
        write_pwm0(i2c, duty);
    }
    LAMP_DUTY.store(duty, Ordering::Relaxed);
    HAVE_LAMP.store(true, Ordering::Relaxed);
    crate::cdc::lamp(duty);
}

/// Controls the M5PM1 red power/status LED indicator.
#[cfg(feature = "sleep")]
pub fn apply_red_led(i2c: &mut SysI2c, on: bool) {
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    let _ = pm1.set_led(on);
}

/// Returns a default empty touch sample representing an idle digitizer state.
pub fn empty_touch(int_high: bool) -> TouchSample {
    TouchSample {
        int_ready: int_high,
        n: 0,
        x: 0,
        y: 0,
        x2: 0,
        y2: 0,
    }
}

#[cfg(feature = "panel")]
const LAMP_EMPTY_RESET: u8 = 4;

/// Fetches multi-touch coordinate data from the FT6336G capacitive digitizer.
#[cfg(feature = "panel")]
pub fn read_points(i2c: &mut SysI2c, int_high: bool, force: bool) -> TouchSample {
    if int_high && !force {
        return empty_touch(true);
    }
    const LEN: usize = 1 + (touch::MAX_POINTS as usize) * touch::M5GFX_POINT_BYTES;
    let mut buf = [0u8; LEN];
    if !ioe::read_burst(i2c, addresses::FT6336G, touch::M5GFX_STATUS_REG, &mut buf) {
        return empty_touch(int_high);
    }
    let Some((n, x, y, x2, y2)) = touch::decode_m5gfx(&buf) else {
        return empty_touch(int_high);
    };
    TouchSample {
        int_ready: int_high,
        n,
        x,
        y,
        x2,
        y2,
    }
}

/// Gesture detector for the display right-edge frontlight brightness slider.
#[cfg(feature = "panel")]
pub struct LampSlide {
    empty: u8,
    armed: bool,
}

#[cfg(feature = "panel")]
impl LampSlide {
    /// Creates a new inactive lamp slider gesture recognizer.
    pub const fn new() -> Self {
        Self {
            empty: 0,
            armed: false,
        }
    }

    /// Indicates whether the right-edge gutter contact is actively tracking.
    #[allow(dead_code)]
    pub const fn armed(&self) -> bool {
        self.armed
    }

    /// Evaluates a touch sample: updates lamp duty if inside the right gutter.
    ///
    /// Returns `true` if the sample was consumed by the gutter slider.
    pub fn feed(&mut self, i2c: &mut SysI2c, sample: &TouchSample) -> bool {
        if sample.n < 1 {
            self.empty = self.empty.saturating_add(1);
            if self.empty >= LAMP_EMPTY_RESET {
                self.empty = 0;
                self.armed = false;
            }
            return false;
        }
        if !touch::in_lamp_gutter(sample.x) {
            self.empty = 0;
            self.armed = false;
            return false;
        }
        self.empty = 0;
        self.armed = true;
        set_frontlight_duty(i2c, duty_from_y(sample.y));
        true
    }
}

#[cfg(feature = "panel")]
fn duty_from_y(y: u16) -> u16 {
    let span = touch::ACTIVE_MAX_Y
        .saturating_sub(touch::ACTIVE_MIN_Y)
        .max(1);
    let y = y.clamp(touch::ACTIVE_MIN_Y, touch::ACTIVE_MAX_Y);
    let from_bottom = touch::ACTIVE_MAX_Y.saturating_sub(y);
    ((u32::from(from_bottom) * u32::from(pmic::PWM0_DUTY_MAX)) / u32::from(span)) as u16
}

/// Enables the frontlight at default brightness level.
fn lamp_on(i2c: &mut SysI2c) {
    enable_pwm0(i2c);
    write_pwm0(i2c, pmic::FRONTLIGHT_DUTY);
    LAMP_DUTY.store(pmic::FRONTLIGHT_DUTY, Ordering::Relaxed);
    HAVE_LAMP.store(true, Ordering::Relaxed);
    crate::cdc::lamp(pmic::FRONTLIGHT_DUTY);
}

#[cfg(feature = "panel")]
fn set_frontlight_duty(i2c: &mut SysI2c, duty: u16) {
    if LAMP_DUTY.load(Ordering::Relaxed) == duty {
        return;
    }
    enable_pwm0(i2c);
    write_pwm0(i2c, duty);
    LAMP_DUTY.store(duty, Ordering::Relaxed);
    HAVE_LAMP.store(true, Ordering::Relaxed);
    crate::cdc::lamp(duty);
}

fn enable_pwm0(i2c: &mut SysI2c) {
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    let _ = pm1.enable_pwm0(pmic::FRONTLIGHT_PWM);
}

fn write_pwm0(i2c: &mut SysI2c, duty: u16) {
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    let _ = pm1.set_pwm0_duty(duty);
}
