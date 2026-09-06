//! ST25R3916 NFC on PaperMono (`C153`) only.
//!
//! Do not construct this module from a Lite image. Official Lite HTML
//! PinMap omits NFC; leftover pads: hardware skill `nyc-lite-nfc-pads`.
//!
//! Citations:
//! - STMicroelectronics ST25R3916 Datasheet (catalog `st25r3916`, DS12484 Rev 8):
//!   - Section 4.3.4: "I2C interface"
//!   - Section 4.4: "Direct commands"
//!   - Section 4.5.3: "Operation control register"
//!   - Section 4.5.4: "Mode definition register"
//!   - Section 4.5.6: "ISO14443A and NFC 106kb/s settings register"
//!   - Section 4.5.34: "Main interrupt register"
//!   - Section 4.5.38: "FIFO status register 1"
//!   - Section 4.5.42: "Number of transmitted bytes register 1"
//!   - Section 4.5.43: "Number of transmitted bytes register 2"
//!   - Section 4.5.80: "IC identity register"
//! - ISO/IEC 14443-3: "Identification cards — Contactless integrated circuit cards — Proximity cards — Part 3: Initialization and anticollision"
//! - Official factory demo firmware
//!   ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo))

use embedded_hal::i2c::I2c;

/// NFC IRQ (GPIO6). PaperMono (`C153`) HTML PinMap.
pub const IRQ: u8 = 6;

/// 7-bit I2C address on the system I2C bus (`0x50`).
///
/// ST25R3916 Section 4.3.4 "I2C interface" (`50h`).
/// Schematic `I2C_EN=VDD`.
pub const ADDRESS: u8 = 0x50;

/// M5IOE1 `PYG4` (`M5IOE1_PIN_4` / `PYB_NFC_EN`): NFC power enable.
///
/// Must be driven high to power the ST25R3916. On PaperMono-Lite (`C153-Lite`),
/// this line is an unpopulated leftover pin.
pub const IOE1_ENABLE: u8 = 4;

/// ST25R3916 Section 4.5.1 "IO configuration register 1" address (`0x00`).
pub const REG_IO_CONF1: u8 = 0x00;

/// ST25R3916 Section 4.5.2 "IO configuration register 2" address (`0x01`).
pub const REG_IO_CONF2: u8 = 0x01;

/// ST25R3916 Section 4.5.3 "Operation control register" address (`0x02`).
pub const REG_OP_CONTROL: u8 = 0x02;

/// ST25R3916 Section 4.5.4 "Mode definition register" address (`0x03`).
pub const REG_MODE_DEFINITION: u8 = 0x03;

/// ST25R3916 Section 4.5.5 "Bit rate definition register" address (`0x04`).
pub const REG_BIT_RATE: u8 = 0x04;

/// ST25R3916 Section 4.5.6 "ISO14443A and NFC 106kb/s settings register" address (`0x05`).
pub const REG_ISO14443A_SETTINGS: u8 = 0x05;

/// ST25R3916 Section 4.5.11 "Auxiliary definition register" address (`0x0A`).
pub const REG_AUX_DEFINITION: u8 = 0x0A;

/// ST25R3916 Section 4.5.12 "Receiver configuration register 1" address (`0x0B`).
pub const REG_RECEIVER_CONF1: u8 = 0x0B;

/// ST25R3916 Section 4.5.13 "Receiver configuration register 2" address (`0x0C`).
pub const REG_RECEIVER_CONF2: u8 = 0x0C;

/// ST25R3916 Section 4.5.14 "Receiver configuration register 3" address (`0x0D`).
pub const REG_RECEIVER_CONF3: u8 = 0x0D;

/// ST25R3916 Section 4.5.15 "Receiver configuration register 4" address (`0x0E`).
pub const REG_RECEIVER_CONF4: u8 = 0x0E;

/// ST25R3916 Section 4.5.34 "Main interrupt register" address (`0x1A`).
pub const REG_MAIN_IRQ: u8 = 0x1A;

/// ST25R3916 Section 4.5.35 "Timer and NFC interrupt register" address (`0x1B`).
pub const REG_TIMER_NFC_IRQ: u8 = 0x1B;

/// ST25R3916 Section 4.5.36 "Error and wake-up interrupt register" address (`0x1C`).
pub const REG_ERROR_IRQ: u8 = 0x1C;

/// ST25R3916 Section 4.5.38 "FIFO status register 1" address (`0x1E`).
pub const REG_FIFO_STATUS1: u8 = 0x1E;

/// ST25R3916 Section 4.5.39 "FIFO status register 2" address (`0x1F`).
pub const REG_FIFO_STATUS2: u8 = 0x1F;

/// ST25R3916 Section 4.5.40 "Collision display register" address (`0x20`).
pub const REG_COLLISION_DISPLAY: u8 = 0x20;

/// ST25R3916 Section 4.5.42 "Number of transmitted bytes register 1" address (`0x22`).
pub const REG_NUM_TX_BYTES1: u8 = 0x22;

