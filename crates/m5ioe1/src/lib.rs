//! M5IOE1 expander registers, GPIO banks, and IP2315 gate typestate.
//!
//! Catalog id `m5ioe1`, UM V 1.4 Table 3 Register Map and section
//! “GPIO control ( IO1-IO14 )”. Outputs default open-drain.
//! `PYG11` is the IP2315 I2C gate on PaperMono; [`M5ioe1::new`]
//! parks it.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(test)]
extern crate std;

use core::marker::PhantomData;

use embedded_hal::i2c::I2c;

/// Catalog UM 7-bit base. PaperMono board `begin` uses `0x4F`.
pub const UM_ADDRESS: u8 = 0x6F;
/// PaperMono / Lite official `begin` address.
pub const BOARD_ADDRESS: u8 = 0x4F;

/// `PYG11`: IP2315 I2C gate. Keep the charger off the bus except
/// the charge transaction.
pub const IP2315_I2C_GATE: u8 = 11;

/// Catalog id `m5ioe1`, Table 3 Register Map, `UID_L` (read-only).
pub const UID_L: u8 = 0x00;
/// Catalog id `m5ioe1`, Table 3 Register Map, `REV` (read-only).
pub const REV: u8 = 0x02;
/// Catalog id `m5ioe1`, Table 3, `GPIO_M_L`. M=1 output, M=0 input.
pub const GPIO_M_L: u8 = 0x03;
/// Catalog id `m5ioe1`, Table 3, `GPIO_M_H`.
pub const GPIO_M_H: u8 = 0x04;
/// Catalog id `m5ioe1`, Table 3, `GPIO_O_L`.
pub const GPIO_O_L: u8 = 0x05;
/// Catalog id `m5ioe1`, Table 3, `GPIO_O_H`.
pub const GPIO_O_H: u8 = 0x06;
/// Catalog id `m5ioe1`, Table 3, `GPIO_I_L` (read-only).
pub const GPIO_I_L: u8 = 0x07;
/// Catalog id `m5ioe1`, Table 3, `GPIO_I_H` (read-only).
pub const GPIO_I_H: u8 = 0x08;
/// Catalog id `m5ioe1`, Table 3, `GPIO_DRV_L`. DRV=0 push-pull.
pub const GPIO_DRV_L: u8 = 0x13;
/// Catalog id `m5ioe1`, Table 3, `GPIO_DRV_H`.
pub const GPIO_DRV_H: u8 = 0x14;

/// Which byte of a 16-bit M5IOE1 GPIO pair a `PYGn` lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bank {
    /// `GPIO_*_L`: IO1–IO8. Bit 0 is IO1.
    Low,
    /// `GPIO_*_H`: IO9–IO14. Bit 0 is IO9.
    High,
}

/// Bank and bit for official `PYGn`.
#[must_use]
pub const fn pin_bit(pyg: u8) -> Option<(Bank, u8)> {
    match pyg {
        1..=8 => Some((Bank::Low, pyg - 1)),
        9..=14 => Some((Bank::High, pyg - 9)),
        _ => None,
    }
}

/// Level of `pyg` in a Low/High input pair.
#[must_use]
pub const fn input_level(low: u8, high: u8, pyg: u8) -> Option<bool> {
    let Some((bank, bit)) = pin_bit(pyg) else {
        return None;
    };
    let byte = match bank {
        Bank::Low => low,
        Bank::High => high,
    };
    Some(byte & (1 << bit) != 0)
}

/// Set or clear `pyg` in a Low/High register pair.
pub fn apply_bit(low: &mut u8, high: &mut u8, pyg: u8, set: bool) {
    let Some((bank, bit)) = pin_bit(pyg) else {
        return;
    };
    let target = match bank {
        Bank::Low => low,
        Bank::High => high,
    };
    if set {
        *target |= 1 << bit;
    } else {
        *target &= !(1 << bit);
    }
}

mod sealed {
    pub trait Sealed {}
}

