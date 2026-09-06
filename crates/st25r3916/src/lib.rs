//! ST25R3916 NFC transceiver driver.
//!
//! Provides hardware abstractions, register and command maps, Initiator
//! (Reader) mode, and Target (Card Emulation) profiles for the
//! STMicroelectronics ST25R3916 NFC IC.
//!
//! # Hardware Safety and Validation Notice
//!
//! ISO/IEC 14443-A Initiator mode is hardware-verified on M5Stack PaperMono (`C153`).
//! Target/Card Emulation profiles are verified against datasheet specifications
//! using mock bus transactions and have not yet been validated with physical
//! external NFC readers.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(test)]
extern crate std;

pub mod commands;
pub mod error;
pub mod framing;
pub mod memory;
pub mod registers;
pub mod target;

pub use commands::{
    register_read_cmd, CMD_GOTO_SENSE, CMD_GOTO_SLEEP, CMD_NFC_INITIAL_FIELD_ON,
    CMD_NFC_RESPONSE_FIELD_ON, CMD_READ_IC_IDENTITY, CMD_RESET_RX_GAIN, CMD_SET_DEFAULT,
    CMD_STOP_ALL, CMD_TRANSMIT_REQA, CMD_TRANSMIT_WITH_CRC, CMD_TRANSMIT_WITHOUT_CRC,
    CMD_TRANSMIT_WUPA, MODE_PT_MEM_A_CONFIG, MODE_PT_MEM_F_CONFIG, MODE_PT_MEM_READ,
    MODE_PT_MEM_TSN,
};
pub use error::Error;
pub use framing::{
    build_text_record, build_uri_record, wrap_in_type2_tlv, ApduError, CommandApdu, NdefError,
    Type2Error, Type2Memory, Type4TagApp, UriPrefix, AID_NDEF_V2, FILE_ID_CC, FILE_ID_NDEF,
    SW_SUCCESS,
};
pub use memory::{NfcFParams, PtMemory};
pub use registers::{
    PtaState, IC_TYPE_ST25R3916, REG_AUX_DISPLAY, REG_BIT_RATE, REG_FIFO_STATUS1,
    REG_FIFO_STATUS2, REG_IC_IDENTITY, REG_IO_CONF1, REG_IO_CONF2, REG_ISO14443A_SETTINGS,
    REG_MAIN_IRQ, REG_MODE_DEFINITION, REG_NFCIP1_PASSIVE_TARGET, REG_OP_CONTROL,
    REG_RECEIVER_CONF1, REG_RECEIVER_CONF2, REG_RECEIVER_CONF3, REG_RECEIVER_CONF4,
    REG_TARGET_DISPLAY, REG_TARGET_IRQ,
};
pub use target::{
    NfcATargetConfig, NfcATargetKind, NfcFBitRate, NfcFTargetConfig, Nfcip1CommunicationMode,
    Nfcip1TargetConfig, TargetInterrupts, TargetModulation,
};

use embedded_hal::i2c::I2c;

/// Decoded IC identity for the ST25R3916.
///
/// ST25R3916 Section 4.5.80 "IC identity register":
/// - Bits [7:3]: `vfe_5_0` (IC type; `00101b = 5` for ST25R3916)
/// - Bits [2:0]: Silicon revision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IcIdentity {
    /// IC type code (bits [7:3]). Expected value is `5` for ST25R3916.
    pub ic_type: u8,
    /// Silicon revision (bits [2:0]).
    pub ic_rev: u8,
}

impl IcIdentity {
    /// Decodes an IC identity from a raw register `0x3F` byte.
    #[inline]
    #[must_use]
    pub const fn from_byte(b: u8) -> Self {
        Self {
            ic_type: (b >> 3) & 0x1F,
            ic_rev: b & 0x07,
        }
    }

    /// Whether this identity matches the ST25R3916 (`ic_type == 5`).
    #[inline]
    #[must_use]
    pub const fn is_st25r3916(&self) -> bool {
        self.ic_type == registers::IC_TYPE_ST25R3916
    }
}

/// ST25R3916 NFC transceiver driver.
#[derive(Debug)]
pub struct St25r3916<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> St25r3916<I2C> {
    /// Creates a new ST25R3916 driver instance over the provided I2C bus.
    #[inline]
    pub const fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Destroys the driver instance and returns the underlying I2C bus (C-FREE).
    #[inline]
    pub fn release(self) -> I2C {
        self.i2c
    }

    /// Returns the active 7-bit I2C device address.
    #[inline]
    pub const fn address(&self) -> u8 {
        self.address
    }

    /// Reads a single 8-bit register from the ST25R3916.
    pub fn read_reg(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        let cmd = commands::register_read_cmd(reg);
        let mut buf = [0u8; 1];
        self.i2c.write_read(self.address, &[cmd], &mut buf)?;
        Ok(buf[0])
    }

    /// Writes a single 8-bit register on the ST25R3916.
    pub fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, &[reg & 0x3F, val])
    }

    /// Modifies a register using a bitwise mask.
    pub fn modify_reg(&mut self, reg: u8, mask: u8, val: u8) -> Result<(), I2C::Error> {
        let current = self.read_reg(reg)?;
        let new_val = (current & !mask) | (val & mask);
        self.write_reg(reg, new_val)
    }

    /// Executes a direct command on the ST25R3916.
    pub fn direct_cmd(&mut self, cmd: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, &[cmd])
    }

    /// Reads and validates the IC identity register (`0x3F`).
    pub fn read_identity(&mut self) -> Result<IcIdentity, I2C::Error> {
        let b = self.read_reg(registers::REG_IC_IDENTITY)?;
        Ok(IcIdentity::from_byte(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    const ADDR: u8 = 0x50;

    #[test]
    fn ic_identity_decoding() {
        let id = IcIdentity::from_byte(0x2A);
        assert_eq!(id.ic_type, 5);
        assert_eq!(id.ic_rev, 2);
        assert!(id.is_st25r3916());

        let foreign = IcIdentity::from_byte(0x10);
        assert!(!foreign.is_st25r3916());
    }

    #[test]
    fn read_identity_transaction() {
        let txns = [Transaction::write_read(
            ADDR,
            std::vec![commands::CMD_READ_IC_IDENTITY],
            std::vec![0x2A],
        )];
        let i2c = Mock::new(&txns);
        let mut nfc = St25r3916::new(i2c, ADDR);
        let id = nfc.read_identity().unwrap();
        assert_eq!(id.ic_type, 5);
        assert_eq!(id.ic_rev, 2);
        nfc.release().done();
    }
}
