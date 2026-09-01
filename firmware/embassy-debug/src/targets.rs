//! Draw a target, wait for a sloppy tap or midline slide, next.
//!
//! Either short A or short B aborts the **whole** walk (not one
//! target) and leaves the card. Hold-A PCM is only in `ui` while
//! sitting, not during this walk.

use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::Input;
use m5stack_papermono_lite::display;
use m5stack_papermono_lite::touch;
use papermono_log::{Edge, POLL_PERIOD_MS};

use crate::cdc;
use crate::ioe::SysI2c;
use crate::panel::{Mark, Panel};
use crate::share;
use crate::touch_bus::{self, LampSlide};

const DOT_X: [u16; 5] = [240, 80, 400, 80, 400];
const DOT_Y: [u16; 5] = [400, 80, 80, 720, 720];
const SLIDE_X_ID: u8 = 5;
const SLIDE_Y_ID: u8 = 6;
const LAST_ID: u8 = 6;

pub enum WalkEnd {
    Done,
    AbortPrev,
    AbortNext,
}

enum Outcome {
    Hit(u16, u16, u16),
    AbortA(u16, u16),
    AbortB(u16, u16),
    Timeout(u16, u16),
}

pub async fn walk(
    i2c: &mut SysI2c,
    panel: &mut Panel,
    btn_a: &Input<'static>,
    btn_b: &Input<'static>,
    tp: &Input<'static>,
    busy: &Input<'static>,
    lamp: &mut LampSlide,
) -> WalkEnd {
    for id in 0..=LAST_ID {
        let (kind, tx, ty, r, mark) = scene(id);
        cdc::touch_target(id, kind, tx, ty, r);
        panel.paint(i2c, mark, busy).await;
        let out = if id < SLIDE_X_ID {
            wait_dot(i2c, btn_a, btn_b, tp, lamp, id, tx, ty).await
        } else {
            wait_slide(i2c, btn_a, btn_b, tp, lamp, id, id == SLIDE_X_ID, tx, ty).await
        };
        match out {
            Outcome::Hit(x, y, d) => cdc::touch_verdict(id, "hit", x, y, tx, ty, d),
            Outcome::AbortA(x, y) => {
                cdc::touch_verdict(id, "abort=btn_a", x, y, tx, ty, 0);
                panel.paint(i2c, Mark::Blank, busy).await;
                return WalkEnd::AbortPrev;
            }
            Outcome::AbortB(x, y) => {
                cdc::touch_verdict(id, "abort=btn_b", x, y, tx, ty, 0);
                panel.paint(i2c, Mark::Blank, busy).await;
                return WalkEnd::AbortNext;
            }
            Outcome::Timeout(x, y) => cdc::touch_verdict(id, "abort=timeout", x, y, tx, ty, 0),
        }
    }
    panel.paint(i2c, Mark::Blank, busy).await;
    WalkEnd::Done
}

