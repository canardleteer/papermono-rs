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
//! - **No NVS Storage Writes**: Pairing bonds are stored strictly in RAM (`HostResources`),
//!   and no NVS storage partition or filesystem is initialized or mounted, ensuring factory
//!   flash sectors and RF calibration data remain untouched.
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

/// Static SSID for PaperMono WPA2-Personal Hotspot.
pub const AP_SSID: &str = "PaperMono-AP";

/// Static 8-character alphanumeric WPA2-Personal password.
pub const AP_PASSWORD: &str = "mono2026";

/// Static IPv4 address string for the PaperMono access point.
pub const AP_IP_STR: &str = "192.168.4.1";

/// Active operational mode of the Wi-Fi subsystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WifiMode {
    /// Radio is idle; neither survey nor hotspot is active.
    Idle,
    /// Channel survey scan is currently running in the background.
    SurveyScanning,
    /// Channel survey scan has completed and cached results are displayed.
    SurveyComplete,
    /// Hotspot mode is active: running SoftAP, DHCP server, and JSON HTTP server.
    Hotspot,
}

/// Control command sent from the UI touch interaction to the Wi-Fi manager task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WifiCommand {
    /// Start or re-run the 2.4 GHz channel survey (disables hotspot if active).
    StartSurvey,
    /// Cancel or stop the active channel survey.
    StopSurvey,
    /// Start the SoftAP and embedded JSON web server (disables survey if active).
    StartHotspot,
    /// Stop the SoftAP and return to idle.
    StopHotspot,
}

/// Discovered access point entry preserved for on-screen survey reporting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurveyApEntry {
    /// ASCII SSID string buffer (up to 18 bytes).
    pub ssid: [u8; 18],
    /// Length of valid ASCII bytes in `ssid`.
    pub ssid_len: u8,
    /// Operating Wi-Fi 2.4 GHz channel (1..=14).
    pub channel: u8,
    /// Signal strength indicator in dBm.
    pub rssi: i8,
    /// Authentication method summary string (e.g. "WPA2", "WPA3", "Open").
    pub auth: &'static str,
}

/// Aggregated survey metrics across all 2.4 GHz channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WifiSurveyData {
    /// Total number of unique access points discovered.
    pub total_aps: u16,
    /// Number of access points detected on Channel 1.
    pub ch1_count: u16,
    /// Number of access points detected on Channel 6.
    pub ch6_count: u16,
    /// Number of access points detected on Channel 11.
    pub ch11_count: u16,
    /// Number of access points detected on other channels.
    pub other_count: u16,
    /// Top 4 strongest access points sorted by RSSI descending.
    pub top_aps: [Option<SurveyApEntry>; 4],
}

/// Live status report of the SoftAP and embedded HTTP web server.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WifiApStatus {
    /// Whether the access point beacon and network stack are actively running.
    pub active: bool,
    /// Number of stations currently associated with the SoftAP.
    pub clients: u16,
    /// Cumulative count of HTTP GET requests served on port 80.
    pub http_requests: u32,
    /// Active network SSID.
    pub ssid: &'static str,
    /// WPA2-Personal authentication password.
    pub password: &'static str,
    /// IPv4 gateway URL address.
    pub ip: &'static str,
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

/// Stores a new [`BlePairStatus`] into the shared atomic registers, emits CDC telemetry, and increments [`BLE_STATE_REV`].
///
/// Updates [`BLE_PAIR_STATUS`] and [`BLE_PAIR_PIN`] using [`Ordering::Relaxed`], followed by
/// a [`Ordering::Release`] fetch-add on [`BLE_STATE_REV`] to establish a happens-before relationship
/// for consumer tasks observing revision counter changes. Emits wire telemetry via [`cdc`].
#[cfg(feature = "radio")]
fn set_pair_status(status: BlePairStatus) {
    if pair_status() == status {
        return;
    }
    match status {
        BlePairStatus::Advertising => {
            BLE_PAIR_STATUS.store(0, Ordering::Relaxed);
            cdc::pair_state("advertising");
        }
        BlePairStatus::Connected => {
            BLE_PAIR_STATUS.store(1, Ordering::Relaxed);
            cdc::pair_state("connected");
        }
        BlePairStatus::Pairing(pin) => {
            BLE_PAIR_PIN.store(pin, Ordering::Relaxed);
            BLE_PAIR_STATUS.store(2, Ordering::Relaxed);
            cdc::pair_pin(pin);
        }
        BlePairStatus::Success => {
            BLE_PAIR_STATUS.store(3, Ordering::Relaxed);
            cdc::pair_ok();
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
            cdc::pair_fail(reason.as_str());
        }
        BlePairStatus::Disabled => {
            BLE_PAIR_STATUS.store(5, Ordering::Relaxed);
        }
    }
    // Bump revision counter with Release ordering so UI task sees the updated status and PIN.
    BLE_STATE_REV.fetch_add(1, Ordering::Release);
}

