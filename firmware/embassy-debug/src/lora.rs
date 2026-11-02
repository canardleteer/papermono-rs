//! Stamp LoRa-1262 (SX1262) transceiver driver and interactive test service.
//!
//! PaperMono's official PinMap and
//! [M5PaperMono-UserDemo LoRa HAL](https://github.com/m5stack/M5PaperMono-UserDemo/blob/main/main/hal/hal_lora.cpp)
//! identify SPI3 on GPIO38/39/40/41, BUSY on GPIO21 and DIO1 on GPIO5.
//! M5PM1 G2 gates `3V3_L2_LoRa`; M5IOE1 PYG10 asserts reset and PYG2
//! controls the antenna path. C153-Lite has no populated radio; runtime
//! detection bypasses initialization and keeps these GPIOs undriven.
//!
//! Each on-demand probe, ping, receive window or sweep uses one explicit
//! startup/shutdown pair. PYG2 stays high during packet and channel operations.
//! Readback confirms digital controls at startup/shutdown and before every TX.
//! Failures warn on CDC and require confirmed cleanup before another startup.
//!
//! TX occurs only after a tap on `[ TX PING ]`. The existing profile sends
//! `SetTxParams` +14 with the retained PA settings and 60 mA OCP code. This
//! command value does not establish radiated power: catalog `sx1262` Rev 2.2
//! §13.1.14.1 “PA Optimal Settings” describes matching-network-dependent rows.
//! RF power and antenna performance remain hardware measurements.
//!
//! Presets remain 915.000 MHz for ping and 917.625/906.875 MHz for reception.
//! CDC logs packet metrics and bounded preview bytes. Embassy timers yield
//! during settling and IRQ polling; SPI transactions and BUSY polling also yield.
//! Ownership and borrowing follow the
//! [Embedded Rust Book](https://docs.rust-embedded.org/book/peripherals/borrowing.html),
//! [Rust on ESP Book](https://docs.espressif.com/projects/rust/book/application-development/),
//! and [Embassy Book](https://embassy.dev/book/).

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, AtomicI16, AtomicI8, AtomicU32, AtomicU8, Ordering};
use embassy_sync::blocking_mutex::CriticalSectionMutex;

#[cfg(feature = "c153")]
use embassy_time::{Duration, Instant, Timer};
#[cfg(feature = "c153")]
use esp_hal::gpio::{Input, Output};
#[cfg(feature = "c153")]
use esp_hal::spi::master::Spi;
#[cfg(feature = "c153")]
use m5stack_papermono::lora::{
    self, PacketStatus, RadioStatus, ReceivePoll, Sx1262, CAL_IMG_902_MHZ, CAL_IMG_928_MHZ,
    FREQ_BENCH_PING_HZ, FREQ_RX_SNIFFER_PRI_HZ, FREQ_RX_SNIFFER_SEC_HZ, IRQ_ALL, IRQ_CRC_ERR,
    IRQ_RX_DONE, IRQ_TIMEOUT, IRQ_TX_DONE, LORA_BW_125_KHZ, LORA_BW_250_KHZ, LORA_CRC_ON,
    LORA_CR_4_5, LORA_HEADER_VARIABLE, LORA_IQ_STANDARD, LORA_LDRO_OFF, LORA_SF11, LORA_SF7,
    OCP_60_MA, PACKET_TYPE_LORA, PA_DEVICE_SEL_SX1262, PA_DUTY_CYCLE_14DBM, PA_HP_MAX_14DBM,
    PA_LUT_DEFAULT, RAMP_40_US, REGULATOR_LDO, STDBY_CONFIG_RC, SYNC_WORD_ALT,
    SYNC_WORD_MESHTASTIC, SYNC_WORD_PRIVATE, TCXO_CTRL_3_0V, TCXO_DEFAULT_DELAY_TICKS,
};
#[cfg(feature = "c153")]
use m5stack_papermono::lora::{
    AntennaCheck, AntennaFailure, CheckPhase, RadioContext, RadioHooks, SessionError,
    StartupFailure, SxError,
};
#[cfg(feature = "c153")]
pub use m5stack_papermono::lora::{ChipMode, CommandStatus};

#[cfg(feature = "c153")]
use crate::ioe;

/// Fallback chip mode on Lite SKU.
#[cfg(not(feature = "c153"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipMode {
    /// Standby RC mode.
    StbyRc,
    /// Other mode code.
    Other(u8),
}

/// Fallback command status on Lite SKU.
#[cfg(not(feature = "c153"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    /// Data available.
    DataAvailable,
    /// Other command status code.
    Other(u8),
}

/// Calculates the center frequency in Hz for a US915 250 kHz uplink channel slot (0..104).
pub const fn us915_channel_freq_hz(slot: u8) -> u32 {
    902_125_000 + (slot as u32) * 250_000
}

/// UI state of the Stamp LoRa-1262 card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoraCardState {
    /// Hardware unpopulated (Lite SKU or missing module).
    Unpopulated,
    /// Transceiver probed and idle in low-power standby.
    Idle {
        /// Raw status byte returned from `CMD_GET_STATUS`.
        raw_status: u8,
        /// Decoded transceiver operating mode.
        mode: ChipMode,
        /// Decoded status of last processed command.
        cmd: CommandStatus,
    },
    /// Transmitted a user-controlled test ping.
    Transmitted {
        /// Carrier frequency in kHz (e.g. 915000).
        freq_khz: u32,
        /// Configured power command in dBm; RF output requires measurement.
        pwr_dbm: i8,
        /// Airtime / execution duration in milliseconds.
        time_ms: u32,
        /// Confirmation that `IRQ_TX_DONE` was raised.
        ok: bool,
    },
    /// Received a valid packet during the listen window.
    Received {
        /// Monitored frequency in kHz.
        freq_khz: u32,
        /// Packet RSSI in dBm.
        rssi: i16,
        /// Packet SNR in dB.
        snr: i8,
        /// Received payload length in bytes.
        len: u8,
        /// First four preview bytes.
        preview: [u8; 4],
    },
    /// Listened on packet sniffer frequency, but no packet was detected (ambient noise measured).
    ListenQuiet {
        /// Monitored frequency in kHz.
        freq_khz: u32,
        /// Instantaneous ambient RF noise floor in dBm.
        ambient_rssi: i16,
    },
    /// Actively listening for incoming packets (up to 60s window).
    Listening {
        /// Monitored frequency in kHz.
        freq_khz: u32,
    },
}

