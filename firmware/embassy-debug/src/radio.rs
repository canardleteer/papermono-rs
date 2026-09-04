//! Bluetooth Low Energy peripheral passkey pairing and passive Wi-Fi channel listening.
//!
//! # Purpose & Educational Reference
//! This module serves as an educational reference demonstrating asynchronous wireless
//! networking on the ESP32-S3 using the Embassy async framework, `esp-radio`, and the
//! pure-Rust `trouble-host` Bluetooth Low Energy stack, grounded in principles from
//! *The Embedded Rust Book*, *The Rust on ESP Book*, and *The Embassy Book*:
//!
//! - **Asynchronous I/O & Cooperative Multitasking**: Both BLE host processing and Wi-Fi
//!   scanning run as non-blocking async Embassy tasks (`#[embassy_executor::task]`), yielding
//!   execution cooperatively to the e-paper UI and telemetry workers without RTOS thread overhead.
//! - **HCI Controller Abstraction**: Bridges the ESP32-S3 hardware radio baseband (`esp-radio`'s
//!   `BleConnector`) with the host stack via `bt_hci::controller::ExternalController`.
//! - **Lock-Free Atomic State Sharing**: Communicates pairing states and 6-digit numeric passkeys
//!   to the interactive UI task via memory-ordered atomic primitives (`AtomicU8`, `AtomicU32`),
//!   preventing mutex contention across asynchronous task boundaries.
//!
//! # Architecture & Privacy Safety
//! - **Privacy Preservation**: Uses an anonymous static random BLE address rather than
//!   the hardware eFuse MAC address, preventing device tracking. Never emits MAC addresses,
//!   SSIDs, BSSIDs, or identity resolving keys (IRKs) over CDC telemetry.
//! - **Passkey Entry Protocol**: Operates as a BLE peripheral advertising as `PaperMono`
//!   with `DisplayOnly` IO capabilities. When a central (e.g. smartphone) initiates pairing,
//!   the Security Manager Protocol (SMP) generates a random 6-digit passkey which is rendered
//!   on the e-paper display for the user to type into their phone.
//! - **GATT Service & Attribute Server**: Exposes a standard Generic Access Profile (GAP)
//!   and a custom pairing service with an encrypted read-only characteristic. Modern smartphone
//!   operating systems require encrypted characteristics or explicit security requests to
//!   prompt the user for bonding and passkey exchange.
//! - **No NVS Storage Writes**: Operates without modifying non-volatile storage flash
//!   sectors (`nvs_enable` disabled in driver), ensuring factory RF calibration data remains intact.
//! - **Coexistence Sequencing**: BLE controller is brought up and executed before
//!   triggering Wi-Fi scans to satisfy ESP32-S3 hardware RF coexistence scheduling.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, AtomicU8, Ordering};

/// Current pairing status of the Bluetooth Low Energy peripheral.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlePairStatus {
    /// Currently broadcasting undirected connectable BLE advertisements (`PaperMono`) and awaiting connection.
    Advertising,
    /// Smartphone / central device has established an active link layer connection; awaiting pairing negotiation.
    Connected,
    /// Passkey pairing in progress; displays the 6-digit PIN code (0..=999,999) to be entered on the phone.
    Pairing(u32),
    /// Pairing and link encryption successfully established and verified.
    Success,
    /// Pairing attempt failed, timed out, or was canceled by user/peer.
    Failed(BleFailReason),
    /// Wireless radio is disabled in this firmware build configuration.
    Disabled,
}

/// Reason code explaining why BLE pairing was rejected or aborted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BleFailReason {
    /// User canceled the passkey prompt or entered an incorrect PIN on the central device.
    PasskeyEntryFailed,
    /// Cryptographic confirm value or Diffie-Hellman key check mismatch during key exchange.
    ConfirmValueFailed,
    /// Peer disconnected before completing the handshake or pairing timed out.
    Timeout,
    /// Authentication requirements (e.g. MITM protection or IO capabilities) could not be negotiated.
    AuthenticationRequirements,
    /// Pairing attempt disallowed due to rate limiting or repeated failures.
    RepeatedAttempts,
    /// Unspecified link layer or host stack rejection.
    Other,
}

