//! Official-portrait 480×800 UI card rendering engine.
//!
//! # Architecture & Display Pipeline
//! This module renders interactive graphical cards directly into two 1-bit-per-pixel
//! framebuffers (`bw` and `red`), which combine to form the 4-level grayscale representation
//! accepted by the SSD1677 e-paper controller:
//!
//! - **Grayscale Encoding**:
//!   - `display::GRAY_BLACK` (tone 0): `bw=0, red=0`
//!   - `display::GRAY_DARK` (tone 1): `bw=1, red=0`
//!   - `display::GRAY_LIGHT` (tone 2): `bw=0, red=1`
//!   - `display::GRAY_WHITE` (tone 3): `bw=1, red=1`
//! - **Embedded Graphics Integration**: Implements the [`embedded_graphics::draw_target::DrawTarget`]
//!   trait via [`GrayInk`], allowing standard text, shapes, and primitives to be rendered directly
//!   into the dual-plane framebuffers.
//! - **Eight-Card Walkthrough**:
//!   1. `Splash`: Displays the Rust Ferris mascot and navigation hints.
//!   2. `Shapes`: Verifies geometry rendering (procedural 3-degree Koch snowflake with microsecond benchmark, triangles, boxes).
//!   3. `Legend`: Provides an on-device quick-reference visual guide for physical buttons, sleep/wake, and touch rails.
//!   4. `Bluetooth`: Displays 6-digit BLE passkey PIN for phone pairing and reports success or failure reason.
//!   5. `WifiSurvey`: Scans 2.4 GHz 802.11 channels, displays channel distribution and top discovered APs.
//!   6. `WifiAp`: Runs WPA2-Personal SoftAP with DHCP and serves JSON system stats over HTTP.
//!   7. `Tones`: 4-gray horizontal bars demonstrating OTP grayscale palette accuracy.
//!   8. `Targets`: Monochromatic calibration points for digitizer latency and accuracy testing.

use core::fmt::Write;
use embassy_time::Instant;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};
use m5stack_papermono_lite::{display, pmic};
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

/// Renders the requested interactive card scene into dual-plane framebuffers.
/// Returns the benchmark render duration in microseconds if the scene computes one.
pub fn render(
    scene: Scene,
    bw: &mut [u8],
    red: &mut [u8],
    charge: Option<ChargeSample>,
) -> Option<u32> {
    match scene {
        Scene::Splash => {
            draw_splash(bw, red);
            None
        }
        Scene::Shapes => Some(draw_shapes(bw, red)),
        Scene::Legend => {
            draw_legend(bw, red, charge);
            None
        }
        Scene::Bluetooth => {
            draw_bluetooth(bw, red);
            None
        }
        Scene::WifiSurvey => {
            draw_wifi_survey(bw, red);
            None
        }
        Scene::WifiAp => {
            draw_wifi_ap(bw, red);
            None
        }
        Scene::Tones => {
            draw_tones(bw, red);
            None
        }
        Scene::Targets => None,
    }
}

/// Renders the sleep screen notice before the device enters low-power light sleep.
#[cfg(feature = "sleep")]
pub fn draw_sleeping(bw: &mut [u8], red: &mut [u8]) {
    const LINE_GAP: i32 = 12;
    const GLYPH_H: i32 = 20;

    clear(bw, red, display::GRAY_WHITE);
    let cx = i32::from(display::WIDTH) / 2;
    let cy = i32::from(display::HEIGHT) / 2;
    let line1_y = cy - (LINE_GAP / 2);
    let line2_y = cy + (LINE_GAP / 2) + GLYPH_H;

    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let _ = Text::with_alignment(
        "sleeping,",
        Point::new(cx, line1_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red));
    let _ = Text::with_alignment(
        "press A or B for 1 second to restart",
        Point::new(cx, line2_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red));
}

