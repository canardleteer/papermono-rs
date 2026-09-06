//! NFC-A Passive Target (Card Emulation) profile.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet DS12484 Rev 8:
//!   - Section 2.2.16: "Target mode"
//!   - Section 4.5.4: "Mode definition register" (ISO14443A passive target, `om = 0001b`)
//!   - Section 4.5.9: "NFCIP-1 passive target definition register" (`fdel`, `d_106_ac_a`)
//!   - Section 4.5.50: "Passive target modulation register"

use super::TargetModulation;
use crate::commands::CMD_STOP_ALL;
use crate::memory::PtMemory;
use crate::registers::{
    MODE_NFC_AR8_AUTO, MODE_TARGET_ISO14443A, OP_CONTROL_EN, OP_CONTROL_RX_EN, REG_MODE_DEFINITION,
    REG_NFCIP1_PASSIVE_TARGET, REG_OP_CONTROL,
};

/// Tag architecture kind for NFC-A card emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcATargetKind {
    /// NFC Type 2 Tag emulation with single-size 4-byte UID.
    Type2Single {
        /// 4-byte NFCID1 UID.
        uid: [u8; 4],
        /// 2-byte SENS_RES (ATQA). Defaults to `[0x44, 0x00]`.
        sens_res: [u8; 2],
        /// Level 1 SAK byte. Defaults to `0x00` (Type 2 Tag).
        sak: u8,
    },
    /// NFC Type 4A Tag emulation with double-size 7-byte UID and ISO-DEP support.
    Type4Double {
        /// 7-byte NFCID1 UID.
        uid: [u8; 7],
        /// 2-byte SENS_RES (ATQA). Defaults to `[0x44, 0x03]`.
        sens_res: [u8; 2],
        /// Level 1 SAK byte with cascade bit set (`0x04`).
        sak1: u8,
        /// Level 2 SAK byte indicating ISO-DEP compliance (`0x20`).
        sak2: u8,
    },
}

impl NfcATargetKind {
    /// Constructs a standard Type 2 Tag profile with the given 4-byte UID.
    #[inline]
    #[must_use]
    pub const fn type2(uid: [u8; 4]) -> Self {
        Self::Type2Single {
            uid,
            sens_res: [0x44, 0x00],
            sak: 0x00,
        }
    }

    /// Constructs a standard Type 4A Tag profile with the given 7-byte UID.
    #[inline]
    #[must_use]
    pub const fn type4a(uid: [u8; 7]) -> Self {
        Self::Type4Double {
            uid,
            sens_res: [0x44, 0x03],
            sak1: 0x04,
            sak2: 0x20, // ISO-DEP compliance
        }
    }
}

/// Configuration profile for ST25R3916 NFC-A passive target emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NfcATargetConfig {
    /// Tag architecture kind (Type 2 4-byte UID vs Type 4A 7-byte UID).
    pub kind: NfcATargetKind,
    /// Antenna modulation impedance settings.
    pub modulation: TargetModulation,
    /// PCD to PICC frame delay time compensation (`fdel[3:0]`). Recommended is 2.
    pub fdel: u8,
}

impl NfcATargetConfig {
    /// Creates a new NFC-A target profile for Type 2 Tag emulation.
    #[inline]
    #[must_use]
    pub const fn type2(uid: [u8; 4]) -> Self {
        Self {
            kind: NfcATargetKind::type2(uid),
            modulation: TargetModulation {
                ptm_res: 0x07,
                pt_res: 0x0F,
            },
            fdel: 0x02,
        }
    }

    /// Creates a new NFC-A target profile for Type 4A Tag emulation.
    #[inline]
    #[must_use]
    pub const fn type4a(uid: [u8; 7]) -> Self {
        Self {
            kind: NfcATargetKind::type4a(uid),
            modulation: TargetModulation {
                ptm_res: 0x07,
                pt_res: 0x0F,
            },
            fdel: 0x02,
        }
    }

