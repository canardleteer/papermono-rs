//! M5PM1 PMIC registers and I2C access.
//!
//! Catalog id `m5pm1`, UM V 1.9 Table 3 Register Map. Board nets
//! (which `Gn` is the lamp) live in the product BSP. PWM0 is the
//! multiplexed engine on GPIO3; PWM1 is a different timer.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(test)]
extern crate std;

use embedded_hal::i2c::I2c;

/// Catalog default 7-bit address (UM). PaperMono board is often
/// this value; pass the observed address into [`M5pm1::new`].
pub const DEFAULT_ADDRESS: u8 = 0x6E;

/// Catalog id `m5pm1`, Table 3 Register Map, `Device_ID` (read-only).
pub const DEVICE_ID: u8 = 0x00;
/// Catalog id `m5pm1`, System Registers, `PWR_SRC` (0x04, read-only).
pub const PWR_SRC: u8 = 0x04;
/// [`PWR_SRC`] bit 0: `5VIN` valid.
pub const PWR_SRC_VIN: u8 = 1 << 0;
/// [`PWR_SRC`] bit 1: `5VINOUT` valid (boost off).
pub const PWR_SRC_VINOUT: u8 = 1 << 1;
/// [`PWR_SRC`] bit 2: battery valid.
pub const PWR_SRC_BAT: u8 = 1 << 2;
/// Sheet VIN add/remove threshold (mV).
pub const VIN_PRESENT_MV: u16 = 2400;
/// Catalog id `m5pm1`, System Registers, `WAKE_SRC` (0x05).
pub const WAKE_SRC: u8 = 0x05;
/// [`WAKE_SRC`] bit 5: GPIO / `EXT_WAKE`.
pub const WAKE_SRC_EXT: u8 = 1 << 5;
/// Catalog id `m5pm1`, System Registers, `PWR_CFG` (0x06).
pub const PWR_CFG: u8 = 0x06;
/// [`PWR_CFG`] bit 0: `CHG_EN` (high = charging enabled).
pub const CHG_EN: u8 = 1 << 0;
/// [`PWR_CFG`] bit 4: `LED_EN` (high = red LED on, low = red LED off).
pub const LED_EN: u8 = 1 << 4;
/// Catalog id `m5pm1`, System Registers, `HOLD_CFG` (0x07).
pub const HOLD_CFG: u8 = 0x07;
/// [`HOLD_CFG`] bit 5: LDO 3.3 V hold.
pub const HOLD_LDO: u8 = 1 << 5;
/// [`HOLD_CFG`] bit 6: 5VIN/OUT retain after off.
pub const HOLD_VIN: u8 = 1 << 6;
/// Catalog id `m5pm1`, System Registers, `SYS_CMD` (0x0C).
pub const SYS_CMD: u8 = 0x0C;
/// [`SYS_CMD`] key nibble. Official `M5PM1_SYS_CMD_KEY`.
pub const SYS_CMD_KEY: u8 = 0xA0;
/// [`SYS_CMD`] shutdown (`M5PM1_SYS_CMD_OFF`).
pub const SYS_CMD_SHUTDOWN: u8 = SYS_CMD_KEY | 0x01;
/// Catalog id `m5pm1`, GPIO Register, `GPIO_MODE` (0x10).
pub const GPIO_MODE: u8 = 0x10;
/// Catalog id `m5pm1`, GPIO Register, `GPIO_IN` (0x12).
pub const GPIO_IN: u8 = 0x12;
/// Catalog id `m5pm1`, GPIO Register, `GPIO_DRV` (0x13).
pub const GPIO_DRV: u8 = 0x13;
/// Catalog id `m5pm1`, GPIO Register, `GPIO_PUPD0` (0x14).
pub const GPIO_PUPD0: u8 = 0x14;
/// Pull-up in a 2-bit `PUPD` field.
pub const GPIO_PULL_UP: u8 = 0x01;
/// Catalog id `m5pm1`, GPIO Register, `GPIO_FUNC0` (0x16).
///
/// Bits [7:6] are GPIO3. `11` = multiplexed PWM.
pub const GPIO_FUNC0: u8 = 0x16;
/// GPIO3 multiplex bits in [`GPIO_FUNC0`].
pub const GPIO3_FUNC_MASK: u8 = 0xC0;
/// Multiplexed function: PWM0 on G3 (not PWM1).
pub const GPIO3_FUNC_PWM: u8 = 0xC0;
/// Catalog id `m5pm1`, GPIO Register, `GPIO_WAKE_EN` (0x18).
pub const GPIO_WAKE_EN: u8 = 0x18;
/// Catalog id `m5pm1`, GPIO Register, `GPIO_WAKE_CFG` (0x19).
pub const GPIO_WAKE_CFG: u8 = 0x19;
/// Catalog id `m5pm1`, ADC Register, `VBAT_L` (0x22).
pub const VBAT_L: u8 = 0x22;
/// Catalog id `m5pm1`, ADC Register, `VIN_L` (0x24).
pub const VIN_L: u8 = 0x24;
/// Catalog id `m5pm1`, PWM Control Register, `PWM0_L` (0x30).
pub const PWM0_L: u8 = 0x30;
/// Catalog id `m5pm1`, PWM Control Register, `PWM0_HC` (0x31).
pub const PWM0_HC: u8 = 0x31;
/// [`PWM0_HC`] bit 4: channel enable.
pub const PWM0_EN: u8 = 1 << 4;
/// Catalog id `m5pm1`, PWM Control Register, frequency low.
pub const PWM_FREQ_L: u8 = 0x34;
/// Modest 12-bit duty (~25%).
pub const FRONTLIGHT_DUTY: u16 = 1024;
/// 12-bit PWM0 full scale.
pub const PWM0_DUTY_MAX: u16 = 4095;
/// Frontlight PWM frequency for M5PM1 G3 into AW9967 (Hz). Catalog id `m5pm1`,
/// section PWM Control (`PWM_FREQ_L` 0x34).
pub const FRONTLIGHT_PWM_HZ: u16 = 5000;
/// Catalog id `m5pm1`, IRQ Register, `IRQ_STATUS1` (0x40).
pub const IRQ_STATUS1: u8 = 0x40;
/// Catalog id `m5pm1`, IRQ Register, `IRQ_STATUS2` (0x41).
pub const IRQ_STATUS2: u8 = 0x41;
/// Catalog id `m5pm1`, IRQ Register, `IRQ_STATUS3` (0x42).
pub const IRQ_STATUS3: u8 = 0x42;
/// Catalog id `m5pm1`, IRQ Register, `IRQ_MASK1` (0x43).
pub const IRQ_MASK1: u8 = 0x43;