#[cfg(feature = "radio")]
use core::cell::RefCell;
#[cfg(feature = "radio")]
use core::fmt::Write;
#[cfg(feature = "radio")]
use core::net::{Ipv4Addr, SocketAddrV4};
#[cfg(feature = "radio")]
use core::sync::atomic::{AtomicBool, AtomicU16};

#[cfg(feature = "radio")]
use bt_hci::cmd::le::{LeSetAdvData, LeSetAdvEnable, LeSetAdvParams, LeSetScanResponseData};
#[cfg(feature = "radio")]
use bt_hci::controller::{ControllerCmdSync, ExternalController};
#[cfg(feature = "radio")]
use edge_dhcp::io::{self, DEFAULT_SERVER_PORT};
#[cfg(feature = "radio")]
use edge_dhcp::server::{Server as DhcpServer, ServerOptions};
#[cfg(feature = "radio")]
use edge_nal::UdpBind;
#[cfg(feature = "radio")]
use edge_nal_embassy::{Udp, UdpBuffers};
#[cfg(feature = "radio")]
use embassy_executor::Spawner;
#[cfg(feature = "radio")]
use embassy_futures::join::join;
#[cfg(feature = "radio")]
use embassy_futures::select::{select, Either};
#[cfg(feature = "radio")]
use embassy_net::tcp::TcpSocket;
#[cfg(feature = "radio")]
use embassy_net::{IpListenEndpoint, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
#[cfg(feature = "radio")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "radio")]
use embassy_sync::blocking_mutex::CriticalSectionMutex;
#[cfg(feature = "radio")]
use embassy_sync::channel::Channel;
#[cfg(feature = "radio")]
use embassy_time::{Duration, Timer};
#[cfg(feature = "radio")]
use embedded_io_async::Write as AsyncWrite;
#[cfg(feature = "radio")]
use esp_hal::peripherals::{BT, WIFI};
#[cfg(feature = "radio")]
use esp_radio::ble::controller::BleConnector;
#[cfg(feature = "radio")]
use esp_radio::wifi::ap::AccessPointConfig;
#[cfg(feature = "radio")]
use esp_radio::wifi::scan::{ScanConfig as WifiScanConfig, ScanTypeConfig};
#[cfg(feature = "radio")]
use esp_radio::wifi::sta::StationConfig;
#[cfg(feature = "radio")]
use esp_radio::wifi::{
    AuthenticationMethod, AuthenticationMethodConfig, Config, ControllerConfig, Interface,
    WifiController,
};
#[cfg(feature = "radio")]
use trouble_host::prelude::*;

#[cfg(feature = "radio")]
use crate::cdc;

/// Static IPv4 gateway address object for the PaperMono access point.
#[cfg(feature = "radio")]
pub const AP_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 1);

/// Active Wi-Fi mode: 0=Idle, 1=SurveyScanning, 2=SurveyComplete, 3=Hotspot.
#[cfg(feature = "radio")]
static WIFI_MODE: AtomicU8 = AtomicU8::new(0);

/// Monotonically increasing revision counter triggering UI re-renders on Wi-Fi state changes.
#[cfg(feature = "radio")]
static WIFI_STATE_REV: AtomicU32 = AtomicU32::new(0);

/// Flag indicating whether the SoftAP and HTTP/DHCP servers are active.
#[cfg(feature = "radio")]
static HOTSPOT_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Number of currently connected Wi-Fi stations.
#[cfg(feature = "radio")]
static AP_CLIENTS: AtomicU16 = AtomicU16::new(0);

