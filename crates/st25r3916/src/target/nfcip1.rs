//! NFCIP-1 (ISO/IEC 18092 Peer-to-Peer) Target profile.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet DS12484 Rev 8:
//!   - Section 2.2.16: "Target mode" ("NFCIP-1 active target", "Bit rate detection mode")
//!   - Section 4.5.3: "Operation control register" (`en_fd_c = 11b`)
//!   - Section 4.5.4: "Mode definition register" (NFCIP-1 active target `om = 0111b`, Bit rate detection `om = 1xxx`)
//!   - Section 4.5.50: "Passive target modulation register"
//!   - Section 4.5.51: "External field detector activation threshold register"
//!   - Section 4.5.52: "External field detector deactivation threshold register"

use super::TargetModulation;
use crate::commands::CMD_STOP_ALL;
use crate::registers::{
    MODE_NFC_AR8_AUTO, MODE_TARGET_BITRATE_DETECT, MODE_TARGET_NFCIP1_ACTIVE, OP_CONTROL_EN,
    OP_CONTROL_RX_EN, REG_EXT_FIELD_DETECTOR_ACT, REG_EXT_FIELD_DETECTOR_DEACT,
    REG_MODE_DEFINITION, REG_OP_CONTROL,
};

/// NFCIP-1 target communication mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nfcip1CommunicationMode {
    /// Active communication mode target (`om = 0111b`, `targ = 1`).
    Active,
    /// Bit rate detection mode with NFC-A and FeliCa detection enabled (`om = 1101b`, `targ = 1`).
    BitRateDetectionAuto,
}

impl Nfcip1CommunicationMode {
    /// Returns the corresponding `REG_MODE_DEFINITION` value including auto-response flag.
    #[inline]
    #[must_use]
    pub const fn to_mode_byte(&self) -> u8 {
        match self {
            Self::Active => MODE_TARGET_NFCIP1_ACTIVE | MODE_NFC_AR8_AUTO,
            // Bit rate detection: om3=1 (0x40), om2=1 (FeliCa, 0x20), om0=1 (ISO14443A, 0x08), targ=1 (0x80)
            Self::BitRateDetectionAuto => MODE_TARGET_BITRATE_DETECT | 0x28 | MODE_NFC_AR8_AUTO,
        }
    }
}

/// Configuration profile for ST25R3916 NFCIP-1 target communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nfcip1TargetConfig {
    /// NFCIP-1 operating mode (Active P2P vs Bit Rate Detection).
    pub mode: Nfcip1CommunicationMode,
    /// External field detector activation threshold (`REG_EXT_FIELD_DETECTOR_ACT`).
    pub act_threshold: u8,
    /// External field detector deactivation threshold (`REG_EXT_FIELD_DETECTOR_DEACT`).
    pub deact_threshold: u8,
    /// Antenna modulation impedance settings.
    pub modulation: TargetModulation,
}

impl Default for Nfcip1TargetConfig {
    #[inline]
    fn default() -> Self {
        Self {
            mode: Nfcip1CommunicationMode::Active,
            act_threshold: 0x13,
            deact_threshold: 0x02,
            modulation: TargetModulation::default(),
        }
    }
}

impl Nfcip1TargetConfig {
    /// Creates a new NFCIP-1 Active P2P target profile.
    #[inline]
    #[must_use]
    pub const fn active() -> Self {
        Self {
            mode: Nfcip1CommunicationMode::Active,
            act_threshold: 0x13,
            deact_threshold: 0x02,
            modulation: TargetModulation {
                ptm_res: 0x07,
                pt_res: 0x0F,
            },
        }
    }

    /// Creates a new NFCIP-1 Bit Rate Detection target profile.
    #[inline]
    #[must_use]
    pub const fn bit_rate_detection() -> Self {
        Self {
            mode: Nfcip1CommunicationMode::BitRateDetectionAuto,
            act_threshold: 0x13,
            deact_threshold: 0x02,
            modulation: TargetModulation {
                ptm_res: 0x07,
                pt_res: 0x0F,
            },
        }
    }
}

impl<I2C: embedded_hal::i2c::I2c> crate::St25r3916<I2C> {
    /// Configures the ST25R3916 into NFCIP-1 target mode according to the profile.
    ///
    /// Executes the required datasheet hardware sequencing:
    /// 1. Halts active transceiver operations (`CMD_STOP_ALL`).
    /// 2. Sets passive target modulation driver resistance (`REG_PASSIVE_TARGET_MOD`).
    /// 3. Programs external field detector activation and deactivation thresholds.
    /// 4. Programs Mode definition register for NFCIP-1 target mode.
    /// 5. Enables chip power, receiver, and automatic external field detector (`en_fd_c = 11b`).
    pub fn configure_target_nfcip1(
        &mut self,
        config: &Nfcip1TargetConfig,
    ) -> Result<(), I2C::Error> {
        self.direct_cmd(CMD_STOP_ALL)?;
        self.set_target_modulation(config.modulation)?;

        // Set external field detection thresholds.
        self.write_reg(REG_EXT_FIELD_DETECTOR_ACT, config.act_threshold)?;
        self.write_reg(REG_EXT_FIELD_DETECTOR_DEACT, config.deact_threshold)?;

        // Set NFCIP-1 mode in Mode definition register.
        self.write_reg(REG_MODE_DEFINITION, config.mode.to_mode_byte())?;

        // Enable power, receiver, and automatic external field detector (en_fd_c = 11b, value 0x03).
        self.write_reg(REG_OP_CONTROL, OP_CONTROL_EN | OP_CONTROL_RX_EN | 0x03)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    const ADDR: u8 = 0x50;

    #[test]
    fn configure_target_nfcip1_active_mock() {
        let config = Nfcip1TargetConfig::active();

        let txns = [
            Transaction::write(ADDR, std::vec![CMD_STOP_ALL]),
            Transaction::write(
                ADDR,
                std::vec![crate::registers::REG_PASSIVE_TARGET_MOD, 0x7F],
            ),
            Transaction::write(ADDR, std::vec![REG_EXT_FIELD_DETECTOR_ACT, 0x13]),
            Transaction::write(ADDR, std::vec![REG_EXT_FIELD_DETECTOR_DEACT, 0x02]),
            Transaction::write(ADDR, std::vec![REG_MODE_DEFINITION, config.mode.to_mode_byte()]),
            Transaction::write(ADDR, std::vec![REG_OP_CONTROL, 0xC3]),
        ];

        let i2c = Mock::new(&txns);
        let mut st = crate::St25r3916::new(i2c, ADDR);
        st.configure_target_nfcip1(&config).unwrap();
        st.release().done();
    }
}