/// Combine an M5PM1 ADC L/H pair (sheet unit: mV).
#[must_use]
pub const fn adc_mv(lo: u8, hi: u8) -> u16 {
    u16::from_le_bytes([lo, hi])
}

/// Minimum battery voltage in millivolts corresponding to 0% remaining charge (cutoff).
pub const BATTERY_EMPTY_MV: u16 = 3300;

/// Maximum battery voltage in millivolts corresponding to 100% full charge for 1S LiPo.
pub const BATTERY_FULL_MV: u16 = 4150;

/// Calculates estimated battery state of charge (percentage 0..=100) from terminal voltage in mV.
///
/// Uses standard 1S LiPo linear discharge approximation matching official M5Stack / M5Unified curve:
/// 3300 mV -> 0%, 4150 mV -> 100%.
#[must_use]
pub const fn battery_percent(vbat_mv: u16) -> u8 {
    if vbat_mv <= BATTERY_EMPTY_MV {
        0
    } else if vbat_mv >= BATTERY_FULL_MV {
        100
    } else {
        let span = (BATTERY_FULL_MV - BATTERY_EMPTY_MV) as u32;
        let delta = (vbat_mv - BATTERY_EMPTY_MV) as u32;
        ((delta * 100) / span) as u8
    }
}

/// Pack PWM0 duty into [`PWM0_L`] / [`PWM0_HC`].
#[must_use]
pub const fn pwm0_bytes(duty: u16) -> (u8, u8) {
    let duty = if duty > PWM0_DUTY_MAX {
        PWM0_DUTY_MAX
    } else {
        duty
    };
    let lo = duty as u8;
    let hi = ((duty >> 8) as u8) & 0x0F;
    if duty == 0 {
        (0, 0)
    } else {
        (lo, PWM0_EN | hi)
    }
}

/// Pack a frequency into [`PWM_FREQ_L`] (LE 16-bit).
#[must_use]
pub const fn pwm_freq_bytes(hz: u16) -> (u8, u8) {
    (hz as u8, (hz >> 8) as u8)
}

/// I2C wrapper. Owns the bus (C-CTOR / C-FREE).
pub struct M5pm1<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C> M5pm1<I2C> {
    /// Take the bus. `addr` is the 7-bit slave address.
    #[inline]
    pub const fn new(i2c: I2C, addr: u8) -> Self {
        Self { i2c, addr }
    }

    /// Return the bus (C-FREE).
    #[inline]
    pub fn release(self) -> I2C {
        self.i2c
    }
}

impl<I2C: I2c> M5pm1<I2C> {
    /// Named-register read.
    pub fn read_at(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        self.i2c.write(self.addr, &[reg])?;
        let mut val = [0u8];
        self.i2c.read(self.addr, &mut val)?;
        Ok(val[0])
    }

    /// Two-byte little-endian read (ADC L/H pairs).
    pub fn read_le16(&mut self, reg: u8) -> Result<u16, I2C::Error> {
        let mut buf = [0u8; 2];
        self.i2c.write_read(self.addr, &[reg], &mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    /// Named-register write.
    pub fn write_at(&mut self, reg: u8, val: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.addr, &[reg, val])
    }

    /// Two data bytes after `reg` (PWM duty / freq).
    pub fn write_pair(&mut self, reg: u8, lo: u8, hi: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.addr, &[reg, lo, hi])
    }

