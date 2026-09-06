//! ST25R3916 register addresses and bit definitions.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet DS12484 Rev 8:
//!   - Section 4.5: "Registers"
//!   - Section 4.5.1: "IO configuration register 1"
//!   - Section 4.5.2: "IO configuration register 2"
//!   - Section 4.5.3: "Operation control register"
//!   - Section 4.5.4: "Mode definition register"
//!   - Section 4.5.5: "Bit rate definition register"
//!   - Section 4.5.6: "ISO14443A and NFC 106kb/s settings register"
//!   - Section 4.5.10: "NFCIP-1 target register"
//!   - Section 4.5.11: "Auxiliary definition register"
//!   - Section 4.5.12: "Receiver configuration register 1"
//!   - Section 4.5.13: "Receiver configuration register 2"
//!   - Section 4.5.14: "Receiver configuration register 3"
//!   - Section 4.5.15: "Receiver configuration register 4"
//!   - Section 4.5.34: "Main interrupt register"
//!   - Section 4.5.35: "Timer and NFC interrupt register"
//!   - Section 4.5.36: "Error and wake-up interrupt register"
//!   - Section 4.5.37: "Target interrupt register"
//!   - Section 4.5.38: "FIFO status register 1"
//!   - Section 4.5.39: "FIFO status register 2"
//!   - Section 4.5.40: "Collision display register"
//!   - Section 4.5.41: "Target display register"
//!   - Section 4.5.42: "Number of transmitted bytes register 1"
//!   - Section 4.5.43: "Number of transmitted bytes register 2"
//!   - Section 4.5.47: "Antenna tuning control register 1"
//!   - Section 4.5.48: "Antenna tuning control register 2"
//!   - Section 4.5.49: "TX driver register"
//!   - Section 4.5.50: "Passive target modulation register"
//!   - Section 4.5.51: "External field detector activation threshold register"
//!   - Section 4.5.52: "External field detector deactivation threshold register"
//!   - Section 4.5.62: "Auxiliary display register"
//!   - Section 4.5.80: "IC identity register"

/// Section 4.5.1 "IO configuration register 1" address (`0x00`).
pub const REG_IO_CONF1: u8 = 0x00;

/// Section 4.5.2 "IO configuration register 2" address (`0x01`).
pub const REG_IO_CONF2: u8 = 0x01;

/// Section 4.5.3 "Operation control register" address (`0x02`).
pub const REG_OP_CONTROL: u8 = 0x02;

/// Section 4.5.4 "Mode definition register" address (`0x03`).
pub const REG_MODE_DEFINITION: u8 = 0x03;

/// Section 4.5.5 "Bit rate definition register" address (`0x04`).
pub const REG_BIT_RATE: u8 = 0x04;

/// Section 4.5.6 "ISO14443A and NFC 106kb/s settings register" address (`0x05`).
pub const REG_ISO14443A_SETTINGS: u8 = 0x05;

/// Section 4.5.9 "NFCIP-1 passive target definition register" address (`0x08`).
pub const REG_NFCIP1_PASSIVE_TARGET: u8 = 0x08;

/// Section 4.5.10 "Stream mode definition register" address (`0x09`).
pub const REG_STREAM_MODE: u8 = 0x09;

/// Section 4.5.11 "Auxiliary definition register" address (`0x0A`).
pub const REG_AUX_DEFINITION: u8 = 0x0A;

/// Section 4.5.12 "Receiver configuration register 1" address (`0x0B`).
pub const REG_RECEIVER_CONF1: u8 = 0x0B;

/// Section 4.5.13 "Receiver configuration register 2" address (`0x0C`).
pub const REG_RECEIVER_CONF2: u8 = 0x0C;

/// Section 4.5.14 "Receiver configuration register 3" address (`0x0D`).
pub const REG_RECEIVER_CONF3: u8 = 0x0D;

/// Section 4.5.15 "Receiver configuration register 4" address (`0x0E`).
pub const REG_RECEIVER_CONF4: u8 = 0x0E;

/// Section 4.5.34 "Main interrupt register" address (`0x1A`).
pub const REG_MAIN_IRQ: u8 = 0x1A;

/// Section 4.5.35 "Timer and NFC interrupt register" address (`0x1B`).
pub const REG_TIMER_NFC_IRQ: u8 = 0x1B;

/// Section 4.5.36 "Error and wake-up interrupt register" address (`0x1C`).
pub const REG_ERROR_IRQ: u8 = 0x1C;

/// Section 4.5.37 "Target interrupt register" address (`0x1D`).
pub const REG_TARGET_IRQ: u8 = 0x1D;

/// Section 4.5.38 "FIFO status register 1" address (`0x1E`).
pub const REG_FIFO_STATUS1: u8 = 0x1E;

