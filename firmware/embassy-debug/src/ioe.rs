//! M5IOE1 I2C I/O expander driver helpers and bus probing utilities.
//!
//! # Architecture & Hardware Provenance
//! The M5IOE1 is an I2C-controlled custom microcontroller acting as an I/O
//! expander on the PaperMono system bus (`GPIO47` SDA / `GPIO48` SCL).
//!
//! Key design considerations:
//! - **Addresses**: Primary address is `0x4F` (factory demo), with a library
//!   fallback of `0x6F`.
//! - **Wake Protocol**: The expander MCU enters low-power sleep; before register
//!   accesses, an address transaction ("wake signal") must be transmitted, followed
//!   by a short settling delay (~10 ms).
//! - **Push-Pull vs Open-Drain**: Power control gates (`EPD_VDD_ENABLE`, `TOUCH_VDD_ENABLE`,
//!   `PDM_VDD_ENABLE`) are configured in push-pull mode (`M=1`, `DRV=0`) to ensure
//!   crisp digital transitions on external MOSFET gates.

use core::sync::atomic::{AtomicU8, Ordering};

use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use m5stack_papermono_lite::addresses;
use m5stack_papermono_lite::ioe1;

/// Type alias for the synchronous blocking I2C driver on the system bus.
pub type SysI2c = I2c<'static, esp_hal::Blocking>;

/// Active runtime address discovered for the M5IOE1 expander (`0x4F` or `0x6F`).
static IOE_ADDR: AtomicU8 = AtomicU8::new(addresses::M5IOE1);

/// Settling delay following power rail application before polling I2C peripherals.
pub const POWER_SETTLE_MS: u64 = 500;

/// Settling delay after sending the M5IOE1 wake signal before attempting register reads.
const WAKE_SETTLE_MS: u64 = 10;

/// Retry interval between 100 kHz bus speed initialization attempts.
const INIT_RETRY_MS: u64 = 800;

/// Probes a register by writing its pointer and reading back one byte.
///
/// An `Ok` result indicates that an ACK was received on both the address and data phases.
pub fn probe_read(i2c: &mut SysI2c, addr: u8, reg: u8) -> bool {
    write_then_read(i2c, addr, reg).is_ok()
}

/// Probes an I2C device address for ACK by reading a single dummy byte.
///
/// Used for devices that do not expose a readable register pointer table (e.g., FT6336G).
pub fn probe_addr(i2c: &mut SysI2c, addr: u8) -> bool {
    let mut val = [0u8];
    i2c.read(addr, &mut val).is_ok()
}

/// Reads a single 8-bit register from a target device.
///
/// Returns `None` on NAK or bus error.
pub fn read_at(i2c: &mut SysI2c, addr: u8, reg: u8) -> Option<u8> {
    write_then_read(i2c, addr, reg).ok()
}

/// Reads a 16-bit little-endian register pair (used for RX8130 RTC timer counters).
#[cfg(feature = "sleep")]
pub fn read_le16(i2c: &mut SysI2c, addr: u8, reg: u8) -> Option<u16> {
    let mut buf = [0u8; 2];
    i2c.write_read(addr, &[reg], &mut buf).ok()?;
    Some(u16::from_le_bytes(buf))
}

/// Performs a burst read from the specified starting register into `buf`.
///
/// Used by touch scanning to fetch FT6336G coordinate registers in a single transaction.
#[cfg(feature = "panel")]
pub fn read_burst(i2c: &mut SysI2c, addr: u8, reg: u8, buf: &mut [u8]) -> bool {
    i2c.write_read(addr, &[reg], buf).is_ok()
}

/// Writes an 8-bit value to a specified peripheral register.
#[cfg(feature = "sleep")]
pub fn write_at(
    i2c: &mut SysI2c,
    addr: u8,
    reg: u8,
    val: u8,
) -> Result<(), esp_hal::i2c::master::Error> {
    i2c.write(addr, &[reg, val])
}

/// Writes a 16-bit little-endian value across two consecutive register addresses.
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

/// Internal helper: reads a 1-byte register via repeated START (`write_read`).
fn write_then_read(i2c: &mut SysI2c, addr: u8, reg: u8) -> Result<u8, esp_hal::i2c::master::Error> {
    let mut val = [0u8];
    i2c.write_read(addr, &[reg], &mut val)?;
    Ok(val[0])
}

/// Validates the presence of an M5IOE1 expander by reading its unique identifier registers.
fn ident(i2c: &mut SysI2c, addr: u8) -> bool {
    let mut uid = [0u8; 2];
    if i2c.write_read(addr, &[ioe1::UID_L], &mut uid).is_err() {
        return false;
    }
    write_then_read(i2c, addr, ioe1::REV).is_ok()
}

/// Transmits a wake-up transaction to the expander MCU.
fn wake(i2c: &mut SysI2c, addr: u8) {
    let _ = i2c.write(addr, &[ioe1::UID_L]);
}

/// Attempts to initialize the M5IOE1 at the given address, trying 100 kHz and 400 kHz clock rates.
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

/// Discovers and initializes the M5IOE1 expander across supported I2C addresses (`0x4F`, `0x6F`).
pub async fn begin_ioe(i2c: &mut SysI2c) -> Option<u8> {
    for &addr in &[addresses::M5IOE1, addresses::M5IOE1_UM] {
        if try_init_at(i2c, addr).await {
            IOE_ADDR.store(addr, Ordering::Relaxed);
            return Some(addr);
        }
    }
    None
}

/// Configures an expander pin as a push-pull digital output and sets its logic level.
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

/// Configures an expander pin as a digital input.
pub fn set_input(i2c: &mut SysI2c, pyg: u8) -> Result<(), esp_hal::i2c::master::Error> {
    m5stack_papermono_lite::m5ioe1::set_input(i2c, IOE_ADDR.load(Ordering::Relaxed), pyg)
}

/// Reads the instantaneous digital logic level of an expander input pin.
pub fn read_input(i2c: &mut SysI2c, pyg: u8) -> Result<bool, esp_hal::i2c::master::Error> {
    m5stack_papermono_lite::m5ioe1::read_input(i2c, IOE_ADDR.load(Ordering::Relaxed), pyg)
}