impl BleFailReason {
    /// Human-readable explanation string for display on the e-paper card and telemetry.
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

/// Encoded integer representation of the active [`BlePairStatus`].
///
/// Encoded states:
/// - `0`: [`BlePairStatus::Advertising`]
/// - `1`: [`BlePairStatus::Connected`]
/// - `2`: [`BlePairStatus::Pairing`] (with PIN payload stored in [`BLE_PAIR_PIN`])
/// - `3`: [`BlePairStatus::Success`]
/// - `4`: [`BlePairStatus::Failed`] (with failure code stored in [`BLE_PAIR_PIN`])
/// - `5`: [`BlePairStatus::Disabled`]
static BLE_PAIR_STATUS: AtomicU8 = AtomicU8::new(0);

/// Numeric payload accompanying the active pairing state.
///
/// When in [`BlePairStatus::Pairing`], stores the 6-digit numeric passkey (0..=999,999).
/// When in [`BlePairStatus::Failed`], stores the integer discriminant mapping to [`BleFailReason`].
static BLE_PAIR_PIN: AtomicU32 = AtomicU32::new(0);

/// Monotonically increasing revision counter incremented on every state transition.
///
/// The interactive UI task samples this counter to detect when the Bluetooth pairing
/// state has transitioned (e.g., from advertising to connected, or from connected to
/// displaying a passkey), prompting an immediate partial e-paper screen refresh.
static BLE_STATE_REV: AtomicU32 = AtomicU32::new(0);

/// Retrieves the current pairing state of the BLE peripheral.
///
/// Atomically decodes [`BLE_PAIR_STATUS`] and [`BLE_PAIR_PIN`]. When the `radio`
/// feature is disabled, unconditionally returns [`BlePairStatus::Disabled`].
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
///
/// Used by the UI navigation task to detect external asynchronous state transitions
/// without requiring an async channel or lock contention.
pub fn state_rev() -> u32 {
    BLE_STATE_REV.load(Ordering::Relaxed)
}

/// Stores a new [`BlePairStatus`] into the shared atomic registers and increments [`BLE_STATE_REV`].
///
/// Updates [`BLE_PAIR_STATUS`] and [`BLE_PAIR_PIN`] using [`Ordering::Relaxed`], followed by
/// a [`Ordering::Release`] fetch-add on [`BLE_STATE_REV`] to establish a happens-before relationship
/// for consumer tasks observing revision counter changes.
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
    // Bump revision counter with Release ordering so UI task sees the updated status and PIN.
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

/// GATT attribute server managing local services and characteristics.
///
/// Generated by the `#[gatt_server]` macro from `trouble-host`. Configured for:
/// - `connections_max = 1`: PaperMono operates strictly as a single-connection peripheral.
/// - `mutex_type = CriticalSectionRawMutex`: Provides interrupt-safe, single-core
///   synchronization without requiring an RTOS or standard library runtime.
/// - `attribute_table_size = 32`: Sufficient capacity for standard GAP service
///   attributes, custom pairing service, and characteristic descriptors without heap allocations.
#[cfg(feature = "radio")]
#[gatt_server(
    connections_max = 1,
    mutex_type = CriticalSectionRawMutex,
    attribute_table_size = 32
)]
struct Server {
    /// Custom pairing service requiring encrypted access.
    pair: PairService,
}

/// Custom GATT service exposing an encrypted token characteristic to provoke pairing.
///
/// Defined with a unique 128-bit UUID (`6b1d0001-5c8a-4f0e-9c3a-2e7b1a0d4f11`).
/// Many smartphone operating systems (such as iOS and Android) do not automatically
/// initiate Security Manager Protocol (SMP) pairing upon raw link layer connection unless
/// an encrypted GATT attribute is accessed. Exposing this service and characteristic
/// ensures that the smartphone's GATT client initiates bonding and passkey exchange.
#[cfg(feature = "radio")]
#[gatt_service(uuid = "6b1d0001-5c8a-4f0e-9c3a-2e7b1a0d4f11")]
struct PairService {
    /// 1-byte read-only token characteristic with `permissions(encrypted)`.
    ///
    /// Any unauthenticated read of this characteristic returns an ATT error
    /// `INSUFFICIENT_AUTHENTICATION`, forcing the central device to prompt the user
    /// for passkey pairing.
    #[characteristic(
        uuid = "6b1d0002-5c8a-4f0e-9c3a-2e7b1a0d4f11",
        read,
        value = 1,
        permissions(encrypted)
    )]
    token: u8,
}

