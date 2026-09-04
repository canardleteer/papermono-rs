//! Inter-task atomic state sharing between the UI task and telemetry background worker.
//!
//! # Architecture & Synchronization
//! In Embassy, peripheral pins like pushbuttons (`GPIO2`, `GPIO3`), touch interrupt
//! (`GPIO4`), and display busy (`GPIO18`) cannot be simultaneously owned by two
//! independent tasks without runtime borrow conflicts.
//!
//! To solve this cleanly without mutex overhead or blocking locks:
//! - **The `ui` task** owns the physical pin drivers (`esp_hal::gpio::Input`),
//!   sampling them at high frequency (10 ms) for snappy touch/gesture response.
//! - **This module** exposes lock-free atomic registers (`AtomicBool`, `AtomicU8`)
//!   that mirror the instantaneous pin levels and active card view.
//! - **The `heartbeat` task** reads these relaxed atomics to periodically emit
//!   `heartbeat`, `gpio`, and `scene` telemetry over CDC without contending for
//!   hardware pin ownership.

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use papermono_log::Scene;

/// Mirror of BUTTON A (`GPIO2` / UP) logic level. `true` indicates high (unpressed).
pub static BTN_A: AtomicBool = AtomicBool::new(true);

/// Mirror of BUTTON B (`GPIO3` / DOWN) logic level. `true` indicates high (unpressed).
pub static BTN_B: AtomicBool = AtomicBool::new(true);

/// Mirror of FT6336G capacitive touch interrupt (`GPIO4`). `true` indicates idle (high).
pub static TP: AtomicBool = AtomicBool::new(true);

/// Mirror of SSD1677 e-paper controller `BUSY` signal (`GPIO18`). `true` indicates refreshing.
pub static BUSY: AtomicBool = AtomicBool::new(false);

/// Liveness indicator indicating whether the interactive eight-card UI task is running.
pub static UI_LIVE: AtomicBool = AtomicBool::new(false);

/// Encoded identifier of the currently displayed UI scene.
static SCENE: AtomicU8 = AtomicU8::new(0);

/// Flag tracking whether an initial UI scene has been rendered.
static HAVE_SCENE: AtomicBool = AtomicBool::new(false);

/// Stores the active UI scene in the shared atomic register and flags it as valid.
pub fn store_scene(scene: Scene) {
    SCENE.store(scene_byte(scene), Ordering::Relaxed);
    HAVE_SCENE.store(true, Ordering::Relaxed);
}

/// Retrieves the most recently rendered UI scene, or `None` if no card has been drawn.
pub fn last_scene() -> Option<Scene> {
    if !HAVE_SCENE.load(Ordering::Relaxed) {
        return None;
    }
    from_scene_byte(SCENE.load(Ordering::Relaxed))
}

/// Converts a high-level UI [`Scene`] enum into a compact 1-byte representation for atomic storage.
const fn scene_byte(scene: Scene) -> u8 {
    match scene {
        Scene::Splash => 0,
        Scene::Shapes => 1,
        Scene::Legend => 2,
        Scene::Bluetooth => 3,
        Scene::WifiSurvey => 4,
        Scene::WifiAp => 5,
        Scene::Tones => 6,
        Scene::Targets => 7,
    }
}

/// Decodes a compact 1-byte representation back into a high-level UI [`Scene`] enum.
const fn from_scene_byte(byte: u8) -> Option<Scene> {
    match byte {
        0 => Some(Scene::Splash),
        1 => Some(Scene::Shapes),
        2 => Some(Scene::Legend),
        3 => Some(Scene::Bluetooth),
        4 => Some(Scene::WifiSurvey),
        5 => Some(Scene::WifiAp),
        6 => Some(Scene::Tones),
        7 => Some(Scene::Targets),
        _ => None,
    }
}
