//! ST25R3916 direct commands and SPI/I2C operation modes.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet DS12484 Rev 8:
//!   - Section 4.3.4 Table 11: "SPI operation modes" / "I2C interface"
//!   - Section 4.4 Table 13: "List of direct commands"
//!   - Section 4.4.1: "Set default"
//!   - Section 4.4.2: "Stop all activities"
//!   - Section 4.4.4: "Transmit commands"
//!   - Section 4.4.5: "NFC field activation"
//!   - Section 4.4.11: "Go to sense (Idle) and Go to sleep (Halt)"
//!   - Section 4.4.12: "Mask and unmask receive data"
//!   - Section 4.4.15: "Reset RX gain"
//!   - Section 4.4.16: "Adjust regulators"

// -------------------------------------------------------------------------
// Direct Commands (Section 4.4 Table 13)
// -------------------------------------------------------------------------

/// Section 4.4.1 Table 13 "Set default" direct command (`0xC1`).
///
/// Puts the ST25R3916 into power-up state.
pub const CMD_SET_DEFAULT: u8 = 0xC1;

/// Section 4.4.2 Table 13 "Stop all activities" direct command (`0xC2`).
///
/// Stops all activities: transmission, reception, direct command execution, timers.
pub const CMD_STOP_ALL: u8 = 0xC2;

/// Section 4.4.4 Table 13 "Transmit with CRC" direct command (`0xC4`).
///
/// Starts a transmit sequence with automatic CRC generation.
pub const CMD_TRANSMIT_WITH_CRC: u8 = 0xC4;

/// Section 4.4.4 Table 13 "Transmit without CRC" direct command (`0xC5`).
///
/// Starts a transmit sequence without automatic CRC generation.
pub const CMD_TRANSMIT_WITHOUT_CRC: u8 = 0xC5;

/// Section 4.4.4 Table 13 "Transmit REQA" direct command (`0xC6`).
///
/// Transmits 7-bit REQA command (ISO14443A mode only).
pub const CMD_TRANSMIT_REQA: u8 = 0xC6;

/// Section 4.4.4 Table 13 "Transmit WUPA" direct command (`0xC7`).
///
/// Transmits 7-bit WUPA command (ISO14443A mode only).
pub const CMD_TRANSMIT_WUPA: u8 = 0xC7;

/// Section 4.4.5 Table 13 "NFC initial field ON" direct command (`0xC8`).
///
/// Performs Initial RF Collision avoidance and switches on the RF field.
pub const CMD_NFC_INITIAL_FIELD_ON: u8 = 0xC8;

/// Section 4.4.5 Table 13 "NFC response field ON" direct command (`0xC9`).
///
/// Performs Response RF Collision avoidance and switches on the RF field.
pub const CMD_NFC_RESPONSE_FIELD_ON: u8 = 0xC9;

/// Section 4.4.11 Table 13 "Go to sense (Idle)" direct command (`0xCD`).
///
/// Puts the passive target logic into Sense (Idle) state.
pub const CMD_GOTO_SENSE: u8 = 0xCD;

/// Section 4.4.11 Table 13 "Go to sleep (Halt)" direct command (`0xCE`).
///
/// Puts the passive target logic into Sleep (Halt) state.
pub const CMD_GOTO_SLEEP: u8 = 0xCE;

/// Section 4.4.12 Table 13 "Mask receive data" direct command (`0xD0`).
///
/// Stops receivers and RX decoders.
pub const CMD_MASK_RECEIVE_DATA: u8 = 0xD0;

/// Section 4.4.12 Table 13 "Unmask receive data" direct command (`0xD1`).
///
/// Starts receivers and RX decoders.
pub const CMD_UNMASK_RECEIVE_DATA: u8 = 0xD1;

/// Section 4.4 Table 13 "Change AM modulation state" direct command (`0xD2`).
pub const CMD_CHANGE_AM_MODULATION: u8 = 0xD2;

/// Section 4.4 Table 13 "Measure amplitude" direct command (`0xD3`).
pub const CMD_MEASURE_AMPLITUDE: u8 = 0xD3;

/// Section 4.4.15 Table 13 "Reset RX gain" direct command (`0xD5`).
///
/// Resets receiver gain to the value in Receiver configuration register 4.
pub const CMD_RESET_RX_GAIN: u8 = 0xD5;

/// Section 4.4.16 Table 13 "Adjust regulators" direct command (`0xD6`).
///
/// Adjusts supply regulators according to the current supply voltage level.
pub const CMD_ADJUST_REGULATORS: u8 = 0xD6;

/// Section 4.4 Table 13 "Calibrate driver timing" direct command (`0xD8`).
pub const CMD_CALIBRATE_DRIVER_TIMING: u8 = 0xD8;

/// Section 4.4 Table 13 "Measure phase" direct command (`0xD9`).
pub const CMD_MEASURE_PHASE: u8 = 0xD9;

/// Section 4.4 Table 13 "Clear FIFO" direct command (`0xDB`).
///
/// Clears FIFO content and resets FIFO pointers.
pub const CMD_CLEAR_FIFO: u8 = 0xDB;

/// Section 4.4 Table 13 "Clear RSSI" direct command (`0xDC`).
pub const CMD_CLEAR_RSSI: u8 = 0xDC;

/// Section 4.4 Table 13 "Transparent mode" direct command (`0xDF`).
pub const CMD_TRANSPARENT_MODE: u8 = 0xDF;

/// Test access register prefix for factory calibration and errata workarounds (`0xFA`).
pub const CMD_TEST_ACCESS: u8 = 0xFA;

// -------------------------------------------------------------------------
// SPI / I2C Operation Modes (Section 4.3.4 Table 11)
// -------------------------------------------------------------------------

/// Section 4.3.4 Table 11 Register Read mode prefix (`0b01xxxxxx = 0x40`).
pub const READ_MODE_PREFIX: u8 = 0x40;

/// Section 4.3.4 Table 11 FIFO load mode byte (`0x80`).
pub const MODE_FIFO_LOAD: u8 = 0x80;

/// Section 4.3.4 Table 11 FIFO read mode byte (`0x9F`).
pub const MODE_FIFO_READ: u8 = 0x9F;

/// Section 4.3.4 Table 11 PT_memory load A-config (`0xA0`).
///
/// Loads passive target memory locations from index 0 onward (NFC-A parameters).
pub const MODE_PT_MEM_A_CONFIG: u8 = 0xA0;

/// Section 4.3.4 Table 11 PT_memory load F-config (`0xA8`).
///
/// Loads passive target memory locations from index 15 onward (NFC-F parameters).
pub const MODE_PT_MEM_F_CONFIG: u8 = 0xA8;

/// Section 4.3.4 Table 11 PT_memory load TSN data (`0xAC`).
///
/// Loads passive target memory locations from index 36 onward (TSN random numbers).
pub const MODE_PT_MEM_TSN: u8 = 0xAC;

/// Section 4.3.4 Table 11 PT_memory read (`0xBF`).
///
/// Reads passive target memory locations from index 0 onward.
pub const MODE_PT_MEM_READ: u8 = 0xBF;

/// Returns the register read command byte for a 6-bit register address.
#[inline]
#[must_use]
pub const fn register_read_cmd(reg: u8) -> u8 {
    READ_MODE_PREFIX | (reg & 0x3F)
}

/// Returns the IC identity register read command (`0x7F`).
pub const CMD_READ_IC_IDENTITY: u8 = register_read_cmd(crate::registers::REG_IC_IDENTITY);
