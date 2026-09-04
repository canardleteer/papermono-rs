//! Six-card interactive UI state machine and navigation controller.
//!
//! # Architecture & Navigation Model
//! This module coordinates the high-level interactive user experience:
//!
//! - **Six-Card Finite State Machine**: Cycles sequentially through the UI scenes:
//!   `Splash` ↔ `Shapes` ↔ `Legend` ↔ `Bluetooth` ↔ `Tones` ↔ `Targets`.
//! - **Physical Button Controls**:
//!   - `BUTTON A` (`GPIO2`): Short press switches to previous card. Long press (>2 s)
//!     triggers low-power sleep; long press (~1 s) triggers an audio recording / tone test
//!     when the `mic` feature is enabled.
//!   - `BUTTON B` (`GPIO3`): Short press advances to the next card. During sleep, holding
//!     either `BUTTON A` or `BUTTON B` for 1 s wakes the device.
//! - **Touch Gutter Gesture**:
//!   - Swiping along the far-right edge of the screen dynamically adjusts the display
//!     frontlight LED brightness via PWM without advancing cards.
//! - **Automatic Telemetry Refresh**:
//!   - When stationary on the `Legend` card, the UI automatically refreshes the display
//!     every 60 seconds with updated battery state-of-charge, voltage, and USB power status.
//! - **Dynamic Bluetooth Pairing Refresh**:
//!   - When stationary on the `Bluetooth` card, any pairing state transition (e.g. phone
//!     connection, passkey display, pairing success, or failure) immediately triggers a fast
//!     monochromatic refresh to update the PIN and status on screen.
//! - **Zero Heap Allocation**: Framebuffers are statically allocated using
//!   [`static_cell::ConstStaticCell`], eliminating heap usage while retaining 480×800
//!   framebuffers in BSS memory.

use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::Input;
#[cfg(feature = "sleep")]
use esp_hal::gpio::{Event, WakeupConfig};
use esp_hal::rtc_cntl::sleep::LowPower;
#[cfg(feature = "sleep")]
use esp_hal::rtc_cntl::sleep::RtcSleepConfig;
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

/// Framebuffer container holding dual 1bpp planes for 4-gray rendering.
struct Planes {
    bw: [u8; display::PLANE_BYTES],
    red: [u8; display::PLANE_BYTES],
}

/// Statically allocated framebuffers in BSS.
static PLANES: ConstStaticCell<Planes> = ConstStaticCell::new(Planes {
    bw: [0u8; display::PLANE_BYTES],
    red: [0u8; display::PLANE_BYTES],
});

/// Navigation intent decoded from button presses or auto-refresh timer.
enum Nav {
    /// Navigate to previous card.
    Prev,
    /// Navigate to next card.
    Next,
    /// Auto-refresh the current card with live telemetry.
    Refresh,
    /// Enter low-power sleep mode.
    #[cfg(feature = "sleep")]
    Sleep,
}

/// Button and touch polling interval (10 ms) for high-responsiveness gesture tracking.
const NAV_POLL_MS: u32 = 10;

/// Automatic refresh interval for the Legend card (60 seconds) to update live battery telemetry.
const LEGEND_AUTO_REFRESH_MS: u32 = 60_000;

/// Button A hold duration (2 seconds) to trigger sleep.
#[cfg(feature = "sleep")]
const BUTTON_HOLD_SLEEP_MS: u32 = 2_000;

