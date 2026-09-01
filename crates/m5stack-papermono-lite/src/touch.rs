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

/// How close to each active-area end a slide must reach.
pub const SLIDE_END_INSET: u16 = 80;
/// Drawn line half-width (pixels).
pub const SLIDE_HALF_W: u16 = 6;

/// Decode one M5GFX `getTouchRaw` buffer (status at `[0]`).
///
/// `x = (data[1] & 0x0F) << 8 | data[2]` (same for each
/// `idx * 6` point). Not a public `ft6336g` map.
#[must_use]
pub fn decode_m5gfx(buf: &[u8]) -> Option<(u8, u16, u16, u16, u16)> {
    if buf.is_empty() {
        return None;
    }
    let n = (buf[0] & 0x0F).min(MAX_POINTS);
    let point = |idx: usize| -> (u16, u16) {
        let base = idx * M5GFX_POINT_BYTES;
        if buf.len() < base + 5 {
            return (0, 0);
        }
        let x = (u16::from(buf[base + 1] & 0x0F) << 8) | u16::from(buf[base + 2]);
        let y = (u16::from(buf[base + 3] & 0x0F) << 8) | u16::from(buf[base + 4]);
        (x, y)
    };
    let (x, y) = if n >= 1 { point(0) } else { (0, 0) };
    let (x2, y2) = if n >= 2 { point(1) } else { (0, 0) };
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
    }
}