/// Section 4.5.39 "FIFO status register 2" address (`0x1F`).
pub const REG_FIFO_STATUS2: u8 = 0x1F;

/// Section 4.5.40 "Collision display register" address (`0x20`).
pub const REG_COLLISION_DISPLAY: u8 = 0x20;

/// Section 4.5.41 "Target display register" address (`0x21`).
pub const REG_TARGET_DISPLAY: u8 = 0x21;

/// Section 4.5.42 "Number of transmitted bytes register 1" address (`0x22`).
pub const REG_NUM_TX_BYTES1: u8 = 0x22;

/// Section 4.5.43 "Number of transmitted bytes register 2" address (`0x23`).
pub const REG_NUM_TX_BYTES2: u8 = 0x23;

/// Section 4.5.47 "Antenna tuning control register 1" address (`0x26`).
pub const REG_ANTENNA_TUNING1: u8 = 0x26;

/// Section 4.5.48 "Antenna tuning control register 2" address (`0x27`).
pub const REG_ANTENNA_TUNING2: u8 = 0x27;

/// Section 4.5.49 "TX driver register" address (`0x28`).
pub const REG_TX_DRIVER: u8 = 0x28;

/// Section 4.5.50 "Passive target modulation register" address (`0x29`).
pub const REG_PASSIVE_TARGET_MOD: u8 = 0x29;

/// Section 4.5.51 "External field detector activation threshold register" address (`0x2A`).
pub const REG_EXT_FIELD_DETECTOR_ACT: u8 = 0x2A;

/// Section 4.5.52 "External field detector deactivation threshold register" address (`0x2B`).
pub const REG_EXT_FIELD_DETECTOR_DEACT: u8 = 0x2B;

/// Section 4.5.62 "Auxiliary display register" address (`0x31`).
pub const REG_AUX_DISPLAY: u8 = 0x31;

/// Section 4.5.80 "IC identity register" address (`0x3F`).
pub const REG_IC_IDENTITY: u8 = 0x3F;

// -------------------------------------------------------------------------
// Register Bit Definitions and Flags
// -------------------------------------------------------------------------

/// Section 4.5.2 `IO configuration register 2`: 3.3V supply selection bit (`sup3V`).
pub const IO_CONF2_SUP3V: u8 = 0x80;

/// Section 4.5.2 `IO configuration register 2`: IO drive level bit (`io_drv_lvl`).
pub const IO_CONF2_IO_DRV_LVL: u8 = 0x08;

/// Section 4.5.2 `IO configuration register 2`: AAT D/A converter enable bit (`aat_en`).
pub const IO_CONF2_AAT_EN: u8 = 0x04;

/// Section 4.5.3 `Operation control register`: enable power and oscillator bit (`en`).
pub const OP_CONTROL_EN: u8 = 0x80;

/// Section 4.5.3 `Operation control register`: enable receiver bit (`rx_en`).
pub const OP_CONTROL_RX_EN: u8 = 0x40;

/// Section 4.5.3 `Operation control register`: enable transmitter bit (`tx_en`).
pub const OP_CONTROL_TX_EN: u8 = 0x20;

/// Section 4.5.4 `Mode definition register`: Target mode bit (`targ = 1`).
pub const MODE_TARG: u8 = 0x80;

/// Section 4.5.4 `Mode definition register`: Initiator ISO14443-A mode (`om = 0000b`).
pub const MODE_INITIATOR_ISO14443A: u8 = 0x00;

/// Section 4.5.4 `Mode definition register`: Passive target ISO14443-A mode (`targ = 1`, `om = 0001b`).
pub const MODE_TARGET_ISO14443A: u8 = MODE_TARG | 0x08;

/// Section 4.5.4 `Mode definition register`: Passive target FeliCa mode (`targ = 1`, `om = 0100b`).
pub const MODE_TARGET_FELICA: u8 = MODE_TARG | 0x20;

/// Section 4.5.4 `Mode definition register`: Target NFCIP-1 active communication mode (`targ = 1`, `om = 0111b`).
pub const MODE_TARGET_NFCIP1_ACTIVE: u8 = MODE_TARG | 0x38;

/// Section 4.5.4 `Mode definition register`: Target Bit rate detection mode (`targ = 1`, `om = 1000b`).
pub const MODE_TARGET_BITRATE_DETECT: u8 = MODE_TARG | 0x40;

/// Section 4.5.4 `Mode definition register`: automatic response handling flag (`nfc_ar = 10b`).
pub const MODE_NFC_AR8_AUTO: u8 = 0x02;

/// Section 4.5.34 `Main interrupt register`: TX end interrupt flag (`I_txe`).
pub const MAIN_IRQ_TXE: u8 = 0x80;

