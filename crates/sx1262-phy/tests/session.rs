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
}
#[derive(Clone)]
struct Bus(Rc<RefCell<Wire>>);
impl spi::ErrorType for Bus {
    type Error = spi::ErrorKind;
}
impl spi::SpiBus for Bus {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Self::Error> {
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
    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        let mut wire = self.0.borrow_mut();
        assert!(wire.selected);
        if wire.spi_error {
            return Err(spi::ErrorKind::Other);
        }
        wire.bytes.extend(bytes);
        wire.frame.extend(bytes);
        Ok(())
    }
    fn transfer(&mut self, out: &mut [u8], input: &[u8]) -> Result<(), Self::Error> {
        self.write(input)?;
        self.read(out)
    }
    fn transfer_in_place(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.write(bytes)?;
        self.read(bytes)
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
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
        let wire = self.0.borrow();
        if wire.pin_error {
            Err(digital::ErrorKind::Other)
        } else {
            Ok(wire.busy)
        }
    }
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|v| !v)
    }
}
fn ready<T>(future: impl Future<Output = T>) -> T {
    let mut future = std::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("model must be ready"),
    }
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
        if self.fail_check == Some(request.phase) {
            return Err("verification");
        }
        Ok(self.mismatch != Some(request.phase)
            && c.antenna == (request.phase != CheckPhase::Shutdown))
    }
}
fn radio<H>(hooks: H) -> (Sx1262<Bus, Pin, Pin, H>, Rc<RefCell<Wire>>) {
    let wire = Rc::new(RefCell::new(Wire::default()));
    (
        Sx1262::with_hooks(
            Bus(wire.clone()),
            Pin(wire.clone()),
            Pin(wire.clone()),
            hooks,
        ),
        wire,
    )
}