/// Cumulative HTTP requests served by the embedded web server.
#[cfg(feature = "radio")]
static HTTP_REQUESTS: AtomicU32 = AtomicU32::new(0);

/// Mutex-protected cached survey results.
#[cfg(feature = "radio")]
static SURVEY_DATA: CriticalSectionMutex<RefCell<Option<WifiSurveyData>>> =
    CriticalSectionMutex::new(RefCell::new(None));

/// Inter-task command channel from UI to Wi-Fi manager.
#[cfg(feature = "radio")]
static WIFI_CMD: Channel<CriticalSectionRawMutex, WifiCommand, 4> = Channel::new();

/// Static memory for embassy-net stack resources.
#[cfg(feature = "radio")]
static STACK_RESOURCES: static_cell::StaticCell<StackResources<4>> = static_cell::StaticCell::new();

/// Static memory for DHCP UDP socket buffers.
#[cfg(feature = "radio")]
static UDP_BUFFERS: static_cell::StaticCell<UdpBuffers<2, 1024, 1024, 4>> =
    static_cell::StaticCell::new();

/// Sends an asynchronous control command to the background Wi-Fi manager task.
pub fn send_wifi_cmd(cmd: WifiCommand) {
    #[cfg(feature = "radio")]
    {
        let _ = WIFI_CMD.try_send(cmd);
    }
    #[cfg(not(feature = "radio"))]
    {
        let _ = cmd;
    }
}

/// Retrieves the active operational mode of the Wi-Fi subsystem.
pub fn wifi_mode() -> WifiMode {
    #[cfg(feature = "radio")]
    {
        match WIFI_MODE.load(Ordering::Relaxed) {
            1 => WifiMode::SurveyScanning,
            2 => WifiMode::SurveyComplete,
            3 => WifiMode::Hotspot,
            _ => WifiMode::Idle,
        }
    }
    #[cfg(not(feature = "radio"))]
    {
        WifiMode::Idle
    }
}

/// Retrieves the monotonically increasing revision counter for Wi-Fi state transitions.
pub fn wifi_state_rev() -> u32 {
    #[cfg(feature = "radio")]
    {
        WIFI_STATE_REV.load(Ordering::Relaxed)
    }
    #[cfg(not(feature = "radio"))]
    {
        0
    }
}

/// Retrieves a copy of the most recently completed channel survey results, if available.
pub fn wifi_survey_data() -> Option<WifiSurveyData> {
    #[cfg(feature = "radio")]
    {
        SURVEY_DATA.lock(|cell| *cell.borrow())
    }
    #[cfg(not(feature = "radio"))]
    {
        None
    }
}

/// Retrieves the live operational status of the SoftAP and embedded HTTP web server.
pub fn wifi_ap_status() -> WifiApStatus {
    #[cfg(feature = "radio")]
    {
        WifiApStatus {
            active: HOTSPOT_ACTIVE.load(Ordering::Relaxed),
            clients: AP_CLIENTS.load(Ordering::Relaxed),
            http_requests: HTTP_REQUESTS.load(Ordering::Relaxed),
            ssid: AP_SSID,
            password: AP_PASSWORD,
            ip: AP_IP_STR,
        }
    }
    #[cfg(not(feature = "radio"))]
    {
        WifiApStatus {
            active: false,
            clients: 0,
            http_requests: 0,
            ssid: AP_SSID,
            password: AP_PASSWORD,
            ip: AP_IP_STR,
        }
    }
}

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
                // Hold failure status for 5 seconds so the user can read the failure reason.
                Timer::after(Duration::from_millis(5000)).await;
                // Return to advertising so a subsequent pairing attempt can begin cleanly.
                set_pair_status(BlePairStatus::Advertising);
            }
        }
    };

    // Join the background host packet runner and the peripheral advertising loop.
    let _ = join(runner.run(), pair_loop).await;
}

/// Stack-allocated buffer writer implementing [`core::fmt::Write`] for zero-allocation formatting.
#[cfg(feature = "radio")]
struct BufWriter<'a> {
    /// Target byte buffer storing formatted ASCII characters.
    buf: &'a mut [u8],
    /// Current write position in bytes.
    pos: usize,
}