// Cross-task atomic state:
// State code: 0 = Unpopulated, 1 = Idle, 2 = Transmitted, 3 = Received, 4 = ListenQuiet, 5 = Listening.
static LORA_STATE_CODE: AtomicU8 = AtomicU8::new(0);
static LORA_INITIALIZED: AtomicBool = AtomicBool::new(false);
static LORA_RAW_STATUS: AtomicU8 = AtomicU8::new(0);
static LORA_FREQ_KHZ: AtomicU32 = AtomicU32::new(915_000);
static LORA_TX_PWR: AtomicI8 = AtomicI8::new(14);
static LORA_TIME_MS: AtomicU32 = AtomicU32::new(0);
static LORA_TX_OK: AtomicBool = AtomicBool::new(false);
static LORA_RSSI: AtomicI16 = AtomicI16::new(0);
static LORA_SNR: AtomicI8 = AtomicI8::new(0);
static LORA_LEN: AtomicU8 = AtomicU8::new(0);
static LORA_PREVIEW: [AtomicU8; 4] = [
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
];
#[cfg(feature = "c153")]
static LORA_SNIFFER_SLOT: AtomicU8 = AtomicU8::new(0);

/// Results of US915 LoRa channel scan.
#[derive(Debug, Clone, Copy)]
pub struct LoraScanData {
    /// True while an active channel sweep is running.
    pub scanning: bool,
    /// Number of completed full sweeps across all 104 channels.
    pub sweeps: u16,
    /// Current channel slot being probed (0..103).
    pub current_slot: u8,
    /// Total valid LoRa packets captured during the session.
    pub total_packets: u16,
    /// Channel slot with the highest activity or signal level.
    pub peak_slot: u8,
    /// Maximum RSSI observed during the scan in dBm.
    pub peak_rssi: i16,
    /// Activity hit counter for each of the 104 channels.
    pub hits: [u8; 104],
}

impl Default for LoraScanData {
    fn default() -> Self {
        Self {
            scanning: false,
            sweeps: 0,
            current_slot: 0,
            total_packets: 0,
            peak_slot: 0,
            peak_rssi: -128,
            hits: [0u8; 104],
        }
    }
}

static SCAN_DATA: CriticalSectionMutex<RefCell<LoraScanData>> =
    CriticalSectionMutex::new(RefCell::new(LoraScanData {
        scanning: false,
        sweeps: 0,
        current_slot: 0,
        total_packets: 0,
        peak_slot: 0,
        peak_rssi: -128,
        hits: [0u8; 104],
    }));

static SCAN_STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Retrieves a snapshot of the current LoRa channel scan status and results.
pub fn lora_scan_data() -> LoraScanData {
    SCAN_DATA.lock(|cell| *cell.borrow())
}

/// Requests the active scanner to stop at the next channel boundary.
pub fn request_scan_stop() {
    SCAN_STOP_REQUESTED.store(true, Ordering::Release);
}

/// Updates the active scanning flag in global scan data.
pub fn set_scanning(scanning: bool) {
    SCAN_DATA.lock(|cell| {
        cell.borrow_mut().scanning = scanning;
    });
}

#[cfg(feature = "c153")]
/// Caller-owned resources stored between diagnostic sessions. esp-hal's Async
/// mode is not Send; only the idle Blocking mode crosses this static mutex.
/// On C153-Lite this container remains empty and no radio output is configured.
struct LoraHardware {
    /// SPI3 resource, temporarily taken by AsyncSpiLease while a session runs.
    spi: Option<Spi<'static, esp_hal::Blocking>>,
    /// NSS output retained for recovery after an interrupted device transaction.
    nss: Output<'static>,
    /// BUSY input on GPIO21, without an internal pull.
    busy: Input<'static>,
}

#[cfg(feature = "c153")]
static LORA_HW: embassy_sync::mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    Option<LoraHardware>,
> = embassy_sync::mutex::Mutex::new(None);

/// Stores initial presence status of the LoRa hardware.
pub fn init_status(available: bool) {
    if available {
        LORA_STATE_CODE.store(1, Ordering::Release);
        LORA_INITIALIZED.store(true, Ordering::Release);
    } else {
        LORA_STATE_CODE.store(0, Ordering::Release);
        LORA_INITIALIZED.store(true, Ordering::Release);
    }
}

/// Registers the physical SPI3 peripheral and pins for LoRa operations.
#[cfg(feature = "c153")]
pub async fn init_hardware(
    spi: Spi<'static, esp_hal::Blocking>,
    nss: Output<'static>,
    busy: Input<'static>,
) {
    let mut lock = LORA_HW.lock().await;
    *lock = Some(LoraHardware {
        spi: Some(spi),
        nss,
        busy,
    });
}

/// Retrieves the current snapshot of the LoRa card UI state.
#[must_use]
pub fn card_state() -> LoraCardState {
    if !LORA_INITIALIZED.load(Ordering::Acquire) {
        return LoraCardState::Unpopulated;
    }

    match LORA_STATE_CODE.load(Ordering::Acquire) {
        0 => LoraCardState::Unpopulated,
        1 => {
            let raw = LORA_RAW_STATUS.load(Ordering::Acquire);
            #[cfg(feature = "c153")]
            {
                let status = RadioStatus::from_byte(raw);
                LoraCardState::Idle {
                    raw_status: raw,
                    mode: status.chip_mode,
                    cmd: status.command_status,
                }
            }
            #[cfg(not(feature = "c153"))]
            {
                LoraCardState::Idle {
                    raw_status: raw,
                    mode: ChipMode::StbyRc,
                    cmd: CommandStatus::DataAvailable,
                }
            }
        }
        2 => LoraCardState::Transmitted {
            freq_khz: LORA_FREQ_KHZ.load(Ordering::Acquire),
            pwr_dbm: LORA_TX_PWR.load(Ordering::Acquire),
            time_ms: LORA_TIME_MS.load(Ordering::Acquire),
            ok: LORA_TX_OK.load(Ordering::Acquire),
        },
        3 => LoraCardState::Received {
            freq_khz: LORA_FREQ_KHZ.load(Ordering::Acquire),
            rssi: LORA_RSSI.load(Ordering::Acquire),
            snr: LORA_SNR.load(Ordering::Acquire),
            len: LORA_LEN.load(Ordering::Acquire),
            preview: [
                LORA_PREVIEW[0].load(Ordering::Acquire),
                LORA_PREVIEW[1].load(Ordering::Acquire),
                LORA_PREVIEW[2].load(Ordering::Acquire),
                LORA_PREVIEW[3].load(Ordering::Acquire),
            ],
        },
        4 => LoraCardState::ListenQuiet {
            freq_khz: LORA_FREQ_KHZ.load(Ordering::Acquire),
            ambient_rssi: LORA_RSSI.load(Ordering::Acquire),
        },
        5 => LoraCardState::Listening {
            freq_khz: LORA_FREQ_KHZ.load(Ordering::Acquire),
        },
        _ => LoraCardState::Unpopulated,
    }
}

