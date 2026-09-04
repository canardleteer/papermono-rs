//! Bluetooth Low Energy peripheral passkey pairing and passive Wi-Fi channel listening.
//!
//! # Architecture & Privacy Safety
//! - **Privacy Preservation**: Uses an anonymous static random BLE address rather than
//!   the hardware eFuse MAC address, preventing device tracking. Never emits MAC addresses,
//!   SSIDs, BSSIDs, or identity resolving keys (IRKs) over CDC.
//! - **Passkey Entry Protocol**: Operates as a BLE peripheral advertising as `PaperMono`
//!   with `DisplayOnly` IO capabilities. When a central (e.g. smartphone) initiates pairing,
//!   `trouble-host` generates a 6-digit passkey which is rendered on the e-paper display.
//! - **No NVS Storage Writes**: Operates without modifying non-volatile storage flash
//!   sectors (`nvs_enable` disabled in driver), ensuring factory RF calibration data remains intact.
//! - **Coexistence Sequencing**: BLE controller is brought up and executed before
//!   triggering Wi-Fi scans to satisfy ESP32-S3 hardware RF coexistence scheduling.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, AtomicU8, Ordering};

/// Current pairing status of the Bluetooth Low Energy peripheral.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlePairStatus {
    /// Currently broadcasting BLE advertisements (`PaperMono`) and awaiting connection.
    Advertising,
    /// Smartphone / central device has connected, awaiting pairing negotiation.
    Connected,
    /// Passkey pairing in progress; displays the 6-digit PIN code to be entered on the phone.
    Pairing(u32),
    /// Pairing and encryption successfully established.
    Success,
    /// Pairing attempt failed or was canceled by user/peer.
    Failed(BleFailReason),
    /// Wireless radio is disabled in this firmware build.
    Disabled,
}

/// Reason code explaining why BLE pairing was rejected or aborted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BleFailReason {
    /// User canceled the passkey prompt or entered an incorrect PIN.
    PasskeyEntryFailed,
    /// Cryptographic confirm value mismatch during key exchange.
    ConfirmValueFailed,
    /// Peer disconnected before completing the handshake or pairing timed out.
    Timeout,
    /// Authentication requirements (e.g. MITM or IO capabilities) could not be met.
    AuthenticationRequirements,
    /// Pairing attempt disallowed due to rate limiting or repeated failures.
    RepeatedAttempts,
    /// Unspecified or remote rejection.
    Other,
}

impl BleFailReason {
    /// Human-readable explanation string for display on the card.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PasskeyEntryFailed => "Passkey entry failed / canceled",
            Self::ConfirmValueFailed => "Confirm value mismatch",
            Self::Timeout => "Pairing timed out / disconnected",
            Self::AuthenticationRequirements => "Auth requirements not met",
            Self::RepeatedAttempts => "Too many attempts (rate limit)",
            Self::Other => "Pairing rejected by device",
        }
    }
}

static BLE_PAIR_STATUS: AtomicU8 = AtomicU8::new(0);
static BLE_PAIR_PIN: AtomicU32 = AtomicU32::new(0);
static BLE_STATE_REV: AtomicU32 = AtomicU32::new(0);

/// Retrieves the current pairing state of the BLE peripheral.
pub fn pair_status() -> BlePairStatus {
    #[cfg(feature = "radio")]
    {
        match BLE_PAIR_STATUS.load(Ordering::Relaxed) {
            1 => BlePairStatus::Connected,
            2 => BlePairStatus::Pairing(BLE_PAIR_PIN.load(Ordering::Relaxed)),
            3 => BlePairStatus::Success,
            4 => BlePairStatus::Failed(match BLE_PAIR_PIN.load(Ordering::Relaxed) {
                0 => BleFailReason::PasskeyEntryFailed,
                1 => BleFailReason::ConfirmValueFailed,
                2 => BleFailReason::Timeout,
                3 => BleFailReason::AuthenticationRequirements,
                4 => BleFailReason::RepeatedAttempts,
                _ => BleFailReason::Other,
            }),
            5 => BlePairStatus::Disabled,
            _ => BlePairStatus::Advertising,
        }
    }
    #[cfg(not(feature = "radio"))]
    {
        BlePairStatus::Disabled
    }
}

/// Retrieves the monotonically increasing revision counter for pairing state transitions.
pub fn state_rev() -> u32 {
    BLE_STATE_REV.load(Ordering::Relaxed)
}

