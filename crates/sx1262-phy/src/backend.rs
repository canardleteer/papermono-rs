//! Borrowed adapter around the published lora-phy SX126x driver.
//!
//! Upstream supplies modem encodings and Semtech errata workarounds. Our adapter
//! keeps SPI blocking, checks BUSY before and after transactions, preserves the
//! original bus error, and leaves session antenna control to the lifecycle hooks.

use crate::*;
use core::future::Future;
use core::task::{Context, Poll, Waker};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::{ErrorKind, ErrorType, Operation, SpiBus};
use lora_phy::mod_params::RadioError;
use lora_phy::mod_traits::InterfaceVariant;

/// Blocking device borrowed only while an upstream chip operation executes.
pub(crate) struct BlockingDevice<'a, SPI: SpiBus, NSS, BUSY> {
    /// Exclusive SPI bus borrow; no mutex is needed inside this transaction.
    pub spi: &'a mut SPI,
    /// Active-low chip select, released on every transaction result.
    pub nss: &'a mut NSS,
    /// BUSY input read before selecting the chip and after deselection.
    pub busy: &'a mut BUSY,
    /// First hardware error retained despite upstream's flattened SPI error.
    pub fault: &'a mut Option<SxError<SPI::Error>>,
    /// Capability granted only by the completed session TX guard.
    pub allow_tx: bool,
}

impl<SPI: SpiBus, NSS, BUSY> ErrorType for BlockingDevice<'_, SPI, NSS, BUSY> {
    type Error = ErrorKind;
}

impl<SPI: SpiBus, NSS: OutputPin, BUSY: InputPin> BlockingDevice<'_, SPI, NSS, BUSY> {
    /// Bounded BUSY check from `sx1262` §8.3.1 “BUSY Control Line”.
    fn wait_busy(&mut self) -> Result<(), SxError<SPI::Error>> {
        for _ in 0..BUSY_TIMEOUT_POLLS {
            if self.busy.is_low().map_err(|_| SxError::Gpio)? {
                return Ok(());
            }
        }
        Err(SxError::BusyTimeout)
    }

    /// Runs a dedicated-bus SPI transaction, including flush and NSS cleanup.
    /// Delay operations are unsupported: the upstream calls we expose use none.
    fn run(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), SxError<SPI::Error>> {
        let Some(Operation::Write(header)) = operations.first() else {
            return Err(SxError::UnsupportedCommand);
        };
        let Some(&command) = header.first() else {
            return Err(SxError::InvalidParam);
        };
        // Reject continuous TX even if a future upstream operation emits it.
        if !matches!(
            command,
            CMD_READ_REGISTER
                | CMD_WRITE_REGISTER
                | CMD_READ_BUFFER
                | CMD_WRITE_BUFFER
                | CMD_SET_STANDBY
                | CMD_SET_MODULATION_PARAMS
                | CMD_SET_PACKET_PARAMS
                | CMD_SET_RF_FREQUENCY
                | CMD_SET_BUFFER_BASE_ADDRESS
                | CMD_SET_TX
        ) {
            return Err(SxError::UnsupportedCommand);
        }
        if command == CMD_SET_TX && !self.allow_tx {
            return Err(SxError::ContextRequired);
        }
        self.wait_busy()?;
        if self.nss.set_low().is_err() {
            let _ = self.nss.set_high();
            return Err(SxError::Gpio);
        }
        let result = (|| {
            for operation in operations {
                match operation {
                    Operation::Read(data) => self.spi.read(data)?,
                    Operation::Write(data) => self.spi.write(data)?,
                    Operation::Transfer(read, write) => self.spi.transfer(read, write)?,
                    Operation::TransferInPlace(data) => self.spi.transfer_in_place(data)?,
                    Operation::DelayNs(_) => return Err(SxError::UnsupportedCommand),
                }
            }
            self.spi.flush().map_err(SxError::Spi)
        })();
        let deselect = self.nss.set_high();
        result?;
        deselect.map_err(|_| SxError::Gpio)?;
        self.wait_busy()
    }
}

impl<SPI: SpiBus, NSS: OutputPin, BUSY: InputPin> embedded_hal_async::spi::SpiDevice
    for BlockingDevice<'_, SPI, NSS, BUSY>
{
    async fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), ErrorKind> {
        match self.run(operations) {
            Ok(()) => Ok(()),
            Err(error) => {
                *self.fault = Some(error);
                Err(ErrorKind::Other)
            }
        }
    }
}

impl<E: embedded_hal::spi::Error> From<E> for SxError<E> {
    fn from(error: E) -> Self {
        Self::Spi(error)
    }
}

/// Upstream reset and antenna hooks are deliberately unavailable/no-op. Session
/// hooks alone own those signals; packet and standby calls cannot toggle them.
pub(crate) struct SessionInterface;
impl InterfaceVariant for SessionInterface {
    async fn reset(
        &mut self,
        _: &mut impl embedded_hal_async::delay::DelayNs,
    ) -> Result<(), RadioError> {
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

/// Concrete borrowed upstream driver; not exposed, so its TX API cannot bypass hooks.
pub(crate) type Backend<'a, S, N, B> = lora_phy::sx126x::Sx126x<
    BlockingDevice<'a, S, N, B>,
    SessionInterface,
    lora_phy::sx126x::Sx1262,
>;

impl<SPI: SpiBus, NSS: OutputPin, BUSY: InputPin, H> Sx1262<SPI, NSS, BUSY, H> {
    /// Lends transport to upstream without transferring ownership or session state.
    pub(crate) fn backend<'a>(
        &'a mut self,
        fault: &'a mut Option<SxError<SPI::Error>>,
        allow_tx: bool,
    ) -> Backend<'a, SPI, NSS, BUSY> {
        lora_phy::sx126x::Sx126x::new(
            BlockingDevice {
                spi: &mut self.spi,
                nss: &mut self.nss,
                busy: &mut self.busy,
                fault,
                allow_tx,
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

/// Polls a fully blocking upstream operation once. Our SPI/interface futures never
/// yield; Pending is an explicit compatibility failure, never a spinning executor.
pub(crate) fn run_ready<T>(
    future: impl Future<Output = Result<T, RadioError>>,
) -> Result<T, RadioError> {
    let mut future = core::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => Err(RadioError::InvalidConfiguration),
    }
}

/// Preserves transport details; maps upstream parameter errors to the chip API.
pub(crate) fn finish<E, T>(
    result: Result<T, RadioError>,
    fault: Option<SxError<E>>,
) -> Result<T, SxError<E>> {
    if let Some(error) = fault {
        return Err(error);
    }
    result.map_err(|_| SxError::InvalidParam)
}