/// IP2315 I2C-gate states.
pub trait GateState: sealed::Sealed {}

/// Gate low: charger parked off the system bus. [`M5ioe1::new`].
#[derive(Debug)]
pub struct Parked;
/// Gate high: charger mounted for a charge transaction.
#[derive(Debug)]
pub struct Mounted;

impl sealed::Sealed for Parked {}
impl sealed::Sealed for Mounted {}
impl GateState for Parked {}
impl GateState for Mounted {}

/// I2C wrapper. Owns the bus (C-CTOR / C-FREE).
pub struct M5ioe1<I2C, S: GateState> {
    i2c: I2C,
    addr: u8,
    _state: PhantomData<S>,
}

impl<I2C, S: GateState> M5ioe1<I2C, S> {
    /// Return the bus (C-FREE). Gate level is left as-is.
    #[inline]
    pub fn release(self) -> I2C {
        self.i2c
    }
}

impl<I2C: I2c> M5ioe1<I2C, Parked> {
    /// Take the bus and park [`IP2315_I2C_GATE`].
    pub fn new(i2c: I2C, addr: u8) -> Result<Self, I2C::Error> {
        let mut this = Self {
            i2c,
            addr,
            _state: PhantomData,
        };
        this.set_push_pull_output(IP2315_I2C_GATE, false)?;
        Ok(this)
    }

    /// Drive [`IP2315_I2C_GATE`] high for a charge transaction.
    pub fn mount(mut self) -> Result<M5ioe1<I2C, Mounted>, I2C::Error> {
        self.set_push_pull_output(IP2315_I2C_GATE, true)?;
        Ok(M5ioe1 {
            i2c: self.i2c,
            addr: self.addr,
            _state: PhantomData,
        })
    }
}

impl<I2C: I2c> M5ioe1<I2C, Mounted> {
    /// Drive [`IP2315_I2C_GATE`] low again.
    pub fn park(mut self) -> Result<M5ioe1<I2C, Parked>, I2C::Error> {
        self.set_push_pull_output(IP2315_I2C_GATE, false)?;
        Ok(M5ioe1 {
            i2c: self.i2c,
            addr: self.addr,
            _state: PhantomData,
        })
    }
}

impl<I2C: I2c, S: GateState> M5ioe1<I2C, S> {
    fn read_reg(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        self.i2c.write(self.addr, &[reg])?;
        let mut val = [0u8];
        self.i2c.read(self.addr, &mut val)?;
        Ok(val[0])
    }

    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.addr, &[reg, val])
    }

    /// Drive `pyg` as push-pull output (DRV=0, M=1).
    pub fn set_push_pull_output(&mut self, pyg: u8, high: bool) -> Result<(), I2C::Error> {
        let mut m_l = self.read_reg(GPIO_M_L)?;
        let mut m_h = self.read_reg(GPIO_M_H)?;
        let mut o_l = self.read_reg(GPIO_O_L)?;
        let mut o_h = self.read_reg(GPIO_O_H)?;
        let mut d_l = self.read_reg(GPIO_DRV_L)?;
        let mut d_h = self.read_reg(GPIO_DRV_H)?;
        apply_bit(&mut m_l, &mut m_h, pyg, true);
        apply_bit(&mut o_l, &mut o_h, pyg, high);
        apply_bit(&mut d_l, &mut d_h, pyg, false);
        self.write_reg(GPIO_DRV_L, d_l)?;
        self.write_reg(GPIO_DRV_H, d_h)?;
        self.write_reg(GPIO_O_L, o_l)?;
        self.write_reg(GPIO_O_H, o_h)?;
        self.write_reg(GPIO_M_L, m_l)?;
        self.write_reg(GPIO_M_H, m_h)
    }

    /// `M=0` input.
    pub fn set_input(&mut self, pyg: u8) -> Result<(), I2C::Error> {
        let mut m_l = self.read_reg(GPIO_M_L)?;
        let mut m_h = self.read_reg(GPIO_M_H)?;
        apply_bit(&mut m_l, &mut m_h, pyg, false);
        self.write_reg(GPIO_M_L, m_l)?;
        self.write_reg(GPIO_M_H, m_h)
    }

    /// `GPIO_I_*` level. `true` is high.
    pub fn read_input(&mut self, pyg: u8) -> Result<bool, I2C::Error> {
        let low = self.read_reg(GPIO_I_L)?;
        let high = self.read_reg(GPIO_I_H)?;
        Ok(input_level(low, high, pyg).unwrap_or(false))
    }
}