/// Returns the current sniffer target frequency in Hz.
#[must_use]
pub fn current_sniffer_freq_hz() -> u32 {
    #[cfg(feature = "c153")]
    {
        if LORA_SNIFFER_SLOT.load(Ordering::Relaxed) == 0 {
            FREQ_RX_SNIFFER_PRI_HZ
        } else {
            FREQ_RX_SNIFFER_SEC_HZ
        }
    }
    #[cfg(not(feature = "c153"))]
    {
        917_625_000
    }
}

/// Sets the shared UI state to active listening.
pub fn set_listening_state(freq_khz: u32) {
    LORA_FREQ_KHZ.store(freq_khz, Ordering::Release);
    LORA_STATE_CODE.store(5, Ordering::Release);
}

/// Records a control or transport failure for both radio cards.
/// Atomic state crosses UI/heartbeat tasks; the system I2C bus stays with its caller.
static LORA_CONTROL_FAILED: AtomicBool = AtomicBool::new(false);

/// Retains unfinished cleanup across diagnostic wrappers and cancelled futures.
/// The SPI3 mutex serializes changes; a later operation confirms shutdown first.
#[cfg(feature = "c153")]
static LORA_NEEDS_CLEANUP: AtomicBool = AtomicBool::new(false);

/// Reports whether the last radio session failed control, verification or cleanup.
/// A later successful startup clears the banner before operations resume.
pub fn control_failed() -> bool {
    LORA_CONTROL_FAILED.load(Ordering::Acquire)
}

/// Borrowed SPI3 transport and PaperMono hooks. System I2C is not stored here.
#[cfg(feature = "c153")]
type RadioDevice<'a> = embedded_hal_bus::spi::ExclusiveDevice<
    AsyncSpiLease<'a>,
    &'a mut Output<'static>,
    embassy_time::Delay,
>;
/// Borrowed async SPI device, BUSY, cooperative delay and board lifecycle hooks.
/// The device owns NSS only for this session; LoraHardware retains the pin.
#[cfg(feature = "c153")]
type Radio<'a> = Sx1262<RadioDevice<'a>, &'a mut Input<'static>, embassy_time::Delay, RadioHooks>;
/// Complete SPI-device error, including NSS, retained by the chip wrapper.
#[cfg(feature = "c153")]
type ChipError =
    SxError<embedded_hal_bus::spi::DeviceError<esp_hal::spi::Error, core::convert::Infallible>>;

/// Re-establishes NSS high before recovery and composes borrowed async SPI/NSS.
/// The SPI3 mutex excludes other callers. No command future is raced against a
/// timer: user stop requests are checked only between completed sequences.
#[cfg(feature = "c153")]
fn session(hw: &mut LoraHardware) -> Radio<'_> {
    // ExclusiveDevice::new drives NSS high even if a previous future was dropped
    // mid-transaction. The retained cleanup flag then forces shutdown/startup.
    let device = embedded_hal_bus::spi::ExclusiveDevice::new(
        AsyncSpiLease::new(&mut hw.spi),
        &mut hw.nss,
        embassy_time::Delay,
    )
    .expect("infallible NSS output");
    Sx1262::with_hooks(
        device,
        &mut hw.busy,
        embassy_time::Delay,
        RadioHooks::new(report_antenna),
    )
}

/// Owns async SPI3 for one session and restores idle mode on Drop, including
/// cancellation. This mode conversion follows esp-hal's single-core async
/// ownership rule; it does not run a command or change antenna/power controls.
#[cfg(feature = "c153")]
struct AsyncSpiLease<'a> {
    /// Async peripheral, always Some until Drop transfers it back to the slot.
    spi: Option<Spi<'static, esp_hal::Async>>,
    /// Vacant caller-owned storage behind the already-held SPI3 mutex guard.
    slot: &'a mut Option<Spi<'static, esp_hal::Blocking>>,
}
#[cfg(feature = "c153")]
impl<'a> AsyncSpiLease<'a> {
    /// Takes the idle peripheral. Exclusivity guarantees the slot is populated.
    fn new(slot: &'a mut Option<Spi<'static, esp_hal::Blocking>>) -> Self {
        let spi = slot
            .take()
            .expect("SPI3 lease returned by previous session")
            .into_async();
        Self {
            spi: Some(spi),
            slot,
        }
    }
    /// Borrows the active async peripheral; the lease keeps it alive until Drop.
    fn spi(&mut self) -> &mut Spi<'static, esp_hal::Async> {
        self.spi.as_mut().expect("active SPI3 lease")
    }
}
#[cfg(feature = "c153")]
impl Drop for AsyncSpiLease<'_> {
    /// Returns ownership to the mutex's idle slot. NSS is re-established at the
    /// next session boundary, before the remembered shutdown recovery runs.
    fn drop(&mut self) {
        if let Some(spi) = self.spi.take() {
            *self.slot = Some(spi.into_blocking());
        }
    }
}
#[cfg(feature = "c153")]
impl embedded_hal::spi::ErrorType for AsyncSpiLease<'_> {
    type Error = esp_hal::spi::Error;
}
#[cfg(feature = "c153")]
impl embedded_hal_async::spi::SpiBus for AsyncSpiLease<'_> {
    /// Reads SPI3 asynchronously; returns the original HAL error.
    async fn read(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        embedded_hal_async::spi::SpiBus::read(self.spi(), data).await
    }
    /// Writes SPI3 asynchronously; device-level NSS remains in ExclusiveDevice.
    async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        embedded_hal_async::spi::SpiBus::write(self.spi(), data).await
    }
    /// Exchanges separate slices asynchronously without allocating a buffer.
    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        embedded_hal_async::spi::SpiBus::transfer(self.spi(), read, write).await
    }
    /// Exchanges one caller-owned slice asynchronously, propagating HAL failures.
    async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        embedded_hal_async::spi::SpiBus::transfer_in_place(self.spi(), data).await
    }
    /// Awaits idle SPI3 before ExclusiveDevice raises NSS, including on errors.
    async fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_hal_async::spi::SpiBus::flush(self.spi()).await
    }
}

/// Lends system I2C for one control operation and uses Embassy's upstream DelayNs
/// implementation. Once the future completes, touch and PMIC may borrow I2C again.
#[cfg(feature = "c153")]
fn context(i2c: &mut ioe::SysI2c) -> RadioContext<'_, ioe::SysI2c, embassy_time::Delay> {
    RadioContext {
        i2c,
        delay: embassy_time::Delay,
        ioe_address: ioe::active_address(),
    }
}

