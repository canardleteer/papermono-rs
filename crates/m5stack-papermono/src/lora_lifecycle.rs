//! PaperMono C153 controls for a generic SX1262 session.
//!
//! Nets follow the official PaperMono PinMap and M5PaperMono-UserDemo
//! `main/hal/hal_lora.cpp`. Expander readback follows catalog `m5ioe1`,
//! UM V1.4 Table 3 “Register Map” and “GPIO control (IO1–IO14)”.

use crate::addresses;
use crate::lora::{CheckPhase, CheckRequest, Hooks, IOE1_ANTENNA_SWITCH, IOE1_RESET, PMIC_ENABLE};
use embedded_hal::i2c::I2c;
use embedded_hal_async::delay::DelayNs;
use m5stack_papermono_lite::{m5ioe1, m5pm1};

/// Existing diagnostic rail settling delay, retained pending C153 measurements.
pub const RAIL_SETTLE_MS: u32 = 15;
/// Existing diagnostic reset-release delay before the chip's BUSY check.
pub const BOOT_SETTLE_MS: u32 = 20;

/// System I2C is lent only for one lifecycle or TX verification call.
/// Other peripherals may use it once the operation's future completes.
pub struct RadioContext<'a, I, D> {
    /// Borrowed system I2C (SDA/SCL); never retained by the radio.
    pub i2c: &'a mut I,
    /// Caller delay implementation, such as `embassy_time::Delay`.
    pub delay: D,
    /// Discovered M5IOE1 address, preserving the firmware's address fallback.
    pub ioe_address: u8,
}

/// Digital antenna-control evidence. Unread fields remain unavailable on bus errors.
/// A sampled level confirms the digital control signal. RF impedance needs measurement.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AntennaEvidence {
    /// M5IOE1 mode bit; true means output.
    pub output: Option<bool>,
    /// Inverse DRV bit; true means push-pull.
    pub push_pull: Option<bool>,
    /// Output latch bit.
    pub latch: Option<bool>,
    /// GPIO input sample bit.
    pub level: Option<bool>,
}

impl AntennaEvidence {
    /// Requires output/push-pull mode and matching latch and input sample.
    pub const fn matches(&self, high: bool) -> bool {
        matches!(self.output, Some(true))
            && matches!(self.push_pull, Some(true))
            && matches!((self.latch, self.level), (Some(latch), Some(level)) if latch == high && level == high)
    }
}

/// Reason a digital confirmation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntennaFailure {
    /// All four fields agree with the expected control state.
    None,
    /// At least one required field could not be read over system I2C.
    Bus,
    /// Mode, drive, latch or sampled level disagreed with the expected state.
    Mismatch,
}

/// One typed antenna confirmation for a caller's serial/UI reporter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AntennaCheck {
    /// Phase, cadence, attempt and verification count from the chip wrapper.
    pub request: CheckRequest,
    /// High for startup/pre-TX, low for shutdown.
    pub expected_high: bool,
    /// Individual register evidence; no inferred RF-path validation.
    pub evidence: AntennaEvidence,
    /// Failure reason, including unavailable readback.
    pub failure: AntennaFailure,
}

/// Board lifecycle hooks with an optional typed reporter. No bus is owned here.
pub struct RadioHooks {
    report: fn(AntennaCheck),
}

/// Default reporter for callers that collect telemetry elsewhere.
fn discard_report(_: AntennaCheck) {}

impl Default for RadioHooks {
    fn default() -> Self {
        Self::new(discard_report)
    }
}

impl RadioHooks {
    /// Installs a serial/UI callback; callback must not retain the borrowed bus.
    pub const fn new(report: fn(AntennaCheck)) -> Self {
        Self { report }
    }
}

/// Reads one expander register using its documented pointer-write/read protocol.
fn read<I: I2c>(i2c: &mut I, address: u8, register: u8) -> Result<u8, I::Error> {
    i2c.write(address, &[register])?;
    let mut byte = [0];
    i2c.read(address, &mut byte)?;
    Ok(byte[0])
}

impl<I: I2c, D: DelayNs> Hooks<RadioContext<'_, I, D>> for RadioHooks {
    type Error = I::Error;
    fn tx_allowed(&self) -> bool {
        true
    }

    async fn startup(&mut self, context: &mut RadioContext<'_, I, D>) -> Result<(), I::Error> {
        // Assert reset before raising antenna and rail. Any failure rolls back
        // through the wrapper's shutdown, including disabled-state readback.
        m5ioe1::set_push_pull_output(context.i2c, context.ioe_address, IOE1_RESET, false)?;
        m5ioe1::set_push_pull_output(context.i2c, context.ioe_address, IOE1_ANTENNA_SWITCH, true)?;
        m5pm1::M5pm1::new(&mut *context.i2c, addresses::M5PM1)
            .set_gpio_output(PMIC_ENABLE, true)?;
        context.delay.delay_ms(RAIL_SETTLE_MS).await;
        m5ioe1::set_push_pull_output(context.i2c, context.ioe_address, IOE1_RESET, true)?;
        context.delay.delay_ms(BOOT_SETTLE_MS).await;
        Ok(())
    }

    async fn shutdown(&mut self, context: &mut RadioContext<'_, I, D>) -> Result<(), I::Error> {
        // Evaluate all control operations before choosing the first error.
        // Readback still runs in the wrapper even when one of these fails.
        let reset =
            m5ioe1::set_push_pull_output(context.i2c, context.ioe_address, IOE1_RESET, false);
        let antenna = m5ioe1::set_push_pull_output(
            context.i2c,
            context.ioe_address,
            IOE1_ANTENNA_SWITCH,
            false,
        );
        let rail = m5pm1::M5pm1::new(&mut *context.i2c, addresses::M5PM1)
            .set_gpio_output(PMIC_ENABLE, false);
        reset.and(antenna).and(rail)
    }

    async fn verify(
        &mut self,
        context: &mut RadioContext<'_, I, D>,
        request: CheckRequest,
    ) -> Result<bool, I::Error> {
        let expected_high = request.phase != CheckPhase::Shutdown;
        let mut evidence = AntennaEvidence::default();
        let mut error = None;
        // PYG2 is IO2, low-bank bit 1 (M5IOE1 UM Table 3). Read all evidence
        // fields, even if one failed, so a warning describes available state.
        let mask = 1 << (IOE1_ANTENNA_SWITCH - 1);
        for (register, target, invert) in [
            (m5ioe1::GPIO_M_L, &mut evidence.output, false),
            (m5ioe1::GPIO_DRV_L, &mut evidence.push_pull, true),
            (m5ioe1::GPIO_O_L, &mut evidence.latch, false),
            (m5ioe1::GPIO_I_L, &mut evidence.level, false),
        ] {
            match read(context.i2c, context.ioe_address, register) {
                Ok(byte) => *target = Some((byte & mask != 0) != invert),
                Err(failure) => {
                    if error.is_none() {
                        error = Some(failure);
                    }
                }
            }
        }
        let matches = evidence.matches(expected_high);
        let failure = if error.is_some() {
            AntennaFailure::Bus
        } else if matches {
            AntennaFailure::None
        } else {
            AntennaFailure::Mismatch
        };
        (self.report)(AntennaCheck {
            request,
            expected_high,
            evidence,
            failure,
        });
        if let Some(error) = error {
            Err(error)
        } else {
            Ok(matches)
        }
    }
}