/// ST25R3916 Section 4.5.43 "Number of transmitted bytes register 2" address (`0x23`).
pub const REG_NUM_TX_BYTES2: u8 = 0x23;

/// ST25R3916 Section 4.5.47 "Antenna tuning control register 1" address (`0x26`).
pub const REG_ANTENNA_TUNING1: u8 = 0x26;

/// ST25R3916 Section 4.5.48 "Antenna tuning control register 2" address (`0x27`).
pub const REG_ANTENNA_TUNING2: u8 = 0x27;

/// ST25R3916 Section 4.5.49 "TX driver register" address (`0x28`).
pub const REG_TX_DRIVER: u8 = 0x28;

/// ST25R3916 Section 4.5.51 "External field detector activation threshold register" address (`0x2A`).
pub const REG_EXT_FIELD_DETECTOR_ACT: u8 = 0x2A;

/// ST25R3916 Section 4.5.52 "External field detector deactivation threshold register" address (`0x2B`).
pub const REG_EXT_FIELD_DETECTOR_DEACT: u8 = 0x2B;

/// ST25R3916 Section 4.5.62 "Auxiliary display register" address (`0x31`).
pub const REG_AUX_DISPLAY: u8 = 0x31;

/// ST25R3916 Section 4.5.80 "IC identity register" address (`0x3F`).
pub const REG_IC_IDENTITY: u8 = 0x3F;

/// ST25R3916 Section 4.3.4 Table 11 "SPI operation modes" Register Read mode command prefix (`0x40`).
///
/// Register Read mode bytes use bits `0b01xxxxxx` where `xxxxxx` is the 6-bit register address.
pub const READ_MODE_PREFIX: u8 = 0x40;

/// ST25R3916 Section 4.3.4 Table 11 FIFO load mode byte (`0x80`).
pub const MODE_FIFO_LOAD: u8 = 0x80;

/// ST25R3916 Section 4.3.4 Table 11 FIFO read mode byte (`0x9F`).
pub const MODE_FIFO_READ: u8 = 0x9F;

/// ST25R3916 Section 4.4.1 Table 13 "Set default" direct command (`0xC1`).
pub const CMD_SET_DEFAULT: u8 = 0xC1;

/// ST25R3916 Section 4.4.2 Table 13 "Stop all activities" direct command (`0xC2`).
pub const CMD_STOP_ALL: u8 = 0xC2;

/// ST25R3916 Section 4.4.4 Table 13 "Transmit with CRC" direct command (`0xC4`).
pub const CMD_TRANSMIT_WITH_CRC: u8 = 0xC4;

/// ST25R3916 Section 4.4.4 Table 13 "Transmit without CRC" direct command (`0xC5`).
pub const CMD_TRANSMIT_WITHOUT_CRC: u8 = 0xC5;

/// ST25R3916 Section 4.4.4 Table 13 "Transmit REQA" direct command (`0xC6`).
pub const CMD_TRANSMIT_REQA: u8 = 0xC6;

/// ST25R3916 Section 4.4.4 Table 13 "Transmit WUPA" direct command (`0xC7`).
pub const CMD_TRANSMIT_WUPA: u8 = 0xC7;

/// ST25R3916 Section 4.4.5 Table 13 "NFC initial field ON" direct command (`0xC8`).
pub const CMD_NFC_INITIAL_FIELD_ON: u8 = 0xC8;

/// ST25R3916 Section 4.4.15 Table 13 "Reset RX gain" direct command (`0xD5`).
///
/// Resets the receiver AGC and squelch baseline before each transceive operation.
pub const CMD_RESET_RX_GAIN: u8 = 0xD5;

/// ST25R3916 Section 4.4.16 Table 13 "Adjust regulators" direct command (`0xD6`).
pub const CMD_ADJUST_REGULATORS: u8 = 0xD6;

/// ST25R3916 Section 4.4.3 Table 13 "Clear FIFO" direct command (`0xDB`).
pub const CMD_CLEAR_FIFO: u8 = 0xDB;

/// ST25R3916 Section 4.4.29 Table 13 "Test access" direct command (`0xFC`).
pub const CMD_TEST_ACCESS: u8 = 0xFC;

/// Register Read command byte for the IC identity register (`0x7F`).
///
/// Equivalent to `READ_MODE_PREFIX | REG_IC_IDENTITY`. Matches the
/// `NFC_READ_IC_IDENTITY_CMD` constant in the official factory demo firmware
/// ([M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)).
pub const CMD_READ_IC_IDENTITY: u8 = READ_MODE_PREFIX | REG_IC_IDENTITY;

/// ST25R3916 Section 4.5.80 "IC identity register" 5-bit IC type code for ST25R3916/7 (`0b00101` = `0x05`).
pub const IC_TYPE_ST25R3916: u8 = 0x05;

/// ST25R3916 Section 4.5.2 "IO configuration register 2" bit 7: 3.3 V supply mode (`sup3V`).
pub const IO_CONF2_SUP3V: u8 = 1 << 7;