/// Main asynchronous UI worker task driving card transitions and user interaction.
#[embassy_executor::task]
pub async fn run(
    mut i2c: SysI2c,
    mut panel: Panel,
    #[allow(unused_mut)] mut btn_a: Input<'static>,
    #[allow(unused_mut)] mut btn_b: Input<'static>,
    tp: Input<'static>,
    busy: Input<'static>,
    mut lpwr: LowPower<'static>,
) {
    #[cfg(not(feature = "sleep"))]
    let _ = &mut lpwr;

    let planes = PLANES.take();
    let mut scene = Scene::Splash;
    let mut lamp = LampSlide::new();
    loop {
        paint(&mut i2c, &mut panel, &busy, scene, planes).await;
        if scene == Scene::Targets {
            panel.enter_mono(&mut i2c, &busy).await;
            match targets::walk(&mut i2c, &mut panel, &btn_a, &btn_b, &tp, &busy).await {
                WalkEnd::Done => {
                    if let Some(nav) =
                        wait_nav(&mut i2c, &btn_a, &btn_b, &tp, &mut lamp, None, false).await
                    {
                        match nav {
                            Nav::Prev => scene = scene.prev(),
                            Nav::Next => scene = scene.next(),
                            Nav::Refresh => {}
                            #[cfg(feature = "sleep")]
                            Nav::Sleep => {
                                enter_sleep(
                                    &mut i2c, &mut panel, &busy, planes, &mut btn_a, &mut btn_b,
                                    &mut lpwr,
                                )
                                .await;
                            }
                        }
                    }
                }
                WalkEnd::AbortPrev => scene = scene.prev(),
                WalkEnd::AbortNext => scene = scene.next(),
            }
        } else {
            let auto_refresh_ms = (scene == Scene::Legend).then_some(LEGEND_AUTO_REFRESH_MS);
            let watch_ble = scene == Scene::Bluetooth;
            if let Some(nav) = wait_nav(
                &mut i2c,
                &btn_a,
                &btn_b,
                &tp,
                &mut lamp,
                auto_refresh_ms,
                watch_ble,
            )
            .await
            {
                match nav {
                    Nav::Prev => scene = scene.prev(),
                    Nav::Next => scene = scene.next(),
                    Nav::Refresh => {}
                    #[cfg(feature = "sleep")]
                    Nav::Sleep => {
                        enter_sleep(
                            &mut i2c, &mut panel, &busy, planes, &mut btn_a, &mut btn_b, &mut lpwr,
                        )
                        .await;
                    }
                }
            }
        }
    }
}

/// Renders a card into framebuffers and triggers an e-paper refresh waveform.
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
    let charge = if scene == Scene::Legend {
        Some(touch_bus::refresh_battery(i2c))
    } else {
        touch_bus::last_charge()
    };
    if let Some(us) = draw::render(scene, &mut planes.bw, &mut planes.red, charge) {
        cdc::snowflake(us);
    }
    if scene.uses_gray() {
        panel.paint_gray(i2c, &planes.bw, &planes.red, busy).await;
    } else {
        panel
            .paint_mono_fast(i2c, &planes.bw, &planes.red, busy)
            .await;
    }
}

/// Waits for tactile button presses, right-edge touch slider gestures, auto-refresh timeouts, or BLE state changes.
///
/// # Parameters
/// - `i2c`: System I2C bus driver for sampling the FT6336G capacitive touch controller.
/// - `btn_a`: Input pin driver for Button A (`GPIO2` / UP).
/// - `btn_b`: Input pin driver for Button B (`GPIO3` / DOWN).
/// - `tp`: Touch interrupt line (`GPIO4` / `TOUCH_INT`).
/// - `lamp`: Frontlight slider tracker updating M5PM1 PWM0 duty from capacitive Y coordinates.
/// - `auto_refresh_ms`: Optional timeout triggering periodic automatic card refresh (e.g. 60 s for battery).
/// - `watch_ble`: When `true`, monitors [`crate::radio::state_rev()`] for BLE pairing state transitions
///   and returns [`Some(Nav::Refresh)`] immediately to repaint the e-paper glass.
///
/// # Returns
/// - `Some(Nav::Prev)` on short press of Button A.
/// - `Some(Nav::Next)` on short press of Button B.
/// - `Some(Nav::Sleep)` on 2-second hold of Button A (when `sleep` feature is active).
/// - `Some(Nav::Refresh)` on auto-refresh timeout or BLE status revision change.
async fn wait_nav(
    i2c: &mut SysI2c,
    btn_a: &Input<'static>,
    btn_b: &Input<'static>,
    tp: &Input<'static>,
    lamp: &mut LampSlide,
    auto_refresh_ms: Option<u32>,
    watch_ble: bool,
) -> Option<Nav> {
    let mut prev_a = btn_a.is_high();
    let mut prev_b = btn_b.is_high();
    let mut a_down: Option<Instant> = None;
    let mut a_held = false;
    let mut t_ms = 0_u32;
    let initial_ble_rev = crate::radio::state_rev();
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

        // Detect BUTTON A long press for audio test or sleep.
        if prev_a && !now_a {
            a_down = Some(Instant::now());
            a_held = false;
        }
        if let Some(start) = a_down {
            if !now_a {
                let duration = Instant::now().duration_since(start);
                #[cfg(feature = "sleep")]
                if duration >= Duration::from_millis(BUTTON_HOLD_SLEEP_MS.into()) {
                    return Some(Nav::Sleep);
                }
                if !a_held && duration >= Duration::from_millis(BUTTON_HOLD_PCM_MS.into()) {
                    #[cfg(feature = "mic")]
                    crate::mic::ask_tone();
                    a_held = true;
                }
            }
        }
        if !prev_a && now_a {
            let held = a_held;
            a_down = None;
            a_held = false;
            prev_a = now_a;
            // Short press BUTTON A: previous card.
            if !held {
                return Some(Nav::Prev);
            }
        } else {
            prev_a = now_a;
        }

        // BUTTON B short press: next card.
        if prev_b && !now_b {
            return Some(Nav::Next);
        }
        prev_b = now_b;

        // Poll touch digitizer for right-gutter frontlight brightness slider.
        let int_high = tp.is_high();
        let sample = touch_bus::read_points(i2c, int_high, true);
        if sample.n >= 1 {
            cdc::touch(&sample);
        }
        let _ = lamp.feed(i2c, &sample);

        if watch_ble && crate::radio::state_rev() != initial_ble_rev {
            return Some(Nav::Refresh);
        }

        t_ms = t_ms.saturating_add(NAV_POLL_MS);
        if let Some(timeout) = auto_refresh_ms {
            if t_ms >= timeout {
                return Some(Nav::Refresh);
            }
        }
        Timer::after(Duration::from_millis(NAV_POLL_MS.into())).await;
    }
}