/// Maximum duration in seconds to wait for a Wi-Fi scan result before timing out.
#[cfg(feature = "radio")]
const WIFI_TIMEOUT_S: u64 = 20;

/// Maximum number of discovered access points to buffer during a Wi-Fi scan.
#[cfg(feature = "radio")]
const WIFI_MAX: usize = 32;

/// Channel dwell time in milliseconds for passive Wi-Fi listening.
#[cfg(feature = "radio")]
const WIFI_PASSIVE_MS: u64 = 150;

/// Atomic storage for the most recently observed count of Wi-Fi access points.
#[cfg(feature = "radio")]
static WIFI_N: AtomicU16 = AtomicU16::new(0);

/// Atomic flag indicating whether at least one valid Wi-Fi scan has completed.
#[cfg(feature = "radio")]
static HAVE_WIFI: AtomicBool = AtomicBool::new(false);

/// Stores the Wi-Fi scan count in atomic registers and emits a CDC telemetry line.
#[cfg(feature = "radio")]
fn store_wifi(n: u16) {
    WIFI_N.store(n, Ordering::Relaxed);
    HAVE_WIFI.store(true, Ordering::Relaxed);
    cdc::wifi(n);
}

/// Retrieves the count of observed Wi-Fi beacons for periodic banner reporting.
///
/// Returns `Some(count)` once the first background scan completes, or `None` if the scan
/// is still in flight or the `radio` feature is disabled.
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

/// Retrieves the count of observed BLE packets for periodic banner reporting.
///
/// Returns `None` as the peripheral operates in connection/pairing mode rather than passive sniffing.
pub fn last_ble() -> Option<u16> {
    None
}

/// Maps a low-level [`trouble_host::Error`] into a user-facing [`BleFailReason`].
///
/// Categorizes Security Manager Protocol (SMP) failure reasons, connection timeouts,
/// and authentication negotiation failures into distinct explanatory enum variants
/// for on-screen e-paper rendering.
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

/// Drives the asynchronous event pump for an active GATT connection until termination.
///
/// Processes incoming [`GattConnectionEvent`] items yielded by the connection:
/// - [`GattConnectionEvent::PassKeyDisplay`]: Updates peripheral status to [`BlePairStatus::Pairing`]
///   with the generated 6-digit numeric passkey, triggering an e-paper display refresh.
/// - [`GattConnectionEvent::PairingComplete`]: Updates status to [`BlePairStatus::Success`].
/// - [`GattConnectionEvent::PairingFailed`]: Maps the error code and returns [`Err(BleFailReason)`].
/// - [`GattConnectionEvent::BondLost`]: Returns [`Err(BleFailReason::Other)`].
/// - [`GattConnectionEvent::Disconnected`]: Handles peer link termination, converting to
///   [`BleFailReason::Timeout`] if the peer disconnected mid-pairing.
/// - [`GattConnectionEvent::Gatt`]: Responds to incoming ATT/GATT attribute requests from the phone.
///
/// # Errors
/// Returns [`Err(BleFailReason)`] if the pairing handshake fails or disconnects prematurely.
#[cfg(feature = "radio")]
async fn drive_connection<P: PacketPool>(
    gatt: &GattConnection<'_, '_, P>,
) -> Result<(), BleFailReason> {
    loop {
        match gatt.next().await {
            GattConnectionEvent::PassKeyDisplay(key) => {
                // Modulo 1,000,000 bounds the numeric passkey to a 6-digit integer (000000..999999).
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
                // If disconnected while awaiting passkey input, classify as timeout.
                if matches!(pair_status(), BlePairStatus::Pairing(_)) {
                    return Err(BleFailReason::Timeout);
                }
                return Ok(());
            }
            GattConnectionEvent::Gatt { event } => {
                // Accept incoming read/write requests from the central device's GATT client.
                if let Ok(reply) = event.accept() {
                    reply.send().await;
                }
            }
            GattConnectionEvent::PassKeyConfirm(_)
            | GattConnectionEvent::PassKeyInput
            | GattConnectionEvent::OobRequest => {
                // DisplayOnly capability never generates PassKeyConfirm or PassKeyInput events.
            }
            _ => {}
        }
    }
}

