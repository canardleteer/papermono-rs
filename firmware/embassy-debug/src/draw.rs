//! Orientation-aware UI card rendering into dual OTP gray planes.
//!
//! # Architecture & Display Pipeline
//! This module renders interactive graphical cards into two 1-bit-per-pixel
//! framebuffers (`bw` and `red`), which combine to form the 4-level grayscale
//! representation accepted by the SSD1677 e-paper controller.
//!
//! Draw in **page** coordinates for the current [`PageRotation`], then map each
//! pixel through [`display::page_to_framebuffer`] onto the official USB-C-down
//! 480×800 framebuffer before writing planes:
//!
//! - Portrait holds ([`PageRotation::Portrait0`] / [`PageRotation::Portrait180`]):
//!   page is 480×800 — existing card layouts are kept.
//! - Landscape holds ([`PageRotation::Landscape0`] / [`PageRotation::Landscape180`]):
//!   page is 800×480 — layouts are compressed so header, body, and footer fit.
//!
//! - **Grayscale Encoding**:
//!   - `display::GRAY_BLACK` (tone 0): `bw=0, red=0`
//!   - `display::GRAY_DARK` (tone 1): `bw=1, red=0`
//!   - `display::GRAY_LIGHT` (tone 2): `bw=0, red=1`
//!   - `display::GRAY_WHITE` (tone 3): `bw=1, red=1`
//! - **Embedded Graphics Integration**: Implements the
//!   [`embedded_graphics::draw_target::DrawTarget`] trait via [`GrayInk`],
//!   allowing standard text, shapes, and primitives to be rendered in page
//!   space into the dual-plane framebuffers.
//! - **Eight-Card Walkthrough**:
//!   1. `Splash`: Displays the Rust Ferris mascot and navigation hints.
//!   2. `Shapes`: Verifies geometry rendering (procedural 3-degree Koch
//!      snowflake with microsecond benchmark, triangles, boxes).
//!   3. `Legend`: Provides an on-device quick-reference visual guide for
//!      physical buttons, sleep/wake, and touch rails.
//!   4. `Bluetooth`: Displays 6-digit BLE passkey PIN for phone pairing and
//!      reports success or failure reason.
//!   5. `WifiSurvey`: Scans 2.4 GHz 802.11 channels, displays channel
//!      distribution and top discovered APs.
//!   6. `WifiAp`: Runs WPA2-Personal SoftAP with DHCP and serves JSON system
//!      stats over HTTP.
//!   7. `Tones`: 4-gray bands (stacked portrait / four-across landscape)
//!      demonstrating OTP grayscale palette accuracy.
//!   8. `Targets`: Monochromatic calibration points for digitizer latency
//!      and accuracy testing.

use core::fmt::Write;
use embassy_time::Instant;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};
use m5stack_papermono_lite::display::{self, PageRotation};
use m5stack_papermono_lite::pmic;
use papermono_log::{ChargeSample, Scene};

use crate::radio::{BlePairStatus, WifiMode};

/// 360×240 packed 1bpp bitmap of Ferris the Rust mascot.
/// Provenance: Generated from SVG via `cargo xtask encode-assets` (see `assets/SOURCE.md`).
const FERRIS: &[u8] = include_bytes!("../assets/ferris.1bpp");
const FERRIS_W: u16 = 360;
const FERRIS_H: u16 = 240;
const FERRIS_BYTES: usize = (FERRIS_W as usize * FERRIS_H as usize) / 8;

const _: () = assert!(FERRIS.len() == FERRIS_BYTES);
const _: () = assert!(FERRIS_W.is_multiple_of(8));

/// Built-in 10×20 glyph cell height. `embedded-graphics` `Text` `y` is the
/// baseline, so a line that must clear the glyph uses this plus spacing.
const GLYPH_H: i32 = 20;

/// Renders the requested interactive card scene into dual-plane framebuffers.
///
/// All drawing uses page coordinates for `rotation`; physical USB-down plane
/// writes happen only inside [`set_gray_page`]. Returns the benchmark render
/// duration in microseconds if the scene computes one.
pub fn render(
    scene: Scene,
    bw: &mut [u8],
    red: &mut [u8],
    charge: Option<ChargeSample>,
    rotation: PageRotation,
) -> Option<u32> {
    match scene {
        Scene::Splash => {
            draw_splash(bw, red, rotation);
            None
        }
        Scene::Shapes => Some(draw_shapes(bw, red, rotation)),
        Scene::Legend => {
            draw_legend(bw, red, charge, rotation);
            None
        }
        Scene::Bluetooth => {
            draw_bluetooth(bw, red, rotation);
            None
        }
        Scene::WifiSurvey => {
            draw_wifi_survey(bw, red, rotation);
            None
        }
        Scene::WifiAp => {
            draw_wifi_ap(bw, red, rotation);
            None
        }
        Scene::Tones => {
            draw_tones(bw, red, rotation);
            None
        }
        Scene::Targets => None,
    }
}

/// Renders the sleep screen notice before the device enters low-power light sleep.
///
/// Text is centered on the current page (`rotation.page_size()`), not the
/// fixed USB-down 480×800 physical canvas.
#[cfg(feature = "sleep")]
pub fn draw_sleeping(bw: &mut [u8], red: &mut [u8], rotation: PageRotation) {
    const LINE_GAP: i32 = 12;

    clear(bw, red, display::GRAY_WHITE, rotation);
    let (page_w, page_h) = rotation.page_size();
    let cx = i32::from(page_w) / 2;
    let cy = i32::from(page_h) / 2;
    let line1_y = cy - (LINE_GAP / 2);
    let line2_y = cy + (LINE_GAP / 2) + GLYPH_H;

    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let _ = Text::with_alignment(
        "sleeping,",
        Point::new(cx, line1_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red, rotation));
    let _ = Text::with_alignment(
        "press A or B for 1 second to restart",
        Point::new(cx, line2_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red, rotation));
}

/// Page-space origin/size of the Wi-Fi survey/AP start/stop touch button.
///
/// Returns `(x, y, w, h)` matching the double-stroked box drawn on
/// [`draw_wifi_survey`] / [`draw_wifi_ap`]. Portrait keeps the historical
/// `(60, 660, 360, 56)` band on a 480×800 page; landscape anchors the same
/// sized control near the bottom of the 800×480 page.
#[must_use]
pub fn wifi_action_rect(rotation: PageRotation) -> (u16, u16, u16, u16) {
    let (pw, ph) = rotation.page_size();
    let x = 60;
    let w = pw.saturating_sub(120);
    let h = 56;
    // Portrait: 800 − 140 = 660 (historical). Landscape: near bottom of 480.
    let y = if rotation.is_portrait() {
        ph.saturating_sub(140)
    } else {
        ph.saturating_sub(100)
    };
    (x, y, w, h)
}

/// Page-space touch hit test for the Wi-Fi survey/AP start/stop button.
///
/// Uses the same geometry as [`wifi_action_rect`] with a small pad so finger
/// contacts near the stroke still register (matches the prior ui.rs band of
/// roughly `x ∈ [50, pw−50]`, `y ∈ [ph−150, ph−60]` on portrait).
#[must_use]
pub fn wifi_action_hit(px: u16, py: u16, rotation: PageRotation) -> bool {
    let (x, y, w, h) = wifi_action_rect(rotation);
    let x0 = x.saturating_sub(10);
    let y0 = y.saturating_sub(10);
    let x1 = x.saturating_add(w).saturating_add(10);
    let y1 = y.saturating_add(h).saturating_add(10);
    px >= x0 && px < x1 && py >= y0 && py < y1
}

/// Renders Card 1: Ferris mascot splash screen and user navigation guide.
///
/// Portrait keeps the historical centered stack. Landscape tightens vertical
/// gaps so the 360×240 Ferris plus two hint lines still fit the 480-tall page.
fn draw_splash(bw: &mut [u8], red: &mut [u8], rotation: PageRotation) {
    let (page_w, page_h) = rotation.page_size();
    let (title_gap, hint_gap, line_gap) = if rotation.is_portrait() {
        (28_i32, 28_i32, 8_i32)
    } else {
        // Compress so Ferris + title + two hints clear the 480-tall page.
        (12, 12, 4)
    };

    clear(bw, red, display::GRAY_WHITE, rotation);
    let ferris_h = i32::from(FERRIS_H);
    let stack_h = ferris_h + title_gap + GLYPH_H + hint_gap + GLYPH_H + line_gap + GLYPH_H;
    let top = (i32::from(page_h) - stack_h) / 2;
    let cx = i32::from(page_w) / 2;
    let ferris_x = (i32::from(page_w) - i32::from(FERRIS_W)) / 2;
    let title_y = top + ferris_h + title_gap + GLYPH_H;
    let hint1_y = title_y + hint_gap + GLYPH_H;
    let hint2_y = hint1_y + line_gap + GLYPH_H;

    blit_ferris(bw, red, ferris_x, top, rotation);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let _ = Text::with_alignment(
        "papermono-rs",
        Point::new(cx, title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red, rotation));
    let _ = Text::with_alignment(
        "BUTTON A / B change cards",
        Point::new(cx, hint1_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red, rotation));
    let _ = Text::with_alignment(
        "Right edge slides frontlight",
        Point::new(cx, hint2_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red, rotation));
}