/// Converts board readback evidence to bounded serial telemetry. A mismatch or bus
/// error marks the card failed; sampled high confirms only the digital control net.
#[cfg(feature = "c153")]
fn report_antenna(check: AntennaCheck) {
    let phase = match check.request.phase {
        CheckPhase::Startup => papermono_log::LoraPhase::Startup,
        CheckPhase::BeforeTx => papermono_log::LoraPhase::BeforeTx,
        CheckPhase::Shutdown => papermono_log::LoraPhase::Shutdown,
    };
    let failure = match check.failure {
        AntennaFailure::None => papermono_log::LoraFailure::None,
        AntennaFailure::Bus => papermono_log::LoraFailure::Bus,
        AntennaFailure::Mismatch => papermono_log::LoraFailure::Mismatch,
    };
    if failure != papermono_log::LoraFailure::None {
        LORA_CONTROL_FAILED.store(true, Ordering::Release);
    }
    crate::cdc::lora_control(&papermono_log::LoraControlSample {
        phase,
        interval: check.request.interval,
        attempts: check.request.attempt,
        checks: check.request.check,
        expected_high: check.expected_high,
        output: check.evidence.output,
        push_pull: check.evidence.push_pull,
        latch: check.evidence.latch,
        level: check.evidence.level,
        failure,
    });
}

/// Emits a typed warning for a control or chip operation whose register evidence
/// is unavailable. The caller always follows this path with explicit cleanup.
#[cfg(feature = "c153")]
fn report_failure(
    sx: &Radio<'_>,
    phase: papermono_log::LoraPhase,
    failure: papermono_log::LoraFailure,
) {
    LORA_CONTROL_FAILED.store(true, Ordering::Release);
    let stats = sx.session_stats();
    crate::cdc::lora_control(&papermono_log::LoraControlSample {
        phase,
        interval: sx.verification_interval().get(),
        attempts: stats.tx_attempts,
        checks: stats.verifications,
        expected_high: phase != papermono_log::LoraPhase::Shutdown,
        output: None,
        push_pull: None,
        latch: None,
        level: None,
        failure,
    });
}

/// Classifies lifecycle errors without discarding the original readback reason.
/// Board bus errors are distinguished from chip SPI/BUSY failures on serial.
#[cfg(feature = "c153")]
fn session_failure<S, H, B>(error: &SessionError<S, H, B>) -> papermono_log::LoraFailure {
    use papermono_log::LoraFailure;
    match error {
        SessionError::TxDenied | SessionError::NotActive | SessionError::RecoveryRequired => {
            LoraFailure::Denied
        }
        SessionError::ReadinessMismatch => LoraFailure::Mismatch,
        SessionError::Verification(_) => LoraFailure::Bus,
        SessionError::Chip(_) => LoraFailure::Chip,
        SessionError::Startup { cause, .. } => match cause {
            StartupFailure::Control(_) => LoraFailure::Control,
            StartupFailure::Chip(_) => LoraFailure::Chip,
            StartupFailure::Verification(_) => LoraFailure::Bus,
            StartupFailure::Mismatch => LoraFailure::Mismatch,
        },
        SessionError::Shutdown(failure) => {
            if failure.control.is_some() {
                LoraFailure::Control
            } else if failure.verification.is_some() {
                LoraFailure::Bus
            } else {
                LoraFailure::Mismatch
            }
        }
    }
}

/// Starts a short diagnostic session at the default verification interval of one.
/// Startup rolls back on failure; the summary preserves its cleanup result. Never
/// cancel this future: power sequencing uses awaited Embassy timers between writes.
#[cfg(feature = "c153")]
async fn start_radio(sx: &mut Radio<'_>, i2c: &mut ioe::SysI2c) -> bool {
    // A failed or interrupted earlier diagnostic may have left the rail enabled.
    // Require confirmed low controls before attempting a new high-state startup.
    if LORA_NEEDS_CLEANUP.load(Ordering::Acquire) && !stop_radio(sx, i2c).await {
        return false;
    }
    LORA_NEEDS_CLEANUP.store(true, Ordering::Release);
    match sx.startup(&mut context(i2c)).await {
        Ok(()) => {
            LORA_CONTROL_FAILED.store(false, Ordering::Release);
            true
        }
        Err(error) => {
            report_failure(
                sx,
                papermono_log::LoraPhase::Startup,
                session_failure(&error),
            );
            let cleanup_ok = matches!(error, SessionError::Startup { cleanup: None, .. });
            LORA_NEEDS_CLEANUP.store(!cleanup_ok, Ordering::Release);
            report_summary(sx, cleanup_ok);
            false
        }
    }
}

/// Emits verification totals, including startup, pre-TX and shutdown checks.
/// Transport/setup failures remain visible in card state even if cleanup succeeds.
#[cfg(feature = "c153")]
fn report_summary(sx: &Radio<'_>, cleanup_ok: bool) {
    let stats = sx.session_stats();
    crate::cdc::lora_session(&papermono_log::LoraSessionSample {
        interval: sx.verification_interval().get(),
        attempts: stats.tx_attempts,
        checks: stats.verifications,
        failures: stats.failures,
        cleanup_ok,
    });
}

/// Explicitly asserts reset, disconnects antenna and disables the rail, attempting
/// all cleanup steps. Disabled-state confirmation runs even after a control error.
/// No packet, channel or standby transition invokes these lifecycle controls.
#[cfg(feature = "c153")]
async fn stop_radio(sx: &mut Radio<'_>, i2c: &mut ioe::SysI2c) -> bool {
    let result = sx.shutdown(&mut context(i2c)).await;
    let ok = result.is_ok();
    LORA_NEEDS_CLEANUP.store(!ok, Ordering::Release);
    if let Err(error) = result {
        report_failure(
            sx,
            papermono_log::LoraPhase::Shutdown,
            session_failure(&error),
        );
    }
    report_summary(sx, ok);
    ok
}