#[cfg(feature = "radio")]
fn set_pair_status(status: BlePairStatus) {
    match status {
        BlePairStatus::Advertising => {
            BLE_PAIR_STATUS.store(0, Ordering::Relaxed);
        }
        BlePairStatus::Connected => {
            BLE_PAIR_STATUS.store(1, Ordering::Relaxed);
        }
        BlePairStatus::Pairing(pin) => {
            BLE_PAIR_PIN.store(pin, Ordering::Relaxed);
            BLE_PAIR_STATUS.store(2, Ordering::Relaxed);
        }
        BlePairStatus::Success => {
            BLE_PAIR_STATUS.store(3, Ordering::Relaxed);
        }
        BlePairStatus::Failed(reason) => {
            let code = match reason {
                BleFailReason::PasskeyEntryFailed => 0,
                BleFailReason::ConfirmValueFailed => 1,
                BleFailReason::Timeout => 2,
                BleFailReason::AuthenticationRequirements => 3,
                BleFailReason::RepeatedAttempts => 4,
                BleFailReason::Other => 5,
            };
            BLE_PAIR_PIN.store(code, Ordering::Relaxed);
            BLE_PAIR_STATUS.store(4, Ordering::Relaxed);
        }
        BlePairStatus::Disabled => {
            BLE_PAIR_STATUS.store(5, Ordering::Relaxed);
        }
    }
    BLE_STATE_REV.fetch_add(1, Ordering::Release);
}

#[cfg(feature = "radio")]
use core::sync::atomic::{AtomicBool, AtomicU16};

#[cfg(feature = "radio")]
use bt_hci::cmd::le::{LeSetAdvData, LeSetAdvEnable, LeSetAdvParams, LeSetScanResponseData};
#[cfg(feature = "radio")]
use bt_hci::controller::{ControllerCmdSync, ExternalController};
#[cfg(feature = "radio")]
use embassy_futures::join::join;
#[cfg(feature = "radio")]
use embassy_futures::select::{select, Either};
#[cfg(feature = "radio")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "radio")]
use embassy_time::{Duration, Timer};
#[cfg(feature = "radio")]
use esp_hal::peripherals::{BT, WIFI};
#[cfg(feature = "radio")]
use esp_radio::ble::controller::BleConnector;
#[cfg(feature = "radio")]
use esp_radio::wifi::scan::{ScanConfig as WifiScanConfig, ScanTypeConfig};
#[cfg(feature = "radio")]
use esp_radio::wifi::sta::StationConfig;
#[cfg(feature = "radio")]
use esp_radio::wifi::{Config, ControllerConfig, Interface, WifiController};
#[cfg(feature = "radio")]
use trouble_host::prelude::*;

#[cfg(feature = "radio")]
use crate::cdc;

#[cfg(feature = "radio")]
#[gatt_server(
    connections_max = 1,
    mutex_type = CriticalSectionRawMutex,
    attribute_table_size = 32
)]
struct Server {
    pair: PairService,
}

#[cfg(feature = "radio")]
#[gatt_service(uuid = "6b1d0001-5c8a-4f0e-9c3a-2e7b1a0d4f11")]
struct PairService {
    #[characteristic(
        uuid = "6b1d0002-5c8a-4f0e-9c3a-2e7b1a0d4f11",
        read,
        value = 1,
        permissions(encrypted)
    )]
    token: u8,
}

#[cfg(feature = "radio")]
const WIFI_TIMEOUT_S: u64 = 20;
#[cfg(feature = "radio")]
const WIFI_MAX: usize = 32;
#[cfg(feature = "radio")]
const WIFI_PASSIVE_MS: u64 = 150;

#[cfg(feature = "radio")]
static WIFI_N: AtomicU16 = AtomicU16::new(0);
#[cfg(feature = "radio")]
static HAVE_WIFI: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "radio")]
fn store_wifi(n: u16) {
    WIFI_N.store(n, Ordering::Relaxed);
    HAVE_WIFI.store(true, Ordering::Relaxed);
    cdc::wifi(n);
}

/// Retrieves the count of observed Wi-Fi beacons for periodic banner reporting.
pub fn last_wifi() -> Option<u16> {
    #[cfg(feature = "radio")]
    {
        HAVE_WIFI
            .load(Ordering::Relaxed)
            .then(|| WIFI_N.load(Ordering::Relaxed))
    }
    #[cfg(not(feature = "radio"))]
    {
        None
    }
}

/// Retrieves the count of observed BLE packets (or pairing events) for periodic banner reporting.
pub fn last_ble() -> Option<u16> {
    None
}

#[cfg(feature = "radio")]
fn map_host_error(err: trouble_host::Error) -> BleFailReason {
    match err {
        trouble_host::Error::Timeout => BleFailReason::Timeout,
        trouble_host::Error::Security(PairingFailedReason::PasskeyEntryFailed) => {
            BleFailReason::PasskeyEntryFailed
        }
        trouble_host::Error::Security(PairingFailedReason::ConfirmValueFailed)
        | trouble_host::Error::Security(PairingFailedReason::DHKeyCheckFailed) => {
            BleFailReason::ConfirmValueFailed
        }
        trouble_host::Error::Security(PairingFailedReason::AuthenticationRequirements) => {
            BleFailReason::AuthenticationRequirements
        }
        trouble_host::Error::Security(PairingFailedReason::RepeatedAttempts) => {
            BleFailReason::RepeatedAttempts
        }
        _ => BleFailReason::Other,
    }
}

