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
//! - **Five-Card Walkthrough**:
//!   1. `Splash`: Displays the Rust Ferris mascot and navigation hints.
//!   2. `Shapes`: Verifies geometry rendering (disks, rounded boxes, triangles, intersecting crossbars).
//!   3. `Legend`: Provides an on-device quick-reference visual guide for physical buttons and touch rails.
//!   4. `Tones`: 4-gray horizontal bars demonstrating OTP grayscale palette accuracy.
//!   5. `Targets`: Monochromatic calibration points for digitizer latency and accuracy testing.

use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};
use m5stack_papermono_lite::display;
use papermono_log::Scene;

/// 360×240 packed 1bpp bitmap of Ferris the Rust mascot.
/// Provenance: Generated from SVG via `cargo xtask encode-assets` (see `assets/SOURCE.md`).
const FERRIS: &[u8] = include_bytes!("../assets/ferris.1bpp");
const FERRIS_W: u16 = 360;
const FERRIS_H: u16 = 240;
const FERRIS_BYTES: usize = (FERRIS_W as usize * FERRIS_H as usize) / 8;

const _: () = assert!(FERRIS.len() == FERRIS_BYTES);
const _: () = assert!(FERRIS_W.is_multiple_of(8));

/// Renders the requested interactive card scene into dual-plane framebuffers.
pub fn render(scene: Scene, bw: &mut [u8], red: &mut [u8]) {
    match scene {
        Scene::Splash => draw_splash(bw, red),
        Scene::Shapes => draw_shapes(bw, red),
        Scene::Legend => draw_legend(bw, red),
        Scene::Tones => draw_tones(bw, red),
        Scene::Targets => {}
    }
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

/// Renders Card 2: Geometric test patterns validating aspect ratio and display orientation.
fn draw_shapes(bw: &mut [u8], red: &mut [u8]) {
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
    fill_disk(bw, red, 120, 200, 70, display::GRAY_BLACK);
    fill_disk(bw, red, 360, 200, 70, display::GRAY_DARK);
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
}

/// Renders Card 3: Legend displaying hardware pinout and button functions.
fn draw_legend(bw: &mut [u8], red: &mut [u8]) {
    clear(bw, red, display::GRAY_WHITE);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let mut ink = GrayInk::new(bw, red);

    let _ = Text::with_alignment(
        "HARDWARE LEGEND",
        Point::new(240, 50),
        style,
        Alignment::Center,
    )
    .draw(&mut ink);

    let items = [
        ("BUTTON A (GPIO2)", "Previous card / hold for mic"),
        ("BUTTON B (GPIO3)", "Next card"),
        ("RIGHT GUTTER", "Frontlight brightness slider"),
        ("TOUCH DIGITIZER", "FT6336G capacitive I2C (0x38)"),
        ("PMIC (M5PM1)", "Power rails, buttons, battery ADC"),
        ("EXPANDER (M5IOE1)", "Power gates & peripheral resets"),
        ("POWER BUTTON", "Short press wake / hold download"),
    ];

    let mut y = 110;
    for (k, v) in items {
        let _ = Text::new(k, Point::new(30, y), style).draw(&mut ink);
        let _ = Text::new(v, Point::new(50, y + 25), style).draw(&mut ink);
        y += 65;
        // The power button item is placed about 10 pixels lower as requested.
        if k == "EXPANDER (M5IOE1)" {
            y += 10;
        }
    }
}

/// Renders Card 4: 4-level grayscale horizontal tone bands.
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

/// Draws a filled circle at center `(cx, cy)` with radius `r`.
fn fill_disk(bw: &mut [u8], red: &mut [u8], cx: u16, cy: u16, r: u16, tone: u8) {
    let r2 = i32::from(r) * i32::from(r);
    let x0 = cx.saturating_sub(r);
    let y0 = cy.saturating_sub(r);
    let x1 = cx.saturating_add(r).min(display::WIDTH.saturating_sub(1));
    let y1 = cy.saturating_add(r).min(display::HEIGHT.saturating_sub(1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = i32::from(x) - i32::from(cx);
            let dy = i32::from(y) - i32::from(cy);
            if dx * dx + dy * dy <= r2 {
                set_gray(bw, red, x, y, tone);
            }
        }
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