fn scene(id: u8) -> (&'static str, u16, u16, u16, Mark) {
    let mid_x = display::WIDTH / 2;
    let mid_y = display::HEIGHT / 2;
    match id {
        SLIDE_X_ID => (
            "slide_x",
            mid_x,
            mid_y,
            touch::SLIDE_HALF_W,
            Mark::HLine {
                y: mid_y,
                half: touch::SLIDE_HALF_W,
            },
        ),
        SLIDE_Y_ID => (
            "slide_y",
            mid_x,
            mid_y,
            touch::SLIDE_HALF_W,
            Mark::VLine {
                x: mid_x,
                half: touch::SLIDE_HALF_W,
            },
        ),
        n => {
            let i = usize::from(n.min(4));
            (
                "dot",
                DOT_X[i],
                DOT_Y[i],
                touch::TARGET_RADIUS_PX,
                Mark::Disk {
                    x: DOT_X[i],
                    y: DOT_Y[i],
                    r: touch::TARGET_RADIUS_PX,
                },
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn wait_dot(
    i2c: &mut SysI2c,
    btn_a: &Input<'static>,
    btn_b: &Input<'static>,
    tp: &Input<'static>,
    lamp: &mut LampSlide,
    id: u8,
    tx: u16,
    ty: u16,
) -> Outcome {
    let deadline = Instant::now() + Duration::from_millis(touch::TARGET_WAIT_MS);
    let mut prev_a = btn_a.is_high();
    let mut prev_b = btn_b.is_high();
    let mut last_x = 0u16;
    let mut last_y = 0u16;
    let mut miss_once = true;
    let mut n0_once = true;
    let mut t_ms = 0_u32;
    while Instant::now() < deadline {
        if let Some(out) =
            abort_buttons(btn_a, btn_b, &mut prev_a, &mut prev_b, t_ms, last_x, last_y)
        {
            return out;
        }
        let int_high = tp.is_high();
        share::TP.store(int_high, core::sync::atomic::Ordering::Relaxed);
        let sample = touch_bus::read_points(i2c, int_high, true);
        if sample.n >= 1 {
            last_x = sample.x;
            last_y = sample.y;
            let d = dist_px(sample.x, sample.y, tx, ty);
            if d <= touch::TARGET_SLOP_PX {
                cdc::touch_at(id, &sample, tx, ty);
                return Outcome::Hit(sample.x, sample.y, d);
            }
            // Gutter sits over the x=400 dots; slop already scored.
            if !lamp.feed(i2c, &sample) {
                n0_once = true;
                cdc::touch_at(id, &sample, tx, ty);
                if miss_once {
                    cdc::touch_verdict(id, "miss", sample.x, sample.y, tx, ty, d);
                    miss_once = false;
                }
            }
        } else if sample.n < 1 {
            miss_once = true;
            if !int_high && n0_once {
                cdc::touch_at(id, &sample, tx, ty);
                n0_once = false;
            }
        }
        t_ms = t_ms.saturating_add(POLL_PERIOD_MS);
        Timer::after(Duration::from_millis(POLL_PERIOD_MS.into())).await;
    }
    Outcome::Timeout(last_x, last_y)
}

#[allow(clippy::too_many_arguments)]
async fn wait_slide(
    i2c: &mut SysI2c,
    btn_a: &Input<'static>,
    btn_b: &Input<'static>,
    tp: &Input<'static>,
    lamp: &mut LampSlide,
    id: u8,
    along_x: bool,
    line_x: u16,
    line_y: u16,
) -> Outcome {
    let deadline = Instant::now() + Duration::from_millis(touch::TARGET_WAIT_MS);
    let mut prev_a = btn_a.is_high();
    let mut prev_b = btn_b.is_high();
    let mut last_x = 0u16;
    let mut last_y = 0u16;
    let mut min_v = u16::MAX;
    let mut max_v = 0u16;
    let mut t_ms = 0_u32;
    let (lo, hi) = if along_x {
        (touch::ACTIVE_MIN_X, touch::ACTIVE_MAX_X)
    } else {
        (touch::ACTIVE_MIN_Y, touch::ACTIVE_MAX_Y)
    };
    while Instant::now() < deadline {
        if let Some(out) =
            abort_buttons(btn_a, btn_b, &mut prev_a, &mut prev_b, t_ms, last_x, last_y)
        {
            return out;
        }
        let int_high = tp.is_high();
        share::TP.store(int_high, core::sync::atomic::Ordering::Relaxed);
        let sample = touch_bus::read_points(i2c, int_high, true);
        if sample.n >= 1 {
            last_x = sample.x;
            last_y = sample.y;
            let on_line = if along_x {
                sample.y.abs_diff(line_y) <= touch::TARGET_SLOP_PX
            } else {
                sample.x.abs_diff(line_x) <= touch::TARGET_SLOP_PX
            };
            if on_line {
                cdc::touch_at(id, &sample, line_x, line_y);
                let v = if along_x { sample.x } else { sample.y };
                min_v = min_v.min(v);
                max_v = max_v.max(v);
                let span = max_v.saturating_sub(min_v);
                if min_v <= lo.saturating_add(touch::SLIDE_END_INSET)
                    && max_v + touch::SLIDE_END_INSET >= hi
                {
                    return Outcome::Hit(last_x, last_y, span);
                }
            } else {
                let _ = lamp.feed(i2c, &sample);
            }
        }
        t_ms = t_ms.saturating_add(POLL_PERIOD_MS);
        Timer::after(Duration::from_millis(POLL_PERIOD_MS.into())).await;
    }
    Outcome::Timeout(last_x, last_y)
}

fn abort_buttons(
    btn_a: &Input<'static>,
    btn_b: &Input<'static>,
    prev_a: &mut bool,
    prev_b: &mut bool,
    t_ms: u32,
    last_x: u16,
    last_y: u16,
) -> Option<Outcome> {
    let now_a = btn_a.is_high();
    let now_b = btn_b.is_high();
    share::BTN_A.store(now_a, core::sync::atomic::Ordering::Relaxed);
    share::BTN_B.store(now_b, core::sync::atomic::Ordering::Relaxed);
    if now_a != *prev_a || now_b != *prev_b {
        cdc::edge(&Edge {
            t_ms,
            btn_a: (now_a != *prev_a).then_some((*prev_a, now_a)),
            btn_b: (now_b != *prev_b).then_some((*prev_b, now_b)),
        });
    }
    if *prev_a && !now_a {
        return Some(Outcome::AbortA(last_x, last_y));
    }
    if *prev_b && !now_b {
        return Some(Outcome::AbortB(last_x, last_y));
    }
    *prev_a = now_a;
    *prev_b = now_b;
    None
}

fn dist_px(ax: u16, ay: u16, bx: u16, by: u16) -> u16 {
    let dx = i32::from(ax) - i32::from(bx);
    let dy = i32::from(ay) - i32::from(by);
    isqrt_u32((dx * dx + dy * dy) as u32)
}

fn isqrt_u32(n: u32) -> u16 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x.min(u32::from(u16::MAX)) as u16
}
