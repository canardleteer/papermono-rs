//! UserDemo `enterRtc10sWakeShutdown`: RX8130 timer → PM1 G0
//! falling → `SYS_CMD` shutdown. ESP power drops; next boot
//! reprints `wake src=`.

use core::sync::atomic::{AtomicU8, Ordering};

use embassy_time::{Duration, Timer};
use m5stack_papermono_lite::addresses;
use m5stack_papermono_lite::pmic;
use m5stack_papermono_lite::rtc;

use crate::cdc;
use crate::ioe::{self, SysI2c};
use crate::touch_bus;

/// `0xFF` = no `wake` line yet.
static LAST_WAKE: AtomicU8 = AtomicU8::new(0xFF);
/// 0 = none, 1 = `sleep rtc=10`, 2 = `sleep abort`.
static LAST_SLEEP: AtomicU8 = AtomicU8::new(0);
const SLEEP_RTC: u8 = 1;
const SLEEP_ABORT: u8 = 2;

/// Show the lamp before we start watching VIN.
const ANNOUNCE_MS: u64 = 2000;
/// Sheet: 5VIN present is a power-on recovery. Wait for unplug.
const VIN_WAIT_MS: u64 = 60_000;
const VIN_POLL_MS: u64 = 100;
/// If `SYS_CMD` did not drop the ESP, stay running.
const ALIVE_AFTER_OFF_MS: u64 = 2000;

fn rmw(i2c: &mut SysI2c, reg: u8, clear: u8, set: u8) -> bool {
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    let Ok(cur) = pm1.read_at(reg) else {
        return false;
    };
    pm1.write_at(reg, (cur & !clear) | set).is_ok()
}

fn write_pm1(i2c: &mut SysI2c, reg: u8, val: u8) -> bool {
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    pm1.write_at(reg, val).is_ok()
}

fn g0_high(i2c: &mut SysI2c) -> Option<bool> {
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    pm1.read_at(pmic::GPIO_IN).ok().map(|v| v & 1 != 0)
}

fn rtc_irq(i2c: &mut SysI2c) -> bool {
    ioe::read_at(i2c, rtc::ADDRESS, rtc::FLAG).is_some_and(|f| f & rtc::FLAG_IRQ != 0)
}

fn stop_timer(i2c: &mut SysI2c) -> bool {
    let Some(ext) = ioe::read_at(i2c, rtc::ADDRESS, rtc::EXTENSION) else {
        return false;
    };
    let Some(ctl) = ioe::read_at(i2c, rtc::ADDRESS, rtc::CONTROL) else {
        return false;
    };
    ioe::write_at(i2c, rtc::ADDRESS, rtc::EXTENSION, ext & !rtc::TIMER_EN).is_ok()
        && ioe::write_at(i2c, rtc::ADDRESS, rtc::CONTROL, ctl & !rtc::TIMER_EN).is_ok()
        && ioe::write_at(i2c, rtc::ADDRESS, rtc::FLAG, rtc::FLAG_CLEAR_IRQ).is_ok()
}

fn arm_timer_10s(i2c: &mut SysI2c) -> bool {
    if !stop_timer(i2c) {
        return false;
    }
    let Some(ext) = ioe::read_at(i2c, rtc::ADDRESS, rtc::EXTENSION) else {
        return false;
    };
    let ext = (ext & !0x17) | rtc::TSEL_64HZ;
    if ioe::write_at(i2c, rtc::ADDRESS, rtc::EXTENSION, ext).is_err() {
        return false;
    }
    if ioe::write_at(i2c, rtc::ADDRESS, rtc::FLAG, rtc::FLAG_CLEAR_IRQ).is_err() {
        return false;
    }
    let Some(ctl) = ioe::read_at(i2c, rtc::ADDRESS, rtc::CONTROL) else {
        return false;
    };
    if ioe::write_at(i2c, rtc::ADDRESS, rtc::CONTROL, ctl | rtc::TIMER_EN).is_err() {
        return false;
    }
    let [lo, hi] = rtc::TIMER_10S_COUNTS.to_le_bytes();
    let mut ok = false;
    for _ in 0..3 {
        if ioe::write_pair(i2c, rtc::ADDRESS, rtc::TIMER_COUNTER_L, lo, hi).is_err() {
            continue;
        }
        if ioe::read_le16(i2c, rtc::ADDRESS, rtc::TIMER_COUNTER_L) == Some(rtc::TIMER_10S_COUNTS) {
            ok = true;
            break;
        }
    }
    if !ok {
        return false;
    }
    ioe::write_at(i2c, rtc::ADDRESS, rtc::EXTENSION, ext | rtc::TIMER_EN).is_ok()
}