/// Renders Card 2: Geometric test patterns validating aspect ratio, display orientation, and procedural rendering.
///
/// Portrait keeps the historical 480×800 stack. Landscape shifts Koch left and
/// the triangle/rect primitives right so the 800×480 page is not a squeezed
/// portrait (same policy as sticky-rs).
fn draw_shapes(bw: &mut [u8], red: &mut [u8], rotation: PageRotation) -> u32 {
    let (page_w, page_h) = rotation.page_size();
    clear(bw, red, display::GRAY_WHITE, rotation);
    stroke_rect(bw, red, 0, 0, page_w, page_h, display::GRAY_BLACK, rotation);
    stroke_rect(
        bw,
        red,
        16,
        16,
        page_w.saturating_sub(32),
        page_h.saturating_sub(32),
        display::GRAY_DARK,
        rotation,
    );

    let (koch_c, koch_r, tri, rect, title_y, time_y) = if rotation.is_portrait() {
        (
            (240_u16, 200_u16),
            135_u16,
            (70_u16, 360_u16, 140_u16, 120_u16),
            (270_u16, 360_u16, 140_u16, 120_u16),
            580_i32,
            615_i32,
        )
    } else {
        (
            (200, 170),
            110,
            (420, 280, 140, 120),
            (600, 280, 140, 120),
            420,
            448,
        )
    };

    let start = Instant::now();
    draw_koch_snowflake(bw, red, 3, koch_c, koch_r, display::GRAY_BLACK, rotation);
    let elapsed_us = Instant::now().duration_since(start).as_micros() as u32;

    fill_triangle_up(
        bw,
        red,
        tri.0,
        tri.1,
        tri.2,
        tri.3,
        display::GRAY_LIGHT,
        rotation,
    );
    fill_rect(
        bw,
        red,
        rect.0,
        rect.1,
        rect.2,
        rect.3,
        display::GRAY_BLACK,
        rotation,
    );
    let cx = i32::from(page_w) / 2;
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let _ = Text::with_alignment(
        "Geometric Calibration",
        Point::new(cx, title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red, rotation));

    let mut buf = [0u8; 48];
    let mut writer = BufWriter {
        buf: &mut buf,
        pos: 0,
    };
    let _ = write!(writer, "Koch Snowflake (3 deg): {elapsed_us} us");
    if let Ok(label) = core::str::from_utf8(&writer.buf[..writer.pos]) {
        let _ = Text::with_alignment(label, Point::new(cx, time_y), style, Alignment::Center)
            .draw(&mut GrayInk::new(bw, red, rotation));
    }

    elapsed_us
}

/// Renders Card 3: Legend displaying hardware pinout, button functions, sleep controls, and battery telemetry.
///
/// Portrait is a stacked key/value document (historical). Landscape puts key
/// at `x = 24` and value at `x = 220` with a ~36 px row step so eight rows
/// still fit the 480-tall page (sticky-rs policy).
fn draw_legend(
    bw: &mut [u8],
    red: &mut [u8],
    charge: Option<ChargeSample>,
    rotation: PageRotation,
) {
    clear(bw, red, display::GRAY_WHITE, rotation);
    let (page_w, page_h) = rotation.page_size();
    let cx = i32::from(page_w) / 2;
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

    let items = [
        ("BUTTON A (GPIO2)", "Previous card / hold 2s sleep"),
        ("BUTTON B (GPIO3)", "Next card / hold 1s wake"),
        ("SLEEP & WAKE", "Hold A 2s: sleep. Hold A/B 1s: wake"),
        ("RIGHT GUTTER", "Frontlight brightness slider"),
        ("TOUCH DIGITIZER", "FT6336G capacitive I2C (0x38)"),
        ("PMIC (M5PM1)", "Power rails, buttons, battery ADC"),
        ("EXPANDER (M5IOE1)", "Power gates & peripheral resets"),
        ("POWER BUTTON", "Short press reset / hold download"),
    ];

    {
        let mut ink = GrayInk::new(bw, red, rotation);
        let _ = Text::with_alignment(
            "HARDWARE LEGEND",
            Point::new(cx, if rotation.is_portrait() { 50 } else { 36 }),
            style,
            Alignment::Center,
        )
        .draw(&mut ink);

        if rotation.is_portrait() {
            let mut y = 90;
            for (k, v) in items {
                let _ = Text::new(k, Point::new(30, y), style).draw(&mut ink);
                let _ = Text::new(v, Point::new(50, y + 25), style).draw(&mut ink);
                y += 70;
                if k == "EXPANDER (M5IOE1)" {
                    y += 10;
                }
            }
        } else {
            let mut y = 80;
            for (k, v) in items {
                let _ = Text::new(k, Point::new(24, y), style).draw(&mut ink);
                let _ = Text::new(v, Point::new(220, y), style).draw(&mut ink);
                y += 36;
            }
        }
    }

    let (pct, vbat, vin, usb) = match charge {
        Some(c) => {
            let pct = pmic::battery_percent(c.vbat);
            let usb = (c.src & pmic::PWR_SRC_VIN != 0) || (c.vin >= pmic::VIN_PRESENT_MV);
            (Some(pct), c.vbat, c.vin, usb)
        }
        None => (None, 0, 0, false),
    };

    // Status / battery band: portrait keeps historical Y; landscape packs
    // against the bottom of the 480-tall page.
    let (rule_y, status_title_y, gauge_y, terminal_y, metrics_y, pwr_y, lipo_y) =
        if rotation.is_portrait() {
            (
                638_u16, 665_i32, 680_u16, 685_u16, 697_i32, 727_i32, 755_i32,
            )
        } else {
            let rule = page_h.saturating_sub(120);
            (
                rule,
                i32::from(rule) + 20,
                rule.saturating_add(32),
                rule.saturating_add(37),
                i32::from(rule) + 48,
                i32::from(rule) + 72,
                i32::from(rule) + 96,
            )
        };

    fill_rect(
        bw,
        red,
        30,
        rule_y,
        page_w.saturating_sub(60),
        2,
        display::GRAY_BLACK,
        rotation,
    );

    // Battery gauge outline: 104x20; positive terminal cap: 4x10.
    stroke_rect(bw, red, 30, gauge_y, 104, 20, display::GRAY_BLACK, rotation);
    fill_rect(
        bw,
        red,
        134,
        terminal_y,
        4,
        10,
        display::GRAY_BLACK,
        rotation,
    );

    if let Some(p) = pct {
        let fill_w = u16::from(p.min(100));
        fill_rect(
            bw,
            red,
            32,
            gauge_y.saturating_add(2),
            fill_w,
            16,
            display::GRAY_BLACK,
            rotation,
        );
    }

    let mut ink = GrayInk::new(bw, red, rotation);
    let _ = Text::new(
        "BATTERY & POWER STATUS",
        Point::new(30, status_title_y),
        style,
    )
    .draw(&mut ink);

    let mut buf = [0u8; 64];
    let mut writer = BufWriter {
        buf: &mut buf,
        pos: 0,
    };
    if let Some(p) = pct {
        let _ = write!(writer, "{p}%  [ {vbat} mV ]");
    } else {
        let _ = write!(writer, "--%  [ ---- mV ]");
    }
    if let Ok(label) = core::str::from_utf8(&writer.buf[..writer.pos]) {
        let _ = Text::new(label, Point::new(150, metrics_y), style).draw(&mut ink);
    }

    let mut buf_pwr = [0u8; 64];
    let mut writer_pwr = BufWriter {
        buf: &mut buf_pwr,
        pos: 0,
    };
    if usb {
        let _ = write!(writer_pwr, "Power: USB connected (VIN: {vin} mV)");
    } else {
        let _ = write!(writer_pwr, "Power: Running on battery");
    }
    if let Ok(label) = core::str::from_utf8(&writer_pwr.buf[..writer_pwr.pos]) {
        let _ = Text::new(label, Point::new(30, pwr_y), style).draw(&mut ink);
    }

    let _ = Text::new(
        "1S LiPo: 3300 mV (0%) - 4150 mV (100%)",
        Point::new(30, lipo_y),
        style,
    )
    .draw(&mut ink);
}

