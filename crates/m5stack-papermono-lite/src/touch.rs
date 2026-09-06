//! FT6336G on the system I2C bus.
//!
//! Reset and VDD are expander nets ([`crate::ioe1::TOUCH_RST`],
//! [`crate::ioe1::TOUCH_VDD_ENABLE`]). INT is [`crate::pins::TOUCH_INT`].
//! XY decode is M5GFX `getTouchRaw` ([`decode_m5gfx`]), not a
//! public `ft6336g` map. Lite (2026-09-01): official-portrait
//! samples matched ink.

/// 7-bit address. Same as [`crate::addresses::FT6336G`].
pub const ADDRESS: u8 = crate::addresses::FT6336G;

/// Touch active-area minimum X (official tables).
///
/// Lite (`C153-Lite`, 2026-09-01) midline slides reached 5.
pub const ACTIVE_MIN_X: u16 = 5;
/// Touch active-area maximum X (official tables).
///
/// Lite slides reached 475.
pub const ACTIVE_MAX_X: u16 = 475;
/// Touch active-area minimum Y (official tables).
///
/// Lite slides reached 5.
pub const ACTIVE_MIN_Y: u16 = 5;
/// Touch active-area maximum Y (official tables).
///
/// Lite slides reached 795.
pub const ACTIVE_MAX_Y: u16 = 795;

/// Official PaperMono Arduino example `MAX_TOUCH_POINTS`.
///
/// Public `ft6336g` FEATURES: 1 point + gestures / 2 points.
/// Lite walk (2026-09-01) saw `n=1` only. Two-point on this
/// FPC is still `nyc-ft6336-points`.
pub const MAX_POINTS: u8 = 2;

/// M5GFX `Touch_FT5x06::getTouchRaw` start register (`reg_number = 2`).
///
/// Not in the public `ft6336g` PDF. Cite M5GFX / official
/// `getTouchRaw` example, not a FocalTech map.
pub const M5GFX_STATUS_REG: u8 = 2;

/// Bytes after the status byte for one M5GFX point (6). XY uses 4.
pub const M5GFX_POINT_BYTES: usize = 6;

/// Finger slop vs a drawn target (pixels, official portrait).
pub const TARGET_SLOP_PX: u16 = 100;

/// Drawn target radius (pixels). Smaller than [`TARGET_SLOP_PX`].
pub const TARGET_RADIUS_PX: u16 = 48;

/// How long to wait for a tap or BUTTON B abort.
pub const TARGET_WAIT_MS: u64 = 90_000;

/// Right-edge lamp gutter width (official portrait pixels).
///
/// USB-C down: contact in `x >= `[`ACTIVE_MAX_X`]`- this`
/// sets PWM0 from Y (top bright, USB-C dim).
///
/// 80 px starts at 395. The targets-card dots at official
/// `(400, 80)` and `(400, 720)` sit inside that strip.
/// The walk scores a slop hit before `LampSlide::feed`.
pub const LAMP_GUTTER_PX: u16 = 80;
/// PWM0 duty counts per official-Y pixel while in the gutter.
pub const LAMP_DUTY_PER_PX: u16 = 8;

/// True when official-portrait `x` is in the right-edge lamp strip.
#[must_use]
pub const fn in_lamp_gutter(x: u16) -> bool {
    x + LAMP_GUTTER_PX >= ACTIVE_MAX_X
}

/// Gutter width for left and right edge sliders (pixels in page coordinates).
pub const GUTTER_PX: u16 = 80;

/// Inset from top and bottom page edges for edge slider deadbands (pixels).
///
/// Ensures fingers reaching the bottom bezel reliably hit 0 (lamp off, volume 0)
/// and fingers reaching the top bezel reliably hit 100% full scale, accounting
/// for physical FT6336G active-area limits (5–795) and human fingertip radius.
pub const SLIDER_INSET_PX: u16 = 40;

/// True when page-space `px` is in the left-edge volume slider strip.
#[must_use]
pub const fn in_page_left_gutter(px: u16) -> bool {
    px <= GUTTER_PX
}

/// True when page-space `px` is in the right-edge lamp slider strip.
#[must_use]
pub const fn in_page_right_gutter(px: u16, page_w: u16) -> bool {
    px.saturating_add(GUTTER_PX) >= page_w
}

/// How close to each active-area end a slide must reach.
pub const SLIDE_END_INSET: u16 = 80;
/// Drawn line half-width (pixels).
pub const SLIDE_HALF_W: u16 = 6;

