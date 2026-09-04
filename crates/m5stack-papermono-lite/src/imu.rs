//! BMI270 on the system I2C bus.
//!
//! Catalog id `bmi270`. Soft-reset then load the Bosch standard
//! 8 KiB config ([`bmi270_config`] / [`bmi270_config.SOURCE.md`])
//! so raw accelerometer samples in `DATA_8`…`DATA_13` are valid.
//! Orientation classification follows the sticky-rs dominant-axis
//! policy; Lite (`C153-Lite`) axis→pose is X↔Y vs sticky LSM6
//! (USB-C down = −X; see [`classify`]).

use crate::display::PageRotation;
use embedded_hal::i2c::I2c;

/// 7-bit address. Same as [`crate::addresses::BMI270`].
pub const ADDRESS: u8 = crate::addresses::BMI270;

/// Catalog id `bmi270`, section 5.2.1 Register (0x00) `CHIP_ID`.
pub const CHIP_ID: u8 = 0x00;

/// POR `CHIP_ID` payload. Catalog id `bmi270`, section 5.2.1.
pub const CHIP_ID_VALUE: u8 = 0x24;

/// Catalog id `bmi270`, section 5.2.12 Register (0x0C) `DATA_8`.
pub const DATA_8: u8 = 0x0C;

/// Catalog id `bmi270`, section 5.2.33 Register (0x21) `INTERNAL_STATUS`.
pub const INTERNAL_STATUS: u8 = 0x21;

/// Catalog id `bmi270`, section 5.2.41 Register (0x40) `ACC_CONF`.
pub const ACC_CONF: u8 = 0x40;

/// Catalog id `bmi270`, section 5.2.42 Register (0x41) `ACC_RANGE`.
pub const ACC_RANGE: u8 = 0x41;

/// Catalog id `bmi270`, Register (0x5B) `INIT_ADDR_0`.
pub const INIT_ADDR_0: u8 = 0x5B;

/// Catalog id `bmi270`, Register (0x5C) `INIT_ADDR_1`.
pub const INIT_ADDR_1: u8 = 0x5C;

/// Catalog id `bmi270`, Register (0x59) `INIT_CTRL`.
pub const INIT_CTRL: u8 = 0x59;

/// Catalog id `bmi270`, Register (0x5E) `INIT_DATA`.
pub const INIT_DATA: u8 = 0x5E;

/// Catalog id `bmi270`, section 5.2.84 Register (0x7C) `PWR_CONF`.
pub const PWR_CONF: u8 = 0x7C;

/// Catalog id `bmi270`, section 5.2.85 Register (0x7D) `PWR_CTRL`.
pub const PWR_CTRL: u8 = 0x7D;

/// Catalog id `bmi270`, section 5.2.86 Register (0x7E) `CMD`.
pub const CMD: u8 = 0x7E;

/// Soft-reset command byte written to [`CMD`] (`0xB6`).
pub const CMD_SOFTRESET: u8 = 0xB6;

/// `PWR_CTRL.acc_en` only.
pub const PWR_CTRL_ACC_EN: u8 = 0x04;

/// `ACC_CONF`: normal filter, ~100 Hz ODR (`0xA8`).
pub const ACC_CONF_NORMAL_100HZ: u8 = 0xA8;

/// `ACC_RANGE` ±2 g (`0x00`).
pub const ACC_RANGE_2G: u8 = 0x00;

/// `PWR_CONF` with `adv_power_save` cleared (datasheet init: `0x00`).
pub const PWR_CONF_APS_OFF: u8 = 0x00;

/// Burst length used when streaming [`BMI270_CONFIG`] (Bosch Sensor API default).
pub const CONFIG_CHUNK: usize = 16;

/// Accelerometer sensitivity at ±2 g, LSB per g (datasheet `S2g`).
pub const SENSITIVITY_LSB_PER_G: i32 = 16_384;

/// Dominant-axis threshold in g (sticky-rs calibrated figure).
pub const DOMINANT_AXIS_THRESHOLD_G: f32 = 0.70;

/// Same threshold in raw LSB at ±2 g.
pub const DOMINANT_AXIS_THRESHOLD_LSB: i32 =
    (DOMINANT_AXIS_THRESHOLD_G * SENSITIVITY_LSB_PER_G as f32) as i32;

/// Bosch standard init blob (8192 bytes).
pub use crate::bmi270_config::BMI270_CONFIG;