    /// Mux G3 to PWM0, push-pull, set 5 kHz.
    ///
    /// `g3` is the numbered GPIO bit in [`GPIO_DRV`] (PaperMono: 3).
    pub fn enable_pwm0(&mut self, g3: u8) -> Result<(), I2C::Error> {
        let func0 = self.read_at(GPIO_FUNC0)?;
        let drv = self.read_at(GPIO_DRV)?;
        let func0 = (func0 & !GPIO3_FUNC_MASK) | GPIO3_FUNC_PWM;
        let drv = drv & !(1 << g3);
        self.write_at(GPIO_FUNC0, func0)?;
        self.write_at(GPIO_DRV, drv)?;
        let (lo, hi) = pwm_freq_bytes(FRONTLIGHT_PWM_HZ);
        self.write_pair(PWM_FREQ_L, lo, hi)
    }

    /// Write PWM0 duty. `0` clears enable (lamp off).
    pub fn set_pwm0_duty(&mut self, duty: u16) -> Result<(), I2C::Error> {
        let (lo, hc) = pwm0_bytes(duty);
        self.write_pair(PWM0_L, lo, hc)
    }

    /// [`SYS_CMD`] shutdown. Explicit on purpose.
    pub fn shutdown(&mut self) -> Result<(), I2C::Error> {
        self.write_at(SYS_CMD, SYS_CMD_SHUTDOWN)
    }

    /// Sets the red status LED ([`PWR_CFG`] bit 4, [`LED_EN`]).
    pub fn set_led(&mut self, on: bool) -> Result<(), I2C::Error> {
        let cfg = self.read_at(PWR_CFG)?;
        let new_cfg = if on { cfg | LED_EN } else { cfg & !LED_EN };
        self.write_at(PWR_CFG, new_cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    #[test]
    fn registers_and_packing() {
        assert_eq!(DEVICE_ID, 0x00);
        assert_eq!(SYS_CMD_SHUTDOWN, 0xA1);
        assert_eq!(adc_mv(0x51, 0x0F), 3921);
        assert_eq!(pwm0_bytes(0), (0, 0));
        assert_eq!(pwm0_bytes(FRONTLIGHT_DUTY), (0x00, PWM0_EN | 0x04));
        assert_eq!(pwm_freq_bytes(FRONTLIGHT_PWM_HZ), (0x88, 0x13));
        const { assert!(PWM0_DUTY_MAX == 4095) };
    }

    #[test]
    fn set_pwm0_duty_writes_pair() {
        let (lo, hc) = pwm0_bytes(FRONTLIGHT_DUTY);
        let i2c = Mock::new(&[Transaction::write(
            DEFAULT_ADDRESS,
            std::vec![PWM0_L, lo, hc],
        )]);
        let mut pm1 = M5pm1::new(i2c, DEFAULT_ADDRESS);
        pm1.set_pwm0_duty(FRONTLIGHT_DUTY).unwrap();
        pm1.release().done();
    }

    #[test]
    fn battery_percent_mapping() {
        assert_eq!(battery_percent(3200), 0);
        assert_eq!(battery_percent(3300), 0);
        assert_eq!(battery_percent(4150), 100);
        assert_eq!(battery_percent(4200), 100);
        assert_eq!(battery_percent(3725), 50);
        assert_eq!(battery_percent(3921), 73);
    }

    #[test]
    fn read_operations() {
        let txns = [
            Transaction::write(DEFAULT_ADDRESS, std::vec![DEVICE_ID]),
            Transaction::read(DEFAULT_ADDRESS, std::vec![0x42]),
            Transaction::write_read(DEFAULT_ADDRESS, std::vec![VBAT_L], std::vec![0x51, 0x0F]),
        ];
        let i2c = Mock::new(&txns);
        let mut pm1 = M5pm1::new(i2c, DEFAULT_ADDRESS);
        assert_eq!(pm1.read_at(DEVICE_ID).unwrap(), 0x42);
        assert_eq!(pm1.read_le16(VBAT_L).unwrap(), 3921);
        pm1.release().done();
    }

    #[test]
    fn set_led_toggles_pwr_cfg_bit_4() {
        let txns = [
            Transaction::write(DEFAULT_ADDRESS, std::vec![PWR_CFG]),
            Transaction::read(DEFAULT_ADDRESS, std::vec![0x17]),
            Transaction::write(DEFAULT_ADDRESS, std::vec![PWR_CFG, 0x07]),
            Transaction::write(DEFAULT_ADDRESS, std::vec![PWR_CFG]),
            Transaction::read(DEFAULT_ADDRESS, std::vec![0x07]),
            Transaction::write(DEFAULT_ADDRESS, std::vec![PWR_CFG, 0x17]),
        ];
        let i2c = Mock::new(&txns);
        let mut pm1 = M5pm1::new(i2c, DEFAULT_ADDRESS);
        pm1.set_led(false).unwrap();
        pm1.set_led(true).unwrap();
        pm1.release().done();
    }
}
