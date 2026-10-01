//! Host model for session policy, transport faults and upstream command traffic.
use embedded_hal::{digital, spi};
use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{Context, Poll, Waker},
};
use sx1262_phy::*;

#[derive(Default)]
struct Wire {
    selected: bool,
    bytes: Vec<u8>,
    frames: Vec<Vec<u8>>,
    frame: Vec<u8>,
    busy: bool,
    pin_error: bool,
    select_error: bool,
    deselect_error: bool,
    spi_error: bool,
    flush_error: bool,
    flushes: u32,
    returned: Vec<u8>,
    delays: Vec<u32>,
    busy_samples: Vec<bool>,
    samples: usize,
    pending_spi: bool,
    fail_command: Option<u8>,
}
#[derive(Clone)]
struct Bus(Rc<RefCell<Wire>>);
impl spi::ErrorType for Bus {
    type Error = spi::ErrorKind;
}
impl embedded_hal_async::spi::SpiBus for Bus {
    async fn read(&mut self, out: &mut [u8]) -> Result<(), Self::Error> {
        yield_once().await;
        let mut wire = self.0.borrow_mut();
        if wire.spi_error {
            return Err(spi::ErrorKind::Other);
        }
        for byte in out {
            *byte = if wire.returned.is_empty() {
                0
            } else {
                wire.returned.remove(0)
            };
        }
        Ok(())
    }
    async fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        yield_once().await;
        if self.0.borrow().pending_spi {
            std::future::pending::<()>().await;
        }
        let mut wire = self.0.borrow_mut();
        assert!(wire.selected);
        if bytes.first() == wire.fail_command.as_ref() && wire.fail_command.is_some() {
            return Err(spi::ErrorKind::Overrun);
        }
        if wire.spi_error {
            return Err(spi::ErrorKind::Other);
        }
        wire.bytes.extend(bytes);
        wire.frame.extend(bytes);
        Ok(())
    }
    async fn transfer(&mut self, out: &mut [u8], input: &[u8]) -> Result<(), Self::Error> {
        self.write(input).await?;
        self.read(out).await
    }
    async fn transfer_in_place(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.write(bytes).await?;
        self.read(bytes).await
    }
    async fn flush(&mut self) -> Result<(), Self::Error> {
        let mut wire = self.0.borrow_mut();
        wire.flushes += 1;
        if wire.flush_error {
            Err(spi::ErrorKind::Other)
        } else {
            Ok(())
        }
    }
}
#[derive(Clone)]
struct Pin(Rc<RefCell<Wire>>);
impl digital::ErrorType for Pin {
    type Error = digital::ErrorKind;
}
impl digital::OutputPin for Pin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let mut wire = self.0.borrow_mut();
        if wire.pin_error {
            return Err(digital::ErrorKind::Other);
        }
        wire.selected = true;
        if wire.select_error {
            return Err(digital::ErrorKind::Other);
        }
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        let mut wire = self.0.borrow_mut();
        wire.selected = false;
        let frame = std::mem::take(&mut wire.frame);
        wire.frames.push(frame);
        if wire.deselect_error {
            Err(digital::ErrorKind::Other)
        } else {
            Ok(())
        }
    }
}
impl digital::InputPin for Pin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let mut wire = self.0.borrow_mut();
        wire.samples += 1;
        if wire.pin_error {
            Err(digital::ErrorKind::Other)
        } else {
            Ok(if wire.busy_samples.is_empty() {
                wire.busy
            } else {
                wire.busy_samples.remove(0)
            })
        }
    }
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|v| !v)
    }
}
fn run<T>(future: impl Future<Output = T>) -> T {
    let mut future = std::pin::pin!(future);
    for _ in 0..10_000 {
        if let Poll::Ready(result) = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            return result;
        }
    }
    panic!("future did not complete within mock executor budget");
}
#[derive(Default)]
struct Control {
    antenna: bool,
    events: Vec<CheckPhase>,
    starts: u32,
    stops: u32,
    unrelated: u32,
}
#[derive(Default)]
struct Policy {
    deny: bool,
    fail_start: bool,
    fail_stop: bool,
    mismatch: Option<CheckPhase>,
    fail_check: Option<CheckPhase>,
    pending_check: Option<CheckPhase>,
}
impl Hooks<Control> for Policy {
    type Error = &'static str;
    fn tx_allowed(&self) -> bool {
        !self.deny
    }
    async fn startup(&mut self, c: &mut Control) -> Result<(), Self::Error> {
        c.starts += 1;
        c.antenna = true;
        if self.fail_start {
            Err("startup")
        } else {
            Ok(())
        }
    }
    async fn shutdown(&mut self, c: &mut Control) -> Result<(), Self::Error> {
        c.stops += 1;
        c.antenna = false;
        if self.fail_stop {
            Err("shutdown")
        } else {
            Ok(())
        }
    }
    async fn verify(
        &mut self,
        c: &mut Control,
        request: CheckRequest,
    ) -> Result<bool, Self::Error> {
        c.events.push(request.phase);
        if self.pending_check == Some(request.phase) {
            std::future::pending::<()>().await;
        }
        if self.fail_check == Some(request.phase) {
            return Err("verification");
        }
        Ok(self.mismatch != Some(request.phase)
            && c.antenna == (request.phase != CheckPhase::Shutdown))
    }
}
#[derive(Clone)]
struct Delay(Rc<RefCell<Wire>>);
impl embedded_hal_async::delay::DelayNs for Delay {
    async fn delay_ns(&mut self, ns: u32) {
        self.0.borrow_mut().delays.push(ns);
        yield_once().await;
    }
}
async fn yield_once() {
    let mut yielded = false;
    std::future::poll_fn(|cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await
}
type Device = embedded_hal_bus::spi::ExclusiveDevice<Bus, Pin, Delay>;
fn radio<H>(hooks: H) -> (Sx1262<Device, Pin, Delay, H>, Rc<RefCell<Wire>>) {
    let wire = Rc::new(RefCell::new(Wire::default()));
    let device = embedded_hal_bus::spi::ExclusiveDevice::new(
        Bus(wire.clone()),
        Pin(wire.clone()),
        Delay(wire.clone()),
    )
    .unwrap();
    wire.borrow_mut().frames.clear();
    (
        Sx1262::with_hooks(device, Pin(wire.clone()), Delay(wire.clone()), hooks),
        wire,
    )
}

#[test]
fn short_and_sustained_sessions_preserve_antenna_and_borrowed_context() {
    for packets in [1, 81] {
        let (mut sx, wire) = radio(Policy::default());
        let mut context = Control::default();
        run(sx.startup(&mut context)).unwrap();
        for i in 0..packets {
            run(sx.set_rf_frequency(915_000_000 + i * 1000)).unwrap();
            run(sx.set_standby(STDBY_CONFIG_RC)).unwrap();
            run(sx.set_rx(0xFF_FFFF)).unwrap();
            run(sx.write_buffer(0, b"ping")).unwrap();
            context.unrelated += 1;
            run(sx.set_tx(&mut context, 1)).unwrap();
            let before = sx.session_stats();
            run(sx.startup(&mut context)).unwrap();
            assert_eq!(sx.session_stats(), before);
            assert!(context.antenna);
        }
        run(sx.shutdown(&mut context)).unwrap();
        assert_eq!((context.starts, context.stops), (1, 1));
        assert!(!context.antenna);
        assert_eq!(sx.session_stats().verifications, u64::from(packets) + 2);
        assert_eq!(
            wire.borrow()
                .frames
                .iter()
                .filter(|f| f.first() == Some(&CMD_SET_TX))
                .count(),
            packets as usize
        );
        let (_, _, _, hooks) = sx.release();
        assert!(!hooks.deny);
    }
}

#[test]
fn schedules_and_interval_changes_reset_only_on_deliberate_change() {
    for interval in [1, 20, 40] {
        let (mut sx, _) = radio(Policy::default());
        let mut c = Control::default();
        sx.set_verification_interval(std::num::NonZeroU32::new(interval).unwrap());
        run(sx.startup(&mut c)).unwrap();
        for _ in 0..81 {
            run(sx.set_tx(&mut c, 1)).unwrap();
        }
        assert_eq!(
            sx.session_stats().verifications,
            1 + 1 + 80 / u64::from(interval)
        );
        let before = sx.session_stats();
        sx.set_verification_interval(sx.verification_interval());
        assert_eq!(before, sx.session_stats());
        sx.set_verification_interval(std::num::NonZeroU32::new(interval + 1).unwrap());
        assert_eq!(sx.session_stats(), SessionStats::new());
        run(sx.set_tx(&mut c, 1)).unwrap();
        assert_eq!(sx.session_stats().verifications, 1);
        run(sx.shutdown(&mut c)).unwrap();
        run(sx.startup(&mut c)).unwrap();
        assert_eq!(sx.session_stats().tx_attempts, 0);
    }
}

#[test]
fn default_denied_permanent_policy_and_raw_guard() {
    let (mut sx, wire) = radio(DenyTx);
    assert_eq!(run(sx.set_tx(&mut (), 1)), Err(SessionError::NotActive));
    run(sx.startup(&mut ())).unwrap();
    assert_eq!(run(sx.set_tx(&mut (), 1)), Err(SessionError::TxDenied));
    assert!(wire.borrow().bytes.is_empty());
    assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    let (mut sx, wire) = radio(AlwaysConnected);
    run(sx.startup(&mut ())).unwrap();
    assert_eq!(
        run(sx.write_cmd(CMD_SET_TX, &[0, 0, 1])),
        Err(SxError::ContextRequired)
    );
    for command in [0xD1, 0xD2, 0xFF, CMD_SET_SLEEP, CMD_SET_RX_DUTY_CYCLE] {
        assert_eq!(
            run(sx.write_cmd_with_context(&mut (), command, &[])),
            Err(SessionError::Chip(SxError::UnsupportedCommand))
        );
    }
    run(sx.write_cmd_with_context(&mut (), CMD_SET_TX, &[0, 0, 1])).unwrap();
    assert_eq!(wire.borrow().frames, vec![vec![CMD_SET_TX, 0, 0, 1]]);
}

#[test]
fn failures_invalidate_readiness_and_recovery_requires_lifecycle_pair() {
    for bus_failure in [false, true] {
        let hooks = Policy {
            mismatch: (!bus_failure).then_some(CheckPhase::BeforeTx),
            fail_check: bus_failure.then_some(CheckPhase::BeforeTx),
            ..Policy::default()
        };
        let (mut sx, wire) = radio(hooks);
        let mut c = Control::default();
        sx.set_verification_interval(std::num::NonZeroU32::new(40).unwrap());
        run(sx.startup(&mut c)).unwrap();
        let result = run(sx.write_cmd_with_context(&mut c, CMD_SET_TX, &[0, 0, 1]));
        assert_eq!(
            result,
            Err(if bus_failure {
                SessionError::Verification("verification")
            } else {
                SessionError::ReadinessMismatch
            })
        );
        assert!(wire.borrow().bytes.is_empty());
        assert_eq!(sx.verification_interval().get(), 1);
        assert_eq!(run(sx.startup(&mut c)), Err(SessionError::RecoveryRequired));
        assert_eq!(
            run(sx.set_tx(&mut c, 1)),
            Err(SessionError::RecoveryRequired)
        );
        run(sx.shutdown(&mut c)).unwrap();
        run(sx.startup(&mut c)).unwrap();
        assert_eq!(sx.session_state(), SessionState::Active);
    }
}

#[test]
fn startup_rollback_and_shutdown_keep_both_errors_and_always_verify() {
    let (mut sx, _) = radio(Policy {
        fail_start: true,
        fail_stop: true,
        fail_check: Some(CheckPhase::Shutdown),
        ..Policy::default()
    });
    let mut c = Control::default();
    let error = run(sx.startup(&mut c)).unwrap_err();
    assert!(matches!(
        error,
        SessionError::Startup {
            cause: StartupFailure::Control("startup"),
            cleanup: Some(ShutdownFailure {
                control: Some("shutdown"),
                verification: Some("verification"),
                ..
            })
        }
    ));
    assert_eq!((c.starts, c.stops), (1, 1));
    assert_eq!(c.events, [CheckPhase::Startup, CheckPhase::Shutdown]);
    assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    let (mut sx, _) = radio(Policy {
        mismatch: Some(CheckPhase::Shutdown),
        ..Policy::default()
    });
    run(sx.startup(&mut c)).unwrap();
    assert!(matches!(
        run(sx.shutdown(&mut c)),
        Err(SessionError::Shutdown(ShutdownFailure {
            mismatch: true,
            ..
        }))
    ));
}

#[test]
fn transport_faults_attempt_nss_release_and_block_later_tx() {
    for fault in ["busy", "spi", "flush", "nss", "gpio", "select"] {
        let (mut sx, wire) = radio(AlwaysConnected);
        run(sx.startup(&mut ())).unwrap();
        {
            let mut w = wire.borrow_mut();
            match fault {
                "busy" => w.busy = true,
                "spi" => w.spi_error = true,
                "flush" => w.flush_error = true,
                "nss" => w.deselect_error = true,
                "select" => w.select_error = true,
                _ => w.pin_error = true,
            }
        }
        assert!(run(sx.set_tx(&mut (), 1)).is_err());
        if fault == "select" {
            embedded_hal::digital::OutputPin::set_high(&mut Pin(wire.clone())).unwrap();
        }
        assert!(!wire.borrow().selected);
        assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    }
    // Local register reads use the same NSS cleanup rule as upstream operations.
    let (mut sx, wire) = radio(AlwaysConnected);
    wire.borrow_mut().select_error = true;
    assert_eq!(
        run(sx.read_reg(REG_OCP)),
        Err(SxError::Spi(embedded_hal_bus::spi::DeviceError::Cs(
            digital::ErrorKind::Other
        )))
    );
    embedded_hal::digital::OutputPin::set_high(&mut Pin(wire.clone())).unwrap();
    assert!(!wire.borrow().selected);
}

#[test]
fn chip_readback_parameter_bounds_and_upstream_workarounds() {
    let (mut sx, wire) = radio(AlwaysConnected);
    for frequency in [149_999_999, 960_000_001] {
        assert_eq!(
            run(sx.set_rf_frequency(frequency)),
            Err(SxError::InvalidParam)
        );
    }
    assert_eq!(
        run(sx.set_tx_params(23, RAMP_40_US)),
        Err(SxError::InvalidParam)
    );
    assert_eq!(
        run(sx.set_ocp(OCP_MAX_CODE + 1)),
        Err(SxError::InvalidParam)
    );
    assert_eq!(run(sx.set_rx(0x100_0000)), Err(SxError::InvalidParam));
    assert_eq!(
        run(sx.write_buffer(0, &[0; 257])),
        Err(SxError::InvalidParam)
    );
    assert_eq!(
        run(sx.set_lora_modulation_params(4, 4, 1, 0)),
        Err(SxError::InvalidParam)
    );
    assert_eq!(
        run(sx.set_cad_params(0, 1, 1, 2, 0)),
        Err(SxError::InvalidParam)
    );
    wire.borrow_mut().returned = vec![0, 0, 0x12, 0x34];
    assert_eq!(run(sx.get_device_errors()).unwrap(), 0x1234);
    run(sx.clear_device_errors()).unwrap();
    run(sx.set_rf_frequency(915_000_000)).unwrap();
    run(sx.configure_lora_modulation(&BaseBandModulationParams::new(
        SpreadingFactor::_7,
        Bandwidth::_125KHz,
        CodingRate::_4_5,
    )))
    .unwrap();
    run(sx.configure_lora_packet(&LoRaPacketParams {
        preamble_symbols: 8,
        implicit_header: false,
        payload_len: 16,
        crc: true,
        invert_iq: false,
    }))
    .unwrap();
    let frames = &wire.borrow().frames;
    assert!(frames.contains(&vec![CMD_SET_RF_FREQUENCY, 0x39, 0x30, 0, 0]));
    assert!(frames.contains(&vec![CMD_SET_MODULATION_PARAMS, 7, 4, 1, 0]));
    assert!(frames.contains(&vec![CMD_SET_PACKET_PARAMS, 0, 8, 0, 16, 1, 0]));
    // Semtech Rev2.2 §15.1 and §15.4, performed by the upstream driver.
    assert!(frames.contains(&vec![CMD_WRITE_REGISTER, 0x08, 0x89, 4]));
    assert!(frames.contains(&vec![CMD_WRITE_REGISTER, 0x07, 0x36, 4]));
    assert_eq!(encode_sync_word(0x12), SYNC_WORD_PRIVATE);
    assert_eq!(RadioStatus::from_byte(0x24).chip_mode, ChipMode::StbyRc);
}

#[test]
fn packet_telemetry_and_fifo_preserve_chip_encodings() {
    let (mut sx, wire) = radio(DenyTx);
    wire.borrow_mut().returned = vec![0x24, 0x24];
    let status = run(sx.get_status()).unwrap();
    assert_eq!(status.chip_mode, ChipMode::StbyRc);
    assert_eq!(status.command_status, CommandStatus::DataAvailable);
    assert!(status.is_ok());
    for (snr_byte, _snr_db) in [(28, 7), (248, -2)] {
        // Catalog sx1262 §13.5.3 GetPacketStatus uses half-dBm RSSI
        // magnitudes and a signed SNR byte in quarter-dB steps.
        wire.borrow_mut().returned = vec![0x24, 0x24, 168, snr_byte, 176];
        assert_eq!(
            run(sx.get_packet_status()).unwrap(),
            PacketStatus {
                rssi_pkt_half_dbm: -168,
                snr_pkt_quarter_db: snr_byte as i8,
                signal_rssi_pkt_half_dbm: -176,
            }
        );
    }
    wire.borrow_mut().returned = vec![0x24, 0x24, 216];
    assert_eq!(run(sx.get_rssi_inst()).unwrap(), -108);
    wire.borrow_mut().returned = vec![0x24, 0x24, 4, 10];
    assert_eq!(run(sx.get_rx_buffer_status()).unwrap(), (4, 10));
    run(sx.write_buffer(10, b"PING")).unwrap();
    wire.borrow_mut().returned = b"PING".to_vec();
    let mut payload = [0; 4];
    run(sx.read_buffer(10, &mut payload)).unwrap();
    assert_eq!(&payload, b"PING");
    assert!(wire
        .borrow()
        .frames
        .contains(&vec![CMD_WRITE_BUFFER, 10, b'P', b'I', b'N', b'G']));
    assert!(wire.borrow().frames.contains(&vec![CMD_READ_BUFFER, 10, 0]));
}

#[test]
fn hardware_mismatch_can_recover_after_cleanup_and_new_startup() {
    let (mut sx, wire) = radio(Policy::default());
    let mut c = Control::default();
    run(sx.startup(&mut c)).unwrap();
    c.antenna = false;
    assert_eq!(
        run(sx.set_tx(&mut c, 1)),
        Err(SessionError::ReadinessMismatch)
    );
    assert!(wire.borrow().frames.is_empty());
    run(sx.shutdown(&mut c)).unwrap();
    run(sx.startup(&mut c)).unwrap();
    run(sx.set_tx(&mut c, 1)).unwrap();
    assert_eq!(sx.session_stats().tx_attempts, 1);
}

struct AwaitingHooks;
impl Hooks<Control> for AwaitingHooks {
    type Error = std::convert::Infallible;
    fn tx_allowed(&self) -> bool {
        true
    }
    async fn startup(&mut self, c: &mut Control) -> Result<(), Self::Error> {
        c.antenna = true;
        Ok(())
    }
    async fn shutdown(&mut self, c: &mut Control) -> Result<(), Self::Error> {
        c.antenna = false;
        Ok(())
    }
    async fn verify(&mut self, _: &mut Control, r: CheckRequest) -> Result<bool, Self::Error> {
        if r.phase != CheckPhase::Shutdown {
            std::future::pending::<()>().await;
        }
        Ok(true)
    }
}
#[test]
fn cancelled_async_startup_requires_cleanup_and_releases_borrow() {
    let (mut sx, _) = radio(AwaitingHooks);
    let mut c = Control::default();
    {
        let future = sx.startup(&mut c);
        let mut future = std::pin::pin!(future);
        assert!(future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending());
    }
    c.unrelated += 1;
    assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    assert_eq!(run(sx.startup(&mut c)), Err(SessionError::RecoveryRequired));
    run(sx.shutdown(&mut c)).unwrap();
    assert!(!c.antenna);
}

#[test]
fn cooperative_busy_budget_and_post_nss_settling() {
    let (mut sx, wire) = radio(AlwaysConnected);
    wire.borrow_mut().busy_samples = vec![true, true, false, true, false];
    run(sx.get_status()).unwrap();
    assert_eq!(
        wire.borrow().delays,
        [1_000_000, 1_000_000, 1_000, 1_000_000]
    );
    assert_eq!(wire.borrow().samples, 5);
    assert!(!wire.borrow().selected);
    sx.set_busy_timing(BusyTiming {
        budget_ms: 5,
        poll_ms: std::num::NonZeroU32::new(2).unwrap(),
    });
    wire.borrow_mut().delays.clear();
    wire.borrow_mut().busy = true;
    assert_eq!(run(sx.get_status()), Err(SxError::BusyTimeout));
    assert_eq!(wire.borrow().delays, [2_000_000, 2_000_000, 1_000_000]);
    assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
}

#[test]
fn cancellation_during_spi_requires_nss_recovery_and_lifecycle_pair() {
    let (mut sx, wire) = radio(AlwaysConnected);
    run(sx.startup(&mut ())).unwrap();
    sx.set_verification_interval(std::num::NonZeroU32::new(40).unwrap());
    wire.borrow_mut().pending_spi = true;
    {
        let modulation = BaseBandModulationParams::new(
            SpreadingFactor::_7,
            Bandwidth::_125KHz,
            CodingRate::_4_5,
        );
        let mut future = std::pin::pin!(sx.configure_lora_modulation(&modulation));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut cx).is_pending());
        assert!(future.as_mut().poll(&mut cx).is_pending());
    }
    assert!(wire.borrow().selected); // Upstream ExclusiveDevice has no cancellation cleanup.
    assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    assert_eq!(sx.verification_interval().get(), 1);
    assert_eq!(
        run(sx.set_tx(&mut (), 1)),
        Err(SessionError::RecoveryRequired)
    );
    assert_eq!(
        run(sx.startup(&mut ())),
        Err(SessionError::RecoveryRequired)
    );
    let (_device, busy, delay, hooks) = sx.release(); // Release borrowed resources before touching caller-owned NSS.
    embedded_hal::digital::OutputPin::set_high(&mut Pin(wire.clone())).unwrap();
    wire.borrow_mut().pending_spi = false;
    let device = embedded_hal_bus::spi::ExclusiveDevice::new(
        Bus(wire.clone()),
        Pin(wire.clone()),
        Delay(wire.clone()),
    )
    .unwrap();
    let mut sx = Sx1262::with_hooks(device, busy, delay, hooks);
    run(sx.shutdown(&mut ())).unwrap();
    run(sx.startup(&mut ())).unwrap();
    run(sx.set_tx(&mut (), 1)).unwrap();
}