    /// Constructs the corresponding 48-byte `PtMemory` representation.
    #[must_use]
    pub const fn to_pt_memory(&self) -> PtMemory {
        match self.kind {
            NfcATargetKind::Type2Single { uid, sens_res, sak } => {
                PtMemory::with_nfc_a_single(uid, sens_res, sak)
            }
            NfcATargetKind::Type4Double {
                uid,
                sens_res,
                sak1,
                sak2,
            } => PtMemory::with_nfc_a_double(uid, sens_res, sak1, sak2),
        }
    }
}

impl<I2C: embedded_hal::i2c::I2c> crate::St25r3916<I2C> {
    /// Configures the ST25R3916 into NFC-A passive target mode according to the profile.
    ///
    /// Executes the required datasheet hardware sequencing:
    /// 1. Halts active transceiver operations (`CMD_STOP_ALL`).
    /// 2. Sets passive target modulation driver resistance (`REG_PASSIVE_TARGET_MOD`).
    /// 3. Loads NFC-A configuration into `PT_Memory` (`MODE_PT_MEM_A_CONFIG`).
    /// 4. Programs FDT compensation and enables autonomous anticollision (`REG_NFCIP1_PASSIVE_TARGET`).
    /// 5. Programs Mode definition register for ISO14443-A target mode with automatic response.
    /// 6. Enables receiver in Operation control register and arms the Sense state machine (`CMD_GOTO_SENSE`).
    pub fn configure_target_nfc_a(&mut self, config: &NfcATargetConfig) -> Result<(), I2C::Error> {
        self.direct_cmd(CMD_STOP_ALL)?;
        self.set_target_modulation(config.modulation)?;
        let mem = config.to_pt_memory();
        self.load_pt_memory_a(&mem)?;

        // Section 4.5.9: fdel in bits [7:4]; bits [3:0]=0 enables AP2P, SENSF_RES, and NFC-A anticollision.
        let fdel_byte = (config.fdel & 0x0F) << 4;
        self.write_reg(REG_NFCIP1_PASSIVE_TARGET, fdel_byte)?;

        // Section 4.5.4: om = 0001b (ISO14443-A target), targ = 1, nfc_ar = 10b (auto response).
        self.write_reg(
            REG_MODE_DEFINITION,
            MODE_TARGET_ISO14443A | MODE_NFC_AR8_AUTO,
        )?;

        // Enable chip power and receiver.
        self.write_reg(REG_OP_CONTROL, OP_CONTROL_EN | OP_CONTROL_RX_EN)?;

        // Transition target logic to Sense (Idle) state waiting for reader REQA/WUPA.
        self.target_goto_sense()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::MODE_PT_MEM_A_CONFIG;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    const ADDR: u8 = 0x50;

    #[test]
    fn configure_target_nfc_a_type2_mock() {
        let config = NfcATargetConfig::type2([0x12, 0x34, 0x56, 0x78]);
        let mem = config.to_pt_memory();

        let mut expected_pt = std::vec![MODE_PT_MEM_A_CONFIG];
        expected_pt.extend_from_slice(mem.a_config());

        let txns = [
            Transaction::write(ADDR, std::vec![CMD_STOP_ALL]),
            Transaction::write(
                ADDR,
                std::vec![crate::registers::REG_PASSIVE_TARGET_MOD, 0x7F],
            ),
            Transaction::write(ADDR, expected_pt),
            Transaction::write(ADDR, std::vec![REG_NFCIP1_PASSIVE_TARGET, 0x20]),
            Transaction::write(
                ADDR,
                std::vec![REG_MODE_DEFINITION, MODE_TARGET_ISO14443A | 0x02],
            ),
            Transaction::write(ADDR, std::vec![REG_OP_CONTROL, 0xC0]),
            Transaction::write(ADDR, std::vec![crate::commands::CMD_GOTO_SENSE]),
        ];

        let i2c = Mock::new(&txns);
        let mut st = crate::St25r3916::new(i2c, ADDR);
        st.configure_target_nfc_a(&config).unwrap();
        st.release().done();
    }
}
