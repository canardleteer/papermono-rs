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
pub mod initiator;
pub mod isodep;
pub mod memory;
pub mod registers;
pub mod smartcard;
pub mod target;

pub use commands::*;
pub use error::Error;
pub use framing::{
    build_text_record, build_uri_record, wrap_in_type2_tlv, ApduError, CommandApdu, NdefError,
    Type2Error, Type2Memory, Type4TagApp, UriPrefix, AID_NDEF_V2, FILE_ID_CC, FILE_ID_NDEF,
    SW_SUCCESS,
};
pub use initiator::{
    Iso14443aCard, AUX_DEF_NO_CRC_RX, ISO14443A_ANTCL, ISO14443A_CASCADE_TAG,
    ISO14443A_CMD_SEL_CL1, ISO14443A_CMD_SEL_CL2, ISO14443A_NVB_ANTICOLLISION,
    ISO14443A_NVB_SELECT, RX_CONF1_Z600K, RX_CONF2_DEFAULT, RX_CONF3_STABILITY, RX_CONF4_STABILITY,
    TX_DRIVER_DEFAULT,
};
pub use isodep::{
    build_rats, fsi_to_bytes, Ats, IBlock, IsoDepError, IsoDepErrorKind, IsoDepSession, Pcb,
    RBlock, SBlock, SBlockKind, DEFAULT_FWT_POLL_ITERATIONS, FSDI_256_BYTES, MAX_ISODEP_FRAME,
    RATS_CMD_START,
};
pub use memory::{NfcFParams, PtMemory};
pub use registers::*;
pub use smartcard::{
    manufacturer_name, probe_fido, probe_openpgp, probe_piv, probe_smart_card, FidoCardInfo,
    OpenPgpCardInfo, PivCardInfo, SmartCardApp, YubiKeyProfiles, AID_FIDO, AID_OPENPGP, AID_PIV,
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

    /// Reads data from the internal FIFO buffer via mode byte `0x9F`.
    pub fn read_fifo(&mut self, buf: &mut [u8]) -> Result<(), I2C::Error> {
        self.i2c
            .write_read(self.address, &[commands::MODE_FIFO_READ], buf)
    }

    /// Loads transmit data into the FIFO buffer via mode byte `0x80`.
    pub fn write_fifo(&mut self, data: &[u8]) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 33];
        let len = data.len().min(32);
        buf[0] = commands::MODE_FIFO_LOAD;
        buf[1..=len].copy_from_slice(&data[..len]);
        self.i2c.write(self.address, &buf[..=len])
    }

    /// Returns the count of bytes currently waiting in the FIFO buffer.
    pub fn fifo_bytes(&mut self) -> Result<usize, I2C::Error> {
        let count = self.read_reg(registers::REG_FIFO_STATUS1)? as usize;
        Ok(count)
    }

    /// Reads the auxiliary display register (`0x31`).
    pub fn read_aux(&mut self) -> Result<u8, I2C::Error> {
        self.read_reg(registers::REG_AUX_DISPLAY)
    }

    /// Reads the main, timer, and error interrupt status registers (`0x1A`, `0x1B`, `0x1C`).
    ///
    /// Clears the interrupt flags on readout.
    pub fn read_irqs(&mut self) -> Result<(u8, u8, u8), I2C::Error> {
        let main = self.read_reg(registers::REG_MAIN_IRQ)?;
        let timer = self.read_reg(registers::REG_TIMER_NFC_IRQ)?;
        let err = self.read_reg(registers::REG_ERROR_IRQ)?;
        Ok((main, timer, err))
    }

    /// Loads transmit data into the internal FIFO in consecutive 32-byte chunks.
    ///
    /// ST25R3916 Section 4.3.4 Table 11: "FIFO load operation (`0x80`)".
    pub fn write_fifo_chunked(&mut self, data: &[u8]) -> Result<(), I2C::Error> {
        let mut offset = 0;
        while offset < data.len() {
            let chunk = (data.len() - offset).min(32);
            self.write_fifo(&data[offset..offset + chunk])?;
            offset += chunk;
        }
        Ok(())
    }

    /// Returns the full 10-bit FIFO byte count from status registers 1 & 2 (`0x1E`, `0x1F`).
    ///
    /// ST25R3916 Section 4.5.38 and Section 4.5.39.
    pub fn fifo_bytes_full(&mut self) -> Result<usize, I2C::Error> {
        let low = self.read_reg(registers::REG_FIFO_STATUS1)? as usize;
        let high = (self.read_reg(registers::REG_FIFO_STATUS2)? & 0x07) as usize;
        Ok((high << 8) | low)
    }

    /// Configures the number of bytes to transmit in registers 1 & 2 (`0x22`, `0x23`).
    ///
    /// ST25R3916 Section 4.5.42 and Section 4.5.43.
    pub fn set_num_tx_bytes(&mut self, n_bytes: usize) -> Result<(), I2C::Error> {
        let b1 = (n_bytes >> 5) as u8;
        let b2 = ((n_bytes & 0x1F) << 3) as u8;
        self.write_reg(registers::REG_NUM_TX_BYTES1, b1)?;
        self.write_reg(registers::REG_NUM_TX_BYTES2, b2)
    }

    /// Transmits a frame with hardware-calculated CRC-16 (A-type EDC).
    ///
    /// ST25R3916 Section 4.4.4 Table 13: "Transmit with CRC" direct command (`0xC4`).
    pub fn transmit_frame_crc(&mut self, data: &[u8]) -> Result<(), I2C::Error> {
        self.set_num_tx_bytes(data.len())?;
        let _ = self.read_reg(registers::REG_MAIN_IRQ);
        self.direct_cmd(commands::CMD_CLEAR_FIFO)?;
        self.write_fifo_chunked(data)?;
        self.direct_cmd(commands::CMD_TRANSMIT_WITH_CRC)
    }

    /// Receives a response frame with hardware CRC-16 validation into `buf`.
    ///
    /// The ST25R3916 receiver places the entire RF frame, including the 2-byte CRC-16 (EDC),
    /// into the FIFO. This method checks for receive completion (`MAIN_IRQ_RXE`), reads the FIFO,
    /// clears any overflow, and strips the trailing 2-byte hardware CRC-16, returning the payload length.
    ///
    /// ST25R3916 Section 4.5.34: "Main interrupt register" (`0x1A`), `MAIN_IRQ_RXE`.
    pub fn receive_frame_crc(
        &mut self,
        buf: &mut [u8],
        max_poll_iterations: u32,
    ) -> Result<Option<usize>, I2C::Error> {
        let mut n = 0;
        let mut poll = 0;
        while poll < max_poll_iterations {
            n = self.fifo_bytes_full()?;
            if n > 0 {
                let irq = self.read_reg(registers::REG_MAIN_IRQ)?;
                if (irq & registers::MAIN_IRQ_RXE) != 0 {
                    n = self.fifo_bytes_full()?;
                    break;
                }
            }
            poll += 1;
        }
        if n < 2 {
            if n > 0 {
                self.direct_cmd(commands::CMD_CLEAR_FIFO)?;
            }
            return Ok(None);
        }
        let read_len = n.min(buf.len() + 2);
        let mut raw = [0u8; 260];
        let fetch_len = read_len.min(raw.len());
        self.read_fifo(&mut raw[..fetch_len])?;
        if n > fetch_len {
            self.direct_cmd(commands::CMD_CLEAR_FIFO)?;
        }
        if fetch_len < 2 {
            return Ok(None);
        }
        let payload_len = (fetch_len - 2).min(buf.len());
        buf[..payload_len].copy_from_slice(&raw[..payload_len]);
        Ok(Some(payload_len))
    }

    /// Activates ISO/IEC 14443-4 transmission protocol by issuing RATS and receiving ATS.
    pub fn activate_isodep(&mut self) -> Result<isodep::Ats, isodep::IsoDepError<I2C::Error>> {
        let (ats, _session) = isodep::IsoDepSession::activate(self, None)?;
        Ok(ats)
    }

    /// Probes an activated ISO-DEP smart card for standard applications (FIDO CTAP, PIV, OpenPGP).
    pub fn probe_smart_card(
        &mut self,
        ats: &isodep::Ats,
    ) -> Result<smartcard::SmartCardApp, isodep::IsoDepError<I2C::Error>> {
        smartcard::probe_smart_card(self, ats)
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