/// Executes an initial non-destructive probe and immediately parks the transceiver.
#[cfg(feature = "c153")]
pub async fn probe_and_park(i2c: &mut ioe::SysI2c) -> bool {
    let mut lock = LORA_HW.lock().await;
    let Some(hw) = lock.as_mut() else {
        init_status(false);
        return false;
    };

    let mut sx = session(hw);
    if !start_radio(&mut sx, i2c).await {
        return false;
    }
    let status_res = sx.get_status().await;

    let success = match status_res {
        Ok(status) if status.is_ok() => {
            let raw = status.raw;
            LORA_RAW_STATUS.store(raw, Ordering::Release);
            init_status(true);
            if sx.stop_rx().await.is_err() {
                report_failure(
                    &sx,
                    papermono_log::LoraPhase::Shutdown,
                    papermono_log::LoraFailure::Chip,
                );
            }
            true
        }
        _ => {
            report_failure(
                &sx,
                papermono_log::LoraPhase::Startup,
                papermono_log::LoraFailure::Chip,
            );
            init_status(false);
            false
        }
    };

    let cleanup_ok = stop_radio(&mut sx, i2c).await;
    success && cleanup_ok
}

/// Fallback probe on Lite SKU.
#[cfg(not(feature = "c153"))]
pub async fn probe_and_park(_i2c: &mut crate::ioe::SysI2c) -> bool {
    init_status(false);
    false
}

/// Transmits a user-controlled test ping at 915.000 MHz with the retained PA profile.
///
/// # RF Safety & Sequencing
/// 1. Powers up `3V3_L2_LoRa` rail via M5PM1 `G2`, connects FPC antenna via M5IOE1 `PYG2`,
///    and releases reset via M5IOE1 `PYG10`.
/// 2. Configures the transceiver in Standby RC (Semtech SX1262 Section 13.1.2 "SetStandby"),
///    LDO regulator mode (Section 13.1.11 "SetRegulatorMode", matching `hal_lora.cpp`),
///    3.0 V TCXO supply (Section 13.3.6 "SetDIO3AsTCXOCtrl"), and automatic RF switch
///    via DIO2 (Section 13.3.5 "SetDIO2AsRfSwitchCtrl").
/// 3. Configures carrier frequency to 915.000 MHz (Section 13.4.1 "SetRfFrequency") and
///    retains the +14 power command and 60 mA OCP register setting
///    (Section 13.1.14 "SetPaConfig", Section 13.4.4 "SetTxParams", Section 12.1 “Registers” (OCP)).
/// 4. Loads payload into the transceiver FIFO (Section 13.2.3 "WriteBuffer") and
///    triggers transmission (Section 13.1.4 "SetTx").
/// 5. Polls for `IRQ_TX_DONE` (Section 13.3.3 "GetIrqStatus"), then immediately returns
///    to Standby RC and parks the hardware rails via explicit shutdown.
#[cfg(feature = "c153")]
pub async fn transmit_ping(i2c: &mut ioe::SysI2c) -> Option<papermono_log::LoraTxSample> {
    let mut lock = LORA_HW.lock().await;
    let hw = lock.as_mut()?;

    let mut sx = session(hw);
    if !start_radio(&mut sx, i2c).await {
        return None;
    }

    let setup = async {
        // 1. Enter Standby RC:
        sx.set_standby(STDBY_CONFIG_RC).await?;
        sx.set_regulator_mode(REGULATOR_LDO).await?;
        sx.set_dio3_as_tcxo_ctrl(TCXO_CTRL_3_0V, TCXO_DEFAULT_DELAY_TICKS)
            .await?;
        sx.calibrate_image(CAL_IMG_902_MHZ, CAL_IMG_928_MHZ).await?;
        sx.set_dio2_as_rf_switch_ctrl(true).await?;

        // 2. Configure LoRa modem on 915.000 MHz:
        sx.set_packet_type(PACKET_TYPE_LORA).await?;
        sx.set_rf_frequency(FREQ_BENCH_PING_HZ).await?;

        // 3. Retained PA profile, +14 power command and 60 mA OCP code:
        sx.set_pa_config(
            PA_DUTY_CYCLE_14DBM,
            PA_HP_MAX_14DBM,
            PA_DEVICE_SEL_SX1262,
            PA_LUT_DEFAULT,
        )
        .await?;
        sx.set_tx_params(14, RAMP_40_US).await?;
        sx.set_ocp(OCP_60_MA).await?;
        sx.configure_tx_clamp().await?;

        // 4. Modulation & Packet format:
        sx.set_lora_modulation_params(LORA_SF7, LORA_BW_125_KHZ, LORA_CR_4_5, LORA_LDRO_OFF)
            .await?;
        let payload = b"PAPEBENCH-PING#001";
        sx.set_lora_packet_params(
            8,
            LORA_HEADER_VARIABLE,
            payload.len() as u8,
            LORA_CRC_ON,
            LORA_IQ_STANDARD,
        )
        .await?;
        sx.set_lora_sync_word(SYNC_WORD_PRIVATE).await?;

        // 5. Load FIFO buffer:
        sx.set_buffer_base_address(0x00, 0x00).await?;
        sx.write_buffer(0x00, payload).await?;

        // 6. Arm TX IRQ:
        sx.clear_irq_status(IRQ_ALL).await?;
        sx.set_dio_irq_params(IRQ_TX_DONE | IRQ_TIMEOUT, IRQ_TX_DONE, 0, 0)
            .await?;

        Ok::<(), ChipError>(())
    }
    .await;
    if setup.is_err() {
        report_failure(
            &sx,
            papermono_log::LoraPhase::BeforeTx,
            papermono_log::LoraFailure::Chip,
        );
        stop_radio(&mut sx, i2c).await;
        return None;
    }

    // 7. Initiate single burst transmission:
    let start = Instant::now();
    if let Err(error) = sx
        .set_tx(&mut context(i2c), TX_COMPLETION_MS * RTC_TICKS_PER_MS)
        .await
    {
        let reason = session_failure(&error);
        report_failure(&sx, papermono_log::LoraPhase::BeforeTx, reason);
        stop_radio(&mut sx, i2c).await;
        return None;
    } // The same 300 ms budget bounds both chip TX and completion polling.

    let mut confirmed = false;
    let mut failure = papermono_log::LoraFailure::Timeout;
    while start.elapsed() < Duration::from_millis(u64::from(TX_COMPLETION_MS)) {
        Timer::after(Duration::from_millis(5)).await;
        match sx.get_irq_status().await {
            Ok(irq) if irq & IRQ_TIMEOUT != 0 => {
                break;
            }
            Ok(irq) if irq & IRQ_TX_DONE != 0 => {
                confirmed = true;
                break;
            }
            Ok(_) => {}
            Err(_) => {
                failure = papermono_log::LoraFailure::Chip;
                break;
            }
        }
    }
    if !confirmed {
        report_failure(&sx, papermono_log::LoraPhase::BeforeTx, failure);
    }
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis() as u32;

    // 8. Immediately return to standby and park the hardware rails:
    let cleanup_ok = finish_radio(&mut sx, i2c).await;

    let sample = papermono_log::LoraTxSample {
        freq_khz: FREQ_BENCH_PING_HZ / 1_000,
        pwr_dbm: 14,
        sf: 7,
        bw_khz: 125,
        time_ms: elapsed_ms,
        ok: confirmed && cleanup_ok,
    };

    LORA_FREQ_KHZ.store(sample.freq_khz, Ordering::Release);
    LORA_TX_PWR.store(sample.pwr_dbm, Ordering::Release);
    LORA_TIME_MS.store(sample.time_ms, Ordering::Release);
    LORA_TX_OK.store(sample.ok, Ordering::Release);
    LORA_STATE_CODE.store(2, Ordering::Release);

    Some(sample)
}

