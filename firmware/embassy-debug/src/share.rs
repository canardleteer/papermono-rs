//! GPIO / UI bits the heartbeat reprints. UI owns the live pins.

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use papermono_log::Scene;

/// BUTTON A (UP) level. `true` is high (idle).
pub static BTN_A: AtomicBool = AtomicBool::new(true);
/// BUTTON B (DOWN) level. `true` is high (idle).
pub static BTN_B: AtomicBool = AtomicBool::new(true);
/// GPIO4 FT `/INT`. High is idle on Lite.
pub static TP: AtomicBool = AtomicBool::new(true);
/// GPIO18 EPD BUSY.
pub static BUSY: AtomicBool = AtomicBool::new(false);
/// `true` when the five-card task owns A/B.
pub static UI_LIVE: AtomicBool = AtomicBool::new(false);

static SCENE: AtomicU8 = AtomicU8::new(0);
static HAVE_SCENE: AtomicBool = AtomicBool::new(false);

pub fn store_scene(scene: Scene) {
    SCENE.store(scene_byte(scene), Ordering::Relaxed);
    HAVE_SCENE.store(true, Ordering::Relaxed);
}

pub fn last_scene() -> Option<Scene> {
    if !HAVE_SCENE.load(Ordering::Relaxed) {
        return None;
    }
    from_scene_byte(SCENE.load(Ordering::Relaxed))
}

const fn scene_byte(scene: Scene) -> u8 {
    match scene {
        Scene::Splash => 0,
        Scene::Shapes => 1,
        Scene::Legend => 2,
        Scene::Tones => 3,
        Scene::Targets => 4,
    }
}

const fn from_scene_byte(byte: u8) -> Option<Scene> {
    match byte {
        0 => Some(Scene::Splash),
        1 => Some(Scene::Shapes),
        2 => Some(Scene::Legend),
        3 => Some(Scene::Tones),
        4 => Some(Scene::Targets),
        _ => None,
    }
}