#[cfg(feature = "radio")]
impl<'a> BufWriter<'a> {
    /// Creates a new buffer writer referencing the destination slice.
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Returns the formatted content as a string slice borrowed from the writer.
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap_or("")
    }

    /// Consumes the writer and returns the formatted content with lifetime `'a`.
    fn finish(self) -> &'a str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap_or("")
    }
}

#[cfg(feature = "radio")]
impl<'a> core::fmt::Write for BufWriter<'a> {
    /// Copies bytes from `s` into `buf` up to the slice capacity.
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remain = self.buf.len().saturating_sub(self.pos);
        let to_copy = bytes.len().min(remain);
        self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;
        Ok(())
    }
}

/// Translates an optional [`AuthenticationMethod`] into a concise display token.
#[cfg(feature = "radio")]
fn auth_str(auth: Option<AuthenticationMethod>) -> &'static str {
    match auth {
        Some(AuthenticationMethod::None) => "Open",
        Some(AuthenticationMethod::Wep) => "WEP",
        Some(AuthenticationMethod::Wpa) => "WPA",
        Some(AuthenticationMethod::Wpa2Personal) => "WPA2",
        Some(AuthenticationMethod::WpaWpa2Personal) => "WPA/WPA2",
        Some(AuthenticationMethod::Wpa2Enterprise) => "WPA2-Ent",
        Some(AuthenticationMethod::Wpa3Personal) => "WPA3",
        Some(AuthenticationMethod::Wpa2Wpa3Personal) => "WPA2/WPA3",
        _ => "Secured",
    }
}

/// Sets the shared atomic Wi-Fi mode and increments the state revision counter.
#[cfg(feature = "radio")]
fn set_wifi_mode(mode: WifiMode) {
    let val = match mode {
        WifiMode::Idle => 0,
        WifiMode::SurveyScanning => 1,
        WifiMode::SurveyComplete => 2,
        WifiMode::Hotspot => 3,
    };
    WIFI_MODE.store(val, Ordering::Release);
    WIFI_STATE_REV.fetch_add(1, Ordering::Release);
}

/// Serializes system telemetry and Wi-Fi state into a compact JSON string without heap allocations.
#[cfg(feature = "radio")]
fn build_status_json(buf: &mut [u8], req_count: u32) -> &str {
    let mut writer = BufWriter::new(buf);
    let charge = crate::touch_bus::last_charge();
    let vbat = charge.map(|c| c.vbat).unwrap_or(0);
    let pct = m5stack_papermono_lite::pmic::battery_percent(vbat);
    let charging = charge.map(|c| c.chg_en).unwrap_or(false);
    let src = charge.map(|c| c.src).unwrap_or(0);
    let lamp = crate::touch_bus::last_lamp().unwrap_or(0);
    let clients = AP_CLIENTS.load(Ordering::Relaxed);
    #[cfg(feature = "panel")]
    let btn_a = u8::from(crate::share::BTN_A.load(Ordering::Relaxed));
    #[cfg(not(feature = "panel"))]
    let btn_a = 0u8;
    #[cfg(feature = "panel")]
    let btn_b = u8::from(crate::share::BTN_B.load(Ordering::Relaxed));
    #[cfg(not(feature = "panel"))]
    let btn_b = 0u8;
    #[cfg(feature = "panel")]
    let scene = crate::share::last_scene()
        .map(|s| s.as_str())
        .unwrap_or("unknown");
    #[cfg(not(feature = "panel"))]
    let scene = "unknown";

    let _ = write!(
        writer,
        "{{\"device\":\"PaperMono\",\"sku\":\"C153-Lite\",\"scene\":\"{scene}\",\"buttons\":{{\"btn_a\":{btn_a},\"btn_b\":{btn_b}}},\"battery\":{{\"vbat_mv\":{vbat},\"percent\":{pct},\"src\":\"{src:02x}\",\"charging\":{charging}}},\"lamp\":{lamp},\"wifi\":{{\"hotspot\":true,\"ssid\":\"{AP_SSID}\",\"clients\":{clients},\"requests\":{req_count}}}}}",
    );
    writer.finish()
}