/// Section 4.5.34 `Main interrupt register`: RX start interrupt flag (`I_rxs`).
pub const MAIN_IRQ_RXS: u8 = 0x40;

/// Section 4.5.34 `Main interrupt register`: RX end interrupt flag (`I_rxe`).
pub const MAIN_IRQ_RXE: u8 = 0x20;

/// Section 4.5.35 `Timer and NFC interrupt register`: No-response timer timeout flag (`I_nre`).
pub const TIMER_NFC_IRQ_NRE: u8 = 0x08;

/// Section 4.5.37 `Passive target interrupt register`: Active state reached (106 kbps transponder selected, `I_wu_a`).
pub const TARGET_IRQ_WU_A: u8 = 0x01;

/// Section 4.5.37 `Passive target interrupt register`: Active* state reached (212/424 kbps transponder selected, `I_wu_a*`).
pub const TARGET_IRQ_WU_A_STAR: u8 = 0x02;

/// Section 4.5.37 `Passive target interrupt register`: NFC-F Active interrupt (`I_wu_f`).
pub const TARGET_IRQ_WU_F: u8 = 0x08;

/// Section 4.5.37 `Passive target interrupt register`: End of receive / automatic response sent (`I_rxe_pta`).
pub const TARGET_IRQ_RXE_PTA: u8 = 0x10;

/// Section 4.5.37 `Passive target interrupt register`: Active P2P field on event (`I_apon`).
pub const TARGET_IRQ_APON: u8 = 0x20;

/// Section 4.5.37 `Passive target interrupt register`: Slot number water level interrupt (`I_sl_wl`).
pub const TARGET_IRQ_SL_WL: u8 = 0x40;

/// Section 4.5.37 `Passive target interrupt register`: PPON2 field on waiting timer interrupt (`I_ppon2`).
pub const TARGET_IRQ_PPON2: u8 = 0x80;

/// Section 4.5.41 Table 69 "Passive target display register" states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtaState {
    /// Target is in Power Off state (`0000b`).
    PowerOff,
    /// Target is in Sense/Idle state waiting for REQA/WUPA (`0001b`).
    Idle,
    /// Target is in Cascade Level 1 anticollision (`0010b`).
    ReadyL1,
    /// Target is in Cascade Level 2 anticollision (`0011b`).
    ReadyL2,
    /// Target is in Active state (transponder selected, MCU handles incoming commands) (`0101b`).
    Active,
    /// Target is in Halt/Sleep state (`1001b`).
    Halt,
    /// Target is in Ready L1* state (212/424 kbps) (`1010b`).
    ReadyL1Star,
    /// Target is in Ready L2* state (212/424 kbps) (`1011b`).
    ReadyL2Star,
    /// Target is in Active* state (212/424 kbps transponder selected) (`1101b`).
    ActiveStar,
    /// Reserved or unknown state.
    Unknown(u8),
}

impl PtaState {
    /// Decodes a 4-bit state nibble from `REG_TARGET_DISPLAY`.
    #[inline]
    #[must_use]
    pub const fn from_nibble(n: u8) -> Self {
        match n & 0x0F {
            0x00 => Self::PowerOff,
            0x01 => Self::Idle,
            0x02 => Self::ReadyL1,
            0x03 => Self::ReadyL2,
            0x05 => Self::Active,
            0x09 => Self::Halt,
            0x0A => Self::ReadyL1Star,
            0x0B => Self::ReadyL2Star,
            0x0D => Self::ActiveStar,
            other => Self::Unknown(other),
        }
    }
}

/// Section 4.5.50 `Passive target modulation register`: Modulation resistance mask (`pt_res[3:0]`).
pub const PASSIVE_TARGET_MOD_PT_RES_MASK: u8 = 0x0F;

/// Section 4.5.50 `Passive target modulation register`: Modulated state resistance mask (`ptm_res[3:0]`).
pub const PASSIVE_TARGET_MOD_PTM_RES_MASK: u8 = 0xF0;

/// Section 4.5.62 `Auxiliary display register`: oscillator stable flag (`osc_ok`).
pub const AUX_DISPLAY_OSC_OK: u8 = 0x80;

/// Section 4.5.62 `Auxiliary display register`: transmitter active flag (`tx_on`).
pub const AUX_DISPLAY_TX_ON: u8 = 1 << 5;

/// Section 4.5.62 `Auxiliary display register`: receiver active flag (`rx_on`).
pub const AUX_DISPLAY_RX_ON: u8 = 1 << 3;

/// Expected ST25R3916 IC type code in bits [7:3] of `REG_IC_IDENTITY`.
pub const IC_TYPE_ST25R3916: u8 = 0x05;