#[test]
fn cancelled_busy_wait_invalidates_an_active_session() {
    let (mut sx, wire) = radio(AlwaysConnected);
    run(sx.startup(&mut ())).unwrap();
    wire.borrow_mut().busy = true;
    {
        let mut future = std::pin::pin!(sx.get_irq_status());
        assert!(future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending());
    }
    assert!(!wire.borrow().selected);
    assert_eq!(
        run(sx.set_tx(&mut (), 1)),
        Err(SessionError::RecoveryRequired)
    );
}

#[test]
fn nonzero_timeout_protects_typed_and_raw_tx() {
    let (mut sx, wire) = radio(AlwaysConnected);
    run(sx.startup(&mut ())).unwrap();
    assert_eq!(
        run(sx.set_tx(&mut (), 0)),
        Err(SessionError::Chip(SxError::InvalidParam))
    );
    assert_eq!(
        run(sx.set_tx(&mut (), MAX_TIMEOUT_TICKS + 1)),
        Err(SessionError::Chip(SxError::InvalidParam))
    );
    assert_eq!(
        run(sx.write_cmd_with_context(&mut (), CMD_SET_TX, &[0, 0, 0])),
        Err(SessionError::Chip(SxError::InvalidParam))
    );
    assert!(wire.borrow().frames.is_empty());
    run(sx.set_tx(&mut (), 300 * 64)).unwrap();
    assert_eq!(wire.borrow().frames, [vec![CMD_SET_TX, 0, 0x4B, 0]]);
}