/// Fallback transmit on Lite SKU.
#[cfg(not(feature = "c153"))]
pub async fn transmit_ping(_i2c: &mut crate::ioe::SysI2c) -> Option<papermono_log::LoraTxSample> {
    None
}

/// Listens for incoming LoRa packets on the sniffer frequency for up to 60 seconds,
/// or until a stop request from a button press or a latched falling edge on the
/// active-low FT6336G interrupt (`GPIO4`). Records measured ambient noise when
/// quiet; returns an error on chip/control failure, with CDC warning and cleanup.
/// Stop requests are observed between awaited sequences; the touch edge is
/// latched by ESP-HAL so a short tap is not lost between radio polls.
///
/// # Hardware Sequencing
/// 1. Engages antenna via M5IOE1 `PYG2`, powers module via M5PM1 `G2`, and releases reset.
/// 2. Configures receiver parameters: Semtech SX1262 Section 13.1.5 "SetRx" (`0xFFFFFF`
///    for continuous reception), Section 13.4.5 "SetModulationParams" (SF11, BW 250 kHz,
///    CR 4/5), Section 13.4.6 "SetPacketParams" (variable header, 255-byte max buffer, CRC on),
///    and Section 12.1 “Registers” (LoRa Sync Word) (`0x24B4`).
/// 3. Arms `IRQ_RX_DONE` on DIO1 (Section 13.3.1 "SetDioIrqParams").
/// 4. Polls for packet arrival. Upon reception, reads packet length and buffer start pointer
///    (Section 13.5.2 "GetRxBufferStatus"), queries packet RSSI and SNR (Section 13.5.3
///    "GetPacketStatus"), reads payload bytes (Section 13.2.4 "ReadBuffer"), and immediately
///    returns to standby and powers down.
#[cfg(feature = "c153")]
pub async fn listen_rx(
    i2c: &mut ioe::SysI2c,
    btn_a: &esp_hal::gpio::Input<'static>,
    btn_b: &esp_hal::gpio::Input<'static>,
    tp: &mut esp_hal::gpio::Input<'static>,
) -> Result<Option<papermono_log::LoraRxSample>, ()> {
    let mut lock = LORA_HW.lock().await;
    let hw = lock.as_mut().ok_or(())?;

    // Alternate sniffer frequency on consecutive taps:
    let slot = LORA_SNIFFER_SLOT.fetch_xor(1, Ordering::Relaxed);
    let target_freq = if slot == 0 {
        FREQ_RX_SNIFFER_PRI_HZ
    } else {
        FREQ_RX_SNIFFER_SEC_HZ
    };

    set_listening_state(target_freq / 1_000);

    let mut sx = session(hw);
    if !start_radio(&mut sx, i2c).await {
        return Err(());
    }

    let setup = async {
        // 1. Enter Standby RC:
        sx.set_standby(STDBY_CONFIG_RC).await?;
        sx.set_regulator_mode(REGULATOR_LDO).await?;
        sx.set_dio3_as_tcxo_ctrl(TCXO_CTRL_3_0V, TCXO_DEFAULT_DELAY_TICKS)
            .await?;
        sx.calibrate_image(CAL_IMG_902_MHZ, CAL_IMG_928_MHZ).await?;
        sx.set_dio2_as_rf_switch_ctrl(true).await?;

        // 2. Configure receiver parameters: SF11, BW 250 kHz (Meshtastic LongFast standard)
        sx.set_packet_type(PACKET_TYPE_LORA).await?;
        sx.set_rf_frequency(target_freq).await?;
        sx.set_lora_modulation_params(LORA_SF11, LORA_BW_250_KHZ, LORA_CR_4_5, LORA_LDRO_OFF)
            .await?;
        sx.set_lora_packet_params(16, LORA_HEADER_VARIABLE, 255, LORA_CRC_ON, LORA_IQ_STANDARD)
            .await?;
        sx.set_lora_sync_word(SYNC_WORD_ALT).await?;

        // 3. Clear IRQs and start continuous reception (0xFFFFFF = Rx Continuous):
        sx.clear_irq_status(IRQ_ALL).await?;
        sx.set_dio_irq_params(
            IRQ_RX_DONE | IRQ_TIMEOUT | IRQ_CRC_ERR | lora::IRQ_HEADER_ERR,
            IRQ_RX_DONE,
            0,
            0,
        )
        .await?;
        sx.set_buffer_base_address(0x00, 0x00).await?;
        sx.set_rx(0xFFFFFF).await?;

        Ok::<(), ChipError>(())
    }
    .await;
    if setup.is_err() {
        report_failure(
            &sx,
            papermono_log::LoraPhase::Startup,
            papermono_log::LoraFailure::Chip,
        );
        stop_radio(&mut sx, i2c).await;
        return Err(());
    }

    // 4. Debounce initial tap: wait until finger is lifted off glass (up to 400 ms)
    for _ in 0..20 {
        if tp.is_high() {
            break;
        }
        Timer::after(Duration::from_millis(20)).await;
    }

    // The UI task awaits this bounded session, so it cannot poll FT6336G touch
    // coordinates while LoRa owns the radio. Latch the active-low GPIO4 edge
    // instead; this needs no system-I2C transaction during reception.
    tp.clear_interrupt();
    tp.listen(esp_hal::gpio::Event::FallingEdge);

    // 5. Polling window (up to 60 seconds: 1200 x 50 ms):
    let mut received = None;
    let mut preview_buf = [0; 4];
    for _ in 0..1200 {
        Timer::after(Duration::from_millis(50)).await;
        if btn_a.is_low() || btn_b.is_low() || tp.is_low() || tp.is_interrupt_set() {
            break;
        }
        match sx.poll_receive(&mut preview_buf).await {
            Ok(ReceivePoll::Packet { len, status, .. }) => {
                received = Some((len, status));
                break;
            }
            Ok(ReceivePoll::Pending | ReceivePoll::Rejected { .. }) => {}
            Ok(ReceivePoll::Timeout { .. }) | Err(_) => {
                tp.unlisten();
                tp.clear_interrupt();
                report_failure(
                    &sx,
                    papermono_log::LoraPhase::Startup,
                    papermono_log::LoraFailure::Chip,
                );
                stop_radio(&mut sx, i2c).await;
                return Err(());
            }
        }
    }
    tp.unlisten();
    tp.clear_interrupt();

    if let Some((len, pkt_status)) = received {
        crate::beep::tone(80);
        let sample = packet_sample(target_freq, len, pkt_status, preview_buf);
        let cleanup_ok = finish_radio(&mut sx, i2c).await;

        LORA_FREQ_KHZ.store(sample.freq_khz, Ordering::Release);
        LORA_RSSI.store(sample.rssi, Ordering::Release);
        LORA_SNR.store(sample.snr, Ordering::Release);
        LORA_LEN.store(sample.len, Ordering::Release);
        LORA_PREVIEW[0].store(preview_buf[0], Ordering::Release);
        LORA_PREVIEW[1].store(preview_buf[1], Ordering::Release);
        LORA_PREVIEW[2].store(preview_buf[2], Ordering::Release);
        LORA_PREVIEW[3].store(preview_buf[3], Ordering::Release);
        LORA_STATE_CODE.store(3, Ordering::Release);

        if cleanup_ok {
            Ok(Some(sample))
        } else {
            Err(())
        }
    } else {
        // A failed signal read is an operation error, never an ambient value.
        let ambient = match sx.get_rssi_inst().await {
            Ok(value) => value,
            Err(_) => {
                report_failure(
                    &sx,
                    papermono_log::LoraPhase::Startup,
                    papermono_log::LoraFailure::Chip,
                );
                stop_radio(&mut sx, i2c).await;
                return Err(());
            }
        };
        let cleanup_ok = finish_radio(&mut sx, i2c).await;

        LORA_FREQ_KHZ.store(target_freq / 1_000, Ordering::Release);
        LORA_RSSI.store(ambient, Ordering::Release);
        LORA_STATE_CODE.store(4, Ordering::Release);

        if cleanup_ok {
            Ok(None)
        } else {
            Err(())
        }
    }
}

