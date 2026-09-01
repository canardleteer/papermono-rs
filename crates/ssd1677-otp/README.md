# ssd1677-otp

Solomon Systech **SSD1677** driver for **panel OTP** waveforms
(PaperMono OTP-Demo). `embedded-hal` 1.0 only. No ESP32-S3
dependency. No MCU `0x32` LUT.

Call site is [`OtpRefresh`](https://docs.rs/ssd1677-otp):
`GrayFull` / `MonoFull` / `Partial`. Do not map official HTML
`epd_*` titles onto those `0x22` bytes. Do not path-dep sticky-rs
`ssd1677-gray4` (a different OTP path).

This README is the crates.io landing page. Relative markdown
links here only resolve inside this package.

## Agent notes

Portable agent rules for this crate live in `AGENTS.md` in this
directory.