#[test]
fn upstream_validation_and_original_transport_faults_survive() {
    let (mut sx, wire) = radio(AlwaysConnected);
    run(sx.set_rf_frequency(300_000_000)).unwrap();
    assert_eq!(
        run(sx.configure_lora_modulation(&BaseBandModulationParams::new(
            SpreadingFactor::_7,
            Bandwidth::_500KHz,
            CodingRate::_4_5,
        ))),
        Err(SxError::Upstream(
            lora_phy::mod_params::RadioError::InvalidBandwidthForFrequency
        ))
    );
    wire.borrow_mut().pin_error = true;
    assert_eq!(
        run(sx.get_status()),
        Err(SxError::Busy(digital::ErrorKind::Other))
    );
    wire.borrow_mut().pin_error = false;
    wire.borrow_mut().fail_command = Some(CMD_SET_MODULATION_PARAMS);
    assert_eq!(
        run(sx.configure_lora_modulation(&BaseBandModulationParams::new(
            SpreadingFactor::_7,
            Bandwidth::_125KHz,
            CodingRate::_4_5,
        ))),
        Err(SxError::Spi(embedded_hal_bus::spi::DeviceError::Spi(
            spi::ErrorKind::Overrun
        )))
    );
    assert!(!wire.borrow().selected);
}

