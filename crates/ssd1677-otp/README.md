# ssd1677-otp

Driver for the Solomon Systech SSD1677 active matrix e-paper display
controller using panel OTP waveforms.

Datasheet:
[Solomon Systech SSD1677 Datasheet Rev 1.0 (M5Stack copy)](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/SSD1677.pdf).

The crate is `#![no_std]` and depends exclusively on `embedded-hal` 1.0,
avoiding any microcontroller-specific dependencies or assumptions about host
architecture.

Panel updates are dispatched through `OtpRefresh` with options for `GrayFull`,
`MonoFull`, and `Partial`. This driver does not implement custom MCU lookup
tables via command 0x32, relying instead on the factory OTP waveforms programmed
directly into the display controller.