/// Aggregates scanned access point metrics into channel distributions and top AP rankings.
#[cfg(feature = "radio")]
fn process_survey_results(aps: &[esp_radio::wifi::ap::AccessPointInfo]) {
    let mut ch1_count = 0u16;
    let mut ch6_count = 0u16;
    let mut ch11_count = 0u16;
    let mut other_count = 0u16;

    for ap in aps {
        match ap.channel {
            1 => ch1_count = ch1_count.saturating_add(1),
            6 => ch6_count = ch6_count.saturating_add(1),
            11 => ch11_count = ch11_count.saturating_add(1),
            _ => other_count = other_count.saturating_add(1),
        }
    }

    let mut indices = [0usize; WIFI_MAX];
    let count = aps.len().min(WIFI_MAX);
    for (i, slot) in indices.iter_mut().enumerate().take(count) {
        *slot = i;
    }
    indices[..count].sort_by(|&a, &b| aps[b].signal_strength.cmp(&aps[a].signal_strength));

    let mut top_aps = [None; 4];
    for (i, &idx) in indices[..count.min(4)].iter().enumerate() {
        let ap = &aps[idx];
        let mut ssid_buf = [0u8; 18];
        let bytes = ap.ssid.as_str().as_bytes();
        let len = bytes.len().min(18);
        ssid_buf[..len].copy_from_slice(&bytes[..len]);
        top_aps[i] = Some(SurveyApEntry {
            ssid: ssid_buf,
            ssid_len: len as u8,
            channel: ap.channel,
            rssi: ap.signal_strength,
            auth: auth_str(ap.auth_method),
        });
    }

    let total = count as u16;
    let survey_data = WifiSurveyData {
        total_aps: total,
        ch1_count,
        ch6_count,
        ch11_count,
        other_count,
        top_aps,
    };
    SURVEY_DATA.lock(|cell| *cell.borrow_mut() = Some(survey_data));
    store_wifi(total);

    let mut log_buf = [0u8; 96];
    let mut writer = BufWriter::new(&mut log_buf);
    let _ = write!(
        writer,
        "count={total} ch1={ch1_count} ch6={ch6_count} ch11={ch11_count} other={other_count}"
    );
    cdc::wifi_survey(writer.as_str());
    WIFI_STATE_REV.fetch_add(1, Ordering::Release);
}

/// Processes an incoming UI interaction command and transitions the controller configuration.
#[cfg(feature = "radio")]
async fn handle_command(cmd: WifiCommand, controller: &mut WifiController<'static>) {
    match cmd {
        WifiCommand::StartSurvey => {
            if HOTSPOT_ACTIVE.load(Ordering::Relaxed) {
                HOTSPOT_ACTIVE.store(false, Ordering::Release);
                AP_CLIENTS.store(0, Ordering::Relaxed);
                let _ = controller.set_config(&Config::Station(StationConfig::default()));
                cdc::wifi_ap("state=stopped");
            }
            set_wifi_mode(WifiMode::SurveyScanning);
        }
        WifiCommand::StopSurvey => {
            set_wifi_mode(WifiMode::Idle);
        }
        WifiCommand::StartHotspot => {
            set_wifi_mode(WifiMode::Hotspot);
        }
        WifiCommand::StopHotspot => {
            if HOTSPOT_ACTIVE.load(Ordering::Relaxed) {
                HOTSPOT_ACTIVE.store(false, Ordering::Release);
                AP_CLIENTS.store(0, Ordering::Relaxed);
                let _ = controller.set_config(&Config::Station(StationConfig::default()));
                cdc::wifi_ap("state=stopped");
            }
            set_wifi_mode(WifiMode::Idle);
        }
    }
}

/// Asynchronous Embassy task driving packet processing for the embassy-net network stack.
#[cfg(feature = "radio")]
#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}