fn preconfig_g0(i2c: &mut SysI2c) -> bool {
    let ok = rmw(i2c, pmic::GPIO_WAKE_EN, 1 << 0 | 1 << 4, 0)
        && write_pm1(i2c, pmic::IRQ_STATUS1, 0)
        && write_pm1(i2c, pmic::IRQ_STATUS2, 0)
        && write_pm1(i2c, pmic::IRQ_STATUS3, 0)
        && rmw(i2c, pmic::IRQ_MASK1, 1 << 0, 1 << 2 | 1 << 3 | 1 << 4)
        && rmw(i2c, pmic::GPIO_FUNC0, 0x03, 0)
        && rmw(i2c, pmic::GPIO_MODE, 1 << 0, 0)
        && rmw(i2c, pmic::GPIO_PUPD0, 0x03, pmic::GPIO_PULL_UP)
        && rmw(i2c, pmic::GPIO_DRV, 1 << 0, 0)
        && rmw(i2c, pmic::GPIO_WAKE_CFG, 1 << 0, 0)
        && rmw(i2c, pmic::HOLD_CFG, pmic::HOLD_LDO | pmic::HOLD_VIN, 0);
    let _ = write_pm1(i2c, pmic::WAKE_SRC, 0);
    ok
}

fn vin_mv(i2c: &mut SysI2c) -> Option<u16> {
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    if let Ok(v) = pm1.read_le16(pmic::VIN_L) {
        return Some(v);
    }
    let lo = pm1.read_at(pmic::VIN_L).ok()?;
    let hi = pm1.read_at(pmic::VIN_L.wrapping_add(1)).ok()?;
    Some(pmic::adc_mv(lo, hi))
}

fn vin_present(i2c: &mut SysI2c) -> bool {
    let adc_on = vin_mv(i2c).is_some_and(|mv| mv >= pmic::VIN_PRESENT_MV);
    let mut pm1 = m5stack_papermono_lite::m5pm1::M5pm1::new(&mut *i2c, addresses::M5PM1);
    let src_on = pm1
        .read_at(pmic::PWR_SRC)
        .is_ok_and(|s| s & (pmic::PWR_SRC_VIN | pmic::PWR_SRC_VINOUT) != 0);
    adc_on || src_on
}

async fn wait_vin_gone(i2c: &mut SysI2c) -> bool {
    let mut waited = 0_u64;
    while waited < VIN_WAIT_MS {
        if !vin_present(i2c) {
            return true;
        }
        Timer::after(Duration::from_millis(VIN_POLL_MS)).await;
        waited = waited.saturating_add(VIN_POLL_MS);
    }
    !vin_present(i2c)
}

fn abort(i2c: &mut SysI2c) {
    let _ = ioe::write_at(i2c, rtc::ADDRESS, rtc::USER_RAM_MARK, 0);
    touch_bus::apply_lamp(i2c, pmic::FRONTLIGHT_DUTY);
    LAST_SLEEP.store(SLEEP_ABORT, Ordering::Relaxed);
    cdc::sleep_abort();
}

/// Reprint `wake` / `sleep` on the 10 s `hello` period.
pub fn reprint() {
    let src = LAST_WAKE.load(Ordering::Relaxed);
    if src != 0xFF {
        cdc::wake(src);
    }
    match LAST_SLEEP.load(Ordering::Relaxed) {
        SLEEP_RTC => cdc::sleep_rtc(10),
        SLEEP_ABORT => cdc::sleep_abort(),
        _ => {}
    }
}

/// Print `wake src=`. If this boot is not a clean GPIO wake,
/// arm the 10 s RTC path. Lamp stays on until 5VIN is gone.
pub async fn maybe_rtc_10s(i2c: &mut SysI2c) {
    let src = ioe::read_at(i2c, addresses::M5PM1, pmic::WAKE_SRC).unwrap_or(0);
    LAST_WAKE.store(src, Ordering::Relaxed);
    cdc::wake(src);

    let _ = stop_timer(i2c);
    let _ = ioe::write_at(i2c, rtc::ADDRESS, rtc::USER_RAM_MARK, 0);
    let g0_idle = g0_high(i2c) == Some(true) && !rtc_irq(i2c);
    if src & pmic::WAKE_SRC_EXT != 0 && g0_idle {
        return;
    }

    LAST_SLEEP.store(SLEEP_RTC, Ordering::Relaxed);
    cdc::sleep_rtc(10);
    Timer::after(Duration::from_millis(ANNOUNCE_MS)).await;

    if !preconfig_g0(i2c) {
        abort(i2c);
        return;
    }
    if !wait_vin_gone(i2c).await {
        abort(i2c);
        return;
    }
    touch_bus::apply_lamp(i2c, 0);
    if !arm_timer_10s(i2c) {
        abort(i2c);
        return;
    }
    let _ = ioe::write_at(i2c, rtc::ADDRESS, rtc::FLAG, rtc::FLAG_CLEAR_IRQ);
    let _ = write_pm1(i2c, pmic::IRQ_STATUS1, 0);
    let _ = write_pm1(i2c, pmic::IRQ_STATUS2, 0);
    let _ = write_pm1(i2c, pmic::IRQ_STATUS3, 0);
    if !rmw(i2c, pmic::GPIO_WAKE_EN, 0, 1 << 0) {
        abort(i2c);
        return;
    }
    Timer::after(Duration::from_millis(100)).await;
    let _ = write_pm1(i2c, pmic::SYS_CMD, pmic::SYS_CMD_SHUTDOWN);
    Timer::after(Duration::from_millis(50)).await;
    let _ = write_pm1(i2c, pmic::SYS_CMD, pmic::SYS_CMD_SHUTDOWN);
    Timer::after(Duration::from_millis(ALIVE_AFTER_OFF_MS)).await;
    abort(i2c);
}
