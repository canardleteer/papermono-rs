//! ST25R3916 Target (Card Emulation / Listener) mode abstractions and profiles.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet DS12484 Rev 8:
//!   - Section 2.2.16: "Target mode"
//!   - Section 4.4.11: "Go to sense (Idle) and Go to sleep (Halt)"
//!   - Section 4.5.4: "Mode definition register"
//!   - Section 4.5.37: "Passive target interrupt register"
//!   - Section 4.5.41: "Passive target display register"
//!   - Section 4.5.50: "Passive target modulation register"

pub mod nfc_a;
pub mod nfc_f;
pub mod nfcip1;

pub use nfc_a::{NfcATargetConfig, NfcATargetKind};
pub use nfc_f::{NfcFBitRate, NfcFTargetConfig};
pub use nfcip1::{Nfcip1CommunicationMode, Nfcip1TargetConfig};

use crate::commands::{CMD_GOTO_SENSE, CMD_GOTO_SLEEP};
use crate::registers::{
    PtaState, REG_PASSIVE_TARGET_MOD, REG_TARGET_DISPLAY, REG_TARGET_IRQ, TARGET_IRQ_APON,
    TARGET_IRQ_PPON2, TARGET_IRQ_RXE_PTA, TARGET_IRQ_SL_WL, TARGET_IRQ_WU_A, TARGET_IRQ_WU_A_STAR,
    TARGET_IRQ_WU_F,
};

/// Target load modulation driver resistance settings according to Section 4.5.50 Table 80 & 81.
///
/// Must be programmed prior to entering passive target mode in the Mode definition register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetModulation {
    /// RFO driver resistance during passive load modulation in modulated state (`ptm_res[3:0]`).
    /// Value range 0..15 (Table 81). Default is 7 (32.0 ohms).
    pub ptm_res: u8,
    /// RFO driver resistance during passive load modulation in unmodulated state (`pt_res[3:0]`).
    /// Value range 0..15 (Table 81). Setting 15 selects High Z.
    pub pt_res: u8,
}

impl Default for TargetModulation {
    #[inline]
    fn default() -> Self {
        Self {
            ptm_res: 0x07, // 32.0 ohms normalized
            pt_res: 0x0F,  // High Z
        }
    }
}

impl TargetModulation {
    /// Encodes the modulation resistances into a byte for `REG_PASSIVE_TARGET_MOD` (`0x29`).
    #[inline]
    #[must_use]
    pub const fn to_byte(&self) -> u8 {
        ((self.ptm_res & 0x0F) << 4) | (self.pt_res & 0x0F)
    }

    /// Decodes a byte from `REG_PASSIVE_TARGET_MOD` (`0x29`).
    #[inline]
    #[must_use]
    pub const fn from_byte(b: u8) -> Self {
        Self {
            ptm_res: (b >> 4) & 0x0F,
            pt_res: b & 0x0F,
        }
    }
}

/// Decoded interrupts from `REG_TARGET_IRQ` (`0x1D`) according to Section 4.5.37 Table 65.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TargetInterrupts {
    /// 106 kbps transponder activation complete (`I_wu_a`). Target is now in Active state.
    pub wu_a: bool,
    /// 212/424 kbps transponder activation complete (`I_wu_a*`). Target is now in Active* state.
    pub wu_a_star: bool,
    /// FeliCa transponder automatic response to SENSF_REQ was sent (`I_wu_f`).
    pub wu_f: bool,
    /// End of receive, automatic response sent by hardware; MCU action required (`I_rxe_pta`).
    pub rxe_pta: bool,
    /// Active P2P RF field turned on without collision (`I_apon`).
    pub apon: bool,
    /// Four unused slot numbers (TSN) remain in PT_Memory water level warning (`I_sl_wl`).
    pub sl_wl: bool,
    /// PPON2 field on waiting timer expired (`I_ppon2`).
    pub ppon2: bool,
}

impl TargetInterrupts {
    /// Decodes a raw byte from `REG_TARGET_IRQ` (`0x1D`).
    #[inline]
    #[must_use]
    pub const fn from_byte(b: u8) -> Self {
        Self {
            wu_a: (b & TARGET_IRQ_WU_A) != 0,
            wu_a_star: (b & TARGET_IRQ_WU_A_STAR) != 0,
            wu_f: (b & TARGET_IRQ_WU_F) != 0,
            rxe_pta: (b & TARGET_IRQ_RXE_PTA) != 0,
            apon: (b & TARGET_IRQ_APON) != 0,
            sl_wl: (b & TARGET_IRQ_SL_WL) != 0,
            ppon2: (b & TARGET_IRQ_PPON2) != 0,
        }
    }
}

impl<I2C: embedded_hal::i2c::I2c> crate::St25r3916<I2C> {
    /// Configures the passive target modulation resistance in `REG_PASSIVE_TARGET_MOD` (`0x29`).
    pub fn set_target_modulation(
        &mut self,
        modulation: TargetModulation,
    ) -> Result<(), I2C::Error> {
        self.write_reg(REG_PASSIVE_TARGET_MOD, modulation.to_byte())
    }

    /// Reads and clears the passive target interrupt register (`0x1D`).
    pub fn read_target_interrupts(&mut self) -> Result<TargetInterrupts, I2C::Error> {
        let b = self.read_reg(REG_TARGET_IRQ)?;
        Ok(TargetInterrupts::from_byte(b))
    }

    /// Reads the active state of the passive target logic from `REG_TARGET_DISPLAY` (`0x21`).
    pub fn read_target_state(&mut self) -> Result<PtaState, I2C::Error> {
        let b = self.read_reg(REG_TARGET_DISPLAY)?;
        Ok(PtaState::from_nibble(b))
    }

    /// Direct command to put the passive target state machine into Sense (Idle) state.
    pub fn target_goto_sense(&mut self) -> Result<(), I2C::Error> {
        self.direct_cmd(CMD_GOTO_SENSE)
    }

    /// Direct command to put the passive target state machine into Sleep (Halt) state.
    pub fn target_goto_sleep(&mut self) -> Result<(), I2C::Error> {
        self.direct_cmd(CMD_GOTO_SLEEP)
    }
}