/// Renders Card 4: Bluetooth Low Energy peripheral pairing with PIN display and status.
///
/// # Visual Hierarchy
/// Portrait keeps the historical 480×800 geometry documented below. Landscape
/// compresses vertical gaps (sticky-rs pair-card policy) so header, PIN,
/// status, tutorial, and footer all fit the 800×480 page; tutorial steps
/// become two columns.
///
/// # Portrait Layout Geometry (480×800)
/// - **Header (y = 50..70)**: Centered title "BLUETOOTH PAIRING" with black dividing bar.
/// - **Device Info & Instruction (y = 110..145)**: Shows local name "Device: PaperMono"
///   and contextual instruction based on [`BlePairStatus`].
/// - **Passkey PIN Display (y = 175..265)**:
///   - Outer bordered container (360x90 px) centered at x = 60.
///   - Six individual digit boxes (40x50 px each, 52 px pitch) when displaying a passkey or hyphens.
///   - Single wide box (300x50 px) displaying "P A I R E D" upon successful pairing.
/// - **Status / Result Frame (y = 295..480)**:
///   - Highlighted status outline box (280x45 px) for terminal states ("SUCCESS" or "FAILED").
///   - Descriptive explanation line (e.g. "Why: Passkey entry failed / canceled").
///   - Contextual troubleshooting guidance for the user.
/// - **Tutorial Walkthrough (y = 510..680)**: Step-by-step instructions for pairing from a phone.
/// - **Footer (y = 720..800)**: Dividing rule and button navigation prompts (Button A / B).
///
/// # Rendering Architecture
/// Primitives (lines, boxes) are drawn first to avoid overdrawing text ink. All typography
/// is rendered via a single [`GrayInk`] rasterizer using [`FONT_10X20`]. Metric text strings
/// are formatted into small stack buffers via [`BufWriter`] without heap allocation.
fn draw_bluetooth(bw: &mut [u8], red: &mut [u8], rotation: PageRotation) {
    clear(bw, red, display::GRAY_WHITE, rotation);
    let status = crate::radio::pair_status();
    let layout = BleLayout::for_rotation(rotation);

    // 1. Draw structural lines and frames
    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.header_bar_y,
        layout.rule_w,
        2,
        display::GRAY_BLACK,
        rotation,
    );

    stroke_rect(
        bw,
        red,
        layout.pin_frame_x,
        layout.pin_frame_y,
        layout.pin_frame_w,
        layout.pin_frame_h,
        display::GRAY_BLACK,
        rotation,
    );
    stroke_rect(
        bw,
        red,
        layout.pin_frame_x.saturating_add(3),
        layout.pin_frame_y.saturating_add(3),
        layout.pin_frame_w.saturating_sub(6),
        layout.pin_frame_h.saturating_sub(6),
        display::GRAY_BLACK,
        rotation,
    );

    if status == BlePairStatus::Success {
        stroke_rect(
            bw,
            red,
            layout.banner_x,
            layout.digit_y,
            layout.banner_w,
            layout.digit_h,
            display::GRAY_BLACK,
            rotation,
        );
    } else {
        for i in 0..6_u16 {
            let x = layout
                .digit_x0
                .saturating_add(i.saturating_mul(layout.digit_pitch));
            stroke_rect(
                bw,
                red,
                x,
                layout.digit_y,
                layout.digit_w,
                layout.digit_h,
                display::GRAY_BLACK,
                rotation,
            );
        }
    }

    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.mid_bar_y,
        layout.rule_w,
        2,
        display::GRAY_BLACK,
        rotation,
    );

    match status {
        BlePairStatus::Success | BlePairStatus::Failed(_) => {
            stroke_rect(
                bw,
                red,
                layout.status_x,
                layout.status_y,
                layout.status_w,
                layout.status_h,
                display::GRAY_BLACK,
                rotation,
            );
            stroke_rect(
                bw,
                red,
                layout.status_x.saturating_add(2),
                layout.status_y.saturating_add(2),
                layout.status_w.saturating_sub(4),
                layout.status_h.saturating_sub(4),
                display::GRAY_BLACK,
                rotation,
            );
        }
        _ => {}
    }

    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.tutorial_bar_y,
        layout.rule_w,
        1,
        display::GRAY_LIGHT,
        rotation,
    );

    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.footer_bar_y,
        layout.rule_w,
        2,
        display::GRAY_BLACK,
        rotation,
    );

    // 2. Draw all text with a single GrayInk to avoid multiple mutable borrow conflicts.
    let mut ink = GrayInk::new(bw, red, rotation);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let cx = layout.cx;

    let _ = Text::with_alignment(
        "BLUETOOTH PAIRING",
        Point::new(cx, layout.title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::with_alignment(
        "Device: PaperMono",
        Point::new(cx, layout.device_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let instruction = match status {
        BlePairStatus::Pairing(_) => "Enter this PIN code on your phone:",
        BlePairStatus::Success => "Device paired and connected!",
        BlePairStatus::Failed(_) => "Pairing attempt failed",
        BlePairStatus::Connected => "Connecting, negotiating pairing...",
        BlePairStatus::Advertising => "Discoverable as 'PaperMono'",
        BlePairStatus::Disabled => "BLE radio disabled in build",
    };
    let _ = Text::with_alignment(
        instruction,
        Point::new(cx, layout.instruction_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    match status {
        BlePairStatus::Pairing(pin) => {
            let digits = [
                (((pin / 100_000) % 10) as u8 + b'0'),
                (((pin / 10_000) % 10) as u8 + b'0'),
                (((pin / 1_000) % 10) as u8 + b'0'),
                (((pin / 100) % 10) as u8 + b'0'),
                (((pin / 10) % 10) as u8 + b'0'),
                ((pin % 10) as u8 + b'0'),
            ];
            for (i, &d) in digits.iter().enumerate() {
                let char_str = core::str::from_utf8(core::slice::from_ref(&d)).unwrap_or("-");
                let x = i32::from(layout.digit_x0)
                    + i32::from(layout.digit_pitch) * i32::try_from(i).unwrap_or(0)
                    + i32::from(layout.digit_w / 2);
                let _ = Text::with_alignment(
                    char_str,
                    Point::new(x, layout.digit_baseline),
                    style,
                    Alignment::Center,
                )
                .draw(&mut ink);
            }
        }
        BlePairStatus::Success => {
            let _ = Text::with_alignment(
                "P A I R E D",
                Point::new(cx, layout.digit_baseline),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        _ => {
            for i in 0..6 {
                let x = i32::from(layout.digit_x0)
                    + i32::from(layout.digit_pitch) * i
                    + i32::from(layout.digit_w / 2);
                let _ = Text::with_alignment(
                    "-",
                    Point::new(x, layout.digit_baseline),
                    style,
                    Alignment::Center,
                )
                .draw(&mut ink);
            }
        }
    }

    match status {
        BlePairStatus::Success => {
            let _ = Text::with_alignment(
                "SUCCESS",
                Point::new(cx, layout.status_text_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
            let _ = Text::with_alignment(
                "Bluetooth connection encrypted.",
                Point::new(cx, layout.status_detail_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Failed(reason) => {
            let _ = Text::with_alignment(
                "FAILED",
                Point::new(cx, layout.status_text_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
            let mut buf = [0u8; 64];
            let mut writer = BufWriter {
                buf: &mut buf,
                pos: 0,
            };
            let _ = write!(writer, "Why: {}", reason.as_str());
            if let Ok(label) = core::str::from_utf8(&writer.buf[..writer.pos]) {
                let _ = Text::with_alignment(
                    label,
                    Point::new(cx, layout.status_detail_y),
                    style,
                    Alignment::Center,
                )
                .draw(&mut ink);
            }
            let _ = Text::with_alignment(
                "Retry pairing from phone settings.",
                Point::new(cx, layout.status_hint_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Connected => {
            let _ = Text::with_alignment(
                "Status: Phone connected",
                Point::new(cx, layout.status_text_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
            let _ = Text::with_alignment(
                "Awaiting passkey exchange...",
                Point::new(cx, layout.status_detail_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Pairing(_) => {
            let _ = Text::with_alignment(
                "Status: Pairing in progress",
                Point::new(cx, layout.status_text_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
            let _ = Text::with_alignment(
                "Enter PIN shown above on phone",
                Point::new(cx, layout.status_detail_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Advertising => {
            let _ = Text::with_alignment(
                "Status: Ready to pair",
                Point::new(cx, layout.status_text_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
            let _ = Text::with_alignment(
                "Select 'PaperMono' in phone Bluetooth",
                Point::new(cx, layout.status_detail_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Disabled => {
            let _ = Text::with_alignment(
                "Status: Radio disabled in build",
                Point::new(cx, layout.status_text_y),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
    }

    let _ = Text::with_alignment(
        "HOW TO PAIR",
        Point::new(cx, layout.howto_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let steps = [
        "1. Open Settings -> Bluetooth on phone",
        "2. Select 'PaperMono' under devices",
        "3. Wait for pairing passkey prompt",
        "4. Enter the 6-digit PIN shown above",
    ];
    if rotation.is_portrait() {
        let mut step_y = layout.step_y0;
        for step in steps {
            let _ =
                Text::new(step, Point::new(i32::from(layout.step_x), step_y), style).draw(&mut ink);
            step_y += 35;
        }
    } else {
        // Two columns so the 480-tall landscape page keeps the footer.
        for (i, step) in steps.iter().enumerate() {
            let col = i32::try_from(i % 2).unwrap_or(0);
            let row = i32::try_from(i / 2).unwrap_or(0);
            let x = i32::from(layout.step_x) + col * 380;
            let y = layout.step_y0 + row * 32;
            let _ = Text::new(*step, Point::new(x, y), style).draw(&mut ink);
        }
    }

    let _ = Text::with_alignment(
        "BUTTON A: Prev   |   BUTTON B: Next",
        Point::new(cx, layout.footer_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);
}

/// Page-space geometry for the Bluetooth card (portrait 480×800 vs landscape 800×480).
struct BleLayout {
    /// Horizontal center of the current page.
    cx: i32,
    /// Centered title baseline.
    title_y: i32,
    /// Hairline under the title.
    header_bar_y: u16,
    /// `Device: PaperMono` baseline.
    device_y: i32,
    /// Status-specific instruction baseline.
    instruction_y: i32,
    /// Outer PIN frame origin X.
    pin_frame_x: u16,
    /// Outer PIN frame origin Y.
    pin_frame_y: u16,
    /// Outer PIN frame width.
    pin_frame_w: u16,
    /// Outer PIN frame height.
    pin_frame_h: u16,
    /// First digit-box origin X.
    digit_x0: u16,
    /// Digit-box origin Y (also the Paired banner Y).
    digit_y: u16,
    /// Digit-box width.
    digit_w: u16,
    /// Digit-box height.
    digit_h: u16,
    /// Distance from one digit-box left edge to the next.
    digit_pitch: u16,
    /// `embedded-graphics` baseline for a digit inside its box.
    digit_baseline: i32,
    /// Wide Paired banner origin X.
    banner_x: u16,
    /// Wide Paired banner width.
    banner_w: u16,
    /// Rule X (header / mid / footer share this).
    rule_x: u16,
    /// Rule width.
    rule_w: u16,
    /// Hairline under the PIN frame.
    mid_bar_y: u16,
    /// Terminal-state outline origin X.
    status_x: u16,
    /// Terminal-state outline origin Y.
    status_y: u16,
    /// Terminal-state outline width.
    status_w: u16,
    /// Terminal-state outline height.
    status_h: u16,
    /// SUCCESS / FAILED / Status: baseline.
    status_text_y: i32,
    /// Detail line under the status word.
    status_detail_y: i32,
    /// Optional third status line (fail retry).
    status_hint_y: i32,
    /// Light rule above the tutorial.
    tutorial_bar_y: u16,
    /// `HOW TO PAIR` baseline.
    howto_y: i32,
    /// First tutorial step X.
    step_x: u16,
    /// First tutorial step baseline.
    step_y0: i32,
    /// Footer hairline Y.
    footer_bar_y: u16,
    /// Footer navigation baseline.
    footer_y: i32,
}

impl BleLayout {
    /// Pick the portrait or landscape constant table for this hold.
    fn for_rotation(rotation: PageRotation) -> Self {
        let (page_w, _) = rotation.page_size();
        let cx = i32::from(page_w / 2);
        if rotation.is_portrait() {
            Self {
                cx,
                title_y: 50,
                header_bar_y: 70,
                device_y: 110,
                instruction_y: 145,
                pin_frame_x: 60,
                pin_frame_y: 175,
                pin_frame_w: 360,
                pin_frame_h: 90,
                digit_x0: 90,
                digit_y: 195,
                digit_w: 40,
                digit_h: 50,
                digit_pitch: 52,
                digit_baseline: 227,
                banner_x: 90,
                banner_w: 300,
                rule_x: 30,
                rule_w: 420,
                mid_bar_y: 295,
                status_x: 100,
                status_y: 315,
                status_w: 280,
                status_h: 45,
                status_text_y: 345,
                status_detail_y: 395,
                status_hint_y: 425,
                tutorial_bar_y: 480,
                howto_y: 510,
                step_x: 45,
                step_y0: 545,
                footer_bar_y: 720,
                footer_y: 755,
            }
        } else {
            Self {
                cx,
                title_y: 28,
                header_bar_y: 42,
                device_y: 64,
                instruction_y: 86,
                pin_frame_x: 220,
                pin_frame_y: 96,
                pin_frame_w: 360,
                pin_frame_h: 80,
                digit_x0: 250,
                digit_y: 112,
                digit_w: 40,
                digit_h: 48,
                digit_pitch: 52,
                digit_baseline: 144,
                banner_x: 250,
                banner_w: 300,
                rule_x: 40,
                rule_w: 720,
                mid_bar_y: 188,
                status_x: 260,
                status_y: 200,
                status_w: 280,
                status_h: 40,
                status_text_y: 226,
                status_detail_y: 250,
                status_hint_y: 270,
                tutorial_bar_y: 278,
                howto_y: 300,
                step_x: 40,
                step_y0: 328,
                footer_bar_y: 420,
                footer_y: 452,
            }
        }
    }
}

/// Renders Card 5: 2.4 GHz Wi-Fi channel survey with channel saturation histogram and top AP table.
///
/// Portrait keeps the historical 480×800 document. Landscape uses `page_w` /
/// `page_h` for rules and footer, tightens vertical spacing, keeps guide lines
/// ≤40 glyphs, and anchors the touch button via [`wifi_action_rect`].
fn draw_wifi_survey(bw: &mut [u8], red: &mut [u8], rotation: PageRotation) {
    clear(bw, red, display::GRAY_WHITE, rotation);
    let mode = crate::radio::wifi_mode();
    let survey = crate::radio::wifi_survey_data();
    let layout = WifiCardLayout::survey(rotation);
    let (btn_x, btn_y, btn_w, btn_h) = wifi_action_rect(rotation);

    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.header_bar_y,
        layout.rule_w,
        2,
        display::GRAY_BLACK,
        rotation,
    );

    stroke_rect(
        bw,
        red,
        layout.banner_x,
        layout.banner_y,
        layout.banner_w,
        layout.banner_h,
        display::GRAY_BLACK,
        rotation,
    );
    stroke_rect(
        bw,
        red,
        layout.banner_x.saturating_add(2),
        layout.banner_y.saturating_add(2),
        layout.banner_w.saturating_sub(4),
        layout.banner_h.saturating_sub(4),
        display::GRAY_BLACK,
        rotation,
    );

    fill_rect(
        bw,
        red,
        layout.rule_x.saturating_add(10),
        layout.ch_div_y,
        layout.rule_w.saturating_sub(20),
        1,
        display::GRAY_LIGHT,
        rotation,
    );
    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.ap_div_y,
        layout.rule_w,
        2,
        display::GRAY_BLACK,
        rotation,
    );
    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.guide_div_y,
        layout.rule_w,
        1,
        display::GRAY_LIGHT,
        rotation,
    );

    stroke_rect(
        bw,
        red,
        btn_x,
        btn_y,
        btn_w,
        btn_h,
        display::GRAY_BLACK,
        rotation,
    );
    stroke_rect(
        bw,
        red,
        btn_x.saturating_add(2),
        btn_y.saturating_add(2),
        btn_w.saturating_sub(4),
        btn_h.saturating_sub(4),
        display::GRAY_BLACK,
        rotation,
    );

    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.footer_bar_y,
        layout.rule_w,
        2,
        display::GRAY_BLACK,
        rotation,
    );

    let mut ink = GrayInk::new(bw, red, rotation);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let cx = layout.cx;

    let _ = Text::with_alignment(
        "WI-FI CHANNEL SURVEY",
        Point::new(cx, layout.title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let status_str = match mode {
        WifiMode::SurveyScanning => "STATUS: SCANNING CHANNELS...",
        WifiMode::SurveyComplete => "STATUS: SCAN COMPLETE",
        WifiMode::Hotspot => "STATUS: HOTSPOT ACTIVE",
        WifiMode::Idle => "STATUS: IDLE (READY TO SCAN)",
    };
    let _ = Text::with_alignment(
        status_str,
        Point::new(cx, layout.status_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::with_alignment(
        "2.4 GHz CHANNEL OCCUPANCY",
        Point::new(cx, layout.ch_title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let (total, ch1, ch6, ch11, other) = if let Some(ref data) = survey {
        (
            data.total_aps,
            data.ch1_count,
            data.ch6_count,
            data.ch11_count,
            data.other_count,
        )
    } else {
        (0, 0, 0, 0, 0)
    };

    let mut buf_tot = [0u8; 48];
    let mut w_tot = BufWriter {
        buf: &mut buf_tot,
        pos: 0,
    };
    let _ = write!(w_tot, "Total Networks Discovered: {total}");
    if let Ok(label) = core::str::from_utf8(&w_tot.buf[..w_tot.pos]) {
        let _ = Text::new(
            label,
            Point::new(i32::from(layout.body_x), layout.ch_line1_y),
            style,
        )
        .draw(&mut ink);
    }

    let mut buf_ch = [0u8; 64];
    let mut w_ch = BufWriter {
        buf: &mut buf_ch,
        pos: 0,
    };
    let _ = write!(w_ch, "Ch 1: {:>2}   |   Ch 6: {:>2}", ch1, ch6);
    if let Ok(label) = core::str::from_utf8(&w_ch.buf[..w_ch.pos]) {
        let _ = Text::new(
            label,
            Point::new(i32::from(layout.body_x), layout.ch_line2_y),
            style,
        )
        .draw(&mut ink);
    }

    let mut buf_ch2 = [0u8; 64];
    let mut w_ch2 = BufWriter {
        buf: &mut buf_ch2,
        pos: 0,
    };
    let _ = write!(w_ch2, "Ch 11: {:>2}  |   Other: {:>2}", ch11, other);
    if let Ok(label) = core::str::from_utf8(&w_ch2.buf[..w_ch2.pos]) {
        let _ = Text::new(
            label,
            Point::new(i32::from(layout.body_x), layout.ch_line3_y),
            style,
        )
        .draw(&mut ink);
    }

    let _ = Text::with_alignment(
        "STRONGEST ACCESS POINTS",
        Point::new(cx, layout.ap_title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::new(
        "SSID           CH     RSSI     AUTH     ",
        Point::new(i32::from(layout.table_x), layout.ap_header_y),
        style,
    )
    .draw(&mut ink);

    let has_entries = survey.as_ref().is_some_and(|s| s.top_aps[0].is_some());
    if has_entries {
        if let Some(ref data) = survey {
            let mut row_y = layout.ap_row0_y;
            let max_rows = if rotation.is_portrait() { 5 } else { 3 };
            for ap in data.top_aps.iter().flatten().take(max_rows) {
                let ssid_slice = &ap.ssid[..usize::from(ap.ssid_len).min(18)];
                let ssid_clean = core::str::from_utf8(ssid_slice).unwrap_or("<hidden>");
                let mut row_buf = [0u8; 64];
                let mut w_row = BufWriter {
                    buf: &mut row_buf,
                    pos: 0,
                };
                let _ = write!(
                    w_row,
                    "{:<14} {:>2} {:>4}dBm {:>8}",
                    ssid_clean, ap.channel, ap.rssi, ap.auth
                );
                if let Ok(label) = core::str::from_utf8(&w_row.buf[..w_row.pos]) {
                    let _ = Text::new(label, Point::new(i32::from(layout.table_x), row_y), style)
                        .draw(&mut ink);
                }
                row_y += layout.ap_row_step;
            }
        }
    } else {
        let _ = Text::with_alignment(
            "No networks scanned yet",
            Point::new(cx, layout.ap_empty_y),
            style,
            Alignment::Center,
        )
        .draw(&mut ink);
    }

    let _ = Text::with_alignment(
        "SURVEY OPERATION",
        Point::new(cx, layout.guide_title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    // Guide lines stay within ~40 glyphs (FONT_10X20) so they do not clip.
    let guide_lines: &[&str] = if rotation.is_portrait() {
        &[
            "1. Tap below to scan channels 1-13.",
            "2. Identifies 2.4 GHz channel saturation.",
            "3. Top 4 APs listed by signal (RSSI).",
            "4. Starting survey halts SoftAP if running.",
        ]
    } else {
        &[
            "1. Tap below to scan channels 1-13.",
            "2. Shows 2.4 GHz channel saturation.",
            "3. Top APs by RSSI; survey stops SoftAP.",
        ]
    };
    let mut guide_y = layout.guide_y0;
    for line in guide_lines {
        let _ =
            Text::new(*line, Point::new(i32::from(layout.guide_x), guide_y), style).draw(&mut ink);
        guide_y += layout.guide_step;
    }

    let button_label = if mode == WifiMode::SurveyScanning {
        "[ STOP SURVEY ]"
    } else {
        "[ START SURVEY ]"
    };
    let btn_label_y = i32::from(btn_y) + i32::from(btn_h / 2) + (GLYPH_H / 2) - 4;
    let _ = Text::with_alignment(
        button_label,
        Point::new(cx, btn_label_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::with_alignment(
        "BUTTON A: Prev   |   BUTTON B: Next",
        Point::new(cx, layout.footer_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);
}

/// Renders Card 6: WPA2-Personal Wi-Fi SoftAP and embedded HTTP web server status.
///
/// Portrait keeps the historical 480×800 document. Landscape compresses vertical
/// spacing, shortens guide lines to ≤40 glyphs, and anchors the touch button via
/// [`wifi_action_rect`].
fn draw_wifi_ap(bw: &mut [u8], red: &mut [u8], rotation: PageRotation) {
    clear(bw, red, display::GRAY_WHITE, rotation);
    let status = crate::radio::wifi_ap_status();
    let layout = WifiCardLayout::hotspot(rotation);
    let (btn_x, btn_y, btn_w, btn_h) = wifi_action_rect(rotation);

    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.header_bar_y,
        layout.rule_w,
        2,
        display::GRAY_BLACK,
        rotation,
    );

    stroke_rect(
        bw,
        red,
        layout.banner_x,
        layout.banner_y,
        layout.banner_w,
        layout.banner_h,
        display::GRAY_BLACK,
        rotation,
    );
    stroke_rect(
        bw,
        red,
        layout.banner_x.saturating_add(2),
        layout.banner_y.saturating_add(2),
        layout.banner_w.saturating_sub(4),
        layout.banner_h.saturating_sub(4),
        display::GRAY_BLACK,
        rotation,
    );

    stroke_rect(
        bw,
        red,
        layout.cred_x,
        layout.cred_y,
        layout.cred_w,
        layout.cred_h,
        display::GRAY_BLACK,
        rotation,
    );
    stroke_rect(
        bw,
        red,
        layout.cred_x.saturating_add(2),
        layout.cred_y.saturating_add(2),
        layout.cred_w.saturating_sub(4),
        layout.cred_h.saturating_sub(4),
        display::GRAY_BLACK,
        rotation,
    );

    for i in 0..8_u16 {
        let bx = layout
            .pass_x0
            .saturating_add(i.saturating_mul(layout.pass_pitch));
        stroke_rect(
            bw,
            red,
            bx,
            layout.pass_y,
            layout.pass_w,
            layout.pass_h,
            display::GRAY_BLACK,
            rotation,
        );
    }

    fill_rect(
        bw,
        red,
        layout.rule_x.saturating_add(10),
        layout.telem_div_y,
        layout.rule_w.saturating_sub(20),
        1,
        display::GRAY_LIGHT,
        rotation,
    );
    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.guide_div_y,
        layout.rule_w,
        1,
        display::GRAY_LIGHT,
        rotation,
    );

    stroke_rect(
        bw,
        red,
        btn_x,
        btn_y,
        btn_w,
        btn_h,
        display::GRAY_BLACK,
        rotation,
    );
    stroke_rect(
        bw,
        red,
        btn_x.saturating_add(2),
        btn_y.saturating_add(2),
        btn_w.saturating_sub(4),
        btn_h.saturating_sub(4),
        display::GRAY_BLACK,
        rotation,
    );

    fill_rect(
        bw,
        red,
        layout.rule_x,
        layout.footer_bar_y,
        layout.rule_w,
        2,
        display::GRAY_BLACK,
        rotation,
    );

    let mut ink = GrayInk::new(bw, red, rotation);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let cx = layout.cx;

    let _ = Text::with_alignment(
        "WI-FI HOTSPOT & SERVER",
        Point::new(cx, layout.title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let status_str = if status.active {
        "STATUS: ACTIVE (HOTSPOT RUNNING)"
    } else {
        "STATUS: STOPPED (OFFLINE)"
    };
    let _ = Text::with_alignment(
        status_str,
        Point::new(cx, layout.status_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::with_alignment(
        "SOFTAP ACCESS CREDENTIALS",
        Point::new(cx, layout.cred_title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::new(
        "Network (SSID): PaperMono-AP",
        Point::new(i32::from(layout.body_x), layout.ssid_y),
        style,
    )
    .draw(&mut ink);

    let _ = Text::new(
        "WPA2 Password:",
        Point::new(i32::from(layout.body_x), layout.pass_label_y),
        style,
    )
    .draw(&mut ink);

    const PASS: &[u8] = b"mono2026";
    for (i, &ch) in PASS.iter().enumerate() {
        let ch_str = core::str::from_utf8(core::slice::from_ref(&ch)).unwrap_or("-");
        let bx = i32::from(layout.pass_x0)
            + i32::from(layout.pass_pitch) * i32::try_from(i).unwrap_or(0)
            + i32::from(layout.pass_w / 2);
        let _ = Text::with_alignment(
            ch_str,
            Point::new(bx, layout.pass_baseline),
            style,
            Alignment::Center,
        )
        .draw(&mut ink);
    }

    let _ = Text::new(
        "Web Server URL:",
        Point::new(i32::from(layout.body_x), layout.url_label_y),
        style,
    )
    .draw(&mut ink);
    let _ = Text::new(
        "http://192.168.4.1/",
        Point::new(i32::from(layout.body_x), layout.url_y),
        style,
    )
    .draw(&mut ink);

    let _ = Text::with_alignment(
        "LIVE SERVER TELEMETRY",
        Point::new(cx, layout.telem_title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let mut buf_cli = [0u8; 48];
    let mut w_cli = BufWriter {
        buf: &mut buf_cli,
        pos: 0,
    };
    let _ = write!(w_cli, "Connected Clients (DHCP):  {}", status.clients);
    if let Ok(label) = core::str::from_utf8(&w_cli.buf[..w_cli.pos]) {
        let _ = Text::new(
            label,
            Point::new(i32::from(layout.telem_x), layout.telem_line1_y),
            style,
        )
        .draw(&mut ink);
    }

    let mut buf_req = [0u8; 48];
    let mut w_req = BufWriter {
        buf: &mut buf_req,
        pos: 0,
    };
    let _ = write!(w_req, "HTTP Requests Served:      {}", status.http_requests);
    if let Ok(label) = core::str::from_utf8(&w_req.buf[..w_req.pos]) {
        let _ = Text::new(
            label,
            Point::new(i32::from(layout.telem_x), layout.telem_line2_y),
            style,
        )
        .draw(&mut ink);
    }

    let _ = Text::new(
        "Gateway IP: 192.168.4.1   | Subnet: /24",
        Point::new(i32::from(layout.telem_x), layout.telem_line3_y),
        style,
    )
    .draw(&mut ink);

    let _ = Text::with_alignment(
        "HOW TO CONNECT & TEST",
        Point::new(cx, layout.guide_title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let tut_steps: &[&str] = if rotation.is_portrait() {
        &[
            "1. Tap [ START HOTSPOT ] button below.",
            "2. Connect phone/PC to 'PaperMono-AP'.",
            "3. Enter WPA2 password 'mono2026'.",
            "4. Fetch http://192.168.4.1/ for status.",
        ]
    } else {
        &[
            "1. Tap [ START HOTSPOT ] below.",
            "2. Join Wi-Fi 'PaperMono-AP'.",
            "3. Password 'mono2026'; open /.",
        ]
    };
    let mut tut_y = layout.guide_y0;
    for step in tut_steps {
        let _ =
            Text::new(*step, Point::new(i32::from(layout.guide_x), tut_y), style).draw(&mut ink);
        tut_y += layout.guide_step;
    }

    let button_label = if status.active {
        "[ STOP HOTSPOT ]"
    } else {
        "[ START HOTSPOT ]"
    };
    let btn_label_y = i32::from(btn_y) + i32::from(btn_h / 2) + (GLYPH_H / 2) - 4;
    let _ = Text::with_alignment(
        button_label,
        Point::new(cx, btn_label_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::with_alignment(
        "BUTTON A: Prev   |   BUTTON B: Next",
        Point::new(cx, layout.footer_y),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);
}

/// Shared page-space geometry for Wi-Fi survey and SoftAP cards.
struct WifiCardLayout {
    cx: i32,
    rule_x: u16,
    rule_w: u16,
    title_y: i32,
    header_bar_y: u16,
    banner_x: u16,
    banner_y: u16,
    banner_w: u16,
    banner_h: u16,
    status_y: i32,
    body_x: u16,
    ch_title_y: i32,
    ch_line1_y: i32,
    ch_line2_y: i32,
    ch_line3_y: i32,
    ch_div_y: u16,
    ap_div_y: u16,
    ap_title_y: i32,
    table_x: u16,
    ap_header_y: i32,
    ap_row0_y: i32,
    ap_row_step: i32,
    ap_empty_y: i32,
    guide_div_y: u16,
    guide_title_y: i32,
    guide_x: u16,
    guide_y0: i32,
    guide_step: i32,
    footer_bar_y: u16,
    footer_y: i32,
    // Hotspot-only fields (survey leaves unused at defaults).
    cred_x: u16,
    cred_y: u16,
    cred_w: u16,
    cred_h: u16,
    cred_title_y: i32,
    ssid_y: i32,
    pass_label_y: i32,
    pass_x0: u16,
    pass_y: u16,
    pass_w: u16,
    pass_h: u16,
    pass_pitch: u16,
    pass_baseline: i32,
    url_label_y: i32,
    url_y: i32,
    telem_div_y: u16,
    telem_title_y: i32,
    telem_x: u16,
    telem_line1_y: i32,
    telem_line2_y: i32,
    telem_line3_y: i32,
}

impl WifiCardLayout {
    /// Portrait / landscape tables for the channel-survey card.
    fn survey(rotation: PageRotation) -> Self {
        let (page_w, page_h) = rotation.page_size();
        let cx = i32::from(page_w / 2);
        let rule_x = 30;
        let rule_w = page_w.saturating_sub(60);
        let (_, btn_y, _, _) = wifi_action_rect(rotation);
        if rotation.is_portrait() {
            Self {
                cx,
                rule_x,
                rule_w,
                title_y: 50,
                header_bar_y: 70,
                banner_x: 50,
                banner_y: 85,
                banner_w: 380,
                banner_h: 45,
                status_y: 115,
                body_x: 45,
                ch_title_y: 155,
                ch_line1_y: 182,
                ch_line2_y: 210,
                ch_line3_y: 238,
                ch_div_y: 255,
                ap_div_y: 265,
                ap_title_y: 285,
                table_x: 35,
                ap_header_y: 312,
                ap_row0_y: 338,
                ap_row_step: 28,
                ap_empty_y: 375,
                guide_div_y: 495,
                guide_title_y: 520,
                guide_x: 40,
                guide_y0: 550,
                guide_step: 26,
                footer_bar_y: 740,
                footer_y: 770,
                cred_x: 0,
                cred_y: 0,
                cred_w: 0,
                cred_h: 0,
                cred_title_y: 0,
                ssid_y: 0,
                pass_label_y: 0,
                pass_x0: 0,
                pass_y: 0,
                pass_w: 0,
                pass_h: 0,
                pass_pitch: 0,
                pass_baseline: 0,
                url_label_y: 0,
                url_y: 0,
                telem_div_y: 0,
                telem_title_y: 0,
                telem_x: 0,
                telem_line1_y: 0,
                telem_line2_y: 0,
                telem_line3_y: 0,
            }
        } else {
            // Compress so occupancy + a few APs + short guide + button fit.
            let footer_bar_y = page_h.saturating_sub(40);
            Self {
                cx,
                rule_x: 40,
                rule_w: page_w.saturating_sub(80),
                title_y: 24,
                header_bar_y: 36,
                banner_x: 60,
                banner_y: 44,
                banner_w: page_w.saturating_sub(120),
                banner_h: 36,
                status_y: 68,
                body_x: 50,
                ch_title_y: 96,
                ch_line1_y: 118,
                ch_line2_y: 138,
                ch_line3_y: 158,
                ch_div_y: 168,
                ap_div_y: 172,
                ap_title_y: 192,
                table_x: 45,
                ap_header_y: 212,
                ap_row0_y: 232,
                ap_row_step: 22,
                ap_empty_y: 250,
                guide_div_y: btn_y.saturating_sub(90),
                guide_title_y: i32::from(btn_y.saturating_sub(78)),
                guide_x: 50,
                guide_y0: i32::from(btn_y.saturating_sub(56)),
                guide_step: 20,
                footer_bar_y,
                footer_y: i32::from(page_h.saturating_sub(16)),
                cred_x: 0,
                cred_y: 0,
                cred_w: 0,
                cred_h: 0,
                cred_title_y: 0,
                ssid_y: 0,
                pass_label_y: 0,
                pass_x0: 0,
                pass_y: 0,
                pass_w: 0,
                pass_h: 0,
                pass_pitch: 0,
                pass_baseline: 0,
                url_label_y: 0,
                url_y: 0,
                telem_div_y: 0,
                telem_title_y: 0,
                telem_x: 0,
                telem_line1_y: 0,
                telem_line2_y: 0,
                telem_line3_y: 0,
            }
        }
    }

    /// Portrait / landscape tables for the SoftAP / HTTP card.
    fn hotspot(rotation: PageRotation) -> Self {
        let (page_w, page_h) = rotation.page_size();
        let cx = i32::from(page_w / 2);
        let (_, btn_y, _, _) = wifi_action_rect(rotation);
        if rotation.is_portrait() {
            Self {
                cx,
                rule_x: 30,
                rule_w: 420,
                title_y: 50,
                header_bar_y: 70,
                banner_x: 50,
                banner_y: 85,
                banner_w: 380,
                banner_h: 45,
                status_y: 115,
                body_x: 60,
                ch_title_y: 0,
                ch_line1_y: 0,
                ch_line2_y: 0,
                ch_line3_y: 0,
                ch_div_y: 0,
                ap_div_y: 0,
                ap_title_y: 0,
                table_x: 0,
                ap_header_y: 0,
                ap_row0_y: 0,
                ap_row_step: 0,
                ap_empty_y: 0,
                guide_div_y: 505,
                guide_title_y: 528,
                guide_x: 40,
                guide_y0: 556,
                guide_step: 26,
                footer_bar_y: 740,
                footer_y: 770,
                cred_x: 40,
                cred_y: 145,
                cred_w: 400,
                cred_h: 205,
                cred_title_y: 170,
                ssid_y: 198,
                pass_label_y: 222,
                pass_x0: 72,
                pass_y: 230,
                pass_w: 36,
                pass_h: 44,
                pass_pitch: 42,
                pass_baseline: 260,
                url_label_y: 302,
                url_y: 328,
                telem_div_y: 365,
                telem_title_y: 388,
                telem_x: 50,
                telem_line1_y: 416,
                telem_line2_y: 444,
                telem_line3_y: 472,
            }
        } else {
            let footer_bar_y = page_h.saturating_sub(40);
            let cred_w = page_w.saturating_sub(80);
            Self {
                cx,
                rule_x: 40,
                rule_w: page_w.saturating_sub(80),
                title_y: 22,
                header_bar_y: 34,
                banner_x: 60,
                banner_y: 40,
                banner_w: page_w.saturating_sub(120),
                banner_h: 32,
                status_y: 62,
                body_x: 70,
                ch_title_y: 0,
                ch_line1_y: 0,
                ch_line2_y: 0,
                ch_line3_y: 0,
                ch_div_y: 0,
                ap_div_y: 0,
                ap_title_y: 0,
                table_x: 0,
                ap_header_y: 0,
                ap_row0_y: 0,
                ap_row_step: 0,
                ap_empty_y: 0,
                guide_div_y: btn_y.saturating_sub(78),
                guide_title_y: i32::from(btn_y.saturating_sub(66)),
                guide_x: 50,
                guide_y0: i32::from(btn_y.saturating_sub(48)),
                guide_step: 18,
                footer_bar_y,
                footer_y: i32::from(page_h.saturating_sub(16)),
                cred_x: 40,
                cred_y: 78,
                cred_w,
                cred_h: 130,
                cred_title_y: 96,
                ssid_y: 116,
                pass_label_y: 134,
                pass_x0: 120,
                pass_y: 140,
                pass_w: 36,
                pass_h: 36,
                pass_pitch: 42,
                pass_baseline: 164,
                url_label_y: 186,
                url_y: 204,
                telem_div_y: 220,
                telem_title_y: 238,
                telem_x: 60,
                telem_line1_y: 256,
                telem_line2_y: 274,
                telem_line3_y: 292,
            }
        }
    }
}

/// Renders Card 7: 4-level grayscale tone bands.
///
/// Portrait: four stacked 140-tall bands (historical). Landscape: four boxes
/// across so the 800×480 page is not four cropped portrait bands (sticky-rs).
fn draw_tones(bw: &mut [u8], red: &mut [u8], rotation: PageRotation) {
    const TONES: [u8; 4] = [
        display::GRAY_BLACK,
        display::GRAY_DARK,
        display::GRAY_LIGHT,
        display::GRAY_WHITE,
    ];

    clear(bw, red, display::GRAY_WHITE, rotation);
    let (page_w, page_h) = rotation.page_size();
    if rotation.is_portrait() {
        const BOX_X: u16 = 40;
        const BOX_H: u16 = 140;
        const YS: [u16; 4] = [80, 250, 420, 590];
        let box_w = page_w.saturating_sub(80);
        for (y, tone) in YS.iter().zip(TONES) {
            fill_rect(bw, red, BOX_X, *y, box_w, BOX_H, tone, rotation);
            stroke_rect(
                bw,
                red,
                BOX_X,
                *y,
                box_w,
                BOX_H,
                display::GRAY_BLACK,
                rotation,
            );
        }
    } else {
        const MARGIN_X: u16 = 32;
        const MARGIN_Y: u16 = 48;
        const GAP: u16 = 16;
        let box_w = page_w
            .saturating_sub(MARGIN_X.saturating_mul(2))
            .saturating_sub(GAP.saturating_mul(3))
            / 4;
        let box_h = page_h.saturating_sub(MARGIN_Y.saturating_mul(2));
        for (i, tone) in TONES.iter().enumerate() {
            let x = MARGIN_X + u16::try_from(i).unwrap_or(0) * (box_w + GAP);
            fill_rect(bw, red, x, MARGIN_Y, box_w, box_h, *tone, rotation);
            stroke_rect(
                bw,
                red,
                x,
                MARGIN_Y,
                box_w,
                box_h,
                display::GRAY_BLACK,
                rotation,
            );
        }
    }
}

/// Blits the 1bpp Ferris bitmap onto dual-plane framebuffers at page coordinate `(x0, y0)`.
fn blit_ferris(bw: &mut [u8], red: &mut [u8], x0: i32, y0: i32, rotation: PageRotation) {
    for y in 0..FERRIS_H {
        for x in 0..FERRIS_W {
            if !ferris_ink(x, y) {
                continue;
            }
            let Some(px) = u16::try_from(x0.saturating_add(i32::from(x))).ok() else {
                continue;
            };
            let Some(py) = u16::try_from(y0.saturating_add(i32::from(y))).ok() else {
                continue;
            };
            set_gray_page(bw, red, px, py, display::GRAY_BLACK, rotation);
        }
    }
}

/// Tests whether the bit corresponding to pixel `(x, y)` in the packed 1bpp Ferris bitmap is asserted.
fn ferris_ink(x: u16, y: u16) -> bool {
    let i = usize::from(y) * usize::from(FERRIS_W) + usize::from(x);
    let byte = FERRIS[i / 8];
    let mask = 0x80u8 >> (i % 8);
    byte & mask != 0
}

/// Fills the current page (not always physical 480×800) with the specified grayscale tone.
fn clear(bw: &mut [u8], red: &mut [u8], tone: u8, rotation: PageRotation) {
    let (page_w, page_h) = rotation.page_size();
    fill_rect(bw, red, 0, 0, page_w, page_h, tone, rotation);
}

/// Renders an order-`depth` Koch snowflake centered at `center` with circumradius `r`.
///
/// Coordinates are page pixels for `rotation`, not physical USB-down framebuffer.
fn draw_koch_snowflake(
    bw: &mut [u8],
    red: &mut [u8],
    depth: u32,
    (cx, cy): (u16, u16),
    r: u16,
    tone: u8,
    rotation: PageRotation,
) {
    let cx_q = (i32::from(cx)) << 16;
    let cy_q = (i32::from(cy)) << 16;
    let r_i64 = i64::from(r);

    // In Q16: sin(60 deg) = 56756, cos(60 deg) = 32768.
    let r_sin = (r_i64 * 56756) as i32;
    let r_cos = (r_i64 * 32768) as i32;

    // Top apex (V0)
    let v0 = (cx_q, cy_q - ((i32::from(r)) << 16));

    // Bottom-right apex (V1)
    let v1 = (cx_q + r_sin, cy_q + r_cos);

    // Bottom-left apex (V2)
    let v2 = (cx_q - r_sin, cy_q + r_cos);

    // 3 sides of the base equilateral triangle in clockwise order
    koch_curve(bw, red, depth, v0, v1, tone, rotation);
    koch_curve(bw, red, depth, v1, v2, tone, rotation);
    koch_curve(bw, red, depth, v2, v0, tone, rotation);
}

/// Recursively generates one side of the Koch snowflake in fixed-point Q16 page coordinates.
fn koch_curve(
    bw: &mut [u8],
    red: &mut [u8],
    depth: u32,
    (x0, y0): (i32, i32),
    (x1, y1): (i32, i32),
    tone: u8,
    rotation: PageRotation,
) {
    if depth == 0 {
        let px0 = (x0 + 32768) >> 16;
        let py0 = (y0 + 32768) >> 16;
        let px1 = (x1 + 32768) >> 16;
        let py1 = (y1 + 32768) >> 16;
        draw_line(bw, red, px0, py0, px1, py1, tone, rotation);
        return;
    }

    let ux = (x1 - x0) / 3;
    let uy = (y1 - y0) / 3;

    let p1 = (x0 + ux, y0 + uy);
    let p3 = (p1.0 + ux, p1.1 + uy);

    // Rotate u by -60 degrees to construct the outward-pointing apex P2:
    // x' = ux * cos(-60 deg) - uy * sin(-60 deg)
    // y' = ux * sin(-60 deg) + uy * cos(-60 deg)
    // with cos(-60) = 32768 / 65536, sin(-60) = -56756 / 65536
    const COS_60: i64 = 32768;
    const SIN_NEG_60: i64 = -56756;

    let rot_x = ((i64::from(ux) * COS_60 - i64::from(uy) * SIN_NEG_60) >> 16) as i32;
    let rot_y = ((i64::from(ux) * SIN_NEG_60 + i64::from(uy) * COS_60) >> 16) as i32;

    let p2 = (p1.0 + rot_x, p1.1 + rot_y);

    koch_curve(bw, red, depth - 1, (x0, y0), p1, tone, rotation);
    koch_curve(bw, red, depth - 1, p1, p2, tone, rotation);
    koch_curve(bw, red, depth - 1, p2, p3, tone, rotation);
    koch_curve(bw, red, depth - 1, p3, (x1, y1), tone, rotation);
}

/// Draws a single-pixel line between two page-space integer coordinates using Bresenham's algorithm.
fn draw_line(
    bw: &mut [u8],
    red: &mut [u8],
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    tone: u8,
    rotation: PageRotation,
) {
    let (page_w, page_h) = rotation.page_size();
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && x0 < i32::from(page_w) && y0 >= 0 && y0 < i32::from(page_h) {
            set_gray_page(bw, red, x0 as u16, y0 as u16, tone, rotation);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Minimal non-allocating string buffer writer for on-screen metrics formatting.
///
/// Implements [`core::fmt::Write`] to enable standard string formatting macros like
/// [`core::fmt::write!`] directly into a caller-supplied fixed-size stack byte slice (`&mut [u8]`)
/// without triggering heap allocation or panic risks in `no_std` environments.
struct BufWriter<'a> {
    /// Destination byte buffer holding formatted ASCII characters.
    buf: &'a mut [u8],
    /// Current write offset into `buf`.
    pos: usize,
}

impl<'a> core::fmt::Write for BufWriter<'a> {
    /// Copies as many bytes of string slice `s` into `buf` as remaining capacity permits.
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remain = self.buf.len().saturating_sub(self.pos);
        let to_copy = bytes.len().min(remain);
        self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;
        Ok(())
    }
}

/// Draws an upward-pointing filled isosceles triangle in page coordinates.
fn fill_triangle_up(
    bw: &mut [u8],
    red: &mut [u8],
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    tone: u8,
    rotation: PageRotation,
) {
    if w < 2 || h == 0 {
        return;
    }
    for row in 0..h {
        let remain = h.saturating_sub(1).saturating_sub(row);
        let inset = (u32::from(w) * u32::from(remain) / (2 * u32::from(h))).min(u32::from(w / 2));
        let inset = inset as u16;
        let ww = w.saturating_sub(inset.saturating_mul(2)).max(1);
        fill_rect(
            bw,
            red,
            x.saturating_add(inset),
            y.saturating_add(row),
            ww,
            1,
            tone,
            rotation,
        );
    }
}

/// Fills a rectangular region in page coordinates with the specified grayscale tone.
fn fill_rect(
    bw: &mut [u8],
    red: &mut [u8],
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    tone: u8,
    rotation: PageRotation,
) {
    let (page_w, page_h) = rotation.page_size();
    for yy in y..y.saturating_add(h).min(page_h) {
        for xx in x..x.saturating_add(w).min(page_w) {
            set_gray_page(bw, red, xx, yy, tone, rotation);
        }
    }
}

/// Draws the single-pixel outline of a rectangle in page coordinates.
fn stroke_rect(
    bw: &mut [u8],
    red: &mut [u8],
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    tone: u8,
    rotation: PageRotation,
) {
    if w == 0 || h == 0 {
        return;
    }
    let (page_w, page_h) = rotation.page_size();
    let x1 = x.saturating_add(w).saturating_sub(1);
    let y1 = y.saturating_add(h).saturating_sub(1);
    for xx in x..=x1.min(page_w.saturating_sub(1)) {
        set_gray_page(bw, red, xx, y, tone, rotation);
        set_gray_page(bw, red, xx, y1, tone, rotation);
    }
    for yy in y..=y1.min(page_h.saturating_sub(1)) {
        set_gray_page(bw, red, x, yy, tone, rotation);
        set_gray_page(bw, red, x1, yy, tone, rotation);
    }
}

/// Sets a page-space pixel by mapping through [`display::page_to_framebuffer`].
fn set_gray_page(
    bw: &mut [u8],
    red: &mut [u8],
    px: u16,
    py: u16,
    tone: u8,
    rotation: PageRotation,
) {
    let Some((x, y)) = display::page_to_framebuffer(px, py, rotation) else {
        return;
    };
    set_gray(bw, red, x, y, tone);
}

/// Sets a pixel at physical USB-down `(x, y)` to the specified 4-gray level across both planes.
fn set_gray(bw: &mut [u8], red: &mut [u8], x: u16, y: u16, tone: u8) {
    let (p1, p2) = display::gray_planes(tone);
    set_plane(bw, x, y, p1);
    set_plane(red, x, y, p2);
}

/// Sets or clears a single bit at physical `(x, y)` in a packed 1bpp framebuffer plane.
fn set_plane(plane: &mut [u8], x: u16, y: u16, on: bool) {
    if x >= display::WIDTH || y >= display::HEIGHT {
        return;
    }
    let i = usize::from(y) * display::BYTES_PER_ROW + usize::from(x) / 8;
    let mask = 0x80u8 >> (x % 8);
    if on {
        plane[i] |= mask;
    } else {
        plane[i] &= !mask;
    }
}

/// Target adapter providing an [`embedded_graphics::draw_target::DrawTarget`] implementation.
///
/// Coordinates are page pixels for [`Self::rotation`]; each inked pixel is mapped
/// through [`set_gray_page`] onto the physical USB-down planes.
struct GrayInk<'a> {
    /// Black/white SSD1677 OTP gray plane.
    bw: &'a mut [u8],
    /// Second plane of the OTP gray pair.
    red: &'a mut [u8],
    /// In-plane hold; [`OriginDimensions`] uses [`PageRotation::page_size`].
    rotation: PageRotation,
}

impl<'a> GrayInk<'a> {
    /// Creates a new drawing target borrowing the dual-plane framebuffers.
    fn new(bw: &'a mut [u8], red: &'a mut [u8], rotation: PageRotation) -> Self {
        Self { bw, red, rotation }
    }
}

impl OriginDimensions for GrayInk<'_> {
    fn size(&self) -> Size {
        let (w, h) = self.rotation.page_size();
        Size::new(u32::from(w), u32::from(h))
    }
}

impl DrawTarget for GrayInk<'_> {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if color != BinaryColor::On || point.x < 0 || point.y < 0 {
                continue;
            }
            let Ok(x) = u16::try_from(point.x) else {
                continue;
            };
            let Ok(y) = u16::try_from(point.y) else {
                continue;
            };
            set_gray_page(self.bw, self.red, x, y, display::GRAY_BLACK, self.rotation);
        }
        Ok(())
    }
}
