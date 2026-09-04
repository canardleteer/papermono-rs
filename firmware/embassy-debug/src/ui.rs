//! Eight-card interactive UI state machine and navigation controller.
//!
//! # Architecture & Navigation Model
//! This module coordinates the high-level interactive user experience:
//!
//! - **Eight-Card Finite State Machine**: Cycles sequentially through the UI scenes:
//!   `Splash` ↔ `Shapes` ↔ `Legend` ↔ `Bluetooth` ↔ `WifiSurvey` ↔ `WifiAp` ↔ `Tones` ↔ `Targets`.
//! - **Physical Button Controls**:
//!   - `BUTTON A` (`GPIO2`): Short press (release edge) switches to previous card.
//!     Long press (>2 s) triggers low-power sleep; long press (~1 s) triggers an
//!     audio recording / tone test when the `mic` feature is enabled.
//!   - `BUTTON B` (`GPIO3`): Short press (release edge) advances to the next card.
//!     During sleep, holding either `BUTTON A` or `BUTTON B` for 1 s wakes the device.
//!   - After each paint, `wait_nav` waits until both buttons are high before arming
//!     edges (holds through slow Shapes paint must not eat the next card).
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
//! - **Interactive On-Screen Touch Buttons**:
//!   - On `WifiSurvey`, tapping `[ START SURVEY ]` / `[ STOP SURVEY ]` triggers or halts 2.4 GHz channel scanning.
//!   - On `WifiAp`, tapping `[ START HOTSPOT ]` / `[ STOP HOTSPOT ]` activates or disables the WPA2-Personal AP.
//! - **Dynamic Wi-Fi Telemetry Refresh**:
//!   - When stationary on `WifiSurvey` or `WifiAp`, any state transition (scan completion, client connection,
//!     or HTTP GET request) immediately triggers a fast refresh to update the display.
//! - **IMU page rotation** (`orient` feature): BMI270 dominant-axis classify (sticky-rs
//!   policy) remaps the current card into portrait/landscape page space. Face-up /
//!   face-down keep the last in-plane page. Lamp gutter stays physical USB-down.
//! - **Soft same-card redraws**: Bluetooth, Wi-Fi survey/hotspot, and Legend
//!   status updates reuse OTP Partial when a mono baseline exists, even after
//!   the usual partial budget, so PIN / AP / battery telemetry does not flash a
//!   full mono wipe. Same-card **orientation** remaps also stay soft (Partial).
//!   Card navigation still takes `MonoFull` once the budget is reached
//!   (DC-balance).
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
use m5stack_papermono_lite::display::{self, PageRotation};
#[cfg(feature = "orient")]
use m5stack_papermono_lite::imu;
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

/// IMU poll period while waiting on a card (sticky-rs uses 250 ms).
#[cfg(feature = "orient")]
const IMU_POLL_MS: u32 = 250;

/// How often to print `imu pose=` while holding still (sticky-rs uses 5 s).
#[cfg(feature = "orient")]
const IMU_REPORT_MS: u32 = 5_000;