/// Fallback listen on Lite SKU.
#[cfg(not(feature = "c153"))]
pub async fn listen_rx(
    _i2c: &mut crate::ioe::SysI2c,
    _btn_a: &esp_hal::gpio::Input<'static>,
    _btn_b: &esp_hal::gpio::Input<'static>,
    _tp: &mut esp_hal::gpio::Input<'static>,
) -> Result<Option<papermono_log::LoraRxSample>, ()> {
    Err(())
}

/// Executes a full sweep across the US915 band (104 channels, 902.125 MHz to 927.875 MHz).
///
/// # Hardware Operation & Double-Duty Scanning
/// - Applies "double duty" scanning to the expected channel neighborhood (slots 61..=63),
///   visiting them twice per sweep and doubling their dwell time (25 ms vs 10 ms).
/// - Queries instantaneous RSSI (Semtech SX1262 Section 13.5.4 "GetRssiInst").
/// - Emits acoustic feedback via GPIO42 passive buzzer: 25 ms chirp on elevated RSSI
///   (above -105 dBm) and 80 ms tone on packet detection.
/// - Parks transceiver and powers down rails after completion, a stop request,
///   or an operation failure. Await each SPI sequence before observing a stop.
#[cfg(feature = "c153")]
pub async fn run_scan_sweep(
    i2c: &mut ioe::SysI2c,
    btn_a: &esp_hal::gpio::Input<'static>,
    btn_b: &esp_hal::gpio::Input<'static>,
    tp: &esp_hal::gpio::Input<'static>,
) -> bool {
    SCAN_STOP_REQUESTED.store(false, Ordering::Release);
    SCAN_DATA.lock(|cell| {
        cell.borrow_mut().scanning = true;
    });

    let mut lock = LORA_HW.lock().await;
    let hw = match lock.as_mut() {
        Some(h) => h,
        None => {
            SCAN_DATA.lock(|cell| {
                cell.borrow_mut().scanning = false;
            });
            return false;
        }
    };

    let mut sx = session(hw);
    if !start_radio(&mut sx, i2c).await {
        SCAN_DATA.lock(|cell| cell.borrow_mut().scanning = false);
        return false;
    }

    let setup = async {
        // Initial transceiver configuration:
        sx.set_standby(STDBY_CONFIG_RC).await?;
        sx.set_regulator_mode(REGULATOR_LDO).await?;
        sx.set_dio3_as_tcxo_ctrl(TCXO_CTRL_3_0V, TCXO_DEFAULT_DELAY_TICKS)
            .await?;
        sx.calibrate_image(CAL_IMG_902_MHZ, CAL_IMG_928_MHZ).await?;
        sx.set_dio2_as_rf_switch_ctrl(true).await?;
        sx.set_packet_type(PACKET_TYPE_LORA).await?;
        sx.set_lora_modulation_params(LORA_SF11, LORA_BW_250_KHZ, LORA_CR_4_5, LORA_LDRO_OFF)
            .await?;
        sx.set_lora_packet_params(16, LORA_HEADER_VARIABLE, 255, LORA_CRC_ON, LORA_IQ_STANDARD)
            .await?;
        sx.set_lora_sync_word(SYNC_WORD_MESHTASTIC).await?;

        Ok::<(), ChipError>(())
    }
    .await;
    if setup.is_err() {
        report_failure(
            &sx,
            papermono_log::LoraPhase::Startup,
            papermono_log::LoraFailure::Chip,
        );
        stop_radio(&mut sx, i2c).await;
        SCAN_DATA.lock(|cell| cell.borrow_mut().scanning = false);
        return false;
    }

    // Double-duty sweep sequence: visit slots 0..=51, then revisit expected slots (61..=63),
    // then continue 52..=103 (which naturally covers 61..=63 again).
    let mut sweep_slots = [0u8; 107];
    let mut idx = 0;
    for s in 0..=51u8 {
        sweep_slots[idx] = s;
        idx += 1;
    }
    for s in 61..=63u8 {
        sweep_slots[idx] = s;
        idx += 1;
    }
    for s in 52..=103u8 {
        sweep_slots[idx] = s;
        idx += 1;
    }

    let mut stopped = false;
    for &slot in &sweep_slots {
        if SCAN_STOP_REQUESTED.load(Ordering::Acquire)
            || btn_a.is_low()
            || btn_b.is_low()
            || tp.is_low()
            || tp.is_interrupt_set()
        {
            stopped = true;
            break;
        }

        let freq_hz = m5stack_papermono::lora::us915_channel_freq_hz(slot);
        let channel = async {
            // Standby changes only the chip mode; PYG2 stays high for this sweep.
            sx.set_standby(STDBY_CONFIG_RC).await?;
            sx.set_rf_frequency(freq_hz).await?;
            sx.clear_irq_status(IRQ_ALL).await?;
            sx.set_dio_irq_params(
                IRQ_RX_DONE | IRQ_TIMEOUT | IRQ_CRC_ERR | lora::IRQ_HEADER_ERR,
                IRQ_RX_DONE,
                0,
                0,
            )
            .await?;
            sx.set_buffer_base_address(0x00, 0x00).await?;
            sx.set_rx(0xFFFFFF).await?;
            Ok::<(), ChipError>(())
        }
        .await;
        if channel.is_err() {
            report_failure(
                &sx,
                papermono_log::LoraPhase::Startup,
                papermono_log::LoraFailure::Chip,
            );
            stopped = true;
            break;
        }

        // Double dwell time for expected channel neighborhood (slots 61..=63 and slot 19):
        let is_expected = (61..=63).contains(&slot) || slot == 19;
        let dwell_ms = if is_expected { 25 } else { 10 };
        Timer::after(Duration::from_millis(dwell_ms)).await;

        let mut preview = [0; 4];
        let readings = async {
            let rssi = sx.get_rssi_inst().await?;
            let packet = sx.poll_receive(&mut preview).await?;
            Ok::<_, ChipError>((rssi, packet))
        }
        .await;
        let (rssi, packet) = match readings {
            Ok(value) => value,
            Err(_) => {
                report_failure(
                    &sx,
                    papermono_log::LoraPhase::Startup,
                    papermono_log::LoraFailure::Chip,
                );
                stopped = true;
                break;
            }
        };
        let mut hit = false;
        let mut packet_received = false;
        let mut sample_opt = None;
        if let ReceivePoll::Packet { len, status, .. } = packet {
            hit = true;
            packet_received = true;
            sample_opt = Some(packet_sample(freq_hz, len, status, preview));
        }
        if matches!(packet, ReceivePoll::Timeout { .. }) {
            report_failure(
                &sx,
                papermono_log::LoraPhase::Startup,
                papermono_log::LoraFailure::Chip,
            );
            stopped = true;
            break;
        }

        if rssi > -105 {
            hit = true;
        }

        // Acoustic beep when activity or packet is heard:
        if packet_received {
            crate::beep::tone(80);
        } else if hit {
            crate::beep::tone(25);
        }

        SCAN_DATA.lock(|cell| {
            let mut data = cell.borrow_mut();
            data.current_slot = slot;
            if rssi > data.peak_rssi {
                data.peak_rssi = rssi;
                data.peak_slot = slot;
            }
            if hit {
                data.hits[slot as usize] = data.hits[slot as usize].saturating_add(1);
            }
            if packet_received {
                data.total_packets = data.total_packets.saturating_add(1);
            }
        });

        if let Some(sample) = sample_opt {
            crate::cdc::lora_rx(&sample);
        }

        if hit {
            crate::cdc::lora_scan(&papermono_log::LoraScanSample {
                slot,
                freq_khz: freq_hz / 1_000,
                rssi,
                packets: if packet_received { 1 } else { 0 },
            });
        }

        Timer::after(Duration::from_millis(2)).await;
    }

    let cleanup_ok = finish_radio(&mut sx, i2c).await;

    stopped |= !cleanup_ok;
    SCAN_DATA.lock(|cell| {
        let mut data = cell.borrow_mut();
        data.scanning = !stopped;
        if !stopped {
            data.sweeps = data.sweeps.saturating_add(1);
        }
    });

    if !stopped {
        let (slot, rssi, packets) = SCAN_DATA.lock(|cell| {
            let data = cell.borrow();
            (data.peak_slot, data.peak_rssi, data.total_packets)
        });
        let freq_hz = m5stack_papermono::lora::us915_channel_freq_hz(slot);
        crate::cdc::lora_scan(&papermono_log::LoraScanSample {
            slot,
            freq_khz: freq_hz / 1_000,
            rssi,
            packets,
        });
    }

    !stopped
}

