//! Official-portrait cards. USB-C down only. No LUT.

use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};
use m5stack_papermono_lite::display;
use papermono_log::Scene;

/// 360×240 packed 1bpp Ferris. Provenance: `assets/SOURCE.md`.
const FERRIS: &[u8] = include_bytes!("../assets/ferris.1bpp");
const FERRIS_W: u16 = 360;
const FERRIS_H: u16 = 240;
const FERRIS_BYTES: usize = (FERRIS_W as usize * FERRIS_H as usize) / 8;

const _: () = assert!(FERRIS.len() == FERRIS_BYTES);
const _: () = assert!(FERRIS_W.is_multiple_of(8));

pub fn render(scene: Scene, bw: &mut [u8], red: &mut [u8]) {
    match scene {
        Scene::Splash => draw_splash(bw, red),
        Scene::Shapes => draw_shapes(bw, red),
        Scene::Legend => draw_legend(bw, red),
        Scene::Tones => draw_tones(bw, red),
        Scene::Targets => {}
    }
}

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
        "slide right edge for lamp",
        Point::new(cx, hint2_y),
        style,
        Alignment::Center,
    )
    .draw(&mut GrayInk::new(bw, red));
}

fn draw_shapes(bw: &mut [u8], red: &mut [u8]) {
    clear(bw, red, display::GRAY_WHITE);
    fill_rect(bw, red, 48, 64, 200, 140, display::GRAY_BLACK);
    fill_disk(bw, red, 320, 280, 90, display::GRAY_BLACK);
    fill_triangle_up(bw, red, 80, 480, 280, 220, display::GRAY_BLACK);
}

fn draw_legend(bw: &mut [u8], red: &mut [u8]) {
    const BOX: u16 = 72;
    const MARGIN: u16 = 8;
    const GAP: i32 = 12;
    // BUTTON A (UP) box centre. Lite, USB-C down, live with the
    // upper black key (2026-09-01).
    const A_Y: u16 = 80;
    // BUTTON B (DOWN) box centre. Close under A, lower black key.
    const B_Y: u16 = 188;
    // Red power key box centre, not the RGB LED (~400). Lite,
    // USB-C down, operator photo 2026-09-01.
    const POWER_Y: u16 = 510;

    clear(bw, red, display::GRAY_WHITE);
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let left_x = MARGIN;
    let right_x = display::WIDTH - MARGIN - BOX;
    let text_left = i32::from(left_x + BOX) + GAP;

    key_box(
        bw,
        red,
        left_x,
        A_Y,
        BOX,
        "BUTTON A (UP)",
        text_left,
        style,
        Alignment::Left,
    );
    key_box(
        bw,
        red,
        left_x,
        B_Y,
        BOX,
        "BUTTON B (DOWN)",
        text_left,
        style,
        Alignment::Left,
    );
    key_box(
        bw,
        red,
        left_x,
        POWER_Y,
        BOX,
        "POWER",
        text_left,
        style,
        Alignment::Left,
    );
    let _ = Text::with_alignment(
        "double-click: off",
        Point::new(text_left, i32::from(POWER_Y) + 28),
        style,
        Alignment::Left,
    )
    .draw(&mut GrayInk::new(bw, red));
    let _ = Text::with_alignment(
        "hold: download",
        Point::new(text_left, i32::from(POWER_Y) + 50),
        style,
        Alignment::Left,
    )
    .draw(&mut GrayInk::new(bw, red));

    let text_right = i32::from(right_x) - GAP;
    key_box(
        bw,
        red,
        right_x,
        A_Y,
        BOX,
        "LAMP UP",
        text_right,
        style,
        Alignment::Right,
    );
    key_box(
        bw,
        red,
        right_x,
        640,
        BOX,
        "LAMP DOWN",
        text_right,
        style,
        Alignment::Right,
    );
}

#[allow(clippy::too_many_arguments)]
fn key_box(
    bw: &mut [u8],
    red: &mut [u8],
    x: u16,
    center: u16,
    box_s: u16,
    label: &str,
    text_x: i32,
    style: MonoTextStyle<'_, BinaryColor>,
    align: Alignment,
) {
    let y = center.saturating_sub(box_s / 2);
    fill_rect(bw, red, x, y, box_s, box_s, display::GRAY_BLACK);
    let _ = Text::with_alignment(
        label,
        Point::new(text_x, i32::from(center) + 6),
        style,
        align,
    )
    .draw(&mut GrayInk::new(bw, red));
}

fn draw_tones(bw: &mut [u8], red: &mut [u8]) {
    const BOX_W: u16 = 360;
    const BOX_H: u16 = 140;
    const BOX_X: u16 = 60;
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

fn ferris_ink(x: u16, y: u16) -> bool {
    let i = usize::from(y) * usize::from(FERRIS_W) + usize::from(x);
    let byte = FERRIS[i / 8];
    let mask = 0x80u8 >> (i % 8);
    byte & mask != 0
}

fn clear(bw: &mut [u8], red: &mut [u8], tone: u8) {
    fill_rect(bw, red, 0, 0, display::WIDTH, display::HEIGHT, tone);
}

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

fn fill_rect(bw: &mut [u8], red: &mut [u8], x: u16, y: u16, w: u16, h: u16, tone: u8) {
    for yy in y..y.saturating_add(h).min(display::HEIGHT) {
        for xx in x..x.saturating_add(w).min(display::WIDTH) {
            set_gray(bw, red, xx, yy, tone);
        }
    }
}

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

fn set_gray(bw: &mut [u8], red: &mut [u8], x: u16, y: u16, tone: u8) {
    let (p1, p2) = display::gray_planes(tone);
    set_plane(bw, x, y, p1);
    set_plane(red, x, y, p2);
}

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

struct GrayInk<'a> {
    bw: &'a mut [u8],
    red: &'a mut [u8],
}

impl<'a> GrayInk<'a> {
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