#[test]
fn receive_rejects_corrupt_packets_before_fifo_and_acknowledges_snapshot() {
    for error in [IRQ_CRC_ERR, IRQ_HEADER_ERR] {
        let (mut sx, wire) = radio(DenyTx);
        let irq = IRQ_RX_DONE | IRQ_PREAMBLE_DETECTED | error;
        wire.borrow_mut().returned = vec![0, 0, (irq >> 8) as u8, irq as u8];
        let mut preview = [0xAA; 4];
        assert_eq!(
            run(sx.poll_receive(&mut preview)),
            Ok(ReceivePoll::Rejected { irq })
        );
        assert_eq!(preview, [0xAA; 4]);
        assert_eq!(
            wire.borrow().frames,
            [
                vec![CMD_GET_IRQ_STATUS, 0, 0, 0],
                vec![CMD_CLEAR_IRQ_STATUS, (irq >> 8) as u8, irq as u8]
            ]
        );
    }
}

#[test]
fn continuous_rx_wraps_fifo_and_preserves_fractional_metrics() {
    let (mut sx, wire) = radio(DenyTx);
    run(sx.set_rx(RX_CONTINUOUS)).unwrap();
    wire.borrow_mut().frames.clear();
    for _ in 0..2 {
        wire.borrow_mut().returned = vec![
            0,
            0,
            0,
            IRQ_RX_DONE as u8,
            0,
            0,
            3,
            255,
            0,
            0,
            169,
            253,
            177,
            1,
            2,
            3,
        ];
        let mut preview = [0; 4];
        assert_eq!(
            run(sx.poll_receive(&mut preview)),
            Ok(ReceivePoll::Packet {
                len: 3,
                copied: 3,
                status: PacketStatus {
                    rssi_pkt_half_dbm: -169,
                    snr_pkt_quarter_db: -3,
                    signal_rssi_pkt_half_dbm: -177
                },
            })
        );
        assert_eq!(preview, [1, 2, 3, 0]);
    }
    assert!(wire
        .borrow()
        .frames
        .contains(&vec![CMD_READ_BUFFER, 255, 0]));
    assert!(!wire
        .borrow()
        .frames
        .iter()
        .any(|f| f.first() == Some(&CMD_SET_STANDBY)));
    let status = PacketStatus {
        rssi_pkt_half_dbm: -169,
        snr_pkt_quarter_db: -3,
        signal_rssi_pkt_half_dbm: -177,
    };
    assert_eq!(
        (
            status.rssi_pkt_dbm(),
            status.snr_pkt_db(),
            status.signal_rssi_pkt_dbm()
        ),
        (-84, 0, -88)
    );
}

