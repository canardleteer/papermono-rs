//! M5IOE1 RMW helpers and read-only I2C probes. Catalog id `m5ioe1`.

use core::sync::atomic::{AtomicU8, Ordering};

use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use m5stack_papermono_lite::addresses;
use m5stack_papermono_lite::ioe1;

pub type SysI2c = I2c<'static, esp_hal::Blocking>;

static IOE_ADDR: AtomicU8 = AtomicU8::new(addresses::M5IOE1);

/// UserDemo `app_main` wait after power (“M5IOE1 and board I2C
/// peripherals … stable after power-up”).
pub const POWER_SETTLE_MS: u64 = 500;
/// Official `M5IOE1::sendWakeSignal` then `_initDevice`.
const WAKE_SETTLE_MS: u64 = 10;
/// Official `_tryInitAtAddress` wait between 100 kHz tries.
const INIT_RETRY_MS: u64 = 800;

/// Set the register pointer, STOP, then read. ACK of that read is
/// presence.
///
/// Do not `write(addr, &[0])` as a data poke: M5IOE1 UID (`0x00`)
/// is read-only.
pub fn probe_read(i2c: &mut SysI2c, addr: u8, reg: u8) -> bool {
    write_then_read(i2c, addr, reg).is_ok()
}

/// Address ACK only. Discards one current-address byte.
///
/// FT6336G (no public map) and parked IP2315 (expect NAK). Do not
/// interpret the payload.
pub fn probe_addr(i2c: &mut SysI2c, addr: u8) -> bool {
    let mut val = [0u8];
    i2c.read(addr, &mut val).is_ok()
}

/// Named-register read. `None` is NAK / bus error, not a zero payload.
pub fn read_at(i2c: &mut SysI2c, addr: u8, reg: u8) -> Option<u8> {
    write_then_read(i2c, addr, reg).ok()
}

/// Two-byte little-endian read (RX8130 timer counts).
#[cfg(feature = "sleep")]
pub fn read_le16(i2c: &mut SysI2c, addr: u8, reg: u8) -> Option<u16> {
    let mut buf = [0u8; 2];
    i2c.write_read(addr, &[reg], &mut buf).ok()?;
    Some(u16::from_le_bytes(buf))
}

/// Write register pointer, then read `buf`. M5GFX `getTouchRaw`
/// starts at [`m5stack_papermono_lite::touch::M5GFX_STATUS_REG`].
#[cfg(feature = "panel")]
pub fn read_burst(i2c: &mut SysI2c, addr: u8, reg: u8, buf: &mut [u8]) -> bool {
    i2c.write_read(addr, &[reg], buf).is_ok()
}

/// Named-register write.
#[cfg(feature = "sleep")]
pub fn write_at(
    i2c: &mut SysI2c,
    addr: u8,
    reg: u8,
    val: u8,
) -> Result<(), esp_hal::i2c::master::Error> {
    i2c.write(addr, &[reg, val])
}

/// Two data bytes after `reg` (RX8130 timer).
#[cfg(feature = "sleep")]
pub fn write_pair(
    i2c: &mut SysI2c,
    addr: u8,
    reg: u8,
    lo: u8,
    hi: u8,
) -> Result<(), esp_hal::i2c::master::Error> {
    i2c.write(addr, &[reg, lo, hi])
}

fn write_then_read(i2c: &mut SysI2c, addr: u8, reg: u8) -> Result<u8, esp_hal::i2c::master::Error> {
    i2c.write(addr, &[reg])?;
    let mut val = [0u8];
    i2c.read(addr, &mut val)?;
    Ok(val[0])
}

fn ident(i2c: &mut SysI2c, addr: u8) -> bool {
    let mut uid = [0u8; 2];
    if i2c.write(addr, &[ioe1::UID_L]).is_err() {
        return false;
    }
    if i2c.read(addr, &mut uid).is_err() {
        return false;
    }
    write_then_read(i2c, addr, ioe1::REV).is_ok()
}

/// Official wake is START+ADDR+W+STOP with no data. `esp-hal`
/// rejects a zero-length `write`; the IDF legacy helper is an
/// ignored UID pointer access. ACK during wake may timeout.
fn wake(i2c: &mut SysI2c, addr: u8) {
    let _ = i2c.write(addr, &[ioe1::UID_L]);
}

async fn try_init_at(i2c: &mut SysI2c, addr: u8) -> bool {
    let hz100 = Config::default();
    let hz400 = Config::default().with_frequency(Rate::from_khz(400));
    let _ = i2c.apply_config(&hz100);

    wake(i2c, addr);
    Timer::after(Duration::from_millis(WAKE_SETTLE_MS)).await;
    if ident(i2c, addr) {
        return true;
    }

    Timer::after(Duration::from_millis(INIT_RETRY_MS)).await;
    wake(i2c, addr);
    Timer::after(Duration::from_millis(WAKE_SETTLE_MS)).await;
    if ident(i2c, addr) {
        return true;
    }

    let _ = i2c.apply_config(&hz400);
    wake(i2c, addr);
    Timer::after(Duration::from_millis(WAKE_SETTLE_MS)).await;
    let ok = ident(i2c, addr);
    let _ = i2c.apply_config(&hz100);
    ok
}

/// UserDemo `begin(0x4F)` then library fallback `0x6F`. Not a
/// `0x70`–`0x76` walk.
pub async fn begin_ioe(i2c: &mut SysI2c) -> Option<u8> {
    for &addr in &[addresses::M5IOE1, addresses::M5IOE1_UM] {
        if try_init_at(i2c, addr).await {
            IOE_ADDR.store(addr, Ordering::Relaxed);
            return Some(addr);
        }
    }
    None
}

/// Drive `pyg` as push-pull output (DRV=0, M=1).
pub fn set_push_pull_output(
    i2c: &mut SysI2c,
    pyg: u8,
    high: bool,
) -> Result<(), esp_hal::i2c::master::Error> {
    m5stack_papermono_lite::m5ioe1::set_push_pull_output(
        i2c,
        IOE_ADDR.load(Ordering::Relaxed),
        pyg,
        high,
    )
}

/// `M=0` input. Catalog id `m5ioe1`, GPIO control.
pub fn set_input(i2c: &mut SysI2c, pyg: u8) -> Result<(), esp_hal::i2c::master::Error> {
    m5stack_papermono_lite::m5ioe1::set_input(i2c, IOE_ADDR.load(Ordering::Relaxed), pyg)
}

/// `GPIO_I_*` level. `true` is high.
pub fn read_input(i2c: &mut SysI2c, pyg: u8) -> Result<bool, esp_hal::i2c::master::Error> {
    m5stack_papermono_lite::m5ioe1::read_input(i2c, IOE_ADDR.load(Ordering::Relaxed), pyg)
}
