//! ESP32-S3 GPIO numbers assigned on **both** SKUs.
//!
//! Plain `u8` so this crate stays host-testable. Firmware maps each
//! constant to an `esp-hal` pin in one place.
//!
//! Source: living HTML **PinMap** on the PaperMono and PaperMono-Lite
//! product pages, absorbed in the hardware skill pin-map. Direction and
//! polarity are official tables, not measured.
//!
//! GPIO5/6/21/38–41 are **not** here. Those nets are NFC/LoRa on
//! `C153`. On Lite, do not drive them as extra GPIO.

/// M5PM1 `BOOT_OUT`. **Strapping** (WPU). Hardware skill
/// `nyc-gpio0-strap`.
pub const PMIC_BOOT_OUT: u8 = 0;
/// M5PM1 IRQ (`G1_PY_IRQ`).
pub const PMIC_IRQ: u8 = 1;
/// USER_KEY1 (Button A).
pub const KEY1: u8 = 2;
/// USER_KEY2 (Button B). **Strapping** (floating at reset).
pub const KEY2: u8 = 3;
/// FT6336G INT (`G4_TP_INT`).
pub const TOUCH_INT: u8 = 4;
/// M5IOE1 IRQ (`PYB_IRQ`).
pub const IOE1_IRQ: u8 = 7;

/// SDMMC DAT3.
pub const SDMMC_DAT3: u8 = 8;
/// SDMMC DAT2.
pub const SDMMC_DAT2: u8 = 9;
/// SDMMC DAT1.
pub const SDMMC_DAT1: u8 = 10;
/// SDMMC DAT0.
pub const SDMMC_DAT0: u8 = 11;
/// SDMMC CMD.
pub const SDMMC_CMD: u8 = 12;
/// SDMMC CLK.
pub const SDMMC_CLK: u8 = 13;

/// EPD MOSI (SPI2). Not shared with microSD.
pub const EPD_MOSI: u8 = 14;
/// EPD SCLK (SPI2).
pub const EPD_SCLK: u8 = 15;
/// EPD chip select.
pub const EPD_CS: u8 = 16;
/// EPD data/command.
pub const EPD_DC: u8 = 17;
/// EPD BUSY. Sheet: high = busy. Glass polarity:
/// hardware skill `nyc-otp-busy`.
pub const EPD_BUSY: u8 = 18;

/// Native USB D−. PDM is **not** on this pad.
pub const USB_DM: u8 = 19;
/// Native USB D+.
pub const USB_DP: u8 = 20;

/// Buzzer PWM (`BB_PWM`). Mux off JTAG `MTMS`.
pub const BUZZER: u8 = 42;

/// UART0 TX (ESP32-S3 default). Debug on this product is native
/// USB-Serial/JTAG on [`USB_DM`] / [`USB_DP`], not a CH343.
pub const UART0_TX: u8 = 43;
/// UART0 RX (ESP32-S3 default).
pub const UART0_RX: u8 = 44;

/// PDM CLK. **Strapping** (WPD). Not a power latch.
pub const PDM_CLK: u8 = 45;
/// PDM DAT. **Strapping** (WPD). Not a power latch.
pub const PDM_DAT: u8 = 46;

/// System I2C SDA (`G47_SYS_SDA`).
pub const SYS_I2C_SDA: u8 = 47;
/// System I2C SCL (`G48_SYS_SCL`).
pub const SYS_I2C_SCL: u8 = 48;

/// GPIOs this crate assigns. Radio pads are omitted on purpose.
pub const ASSIGNED: &[u8] = &[
    PMIC_BOOT_OUT,
    PMIC_IRQ,
    KEY1,
    KEY2,
    TOUCH_INT,
    IOE1_IRQ,
    SDMMC_DAT3,
    SDMMC_DAT2,
    SDMMC_DAT1,
    SDMMC_DAT0,
    SDMMC_CMD,
    SDMMC_CLK,
    EPD_MOSI,
    EPD_SCLK,
    EPD_CS,
    EPD_DC,
    EPD_BUSY,
    USB_DM,
    USB_DP,
    BUZZER,
    UART0_TX,
    UART0_RX,
    PDM_CLK,
    PDM_DAT,
    SYS_I2C_SDA,
    SYS_I2C_SCL,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigned_pins_are_unique() {
        let mut seen = [false; 49];
        for &pin in ASSIGNED {
            assert!(
                (pin as usize) < seen.len(),
                "GPIO {pin} is outside the S3 range this map uses"
            );
            assert!(!seen[pin as usize], "GPIO {pin} is assigned twice");
            seen[pin as usize] = true;
        }
    }

    #[test]
    fn strapping_pins_are_not_on_spi2_or_sdmmc() {
        for bus in [
            EPD_MOSI, EPD_SCLK, EPD_CS, EPD_DC, SDMMC_DAT3, SDMMC_DAT2, SDMMC_DAT1, SDMMC_DAT0,
            SDMMC_CMD, SDMMC_CLK,
        ] {
            assert_ne!(bus, PMIC_BOOT_OUT);
            assert_ne!(bus, KEY2);
            assert_ne!(bus, PDM_CLK);
            assert_ne!(bus, PDM_DAT);
        }
    }

    #[test]
    fn pdm_is_not_on_the_usb_pads() {
        assert_ne!(PDM_CLK, USB_DM);
        assert_ne!(PDM_DAT, USB_DP);
        assert_eq!(PDM_CLK, 45);
        assert_eq!(PDM_DAT, 46);
    }
}
