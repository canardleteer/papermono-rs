//! Borrowed async transport for the reviewed lora-rs SX126x backend.
//!
//! The caller's SpiDevice owns NSS. BUSY polling and the post-NSS settling
//! delay surround every transaction, including upstream errata register access.

use crate::*;
use embedded_hal::{
    digital::InputPin,
    spi::{ErrorKind, ErrorType, Operation},
};
use embedded_hal_async::{delay::DelayNs, spi::SpiDevice};
use lora_phy::{mod_params::RadioError, mod_traits::InterfaceVariant};

/// Waits cooperatively using a requested-delay budget. Scheduler latency and
/// SPI time are outside the configured budget.
pub(crate) async fn wait_busy<B: InputPin, D: DelayNs, E>(
    busy: &mut B,
    delay: &mut D,
    timing: BusyTiming,
) -> Result<(), SxError<E, B::Error>> {
    let mut remaining = timing.budget_ms;
    loop {
        if busy.is_low().map_err(SxError::Busy)? {
            return Ok(());
        }
        if remaining == 0 {
            return Err(SxError::BusyTimeout);
        }
        let step = remaining.min(timing.poll_ms.get());
        delay.delay_ms(step).await;
        remaining -= step;
    }
}

/// Runs one complete device transaction. Await to completion: a cancelled
/// SpiDevice future may leave NSS low. Only the caller can recover its NSS pin.
pub(crate) async fn transaction<S: SpiDevice, B: InputPin, D: DelayNs>(
    spi: &mut S,
    busy: &mut B,
    delay: &mut D,
    timing: BusyTiming,
    operations: &mut [Operation<'_, u8>],
) -> Result<(), SxError<S::Error, B::Error>> {
    wait_busy(busy, delay, timing).await?;
    spi.transaction(operations).await.map_err(SxError::Spi)?;
    // sx1262 §8.3.1 “BUSY Control Line”: allow BUSY to assert after NSS
    // rises before testing readiness. The one-us delay needs C153 bench evidence.
    delay.delay_us(NSS_SETTLE_US).await;
    wait_busy(busy, delay, timing).await
}

/// Borrowed device; retains the original transport fault when upstream flattens it.
pub(crate) struct BorrowedDevice<'a, S: SpiDevice, B: InputPin, D> {
    pub spi: &'a mut S,
    pub busy: &'a mut B,
    pub delay: &'a mut D,
    pub timing: BusyTiming,
    pub fault: &'a mut Option<SxError<S::Error, B::Error>>,
}
impl<S: SpiDevice, B: InputPin, D> ErrorType for BorrowedDevice<'_, S, B, D> {
    type Error = ErrorKind;
}
impl<S: SpiDevice, B: InputPin, D: DelayNs> SpiDevice for BorrowedDevice<'_, S, B, D> {
    async fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), ErrorKind> {
        match transaction(self.spi, self.busy, self.delay, self.timing, operations).await {
            Ok(()) => Ok(()),
            Err(error) => {
                *self.fault = Some(error);
                Err(ErrorKind::Other)
            }
        }
    }
}

/// Upstream reset/IRQ operations are unavailable. Antenna changes are no-ops;
/// only the local explicit session hooks may control that hardware.
pub(crate) struct SessionInterface;
impl InterfaceVariant for SessionInterface {
    async fn reset(&mut self, _: &mut impl DelayNs) -> Result<(), RadioError> {
        Err(RadioError::Reset)
    }
    async fn wait_on_busy(&mut self) -> Result<(), RadioError> {
        Ok(())
    }
    async fn await_irq(&mut self) -> Result<(), RadioError> {
        Err(RadioError::Irq)
    }
    async fn enable_rf_switch_rx(&mut self) -> Result<(), RadioError> {
        Ok(())
    }
    async fn enable_rf_switch_tx(&mut self) -> Result<(), RadioError> {
        Ok(())
    }
    async fn disable_rf_switch(&mut self) -> Result<(), RadioError> {
        Ok(())
    }
}

/// Unexposed upstream driver: callers cannot reach upstream's unbounded TX API.
pub(crate) type Backend<'a, S, B, D> = lora_phy::sx126x::Sx126x<
    BorrowedDevice<'a, S, B, D>,
    SessionInterface,
    lora_phy::sx126x::Sx1262,
>;
impl<S: SpiDevice, B: InputPin, D: DelayNs, H> Sx1262<S, B, D, H> {
    /// Lends transport without giving upstream ownership of lifecycle state.
    pub(crate) fn backend<'a>(
        &'a mut self,
        fault: &'a mut Option<SxError<S::Error, B::Error>>,
    ) -> Backend<'a, S, B, D> {
        lora_phy::sx126x::Sx126x::new(
            BorrowedDevice {
                spi: &mut self.spi,
                busy: &mut self.busy,
                delay: &mut self.delay,
                timing: self.busy_timing,
                fault,
            },
            SessionInterface,
            lora_phy::sx126x::Config {
                chip: lora_phy::sx126x::Sx1262,
                tcxo_ctrl: None,
                use_dcdc: false,
                rx_boost: false,
            },
        )
    }
}

/// Keeps transport and upstream failures distinct, including upstream validation.
pub(crate) fn finish<S, B, T>(
    result: Result<T, RadioError>,
    fault: Option<SxError<S, B>>,
) -> Result<T, SxError<S, B>> {
    if let Some(error) = fault {
        return Err(error);
    }
    result.map_err(SxError::Upstream)
}