/// Renders Card 1: Ferris mascot splash screen and user navigation guide.
fn draw_splash(bw: &mut [u8], red: &mut [u8]) {
    const TITLE_GAP: i32 = 28;
    const HINT_GAP: i32 = 28;
    const LINE_GAP: i32 = 8;
    const GLYPH_H: i32 = 20;

    clear(bw, red, display::GRAY_WHITE);
    let ferris_h = i32::from(FERRIS_H);
    let stack_h = ferris_h + TITLE_GAP + GLYPH_H + HINT_GAP + GLYPH_H + LINE_GAP + GLYPH_H;
    let top = (i32::from(display::HEIGHT) - stack_h) / 2;
    let cx = i32::from(display::WIDTH) / 2;
    let ferris_x = (i32::from(display::WIDTH) - i32::from(FERRIS_W)) / 2;
    let title_y = top + ferris_h + TITLE_GAP + GLYPH_H;
    let hint1_y = title_y + HINT_GAP + GLYPH_H;
    let hint2_y = hint1_y + LINE_GAP + GLYPH_H;

    blit_ferris(bw, red, ferris_x, top);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let _ = Text::with_alignment(
        "papermono-rs",
        Point::new(cx, title_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red));
    let _ = Text::with_alignment(
        "BUTTON A / B change cards",
        Point::new(cx, hint1_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red));
    let _ = Text::with_alignment(
        "Right edge slides frontlight",
        Point::new(cx, hint2_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red));
}

/// Renders Card 2: Geometric test patterns validating aspect ratio, display orientation, and procedural rendering.
fn draw_shapes(bw: &mut [u8], red: &mut [u8]) -> u32 {
    clear(bw, red, display::GRAY_WHITE);
    stroke_rect(
        bw,
        red,
        0,
        0,
        display::WIDTH,
        display::HEIGHT,
        display::GRAY_BLACK,
    );
    stroke_rect(
        bw,
        red,
        16,
        16,
        display::WIDTH - 32,
        display::HEIGHT - 32,
        display::GRAY_DARK,
    );

    let start = Instant::now();
    draw_koch_snowflake(bw, red, 3, (240, 200), 135, display::GRAY_BLACK);
    let elapsed_us = Instant::now().duration_since(start).as_micros() as u32;

    fill_triangle_up(bw, red, 70, 360, 140, 120, display::GRAY_LIGHT);
    fill_rect(bw, red, 270, 360, 140, 120, display::GRAY_BLACK);
    let cx = i32::from(display::WIDTH) / 2;
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let _ = Text::with_alignment(
        "Geometric Calibration",
        Point::new(cx, 580),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red));

    let mut buf = [0u8; 48];
    let mut writer = BufWriter {
        buf: &mut buf,
        pos: 0,
    };
    let _ = write!(writer, "Koch Snowflake (3 deg): {elapsed_us} us");
    if let Ok(label) = core::str::from_utf8(&writer.buf[..writer.pos]) {
        let _ = Text::with_alignment(label, Point::new(cx, 615), style, Alignment::Center)
            .draw(&mut GrayInk::new(bw, red));
    }

    elapsed_us
}

/// Renders Card 3: Legend displaying hardware pinout, button functions, sleep controls, and battery telemetry.
fn draw_legend(bw: &mut [u8], red: &mut [u8], charge: Option<ChargeSample>) {
    clear(bw, red, display::GRAY_WHITE);
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
        let mut ink = GrayInk::new(bw, red);
        let _ = Text::with_alignment(
            "HARDWARE LEGEND",
            Point::new(240, 50),
            style,
            Alignment::Center,
        )
        .draw(&mut ink);

        let mut y = 90;
        for (k, v) in items {
            let _ = Text::new(k, Point::new(30, y), style).draw(&mut ink);
            let _ = Text::new(v, Point::new(50, y + 25), style).draw(&mut ink);
            y += 70;
            if k == "EXPANDER (M5IOE1)" {
                y += 10;
            }
        }
    }

    // Status section separator
    fill_rect(bw, red, 30, 638, 420, 2, display::GRAY_BLACK);

    let (pct, vbat, vin, usb) = match charge {
        Some(c) => {
            let pct = pmic::battery_percent(c.vbat);
            let usb = (c.src & pmic::PWR_SRC_VIN != 0) || (c.vin >= pmic::VIN_PRESENT_MV);
            (Some(pct), c.vbat, c.vin, usb)
        }
        None => (None, 0, 0, false),
    };

    // Battery gauge outline: 104x20 at (30, 680)
    stroke_rect(bw, red, 30, 680, 104, 20, display::GRAY_BLACK);
    // Positive terminal cap: 4x10 at (134, 685)
    fill_rect(bw, red, 134, 685, 4, 10, display::GRAY_BLACK);

    if let Some(p) = pct {
        let fill_w = u16::from(p.min(100));
        fill_rect(bw, red, 32, 682, fill_w, 16, display::GRAY_BLACK);
    }

    let mut ink = GrayInk::new(bw, red);
    let _ = Text::new("BATTERY & POWER STATUS", Point::new(30, 665), style).draw(&mut ink);

    // Text metrics next to gauge (vertically aligned with gauge):
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
        let _ = Text::new(label, Point::new(150, 697), style).draw(&mut ink);
    }

    // Line 2: Power supply details
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
        let _ = Text::new(label, Point::new(30, 727), style).draw(&mut ink);
    }

    // Line 3: 1S LiPo specifications
    let _ = Text::new(
        "1S LiPo: 3300 mV (0%) - 4150 mV (100%)",
        Point::new(30, 755),
        style,
    )
    .draw(&mut ink);
}