/// One-shot push-pull write without typestate (shared-bus bring-up).
pub fn set_push_pull_output<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
    pyg: u8,
    high: bool,
) -> Result<(), I2C::Error> {
    let mut m_l = read_reg(i2c, addr, GPIO_M_L)?;
    let mut m_h = read_reg(i2c, addr, GPIO_M_H)?;
    let mut o_l = read_reg(i2c, addr, GPIO_O_L)?;
    let mut o_h = read_reg(i2c, addr, GPIO_O_H)?;
    let mut d_l = read_reg(i2c, addr, GPIO_DRV_L)?;
    let mut d_h = read_reg(i2c, addr, GPIO_DRV_H)?;
    apply_bit(&mut m_l, &mut m_h, pyg, true);
    apply_bit(&mut o_l, &mut o_h, pyg, high);
    apply_bit(&mut d_l, &mut d_h, pyg, false);
    write_reg(i2c, addr, GPIO_DRV_L, d_l)?;
    write_reg(i2c, addr, GPIO_DRV_H, d_h)?;
    write_reg(i2c, addr, GPIO_O_L, o_l)?;
    write_reg(i2c, addr, GPIO_O_H, o_h)?;
    write_reg(i2c, addr, GPIO_M_L, m_l)?;
    write_reg(i2c, addr, GPIO_M_H, m_h)
}

/// One-shot `M=0` input.
pub fn set_input<I2C: I2c>(i2c: &mut I2C, addr: u8, pyg: u8) -> Result<(), I2C::Error> {
    let mut m_l = read_reg(i2c, addr, GPIO_M_L)?;
    let mut m_h = read_reg(i2c, addr, GPIO_M_H)?;
    apply_bit(&mut m_l, &mut m_h, pyg, false);
    write_reg(i2c, addr, GPIO_M_L, m_l)?;
    write_reg(i2c, addr, GPIO_M_H, m_h)
}

/// One-shot `GPIO_I_*` level.
pub fn read_input<I2C: I2c>(i2c: &mut I2C, addr: u8, pyg: u8) -> Result<bool, I2C::Error> {
    let low = read_reg(i2c, addr, GPIO_I_L)?;
    let high = read_reg(i2c, addr, GPIO_I_H)?;
    Ok(input_level(low, high, pyg).unwrap_or(false))
}

fn read_reg<I2C: I2c>(i2c: &mut I2C, addr: u8, reg: u8) -> Result<u8, I2C::Error> {
    i2c.write(addr, &[reg])?;
    let mut val = [0u8];
    i2c.read(addr, &mut val)?;
    Ok(val[0])
}