/// Asynchronous Embassy task providing IPv4 DHCP lease distribution to connected stations.
#[cfg(feature = "radio")]
#[embassy_executor::task]
pub async fn dhcp_task(stack: Stack<'static>) {
    let buffers = UDP_BUFFERS.init(UdpBuffers::new());
    let unbound = Udp::new(stack, buffers);
    let Ok(mut socket) = unbound
        .bind(core::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            DEFAULT_SERVER_PORT,
        )))
        .await
    else {
        return;
    };

    let mut buf = [0u8; 1500];
    let mut gw_buf = [Ipv4Addr::UNSPECIFIED];

    loop {
        if !HOTSPOT_ACTIVE.load(Ordering::Relaxed) {
            Timer::after(Duration::from_millis(200)).await;
            continue;
        }
        let _ = io::server::run(
            &mut DhcpServer::<_, 64>::new_with_et(AP_IP),
            &ServerOptions::new(AP_IP, Some(&mut gw_buf)),
            &mut socket,
            &mut buf,
        )
        .await;
        Timer::after(Duration::from_millis(200)).await;
    }
}

/// Asynchronous Embassy task serving HTTP GET requests on TCP port 80 with live JSON system stats.
#[cfg(feature = "radio")]
#[embassy_executor::task]
pub async fn http_task(stack: Stack<'static>) {
    let mut rx_buf = [0u8; 1024];
    let mut tx_buf = [0u8; 1024];
    let mut socket = TcpSocket::new(stack, &mut rx_buf, &mut tx_buf);
    socket.set_timeout(Some(Duration::from_secs(5)));

    loop {
        if !HOTSPOT_ACTIVE.load(Ordering::Relaxed) {
            Timer::after(Duration::from_millis(200)).await;
            continue;
        }
        let accept_res = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 80,
            })
            .await;
        if accept_res.is_err() {
            Timer::after(Duration::from_millis(100)).await;
            continue;
        }

        // Read HTTP request header
        let mut req_buf = [0u8; 512];
        let mut n = 0;
        while n < req_buf.len() {
            match socket.read(&mut req_buf[n..]).await {
                Ok(0) | Err(_) => break,
                Ok(read_len) => {
                    n += read_len;
                    if req_buf[..n].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
            }
        }
        let req_str = core::str::from_utf8(&req_buf[..n]).unwrap_or("");
        let path = req_str.split_whitespace().nth(1).unwrap_or("/");

        let req_count = HTTP_REQUESTS.fetch_add(1, Ordering::Relaxed) + 1;
        WIFI_STATE_REV.fetch_add(1, Ordering::Release);
        cdc::wifi_http(req_count, path);

        let mut json_buf = [0u8; 512];
        let json_str = build_status_json(&mut json_buf, req_count);

        let mut header_buf = [0u8; 128];
        let mut writer = BufWriter::new(&mut header_buf);
        let _ = write!(
            writer,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            json_str.len()
        );
        let header_str = writer.as_str();

        let _ = socket.write_all(header_str.as_bytes()).await;
        let _ = socket.write_all(json_str.as_bytes()).await;
        let _ = socket.flush().await;

        Timer::after(Duration::from_millis(50)).await;
        socket.close();
        Timer::after(Duration::from_millis(50)).await;
        socket.abort();
    }
}