/// Consecutive IMU polls that must agree on a new page before remapping.
///
/// Avoids a one-sample chatter (hand torque on a button press) from stealing
/// the next card edge as a soft orientation refresh.
#[cfg(feature = "orient")]
const IMU_STABLE_POLLS: u8 = 3;

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
    let mut last_painted: Option<Scene> = None;
    let mut last_rotation: Option<PageRotation> = None;
    let mut rotation = PageRotation::Portrait0;
    let mut lamp = LampSlide::new();

    #[cfg(feature = "orient")]
    {
        let _ = imu::soft_reset(&mut i2c);
        Timer::after(Duration::from_millis(30)).await;
        let _ = imu::disable_adv_power_save(&mut i2c);
        Timer::after(Duration::from_millis(1)).await;
        match imu::load_config(&mut i2c) {
            Ok(()) => cdc::imu("cfg-ok", 0, 0, 0),
            Err(_) => cdc::imu("cfg-fail", 0, 0, 0),
        }
        Timer::after(Duration::from_millis(50)).await;
        let status = imu::read_internal_status(&mut i2c).unwrap_or(0);
        cdc::imu("status", i16::from(status), 0, 0);
        let _ = imu::enable_accel_sampling(&mut i2c);
        Timer::after(Duration::from_millis(20)).await;
        // Seed rotation from the first readable sample when possible.
        if let Ok(sample) = imu::read_accel(&mut i2c) {
            if let Some(pose) = imu::classify(sample.x, sample.y, sample.z) {
                cdc::imu(pose.as_str(), sample.x, sample.y, sample.z);
                if let Some(page) = pose.page_rotation() {
                    rotation = page;
                }
            } else {
                cdc::imu("none", sample.x, sample.y, sample.z);
            }
        }
    }

    loop {
        let same_scene = last_painted == Some(scene);
        let orient_changed = last_rotation.is_some_and(|r| r != rotation);
        let soft = same_scene
            && (orient_changed
                || (last_rotation == Some(rotation) && scene_allows_soft_refresh(scene)));
        let drawn_revs = paint(&mut i2c, &mut panel, &busy, scene, planes, soft, rotation).await;
        last_painted = Some(scene);
        last_rotation = Some(rotation);
        if scene == Scene::Targets {
            panel.enter_mono(&mut i2c, &busy).await;
            match targets::walk(&mut i2c, &mut panel, &btn_a, &btn_b, &tp, &busy).await {
                WalkEnd::Done => {
                    let ctx = NavContext {
                        scene,
                        auto_refresh_ms: None,
                        ble_watch_rev: None,
                        wifi_watch_rev: None,
                        rotation,
                    };
                    if let Some(nav) =
                        wait_nav(&mut i2c, &btn_a, &btn_b, &tp, &mut lamp, ctx, &mut rotation).await
                    {
                        match nav {
                            Nav::Prev => scene = scene.prev(),
                            Nav::Next => scene = scene.next(),
                            Nav::Refresh => {}
                            #[cfg(feature = "sleep")]
                            Nav::Sleep => {
                                enter_sleep(
                                    &mut i2c, &mut panel, &busy, planes, &mut btn_a, &mut btn_b,
                                    &mut lpwr, rotation,
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
            let ble_watch_rev = (scene == Scene::Bluetooth).then_some(drawn_revs.ble);
            let wifi_watch_rev =
                (scene == Scene::WifiSurvey || scene == Scene::WifiAp).then_some(drawn_revs.wifi);
            let ctx = NavContext {
                scene,
                auto_refresh_ms,
                ble_watch_rev,
                wifi_watch_rev,
                rotation,
            };
            if let Some(nav) =
                wait_nav(&mut i2c, &btn_a, &btn_b, &tp, &mut lamp, ctx, &mut rotation).await
            {
                match nav {
                    Nav::Prev => scene = scene.prev(),
                    Nav::Next => scene = scene.next(),
                    Nav::Refresh => {}
                    #[cfg(feature = "sleep")]
                    Nav::Sleep => {
                        enter_sleep(
                            &mut i2c, &mut panel, &busy, planes, &mut btn_a, &mut btn_b, &mut lpwr,
                            rotation,
                        )
                        .await;
                    }
                }
            }
        }
    }
}

/// Contextual configuration and dynamic telemetry watch revisions for card navigation polling.
struct NavContext {
    scene: Scene,
    auto_refresh_ms: Option<u32>,
    ble_watch_rev: Option<u32>,
    wifi_watch_rev: Option<u32>,
    /// Current in-plane page used for touch hit-testing.
    rotation: PageRotation,
}

/// State revision snapshot observed before rendering begins, tracking asynchronous radio events.
struct DrawnRevs {
    ble: u32,
    wifi: u32,
}

/// Same-card telemetry scenes that should stay on OTP Partial for live redraws.
const fn scene_allows_soft_refresh(scene: Scene) -> bool {
    matches!(
        scene,
        Scene::Legend | Scene::Bluetooth | Scene::WifiSurvey | Scene::WifiAp
    )
}

/// Renders a card into framebuffers and triggers an e-paper refresh waveform.
///
/// Returns the BLE and Wi-Fi state revisions observed before rendering began, allowing
/// caller tasks to detect if asynchronous radio events arrived during the panel refresh.
///
/// When `soft` is true (same-card Bluetooth / Wi-Fi / Legend status update, or
/// same-card orientation remap), the mono path prefers OTP Partial even after
/// the usual partial budget so a black-and-white redraw does not flash
/// `MonoFull`. Card navigation passes `soft = false`.
async fn paint(
    i2c: &mut SysI2c,
    panel: &mut Panel,
    busy: &Input<'static>,
    scene: Scene,
    planes: &mut Planes,
    soft: bool,
    rotation: PageRotation,
) -> DrawnRevs {
    let ble_rev = crate::radio::state_rev();
    let wifi_rev = crate::radio::wifi_state_rev();
    share::store_scene(scene);
    cdc::scene(scene);
    if scene == Scene::Targets {
        return DrawnRevs {
            ble: ble_rev,
            wifi: wifi_rev,
        };
    }
    let charge = if scene == Scene::Legend {
        Some(touch_bus::refresh_battery(i2c))
    } else {
        touch_bus::last_charge()
    };
    if let Some(us) = draw::render(scene, &mut planes.bw, &mut planes.red, charge, rotation) {
        cdc::snowflake(us);
    }
    if scene.uses_gray() {
        panel.paint_gray(i2c, &planes.bw, &planes.red, busy).await;
    } else {
        panel
            .paint_mono_fast(i2c, &planes.bw, &planes.red, busy, soft)
            .await;
    }
    DrawnRevs {
        ble: ble_rev,
        wifi: wifi_rev,
    }
}

/// Waits for tactile button presses, right-edge touch slider gestures, auto-refresh timeouts, BLE/Wi-Fi state changes, or on-screen touch buttons.
///
/// # Parameters
/// - `i2c`: System I2C bus driver for sampling the FT6336G capacitive touch controller.
/// - `btn_a`: Input pin driver for Button A (`GPIO2` / UP).
/// - `btn_b`: Input pin driver for Button B (`GPIO3` / DOWN).
/// - `tp`: Touch interrupt line (`GPIO4` / `TOUCH_INT`).
/// - `lamp`: Frontlight slider tracker updating M5PM1 PWM0 duty from capacitive Y coordinates.
/// - `ctx`: Polling context holding scene and watched telemetry revisions.
///
/// # Returns
/// - `Some(Nav::Prev)` on short press of Button A (release edge).
/// - `Some(Nav::Next)` on short press of Button B (release edge).
/// - `Some(Nav::Sleep)` on 2-second hold of Button A (when `sleep` feature is active).
/// - `Some(Nav::Refresh)` on auto-refresh timeout, BLE/Wi-Fi status revision change,
///   stable orientation change, or touch button toggle.
///
/// Both buttons must be released before edges are armed. That drops a hold that
/// started during the previous paint (Shapes is slow) so the first post-paint
/// release is not mistaken for a missing press.
async fn wait_nav(
    i2c: &mut SysI2c,
    btn_a: &Input<'static>,
    btn_b: &Input<'static>,
    tp: &Input<'static>,
    lamp: &mut LampSlide,
    ctx: NavContext,
    #[cfg_attr(not(feature = "orient"), allow(unused_variables))] rotation: &mut PageRotation,
) -> Option<Nav> {
    // Drain holds that overlapped the previous EPD paint / snowflake work.
    loop {
        let a = btn_a.is_high();
        let b = btn_b.is_high();
        share::BTN_A.store(a, core::sync::atomic::Ordering::Relaxed);
        share::BTN_B.store(b, core::sync::atomic::Ordering::Relaxed);
        share::TP.store(tp.is_high(), core::sync::atomic::Ordering::Relaxed);
        if a && b {
            break;
        }
        Timer::after(Duration::from_millis(NAV_POLL_MS.into())).await;
    }

    let mut prev_a = true;
    let mut prev_b = true;
    let mut a_down: Option<Instant> = None;
    let mut a_held = false;
    let mut t_ms = 0_u32;
    let mut button_touch_down = false;
    #[cfg(feature = "orient")]
    let mut imu_since_ms = 0_u32;
    #[cfg(feature = "orient")]
    let mut imu_report_ms = 0_u32;
    #[cfg(feature = "orient")]
    let mut pending_page: Option<PageRotation> = None;
    #[cfg(feature = "orient")]
    let mut pending_count = 0_u8;
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
        // Short press BUTTON A / B on release so a hold through paint cannot
        // leave the next wait armed on a stuck low and eat the first press.
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

        if !prev_b && now_b {
            return Some(Nav::Next);
        }
        prev_b = now_b;

        // Poll touch digitizer for right-gutter frontlight brightness slider and on-screen buttons.
        let int_high = tp.is_high();
        let sample = touch_bus::read_points(i2c, int_high, true);
        if sample.n >= 1 {
            cdc::touch(&sample);
        }
        let _ = lamp.feed(i2c, &sample);

        // Wi-Fi action button is laid out in page space; map physical touch → page.
        let in_button = if sample.n >= 1 {
            display::framebuffer_to_page(sample.x, sample.y, ctx.rotation)
                .is_some_and(|(px, py)| draw::wifi_action_hit(px, py, ctx.rotation))
        } else {
            false
        };
        if in_button {
            if !button_touch_down {
                button_touch_down = true;
                match ctx.scene {
                    Scene::WifiSurvey => {
                        let mode = crate::radio::wifi_mode();
                        if mode == crate::radio::WifiMode::SurveyScanning {
                            crate::radio::send_wifi_cmd(crate::radio::WifiCommand::StopSurvey);
                        } else {
                            crate::radio::send_wifi_cmd(crate::radio::WifiCommand::StartSurvey);
                        }
                        return Some(Nav::Refresh);
                    }
                    Scene::WifiAp => {
                        let ap_status = crate::radio::wifi_ap_status();
                        if ap_status.active {
                            crate::radio::send_wifi_cmd(crate::radio::WifiCommand::StopHotspot);
                        } else {
                            crate::radio::send_wifi_cmd(crate::radio::WifiCommand::StartHotspot);
                        }
                        return Some(Nav::Refresh);
                    }
                    _ => {}
                }
            }
        } else if sample.n == 0 {
            button_touch_down = false;
        }

        // Radio / orientation refresh after buttons so a same-tick edge wins.
        if let Some(rev) = ctx.ble_watch_rev {
            if crate::radio::state_rev() != rev {
                return Some(Nav::Refresh);
            }
        }
        if let Some(rev) = ctx.wifi_watch_rev {
            if crate::radio::wifi_state_rev() != rev {
                return Some(Nav::Refresh);
            }
        }

        #[cfg(feature = "orient")]
        {
            imu_since_ms = imu_since_ms.saturating_add(NAV_POLL_MS);
            imu_report_ms = imu_report_ms.saturating_add(NAV_POLL_MS);
            if imu_since_ms >= IMU_POLL_MS {
                imu_since_ms = 0;
                if let Ok(sample) = imu::read_accel(i2c) {
                    let pose = imu::classify(sample.x, sample.y, sample.z);
                    let pose_token = pose.map_or("none", imu::Orientation::as_str);
                    if imu_report_ms >= IMU_REPORT_MS {
                        imu_report_ms = 0;
                        cdc::imu(pose_token, sample.x, sample.y, sample.z);
                    }
                    if let Some(page) = pose.and_then(imu::Orientation::page_rotation) {
                        if page != *rotation {
                            if pending_page == Some(page) {
                                pending_count = pending_count.saturating_add(1);
                            } else {
                                pending_page = Some(page);
                                pending_count = 1;
                            }
                            if pending_count >= IMU_STABLE_POLLS {
                                cdc::imu(pose_token, sample.x, sample.y, sample.z);
                                *rotation = page;
                                return Some(Nav::Refresh);
                            }
                        } else {
                            pending_page = None;
                            pending_count = 0;
                        }
                    }
                }
            }
        }

        t_ms = t_ms.saturating_add(NAV_POLL_MS);
        if let Some(timeout) = ctx.auto_refresh_ms {
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
    rotation: PageRotation,
) {
    // 1. Draw and paint the sleep notice to the e-paper panel.
    draw::draw_sleeping(&mut planes.bw, &mut planes.red, rotation);
    panel
        .paint_mono_fast(i2c, &planes.bw, &planes.red, busy, false)
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