/// One accelerometer sample in sensor frame (raw int16 at ±2 g).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccelSample {
    /// Sensor X axis (raw LSB).
    pub x: i16,
    /// Sensor Y axis (raw LSB).
    pub y: i16,
    /// Sensor Z axis (raw LSB).
    pub z: i16,
}

/// Enclosure orientation from accelerometer gravity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// USB-C on the bottom short edge.
    Portrait0,
    /// USB-C on the top short edge.
    Portrait180,
    /// USB-C on the right short edge.
    Landscape0,
    /// USB-C on the left short edge.
    Landscape180,
    /// Gravity dominant on +Z: lying face up.
    FaceUp,
    /// Gravity dominant on −Z: lying face down.
    FaceDown,
}

impl Orientation {
    /// In-plane page for this pose. `None` when the unit is flat.
    #[must_use]
    pub const fn page_rotation(self) -> Option<PageRotation> {
        match self {
            Self::Portrait0 => Some(PageRotation::Portrait0),
            Self::Portrait180 => Some(PageRotation::Portrait180),
            Self::Landscape0 => Some(PageRotation::Landscape0),
            Self::Landscape180 => Some(PageRotation::Landscape180),
            Self::FaceUp | Self::FaceDown => None,
        }
    }

    /// CDC / log token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Portrait0 => "Portrait0",
            Self::Portrait180 => "Portrait180",
            Self::Landscape0 => "Landscape0",
            Self::Landscape180 => "Landscape180",
            Self::FaceUp => "FaceUp",
            Self::FaceDown => "FaceDown",
        }
    }
}

/// Soft-reset the BMI270. Wait ≥2 ms (prefer ~30 ms), then
/// [`disable_adv_power_save`].
pub fn soft_reset<I2C, E>(i2c: &mut I2C) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    i2c.write(ADDRESS, &[CMD, CMD_SOFTRESET])
}

/// Clears `PWR_CONF.adv_power_save` (`0x00`). Wait ≥450 µs before
/// [`load_config`].
pub fn disable_adv_power_save<I2C, E>(i2c: &mut I2C) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    i2c.write(ADDRESS, &[PWR_CONF, PWR_CONF_APS_OFF])
}

/// Streams [`BMI270_CONFIG`] via `INIT_ADDR_*` + `INIT_DATA`, then
/// asserts `INIT_CTRL`. Wait ≥20 ms, then [`read_internal_status`].
pub fn load_config<I2C, E>(i2c: &mut I2C) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    i2c.write(ADDRESS, &[INIT_CTRL, 0x00])?;
    let mut index = 0usize;
    while index < BMI270_CONFIG.len() {
        let end = (index + CONFIG_CHUNK).min(BMI270_CONFIG.len());
        let chunk = &BMI270_CONFIG[index..end];
        // Bosch Sensor API `upload_file`: word address = byte_index / 2.
        let word = (index / 2) as u16;
        let addr0 = (word & 0x0F) as u8;
        let addr1 = (word >> 4) as u8;
        i2c.write(ADDRESS, &[INIT_ADDR_0, addr0, addr1])?;
        let mut buf = [0u8; CONFIG_CHUNK + 1];
        buf[0] = INIT_DATA;
        buf[1..=chunk.len()].copy_from_slice(chunk);
        i2c.write(ADDRESS, &buf[..=chunk.len()])?;
        index = end;
    }
    i2c.write(ADDRESS, &[INIT_CTRL, 0x01])
}

/// Reads [`INTERNAL_STATUS`]. Bit 0 set ⇒ init message OK.
pub fn read_internal_status<I2C, E>(i2c: &mut I2C) -> Result<u8, E>
where
    I2C: I2c<Error = E>,
{
    let mut status = [0u8];
    i2c.write_read(ADDRESS, &[INTERNAL_STATUS], &mut status)?;
    Ok(status[0])
}

/// Configures ±2 g accel at ~100 Hz and sets `PWR_CTRL.acc_en`.
pub fn enable_accel_sampling<I2C, E>(i2c: &mut I2C) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    // APS must stay off for unrestricted config writes.
    i2c.write(ADDRESS, &[PWR_CONF, PWR_CONF_APS_OFF])?;
    i2c.write(ADDRESS, &[ACC_CONF, ACC_CONF_NORMAL_100HZ])?;
    i2c.write(ADDRESS, &[ACC_RANGE, ACC_RANGE_2G])?;
    i2c.write(ADDRESS, &[PWR_CTRL, PWR_CTRL_ACC_EN])
}

