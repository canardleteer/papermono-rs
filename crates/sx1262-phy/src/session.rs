//! Explicit session state and caller-owned power/antenna policy.

use crate::{Sx1262, SxError, CMD_SET_TX};
use core::convert::Infallible;
use core::num::NonZeroU32;

/// Reason for a readiness check. Startup and shutdown checks always run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckPhase {
    /// Confirm enabled hardware after reset release and BUSY readiness.
    Startup,
    /// Confirm enabled hardware immediately before a scheduled TX attempt.
    BeforeTx,
    /// Confirm disabled hardware after all shutdown control steps.
    Shutdown,
}

/// Verification schedule and identity supplied to a hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckRequest {
    /// Lifecycle phase; shutdown expects the caller's disabled state.
    pub phase: CheckPhase,
    /// Number of TX attempts between checks; the first attempt always checks.
    pub interval: u32,
    /// One-based TX attempt number, or attempts completed at lifecycle checks.
    pub attempt: u64,
    /// One-based check number within this session, including lifecycle checks.
    pub check: u64,
}

/// Caller-owned hardware controls. Context is borrowed for one awaited operation
/// and is never stored by the chip driver. The caller supplies synchronization.
///
/// Power, reset, antenna control and oscillator settling belong in these hooks.
/// SPI configuration (including DIO2 and TCXO settings) remains explicit.
/// Implementations of shutdown must attempt all cleanup steps after a failure.
#[allow(async_fn_in_trait)]
pub trait Hooks<C: ?Sized> {
    /// Error from caller hardware control or readback.
    type Error;
    /// Permission policy checked before every TX, including unscheduled checks.
    fn tx_allowed(&self) -> bool {
        false
    }
    /// Assert reset, establish power/antenna state, settle, then release reset.
    async fn startup(&mut self, context: &mut C) -> Result<(), Self::Error>;
    /// Assert reset, disconnect antenna and remove power; attempt every step.
    async fn shutdown(&mut self, context: &mut C) -> Result<(), Self::Error>;
    /// Read fresh evidence. `false` means mismatch; errors mean unavailable evidence.
    /// Shutdown checks expect disabled state; other checks expect enabled state.
    async fn verify(&mut self, context: &mut C, request: CheckRequest)
        -> Result<bool, Self::Error>;
}

/// Default no-op lifecycle policy. Every TX is denied.
#[derive(Debug, Clone, Copy, Default)]
pub struct DenyTx;

/// Explicit policy for externally managed power/reset and a permanent antenna.
/// Constructing this policy asserts that the caller has established those facts.
/// No electrical or RF measurement is performed by this zero-sized policy.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlwaysConnected;

impl<C: ?Sized> Hooks<C> for DenyTx {
    type Error = Infallible;
    async fn startup(&mut self, _: &mut C) -> Result<(), Infallible> {
        Ok(())
    }
    async fn shutdown(&mut self, _: &mut C) -> Result<(), Infallible> {
        Ok(())
    }
    async fn verify(&mut self, _: &mut C, _: CheckRequest) -> Result<bool, Infallible> {
        Ok(true)
    }
}

impl<C: ?Sized> Hooks<C> for AlwaysConnected {
    type Error = Infallible;
    fn tx_allowed(&self) -> bool {
        true
    }
    async fn startup(&mut self, _: &mut C) -> Result<(), Infallible> {
        Ok(())
    }
    async fn shutdown(&mut self, _: &mut C) -> Result<(), Infallible> {
        Ok(())
    }
    async fn verify(&mut self, _: &mut C, _: CheckRequest) -> Result<bool, Infallible> {
        Ok(true)
    }
}

/// Session state. Faulted or interrupted lifecycle operations require shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// No active session; successful shutdown has completed.
    Inactive,
    /// Startup and enabled-state verification succeeded.
    Active,
    /// Readiness invalidated, or an awaited operation was interrupted.
    NeedsShutdown,
}

/// Counters retained across packet, standby and frequency operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionStats {
    /// Guarded TX attempts, including permission denials and SPI errors.
    pub tx_attempts: u64,
    /// Hook verification calls, including startup, rollback and shutdown.
    pub verifications: u64,
    /// Chip, lifecycle and verification failures, including permission denials.
    pub failures: u64,
}

impl SessionStats {
    /// Empty counters used for construction, actual startup and cadence changes.
    pub const fn new() -> Self {
        Self {
            tx_attempts: 0,
            verifications: 0,
            failures: 0,
        }
    }
}
impl Default for SessionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Primary reason startup could not establish a session.
#[derive(Debug, PartialEq)]
pub enum StartupFailure<S, H, B = Infallible> {
    /// Caller startup control failed.
    Control(H),
    /// SPI pin/BUSY readiness failed.
    Chip(SxError<S, B>),
    /// Caller readback failed.
    Verification(H),
    /// Readback disagreed with the enabled state.
    Mismatch,
}