#[test]
fn receive_io_failure_is_not_a_fabricated_packet() {
    let (mut sx, wire) = radio(AlwaysConnected);
    run(sx.startup(&mut ())).unwrap();
    wire.borrow_mut().returned = vec![0, 0, 0, IRQ_RX_DONE as u8, 0, 0, 4, 0, 0, 0, 100, 0, 100];
    wire.borrow_mut().fail_command = Some(CMD_READ_BUFFER);
    assert!(matches!(
        run(sx.poll_receive(&mut [0; 4])),
        Err(SxError::Spi(_))
    ));
    assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    assert!(!wire.borrow().selected);
}

#[test]
fn timed_rx_terminal_events_and_early_stop_clear_rtc_without_clobbering_bits() {
    for terminal in [0, IRQ_TIMEOUT, IRQ_RX_DONE | IRQ_CRC_ERR] {
        let (mut sx, wire) = radio(DenyTx);
        run(sx.set_rx(64)).unwrap();
        wire.borrow_mut().frames.clear();
        if terminal == 0 {
            wire.borrow_mut().returned = vec![0xA0];
            run(sx.set_standby(STDBY_CONFIG_RC)).unwrap();
        } else {
            wire.borrow_mut().returned = vec![0, 0, (terminal >> 8) as u8, terminal as u8, 0xA0];
            run(sx.poll_receive(&mut [])).unwrap();
        }
        assert!(wire
            .borrow()
            .frames
            .contains(&vec![CMD_WRITE_REGISTER, 9, 2, 0]));
        assert!(wire
            .borrow()
            .frames
            .contains(&vec![CMD_WRITE_REGISTER, 9, 0x44, 0xA2]));
        wire.borrow_mut().frames.clear();
        run(sx.stop_rx()).unwrap();
        assert_eq!(wire.borrow().frames, [vec![CMD_SET_STANDBY, 0]]);
    }
}

