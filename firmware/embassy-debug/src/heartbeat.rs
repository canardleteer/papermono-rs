//! Background heartbeat, telemetry emission, and periodic banner reporting task.
//!
//! # Architecture & Responsibilities
//! This module runs as an autonomous Embassy asynchronous task (`run`), driving the
//! periodic telemetry streams that mirror the [`simple-debug`] protocol:
//!
//! - **Button Polling & Edge Telemetry (10 ms)**: If the interactive UI is disabled,
//!   this task directly samples tactile buttons `GPIO2` and `GPIO3` and emits edge events.
//!   When the interactive UI is active, the UI task owns those pins, and this task reads
//!   the lock-free atomic mirrors in [`crate::share`].
//! - **Liveness Heartbeat (1 Hz)**: Emits `heartbeat` lines displaying instantaneous
//!   button states.
//! - **Periodic Metadata Banner (10 s)**: Emits `hello`, `git`, and comprehensive `gpio`
//!   logic levels, plus reprinting cached peripheral statuses:
//!   - Leftover unassigned radio pins on Lite hardware (`leftover`).
//!   - Wi-Fi and BLE discovery counts (`wifi`, `ble`).
//!   - I2C peripheral discovery matrix (`i2c`).
//!   - IP2315 battery charger telemetry (`charge`).
//!   - Frontlight LED PWM duty cycle (`lamp`).
//!   - PDM microphone energy metrics (`mic`).
//!   - E-paper panel refresh waveform and geometry stamps (`panel`).
//!   - Active UI card scene (`scene`).

use embassy_time::{Duration, Timer};
use esp_hal::gpio::Input;
use papermono_log::{
    Edge, GpioSample, Hello, LeftoverSample, Snapshot, HEARTBEAT_PERIOD_MS, HELLO_PERIOD_MS,
    MILLIS_PER_SEC, POLL_PERIOD_MS,
};

use crate::cdc;

/// Peripheral input pins passed to the heartbeat worker.
///
/// Pins marked as `Option` are set to `None` when the interactive UI task
/// takes ownership of them, falling back to shared atomic state.
pub struct Inputs {
    /// Tactile BUTTON A (`GPIO2` / UP).
    pub btn_a: Option<Input<'static>>,
    /// Tactile BUTTON B (`GPIO3` / DOWN).
    pub btn_b: Option<Input<'static>>,
    /// PMIC boot mode output (`GPIO0`).
    pub boot: Input<'static>,
    /// PMIC interrupt line (`GPIO1`).
    pub pmic_irq: Input<'static>,
    /// FT6336G capacitive touch interrupt (`GPIO4`).
    pub tp: Option<Input<'static>>,
    /// M5IOE1 I/O expander interrupt (`GPIO7`).
    pub ioe: Input<'static>,
    /// SSD1677 e-paper busy flag (`GPIO18`).
    pub busy: Option<Input<'static>>,
    /// Leftover LoRa IRQ pin on Lite (`GPIO5`).
    pub lora_irq: Input<'static>,
    /// Leftover NFC IRQ pin on Lite (`GPIO6`).
    pub nfc_irq: Input<'static>,
    /// Leftover SX1262 BUSY pin on Lite (`GPIO21`).
    pub sx_busy: Input<'static>,
}

/// Helper function to read either a directly owned hardware pin or fall back to an atomic mirror.
fn pin_or_shared(pin: Option<&Input<'static>>, shared: bool) -> bool {
    if let Some(p) = pin {
        return p.is_high();
    }
    shared
}