/// Shutdown evidence retains control and verification failures independently.
#[derive(Debug, PartialEq)]
pub struct ShutdownFailure<H> {
    /// First control failure reported by the hook, after attempting cleanup.
    pub control: Option<H>,
    /// Readback error, even when control already failed.
    pub verification: Option<H>,
    /// Readback returned a disabled-state mismatch.
    pub mismatch: bool,
}

/// Errors from lifecycle and guarded TX operations.
#[derive(Debug, PartialEq)]
pub enum SessionError<S, H, B = Infallible> {
    /// Async chip operation failed.
    Chip(SxError<S, B>),
    /// Startup failed and rollback was attempted; preserve both results.
    Startup {
        /// Original startup failure.
        cause: StartupFailure<S, H, B>,
        /// Rollback failure, if disabled-state cleanup could not be confirmed.
        cleanup: Option<ShutdownFailure<H>>,
    },
    /// Shutdown could not confirm cleanup.
    Shutdown(ShutdownFailure<H>),
    /// No active session exists.
    NotActive,
    /// Shutdown followed by startup is required before further TX.
    RecoveryRequired,
    /// Caller policy denied TX; readiness was invalidated.
    TxDenied,
    /// Enabled hardware readback disagreed; readiness was invalidated.
    ReadinessMismatch,
    /// Hardware verification failed; readiness was invalidated.
    Verification(H),
}