#[test]
fn clamp_and_upstream_errata_preserve_unrelated_bits_and_sf5_preamble() {
    let (mut sx, wire) = radio(DenyTx);
    wire.borrow_mut().returned = vec![0xC1];
    run(sx.configure_tx_clamp()).unwrap();
    assert!(wire
        .borrow()
        .frames
        .contains(&vec![CMD_WRITE_REGISTER, 8, 0xD8, 0xDF]));
    for (bw, ldro) in [
        (Bandwidth::_7KHz, 1),
        (Bandwidth::_125KHz, 0),
        (Bandwidth::_500KHz, 0),
    ] {
        wire.borrow_mut().returned = vec![0xA7];
        run(sx.configure_lora_modulation(&BaseBandModulationParams::new(
            SpreadingFactor::_8,
            bw,
            CodingRate::_4_5,
        )))
        .unwrap();
        assert!(wire
            .borrow()
            .frames
            .iter()
            .any(|f| f.first() == Some(&CMD_SET_MODULATION_PARAMS) && f[4] == ldro));
        let workaround = if bw == Bandwidth::_500KHz { 0xA3 } else { 0xA7 };
        assert!(wire
            .borrow()
            .frames
            .contains(&vec![CMD_WRITE_REGISTER, 8, 0x89, workaround]));
    }
    run(sx.configure_lora_modulation(&BaseBandModulationParams::new(
        SpreadingFactor::_5,
        Bandwidth::_125KHz,
        CodingRate::_4_5,
    )))
    .unwrap();
    wire.borrow_mut().returned = vec![0xFF];
    run(sx.configure_lora_packet(&LoRaPacketParams {
        preamble_symbols: 1,
        implicit_header: false,
        payload_len: 4,
        crc: false,
        invert_iq: true,
    }))
    .unwrap();
    assert!(wire
        .borrow()
        .frames
        .contains(&vec![CMD_SET_PACKET_PARAMS, 0, 12, 0, 4, 0, 1]));
    assert!(wire
        .borrow()
        .frames
        .contains(&vec![CMD_WRITE_REGISTER, 7, 0x36, 0xFB]));
}