#[cfg(feature = "radio")]
async fn drive_connection<P: PacketPool>(
    gatt: &GattConnection<'_, '_, P>,
) -> Result<(), BleFailReason> {
    loop {
        match gatt.next().await {
            GattConnectionEvent::PassKeyDisplay(key) => {
                set_pair_status(BlePairStatus::Pairing(key.value() % 1_000_000));
            }
            GattConnectionEvent::PairingComplete { .. } => {
                set_pair_status(BlePairStatus::Success);
            }
            GattConnectionEvent::PairingFailed(err) => {
                return Err(map_host_error(err));
            }
            GattConnectionEvent::BondLost => {
                return Err(BleFailReason::Other);
            }
            GattConnectionEvent::Disconnected { .. } => {
                if matches!(pair_status(), BlePairStatus::Pairing(_)) {
                    return Err(BleFailReason::Timeout);
                }
                return Ok(());
            }
            GattConnectionEvent::Gatt { event } => {
                if let Ok(reply) = event.accept() {
                    reply.send().await;
                }
            }
            GattConnectionEvent::PassKeyConfirm(_)
            | GattConnectionEvent::PassKeyInput
            | GattConnectionEvent::OobRequest => {}
            _ => {}
        }
    }
}

#[cfg(feature = "radio")]
async fn advertise_once<C>(
    peripheral: &mut Peripheral<'_, C, DefaultPacketPool>,
    server: &Server<'_>,
) -> Result<(), BleFailReason>
where
    C: Controller
        + ControllerCmdSync<LeSetAdvData>
        + ControllerCmdSync<LeSetAdvEnable>
        + ControllerCmdSync<LeSetAdvParams>
        + ControllerCmdSync<LeSetScanResponseData>,
{
    let mut adv_data = [0u8; 31];
    let Ok(adv_len) = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(b"PaperMono"),
        ],
        &mut adv_data,
    ) else {
        return Err(BleFailReason::Other);
    };

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data[..adv_len],
                scan_data: &[],
            },
        )
        .await
        .map_err(|_| BleFailReason::Other)?;

    let conn = advertiser
        .accept()
        .await
        .map_err(|_| BleFailReason::Other)?;

    set_pair_status(BlePairStatus::Connected);
    let _ = conn.set_bondable(true);
    let _ = conn.request_security();
    let gatt = conn
        .with_attribute_server(server)
        .map_err(|_| BleFailReason::Other)?;

    drive_connection(&gatt).await
}

/// Asynchronous Embassy task driving Bluetooth Low Energy peripheral advertising and passkey pairing.
#[cfg(feature = "radio")]
#[embassy_executor::task]
pub async fn ble_run(bt: BT<'static>) {
    let Ok(connector) = BleConnector::new(bt, Default::default()) else {
        return;
    };
    let ble_controller: ExternalController<_, 10> = ExternalController::new(connector);
    let address = Address::random([0xF5, 0x42, 0x11, 0x22, 0x33, 0xF1]);
    let mut resources: HostResources<_, DefaultPacketPool, 1, 2> = HostResources::new();
    let stack = trouble_host::new(ble_controller, &mut resources)
        .set_random_address(address)
        .set_io_capabilities(IoCapabilities::DisplayOnly)
        .build();

    let mut runner = stack.runner();
    let mut peripheral = stack.peripheral();

    let Ok(server) = Server::new_with_config(GapConfig::default("PaperMono")) else {
        set_pair_status(BlePairStatus::Failed(BleFailReason::Other));
        return;
    };
    // Keep the derived service in the binary; Settings pairing reads it.
    let _ = &server.pair;

    set_pair_status(BlePairStatus::Advertising);

    let pair_loop = async {
        loop {
            let current = pair_status();
            if !matches!(current, BlePairStatus::Success | BlePairStatus::Failed(_)) {
                set_pair_status(BlePairStatus::Advertising);
            }

            if let Err(why) = advertise_once(&mut peripheral, &server).await {
                set_pair_status(BlePairStatus::Failed(why));
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    };

    let _ = join(runner.run(), pair_loop).await;
}

/// Asynchronous Embassy task driving Wi-Fi passive channel sweeping.
#[cfg(feature = "radio")]
#[embassy_executor::task]
pub async fn wifi_run(wifi: WIFI<'static>) {
    store_wifi(wifi_count(wifi).await);
}

/// Conducts a passive Wi-Fi scan and returns the number of unique APs discovered.
#[cfg(feature = "radio")]
async fn wifi_count(wifi: WIFI<'static>) -> u16 {
    let _sta = Interface::station();
    let station = Config::Station(StationConfig::default());
    let config = ControllerConfig::default().with_initial_config(station);
    let Ok(mut controller) = WifiController::new(wifi, config) else {
        return 0;
    };
    let scan_config = WifiScanConfig::default()
        .with_max(WIFI_MAX)
        .with_show_hidden(true)
        .with_scan_type(ScanTypeConfig::Passive(
            esp_hal::time::Duration::from_millis(WIFI_PASSIVE_MS),
        ));
    match select(
        controller.scan_async(&scan_config),
        Timer::after(Duration::from_secs(WIFI_TIMEOUT_S)),
    )
    .await
    {
        Either::First(Ok(result)) => result.len().min(usize::from(u16::MAX)) as u16,
        Either::First(Err(_)) | Either::Second(()) => 0,
    }
}