/// Main telemetry polling and emission loop running as an Embassy background task.
#[embassy_executor::task]
pub async fn run(pins: Inputs, hello: Hello) {
    let Inputs {
        btn_a,
        btn_b,
        boot,
        pmic_irq,
        tp,
        ioe,
        busy,
        lora_irq,
        nfc_irq,
        sx_busy,
    } = pins;
    let mut t_ms = 0_u32;
    let mut prev_a = pin_or_shared(btn_a.as_ref(), true);
    let mut prev_b = pin_or_shared(btn_b.as_ref(), true);
    #[cfg(feature = "touch")]
    let mut prev_tp = pin_or_shared(tp.as_ref(), true);

    loop {
        // Read shared atomic mirrors if the UI task owns the physical pins.
        #[cfg(feature = "panel")]
        let shared_a = crate::share::BTN_A.load(core::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "panel")]
        let shared_b = crate::share::BTN_B.load(core::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "panel")]
        let shared_tp = crate::share::TP.load(core::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "panel")]
        let shared_busy = crate::share::BUSY.load(core::sync::atomic::Ordering::Relaxed);
        #[cfg(not(feature = "panel"))]
        let shared_a = true;
        #[cfg(not(feature = "panel"))]
        let shared_b = true;
        #[cfg(not(feature = "panel"))]
        let shared_tp = true;
        #[cfg(not(feature = "panel"))]
        let shared_busy = false;

        let now_a = pin_or_shared(btn_a.as_ref(), shared_a);
        let now_b = pin_or_shared(btn_b.as_ref(), shared_b);
        let now_tp = pin_or_shared(tp.as_ref(), shared_tp);
        let now_busy = pin_or_shared(busy.as_ref(), shared_busy);
        let ui = btn_a.is_none();

        #[cfg(feature = "touch")]
        {
            if now_tp != prev_tp {
                cdc::touch(&crate::touch_bus::empty_touch(now_tp));
                prev_tp = now_tp;
            }
        }
        // If the UI is not running, handle button edges and trigger audio test on button press.
        if !ui && (now_a != prev_a || now_b != prev_b) {
            cdc::edge(&Edge {
                t_ms,
                btn_a: (now_a != prev_a).then_some((prev_a, now_a)),
                btn_b: (now_b != prev_b).then_some((prev_b, now_b)),
            });
            if prev_a && !now_a {
                #[cfg(feature = "mic")]
                crate::mic::ask_tone();
            }
            prev_a = now_a;
            prev_b = now_b;
        } else if ui {
            prev_a = now_a;
            prev_b = now_b;
        }

        // 10-second periodic banner emission.
        if t_ms.is_multiple_of(HELLO_PERIOD_MS) {
            cdc::hello(&Hello {
                t_s: t_ms / MILLIS_PER_SEC,
                ..hello
            });
            cdc::git();
            cdc::gpio(&GpioSample {
                boot: boot.is_high(),
                pmic_irq: pmic_irq.is_high(),
                tp: now_tp,
                ioe: ioe.is_high(),
                busy: now_busy,
            });
            cdc::leftover(&LeftoverSample {
                lora_irq: lora_irq.is_high(),
                nfc_irq: nfc_irq.is_high(),
                sx_busy: sx_busy.is_high(),
            });
            #[cfg(feature = "radio")]
            if let Some(n) = crate::radio::last_wifi() {
                cdc::wifi(n);
            }
            #[cfg(feature = "radio")]
            if let Some(n) = crate::radio::last_ble() {
                cdc::ble(n);
            }
            #[cfg(feature = "touch")]
            if let Some(sample) = crate::touch_bus::last_i2c() {
                cdc::i2c(&sample);
            }
            #[cfg(feature = "touch")]
            if let Some(sample) = crate::touch_bus::last_charge() {
                cdc::charge(&sample);
            }
            #[cfg(feature = "touch")]
            cdc::touch(&crate::touch_bus::empty_touch(now_tp));
            #[cfg(feature = "touch")]
            if let Some(duty) = crate::touch_bus::last_lamp() {
                cdc::lamp(duty);
            }
            #[cfg(feature = "mic")]
            if let Some(sample) = crate::mic::last() {
                cdc::mic(&sample);
            }
            #[cfg(feature = "panel")]
            if let Some(stamp) = crate::panel::last() {
                cdc::panel(&stamp);
            }
            #[cfg(feature = "panel")]
            if let Some(scene) = crate::share::last_scene() {
                cdc::scene(scene);
            }
        }

        // 1-second periodic liveness heartbeat.
        if t_ms.is_multiple_of(HEARTBEAT_PERIOD_MS) {
            cdc::heartbeat(&Snapshot {
                t_s: t_ms / MILLIS_PER_SEC,
                btn_a: now_a,
                btn_b: now_b,
            });
        }

        // Advance simulated time and yield executor.
        t_ms = t_ms.saturating_add(POLL_PERIOD_MS);
        Timer::after(Duration::from_millis(POLL_PERIOD_MS.into())).await;
    }
}