#[test]
fn calibration_band_cache_survives_active_startup_and_resets_after_shutdown() {
    let (mut sx, wire) = radio(AlwaysConnected);
    run(sx.startup(&mut ())).unwrap();
    for freq in [902_125_000, 915_000_000, 927_875_000] {
        run(sx.set_rf_frequency(freq)).unwrap();
    }
    assert_eq!(sx.calibration_band(), Some(CalibrationBand::Mhz902_928));
    run(sx.startup(&mut ())).unwrap();
    assert_eq!(
        wire.borrow()
            .frames
            .iter()
            .filter(|f| f.first() == Some(&CMD_CALIBRATE_IMAGE))
            .count(),
        1
    );
    run(sx.set_rf_frequency(868_000_000)).unwrap();
    assert_eq!(sx.calibration_band(), Some(CalibrationBand::Mhz863_870));
    run(sx.shutdown(&mut ())).unwrap();
    run(sx.startup(&mut ())).unwrap();
    assert_eq!(sx.calibration_band(), None);
    assert_eq!(CalibrationBand::for_frequency(900_000_000), None);
}

#[test]
fn cancelled_pre_tx_verification_resets_cadence_and_releases_context() {
    let (mut sx, wire) = radio(Policy {
        pending_check: Some(CheckPhase::BeforeTx),
        ..Policy::default()
    });
    let mut c = Control::default();
    run(sx.startup(&mut c)).unwrap();
    sx.set_verification_interval(std::num::NonZeroU32::new(40).unwrap());
    {
        let mut future = std::pin::pin!(sx.set_tx(&mut c, 1));
        assert!(future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending());
    }
    c.unrelated += 1;
    assert!(wire.borrow().bytes.is_empty());
    assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    assert_eq!(sx.verification_interval().get(), 1);
    run(sx.shutdown(&mut c)).unwrap();
    run(sx.startup(&mut c)).unwrap();
}

#[test]
fn timed_valid_rx_cleans_rtc_after_fifo_and_preserves_unobserved_irqs() {
    let (mut sx, wire) = radio(DenyTx);
    run(sx.set_rx(100)).unwrap();
    wire.borrow_mut().frames.clear();
    wire.borrow_mut().returned = vec![
        0,
        0,
        0,
        IRQ_RX_DONE as u8,
        0,
        0,
        1,
        250,
        0,
        0,
        201,
        7,
        203,
        0x42,
        0xD0,
    ];
    let mut out = [0; 4];
    assert!(matches!(
        run(sx.poll_receive(&mut out)),
        Ok(ReceivePoll::Packet {
            len: 1,
            copied: 1,
            ..
        })
    ));
    assert_eq!(out, [0x42, 0, 0, 0]);
    let frames = &wire.borrow().frames;
    assert!(frames.contains(&vec![CMD_CLEAR_IRQ_STATUS, 0, IRQ_RX_DONE as u8]));
    assert!(frames.contains(&vec![CMD_WRITE_REGISTER, 9, 0x44, 0xD2]));
}