/// ST25R3916 Section 4.5.2 "IO configuration register 2" bit 5: enable AAT D/A (`aat_en`).
pub const IO_CONF2_AAT_EN: u8 = 1 << 5;

/// ST25R3916 Section 4.5.2 "IO configuration register 2" bit 2: increase IO driving level (`io_drv_lvl`).
pub const IO_CONF2_IO_DRV_LVL: u8 = 1 << 2;

/// Operation control register bit 7: oscillator and regulator enable (`en`).
pub const OP_CONTROL_EN: u8 = 1 << 7;

/// Operation control register bit 6: receiver enable (`rx_en`).
pub const OP_CONTROL_RX_EN: u8 = 1 << 6;

/// Operation control register bit 3: transmitter enable (`tx_en`).
pub const OP_CONTROL_TX_EN: u8 = 1 << 3;

/// Operation control register bit 2: wake-up mode enable (`wu`).
pub const OP_CONTROL_WU: u8 = 1 << 2;

/// ST25R3916 Section 4.5.62 "Auxiliary display register" bit 4: crystal oscillator stable (`osc_ok`).
pub const AUX_DISPLAY_OSC_OK: u8 = 1 << 4;

/// ST25R3916 Section 4.5.62 "Auxiliary display register" bit 5: transmission active (`tx_on`).
pub const AUX_DISPLAY_TX_ON: u8 = 1 << 5;

/// ST25R3916 Section 4.5.62 "Auxiliary display register" bit 3: receiver active (`rx_on`).
pub const AUX_DISPLAY_RX_ON: u8 = 1 << 3;

/// Mode definition register: Initiator ISO14443A mode (`om0 = 1`, `0x08`).
///
/// ST25R3916 Section 4.5.4 Table 23 "Initiator operation modes".
pub const MODE_INITIATOR_ISO14443A: u8 = 0x08;

/// Mode definition register bit 0: automatic NFC-A response mode (`nfc_ar0`).
pub const MODE_NFC_AR8_AUTO: u8 = 1 << 0;

/// ISO14443A and NFC 106kb/s settings register bit 0: anticollision frame (`antcl`).
///
/// ST25R3916 Section 4.5.6 Table 27.
pub const ISO14443A_ANTCL: u8 = 1 << 0;

/// ST25R3916 Section 4.5.11 "Auxiliary definition register" bit 7: receive without CRC (`no_crc_rx`).
pub const AUX_DEF_NO_CRC_RX: u8 = 1 << 7;

/// ST25R3916 Section 4.5.12 "Receiver configuration register 1" bit 3: 600k input impedance (`z_600k`).
pub const RX_CONF1_Z600K: u8 = 1 << 3;

/// ST25R3916 Section 4.5.13 "Receiver configuration register 2": dynamic squelch, AGC enable, AGC full period, AGC ratio 6:3 (`0x2D`).
pub const RX_CONF2_DEFAULT: u8 = 0x2D;

/// ST25R3916 Section 4.5.14 "Receiver configuration register 3": stability-focused receiver gain (`0xD8`).
pub const RX_CONF3_STABILITY: u8 = 0xD8;

/// ST25R3916 Section 4.5.15 "Receiver configuration register 4": stability-focused receiver gain (`0x22`).
pub const RX_CONF4_STABILITY: u8 = 0x22;

/// ST25R3916 Section 4.5.49 "TX driver register": 22% AM modulation index (index 13: `0xD0`).
///
/// Matches default `tx_am_modulation = 13` in the official M5Unit-NFC driver.
pub const TX_DRIVER_DEFAULT: u8 = 0xD0;

/// ISO/IEC 14443-3 Section 6.5.2 Select Command Cascade Level 1 (`0x93`).
pub const ISO14443A_CMD_SEL_CL1: u8 = 0x93;

/// ISO/IEC 14443-3 Section 6.5.2 Select Command Cascade Level 2 (`0x95`).
pub const ISO14443A_CMD_SEL_CL2: u8 = 0x95;

/// ISO/IEC 14443-3 Section 6.5.2 Number of Valid Bits for initial Anticollision (`0x20` = 2 bytes transmitted).
pub const ISO14443A_NVB_ANTICOLLISION: u8 = 0x20;

/// ISO/IEC 14443-3 Section 6.5.2 Number of Valid Bits for complete Select (`0x70` = 7 bytes transmitted).
pub const ISO14443A_NVB_SELECT: u8 = 0x70;

/// ISO/IEC 14443-3 Section 6.5.3 Cascade Tag byte (`0x88`) indicating a multi-byte UID.
pub const ISO14443A_CASCADE_TAG: u8 = 0x88;

/// Generates a single-byte I2C register read command for an ST25R3916 6-bit register address.
///
/// ST25R3916 Section 4.3.4 Figure 22 "Reading a single byte from a register".
#[inline]
#[must_use]
pub const fn register_read_cmd(reg_addr: u8) -> u8 {
    READ_MODE_PREFIX | (reg_addr & 0x3F)
}

