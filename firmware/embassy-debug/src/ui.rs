//! Five-card walk: A prev, B next, right-edge lamp, hold-A PCM.

use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::Input;
use m5stack_papermono_lite::display;
use papermono_log::{Edge, Scene, BUTTON_HOLD_PCM_MS};
use static_cell::ConstStaticCell;

use crate::cdc;
use crate::draw;
use crate::ioe::SysI2c;
use crate::panel::Panel;
use crate::share;
use crate::targets::{self, WalkEnd};
use crate::touch_bus::{self, LampSlide};

struct Planes {
    bw: [u8; display::PLANE_BYTES],
    red: [u8; display::PLANE_BYTES],
}

static PLANES: ConstStaticCell<Planes> = ConstStaticCell::new(Planes {
    bw: [0u8; display::PLANE_BYTES],
    red: [0u8; display::PLANE_BYTES],
});

enum Nav {
    Prev,
    Next,
}

/// Faster than the 50 ms heartbeat poll so a gutter stroke is
/// not missed between `/INT` high blips.
const NAV_POLL_MS: u32 = 10;

#[embassy_executor::task]
pub async fn run(
    mut i2c: SysI2c,
    mut panel: Panel,
    btn_a: Input<'static>,
    btn_b: Input<'static>,
    tp: Input<'static>,
    busy: Input<'static>,
) {
    let planes = PLANES.take();
    let mut scene = Scene::Splash;
    let mut lamp = LampSlide::new();
    loop {
        paint(&mut i2c, &mut panel, &busy, scene, planes).await;
        if scene == Scene::Targets {
            panel.enter_mono(&mut i2c, &busy).await;
            match targets::walk(&mut i2c, &mut panel, &btn_a, &btn_b, &tp, &busy, &mut lamp).await {
                WalkEnd::Done => {
                    if let Some(nav) = wait_nav(&mut i2c, &btn_a, &btn_b, &tp, &mut lamp).await {
                        scene = apply(scene, nav);
                    }
                }
                WalkEnd::AbortPrev => scene = scene.prev(),
                WalkEnd::AbortNext => scene = scene.next(),
            }
        } else if let Some(nav) = wait_nav(&mut i2c, &btn_a, &btn_b, &tp, &mut lamp).await {
            scene = apply(scene, nav);
        }
    }
}

async fn paint(
    i2c: &mut SysI2c,
    panel: &mut Panel,
    busy: &Input<'static>,
    scene: Scene,
    planes: &mut Planes,
) {
    share::store_scene(scene);
    cdc::scene(scene);
    if scene == Scene::Targets {
        return;
    }
    draw::render(scene, &mut planes.bw, &mut planes.red);
    if scene.uses_gray() {
        panel.paint_gray(i2c, &planes.bw, &planes.red, busy).await;
    } else {
        panel
            .paint_mono_fast(i2c, &planes.bw, &planes.red, busy)
            .await;
    }
}

fn apply(scene: Scene, nav: Nav) -> Scene {
    match nav {
        Nav::Prev => scene.prev(),
        Nav::Next => scene.next(),
    }
}

async fn wait_nav(
    i2c: &mut SysI2c,
    btn_a: &Input<'static>,
    btn_b: &Input<'static>,
    tp: &Input<'static>,
    lamp: &mut LampSlide,
) -> Option<Nav> {
    let mut prev_a = btn_a.is_high();
    let mut prev_b = btn_b.is_high();
    let mut a_down: Option<Instant> = None;
    let mut a_held = false;
    let mut t_ms = 0_u32;
    loop {
        let now_a = btn_a.is_high();
        let now_b = btn_b.is_high();
        share::BTN_A.store(now_a, core::sync::atomic::Ordering::Relaxed);
        share::BTN_B.store(now_b, core::sync::atomic::Ordering::Relaxed);
        share::TP.store(tp.is_high(), core::sync::atomic::Ordering::Relaxed);

        if now_a != prev_a || now_b != prev_b {
            cdc::edge(&Edge {
                t_ms,
                btn_a: (now_a != prev_a).then_some((prev_a, now_a)),
                btn_b: (now_b != prev_b).then_some((prev_b, now_b)),
            });
        }

        if prev_a && !now_a {
            a_down = Some(Instant::now());
            a_held = false;
        }
        if let Some(start) = a_down {
            if !now_a
                && !a_held
                && Instant::now().duration_since(start)
                    >= Duration::from_millis(BUTTON_HOLD_PCM_MS.into())
            {
                #[cfg(feature = "mic")]
                crate::mic::ask_tone();
                a_held = true;
            }
        }
        if !prev_a && now_a {
            let held = a_held;
            a_down = None;
            a_held = false;
            prev_a = now_a;
            if !held {
                return Some(Nav::Prev);
            }
        } else {
            prev_a = now_a;
        }

        if prev_b && !now_b {
            return Some(Nav::Next);
        }
        prev_b = now_b;

        let int_high = tp.is_high();
        let sample = touch_bus::read_points(i2c, int_high, true);
        if sample.n >= 1 {
            cdc::touch(&sample);
        }
        let _ = lamp.feed(i2c, &sample);

        t_ms = t_ms.saturating_add(NAV_POLL_MS);
        Timer::after(Duration::from_millis(NAV_POLL_MS.into())).await;
    }
}