/// Reads one shadowed accel sample (`DATA_8`…`DATA_13`).
pub fn read_accel<I2C, E>(i2c: &mut I2C) -> Result<AccelSample, E>
where
    I2C: I2c<Error = E>,
{
    let mut raw = [0u8; 6];
    i2c.write_read(ADDRESS, &[DATA_8], &mut raw)?;
    Ok(AccelSample {
        x: i16::from_le_bytes([raw[0], raw[1]]),
        y: i16::from_le_bytes([raw[2], raw[3]]),
        z: i16::from_le_bytes([raw[4], raw[5]]),
    })
}

/// Classifies orientation from a raw accelerometer sample.
///
/// Returns `None` when no axis passes [`DOMINANT_AXIS_THRESHOLD_LSB`].
///
/// **Lite (`C153-Lite`, 2026-09-04) axis map** (glass toward operator):
/// USB-C down was dominant on **X** (not Y), so X↔Y vs sticky-rs LSM6.
/// Confirmed: USB-C down painted a landscape page before this swap;
/// USB-C left painted a portrait page. Signs: −X → USB-C down
/// ([`Orientation::Portrait0`]), +X → USB-C up. Landscape Y signs
/// were inverted once (2026-09-04): +Y → USB-C right
/// ([`Orientation::Landscape0`]), −Y → USB-C left
/// ([`Orientation::Landscape180`]).
#[must_use]
pub fn classify(x: i16, y: i16, z: i16) -> Option<Orientation> {
    let (x, y, z) = (i32::from(x), i32::from(y), i32::from(z));

    let dominant = [x.abs(), y.abs(), z.abs()]
        .into_iter()
        .max()
        .unwrap_or_default();
    if dominant < DOMINANT_AXIS_THRESHOLD_LSB {
        return None;
    }

    // PaperMono BMI270: X is the short-edge (USB) axis; Y is the long-edge axis.
    Some(if x.abs() == dominant {
        if x < 0 {
            Orientation::Portrait0
        } else {
            Orientation::Portrait180
        }
    } else if y.abs() == dominant {
        if y < 0 {
            Orientation::Landscape180
        } else {
            Orientation::Landscape0
        }
    } else if z > 0 {
        Orientation::FaceUp
    } else {
        Orientation::FaceDown
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_G: i16 = 16_384;

    #[test]
    fn chip_id_register_is_not_the_payload() {
        assert_eq!(ADDRESS, crate::addresses::BMI270);
        assert_eq!(CHIP_ID, 0x00);
        assert_eq!(CHIP_ID_VALUE, 0x24);
        assert_ne!(CHIP_ID, CHIP_ID_VALUE);
        assert_eq!(INTERNAL_STATUS, 0x21);
    }

    #[test]
    fn standard_config_is_8192_bytes() {
        assert_eq!(BMI270_CONFIG.len(), 8192);
    }

    #[test]
    fn threshold_matches_sticky_figure() {
        assert_eq!(DOMINANT_AXIS_THRESHOLD_LSB, 11_468);
    }

    #[test]
    fn each_axis_maps_to_the_enclosure_pose() {
        // Lite BMI270: X = USB short-edge axis, Y = long-edge (2026-09-04).
        assert_eq!(classify(-ONE_G, 0, 0), Some(Orientation::Portrait0));
        assert_eq!(classify(ONE_G, 0, 0), Some(Orientation::Portrait180));
        assert_eq!(classify(0, ONE_G, 0), Some(Orientation::Landscape0));
        assert_eq!(classify(0, -ONE_G, 0), Some(Orientation::Landscape180));
        assert_eq!(classify(0, 0, ONE_G), Some(Orientation::FaceUp));
        assert_eq!(classify(0, 0, -ONE_G), Some(Orientation::FaceDown));
    }

    #[test]
    fn ambiguous_sample_is_unknown() {
        let component = (0.6 * f32::from(ONE_G)) as i16;
        assert_eq!(classify(component, component, 0), None);
        assert_eq!(classify(0, 0, 0), None);
    }

    #[test]
    fn face_up_is_not_confused_with_portrait() {
        let noise = 1_000;
        assert_eq!(classify(noise, noise, ONE_G), Some(Orientation::FaceUp));
    }

    #[test]
    fn only_in_plane_poses_have_a_page_rotation() {
        assert_eq!(
            Orientation::Portrait0.page_rotation(),
            Some(PageRotation::Portrait0)
        );
        assert_eq!(Orientation::FaceUp.page_rotation(), None);
    }
}