/// Decoded hardware identity returned by reading the ST25R3916 IC identity register (`0x3F`).
///
/// ST25R3916 Section 4.5.80 Table 117 "IC identity register".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IcIdentity {
    /// 5-bit IC type code (bits 7:3).
    pub ic_type: u8,
    /// 3-bit silicon revision code (bits 2:0).
    pub ic_rev: u8,
    /// Raw unparsed byte.
    pub raw: u8,
}

impl IcIdentity {
    /// Parses an IC identity register byte.
    #[inline]
    #[must_use]
    pub const fn from_byte(raw: u8) -> Self {
        Self {
            ic_type: (raw >> 3) & 0x1F,
            ic_rev: raw & 0x07,
            raw,
        }
    }

    /// Returns `true` if the IC type code matches the ST25R3916/7 silicon identifier (`0x05`).
    #[inline]
    #[must_use]
    pub const fn is_st25r3916(&self) -> bool {
        self.ic_type == IC_TYPE_ST25R3916
    }
}

/// Detected ISO/IEC 14443-A contactless card or synthetic tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iso14443aCard {
    /// 2-byte Answer To Request Type A (ATQA).
    pub atqa: [u8; 2],
    /// 1-byte Select Acknowledge (SAK).
    pub sak: u8,
    /// Number of valid UID bytes (typically 4 or 7).
    pub uid_len: usize,
    /// Raw Unique Identifier (UID) buffer.
    pub uid: [u8; 10],
}

impl Iso14443aCard {
    /// Returns the active UID bytes as a slice.
    #[inline]
    #[must_use]
    pub fn uid(&self) -> &[u8] {
        &self.uid[..self.uid_len]
    }

    /// Returns `true` if this card has a 4-byte single-size UID.
    #[inline]
    #[must_use]
    pub const fn is_single_size(&self) -> bool {
        self.uid_len == 4
    }

    /// Returns `true` if this card has a 7-byte double-size UID.
    #[inline]
    #[must_use]
    pub const fn is_double_size(&self) -> bool {
        self.uid_len == 7
    }
}

/// Thin ST25R3916 NFC controller driver over an `embedded-hal` 1.0 I2C bus.
pub struct St25r3916<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> St25r3916<I2C> {
    /// Creates a new ST25R3916 driver instance.
    #[inline]
    pub const fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Releases the underlying I2C peripheral.
    #[inline]
    pub fn release(self) -> I2C {
        self.i2c
    }

    /// Reads a single 8-bit register from register space A.
    pub fn read_reg(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        let cmd = register_read_cmd(reg);
        let mut buf = [0u8; 1];
        self.i2c.write_read(self.address, &[cmd], &mut buf)?;
        Ok(buf[0])
    }