/// Decode one M5GFX `getTouchRaw` buffer (status at `[0]`).
///
/// `x = (data[1] & 0x0F) << 8 | data[2]` (same for each
/// `idx * 6` point). Validates event flags and coordinate bounds.
/// Not a public `ft6336g` map.
#[must_use]
pub fn decode_m5gfx(buf: &[u8]) -> Option<(u8, u16, u16, u16, u16)> {
    if buf.is_empty() {
        return None;
    }
    let raw_n = buf[0] & 0x0F;
    if raw_n > MAX_POINTS {
        return Some((0, 0, 0, 0, 0));
    }
    let n = raw_n;
    let point = |idx: usize| -> (u16, u16) {
        let base = idx * M5GFX_POINT_BYTES;
        if buf.len() < base + 5 {
            return (0, 0);
        }
        let event = buf[base + 1] >> 6;
        if event == 1 || event == 3 {
            // Lift Up (1) or No Event (3)
            return (0, 0);
        }
        let x = (u16::from(buf[base + 1] & 0x0F) << 8) | u16::from(buf[base + 2]);
        let y = (u16::from(buf[base + 3] & 0x0F) << 8) | u16::from(buf[base + 4]);
        if x >= crate::display::WIDTH || y >= crate::display::HEIGHT {
            return (0, 0);
        }
        (x, y)
    };
    let (x, y) = if n >= 1 { point(0) } else { (0, 0) };
    let (x2, y2) = if n >= 2 { point(1) } else { (0, 0) };
    let n = if n >= 1 && x == 0 && y == 0 { 0 } else { n };
    Some((n, x, y, x2, y2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_area_fits_official_panel() {
        const { assert!(ACTIVE_MIN_X < ACTIVE_MAX_X) };
        const { assert!(ACTIVE_MIN_Y < ACTIVE_MAX_Y) };
        const { assert!(ACTIVE_MAX_X <= crate::display::WIDTH) };
        const { assert!(ACTIVE_MAX_Y <= crate::display::HEIGHT) };
        const { assert!(SLIDER_INSET_PX > ACTIVE_MIN_Y) };
        const { assert!(ACTIVE_MAX_Y > crate::display::HEIGHT - SLIDER_INSET_PX) };
    }

    #[test]
    fn address_matches_the_system_bus_table() {
        assert_eq!(ADDRESS, crate::addresses::FT6336G);
        assert_eq!(MAX_POINTS, 2);
        assert_eq!(M5GFX_STATUS_REG, 2);
        const { assert!(TARGET_RADIUS_PX < TARGET_SLOP_PX) };
        const { assert!(ACTIVE_MIN_X + SLIDE_END_INSET < ACTIVE_MAX_X - SLIDE_END_INSET) };
        const { assert!(ACTIVE_MIN_Y + SLIDE_END_INSET < ACTIVE_MAX_Y - SLIDE_END_INSET) };
        const { assert!(in_lamp_gutter(ACTIVE_MAX_X)) };
        const {
            assert!(!in_lamp_gutter(
                ACTIVE_MAX_X.saturating_sub(LAMP_GUTTER_PX + 1)
            ))
        };
        const { assert!(LAMP_DUTY_PER_PX == 8) };
        // Targets-card first top-right / bottom-right dots.
        const { assert!(in_lamp_gutter(400)) };
    }

    #[test]
    fn page_gutter_boundaries() {
        // Portrait 480x800
        assert!(in_page_left_gutter(0));
        assert!(in_page_left_gutter(80));
        assert!(!in_page_left_gutter(81));
        assert!(!in_page_right_gutter(399, 480));
        assert!(in_page_right_gutter(400, 480));
        assert!(in_page_right_gutter(479, 480));

        // Landscape 800x480
        assert!(in_page_left_gutter(0));
        assert!(in_page_left_gutter(80));
        assert!(!in_page_left_gutter(81));
        assert!(!in_page_right_gutter(719, 800));
        assert!(in_page_right_gutter(720, 800));
        assert!(in_page_right_gutter(799, 800));
    }

    #[test]
    fn m5gfx_one_and_two_points() {
        let one = [1, 0x00, 240, 0x01, 144];
        assert_eq!(decode_m5gfx(&one), Some((1, 240, 400, 0, 0)));
        let mut two = [0u8; 11];
        two[0] = 2;
        two[1] = 0x00;
        two[2] = 80;
        two[3] = 0x00;
        two[4] = 80;
        two[7] = 0x01;
        two[8] = 144;
        two[9] = 0x02;
        two[10] = 208;
        assert_eq!(decode_m5gfx(&two), Some((2, 80, 80, 400, 720)));

        // Floating / unready I2C bus returns 0xFF:
        let floating = [0xFF; 13];
        assert_eq!(decode_m5gfx(&floating), Some((0, 0, 0, 0, 0)));

        // Event flag 1 (Lift Up) on point 0:
        let lift = [1, 0x40, 240, 0x01, 144];
        assert_eq!(decode_m5gfx(&lift), Some((0, 0, 0, 0, 0)));

        // Event flag 3 (No Event) on point 0:
        let no_event = [1, 0xC0, 240, 0x01, 144];
        assert_eq!(decode_m5gfx(&no_event), Some((0, 0, 0, 0, 0)));

        // Out-of-bounds coordinate (e.g. 4095):
        let oob = [1, 0x0F, 0xFF, 0x0F, 0xFF];
        assert_eq!(decode_m5gfx(&oob), Some((0, 0, 0, 0, 0)));
    }
}
