//! 50 ms button poll, 1 Hz `hb`, 10 s `hello` / `git` / `gpio`.
//!
//! Bring-up `i2c` / `mic` stamps are stored and reprinted here. USB
//! `monitor --reset` does not recapture on Lite. When the five-card
//! UI is live, A/B are not stolen here (hold-A PCM is in `ui`).

use embassy_time::{Duration, Timer};
use esp_hal::gpio::Input;
use papermono_log::{
    Edge, GpioSample, Hello, LeftoverSample, Snapshot, HEARTBEAT_PERIOD_MS, HELLO_PERIOD_MS,
    MILLIS_PER_SEC, POLL_PERIOD_MS,
};

use crate::cdc;

pub struct Inputs {
    pub btn_a: Option<Input<'static>>,
    pub btn_b: Option<Input<'static>>,
    pub boot: Input<'static>,
    pub pmic_irq: Input<'static>,
    pub tp: Option<Input<'static>>,
    pub ioe: Input<'static>,
    pub busy: Option<Input<'static>>,
    pub lora_irq: Input<'static>,
    pub nfc_irq: Input<'static>,
    pub sx_busy: Input<'static>,
}

fn pin_or_shared(pin: Option<&Input<'static>>, shared: bool) -> bool {
    if let Some(p) = pin {
        return p.is_high();
    }
    shared
}

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
            #[cfg(feature = "sleep")]
            crate::sleep_wake::reprint();
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

        if t_ms.is_multiple_of(HEARTBEAT_PERIOD_MS) {
            cdc::heartbeat(&Snapshot {
                t_s: t_ms / MILLIS_PER_SEC,
                btn_a: now_a,
                btn_b: now_b,
            });
        }

        t_ms = t_ms.saturating_add(POLL_PERIOD_MS);
        Timer::after(Duration::from_millis(POLL_PERIOD_MS.into())).await;
    }
}