/// Renders Card 4: Bluetooth Low Energy peripheral pairing with PIN display and status.
///
/// # Visual Hierarchy & Layout Geometry (480x800 Portrait)
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
fn draw_bluetooth(bw: &mut [u8], red: &mut [u8]) {
    // Fill the entire 480x800 canvas with paper white before drawing.
    clear(bw, red, display::GRAY_WHITE);
    let status = crate::radio::pair_status();

    // 1. Draw structural lines and frames
    // Header separator bar (420 px wide, 2 px high).
    fill_rect(bw, red, 30, 70, 420, 2, display::GRAY_BLACK);

    // Outer double-bordered frame for the 6-digit PIN code container.
    stroke_rect(bw, red, 60, 175, 360, 90, display::GRAY_BLACK);
    stroke_rect(bw, red, 63, 178, 354, 84, display::GRAY_BLACK);

    // Draw either a consolidated "PAIRED" banner box or six discrete digit boxes.
    if status == BlePairStatus::Success {
        stroke_rect(bw, red, 90, 195, 300, 50, display::GRAY_BLACK);
    } else {
        // Six 40x50 px boxes spaced at 52 px intervals (x = 90, 142, 194, 246, 298, 350).
        for i in 0..6 {
            let x = 90 + i * 52;
            stroke_rect(bw, red, x, 195, 40, 50, display::GRAY_BLACK);
        }
    }

    // Horizontal separator separating PIN container from status messages.
    fill_rect(bw, red, 30, 295, 420, 2, display::GRAY_BLACK);

    // Prominent double-line status outline box for terminal states (Success / Failed).
    match status {
        BlePairStatus::Success | BlePairStatus::Failed(_) => {
            stroke_rect(bw, red, 100, 315, 280, 45, display::GRAY_BLACK);
            stroke_rect(bw, red, 102, 317, 276, 41, display::GRAY_BLACK);
        }
        _ => {}
    }

    // Light gray dividing line above the tutorial walkthrough steps.
    fill_rect(bw, red, 30, 480, 420, 1, display::GRAY_LIGHT);

    // Footer navigation dividing bar.
    fill_rect(bw, red, 30, 720, 420, 2, display::GRAY_BLACK);

    // 2. Draw all text with a single GrayInk to avoid multiple mutable borrow conflicts.
    let mut ink = GrayInk::new(bw, red);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

    // Card Header
    let _ = Text::with_alignment(
        "BLUETOOTH PAIRING",
        Point::new(240, 50),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    // BLE Local Name
    let _ = Text::with_alignment(
        "Device: PaperMono",
        Point::new(240, 110),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    // Contextual instruction string above the PIN boxes.
    let instruction = match status {
        BlePairStatus::Pairing(_) => "Enter this PIN code on your phone:",
        BlePairStatus::Success => "Device paired and connected!",
        BlePairStatus::Failed(_) => "Pairing attempt failed",
        BlePairStatus::Connected => "Connecting, negotiating pairing...",
        BlePairStatus::Advertising => "Discoverable as 'PaperMono'",
        BlePairStatus::Disabled => "BLE radio disabled in build",
    };
    let _ = Text::with_alignment(instruction, Point::new(240, 145), style, Alignment::Center)
        .draw(&mut ink);

    // PIN Content: render individual digits, "P A I R E D", or idle placeholder hyphens.
    match status {
        BlePairStatus::Pairing(pin) => {
            // Decompose 6-digit numeric passkey into individual ASCII digits.
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
                // Center each digit within its 40x50 px box (box left = 90 + i*52, box center = +20).
                let x = 90 + (i as i32) * 52 + 20;
                let _ =
                    Text::with_alignment(char_str, Point::new(x, 227), style, Alignment::Center)
                        .draw(&mut ink);
            }
        }
        BlePairStatus::Success => {
            let _ = Text::with_alignment(
                "P A I R E D",
                Point::new(240, 227),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        _ => {
            // Placeholder hyphens when no active passkey is generated.
            for i in 0..6 {
                let x = 90 + i * 52 + 20;
                let _ = Text::with_alignment("-", Point::new(x, 227), style, Alignment::Center)
                    .draw(&mut ink);
            }
        }
    }

    // Result section: display status label, details, and actionable guidance.
    match status {
        BlePairStatus::Success => {
            let _ = Text::with_alignment("SUCCESS", Point::new(240, 345), style, Alignment::Center)
                .draw(&mut ink);
            let _ = Text::with_alignment(
                "Bluetooth connection encrypted.",
                Point::new(240, 395),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Failed(reason) => {
            let _ = Text::with_alignment("FAILED", Point::new(240, 345), style, Alignment::Center)
                .draw(&mut ink);
            // Format failure reason into stack buffer without dynamic allocation.
            let mut buf = [0u8; 64];
            let mut writer = BufWriter {
                buf: &mut buf,
                pos: 0,
            };
            let _ = write!(writer, "Why: {}", reason.as_str());
            if let Ok(label) = core::str::from_utf8(&writer.buf[..writer.pos]) {
                let _ = Text::with_alignment(label, Point::new(240, 395), style, Alignment::Center)
                    .draw(&mut ink);
            }
            let _ = Text::with_alignment(
                "Retry pairing from phone settings.",
                Point::new(240, 425),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Connected => {
            let _ = Text::with_alignment(
                "Status: Phone connected",
                Point::new(240, 345),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
            let _ = Text::with_alignment(
                "Awaiting passkey exchange...",
                Point::new(240, 395),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Pairing(_) => {
            let _ = Text::with_alignment(
                "Status: Pairing in progress",
                Point::new(240, 345),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
            let _ = Text::with_alignment(
                "Enter PIN shown above on phone",
                Point::new(240, 395),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Advertising => {
            let _ = Text::with_alignment(
                "Status: Ready to pair",
                Point::new(240, 345),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
            let _ = Text::with_alignment(
                "Select 'PaperMono' in phone Bluetooth",
                Point::new(240, 395),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
        BlePairStatus::Disabled => {
            let _ = Text::with_alignment(
                "Status: Radio disabled in build",
                Point::new(240, 345),
                style,
                Alignment::Center,
            )
            .draw(&mut ink);
        }
    }

    // Step-by-step phone pairing tutorial instructions.
    let _ = Text::with_alignment(
        "HOW TO PAIR",
        Point::new(240, 510),
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
    let mut step_y = 545;
    for step in steps {
        let _ = Text::new(step, Point::new(45, step_y), style).draw(&mut ink);
        step_y += 35;
    }

    // Navigation footer
    let _ = Text::with_alignment(
        "BUTTON A: Prev   |   BUTTON B: Next",
        Point::new(240, 755),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);
}

/// Renders Card 5: 2.4 GHz Wi-Fi channel survey with channel saturation histogram and top AP table.
///
/// # Visual Hierarchy & Layout Geometry (480x800 Portrait)
/// - **Header (y = 50..70)**: Centered title "WI-FI CHANNEL SURVEY" with black dividing bar.
/// - **Status Banner (y = 85..130)**:
///   - Outlined double-border box (380x45 px) indicating active scan or idle state.
/// - **Channel Distribution (y = 150..250)**:
///   - Section header "2.4 GHz CHANNEL OCCUPANCY" and divider rule.
///   - Aggregated count of total networks discovered.
///   - Non-overlapping primary channels (1, 6, 11) and secondary channel counts.
/// - **Strongest Access Points (y = 265..485)**:
///   - Section header "STRONGEST ACCESS POINTS" with table column headers.
///   - Up to 5 strongest AP records showing SSID, channel, RSSI (dBm), and security method.
/// - **Operational Walkthrough (y = 505..645)**:
///   - Explanatory notes on passive beacon sniffing, channel occupancy, and mutual exclusion with SoftAP.
///     Guide lines stay within ~40 glyphs (FONT_10X20 at x = 40) so they do not clip the right edge.
/// - **Touch Button Box (y = 660..716)**:
///   - Prominent double-line border `[ START SURVEY ]` or `[ STOP SURVEY ]` responsive to touch taps.
/// - **Footer (y = 740..780)**: Navigation guide for Button A / Button B.
fn draw_wifi_survey(bw: &mut [u8], red: &mut [u8]) {
    // Fill canvas with paper white before rendering card elements.
    clear(bw, red, display::GRAY_WHITE);
    let mode = crate::radio::wifi_mode();
    let survey = crate::radio::wifi_survey_data();

    // 1. Draw structural lines, dividers, and bounding boxes.
    // Header separator bar (420 px wide, 2 px high).
    fill_rect(bw, red, 30, 70, 420, 2, display::GRAY_BLACK);

    // Status banner double-line container.
    stroke_rect(bw, red, 50, 85, 380, 45, display::GRAY_BLACK);
    stroke_rect(bw, red, 52, 87, 376, 41, display::GRAY_BLACK);

    // Light divider below channel distribution metrics.
    fill_rect(bw, red, 40, 255, 400, 1, display::GRAY_LIGHT);

    // Divider separating channel distribution from AP table.
    fill_rect(bw, red, 30, 265, 420, 2, display::GRAY_BLACK);

    // Light divider separating AP table from operation guide.
    fill_rect(bw, red, 30, 495, 420, 1, display::GRAY_LIGHT);

    // Touch button bounding box (x = 60..420, y = 660..716).
    stroke_rect(bw, red, 60, 660, 360, 56, display::GRAY_BLACK);
    stroke_rect(bw, red, 62, 662, 356, 52, display::GRAY_BLACK);

    // Footer navigation dividing rule.
    fill_rect(bw, red, 30, 740, 420, 2, display::GRAY_BLACK);

    // 2. Render all typography using GrayInk rasterizer.
    let mut ink = GrayInk::new(bw, red);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

    // Title header
    let _ = Text::with_alignment(
        "WI-FI CHANNEL SURVEY",
        Point::new(240, 50),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    // Status banner text
    let status_str = match mode {
        WifiMode::SurveyScanning => "STATUS: SCANNING CHANNELS...",
        WifiMode::SurveyComplete => "STATUS: SCAN COMPLETE",
        WifiMode::Hotspot => "STATUS: HOTSPOT ACTIVE",
        WifiMode::Idle => "STATUS: IDLE (READY TO SCAN)",
    };
    let _ = Text::with_alignment(status_str, Point::new(240, 115), style, Alignment::Center)
        .draw(&mut ink);

    // 2.4 GHz channel distribution section
    let _ = Text::with_alignment(
        "2.4 GHz CHANNEL OCCUPANCY",
        Point::new(240, 155),
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
        let _ = Text::new(label, Point::new(45, 182), style).draw(&mut ink);
    }

    let mut buf_ch = [0u8; 64];
    let mut w_ch = BufWriter {
        buf: &mut buf_ch,
        pos: 0,
    };
    let _ = write!(w_ch, "Ch 1: {:>2}   |   Ch 6: {:>2}", ch1, ch6);
    if let Ok(label) = core::str::from_utf8(&w_ch.buf[..w_ch.pos]) {
        let _ = Text::new(label, Point::new(45, 210), style).draw(&mut ink);
    }

    let mut buf_ch2 = [0u8; 64];
    let mut w_ch2 = BufWriter {
        buf: &mut buf_ch2,
        pos: 0,
    };
    let _ = write!(w_ch2, "Ch 11: {:>2}  |   Other: {:>2}", ch11, other);
    if let Ok(label) = core::str::from_utf8(&w_ch2.buf[..w_ch2.pos]) {
        let _ = Text::new(label, Point::new(45, 238), style).draw(&mut ink);
    }

    // Strongest access points table
    let _ = Text::with_alignment(
        "STRONGEST ACCESS POINTS",
        Point::new(240, 285),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::new(
        "SSID           CH     RSSI     AUTH     ",
        Point::new(35, 312),
        style,
    )
    .draw(&mut ink);

    let has_entries = survey.as_ref().is_some_and(|s| s.top_aps[0].is_some());
    if has_entries {
        if let Some(ref data) = survey {
            let mut row_y = 338;
            for ap in data.top_aps.iter().flatten() {
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
                    let _ = Text::new(label, Point::new(35, row_y), style).draw(&mut ink);
                }
                row_y += 28;
            }
        }
    } else {
        let _ = Text::with_alignment(
            "No networks scanned yet",
            Point::new(240, 375),
            style,
            Alignment::Center,
        )
        .draw(&mut ink);
    }

    // Survey operation walkthrough
    let _ = Text::with_alignment(
        "SURVEY OPERATION",
        Point::new(240, 520),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let guide_lines = [
        "1. Tap below to scan channels 1-13.",
        "2. Identifies 2.4 GHz channel saturation.",
        "3. Top 4 APs listed by signal (RSSI).",
        "4. Starting survey halts SoftAP if running.",
    ];
    let mut guide_y = 550;
    for line in guide_lines {
        let _ = Text::new(line, Point::new(40, guide_y), style).draw(&mut ink);
        guide_y += 26;
    }

    // Touch button label
    let button_label = if mode == WifiMode::SurveyScanning {
        "[ STOP SURVEY ]"
    } else {
        "[ START SURVEY ]"
    };
    let _ = Text::with_alignment(button_label, Point::new(240, 696), style, Alignment::Center)
        .draw(&mut ink);

    // Navigation footer
    let _ = Text::with_alignment(
        "BUTTON A: Prev   |   BUTTON B: Next",
        Point::new(240, 770),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);
}

/// Renders Card 6: WPA2-Personal Wi-Fi SoftAP and embedded HTTP web server status.
///
/// # Visual Hierarchy & Layout Geometry (480x800 Portrait)
/// - **Header (y = 50..70)**: Centered title "WI-FI HOTSPOT & SERVER" with black dividing bar.
/// - **Status Banner (y = 85..130)**:
///   - Outlined double-border box (380x45 px) indicating whether the SoftAP is Stopped or Active.
/// - **SoftAP Access Credentials (y = 145..355)**:
///   - Bordered box (400x205 px) displaying SSID ("PaperMono-AP").
///   - 8 individual segmented digit boxes (36x44 px each) displaying WPA2 passkey "mono2026".
///   - Web server URL endpoint ("http://192.168.4.1/").
/// - **Live Server Telemetry (y = 370..495)**:
///   - Real-time DHCP client count, HTTP requests served, and gateway network subnet.
/// - **Tutorial Walkthrough (y = 515..645)**:
///   - Step-by-step connection guide for phones, tablets, and host PCs.
/// - **Touch Button Box (y = 660..716)**:
///   - Prominent double-line border `[ START HOTSPOT ]` or `[ STOP HOTSPOT ]` responsive to touch taps.
/// - **Footer (y = 740..780)**: Navigation guide for Button A / Button B.
fn draw_wifi_ap(bw: &mut [u8], red: &mut [u8]) {
    // Fill canvas with paper white before rendering card elements.
    clear(bw, red, display::GRAY_WHITE);
    let status = crate::radio::wifi_ap_status();

    // 1. Draw structural lines, dividers, and credential containers.
    // Header separator bar (420 px wide, 2 px high).
    fill_rect(bw, red, 30, 70, 420, 2, display::GRAY_BLACK);

    // Status banner double-line container.
    stroke_rect(bw, red, 50, 85, 380, 45, display::GRAY_BLACK);
    stroke_rect(bw, red, 52, 87, 376, 41, display::GRAY_BLACK);

    // SoftAP access credentials container box.
    stroke_rect(bw, red, 40, 145, 400, 205, display::GRAY_BLACK);
    stroke_rect(bw, red, 42, 147, 396, 201, display::GRAY_BLACK);

    // 8 segmented password digit boxes for "mono2026".
    for i in 0..8 {
        let bx = 72 + (i as u16) * 42;
        stroke_rect(bw, red, bx, 230, 36, 44, display::GRAY_BLACK);
    }

    // Telemetry divider.
    fill_rect(bw, red, 40, 365, 400, 1, display::GRAY_LIGHT);

    // Tutorial guide divider.
    fill_rect(bw, red, 30, 505, 420, 1, display::GRAY_LIGHT);

    // Touch button bounding box (x = 60..420, y = 660..716).
    stroke_rect(bw, red, 60, 660, 360, 56, display::GRAY_BLACK);
    stroke_rect(bw, red, 62, 662, 356, 52, display::GRAY_BLACK);

    // Footer navigation dividing rule.
    fill_rect(bw, red, 30, 740, 420, 2, display::GRAY_BLACK);

    // 2. Render all typography using GrayInk rasterizer.
    let mut ink = GrayInk::new(bw, red);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

    // Title header
    let _ = Text::with_alignment(
        "WI-FI HOTSPOT & SERVER",
        Point::new(240, 50),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    // Status banner text
    let status_str = if status.active {
        "STATUS: ACTIVE (HOTSPOT RUNNING)"
    } else {
        "STATUS: STOPPED (OFFLINE)"
    };
    let _ = Text::with_alignment(status_str, Point::new(240, 115), style, Alignment::Center)
        .draw(&mut ink);

    // Credentials container content
    let _ = Text::with_alignment(
        "SOFTAP ACCESS CREDENTIALS",
        Point::new(240, 170),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let _ = Text::new("Network (SSID): PaperMono-AP", Point::new(60, 198), style).draw(&mut ink);

    let _ = Text::new("WPA2 Password:", Point::new(60, 222), style).draw(&mut ink);

    // Segmented password characters
    const PASS: &[u8] = b"mono2026";
    for (i, &ch) in PASS.iter().enumerate() {
        let ch_str = core::str::from_utf8(core::slice::from_ref(&ch)).unwrap_or("-");
        let bx = 72 + (i as i32) * 42 + 18;
        let _ = Text::with_alignment(ch_str, Point::new(bx, 260), style, Alignment::Center)
            .draw(&mut ink);
    }

    let _ = Text::new("Web Server URL:", Point::new(60, 302), style).draw(&mut ink);
    let _ = Text::new("http://192.168.4.1/", Point::new(60, 328), style).draw(&mut ink);

    // Live telemetry section
    let _ = Text::with_alignment(
        "LIVE SERVER TELEMETRY",
        Point::new(240, 388),
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
        let _ = Text::new(label, Point::new(50, 416), style).draw(&mut ink);
    }

    let mut buf_req = [0u8; 48];
    let mut w_req = BufWriter {
        buf: &mut buf_req,
        pos: 0,
    };
    let _ = write!(w_req, "HTTP Requests Served:      {}", status.http_requests);
    if let Ok(label) = core::str::from_utf8(&w_req.buf[..w_req.pos]) {
        let _ = Text::new(label, Point::new(50, 444), style).draw(&mut ink);
    }

    let _ = Text::new(
        "Gateway IP: 192.168.4.1   | Subnet: /24",
        Point::new(50, 472),
        style,
    )
    .draw(&mut ink);

    // Tutorial walkthrough
    let _ = Text::with_alignment(
        "HOW TO CONNECT & TEST",
        Point::new(240, 528),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let tut_steps = [
        "1. Tap [ START HOTSPOT ] button below.",
        "2. Connect phone/PC to 'PaperMono-AP'.",
        "3. Enter WPA2 password 'mono2026'.",
        "4. Fetch http://192.168.4.1/ for status.",
    ];
    let mut tut_y = 556;
    for step in tut_steps {
        let _ = Text::new(step, Point::new(40, tut_y), style).draw(&mut ink);
        tut_y += 26;
    }

    // Touch button label
    let button_label = if status.active {
        "[ STOP HOTSPOT ]"
    } else {
        "[ START HOTSPOT ]"
    };
    let _ = Text::with_alignment(button_label, Point::new(240, 696), style, Alignment::Center)
        .draw(&mut ink);

    // Navigation footer
    let _ = Text::with_alignment(
        "BUTTON A: Prev   |   BUTTON B: Next",
        Point::new(240, 770),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);
}

/// Renders Card 7: 4-level grayscale horizontal tone bands.
fn draw_tones(bw: &mut [u8], red: &mut [u8]) {
    const BOX_X: u16 = 40;
    const BOX_W: u16 = display::WIDTH - 80;
    const BOX_H: u16 = 140;

    const BOXES: [(u16, u8); 4] = [
        (80, display::GRAY_BLACK),
        (250, display::GRAY_DARK),
        (420, display::GRAY_LIGHT),
        (590, display::GRAY_WHITE),
    ];
    clear(bw, red, display::GRAY_WHITE);
    for (y, tone) in BOXES {
        fill_rect(bw, red, BOX_X, y, BOX_W, BOX_H, tone);
        stroke_rect(bw, red, BOX_X, y, BOX_W, BOX_H, display::GRAY_BLACK);
    }
}

/// Blits the 1bpp Ferris bitmap onto dual-plane framebuffers at the specified coordinate.
fn blit_ferris(bw: &mut [u8], red: &mut [u8], x0: i32, y0: i32) {
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
            set_gray(bw, red, px, py, display::GRAY_BLACK);
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

/// Fills the entire dual-plane framebuffer with the specified grayscale tone.
fn clear(bw: &mut [u8], red: &mut [u8], tone: u8) {
    fill_rect(bw, red, 0, 0, display::WIDTH, display::HEIGHT, tone);
}

/// Renders an order-`depth` Koch snowflake centered at `center` with circumradius `r`.
fn draw_koch_snowflake(
    bw: &mut [u8],
    red: &mut [u8],
    depth: u32,
    (cx, cy): (u16, u16),
    r: u16,
    tone: u8,
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
    koch_curve(bw, red, depth, v0, v1, tone);
    koch_curve(bw, red, depth, v1, v2, tone);
    koch_curve(bw, red, depth, v2, v0, tone);
}

/// Recursively generates one side of the Koch snowflake in fixed-point Q16 coordinates.
fn koch_curve(
    bw: &mut [u8],
    red: &mut [u8],
    depth: u32,
    (x0, y0): (i32, i32),
    (x1, y1): (i32, i32),
    tone: u8,
) {
    if depth == 0 {
        let px0 = (x0 + 32768) >> 16;
        let py0 = (y0 + 32768) >> 16;
        let px1 = (x1 + 32768) >> 16;
        let py1 = (y1 + 32768) >> 16;
        draw_line(bw, red, px0, py0, px1, py1, tone);
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

    koch_curve(bw, red, depth - 1, (x0, y0), p1, tone);
    koch_curve(bw, red, depth - 1, p1, p2, tone);
    koch_curve(bw, red, depth - 1, p2, p3, tone);
    koch_curve(bw, red, depth - 1, p3, (x1, y1), tone);
}

/// Draws a single-pixel line between two integer coordinates using Bresenham's algorithm.
fn draw_line(bw: &mut [u8], red: &mut [u8], mut x0: i32, mut y0: i32, x1: i32, y1: i32, tone: u8) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && x0 < i32::from(display::WIDTH) && y0 >= 0 && y0 < i32::from(display::HEIGHT) {
            set_gray(bw, red, x0 as u16, y0 as u16, tone);
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

/// Draws an upward-pointing filled isosceles triangle.
fn fill_triangle_up(bw: &mut [u8], red: &mut [u8], x: u16, y: u16, w: u16, h: u16, tone: u8) {
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
        );
    }
}

/// Fills a rectangular region with the specified grayscale tone.
fn fill_rect(bw: &mut [u8], red: &mut [u8], x: u16, y: u16, w: u16, h: u16, tone: u8) {
    for yy in y..y.saturating_add(h).min(display::HEIGHT) {
        for xx in x..x.saturating_add(w).min(display::WIDTH) {
            set_gray(bw, red, xx, yy, tone);
        }
    }
}

/// Draws the single-pixel outline of a rectangle.
fn stroke_rect(bw: &mut [u8], red: &mut [u8], x: u16, y: u16, w: u16, h: u16, tone: u8) {
    if w == 0 || h == 0 {
        return;
    }
    let x1 = x.saturating_add(w).saturating_sub(1);
    let y1 = y.saturating_add(h).saturating_sub(1);
    for xx in x..=x1.min(display::WIDTH.saturating_sub(1)) {
        set_gray(bw, red, xx, y, tone);
        set_gray(bw, red, xx, y1, tone);
    }
    for yy in y..=y1.min(display::HEIGHT.saturating_sub(1)) {
        set_gray(bw, red, x, yy, tone);
        set_gray(bw, red, x1, yy, tone);
    }
}

/// Sets a pixel at `(x, y)` to the specified 4-gray level across both `bw` and `red` planes.
fn set_gray(bw: &mut [u8], red: &mut [u8], x: u16, y: u16, tone: u8) {
    let (p1, p2) = display::gray_planes(tone);
    set_plane(bw, x, y, p1);
    set_plane(red, x, y, p2);
}

/// Sets or clears a single bit at `(x, y)` in a packed 1bpp framebuffer plane.
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
struct GrayInk<'a> {
    bw: &'a mut [u8],
    red: &'a mut [u8],
}

impl<'a> GrayInk<'a> {
    /// Creates a new drawing target borrowing the dual-plane framebuffers.
    fn new(bw: &'a mut [u8], red: &'a mut [u8]) -> Self {
        Self { bw, red }
    }
}

impl OriginDimensions for GrayInk<'_> {
    fn size(&self) -> Size {
        Size::new(u32::from(display::WIDTH), u32::from(display::HEIGHT))
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
            set_gray(self.bw, self.red, x, y, display::GRAY_BLACK);
        }
        Ok(())
    }
}