/// Transitions the system into light sleep after drawing the sleep screen, and waits for a 1-second hold on Button A or B to wake.
#[cfg(feature = "sleep")]
async fn enter_sleep(
    i2c: &mut SysI2c,
    panel: &mut Panel,
    busy: &Input<'static>,
    planes: &mut Planes,
    btn_a: &mut Input<'static>,
    btn_b: &mut Input<'static>,
    lpwr: &mut LowPower<'static>,
) {
    // 1. Draw and paint the sleep notice to the e-paper panel.
    draw::draw_sleeping(&mut planes.bw, &mut planes.red);
    panel
        .paint_mono_fast(i2c, &planes.bw, &planes.red, busy)
        .await;

    // 2. Wait until Button A (and Button B) are fully released before arming sleep.
    while btn_a.is_low() || btn_b.is_low() {
        Timer::after(Duration::from_millis(10)).await;
    }
    Timer::after(Duration::from_millis(50)).await;

    // 3. Save active frontlight duty and turn off frontlight and red LED.
    let saved_duty =
        touch_bus::last_lamp().unwrap_or(m5stack_papermono_lite::pmic::FRONTLIGHT_DUTY);
    touch_bus::apply_lamp(i2c, 0);
    touch_bus::apply_red_led(i2c, false);

    // 4. Configure low-power wake paths on Button A (GPIO2) and Button B (GPIO3).
    let config = WakeupConfig::default().with_low_power_path(true);
    let _ = btn_a.apply_wakeup_config(&config);
    let _ = btn_b.apply_wakeup_config(&config);

    // 5. Sleep loop: sleep until low level on Button A or B, then qualify with 1-second hold.
    const WAKE_HOLD_MS: u32 = 1_000;
    const POLL_MS: u32 = 10;
    loop {
        btn_a.listen(Event::LowLevel);
        btn_b.listen(Event::LowLevel);

        lpwr.sleep_light(RtcSleepConfig::default());

        btn_a.unlisten();
        btn_b.unlisten();
        btn_a.clear_interrupt();
        btn_b.clear_interrupt();

        // Evaluate hold duration: must stay low for at least WAKE_HOLD_MS.
        let mut held_ms = 0_u32;
        let mut confirmed = false;
        while btn_a.is_low() || btn_b.is_low() {
            Timer::after(Duration::from_millis(POLL_MS.into())).await;
            held_ms = held_ms.saturating_add(POLL_MS);
            if held_ms >= WAKE_HOLD_MS {
                confirmed = true;
                break;
            }
        }

        if confirmed {
            break;
        }

        // Accidental tap / short release (<1s): wait until buttons are idle high before sleeping again.
        while btn_a.is_low() || btn_b.is_low() {
            Timer::after(Duration::from_millis(10)).await;
        }
        Timer::after(Duration::from_millis(50)).await;
    }

    // 6. Confirmed wake-up: wait for button release so it doesn't trigger card navigation.
    while btn_a.is_low() || btn_b.is_low() {
        Timer::after(Duration::from_millis(10)).await;
    }
    Timer::after(Duration::from_millis(50)).await;

    share::BTN_A.store(true, core::sync::atomic::Ordering::Relaxed);
    share::BTN_B.store(true, core::sync::atomic::Ordering::Relaxed);

    // 7. Restore frontlight duty and red LED.
    touch_bus::apply_lamp(i2c, saved_duty);
    touch_bus::apply_red_led(i2c, true);
}