#[test]
fn short_and_sustained_sessions_preserve_antenna_and_borrowed_context() {
    for packets in [1, 81] {
        let (mut sx, wire) = radio(Policy::default());
        let mut context = Control::default();
        ready(sx.startup(&mut context)).unwrap();
        for i in 0..packets {
            sx.set_rf_frequency(915_000_000 + i * 1000).unwrap();
            sx.set_standby(STDBY_CONFIG_RC).unwrap();
            sx.set_rx(0xFF_FFFF).unwrap();
            sx.write_buffer(0, b"ping").unwrap();
            context.unrelated += 1;
            ready(sx.set_tx(&mut context, 0)).unwrap();
            let before = sx.session_stats();
            ready(sx.startup(&mut context)).unwrap();
            assert_eq!(sx.session_stats(), before);
            assert!(context.antenna);
        }
        ready(sx.shutdown(&mut context)).unwrap();
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
        ready(sx.startup(&mut c)).unwrap();
        for _ in 0..81 {
            ready(sx.set_tx(&mut c, 1)).unwrap();
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
        ready(sx.set_tx(&mut c, 0)).unwrap();
        assert_eq!(sx.session_stats().verifications, 1);
        ready(sx.shutdown(&mut c)).unwrap();
        ready(sx.startup(&mut c)).unwrap();
        assert_eq!(sx.session_stats().tx_attempts, 0);
    }
}

#[test]
fn default_denied_permanent_policy_and_raw_guard() {
    let (mut sx, wire) = radio(DenyTx);
    assert_eq!(ready(sx.set_tx(&mut (), 0)), Err(SessionError::NotActive));
    ready(sx.startup(&mut ())).unwrap();
    assert_eq!(ready(sx.set_tx(&mut (), 0)), Err(SessionError::TxDenied));
    assert!(wire.borrow().bytes.is_empty());
    assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    let (mut sx, wire) = radio(AlwaysConnected);
    ready(sx.startup(&mut ())).unwrap();
    assert_eq!(
        sx.write_cmd(CMD_SET_TX, &[0, 0, 0]),
        Err(SxError::ContextRequired)
    );
    for command in [0xD1, 0xD2, 0xFF, CMD_SET_SLEEP, CMD_SET_RX_DUTY_CYCLE] {
        assert_eq!(
            ready(sx.write_cmd_with_context(&mut (), command, &[])),
            Err(SessionError::Chip(SxError::UnsupportedCommand))
        );
    }
    ready(sx.write_cmd_with_context(&mut (), CMD_SET_TX, &[0, 0, 0])).unwrap();
    assert_eq!(wire.borrow().frames, vec![vec![CMD_SET_TX, 0, 0, 0]]);
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
        ready(sx.startup(&mut c)).unwrap();
        let result = ready(sx.write_cmd_with_context(&mut c, CMD_SET_TX, &[0, 0, 0]));
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
        assert_eq!(
            ready(sx.startup(&mut c)),
            Err(SessionError::RecoveryRequired)
        );
        assert_eq!(
            ready(sx.set_tx(&mut c, 0)),
            Err(SessionError::RecoveryRequired)
        );
        ready(sx.shutdown(&mut c)).unwrap();
        ready(sx.startup(&mut c)).unwrap();
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
    let error = ready(sx.startup(&mut c)).unwrap_err();
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
    ready(sx.startup(&mut c)).unwrap();
    assert!(matches!(
        ready(sx.shutdown(&mut c)),
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
        ready(sx.startup(&mut ())).unwrap();
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
        assert!(ready(sx.set_tx(&mut (), 0)).is_err());
        assert!(!wire.borrow().selected);
        assert_eq!(sx.session_state(), SessionState::NeedsShutdown);
    }
    // Local register reads use the same NSS cleanup rule as upstream operations.
    let (mut sx, wire) = radio(AlwaysConnected);
    wire.borrow_mut().select_error = true;
    assert_eq!(sx.read_reg(REG_OCP), Err(SxError::Gpio));
    assert!(!wire.borrow().selected);
}

#[test]
fn chip_readback_parameter_bounds_and_upstream_workarounds() {
    let (mut sx, wire) = radio(AlwaysConnected);
    for frequency in [149_999_999, 960_000_001] {
        assert_eq!(sx.set_rf_frequency(frequency), Err(SxError::InvalidParam));
    }
    assert_eq!(sx.set_tx_params(23, RAMP_40_US), Err(SxError::InvalidParam));
    assert_eq!(sx.set_ocp(OCP_MAX_CODE + 1), Err(SxError::InvalidParam));
    assert_eq!(sx.set_rx(0x100_0000), Err(SxError::InvalidParam));
    assert_eq!(sx.write_buffer(255, &[1, 2]), Err(SxError::InvalidParam));
    assert_eq!(
        sx.set_lora_modulation_params(4, 4, 1, 0),
        Err(SxError::InvalidParam)
    );
    assert_eq!(sx.set_cad_params(0, 1, 1, 2, 0), Err(SxError::InvalidParam));
    wire.borrow_mut().returned = vec![0, 0, 0x12, 0x34];
    assert_eq!(sx.get_device_errors().unwrap(), 0x1234);
    sx.clear_device_errors().unwrap();
    sx.set_rf_frequency(915_000_000).unwrap();
    sx.configure_lora_modulation(&BaseBandModulationParams::new(
        SpreadingFactor::_7,
        Bandwidth::_125KHz,
        CodingRate::_4_5,
    ))
    .unwrap();
    sx.configure_lora_packet(&LoRaPacketParams {
        preamble_symbols: 8,
        implicit_header: false,
        payload_len: 16,
        crc: true,
        invert_iq: false,
    })
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
    let status = sx.get_status().unwrap();
    assert_eq!(status.chip_mode, ChipMode::StbyRc);
    assert_eq!(status.command_status, CommandStatus::DataAvailable);
    assert!(status.is_ok());
    for (snr_byte, snr_db) in [(28, 7), (248, -2)] {
        // Catalog sx1262 §13.5.3 GetPacketStatus uses half-dBm RSSI
        // magnitudes and a signed SNR byte in quarter-dB steps.
        wire.borrow_mut().returned = vec![0x24, 0x24, 168, snr_byte, 176];
        assert_eq!(
            sx.get_packet_status().unwrap(),
            PacketStatus {
                rssi_pkt_dbm: -84,
                snr_pkt_db: snr_db,
                signal_rssi_pkt_dbm: -88,
            }
        );
    }
    wire.borrow_mut().returned = vec![0x24, 0x24, 216];
    assert_eq!(sx.get_rssi_inst().unwrap(), -108);
    wire.borrow_mut().returned = vec![0x24, 0x24, 4, 10];
    assert_eq!(sx.get_rx_buffer_status().unwrap(), (4, 10));
    sx.write_buffer(10, b"PING").unwrap();
    wire.borrow_mut().returned = b"PING".to_vec();
    let mut payload = [0; 4];
    sx.read_buffer(10, &mut payload).unwrap();
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
    ready(sx.startup(&mut c)).unwrap();
    c.antenna = false;
    assert_eq!(
        ready(sx.set_tx(&mut c, 0)),
        Err(SessionError::ReadinessMismatch)
    );
    assert!(wire.borrow().frames.is_empty());
    ready(sx.shutdown(&mut c)).unwrap();
    ready(sx.startup(&mut c)).unwrap();
    ready(sx.set_tx(&mut c, 0)).unwrap();
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
    assert_eq!(
        ready(sx.startup(&mut c)),
        Err(SessionError::RecoveryRequired)
    );
    ready(sx.shutdown(&mut c)).unwrap();
    assert!(!c.antenna);
}