/// Fallback scan sweep on Lite SKU.
#[cfg(not(feature = "c153"))]
pub async fn run_scan_sweep(
    _i2c: &mut crate::ioe::SysI2c,
    _btn_a: &esp_hal::gpio::Input<'static>,
    _btn_b: &esp_hal::gpio::Input<'static>,
    _tp: &esp_hal::gpio::Input<'static>,
) -> bool {
    false
}

/// Existing ping completion limit, now also programmed into the hardware timer.
/// Catalog `sx1262` §13.1.4 “SetTx”; 300 ms bounds this diagnostic.
#[cfg(feature = "c153")]
const TX_COMPLETION_MS: u32 = 300;
/// RTC ticks per millisecond (15.625 us each), same SetTx section.
#[cfg(feature = "c153")]
const RTC_TICKS_PER_MS: u32 = 64;

/// Stops chip reception and then shuts down board controls even if chip I/O
/// fails. The return combines both results, and failures remain visible on CDC.
#[cfg(feature = "c153")]
async fn finish_radio(sx: &mut Radio<'_>, i2c: &mut ioe::SysI2c) -> bool {
    let chip_ok = sx.stop_rx().await.is_ok();
    if !chip_ok {
        report_failure(
            sx,
            papermono_log::LoraPhase::Shutdown,
            papermono_log::LoraFailure::Chip,
        );
    }
    let cleanup_ok = stop_radio(sx, i2c).await;
    chip_ok && cleanup_ok
}

/// Emits lossless half-dBm/quarter-dB metrics, then builds the existing whole-unit
/// RX record. Preview bytes beyond the chip length stay zero in the stack buffer.
#[cfg(feature = "c153")]
fn packet_sample(
    freq: u32,
    len: u8,
    status: PacketStatus,
    preview: [u8; 4],
) -> papermono_log::LoraRxSample {
    crate::cdc::lora_metrics(&papermono_log::LoraMetricsSample {
        freq_khz: freq / 1_000,
        rssi_half_dbm: status.rssi_pkt_half_dbm,
        snr_quarter_db: status.snr_pkt_quarter_db,
        signal_rssi_half_dbm: status.signal_rssi_pkt_half_dbm,
    });
    papermono_log::LoraRxSample {
        freq_khz: freq / 1_000,
        rssi: status.rssi_pkt_dbm(),
        snr: status.snr_pkt_db(),
        len,
        first_byte: preview[0],
        last_byte: preview[3.min(usize::from(len).saturating_sub(1))],
    }
}