impl<SPI, BUSY, DELAY, H> Sx1262<SPI, BUSY, DELAY, H>
where
    SPI: embedded_hal_async::spi::SpiDevice,
    DELAY: embedded_hal_async::delay::DelayNs,
    BUSY: embedded_hal::digital::InputPin,
{
    /// Current recorded readiness state; no device I/O.
    pub const fn session_state(&self) -> SessionState {
        self.state
    }
    /// Session telemetry; no device I/O.
    pub const fn session_stats(&self) -> SessionStats {
        self.stats
    }
    /// Explicitly changes check cadence and restarts its counters. The first TX
    /// checks again. A cadence change never repairs invalidated readiness.
    pub fn set_verification_interval(&mut self, interval: NonZeroU32) {
        if interval != self.interval {
            self.interval = interval;
            self.stats = SessionStats::new();
        }
    }
    /// Current cadence; starts at one and returns to one after verification failure.
    pub const fn verification_interval(&self) -> NonZeroU32 {
        self.interval
    }

    /// Starts one session. Repeated startup while active is a no-op, preserving
    /// chip configuration and counters. Startup failures attempt shutdown rollback.
    /// Keep lifecycle futures alive until completion; cancellation requires shutdown.
    pub async fn startup<C: ?Sized>(
        &mut self,
        context: &mut C,
    ) -> Result<(), SessionError<SPI::Error, H::Error, BUSY::Error>>
    where
        H: Hooks<C>,
    {
        match self.state {
            SessionState::Active => return Ok(()),
            SessionState::NeedsShutdown => return Err(SessionError::RecoveryRequired),
            SessionState::Inactive => {}
        }
        self.stats = SessionStats::new();
        self.modulation = None;
        self.frequency_hz = None;
        self.calibration_band = None;
        self.packet = None;
        self.timed_rx = false;
        self.state = SessionState::NeedsShutdown;
        let control = self.hooks.startup(context).await;
        // Do not wait for a powered chip after a failed control operation, but
        // still collect startup evidence before rollback. Cadence never skips it.
        let chip = if control.is_ok() {
            self.wait_busy().await
        } else {
            Ok(())
        };
        let request = self.check_request(CheckPhase::Startup);
        let verification = self.hooks.verify(context, request).await;
        let cause = if let Err(error) = control {
            Some(StartupFailure::Control(error))
        } else if let Err(error) = chip {
            Some(StartupFailure::Chip(error))
        } else {
            match verification {
                Ok(true) => None,
                Ok(false) => Some(StartupFailure::Mismatch),
                Err(error) => Some(StartupFailure::Verification(error)),
            }
        };
        if let Some(cause) = cause {
            self.stats.failures = self.stats.failures.saturating_add(1);
            self.interval = NonZeroU32::MIN;
            let cleanup = self.shutdown_inner(context).await.err();
            return Err(SessionError::Startup { cause, cleanup });
        }
        self.state = SessionState::Active;
        Ok(())
    }

    /// Invalidates readiness before awaiting cleanup. Always runs shutdown control
    /// and disabled-state verification, even when control fails or already inactive.
    pub async fn shutdown<C: ?Sized>(
        &mut self,
        context: &mut C,
    ) -> Result<(), SessionError<SPI::Error, H::Error, BUSY::Error>>
    where
        H: Hooks<C>,
    {
        self.shutdown_inner(context)
            .await
            .map_err(SessionError::Shutdown)
    }

    /// Runs both shutdown steps and preserves failures without early returns.
    async fn shutdown_inner<C: ?Sized>(
        &mut self,
        context: &mut C,
    ) -> Result<(), ShutdownFailure<H::Error>>
    where
        H: Hooks<C>,
    {
        self.state = SessionState::NeedsShutdown;
        let control = self.hooks.shutdown(context).await.err();
        let request = self.check_request(CheckPhase::Shutdown);
        let (verification, mismatch) = match self.hooks.verify(context, request).await {
            Ok(matches) => (None, !matches),
            Err(error) => (Some(error), false),
        };
        if control.is_some() || verification.is_some() || mismatch {
            self.stats.failures = self.stats.failures.saturating_add(1);
            self.interval = NonZeroU32::MIN;
            Err(ShutdownFailure {
                control,
                verification,
                mismatch,
            })
        } else {
            self.state = SessionState::Inactive;
            Ok(())
        }
    }

    /// Allocates a check number without resetting packet counters.
    fn check_request(&mut self, phase: CheckPhase) -> CheckRequest {
        self.stats.verifications = self.stats.verifications.saturating_add(1);
        CheckRequest {
            phase,
            interval: self.interval.get(),
            attempt: self.stats.tx_attempts,
            check: self.stats.verifications,
        }
    }

    /// Invalidates recorded readiness after denial, mismatch or failed readback.
    fn invalidate(&mut self) {
        self.state = SessionState::NeedsShutdown;
        self.interval = NonZeroU32::MIN;
        self.stats.failures = self.stats.failures.saturating_add(1);
    }

    /// Starts a packet TX after permission and recorded readiness checks. Checks
    /// fresh hardware on attempts 1, N+1, 2N+1, ...; SPI and BUSY polling await.
    /// Timeout is a 24-bit count of 15.625 us ticks; zero is rejected so TX
    /// always has a hardware limit.
    pub async fn set_tx<C: ?Sized>(
        &mut self,
        context: &mut C,
        timeout_ticks: u32,
    ) -> Result<(), SessionError<SPI::Error, H::Error, BUSY::Error>>
    where
        H: Hooks<C>,
    {
        if timeout_ticks == 0 || timeout_ticks > crate::MAX_TIMEOUT_TICKS {
            return Err(SessionError::Chip(SxError::InvalidParam));
        }
        let bytes = timeout_ticks.to_be_bytes();
        self.write_cmd_with_context(context, CMD_SET_TX, &bytes[1..])
            .await
    }

    /// Raw writes share the typed TX guard. Unsupported commands are rejected
    /// before SPI; non-TX commands neither start nor stop the caller's session.
    pub async fn write_cmd_with_context<C: ?Sized>(
        &mut self,
        context: &mut C,
        cmd: u8,
        params: &[u8],
    ) -> Result<(), SessionError<SPI::Error, H::Error, BUSY::Error>>
    where
        H: Hooks<C>,
    {
        if cmd != CMD_SET_TX {
            return self
                .write_cmd(cmd, params)
                .await
                .map_err(SessionError::Chip);
        }
        if params.len() != 3 || params == [0, 0, 0] {
            return Err(SessionError::Chip(SxError::InvalidParam));
        }
        match self.state {
            SessionState::Inactive => return Err(SessionError::NotActive),
            SessionState::NeedsShutdown => return Err(SessionError::RecoveryRequired),
            SessionState::Active => {}
        }
        self.stats.tx_attempts = self.stats.tx_attempts.saturating_add(1);
        if !self.hooks.tx_allowed() {
            self.invalidate();
            return Err(SessionError::TxDenied);
        }
        if (self.stats.tx_attempts - 1).is_multiple_of(u64::from(self.interval.get())) {
            let request = self.check_request(CheckPhase::BeforeTx);
            // An interrupted verification cannot leave stale permission behind.
            let saved = self.begin_io();
            match self.hooks.verify(context, request).await {
                Ok(true) => (self.state, self.interval) = saved,
                Ok(false) => {
                    self.invalidate();
                    return Err(SessionError::ReadinessMismatch);
                }
                Err(error) => {
                    self.invalidate();
                    return Err(SessionError::Verification(error));
                }
            }
        }
        // Upstream do_tx disables the hardware timer. Retain local guarded TX
        // with the caller's nonzero 24-bit limit instead.
        if self.timed_rx {
            self.stop_rx().await.map_err(SessionError::Chip)?;
        }
        self.write_unchecked(cmd, params)
            .await
            .map_err(SessionError::Chip)
    }
}