    /// Writes a single 8-bit register in register space A.
    pub fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), I2C::Error> {
        let cmd = reg & 0x3F;
        self.i2c.write(self.address, &[cmd, val])
    }

    /// Dispatches a direct command byte (`0xC0..=0xFF`).
    pub fn direct_cmd(&mut self, cmd: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, &[cmd])
    }

    /// Reads data from the internal FIFO buffer via mode byte `0x9F`.
    pub fn read_fifo(&mut self, buf: &mut [u8]) -> Result<(), I2C::Error> {
        self.i2c.write_read(self.address, &[MODE_FIFO_READ], buf)
    }

    /// Loads transmit data into the FIFO buffer via mode byte `0x80`.
    pub fn write_fifo(&mut self, data: &[u8]) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 33];
        let len = data.len().min(32);
        buf[0] = MODE_FIFO_LOAD;
        buf[1..=len].copy_from_slice(&data[..len]);
        self.i2c.write(self.address, &buf[..=len])
    }

    /// Returns the count of bytes currently waiting in the FIFO buffer.
    pub fn fifo_bytes(&mut self) -> Result<usize, I2C::Error> {
        let count = self.read_reg(REG_FIFO_STATUS1)? as usize;
        Ok(count)
    }

    /// Queries the IC identity register (`0x3F`).
    pub fn read_identity(&mut self) -> Result<IcIdentity, I2C::Error> {
        let raw = self.read_reg(REG_IC_IDENTITY)?;
        Ok(IcIdentity::from_byte(raw))
    }

    /// Reads the auxiliary display register (`0x31`).
    pub fn read_aux(&mut self) -> Result<u8, I2C::Error> {
        self.read_reg(REG_AUX_DISPLAY)
    }

    /// Reads the main, timer, and error interrupt status registers (`0x1A`, `0x1B`, `0x1C`).
    ///
    /// Clears the interrupt flags on readout.
    pub fn read_irqs(&mut self) -> Result<(u8, u8, u8), I2C::Error> {
        let main = self.read_reg(REG_MAIN_IRQ)?;
        let timer = self.read_reg(REG_TIMER_NFC_IRQ)?;
        let err = self.read_reg(REG_ERROR_IRQ)?;
        Ok((main, timer, err))
    }

    /// Powers up the oscillator, receiver, and transmitter for ISO14443-A polling.
    ///
    /// Implements Section 4.2.13 "Reader operation" sequencing:
    /// 1. Enter Ready mode by enabling oscillator and regulator (`REG_OP_CONTROL = OP_CONTROL_EN`).
    /// 2. Configure 3.3 V supply mode and enhanced I2C driving level in `REG_IO_CONF2`.
    /// 3. Poll `REG_AUX_DISPLAY` until `osc_ok == 1` confirms crystal clock stability.
    /// 4. Configure Initiator ISO14443-A mode and 106 kbps bit rate.
    /// 5. Enable transmitter and receiver (`tx_en`, `rx_en`).
    /// 6. Reset receiver gain / squelch (`CMD_RESET_RX_GAIN`).
    pub fn enable_field(&mut self) -> Result<(), I2C::Error> {
        // 1. Defensive reset: stop lingering operations and clear control state.
        let _ = self.direct_cmd(CMD_STOP_ALL);
        let _ = self.write_reg(REG_OP_CONTROL, 0x00);
        let _ = self.direct_cmd(CMD_SET_DEFAULT);

        // 2. Hardware errata workaround from ST25R3916 application note / M5Unit-NFC:
        // prevent internal overheat protection from triggering prematurely below junction temperature.
        let _ = self.i2c.write(self.address, &[CMD_TEST_ACCESS, 0x04, 0x10]);

        // 3. IO & Antenna Configuration: disable MCU_CLK output, configure 3.3V supply, enhanced IO drive, and AAT D/A.
        self.write_reg(REG_IO_CONF1, 0x07)?;
        self.write_reg(
            REG_IO_CONF2,
            IO_CONF2_SUP3V | IO_CONF2_IO_DRV_LVL | IO_CONF2_AAT_EN,
        )?;
        self.write_reg(REG_TX_DRIVER, TX_DRIVER_DEFAULT)?;
        self.write_reg(REG_ANTENNA_TUNING1, 0x82)?;
        self.write_reg(REG_ANTENNA_TUNING2, 0x82)?;
        self.write_reg(REG_EXT_FIELD_DETECTOR_ACT, 0x13)?;
        self.write_reg(REG_EXT_FIELD_DETECTOR_DEACT, 0x02)?;

        // 4. Ready mode: enable oscillator, regulator, and automatic external field detector.
        self.write_reg(REG_OP_CONTROL, OP_CONTROL_EN | 0x03)?;

        // 5. Wait for crystal oscillator stability before configuring mode registers
        // (Section 4.5.4 footnote: "Register can be written only in case crystal clock is present and stable (oscok = 1)").
        for _ in 0..100 {
            let aux = self.read_reg(REG_AUX_DISPLAY)?;
            if (aux & AUX_DISPLAY_OSC_OK) != 0 {
                break;
            }
        }

        // 6. Adjust regulators to calibrate PSRR against actual rail voltage.
        self.direct_cmd(CMD_ADJUST_REGULATORS)?;

        // 7. Configure Initiator ISO14443-A mode with automatic response handling and 106 kbps rate.
        self.write_reg(
            REG_MODE_DEFINITION,
            MODE_INITIATOR_ISO14443A | MODE_NFC_AR8_AUTO,
        )?;
        self.write_reg(REG_BIT_RATE, 0x00)?;
        self.write_reg(REG_ISO14443A_SETTINGS, 0x00)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;

        // 8. Configure receiver frontend: 600k input impedance, dynamic squelch, AGC, stability-focused gain.
        self.write_reg(REG_RECEIVER_CONF1, RX_CONF1_Z600K)?;
        self.write_reg(REG_RECEIVER_CONF2, RX_CONF2_DEFAULT)?;
        self.write_reg(REG_RECEIVER_CONF3, RX_CONF3_STABILITY)?;
        self.write_reg(REG_RECEIVER_CONF4, RX_CONF4_STABILITY)?;
        self.direct_cmd(CMD_RESET_RX_GAIN)?;

        // 9. Energize 13.56 MHz RF field with initial collision avoidance, then enable TX and RX.
        self.direct_cmd(CMD_NFC_INITIAL_FIELD_ON)?;
        self.write_reg(
            REG_OP_CONTROL,
            OP_CONTROL_EN | OP_CONTROL_RX_EN | OP_CONTROL_TX_EN | 0x03,
        )
    }

    /// Safely halts transmission and shuts down the RF field and oscillator.
    pub fn disable_field(&mut self) -> Result<(), I2C::Error> {
        let _ = self.direct_cmd(CMD_STOP_ALL);
        self.write_reg(REG_OP_CONTROL, 0x00)
    }

    /// Transmits an ISO14443-A WUPA short frame (`Transmit WUPA`, direct command `0xC7`).
    ///
    /// Wakes up both IDLE and HALT transponders. Sets `antcl` and `no_crc_rx` so the
    /// chip expects a 7-bit transmission and a 16-bit ATQA response without CRC.
    pub fn send_wupa(&mut self) -> Result<(), I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, ISO14443A_ANTCL)?;
        self.write_reg(REG_AUX_DEFINITION, AUX_DEF_NO_CRC_RX)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 0x00)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        self.direct_cmd(CMD_TRANSMIT_WUPA)
    }

    /// Transmits an ISO14443-A REQA short frame (`Transmit REQA`, direct command `0xC6`).
    ///
    /// Wakes up IDLE transponders. Sets `antcl` and `no_crc_rx` so the
    /// chip expects a 7-bit transmission and a 16-bit ATQA response without CRC.
    pub fn send_reqa(&mut self) -> Result<(), I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, ISO14443A_ANTCL)?;
        self.write_reg(REG_AUX_DEFINITION, AUX_DEF_NO_CRC_RX)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 0x00)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        self.direct_cmd(CMD_TRANSMIT_REQA)
    }

    /// Reads the 2-byte ATQA response from the FIFO if available.
    pub fn read_atqa(&mut self) -> Result<Option<[u8; 2]>, I2C::Error> {
        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 2 {
                break;
            }
            count += 1;
        }
        if n < 2 {
            return Ok(None);
        }
        let mut atqa = [0u8; 2];
        self.read_fifo(&mut atqa)?;
        Ok(Some(atqa))
    }

    /// Performs ISO14443-A Cascade Level 1 anticollision (`0x93`, `0x20`).
    ///
    /// Transmits without CRC with collision detection enabled.
    /// Returns 5 bytes: 4 UID bytes plus 1 BCC byte.
    pub fn anticollision_cl1(&mut self) -> Result<Option<[u8; 5]>, I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, ISO14443A_ANTCL)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 2 << 3)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        self.write_fifo(&[ISO14443A_CMD_SEL_CL1, ISO14443A_NVB_ANTICOLLISION])?;
        self.direct_cmd(CMD_TRANSMIT_WITHOUT_CRC)?;

        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 5 {
                break;
            }
            count += 1;
        }
        if n < 5 {
            return Ok(None);
        }
        let mut resp = [0u8; 5];
        self.read_fifo(&mut resp)?;
        let expected_bcc = resp[0] ^ resp[1] ^ resp[2] ^ resp[3];
        if resp[4] != expected_bcc {
            return Ok(None);
        }
        Ok(Some(resp))
    }

    /// Selects Cascade Level 1 with CRC. Returns SAK byte.
    pub fn select_cl1(&mut self, uid_cl1: [u8; 4], bcc: u8) -> Result<Option<u8>, I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, 0x00)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 7 << 3)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        let payload = [
            ISO14443A_CMD_SEL_CL1,
            ISO14443A_NVB_SELECT,
            uid_cl1[0],
            uid_cl1[1],
            uid_cl1[2],
            uid_cl1[3],
            bcc,
        ];
        self.write_fifo(&payload)?;
        self.direct_cmd(CMD_TRANSMIT_WITH_CRC)?;

        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 1 {
                break;
            }
            count += 1;
        }
        if n < 1 {
            return Ok(None);
        }
        let mut sak = [0u8; 1];
        self.read_fifo(&mut sak)?;
        Ok(Some(sak[0]))
    }

    /// Performs ISO14443-A Cascade Level 2 anticollision (`0x95`, `0x20`).
    ///
    /// Transmits without CRC. Returns 5 bytes: 4 UID bytes plus 1 BCC byte.
    pub fn anticollision_cl2(&mut self) -> Result<Option<[u8; 5]>, I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, ISO14443A_ANTCL)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 2 << 3)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        self.write_fifo(&[ISO14443A_CMD_SEL_CL2, ISO14443A_NVB_ANTICOLLISION])?;
        self.direct_cmd(CMD_TRANSMIT_WITHOUT_CRC)?;

        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 5 {
                break;
            }
            count += 1;
        }
        if n < 5 {
            return Ok(None);
        }
        let mut resp = [0u8; 5];
        self.read_fifo(&mut resp)?;
        let expected_bcc = resp[0] ^ resp[1] ^ resp[2] ^ resp[3];
        if resp[4] != expected_bcc {
            return Ok(None);
        }
        Ok(Some(resp))
    }

    /// Selects Cascade Level 2 with CRC. Returns SAK byte.
    pub fn select_cl2(&mut self, uid_cl2: [u8; 4], bcc: u8) -> Result<Option<u8>, I2C::Error> {
        self.write_reg(REG_ISO14443A_SETTINGS, 0x00)?;
        self.write_reg(REG_AUX_DEFINITION, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES1, 0x00)?;
        self.write_reg(REG_NUM_TX_BYTES2, 7 << 3)?;
        let _ = self.read_reg(REG_MAIN_IRQ);
        self.direct_cmd(CMD_CLEAR_FIFO)?;
        let payload = [
            ISO14443A_CMD_SEL_CL2,
            ISO14443A_NVB_SELECT,
            uid_cl2[0],
            uid_cl2[1],
            uid_cl2[2],
            uid_cl2[3],
            bcc,
        ];
        self.write_fifo(&payload)?;
        self.direct_cmd(CMD_TRANSMIT_WITH_CRC)?;

        let mut count = 0;
        let mut n = 0;
        while count < 80 {
            n = self.fifo_bytes()?;
            if n >= 1 {
                break;
            }
            count += 1;
        }
        if n < 1 {
            return Ok(None);
        }
        let mut sak = [0u8; 1];
        self.read_fifo(&mut sak)?;
        Ok(Some(sak[0]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    #[test]
    fn nfc_is_not_a_lite_system_address() {
        assert_ne!(ADDRESS, crate::addresses::FT6336G);
        assert_ne!(ADDRESS, crate::addresses::M5IOE1);
        assert_ne!(ADDRESS, crate::addresses::M5PM1);
        assert_ne!(ADDRESS, crate::addresses::IP2315);
        assert_ne!(ADDRESS, crate::addresses::BMI270);
        assert_ne!(ADDRESS, crate::addresses::RX8130CE);
    }

    #[test]
    fn identity_command_encodes_register_3f() {
        assert_eq!(CMD_READ_IC_IDENTITY, 0x7F);
        assert_eq!(register_read_cmd(REG_IC_IDENTITY), 0x7F);
        assert_eq!(register_read_cmd(REG_OP_CONTROL), 0x42);
        assert_eq!(register_read_cmd(REG_FIFO_STATUS1), 0x5E);
    }

    #[test]
    fn parse_st25r3916_identity() {
        let default_ident = IcIdentity::from_byte(0x2A);
        assert_eq!(default_ident.ic_type, 5);
        assert_eq!(default_ident.ic_rev, 2);
        assert!(default_ident.is_st25r3916());

        let rev3_ident = IcIdentity::from_byte((5 << 3) | 3);
        assert!(rev3_ident.is_st25r3916());

        let foreign_ident = IcIdentity::from_byte(0x00);
        assert!(!foreign_ident.is_st25r3916());
    }

    #[test]
    fn card_uid_and_size_properties() {
        let single = Iso14443aCard {
            atqa: [0x04, 0x00],
            sak: 0x08,
            uid_len: 4,
            uid: [0x08, 0x2C, 0xA1, 0x3F, 0, 0, 0, 0, 0, 0],
        };
        assert!(single.is_single_size());
        assert!(!single.is_double_size());
        assert_eq!(single.uid(), &[0x08, 0x2C, 0xA1, 0x3F]);

        let double = Iso14443aCard {
            atqa: [0x44, 0x00],
            sak: 0x00,
            uid_len: 7,
            uid: [0x04, 0xA2, 0x3B, 0x5C, 0x1D, 0x8E, 0x4A, 0, 0, 0],
        };
        assert!(!double.is_single_size());
        assert!(double.is_double_size());
        assert_eq!(double.uid(), &[0x04, 0xA2, 0x3B, 0x5C, 0x1D, 0x8E, 0x4A]);
    }

    #[test]
    fn st25r3916_read_identity_mock() {
        let txns = [Transaction::write_read(
            ADDRESS,
            std::vec![CMD_READ_IC_IDENTITY],
            std::vec![0x2A],
        )];
        let i2c = Mock::new(&txns);
        let mut st = St25r3916::new(i2c, ADDRESS);
        let id = st.read_identity().unwrap();
        assert_eq!(id.ic_type, 5);
        assert_eq!(id.ic_rev, 2);
        st.release().done();
    }

    #[test]
    fn st25r3916_field_enable_and_disable_mock() {
        let txns = [
            // enable_field:
            Transaction::write(ADDRESS, std::vec![CMD_STOP_ALL]),
            Transaction::write(ADDRESS, std::vec![REG_OP_CONTROL, 0x00]),
            Transaction::write(ADDRESS, std::vec![CMD_SET_DEFAULT]),
            Transaction::write(ADDRESS, std::vec![CMD_TEST_ACCESS, 0x04, 0x10]),
            Transaction::write(ADDRESS, std::vec![REG_IO_CONF1, 0x07]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    REG_IO_CONF2,
                    IO_CONF2_SUP3V | IO_CONF2_IO_DRV_LVL | IO_CONF2_AAT_EN,
                ],
            ),
            Transaction::write(ADDRESS, std::vec![REG_TX_DRIVER, TX_DRIVER_DEFAULT]),
            Transaction::write(ADDRESS, std::vec![REG_ANTENNA_TUNING1, 0x82]),
            Transaction::write(ADDRESS, std::vec![REG_ANTENNA_TUNING2, 0x82]),
            Transaction::write(ADDRESS, std::vec![REG_EXT_FIELD_DETECTOR_ACT, 0x13]),
            Transaction::write(ADDRESS, std::vec![REG_EXT_FIELD_DETECTOR_DEACT, 0x02]),
            Transaction::write(ADDRESS, std::vec![REG_OP_CONTROL, OP_CONTROL_EN | 0x03]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_AUX_DISPLAY)],
                std::vec![AUX_DISPLAY_OSC_OK],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_ADJUST_REGULATORS]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    REG_MODE_DEFINITION,
                    MODE_INITIATOR_ISO14443A | MODE_NFC_AR8_AUTO,
                ],
            ),
            Transaction::write(ADDRESS, std::vec![REG_BIT_RATE, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_ISO14443A_SETTINGS, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_AUX_DEFINITION, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_RECEIVER_CONF1, RX_CONF1_Z600K]),
            Transaction::write(ADDRESS, std::vec![REG_RECEIVER_CONF2, RX_CONF2_DEFAULT]),
            Transaction::write(ADDRESS, std::vec![REG_RECEIVER_CONF3, RX_CONF3_STABILITY]),
            Transaction::write(ADDRESS, std::vec![REG_RECEIVER_CONF4, RX_CONF4_STABILITY]),
            Transaction::write(ADDRESS, std::vec![CMD_RESET_RX_GAIN]),
            Transaction::write(ADDRESS, std::vec![CMD_NFC_INITIAL_FIELD_ON]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    REG_OP_CONTROL,
                    OP_CONTROL_EN | OP_CONTROL_RX_EN | OP_CONTROL_TX_EN | 0x03,
                ],
            ),
            // disable_field:
            Transaction::write(ADDRESS, std::vec![CMD_STOP_ALL]),
            Transaction::write(ADDRESS, std::vec![REG_OP_CONTROL, 0x00]),
        ];
        let i2c = Mock::new(&txns);
        let mut st = St25r3916::new(i2c, ADDRESS);
        st.enable_field().unwrap();
        st.disable_field().unwrap();
        st.release().done();
    }

    #[test]
    fn st25r3916_single_size_poll_sequence_mock() {
        let txns = [
            // send_reqa:
            Transaction::write(ADDRESS, std::vec![REG_ISO14443A_SETTINGS, ISO14443A_ANTCL]),
            Transaction::write(ADDRESS, std::vec![REG_AUX_DEFINITION, AUX_DEF_NO_CRC_RX]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES2, 0x00]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(ADDRESS, std::vec![CMD_TRANSMIT_REQA]),
            // read_atqa:
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![2],
            ),
            Transaction::write_read(ADDRESS, std::vec![MODE_FIFO_READ], std::vec![0x04, 0x00]),
            // anticollision_cl1:
            Transaction::write(ADDRESS, std::vec![REG_ISO14443A_SETTINGS, ISO14443A_ANTCL]),
            Transaction::write(ADDRESS, std::vec![REG_AUX_DEFINITION, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES2, 2 << 3]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    MODE_FIFO_LOAD,
                    ISO14443A_CMD_SEL_CL1,
                    ISO14443A_NVB_ANTICOLLISION,
                ],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_TRANSMIT_WITHOUT_CRC]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![5],
            ),
            Transaction::write_read(
                ADDRESS,
                std::vec![MODE_FIFO_READ],
                std::vec![0x08, 0x2C, 0xA1, 0x3F, 0x08 ^ 0x2C ^ 0xA1 ^ 0x3F],
            ),
            // select_cl1:
            Transaction::write(ADDRESS, std::vec![REG_ISO14443A_SETTINGS, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_AUX_DEFINITION, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDRESS, std::vec![REG_NUM_TX_BYTES2, 7 << 3]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(
                ADDRESS,
                std::vec![
                    MODE_FIFO_LOAD,
                    ISO14443A_CMD_SEL_CL1,
                    ISO14443A_NVB_SELECT,
                    0x08,
                    0x2C,
                    0xA1,
                    0x3F,
                    0x08 ^ 0x2C ^ 0xA1 ^ 0x3F,
                ],
            ),
            Transaction::write(ADDRESS, std::vec![CMD_TRANSMIT_WITH_CRC]),
            Transaction::write_read(
                ADDRESS,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![1],
            ),
            Transaction::write_read(ADDRESS, std::vec![MODE_FIFO_READ], std::vec![0x08]),
        ];
        let i2c = Mock::new(&txns);
        let mut st = St25r3916::new(i2c, ADDRESS);

        st.send_reqa().unwrap();
        let atqa = st.read_atqa().unwrap().expect("atqa");
        assert_eq!(atqa, [0x04, 0x00]);

        let cl1 = st.anticollision_cl1().unwrap().expect("cl1");
        assert_eq!(&cl1[..4], &[0x08, 0x2C, 0xA1, 0x3F]);

        let sak = st
            .select_cl1([cl1[0], cl1[1], cl1[2], cl1[3]], cl1[4])
            .unwrap()
            .expect("sak");
        assert_eq!(sak, 0x08);

        st.release().done();
    }
}