fn write_reg<I2C: I2c>(i2c: &mut I2C, addr: u8, reg: u8, val: u8) -> Result<(), I2C::Error> {
    i2c.write(addr, &[reg, val])
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    #[test]
    fn pyg11_is_high_bank_bit_2() {
        assert_eq!(pin_bit(IP2315_I2C_GATE), Some((Bank::High, 2)));
        assert_eq!(input_level(0x01, 0, 1), Some(true));
        let mut low = 0u8;
        let mut high = 0u8;
        apply_bit(&mut low, &mut high, IP2315_I2C_GATE, true);
        assert_eq!(high, 1 << 2);
    }

    fn read_txn(reg: u8, val: u8) -> [Transaction; 2] {
        [
            Transaction::write(BOARD_ADDRESS, std::vec![reg]),
            Transaction::read(BOARD_ADDRESS, std::vec![val]),
        ]
    }

    #[test]
    fn new_parks_pyg11() {
        let mut txns = std::vec![];
        for &(reg, val) in &[
            (GPIO_M_L, 0u8),
            (GPIO_M_H, 0),
            (GPIO_O_L, 0),
            (GPIO_O_H, 0),
            (GPIO_DRV_L, 0xFF),
            (GPIO_DRV_H, 0xFF),
        ] {
            txns.extend(read_txn(reg, val));
        }
        // After apply_bit: M high bit 2 set, O high bit 2 clear, DRV high bit 2 clear.
        txns.push(Transaction::write(
            BOARD_ADDRESS,
            std::vec![GPIO_DRV_L, 0xFF],
        ));
        txns.push(Transaction::write(
            BOARD_ADDRESS,
            std::vec![GPIO_DRV_H, !(1u8 << 2)],
        ));
        txns.push(Transaction::write(BOARD_ADDRESS, std::vec![GPIO_O_L, 0]));
        txns.push(Transaction::write(BOARD_ADDRESS, std::vec![GPIO_O_H, 0]));
        txns.push(Transaction::write(BOARD_ADDRESS, std::vec![GPIO_M_L, 0]));
        txns.push(Transaction::write(
            BOARD_ADDRESS,
            std::vec![GPIO_M_H, 1 << 2],
        ));
        let i2c = Mock::new(&txns);
        let ioe = M5ioe1::new(i2c, BOARD_ADDRESS).unwrap();
        ioe.release().done();
    }

    #[test]
    fn free_fn_set_push_pull_output() {
        let mut txns = std::vec![];
        for &(reg, val) in &[
            (GPIO_M_L, 0u8),
            (GPIO_M_H, 0),
            (GPIO_O_L, 0),
            (GPIO_O_H, 0),
            (GPIO_DRV_L, 0xFF),
            (GPIO_DRV_H, 0xFF),
        ] {
            txns.extend(read_txn(reg, val));
        }
        txns.push(Transaction::write(
            BOARD_ADDRESS,
            std::vec![GPIO_DRV_L, !(1u8 << 2)],
        ));
        txns.push(Transaction::write(
            BOARD_ADDRESS,
            std::vec![GPIO_DRV_H, 0xFF],
        ));
        txns.push(Transaction::write(
            BOARD_ADDRESS,
            std::vec![GPIO_O_L, 1 << 2],
        ));
        txns.push(Transaction::write(BOARD_ADDRESS, std::vec![GPIO_O_H, 0]));
        txns.push(Transaction::write(
            BOARD_ADDRESS,
            std::vec![GPIO_M_L, 1 << 2],
        ));
        txns.push(Transaction::write(BOARD_ADDRESS, std::vec![GPIO_M_H, 0]));
        let mut i2c = Mock::new(&txns);
        set_push_pull_output(&mut i2c, BOARD_ADDRESS, 3, true).unwrap();
        i2c.done();
    }

    #[test]
    fn input_operations() {
        let mut txns = std::vec![];
        txns.extend(read_txn(GPIO_M_L, 0xFF));
        txns.extend(read_txn(GPIO_M_H, 0xFF));
        txns.push(Transaction::write(
            BOARD_ADDRESS,
            std::vec![GPIO_M_L, !(1u8 << 0)],
        ));
        txns.push(Transaction::write(BOARD_ADDRESS, std::vec![GPIO_M_H, 0xFF]));
        txns.extend(read_txn(GPIO_I_L, 0x01));
        txns.extend(read_txn(GPIO_I_H, 0x00));
        let mut i2c = Mock::new(&txns);
        set_input(&mut i2c, BOARD_ADDRESS, 1).unwrap();
        assert!(read_input(&mut i2c, BOARD_ADDRESS, 1).unwrap());
        i2c.done();
    }
}