/// Central state machine orchestrator managing channel surveys and SoftAP network lifecycle.
#[cfg(feature = "radio")]
#[embassy_executor::task]
pub async fn wifi_manager_task(mut controller: WifiController<'static>) {
    // 1. Conduct initial boot-time passive survey so `last_wifi()` is populated for telemetry.
    let scan_cfg = WifiScanConfig::default()
        .with_max(WIFI_MAX)
        .with_show_hidden(true)
        .with_scan_type(ScanTypeConfig::Passive(
            esp_hal::time::Duration::from_millis(WIFI_PASSIVE_MS),
        ));
    if let Ok(aps) = controller.scan_async(&scan_cfg).await {
        process_survey_results(&aps);
        set_wifi_mode(WifiMode::SurveyComplete);
    } else {
        set_wifi_mode(WifiMode::Idle);
    }

    loop {
        match wifi_mode() {
            WifiMode::Idle | WifiMode::SurveyComplete => {
                let cmd = WIFI_CMD.receive().await;
                handle_command(cmd, &mut controller).await;
            }
            WifiMode::SurveyScanning => {
                let _ = controller.set_config(&Config::Station(StationConfig::default()));
                let scan_cfg = WifiScanConfig::default()
                    .with_max(WIFI_MAX)
                    .with_show_hidden(true)
                    .with_scan_type(ScanTypeConfig::Passive(
                        esp_hal::time::Duration::from_millis(WIFI_PASSIVE_MS),
                    ));

                match select(controller.scan_async(&scan_cfg), WIFI_CMD.receive()).await {
                    Either::First(Ok(aps)) => {
                        process_survey_results(&aps);
                        set_wifi_mode(WifiMode::SurveyComplete);
                    }
                    Either::First(Err(_)) => {
                        set_wifi_mode(WifiMode::Idle);
                    }
                    Either::Second(cmd) => {
                        handle_command(cmd, &mut controller).await;
                    }
                }
            }
            WifiMode::Hotspot => {
                let ap_cfg = Config::AccessPoint(
                    AccessPointConfig::default()
                        .with_ssid(AP_SSID.try_into().unwrap())
                        .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                            AP_PASSWORD.try_into().unwrap(),
                        )),
                );
                let _ = controller.set_config(&ap_cfg);
                HOTSPOT_ACTIVE.store(true, Ordering::Release);
                AP_CLIENTS.store(0, Ordering::Relaxed);
                cdc::wifi_ap("state=active ssid=PaperMono-AP pass=mono2026 ip=192.168.4.1");
                WIFI_STATE_REV.fetch_add(1, Ordering::Release);

                loop {
                    match select(
                        controller.wait_for_access_point_connected_event_async(),
                        WIFI_CMD.receive(),
                    )
                    .await
                    {
                        Either::First(Ok(event)) => match event {
                            esp_radio::wifi::ap::EventInfo::Connected(_) => {
                                let c = AP_CLIENTS.fetch_add(1, Ordering::Relaxed) + 1;
                                let mut buf = [0u8; 48];
                                let mut w = BufWriter::new(&mut buf);
                                let _ = write!(w, "client=connected count={c}");
                                cdc::wifi_ap(w.as_str());
                                WIFI_STATE_REV.fetch_add(1, Ordering::Release);
                            }
                            esp_radio::wifi::ap::EventInfo::Disconnected(_) => {
                                let c = AP_CLIENTS
                                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                                        Some(val.saturating_sub(1))
                                    })
                                    .unwrap_or(0);
                                let mut buf = [0u8; 48];
                                let mut w = BufWriter::new(&mut buf);
                                let _ = write!(w, "client=disconnected count={c}");
                                cdc::wifi_ap(w.as_str());
                                WIFI_STATE_REV.fetch_add(1, Ordering::Release);
                            }
                        },
                        Either::First(Err(_)) => {
                            Timer::after(Duration::from_millis(500)).await;
                        }
                        Either::Second(cmd) => {
                            HOTSPOT_ACTIVE.store(false, Ordering::Release);
                            AP_CLIENTS.store(0, Ordering::Relaxed);
                            let _ =
                                controller.set_config(&Config::Station(StationConfig::default()));
                            cdc::wifi_ap("state=stopped");
                            WIFI_STATE_REV.fetch_add(1, Ordering::Release);

                            handle_command(cmd, &mut controller).await;
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Initializes the Wi-Fi subsystem, network driver, DHCP server, and HTTP status daemon.
///
/// Spawns the embassy-net runner, DHCP lease server, JSON HTTP status daemon, and the
/// mutual-exclusion Wi-Fi manager that owns the SoftAP / channel-survey state machine.
/// Only compiled when the `radio` feature is enabled (called from `main` under the same gate).
#[cfg(feature = "radio")]
pub fn init_wifi(wifi: WIFI<'static>, spawner: Spawner) {
    let wifi_ap_device = Interface::access_point();
    let Ok(controller) = WifiController::new(
        wifi,
        ControllerConfig::default().with_initial_config(Config::Station(StationConfig::default())),
    ) else {
        return;
    };

    let ap_net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(AP_IP, 24),
        gateway: Some(AP_IP),
        dns_servers: Default::default(),
    });
    let seed = 0xA5A5_5A5A_1234_5678;
    let stack_res = STACK_RESOURCES.init(StackResources::new());
    let (stack, runner) = embassy_net::new(wifi_ap_device, ap_net_config, stack_res, seed);

    spawner.spawn(net_task(runner).unwrap());
    spawner.spawn(dhcp_task(stack).unwrap());
    spawner.spawn(http_task(stack).unwrap());
    spawner.spawn(wifi_manager_task(controller).unwrap());
}
