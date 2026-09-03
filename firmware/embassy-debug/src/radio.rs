//! Passive Wi-Fi beacon listening and BLE advertisement scanning without association.
//!
//! # Architecture & Privacy Safety
//! This module coordinates passive radio listening:
//!
//! - **No Association / Connection**: Operates purely as a passive observer; does not
//!   attempt Wi-Fi authentication or Bluetooth pairing.
//! - **Privacy Preservation**: Never logs or emits MAC addresses, SSIDs, BSSIDs, or
//!   identity resolving keys (IRKs) over CDC. Only emits aggregate counts (`wifi n=`, `ble n=`).
//! - **No NVS Storage Writes**: Operates without modifying non-volatile storage flash
//!   sectors (`nvs_enable` disabled in the driver), ensuring factory RF calibration data
//!   remains uncorrupted.
//! - **Coexistence Sequencing**: BLE controller is brought up and executed before
//!   triggering Wi-Fi scans to satisfy ESP32-S3 hardware RF coexistence scheduling.

use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use bt_hci::cmd::le::LeSetScanParams;
use bt_hci::controller::ControllerCmdSync;
use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::{BT, WIFI};
use esp_radio::ble::controller::BleConnector;
use esp_radio::wifi::scan::{ScanConfig as WifiScanConfig, ScanTypeConfig};
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{Config, ControllerConfig, Interface, WifiController};
use trouble_host::prelude::*;

use crate::cdc;

/// Duration of the BLE passive advertisement listening window in seconds.
const BLE_WINDOW_S: u64 = 8;

/// Timeout after which a stalled Wi-Fi scan is aborted.
const WIFI_TIMEOUT_S: u64 = 20;

/// Maximum number of access points to buffer in memory during scan.
const WIFI_MAX: usize = 32;

/// Passive listening dwell time per Wi-Fi 2.4 GHz channel in milliseconds.
const WIFI_PASSIVE_MS: u64 = 150;

static WIFI_N: AtomicU16 = AtomicU16::new(0);
static BLE_N: AtomicU16 = AtomicU16::new(0);
static HAVE_WIFI: AtomicBool = AtomicBool::new(false);
static HAVE_BLE: AtomicBool = AtomicBool::new(false);

/// BLE advertisement packet handler counting valid incoming reports.
struct CountAdv {
    n: AtomicU16,
}

impl EventHandler for CountAdv {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        while let Some(Ok(_)) = it.next() {
            let _ = self
                .n
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                    Some(v.saturating_add(1))
                });
        }
    }
}

fn store_wifi(n: u16) {
    WIFI_N.store(n, Ordering::Relaxed);
    HAVE_WIFI.store(true, Ordering::Relaxed);
    cdc::wifi(n);
}

fn store_ble(n: u16) {
    BLE_N.store(n, Ordering::Relaxed);
    HAVE_BLE.store(true, Ordering::Relaxed);
    cdc::ble(n);
}

/// Retrieves the count of observed Wi-Fi beacons for periodic banner reporting.
pub fn last_wifi() -> Option<u16> {
    HAVE_WIFI
        .load(Ordering::Relaxed)
        .then(|| WIFI_N.load(Ordering::Relaxed))
}

/// Retrieves the count of observed BLE advertisement packets for periodic banner reporting.
pub fn last_ble() -> Option<u16> {
    HAVE_BLE
        .load(Ordering::Relaxed)
        .then(|| BLE_N.load(Ordering::Relaxed))
}

/// Asynchronous Embassy task driving Bluetooth Low Energy passive scanning.
#[embassy_executor::task]
pub async fn ble_run(bt: BT<'static>) {
    store_ble(ble_count(bt).await);
}

/// Asynchronous Embassy task driving Wi-Fi passive channel sweeping.
#[embassy_executor::task]
pub async fn wifi_run(wifi: WIFI<'static>) {
    store_wifi(wifi_count(wifi).await);
}

/// Conducts a passive Wi-Fi scan and returns the number of unique APs discovered.
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

/// Configures BLE controller and initiates passive scanning.
async fn ble_count(bt: BT<'static>) -> u16 {
    let Ok(connector) = BleConnector::new(bt, Default::default()) else {
        return 0;
    };
    let controller: ExternalController<_, 1> = ExternalController::new(connector);
    ble_window(controller).await
}

/// Runs a trouble-host BLE scan session for [`BLE_WINDOW_S`] seconds.
async fn ble_window<C>(controller: C) -> u16
where
    C: Controller + ControllerCmdSync<LeSetScanParams>,
{
    let address = Address::random([0x42, 0x00, 0x00, 0x00, 0x00, 0x01]);
    let mut resources: HostResources<_, DefaultPacketPool, 1, 1> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(address)
        .build();
    let mut runner = stack.runner();
    let central = stack.central();
    let counter = CountAdv {
        n: AtomicU16::new(0),
    };
    let mut scanner = Scanner::new(central);
    let _ = select(runner.run_with_handler(&counter), async {
        let config = ScanConfig {
            active: false,
            phys: PhySet::M1,
            interval: Duration::from_millis(100),
            window: Duration::from_millis(100),
            ..Default::default()
        };
        let _session = scanner.scan(&config).await.ok();
        Timer::after(Duration::from_secs(BLE_WINDOW_S)).await;
    })
    .await;
    counter.n.load(Ordering::Relaxed)
}