/// Executes a single advertising cycle, accepting an incoming central connection and driving pairing.
///
/// # Procedure
/// 1. Encodes BLE advertising payload containing:
///    - Flags: `LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED`
///    - Complete Local Name: `"PaperMono"`
/// 2. Starts connectable, scannable undirected advertising via [`Peripheral::advertise`].
/// 3. Awaits incoming central connection via [`Advertiser::accept`].
/// 4. Transitions status to [`BlePairStatus::Connected`].
/// 5. Marks connection as bondable ([`Connection::set_bondable`]) to exchange long-term keys.
/// 6. Sends an SMP Security Request ([`Connection::request_security`]) to initiate pairing.
/// 7. Attaches the local [`Server`] GATT attribute table ([`Connection::with_attribute_server`]).
/// 8. Executes [`drive_connection`] until session completion or failure.
///
/// # Errors
/// Returns [`Err(BleFailReason)`] if advertising encoding, connection acceptance,
/// attribute server binding, or pairing fails.
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
    // Construct standard advertising packet (maximum 31 bytes).
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

    // Begin connectable undirected advertising.
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

    // Await central link layer connection.
    let conn = advertiser
        .accept()
        .await
        .map_err(|_| BleFailReason::Other)?;

    set_pair_status(BlePairStatus::Connected);

    // Request bonding so that smartphone saves pairing keys across reconnections.
    let _ = conn.set_bondable(true);

    // Proactively request security handshake from the central.
    let _ = conn.request_security();

    // Bind local GATT attribute server so phone can perform service discovery.
    let gatt = conn
        .with_attribute_server(server)
        .map_err(|_| BleFailReason::Other)?;

    // Pump connection events until termination.
    drive_connection(&gatt).await
}

/// Asynchronous Embassy task driving Bluetooth Low Energy peripheral advertising and passkey pairing.
///
/// # Hardware Initialization & Stack Configuration
/// - Initializes the ESP32-S3 hardware Bluetooth controller via [`BleConnector::new`].
/// - Wraps the connector in an [`ExternalController`] configured with an HCI queue depth of 10
///   to prevent packet drops during rapid SMP/ATT burst transactions.
/// - Configures an anonymous static random address (`0xF5, 0x42, 0x11, 0x22, 0x33, 0xF1`) to
///   prevent physical device tracking and preserve privacy.
/// - Sets IO capabilities to [`IoCapabilities::DisplayOnly`], signaling to the central that
///   PaperMono can display a 6-digit numeric passkey but possesses no numeric keyboard for input.
/// - Instantiates the GATT attribute [`Server`] with GAP appearance name `"PaperMono"`.
///
/// # Embassy Scheduling
/// Runs the background host runner ([`Runner::run`]) concurrently with the continuous
/// advertising and pairing state loop via [`embassy_futures::join::join`].
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
            // Preserve terminal success or failure displays until next connection attempt.
            if !matches!(current, BlePairStatus::Success | BlePairStatus::Failed(_)) {
                set_pair_status(BlePairStatus::Advertising);
            }

            if let Err(why) = advertise_once(&mut peripheral, &server).await {
                set_pair_status(BlePairStatus::Failed(why));
                // Back off for 5 seconds on pairing failure before returning to advertising.
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    };

    // Join the background host packet runner and the peripheral advertising loop.
    let _ = join(runner.run(), pair_loop).await;
}

/// Asynchronous Embassy task driving Wi-Fi passive channel sweeping.
///
/// Samples all active 2.4 GHz channels passively without transmitting probe requests
/// or associating with an AP, storing the discovered AP count in atomic memory.
#[cfg(feature = "radio")]
#[embassy_executor::task]
pub async fn wifi_run(wifi: WIFI<'static>) {
    store_wifi(wifi_count(wifi).await);
}

/// Conducts a passive Wi-Fi scan and returns the number of unique APs discovered.
///
/// Configures the ESP32-S3 Wi-Fi controller in station mode, sets passive channel dwell
/// time to [`WIFI_PASSIVE_MS`], and races the scan against a [`WIFI_TIMEOUT_S`] timer.
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
