//! NFC-F (FeliCa) Passive Target (Card Emulation) profile.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet DS12484 Rev 8:
//!   - Section 2.2.16: "Target mode"
//!   - Section 2.2.17 Table 8: "NFC-212/424k SENS_RES format"
//!   - Section 4.5.4: "Mode definition register" (FeliCa passive target, `om = 0100b`)
//!   - Section 4.5.5: "Bit rate definition register"
//!   - Section 4.5.50: "Passive target modulation register"

use super::TargetModulation;
use crate::commands::CMD_STOP_ALL;
use crate::memory::{NfcFParams, PtMemory};
use crate::registers::{
    MODE_NFC_AR8_AUTO, MODE_TARGET_FELICA, OP_CONTROL_EN, OP_CONTROL_RX_EN, REG_BIT_RATE,
    REG_MODE_DEFINITION, REG_OP_CONTROL,
};

/// Bit rate selection for NFC-F communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcFBitRate {
    /// 212 kbit/s bit rate (`fc / 64`).
    Kbps212,
    /// 424 kbit/s bit rate (`fc / 32`).
    Kbps424,
}

impl NfcFBitRate {
    /// Returns the register value for `REG_BIT_RATE` (`0x04`) for Tx and Rx.
    #[inline]
    #[must_use]
    pub const fn to_reg_val(&self) -> u8 {
        match self {
            Self::Kbps212 => 0x11, // tx_rate=1, rx_rate=1
            Self::Kbps424 => 0x22, // tx_rate=2, rx_rate=2
        }
    }
}

/// Configuration profile for ST25R3916 NFC-F (FeliCa) passive target emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NfcFTargetConfig {
    /// SENSF_RES parameters (System Code, NFCID2, PAD, MRTI).
    pub params: NfcFParams,
    /// Operating bit rate (212 or 424 kbps).
    pub bit_rate: NfcFBitRate,
    /// Antenna modulation impedance settings.
    pub modulation: TargetModulation,
}

impl NfcFTargetConfig {
    /// Creates a default NFC-F target profile with wildcard System Code (`0xFFFF`).
    #[inline]
    #[must_use]
    pub const fn new(nfcid2: [u8; 8], bit_rate: NfcFBitRate) -> Self {
        Self {
            params: NfcFParams {
                system_code: [0xFF, 0xFF],
                nfcid2,
                pad0: [0x00, 0x00],
                pad1: [0x00, 0x00, 0x00],
                mrti_check: 0x00,
                mrti_update: 0x00,
                pad2: 0x00,
            },
            bit_rate,
            modulation: TargetModulation {
                ptm_res: 0x07,
                pt_res: 0x0F,
            },
        }
    }

    /// Constructs the corresponding 48-byte `PtMemory` representation.
    #[must_use]
    pub const fn to_pt_memory(&self) -> PtMemory {
        PtMemory::new().with_nfc_f(&self.params)
    }
}

impl<I2C: embedded_hal::i2c::I2c> crate::St25r3916<I2C> {
    /// Configures the ST25R3916 into NFC-F (FeliCa) passive target mode according to the profile.
    ///
    /// Executes the required datasheet hardware sequencing:
    /// 1. Halts active transceiver operations (`CMD_STOP_ALL`).
    /// 2. Sets passive target modulation driver resistance (`REG_PASSIVE_TARGET_MOD`).
    /// 3. Loads NFC-F configuration into `PT_Memory` (`MODE_PT_MEM_F_CONFIG`).
    /// 4. Programs bit rate for 212 or 424 kbps (`REG_BIT_RATE`).
    /// 5. Programs Mode definition register for FeliCa target mode with automatic response.
    /// 6. Enables receiver in Operation control register.
    pub fn configure_target_nfc_f(&mut self, config: &NfcFTargetConfig) -> Result<(), I2C::Error> {
        self.direct_cmd(CMD_STOP_ALL)?;
        self.set_target_modulation(config.modulation)?;
        let mem = config.to_pt_memory();
        self.load_pt_memory_f(&mem)?;

        // Set TX and RX bit rates.
        self.write_reg(REG_BIT_RATE, config.bit_rate.to_reg_val())?;

        // Section 4.5.4: om = 0100b (FeliCa target), targ = 1, nfc_ar = 10b (auto response).
        self.write_reg(REG_MODE_DEFINITION, MODE_TARGET_FELICA | MODE_NFC_AR8_AUTO)?;

        // Enable chip power and receiver.
        self.write_reg(REG_OP_CONTROL, OP_CONTROL_EN | OP_CONTROL_RX_EN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::MODE_PT_MEM_F_CONFIG;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    const ADDR: u8 = 0x50;

    #[test]
    fn configure_target_nfc_f_mock() {
        let config = NfcFTargetConfig::new([1, 2, 3, 4, 5, 6, 7, 8], NfcFBitRate::Kbps212);
        let mem = config.to_pt_memory();

        let mut expected_pt = std::vec![MODE_PT_MEM_F_CONFIG];
        expected_pt.extend_from_slice(mem.f_config());

        let txns = [
            Transaction::write(ADDR, std::vec![CMD_STOP_ALL]),
            Transaction::write(
                ADDR,
                std::vec![crate::registers::REG_PASSIVE_TARGET_MOD, 0x7F],
            ),
            Transaction::write(ADDR, expected_pt),
            Transaction::write(ADDR, std::vec![REG_BIT_RATE, 0x11]),
            Transaction::write(
                ADDR,
                std::vec![REG_MODE_DEFINITION, MODE_TARGET_FELICA | 0x02],
            ),
            Transaction::write(ADDR, std::vec![REG_OP_CONTROL, 0xC0]),
        ];

        let i2c = Mock::new(&txns);
        let mut st = crate::St25r3916::new(i2c, ADDR);
        st.configure_target_nfc_f(&config).unwrap();
        st.release().done();
    }
}
